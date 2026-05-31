use crate::game::*;
use bevy_egui::egui;

use super::city_intel::{
    city_development_intel, city_facility_intel, city_monthly_trend_intel, city_overview_intel,
    city_resource_intel, city_stability_intel,
};
use super::labels::{
    confidence_label, development_focus_label, diplomacy_label, facility_kind_label,
    officer_gender_label, officer_relationship_label,
};
use super::state::{CityPanelTab, CommandAction, CommandCategory, GameUiState};
use super::style::{
    war_danger, war_gold, war_sub_panel_frame, war_success, war_text_muted, war_warning,
};

pub(super) fn selected_city_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let Some(game) = ui_state.game.as_ref().cloned() else {
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
    let pending_command = pending_command_for_city(&game, &city.id).cloned();
    let available_officers = available_officers_for_city(&game, &city);
    let selected_officer_id = ensure_selected_officer(ui_state, &city, &available_officers);
    let selected_officer = selected_officer_id
        .as_ref()
        .and_then(|officer_id| game.officers.get(officer_id));
    sync_command_selection_to_city_tab(ui_state);

    command_tent_header(ui, &game, &city, faction_name, pending_command.as_ref());
    ui.add_space(8.0);
    city_tab_bar(ui, ui_state);
    ui.add_space(8.0);

    match ui_state.selected_city_tab {
        CityPanelTab::Overview => overview_tab(ui, &game, &city, faction_name),
        CityPanelTab::Domestic | CityPanelTab::Military | CityPanelTab::Diplomacy => {
            command_tab(
                ui,
                ui_state,
                &game,
                &city,
                &available_officers,
                selected_officer,
                selected_officer_id.as_ref(),
                pending_command.as_ref(),
            );
        }
    }
}

fn command_tent_header(
    ui: &mut egui::Ui,
    game: &GameState,
    city: &City,
    faction_name: &str,
    pending_command: Option<&Command>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(&city.name)
                .size(24.0)
                .color(war_gold())
                .strong(),
        );
        ui.separator();
        ui.label(format!(
            "{faction_name}  {}年{}月  第{}回合",
            game.year, game.month, game.turn
        ));
        ui.separator();
        ui.label(format!(
            "金 {}  粮 {}  建材 {}  兵 {}",
            city.gold, city.food, city.materials, city.troops
        ));
        ui.separator();
        match pending_command {
            Some(command) => ui.colored_label(
                status_warning_color(),
                format!("待命令: {}", command_title(command)),
            ),
            None => ui.colored_label(status_ready_color(), "本城可下令"),
        };
    });
}

fn city_tab_bar(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.horizontal(|ui| {
        for tab in [
            CityPanelTab::Overview,
            CityPanelTab::Domestic,
            CityPanelTab::Military,
            CityPanelTab::Diplomacy,
        ] {
            if ui
                .selectable_label(ui_state.selected_city_tab == tab, city_tab_label(tab))
                .clicked()
            {
                ui_state.selected_city_tab = tab;
                sync_command_selection_to_city_tab(ui_state);
            }
        }
    });
}

fn overview_tab(ui: &mut egui::Ui, game: &GameState, city: &City, faction_name: &str) {
    let projection = city_projection(game, city);
    ui.columns(3, |columns| {
        columns[0].set_width(330.0);
        war_sub_panel_frame().show(&mut columns[0], |ui| {
            section_title(ui, "城池概览");
            city_overview_intel(ui, city, faction_name);
            ui.add_space(8.0);
            city_resource_intel(ui, city);
        });

        columns[1].set_width(330.0);
        war_sub_panel_frame().show(&mut columns[1], |ui| {
            section_title(ui, "发展与军备");
            city_development_intel(ui, city);
            ui.add_space(8.0);
            city_stability_intel(ui, city);
        });

        war_sub_panel_frame().show(&mut columns[2], |ui| {
            section_title(ui, "月报与设施");
            city_monthly_trend_intel(ui, &projection);
            ui.add_space(8.0);
            city_facility_intel(ui, city);
            if city.faction_id != game.player_faction_id {
                ui.add_space(8.0);
                ui.colored_label(war_text_muted(), "非己方城池，只能查看情报。");
            }
        });
    });
}

