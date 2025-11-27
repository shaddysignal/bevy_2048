mod components;
pub mod effects;
mod sprites;
mod states;
mod systems;
mod utils;

use crate::game::components::*;
use crate::game::sprites::sprites_plugin;
use crate::game::states::*;
use crate::game::systems::effect::*;
use crate::game::systems::game_logic::*;
use crate::game::systems::input::*;
use crate::game::systems::movement::*;
use crate::game::systems::process::*;
use crate::game::utils::*;
use crate::menu::{despawn_screen, AppState};
use bevy::app::App;
use bevy::color::Color;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rstar::{Point, RTree};
use crate::SharedRand;

// Constants that interchangeable during the run
const SIZE: usize = 4;
const RECT_SIZE: f32 = 250.;
const X_SHIFT: f32 = RECT_SIZE * 2.;
const Y_SHIFT: f32 = RECT_SIZE * 1.5;

#[derive(Clone, PartialEq, Debug)]
struct TreeNode {
    pub entity: Option<Entity>,
    pub position: [f32; 2],
}

impl Point for TreeNode {
    type Scalar = f32;
    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        TreeNode {
            entity: None,
            position: [generator(0), generator(1)],
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        self.position[index]
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        &mut self.position[index]
    }
}

pub fn game_plugin(app: &mut App) {
    app.add_plugins((sprites_plugin, ShapePlugin))
        .insert_resource(BoardStateResource(Board::<SIZE, Entity>(vec![
            None;
            SIZE * SIZE
        ])))
        .insert_resource(GameParams { move_time: 1. })
        .add_message::<CollisionMessage>()
        .add_message::<DirectionMessage>()
        .add_message::<QueuedMoveMessage>()
        .add_message::<MergeEffectMessage>()
        .init_state::<GameState>()
        .add_systems(OnEnter(AppState::Game), (board_setup, game_ui_setup))
        .configure_sets(Update, GameSet.run_if(in_state(AppState::Game)))
        .configure_sets(FixedUpdate, GameSet.run_if(in_state(AppState::Game)))
        // Input stuff
        .add_systems(
            Update,
            generate_direction_messages
                .run_if(in_state(GameState::Wait))
                .in_set(InputSet),
        )
        // Game processes and preparations for movement section
        .add_systems(
            FixedUpdate,
            (
                (process_direction_messages, process_queued_move_messages)
                    .chain()
                    .run_if(in_state(GameState::Process))
                    .in_set(GameSet),
            ),
        )
        // Section for UI updates (movement and such)
        .add_systems(
            Update,
            (collision_system, process_collision_messages_system, merge_effect_system, queued_movement_system, queued_system_finished)
                .chain()
                .run_if(in_state(GameState::Movement))
                .in_set(GameSet)
        )
        .add_systems(Update, log_transitions::<GameState>)
        .add_systems(
            OnEnter(GameState::Decision),
            (produce_new_tile_system, the_end_system)
                .chain()
                .in_set(GameSet),
        )
        // TODO: Win/Lose should trigger effects, need to do something about that in OnEnter functions for Win/Lose states.
        .add_systems(OnExit(AppState::Game), despawn_screen::<OnGameScreen>);
}

fn board_setup(
    mut commands: Commands,
    mut shared_rand: ResMut<SharedRand>,
    mut board_state_resource: ResMut<BoardStateResource>,
) {
    let board = &mut board_state_resource.0;

    let Some((col1, row1, val1)) = acquire_empty_tile(shared_rand.as_mut(), board) else { panic!("No empty tile during board setup") };
    let entity1 = commands.spawn(produce_block_bundle(col1, row1, val1)).id();
    board[col1 + row1 * SIZE] = Some(entity1);
    trace!("Board at {}x{} filled with {}", col1, row1, val1);

    let Some((col2, row2, val2)) = acquire_empty_tile(shared_rand.as_mut(), board) else { panic!("No empty tile during board setup") };
    let entity2 = commands.spawn(produce_block_bundle(col2, row2, val2)).id();
    board[col2 + row2 * SIZE] = Some(entity2);
    trace!("Board at {}x{} filled with {}", col2, row2, val2);
}

