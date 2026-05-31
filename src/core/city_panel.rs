use crate::game::*;
use bevy_egui::egui;

use super::actions::queue_selected_city_command;
use super::labels::{
    city_scale_label, confidence_label, development_focus_label, diplomacy_label,
    facility_kind_label, officer_gender_label, officer_relationship_label,
};
use super::state::{CityTab, GameUiState};

pub(super) fn selected_city_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let Some(game) = &mut ui_state.game else {
        return;
    };
    let Some(city_id) = ui_state.selected_city_id.clone() else {
        ui.label("请选择城池");
        return;
    };
    let Some(city) = game.cities.get(&city_id).cloned() else {
        ui.label("城池不存在");
        return;
    };

    let faction_name = game
        .factions
        .get(&city.faction_id)
        .map(|faction| faction.name.as_str())
        .unwrap_or("未知");
    ui.heading(&city.name);
    ui.label(format!("归属: {faction_name}"));
    ui.label(format!(
        "{}级城镇 | 槽位 {}/{}",
        city.level,
        city.facilities.len(),
        city.facility_slots()
    ));
    ui.label(format!(
        "人口 {} | 金 {} | 粮 {} | 建材 {} | 兵 {}",
        city.population, city.gold, city.food, city.materials, city.troops
    ));
    ui.label(format!(
        "农业 {} | 商业 {} | 城防 {} | 训练 {} | 治安 {}",
        city.agriculture, city.commerce, city.defense, city.training, city.order
    ));
    let officer_salary = game
        .officers_in_city(&city.id)
        .into_iter()
        .filter(|officer| officer.faction_id == city.faction_id)
        .map(officer_monthly_salary)
        .sum();
    let projection = project_city_monthly_change(&city, officer_salary);
    ui.label(format!(
        "预计月净 金 {:+} | 粮 {:+} | 建材 {:+} | 人口 {:+} | 兵 {:+}",
        projection.net_gold,
        projection.net_food,
        projection.net_materials,
        projection.population_delta,
        projection.troop_delta
    ));
    ui.label(format!(
        "维护 金 -{} | 粮 -{} | 建材 -{} | 驻军粮 -{} | 俸禄 -{}",
        projection.facility_gold_maintenance,
        projection.facility_food_maintenance,
        projection.facility_materials_maintenance,
        projection.troop_food_upkeep,
        projection.officer_salary
    ));
    if city.facilities.is_empty() {
        ui.label("设施: 无");
    } else {
        let facilities = city
            .facilities
            .iter()
            .map(|facility| format!("{}{}级", facility_kind_label(facility.kind), facility.level))
            .collect::<Vec<_>>()
            .join(" / ");
        ui.label(format!("设施: {facilities}"));
    }
    if let Some(profile) = &city.profile {
        ui.label(format!(
            "{}{} | 规模 {} | 战略 {} | 可信度 {}",
            profile.province,
            profile.commandery,
            city_scale_label(&profile.scale),
            profile.strategic_rank,
            confidence_label(&profile.confidence)
        ));
        ui.label(format!(
            "人口区间 {}-{} | 农 {} 商 {} 防 {}",
            profile.population_min,
            profile.population_max,
            profile.agriculture_base,
            profile.commerce_base,
            profile.defense_base
        ));
        if !profile.notes.is_empty() {
            ui.label(&profile.notes);
        }
    }

    if let Some(governor_id) = &city.governor_id
        && let Some(governor) = game.officers.get(governor_id)
    {
        ui.label(format!("太守: {}", governor.name));
    }

    ui.separator();
    ui.horizontal(|ui| {
        ui.selectable_value(&mut ui_state.city_tab, CityTab::Construction, "建设");
        ui.selectable_value(&mut ui_state.city_tab, CityTab::Governance, "政务");
    });

    if city.faction_id != game.player_faction_id {
        ui.separator();
        officer_roster(ui, game, &city);
        ui.label("非己方城池，只能查看。");
        return;
    }

    let pending_city_ids = game.pending_city_ids();
    if pending_city_ids.contains(city.id.as_str()) {
        ui.separator();
        officer_roster(ui, game, &city);
        ui.label("本城本月已有待执行命令。");
        return;
    }

    let pending_officers = game.pending_officer_ids();
    let available_officers: Vec<_> = game
        .officers_in_city(&city.id)
        .into_iter()
        .filter(|officer| !pending_officers.contains(officer.id.as_str()))
        .cloned()
        .collect();
    if available_officers.is_empty() {
        ui.separator();
        ui.label("本城没有可行动武将。");
        return;
    }

    let selected_officer = ui_state
        .selected_officers
        .entry(city.id.clone())
        .or_insert_with(|| available_officers[0].id.clone());
    if !available_officers
        .iter()
        .any(|officer| officer.id == *selected_officer)
    {
        *selected_officer = available_officers[0].id.clone();
    }

    ui.separator();
    egui::ComboBox::from_id_salt(format!("officer_{}", city.id))
        .selected_text(
            game.officers
                .get(selected_officer)
                .map(|officer| officer.name.as_str())
                .unwrap_or("选择武将"),
        )
        .show_ui(ui, |ui| {
            for officer in &available_officers {
                ui.selectable_value(selected_officer, officer.id.clone(), &officer.name);
            }
        });

    match ui_state.city_tab {
        CityTab::Construction => {
            ui.heading("建设");
            city_core_controls(ui, ui_state, &city);
            facility_controls(ui, ui_state, &city);
            develop_controls(ui, ui_state, &city);
            recruit_controls(ui, ui_state, &city);
            train_controls(ui, ui_state, &city);
        }
        CityTab::Governance => {
            officer_roster(ui, game, &city);
            ui.heading("政务");
            appoint_controls(ui, ui_state, &city, &available_officers);
            transfer_controls(ui, ui_state, &city);
            expedition_controls(ui, ui_state, &city);
            diplomacy_controls(ui, ui_state, &city);
        }
    }
}

