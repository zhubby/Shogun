use crate::game::*;
use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use std::cmp::Ordering;

use super::super::HUD_MARGIN;
use super::super::i18n::{Translator, args};
use super::super::labels::{
    development_focus_label, diplomacy_label, facility_kind_label, troop_kind_label,
};
use super::super::map::faction_color;
use super::super::state::{FactionOverviewSort, FactionOverviewSortColumn, GameUiState};
use super::super::style::{
    modal_title_bar, war_danger, war_gold, war_panel_frame, war_sub_panel_frame, war_success,
    war_text_muted, war_warning,
};
use super::officer_common::officer_status_label;

pub(super) fn faction_overview_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.faction_overview_open && ui_state.faction_detail_id.is_none() {
        return;
    }

    if ui_state.faction_overview_open {
        let modal_width = (screen.width() * 0.78)
            .clamp(680.0, 1040.0)
            .min((screen.width() - HUD_MARGIN * 2.0).max(360.0));
        let modal_height = (screen.height() * 0.72)
            .clamp(420.0, 640.0)
            .min((screen.height() - HUD_MARGIN * 2.0).max(320.0));
        egui::Area::new(egui::Id::new("hud_faction_overview"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                war_panel_frame().show(ui, |ui| {
                    ui.set_width(modal_width);
                    ui.set_min_height(modal_height);
                    if modal_title_bar(ui, t, &t.text("faction-overview-title")) {
                        ui_state.faction_overview_open = false;
                    }
                    ui.separator();
                    let Some(game) = ui_state.game.as_ref() else {
                        ui.label(t.text("message-no-game-state"));
                        return;
                    };
                    faction_overview_table(
                        ui,
                        game,
                        &mut ui_state.faction_overview_sort,
                        &mut ui_state.selected_faction_overview_id,
                        &mut ui_state.faction_detail_id,
                        t,
                        modal_height - 70.0,
                    );
                });
            });
    }

    faction_detail_modal(ctx, ui_state, t, screen);
}

fn faction_overview_table(
    ui: &mut egui::Ui,
    game: &GameState,
    sort: &mut FactionOverviewSort,
    selected_faction_id: &mut Option<FactionId>,
    faction_detail_id: &mut Option<FactionId>,
    t: &Translator,
    max_height: f32,
) {
    let rows = faction_overview_rows(game, *sort);
    ui.label(t.text_args(
        "faction-overview-count",
        &super::super::i18n::args([("count", rows.len().to_string())]),
    ));

    let table_height = max_height.max(260.0);
    TableBuilder::new(ui)
        .id_salt("hud_faction_overview_table")
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .min_scrolled_height(table_height)
        .max_scroll_height(table_height)
        .auto_shrink([false, false])
        .column(Column::exact(54.0))
        .column(Column::remainder().at_least(126.0).clip(true))
        .column(Column::exact(62.0))
        .column(Column::exact(68.0))
        .columns(Column::initial(86.0).at_least(70.0).clip(true), 3)
        .column(Column::initial(94.0).at_least(78.0).clip(true))
        .column(Column::initial(104.0).at_least(82.0).clip(true))
        .column(Column::initial(92.0).at_least(76.0).clip(true))
        .header(28.0, |mut header| {
            header.col(|ui| {
                ui.centered_and_justified(|ui| {
                    ui.strong(t.text("faction-overview-column-tools"));
                });
            });
            sortable_header_cell(
                &mut header,
                sort,
                FactionOverviewSortColumn::Faction,
                t.text("faction-overview-column-faction"),
            );
            sortable_header_cell(
                &mut header,
                sort,
                FactionOverviewSortColumn::Cities,
                t.text("faction-overview-column-cities"),
            );
            sortable_header_cell(
                &mut header,
                sort,
                FactionOverviewSortColumn::Officers,
                t.text("faction-overview-column-officers"),
            );
            sortable_header_cell(
                &mut header,
                sort,
                FactionOverviewSortColumn::Gold,
                t.text("resource-gold"),
            );
            sortable_header_cell(
                &mut header,
                sort,
                FactionOverviewSortColumn::Food,
                t.text("resource-food"),
            );
            sortable_header_cell(
                &mut header,
                sort,
                FactionOverviewSortColumn::Materials,
                t.text("resource-materials"),
            );
            sortable_header_cell(
                &mut header,
                sort,
                FactionOverviewSortColumn::Troops,
                t.text("resource-troops"),
            );
            sortable_header_cell(
                &mut header,
                sort,
                FactionOverviewSortColumn::Population,
                t.text("resource-population"),
            );
            sortable_header_cell(
                &mut header,
                sort,
                FactionOverviewSortColumn::Wounded,
                t.text("resource-wounded"),
            );
        })
        .body(|mut body| {
            for row in rows {
                let selected = selected_faction_id.as_deref() == Some(row.faction_id.as_str());
                body.row(30.0, |mut table_row| {
                    table_row.set_selected(selected);
                    table_row.col(|ui| {
                        faction_row_tools(ui, &row, selected_faction_id, faction_detail_id, t)
                    });
                    table_row.col(|ui| row_cell(ui, &row.faction_name));
                    table_row.col(|ui| row_cell(ui, row.city_count.to_string()));
                    table_row.col(|ui| row_cell(ui, row.active_officer_count.to_string()));
                    table_row.col(|ui| row_cell(ui, row.gold.to_string()));
                    table_row.col(|ui| row_cell(ui, row.food.to_string()));
                    table_row.col(|ui| row_cell(ui, row.materials.to_string()));
                    table_row.col(|ui| row_cell(ui, row.troops.to_string()));
                    table_row.col(|ui| row_cell(ui, row.population.to_string()));
                    table_row.col(|ui| row_cell(ui, row.wounded.to_string()));

                    let response = table_row.response();
                    if response.clicked() || response.secondary_clicked() {
                        *selected_faction_id = Some(row.faction_id.clone());
                    }
                    if response.double_clicked() {
                        *selected_faction_id = Some(row.faction_id.clone());
                        *faction_detail_id = Some(row.faction_id.clone());
                    }
                });
            }
        });
}

fn sortable_header_cell(
    row: &mut egui_extras::TableRow<'_, '_>,
    sort: &mut FactionOverviewSort,
    column: FactionOverviewSortColumn,
    text: String,
) {
    row.col(|ui| {
        let marker = if sort.column == column {
            if sort.descending {
                egui_phosphor::regular::CARET_DOWN
            } else {
                egui_phosphor::regular::CARET_UP
            }
        } else {
            " "
        };
        let label = egui::WidgetText::from(egui::text::LayoutJob::simple(
            format!("{text}  {marker}"),
            egui::FontId::proportional(14.0),
            war_gold(),
            ui.available_width(),
        ));
        if ui.button(label).clicked() {
            sort.activate(column);
        }
    });
}

