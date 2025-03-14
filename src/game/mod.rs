mod components;
mod sprites;

use crate::game::components::{
    Block, Board, BoardStateResource, Collider, CollisionEvent, Direction, DirectionEvent,
    Position, QueuedMove, QueuedMoveEvent, RotateBy, Value,
};
use crate::game::sprites::{sprites_plugin};
use crate::menu::{despawn_screen, AppState};
use crate::SharedRand;
use bevy::app::App;
use bevy::color::Color;
use bevy::hierarchy::{BuildChildren, ChildBuild};
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::Rng;
use std::cmp::max;

const SIZE: usize = 4;
const RECT_SIZE: f32 = 250.0;

#[derive(States, Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
enum GameState {
    Win,
    Lose,
    #[default]
    Wait,
    Process,
    Movement,
    Decision,
}

#[derive(SystemSet, Clone, Eq, PartialEq, Hash, Debug)]
struct GameSet;

pub fn game_plugin(app: &mut App) {
    app.add_plugins((sprites_plugin, ShapePlugin))
        .insert_resource(BoardStateResource(Board::<Entity>(vec![None; SIZE * SIZE])))
        .add_event::<CollisionEvent>()
        .add_event::<DirectionEvent>()
        .add_event::<QueuedMoveEvent>()
        .init_state::<GameState>()
        .add_systems(OnEnter(AppState::Game), (board_setup, game_ui_setup))
        .configure_sets(FixedUpdate, GameSet.run_if(in_state(AppState::Game)))
        .add_systems(
            FixedUpdate,
            (
                generate_direction_events
                    .run_if(in_state(GameState::Wait))
                    .in_set(GameSet),
                (process_direction_events, process_queued_move_events)
                    .chain()
                    .run_if(in_state(GameState::Process))
                    .in_set(GameSet),
                queued_movement_system
                    .run_if(in_state(GameState::Movement))
                    .in_set(GameSet),
                (produce_new_tile_system, the_end_system)
                    .chain()
                    .run_if(in_state(GameState::Decision))
                    .in_set(GameSet),
            ),
        )
        .add_systems(OnExit(AppState::Game), despawn_screen::<OnGameScreen>);
}

#[derive(Component)]
struct OnGameScreen;

fn col_to_x(col: usize) -> f32 {
    col as f32 * RECT_SIZE - 500.0
}

fn row_to_y(row: usize) -> f32 {
    -(row as f32) * RECT_SIZE + 375.0
}

fn board_setup(
    mut commands: Commands,
    mut shared_rand: ResMut<SharedRand>,
    mut board_state_resource: ResMut<BoardStateResource>,
) {
    // board
    for i in 0..4 {
        for j in 0..4 {
            let rect = shapes::Rectangle {
                extents: Vec2::splat(RECT_SIZE),
                origin: RectangleOrigin::Center,
                radii: None,
            };

            commands.spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&rect),
                    transform: Transform::from_xyz(col_to_x(i), row_to_y(j), 0.0),
                    ..default()
                },
                Stroke::new(Color::WHITE, 10.0),
                Fill::color(Color::srgba(0f32, 0f32, 0f32, 0.5)),
                OnGameScreen,
            ));
        }
    }

    // tiles
    let board = &mut board_state_resource.0;

    let (col1, row1, val1) = acquire_empty_tile(shared_rand.as_mut(), board);
    let entity1 = commands.spawn(produce_block_bundle(col1, row1, val1)).id();
    board[col1 + row1 * 4] = Some(entity1);
    debug!("Board at {}x{} filled with {}", col1, row1, val1);

    let (col2, row2, val2) = acquire_empty_tile(shared_rand.as_mut(), board);
    let entity2 = commands.spawn(produce_block_bundle(col2, row2, val2)).id();
    board[col2 + row2 * 4] = Some(entity2);
    debug!("Board at {}x{} filled with {}", col2, row2, val2);
}