pub(super) fn officer_roster(ui: &mut egui::Ui, game: &GameState, city: &City) {
    ui.heading("武将");
    let officers = game.officers_in_city(&city.id);
    if officers.is_empty() {
        ui.label("无武将");
        return;
    }
    for officer in officers {
        officer_row(ui, officer);
    }
}

pub(super) fn city_core_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("城镇核心", |ui| {
        ui.label(format!(
            "当前 {} 级，设施槽位 {}",
            city.level,
            city.facility_slots()
        ));
        if city.level >= CITY_MAX_LEVEL {
            ui.label("已达最高等级");
            return;
        }
        let next_level = city.level + 1;
        let cost = city_core_upgrade_cost(next_level);
        ui.label(format!(
            "升至 {} 级需要 金 {} / 粮 {} / 建材 {}，治安至少 45",
            next_level, cost.gold, cost.food, cost.materials
        ));
        if ui.button("升级城镇核心").clicked() {
            queue_selected_city_command(ui_state, city, CommandKind::UpgradeCityCore);
        }
    });
}

pub(super) fn facility_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("设施建设", |ui| {
        egui::ComboBox::from_id_salt("facility_kind")
            .selected_text(facility_kind_label(ui_state.selected_facility_kind))
            .show_ui(ui, |ui| {
                for kind in ALL_FACILITY_KINDS {
                    ui.selectable_value(
                        &mut ui_state.selected_facility_kind,
                        kind,
                        facility_kind_label(kind),
                    );
                }
            });

        let kind = ui_state.selected_facility_kind;
        match facility_build_preview(city, kind) {
            Ok((target_level, cost, action)) => {
                ui.label(format!(
                    "{action}至 {} 级需要 金 {} / 粮 {} / 建材 {}",
                    target_level, cost.gold, cost.food, cost.materials
                ));
                if ui.button(action).clicked() {
                    queue_selected_city_command(
                        ui_state,
                        city,
                        CommandKind::BuildFacility { kind },
                    );
                }
            }
            Err(message) => {
                ui.label(message);
            }
        }
    });
}

