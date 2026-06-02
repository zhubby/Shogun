use crate::game::*;
use bevy_egui::egui;
use egui_extras::{Column, TableBuilder};
use std::collections::{BTreeMap, BTreeSet};

use super::super::i18n::{Translator, UiLanguage, args};
use super::super::labels::officer_gender_label;
use super::super::state::{
    GameUiState, OfficerBrowserFilters, OfficerGenderFilter, OfficerStatusFilter,
};
use super::super::style::{
    modal_content_width, modal_title_bar, war_gold, war_panel_frame, war_text_muted,
};
use super::officer_common::officer_status_label;
pub(in crate::core) const OFFICER_BROWSER_MODAL_WIDTH: f32 = 1060.0;

pub(super) fn officer_browser_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.officer_browser_open {
        return;
    }

    let width = modal_content_width(screen, OFFICER_BROWSER_MODAL_WIDTH);
    let height = (screen.height() * 0.76).clamp(420.0, 680.0);
    egui::Area::new(egui::Id::new("hud_officer_browser"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("officer-browser-title")) {
                    ui_state.officer_browser_open = false;
                }
                ui.separator();
                if let Some(game) = &ui_state.game {
                    officer_browser_filters(
                        ui,
                        game,
                        &mut ui_state.officer_browser_filters,
                        "hud_officer_browser_filters",
                        t,
                    );
                } else {
                    ui.label(t.text("message-no-game-state"));
                }
                ui.separator();
                if let Some(game) = &ui_state.game {
                    let response = officer_browser_table(
                        ui,
                        game,
                        &ui_state.officer_browser_filters,
                        OfficerBrowserTableOptions {
                            max_height: height - 118.0,
                            id_salt: "hud_officer_browser_table",
                            selected_officer_id: ui_state.officer_browser_selected_id.as_deref(),
                            editable: false,
                            retainer_faction_id: None,
                        },
                        t,
                    );
                    if let Some(officer_id) = response.selected_officer_id {
                        ui_state.officer_browser_selected_id = Some(officer_id);
                    }
                    if let Some(officer_id) = response.view_officer_id {
                        ui_state.officer_detail_id = Some(officer_id);
                    }
                }
            });
        });
}

pub(super) fn retainer_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.retainers_open {
        return;
    }

    let width = modal_content_width(screen, OFFICER_BROWSER_MODAL_WIDTH);
    let height = (screen.height() * 0.76).clamp(420.0, 680.0);
    egui::Area::new(egui::Id::new("hud_retainers"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("retainer-title")) {
                    ui_state.retainers_open = false;
                }
                ui.separator();
                if let Some(game) = &ui_state.game {
                    officer_browser_filters(
                        ui,
                        game,
                        &mut ui_state.retainer_filters,
                        "hud_retainer_filters",
                        t,
                    );
                } else {
                    ui.label(t.text("message-no-game-state"));
                }
                ui.separator();
                let Some(game) = ui_state.game.as_ref() else {
                    return;
                };
                let player_faction_id = game.player_faction_id.clone();
                let response = officer_browser_table(
                    ui,
                    game,
                    &ui_state.retainer_filters,
                    OfficerBrowserTableOptions {
                        max_height: height - 142.0,
                        id_salt: "hud_retainer_table",
                        selected_officer_id: ui_state.retainer_selected_id.as_deref(),
                        editable: false,
                        retainer_faction_id: Some(player_faction_id.as_str()),
                    },
                    t,
                );
                if let Some(officer_id) = response.selected_officer_id.clone() {
                    ui_state.retainer_selected_id = Some(officer_id);
                }
                if let Some(officer_id) = response.view_officer_id {
                    ui_state.officer_detail_id = Some(officer_id);
                }

                if response.appoint_officer_id.is_some() || response.dismiss_officer_id.is_some() {
                    let Some(game) = ui_state.game.as_mut() else {
                        return;
                    };
                    if let Some((officer_id, office_id)) =
                        response.appoint_officer_id.zip(response.appoint_office_id)
                    {
                        match appoint_official_post(
                            game,
                            &player_faction_id,
                            &officer_id,
                            &office_id,
                        ) {
                            Ok(()) => {
                                let (officer_name, loyalty) = game
                                    .officers
                                    .get(&officer_id)
                                    .map(|officer| (officer.name.clone(), officer.loyalty))
                                    .unwrap_or((officer_id, 0));
                                let office_name = official_post_spec(&office_id)
                                    .map(|spec| spec.name)
                                    .unwrap_or(office_id.as_str());
                                ui_state.message = t.text_args(
                                    "message-officer-appointed",
                                    &args([
                                        ("officer", officer_name),
                                        ("office", office_name.to_string()),
                                        ("loyalty", loyalty.to_string()),
                                    ]),
                                );
                            }
                            Err(error) => ui_state.message = error.to_string(),
                        }
                    } else if let Some(officer_id) = response.dismiss_officer_id {
                        match dismiss_official_post(game, &player_faction_id, &officer_id) {
                            Ok(()) => {
                                let (officer_name, loyalty) = game
                                    .officers
                                    .get(&officer_id)
                                    .map(|officer| (officer.name.clone(), officer.loyalty))
                                    .unwrap_or((officer_id, 0));
                                ui_state.message = t.text_args(
                                    "message-officer-dismissed",
                                    &args([
                                        ("officer", officer_name),
                                        ("loyalty", loyalty.to_string()),
                                    ]),
                                );
                            }
                            Err(error) => ui_state.message = error.to_string(),
                        }
                    }
                }
            });
        });
}

