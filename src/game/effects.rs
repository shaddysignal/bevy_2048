use crate::game::components::Direction;
use crate::game::{col_to_x, row_to_y};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, SpecializedMeshPipelineError};
use bevy_hanabi::prelude::*;
use std::collections::HashMap;
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};

#[derive(Resource)]
struct EffectContainer {
    pub effects: HashMap<String, Handle<EffectAsset>>,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SparksMaterials {
    #[uniform(0)]
    pub(crate) color: LinearRgba,
    #[uniform(1)]
    pub(crate) left: Vec2,
    #[uniform(2)]
    pub(crate) right: Vec2,
}

impl Material2d for SparksMaterials {
    fn fragment_shader() -> ShaderRef {
        "shaders/sparks_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BloomMaterial {
    #[uniform(0)]
    pub(crate) color: LinearRgba,
}

impl Material2d for BloomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bloom_material.wgsl".into()
    }
    
    // // Bevy assumes by default that vertex shaders use the "vertex" entry point
    // // and fragment shaders use the "fragment" entry point (for WGSL shaders).
    // // GLSL uses "main" as the entry point, so we must override the defaults here
    // fn specialize(
    //     descriptor: &mut RenderPipelineDescriptor,
    //     _layout: &MeshVertexBufferLayoutRef,
    //     _key: Material2dKey<Self>,
    // ) -> Result<(), SpecializedMeshPipelineError> {
    //     descriptor.vertex.entry_point = "main".into();
    //     descriptor.fragment.as_mut().unwrap().entry_point = "mainImage".into();
    //     Ok(())
    // }
}

pub fn effects_plugin(app: &mut App) {
    app.add_plugins((Material2dPlugin::<BloomMaterial>::default(), Material2dPlugin::<SparksMaterials>::default()))
        .insert_resource(EffectContainer {
            effects: HashMap::new(),
        })
        .add_systems(Startup, (create_wait_effect, setup_effects).chain());
}

fn produce_wait_effect(
    effect_assets: &mut ResMut<Assets<EffectAsset>>,
    gradient: &Gradient<Vec4>,
    direction: Direction,
) -> Handle<EffectAsset> {
    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(0.5).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let (col, row, shift_x, shift_y) = match direction {
        Direction::Down => (3, 0, 125., 125.),
        Direction::Up => (0, 3, -125., -125.),
        Direction::Left => (3, 3, 125., -125.),
        Direction::Right => (0, 0, -125., 125.),
    };
    let position = (writer.lit(vec3(col_to_x(col) + shift_x, row_to_y(row) + shift_y, 0.))
        + writer
            .lit(vec3(0., 0., 0.))
            .uniform(writer.lit(vec3(2.5, 2.5, 0.))))
    .expr();
    let init_pos = SetAttributeModifier::new(Attribute::POSITION, position);

    let vel_vec = match direction {
        Direction::Down => vec3(0., -800., 0.),
        Direction::Up => vec3(0., 800., 0.),
        Direction::Left => vec3(-800., 0., 0.),
        Direction::Right => vec3(800., 0., 0.),
    };
    let vel = writer.lit(vel_vec).expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel);

    let mut module = writer.finish();

    let round = RoundModifier::constant(&mut module, 2.0 / 3.0);

    // Create a new effect asset spawning 30 particles per second from a circle
    // and slowly fading from blue-ish to transparent over their lifetime.
    // By default, the asset spawns the particles at Z=0.
    let spawner = Spawner::rate(360.0.into()).with_starts_active(false);
    effect_assets.add(
        EffectAsset::new(2048, spawner, module)
            .with_name(format!("wait_effect_{:?}", direction))
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::splat(10.)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier {
                gradient: gradient.clone(),
            })
            .render(round),
    )
}

fn create_wait_effect(
    mut effect_container: ResMut<EffectContainer>,
    mut effect_assets: ResMut<Assets<EffectAsset>>,
) {
    // Define a color gradient from red to transparent black
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1., 0., 0., 1.));
    gradient.add_key(1.0, Vec4::splat(0.1));

    {
        let effect = produce_wait_effect(&mut effect_assets, &gradient, Direction::Right);
        effect_container.effects.insert("wait_right".into(), effect);
    }

    {
        let effect = produce_wait_effect(&mut effect_assets, &gradient, Direction::Left);
        effect_container.effects.insert("wait_left".into(), effect);
    }

    {
        let effect = produce_wait_effect(&mut effect_assets, &gradient, Direction::Down);
        effect_container.effects.insert("wait_down".into(), effect);
    }

    {
        let effect = produce_wait_effect(&mut effect_assets, &gradient, Direction::Up);
        effect_container.effects.insert("wait_up".into(), effect);
    }
}

