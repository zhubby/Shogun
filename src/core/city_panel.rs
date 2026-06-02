use crate::game::*;
use bevy_egui::egui;

use super::city_intel::{
    city_development_intel, city_facility_intel, city_monthly_trend_intel, city_overview_intel,
    city_resource_intel, city_stability_intel,
};
use super::i18n::{Translator, args};
use super::labels::{
    confidence_label, development_focus_label, diplomacy_label, facility_kind_label,
    officer_gender_label, officer_relationship_label, troop_kind_label,
};
use super::state::{CityPanelTab, CommandAction, CommandCategory, GameUiState};
use super::style::{
    war_danger, war_gold, war_sub_panel_frame, war_success, war_text_muted, war_warning,
};

pub(super) fn selected_city_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    let Some(game) = ui_state.game.as_ref().cloned() else {
        return;
    };
    let Some(city_id) = ui_state.selected_city_id.clone() else {
        ui.label(t.text("selected-city-none"));
        return;
    };
    let Some(city) = game.cities.get(&city_id).cloned() else {
        ui.label(t.text("selected-city-missing"));
        return;
    };

    let unknown = t.text("unknown");
    let faction_name = game
        .factions
        .get(&city.faction_id)
        .map(|faction| faction.name.as_str())
        .unwrap_or(unknown.as_str());
    let pending_command = pending_command_for_city(&game, &city.id).cloned();
    sync_command_selection_to_city_tab(ui_state);

    command_tent_header(ui, &game, &city, faction_name, pending_command.as_ref(), t);
    ui.add_space(8.0);
    city_tab_bar(ui, ui_state, t);
    ui.add_space(8.0);

    match ui_state.selected_city_tab {
        CityPanelTab::Overview => overview_tab(ui, &game, &city, faction_name, t),
        CityPanelTab::Domestic | CityPanelTab::Military | CityPanelTab::Diplomacy => {
            command_tab(ui, ui_state, &game, &city, pending_command.as_ref(), t);
        }
    }
}

fn command_tent_header(
    ui: &mut egui::Ui,
    game: &GameState,
    city: &City,
    faction_name: &str,
    pending_command: Option<&Command>,
    t: &Translator,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(&city.name)
                .size(24.0)
                .color(war_gold())
                .strong(),
        );
        ui.separator();
        ui.label(t.text_args(
            "command-tent-date",
            &args([
                ("faction", faction_name.to_string()),
                ("year", game.year.to_string()),
                ("month", game.month.to_string()),
                ("turn", game.turn.to_string()),
            ]),
        ));
        ui.separator();
        ui.label(t.text_args(
            "command-tent-resources",
            &args([
                ("gold", city.gold.to_string()),
                ("food", city.food.to_string()),
                ("materials", city.materials.to_string()),
                ("troops", city.troops.total().to_string()),
            ]),
        ));
        ui.separator();
        match pending_command {
            Some(command) => ui.colored_label(
                status_warning_color(),
                t.text_args(
                    "command-pending",
                    &args([("command", command_title(command, t))]),
                ),
            ),
            None => ui.colored_label(status_ready_color(), t.text("command-city-ready")),
        };
    });
}

fn city_tab_bar(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.horizontal(|ui| {
        for tab in [
            CityPanelTab::Overview,
            CityPanelTab::Domestic,
            CityPanelTab::Military,
            CityPanelTab::Diplomacy,
        ] {
            if ui
                .selectable_label(ui_state.selected_city_tab == tab, city_tab_label(tab, t))
                .clicked()
            {
                ui_state.selected_city_tab = tab;
                sync_command_selection_to_city_tab(ui_state);
            }
        }
    });
}

fn overview_tab(
    ui: &mut egui::Ui,
    game: &GameState,
    city: &City,
    faction_name: &str,
    t: &Translator,
) {
    let projection = city_projection(game, city);
    ui.columns(3, |columns| {
        columns[0].set_width(330.0);
        war_sub_panel_frame().show(&mut columns[0], |ui| {
            section_title(ui, &t.text("city-overview-title"));
            city_overview_intel(ui, city, faction_name, t);
            ui.add_space(8.0);
            city_resource_intel(ui, city, t);
        });

        columns[1].set_width(330.0);
        war_sub_panel_frame().show(&mut columns[1], |ui| {
            section_title(ui, &t.text("city-development-title"));
            city_development_intel(ui, city, t);
            ui.add_space(8.0);
            city_stability_intel(ui, city, t);
        });

        war_sub_panel_frame().show(&mut columns[2], |ui| {
            section_title(ui, &t.text("city-monthly-facilities-title"));
            city_monthly_trend_intel(ui, &projection, t);
            ui.add_space(8.0);
            city_facility_intel(ui, city, t);
            if city.faction_id != game.player_faction_id {
                ui.add_space(8.0);
                ui.colored_label(war_text_muted(), t.text("command-view-intel-only"));
            }
        });
    });
}

fn command_tab(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    pending_command: Option<&Command>,
    t: &Translator,
) {
    let available_officers = available_officers_for_city(game, city);
    let selected_officer_id = ensure_selected_officer(ui_state, city, &available_officers);
    let selected_officer = selected_officer_id
        .as_ref()
        .and_then(|officer_id| game.officers.get(officer_id));

    ui.columns(3, |columns| {
        columns[0].set_width(300.0);
        command_action_column(
            &mut columns[0],
            ui_state,
            game,
            city,
            &available_officers,
            selected_officer_id.as_ref(),
            t,
        );

        columns[1].set_width(370.0);
        let command_context = CommandPanelContext {
            game,
            city,
            selected_officer,
            selected_officer_id: selected_officer_id.as_ref(),
            available_officers: &available_officers,
            pending_command,
            t,
        };

        command_parameter_column(&mut columns[1], ui_state, &command_context);

        command_preview_column(&mut columns[2], ui_state, &command_context);
    });
}

