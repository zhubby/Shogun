use bevy_egui::egui;
use egui_extras::{Size, StripBuilder};

use super::super::HUD_MARGIN;
use super::super::actions::open_city;
use super::super::city_intel::city_summary_intel;
use super::super::city_panel::selected_city_panel;
use super::super::i18n::{Translator, args};
use super::super::map::{draw_city_marker_icon, faction_color};
use super::super::state::GameUiState;
use super::super::style::{
    modal_title_bar, war_border, war_gold, war_panel_frame, war_sub_panel_frame, war_text,
    war_text_muted,
};
pub(super) fn city_list_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.city_list_open {
        return;
    }

    egui::Area::new(egui::Id::new("hud_city_list_scrim"))
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
                ui_state.city_list_open = false;
            }
        });

    let modal_width = (screen.width() * 0.64)
        .clamp(520.0, 860.0)
        .min((screen.width() - HUD_MARGIN * 2.0).max(360.0));
    let modal_height = (screen.height() * 0.72)
        .clamp(420.0, 640.0)
        .min((screen.height() - HUD_MARGIN * 2.0).max(320.0));
    egui::Area::new(egui::Id::new("hud_city_list"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(modal_width);
                ui.set_min_height(modal_height);
                if modal_title_bar(ui, t, &t.text("city-list-title")) {
                    ui_state.city_list_open = false;
                }
                ui.separator();
                city_list(ui, ui_state, t, modal_width, modal_height);
            });
        });
}

pub(super) fn city_drawer_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.city_drawer_open {
        return;
    }
    let modal_width = (screen.width() - HUD_MARGIN * 2.0).clamp(760.0, 1120.0);
    let modal_height = (screen.height() - HUD_MARGIN * 2.0).clamp(520.0, 760.0);
    egui::Area::new(egui::Id::new("hud_city_drawer"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(modal_width);
                ui.set_min_height(modal_height);
                if modal_title_bar(ui, t, &t.text("command-tent-title")) {
                    ui_state.city_drawer_open = false;
                }
                ui.separator();
                ui.allocate_ui_with_layout(
                    egui::vec2(modal_width, modal_height - 54.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        selected_city_panel(ui, ui_state, t);
                    },
                );
            });
        });
}

