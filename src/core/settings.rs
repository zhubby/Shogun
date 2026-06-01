use bevy::prelude::Window;
use bevy_egui::egui;

use super::HUD_MARGIN;
use super::audio::{MainMenuAudio, available_output_device_names};
use super::display_settings::{
    DisplayMode, DisplayResolution, GameSettings, normalize_master_volume,
};
use super::i18n::{Translator, UiLanguage, args};
use super::state::{GameUiState, SettingsTab};
use super::style::{modal_title_bar, war_gold, war_panel_frame, war_text_muted};

pub(super) fn settings_modal(ctx: &egui::Context, ui_state: &mut GameUiState) -> bool {
    let t = Translator::new(ui_state.pending_settings.general.ui_language);
    if !ui_state.audio_output_devices_refresh_attempted {
        refresh_audio_output_devices(ui_state);
    }

    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new("settings_modal_scrim"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120),
            );
            if response.clicked() {
                ui_state.settings_open = false;
            }
        });

    let mut apply_settings = false;
    let modal_width = (screen.width() - HUD_MARGIN * 2.0).clamp(340.0, 620.0);
    egui::Area::new(egui::Id::new("settings_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(modal_width);
                if modal_title_bar(ui, &t, &t.text("settings-title")) {
                    ui_state.settings_open = false;
                }
                ui.separator();
                apply_settings |= settings_controls(ui, ui_state, &t);
            });
        });
    apply_settings
}

pub(super) fn refresh_audio_output_devices(ui_state: &mut GameUiState) {
    ui_state.audio_output_devices_refresh_attempted = true;
    match available_output_device_names() {
        Ok(devices) => {
            ui_state.audio_output_devices = devices;
            ui_state.audio_output_devices_error = None;
        }
        Err(error) => {
            ui_state.audio_output_devices.clear();
            ui_state.audio_output_devices_error = Some(error);
        }
    }
}

pub(super) fn settings_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
) -> bool {
    let mut apply_settings = false;
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(t.text_args(
                "settings-config-path",
                &args([("path", ui_state.settings_store.path().display().to_string())]),
            ))
            .color(war_text_muted()),
        );
        ui.add_space(8.0);

        settings_tabs(ui, ui_state, t);
        ui.add_space(10.0);

        match ui_state.settings_tab {
            SettingsTab::Display => display_settings_controls(ui, ui_state, t),
            SettingsTab::Audio => audio_settings_controls(ui, ui_state, t),
            SettingsTab::Language => language_settings_controls(ui, ui_state, t),
        }

        if ui_state.pending_settings != ui_state.applied_settings {
            ui.add_space(8.0);
            ui.colored_label(war_gold(), t.text("settings-unsaved-changes"));
        }

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_sized(
                    [132.0, 34.0],
                    egui::Button::new(t.text("settings-apply-save")),
                )
                .clicked()
            {
                apply_settings = true;
                ui_state.settings_open = false;
            }
            if ui.button(t.text("settings-restore-defaults")).clicked() {
                ui_state.pending_settings = GameSettings::default();
                ui_state.message = t.text("message-settings-restored");
            }
        });
    });
    apply_settings
}

fn settings_tabs(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut ui_state.settings_tab,
            SettingsTab::Display,
            t.text("settings-tab-display"),
        );
        ui.selectable_value(
            &mut ui_state.settings_tab,
            SettingsTab::Audio,
            t.text("settings-tab-audio"),
        );
        ui.selectable_value(
            &mut ui_state.settings_tab,
            SettingsTab::Language,
            t.text("settings-tab-language"),
        );
    });
}

fn display_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(t.text("settings-display-resolution")).color(war_text_muted()),
            );
            egui::ComboBox::from_id_salt("display_resolution")
                .selected_text(ui_state.pending_settings.display.resolution.to_string())
                .show_ui(ui, |ui| {
                    for resolution in DisplayResolution::presets() {
                        ui.selectable_value(
                            &mut ui_state.pending_settings.display.resolution,
                            *resolution,
                            resolution.to_string(),
                        );
                    }
                });
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(t.text("settings-display-mode")).color(war_text_muted()));
            for mode in DisplayMode::variants() {
                ui.radio_value(
                    &mut ui_state.pending_settings.display.display_mode,
                    *mode,
                    mode.label(t),
                );
            }
        });

        ui.checkbox(
            &mut ui_state.pending_settings.display.vsync,
            t.text("settings-vsync"),
        );
    });
}

