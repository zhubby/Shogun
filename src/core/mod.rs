mod actions;
mod app_icon;
mod audio;
mod city_intel;
mod city_panel;
mod display_settings;
mod hud;
mod i18n;
mod labels;
mod map;
mod menu;
mod settings;
mod state;
mod style;

use bevy::prelude::*;
use bevy::window::{EnabledButtons, PrimaryWindow};
use bevy_asset_loader::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use audio::MainMenuAudio;
use display_settings::GameSettingsStore;
use hud::in_game;
use i18n::{Translator, args};
use menu::{MainMenuAction, MainMenuAssets, main_menu, prepare_main_menu_assets_for_egui};
use settings::apply_pending_game_settings;
use state::{GameUiState, Screen};
use style::{configure_egui_fonts, configure_egui_theme};

pub(super) const MAP_MIN_ZOOM: f32 = 0.65;
pub(super) const MAP_MAX_ZOOM: f32 = 5.0;
pub(super) const MAP_ZOOM_STEP: f32 = 1.2;
pub(super) const HUD_MARGIN: f32 = 16.0;
pub(super) const HUD_TOP_OFFSET: f32 = 14.0;
pub(super) const HUD_TOP_HEIGHT: f32 = 68.0;

pub fn run() {
    let settings_store = GameSettingsStore::with_default_path();
    let loaded_settings = settings_store.load();
    let initial_display_settings = loaded_settings.settings.display;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "三国争霸 Shogun".to_string(),
                resolution: initial_display_settings.window_resolution(),
                mode: initial_display_settings.window_mode(),
                present_mode: initial_display_settings.present_mode(),
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
        .add_plugins(app_icon::AppIconPlugin)
        .init_collection::<MainMenuAssets>()
        .insert_non_send_resource(MainMenuAudio::default())
        .insert_resource(GameUiState::new(settings_store, loaded_settings))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, sync_main_menu_bgm)
        .add_systems(
            EguiPrimaryContextPass,
            (prepare_main_menu_assets_for_egui, game_ui_system).chain(),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn sync_main_menu_bgm(
    mut ui_state: ResMut<GameUiState>,
    mut main_menu_audio: NonSendMut<MainMenuAudio>,
) {
    let audio_settings = ui_state.applied_settings.audio.clone();
    let t = Translator::new(ui_state.applied_settings.general.ui_language);
    match main_menu_audio.sync(
        ui_state.screen,
        ui_state.main_menu_bgm_enabled,
        &audio_settings,
    ) {
        Ok(Some(warning)) => ui_state.message = warning,
        Ok(None) => {}
        Err(error) => {
            ui_state.main_menu_bgm_enabled = false;
            ui_state.message = t.text_args("message-bgm-unavailable", &args([("error", error)]));
        }
    }
}

fn game_ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<GameUiState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut app_exit_writer: MessageWriter<AppExit>,
    mut main_menu_audio: NonSendMut<MainMenuAudio>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    configure_egui_fonts(ctx, &mut ui_state);
    configure_egui_theme(ctx);
    let t = Translator::new(ui_state.applied_settings.general.ui_language);

    match ui_state.screen {
        Screen::MainMenu => match main_menu(ctx, &mut ui_state) {
            MainMenuAction::None => {}
            MainMenuAction::ApplyGameSettings => match windows.single_mut() {
                Ok(mut window) => {
                    apply_pending_game_settings(&mut ui_state, &mut window, &mut main_menu_audio)
                }
                Err(_) => ui_state.message = t.text("message-main-window-missing"),
            },
            MainMenuAction::Exit => {
                main_menu_audio.stop();
                app_exit_writer.write(AppExit::Success);
            }
        },
        Screen::InGame => in_game(ctx, &mut ui_state),
    }
}
