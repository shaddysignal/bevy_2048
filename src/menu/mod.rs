pub mod menu_mod;
mod splash;

use bevy::prelude::*;

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

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum MenuState {
    Main,
    Settings,
    SettingsSound,
    #[default]
    Disabled,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Copy, Clone)]
pub struct Volume(i8);

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}