fn command_tab(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    available_officers: &[Officer],
    selected_officer: Option<&Officer>,
    selected_officer_id: Option<&OfficerId>,
    pending_command: Option<&Command>,
) {
    ui.columns(3, |columns| {
        columns[0].set_width(300.0);
        command_action_column(
            &mut columns[0],
            ui_state,
            game,
            city,
            available_officers,
            selected_officer_id,
        );

        columns[1].set_width(370.0);
        command_parameter_column(
            &mut columns[1],
            ui_state,
            game,
            city,
            selected_officer,
            pending_command,
        );

        command_preview_column(
            &mut columns[2],
            ui_state,
            game,
            city,
            selected_officer,
            selected_officer_id,
            pending_command,
        );
    });
}

fn command_action_column(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    available_officers: &[Officer],
    selected_officer_id: Option<&OfficerId>,
) {
    war_sub_panel_frame().show(ui, |ui| {
        section_title(ui, "执令武将");
        if city.faction_id != game.player_faction_id {
            ui.colored_label(war_text_muted(), "非己方城池，只能查看。");
            officer_roster_list(ui, game, city, 250.0);
        } else if available_officers.is_empty() {
            ui.colored_label(war_text_muted(), "本城没有可行动武将。");
        } else {
            egui::ScrollArea::vertical()
                .id_salt(("city_command_officers", &city.id))
                .max_height(196.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for officer in available_officers {
                        let selected =
                            selected_officer_id.is_some_and(|officer_id| officer_id == &officer.id);
                        if ui
                            .selectable_label(selected, officer_command_label(officer))
                            .clicked()
                        {
                            ui_state
                                .selected_officers
                                .insert(city.id.clone(), officer.id.clone());
                        }
                    }
                });
        }
    });

    ui.add_space(8.0);
    war_sub_panel_frame().show(ui, |ui| {
        section_title(ui, "军令");
        for action in command_actions(ui_state.selected_command_category) {
            let response = ui.selectable_label(
                ui_state.selected_command_action == *action,
                command_action_label(*action),
            );
            if response.clicked() {
                ui_state.selected_command_action = *action;
            }
            response.on_hover_text(command_action_hint(*action));
        }
    });
}

fn city_projection(game: &GameState, city: &City) -> CityMonthlyProjection {
    let officer_salary = game
        .officers_in_city(&city.id)
        .into_iter()
        .filter(|officer| officer.faction_id == city.faction_id)
        .map(officer_monthly_salary)
        .sum();
    project_city_monthly_change_with_effects(
        city,
        officer_salary,
        city_official_effects(game, &city.id),
    )
}

fn sync_command_selection_to_city_tab(ui_state: &mut GameUiState) {
    let Some(category) = city_tab_command_category(ui_state.selected_city_tab) else {
        return;
    };
    ui_state.selected_command_category = category;
    ensure_command_action(ui_state);
}

fn city_tab_command_category(tab: CityPanelTab) -> Option<CommandCategory> {
    match tab {
        CityPanelTab::Overview => None,
        CityPanelTab::Domestic => Some(CommandCategory::Domestic),
        CityPanelTab::Military => Some(CommandCategory::Military),
        CityPanelTab::Diplomacy => Some(CommandCategory::Diplomacy),
    }
}

fn city_tab_label(tab: CityPanelTab) -> &'static str {
    match tab {
        CityPanelTab::Overview => "概览",
        CityPanelTab::Domestic => "内政",
        CityPanelTab::Military => "军事",
        CityPanelTab::Diplomacy => "任命外交",
    }
}