fn facility_build_preview(
    city: &City,
    kind: FacilityKind,
) -> Result<(u8, ResourceCost, &'static str), &'static str> {
    if let Some(facility) = city.facility(kind) {
        if facility.level >= FACILITY_MAX_LEVEL {
            return Err("该设施已达最高等级");
        }
        let target_level = facility.level + 1;
        if target_level > city.level {
            return Err("设施等级不能超过城镇核心等级");
        }
        return Ok((
            target_level,
            facility_upgrade_cost(kind, target_level),
            "升级设施",
        ));
    }
    if city.facilities.len() >= city.facility_slots() {
        return Err("设施槽位已满，请先升级城镇核心");
    }
    Ok((1, facility_upgrade_cost(kind, 1), "建设设施"))
}

pub(super) fn develop_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("开发", |ui| {
        egui::ComboBox::from_id_salt("develop_focus")
            .selected_text(development_focus_label(&ui_state.selected_focus))
            .show_ui(ui, |ui| {
                for focus in [
                    DevelopmentFocus::Agriculture,
                    DevelopmentFocus::Commerce,
                    DevelopmentFocus::Defense,
                    DevelopmentFocus::Order,
                ] {
                    ui.selectable_value(
                        &mut ui_state.selected_focus,
                        focus.clone(),
                        development_focus_label(&focus),
                    );
                }
            });
        if ui.button("提交开发").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::Develop {
                    focus: ui_state.selected_focus.clone(),
                },
            );
        }
    });
}

pub(super) fn recruit_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("征兵", |ui| {
        ui.add(egui::Slider::new(&mut ui_state.recruit_amount, 100..=5000).text("兵力"));
        if ui.button("提交征兵").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::Recruit {
                    amount: ui_state.recruit_amount,
                },
            );
        }
    });
}

pub(super) fn train_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("训练", |ui| {
        if ui.button("提交训练").clicked() {
            queue_selected_city_command(ui_state, city, CommandKind::Train);
        }
    });
}

pub(super) fn appoint_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    city: &City,
    available_officers: &[Officer],
) {
    ui.collapsing("任命太守", |ui| {
        let target = ui_state
            .selected_officers
            .get(&city.id)
            .cloned()
            .unwrap_or_else(|| available_officers[0].id.clone());
        if ui.button("任命当前武将为太守").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::AppointGovernor {
                    target_officer_id: target,
                },
            );
        }
    });
}

pub(super) fn transfer_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("调动", |ui| {
        let Some(game) = &ui_state.game else {
            return;
        };
        let targets: Vec<_> = game
            .cities
            .values()
            .filter(|target| {
                target.faction_id == game.player_faction_id
                    && game.are_adjacent(&city.id, &target.id)
            })
            .cloned()
            .collect();
        if targets.is_empty() {
            ui.label("无邻接己方城池");
            return;
        }
        let selected = ui_state
            .selected_transfer_target
            .get_or_insert_with(|| targets[0].id.clone());
        if !targets.iter().any(|target| target.id == *selected) {
            *selected = targets[0].id.clone();
        }
        egui::ComboBox::from_id_salt("transfer_target")
            .selected_text(
                game.cities
                    .get(selected)
                    .map(|city| city.name.as_str())
                    .unwrap_or("目标"),
            )
            .show_ui(ui, |ui| {
                for target in &targets {
                    ui.selectable_value(selected, target.id.clone(), &target.name);
                }
            });
        ui.add(egui::Slider::new(&mut ui_state.transfer_troops, 0..=city.troops).text("兵力"));
        let selected_target_id = selected.clone();
        if ui.button("提交调动").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::Transfer {
                    target_city_id: selected_target_id,
                    troops: ui_state.transfer_troops,
                    officer_ids: Vec::new(),
                },
            );
        }
    });
}

pub(super) fn expedition_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("出征", |ui| {
        let Some(game) = &ui_state.game else {
            return;
        };
        let targets: Vec<_> = game
            .cities
            .values()
            .filter(|target| {
                target.faction_id != game.player_faction_id
                    && game.are_adjacent(&city.id, &target.id)
            })
            .cloned()
            .collect();
        if targets.is_empty() {
            ui.label("无邻接敌方城池");
            return;
        }
        let selected = ui_state
            .selected_expedition_target
            .get_or_insert_with(|| targets[0].id.clone());
        if !targets.iter().any(|target| target.id == *selected) {
            *selected = targets[0].id.clone();
        }
        egui::ComboBox::from_id_salt("expedition_target")
            .selected_text(
                game.cities
                    .get(selected)
                    .map(|city| city.name.as_str())
                    .unwrap_or("目标"),
            )
            .show_ui(ui, |ui| {
                for target in &targets {
                    ui.selectable_value(selected, target.id.clone(), &target.name);
                }
            });
        ui.add(
            egui::Slider::new(&mut ui_state.expedition_troops, 100..=city.troops.max(100))
                .text("兵力"),
        );
        let selected_target_id = selected.clone();
        if ui.button("提交出征").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::Expedition {
                    target_city_id: selected_target_id,
                    troops: ui_state.expedition_troops.min(city.troops),
                },
            );
        }
    });
}

