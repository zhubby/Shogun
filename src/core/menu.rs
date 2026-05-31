use bevy_egui::egui;

use crate::build_info::menu_build_label;
use crate::game::{
    Controller, Faction, GameState, HistoricalCatalog, Officer, OfficerCatalog, OfficerGender,
    OfficerProfile, OfficerProfileUpdate, OfficerStats, OfficerStatus, SourceConfidence,
    SqliteHistoricalCatalog,
};

use super::actions::{enter_game, refresh_saves, start_history_game, start_json_game};
use super::hud::{officer_browser_filters, officer_browser_table};
use super::labels::{confidence_label, officer_gender_label};
use super::settings::settings_modal;
use super::state::{GameUiState, OfficerEditDraft, refresh_history_factions, refresh_history_menu};
use super::style::{
    draw_menu_background, modal_title_bar, war_bar_frame, war_gold, war_panel_frame,
    war_sub_panel_frame, war_text_muted,
};
use super::{HUD_MARGIN, HUD_TOP_OFFSET};

pub(super) fn main_menu(ctx: &egui::Context, ui_state: &mut GameUiState) -> bool {
    let mut apply_display_settings = false;
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter_at(rect);
            draw_menu_background(&painter, rect);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(34.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("大将军")
                                .size(42.0)
                                .color(war_gold())
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("Shogun")
                                .size(18.0)
                                .color(war_text_muted()),
                        );
                        ui.label(
                            egui::RichText::new(menu_build_label())
                                .size(12.0)
                                .color(war_text_muted()),
                        );
                    });
                    ui.add_space(24.0);

                    let total_width = (ui.available_width() - HUD_MARGIN * 2.0).min(1060.0);
                    let stacked_menu = total_width < 900.0;
                    let panel_width = if stacked_menu {
                        total_width
                    } else {
                        (total_width - 18.0) * 0.5
                    };
                    let left_pad = ((ui.available_width() - total_width) * 0.5).max(HUD_MARGIN);

                    if stacked_menu {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.add_space(left_pad);
                            war_panel_frame().show(ui, |ui| {
                                ui.set_width(panel_width);
                                new_game_menu(ui, ui_state);
                            });
                        });
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.add_space(left_pad);
                            war_panel_frame().show(ui, |ui| {
                                ui.set_width(panel_width);
                                load_game_menu(ui, ui_state);
                            });
                        });
                    } else {
                        ui.horizontal_top(|ui| {
                            ui.add_space(left_pad);
                            war_panel_frame().show(ui, |ui| {
                                ui.set_width(panel_width);
                                new_game_menu(ui, ui_state);
                            });
                            ui.add_space(18.0);
                            war_panel_frame().show(ui, |ui| {
                                ui.set_width(panel_width);
                                load_game_menu(ui, ui_state);
                            });
                        });
                    }

                    if !ui_state.message.is_empty() {
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            ui.add_space(left_pad);
                            war_panel_frame().show(ui, |ui| {
                                ui.set_width(total_width);
                                ui.colored_label(war_gold(), &ui_state.message);
                            });
                        });
                    }
                });
        });
    main_menu_settings_button(ctx, ui_state);
    if ui_state.settings_open {
        apply_display_settings |= settings_modal(ctx, ui_state);
    }
    if ui_state.officer_settings_open {
        officer_settings_modal(ctx, ui_state);
    }
    if ui_state.officer_edit_open {
        officer_profile_edit_modal(ctx, ui_state);
    }
    apply_display_settings
}