fn command_action_column(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    available_officers: &[Officer],
    selected_officer_id: Option<&OfficerId>,
    t: &Translator,
) {
    war_sub_panel_frame().show(ui, |ui| {
        section_title(ui, &t.text("command-officer-section"));
        if city.faction_id != game.player_faction_id {
            ui.colored_label(war_text_muted(), t.text("command-view-only"));
            officer_roster_list(ui, game, city, 250.0, t);
        } else if available_officers.is_empty() {
            ui.colored_label(war_text_muted(), t.text("command-no-available-officers"));
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
        section_title(ui, &t.text("command-section"));
        for action in command_actions(ui_state.selected_command_category) {
            let response = ui.selectable_label(
                ui_state.selected_command_action == *action,
                command_action_label(*action, t),
            );
            if response.clicked() {
                ui_state.selected_command_action = *action;
            }
            response.on_hover_text(command_action_hint(*action, t));
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
        city_combined_effects(game, &city.id),
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

fn city_tab_label(tab: CityPanelTab, t: &Translator) -> String {
    match tab {
        CityPanelTab::Overview => t.text("city-tab-overview"),
        CityPanelTab::Domestic => t.text("city-tab-domestic"),
        CityPanelTab::Military => t.text("city-tab-military"),
        CityPanelTab::Diplomacy => t.text("city-tab-appointments-diplomacy"),
    }
}

struct CommandPanelContext<'a> {
    game: &'a GameState,
    city: &'a City,
    selected_officer: Option<&'a Officer>,
    selected_officer_id: Option<&'a OfficerId>,
    available_officers: &'a [Officer],
    pending_command: Option<&'a Command>,
    t: &'a Translator,
}

fn command_parameter_column(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    context: &CommandPanelContext<'_>,
) {
    war_sub_panel_frame().show(ui, |ui| {
        let CommandPanelContext {
            game,
            city,
            selected_officer,
            selected_officer_id,
            available_officers,
            pending_command,
            t,
        } = *context;
        section_title(
            ui,
            &command_action_label(ui_state.selected_command_action, t),
        );
        ui.colored_label(
            war_text_muted(),
            command_action_hint(ui_state.selected_command_action, t),
        );
        ui.separator();

        if city.faction_id != game.player_faction_id {
            ui.colored_label(war_text_muted(), t.text("command-view-intel-only"));
            return;
        }

        if pending_command.is_some() {
            ui.colored_label(war_text_muted(), t.text("command-withdraw-before-edit"));
            return;
        }

        match ui_state.selected_command_action {
            CommandAction::RecruitOfficer => {
                recruit_officer_parameter_controls(ui, ui_state, game, city, selected_officer, t)
            }
            _ if selected_officer.is_none() => {
                ui.colored_label(war_text_muted(), t.text("command-select-officer"));
            }
            CommandAction::Develop => develop_parameter_controls(ui, ui_state, selected_officer, t),
            CommandAction::UpgradeCityCore => upgrade_parameter_controls(ui, city, t),
            CommandAction::BuildFacility => facility_parameter_controls(ui, ui_state, city, t),
            CommandAction::Recruit => recruit_parameter_controls(ui, ui_state, game, city, t),
            CommandAction::Train => train_parameter_controls(ui, selected_officer, t),
            CommandAction::AppointGovernor => {
                appoint_parameter_controls(ui, city, selected_officer, t)
            }
            CommandAction::Transfer => transfer_parameter_controls(ui, ui_state, game, city, t),
            CommandAction::Expedition => expedition_parameter_controls(
                ui,
                ui_state,
                game,
                city,
                selected_officer_id,
                available_officers,
                t,
            ),
        }
    });
}

fn command_preview_column(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    context: &CommandPanelContext<'_>,
) {
    war_sub_panel_frame().show(ui, |ui| {
        let CommandPanelContext {
            game,
            city,
            selected_officer,
            selected_officer_id,
            pending_command,
            t,
            ..
        } = *context;
        section_title(ui, &t.text("command-preview-title"));
        if ui_state.selected_command_action == CommandAction::RecruitOfficer {
            recruitment_preview_column(ui, ui_state, game, city, selected_officer, t);
            return;
        }
        if let Some(command) = pending_command {
            ui.colored_label(status_warning_color(), t.text("command-existing-pending"));
            ui.label(command_summary(game, command, t));
            ui.separator();
            if ui.button(t.text("command-withdraw-city")).clicked() {
                withdraw_pending_command(ui_state, &city.id, t);
            }
            return;
        }

        if city.faction_id != game.player_faction_id {
            ui.colored_label(war_text_muted(), t.text("command-non-owned-disabled"));
            return;
        }

        let Some(command) = build_candidate_command(ui_state, game, city, selected_officer_id)
        else {
            ui.colored_label(war_text_muted(), t.text("command-incomplete"));
            return;
        };

        for line in command_preview_lines(game, city, selected_officer, &command, t) {
            ui.label(line);
        }
        ui.separator();

        match preview_command(game, &command) {
            Ok(()) => {
                ui.colored_label(status_ready_color(), t.text("command-can-submit"));
                if ui.button(t.text("command-submit")).clicked() {
                    submit_candidate_command(ui_state, command, t);
                }
            }
            Err(message) => {
                ui.colored_label(
                    status_error_color(),
                    t.text_args("command-cannot-submit", &args([("message", message)])),
                );
                ui.add_enabled(false, egui::Button::new(t.text("command-submit")));
            }
        }
    });
}

fn recruitment_preview_column(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    selected_officer: Option<&Officer>,
    t: &Translator,
) {
    if city.faction_id != game.player_faction_id {
        ui.colored_label(war_text_muted(), t.text("command-non-owned-disabled"));
        return;
    }
    let active_tasks = game
        .officer_recruitments
        .iter()
        .filter(|task| task.source_city_id == city.id)
        .collect::<Vec<_>>();
    if !active_tasks.is_empty() {
        ui.label(t.text("command-recruit-officer-active-title"));
        for task in active_tasks {
            ui.horizontal_wrapped(|ui| {
                ui.label(recruitment_task_line(game, task, t));
                if ui
                    .button(t.text("command-recruit-officer-cancel"))
                    .clicked()
                {
                    cancel_recruitment_task(ui_state, &task.id, t);
                }
            });
        }
        ui.separator();
    }

    let Some(recruiter) = selected_officer else {
        ui.colored_label(war_text_muted(), t.text("command-select-officer"));
        return;
    };
    let Some(target_id) = ui_state.selected_recruitment_target.clone() else {
        ui.colored_label(war_text_muted(), t.text("command-recruit-officer-select"));
        return;
    };
    let target_name = officer_name(game, &target_id);
    ui.label(t.text_args(
        "preview-recruit-officer",
        &args([
            ("recruiter", recruiter.name.clone()),
            ("target", target_name),
        ]),
    ));
    match preview_officer_recruitment(game, &city.id, &recruiter.id, &target_id) {
        Ok(()) => {
            ui.colored_label(status_ready_color(), t.text("command-can-submit"));
            if ui
                .button(t.text("command-recruit-officer-submit"))
                .clicked()
            {
                submit_officer_recruitment(ui_state, &city.id, &recruiter.id, &target_id, t);
            }
        }
        Err(message) => {
            ui.colored_label(
                status_error_color(),
                t.text_args("command-cannot-submit", &args([("message", message)])),
            );
            ui.add_enabled(
                false,
                egui::Button::new(t.text("command-recruit-officer-submit")),
            );
        }
    }
}

const DOMESTIC_ACTIONS: [CommandAction; 5] = [
    CommandAction::Develop,
    CommandAction::UpgradeCityCore,
    CommandAction::BuildFacility,
    CommandAction::Recruit,
    CommandAction::Train,
];
const MILITARY_ACTIONS: [CommandAction; 2] = [CommandAction::Transfer, CommandAction::Expedition];
const DIPLOMACY_ACTIONS: [CommandAction; 2] = [
    CommandAction::AppointGovernor,
    CommandAction::RecruitOfficer,
];

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

fn command_action_label(action: CommandAction, t: &Translator) -> String {
    match action {
        CommandAction::Develop => t.text("command-develop"),
        CommandAction::UpgradeCityCore => t.text("command-upgrade-core"),
        CommandAction::BuildFacility => t.text("command-build-facility"),
        CommandAction::Recruit => t.text("command-recruit"),
        CommandAction::RecruitOfficer => t.text("command-recruit-officer"),
        CommandAction::Train => t.text("command-train"),
        CommandAction::AppointGovernor => t.text("command-appoint-governor"),
        CommandAction::Transfer => t.text("command-transfer"),
        CommandAction::Expedition => t.text("command-expedition"),
    }
}

fn command_action_hint(action: CommandAction, t: &Translator) -> String {
    match action {
        CommandAction::Develop => t.text("command-develop-hint"),
        CommandAction::UpgradeCityCore => t.text("command-upgrade-core-hint"),
        CommandAction::BuildFacility => t.text("command-build-facility-hint"),
        CommandAction::Recruit => t.text("command-recruit-hint"),
        CommandAction::RecruitOfficer => t.text("command-recruit-officer-hint"),
        CommandAction::Train => t.text("command-train-hint"),
        CommandAction::AppointGovernor => t.text("command-appoint-governor-hint"),
        CommandAction::Transfer => t.text("command-transfer-hint"),
        CommandAction::Expedition => t.text("command-expedition-hint"),
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
    t: &Translator,
) {
    egui::ComboBox::from_id_salt("command_develop_focus")
        .selected_text(development_focus_label(t, &ui_state.selected_focus))
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
                    development_focus_label(t, &focus),
                );
            }
        });
    if let Some(officer) = selected_officer {
        ui.label(t.text_args(
            "command-develop-cost-hint",
            &args([("officer", officer.name.clone())]),
        ));
    }
}

fn upgrade_parameter_controls(ui: &mut egui::Ui, city: &City, t: &Translator) {
    ui.label(t.text_args(
        "command-upgrade-current",
        &args([
            ("level", city.level.to_string()),
            ("slots", city.facility_slots().to_string()),
        ]),
    ));
    if city.level >= CITY_MAX_LEVEL {
        ui.colored_label(war_text_muted(), t.text("command-upgrade-max-level"));
        return;
    }
    let next_level = city.level + 1;
    let cost = city_core_upgrade_cost(next_level);
    ui.label(t.text_args(
        "command-upgrade-cost",
        &args([
            ("level", next_level.to_string()),
            ("gold", cost.gold.to_string()),
            ("food", cost.food.to_string()),
            ("materials", cost.materials.to_string()),
        ]),
    ));
    ui.label(t.text("command-upgrade-order-hint"));
}

fn facility_parameter_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    city: &City,
    t: &Translator,
) {
    egui::ComboBox::from_id_salt("command_facility_kind")
        .selected_text(facility_kind_label(t, ui_state.selected_facility_kind))
        .show_ui(ui, |ui| {
            for kind in ALL_FACILITY_KINDS {
                ui.selectable_value(
                    &mut ui_state.selected_facility_kind,
                    kind,
                    facility_kind_label(t, kind),
                );
            }
        });

    match facility_build_preview(city, ui_state.selected_facility_kind) {
        Ok((target_level, cost, action)) => {
            ui.label(t.text_args(
                "command-facility-cost",
                &args([
                    ("action", t.text(action)),
                    ("level", target_level.to_string()),
                    ("gold", cost.gold.to_string()),
                    ("food", cost.food.to_string()),
                    ("materials", cost.materials.to_string()),
                ]),
            ));
        }
        Err(message) => {
            ui.colored_label(war_text_muted(), t.text(message));
        }
    }
}