pub(super) fn city_list(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    width: f32,
    height: f32,
) {
    let Some(game) = &ui_state.game else {
        return;
    };
    let mut rows: Vec<_> = game
        .cities
        .values()
        .map(|city| {
            let faction = game.factions.get(&city.faction_id);
            let faction_name = faction
                .map(|faction| faction.name.clone())
                .unwrap_or_else(|| t.text("unknown"));
            let color = faction.map(faction_color).unwrap_or_else(war_border);
            (
                city.id.clone(),
                city.name.clone(),
                faction_name,
                city.gold,
                city.food,
                city.troops.total(),
                color,
                city.faction_id == game.player_faction_id,
            )
        })
        .collect();
    rows.sort_by(|a, b| a.1.cmp(&b.1));

    let body_height = (height - 70.0).max(300.0);
    let content_width = ui.available_width().min(width);
    let panel_gap = 12.0;
    let panel_inner_margin_x = 20.0;
    let panel_inner_margin_y = 16.0;
    let list_outer_width = (content_width * 0.42).clamp(260.0, 380.0).max(220.0);
    let preview_outer_width = (content_width - list_outer_width - panel_gap).max(0.0);
    let list_width = (list_outer_width - panel_inner_margin_x).max(220.0);
    let preview_width = (preview_outer_width - panel_inner_margin_x).max(180.0);
    let panel_inner_height = (body_height - panel_inner_margin_y).max(260.0);

    StripBuilder::new(ui)
        .size(Size::exact(list_outer_width))
        .size(Size::exact(panel_gap))
        .size(Size::exact(preview_outer_width))
        .cell_layout(egui::Layout::top_down(egui::Align::Min))
        .clip(true)
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.set_height(body_height);
                war_sub_panel_frame().show(ui, |ui| {
                    ui.set_width(list_width);
                    egui::ScrollArea::vertical()
                        .id_salt("city_list")
                        .max_height(panel_inner_height)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (
                                city_id,
                                city_name,
                                faction_name,
                                gold,
                                food,
                                troops,
                                color,
                                player_owned,
                            ) in rows
                            {
                                let selected =
                                    ui_state.selected_city_id.as_deref() == Some(city_id.as_str());
                                let resources = t.text_args(
                                    "city-list-row-resources",
                                    &args([
                                        ("gold", gold.to_string()),
                                        ("food", food.to_string()),
                                        ("troops", troops.to_string()),
                                    ]),
                                );
                                let response = city_list_row(
                                    ui,
                                    selected,
                                    &city_name,
                                    &faction_name,
                                    &resources,
                                    color,
                                    player_owned,
                                );

                                if response.clicked() {
                                    ui_state.selected_city_id = Some(city_id.clone());
                                }
                                if response.double_clicked() {
                                    open_city(ui_state, city_id.clone());
                                    ui_state.city_list_open = false;
                                }
                            }
                        });
                });
            });

            let preview = ui_state.game.as_ref().and_then(|game| {
                let city = game.cities.get(ui_state.selected_city_id.as_deref()?)?;
                let faction_name = game
                    .factions
                    .get(&city.faction_id)
                    .map(|faction| faction.name.clone())
                    .unwrap_or_else(|| t.text("unknown"));
                Some((city.id.clone(), city.clone(), faction_name))
            });

            strip.empty();

            strip.cell(|ui| {
                ui.set_height(body_height);
                war_sub_panel_frame().show(ui, |ui| {
                    ui.set_width(preview_width);
                    ui.set_min_height(panel_inner_height);
                    if let Some((city_id, city, faction_name)) = preview {
                        city_summary_intel(ui, &city, &faction_name, t);
                        ui.add_space(10.0);
                        if ui
                            .add_sized(
                                [preview_width, 30.0],
                                egui::Button::new(t.text("open-command-tent")),
                            )
                            .clicked()
                        {
                            open_city(ui_state, city_id);
                            ui_state.city_list_open = false;
                        }
                    } else {
                        ui.label(t.text("selected-city-none"));
                    }
                });
            });
        });
}

fn city_list_row(
    ui: &mut egui::Ui,
    selected: bool,
    city_name: &str,
    faction_name: &str,
    resources: &str,
    color: egui::Color32,
    player_owned: bool,
) -> egui::Response {
    let row_size = egui::vec2(ui.available_width(), 64.0);
    let (rect, response) = ui.allocate_exact_size(row_size, egui::Sense::click());
    if !ui.is_rect_visible(rect) {
        return response;
    }

    let fill = if selected {
        egui::Color32::from_rgba_unmultiplied(113, 80, 42, 220)
    } else if response.hovered() {
        egui::Color32::from_rgba_unmultiplied(52, 42, 29, 220)
    } else {
        egui::Color32::from_rgba_unmultiplied(24, 21, 16, 96)
    };
    let stroke = egui::Stroke::new(
        if selected { 1.6 } else { 1.0 },
        if selected {
            war_gold()
        } else {
            color.gamma_multiply(0.65)
        },
    );

    let painter = ui.painter().with_clip_rect(rect);
    painter.rect(
        rect.shrink(1.0),
        5.0,
        fill,
        stroke,
        egui::StrokeKind::Inside,
    );

    let icon_center = egui::pos2(rect.left() + 31.0, rect.center().y + 1.0);
    draw_city_marker_icon(
        &painter,
        icon_center,
        0.42,
        0.42,
        color,
        selected,
        player_owned,
    );

    let text_x = rect.left() + 62.0;
    painter.text(
        egui::pos2(text_x, rect.top() + 8.0),
        egui::Align2::LEFT_TOP,
        city_name,
        egui::FontId::proportional(18.0),
        if selected { war_gold() } else { war_text() },
    );
    painter.text(
        egui::pos2(text_x, rect.top() + 31.0),
        egui::Align2::LEFT_TOP,
        faction_name,
        egui::FontId::proportional(13.0),
        war_text_muted(),
    );
    painter.text(
        egui::pos2(text_x, rect.top() + 47.0),
        egui::Align2::LEFT_TOP,
        resources,
        egui::FontId::proportional(12.0),
        war_text_muted(),
    );

    response
}