pub(super) fn new_game_menu(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(ui.available_width());
        ui.heading(egui::RichText::new("新开局").color(war_gold()));
        ui.add_space(8.0);
        if !ui_state.history_scenarios.is_empty() {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("剧本").color(war_text_muted()));
                if ui.button("刷新资料库").clicked() {
                    refresh_history_menu(ui_state);
                }
            });
            let mut scenario_changed = false;
            egui::ScrollArea::vertical()
                .id_salt("main_menu_scenarios")
                .max_height(190.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for scenario in ui_state.history_scenarios.clone() {
                        let label = format!(
                            "{} ({}年{}月)",
                            scenario.name, scenario.year, scenario.month
                        );
                        if ui
                            .radio_value(&mut ui_state.selected_scenario_id, scenario.id, label)
                            .changed()
                        {
                            scenario_changed = true;
                        }
                    }
                });
            if scenario_changed {
                refresh_history_factions(ui_state);
            }

            ui.add_space(10.0);
            ui.label(egui::RichText::new("势力").color(war_text_muted()));
            egui::ScrollArea::vertical()
                .id_salt("main_menu_factions")
                .max_height(160.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for faction in ui_state
                        .history_factions
                        .iter()
                        .filter(|faction| faction.selectable)
                        .cloned()
                        .collect::<Vec<_>>()
                    {
                        ui.radio_value(&mut ui_state.selected_faction_id, faction.id, faction.name);
                    }
                });
            ui.add_space(12.0);
            if ui
                .add_sized([ui.available_width(), 34.0], egui::Button::new("开始游戏"))
                .clicked()
            {
                start_history_game(ui_state);
            }
        } else {
            ui.label(egui::RichText::new("选择势力").color(war_text_muted()));
            for faction_id in &ui_state.json_scenario.player_selectable_factions {
                let faction_name = ui_state
                    .json_scenario
                    .factions
                    .iter()
                    .find(|faction| &faction.id == faction_id)
                    .map(|faction| faction.name.as_str())
                    .unwrap_or(faction_id);
                ui.radio_value(
                    &mut ui_state.selected_faction_id,
                    faction_id.clone(),
                    faction_name,
                );
            }
            ui.add_space(12.0);
            if ui
                .add_sized(
                    [ui.available_width(), 34.0],
                    egui::Button::new("开始兼容小剧本"),
                )
                .clicked()
            {
                start_json_game(ui_state);
            }
        }
    });
}

pub(super) fn load_game_menu(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(ui.available_width());
        ui.heading(egui::RichText::new("读取存档").color(war_gold()));
        ui.label(
            egui::RichText::new(format!(
                "目录: {}",
                ui_state.save_manager.base_dir().display()
            ))
            .color(war_text_muted()),
        );
        if ui.button("刷新存档列表").clicked() {
            refresh_saves(ui_state);
        }
        ui.add_space(8.0);
        let slots = ui_state.save_slots.clone();
        if slots.is_empty() {
            ui.label("暂无存档");
        }
        egui::ScrollArea::vertical()
            .id_salt("main_menu_saves")
            .max_height(430.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for slot in slots {
                    war_sub_panel_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(format!(
                            "{} - {}年{}月 第{}回合",
                            slot.display_name, slot.year, slot.month, slot.turn
                        ));
                        ui.horizontal(|ui| {
                            if ui.button("读取").clicked() {
                                match ui_state.save_manager.load_slot(&slot.slot_id) {
                                    Ok(game) => enter_game(
                                        ui_state,
                                        game,
                                        format!("读取存档 {}", slot.display_name),
                                    ),
                                    Err(error) => {
                                        let _ = ui_state.save_manager.delete_slot(&slot.slot_id);
                                        refresh_saves(ui_state);
                                        ui_state.message = format!("存档已失效，已丢弃: {error}");
                                    }
                                }
                            }
                            if ui.button("删除").clicked() {
                                match ui_state.save_manager.delete_slot(&slot.slot_id) {
                                    Ok(()) => {
                                        refresh_saves(ui_state);
                                        ui_state.message =
                                            format!("删除存档 {}", slot.display_name);
                                    }
                                    Err(error) => ui_state.message = error.to_string(),
                                }
                            }
                        });
                    });
                    ui.add_space(6.0);
                }
            });
    });
}

pub(super) fn main_menu_settings_button(ctx: &egui::Context, ui_state: &mut GameUiState) {
    if ui_state.settings_open || ui_state.officer_settings_open {
        return;
    }

    egui::Area::new(egui::Id::new("main_menu_settings_button"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_TOP,
            egui::vec2(-HUD_MARGIN, HUD_TOP_OFFSET),
        )
        .show(ctx, |ui| {
            war_bar_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_sized([104.0, 32.0], egui::Button::new("显示设置"))
                        .clicked()
                    {
                        ui_state.settings_open = true;
                    }
                    if ui
                        .add_sized([104.0, 32.0], egui::Button::new("武将设置"))
                        .clicked()
                    {
                        open_officer_settings(ui_state);
                    }
                });
            });
        });
}

