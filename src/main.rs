use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};

const SNAKE_HEAD_COLOR: Color = Color::linear_rgb(0.7, 0.7, 0.7);
const AREA_WIDTH: u32 = 10;
const AREA_HEIGHT: u32 = 10;

#[derive(Component, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(dim: f32) -> Self {
        Size {
            width: dim,
            height: dim,
        }
    }
}

#[derive(Component)]
struct SnakeHead;

fn spawn_snake(mut commands: Commands) {
    commands
        .spawn((
            Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default()
            },
            Transform {
                scale: Vec3::new(10.0, 10.0, 10.0),
                ..default()
            },
        ))
        .insert(SnakeHead)
        .insert(Position { x: 3, y: 3 })
        .insert(Size::square(0.8));
}

fn snake_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut head_positions: Query<&mut Position, With<SnakeHead>>,
) {
    for mut pos in head_positions.iter_mut() {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            pos.x = (pos.x - 1).max(0);
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            pos.x = (pos.x + 1).min(AREA_WIDTH as i32 - 1)
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            pos.y = (pos.y - 1).max(0);
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            pos.y = (pos.y + 1).min(AREA_HEIGHT as i32 - 1)
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn size_scaling(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Size, &mut Transform)>,
) {
    if let Ok(window) = window_q.single() {
        for (sprite_size, mut transform) in q.iter_mut() {
            transform.scale = Vec3::new(
                sprite_size.width / AREA_WIDTH as f32 * window.resolution.width(),
                sprite_size.height / AREA_HEIGHT as f32 * window.resolution.height(),
                1.0,
            )
        }
    }
}

fn position_translation(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    if let Ok(window) = window_q.single() {
        for (pos, mut transform) in q.iter_mut() {
            transform.translation = Vec3::new(
                convert(pos.x as f32, window.resolution.width(), AREA_WIDTH as f32),
                convert(pos.y as f32, window.resolution.height(), AREA_HEIGHT as f32),
                0.0,
            )
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "A terrible version of Snake".to_string(),
                resizable: false,
                resolution: {
                    let mut res = WindowResolution::default();
                    res.set(500.0, 500.0);
                    res
                },
                ..default()
            }),
            ..Default::default()
        }))
        .insert_resource(ClearColor(Color::linear_rgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, (setup_camera, spawn_snake))
        .add_systems(Update, snake_movement)
        .add_systems(PostUpdate, (position_translation, size_scaling))
        .run();
}