fn produce_block_bundle(
    col: usize,
    row: usize,
    val: usize,
) -> (Block, Transform, Collider, Position, Value, OnGameScreen) {
    (
        Block,
        Transform {
            translation: Vec3::new(col_to_x(col), row_to_y(row), 1f32),
            ..default()
        },
        Collider,
        Position(col, row),
        Value(val),
        OnGameScreen,
    )
}

fn acquire_empty_tile(
    shared_rand: &mut SharedRand,
    board: &Board<Entity>,
) -> (usize, usize, usize) {
    let val: usize = shared_rand.random_range(1..3);

    let row: usize = shared_rand.random_range(0..4);
    let col: usize = shared_rand.random_range(0..4);

    if board[col + row * 4].is_none() {
        (col, row, val)
    } else {
        acquire_empty_tile(shared_rand, board)
    }
}

fn generate_direction_events(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut direction_events: EventWriter<DirectionEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        direction_events.send(DirectionEvent(Direction::Left));
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        direction_events.send(DirectionEvent(Direction::Right));
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        direction_events.send(DirectionEvent(Direction::Down));
    } else if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        direction_events.send(DirectionEvent(Direction::Up));
    }

    if keyboard_input.any_just_pressed([
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowDown,
        KeyCode::ArrowUp,
    ]) {
        game_state.set(GameState::Process);
    }
}

fn process_direction_events(
    mut board_state_resource: ResMut<BoardStateResource>,
    mut game_state: ResMut<NextState<GameState>>,
    mut direction_events: EventReader<DirectionEvent>,
    mut queued_move_events: EventWriter<QueuedMoveEvent>,
    block_query: Query<&Value, With<Block>>,
) {
    if direction_events.is_empty() {
        return;
    }

    // Take first to process, clear others
    let board = &board_state_resource.0;
    let event = direction_events.read().next().unwrap();
    let rotate_value = RotateBy::from_direction(&event.0);
    let mut rotated_board = rotate_board(board, rotate_value);
    let mut merges: Vec<usize> = Vec::new();

    debug!("{}", board);
    debug!("Rotated {}", rotated_board);

    // TODO: holy this is so much over typing, probably doing something wrong all together
    for c in 0..4 {
        let mut row_to_be_filled: usize = 0;

        for r in 0..4 {
            let to_merge = row_to_be_filled as i32 - 1;
            let location = c + r * 4;
            let new_location = max(0, c as i32 + to_merge * 4) as usize;
            let tile = rotated_board[location];

            if tile.is_some() {
                // TODO: when more then 2 the same - they all merge together. More testing to catch no valid entity for value in movement

                // if there: process and make event for movement
                let mut to_row = to_merge;
                let mut merged_tile = None;

                let current = extract_value(&rotated_board, &block_query, location);
                let new_current = extract_value(&rotated_board, &block_query, new_location);

                rotated_board[location] = None;

                if (0..4).contains(&to_merge) && !merges.contains(&new_location) && current == new_current {
                    merged_tile = rotated_board[new_location];
                    rotated_board[new_location] = tile;

                    merges.push(new_location);
                } else {
                    rotated_board[c + row_to_be_filled * 4] = tile;
                    to_row = row_to_be_filled as i32;

                    row_to_be_filled += 1;
                }

                // restoring index
                let (original_c, original_r) = rotate_index(c, r, rotate_value.revert());
                let (original_to_col, original_to_row) =
                    rotate_index(c, to_row as usize, rotate_value.revert());

                if (original_c, original_r) != (original_to_col, original_to_row) {
                    queued_move_events.send(QueuedMoveEvent(
                        board[original_c + original_r * 4]
                            .expect("Value in original coords should exist"),
                        Position(original_to_col, original_to_row),
                        Timer::from_seconds(0.5, TimerMode::Once),
                        merged_tile
                    ));
                }
            }
        }
    }

    let new_board = rotate_board(&rotated_board, rotate_value.revert());

    direction_events.clear();
    if new_board.0 != board.0 {
        game_state.set(GameState::Movement);
    } else {
        debug!("No changes after shuffle");
        game_state.set(GameState::Wait);
        return;
    }

    *board_state_resource = BoardStateResource(new_board);
}