pub(super) fn open_officer_settings(ui_state: &mut GameUiState) {
    ui_state.officer_settings_editable = false;
    ui_state.officer_settings_selected_id = None;
    ui_state.officer_edit_open = false;
    ui_state.officer_edit_draft = None;
    ui_state.officer_edit_error = None;
    ui_state.officer_settings_game = load_officer_settings_game(ui_state);
    ui_state.officer_settings_open = true;
}

fn load_officer_settings_game(ui_state: &mut GameUiState) -> Option<GameState> {
    if let Ok(catalog) = SqliteHistoricalCatalog::open_default() {
        let scenario_id = if ui_state.selected_scenario_id.is_empty() {
            catalog
                .scenarios()
                .ok()
                .and_then(|scenarios| scenarios.first().map(|scenario| scenario.id.clone()))
        } else {
            Some(ui_state.selected_scenario_id.clone())
        };
        if let Some(scenario_id) = scenario_id {
            let faction_id = catalog
                .selectable_factions(&scenario_id)
                .ok()
                .and_then(|factions| {
                    factions
                        .iter()
                        .find(|faction| faction.selectable)
                        .or_else(|| factions.first())
                        .map(|faction| faction.id.clone())
                })
                .unwrap_or_else(|| ui_state.selected_faction_id.clone());
            match catalog.build_game(&scenario_id, &faction_id) {
                Ok(mut game) => {
                    match catalog.officer_profiles() {
                        Ok(profiles) => {
                            extend_game_with_catalog_officers(&mut game, profiles);
                            ui_state.officer_settings_editable = true;
                        }
                        Err(error) => ui_state.message = format!("读取全量武将资料失败: {error}"),
                    }
                    return Some(game);
                }
                Err(error) => ui_state.message = format!("读取武将资料失败: {error}"),
            }
        }
    }

    match ui_state
        .json_scenario
        .build_game(&ui_state.selected_faction_id)
    {
        Ok(game) => Some(game),
        Err(error) => {
            ui_state.message = format!("读取兼容武将资料失败: {error}");
            None
        }
    }
}

fn extend_game_with_catalog_officers(game: &mut GameState, profiles: Vec<OfficerProfile>) {
    let catalog_faction_id = "catalog".to_string();
    game.factions
        .entry(catalog_faction_id.clone())
        .or_insert_with(|| Faction {
            id: catalog_faction_id.clone(),
            name: "资料库".to_string(),
            ruler_id: String::new(),
            color: [0.52, 0.46, 0.34],
            selectable: false,
            controlled_by: Controller::RuleAi,
        });

    for profile in profiles {
        game.officers
            .entry(profile.id.clone())
            .or_insert_with(|| officer_from_profile(profile, &catalog_faction_id));
    }
}

fn officer_from_profile(profile: OfficerProfile, faction_id: &str) -> Officer {
    Officer {
        id: profile.id.clone(),
        name: profile.name.clone(),
        faction_id: faction_id.to_string(),
        city_id: None,
        office_id: None,
        stats: profile.stats,
        loyalty: 80,
        gender: profile.gender.clone(),
        status: OfficerStatus::Unavailable,
        profile: Some(profile),
    }
}

pub(super) fn officer_settings_modal(ctx: &egui::Context, ui_state: &mut GameUiState) {
    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new("officer_settings_modal_scrim"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120),
            );
            if response.clicked() && !ui_state.officer_edit_open {
                ui_state.officer_settings_open = false;
            }
        });

    let width = (screen.width() * 0.86).clamp(760.0, 1120.0);
    let height = (screen.height() * 0.78).clamp(460.0, 720.0);
    egui::Area::new(egui::Id::new("officer_settings_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, "武将设置") {
                    ui_state.officer_settings_open = false;
                }
                ui.separator();
                if let Some(game) = &ui_state.officer_settings_game {
                    officer_browser_filters(
                        ui,
                        game,
                        &mut ui_state.officer_settings_filters,
                        "main_menu_officer_settings_filters",
                    );
                    if !ui_state.officer_settings_editable {
                        ui.colored_label(war_text_muted(), "当前资料来源只读，无法编辑武将资料");
                    }
                    ui.separator();
                    let response = officer_browser_table(
                        ui,
                        game,
                        &ui_state.officer_settings_filters,
                        height - 118.0,
                        "main_menu_officer_settings_table",
                        ui_state.officer_settings_selected_id.as_deref(),
                        ui_state.officer_settings_editable,
                        None,
                    );
                    if let Some(officer_id) = response.selected_officer_id {
                        ui_state.officer_settings_selected_id = Some(officer_id);
                    }
                    if let Some(officer_id) = response.edit_officer_id {
                        open_officer_profile_editor(ui_state, &officer_id);
                    }
                } else {
                    ui.label("暂无武将资料");
                }
            });
        });
}

