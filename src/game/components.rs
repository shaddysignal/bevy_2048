use std::fmt::{Display, Formatter};
use bevy::ecs::component::{ComponentId};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use crate::game::sprites::BoardSprites;

/// Resource to hold current and previous boards
#[derive(Resource)]
pub struct BoardStateResource(pub Board<Entity>);

/// presumably vec of NxN map, with `col + row * N` as index
#[derive(Resource, Deref, DerefMut, Eq, PartialEq, Debug, Clone)]
pub struct Board<T>(pub Vec<Option<T>>);

impl Display for Board<Entity> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Board :")?;
        writeln!(f, "      {:?}", &self.0[0..4])?;
        writeln!(f, "      {:?}", &self.0[4..8])?;
        writeln!(f, "      {:?}", &self.0[8..12])?;
        writeln!(f, "      {:?}", &self.0[12..16])
    }
}

/// Marker for movable element
#[derive(Component)]
pub struct Block;

/// Marker for possibility of collision
#[derive(Component)]
pub struct Collider;

/// Represents col and row for [Block] on [Board]
#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Position(pub usize, pub usize);

/// Represents value for [Block] on [Board]
#[derive(Component, Eq, PartialEq)]
#[component(on_insert = on_value_insert)]
pub struct Value(pub usize);

fn on_value_insert(mut world: DeferredWorld, entity: Entity, _component_id: ComponentId) {
    let new_value = world.get::<Value>(entity).expect("Value exists on block");
    let board_sprites = world.get_resource::<BoardSprites>().expect("BoardSprites available");
    
    let new_sprites = board_sprites.get(&new_value.0).expect("Sprite for value does not exist").clone();
    world.commands().entity(entity).insert(new_sprites);
}

/// Move to be processed
/// 1. move to
/// 1. in time
#[derive(Component)]
pub struct QueuedMove(pub Position, pub Timer);

/// Merge to be processed
/// 1. move to
/// 1. in time
/// 1. merge to
#[derive(Component)]
pub struct QueuedMerge(pub Position, pub Timer, pub Entity);

/// Direction of board shuffle
#[derive(Default, Debug)]
pub enum Direction {
    Left,
    Right,
    Up,
    #[default]
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
            Direction::Left => RotateBy::Right,
            Direction::Right => RotateBy::Left,
            Direction::Up => RotateBy::None,
            Direction::Down => RotateBy::Full,
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

/// Event for movement to new position in global coordinates
/// 1. entity to move
/// 1. position component to move to
/// 1. time to move to position
/// 1. block merge happens
#[derive(Event)]
pub struct QueuedMoveEvent(pub Entity, pub Position, pub Timer, pub Option<Entity>);

/// Event to signal collision
#[derive(Event, Default)]
pub struct CollisionEvent;

/// Event for board shuffle
#[derive(Event, Default)]
pub struct DirectionEvent(pub Direction);
