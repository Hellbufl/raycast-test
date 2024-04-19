use std::f32::consts::{PI, TAU};
use bevy::prelude::*;

pub const RAYCAST_DEPTH: u32 = 100;
pub const FOV: f32 = PI / 2.;
pub const DEBUG_MAP_MODE: bool = false;
pub const PLAYER_SPEED: f32 = 3.;
pub const PLAYER_TURNING_SPEED: f32 = PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (draw_scene, update_player))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((Player{}, Transform{position: Vec2::new(0.0, 0.0), rotation: 0.0}));
    commands.spawn(Environment{ walls: vec![(-3,-3), (-2,-3), (-1,-3), (-1, -4), (0, -4), (1, -4), (2, -4), (2, -3), (2, -2), (3, -2), (3, -1), (3, 0), (3, 1), (3, 2), (2, 2), (1, 2), (0, 2), (-1, 2), (-2, 2), (-3, 2), (-3, 1), (-3, 0), (-3, -1), (-3, -2)] });
}

#[derive(Component)]
struct Player {}

#[derive(Component)]
struct Transform {
    position: Vec2,
    rotation: f32,
}

#[derive(Component)]
struct Environment { walls: Vec<(i32, i32)>}

fn update_player(
    mut player_query: Query<&mut Transform, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut transform = player_query.get_single_mut().expect("p");

    let mut direction = Vec2::ZERO;
    
    if keyboard.pressed(KeyCode::ArrowLeft)  {
        transform.rotation += PLAYER_TURNING_SPEED * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::ArrowRight)  {
        transform.rotation -= PLAYER_TURNING_SPEED * time.delta_seconds();
    }

    transform.rotation = transform.rotation % TAU;

    if keyboard.pressed(KeyCode::KeyW)  {
        direction += Vec2::new(1.0, 0.0);
    }
    if keyboard.pressed(KeyCode::KeyA)  {
        direction += Vec2::new(0.0, 1.0);
    }
    if keyboard.pressed(KeyCode::KeyS)  {
        direction += Vec2::new(-1.0, 0.0);
    }
    if keyboard.pressed(KeyCode::KeyD)  {
        direction += Vec2::new(0.0, -1.0);
    }

    if direction.length() > 0.0 {
        direction = direction.normalize();
        let player_direction = Vec2::from_angle(transform.rotation);
        transform.position += direction.rotate(player_direction) * PLAYER_SPEED * time.delta_seconds();
    }
}

fn raycast(
    walls: &Vec<(i32, i32)>,
    start_pos: Vec2,
    direction: Vec2,
) -> Option<f32> {
    let mut current_cell = (start_pos.x.floor() as i32, start_pos.y.floor() as i32);

    if direction.x == 0.0 || direction.y == 0.0 {
        let mut ray_length = start_pos.fract().dot(direction);

        if ray_length < 0.0 { ray_length = ray_length.abs() }
        else                { ray_length = 1.0 - ray_length }

        for _ in 1..RAYCAST_DEPTH {
            current_cell.0 += direction.x as i32;
            current_cell.1 += direction.y as i32;

            if walls.contains(&current_cell) { return Some(ray_length) }

            ray_length += 1.0;
        }
        return None;
    }

    let x_intercept = |x: i32| -> f32 {
        let mut a = x as f32 - start_pos.fract().x;
        if direction.x < 0.0 { a += 1.0 }
        a / direction.x
    };

    let y_intercept = |y: i32| -> f32 {
        let mut a = y as f32 - start_pos.fract().y;
        if direction.y < 0.0 { a += 1.0 }
        a / direction.y
    };

    let step_direction = (direction.signum().x as i32, direction.signum().y as i32);
    let mut distance: f32;

    for _ in 1..RAYCAST_DEPTH {
        let steps_taken = (current_cell.0 - start_pos.x.floor() as i32, current_cell.1 - start_pos.y.floor() as i32);

        let x_intercept_distance = x_intercept(steps_taken.0 + step_direction.0);
        let y_intercept_distance = y_intercept(steps_taken.1 + step_direction.1);

        if x_intercept_distance < y_intercept_distance {
            current_cell.0 += step_direction.0;
            distance = x_intercept_distance;
        }
        else {
            current_cell.1 += step_direction.1;
            distance = y_intercept_distance;
        }

        if walls.contains(&current_cell) {
            return Some(distance);
        }
    }

    None
}

fn draw_scene(
    window_query: Query<&Window>,
    player_query: Query<&Transform, With<Player>>,
    walls_query: Query<&Environment>,
    mut gizmos: Gizmos,
) {
    let scale = 100.;

    let window = window_query.get_single().unwrap();
    let player = player_query.get_single().unwrap();
    let environment = walls_query.get_single().unwrap();

    let resolution = &window.resolution;

    if DEBUG_MAP_MODE {
        gizmos.arrow_2d(Vec2::ZERO, Vec2::X * scale, Color::GRAY);
        gizmos.arrow_2d(Vec2::ZERO, Vec2::Y * scale, Color::GRAY);

        gizmos.linestrip_gradient_2d([
            (player.position * scale + Vec2::from_angle(player.rotation) * scale / 3., Color::BLUE),
            (player.position * scale - Vec2::from_angle(player.rotation) * scale / 6. + Vec2::from_angle(player.rotation + PI / 2.) * scale / 6., Color::RED),
            (player.position * scale - Vec2::from_angle(player.rotation) * scale / 6. - Vec2::from_angle(player.rotation + PI / 2.) * scale / 6., Color::GREEN),
            (player.position * scale + Vec2::from_angle(player.rotation) * scale / 3., Color::BLUE),
        ]);

        for w in &environment.walls {
            gizmos.rect_2d(
                (Vec2::new(w.0 as f32, w.1 as f32) + Vec2::splat(0.5)) * scale,
                0.,
                Vec2::splat(scale),
                Color::WHITE,
            );    
        }
    }

    for column in 0..resolution.width() as i32 {
        let focal = resolution.width() / (2. * (FOV / 2.).tan());
        let angle = ((column as f32 - resolution.width() / 2.) / focal).atan();
        let ray_direction = Vec2::from_angle(angle).rotate(Vec2::from_angle(player.rotation));

        let wall_distance = raycast(&environment.walls, player.position, ray_direction);

        if DEBUG_MAP_MODE {
            if wall_distance.is_some() {
                gizmos.line_2d(player.position * scale, (player.position + ray_direction * wall_distance.unwrap()) * scale, Color::GREEN);
            }
            else {
                gizmos.line_2d(player.position * scale, (player.position + ray_direction * 100.) * scale, Color::RED);
            }
        }
        else if wall_distance.is_some() {
            let percieved_wall_size = resolution.height() / (wall_distance.unwrap() * angle.cos());

            let wall_color = Color::hsl(0., 0., 3. / wall_distance.unwrap());
            // let floor_color_far = Color::hsl(0., 0., 1.0 - (resolution.height() - percieved_wall_size) / resolution.height());
            // let floor_color_near = Color::hsl(0., 0., 1.0);

            gizmos.line_2d(Vec2::new(resolution.width() / 2. - column as f32, -percieved_wall_size / 2.), 
                            Vec2::new(resolution.width() / 2. - column as f32, percieved_wall_size / 2.), wall_color);

            // gizmos.linestrip_gradient_2d([
            //     (Vec2::new(resolution.width() / 2. - column as f32, -percieved_wall_size / 2.), floor_color_far),
            //     (Vec2::new(resolution.width() / 2. - column as f32, -resolution.height() / 2.), floor_color_near),
            // ]);
        
        }
    }
}