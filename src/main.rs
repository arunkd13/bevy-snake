use std::time::Duration;

use bevy::log::LogSettings;
use bevy::prelude::*;
use rand::Rng;

const ARENA_WIDTH: u32 = 15;
const ARENA_HEIGHT: u32 = 15;
const FOOD_SPAWN_PERIOD_MS: u64 = 1500;
const SNAKE_MOVEMENT_PERIOD_MS: u64 = 250;

const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
enum AppState {
    Starting,
    Running,
    Paused,
    Ended,
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(self: Self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
    input_direction: Option<Direction>,
}

#[derive(Component)]
struct SnakeSegment;

#[derive(Component)]
struct SnakeBody(Vec<Entity>);

#[derive(Component)]
struct LastTailPos(Position);

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
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
    fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct Food;

struct FoodSpawnerConfig {
    timer: Timer,
}

struct SnakeMovementConfig {
    timer: Timer,
}

fn setup_game(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width(),
            sprite_size.height / ARENA_HEIGHT as f32 * window.height(),
            1.0,
        );
    }
}

fn position_translation(
    windows: Res<Windows>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.0)
            + (tile_size / 2.0)
    }

    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn snake_head_movement_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut q: Query<&mut SnakeHead>,
) {
    for mut head in q.iter_mut() {
        let dir: Option<Direction> = if keyboard_input.pressed(KeyCode::Left) {
            Some(Direction::Left)
        } else if keyboard_input.pressed(KeyCode::Right) {
            Some(Direction::Right)
        } else if keyboard_input.pressed(KeyCode::Up) {
            Some(Direction::Up)
        } else if keyboard_input.pressed(KeyCode::Down) {
            Some(Direction::Down)
        } else {
            None
        };

        if dir.is_some() {
            if dir.unwrap() != head.direction.opposite() {
                head.input_direction = dir;
            }
        }
    }
}

fn game_control_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut app_state: ResMut<State<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        match app_state.current() {
            AppState::Running => app_state.set(AppState::Paused).unwrap(),
            AppState::Paused => app_state.set(AppState::Running).unwrap(),
            AppState::Ended => app_state.set(AppState::Starting).unwrap(),
            AppState::Starting => (),
        }
    }
}

fn get_next_head_pos(
    head_pos: Position,
    direction: Direction,
    segments: &Query<&mut Position, (With<SnakeSegment>, Without<SnakeHead>)>,
) -> Option<Position> {
    let mut next_head_pos = head_pos;
    match direction {
        Direction::Left => {
            next_head_pos.x -= 1;
        }
        Direction::Right => {
            next_head_pos.x += 1;
        }
        Direction::Up => {
            next_head_pos.y += 1;
        }
        Direction::Down => {
            next_head_pos.y -= 1;
        }
    }

    if next_head_pos.x < 0
        || next_head_pos.y < 0
        || next_head_pos.x as u32 >= ARENA_WIDTH
        || next_head_pos.y as u32 >= ARENA_HEIGHT
    {
        return None;
    } else {
        for segment_pos in segments.iter() {
            if next_head_pos == *segment_pos {
                return None;
            }
        }
    }

    return Some(next_head_pos);
}

fn snake_movement(
    mut heads: Query<(
        &mut Position,
        &mut SnakeHead,
        &SnakeBody,
        &mut LastTailPos,
    )>,
    mut segments: Query<
        &mut Position,
        (With<SnakeSegment>, Without<SnakeHead>),
    >,
    time: Res<Time>,
    mut config: ResMut<SnakeMovementConfig>,
    mut app_state: ResMut<State<AppState>>,
) {
    config.timer.tick(time.delta());
    if !config.timer.finished() {
        return;
    }

    for (mut head_pos, mut head, body, mut last_tail_pos) in heads.iter_mut() {
        if head.input_direction.is_some() {
            head.direction = head.input_direction.unwrap();
        }
        let next_head_pos =
            get_next_head_pos(*head_pos, head.direction, &segments);
        if next_head_pos.is_none() {
            app_state.set(AppState::Ended).unwrap();
            return;
        }

        let mut front_pos = *head_pos;
        for entity in body.0.iter() {
            let mut pos = segments.get_mut(*entity).unwrap();
            let temp_pos = *pos;
            *pos = front_pos;
            front_pos = temp_pos;
        }
        *last_tail_pos = LastTailPos(front_pos);
        *head_pos = next_head_pos.unwrap();
    }
}

