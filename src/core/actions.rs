use crate::game::*;

use super::map::reset_map_view;
use super::state::{CityPanelTab, CommandAction, CommandCategory, GameUiState, Screen};

pub(super) fn start_history_game(ui_state: &mut GameUiState) {
    if ui_state.selected_scenario_id.is_empty() {
        ui_state.message = "没有可用的 SQLite 历史剧本".to_string();
        return;
    }
    match SqliteHistoricalCatalog::open_default().and_then(|catalog| {
        catalog.build_game(
            &ui_state.selected_scenario_id,
            &ui_state.selected_faction_id,
        )
    }) {
        Ok(game) => enter_game(ui_state, game, "新游戏开始".to_string()),
        Err(error) => ui_state.message = error.to_string(),
    }
}

pub(super) fn start_json_game(ui_state: &mut GameUiState) {
    match ui_state
        .json_scenario
        .build_game(&ui_state.selected_faction_id)
    {
        Ok(game) => enter_game(ui_state, game, "兼容小剧本开始".to_string()),
        Err(error) => ui_state.message = error.to_string(),
    }
}

pub(super) fn enter_game(ui_state: &mut GameUiState, game: GameState, message: String) {
    ui_state.selected_city_id = first_player_city(&game);
    ui_state.selected_officers.clear();
    ui_state.selected_transfer_target = None;
    ui_state.selected_expedition_target = None;
    ui_state.expedition_deputy_one = None;
    ui_state.expedition_deputy_two = None;
    ui_state.selected_diplomacy_target = None;
    ui_state.selected_city_tab = CityPanelTab::Overview;
    ui_state.selected_command_category = CommandCategory::Domestic;
    ui_state.selected_command_action = CommandAction::Develop;
    ui_state.city_drawer_open = false;
    ui_state.city_list_open = false;
    ui_state.technology_open = false;
    ui_state.reports_open = true;
    ui_state.save_panel_open = false;
    ui_state.main_menu_new_game_open = false;
    ui_state.main_menu_load_game_open = false;
    reset_map_view(ui_state);
    ui_state.game = Some(game);
    ui_state.screen = Screen::InGame;
    ui_state.message = message;
}

pub(super) fn finish_current_turn(ui_state: &mut GameUiState) {
    let Some(game) = &mut ui_state.game else {
        ui_state.message = "尚未开始游戏".to_string();
        return;
    };
    if game.status != GameStatus::Running {
        return;
    }
    let provider = RuleBasedAiProvider;
    let report = finish_turn(game, &provider);
    ui_state.message = format!("完成 {} 条结算记录", report.entries.len());
    ui_state.selected_city_id = first_player_city(game);
    ui_state.city_drawer_open = false;
}

pub(super) fn clear_pending_commands(ui_state: &mut GameUiState) {
    let Some(game) = &mut ui_state.game else {
        ui_state.message = "尚未开始游戏".to_string();
        return;
    };
    game.pending_commands.clear();
    ui_state.message = "已清空玩家待命令".to_string();
}

pub(super) fn open_city(ui_state: &mut GameUiState, city_id: CityId) {
    ui_state.selected_city_id = Some(city_id);
    ui_state.city_drawer_open = true;
}

pub(super) fn finish_turn(game: &mut GameState, provider: &RuleBasedAiProvider) -> TurnReport {
    if is_history_scenario(&game.scenario_id)
        && let Ok(catalog) = SqliteHistoricalCatalog::open_default()
    {
        return finish_turn_with_ai_with_history(game, provider, &catalog);
    }
    finish_turn_with_ai(game, provider)
}

pub(super) fn is_history_scenario(scenario_id: &str) -> bool {
    matches!(scenario_id, "ad190" | "ad200" | "ad208" | "ad220")
}

pub(super) fn refresh_saves(ui_state: &mut GameUiState) {
    ui_state.save_slots = ui_state.save_manager.list_slots().unwrap_or_default();
}

pub(super) fn first_player_city(game: &GameState) -> Option<CityId> {
    game.cities
        .values()
        .find(|city| city.faction_id == game.player_faction_id)
        .map(|city| city.id.clone())
}
