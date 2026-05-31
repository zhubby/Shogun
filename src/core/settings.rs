use bevy::prelude::Window;
use bevy_egui::egui;

use super::HUD_MARGIN;
use super::display_settings::{DisplayMode, DisplayResolution, DisplaySettings};
use super::state::GameUiState;
use super::style::{modal_title_bar, war_gold, war_panel_frame, war_text_muted};

pub(super) fn settings_modal(ctx: &egui::Context, ui_state: &mut GameUiState) -> bool {
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
    let modal_width = (screen.width() - HUD_MARGIN * 2.0).clamp(320.0, 560.0);
    egui::Area::new(egui::Id::new("settings_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(modal_width);
                if modal_title_bar(ui, "显示设置") {
                    ui_state.settings_open = false;
                }
                ui.separator();
                apply_settings |= settings_controls(ui, ui_state);
            });
        });
    apply_settings
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

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("分辨率").color(war_text_muted()));
            egui::ComboBox::from_id_salt("display_resolution")
                .selected_text(ui_state.pending_settings.resolution.to_string())
                .show_ui(ui, |ui| {
                    for resolution in DisplayResolution::presets() {
                        ui.selectable_value(
                            &mut ui_state.pending_settings.resolution,
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
                    &mut ui_state.pending_settings.display_mode,
                    *mode,
                    mode.label(),
                );
            }
        });

        ui.checkbox(&mut ui_state.pending_settings.vsync, "垂直同步");

        if ui_state.pending_settings != ui_state.applied_settings {
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
                ui_state.pending_settings = DisplaySettings::default();
                ui_state.message = "显示设置已恢复默认，点击应用并保存生效".to_string();
            }
        });
    });
    apply_settings
}

pub(super) fn apply_pending_display_settings(ui_state: &mut GameUiState, window: &mut Window) {
    let settings = ui_state.pending_settings;
    settings.apply_to_window(window);
    ui_state.applied_settings = settings;
    match ui_state.settings_store.save(settings) {
        Ok(()) => {
            ui_state.message = format!(
                "显示设置已保存到 {}",
                ui_state.settings_store.path().display()
            );
        }
        Err(error) => {
            ui_state.message = format!("显示设置已应用，但保存失败: {error}");
        }
    }
}
