use bevy_egui::egui;

use super::super::actions::open_city;
use super::super::city_intel::city_summary_intel_with_title_action;
use super::super::i18n::Translator;
use super::super::map::{reset_map_view, zoom_map};
use super::super::state::GameUiState;
use super::super::style::{square_icon_button, war_gold, war_panel_frame, war_text_muted};
use super::super::{HUD_MARGIN, HUD_TOP_HEIGHT, HUD_TOP_OFFSET, MAP_ZOOM_STEP};
pub(super) fn map_controls_hud(ctx: &egui::Context, ui_state: &mut GameUiState, t: &Translator) {
    egui::Area::new(egui::Id::new("hud_map_controls"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_TOP,
            egui::vec2(-HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(270.0);
                map_controls(ui, ui_state, t);
            });
        });
}

pub(super) fn left_city_summary_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
) {
    egui::Area::new(egui::Id::new("hud_left_city_summary"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(285.0);
                selected_city_summary(ui, ui_state, t);
            });
        });
}

pub(super) fn selected_city_summary(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    let summary = ui_state.game.as_ref().and_then(|game| {
        let city = game.cities.get(ui_state.selected_city_id.as_deref()?)?;
        let faction_name = game
            .factions
            .get(&city.faction_id)
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| t.text("unknown"));
        Some((city.id.clone(), city.clone(), faction_name))
    });

    let Some((city_id, city, faction_name)) = summary else {
        ui.label(t.text("selected-city-none"));
        return;
    };

    let mut open_requested = false;
    city_summary_intel_with_title_action(ui, &city, &faction_name, t, |ui| {
        if square_icon_button(
            ui,
            egui_phosphor::regular::GAVEL,
            t.text("open-command-tent"),
            egui::vec2(30.0, 30.0),
        )
        .clicked()
        {
            open_requested = true;
        }
    });
    if open_requested {
        open_city(ui_state, city_id);
    }
}

pub(super) fn map_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.horizontal(|ui| {
        ui.heading(egui::RichText::new(t.text("map-title")).color(war_gold()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_enabled(
                ui_state.map_boundaries.is_some(),
                egui::Checkbox::new(
                    &mut ui_state.map_boundaries_enabled,
                    t.text("map-boundaries"),
                ),
            );
        });
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button("-").clicked() {
            zoom_map(ui_state, 1.0 / MAP_ZOOM_STEP, None, None);
        }
        ui.add_sized(
            [58.0, 28.0],
            egui::Label::new(format!("{:.0}%", ui_state.map_zoom * 100.0)),
        );
        if ui.button("+").clicked() {
            zoom_map(ui_state, MAP_ZOOM_STEP, None, None);
        }
        if ui.button(t.text("common-reset")).clicked() {
            reset_map_view(ui_state);
        }
    });
    if ui_state.map_boundaries.is_none() {
        ui.colored_label(war_text_muted(), t.text("map-boundary-asset-missing"));
    }
}