fn command_parameter_column(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    selected_officer: Option<&Officer>,
    pending_command: Option<&Command>,
) {
    war_sub_panel_frame().show(ui, |ui| {
        section_title(ui, command_action_label(ui_state.selected_command_action));
        ui.colored_label(
            war_text_muted(),
            command_action_hint(ui_state.selected_command_action),
        );
        ui.separator();

        if city.faction_id != game.player_faction_id {
            ui.colored_label(war_text_muted(), "非己方城池，只能查看情报。");
            return;
        }

        if pending_command.is_some() {
            ui.colored_label(war_text_muted(), "撤销本城待命令后可重新配置军令。");
            return;
        }

        if selected_officer.is_none() {
            ui.colored_label(war_text_muted(), "请选择一名可行动武将。");
            return;
        }

        match ui_state.selected_command_action {
            CommandAction::Develop => develop_parameter_controls(ui, ui_state, selected_officer),
            CommandAction::UpgradeCityCore => upgrade_parameter_controls(ui, city),
            CommandAction::BuildFacility => facility_parameter_controls(ui, ui_state, city),
            CommandAction::Recruit => recruit_parameter_controls(ui, ui_state, city),
            CommandAction::Train => train_parameter_controls(ui, selected_officer),
            CommandAction::AppointGovernor => {
                appoint_parameter_controls(ui, city, selected_officer)
            }
            CommandAction::Transfer => transfer_parameter_controls(ui, ui_state, game, city),
            CommandAction::Expedition => expedition_parameter_controls(ui, ui_state, game, city),
            CommandAction::Diplomacy => diplomacy_parameter_controls(ui, ui_state, game),
        }
    });
}

fn command_preview_column(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    selected_officer: Option<&Officer>,
    selected_officer_id: Option<&OfficerId>,
    pending_command: Option<&Command>,
) {
    war_sub_panel_frame().show(ui, |ui| {
        section_title(ui, "军令预览");
        if let Some(command) = pending_command {
            ui.colored_label(status_warning_color(), "本城已有待执行军令");
            ui.label(command_summary(game, command));
            ui.separator();
            if ui.button("撤销本城军令").clicked() {
                withdraw_pending_command(ui_state, &city.id);
            }
            return;
        }

        if city.faction_id != game.player_faction_id {
            ui.colored_label(war_text_muted(), "非己方城池不可下令。");
            return;
        }

        let Some(command) = build_candidate_command(ui_state, game, city, selected_officer_id)
        else {
            ui.colored_label(war_text_muted(), "军令参数尚未完整。");
            return;
        };

        for line in command_preview_lines(game, city, selected_officer, &command) {
            ui.label(line);
        }
        ui.separator();

        match preview_command(game, &command) {
            Ok(()) => {
                ui.colored_label(status_ready_color(), "可以提交");
                if ui.button("提交军令").clicked() {
                    submit_candidate_command(ui_state, command);
                }
            }
            Err(message) => {
                ui.colored_label(status_error_color(), format!("不可提交: {message}"));
                ui.add_enabled(false, egui::Button::new("提交军令"));
            }
        }
    });
}

const DOMESTIC_ACTIONS: [CommandAction; 5] = [
    CommandAction::Develop,
    CommandAction::UpgradeCityCore,
    CommandAction::BuildFacility,
    CommandAction::Recruit,
    CommandAction::Train,
];
const MILITARY_ACTIONS: [CommandAction; 2] = [CommandAction::Transfer, CommandAction::Expedition];
const DIPLOMACY_ACTIONS: [CommandAction; 2] =
    [CommandAction::AppointGovernor, CommandAction::Diplomacy];

fn ensure_command_action(ui_state: &mut GameUiState) {
    if !command_actions(ui_state.selected_command_category)
        .contains(&ui_state.selected_command_action)
    {
        ui_state.selected_command_action =
            default_action_for_category(ui_state.selected_command_category);
    }
}

fn command_actions(category: CommandCategory) -> &'static [CommandAction] {
    match category {
        CommandCategory::Domestic => &DOMESTIC_ACTIONS,
        CommandCategory::Military => &MILITARY_ACTIONS,
        CommandCategory::Diplomacy => &DIPLOMACY_ACTIONS,
    }
}

