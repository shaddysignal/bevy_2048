use bevy::prelude::*;

pub fn animate_sprite_plugin(app: &mut App) {
    app.add_systems(Update, animate_sprite);
}

#[derive(Component, Clone)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}

impl AnimationIndices {
    pub fn new(first: usize, last: usize) -> Self {
        Self { first, last }
    }
}

#[derive(Component, Clone, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

impl AnimationTimer {
    pub fn new(timer: Timer) -> Self { Self(timer) }
}

#[derive(Bundle, Clone)]
pub struct AnimationBundle {
    sprite: Sprite,
    animation_indices: AnimationIndices,
    animation_timer: AnimationTimer,
}

impl AnimationBundle {
    pub fn new(sprite: Sprite, animation_indices: AnimationIndices, animation_timer: AnimationTimer) -> Self {
        Self {
            sprite,
            animation_indices,
            animation_timer
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}