pub(in crate::core) fn officer_browser_filters(
    ui: &mut egui::Ui,
    game: &GameState,
    filters: &mut OfficerBrowserFilters,
    id_salt: &'static str,
    t: &Translator,
) {
    const FILTER_HEIGHT: f32 = 30.0;
    ui.set_max_width(ui.available_width());
    egui::ScrollArea::horizontal()
        .id_salt((id_salt, "filter_scroll"))
        .scroll_bar_visibility(egui::containers::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .max_height(FILTER_HEIGHT + 4.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            officer_browser_filter_controls(ui, game, filters, id_salt, t, FILTER_HEIGHT);
        });
}

fn officer_browser_filter_controls(
    ui: &mut egui::Ui,
    game: &GameState,
    filters: &mut OfficerBrowserFilters,
    id_salt: &'static str,
    t: &Translator,
    filter_height: f32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().interact_size.y = filter_height;
        ui.spacing_mut().item_spacing.x = 12.0;
        ui.label(t.text("officer-filter-search"));
        ui.add_sized(
            [260.0, filter_height],
            egui::TextEdit::singleline(&mut filters.search)
                .hint_text(t.text("officer-filter-search-hint")),
        );

        egui::ComboBox::from_id_salt((id_salt, "gender"))
            .width(160.0)
            .selected_text(officer_gender_filter_label(filters.gender, t))
            .show_ui(ui, |ui| {
                for filter in [
                    OfficerGenderFilter::All,
                    OfficerGenderFilter::Male,
                    OfficerGenderFilter::Female,
                ] {
                    ui.selectable_value(
                        &mut filters.gender,
                        filter,
                        officer_gender_filter_label(filter, t),
                    );
                }
            });

        let selected_faction_text = filters
            .faction_id
            .as_deref()
            .and_then(|id| game.factions.get(id))
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| t.text("officer-filter-all-factions"));
        egui::ComboBox::from_id_salt((id_salt, "faction"))
            .width(170.0)
            .selected_text(selected_faction_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut filters.faction_id,
                    None,
                    t.text("officer-filter-all-factions"),
                );
                for faction in game.factions.values() {
                    ui.selectable_value(
                        &mut filters.faction_id,
                        Some(faction.id.clone()),
                        &faction.name,
                    );
                }
            });

        egui::ComboBox::from_id_salt((id_salt, "status"))
            .width(170.0)
            .selected_text(officer_status_filter_label(filters.status, t))
            .show_ui(ui, |ui| {
                for filter in [
                    OfficerStatusFilter::All,
                    OfficerStatusFilter::Active,
                    OfficerStatusFilter::Minor,
                    OfficerStatusFilter::Wild,
                    OfficerStatusFilter::Unavailable,
                    OfficerStatusFilter::Dead,
                ] {
                    ui.selectable_value(
                        &mut filters.status,
                        filter,
                        officer_status_filter_label(filter, t),
                    );
                }
            });

        let selected_city_text = filters
            .city_id
            .as_deref()
            .and_then(|id| game.cities.get(id))
            .map(|city| city.name.clone())
            .unwrap_or_else(|| t.text("officer-filter-all-cities"));
        egui::ComboBox::from_id_salt((id_salt, "city"))
            .width(170.0)
            .selected_text(selected_city_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut filters.city_id,
                    None,
                    t.text("officer-filter-all-cities"),
                );
                let mut cities: Vec<_> = game.cities.values().collect();
                cities.sort_by(|a, b| a.name.cmp(&b.name));
                for city in cities {
                    ui.selectable_value(&mut filters.city_id, Some(city.id.clone()), &city.name);
                }
            });

        let selected_tag_text = selected_tag_filter_text(game, filters, t);
        egui::ComboBox::from_id_salt((id_salt, "tags"))
            .width(190.0)
            .selected_text(selected_tag_text)
            .show_ui(ui, |ui| {
                if ui.button(t.text("officer-filter-clear-tags")).clicked() {
                    filters.tag_ids.clear();
                    ui.close();
                }
                let definitions_by_category = officer_tag_definitions_by_category(game);
                for (category, definitions) in definitions_by_category {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(officer_tag_category_label(category, t))
                            .color(war_text_muted()),
                    );
                    for definition in definitions {
                        let mut selected = filters.tag_ids.contains(&definition.id);
                        if ui
                            .checkbox(&mut selected, officer_tag_label(definition, t))
                            .changed()
                        {
                            if selected {
                                filters.tag_ids.insert(definition.id.clone());
                            } else {
                                filters.tag_ids.remove(&definition.id);
                            }
                        }
                    }
                }
            });

        if ui
            .add_sized(
                [filter_height, filter_height],
                egui::Button::new(
                    egui::RichText::new(egui_phosphor::regular::ARROW_COUNTER_CLOCKWISE)
                        .size(17.0)
                        .color(war_gold()),
                ),
            )
            .on_hover_text(t.text("common-reset"))
            .clicked()
        {
            filters.reset();
        }
    });
}