fn default_action_for_category(category: CommandCategory) -> CommandAction {
    command_actions(category)[0]
}

fn command_action_label(action: CommandAction) -> &'static str {
    match action {
        CommandAction::Develop => "开发",
        CommandAction::UpgradeCityCore => "扩建核心",
        CommandAction::BuildFacility => "建设设施",
        CommandAction::Recruit => "征兵",
        CommandAction::Train => "训练",
        CommandAction::AppointGovernor => "任命太守",
        CommandAction::Transfer => "调动",
        CommandAction::Expedition => "出征",
        CommandAction::Diplomacy => "外交",
    }
}

fn command_action_hint(action: CommandAction) -> &'static str {
    match action {
        CommandAction::Develop => "投入金钱提升农业、商业、城防或治安。",
        CommandAction::UpgradeCityCore => "扩张城镇规模，增加设施上限。",
        CommandAction::BuildFacility => "建设或升级设施，改变长期收入与维护。",
        CommandAction::Recruit => "消耗人口、金钱和粮草补充兵力。",
        CommandAction::Train => "提高驻军训练度，增强战斗表现。",
        CommandAction::AppointGovernor => "让当前武将接任太守。",
        CommandAction::Transfer => "向邻接己方城池调动兵力。",
        CommandAction::Expedition => "攻击邻接敌方城池。",
        CommandAction::Diplomacy => "对存续势力提出外交行动。",
    }
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(egui::RichText::new(title).color(war_gold()).strong());
}

fn status_ready_color() -> egui::Color32 {
    war_success()
}

fn status_warning_color() -> egui::Color32 {
    war_warning()
}

fn status_error_color() -> egui::Color32 {
    war_danger()
}

fn pending_command_for_city<'a>(game: &'a GameState, city_id: &str) -> Option<&'a Command> {
    game.pending_commands
        .iter()
        .find(|command| command.city_id == city_id)
}

fn available_officers_for_city(game: &GameState, city: &City) -> Vec<Officer> {
    if city.faction_id != game.player_faction_id {
        return Vec::new();
    }
    let pending_officers = game.pending_officer_ids();
    game.officers_in_city(&city.id)
        .into_iter()
        .filter(|officer| {
            officer.faction_id == game.player_faction_id
                && !pending_officers.contains(officer.id.as_str())
        })
        .cloned()
        .collect()
}

fn ensure_selected_officer(
    ui_state: &mut GameUiState,
    city: &City,
    available_officers: &[Officer],
) -> Option<OfficerId> {
    if available_officers.is_empty() {
        ui_state.selected_officers.remove(&city.id);
        return None;
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
    Some(selected_officer.clone())
}

fn officer_command_label(officer: &Officer) -> String {
    format!(
        "{}  统{} 武{} 智{} 政{} 魅{}",
        officer.name,
        officer.stats.leadership,
        officer.stats.strength,
        officer.stats.intelligence,
        officer.stats.politics,
        officer.stats.charm
    )
}

fn develop_parameter_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    selected_officer: Option<&Officer>,
) {
    egui::ComboBox::from_id_salt("command_develop_focus")
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
    if let Some(officer) = selected_officer {
        ui.label(format!(
            "消耗 金 80；{} 的政治与魅力会影响提升幅度。",
            officer.name
        ));
    }
}

fn upgrade_parameter_controls(ui: &mut egui::Ui, city: &City) {
    ui.label(format!(
        "当前 {} 级，设施槽位 {}",
        city.level,
        city.facility_slots()
    ));
    if city.level >= CITY_MAX_LEVEL {
        ui.colored_label(war_text_muted(), "城镇核心已达最高等级。");
        return;
    }
    let next_level = city.level + 1;
    let cost = city_core_upgrade_cost(next_level);
    ui.label(format!(
        "升至 {next_level} 级需要 金 {} / 粮 {} / 建材 {}。",
        cost.gold, cost.food, cost.materials
    ));
    ui.label("治安至少 45；扩建后治安会下降 2。");
}