fn snake_eating_and_growth(
    mut commands: Commands,
    food_positions: Query<(&Position, Entity), With<Food>>,
    mut head_positions: Query<
        (&Position, &LastTailPos, &mut SnakeBody),
        With<SnakeHead>,
    >,
) {
    for (head_pos, last_tail_pos, mut body) in head_positions.iter_mut() {
        for (food_pos, food) in food_positions.iter() {
            if head_pos == food_pos {
                commands.entity(food).despawn();
                spawn_segment(&mut commands, last_tail_pos.0, &mut *body);
            }
        }
    }
}

fn game_reset(
    mut commands: Commands,
    foods: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
) {
    for food in foods.iter() {
        commands.entity(food).despawn();
    }
    for segment in segments.iter() {
        commands.entity(segment).despawn();
    }
}

fn spawn_segment(commands: &mut Commands, pos: Position, body: &mut SnakeBody) {
    let segment = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakeSegment)
        .insert(pos)
        .insert(Size::square(0.75))
        .id();
    body.0.push(segment);
}

fn spawn_snake(mut commands: Commands, mut app_state: ResMut<State<AppState>>) {
    let head = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakeHead {
            direction: Direction::Up,
            input_direction: None,
        })
        .insert(SnakeSegment)
        .insert(Position { x: 3, y: 3 })
        .insert(Size::square(0.8))
        .id();

    let tail_pos = Position { x: 3, y: 2 };
    let mut body = SnakeBody(Vec::default());
    spawn_segment(&mut commands, tail_pos, &mut body);

    commands
        .entity(head)
        .insert(body)
        .insert(LastTailPos(tail_pos));

    commands.insert_resource(SnakeMovementConfig {
        timer: Timer::new(
            Duration::from_millis(SNAKE_MOVEMENT_PERIOD_MS),
            true,
        ),
    });
    app_state.set(AppState::Running).unwrap();
}

fn start_food_spawner(mut commands: Commands) {
    commands.insert_resource(FoodSpawnerConfig {
        timer: Timer::new(Duration::from_millis(FOOD_SPAWN_PERIOD_MS), true),
    })
}

fn food_spawner(
    mut commands: Commands,
    segments: Query<&Position, With<SnakeSegment>>,
    foods: Query<&Position, With<Food>>,
    time: Res<Time>,
    mut config: ResMut<FoodSpawnerConfig>,
) {
    config.timer.tick(time.delta());

    if config.timer.finished() && foods.is_empty() {
        let mut rng = rand::thread_rng();
        let pos = Position {
            x: rng.gen_range(0..ARENA_WIDTH as i32),
            y: rng.gen_range(0..ARENA_HEIGHT as i32),
        };

        let mut vacant = true;
        for segment_pos in segments.iter() {
            if pos == *segment_pos {
                vacant = false;
            }
        }
        if vacant {
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: FOOD_COLOR,
                        ..default()
                    },
                    ..default()
                })
                .insert(Food)
                .insert(pos)
                .insert(Size::square(0.8));
        }
    }
}

fn main() {
    App::new()
        .insert_resource(LogSettings {
            filter: "warn,bevy_snake=debug".into(),
            level: bevy::log::Level::DEBUG,
        })
        .insert_resource(WindowDescriptor {
            title: "Snake".to_string(),
            width: 500.0,
            height: 500.0,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_startup_system(setup_game)
        .add_system(game_control_input)
        .add_system_set(
            SystemSet::on_update(AppState::Starting)
                .with_system(spawn_snake)
                .with_system(start_food_spawner),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Running)
                .with_system(snake_movement)
                .with_system(snake_eating_and_growth.after(snake_movement)),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Running)
                .with_system(snake_head_movement_input.before(snake_movement)),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Running).with_system(food_spawner),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::Ended).with_system(game_reset),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(size_scaling)
                .with_system(position_translation),
        )
        .add_state(AppState::Starting)
        .add_plugins(DefaultPlugins)
        .run();
}
