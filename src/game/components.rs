use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use std::fmt::{Display, Formatter};
use crate::game::SIZE;

/// Marker for game screen
#[derive(Component)]
pub struct OnGameScreen;

/// Resource to hold current and previous boards
#[derive(Resource)]
pub struct BoardStateResource(pub Board<SIZE, Entity>);

/// presumably vec of NxN map, with `col + row * N` as index
#[derive(Resource, Deref, DerefMut, Eq, PartialEq, Debug, Clone)]
pub struct Board<const N: usize, T>(pub Vec<Option<T>>);

impl<const N: usize, T> Board<N, T> {
    pub fn empty_indices(&self) -> Vec<usize> {
        self.0.iter()
            .enumerate()
            .filter(|(_, e)| e.is_none())
            .map(|(i, _)| i)
            .collect()
    }
}

impl Display for Board<SIZE, Entity> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (chunks, []) = self.as_chunks::<SIZE>() else { panic!("Board should be SIZExSIZE") };

        writeln!(f, "Board :")?;
        for chunk in chunks {
            writeln!(f, "      {:?}", chunk)?;
        }
        write!(f, "")
    }
}

/// Marker for movable element
#[derive(Component)]
pub struct Block;

/// Represents a thing that can collide
#[derive(Component)]
pub struct Collider;

/// Represents col and row for [Block] on [Board]
#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Position(pub usize, pub usize);

/// Represents value for [Block] on [Board]
#[derive(Component, Eq, PartialEq, Debug)]
#[component(on_insert = on_value_insert)]
pub struct Value(pub usize);

// TODO: need to see is it the way to do it. Tests pass that way, maybe there other way to mock BoardSprites in tests.
#[cfg(not(test))]
fn on_value_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    use crate::game::sprites::BoardSprites;

    let new_value = world.get::<Value>(entity).expect("No value exists on block");
    let board_sprites = world.get_resource::<BoardSprites>().expect("No BoardSprites available");

    let new_sprites = board_sprites.get(&new_value.0).expect("No sprite for value exists").clone();
    world.commands().entity(entity).insert(new_sprites);
}

#[cfg(test)]
fn on_value_insert(world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let new_value = world.get::<Value>(entity).expect("No value exists on block");
    println!("Inserted Value: {}", new_value.0);
}

/// Move to be processed
/// 1. move to
/// 1. in time
/// 1. merge to
#[derive(Component)]
pub struct QueuedMove(pub Position, pub Timer, pub Option<Entity>);

/// Direction
#[derive(Default, Eq, PartialEq, Debug)]
pub enum Direction {
    Left,
    #[default]
    Right,
    Up,
    Down
}

/// Direction of rotation
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum RotateBy {
    None, Left, Right, Full
}

impl RotateBy {
    pub fn from_direction(direction: &Direction) -> Self {
        match direction {
            Direction::Left => RotateBy::None,
            Direction::Right => RotateBy::Full,
            Direction::Up => RotateBy::Left,
            Direction::Down => RotateBy::Right,
        }
    }

    pub fn revert(&self) -> Self {
        match self {
            RotateBy::None => RotateBy::None,
            RotateBy::Left => RotateBy::Right,
            RotateBy::Right => RotateBy::Left,
            RotateBy::Full => RotateBy::Full,
        }
    }
}

/// Message for movement to new position in global coordinates
/// 1. entity to move
/// 1. position component to move to
/// 1. time to move to position
/// 1. block merge happens
#[derive(Message)]
pub struct QueuedMoveMessage(pub Entity, pub Position, pub Timer, pub Option<Entity>);

/// Message to signal collision
/// 1. left: attractor
/// 1. right: attracted
#[derive(Message)]
pub struct CollisionMessage{
    pub left: Entity,
    pub right: Entity,
    pub direction: Direction,
}

/// Message for board shuffle
#[derive(Message, Default)]
pub struct DirectionMessage(pub Direction);

/// Message for displaying merge effect
#[derive(Message)]
pub struct MergeEffectMessage {
    pub entity: Entity,
    pub line: (Vec2, Vec2)
}

/// Params for changeable game params
#[derive(Resource)]
pub struct GameParams {
    pub move_time: f32,
}