fn recruit_parameter_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    t: &Translator,
) {
    egui::ComboBox::from_id_salt("command_recruit_kind")
        .selected_text(troop_kind_label(t, ui_state.selected_recruit_kind))
        .show_ui(ui, |ui| {
            for kind in TroopKind::ALL {
                ui.selectable_value(
                    &mut ui_state.selected_recruit_kind,
                    kind,
                    troop_kind_label(t, kind),
                );
            }
        });
    ui_state.recruit_amount = ui_state.recruit_amount.clamp(100, 5000);
    ui.add(
        egui::Slider::new(&mut ui_state.recruit_amount, 100..=5000)
            .text(t.text("command-recruit-amount")),
    );
    let cost = recruit_cost_for_faction_kind(
        game,
        &city.faction_id,
        ui_state.selected_recruit_kind,
        ui_state.recruit_amount,
    );
    ui.label(t.text_args(
        "command-recruit-cost",
        &args([
            ("gold", cost.gold.to_string()),
            ("food", cost.food.to_string()),
            ("population", (ui_state.recruit_amount * 2).to_string()),
        ]),
    ));
    ui.label(t.text_args(
        "command-recruit-current",
        &args([
            ("population", city.population.to_string()),
            ("troops", troop_pool_summary(city.troops, t)),
        ]),
    ));
}

fn recruit_officer_parameter_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    selected_officer: Option<&Officer>,
    t: &Translator,
) {
    let candidates = wild_recruitment_candidates(game);
    if candidates.is_empty() {
        ui.colored_label(war_text_muted(), t.text("command-recruit-officer-none"));
        ui_state.selected_recruitment_target = None;
        return;
    }
    ensure_selected_recruitment_target(ui_state, &candidates);
    let selected_target_id = ui_state.selected_recruitment_target.clone();
    egui::ComboBox::from_id_salt("command_recruitment_target")
        .selected_text(
            selected_target_id
                .as_deref()
                .and_then(|id| game.officers.get(id))
                .map(|officer| wild_officer_label(game, officer, t))
                .unwrap_or_else(|| t.text("common-none-selected")),
        )
        .show_ui(ui, |ui| {
            for officer in &candidates {
                ui.selectable_value(
                    &mut ui_state.selected_recruitment_target,
                    Some(officer.id.clone()),
                    wild_officer_label(game, officer, t),
                );
            }
        });

    if let Some(task) = active_recruitment_for_source(game, &city.id) {
        ui.separator();
        ui.colored_label(war_warning(), t.text("command-recruit-officer-active"));
        ui.label(recruitment_task_line(game, task, t));
    }

    let Some(target_id) = ui_state.selected_recruitment_target.as_deref() else {
        return;
    };
    if let Some(target) = game.officers.get(target_id) {
        ui.label(t.text_args(
            "command-recruit-officer-target",
            &args([
                ("officer", target.name.clone()),
                ("city", officer_city_name(game, target, t)),
                ("charm", target.stats.charm.to_string()),
            ]),
        ));
    }
    if let Some(recruiter) = selected_officer {
        let preview_task = OfficerRecruitmentTask {
            id: "preview".to_string(),
            issuer_faction_id: game.player_faction_id.clone(),
            source_city_id: city.id.clone(),
            recruiter_officer_id: recruiter.id.clone(),
            target_officer_id: target_id.to_string(),
            progress: 0,
            attempt_months: 0,
            started_turn: game.turn,
        };
        if let Some(gain) = officer_recruitment_progress_gain(game, &preview_task) {
            ui.label(t.text_args(
                "command-recruit-officer-gain",
                &args([("gain", gain.to_string())]),
            ));
        }
    }
}