pub(in crate::core) fn officer_browser_table(
    ui: &mut egui::Ui,
    game: &GameState,
    filters: &OfficerBrowserFilters,
    options: OfficerBrowserTableOptions<'_>,
    t: &Translator,
) -> OfficerBrowserTableResponse {
    let rows = options.retainer_faction_id.map_or_else(
        || filtered_officer_rows(filters, game, t),
        |faction_id| retainer_officer_rows(filters, game, faction_id, t),
    );
    let mut table_response = OfficerBrowserTableResponse::default();
    ui.label(t.text_args(
        "officer-browser-count",
        &args([("count", rows.len().to_string())]),
    ));

    let table_height = options.max_height.max(260.0);
    TableBuilder::new(ui)
        .id_salt(options.id_salt)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .min_scrolled_height(table_height)
        .max_scroll_height(table_height)
        .auto_shrink([false, false])
        .column(Column::exact(54.0))
        .column(Column::remainder().at_least(96.0).clip(true))
        .column(Column::exact(52.0))
        .column(Column::exact(46.0))
        .column(Column::remainder().at_least(96.0).clip(true))
        .column(Column::initial(82.0).at_least(64.0).clip(true))
        .column(Column::initial(88.0).at_least(68.0).clip(true))
        .column(Column::exact(58.0))
        .column(Column::initial(72.0).at_least(58.0).clip(true))
        .column(Column::exact(58.0))
        .columns(Column::exact(52.0), 5)
        .header(26.0, |mut header| {
            officer_table_header_cell_centered(&mut header, t.text("officer-column-tools"));
            officer_table_header_cell(&mut header, t.text("officer-column-name"));
            officer_table_header_cell(&mut header, t.text("officer-column-gender"));
            officer_table_header_cell(&mut header, t.text("officer-column-age"));
            officer_table_header_cell(&mut header, t.text("officer-column-faction"));
            officer_table_header_cell(&mut header, t.text("officer-column-city"));
            officer_table_header_cell(&mut header, t.text("officer-column-office"));
            officer_table_header_cell(&mut header, t.text("officer-column-salary"));
            officer_table_header_cell(&mut header, t.text("officer-column-status"));
            officer_table_header_cell(&mut header, t.text("officer-column-loyalty"));
            officer_table_header_cell(&mut header, t.text("stat-leadership"));
            officer_table_header_cell(&mut header, t.text("stat-strength"));
            officer_table_header_cell(&mut header, t.text("stat-intelligence"));
            officer_table_header_cell(&mut header, t.text("stat-politics"));
            officer_table_header_cell(&mut header, t.text("stat-charm"));
        })
        .body(|mut body| {
            for row in rows {
                let selected = options.selected_officer_id == Some(row.id.as_str());
                body.row(30.0, |mut table_row| {
                    table_row.set_selected(selected);
                    table_row.col(|ui| officer_row_tools(ui, &row, &mut table_response, t));
                    table_row.col(|ui| officer_row_cell(ui, &row.name));
                    table_row.col(|ui| officer_row_cell(ui, &row.gender));
                    table_row.col(|ui| officer_row_cell(ui, row.age.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, &row.faction_name));
                    table_row.col(|ui| officer_row_cell(ui, &row.city_name));
                    table_row.col(|ui| officer_row_cell(ui, &row.office_name));
                    table_row.col(|ui| officer_row_cell(ui, row.salary.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, &row.status));
                    table_row.col(|ui| officer_row_cell(ui, row.loyalty.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, row.leadership.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, row.strength.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, row.intelligence.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, row.politics.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, row.charm.to_string()));

                    let row_response = table_row.response();
                    if row_response.clicked() || row_response.secondary_clicked() {
                        table_response.selected_officer_id = Some(row.id.clone());
                    }
                    if options.editable || options.retainer_faction_id.is_some() {
                        row_response.context_menu(|ui| {
                            officer_row_context_menu(
                                ui,
                                game,
                                &row,
                                &mut table_response,
                                options.editable,
                                options.retainer_faction_id,
                                t,
                            );
                        });
                    }
                });
            }
        });
    table_response
}