fn produce_new_tile_system(
    mut commands: Commands,
    mut shared_rand: ResMut<SharedRand>,
    mut board_state: ResMut<BoardStateResource>,
) {
    let BoardStateResource(board) = board_state.as_mut();

    let (col, row, val) = acquire_empty_tile(shared_rand.as_mut(), board);
    let entity = commands.spawn(produce_block_bundle(col, row, val)).id();

    board[col + row * 4] = Some(entity);
}

fn the_end_system(
    mut game_state: ResMut<NextState<GameState>>,
    board_state_resource: Res<BoardStateResource>,
    block_query: Query<&Value, With<Block>>,
) {
    let board = &board_state_resource.0;
    for c in 0..4 {
        for r in 0..4 {
            let tile_value = extract_value(board, &block_query, c + r * 4);
            if tile_value == 11 {
                // win
                info!("win");
                game_state.set(GameState::Win);
                return;
            }

            if is_neighbours_mergeable(board, &block_query, c, r) {
                game_state.set(GameState::Wait);
                return;
            }
        }
    }

    // game over
    info!("lose");
    game_state.set(GameState::Lose);
}

fn is_neighbours_mergeable(
    board: &Board<Entity>,
    block_query: &Query<&Value, With<Block>>,
    col: usize,
    row: usize,
) -> bool {
    let tile = extract_value(board, block_query, col + row * 4);
    let range = 0..4i32;

    if tile == 0 {
        return true;
    } else {
        if range.contains(&(col as i32 + 1))
            && (tile == extract_value(board, block_query, col + 1 + row * 4))
        {
            return true;
        }
        if range.contains(&(col as i32 - 1))
            && (tile == extract_value(board, block_query, col - 1 + row * 4))
        {
            return true;
        }
        if range.contains(&(row as i32 + 1))
            && (tile == extract_value(board, block_query, col + (row + 1) * 4))
        {
            return true;
        }
        if range.contains(&(row as i32 - 1))
            && (tile == extract_value(board, block_query, col + (row - 1) * 4))
        {
            return true;
        }
    }

    false
}

fn extract_value(
    board: &Board<Entity>,
    block_query: &Query<&Value, With<Block>>,
    index: usize,
) -> usize {
    let tile = board[index];
    if tile.is_none() {
        0
    } else {
        let tile_value = block_query
            .get(tile.unwrap())
            .expect("All block should have value and be on board");
        tile_value.0
    }
}

fn rotate_board<T: Copy>(board: &Board<T>, rotate_by: RotateBy) -> Board<T> {
    let mut rotated: Vec<Option<T>> = vec![None; 16];

    for i in 0..4 {
        for j in 0..4 {
            let (c, r) = rotate_index(i, j, rotate_by);
            rotated[c + r * 4] = board[i + j * 4];
        }
    }

    Board(rotated)
}

fn rotate_index(c: usize, r: usize, rotate_by: RotateBy) -> (usize, usize) {
    match rotate_by {
        RotateBy::None => (c, r),
        RotateBy::Left => (r, 3 - c),
        RotateBy::Right => (3 - r, c),
        RotateBy::Full => (3 - c, 3 - r),
    }
}

fn process_queued_move_events(
    mut commands: Commands,
    mut queued_move_events: EventReader<QueuedMoveEvent>,
) {
    for QueuedMoveEvent(entity, to, in_time, to_merge_with) in queued_move_events.read() {
        // TODO: Simply hide the occasional error of non existing entity, most likely connected to merge
        if commands.get_entity(*entity).is_some() {
            commands
                .entity(*entity)
                .insert(QueuedMove(*to, in_time.clone(), *to_merge_with));
        } else {
            error!(
                "Entity({}) was deleted before processing event, movement to {:?}",
                entity, to
            );
        }
    }
}