fn open_officer_profile_editor(ui_state: &mut GameUiState, officer_id: &str) {
    if !ui_state.officer_settings_editable {
        ui_state.message = "当前资料来源只读，无法编辑武将资料".to_string();
        return;
    }
    let profile = ui_state
        .officer_settings_game
        .as_ref()
        .and_then(|game| game.officers.get(officer_id))
        .and_then(|officer| officer.profile.as_ref());
    let Some(profile) = profile else {
        ui_state.officer_edit_error = Some(format!("武将 {officer_id} 缺少资料档案"));
        return;
    };
    ui_state.officer_settings_selected_id = Some(officer_id.to_string());
    ui_state.officer_edit_draft = Some(OfficerEditDraft::from_profile(profile));
    ui_state.officer_edit_error = None;
    ui_state.officer_edit_open = true;
}

fn officer_profile_edit_modal(ctx: &egui::Context, ui_state: &mut GameUiState) {
    let screen = ctx.content_rect();
    let width = (screen.width() * 0.72).clamp(620.0, 900.0);
    let height = (screen.height() * 0.78).clamp(460.0, 720.0);
    egui::Area::new(egui::Id::new("officer_profile_edit_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, "编辑武将") {
                    close_officer_profile_editor(ui_state);
                }
                ui.separator();
                if ui_state.officer_edit_draft.is_some() {
                    egui::ScrollArea::vertical()
                        .id_salt("officer_profile_edit_scroll")
                        .max_height(height - 116.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            officer_profile_edit_form(ui, ui_state);
                        });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([108.0, 34.0], egui::Button::new("保存"))
                            .clicked()
                        {
                            save_officer_profile_edit(ui_state);
                        }
                        if ui.button("取消").clicked() {
                            close_officer_profile_editor(ui_state);
                        }
                    });
                } else {
                    ui.label("未选择武将");
                }
            });
        });
}

fn officer_profile_edit_form(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let Some(draft) = ui_state.officer_edit_draft.as_mut() else {
        return;
    };
    ui.label(egui::RichText::new(format!("ID: {}", draft.id)).color(war_text_muted()));
    ui.add_space(6.0);
    egui::Grid::new("officer_profile_edit_grid")
        .num_columns(2)
        .spacing(egui::vec2(18.0, 8.0))
        .show(ui, |ui| {
            ui.label("姓名");
            ui.text_edit_singleline(&mut draft.name);
            ui.end_row();

            ui.label("字");
            ui.text_edit_singleline(&mut draft.courtesy_name);
            ui.end_row();

            ui.label("籍贯");
            ui.text_edit_singleline(&mut draft.native_place);
            ui.end_row();

            ui.label("生年");
            ui.text_edit_singleline(&mut draft.birth_year);
            ui.end_row();

            ui.label("卒年");
            ui.text_edit_singleline(&mut draft.death_year);
            ui.end_row();

            ui.label("性别");
            egui::ComboBox::from_id_salt("officer_edit_gender")
                .selected_text(officer_gender_label(&draft.gender))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut draft.gender, OfficerGender::Male, "男");
                    ui.selectable_value(&mut draft.gender, OfficerGender::Female, "女");
                });
            ui.end_row();

            ui.label("可信度");
            egui::ComboBox::from_id_salt("officer_edit_confidence")
                .selected_text(confidence_label(&draft.confidence))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut draft.confidence, SourceConfidence::High, "高");
                    ui.selectable_value(&mut draft.confidence, SourceConfidence::Medium, "中");
                    ui.selectable_value(&mut draft.confidence, SourceConfidence::Low, "低");
                });
            ui.end_row();

            ui.label("标签");
            ui.text_edit_singleline(&mut draft.tags);
            ui.end_row();
        });

    ui.add_space(10.0);
    ui.horizontal_wrapped(|ui| {
        ability_drag(ui, "统率", &mut draft.leadership);
        ability_drag(ui, "武力", &mut draft.strength);
        ability_drag(ui, "智力", &mut draft.intelligence);
        ability_drag(ui, "政治", &mut draft.politics);
        ability_drag(ui, "魅力", &mut draft.charm);
    });

    ui.add_space(10.0);
    ui.label("详细生平");
    ui.add_sized(
        [ui.available_width(), 150.0],
        egui::TextEdit::multiline(&mut draft.biography),
    );
    ui.add_space(8.0);
    ui.label("备注");
    ui.add_sized(
        [ui.available_width(), 72.0],
        egui::TextEdit::multiline(&mut draft.notes),
    );

    if let Some(error) = &ui_state.officer_edit_error {
        ui.add_space(8.0);
        ui.colored_label(egui::Color32::from_rgb(220, 92, 72), error);
    }
}

