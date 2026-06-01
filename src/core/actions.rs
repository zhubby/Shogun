use crate::agent::{
    OpenAiResponsesAgentProvider, child_name::AgentChildNameGenerator, default_checkpoint_dir,
};
use crate::ai::{OpenAiApiType, OpenAiConfig};
use crate::game::*;
use std::time::Duration;

use super::i18n::{Translator, args};
use super::map::reset_map_view;
use super::state::{CityPanelTab, CommandAction, CommandCategory, GameUiState, Screen};

pub(super) fn start_history_game(ui_state: &mut GameUiState) {
    let t = Translator::new(ui_state.applied_settings.general.ui_language);
    if ui_state.selected_scenario_id.is_empty() {
        ui_state.message = t.text("message-no-history-scenario");
        return;
    }
    match SqliteHistoricalCatalog::open_default().and_then(|catalog| {
        catalog.build_game(
            &ui_state.selected_scenario_id,
            &ui_state.selected_faction_id,
        )
    }) {
        Ok(game) => enter_game(ui_state, game, t.text("message-new-game-started")),
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
    ui_state.events_open = false;
    ui_state.selected_event_id = None;
    ui_state.event_message.clear();
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
    let t = Translator::new(ui_state.applied_settings.general.ui_language);
    let child_name_generator = ai_child_name_generator(ui_state);
    let Some(game) = &mut ui_state.game else {
        ui_state.message = t.text("message-game-not-started");
        return;
    };
    if game.status != GameStatus::Running {
        return;
    }
    let provider = RuleBasedAiProvider;
    let report = match child_name_generator {
        Some(generator) => finish_turn_with_child_generator(game, &provider, &generator),
        None => finish_turn(game, &provider),
    };
    ui_state.message = t.text_args(
        "message-turn-finished",
        &args([("count", report.entries.len().to_string())]),
    );
    ui_state.selected_city_id = first_player_city(game);
    ui_state.city_drawer_open = false;
}

pub(super) fn clear_pending_commands(ui_state: &mut GameUiState) {
    let t = Translator::new(ui_state.applied_settings.general.ui_language);
    let Some(game) = &mut ui_state.game else {
        ui_state.message = t.text("message-game-not-started");
        return;
    };
    game.pending_commands.clear();
    ui_state.message = t.text("message-pending-commands-cleared");
}

pub(super) fn open_city(ui_state: &mut GameUiState, city_id: CityId) {
    ui_state.selected_city_id = Some(city_id);
    ui_state.city_drawer_open = true;
}

pub(super) fn finish_turn(game: &mut GameState, provider: &RuleBasedAiProvider) -> TurnReport {
    if let Ok(catalog) = SqliteHistoricalCatalog::open_default() {
        return finish_turn_with_ai_with_history(game, provider, &catalog);
    }
    finish_turn_with_ai(game, provider)
}

fn finish_turn_with_child_generator(
    game: &mut GameState,
    provider: &RuleBasedAiProvider,
    generator: &dyn OfficerGenerationProvider,
) -> TurnReport {
    if let Ok(catalog) = SqliteHistoricalCatalog::open_default() {
        return finish_turn_with_ai_with_history_and_generation(
            game, provider, &catalog, generator,
        );
    }
    finish_turn_with_ai_with_generation(game, provider, generator)
}

fn ai_child_name_generator(
    ui_state: &GameUiState,
) -> Option<AgentChildNameGenerator<OpenAiResponsesAgentProvider>> {
    let settings = &ui_state.applied_settings.ai.reasoning;
    if settings.api_type != OpenAiApiType::Responses
        || settings.token.trim().is_empty()
        || settings.model_name.trim().is_empty()
    {
        return None;
    }
    let config = OpenAiConfig::new(settings.token.clone())
        .with_api_type(settings.api_type)
        .with_api_url(settings.api_url.clone())
        .with_timeout(Duration::from_secs(5));
    let provider = OpenAiResponsesAgentProvider::new(config).ok()?;
    let checkpoint_dir = default_checkpoint_dir(ui_state.save_manager.base_dir());
    Some(
        AgentChildNameGenerator::new(
            provider,
            settings.model_name.clone(),
            Duration::from_secs(5),
        )
        .with_checkpoint_dir(checkpoint_dir),
    )
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