fn create_merge_effects(
    mut effect_container: ResMut<EffectContainer>,
    mut effect_assets: ResMut<Assets<EffectAsset>>,
    edge: (Vec2, Vec2),
) {
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(1., 1., 1., 1.));
    color_gradient.add_key(0.3, Vec4::new(1., 1., 0., 1.));
    color_gradient.add_key(0.6, Vec4::new(1., 0., 0., 0.2));
    color_gradient.add_key(1.0, Vec4::splat(0.));

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(4.));
    size_gradient.add_key(0.3, Vec3::splat(6.));
    size_gradient.add_key(1.0, Vec3::splat(0.));

    let writer = ExprWriter::new();

    // Give a bit of variation by randomizing the age per particle. This will
    // control the starting color and starting size of particles.
    let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.8).normal(writer.lit(1.2)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add constant downward acceleration to simulate gravity
    let accel = writer.lit(Vec3::Y * -16.).expr();
    let update_accel = AccelModifier::new(accel);

    // Add drag to make particles slow down a bit after the initial explosion
    let drag = writer.lit(4.).expr();
    let update_drag = LinearDragModifier::new(drag);

    let (edge_start, edge_end) = edge;
    let edge_line = edge_start - edge_end;
    let edge_middle = edge_line / 2. + edge_end;
    let pos = writer
        .lit(vec3(edge_middle.x, edge_middle.y, 0.))
        .uniform(writer.lit(vec3(edge_line.x, edge_line.y, 0.)))
        .expr();
    let init_pos = SetAttributeModifier::new(Attribute::POSITION, pos);

    // Give a bit of variation by randomizing the initial speed
    let speed = writer
        .lit(vec3(60., 60., 0.))
        .uniform(writer.lit(vec3(20., 20., 0.)))
        .expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, speed);

    // Clear the trail velocity so trail particles just stay in place as they fade
    // away
    let init_vel_trail =
        SetAttributeModifier::new(Attribute::VELOCITY, writer.lit(Vec3::ZERO).expr());

    let lead = ParticleGroupSet::single(0);
    let trail = ParticleGroupSet::single(1);

    let effect = EffectAsset::new(
        // 2k lead particles, with 32 trail particles each
        2048,
        Spawner::rate(2048.0.into()),
        writer.finish(),
    )
    // Tie together trail particles to make arcs. This way we don't need a lot of them, yet there's
    // a continuity between them.
    .with_ribbons(2048 * 32, 1.0 / 64.0, 0.2, 0)
    .with_name("block_merge")
    .init_groups(init_pos, lead)
    .init_groups(init_vel, lead)
    .init_groups(init_age, lead)
    .init_groups(init_lifetime, lead)
    .init_groups(init_vel_trail, trail)
    .update_groups(update_drag, lead)
    .update_groups(update_accel, lead)
    .render_groups(
        ColorOverLifetimeModifier {
            gradient: color_gradient.clone(),
        },
        lead,
    )
    .render_groups(
        SizeOverLifetimeModifier {
            gradient: size_gradient.clone(),
            screen_space_size: false,
        },
        lead,
    )
    .render_groups(
        ColorOverLifetimeModifier {
            gradient: color_gradient,
        },
        trail,
    )
    .render_groups(
        SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        },
        trail,
    );

    let sparks_effect = effect_assets.add(effect);
    effect_container
        .effects
        .insert("sparks".into(), sparks_effect);
}

fn setup_effects(mut commands: Commands, effect_container: ResMut<EffectContainer>) {
    commands.spawn_batch([
        (
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect_container.effects["wait_up"].clone())
                    .with_z_layer_2d(Some(0.1)),
                ..default()
            },
            Name::new("wait_group"),
        ),
        (
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect_container.effects["wait_down"].clone())
                    .with_z_layer_2d(Some(0.1)),
                ..default()
            },
            Name::new("wait_group"),
        ),
        (
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect_container.effects["wait_left"].clone())
                    .with_z_layer_2d(Some(0.1)),
                ..default()
            },
            Name::new("wait_group"),
        ),
        (
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect_container.effects["wait_right"].clone())
                    .with_z_layer_2d(Some(0.1)),
                ..default()
            },
            Name::new("wait_group"),
        ),
    ]);
}