fn row_cell(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) {
    ui.add(egui::Label::new(text).truncate());
}

fn faction_row_tools(
    ui: &mut egui::Ui,
    row: &FactionOverviewRow,
    selected_faction_id: &mut Option<FactionId>,
    faction_detail_id: &mut Option<FactionId>,
    t: &Translator,
) {
    let response = ui
        .centered_and_justified(|ui| {
            ui.add_sized(
                [32.0, 24.0],
                egui::Button::new(
                    egui::RichText::new(egui_phosphor::regular::EYE)
                        .size(16.0)
                        .color(war_gold()),
                ),
            )
        })
        .inner
        .on_hover_text(t.text("faction-overview-action-view"));
    if response.clicked() {
        *selected_faction_id = Some(row.faction_id.clone());
        *faction_detail_id = Some(row.faction_id.clone());
    }
}

fn faction_detail_modal(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    let Some(faction_id) = ui_state.faction_detail_id.clone() else {
        return;
    };
    let Some(game) = ui_state.game.as_ref() else {
        ui_state.faction_detail_id = None;
        return;
    };
    let Some(snapshot) = faction_detail_snapshot(game, &faction_id, t) else {
        ui_state.faction_detail_id = None;
        return;
    };

    let width = (screen.width() * 0.82)
        .clamp(820.0, 1080.0)
        .min((screen.width() - HUD_MARGIN * 2.0).max(360.0));
    let height = (screen.height() * 0.80)
        .clamp(520.0, 740.0)
        .min((screen.height() - HUD_MARGIN * 2.0).max(320.0));
    let mut close_requested = false;

    egui::Area::new(egui::Id::new("hud_faction_detail"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                let title = t.text_args(
                    "faction-detail-title",
                    &args([("faction", snapshot.name.clone())]),
                );
                if modal_title_bar(ui, t, &title) {
                    close_requested = true;
                    return;
                }
                ui.separator();
                faction_detail_header(ui, &snapshot, t);
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt(("faction_detail_body", &snapshot.id))
                    .max_height(height - 126.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.columns(2, |columns| {
                            columns[0].set_width((width * 0.38).clamp(300.0, 390.0));
                            faction_identity_section(&mut columns[0], &snapshot, t);
                            columns[0].add_space(8.0);
                            faction_resources_section(&mut columns[0], &snapshot, t);
                            columns[0].add_space(8.0);
                            faction_technology_section(&mut columns[0], &snapshot, t);

                            faction_cities_section(&mut columns[1], &snapshot, t);
                            columns[1].add_space(8.0);
                            faction_officers_section(&mut columns[1], &snapshot, t);
                            columns[1].add_space(8.0);
                            faction_diplomacy_section(&mut columns[1], &snapshot, t);
                            columns[1].add_space(8.0);
                            faction_actions_section(&mut columns[1], &snapshot, t);
                        });
                    });
            });
        });

    if close_requested {
        ui_state.faction_detail_id = None;
    }
}

fn faction_detail_header(ui: &mut egui::Ui, snapshot: &FactionDetailSnapshot, t: &Translator) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(&snapshot.name)
                .size(24.0)
                .color(war_gold())
                .strong(),
        );
        ui.separator();
        ui.colored_label(faction_status_color(snapshot.alive), &snapshot.status_label);
        ui.separator();
        ui.label(t.text_args(
            "faction-detail-header-ruler",
            &args([("ruler", snapshot.ruler_name.clone())]),
        ));
        ui.separator();
        ui.label(t.text_args(
            "faction-detail-header-cities",
            &args([("count", snapshot.resources.city_count.to_string())]),
        ));
    });
}

fn faction_identity_section(ui: &mut egui::Ui, snapshot: &FactionDetailSnapshot, t: &Translator) {
    war_sub_panel_frame().show(ui, |ui| {
        detail_section_title(ui, &t.text("faction-detail-section-identity"));
        detail_kv(ui, &t.text("faction-detail-id"), snapshot.id.clone());
        detail_kv(
            ui,
            &t.text("faction-detail-status"),
            snapshot.status_label.clone(),
        );
        detail_kv(
            ui,
            &t.text("faction-detail-controller"),
            snapshot.controller_label.clone(),
        );
        detail_kv(
            ui,
            &t.text("faction-detail-selectable"),
            snapshot.selectable_label.clone(),
        );
        detail_kv(
            ui,
            &t.text("faction-detail-ruler"),
            snapshot.ruler_name.clone(),
        );
        detail_kv(
            ui,
            &t.text("faction-detail-heir"),
            snapshot.heir_name.clone(),
        );
        ui.horizontal_wrapped(|ui| {
            ui.add_sized(
                [96.0, 20.0],
                egui::Label::new(
                    egui::RichText::new(t.text("faction-detail-color")).color(war_text_muted()),
                ),
            );
            let (rect, _) = ui.allocate_exact_size(egui::vec2(42.0, 16.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 3.0, snapshot.color);
            ui.painter().rect_stroke(
                rect,
                3.0,
                egui::Stroke::new(1.0, war_gold()),
                egui::StrokeKind::Inside,
            );
        });
    });
}

fn faction_resources_section(ui: &mut egui::Ui, snapshot: &FactionDetailSnapshot, t: &Translator) {
    war_sub_panel_frame().show(ui, |ui| {
        detail_section_title(ui, &t.text("faction-detail-section-resources"));
        let resources = &snapshot.resources;
        detail_kv(
            ui,
            &t.text("faction-overview-column-cities"),
            resources.city_count.to_string(),
        );
        detail_kv(
            ui,
            &t.text("faction-detail-active-officers"),
            resources.active_officer_count.to_string(),
        );
        detail_kv(
            ui,
            &t.text("faction-detail-total-officers"),
            resources.total_officer_count.to_string(),
        );
        detail_kv(ui, &t.text("resource-gold"), resources.gold.to_string());
        detail_kv(ui, &t.text("resource-food"), resources.food.to_string());
        detail_kv(
            ui,
            &t.text("resource-materials"),
            resources.materials.to_string(),
        );
        detail_kv(
            ui,
            &t.text("resource-population"),
            resources.population.to_string(),
        );
        detail_kv(
            ui,
            &t.text("resource-troops"),
            resources.troops.total().to_string(),
        );
        detail_kv(
            ui,
            &t.text("faction-detail-troop-breakdown"),
            t.text_args(
                "troop-pool-short",
                &args([
                    ("infantry", resources.troops.infantry.to_string()),
                    ("cavalry", resources.troops.cavalry.to_string()),
                    ("archers", resources.troops.archers.to_string()),
                ]),
            ),
        );
        detail_kv(
            ui,
            &t.text("resource-wounded"),
            resources.wounded.to_string(),
        );
    });
}

fn faction_technology_section(ui: &mut egui::Ui, snapshot: &FactionDetailSnapshot, t: &Translator) {
    war_sub_panel_frame().show(ui, |ui| {
        detail_section_title(ui, &t.text("faction-detail-section-technology"));
        detail_kv(
            ui,
            &t.text("technology-active"),
            snapshot.technology.active_label.clone(),
        );
        detail_kv(
            ui,
            &t.text("technology-progress"),
            snapshot.technology.progress_label.clone(),
        );
        detail_kv(
            ui,
            &t.text("technology-funded"),
            snapshot.technology.funded_count.to_string(),
        );
        detail_kv(
            ui,
            &t.text("technology-completed"),
            snapshot.technology.completed_count.to_string(),
        );
        if snapshot.technology.completed_names.is_empty() {
            ui.colored_label(war_text_muted(), t.text("faction-detail-none"));
        } else {
            ui.label(snapshot.technology.completed_names.join(", "));
        }
    });
}

fn faction_cities_section(ui: &mut egui::Ui, snapshot: &FactionDetailSnapshot, t: &Translator) {
    war_sub_panel_frame().show(ui, |ui| {
        detail_section_title(ui, &t.text("faction-detail-section-cities"));
        if snapshot.cities.is_empty() {
            ui.colored_label(war_text_muted(), t.text("faction-detail-empty-cities"));
            return;
        }
        egui::ScrollArea::vertical()
            .id_salt(("faction_detail_cities", &snapshot.id))
            .max_height(150.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for city in &snapshot.cities {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} · {} {} · {} {} / {} {} / {} {}",
                            city.name,
                            t.text("faction-detail-governor"),
                            city.governor_name,
                            t.text("resource-population"),
                            city.population,
                            t.text("resource-troops"),
                            city.troops,
                            t.text("resource-gold"),
                            city.gold,
                        ))
                        .strong(),
                    );
                    ui.colored_label(
                        war_text_muted(),
                        t.text_args(
                            "faction-detail-city-line",
                            &args([
                                ("food", city.food.to_string()),
                                ("materials", city.materials.to_string()),
                                ("defense", city.defense.to_string()),
                                ("training", city.training.to_string()),
                                ("order", city.order.to_string()),
                            ]),
                        ),
                    );
                    ui.add_space(3.0);
                }
            });
    });
}

