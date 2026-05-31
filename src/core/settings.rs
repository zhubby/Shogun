use bevy::prelude::Window;
use bevy_egui::egui;

use super::HUD_MARGIN;
use super::audio::{MainMenuAudio, available_output_device_names};
use super::display_settings::{
    DisplayMode, DisplayResolution, GameSettings, normalize_master_volume,
};
use super::state::{GameUiState, SettingsTab};
use super::style::{modal_title_bar, war_gold, war_panel_frame, war_text_muted};

pub(super) fn settings_modal(ctx: &egui::Context, ui_state: &mut GameUiState) -> bool {
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
                if modal_title_bar(ui, "游戏设置") {
                    ui_state.settings_open = false;
                }
                ui.separator();
                apply_settings |= settings_controls(ui, ui_state);
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

pub(super) fn settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) -> bool {
    let mut apply_settings = false;
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(format!(
                "配置: {}",
                ui_state.settings_store.path().display()
            ))
            .color(war_text_muted()),
        );
        ui.add_space(8.0);

        settings_tabs(ui, ui_state);
        ui.add_space(10.0);

        match ui_state.settings_tab {
            SettingsTab::Display => display_settings_controls(ui, ui_state),
            SettingsTab::Audio => audio_settings_controls(ui, ui_state),
        }

        if ui_state.pending_settings != ui_state.applied_settings {
            ui.add_space(8.0);
            ui.colored_label(war_gold(), "有未应用更改");
        }

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_sized([132.0, 34.0], egui::Button::new("应用并保存"))
                .clicked()
            {
                apply_settings = true;
                ui_state.settings_open = false;
            }
            if ui.button("恢复默认").clicked() {
                ui_state.pending_settings = GameSettings::default();
                ui_state.message = "游戏设置已恢复默认，点击应用并保存生效".to_string();
            }
        });
    });
    apply_settings
}

fn settings_tabs(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.horizontal(|ui| {
        ui.selectable_value(&mut ui_state.settings_tab, SettingsTab::Display, "图像");
        ui.selectable_value(&mut ui_state.settings_tab, SettingsTab::Audio, "声音");
    });
}

fn display_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("分辨率").color(war_text_muted()));
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
            ui.label(egui::RichText::new("显示模式").color(war_text_muted()));
            for mode in DisplayMode::variants() {
                ui.radio_value(
                    &mut ui_state.pending_settings.display.display_mode,
                    *mode,
                    mode.label(),
                );
            }
        });

        ui.checkbox(&mut ui_state.pending_settings.display.vsync, "垂直同步");
    });
}

fn audio_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("系统音量").color(war_text_muted()));
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
            ui.label(egui::RichText::new("输出设备").color(war_text_muted()));
            let selected_text = ui_state
                .pending_settings
                .audio
                .output_device_name
                .as_deref()
                .unwrap_or("系统默认")
                .to_string();
            let device_names = ui_state.audio_output_devices.clone();
            egui::ComboBox::from_id_salt("audio_output_device")
                .width(260.0)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut ui_state.pending_settings.audio.output_device_name,
                        None,
                        "系统默认",
                    );
                    for device_name in device_names {
                        ui.selectable_value(
                            &mut ui_state.pending_settings.audio.output_device_name,
                            Some(device_name.clone()),
                            &device_name,
                        );
                    }
                });

            if ui.button("刷新").clicked() {
                refresh_audio_output_devices(ui_state);
            }
        });

        if let Some(error) = &ui_state.audio_output_devices_error {
            ui.colored_label(war_gold(), error);
        } else if ui_state.audio_output_devices.is_empty() {
            ui.label(egui::RichText::new("未检测到额外输出设备").color(war_text_muted()));
        }
    });
}

pub(super) fn apply_pending_game_settings(
    ui_state: &mut GameUiState,
    window: &mut Window,
    main_menu_audio: &mut MainMenuAudio,
) {
    let settings = match ui_state.pending_settings.clone().validated() {
        Ok(settings) => settings,
        Err(error) => {
            ui_state.message = format!("游戏设置无效: {error}");
            return;
        }
    };
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
        Err(error) => Some(format!("背景音乐不可用: {error}")),
    };

    match ui_state.settings_store.save(settings) {
        Ok(()) => {
            ui_state.message = format!(
                "游戏设置已保存到 {}",
                ui_state.settings_store.path().display()
            );
        }
        Err(error) => {
            ui_state.message = format!("游戏设置已应用，但保存失败: {error}");
        }
    }

    if let Some(audio_message) = audio_message {
        if !ui_state.message.is_empty() {
            ui_state.message.push('\n');
        }
        ui_state.message.push_str(&audio_message);
    }
}
