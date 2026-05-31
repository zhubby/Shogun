mod actions;
mod app_icon;
mod city_intel;
mod city_panel;
mod display_settings;
mod hud;
mod labels;
mod map;
mod menu;
mod settings;
mod state;
mod style;

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::prelude::*;
use bevy::window::{EnabledButtons, PrimaryWindow};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use display_settings::DisplaySettingsStore;
use hud::in_game;
use menu::{MainMenuAction, main_menu};
use settings::apply_pending_display_settings;
use state::{GameUiState, Screen};
use std::sync::Arc;
use style::{configure_egui_fonts, configure_egui_theme};

pub(super) const MAP_MIN_ZOOM: f32 = 0.65;
pub(super) const MAP_MAX_ZOOM: f32 = 5.0;
pub(super) const MAP_ZOOM_STEP: f32 = 1.2;
pub(super) const HUD_MARGIN: f32 = 16.0;
pub(super) const HUD_TOP_OFFSET: f32 = 14.0;
pub(super) const HUD_TOP_HEIGHT: f32 = 68.0;
const MAIN_MENU_BGM_BYTES: &[u8] = include_bytes!("../../assets/audio/bgm.mp3");

pub fn run() {
    let settings_store = DisplaySettingsStore::with_default_path();
    let loaded_settings = settings_store.load();
    let initial_settings = loaded_settings.settings;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "三国争霸 Shogun".to_string(),
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
        .add_plugins(app_icon::AppIconPlugin)
        .insert_resource(GameUiState::new(settings_store, loaded_settings))
        .add_systems(Startup, (setup_camera, setup_main_menu_bgm_asset))
        .add_systems(Update, sync_main_menu_bgm)
        .add_systems(EguiPrimaryContextPass, game_ui_system)
        .run();
}

#[derive(Component)]
struct MainMenuBgm;

#[derive(Resource)]
struct MainMenuBgmAsset {
    handle: Handle<AudioSource>,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_main_menu_bgm_asset(
    mut commands: Commands,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    let handle = audio_sources.add(AudioSource {
        bytes: Arc::from(MAIN_MENU_BGM_BYTES),
    });
    commands.insert_resource(MainMenuBgmAsset { handle });
}

fn sync_main_menu_bgm(
    mut commands: Commands,
    bgm_asset: Res<MainMenuBgmAsset>,
    ui_state: Res<GameUiState>,
    bgm_entities: Query<Entity, With<MainMenuBgm>>,
) {
    let should_play = ui_state.screen == Screen::MainMenu && ui_state.main_menu_bgm_enabled;
    let mut bgm_iter = bgm_entities.iter();
    let active_bgm = bgm_iter.next();

    for entity in bgm_iter {
        commands.entity(entity).despawn();
    }

    if should_play {
        if active_bgm.is_none() {
            commands.spawn((
                AudioPlayer::new(bgm_asset.handle.clone()),
                PlaybackSettings::LOOP,
                MainMenuBgm,
            ));
        }
    } else if let Some(entity) = active_bgm {
        commands.entity(entity).despawn();
    }
}

fn game_ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<GameUiState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut app_exit_writer: MessageWriter<AppExit>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    configure_egui_fonts(ctx, &mut ui_state);
    configure_egui_theme(ctx);

    match ui_state.screen {
        Screen::MainMenu => match main_menu(ctx, &mut ui_state) {
            MainMenuAction::None => {}
            MainMenuAction::ApplyDisplaySettings => match windows.single_mut() {
                Ok(mut window) => apply_pending_display_settings(&mut ui_state, &mut window),
                Err(_) => ui_state.message = "找不到主窗口，无法应用显示设置".to_string(),
            },
            MainMenuAction::Exit => {
                app_exit_writer.write(AppExit::Success);
            }
        },
        Screen::InGame => in_game(ctx, &mut ui_state),
    }
}