fn audio_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(t.text("settings-master-volume")).color(war_text_muted()));
            let mut volume = ui_state.pending_settings.audio.master_volume;
            let changed = ui
                .add(
                    egui::Slider::new(&mut volume, 0.0..=1.0)
                        .show_value(false)
                        .fixed_decimals(0),
                )
                .changed();
            if changed {
                ui_state.pending_settings.audio.master_volume = normalize_master_volume(volume);
            }
            ui.label(format!(
                "{}%",
                (ui_state.pending_settings.audio.master_volume * 100.0).round() as u32
            ));
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(t.text("settings-output-device")).color(war_text_muted()));
            let selected_text = ui_state
                .pending_settings
                .audio
                .output_device_name
                .clone()
                .unwrap_or_else(|| t.text("settings-system-default"));
            let device_names = ui_state.audio_output_devices.clone();
            egui::ComboBox::from_id_salt("audio_output_device")
                .width(260.0)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut ui_state.pending_settings.audio.output_device_name,
                        None,
                        t.text("settings-system-default"),
                    );
                    for device_name in device_names {
                        ui.selectable_value(
                            &mut ui_state.pending_settings.audio.output_device_name,
                            Some(device_name.clone()),
                            &device_name,
                        );
                    }
                });

            if ui.button(t.text("settings-refresh")).clicked() {
                refresh_audio_output_devices(ui_state);
            }
        });

        if let Some(error) = &ui_state.audio_output_devices_error {
            ui.colored_label(war_gold(), error);
        } else if ui_state.audio_output_devices.is_empty() {
            ui.label(
                egui::RichText::new(t.text("settings-no-extra-output-devices"))
                    .color(war_text_muted()),
            );
        }
    });
}

fn language_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(t.text("settings-ui-language")).color(war_text_muted()));
        egui::ComboBox::from_id_salt("ui_language")
            .width(180.0)
            .selected_text(ui_state.pending_settings.general.ui_language.label())
            .show_ui(ui, |ui| {
                for language in UiLanguage::available() {
                    ui.selectable_value(
                        &mut ui_state.pending_settings.general.ui_language,
                        *language,
                        language.label(),
                    );
                }
            });
    });
    ui.colored_label(war_text_muted(), t.text("settings-language-apply-hint"));
}

pub(super) fn apply_pending_game_settings(
    ui_state: &mut GameUiState,
    window: &mut Window,
    main_menu_audio: &mut MainMenuAudio,
) {
    let settings = match ui_state.pending_settings.clone().validated() {
        Ok(settings) => settings,
        Err(error) => {
            let t = Translator::new(ui_state.pending_settings.general.ui_language);
            ui_state.message = t.text_args(
                "message-settings-invalid",
                &args([("error", error.to_string())]),
            );
            return;
        }
    };
    let t = Translator::new(settings.general.ui_language);
    settings.display.apply_to_window(window);
    ui_state.pending_settings = settings.clone();
    ui_state.applied_settings = settings.clone();

    let audio_message = match main_menu_audio.sync(
        ui_state.screen,
        ui_state.main_menu_bgm_enabled,
        &settings.audio,
    ) {
        Ok(Some(warning)) => Some(warning),
        Ok(None) => None,
        Err(error) => Some(t.text_args("message-bgm-unavailable", &args([("error", error)]))),
    };

    match ui_state.settings_store.save(settings) {
        Ok(()) => {
            ui_state.message = t.text_args(
                "message-settings-saved",
                &args([("path", ui_state.settings_store.path().display().to_string())]),
            );
        }
        Err(error) => {
            ui_state.message = t.text_args(
                "message-settings-save-failed",
                &args([("error", error.to_string())]),
            );
        }
    }

    if let Some(audio_message) = audio_message {
        if !ui_state.message.is_empty() {
            ui_state.message.push('\n');
        }
        ui_state.message.push_str(&audio_message);
    }
}
