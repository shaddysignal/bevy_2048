use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

/// Marker for effects on game screen
#[derive(Component)]
pub struct EffectMarker;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SparksMaterial {
    #[uniform(0)]
    pub(crate) color: LinearRgba,
    #[uniform(1)]
    pub(crate) left: Vec2,
    #[uniform(2)]
    pub(crate) right: Vec2,
    #[uniform(5)]
    pub(crate) mesh_size: Vec4,
}

impl Material2d for SparksMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sparks_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

pub fn effects_plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<SparksMaterial>::default());
}