fn facility_parameter_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    egui::ComboBox::from_id_salt("command_facility_kind")
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

    match facility_build_preview(city, ui_state.selected_facility_kind) {
        Ok((target_level, cost, action)) => {
            ui.label(format!(
                "{action}至 {target_level} 级需要 金 {} / 粮 {} / 建材 {}。",
                cost.gold, cost.food, cost.materials
            ));
        }
        Err(message) => {
            ui.colored_label(war_text_muted(), message);
        }
    }
}

fn recruit_parameter_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui_state.recruit_amount = ui_state.recruit_amount.clamp(100, 5000);
    ui.add(egui::Slider::new(&mut ui_state.recruit_amount, 100..=5000).text("征兵数"));
    let cost = recruit_cost(ui_state.recruit_amount);
    ui.label(format!(
        "消耗 金 {} / 粮 {} / 人口 {}。",
        cost.gold,
        cost.food,
        ui_state.recruit_amount * 2
    ));
    ui.label(format!(
        "当前人口 {}，驻军 {}。",
        city.population, city.troops
    ));
}

fn train_parameter_controls(ui: &mut egui::Ui, selected_officer: Option<&Officer>) {
    ui.label("消耗 金 40，提高驻军训练度。");
    if let Some(officer) = selected_officer {
        let gain = (6 + officer.stats.leadership / 12).min(15);
        ui.label(format!(
            "预计训练提升约 {gain}，受 {} 的统率影响。",
            officer.name
        ));
    }
}

fn appoint_parameter_controls(ui: &mut egui::Ui, city: &City, selected_officer: Option<&Officer>) {
    if let Some(officer) = selected_officer {
        ui.label(format!("将 {} 任命为 {} 太守。", officer.name, city.name));
        ui.label("任命会占用本月城市军令和该武将行动。");
    }
}

fn transfer_parameter_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
) {
    let targets = adjacent_cities(game, city, true);
    if targets.is_empty() {
        ui.colored_label(war_text_muted(), "无邻接己方城池。");
        return;
    }
    let Some(selected_target_id) =
        ensure_selected_target(&mut ui_state.selected_transfer_target, &targets)
    else {
        return;
    };
    egui::ComboBox::from_id_salt("command_transfer_target")
        .selected_text(city_name(game, &selected_target_id))
        .show_ui(ui, |ui| {
            for target in &targets {
                ui.selectable_value(
                    &mut ui_state.selected_transfer_target,
                    Some(target.id.clone()),
                    &target.name,
                );
            }
        });
    if city.troops == 0 {
        ui_state.transfer_troops = 0;
        ui.colored_label(war_text_muted(), "本城无可调动兵力。");
    } else {
        ui_state.transfer_troops = ui_state.transfer_troops.clamp(1, city.troops);
        ui.add(egui::Slider::new(&mut ui_state.transfer_troops, 1..=city.troops).text("兵力"));
    }
}

fn expedition_parameter_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
) {
    let targets = adjacent_cities(game, city, false);
    if targets.is_empty() {
        ui.colored_label(war_text_muted(), "无邻接敌方城池。");
        return;
    }
    let Some(selected_target_id) =
        ensure_selected_target(&mut ui_state.selected_expedition_target, &targets)
    else {
        return;
    };
    egui::ComboBox::from_id_salt("command_expedition_target")
        .selected_text(city_name(game, &selected_target_id))
        .show_ui(ui, |ui| {
            for target in &targets {
                let faction_name = game
                    .factions
                    .get(&target.faction_id)
                    .map(|faction| faction.name.as_str())
                    .unwrap_or("未知");
                ui.selectable_value(
                    &mut ui_state.selected_expedition_target,
                    Some(target.id.clone()),
                    format!("{} ({faction_name})", target.name),
                );
            }
        });
    if city.troops == 0 {
        ui_state.expedition_troops = 0;
        ui.colored_label(war_text_muted(), "本城无可出征兵力。");
    } else {
        ui_state.expedition_troops = ui_state.expedition_troops.clamp(1, city.troops);
        ui.add(egui::Slider::new(&mut ui_state.expedition_troops, 1..=city.troops).text("兵力"));
    }
}