fn ability_drag(ui: &mut egui::Ui, label: &str, value: &mut u8) {
    ui.label(label);
    ui.add(
        egui::DragValue::new(value)
            .range(1..=100)
            .speed(1.0)
            .fixed_decimals(0),
    );
}

fn save_officer_profile_edit(ui_state: &mut GameUiState) {
    let Some(draft) = ui_state.officer_edit_draft.clone() else {
        return;
    };
    let update = match draft_to_update(&draft) {
        Ok(update) => update,
        Err(error) => {
            ui_state.officer_edit_error = Some(error);
            return;
        }
    };
    let result = SqliteHistoricalCatalog::open_default()
        .and_then(|catalog| catalog.update_officer_profile(&draft.id, &update));
    match result {
        Ok(profile) => {
            sync_updated_officer_profile(ui_state, profile);
            ui_state.message = format!("已保存武将 {}", draft.name.trim());
            close_officer_profile_editor(ui_state);
        }
        Err(error) => {
            ui_state.officer_edit_error = Some(error.to_string());
        }
    }
}

fn draft_to_update(draft: &OfficerEditDraft) -> Result<OfficerProfileUpdate, String> {
    if draft.name.trim().is_empty() {
        return Err("武将姓名不能为空".to_string());
    }
    Ok(OfficerProfileUpdate {
        name: draft.name.trim().to_string(),
        courtesy_name: optional_text(&draft.courtesy_name),
        native_place: optional_text(&draft.native_place),
        birth_year: optional_year(&draft.birth_year, "生年")?,
        death_year: optional_year(&draft.death_year, "卒年")?,
        gender: draft.gender.clone(),
        stats: OfficerStats {
            leadership: draft.leadership,
            strength: draft.strength,
            intelligence: draft.intelligence,
            politics: draft.politics,
            charm: draft.charm,
        },
        tags: draft
            .tags
            .split(',')
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect(),
        confidence: draft.confidence.clone(),
        biography: draft.biography.trim().to_string(),
        notes: draft.notes.trim().to_string(),
    })
}