fn train_parameter_controls(ui: &mut egui::Ui, selected_officer: Option<&Officer>, t: &Translator) {
    ui.label(t.text("command-train-cost-hint"));
    if let Some(officer) = selected_officer {
        let gain = (6 + officer.stats.leadership / 12).min(15);
        ui.label(t.text_args(
            "command-train-estimate",
            &args([
                ("gain", gain.to_string()),
                ("officer", officer.name.clone()),
            ]),
        ));
    }
}

fn appoint_parameter_controls(
    ui: &mut egui::Ui,
    city: &City,
    selected_officer: Option<&Officer>,
    t: &Translator,
) {
    if let Some(officer) = selected_officer {
        ui.label(t.text_args(
            "command-appoint-target",
            &args([
                ("officer", officer.name.clone()),
                ("city", city.name.clone()),
            ]),
        ));
        ui.label(t.text("command-appoint-hint"));
    }
}

fn transfer_parameter_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    t: &Translator,
) {
    let targets = adjacent_cities(game, city, true);
    if targets.is_empty() {
        ui.colored_label(war_text_muted(), t.text("command-transfer-no-targets"));
        return;
    }
    let Some(selected_target_id) =
        ensure_selected_target(&mut ui_state.selected_transfer_target, &targets)
    else {
        return;
    };
    egui::ComboBox::from_id_salt("command_transfer_target")
        .selected_text(target_travel_label(game, &city.id, &selected_target_id, t))
        .show_ui(ui, |ui| {
            for target in &targets {
                ui.selectable_value(
                    &mut ui_state.selected_transfer_target,
                    Some(target.id.clone()),
                    target_travel_label(game, &city.id, &target.id, t),
                );
            }
        });
    ui.label(travel_summary_line(game, &city.id, &selected_target_id, t));
    if city.troops.is_empty() {
        ui_state.transfer_troops = TroopPool::default();
        ui.colored_label(war_text_muted(), t.text("command-transfer-no-troops"));
    } else {
        ui.label(t.text_args(
            "command-city-garrison",
            &args([("troops", troop_pool_summary(city.troops, t))]),
        ));
        troop_pool_sliders(ui, &mut ui_state.transfer_troops, city.troops, false, t);
    }
}

fn troop_pool_sliders(
    ui: &mut egui::Ui,
    selected: &mut TroopPool,
    available: TroopPool,
    allow_zero_total: bool,
    t: &Translator,
) {
    for kind in TroopKind::ALL {
        let max = available.get(kind);
        let mut value = selected.get(kind).min(max);
        ui.add(egui::Slider::new(&mut value, 0..=max).text(troop_kind_label(t, kind)));
        selected.set(kind, value);
    }
    if !allow_zero_total && selected.is_empty() && !available.is_empty() {
        for fallback in TroopKind::ALL {
            let amount = available.get(fallback).min(1);
            if amount > 0 {
                selected.add(fallback, amount);
                break;
            }
        }
    }
}