pub(in crate::core) fn officer_tag_label(
    definition: &OfficerTagDefinition,
    t: &Translator,
) -> String {
    match t.language() {
        UiLanguage::English => definition.label_en.clone(),
        UiLanguage::SimplifiedChinese => definition.label_zh.clone(),
    }
}

pub(super) fn officer_tag_display_text(game: &GameState, tag_id: &str, t: &Translator) -> String {
    game.officer_tag_definitions
        .iter()
        .find(|definition| definition.id == tag_id)
        .map(|definition| officer_tag_label(definition, t))
        .unwrap_or_else(|| tag_id.to_string())
}

fn selected_tag_filter_text(
    game: &GameState,
    filters: &OfficerBrowserFilters,
    t: &Translator,
) -> String {
    match filters.tag_ids.len() {
        0 => t.text("officer-filter-all-tags"),
        1 => filters
            .tag_ids
            .iter()
            .next()
            .map(|tag_id| officer_tag_display_text(game, tag_id, t))
            .unwrap_or_else(|| t.text("officer-filter-all-tags")),
        count => t.text_args(
            "officer-filter-selected-tags",
            &args([("count", count.to_string())]),
        ),
    }
}

pub(in crate::core) fn officer_tag_definitions_by_category(
    game: &GameState,
) -> BTreeMap<OfficerTagCategory, Vec<&OfficerTagDefinition>> {
    let mut grouped: BTreeMap<OfficerTagCategory, Vec<&OfficerTagDefinition>> = BTreeMap::new();
    for definition in &game.officer_tag_definitions {
        grouped
            .entry(definition.category)
            .or_default()
            .push(definition);
    }
    grouped
}

pub(in crate::core) fn officer_tag_category_label(
    category: OfficerTagCategory,
    t: &Translator,
) -> String {
    let key = match category {
        OfficerTagCategory::Role => "officer-tag-category-role",
        OfficerTagCategory::Affiliation => "officer-tag-category-affiliation",
        OfficerTagCategory::Source => "officer-tag-category-source",
        OfficerTagCategory::Batch => "officer-tag-category-batch",
        OfficerTagCategory::Basis => "officer-tag-category-basis",
        OfficerTagCategory::Region => "officer-tag-category-region",
        OfficerTagCategory::Context => "officer-tag-category-context",
    };
    t.text(key)
}

#[derive(Clone, Copy, Debug)]
pub(in crate::core) struct OfficerBrowserTableOptions<'a> {
    pub(in crate::core) max_height: f32,
    pub(in crate::core) id_salt: &'static str,
    pub(in crate::core) selected_officer_id: Option<&'a str>,
    pub(in crate::core) editable: bool,
    pub(in crate::core) retainer_faction_id: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub(super) struct OfficerBrowserRow {
    id: OfficerId,
    name: String,
    faction_id: FactionId,
    gender: String,
    age: u32,
    faction_name: String,
    city_name: String,
    office_name: String,
    salary: i32,
    status: String,
    loyalty: u8,
    leadership: u8,
    strength: u8,
    intelligence: u8,
    politics: u8,
    charm: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::core) struct OfficerBrowserTableResponse {
    pub(in crate::core) selected_officer_id: Option<OfficerId>,
    pub(in crate::core) view_officer_id: Option<OfficerId>,
    pub(in crate::core) edit_officer_id: Option<OfficerId>,
    pub(super) appoint_officer_id: Option<OfficerId>,
    pub(super) appoint_office_id: Option<OfficialPostId>,
    pub(super) dismiss_officer_id: Option<OfficerId>,
}

