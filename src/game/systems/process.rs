use crate::game::components::*;
use crate::game::states::*;
use crate::game::systems::game_logic::*;
use crate::game::SIZE;
use bevy::prelude::*;

pub fn process_direction_messages(
    mut board_state_resource: ResMut<BoardStateResource>,
    game_params: Res<GameParams>,
    mut game_state: ResMut<NextState<GameState>>,
    mut direction_message: MessageReader<DirectionMessage>,
    mut queued_move_message: MessageWriter<QueuedMoveMessage>,
    block_query: Query<&Value, With<Block>>,
    mut transform_query: Query<&mut Transform, With<Block>>,
) {
    if direction_message.is_empty() {
        return;
    }

    let board = &board_state_resource.0;
    // Take first to process, clear others
    let message = direction_message.read().last().unwrap();
    let rotate_value = RotateBy::from_direction(&message.0);
    let mut rotated_board = rotate_board(board, rotate_value);

    trace!("{}", board);
    trace!("Rotated {}", rotated_board);

    let (chunks, []) = rotated_board.as_chunks_mut::<SIZE>() else {
        panic!("Board should be {SIZE}x{SIZE}")
    };
    for row in 0..SIZE {
        let move_ops = process_row(&mut chunks[row], &block_query);

        for move_op in move_ops {
            // Convert rotated coordinates back to original board coordinates
            let (original_column, original_row) =
                rotate_index::<SIZE>(move_op.from, row, rotate_value.revert());
            let (final_column, final_row) =
                rotate_index::<SIZE>(move_op.to, row, rotate_value.revert());

            // Record movement if position changed
            if (original_column, original_row) != (final_column, final_row) {
                if let Some(merge_entity) = move_op.merged {
                    let Ok(mut merge_transform) = transform_query.get_mut(merge_entity) else { panic!("Merged entity should always have Transform"); };
                    merge_transform.translation.z = 8.;
                }

                queued_move_message.write(QueuedMoveMessage(
                    move_op.entity,
                    Position(final_column, final_row),
                    Timer::from_seconds(game_params.move_time, TimerMode::Once),
                    move_op.merged,
                ));
            }
        }
    }

    let new_board = rotate_board(&rotated_board, rotate_value.revert());

    if new_board.0 != board.0 {
        game_state.set(GameState::Movement);
    } else {
        game_state.set(GameState::Wait);
        return;
    }

    *board_state_resource = BoardStateResource(new_board);
}

struct MoveOp {
    entity: Entity,
    to: usize,
    from: usize,
    merged: Option<Entity>,
}

fn process_row(
    row: &mut [Option<Entity>; SIZE],
    block_query: &Query<&Value, With<Block>>,
) -> Vec<MoveOp> {
    let mut moves: Vec<MoveOp> = Vec::new();

    let mut next_available_column = 0;
    let mut current_column = 1;
    let mut merges: Vec<usize> = Vec::new();

    while current_column < SIZE {
        if let Some(current_block) = row[current_column] {
            row[current_column] = None;

            if let Some(target_block) = row[next_available_column] {
                let current_value = block_query.get(current_block).unwrap().0;
                let target_value = block_query.get(target_block).unwrap().0;

                let can_merge =
                    !merges.contains(&next_available_column) && current_value == target_value;

                if can_merge {
                    merges.push(next_available_column);

                    row[next_available_column] = Some(current_block);

                    // Generate info for merging current_block with target_block
                    moves.push(MoveOp {
                        entity: current_block,
                        to: next_available_column,
                        from: current_column,
                        merged: Some(target_block),
                    });
                } else {
                    row[next_available_column] = Some(target_block);
                    row[next_available_column + 1] = Some(current_block);

                    // Generate info for moving current_block next to target_block
                    moves.push(MoveOp {
                        entity: current_block,
                        to: next_available_column + 1,
                        from: current_column,
                        merged: None,
                    });
                }

                // Move on, find the next available column to move
                next_available_column += 1;
                current_column += 1;
            } else {
                row[next_available_column] = Some(current_block);

                // Generate info for moving current_block to next_available_column
                moves.push(MoveOp {
                    entity: current_block,
                    to: next_available_column,
                    from: current_column,
                    merged: None,
                });
            }
        } else {
            // Move on, find the next available column to move
            current_column += 1;
        }
    }

    moves
}

pub fn process_queued_move_messages(
    mut commands: Commands,
    mut queued_move_messages: MessageReader<QueuedMoveMessage>,
) {
    for QueuedMoveMessage(entity, to, in_time, to_merge_with) in queued_move_messages.read() {
        let mut entity_commands = commands.entity(*entity);
        entity_commands.insert(QueuedMove(*to, in_time.clone(), *to_merge_with));
    }
}