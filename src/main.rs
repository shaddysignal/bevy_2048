mod animation_sprite;
mod game;
mod menu;

use bevy::camera::Viewport;
use bevy::camera::ScalingMode;
use crate::game::effects;
use bevy::post_process::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::view::Hdr;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use crate::menu::{AppState, MenuState};
use crate::menu::menu_mod::MenuButtonAction;

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
            menu::main_menu_plugin,
            game::game_plugin,
            animation_sprite::animate_sprite_plugin,
            effects::effects_plugin,
        ))
        .add_systems(Startup, camera_setup)
        .insert_resource(SharedRand::default())
        .add_systems(Update, menu_action)
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
    window: Single<&Window>,
) {
    let window_size = Vec2::new(1920., 1080.);
    let scale = window_size.y / window.resolution.physical_size().y as f32;

    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(Viewport {
                physical_position: UVec2::ZERO,
                physical_size: window_size.as_uvec2(),
                ..default()
            }),
            ..default()
        },
        Hdr,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax { max_width: window_size.x, max_height: window_size.y },
            scale,
            ..OrthographicProjection::default_2d()
        }),
        Tonemapping::TonyMcMapface,
        Msaa::Sample4,
        Bloom::default(),
    ));
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: MessageWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.write(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    game_state.set(AppState::Game);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                MenuButtonAction::SettingsSound => {
                    menu_state.set(MenuState::SettingsSound);
                }
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                MenuButtonAction::BackToSettings => {
                    menu_state.set(MenuState::Settings);
                }
            }
        }
    }
}