fn officer_table_header_cell(row: &mut egui_extras::TableRow<'_, '_>, text: String) {
    row.col(|ui| {
        ui.strong(text);
    });
}

fn officer_table_header_cell_centered(row: &mut egui_extras::TableRow<'_, '_>, text: String) {
    row.col(|ui| {
        ui.centered_and_justified(|ui| {
            ui.strong(text);
        });
    });
}

fn officer_row_tools(
    ui: &mut egui::Ui,
    row: &OfficerBrowserRow,
    table_response: &mut OfficerBrowserTableResponse,
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
        .on_hover_text(t.text("officer-action-view"));
    if response.clicked() {
        table_response.selected_officer_id = Some(row.id.clone());
        table_response.view_officer_id = Some(row.id.clone());
    }
}

fn officer_row_cell(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) {
    ui.add(egui::Label::new(text).truncate());
}

fn officer_row_context_menu(
    ui: &mut egui::Ui,
    game: &GameState,
    row: &OfficerBrowserRow,
    table_response: &mut OfficerBrowserTableResponse,
    editable: bool,
    retainer_faction_id: Option<&str>,
    t: &Translator,
) {
    if editable && ui.button(t.text("officer-action-edit")).clicked() {
        table_response.selected_officer_id = Some(row.id.clone());
        table_response.edit_officer_id = Some(row.id.clone());
        ui.close();
    }
    if retainer_faction_id.is_some() {
        ui.menu_button(t.text("officer-action-appoint-office"), |ui| {
            for spec in official_post_specs() {
                let occupant = game.officers.values().find(|officer| {
                    officer.faction_id == row.faction_id
                        && officer.office_id.as_deref() == Some(spec.id)
                });
                let label = if let Some(occupant) = occupant {
                    if occupant.id == row.id {
                        t.text_args(
                            "official-post-current",
                            &args([
                                ("name", spec.name.to_string()),
                                ("rank", official_rank_label(spec.rank).to_string()),
                            ]),
                        )
                    } else {
                        t.text_args(
                            "official-post-occupied",
                            &args([
                                ("name", spec.name.to_string()),
                                ("rank", official_rank_label(spec.rank).to_string()),
                                ("occupant", occupant.name.clone()),
                            ]),
                        )
                    }
                } else {
                    t.text_args(
                        "official-post-empty",
                        &args([
                            ("name", spec.name.to_string()),
                            ("rank", official_rank_label(spec.rank).to_string()),
                        ]),
                    )
                };
                if ui.button(label).clicked() {
                    table_response.selected_officer_id = Some(row.id.clone());
                    table_response.appoint_officer_id = Some(row.id.clone());
                    table_response.appoint_office_id = Some(spec.id.to_string());
                    ui.close();
                }
            }
        });
        if row.office_name != t.text("none")
            && ui.button(t.text("officer-action-dismiss-office")).clicked()
        {
            table_response.selected_officer_id = Some(row.id.clone());
            table_response.dismiss_officer_id = Some(row.id.clone());
            ui.close();
        }
    }
}