fn expedition_parameter_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    city: &City,
    selected_officer_id: Option<&OfficerId>,
    available_officers: &[Officer],
    t: &Translator,
) {
    let targets = adjacent_cities(game, city, false);
    if targets.is_empty() {
        ui.colored_label(war_text_muted(), t.text("command-expedition-no-targets"));
        return;
    }
    let Some(selected_target_id) =
        ensure_selected_target(&mut ui_state.selected_expedition_target, &targets)
    else {
        return;
    };
    egui::ComboBox::from_id_salt("command_expedition_target")
        .selected_text(target_travel_label(game, &city.id, &selected_target_id, t))
        .show_ui(ui, |ui| {
            for target in &targets {
                let unknown = t.text("unknown");
                let faction_name = game
                    .factions
                    .get(&target.faction_id)
                    .map(|faction| faction.name.as_str())
                    .unwrap_or(unknown.as_str());
                ui.selectable_value(
                    &mut ui_state.selected_expedition_target,
                    Some(target.id.clone()),
                    t.text_args(
                        "target-travel-with-faction",
                        &args([
                            ("city", target.name.clone()),
                            ("faction", faction_name.to_string()),
                            ("travel", travel_summary(game, &city.id, &target.id, t)),
                        ]),
                    ),
                );
            }
        });
    ui.label(travel_summary_line(game, &city.id, &selected_target_id, t));
    if let Some(target) = game.cities.get(&selected_target_id) {
        ui.label(
            t.text_args(
                "command-expedition-target-summary",
                &args([
                    ("troops", troop_pool_summary(target.troops, t)),
                    ("defense", target.defense.to_string()),
                    (
                        "governor",
                        target
                            .governor_id
                            .as_deref()
                            .map(|officer_id| officer_name(game, officer_id))
                            .unwrap_or_else(|| t.text("none")),
                    ),
                ]),
            ),
        );
    }
    if city.troops.is_empty() {
        ui_state.expedition_main_troops = 0;
        ui_state.expedition_deputy_one_troops = 0;
        ui_state.expedition_deputy_two_troops = 0;
        ui.colored_label(war_text_muted(), t.text("command-expedition-no-troops"));
        return;
    }

    let Some(main_id) = selected_officer_id else {
        return;
    };
    ui.separator();
    ui.label(t.text_args(
        "command-city-garrison",
        &args([("troops", troop_pool_summary(city.troops, t))]),
    ));
    if let Some(main) = game.officers.get(main_id) {
        expedition_assignment_controls(
            ui,
            &t.text("expedition-role-commander"),
            main,
            ExpeditionAssignmentControls {
                available: city.troops,
                kind: &mut ui_state.expedition_main_kind,
                troops: &mut ui_state.expedition_main_troops,
                required: true,
            },
            t,
        );
    }

    normalize_deputy_selection(
        &mut ui_state.expedition_deputy_one,
        available_officers,
        &[
            Some(main_id.as_str()),
            ui_state.expedition_deputy_two.as_deref(),
        ],
    );
    normalize_deputy_selection(
        &mut ui_state.expedition_deputy_two,
        available_officers,
        &[
            Some(main_id.as_str()),
            ui_state.expedition_deputy_one.as_deref(),
        ],
    );
    officer_option_combo(
        ui,
        &t.text("expedition-role-deputy-one"),
        "command_expedition_deputy_one",
        &mut ui_state.expedition_deputy_one,
        available_officers,
        &[
            Some(main_id.as_str()),
            ui_state.expedition_deputy_two.as_deref(),
        ],
        t,
    );
    if let Some(officer) = ui_state
        .expedition_deputy_one
        .as_ref()
        .and_then(|id| game.officers.get(id))
    {
        expedition_assignment_controls(
            ui,
            &t.text("expedition-role-deputy-one"),
            officer,
            ExpeditionAssignmentControls {
                available: city.troops,
                kind: &mut ui_state.expedition_deputy_one_kind,
                troops: &mut ui_state.expedition_deputy_one_troops,
                required: false,
            },
            t,
        );
    } else {
        ui_state.expedition_deputy_one_troops = 0;
    }
    officer_option_combo(
        ui,
        &t.text("expedition-role-deputy-two"),
        "command_expedition_deputy_two",
        &mut ui_state.expedition_deputy_two,
        available_officers,
        &[
            Some(main_id.as_str()),
            ui_state.expedition_deputy_one.as_deref(),
        ],
        t,
    );
    if let Some(officer) = ui_state
        .expedition_deputy_two
        .as_ref()
        .and_then(|id| game.officers.get(id))
    {
        expedition_assignment_controls(
            ui,
            &t.text("expedition-role-deputy-two"),
            officer,
            ExpeditionAssignmentControls {
                available: city.troops,
                kind: &mut ui_state.expedition_deputy_two_kind,
                troops: &mut ui_state.expedition_deputy_two_troops,
                required: false,
            },
            t,
        );
    } else {
        ui_state.expedition_deputy_two_troops = 0;
    }

    let assignments = expedition_assignments_from_ui(ui_state, main_id);
    let departing_troops = expedition_assignment_pool(&assignments);
    let monthly_supply = expedition_monthly_supply_for_troops(departing_troops);
    let max_supply = city.food.max(0) as u32;
    if max_supply == 0 {
        ui_state.expedition_food_supply = 0;
        ui.colored_label(war_text_muted(), t.text("command-expedition-no-food"));
        return;
    }
    let travel_months = game
        .road_distance_li(&city.id, &selected_target_id)
        .map(|distance| travel_months_for_faction(game, &city.faction_id, distance))
        .unwrap_or(1);
    let recommended_supply = monthly_supply
        .saturating_mul(travel_months.saturating_add(3))
        .clamp(1, max_supply);
    if ui_state.expedition_food_supply == 0 || ui_state.expedition_food_supply > max_supply {
        ui_state.expedition_food_supply = recommended_supply;
    }
    ui.add(
        egui::Slider::new(&mut ui_state.expedition_food_supply, 1..=max_supply)
            .text(t.text("command-expedition-food-supply")),
    );
    let supported_months = ui_state.expedition_food_supply / monthly_supply.max(1);
    ui.label(t.text_args(
        "command-expedition-supply-estimate",
        &args([
            ("monthly", monthly_supply.to_string()),
            ("months", supported_months.to_string()),
            ("recommended", recommended_supply.to_string()),
        ]),
    ));
}

struct ExpeditionAssignmentControls<'a> {
    available: TroopPool,
    kind: &'a mut TroopKind,
    troops: &'a mut u32,
    required: bool,
}

fn expedition_assignment_controls(
    ui: &mut egui::Ui,
    label: &str,
    officer: &Officer,
    assignment: ExpeditionAssignmentControls<'_>,
    t: &Translator,
) {
    let ExpeditionAssignmentControls {
        available,
        kind,
        troops,
        required,
    } = assignment;
    let capacity = command_capacity_for_officer(officer);
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(format!("{label}: {}", officer.name));
            ui.colored_label(
                war_text_muted(),
                t.text_args(
                    "expedition-capacity",
                    &args([("capacity", capacity.to_string())]),
                ),
            );
        });
        egui::ComboBox::from_id_salt(("expedition_kind", label, &officer.id))
            .selected_text(troop_kind_label(t, *kind))
            .show_ui(ui, |ui| {
                for troop_kind in TroopKind::ALL {
                    ui.selectable_value(kind, troop_kind, troop_kind_label(t, troop_kind));
                }
            });
        let max_troops = available.get(*kind).min(capacity);
        if max_troops == 0 {
            *troops = 0;
            ui.colored_label(war_text_muted(), t.text("expedition-kind-no-troops"));
            return;
        }
        let min_troops = u32::from(required);
        *troops = (*troops).clamp(min_troops, max_troops);
        ui.add(
            egui::Slider::new(troops, min_troops..=max_troops).text(t.text_args(
                "expedition-kind-troops",
                &args([("kind", troop_kind_label(t, *kind))]),
            )),
        );
    });
}

fn normalize_deputy_selection(
    selected: &mut Option<OfficerId>,
    available_officers: &[Officer],
    excluded: &[Option<&str>],
) {
    let Some(selected_id) = selected.as_deref() else {
        return;
    };
    let valid = available_officers.iter().any(|officer| {
        officer.id == selected_id
            && !excluded
                .iter()
                .flatten()
                .any(|excluded_id| *excluded_id == officer.id)
    });
    if !valid {
        *selected = None;
    }
}

