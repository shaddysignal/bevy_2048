use crate::game::effects::{EffectMarker, SparksMaterial};
use bevy::prelude::*;
use crate::game::components::*;
use crate::game::RECT_SIZE;

pub fn merge_effect_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SparksMaterial>>,
    mut merge_effect_messages: MessageReader<MergeEffectMessage>,
    effects_query: Query<(&ChildOf, &MeshMaterial2d<SparksMaterial>), With<EffectMarker>>,
) {
    for merge_effect_message in merge_effect_messages.read() {
        let merge_effect = effects_query.iter().find(|(e, _)| { merge_effect_message.entity.eq(&e.0) });

        if let Some(merge_effect) = merge_effect {
            let Some(material) = materials.get_mut(merge_effect.1) else { panic!("Materials should contain created material") };
            material.left = vec4(merge_effect_message.line.0.x, merge_effect_message.line.0.y, 0., 0.);
            material.right = vec4(merge_effect_message.line.1.x, merge_effect_message.line.1.y, 0., 0.);
        } else {
            let mut entity_commands = commands.entity(merge_effect_message.entity);
            let mesh_size = RECT_SIZE * 1.25;

            entity_commands.with_child((
                Mesh2d(meshes.add(Rectangle::from_size(Vec2::new(mesh_size, mesh_size)))),
                MeshMaterial2d(materials.add(SparksMaterial {
                    color: LinearRgba::BLUE,
                    left: vec4(merge_effect_message.line.0.x, merge_effect_message.line.0.y, 0., 0.),
                    right: vec4(merge_effect_message.line.1.x, merge_effect_message.line.1.y, 0., 0.),
                    mesh_size: vec4(mesh_size, mesh_size, 0., 0.),
                })),
                Transform::from_xyz(0., 0., 11.),
                EffectMarker,
            ));
        }
    }
}
