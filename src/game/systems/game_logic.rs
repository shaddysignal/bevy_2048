use crate::game::components::*;
use crate::game::states::*;
use crate::game::utils::*;
use crate::game::*;
use crate::SharedRand;
use bevy::prelude::*;
use rand::Rng;

const COLLISION_EPSILON: f32 = 0.96;

pub fn produce_block_bundle(
    col: usize,
    row: usize,
    val: usize,
) -> (Block, Transform, Collider, Position, Value, OnGameScreen) {
    (
        Block,
        Transform::from_translation(Vec3::new(
            col_to_x(col as i32, RECT_SIZE),
            row_to_y(row as i32, RECT_SIZE),
            9.,
        )),
        Collider,
        Position(col, row),
        Value(val),
        OnGameScreen,
    )
}

pub fn is_neighbours_mergeable(
    board: &Board<4, Entity>,
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

pub fn extract_value(
    board: &Board<4, Entity>,
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

pub fn rotate_board<const N: usize, T: Copy>(
    board: &Board<N, T>,
    rotate_by: RotateBy,
) -> Board<N, T> {
    let mut rotated: Vec<Option<T>> = vec![None; N * N];

    for i in 0..N {
        for j in 0..N {
            let (c, r) = rotate_index::<N>(i, j, rotate_by);
            rotated[c + r * N] = board[i + j * N];
        }
    }

    Board(rotated)
}

pub fn rotate_index<const N: usize>(c: usize, r: usize, rotate_by: RotateBy) -> (usize, usize) {
    match rotate_by {
        RotateBy::None => (c, r),
        RotateBy::Left => (r, N - 1 - c),
        RotateBy::Right => (N - 1 - r, c),
        RotateBy::Full => (N - 1 - c, N - 1 - r),
    }
}

pub fn the_end_system(
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
    game_state.set(GameState::Lose);
}

#[inline]
pub fn acquire_empty_tile<const N: usize>(
    shared_rand: &mut SharedRand,
    board: &Board<N, Entity>,
) -> Option<(usize, usize, usize)> {
    let empty_spaces = board.empty_indices();
    if empty_spaces.is_empty() {
        return None;
    }

    let big_val: bool = shared_rand.random_ratio(1, 5);

    let index_rand = shared_rand.random_range(0..empty_spaces.len());
    let row: usize = empty_spaces[index_rand] / N;
    let col: usize = empty_spaces[index_rand] % N;

    Some((col, row, if big_val { 2 } else { 1 }))
}

pub fn produce_new_tile_system(
    mut commands: Commands,
    mut shared_rand: ResMut<SharedRand>,
    mut board_state: ResMut<BoardStateResource>,
) {
    let BoardStateResource(board) = board_state.as_mut();

    let Some((col, row, val)) = acquire_empty_tile(shared_rand.as_mut(), board) else {
        panic!("Shouldn't acquire empty tile after ending")
    };
    let entity = commands.spawn(produce_block_bundle(col, row, val)).id();

    board[col + row * SIZE] = Some(entity);
    trace!("produced new tile at [{}, {}] with value {}", col, row, val);
}

pub fn collision_system(
    collider_query: Query<(Entity, &Transform), With<Collider>>,
    mut collision_messages: MessageWriter<CollisionMessage>,
) {
    // Recreate tree every frame, rstar crate is specifically calls for bulk load as faster method
    let tree_nodes = collider_query
        .iter()
        .map(|(entity, transform)| TreeNode {
            entity: Some(entity),
            position: [transform.translation.x, transform.translation.y],
        })
        .collect::<Vec<_>>();
    let collision_tree = RTree::bulk_load(tree_nodes);
    let mut collisions = Vec::new();
    let mut processed = Vec::new();

    for (entity, transform) in collider_query.iter() {
        let mut neighbors = collision_tree.nearest_neighbor_iter_with_distance_2(&TreeNode {
            entity: Some(entity),
            position: [transform.translation.x, transform.translation.y],
        });
        // Skip closest since it is the entity itself
        neighbors.next();

        let Some(closest_neighbor) = neighbors.next() else {
            warn!("Collision tree has less then 2 nodes");
            continue;
        };
        if closest_neighbor.1 < (RECT_SIZE * RECT_SIZE * COLLISION_EPSILON)
            && !processed.contains(&entity)
        {
            let higher;
            let lower;
            let direction;
            let other_transform = collider_query
                .get(closest_neighbor.0.entity.unwrap())
                .unwrap()
                .1;

            if transform.translation.z < other_transform.translation.z {
                higher = entity;
                lower = closest_neighbor.0.entity.unwrap();
                direction = get_direction(transform.translation, other_transform.translation);
            } else {
                higher = closest_neighbor.0.entity.unwrap();
                lower = entity;
                direction = get_direction(other_transform.translation, transform.translation);
            }

            processed.push(lower);
            processed.push(higher);
            // Ignore collision if direction can't be ascertained
            if let Some(direction) = direction {
                collisions.push((lower, higher, direction));
            }
        }
    }

    for (e1, e2, d) in collisions {
        collision_messages.write(CollisionMessage {
            left: e1,
            right: e2,
            direction: d,
        });
    }
}

#[inline]
fn get_direction(from: Vec3, to: Vec3) -> Option<Direction> {
    match from.xy() - to.xy() {
        v if v.x == 0. && v.y < 0. => Some(Direction::Up),
        v if v.x == 0. && v.y > 0. => Some(Direction::Down),
        v if v.x < 0. && v.y == 0. => Some(Direction::Left),
        v if v.x > 0. && v.y == 0. => Some(Direction::Right),
        _ => None,
    }
}

pub fn process_collision_messages_system(
    position_query: Query<&Transform, With<Collider>>,
    mut collision_messages: MessageReader<CollisionMessage>,
    mut merge_effect_message: MessageWriter<MergeEffectMessage>,
) {
    for collision_message in collision_messages.read() {
        let p1 = position_query.get(collision_message.left).unwrap();
        let p2 = position_query.get(collision_message.right).unwrap();

        let bound1 = translation_to_rect(p1.translation, RECT_SIZE);
        let bound2 = translation_to_rect(p2.translation, RECT_SIZE);
        let collider_bound = bound1.intersect(bound2);
        let collider_center = collider_bound.center();
        let bound = match collision_message.direction {
            Direction::Right => (
                vec2(collider_bound.max.x, collider_bound.min.y) - collider_center,
                vec2(collider_bound.max.x, collider_bound.max.y) - collider_center,
            ),
            Direction::Left => (
                vec2(collider_bound.min.x, collider_bound.min.y) - collider_center,
                vec2(collider_bound.min.x, collider_bound.max.y) - collider_center,
            ),
            Direction::Down => (
                vec2(collider_bound.min.x, collider_bound.min.y) - collider_center,
                vec2(collider_bound.max.x, collider_bound.min.y) - collider_center,
            ),
            Direction::Up => (
                vec2(collider_bound.min.x, collider_bound.max.y) - collider_center,
                vec2(collider_bound.max.x, collider_bound.max.y) - collider_center,
            ),
        };

        merge_effect_message.write(MergeEffectMessage {
            entity: collision_message.right,
            line: bound,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{RngCore, SeedableRng};
    use rand_chacha::rand_core::impls;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn rotate_test() {
        let board_vec: Vec<Option<usize>> = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
            .iter()
            .map(|&x| Some(x))
            .collect();
        let full_board = rotate_board(&Board::<4, usize>(board_vec.clone()), RotateBy::Full);
        assert_eq!(
            full_board,
            Board::<4, usize>(
                [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
                    .iter()
                    .map(|&x| Some(x))
                    .collect()
            )
        );

        let left_board = rotate_board(&Board::<4, usize>(board_vec.clone()), RotateBy::Left);
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

        let right_board = rotate_board(&Board::<4, usize>(board_vec.clone()), RotateBy::Right);
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
        assert_eq!((0, 3), rotate_index::<4>(0, 0, rotate_value));
        assert_eq!((0, 0), rotate_index::<4>(0, 3, rotate_value.revert()));

        assert_eq!((1, 1), rotate_index::<4>(1, 2, RotateBy::Right));
    }

    #[test]
    fn get_direction_test() {
        assert_eq!(
            get_direction(Vec3::new(1., 0., 0.), Vec3::new(1., 1., 0.)),
            Some(Direction::Down)
        );

        assert_eq!(
            get_direction(Vec3::new(1., 0., 0.), Vec3::new(1., -1., 0.)),
            Some(Direction::Up)
        );

        assert_eq!(
            get_direction(Vec3::new(1., 0., 0.), Vec3::new(2., 0., 0.)),
            Some(Direction::Right)
        );

        assert_eq!(
            get_direction(Vec3::new(1., 0., 0.), Vec3::new(0., 0., 0.)),
            Some(Direction::Left)
        );

        assert_eq!(
            get_direction(Vec3::new(1., 0., 0.), Vec3::new(1., 0., 0.)),
            None
        );

        // No tests for diagonal since it never will happen by the design choice
    }

    #[test]
    fn acquire_empty_tile_test() {
        // ChaCha8Rng does not seem to extend anything for us to abstract away. So we implement special deterministic Rand, and pray that test passes (put specific output)
        let mut rand = SharedRand(ChaCha8Rng::from_rng(&mut TestRand { seq: vec![0, 1, 2, 3], index: 0 }));
        let board = Board::<2, Entity>(vec![None; 4]);

        assert_eq!(acquire_empty_tile(&mut rand, &board), Some((1, 0, 1)));

        let mut rand = SharedRand(ChaCha8Rng::from_rng(&mut TestRand { seq: vec![3, 1, 2, 3], index: 0 }));
        let board = Board::<2, Entity>(vec![None; 4]);

        assert_eq!(acquire_empty_tile(&mut rand, &board), Some((1, 0, 2)));

        let mut rand = SharedRand(ChaCha8Rng::from_rng(&mut TestRand { seq: vec![3, 1, 2, 3], index: 0 }));
        let board = Board::<2, Entity>(vec![Some(Entity::from_bits(rand.next_u64())); 4]);

        assert_eq!(acquire_empty_tile(&mut rand, &board), None);
    }

    struct TestRand {
        seq: Vec<u8>,
        index: usize
    }
    impl RngCore for TestRand {
        fn next_u32(&mut self) -> u32 {
            let value = self.seq[self.index];
            self.index += 1;
            if self.index >= self.seq.len() {
                self.index = 0
            }

            value as u32
        }

        fn next_u64(&mut self) -> u64 {
            self.next_u32() as u64
        }

        fn fill_bytes(&mut self, dst: &mut [u8]) {
            impls::fill_bytes_via_next(self, dst)
        }
    }
}