fn officer_option_combo(
    ui: &mut egui::Ui,
    label: &str,
    id_salt: &'static str,
    selected: &mut Option<OfficerId>,
    available_officers: &[Officer],
    excluded: &[Option<&str>],
    t: &Translator,
) {
    let selected_text = selected
        .as_deref()
        .and_then(|id| available_officers.iter().find(|officer| officer.id == id))
        .map(|officer| officer.name.clone())
        .unwrap_or_else(|| t.text("common-none-selected"));
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(t.text_args(
            "officer-option-selected",
            &args([("label", label.to_string()), ("selected", selected_text)]),
        ))
        .show_ui(ui, |ui| {
            ui.selectable_value(selected, None, t.text("common-none-selected"));
            for officer in available_officers {
                if excluded
                    .iter()
                    .flatten()
                    .any(|excluded_id| *excluded_id == officer.id)
                {
                    continue;
                }
                ui.selectable_value(
                    selected,
                    Some(officer.id.clone()),
                    officer_command_label(officer),
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
            ownership_matches
                && route_distance_li_for_faction(game, &city.faction_id, &city.id, &target.id, !own)
                    .is_some()
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

fn target_travel_label(
    game: &GameState,
    from_city_id: &str,
    target_city_id: &str,
    t: &Translator,
) -> String {
    t.text_args(
        "target-travel-label",
        &args([
            ("city", city_name(game, target_city_id)),
            (
                "travel",
                travel_summary(game, from_city_id, target_city_id, t),
            ),
        ]),
    )
}

fn travel_summary_line(
    game: &GameState,
    from_city_id: &str,
    target_city_id: &str,
    t: &Translator,
) -> String {
    t.text_args(
        "travel-summary-line",
        &args([(
            "travel",
            travel_summary(game, from_city_id, target_city_id, t),
        )]),
    )
}

fn travel_summary(
    game: &GameState,
    from_city_id: &str,
    target_city_id: &str,
    t: &Translator,
) -> String {
    match (
        game.cities.get(from_city_id).and_then(|from_city| {
            let target_may_be_hostile = game
                .cities
                .get(target_city_id)
                .is_some_and(|target_city| target_city.faction_id != from_city.faction_id);
            route_distance_li_for_faction(
                game,
                &from_city.faction_id,
                from_city_id,
                target_city_id,
                target_may_be_hostile,
            )
        }),
        game.cities.get(from_city_id).and_then(|from_city| {
            let target_may_be_hostile = game
                .cities
                .get(target_city_id)
                .is_some_and(|target_city| target_city.faction_id != from_city.faction_id);
            route_distance_li_for_faction(
                game,
                &from_city.faction_id,
                from_city_id,
                target_city_id,
                target_may_be_hostile,
            )
            .map(|distance| travel_months_for_faction(game, &from_city.faction_id, distance))
        }),
    ) {
        (Some(distance), Some(months)) => t.text_args(
            "travel-summary",
            &args([
                ("distance", distance.to_string()),
                ("months", months.to_string()),
            ]),
        ),
        _ => t.text("travel-unknown"),
    }
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
            kind: ui_state.selected_recruit_kind,
            amount: ui_state.recruit_amount,
        },
        CommandAction::RecruitOfficer => return None,
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
            assignments: expedition_assignments_from_ui(ui_state, &officer_id),
            food_supply: ui_state.expedition_food_supply,
        },
    };
    Some(Command {
        issuer_faction_id: game.player_faction_id.clone(),
        city_id: city.id.clone(),
        officer_id: Some(officer_id),
        kind,
    })
}

fn expedition_assignments_from_ui(
    ui_state: &GameUiState,
    commander_id: &OfficerId,
) -> Vec<ExpeditionAssignment> {
    let mut assignments = vec![ExpeditionAssignment::commander(
        commander_id.clone(),
        ui_state.expedition_main_kind,
        ui_state.expedition_main_troops,
    )];
    if let Some(officer_id) = &ui_state.expedition_deputy_one
        && ui_state.expedition_deputy_one_troops > 0
    {
        assignments.push(ExpeditionAssignment::deputy(
            officer_id.clone(),
            ui_state.expedition_deputy_one_kind,
            ui_state.expedition_deputy_one_troops,
        ));
    }
    if let Some(officer_id) = &ui_state.expedition_deputy_two
        && ui_state.expedition_deputy_two_troops > 0
    {
        assignments.push(ExpeditionAssignment::deputy(
            officer_id.clone(),
            ui_state.expedition_deputy_two_kind,
            ui_state.expedition_deputy_two_troops,
        ));
    }
    assignments
}

fn preview_command(game: &GameState, command: &Command) -> Result<(), String> {
    let mut preview = game.clone();
    queue_player_command(&mut preview, command.clone()).map_err(|error| error.to_string())
}

fn submit_candidate_command(ui_state: &mut GameUiState, command: Command, t: &Translator) {
    let Some(game) = &mut ui_state.game else {
        ui_state.message = t.text("message-game-not-started");
        return;
    };
    let city_name = city_name(game, &command.city_id);
    match queue_player_command(game, command) {
        Ok(()) => {
            ui_state.message =
                t.text_args("message-command-submitted", &args([("city", city_name)]))
        }
        Err(error) => ui_state.message = error.to_string(),
    }
}

fn withdraw_pending_command(ui_state: &mut GameUiState, city_id: &str, t: &Translator) {
    let Some(game) = &mut ui_state.game else {
        ui_state.message = t.text("message-game-not-started");
        return;
    };
    if remove_pending_command_for_city(game, city_id).is_some() {
        let city_name = city_name(game, city_id);
        ui_state.message = t.text_args("message-command-withdrawn", &args([("city", city_name)]));
    } else {
        ui_state.message = t.text("message-command-no-pending");
    }
}

fn remove_pending_command_for_city(game: &mut GameState, city_id: &str) -> Option<Command> {
    let index = game
        .pending_commands
        .iter()
        .position(|command| command.city_id == city_id)?;
    Some(game.pending_commands.remove(index))
}

fn preview_officer_recruitment(
    game: &GameState,
    source_city_id: &str,
    recruiter_officer_id: &str,
    target_officer_id: &str,
) -> Result<(), String> {
    let mut preview = game.clone();
    start_officer_recruitment(
        &mut preview,
        &game.player_faction_id,
        source_city_id,
        recruiter_officer_id,
        target_officer_id,
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn submit_officer_recruitment(
    ui_state: &mut GameUiState,
    source_city_id: &str,
    recruiter_officer_id: &str,
    target_officer_id: &str,
    t: &Translator,
) {
    let Some(game) = &mut ui_state.game else {
        ui_state.message = t.text("message-game-not-started");
        return;
    };
    match start_officer_recruitment(
        game,
        &game.player_faction_id.clone(),
        source_city_id,
        recruiter_officer_id,
        target_officer_id,
    ) {
        Ok(task) => {
            ui_state.message = t.text_args(
                "message-recruitment-started",
                &args([
                    ("recruiter", officer_name(game, &task.recruiter_officer_id)),
                    ("target", officer_name(game, &task.target_officer_id)),
                ]),
            );
        }
        Err(error) => ui_state.message = error.to_string(),
    }
}

fn cancel_recruitment_task(ui_state: &mut GameUiState, task_id: &str, t: &Translator) {
    let Some(game) = &mut ui_state.game else {
        ui_state.message = t.text("message-game-not-started");
        return;
    };
    match cancel_officer_recruitment(game, task_id) {
        Ok(task) => {
            ui_state.message = t.text_args(
                "message-recruitment-cancelled",
                &args([("target", officer_name(game, &task.target_officer_id))]),
            );
        }
        Err(error) => ui_state.message = error.to_string(),
    }
}

fn active_recruitment_for_source<'a>(
    game: &'a GameState,
    city_id: &str,
) -> Option<&'a OfficerRecruitmentTask> {
    game.officer_recruitments
        .iter()
        .find(|task| task.source_city_id == city_id)
}

fn command_preview_lines(
    game: &GameState,
    city: &City,
    officer: Option<&Officer>,
    command: &Command,
    t: &Translator,
) -> Vec<String> {
    let mut lines = vec![
        t.text_args("preview-city", &args([("city", city.name.clone())])),
        t.text_args(
            "preview-officer",
            &args([(
                "officer",
                officer
                    .map(|officer| officer.name.clone())
                    .unwrap_or_else(|| t.text("common-none-selected")),
            )]),
        ),
        t.text_args(
            "preview-command",
            &args([("command", command_title(command, t))]),
        ),
    ];
    match &command.kind {
        CommandKind::Develop { focus } => {
            lines.push(t.text_args(
                "preview-develop-target",
                &args([("focus", development_focus_label(t, focus))]),
            ));
            lines.push(t.text_args(
                "resource-cost-gold-only",
                &args([("gold", "80".to_string())]),
            ));
        }
        CommandKind::UpgradeCityCore => {
            if city.level < CITY_MAX_LEVEL {
                let cost = city_core_upgrade_cost(city.level + 1);
                lines.push(t.text_args(
                    "preview-upgrade-target",
                    &args([("level", (city.level + 1).to_string())]),
                ));
                lines.push(resource_cost_line("resource-cost", cost, t));
            }
        }
        CommandKind::BuildFacility { kind } => match facility_build_preview(city, *kind) {
            Ok((target_level, cost, action)) => {
                lines.push(t.text_args(
                    "preview-facility-target",
                    &args([
                        ("action", t.text(action)),
                        ("facility", facility_kind_label(t, *kind)),
                        ("level", target_level.to_string()),
                    ]),
                ));
                lines.push(resource_cost_line("resource-cost", cost, t));
            }
            Err(message) => lines.push(t.text(message)),
        },
        CommandKind::Recruit { kind, amount } => {
            lines.push(t.text_args(
                "preview-troop-kind",
                &args([("kind", troop_kind_label(t, *kind))]),
            ));
            lines.push(t.text_args(
                "preview-troops-add",
                &args([("amount", amount.to_string())]),
            ));
            lines.push(resource_cost_line(
                "resource-cost",
                recruit_cost_for_faction_kind(game, &command.issuer_faction_id, *kind, *amount),
                t,
            ));
            lines.push(t.text_args(
                "preview-population-cost",
                &args([("population", (amount * 2).to_string())]),
            ));
        }
        CommandKind::Train => {
            lines.push(t.text_args(
                "resource-cost-gold-only",
                &args([("gold", "40".to_string())]),
            ));
            lines.push(t.text("preview-train-effect"));
        }
        CommandKind::AppointGovernor { target_officer_id } => {
            lines.push(t.text_args(
                "preview-appoint-target",
                &args([("officer", officer_name(game, target_officer_id))]),
            ));
        }
        CommandKind::Transfer {
            target_city_id,
            troops,
            ..
        } => {
            lines.push(t.text_args(
                "preview-target",
                &args([("target", city_name(game, target_city_id))]),
            ));
            lines.push(travel_summary_line(
                game,
                &command.city_id,
                target_city_id,
                t,
            ));
            lines.push(t.text_args(
                "preview-departing-troops",
                &args([("troops", troop_pool_summary(*troops, t))]),
            ));
        }
        CommandKind::Expedition {
            target_city_id,
            assignments,
            food_supply,
        } => {
            let target = game.cities.get(target_city_id);
            lines.push(t.text_args(
                "preview-target",
                &args([("target", city_name(game, target_city_id))]),
            ));
            if let Some(target) = target {
                lines.push(t.text_args(
                    "preview-enemy",
                    &args([("faction", faction_name(game, &target.faction_id))]),
                ));
            }
            lines.push(travel_summary_line(
                game,
                &command.city_id,
                target_city_id,
                t,
            ));
            lines.push(t.text_args(
                "preview-departing-troops",
                &args([(
                    "troops",
                    troop_pool_summary(expedition_assignment_pool(assignments), t),
                )]),
            ));
            let monthly_supply =
                expedition_monthly_supply_for_troops(expedition_assignment_pool(assignments));
            lines.push(t.text_args(
                "preview-expedition-supply",
                &args([
                    ("supply", food_supply.to_string()),
                    ("monthly", monthly_supply.to_string()),
                    ("months", (food_supply / monthly_supply.max(1)).to_string()),
                ]),
            ));
            for assignment in assignments {
                lines.push(t.text_args(
                    "preview-assignment",
                    &args([
                        ("officer", officer_name(game, &assignment.officer_id)),
                        ("kind", troop_kind_label(t, assignment.troop_kind)),
                        ("troops", assignment.troops.to_string()),
                    ]),
                ));
            }
        }
        CommandKind::Diplomacy {
            target_faction_id,
            proposal,
        } => {
            lines.push(t.text_args(
                "preview-target",
                &args([("target", faction_name(game, target_faction_id))]),
            ));
            lines.push(t.text_args(
                "preview-proposal",
                &args([("proposal", diplomacy_label(t, proposal))]),
            ));
        }
    }
    lines
}

fn resource_cost_line(key: &str, cost: ResourceCost, t: &Translator) -> String {
    t.text_args(
        key,
        &args([
            ("gold", cost.gold.to_string()),
            ("food", cost.food.to_string()),
            ("materials", cost.materials.to_string()),
        ]),
    )
}

fn troop_pool_summary(troops: TroopPool, t: &Translator) -> String {
    t.text_args(
        "troop-pool-total",
        &args([
            ("total", troops.total().to_string()),
            ("infantry", troops.infantry.to_string()),
            ("cavalry", troops.cavalry.to_string()),
            ("archers", troops.archers.to_string()),
        ]),
    )
}

fn wild_recruitment_candidates(game: &GameState) -> Vec<Officer> {
    let tasked_targets = game
        .officer_recruitments
        .iter()
        .map(|task| task.target_officer_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut officers = game
        .officers
        .values()
        .filter(|officer| {
            officer.status == OfficerStatus::Wild
                && officer.faction_id == WILD_FACTION_ID
                && officer.is_adult_at(game.year)
                && officer.city_id.is_some()
                && !tasked_targets.contains(officer.id.as_str())
        })
        .cloned()
        .collect::<Vec<_>>();
    officers.sort_by(|a, b| {
        a.city_id
            .cmp(&b.city_id)
            .then_with(|| b.stats.charm.cmp(&a.stats.charm))
            .then_with(|| a.name.cmp(&b.name))
    });
    officers
}

fn ensure_selected_recruitment_target(
    ui_state: &mut GameUiState,
    candidates: &[Officer],
) -> Option<OfficerId> {
    if candidates.is_empty() {
        ui_state.selected_recruitment_target = None;
        return None;
    }
    let selected = ui_state
        .selected_recruitment_target
        .get_or_insert_with(|| candidates[0].id.clone());
    if !candidates.iter().any(|officer| officer.id == *selected) {
        *selected = candidates[0].id.clone();
    }
    Some(selected.clone())
}

fn wild_officer_label(game: &GameState, officer: &Officer, t: &Translator) -> String {
    t.text_args(
        "wild-officer-label",
        &args([
            ("officer", officer.name.clone()),
            ("city", officer_city_name(game, officer, t)),
            ("leadership", officer.stats.leadership.to_string()),
            ("politics", officer.stats.politics.to_string()),
            ("charm", officer.stats.charm.to_string()),
        ]),
    )
}

fn officer_city_name(game: &GameState, officer: &Officer, t: &Translator) -> String {
    officer
        .city_id
        .as_deref()
        .map(|city_id| city_name(game, city_id))
        .unwrap_or_else(|| t.text("officer-city-unassigned"))
}

fn recruitment_task_line(
    game: &GameState,
    task: &OfficerRecruitmentTask,
    t: &Translator,
) -> String {
    t.text_args(
        "recruitment-task-line",
        &args([
            ("recruiter", officer_name(game, &task.recruiter_officer_id)),
            ("target", officer_name(game, &task.target_officer_id)),
            ("progress", task.progress.to_string()),
            ("months", task.attempt_months.to_string()),
        ]),
    )
}

fn expedition_assignment_pool(assignments: &[ExpeditionAssignment]) -> TroopPool {
    let mut troops = TroopPool::default();
    for assignment in assignments {
        troops.add(assignment.troop_kind, assignment.troops);
    }
    troops
}

fn command_summary(game: &GameState, command: &Command, t: &Translator) -> String {
    let mut summary = t.text_args(
        "command-summary",
        &args([
            ("city", city_name(game, &command.city_id)),
            (
                "officer",
                command
                    .officer_id
                    .as_deref()
                    .map(|officer_id| officer_name(game, officer_id))
                    .unwrap_or_else(|| t.text("common-none-selected")),
            ),
            ("command", command_title(command, t)),
        ]),
    );
    match &command.kind {
        CommandKind::Transfer { target_city_id, .. }
        | CommandKind::Expedition { target_city_id, .. } => {
            summary = t.text_args(
                "command-summary-with-travel",
                &args([
                    ("summary", summary),
                    (
                        "travel",
                        travel_summary(game, &command.city_id, target_city_id, t),
                    ),
                ]),
            );
        }
        _ => {}
    }
    summary
}

fn command_title(command: &Command, t: &Translator) -> String {
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
        CommandKind::AppointGovernor { .. } => t.text("command-appoint-governor"),
        CommandKind::Transfer { troops, .. } => t.text_args(
            "command-title-transfer",
            &args([("troops", troops.total().to_string())]),
        ),
        CommandKind::Expedition { assignments, .. } => {
            let total: u32 = assignments.iter().map(|assignment| assignment.troops).sum();
            t.text_args(
                "command-title-expedition",
                &args([("troops", total.to_string())]),
            )
        }
        CommandKind::Diplomacy { proposal, .. } => t.text_args(
            "command-title-diplomacy",
            &args([("proposal", diplomacy_label(t, proposal))]),
        ),
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

fn officer_roster_list(
    ui: &mut egui::Ui,
    game: &GameState,
    city: &City,
    max_height: f32,
    t: &Translator,
) {
    let officers = game.officers_in_city(&city.id);
    if officers.is_empty() {
        ui.label(t.text("officer-none"));
        return;
    }
    egui::ScrollArea::vertical()
        .id_salt(("city_officer_roster", &city.id))
        .max_height(max_height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for officer in officers {
                officer_row(ui, officer, t);
            }
        });
}

fn facility_build_preview(
    city: &City,
    kind: FacilityKind,
) -> Result<(u8, ResourceCost, &'static str), &'static str> {
    if let Some(facility) = city.facility(kind) {
        if facility.level >= FACILITY_MAX_LEVEL {
            return Err("facility-error-max-level");
        }
        let target_level = facility.level + 1;
        if target_level > city.level {
            return Err("facility-error-level-exceeds-core");
        }
        return Ok((
            target_level,
            facility_upgrade_cost(kind, target_level),
            "facility-action-upgrade",
        ));
    }
    if city.facilities.len() >= city.facility_slots() {
        return Err("facility-error-slots-full");
    }
    Ok((1, facility_upgrade_cost(kind, 1), "facility-action-build"))
}

pub(super) fn officer_row(ui: &mut egui::Ui, officer: &Officer, t: &Translator) {
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
        ui.label(t.text_args(
            "officer-loyalty",
            &args([("loyalty", officer.loyalty.to_string())]),
        ));
        if let Some(profile) = &officer.profile {
            let courtesy = profile
                .courtesy_name
                .clone()
                .unwrap_or_else(|| t.text("none"));
            let native_place = profile
                .native_place
                .clone()
                .unwrap_or_else(|| t.text("unknown"));
            let birth = profile
                .birth_year
                .map(|year| year.to_string())
                .unwrap_or_else(|| t.text("unknown"));
            let death = profile
                .death_year
                .map(|year| year.to_string())
                .unwrap_or_else(|| t.text("unknown"));
            ui.label(t.text_args(
                "officer-profile-line",
                &args([
                    ("gender", officer_gender_label(t, &profile.gender)),
                    ("courtesy", courtesy),
                    ("native_place", native_place),
                ]),
            ));
            ui.label(t.text_args(
                "officer-life-confidence-line",
                &args([
                    ("birth", birth),
                    ("death", death),
                    ("confidence", confidence_label(t, &profile.confidence)),
                ]),
            ));
            if !profile.tags.is_empty() {
                ui.label(t.text_args("officer-tags", &args([("tags", profile.tags.join(", "))])));
            }
            if !profile.biography.is_empty() {
                ui.separator();
                ui.label(t.text("officer-biography"));
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
                ui.label(t.text("officer-relationships"));
                for relationship in &profile.relationships {
                    let notes = if relationship.notes.is_empty() {
                        String::new()
                    } else {
                        format!(" - {}", relationship.notes)
                    };
                    ui.label(format!(
                        "{}: {}{}",
                        officer_relationship_label(t, &relationship.kind),
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
        SqliteHistoricalCatalog::in_memory_from_seed()
            .unwrap()
            .build_game("ad200", "liu_bei")
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
