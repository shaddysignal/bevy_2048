use bevy::prelude::*;
use crate::game::components::*;
use crate::game::RECT_SIZE;
use crate::game::states::*;
use crate::game::utils::*;

pub fn queued_system_finished(
    move_block_query: Query<Entity, With<QueuedMove>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if move_block_query.is_empty() {
        // One-time set guaranteed by system being in Update schedule, the same as StateTransition
        next_game_state.set(GameState::Decision);
    }
}

pub fn queued_movement_system(
    time: Res<Time>,
    mut commands: Commands,
    mut moving_block_query: Query<(Entity, &Value, &mut Transform, &mut QueuedMove)>,
) {
    let mut to_delete = Vec::new();

    for (entity, value, mut transform, mut queued_move) in moving_block_query.iter_mut() {
        let QueuedMove(to, timer, merge_entity) = queued_move.as_mut();

        let to_vec = vec2(col_to_x(to.0 as i32, RECT_SIZE), row_to_y(to.1 as i32, RECT_SIZE));

        timer.tick(time.delta());
        if timer.is_finished() {
            transform.translation.x = to_vec.x;
            transform.translation.y = to_vec.y;

            commands.entity(entity).remove::<QueuedMove>();

            if let Some(merge_entity) = merge_entity {
                to_delete.push(*merge_entity);
                commands.entity(entity).insert(Value(value.0 + 1));
            }
        } else {
            let path = to_vec - transform.translation.xy();
            let delta = transform.translation.xy() + path * timer.fraction_remaining() * 0.125;

            transform.translation.x = delta.x;
            transform.translation.y = delta.y;
        }
    }

    for e in to_delete {
        commands.entity(e).despawn();
    }
}