fn game_ui_setup(
    mut commands: Commands,
) {
    // Board
    for i in 0..SIZE {
        for j in 0..SIZE {
            let rect = shapes::Rectangle {
                extents: Vec2::splat(RECT_SIZE),
                origin: RectangleOrigin::Center,
                radii: None,
            };

            commands.spawn((
                ShapeBuilder::with(&rect)
                    .fill(Color::srgba(0f32, 0f32, 0f32, 0.5))
                    .stroke((Color::WHITE, 10.0))
                    .build(),
                Transform::from_xyz(
                    col_to_x(i as i32, RECT_SIZE),
                    row_to_y(j as i32, RECT_SIZE),
                    0.0,
                ),
                OnGameScreen,
            ));
        }
    }

    // Buttons
    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Start,
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|parent| {
            #[cfg(not(target_arch = "wasm32"))]
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
                            width: Val::Px(75.),
                            height: Val::Px(65.),
                            margin: UiRect::all(Val::Px(20.)),
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
                                font_size: 33.,
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
    use bevy::state::app::StatesPlugin;
    use bevy::time::TimePlugin;

    fn common_app_setup(app: &mut App, board_vec: Vec<usize>) {
        app.add_message::<DirectionMessage>();
        app.add_message::<QueuedMoveMessage>();

        let board_vec: Vec<usize> = board_vec;
        let board_vec: Vec<Option<Entity>> = board_vec
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                if x != 0 {
                    Some(
                        app.world_mut()
                            .spawn(produce_block_bundle(i / 4, i % 4, x))
                            .id(),
                    )
                } else {
                    None
                }
            })
            .collect();
        app.insert_resource(BoardStateResource(Board(board_vec.clone())));
        app.insert_resource(GameParams { move_time: 0. });
        app.init_state::<GameState>();
    }

    #[test]
    fn merge_testing() {
        let mut app = App::new();

        app.add_plugins(StatesPlugin::default());
        common_app_setup(
            &mut app,
            vec![1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        );

        app.add_systems(Update, process_direction_messages);

        app.world_mut()
            .resource_mut::<Messages<DirectionMessage>>()
            .write(DirectionMessage(Direction::Up));

        app.update();

        let queued_move_messages = app.world_mut().resource::<Messages<QueuedMoveMessage>>();
        let mut cursor = queued_move_messages.get_cursor();
        let mut cursor_iter = cursor.read(queued_move_messages);
        let queued_move = cursor_iter.next().unwrap();
        let has_more = cursor_iter.next().is_some();

        assert_eq!(has_more, false);
        assert_eq!(queued_move.1, Position(0, 0));
        assert_eq!(queued_move.3.is_some(), true);

        let merge_with_entity = queued_move.3.unwrap();

        let board = app
            .world_mut()
            .resource_mut::<BoardStateResource>()
            .0
            .clone();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<Block>>()
                .iter(app.world())
                .len(),
            2
        );

        let merge_to_entity = board.0[0];
        assert_eq!(
            app.world_mut()
                .get::<Value>(merge_to_entity.unwrap())
                .unwrap()
                .0,
            1
        );
        assert_eq!(
            app.world_mut().get::<Value>(merge_with_entity).unwrap().0,
            1
        );
    }

    #[test]
    fn merge_with_movement() {
        let mut app = App::new();

        app.add_plugins((StatesPlugin::default(), TimePlugin::default()));
        common_app_setup(
            &mut app,
            vec![0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
        );

        app.add_systems(
            Update,
            (
                process_direction_messages,
                process_queued_move_messages,
                queued_movement_system,
                queued_system_finished,
            )
                .chain(),
        );

        app.world_mut()
            .resource_mut::<Messages<DirectionMessage>>()
            .write(DirectionMessage(Direction::Down));

        app.update();

        let queued_move_messages = app.world_mut().resource::<Messages<QueuedMoveMessage>>();
        let mut cursor = queued_move_messages.get_cursor();
        let cursor_iter = cursor.read(queued_move_messages);
        let messages_created = cursor_iter.len();

        assert_eq!(messages_created, 2);

        let board = app
            .world_mut()
            .resource_mut::<BoardStateResource>()
            .0
            .clone();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<Block>>()
                .iter(app.world())
                .len(),
            1
        );

        let merge_to_entity = board.0[12];
        assert_eq!(
            app.world_mut()
                .get::<Value>(merge_to_entity.unwrap())
                .unwrap()
                .0,
            2
        );
        let merge_with_entity = board.0[4];
        assert_eq!(merge_with_entity.is_none(), true);
        let merge_with_entity = board.0[8];
        assert_eq!(merge_with_entity.is_none(), true);
    }

    #[test]
    fn merge_with_stacks() {
        let mut app = App::new();

        app.add_plugins((StatesPlugin::default(), TimePlugin::default()));
        common_app_setup(
            &mut app,
            vec![1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
        );

        app.add_systems(
            Update,
            (
                process_direction_messages,
                process_queued_move_messages,
                queued_movement_system,
                queued_system_finished,
            )
                .chain(),
        );

        app.world_mut()
            .resource_mut::<Messages<DirectionMessage>>()
            .write(DirectionMessage(Direction::Up));

        app.update();

        let queued_move_messages = app.world_mut().resource::<Messages<QueuedMoveMessage>>();
        let mut cursor = queued_move_messages.get_cursor();
        let cursor_iter = cursor.read(queued_move_messages);
        let messages_created = cursor_iter.len();

        assert_eq!(messages_created, 2);

        let board = app
            .world_mut()
            .resource_mut::<BoardStateResource>()
            .0
            .clone();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<Block>>()
                .iter(app.world())
                .len(),
            2
        );

        let merge_to_entity = board.0[0];
        assert_eq!(
            app.world_mut()
                .get::<Value>(merge_to_entity.unwrap())
                .unwrap()
                .0,
            2
        );
        let merge_with_entity = board.0[4];
        assert_eq!(
            app.world_mut()
                .get::<Value>(merge_with_entity.unwrap())
                .unwrap()
                .0,
            1
        );
        let no_merge_entity = board.0[8];
        assert_eq!(no_merge_entity.is_none(), true);
    }
}