fn diplomacy_parameter_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, game: &GameState) {
    let targets: Vec<_> = game
        .factions
        .values()
        .filter(|faction| faction.id != game.player_faction_id && game.faction_alive(&faction.id))
        .cloned()
        .collect();
    if targets.is_empty() {
        ui.colored_label(war_text_muted(), "无外交目标。");
        return;
    }
    let selected = ui_state
        .selected_diplomacy_target
        .get_or_insert_with(|| targets[0].id.clone());
    if !targets.iter().any(|target| target.id == *selected) {
        *selected = targets[0].id.clone();
    }
    egui::ComboBox::from_id_salt("command_diplomacy_target")
        .selected_text(faction_name(game, selected))
        .show_ui(ui, |ui| {
            for target in &targets {
                ui.selectable_value(selected, target.id.clone(), &target.name);
            }
        });
    egui::ComboBox::from_id_salt("command_diplomacy_proposal")
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
}

fn adjacent_cities(game: &GameState, city: &City, own: bool) -> Vec<City> {
    let mut targets: Vec<_> = game
        .cities
        .values()
        .filter(|target| {
            let ownership_matches = if own {
                target.faction_id == game.player_faction_id
            } else {
                target.faction_id != game.player_faction_id
            };
            ownership_matches && game.are_adjacent(&city.id, &target.id)
        })
        .cloned()
        .collect();
    targets.sort_by(|a, b| a.name.cmp(&b.name));
    targets
}

fn ensure_selected_target(selected: &mut Option<CityId>, targets: &[City]) -> Option<CityId> {
    if targets.is_empty() {
        *selected = None;
        return None;
    }
    let needs_reset = selected
        .as_ref()
        .is_none_or(|city_id| !targets.iter().any(|target| target.id == *city_id));
    if needs_reset {
        *selected = Some(targets[0].id.clone());
    }
    selected.clone()
}

fn build_candidate_command(
    ui_state: &GameUiState,
    game: &GameState,
    city: &City,
    selected_officer_id: Option<&OfficerId>,
) -> Option<Command> {
    let officer_id = selected_officer_id?.clone();
    let kind = match ui_state.selected_command_action {
        CommandAction::Develop => CommandKind::Develop {
            focus: ui_state.selected_focus.clone(),
        },
        CommandAction::UpgradeCityCore => CommandKind::UpgradeCityCore,
        CommandAction::BuildFacility => CommandKind::BuildFacility {
            kind: ui_state.selected_facility_kind,
        },
        CommandAction::Recruit => CommandKind::Recruit {
            amount: ui_state.recruit_amount,
        },
        CommandAction::Train => CommandKind::Train,
        CommandAction::AppointGovernor => CommandKind::AppointGovernor {
            target_officer_id: officer_id.clone(),
        },
        CommandAction::Transfer => CommandKind::Transfer {
            target_city_id: ui_state.selected_transfer_target.clone()?,
            troops: ui_state.transfer_troops,
            officer_ids: Vec::new(),
        },
        CommandAction::Expedition => CommandKind::Expedition {
            target_city_id: ui_state.selected_expedition_target.clone()?,
            troops: ui_state.expedition_troops,
        },
        CommandAction::Diplomacy => CommandKind::Diplomacy {
            target_faction_id: ui_state.selected_diplomacy_target.clone()?,
            proposal: ui_state.selected_diplomacy_proposal.clone(),
        },
    };
    Some(Command {
        issuer_faction_id: game.player_faction_id.clone(),
        city_id: city.id.clone(),
        officer_id: Some(officer_id),
        kind,
    })
}

fn preview_command(game: &GameState, command: &Command) -> Result<(), String> {
    let mut preview = game.clone();
    queue_player_command(&mut preview, command.clone()).map_err(|error| error.to_string())
}

