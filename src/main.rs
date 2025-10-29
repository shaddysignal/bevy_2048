mod menu;
mod game;
mod animation_sprite;

use bevy::log::LogPlugin;
use bevy::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use crate::menu::MenuPlugin;

fn main() {
    // this code is compiled only if debug assertions are enabled (debug mode)
    #[cfg(debug_assertions)]
    let log_plugin = LogPlugin {
        level: bevy::log::Level::DEBUG,
        filter: "info,wgpu_core=warn,wgpu_hal=warn,naga=warn,bevy_2048_p=debug".into(),
        custom_layer: |_| None,
        fmt_layer: |_| None,
    };

    // this code is compiled only if debug assertions are disabled (release mode)
    #[cfg(not(debug_assertions))]
    let log_plugin = LogPlugin {
        level: bevy::log::Level::INFO,
        filter: "info,wgpu_core=warn,wgpu_hal=warn".into(),
        custom_layer: |_| None,
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(log_plugin)
                .set(ImagePlugin::default_nearest()))
        .add_plugins((MenuPlugin, game::game_plugin))
        .add_plugins(animation_sprite::animate_sprite_plugin)
        .insert_resource(SharedRand::default())
        .run();
}

#[derive(Resource, Deref, DerefMut)]
struct SharedRand(ChaCha8Rng);
impl Default for SharedRand {
    fn default() -> Self {
        let rng = if cfg!(target_arch = "wasm32") {
            let mut seed = [0u8; 32];
            getrandom::fill(&mut seed).expect("failed to fill seed");
            ChaCha8Rng::from_seed(seed)
        } else {
            ChaCha8Rng::from_os_rng()
        };
        
        Self(rng)
    }
}