fn faction_officers_section(ui: &mut egui::Ui, snapshot: &FactionDetailSnapshot, t: &Translator) {
    war_sub_panel_frame().show(ui, |ui| {
        detail_section_title(ui, &t.text("faction-detail-section-officers"));
        if snapshot.officers.is_empty() {
            ui.colored_label(war_text_muted(), t.text("faction-detail-empty-officers"));
            return;
        }
        egui::ScrollArea::vertical()
            .id_salt(("faction_detail_officers", &snapshot.id))
            .max_height(180.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for officer in &snapshot.officers {
                    let role = if officer.role_labels.is_empty() {
                        t.text("none")
                    } else {
                        officer.role_labels.join(", ")
                    };
                    ui.label(
                        egui::RichText::new(format!(
                            "{} · {} · {} · {}",
                            officer.name, officer.status_label, role, officer.city_name
                        ))
                        .strong(),
                    );
                    ui.colored_label(
                        war_text_muted(),
                        t.text_args(
                            "faction-detail-officer-line",
                            &args([
                                ("loyalty", officer.loyalty.to_string()),
                                ("leadership", officer.leadership.to_string()),
                                ("strength", officer.strength.to_string()),
                                ("intelligence", officer.intelligence.to_string()),
                                ("politics", officer.politics.to_string()),
                                ("charm", officer.charm.to_string()),
                            ]),
                        ),
                    );
                    ui.add_space(3.0);
                }
            });
    });
}

fn faction_diplomacy_section(ui: &mut egui::Ui, snapshot: &FactionDetailSnapshot, t: &Translator) {
    war_sub_panel_frame().show(ui, |ui| {
        detail_section_title(ui, &t.text("faction-detail-section-diplomacy"));
        if snapshot.diplomacy.is_empty() {
            ui.colored_label(war_text_muted(), t.text("faction-detail-empty-diplomacy"));
            return;
        }
        egui::ScrollArea::vertical()
            .id_salt(("faction_detail_diplomacy", &snapshot.id))
            .max_height(130.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for relation in &snapshot.diplomacy {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(&relation.faction_name).strong());
                        ui.separator();
                        ui.colored_label(
                            relation_score_color(relation.score),
                            relation.score.to_string(),
                        );
                        ui.separator();
                        ui.label(&relation.truce_label);
                    });
                }
            });
    });
}

fn faction_actions_section(ui: &mut egui::Ui, snapshot: &FactionDetailSnapshot, t: &Translator) {
    war_sub_panel_frame().show(ui, |ui| {
        detail_section_title(ui, &t.text("faction-detail-section-actions"));
        let mut any = false;
        if !snapshot.pending_commands.is_empty() {
            any = true;
            ui.label(egui::RichText::new(t.text("faction-detail-pending-commands")).strong());
            for action in &snapshot.pending_commands {
                ui.colored_label(war_text_muted(), action);
            }
            ui.add_space(4.0);
        }
        if !snapshot.army_movements.is_empty() {
            any = true;
            ui.label(egui::RichText::new(t.text("faction-detail-army-movements")).strong());
            for action in &snapshot.army_movements {
                ui.colored_label(war_text_muted(), action);
            }
            ui.add_space(4.0);
        }
        if !snapshot.recruitments.is_empty() {
            any = true;
            ui.label(egui::RichText::new(t.text("faction-detail-recruitments")).strong());
            for action in &snapshot.recruitments {
                ui.colored_label(war_text_muted(), action);
            }
        }
        if !any {
            ui.colored_label(war_text_muted(), t.text("faction-detail-empty-actions"));
        }
    });
}

fn detail_section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(egui::RichText::new(title).color(war_gold()).strong());
    ui.add_space(4.0);
}

fn detail_kv(ui: &mut egui::Ui, key: &str, value: String) {
    ui.horizontal_wrapped(|ui| {
        ui.add_sized(
            [96.0, 20.0],
            egui::Label::new(egui::RichText::new(key).color(war_text_muted())),
        );
        ui.label(value);
    });
}

fn faction_status_color(alive: bool) -> egui::Color32 {
    if alive { war_success() } else { war_danger() }
}