pub(super) fn diplomacy_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("外交", |ui| {
        let Some(game) = &ui_state.game else {
            return;
        };
        let targets: Vec<_> = game
            .factions
            .values()
            .filter(|faction| {
                faction.id != game.player_faction_id && game.faction_alive(&faction.id)
            })
            .cloned()
            .collect();
        if targets.is_empty() {
            ui.label("无外交目标");
            return;
        }
        let selected = ui_state
            .selected_diplomacy_target
            .get_or_insert_with(|| targets[0].id.clone());
        if !targets.iter().any(|target| target.id == *selected) {
            *selected = targets[0].id.clone();
        }
        egui::ComboBox::from_id_salt("diplomacy_target")
            .selected_text(
                game.factions
                    .get(selected)
                    .map(|faction| faction.name.as_str())
                    .unwrap_or("目标"),
            )
            .show_ui(ui, |ui| {
                for target in &targets {
                    ui.selectable_value(selected, target.id.clone(), &target.name);
                }
            });
        egui::ComboBox::from_id_salt("diplomacy_proposal")
            .selected_text(diplomacy_label(&ui_state.selected_diplomacy_proposal))
            .show_ui(ui, |ui| {
                for proposal in [
                    DiplomacyProposal::ImproveRelations,
                    DiplomacyProposal::Truce,
                    DiplomacyProposal::DeclareWar,
                ] {
                    ui.selectable_value(
                        &mut ui_state.selected_diplomacy_proposal,
                        proposal.clone(),
                        diplomacy_label(&proposal),
                    );
                }
            });
        let selected_target_id = selected.clone();
        if ui.button("提交外交").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::Diplomacy {
                    target_faction_id: selected_target_id,
                    proposal: ui_state.selected_diplomacy_proposal.clone(),
                },
            );
        }
    });
}

pub(super) fn officer_row(ui: &mut egui::Ui, officer: &Officer) {
    let title = format!(
        "{} 统{} 武{} 智{} 政{} 魅{}",
        officer.name,
        officer.stats.leadership,
        officer.stats.strength,
        officer.stats.intelligence,
        officer.stats.politics,
        officer.stats.charm
    );
    ui.collapsing(title, |ui| {
        ui.label(format!("忠诚 {}", officer.loyalty));
        if let Some(profile) = &officer.profile {
            let courtesy = profile.courtesy_name.as_deref().unwrap_or("无");
            let native_place = profile.native_place.as_deref().unwrap_or("未详");
            let birth = profile
                .birth_year
                .map(|year| year.to_string())
                .unwrap_or_else(|| "未详".to_string());
            let death = profile
                .death_year
                .map(|year| year.to_string())
                .unwrap_or_else(|| "未详".to_string());
            ui.label(format!(
                "性别 {} | 字 {courtesy} | 籍贯 {native_place}",
                officer_gender_label(&profile.gender)
            ));
            ui.label(format!(
                "生卒 {birth}-{death} | 可信度 {}",
                confidence_label(&profile.confidence)
            ));
            if !profile.tags.is_empty() {
                ui.label(format!("标签 {}", profile.tags.join(", ")));
            }
            if !profile.biography.is_empty() {
                ui.separator();
                ui.label("生平");
                egui::ScrollArea::vertical()
                    .id_salt(format!("officer_bio_{}", profile.id))
                    .max_height(96.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.label(&profile.biography);
                    });
            }
            if !profile.relationships.is_empty() {
                ui.separator();
                ui.label("关系");
                for relationship in &profile.relationships {
                    let notes = if relationship.notes.is_empty() {
                        String::new()
                    } else {
                        format!(" - {}", relationship.notes)
                    };
                    ui.label(format!(
                        "{}: {}{}",
                        officer_relationship_label(&relationship.kind),
                        relationship.target_name,
                        notes
                    ));
                }
            }
            if !profile.notes.is_empty() {
                ui.label(&profile.notes);
            }
        }
    });
}
