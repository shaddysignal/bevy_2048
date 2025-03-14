use bevy::app::Startup;
use bevy::asset::AssetServer;
use bevy::math::UVec2;
use bevy::prelude::{App, Assets, Commands, Deref, DerefMut, Handle, Image, Res, ResMut, Resource, Sprite, TextureAtlas, TextureAtlasLayout, Timer, TimerMode};
use std::collections::HashMap;

use crate::animation_sprite::{AnimationBundle, AnimationIndices, AnimationTimer};

pub fn sprites_plugin(app: &mut App) {
    app.add_systems(Startup, init_board_sprites);
}

fn init_board_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.insert_resource(BoardSprites::new(asset_server, texture_atlas_layouts));
}

#[derive(Resource, Deref, DerefMut)]
pub struct BoardSprites {
    bundles: HashMap<usize, AnimationBundle>
}

impl BoardSprites {

    fn new(
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>
    ) -> Self {
        let texture = asset_server.load("nodes/sprite-all-2x11.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(250), 2, 11, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let first = Self::construct_animation_bundle(texture.clone(), (0, 1), texture_atlas_layout.clone());
        let second = Self::construct_animation_bundle(texture.clone(), (2, 3), texture_atlas_layout.clone());
        let third = Self::construct_animation_bundle(texture.clone(), (4, 5), texture_atlas_layout.clone());
        let forth = Self::construct_animation_bundle(texture.clone(), (6, 7), texture_atlas_layout.clone());
        let fifth = Self::construct_animation_bundle(texture.clone(), (8, 9), texture_atlas_layout.clone());
        let sixth = Self::construct_animation_bundle(texture.clone(), (10, 11), texture_atlas_layout.clone());
        let seventh = Self::construct_animation_bundle(texture.clone(), (12, 13), texture_atlas_layout.clone());
        let eight = Self::construct_animation_bundle(texture.clone(), (14, 15), texture_atlas_layout.clone());
        let ninth = Self::construct_animation_bundle(texture.clone(), (16, 17), texture_atlas_layout.clone());
        let tenth = Self::construct_animation_bundle(texture.clone(), (18, 19), texture_atlas_layout.clone());
        let eleventh = Self::construct_animation_bundle(texture.clone(), (20, 21), texture_atlas_layout.clone());

        let mut animation_map = HashMap::with_capacity(11);
        animation_map.insert(1, first);
        animation_map.insert(2, second);
        animation_map.insert(3, third);
        animation_map.insert(4, forth);
        animation_map.insert(5, fifth);
        animation_map.insert(6, sixth);
        animation_map.insert(7, seventh);
        animation_map.insert(8, eight);
        animation_map.insert(9, ninth);
        animation_map.insert(10, tenth);
        animation_map.insert(11, eleventh);

        Self {
            bundles: animation_map
        }
    }

    fn construct_animation_bundle(
        texture: Handle<Image>,
        indices: (usize, usize),
        texture_atlas_layout: Handle<TextureAtlasLayout>
    ) -> AnimationBundle {
        AnimationBundle::new(
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: indices.0,
                }
            ),
            AnimationIndices::new(indices.0, indices.1),
            AnimationTimer::new(Timer::from_seconds(0.5, TimerMode::Repeating))
        )
    }

}