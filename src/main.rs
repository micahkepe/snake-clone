/*!
A (bad) snake clone written in Bevy

Adapted from: <https://mbuffett.com/posts/bevy-snake-tutorial/>
*/
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};
use rand::random;

/// The color of the snake's head
const SNAKE_HEAD_COLOR: Color = Color::linear_rgb(0.7, 0.7, 0.7);
/// The color of the snake's tail segment tiles.
const SNAKE_SEGMENT_COLOR: Color = Color::linear_rgb(0.3, 0.3, 0.3);
/// The speed of the snake's head (number of tiles per update)
const SNAKE_HEAD_VELOCITY: i32 = 1;

/// The color of the food
const FOOD_COLOR: Color = Color::linear_rgb(1., 47. / 255., 136. / 255.); // #ff2f88
/// The width of the game board (in tiles)
const AREA_WIDTH: u32 = 10;
/// The height of the game board (in tiles)
const AREA_HEIGHT: u32 = 10;

/// Food component.
#[derive(Debug, Component)]
struct Food;

/// 2D position component on the game board grid.
#[derive(Component, Debug, Clone, Copy)]
struct Position {
    x: i32,
    y: i32,
}

/// The direction of the snake's head.
#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    /// Returns the opposite direction.
    fn opposite(self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
        }
    }
}

/// The size of the sprites.
#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    /// Creates a square sprite.
    pub fn square(dim: f32) -> Self {
        Size {
            width: dim,
            height: dim,
        }
    }
}

/// The timer for the food spawning system.
#[derive(Resource)]
struct FoodSpawnTimer(Timer);

/// Spawns food at random locations on the game board.
///
/// TODO: on fixed timer for now, but should be event-based once snake consumes last food item.
fn food_spawner(time: Res<Time>, mut timer: ResMut<FoodSpawnTimer>, mut commands: Commands) {
    if timer.0.tick(time.delta()).just_finished() {
        commands
            .spawn((
                Sprite {
                    color: FOOD_COLOR,
                    ..Default::default()
                },
                Transform {
                    translation: vec3(0.0, 0.0, 0.0),
                    ..Default::default()
                },
            ))
            .insert(Food)
            .insert(Position {
                x: (random::<f32>() * AREA_WIDTH as f32) as i32,
                y: (random::<f32>() * AREA_HEIGHT as f32) as i32,
            })
            .insert(Size::square(0.8));
    }
}

/// The snake's head component.
#[derive(Component)]
struct SnakeHead {
    /// The direction the snake is currently facing.
    direction: Direction,
    /// The last direction the snake was facing, if any.
    last_direction: Option<Direction>,
}

/// Spawns the snake.
fn spawn_snake(mut commands: Commands, mut segments: ResMut<SnakeSegments>) {
    *segments = SnakeSegments(vec![
        commands
            .spawn(Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default()
            })
            .insert(SnakeHead {
                direction: Direction::Up,
                last_direction: None,
            })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .id(),
        spawn_segment(commands, Position { x: 3, y: 2 }),
    ]);
}

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Resource)]
struct SnakeSegments(Vec<Entity>);

/// Spawns a snake tail segment at the position given by [`Position`].
fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn(Sprite {
            color: SNAKE_SEGMENT_COLOR,
            ..Default::default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}

/// The timer for the snake's movement system.
#[derive(Resource)]
struct SnakeMovementTimer(Timer);

/// Handles the snake's movement input.
fn snake_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut heads: Query<&mut SnakeHead>,
) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::ArrowLeft) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::ArrowUp) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
            Direction::Right
        } else {
            head.direction
        };

        if dir != head.direction.opposite() {
            head.last_direction = Some(head.direction);
            head.direction = dir
        }
    }
}

/// Moves the snake's head on a timer.
fn snake_movement(
    time: Res<Time>,
    mut timer: ResMut<SnakeMovementTimer>,
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    if timer.0.tick(time.delta()).just_finished()
        && let Some((head_entity, head)) = heads.iter_mut().next()
    {
        let segment_positions: Vec<Position> = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect();
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        match &head.direction {
            Direction::Left => {
                head_pos.x = (head_pos.x - SNAKE_HEAD_VELOCITY).rem_euclid(AREA_WIDTH as i32);
            }
            Direction::Down => {
                head_pos.y = (head_pos.y - SNAKE_HEAD_VELOCITY).rem_euclid(AREA_HEIGHT as i32);
            }
            Direction::Up => {
                head_pos.y = (head_pos.y + SNAKE_HEAD_VELOCITY).rem_euclid(AREA_HEIGHT as i32);
            }
            Direction::Right => {
                head_pos.x = (head_pos.x + SNAKE_HEAD_VELOCITY).rem_euclid(AREA_WIDTH as i32);
            }
        }
        // Update the positions of the trailing segments of the snake to the position of
        // segment ahead of it
        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1)) // off-by-one
            .for_each(|(pos, segment)| *positions.get_mut(*segment).unwrap() = *pos);
    }
}

/// Sets up the camera.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Scales the sprites to fit the game board.
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

/// Translates the sprites to fit the game board.
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
    } else {
        warn!("Could not get window resolution");
    }
}

/// The main Bevy app.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                name: Some("(Bad) Snake".to_string()),
                title: "(Bad) Snake".to_string(),
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
        .add_systems(
            Update,
            (
                snake_movement,
                food_spawner,
                snake_movement_input.before(snake_movement),
            ),
        )
        .add_systems(PostUpdate, (position_translation, size_scaling))
        .insert_resource(SnakeSegments::default())
        .insert_resource(FoodSpawnTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .insert_resource(SnakeMovementTimer(Timer::from_seconds(
            0.15,
            TimerMode::Repeating,
        )))
        .run();
}