fn optional_text(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn optional_year(value: &str, label: &str) -> Result<Option<i32>, String> {
    let value = value.trim();
    if value.is_empty() {
        Ok(None)
    } else {
        value
            .parse::<i32>()
            .map(Some)
            .map_err(|_| format!("{label} 必须是整数，留空表示未知"))
    }
}

fn sync_updated_officer_profile(ui_state: &mut GameUiState, profile: OfficerProfile) {
    if let Some(game) = &mut ui_state.officer_settings_game
        && let Some(officer) = game.officers.get_mut(&profile.id)
    {
        officer.name = profile.name.clone();
        officer.gender = profile.gender.clone();
        officer.stats = profile.stats;
        officer.profile = Some(profile);
    }
}

fn close_officer_profile_editor(ui_state: &mut GameUiState) {
    ui_state.officer_edit_open = false;
    ui_state.officer_edit_draft = None;
    ui_state.officer_edit_error = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::display_settings::{
        DisplaySettings, DisplaySettingsStore, LoadedDisplaySettings,
    };
    use crate::game::ScenarioData;

    fn test_profile(name: &str) -> OfficerProfile {
        OfficerProfile {
            id: "liu_bei".to_string(),
            name: name.to_string(),
            courtesy_name: Some("玄德".to_string()),
            native_place: Some("涿郡涿县".to_string()),
            birth_year: Some(161),
            death_year: Some(223),
            gender: OfficerGender::Male,
            stats: OfficerStats {
                leadership: 76,
                strength: 72,
                intelligence: 78,
                politics: 80,
                charm: 99,
            },
            tags: vec!["ruler".to_string()],
            confidence: SourceConfidence::High,
            biography: "刘备生平".to_string(),
            relationships: Vec::new(),
            notes: "测试".to_string(),
        }
    }

    fn ui_state_with_officer_settings_profile() -> GameUiState {
        let mut state = GameUiState::new(
            DisplaySettingsStore::with_default_path(),
            LoadedDisplaySettings {
                settings: DisplaySettings::default(),
                message: None,
            },
        );
        let mut game = ScenarioData::default_scenario()
            .unwrap()
            .build_game("liu_bei")
            .unwrap();
        let profile = test_profile("刘备");
        let officer = game.officers.get_mut("liu_bei").unwrap();
        officer.profile = Some(profile.clone());
        officer.name = profile.name.clone();
        officer.gender = profile.gender.clone();
        officer.stats = profile.stats;
        state.officer_settings_game = Some(game);
        state
    }

    #[test]
    fn draft_to_update_normalizes_blank_optional_fields_and_tags() {
        let mut draft = OfficerEditDraft::from_profile(&test_profile("刘备"));
        draft.courtesy_name = " ".to_string();
        draft.native_place = " 涿郡 ".to_string();
        draft.birth_year.clear();
        draft.death_year = "223".to_string();
        draft.tags = " ruler, edited ,, ".to_string();

        let update = draft_to_update(&draft).unwrap();

        assert_eq!(update.courtesy_name, None);
        assert_eq!(update.native_place.as_deref(), Some("涿郡"));
        assert_eq!(update.birth_year, None);
        assert_eq!(update.death_year, Some(223));
        assert_eq!(update.tags, ["ruler", "edited"]);
    }

    #[test]
    fn draft_to_update_rejects_invalid_year_without_mutating_state() {
        let mut state = ui_state_with_officer_settings_profile();
        state.officer_edit_draft = Some(OfficerEditDraft::from_profile(&test_profile("刘备")));
        state.officer_edit_draft.as_mut().unwrap().birth_year = "abc".to_string();

        let result = draft_to_update(state.officer_edit_draft.as_ref().unwrap());

        assert!(result.is_err());
        let officer = &state.officer_settings_game.as_ref().unwrap().officers["liu_bei"];
        assert_eq!(officer.name, "刘备");
    }

    #[test]
    fn sync_updated_officer_profile_refreshes_visible_officer_fields() {
        let mut state = ui_state_with_officer_settings_profile();
        let mut profile = test_profile("刘备改");
        profile.gender = OfficerGender::Female;
        profile.stats.leadership = 90;

        sync_updated_officer_profile(&mut state, profile);

        let officer = &state.officer_settings_game.as_ref().unwrap().officers["liu_bei"];
        assert_eq!(officer.name, "刘备改");
        assert_eq!(officer.gender, OfficerGender::Female);
        assert_eq!(officer.stats.leadership, 90);
        assert_eq!(officer.profile.as_ref().unwrap().name, "刘备改");
    }

    #[test]
    fn closing_editor_discards_draft_without_changing_officer() {
        let mut state = ui_state_with_officer_settings_profile();
        let mut draft = OfficerEditDraft::from_profile(&test_profile("刘备"));
        draft.name = "未保存".to_string();
        state.officer_edit_open = true;
        state.officer_edit_draft = Some(draft);

        close_officer_profile_editor(&mut state);

        let officer = &state.officer_settings_game.as_ref().unwrap().officers["liu_bei"];
        assert!(!state.officer_edit_open);
        assert_eq!(officer.name, "刘备");
        assert!(state.officer_edit_draft.is_none());
    }
}