fn submit_candidate_command(ui_state: &mut GameUiState, command: Command) {
    let Some(game) = &mut ui_state.game else {
        ui_state.message = "尚未开始游戏".to_string();
        return;
    };
    let city_name = city_name(game, &command.city_id);
    match queue_player_command(game, command) {
        Ok(()) => ui_state.message = format!("已提交 {city_name} 的命令"),
        Err(error) => ui_state.message = error.to_string(),
    }
}

fn withdraw_pending_command(ui_state: &mut GameUiState, city_id: &str) {
    let Some(game) = &mut ui_state.game else {
        ui_state.message = "尚未开始游戏".to_string();
        return;
    };
    if remove_pending_command_for_city(game, city_id).is_some() {
        let city_name = city_name(game, city_id);
        ui_state.message = format!("已撤销 {city_name} 的待命令");
    } else {
        ui_state.message = "本城没有待命令".to_string();
    }
}

fn remove_pending_command_for_city(game: &mut GameState, city_id: &str) -> Option<Command> {
    let index = game
        .pending_commands
        .iter()
        .position(|command| command.city_id == city_id)?;
    Some(game.pending_commands.remove(index))
}

fn command_preview_lines(
    game: &GameState,
    city: &City,
    officer: Option<&Officer>,
    command: &Command,
) -> Vec<String> {
    let mut lines = vec![
        format!("城池: {}", city.name),
        format!(
            "执令: {}",
            officer
                .map(|officer| officer.name.as_str())
                .unwrap_or("未选择")
        ),
        format!("军令: {}", command_title(command)),
    ];
    match &command.kind {
        CommandKind::Develop { focus } => {
            lines.push(format!("目标: 提升{}", development_focus_label(focus)));
            lines.push("消耗: 金 80".to_string());
        }
        CommandKind::UpgradeCityCore => {
            if city.level < CITY_MAX_LEVEL {
                let cost = city_core_upgrade_cost(city.level + 1);
                lines.push(format!("目标: 城镇核心升至 {} 级", city.level + 1));
                lines.push(resource_cost_line("消耗", cost));
            }
        }
        CommandKind::BuildFacility { kind } => match facility_build_preview(city, *kind) {
            Ok((target_level, cost, action)) => {
                lines.push(format!(
                    "{action}: {} 至 {} 级",
                    facility_kind_label(*kind),
                    target_level
                ));
                lines.push(resource_cost_line("消耗", cost));
            }
            Err(message) => lines.push(message.to_string()),
        },
        CommandKind::Recruit { amount } => {
            lines.push(format!("兵力: +{amount}"));
            lines.push(resource_cost_line("消耗", recruit_cost(*amount)));
            lines.push(format!("人口: -{}", amount * 2));
        }
        CommandKind::Train => {
            lines.push("消耗: 金 40".to_string());
            lines.push("效果: 提高驻军训练度".to_string());
        }
        CommandKind::AppointGovernor { target_officer_id } => {
            lines.push(format!(
                "目标: {} 太守",
                officer_name(game, target_officer_id)
            ));
        }
        CommandKind::Transfer {
            target_city_id,
            troops,
            ..
        } => {
            lines.push(format!("目标: {}", city_name(game, target_city_id)));
            lines.push(format!("调动兵力: {troops}"));
        }
        CommandKind::Expedition {
            target_city_id,
            troops,
        } => {
            let target = game.cities.get(target_city_id);
            lines.push(format!("目标: {}", city_name(game, target_city_id)));
            if let Some(target) = target {
                lines.push(format!("敌方: {}", faction_name(game, &target.faction_id)));
            }
            lines.push(format!("出征兵力: {troops}"));
        }
        CommandKind::Diplomacy {
            target_faction_id,
            proposal,
        } => {
            lines.push(format!("目标: {}", faction_name(game, target_faction_id)));
            lines.push(format!("提案: {}", diplomacy_label(proposal)));
        }
    }
    lines
}

