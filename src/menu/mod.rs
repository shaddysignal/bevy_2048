pub mod menu_mod;
mod splash;

use bevy::camera::{ Viewport, ScalingMode};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub fn main_menu_plugin(app: &mut App) {
    app
        .insert_resource(Volume(7))
        .init_state::<AppState>()
        .add_plugins((splash::splash_plugin, menu_mod::menu_plugin));
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Hash, Debug, States)]
pub enum AppState {
    #[default]
    Splash,
    Menu,
    Game,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Copy, Clone)]
pub struct Volume(i8);

fn setup(mut commands: Commands) { //winit_windows: NonSend<WinitWindows>, window_query: Query<Entity, With<PrimaryWindow>>) {
    let window_size = vec2(1920.0, 1080.0);
    // let monitor = window_query
    //     .get_single()
    //     .ok()
    //     .and_then(|entity| winit_windows.get_window(entity))
    //     .and_then(|winit_window| winit_window.current_monitor())
    //     .expect("Couldn't get monitor");
    //
    // let scale = height / monitor.size().height as f32;
    let scale = 1.0;

    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(Viewport {
                physical_position: (window_size * 0.0).as_uvec2(),
                physical_size: (window_size * 0.95).as_uvec2(),
                ..default()
            }),
            ..default()
        },
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax { max_width: window_size.x, max_height: window_size.y },
            scale,
            ..OrthographicProjection::default_2d()
        }),
        Msaa::Sample4,
    ));
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}