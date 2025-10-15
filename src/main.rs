mod animation_sprite;
mod game;
mod menu;

use crate::game::effects;
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy_hanabi::prelude::*;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

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
        fmt_layer: |_| None,
    };

    let default_plugins = DefaultPlugins
        .set(log_plugin)
        .set(ImagePlugin::default_nearest());

    App::new()
        .add_plugins((
            default_plugins,
            HanabiPlugin,
            menu::main_menu_plugin,
            game::game_plugin,
            animation_sprite::animate_sprite_plugin,
            effects::effects_plugin,
        ))
        .add_systems(Startup, camera_setup)
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

fn camera_setup(
    mut commands: Commands,
    winit_windows: NonSend<WinitWindows>,
    window_query: Query<Entity, With<PrimaryWindow>>,
) {
    let (width, height) = (1920.0, 1080.0);
    let scale = if cfg!(target_arch = "wasm32") {
        1.0
    } else {
        let monitor = window_query
            .get_single()
            .ok()
            .and_then(|entity| winit_windows.get_window(entity))
            .and_then(|winit_window| winit_window.current_monitor())
            .expect("Couldn't get monitor");

        height / monitor.size().height as f32
    };

    commands.spawn((
        Camera2d,
        Camera {
            hdr: true,
            ..default()
        },
        OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax {
                max_width: width,
                max_height: height,
            },
            scale,
            ..OrthographicProjection::default_2d()
        },
        Tonemapping::TonyMcMapface,
        Msaa::Sample4,
        Bloom::default(),
    ));
}

// #[cfg(target_family = "wasm")]
// mod wasm_workaround {
//     extern "C" {
//         pub(super) fn __wasm_call_ctors();
//     }
// }
// 
// #[wasm_bindgen(start)]
// fn start() {
// 
//     // fix:
//     // freestyle::block::_::__ctor::h5e2299a836106c67:: Read a negative address value from the stack. Did we run out of memory?
//     #[cfg(target_family = "wasm")]
//     unsafe { wasm_workaround::__wasm_call_ctors()};
// 
// }
