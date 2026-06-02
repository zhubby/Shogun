mod chrome;
mod city_list;
mod diplomacy;
mod events;
mod faction_overview;
mod map_overlays;
mod officer_browser;
mod officer_common;
mod officer_detail;
mod save_report;
mod shrine;
mod technology;
mod turn_summary;

pub(super) use officer_browser::{
    OFFICER_BROWSER_MODAL_WIDTH, OfficerBrowserTableOptions, officer_browser_filters,
    officer_browser_table, officer_tag_category_label, officer_tag_definitions_by_category,
    officer_tag_label,
};
pub(super) use officer_detail::{OfficerPortraitModalContext, officer_detail_modal_for_game};

use bevy_egui::egui;

use super::i18n::Translator;
use super::map::map_panel;
use super::runtime::CoreAsyncRuntime;
use super::state::GameUiState;

pub(super) fn in_game(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    async_runtime: &CoreAsyncRuntime,
) {
    ui_state.officer_portraits.poll_task_events();
    let t = Translator::new(ui_state.applied_settings.general.ui_language);
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            map_panel(ui, ui_state, &t);
        });

    in_game_hud(ctx, ui_state, &t, async_runtime);
}

pub(super) fn in_game_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    async_runtime: &CoreAsyncRuntime,
) {
    let screen = ctx.content_rect();
    chrome::top_status_hud(ctx, ui_state, t, screen);
    map_overlays::map_controls_hud(ctx, ui_state, t);
    map_overlays::left_city_summary_hud(ctx, ui_state, t);
    city_list::city_list_hud(ctx, ui_state, t, screen);
    save_report::save_hud(ctx, ui_state, t, screen);
    city_list::city_drawer_hud(ctx, ui_state, t, screen);
    save_report::report_hud(ctx, ui_state, t, screen);
    chrome::bottom_map_actions_hud(ctx, ui_state, t);
    diplomacy::diplomacy_hud(ctx, ui_state, t, screen);
    faction_overview::faction_overview_hud(ctx, ui_state, t, screen);
    officer_browser::officer_browser_hud(ctx, ui_state, t, screen);
    officer_browser::retainer_hud(ctx, ui_state, t, screen);
    shrine::shrine_hud(ctx, ui_state, t, screen);
    officer_detail::officer_detail_modal(ctx, ui_state, t, screen, async_runtime);
    technology::technology_hud(ctx, ui_state, t, screen);
    events::event_center_hud(ctx, ui_state, t, screen);
    events::event_popup_hud(ctx, ui_state, t, screen);
    turn_summary::turn_summary_hud(ctx, ui_state, t, screen);
    chrome::return_main_menu_confirm_hud(ctx, ui_state, t, screen);
}

#[cfg(test)]
pub(in crate::core::hud) mod test_support {
    use crate::core::i18n::{Translator, UiLanguage};
    use crate::core::settings::{GameSettings, GameSettingsStore, LoadedGameSettings};
    use crate::core::state::GameUiState;
    use crate::game::{
        GameState, HistoricalCatalog, OfficerGender, OfficerProfile, OfficerRelationship,
        OfficerRelationshipKind, OfficerStatus, SourceConfidence, SqliteHistoricalCatalog,
    };

    pub(in crate::core::hud) fn ui_state_with_game() -> GameUiState {
        let mut state = GameUiState::new(
            GameSettingsStore::with_default_path(),
            LoadedGameSettings {
                settings: GameSettings::default(),
                message: None,
            },
        );
        state.game = Some(
            SqliteHistoricalCatalog::in_memory_from_seed()
                .unwrap()
                .build_game("ad200", "liu_bei")
                .unwrap(),
        );
        state
    }

    pub(in crate::core::hud) fn zh() -> Translator {
        Translator::new(UiLanguage::SimplifiedChinese)
    }

    pub(in crate::core::hud) fn add_static_spouse_and_child(game: &mut GameState) {
        let stats = game.officers["liu_bei"].stats;
        let mut spouse = game.officers["liu_bei"].clone();
        spouse.id = "lady_static".to_string();
        spouse.name = "静夫人".to_string();
        spouse.birth_year = game.year - 30;
        spouse.gender = OfficerGender::Female;
        spouse.profile = Some(OfficerProfile {
            id: spouse.id.clone(),
            name: spouse.name.clone(),
            courtesy_name: None,
            native_place: None,
            birth_year: Some(spouse.birth_year),
            death_year: None,
            gender: OfficerGender::Female,
            stats,
            tags: Vec::new(),
            confidence: SourceConfidence::High,
            biography: String::new(),
            relationships: vec![OfficerRelationship {
                target_id: "liu_bei".to_string(),
                target_name: "刘备".to_string(),
                kind: OfficerRelationshipKind::Spouse,
                confidence: SourceConfidence::High,
                notes: "test".to_string(),
                source: "test".to_string(),
            }],
            notes: String::new(),
        });

        let mut child = game.officers["liu_bei"].clone();
        child.id = "liu_static_child".to_string();
        child.name = "刘承".to_string();
        child.birth_year = game.year - 10;
        child.gender = OfficerGender::Male;
        child.status = OfficerStatus::Minor;
        child.profile = Some(OfficerProfile {
            id: child.id.clone(),
            name: child.name.clone(),
            courtesy_name: None,
            native_place: None,
            birth_year: Some(child.birth_year),
            death_year: None,
            gender: OfficerGender::Male,
            stats,
            tags: Vec::new(),
            confidence: SourceConfidence::High,
            biography: String::new(),
            relationships: vec![OfficerRelationship {
                target_id: "liu_bei".to_string(),
                target_name: "刘备".to_string(),
                kind: OfficerRelationshipKind::ParentChild,
                confidence: SourceConfidence::High,
                notes: "test".to_string(),
                source: "test".to_string(),
            }],
            notes: String::new(),
        });

        let liu_profile = game
            .officers
            .get_mut("liu_bei")
            .unwrap()
            .profile
            .as_mut()
            .unwrap();
        liu_profile.relationships.push(OfficerRelationship {
            target_id: "lady_static".to_string(),
            target_name: "静夫人".to_string(),
            kind: OfficerRelationshipKind::Spouse,
            confidence: SourceConfidence::High,
            notes: "test".to_string(),
            source: "test".to_string(),
        });
        liu_profile.relationships.push(OfficerRelationship {
            target_id: "liu_static_child".to_string(),
            target_name: "刘承".to_string(),
            kind: OfficerRelationshipKind::ParentChild,
            confidence: SourceConfidence::High,
            notes: "test".to_string(),
            source: "test".to_string(),
        });

        game.officers.insert(spouse.id.clone(), spouse);
        game.officers.insert(child.id.clone(), child);
    }
}
