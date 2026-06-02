mod actions;
mod app_icon;
mod audio;
mod city_intel;
mod city_panel;
mod hud;
mod i18n;
mod labels;
mod map;
mod menu;
mod officer_portrait_ui;
mod portraits;
mod runtime;
mod settings;
mod shortcuts;
mod state;
mod style;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::window::{EnabledButtons, PrimaryWindow};
use bevy_asset_loader::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use std::path::{Path, PathBuf};

use audio::MainMenuAudio;
use hud::in_game;
use i18n::{Translator, args};
use menu::{MainMenuAction, MainMenuAssets, main_menu, prepare_main_menu_assets_for_egui};
use runtime::CoreAsyncRuntime;
use settings::{GameSettingsStore, apply_pending_game_settings, settings_modal};
use shortcuts::handle_shortcut_input;
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
    let asset_dir = runtime_assets_dir();
    let core_async_runtime =
        CoreAsyncRuntime::new().expect("failed to initialize core async runtime");

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_dir.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
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
                }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(app_icon::AppIconPlugin)
        .init_collection::<MainMenuAssets>()
        .insert_non_send_resource(MainMenuAudio::default())
        .insert_resource(core_async_runtime)
        .insert_resource(GameUiState::new(settings_store, loaded_settings))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, sync_main_menu_bgm)
        .add_systems(
            EguiPrimaryContextPass,
            (prepare_main_menu_assets_for_egui, game_ui_system).chain(),
        )
        .run();
}

pub(super) fn asset_path(path: impl AsRef<Path>) -> PathBuf {
    runtime_assets_dir().join(path)
}

pub(super) fn runtime_assets_dir() -> PathBuf {
    asset_dir_candidates()
        .into_iter()
        .find_map(existing_directory)
        .unwrap_or_else(|| PathBuf::from("assets"))
}

fn asset_dir_candidates() -> Vec<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    let cwd = std::env::current_dir().ok();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok().map(PathBuf::from);

    asset_dir_candidates_from(exe_dir.as_deref(), cwd.as_deref(), manifest_dir.as_deref())
}

fn asset_dir_candidates_from(
    exe_dir: Option<&Path>,
    cwd: Option<&Path>,
    manifest_dir: Option<&Path>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(exe_dir) = exe_dir {
        candidates.push(exe_dir.join("../Resources/assets"));
        candidates.push(exe_dir.join("assets"));
        candidates.push(
            exe_dir
                .join("../../share")
                .join(env!("CARGO_PKG_NAME"))
                .join("assets"),
        );
    }

    if let Some(cwd) = cwd {
        candidates.push(cwd.join("assets"));
    }

    if let Some(manifest_dir) = manifest_dir {
        candidates.push(manifest_dir.join("assets"));
    }

    candidates
}

fn existing_directory(path: PathBuf) -> Option<PathBuf> {
    if !path.is_dir() {
        return None;
    }
    Some(path.canonicalize().unwrap_or(path))
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
    keyboard_input: Res<ButtonInput<KeyCode>>,
    async_runtime: Res<CoreAsyncRuntime>,
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
    handle_shortcut_input(ctx, &keyboard_input, &mut ui_state);
    let t = Translator::new(ui_state.applied_settings.general.ui_language);

    match ui_state.screen {
        Screen::MainMenu => match main_menu(ctx, &mut ui_state, async_runtime.as_ref()) {
            MainMenuAction::None => {}
            MainMenuAction::Exit => {
                main_menu_audio.stop();
                app_exit_writer.write(AppExit::Success);
            }
        },
        Screen::InGame => in_game(ctx, &mut ui_state, async_runtime.as_ref()),
    }

    if ui_state.settings_open && settings_modal(ctx, &mut ui_state) {
        match windows.single_mut() {
            Ok(mut window) => {
                apply_pending_game_settings(&mut ui_state, &mut window, &mut main_menu_audio)
            }
            Err(_) => ui_state.message = t.text("message-main-window-missing"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_assets_dir_resolves_project_assets() {
        assert!(runtime_assets_dir().join("icons/banner_logo.png").is_file());
    }

    #[test]
    fn asset_dir_candidates_cover_packaged_layouts() {
        let candidates = asset_dir_candidates_from(
            Some(Path::new("/opt/Shogun.app/Contents/MacOS")),
            Some(Path::new("/opt/shogun/share/shogun")),
            Some(Path::new("/repo/Shogun")),
        );

        assert!(candidates.contains(&PathBuf::from(
            "/opt/Shogun.app/Contents/MacOS/../Resources/assets"
        )));
        assert!(candidates.contains(&PathBuf::from("/opt/Shogun.app/Contents/MacOS/assets")));
        assert!(candidates.contains(&PathBuf::from(
            "/opt/Shogun.app/Contents/MacOS/../../share/shogun/assets"
        )));
        assert!(candidates.contains(&PathBuf::from("/opt/shogun/share/shogun/assets")));
        assert!(candidates.contains(&PathBuf::from("/repo/Shogun/assets")));
    }
}