fn resource_cost_line(prefix: &str, cost: ResourceCost) -> String {
    format!(
        "{prefix}: 金 {} / 粮 {} / 建材 {}",
        cost.gold, cost.food, cost.materials
    )
}

fn command_summary(game: &GameState, command: &Command) -> String {
    format!(
        "{} | {} | {}",
        city_name(game, &command.city_id),
        command
            .officer_id
            .as_deref()
            .map(|officer_id| officer_name(game, officer_id))
            .unwrap_or_else(|| "未选武将".to_string()),
        command_title(command)
    )
}

fn command_title(command: &Command) -> String {
    match &command.kind {
        CommandKind::Develop { focus } => format!("开发{}", development_focus_label(focus)),
        CommandKind::UpgradeCityCore => "扩建核心".to_string(),
        CommandKind::BuildFacility { kind } => format!("建设{}", facility_kind_label(*kind)),
        CommandKind::Recruit { amount } => format!("征兵 {amount}"),
        CommandKind::Train => "训练".to_string(),
        CommandKind::AppointGovernor { .. } => "任命太守".to_string(),
        CommandKind::Transfer { troops, .. } => format!("调动 {troops}"),
        CommandKind::Expedition { troops, .. } => format!("出征 {troops}"),
        CommandKind::Diplomacy { proposal, .. } => format!("外交{}", diplomacy_label(proposal)),
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

fn officer_name(game: &GameState, officer_id: &str) -> String {
    game.officers
        .get(officer_id)
        .map(|officer| officer.name.clone())
        .unwrap_or_else(|| officer_id.to_string())
}

fn officer_roster_list(ui: &mut egui::Ui, game: &GameState, city: &City, max_height: f32) {
    let officers = game.officers_in_city(&city.id);
    if officers.is_empty() {
        ui.label("无武将");
        return;
    }
    egui::ScrollArea::vertical()
        .id_salt(("city_officer_roster", &city.id))
        .max_height(max_height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for officer in officers {
                officer_row(ui, officer);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_game() -> GameState {
        ScenarioData::default_scenario()
            .unwrap()
            .build_game("liu_bei")
            .unwrap()
    }

    fn test_command(city_id: &str, officer_id: &str, kind: CommandKind) -> Command {
        Command {
            issuer_faction_id: "liu_bei".to_string(),
            city_id: city_id.to_string(),
            officer_id: Some(officer_id.to_string()),
            kind,
        }
    }

    #[test]
    fn pending_command_for_city_finds_only_matching_city() {
        let mut game = test_game();
        game.pending_commands.push(test_command(
            "pingyuan",
            "liu_bei",
            CommandKind::Develop {
                focus: DevelopmentFocus::Agriculture,
            },
        ));

        let pending = pending_command_for_city(&game, "pingyuan");
        assert!(pending.is_some());
        assert!(pending_command_for_city(&game, "xiapi").is_none());
    }

    #[test]
    fn remove_pending_command_for_city_keeps_other_city_commands() {
        let mut game = test_game();
        game.pending_commands.push(test_command(
            "pingyuan",
            "liu_bei",
            CommandKind::Develop {
                focus: DevelopmentFocus::Agriculture,
            },
        ));
        game.pending_commands
            .push(test_command("xiapi", "zhang_fei", CommandKind::Train));

        let removed = remove_pending_command_for_city(&mut game, "pingyuan");

        assert!(removed.is_some());
        assert_eq!(game.pending_commands.len(), 1);
        assert_eq!(game.pending_commands[0].city_id, "xiapi");
    }

    #[test]
    fn preview_command_uses_pending_command_reservations() {
        let mut game = test_game();
        game.pending_commands.push(test_command(
            "pingyuan",
            "liu_bei",
            CommandKind::Develop {
                focus: DevelopmentFocus::Agriculture,
            },
        ));
        let command = test_command("pingyuan", "guan_yu", CommandKind::Train);

        let result = preview_command(&game, &command);

        assert!(result.unwrap_err().contains("平原 本月已经有命令"));
    }
}