fn relation_score_color(score: i16) -> egui::Color32 {
    match score {
        50..=i16::MAX => war_success(),
        0..=49 => war_gold(),
        -39..=-1 => war_warning(),
        _ => war_danger(),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::core::hud) struct FactionOverviewRow {
    pub(in crate::core::hud) faction_id: FactionId,
    pub(in crate::core::hud) faction_name: String,
    pub(in crate::core::hud) city_count: usize,
    pub(in crate::core::hud) active_officer_count: usize,
    pub(in crate::core::hud) gold: i64,
    pub(in crate::core::hud) food: i64,
    pub(in crate::core::hud) materials: i64,
    pub(in crate::core::hud) troops: u64,
    pub(in crate::core::hud) population: u64,
    pub(in crate::core::hud) wounded: u64,
}

pub(in crate::core::hud) fn faction_overview_rows(
    game: &GameState,
    sort: FactionOverviewSort,
) -> Vec<FactionOverviewRow> {
    let mut rows: Vec<_> = game
        .factions
        .values()
        .filter(|faction| game.faction_alive(&faction.id))
        .map(|faction| faction_overview_row(game, faction))
        .collect();
    sort_faction_overview_rows(&mut rows, sort);
    rows
}

fn faction_overview_row(game: &GameState, faction: &Faction) -> FactionOverviewRow {
    let mut row = FactionOverviewRow {
        faction_id: faction.id.clone(),
        faction_name: faction.name.clone(),
        active_officer_count: game
            .officers
            .values()
            .filter(|officer| officer.faction_id == faction.id && officer.is_active())
            .count(),
        ..FactionOverviewRow::default()
    };

    for city in game
        .cities
        .values()
        .filter(|city| city.faction_id == faction.id)
    {
        row.city_count += 1;
        row.gold += i64::from(city.gold);
        row.food += i64::from(city.food);
        row.materials += i64::from(city.materials);
        row.troops += u64::from(city.troops.total());
        row.population += u64::from(city.population);
        row.wounded += u64::from(city.wounded_troops.total());
    }

    row
}

pub(in crate::core::hud) fn sort_faction_overview_rows(
    rows: &mut [FactionOverviewRow],
    sort: FactionOverviewSort,
) {
    rows.sort_by(|a, b| {
        let primary = compare_faction_overview_rows(a, b, sort.column);
        let ordering = if sort.descending {
            primary.reverse()
        } else {
            primary
        };
        ordering
            .then_with(|| (&a.faction_name, &a.faction_id).cmp(&(&b.faction_name, &b.faction_id)))
    });
}

fn compare_faction_overview_rows(
    a: &FactionOverviewRow,
    b: &FactionOverviewRow,
    column: FactionOverviewSortColumn,
) -> Ordering {
    match column {
        FactionOverviewSortColumn::Faction => {
            (&a.faction_name, &a.faction_id).cmp(&(&b.faction_name, &b.faction_id))
        }
        FactionOverviewSortColumn::Cities => a.city_count.cmp(&b.city_count),
        FactionOverviewSortColumn::Officers => a.active_officer_count.cmp(&b.active_officer_count),
        FactionOverviewSortColumn::Gold => a.gold.cmp(&b.gold),
        FactionOverviewSortColumn::Food => a.food.cmp(&b.food),
        FactionOverviewSortColumn::Materials => a.materials.cmp(&b.materials),
        FactionOverviewSortColumn::Troops => a.troops.cmp(&b.troops),
        FactionOverviewSortColumn::Population => a.population.cmp(&b.population),
        FactionOverviewSortColumn::Wounded => a.wounded.cmp(&b.wounded),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::core::hud) struct FactionDetailSnapshot {
    id: FactionId,
    name: String,
    alive: bool,
    status_label: String,
    controller_label: String,
    selectable_label: String,
    ruler_name: String,
    heir_name: String,
    color: egui::Color32,
    resources: FactionDetailResources,
    cities: Vec<FactionDetailCity>,
    officers: Vec<FactionDetailOfficer>,
    diplomacy: Vec<FactionDetailDiplomacy>,
    technology: FactionDetailTechnology,
    pending_commands: Vec<String>,
    army_movements: Vec<String>,
    recruitments: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::core::hud) struct FactionDetailResources {
    city_count: usize,
    active_officer_count: usize,
    total_officer_count: usize,
    gold: i64,
    food: i64,
    materials: i64,
    population: u64,
    troops: TroopPool,
    wounded: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FactionDetailCity {
    name: String,
    governor_name: String,
    population: u32,
    gold: i32,
    food: i32,
    materials: i32,
    troops: u32,
    defense: u16,
    training: u8,
    order: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FactionDetailOfficer {
    name: String,
    status_label: String,
    role_labels: Vec<String>,
    city_name: String,
    loyalty: u8,
    leadership: u8,
    strength: u8,
    intelligence: u8,
    politics: u8,
    charm: u8,
    rank_score: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FactionDetailDiplomacy {
    faction_name: String,
    score: i16,
    truce_label: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FactionDetailTechnology {
    active_label: String,
    progress_label: String,
    funded_count: usize,
    completed_count: usize,
    completed_names: Vec<String>,
}

pub(in crate::core::hud) fn faction_detail_snapshot(
    game: &GameState,
    faction_id: &str,
    t: &Translator,
) -> Option<FactionDetailSnapshot> {
    let faction = game.factions.get(faction_id)?;
    let resources = faction_detail_resources(game, faction_id);
    let alive = resources.city_count > 0;
    Some(FactionDetailSnapshot {
        id: faction.id.clone(),
        name: faction.name.clone(),
        alive,
        status_label: if alive {
            t.text("faction-detail-status-alive")
        } else {
            t.text("faction-detail-status-destroyed")
        },
        controller_label: controller_label(&faction.controlled_by, t),
        selectable_label: yes_no_label(faction.selectable, t),
        ruler_name: officer_name(game, &faction.ruler_id, t),
        heir_name: faction
            .heir_id
            .as_deref()
            .map(|id| officer_name(game, id, t))
            .unwrap_or_else(|| t.text("none")),
        color: faction_color(faction),
        resources,
        cities: faction_detail_cities(game, faction_id, t),
        officers: faction_detail_officers(game, faction, t),
        diplomacy: faction_detail_diplomacy(game, faction_id, t),
        technology: faction_detail_technology(game, faction_id, t),
        pending_commands: faction_pending_commands(game, faction_id, t),
        army_movements: faction_army_movements(game, faction_id, t),
        recruitments: faction_recruitments(game, faction_id, t),
    })
}

fn faction_detail_resources(game: &GameState, faction_id: &str) -> FactionDetailResources {
    let mut resources = FactionDetailResources {
        active_officer_count: game
            .officers
            .values()
            .filter(|officer| officer.faction_id == faction_id && officer.is_active())
            .count(),
        total_officer_count: game
            .officers
            .values()
            .filter(|officer| officer.faction_id == faction_id)
            .count(),
        ..FactionDetailResources::default()
    };
    for city in game
        .cities
        .values()
        .filter(|city| city.faction_id == faction_id)
    {
        resources.city_count += 1;
        resources.gold += i64::from(city.gold);
        resources.food += i64::from(city.food);
        resources.materials += i64::from(city.materials);
        resources.population += u64::from(city.population);
        resources.troops.add_pool(city.troops);
        resources.wounded += u64::from(city.wounded_troops.total());
    }
    resources
}

fn faction_detail_cities(
    game: &GameState,
    faction_id: &str,
    t: &Translator,
) -> Vec<FactionDetailCity> {
    let mut cities: Vec<_> = game
        .cities
        .values()
        .filter(|city| city.faction_id == faction_id)
        .map(|city| FactionDetailCity {
            name: city.name.clone(),
            governor_name: city
                .governor_id
                .as_deref()
                .map(|id| officer_name(game, id, t))
                .unwrap_or_else(|| t.text("none")),
            population: city.population,
            gold: city.gold,
            food: city.food,
            materials: city.materials,
            troops: city.troops.total(),
            defense: city.defense,
            training: city.training,
            order: city.order,
        })
        .collect();
    cities.sort_by(|a, b| b.troops.cmp(&a.troops).then_with(|| a.name.cmp(&b.name)));
    cities
}

fn faction_detail_officers(
    game: &GameState,
    faction: &Faction,
    t: &Translator,
) -> Vec<FactionDetailOfficer> {
    let mut officers: Vec<_> = game
        .officers
        .values()
        .filter(|officer| officer.faction_id == faction.id)
        .map(|officer| {
            let role_labels = officer_role_labels(game, faction, officer, t);
            FactionDetailOfficer {
                name: officer.name.clone(),
                status_label: officer_status_label(&officer.status, t),
                role_labels,
                city_name: officer
                    .city_id
                    .as_deref()
                    .and_then(|city_id| game.cities.get(city_id))
                    .map(|city| city.name.clone())
                    .unwrap_or_else(|| t.text("officer-city-unassigned")),
                loyalty: officer.loyalty,
                leadership: officer.stats.leadership,
                strength: officer.stats.strength,
                intelligence: officer.stats.intelligence,
                politics: officer.stats.politics,
                charm: officer.stats.charm,
                rank_score: officer_rank_score(game, faction, officer),
            }
        })
        .collect();
    officers.sort_by(|a, b| {
        b.rank_score
            .cmp(&a.rank_score)
            .then_with(|| b.leadership.cmp(&a.leadership))
            .then_with(|| b.politics.cmp(&a.politics))
            .then_with(|| a.name.cmp(&b.name))
    });
    officers
}

fn officer_role_labels(
    game: &GameState,
    faction: &Faction,
    officer: &Officer,
    t: &Translator,
) -> Vec<String> {
    let mut roles = Vec::new();
    if faction.ruler_id == officer.id {
        roles.push(t.text("officer-detail-graph-ruler"));
    }
    if faction.heir_id.as_deref() == Some(officer.id.as_str()) {
        roles.push(t.text("officer-detail-graph-heir"));
    }
    if game
        .cities
        .values()
        .any(|city| city.governor_id.as_deref() == Some(officer.id.as_str()))
    {
        roles.push(t.text("officer-detail-graph-governor"));
    }
    if let Some(spec) = officer.office_id.as_deref().and_then(official_post_spec) {
        roles.push(spec.name.to_string());
    }
    roles
}

fn officer_rank_score(game: &GameState, faction: &Faction, officer: &Officer) -> i32 {
    let mut score = 0;
    if faction.ruler_id == officer.id {
        score += 10_000;
    }
    if faction.heir_id.as_deref() == Some(officer.id.as_str()) {
        score += 8_000;
    }
    if game
        .cities
        .values()
        .any(|city| city.governor_id.as_deref() == Some(officer.id.as_str()))
    {
        score += 4_000;
    }
    if let Some(spec) = officer.office_id.as_deref().and_then(official_post_spec) {
        score += i32::from(official_rank_order(spec.rank)) * 100;
    }
    score
}

fn faction_detail_diplomacy(
    game: &GameState,
    faction_id: &str,
    t: &Translator,
) -> Vec<FactionDetailDiplomacy> {
    let mut rows: Vec<_> = game
        .factions
        .values()
        .filter(|faction| faction.id != faction_id && game.faction_alive(&faction.id))
        .map(|other| {
            let relation = game.relation(faction_id, &other.id);
            let score = relation.map(|relation| relation.score).unwrap_or_default();
            let truce_label = relation
                .and_then(|relation| relation.truce_until_turn)
                .filter(|until| *until >= game.turn)
                .map(|until| {
                    t.text_args(
                        "faction-detail-truce-until",
                        &args([("turn", until.to_string())]),
                    )
                })
                .unwrap_or_else(|| t.text("faction-detail-no-truce"));
            FactionDetailDiplomacy {
                faction_name: other.name.clone(),
                score,
                truce_label,
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.faction_name.cmp(&b.faction_name))
    });
    rows
}

fn faction_detail_technology(
    game: &GameState,
    faction_id: &str,
    t: &Translator,
) -> FactionDetailTechnology {
    let Some(state) = game.technologies.get(faction_id) else {
        return FactionDetailTechnology {
            active_label: t.text("none"),
            progress_label: t.text("none"),
            ..FactionDetailTechnology::default()
        };
    };
    let active_label = state
        .active
        .as_deref()
        .and_then(|id| game.technology_catalog.spec(id))
        .map(|spec| spec.name.clone())
        .unwrap_or_else(|| t.text("none"));
    let progress_label = state
        .active
        .as_deref()
        .and_then(|id| game.technology_catalog.spec(id))
        .map(|spec| {
            let progress = state.progress.get(&spec.id).copied().unwrap_or_default();
            t.text_args(
                "technology-progress",
                &args([
                    ("progress", progress.to_string()),
                    ("turns", spec.turns.to_string()),
                ]),
            )
        })
        .unwrap_or_else(|| t.text("none"));
    let mut completed_names: Vec<_> = state
        .completed
        .iter()
        .filter_map(|id| game.technology_catalog.spec(id))
        .map(|spec| spec.name.clone())
        .collect();
    completed_names.sort();
    FactionDetailTechnology {
        active_label,
        progress_label,
        funded_count: state.funded.len(),
        completed_count: state.completed.len(),
        completed_names,
    }
}

fn faction_pending_commands(game: &GameState, faction_id: &str, t: &Translator) -> Vec<String> {
    game.pending_commands
        .iter()
        .filter(|command| command.issuer_faction_id == faction_id)
        .map(|command| {
            let city = game
                .cities
                .get(&command.city_id)
                .map(|city| city.name.clone())
                .unwrap_or_else(|| command.city_id.clone());
            format!("{city}: {}", command_summary(game, command, t))
        })
        .collect()
}

fn faction_army_movements(game: &GameState, faction_id: &str, t: &Translator) -> Vec<String> {
    game.army_movements
        .iter()
        .filter(|movement| movement.issuer_faction_id == faction_id)
        .map(|movement| {
            let kind = match movement.kind {
                ArmyMovementKind::Transfer => t.text("command-transfer"),
                ArmyMovementKind::Expedition => t.text("command-expedition"),
            };
            let source = game
                .cities
                .get(&movement.source_city_id)
                .map(|city| city.name.clone())
                .unwrap_or_else(|| movement.source_city_id.clone());
            let target = game
                .cities
                .get(&movement.target_city_id)
                .map(|city| city.name.clone())
                .unwrap_or_else(|| movement.target_city_id.clone());
            let commander = officer_name(game, &movement.commander_id, t);
            t.text_args(
                "faction-detail-army-line",
                &args([
                    ("kind", kind),
                    ("source", source),
                    ("target", target),
                    ("commander", commander),
                    ("troops", movement.troops.total().to_string()),
                    ("arrival", movement.arrival_turn.to_string()),
                ]),
            )
        })
        .collect()
}

fn faction_recruitments(game: &GameState, faction_id: &str, t: &Translator) -> Vec<String> {
    game.officer_recruitments
        .iter()
        .filter(|task| task.issuer_faction_id == faction_id)
        .map(|task| {
            let source = game
                .cities
                .get(&task.source_city_id)
                .map(|city| city.name.clone())
                .unwrap_or_else(|| task.source_city_id.clone());
            let recruiter = officer_name(game, &task.recruiter_officer_id, t);
            let target = officer_name(game, &task.target_officer_id, t);
            t.text_args(
                "faction-detail-recruitment-line",
                &args([
                    ("city", source),
                    ("recruiter", recruiter),
                    ("target", target),
                    ("progress", task.progress.to_string()),
                ]),
            )
        })
        .collect()
}

fn command_summary(game: &GameState, command: &Command, t: &Translator) -> String {
    match &command.kind {
        CommandKind::Develop { focus } => t.text_args(
            "command-title-develop",
            &args([("focus", development_focus_label(t, focus))]),
        ),
        CommandKind::UpgradeCityCore => t.text("command-upgrade-core"),
        CommandKind::BuildFacility { kind } => t.text_args(
            "command-title-build-facility",
            &args([("facility", facility_kind_label(t, *kind))]),
        ),
        CommandKind::Recruit { kind, amount } => t.text_args(
            "command-title-recruit",
            &args([
                ("kind", troop_kind_label(t, *kind)),
                ("amount", amount.to_string()),
            ]),
        ),
        CommandKind::Train => t.text("command-train"),
        CommandKind::AppointGovernor { target_officer_id } => t.text_args(
            "command-appoint-target",
            &args([
                ("officer", officer_name(game, target_officer_id, t)),
                (
                    "city",
                    game.cities
                        .get(&command.city_id)
                        .map(|city| city.name.clone())
                        .unwrap_or_else(|| command.city_id.clone()),
                ),
            ]),
        ),
        CommandKind::Transfer {
            target_city_id,
            troops,
            ..
        } => {
            t.text_args(
                "command-title-transfer",
                &args([(
                    "troops",
                    t.text_args(
                        "troop-pool-short",
                        &args([
                            ("infantry", troops.infantry.to_string()),
                            ("cavalry", troops.cavalry.to_string()),
                            ("archers", troops.archers.to_string()),
                        ]),
                    ),
                )]),
            ) + &format!(" -> {}", city_name(game, target_city_id))
        }
        CommandKind::Expedition {
            target_city_id,
            assignments,
            ..
        } => {
            let troops = assignments
                .iter()
                .fold(TroopPool::default(), |mut pool, assignment| {
                    pool.add(assignment.troop_kind, assignment.troops);
                    pool
                });
            t.text_args(
                "command-title-expedition",
                &args([(
                    "troops",
                    t.text_args(
                        "troop-pool-short",
                        &args([
                            ("infantry", troops.infantry.to_string()),
                            ("cavalry", troops.cavalry.to_string()),
                            ("archers", troops.archers.to_string()),
                        ]),
                    ),
                )]),
            ) + &format!(" -> {}", city_name(game, target_city_id))
        }
        CommandKind::Diplomacy {
            target_faction_id,
            proposal,
        } => {
            t.text_args(
                "command-title-diplomacy",
                &args([("proposal", diplomacy_label(t, proposal))]),
            ) + &format!(" -> {}", faction_name(game, target_faction_id))
        }
    }
}

fn city_name(game: &GameState, city_id: &str) -> String {
    game.cities
        .get(city_id)
        .map(|city| city.name.clone())
        .unwrap_or_else(|| city_id.to_string())
}

fn faction_name(game: &GameState, faction_id: &str) -> String {
    game.factions
        .get(faction_id)
        .map(|faction| faction.name.clone())
        .unwrap_or_else(|| faction_id.to_string())
}

fn officer_name(game: &GameState, officer_id: &str, t: &Translator) -> String {
    game.officers
        .get(officer_id)
        .map(|officer| officer.name.clone())
        .unwrap_or_else(|| {
            if officer_id.is_empty() {
                t.text("unknown")
            } else {
                officer_id.to_string()
            }
        })
}

fn controller_label(controller: &Controller, t: &Translator) -> String {
    match controller {
        Controller::Player => t.text("faction-detail-controller-player"),
        Controller::RuleAi => t.text("faction-detail-controller-ai"),
    }
}

fn yes_no_label(value: bool, t: &Translator) -> String {
    if value {
        t.text("common-yes")
    } else {
        t.text("common-no")
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct FactionRowInteraction {
    clicked: bool,
    double_clicked: bool,
    view_clicked: bool,
}

#[cfg(test)]
fn apply_faction_row_interaction(
    row_id: &str,
    selected_faction_id: &mut Option<FactionId>,
    faction_detail_id: &mut Option<FactionId>,
    interaction: FactionRowInteraction,
) {
    if interaction.clicked || interaction.double_clicked || interaction.view_clicked {
        *selected_faction_id = Some(row_id.to_string());
    }
    if interaction.double_clicked || interaction.view_clicked {
        *faction_detail_id = Some(row_id.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::ui_state_with_game;
    use super::*;
    use crate::core::state::FactionOverviewSortColumn;
    use crate::game::{Faction, FactionTechnologyState, OfficerRecruitmentTask, OfficerStatus};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn faction_rows_exclude_factions_without_cities() {
        let mut state = ui_state_with_game();
        let game = state.game.as_mut().unwrap();
        game.factions.insert(
            "empty_faction".to_string(),
            Faction {
                id: "empty_faction".to_string(),
                name: "空势力".to_string(),
                ruler_id: "liu_bei".to_string(),
                heir_id: None,
                color: [0.3, 0.3, 0.3],
                selectable: false,
                controlled_by: Controller::RuleAi,
            },
        );

        let rows = faction_overview_rows(game, FactionOverviewSort::default());

        assert!(!rows.iter().any(|row| row.faction_id == "empty_faction"));
    }

    #[test]
    fn faction_rows_count_only_active_officers() {
        let mut state = ui_state_with_game();
        let game = state.game.as_mut().unwrap();
        let faction_id = game.player_faction_id.clone();
        for officer in game.officers.values_mut() {
            if officer.faction_id == faction_id {
                officer.faction_id = WILD_FACTION_ID.to_string();
            }
        }

        let template = game.officers.values().next().unwrap().clone();
        let mut officers = BTreeMap::new();
        for (id, status) in [
            ("active_one", OfficerStatus::Active),
            ("active_two", OfficerStatus::Active),
            ("minor", OfficerStatus::Minor),
            ("dead", OfficerStatus::Dead),
            ("unavailable", OfficerStatus::Unavailable),
        ] {
            let mut officer = template.clone();
            officer.id = id.to_string();
            officer.name = id.to_string();
            officer.faction_id = faction_id.clone();
            officer.status = status;
            officers.insert(officer.id.clone(), officer);
        }
        game.officers.extend(officers);

        let rows = faction_overview_rows(game, FactionOverviewSort::default());
        let row = rows
            .iter()
            .find(|row| row.faction_id == faction_id)
            .unwrap();

        assert_eq!(row.active_officer_count, 2);
    }

    #[test]
    fn faction_rows_aggregate_city_resources() {
        let mut state = ui_state_with_game();
        let game = state.game.as_mut().unwrap();
        let faction_id = game.player_faction_id.clone();
        let city_ids = game
            .cities
            .values()
            .filter(|city| city.faction_id == faction_id)
            .take(2)
            .map(|city| city.id.clone())
            .collect::<Vec<_>>();
        assert_eq!(city_ids.len(), 2);
        let other_faction_id = game
            .factions
            .keys()
            .find(|id| *id != &faction_id)
            .unwrap()
            .clone();
        for city in game.cities.values_mut() {
            if city.faction_id == faction_id && !city_ids.contains(&city.id) {
                city.faction_id = other_faction_id.clone();
            }
        }

        {
            let city = game.cities.get_mut(&city_ids[0]).unwrap();
            city.gold = 10;
            city.food = 20;
            city.materials = 30;
            city.troops = TroopPool {
                infantry: 40,
                cavalry: 50,
                archers: 60,
            };
            city.population = 70;
            city.wounded_troops = TroopPool {
                infantry: 1,
                cavalry: 2,
                archers: 3,
            };
        }
        {
            let city = game.cities.get_mut(&city_ids[1]).unwrap();
            city.gold = 100;
            city.food = 200;
            city.materials = 300;
            city.troops = TroopPool {
                infantry: 400,
                cavalry: 500,
                archers: 600,
            };
            city.population = 700;
            city.wounded_troops = TroopPool {
                infantry: 10,
                cavalry: 20,
                archers: 30,
            };
        }

        let rows = faction_overview_rows(game, FactionOverviewSort::default());
        let row = rows
            .iter()
            .find(|row| row.faction_id == faction_id)
            .unwrap();

        assert_eq!(row.city_count, 2);
        assert_eq!(row.gold, 110);
        assert_eq!(row.food, 220);
        assert_eq!(row.materials, 330);
        assert_eq!(row.troops, 1_650);
        assert_eq!(row.population, 770);
        assert_eq!(row.wounded, 66);
    }

    #[test]
    fn faction_rows_sort_with_default_direction_toggle_and_tiebreaker() {
        let mut rows = vec![
            test_row("b", "B", 3, 20),
            test_row("a", "A", 3, 40),
            test_row("c", "C", 1, 30),
        ];

        sort_faction_overview_rows(&mut rows, FactionOverviewSort::default());
        assert_eq!(row_ids(&rows), vec!["A", "B", "C"]);

        let mut sort = FactionOverviewSort::default();
        sort.activate(FactionOverviewSortColumn::Cities);
        sort_faction_overview_rows(&mut rows, sort);
        assert_eq!(row_ids(&rows), vec!["C", "A", "B"]);

        sort.activate(FactionOverviewSortColumn::Faction);
        sort_faction_overview_rows(&mut rows, sort);
        assert_eq!(row_ids(&rows), vec!["A", "B", "C"]);

        sort.activate(FactionOverviewSortColumn::Gold);
        sort_faction_overview_rows(&mut rows, sort);
        assert_eq!(row_ids(&rows), vec!["A", "C", "B"]);
    }

    #[test]
    fn faction_detail_snapshot_aggregates_resources_and_people() {
        let mut state = ui_state_with_game();
        let game = state.game.as_mut().unwrap();
        let t = super::super::test_support::zh();
        let faction_id = game.player_faction_id.clone();
        let city_ids = isolate_two_player_cities(game, &faction_id);
        game.factions.get_mut(&faction_id).unwrap().heir_id = Some("guan_yu".to_string());
        game.cities.get_mut(&city_ids[0]).unwrap().governor_id = Some("liu_bei".to_string());
        game.officers.get_mut("liu_bei").unwrap().office_id = Some("taifu".to_string());
        game.officers.get_mut("zhao_yun").unwrap().status = OfficerStatus::Unavailable;

        configure_city_resources(game, &city_ids[0], 10);
        configure_city_resources(game, &city_ids[1], 100);

        let snapshot = faction_detail_snapshot(game, &faction_id, &t).unwrap();

        assert!(snapshot.alive);
        assert_eq!(snapshot.heir_name, "关羽");
        assert_eq!(snapshot.resources.city_count, 2);
        assert_eq!(snapshot.resources.gold, 110);
        assert_eq!(snapshot.resources.food, 220);
        assert_eq!(snapshot.resources.materials, 330);
        assert_eq!(snapshot.resources.population, 770);
        assert_eq!(snapshot.resources.troops, TroopPool::new(440, 550, 660));
        assert_eq!(snapshot.resources.wounded, 66);
        assert!(snapshot.resources.total_officer_count >= snapshot.resources.active_officer_count);
        let ruler = snapshot
            .officers
            .iter()
            .find(|officer| officer.name == "刘备")
            .unwrap();
        assert!(ruler.role_labels.iter().any(|role| role == "君主"));
        assert!(ruler.role_labels.iter().any(|role| role == "太守"));
        assert!(ruler.role_labels.iter().any(|role| role == "太傅"));
    }

    #[test]
    fn faction_detail_snapshot_lists_diplomacy_technology_and_actions() {
        let mut state = ui_state_with_game();
        let game = state.game.as_mut().unwrap();
        let t = super::super::test_support::zh();
        let faction_id = game.player_faction_id.clone();
        let other_faction_id = game
            .factions
            .keys()
            .find(|id| *id != &faction_id && game.faction_alive(id))
            .unwrap()
            .clone();
        let turn = game.turn;
        {
            let relation = game.relation_mut(&faction_id, &other_faction_id);
            relation.score = 42;
            relation.truce_until_turn = Some(turn + 3);
        }
        game.technologies.insert(
            faction_id.clone(),
            FactionTechnologyState {
                active: Some("militia_drill".to_string()),
                progress: BTreeMap::from([("militia_drill".to_string(), 1)]),
                funded: BTreeSet::from(["militia_drill".to_string()]),
                completed: BTreeSet::from(["household_registers".to_string()]),
            },
        );
        game.pending_commands.push(Command {
            issuer_faction_id: faction_id.clone(),
            city_id: "pingyuan".to_string(),
            officer_id: Some("liu_bei".to_string()),
            kind: CommandKind::Train,
        });
        game.pending_commands.push(Command {
            issuer_faction_id: other_faction_id.clone(),
            city_id: "xuchang".to_string(),
            officer_id: Some("cao_cao".to_string()),
            kind: CommandKind::Train,
        });
        game.army_movements.push(ArmyMovement {
            kind: ArmyMovementKind::Transfer,
            issuer_faction_id: faction_id.clone(),
            source_city_id: "pingyuan".to_string(),
            target_city_id: "xiapi".to_string(),
            commander_id: "liu_bei".to_string(),
            officer_ids: vec!["liu_bei".to_string()],
            troops: TroopPool::new(1, 2, 3),
            food_supply: 0,
            wounded_troops: TroopPool::default(),
            assignments: Vec::new(),
            siege_started_turn: None,
            training: 50,
            distance_li: 100,
            departure_turn: game.turn,
            arrival_turn: game.turn + 1,
        });
        game.officer_recruitments.push(OfficerRecruitmentTask {
            id: "task".to_string(),
            issuer_faction_id: faction_id.clone(),
            source_city_id: "pingyuan".to_string(),
            recruiter_officer_id: "liu_bei".to_string(),
            target_officer_id: "zhao_yun".to_string(),
            progress: 35,
            attempt_months: 1,
            started_turn: game.turn,
        });

        let snapshot = faction_detail_snapshot(game, &faction_id, &t).unwrap();

        assert!(
            snapshot.diplomacy.iter().any(
                |row| row.score == 42 && row.truce_label.contains(&(game.turn + 3).to_string())
            )
        );
        assert_eq!(snapshot.technology.active_label, "乡勇操练");
        assert_eq!(snapshot.technology.funded_count, 1);
        assert_eq!(snapshot.technology.completed_count, 1);
        assert_eq!(snapshot.technology.completed_names, vec!["户籍清丈"]);
        assert_eq!(snapshot.pending_commands.len(), 1);
        assert!(snapshot.pending_commands[0].contains("训练"));
        assert_eq!(snapshot.army_movements.len(), 1);
        assert!(snapshot.army_movements[0].contains("刘备"));
        assert_eq!(snapshot.recruitments.len(), 1);
        assert!(snapshot.recruitments[0].contains("赵云"));
    }

    #[test]
    fn faction_detail_snapshot_handles_destroyed_faction() {
        let mut state = ui_state_with_game();
        let game = state.game.as_mut().unwrap();
        let t = super::super::test_support::zh();
        let faction_id = game.player_faction_id.clone();
        for city in game.cities.values_mut() {
            if city.faction_id == faction_id {
                city.faction_id = "cao_cao".to_string();
            }
        }

        let snapshot = faction_detail_snapshot(game, &faction_id, &t).unwrap();

        assert!(!snapshot.alive);
        assert_eq!(snapshot.status_label, "已灭亡");
        assert!(snapshot.cities.is_empty());
    }

    #[test]
    fn faction_row_interaction_selects_and_opens_detail() {
        let mut selected = None;
        let mut detail = None;

        apply_faction_row_interaction(
            "liu_bei",
            &mut selected,
            &mut detail,
            FactionRowInteraction {
                clicked: true,
                ..FactionRowInteraction::default()
            },
        );
        assert_eq!(selected.as_deref(), Some("liu_bei"));
        assert_eq!(detail, None);

        apply_faction_row_interaction(
            "cao_cao",
            &mut selected,
            &mut detail,
            FactionRowInteraction {
                view_clicked: true,
                ..FactionRowInteraction::default()
            },
        );
        assert_eq!(selected.as_deref(), Some("cao_cao"));
        assert_eq!(detail.as_deref(), Some("cao_cao"));

        apply_faction_row_interaction(
            "sun_quan",
            &mut selected,
            &mut detail,
            FactionRowInteraction {
                double_clicked: true,
                ..FactionRowInteraction::default()
            },
        );
        assert_eq!(selected.as_deref(), Some("sun_quan"));
        assert_eq!(detail.as_deref(), Some("sun_quan"));
    }

    fn test_row(id: &str, name: &str, city_count: usize, gold: i64) -> FactionOverviewRow {
        FactionOverviewRow {
            faction_id: id.to_string(),
            faction_name: name.to_string(),
            city_count,
            gold,
            ..FactionOverviewRow::default()
        }
    }

    fn row_ids(rows: &[FactionOverviewRow]) -> Vec<&str> {
        rows.iter().map(|row| row.faction_name.as_str()).collect()
    }

    fn isolate_two_player_cities(game: &mut GameState, faction_id: &str) -> Vec<CityId> {
        let city_ids = game
            .cities
            .values()
            .filter(|city| city.faction_id == faction_id)
            .take(2)
            .map(|city| city.id.clone())
            .collect::<Vec<_>>();
        assert_eq!(city_ids.len(), 2);
        let other_faction_id = game
            .factions
            .keys()
            .find(|id| *id != faction_id)
            .unwrap()
            .clone();
        for city in game.cities.values_mut() {
            if city.faction_id == faction_id && !city_ids.contains(&city.id) {
                city.faction_id = other_faction_id.clone();
            }
        }
        city_ids
    }

    fn configure_city_resources(game: &mut GameState, city_id: &str, base: u32) {
        let city = game.cities.get_mut(city_id).unwrap();
        city.gold = base as i32;
        city.food = (base * 2) as i32;
        city.materials = (base * 3) as i32;
        city.troops = TroopPool::new(base * 4, base * 5, base * 6);
        city.population = base * 7;
        city.wounded_troops = TroopPool::new(base / 10, base / 5, base * 3 / 10);
    }
}
