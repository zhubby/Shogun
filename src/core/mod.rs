mod actions;
mod city_panel;
mod display_settings;
mod hud;
mod labels;
mod map;
mod menu;
mod settings;
mod state;
mod style;

use bevy::prelude::*;
use bevy::window::{EnabledButtons, PrimaryWindow};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use display_settings::DisplaySettingsStore;
use hud::in_game;
use menu::main_menu;
use settings::apply_pending_display_settings;
use state::{GameUiState, Screen};
use style::{configure_egui_fonts, configure_egui_theme};

pub(super) const MAP_MIN_ZOOM: f32 = 0.65;
pub(super) const MAP_MAX_ZOOM: f32 = 5.0;
pub(super) const MAP_ZOOM_STEP: f32 = 1.2;
pub(super) const HUD_MARGIN: f32 = 16.0;
pub(super) const HUD_TOP_OFFSET: f32 = 14.0;
pub(super) const HUD_TOP_HEIGHT: f32 = 68.0;
pub(super) const CITY_DRAWER_WIDTH: f32 = 390.0;

pub fn run() {
    let settings_store = DisplaySettingsStore::with_default_path();
    let loaded_settings = settings_store.load();
    let initial_settings = loaded_settings.settings;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "大将军 Shogun".to_string(),
                resolution: initial_settings.window_resolution(),
                mode: initial_settings.window_mode(),
                present_mode: initial_settings.present_mode(),
                resizable: false,
                enabled_buttons: EnabledButtons {
                    maximize: false,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(GameUiState::new(settings_store, loaded_settings))
        .add_systems(Startup, setup_camera)
        .add_systems(EguiPrimaryContextPass, game_ui_system)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn game_ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<GameUiState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    configure_egui_fonts(ctx, &mut ui_state);
    configure_egui_theme(ctx);

    match ui_state.screen {
        Screen::MainMenu => {
            if main_menu(ctx, &mut ui_state) {
                match windows.single_mut() {
                    Ok(mut window) => apply_pending_display_settings(&mut ui_state, &mut window),
                    Err(_) => ui_state.message = "找不到主窗口，无法应用显示设置".to_string(),
                }
            }
        }
        Screen::InGame => in_game(ctx, &mut ui_state),
    }
}