pub(super) fn filtered_officer_rows(
    filters: &OfficerBrowserFilters,
    game: &GameState,
    t: &Translator,
) -> Vec<OfficerBrowserRow> {
    let search = filters.search.trim().to_lowercase();
    let officers: Vec<_> = game.officers.values().collect();
    let mut rows: Vec<_> = officers
        .into_iter()
        .filter(|officer| officer_matches_filters(officer, game, filters, &search))
        .map(|officer| {
            let faction_name = game
                .factions
                .get(&officer.faction_id)
                .map(|faction| faction.name.clone())
                .unwrap_or_else(|| t.text("unknown"));
            let city_name = officer
                .city_id
                .as_deref()
                .and_then(|city_id| game.cities.get(city_id))
                .map(|city| city.name.clone())
                .unwrap_or_else(|| t.text("officer-city-unassigned"));
            let office_name = officer
                .office_id
                .as_deref()
                .and_then(official_post_spec)
                .map(|spec| spec.name.to_string())
                .unwrap_or_else(|| t.text("none"));
            OfficerBrowserRow {
                id: officer.id.clone(),
                name: officer.name.clone(),
                faction_id: officer.faction_id.clone(),
                gender: officer_gender_label(t, &officer.gender),
                age: officer.age_at(game.year),
                faction_name,
                city_name,
                office_name,
                salary: officer_monthly_salary(officer),
                status: officer_status_label(&officer.status, t),
                loyalty: officer.loyalty,
                leadership: officer.stats.leadership,
                strength: officer.stats.strength,
                intelligence: officer.stats.intelligence,
                politics: officer.stats.politics,
                charm: officer.stats.charm,
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        (&a.faction_name, &a.city_name, &a.name, &a.id).cmp(&(
            &b.faction_name,
            &b.city_name,
            &b.name,
            &b.id,
        ))
    });
    rows
}

pub(super) fn retainer_officer_rows(
    filters: &OfficerBrowserFilters,
    game: &GameState,
    faction_id: &str,
    t: &Translator,
) -> Vec<OfficerBrowserRow> {
    let mut locked_filters = filters.clone();
    locked_filters.faction_id = Some(faction_id.to_string());
    locked_filters.status = OfficerStatusFilter::Active;
    filtered_officer_rows(&locked_filters, game, t)
}

fn officer_matches_filters(
    officer: &Officer,
    game: &GameState,
    filters: &OfficerBrowserFilters,
    search: &str,
) -> bool {
    if !officer_gender_matches(&officer.gender, filters.gender) {
        return false;
    }
    if filters
        .faction_id
        .as_ref()
        .is_some_and(|faction_id| officer.faction_id != *faction_id)
    {
        return false;
    }
    if !officer_status_matches(&officer.status, filters.status) {
        return false;
    }
    if filters
        .city_id
        .as_ref()
        .is_some_and(|city_id| officer.city_id.as_deref() != Some(city_id.as_str()))
    {
        return false;
    }
    if !officer_tags_match_filters(officer, game, filters) {
        return false;
    }
    search.is_empty() || officer_search_text(officer, game).contains(search)
}

fn officer_tags_match_filters(
    officer: &Officer,
    game: &GameState,
    filters: &OfficerBrowserFilters,
) -> bool {
    if filters.tag_ids.is_empty() {
        return true;
    }
    let officer_tags = normalized_officer_tag_ids(officer, game);
    let definition_by_id = game
        .officer_tag_definitions
        .iter()
        .map(|definition| (definition.id.as_str(), definition.category))
        .collect::<BTreeMap<_, _>>();
    let mut selected_by_category: BTreeMap<OfficerTagCategory, BTreeSet<&str>> = BTreeMap::new();
    for tag_id in &filters.tag_ids {
        let Some(category) = definition_by_id.get(tag_id.as_str()).copied() else {
            return false;
        };
        selected_by_category
            .entry(category)
            .or_default()
            .insert(tag_id.as_str());
    }
    selected_by_category
        .values()
        .all(|selected| selected.iter().any(|tag_id| officer_tags.contains(*tag_id)))
}

fn normalized_officer_tag_ids(officer: &Officer, game: &GameState) -> BTreeSet<String> {
    officer
        .profile
        .as_ref()
        .into_iter()
        .flat_map(|profile| profile.tags.iter())
        .filter_map(|tag| {
            if game
                .officer_tag_definitions
                .iter()
                .any(|definition| definition.id == *tag)
            {
                Some(tag.clone())
            } else {
                game.officer_tag_aliases.get(tag).cloned()
            }
        })
        .collect()
}

fn officer_search_text(officer: &Officer, game: &GameState) -> String {
    let faction_name = game
        .factions
        .get(&officer.faction_id)
        .map(|faction| faction.name.as_str())
        .unwrap_or("未知");
    let city_name = officer
        .city_id
        .as_deref()
        .and_then(|city_id| game.cities.get(city_id))
        .map(|city| city.name.as_str())
        .unwrap_or("未配置");
    let mut text = format!(
        "{} {} {} {}",
        officer.name, officer.id, faction_name, city_name
    );
    if let Some(profile) = &officer.profile {
        if let Some(courtesy) = &profile.courtesy_name {
            text.push(' ');
            text.push_str(courtesy);
        }
        if let Some(native_place) = &profile.native_place {
            text.push(' ');
            text.push_str(native_place);
        }
        for tag_id in normalized_officer_tag_ids(officer, game) {
            text.push(' ');
            text.push_str(&tag_id);
            if let Some(definition) = game
                .officer_tag_definitions
                .iter()
                .find(|definition| definition.id == tag_id)
            {
                text.push(' ');
                text.push_str(&definition.label_zh);
                text.push(' ');
                text.push_str(&definition.label_en);
            }
        }
    }
    if let Some(spec) = officer.office_id.as_deref().and_then(official_post_spec) {
        text.push(' ');
        text.push_str(spec.name);
        text.push(' ');
        text.push_str(official_rank_label(spec.rank));
    }
    text.to_lowercase()
}

fn officer_gender_matches(gender: &OfficerGender, filter: OfficerGenderFilter) -> bool {
    match filter {
        OfficerGenderFilter::All => true,
        OfficerGenderFilter::Male => *gender == OfficerGender::Male,
        OfficerGenderFilter::Female => *gender == OfficerGender::Female,
    }
}

fn officer_status_matches(status: &OfficerStatus, filter: OfficerStatusFilter) -> bool {
    match filter {
        OfficerStatusFilter::All => true,
        OfficerStatusFilter::Active => *status == OfficerStatus::Active,
        OfficerStatusFilter::Minor => *status == OfficerStatus::Minor,
        OfficerStatusFilter::Wild => *status == OfficerStatus::Wild,
        OfficerStatusFilter::Unavailable => *status == OfficerStatus::Unavailable,
        OfficerStatusFilter::Dead => *status == OfficerStatus::Dead,
    }
}

fn officer_gender_filter_label(filter: OfficerGenderFilter, t: &Translator) -> String {
    match filter {
        OfficerGenderFilter::All => t.text("officer-filter-all-genders"),
        OfficerGenderFilter::Male => t.text("gender-male"),
        OfficerGenderFilter::Female => t.text("gender-female"),
    }
}

fn officer_status_filter_label(filter: OfficerStatusFilter, t: &Translator) -> String {
    match filter {
        OfficerStatusFilter::All => t.text("officer-filter-all-statuses"),
        OfficerStatusFilter::Active => t.text("officer-status-active"),
        OfficerStatusFilter::Minor => t.text("officer-status-minor"),
        OfficerStatusFilter::Wild => t.text("officer-status-wild"),
        OfficerStatusFilter::Unavailable => t.text("officer-status-unavailable"),
        OfficerStatusFilter::Dead => t.text("officer-status-dead"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::state::{OfficerGenderFilter, OfficerStatusFilter};
    use super::super::test_support::{ui_state_with_game, zh};
    use super::*;
    #[test]
    fn officer_browser_search_matches_name_id_faction_and_city() {
        let mut state = ui_state_with_game();
        let game = state.game.as_ref().unwrap();

        state.officer_browser_filters.search = "关羽".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game, &zh())
                .iter()
                .any(|row| row.name == "关羽")
        );

        state.officer_browser_filters.search = "guan_yu".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game, &zh())
                .iter()
                .any(|row| row.id == "guan_yu" && row.name == "关羽")
        );

        state.officer_browser_filters.search = "刘备军".to_string();
        assert!(!filtered_officer_rows(&state.officer_browser_filters, game, &zh()).is_empty());

        state.officer_browser_filters.search = "平原".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game, &zh())
                .iter()
                .all(|row| row.city_name == "平原")
        );
    }

    #[test]
    fn officer_browser_filters_can_be_combined() {
        let mut state = ui_state_with_game();
        {
            let game = state.game.as_mut().unwrap();
            let officer = game.officers.get_mut("guan_yu").unwrap();
            officer.gender = OfficerGender::Male;
        }
        state.officer_browser_filters.gender = OfficerGenderFilter::Male;
        state.officer_browser_filters.faction_id = Some("liu_bei".to_string());
        state.officer_browser_filters.status = OfficerStatusFilter::Active;
        state.officer_browser_filters.city_id = Some("pingyuan".to_string());

        let game = state.game.as_ref().unwrap();
        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());

        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| {
            row.gender == "男"
                && row.faction_name == "刘备军"
                && row.status == "在任"
                && row.city_name == "平原"
        }));
    }

    #[test]
    fn officer_browser_tag_filters_use_or_within_category_and_and_across_categories() {
        let mut state = ui_state_with_game();
        state
            .officer_browser_filters
            .tag_ids
            .insert("role:general".to_string());
        state
            .officer_browser_filters
            .tag_ids
            .insert("role:administrator".to_string());
        state
            .officer_browser_filters
            .tag_ids
            .insert("affiliation:shu_han".to_string());

        let game = state.game.as_ref().unwrap();
        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());

        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| {
            let officer = &game.officers[&row.id];
            let tags = normalized_officer_tag_ids(officer, game);
            (tags.contains("role:general") || tags.contains("role:administrator"))
                && tags.contains("affiliation:shu_han")
        }));
    }

    #[test]
    fn officer_browser_tag_filter_matches_legacy_profile_tags_and_resets() {
        let mut state = ui_state_with_game();
        {
            let game = state.game.as_mut().unwrap();
            let officer = game.officers.get_mut("liu_bei").unwrap();
            let profile = officer.profile.as_mut().unwrap();
            profile.tags = vec!["ruler".to_string(), "shu_han".to_string()];
        }
        state
            .officer_browser_filters
            .tag_ids
            .insert("role:ruler".to_string());
        state
            .officer_browser_filters
            .tag_ids
            .insert("affiliation:shu_han".to_string());

        let game = state.game.as_ref().unwrap();
        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());

        assert!(rows.iter().any(|row| row.id == "liu_bei"));
        state.officer_browser_filters.reset();
        assert!(state.officer_browser_filters.tag_ids.is_empty());
    }

    #[test]
    fn officer_browser_empty_filters_return_all_officers_stably_sorted() {
        let state = ui_state_with_game();
        let game = state.game.as_ref().unwrap();

        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());
        let sorted_names = rows.windows(2).all(|pair| {
            (
                &pair[0].faction_name,
                &pair[0].city_name,
                &pair[0].name,
                &pair[0].id,
            ) <= (
                &pair[1].faction_name,
                &pair[1].city_name,
                &pair[1].name,
                &pair[1].id,
            )
        });

        assert_eq!(rows.len(), game.officers.len());
        assert!(sorted_names);
    }

    #[test]
    fn officer_browser_rows_reflect_updated_officer_profile_fields() {
        let mut state = ui_state_with_game();
        {
            let game = state.game.as_mut().unwrap();
            let officer = game.officers.get_mut("guan_yu").unwrap();
            officer.name = "关羽改".to_string();
            officer.gender = OfficerGender::Female;
            officer.stats.leadership = 91;
            officer.stats.strength = 92;
        }
        state.officer_browser_filters.search = "guan_yu".to_string();

        let game = state.game.as_ref().unwrap();
        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "guan_yu");
        assert_eq!(rows[0].name, "关羽改");
        assert_eq!(rows[0].gender, "女");
        assert_eq!(rows[0].leadership, 91);
        assert_eq!(rows[0].strength, 92);
    }

    #[test]
    fn officer_browser_rows_include_office_and_salary() {
        let mut state = ui_state_with_game();
        let base_salary;
        {
            let game = state.game.as_mut().unwrap();
            base_salary = officer_base_monthly_salary(&game.officers["guan_yu"]);
            appoint_official_post(game, "liu_bei", "guan_yu", "taifu").unwrap();
        }
        state.officer_browser_filters.search = "万石".to_string();

        let game = state.game.as_ref().unwrap();
        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());

        assert!(rows.iter().any(|row| {
            row.id == "guan_yu"
                && row.office_name == "太傅"
                && row.salary == base_salary + official_rank_salary_bonus(OfficialRank::WanShi)
        }));
    }

    #[test]
    fn retainer_rows_only_return_player_active_officers() {
        let mut state = ui_state_with_game();
        state.retainer_filters.faction_id = Some("cao_cao".to_string());
        state.retainer_filters.status = OfficerStatusFilter::Dead;
        {
            let game = state.game.as_mut().unwrap();
            game.officers.get_mut("jian_yong").unwrap().status = OfficerStatus::Unavailable;
        }

        let game = state.game.as_ref().unwrap();
        let rows = retainer_officer_rows(
            &state.retainer_filters,
            game,
            &game.player_faction_id,
            &zh(),
        );

        assert!(!rows.is_empty());
        assert!(
            rows.iter()
                .all(|row| row.faction_id == game.player_faction_id && row.status == "在任")
        );
        assert!(rows.iter().all(|row| row.id != "cao_cao"));
        assert!(rows.iter().all(|row| row.id != "jian_yong"));
    }
}