fn queued_movement_system(
    time: Res<Time>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut block_query: Query<(Entity, &Position, &Value, &mut Transform, &mut QueuedMove)>,
) {
    for (entity, _position, value, mut transform, mut queued_move) in block_query.iter_mut() {
        let QueuedMove(to, timer, to_merge_with) = queued_move.as_mut();

        let to_vec = Vec2::new(col_to_x(to.0), row_to_y(to.1));

        timer.tick(time.delta());
        if timer.finished() {
            transform.translation.x = to_vec.x;
            transform.translation.y = to_vec.y;

            commands.entity(entity).remove::<QueuedMove>();

            if let Some(merge_entity) = to_merge_with {
                commands.entity(*merge_entity).despawn();
                commands.entity(entity).insert(Value(value.0 + 1));
            }
        } else {
            let path = to_vec - Vec2::new(transform.translation.x, transform.translation.y);
            let path = path * timer.fraction();

            transform.translation.x += path.x;
            transform.translation.y += path.y;
        }
    }

    if block_query.is_empty() {
        game_state.set(GameState::Decision);
    }
}

fn game_ui_setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Start,
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|parent| {
            // First create a `Node` for centering what we want to display
            parent
                .spawn((
                    Node {
                        // This will display its children in a column, from top to bottom
                        flex_direction: FlexDirection::Column,
                        // `align_items` will align children on the cross axis. Here the main axis is
                        // vertical (column), so the cross axis is horizontal. This will center the
                        // children
                        align_items: AlignItems::Start,
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ))
                .with_children(|p| {
                    p.spawn((
                        Button,
                        Node {
                            width: Val::Px(75.0),
                            height: Val::Px(65.0),
                            margin: UiRect::all(Val::Px(20.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                        crate::menu::menu_mod::MenuButtonAction::Quit,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new("X"),
                            TextFont {
                                font_size: 33.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                    });
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_test() {
        let board_vec: Vec<Option<usize>> = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
            .iter()
            .map(|&x| Some(x))
            .collect();
        let full_board = rotate_board(&Board::<usize>(board_vec.clone()), RotateBy::Full);
        assert_eq!(
            full_board,
            Board::<usize>(
                [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
                    .iter()
                    .map(|&x| Some(x))
                    .collect()
            )
        );

        let left_board = rotate_board(&Board(board_vec.clone()), RotateBy::Left);
        assert_eq!(
            left_board,
            Board(
                [3, 7, 11, 15, 2, 6, 10, 14, 1, 5, 9, 13, 0, 4, 8, 12]
                    .iter()
                    .map(|&x| Some(x))
                    .collect()
            )
        );
        let orig_board = rotate_board(&left_board, RotateBy::Right);
        assert_eq!(orig_board, Board(board_vec.clone()));

        let right_board = rotate_board(&Board(board_vec.clone()), RotateBy::Right);
        assert_eq!(
            right_board,
            Board(
                [12, 8, 4, 0, 13, 9, 5, 1, 14, 10, 6, 2, 15, 11, 7, 3]
                    .iter()
                    .map(|&x| Some(x))
                    .collect()
            )
        );
        let orig_board = rotate_board(&right_board, RotateBy::Left);
        assert_eq!(orig_board, Board(board_vec.clone()));
    }

    #[test]
    fn rotate_index_test() {
        let rotate_value = RotateBy::Left;
        assert_eq!((0, 3), rotate_index(0, 0, rotate_value));
        assert_eq!((0, 0), rotate_index(0, 3, rotate_value.revert()));

        assert_eq!((1, 1), rotate_index(1, 2, RotateBy::Right));
    }
}
