use bevy::{
    image::Image as BevyImage,
    prelude::{Assets, Handle, Res, ResMut, Resource},
    render::render_resource::TextureFormat,
};
use bevy_asset_loader::prelude::AssetCollection;
use bevy_egui::{EguiContexts, EguiTextureHandle, egui};

use crate::build_info::menu_build_label;
use crate::game::{
    Controller, Faction, GameState, HistoricalCatalog, HistoricalScenario, Officer, OfficerCatalog,
    OfficerGender, OfficerProfile, OfficerProfileUpdate, OfficerStats, OfficerStatus,
    SourceConfidence, SqliteHistoricalCatalog,
};

use super::HUD_MARGIN;
use super::actions::{enter_game, refresh_saves, start_history_game};
use super::hud::{
    OfficerBrowserTableOptions, OfficerPortraitModalContext, officer_browser_filters,
    officer_browser_table, officer_detail_modal_for_game, officer_tag_category_label,
    officer_tag_definitions_by_category, officer_tag_label,
};
use super::i18n::{Translator, args};
use super::labels::{confidence_label, officer_gender_label};
use super::officer_portrait_ui::{officer_portrait_status_line, paint_officer_portrait_preview};
use super::portraits::{
    OFFICER_PORTRAIT_ASPECT_HEIGHT, OFFICER_PORTRAIT_ASPECT_WIDTH, OfficerPortraitTaskState,
    officer_portrait_path,
};
use super::runtime::CoreAsyncRuntime;
use super::settings::refresh_audio_output_devices;
use super::state::{
    GameUiState, MenuBannerLogo, MenuCloudPattern, MenuIllustration, OfficerEditDraft,
    refresh_history_factions, refresh_history_menu,
};
use super::style::{
    modal_title_bar, war_border, war_gold, war_panel_frame, war_sub_panel_frame, war_text,
    war_text_muted, war_warning,
};

#[cfg(test)]
const BANNER_LOGO_ASSET_PATH: &str = "icons/banner_logo.png";
const BANNER_LOGO_ALPHA_THRESHOLD: u8 = 8;
const BANNER_LOGO_BACKGROUND_ALPHA_THRESHOLD: u8 = 42;
const BANNER_LOGO_CROP_PADDING: usize = 32;
const BANNER_LOGO_MAX_WIDTH: f32 = 860.0;
const BANNER_LOGO_MAX_HEIGHT: f32 = 340.0;
const MAIN_MENU_ILLUSTRATION_COUNT: usize = 6;
const MAIN_MENU_DEFAULT_ILLUSTRATION_INDEX: usize = 0;
#[cfg(test)]
const MAIN_MENU_ILLUSTRATION_ASSET_PATHS: [&str; MAIN_MENU_ILLUSTRATION_COUNT] = [
    "Illustrations/main_menu_0.png",
    "Illustrations/main_menu_1.png",
    "Illustrations/main_menu_2.png",
    "Illustrations/main_menu_3.png",
    "Illustrations/main_menu_4.png",
    "Illustrations/main_menu_5.png",
];
#[cfg(test)]
const MAIN_MENU_CLOUD_PATTERN_ASSET_PATH: &str = "Illustrations/main_menu_cloud_pattern.png";
const MAIN_MENU_CLOUD_PATTERN_ALPHA: u8 = 38;
const MAIN_MENU_CLOUD_PATTERN_UV_INSET: f32 = 0.08;
const MAIN_MENU_CONTROL_MIN_WIDTH: f32 = 420.0;
const MAIN_MENU_CONTROL_MAX_WIDTH: f32 = 620.0;
const MAIN_MENU_ART_MIN_WIDTH: f32 = 360.0;
const MAIN_MENU_BUTTON_WIDTH: f32 = 236.0;
const MAIN_MENU_BUTTON_HEIGHT: f32 = 36.0;
const MAIN_MENU_BUTTON_SPACING: f32 = 7.0;
const MAIN_MENU_BGM_BUTTON_SIZE: f32 = 36.0;
const MAIN_MENU_BGM_BUTTON_MARGIN: f32 = 12.0;
const NEW_GAME_CONTENT_HEIGHT: f32 = 442.0;
const NEW_GAME_COLUMN_GAP: f32 = 14.0;
const NEW_GAME_START_BUTTON_HEIGHT: f32 = 38.0;
const OFFICER_PORTRAIT_PANEL_WIDTH: f32 = 236.0;
const OFFICER_PORTRAIT_GAP: f32 = 14.0;

#[derive(AssetCollection, Resource)]
pub(super) struct MainMenuAssets {
    #[asset(path = "icons/banner_logo.png")]
    banner_logo: Handle<BevyImage>,
    #[asset(path = "Illustrations/main_menu_0.png")]
    illustration_0: Handle<BevyImage>,
    #[asset(path = "Illustrations/main_menu_1.png")]
    illustration_1: Handle<BevyImage>,
    #[asset(path = "Illustrations/main_menu_2.png")]
    illustration_2: Handle<BevyImage>,
    #[asset(path = "Illustrations/main_menu_3.png")]
    illustration_3: Handle<BevyImage>,
    #[asset(path = "Illustrations/main_menu_4.png")]
    illustration_4: Handle<BevyImage>,
    #[asset(path = "Illustrations/main_menu_5.png")]
    illustration_5: Handle<BevyImage>,
    #[asset(path = "Illustrations/main_menu_cloud_pattern.png")]
    cloud_pattern: Handle<BevyImage>,
}

impl MainMenuAssets {
    fn illustration_handle(&self, index: usize) -> Option<&Handle<BevyImage>> {
        match index {
            0 => Some(&self.illustration_0),
            1 => Some(&self.illustration_1),
            2 => Some(&self.illustration_2),
            3 => Some(&self.illustration_3),
            4 => Some(&self.illustration_4),
            5 => Some(&self.illustration_5),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MainMenuAction {
    None,
    Exit,
}

pub(super) fn main_menu(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    async_runtime: &CoreAsyncRuntime,
) -> MainMenuAction {
    ui_state.officer_portraits.poll_task_events();
    let t = Translator::new(ui_state.applied_settings.general.ui_language);
    let mut action = MainMenuAction::None;
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            action = main_menu_columns(ui, ui_state, &t);
        });

    if ui_state.main_menu_new_game_open {
        new_game_modal(ctx, ui_state, &t);
    }
    if ui_state.main_menu_load_game_open {
        load_game_modal(ctx, ui_state, &t);
    }
    if ui_state.officer_settings_open {
        officer_settings_modal(ctx, ui_state, &t);
    }
    if ui_state.officer_edit_open {
        officer_profile_edit_modal(ctx, ui_state, &t, async_runtime);
    }
    let screen = ctx.content_rect();
    if ui_state.officer_settings_open {
        let officer_detail_id = ui_state.officer_detail_id.clone();
        let api_key = ui_state.applied_settings.ai.multimodal.api_key.clone();
        let model_name = ui_state.applied_settings.ai.multimodal.model_name.clone();
        let close_detail = match ui_state.officer_settings_game.as_ref() {
            Some(game) => officer_detail_modal_for_game(
                ctx,
                officer_detail_id.as_deref(),
                OfficerPortraitModalContext {
                    store: &mut ui_state.officer_portraits,
                    api_key: &api_key,
                    model_name: &model_name,
                    async_runtime,
                },
                &t,
                screen,
                game,
            ),
            None => ui_state.officer_detail_id.is_some(),
        };
        if close_detail {
            ui_state.officer_detail_id = None;
        }
    } else {
        ui_state.officer_detail_id = None;
    }
    action
}

pub(super) fn prepare_main_menu_assets_for_egui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<GameUiState>,
    menu_assets: Option<Res<MainMenuAssets>>,
    images: Res<Assets<BevyImage>>,
) {
    let Some(menu_assets) = menu_assets else {
        return;
    };

    if ui_state.banner_logo.is_none()
        && ui_state.banner_logo_error.is_none()
        && let Some(image) = images.get(&menu_assets.banner_logo)
    {
        match contexts.ctx_mut() {
            Ok(ctx) => match register_banner_logo(ctx, image) {
                Ok(logo) => ui_state.banner_logo = Some(logo),
                Err(error) => ui_state.banner_logo_error = Some(error),
            },
            Err(error) => {
                ui_state.banner_logo_error = Some(format!("egui context unavailable: {error}"));
            }
        }
    }

    ensure_main_menu_illustration_slots(&mut ui_state);
    for index in 0..MAIN_MENU_ILLUSTRATION_COUNT {
        if ui_state.main_menu_illustrations[index].is_some()
            || ui_state.main_menu_illustration_errors[index].is_some()
        {
            continue;
        }
        let Some(handle) = menu_assets.illustration_handle(index) else {
            ui_state.main_menu_illustration_errors[index] = Some(format!(
                "main menu illustration index {index} is out of range"
            ));
            continue;
        };
        let Some(image) = images.get(handle) else {
            continue;
        };
        match register_main_menu_illustration(&mut contexts, handle, image, index) {
            Ok(illustration) => ui_state.main_menu_illustrations[index] = Some(illustration),
            Err(error) => ui_state.main_menu_illustration_errors[index] = Some(error),
        }
    }

    if ui_state.main_menu_cloud_pattern.is_none()
        && ui_state.main_menu_cloud_pattern_error.is_none()
        && let Some(image) = images.get(&menu_assets.cloud_pattern)
    {
        match register_main_menu_cloud_pattern(&mut contexts, &menu_assets.cloud_pattern, image) {
            Ok(pattern) => ui_state.main_menu_cloud_pattern = Some(pattern),
            Err(error) => ui_state.main_menu_cloud_pattern_error = Some(error),
        }
    }
}

fn main_menu_columns(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
) -> MainMenuAction {
    let content_rect = ui.max_rect().shrink(HUD_MARGIN);
    if content_rect.width() <= 0.0 || content_rect.height() <= 0.0 {
        return MainMenuAction::None;
    }

    let left_width = main_menu_control_width(content_rect.width());
    let left_height = (content_rect.height() * 0.78).clamp(520.0, 680.0);
    let left_offset = (content_rect.width() * 0.105).clamp(118.0, 245.0);
    let left_rect = egui::Rect::from_min_size(
        egui::pos2(
            content_rect.left() + left_offset,
            content_rect.center().y - left_height.min(content_rect.height()).min(680.0) * 0.5,
        ),
        egui::vec2(left_width, left_height.min(content_rect.height())),
    );
    let art_gap = (content_rect.width() * 0.012).clamp(14.0, 28.0);
    let art_rect = egui::Rect::from_min_max(
        egui::pos2(left_rect.right() - art_gap, content_rect.top() + 10.0),
        egui::pos2(content_rect.right() - 10.0, content_rect.bottom() - 10.0),
    );

    draw_main_menu_illustration(ui, ui_state, content_rect, art_rect);
    let action = draw_main_menu_left_panel(ui, ui_state, t, left_rect);
    draw_main_menu_bgm_button(ui, ui_state, t, content_rect);
    action
}

fn main_menu_control_width(total_width: f32) -> f32 {
    let preferred =
        (total_width * 0.36).clamp(MAIN_MENU_CONTROL_MIN_WIDTH, MAIN_MENU_CONTROL_MAX_WIDTH);
    let max_left = total_width - MAIN_MENU_ART_MIN_WIDTH;
    if max_left >= MAIN_MENU_CONTROL_MIN_WIDTH {
        preferred.min(max_left)
    } else {
        (total_width * 0.40).clamp(280.0, MAIN_MENU_CONTROL_MIN_WIDTH)
    }
}

fn draw_main_menu_left_panel(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    rect: egui::Rect,
) -> MainMenuAction {
    let inner = rect.shrink2(egui::vec2(6.0, 16.0));
    let mut action = MainMenuAction::None;
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::top_down(egui::Align::Center)),
        |ui| {
            ui.set_width(inner.width());
            ui.add_space(4.0);
            banner_logo(ui, ui_state, t);
            ui.add(
                egui::Label::new(
                    egui::RichText::new(menu_build_label())
                        .size(12.0)
                        .color(war_text_muted()),
                )
                .truncate(),
            );
            ui.add_space((inner.height() * 0.04).clamp(12.0, 24.0));

            action = draw_main_menu_buttons(ui, ui_state, t);

            if !ui_state.message.is_empty() {
                let reserved_height = 50.0;
                ui.add_space((ui.available_height() - reserved_height).max(10.0));
                war_sub_panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let summary = ui_state.message.replace('\n', " / ");
                    let response = ui.add(
                        egui::Label::new(egui::RichText::new(summary).size(12.0).color(war_gold()))
                            .truncate(),
                    );
                    response.on_hover_text(&ui_state.message);
                });
            }
        },
    );
    action
}

fn draw_main_menu_buttons(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
) -> MainMenuAction {
    let button_width = ui.available_width().min(MAIN_MENU_BUTTON_WIDTH);
    let mut action = MainMenuAction::None;
    let mut hovered_illustration_index = None;

    let new_game_response = main_menu_button(ui, button_width, &t.text("main-menu-new-game"));
    if new_game_response.hovered() {
        hovered_illustration_index = Some(1);
    }
    if new_game_response.clicked() {
        close_main_menu_popups(ui_state);
        ui_state.main_menu_new_game_open = true;
    }
    ui.add_space(MAIN_MENU_BUTTON_SPACING);
    let load_game_response = main_menu_button(ui, button_width, &t.text("main-menu-load-game"));
    if load_game_response.hovered() {
        hovered_illustration_index = Some(2);
    }
    if load_game_response.clicked() {
        close_main_menu_popups(ui_state);
        refresh_saves(ui_state);
        ui_state.main_menu_load_game_open = true;
    }
    ui.add_space(MAIN_MENU_BUTTON_SPACING);
    let officer_response = main_menu_button(ui, button_width, &t.text("main-menu-officers"));
    if officer_response.hovered() {
        hovered_illustration_index = Some(3);
    }
    if officer_response.clicked() {
        close_main_menu_popups(ui_state);
        open_officer_settings(ui_state);
    }
    ui.add_space(MAIN_MENU_BUTTON_SPACING);
    let settings_response = main_menu_button(ui, button_width, &t.text("main-menu-settings"));
    if settings_response.hovered() {
        hovered_illustration_index = Some(4);
    }
    if settings_response.clicked() {
        close_main_menu_popups(ui_state);
        refresh_audio_output_devices(ui_state);
        ui_state.settings_open = true;
    }
    ui.add_space(MAIN_MENU_BUTTON_SPACING);
    let exit_response = main_menu_button(ui, button_width, &t.text("main-menu-exit"));
    if exit_response.hovered() {
        hovered_illustration_index = Some(5);
    }
    if exit_response.clicked() {
        action = MainMenuAction::Exit;
    }

    set_main_menu_hovered_illustration(ui, ui_state, hovered_illustration_index);

    action
}

fn set_main_menu_hovered_illustration(
    ui: &egui::Ui,
    ui_state: &mut GameUiState,
    hovered_illustration_index: Option<usize>,
) {
    if ui_state.main_menu_hovered_illustration_index != hovered_illustration_index {
        ui_state.main_menu_hovered_illustration_index = hovered_illustration_index;
        ui.ctx().request_repaint();
    }
}

fn main_menu_button(ui: &mut egui::Ui, width: f32, label: &str) -> egui::Response {
    ui.add_sized(
        [width, MAIN_MENU_BUTTON_HEIGHT],
        egui::Button::new(
            egui::RichText::new(label)
                .size(15.5)
                .color(war_gold())
                .strong(),
        )
        .truncate(),
    )
}

fn draw_main_menu_bgm_button(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    content_rect: egui::Rect,
) {
    let button_rect = egui::Rect::from_min_size(
        egui::pos2(
            content_rect.right() - MAIN_MENU_BGM_BUTTON_MARGIN - MAIN_MENU_BGM_BUTTON_SIZE,
            content_rect.top() + MAIN_MENU_BGM_BUTTON_MARGIN,
        ),
        egui::vec2(MAIN_MENU_BGM_BUTTON_SIZE, MAIN_MENU_BGM_BUTTON_SIZE),
    );
    let (icon, tooltip) = if ui_state.main_menu_bgm_enabled {
        (
            egui_phosphor::regular::SPEAKER_HIGH,
            t.text("main-menu-bgm-disable"),
        )
    } else {
        (
            egui_phosphor::regular::SPEAKER_X,
            t.text("main-menu-bgm-enable"),
        )
    };

    let response = ui
        .put(
            button_rect,
            egui::Button::new(egui::RichText::new(icon).size(20.0).color(war_gold())),
        )
        .on_hover_text(tooltip);
    if response.clicked() {
        ui_state.main_menu_bgm_enabled = !ui_state.main_menu_bgm_enabled;
        ui.ctx().request_repaint();
    }
}

fn close_main_menu_popups(ui_state: &mut GameUiState) {
    ui_state.main_menu_new_game_open = false;
    ui_state.main_menu_load_game_open = false;
    ui_state.settings_open = false;
    ui_state.officer_settings_open = false;
}

pub(super) fn new_game_menu(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.set_width(ui.available_width());
    new_game_header(ui, ui_state, t);
    ui.add_space(12.0);

    if ui_state.history_scenarios.is_empty() {
        new_game_empty_catalog(ui, ui_state, t);
        return;
    }

    ui.columns(2, |columns| {
        columns[0].set_width((columns[0].available_width() - NEW_GAME_COLUMN_GAP * 0.5).max(0.0));
        columns[1].set_width((columns[1].available_width() - NEW_GAME_COLUMN_GAP * 0.5).max(0.0));
        new_game_scenario_column(&mut columns[0], ui_state, t);
        new_game_faction_column(&mut columns[1], ui_state, t);
    });

    ui.add_space(12.0);
    new_game_footer(ui, ui_state, t);
}

fn new_game_header(ui: &mut egui::Ui, ui_state: &GameUiState, t: &Translator) {
    let scenario = selected_scenario(ui_state);
    let faction = selected_faction(ui_state);
    war_sub_panel_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(t.text("new-game-title"))
                        .size(20.0)
                        .color(war_gold())
                        .strong(),
                );
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new(new_game_selection_summary(scenario, faction, t))
                        .size(13.0)
                        .color(war_text_muted()),
                );
            });
        });
    });
}

fn new_game_selection_summary(
    scenario: Option<&HistoricalScenario>,
    faction: Option<&Faction>,
    t: &Translator,
) -> String {
    let scenario = scenario
        .map(|scenario| {
            t.text_args(
                "new-game-scenario-line",
                &args([
                    ("name", scenario.name.clone()),
                    ("year", scenario.year.to_string()),
                    ("month", scenario.month.to_string()),
                ]),
            )
        })
        .unwrap_or_else(|| t.text("common-none-selected"));
    let faction = faction
        .map(|faction| faction.name.clone())
        .unwrap_or_else(|| t.text("common-none-selected"));
    format!("{scenario} / {faction}")
}

fn new_game_empty_catalog(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    let height = NEW_GAME_CONTENT_HEIGHT + NEW_GAME_START_BUTTON_HEIGHT + 12.0;
    war_sub_panel_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_min_height(height);
        ui.vertical_centered_justified(|ui| {
            ui.add_space((height * 0.38).max(0.0));
            ui.label(
                egui::RichText::new(t.text("new-game-history-required"))
                    .color(war_warning())
                    .size(15.0),
            );
            ui.add_space(10.0);
            if ui.button(t.text("new-game-refresh-catalog")).clicked() {
                refresh_history_menu(ui_state);
            }
        });
    });
}

fn new_game_scenario_column(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    war_sub_panel_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_min_height(NEW_GAME_CONTENT_HEIGHT);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(t.text("new-game-scenario"))
                    .color(war_gold())
                    .strong(),
            );
            ui.add_space((ui.available_width() - 122.0).max(0.0));
            if ui
                .small_button(t.text("new-game-refresh-catalog"))
                .on_hover_text(t.text("new-game-refresh-catalog"))
                .clicked()
            {
                refresh_history_menu(ui_state);
            }
        });
        ui.add_space(8.0);

        let mut scenario_changed = false;
        egui::ScrollArea::vertical()
            .id_salt("main_menu_scenarios")
            .max_height(NEW_GAME_CONTENT_HEIGHT - 42.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for scenario in ui_state.history_scenarios.clone() {
                    if scenario_choice_card(ui, ui_state, &scenario, t) {
                        scenario_changed = true;
                    }
                    ui.add_space(7.0);
                }
            });
        if scenario_changed {
            refresh_history_factions(ui_state);
        }
    });
}

fn new_game_faction_column(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    war_sub_panel_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_min_height(NEW_GAME_CONTENT_HEIGHT);
        ui.label(
            egui::RichText::new(t.text("new-game-faction"))
                .color(war_gold())
                .strong(),
        );
        ui.add_space(8.0);

        let faction = selected_faction(ui_state);
        selected_faction_summary(ui, faction, t);
        ui.add_space(10.0);

        let selectable_factions = ui_state
            .history_factions
            .iter()
            .filter(|faction| faction.selectable)
            .cloned()
            .collect::<Vec<_>>();
        if selectable_factions.is_empty() {
            ui.label(
                egui::RichText::new(t.text("common-none-selected"))
                    .color(war_warning())
                    .size(15.0),
            );
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("main_menu_factions")
            .max_height(NEW_GAME_CONTENT_HEIGHT - 118.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for faction in selectable_factions {
                    faction_choice_card(ui, ui_state, faction, t);
                    ui.add_space(7.0);
                }
            });
    });
}

fn selected_faction_summary(ui: &mut egui::Ui, faction: Option<&Faction>, t: &Translator) {
    let rect_height = 54.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), rect_height),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        4.0,
        egui::Color32::from_rgba_unmultiplied(22, 18, 13, 210),
    );
    painter.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(138, 101, 58, 92)),
        egui::StrokeKind::Inside,
    );

    let marker_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 12.0, rect.center().y - 7.0),
        egui::vec2(14.0, 14.0),
    );
    let marker_color = faction
        .map(faction_color)
        .unwrap_or_else(|| egui::Color32::from_rgba_unmultiplied(118, 85, 48, 150));
    painter.circle_filled(marker_rect.center(), 7.0, marker_color);
    painter.circle_stroke(
        marker_rect.center(),
        7.0,
        egui::Stroke::new(1.0, war_gold()),
    );

    painter.text(
        egui::pos2(rect.left() + 34.0, rect.top() + 10.0),
        egui::Align2::LEFT_TOP,
        faction.map(|faction| faction.name.as_str()).unwrap_or(""),
        egui::FontId::proportional(17.0),
        war_text(),
    );
    if faction.is_none() {
        painter.text(
            egui::pos2(rect.left() + 34.0, rect.top() + 10.0),
            egui::Align2::LEFT_TOP,
            t.text("common-none-selected"),
            egui::FontId::proportional(17.0),
            war_text_muted(),
        );
    }
    painter.text(
        egui::pos2(rect.left() + 34.0, rect.top() + 33.0),
        egui::Align2::LEFT_TOP,
        t.text("new-game-faction"),
        egui::FontId::proportional(12.0),
        war_text_muted(),
    );
}

fn new_game_footer(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    let can_start =
        !ui_state.selected_scenario_id.is_empty() && !ui_state.selected_faction_id.is_empty();
    let button = egui::Button::new(
        egui::RichText::new(t.text("new-game-start"))
            .size(16.0)
            .strong(),
    )
    .fill(if can_start {
        egui::Color32::from_rgb(77, 45, 28)
    } else {
        egui::Color32::from_rgb(45, 36, 27)
    })
    .stroke(egui::Stroke::new(1.0, war_border()));
    if ui
        .add_enabled_ui(can_start, |ui| {
            ui.add_sized([ui.available_width(), NEW_GAME_START_BUTTON_HEIGHT], button)
        })
        .inner
        .clicked()
    {
        start_history_game(ui_state);
    }
}

fn scenario_choice_card(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    scenario: &HistoricalScenario,
    t: &Translator,
) -> bool {
    let selected = ui_state.selected_scenario_id == scenario.id;
    let response = selectable_card(
        ui,
        selected,
        scenario.name.as_str(),
        &t.text_args(
            "new-game-scenario-date",
            &args([
                ("year", scenario.year.to_string()),
                ("month", scenario.month.to_string()),
            ]),
        ),
        None,
        58.0,
    )
    .on_hover_text(t.text_args(
        "new-game-scenario-line",
        &args([
            ("name", scenario.name.clone()),
            ("year", scenario.year.to_string()),
            ("month", scenario.month.to_string()),
        ]),
    ));
    if response.clicked() && !selected {
        ui_state.selected_scenario_id = scenario.id.clone();
        return true;
    }
    false
}

fn faction_choice_card(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    faction: Faction,
    t: &Translator,
) -> egui::Response {
    let selected = ui_state.selected_faction_id == faction.id;
    let response = selectable_card(
        ui,
        selected,
        faction.name.as_str(),
        &t.text("new-game-selectable-faction"),
        Some(faction_color(&faction)),
        46.0,
    );
    if response.clicked() && !selected {
        ui_state.selected_faction_id = faction.id;
    }
    response
}

fn selectable_card(
    ui: &mut egui::Ui,
    selected: bool,
    title: &str,
    subtitle: &str,
    marker_color: Option<egui::Color32>,
    height: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click(),
    );
    let hovered = response.hovered();
    let fill = if selected {
        egui::Color32::from_rgba_unmultiplied(80, 42, 28, 230)
    } else if hovered {
        egui::Color32::from_rgba_unmultiplied(52, 39, 26, 230)
    } else {
        egui::Color32::from_rgba_unmultiplied(24, 20, 15, 205)
    };
    let stroke_color = if selected || hovered {
        war_gold()
    } else {
        egui::Color32::from_rgba_unmultiplied(138, 101, 58, 95)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, fill);
    painter.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );

    let marker_center = egui::pos2(rect.left() + 18.0, rect.center().y);
    painter.circle_stroke(marker_center, 8.0, egui::Stroke::new(1.0, stroke_color));
    if selected {
        painter.circle_filled(marker_center, 4.5, war_gold());
    } else if let Some(color) = marker_color {
        painter.circle_filled(marker_center, 5.0, color);
    }

    painter.text(
        egui::pos2(rect.left() + 34.0, rect.top() + 9.0),
        egui::Align2::LEFT_TOP,
        title,
        egui::FontId::proportional(15.5),
        if selected {
            war_text()
        } else {
            war_text_muted()
        },
    );
    painter.text(
        egui::pos2(rect.left() + 34.0, rect.top() + height - 23.0),
        egui::Align2::LEFT_TOP,
        subtitle,
        egui::FontId::proportional(12.0),
        war_text_muted(),
    );

    response
}

fn selected_scenario(ui_state: &GameUiState) -> Option<&HistoricalScenario> {
    ui_state
        .history_scenarios
        .iter()
        .find(|scenario| scenario.id == ui_state.selected_scenario_id)
}

fn selected_faction(ui_state: &GameUiState) -> Option<&Faction> {
    ui_state
        .history_factions
        .iter()
        .find(|faction| faction.selectable && faction.id == ui_state.selected_faction_id)
}

fn faction_color(faction: &Faction) -> egui::Color32 {
    let [red, green, blue] = faction.color;
    egui::Color32::from_rgb(
        (red.clamp(0.0, 1.0) * 255.0).round() as u8,
        (green.clamp(0.0, 1.0) * 255.0).round() as u8,
        (blue.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

pub(super) fn load_game_menu(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(ui.available_width());
        ui.heading(egui::RichText::new(t.text("load-game-title")).color(war_gold()));
        ui.label(
            egui::RichText::new(t.text_args(
                "load-game-directory",
                &args([(
                    "path",
                    ui_state.save_manager.base_dir().display().to_string(),
                )]),
            ))
            .color(war_text_muted()),
        );
        if ui.button(t.text("load-game-refresh")).clicked() {
            refresh_saves(ui_state);
        }
        ui.add_space(8.0);
        let slots = ui_state.save_slots.clone();
        if slots.is_empty() {
            ui.label(t.text("load-game-empty"));
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
                        ui.label(t.text_args(
                            "load-game-slot-line",
                            &args([
                                ("name", slot.display_name.clone()),
                                ("year", slot.year.to_string()),
                                ("month", slot.month.to_string()),
                                ("turn", slot.turn.to_string()),
                            ]),
                        ));
                        ui.horizontal(|ui| {
                            if ui.button(t.text("load-game-load")).clicked() {
                                match ui_state.save_manager.load_slot(&slot.slot_id) {
                                    Ok(game) => enter_game(
                                        ui_state,
                                        game,
                                        t.text_args(
                                            "message-save-loaded",
                                            &args([("name", slot.display_name.clone())]),
                                        ),
                                    ),
                                    Err(error) => {
                                        let _ = ui_state.save_manager.delete_slot(&slot.slot_id);
                                        refresh_saves(ui_state);
                                        ui_state.message = t.text_args(
                                            "message-save-invalid-discarded",
                                            &args([("error", error.to_string())]),
                                        );
                                    }
                                }
                            }
                            if ui.button(t.text("load-game-delete")).clicked() {
                                match ui_state.save_manager.delete_slot(&slot.slot_id) {
                                    Ok(()) => {
                                        refresh_saves(ui_state);
                                        ui_state.message = t.text_args(
                                            "message-save-deleted",
                                            &args([("name", slot.display_name.clone())]),
                                        );
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

fn new_game_modal(ctx: &egui::Context, ui_state: &mut GameUiState, t: &Translator) {
    main_menu_scrim(ctx, ui_state);

    let screen = ctx.content_rect();
    let width = (screen.width() * 0.66).clamp(760.0, 920.0);
    let height = (screen.height() - HUD_MARGIN * 2.0).clamp(560.0, 680.0);
    egui::Area::new(egui::Id::new("main_menu_new_game_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("main-menu-new-game")) {
                    ui_state.main_menu_new_game_open = false;
                }
                ui.add_space(8.0);
                new_game_menu(ui, ui_state, t);
            });
        });
}

fn load_game_modal(ctx: &egui::Context, ui_state: &mut GameUiState, t: &Translator) {
    main_menu_scrim(ctx, ui_state);

    let screen = ctx.content_rect();
    let width = (screen.width() * 0.52).clamp(480.0, 720.0);
    let height = (screen.height() - HUD_MARGIN * 2.0).clamp(500.0, 700.0);
    egui::Area::new(egui::Id::new("main_menu_load_game_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("main-menu-load-game")) {
                    ui_state.main_menu_load_game_open = false;
                }
                ui.separator();
                load_game_menu(ui, ui_state, t);
            });
        });
}

fn main_menu_scrim(ctx: &egui::Context, ui_state: &mut GameUiState) {
    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new("main_menu_choice_modal_scrim"))
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
                ui_state.main_menu_new_game_open = false;
                ui_state.main_menu_load_game_open = false;
            }
        });
}

fn draw_main_menu_illustration(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    background_rect: egui::Rect,
    art_rect: egui::Rect,
) {
    paint_illustration_stage(ui.painter(), background_rect, art_rect);
    draw_cloud_pattern(ui, ui_state, background_rect.shrink(28.0));

    ensure_main_menu_illustration_slots(ui_state);
    let illustration_index = active_main_menu_illustration_index(ui_state);

    if let Some(illustration) = &ui_state.main_menu_illustrations[illustration_index] {
        let fit_size = fit_contained_size(illustration.crop_size, art_rect.size());
        let image_rect = egui::Align2::CENTER_CENTER.align_size_within_rect(fit_size, art_rect);
        paint_illustration_underlay(ui.painter(), background_rect, image_rect);
        egui::Image::from_texture((illustration.texture_id, fit_size))
            .fit_to_exact_size(fit_size)
            .uv(illustration.crop_uv)
            .tint(egui::Color32::WHITE)
            .paint_at(ui, image_rect);
    } else if let Some(error) = &ui_state.main_menu_illustration_errors[illustration_index] {
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(art_rect.shrink(24.0))
                .layout(egui::Layout::centered_and_justified(
                    egui::Direction::TopDown,
                )),
            |ui| {
                ui.label(egui::RichText::new(error).color(war_text_muted()));
            },
        );
    }
}

fn ensure_main_menu_illustration_slots(ui_state: &mut GameUiState) {
    ui_state
        .main_menu_illustrations
        .resize_with(MAIN_MENU_ILLUSTRATION_COUNT, || None);
    ui_state
        .main_menu_illustration_errors
        .resize_with(MAIN_MENU_ILLUSTRATION_COUNT, || None);
}

fn active_main_menu_illustration_index(ui_state: &GameUiState) -> usize {
    ui_state
        .main_menu_hovered_illustration_index
        .filter(|index| *index < MAIN_MENU_ILLUSTRATION_COUNT)
        .unwrap_or(MAIN_MENU_DEFAULT_ILLUSTRATION_INDEX)
}

fn draw_cloud_pattern(ui: &mut egui::Ui, ui_state: &mut GameUiState, rect: egui::Rect) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let Some(pattern) = &ui_state.main_menu_cloud_pattern else {
        return;
    };

    if pattern.size.x <= 0.0 || pattern.size.y <= 0.0 {
        return;
    }

    let painter = ui.painter().with_clip_rect(rect);
    let tint = egui::Color32::from_rgba_unmultiplied(255, 255, 255, MAIN_MENU_CLOUD_PATTERN_ALPHA);
    let uv = egui::Rect::from_min_max(
        egui::pos2(
            MAIN_MENU_CLOUD_PATTERN_UV_INSET,
            MAIN_MENU_CLOUD_PATTERN_UV_INSET,
        ),
        egui::pos2(
            1.0 - MAIN_MENU_CLOUD_PATTERN_UV_INSET,
            1.0 - MAIN_MENU_CLOUD_PATTERN_UV_INSET,
        ),
    );
    let crop_size = pattern.size * (1.0 - MAIN_MENU_CLOUD_PATTERN_UV_INSET * 2.0);
    let decal_width = (rect.width() * 0.45).clamp(420.0, 600.0);
    let decal_size = egui::vec2(
        decal_width,
        decal_width * crop_size.y / crop_size.x.max(1.0),
    );

    for center_factor in [
        egui::vec2(0.13, 0.16),
        egui::vec2(0.57, 0.12),
        egui::vec2(0.91, 0.30),
        egui::vec2(0.20, 0.58),
        egui::vec2(0.66, 0.55),
        egui::vec2(0.08, 0.88),
        egui::vec2(0.55, 0.91),
        egui::vec2(0.94, 0.82),
    ] {
        let center = egui::pos2(
            rect.left() + rect.width() * center_factor.x,
            rect.top() + rect.height() * center_factor.y,
        );
        let decal_rect = egui::Align2::CENTER_CENTER
            .align_size_within_rect(decal_size, egui::Rect::from_center_size(center, decal_size));
        painter.image(pattern.texture_id, decal_rect, uv, tint);
    }
}

fn paint_illustration_stage(painter: &egui::Painter, rect: egui::Rect, art_rect: egui::Rect) {
    painter.rect_filled(rect, 12.0, egui::Color32::from_rgb(244, 239, 224));
    paint_center_gradient(
        painter,
        art_rect.expand2(egui::vec2(90.0, 32.0)).intersect(rect),
        egui::Color32::from_rgba_unmultiplied(255, 252, 240, 92),
        egui::Color32::TRANSPARENT,
    );
    painter.rect_stroke(
        rect,
        12.0,
        egui::Stroke::new(
            1.5,
            egui::Color32::from_rgba_unmultiplied(215, 162, 72, 210),
        ),
        egui::StrokeKind::Inside,
    );
}

fn paint_illustration_underlay(
    painter: &egui::Painter,
    stage_rect: egui::Rect,
    image_rect: egui::Rect,
) {
    let wash_rect = image_rect
        .expand2(egui::vec2(72.0, 44.0))
        .intersect(stage_rect);
    paint_center_gradient(
        painter,
        wash_rect,
        egui::Color32::from_rgba_unmultiplied(250, 247, 232, 82),
        egui::Color32::TRANSPARENT,
    );

    let ink_rect = image_rect
        .expand2(egui::vec2(26.0, 18.0))
        .intersect(stage_rect);
    paint_center_gradient(
        painter,
        ink_rect,
        egui::Color32::from_rgba_unmultiplied(96, 84, 64, 28),
        egui::Color32::TRANSPARENT,
    );
}

fn paint_center_gradient(
    painter: &egui::Painter,
    rect: egui::Rect,
    center: egui::Color32,
    edge: egui::Color32,
) {
    let mut mesh = egui::epaint::Mesh::default();
    mesh.colored_vertex(rect.left_top(), edge);
    mesh.colored_vertex(rect.right_top(), edge);
    mesh.colored_vertex(rect.right_bottom(), edge);
    mesh.colored_vertex(rect.left_bottom(), edge);
    mesh.colored_vertex(rect.center(), center);
    mesh.add_triangle(0, 1, 4);
    mesh.add_triangle(1, 2, 4);
    mesh.add_triangle(2, 3, 4);
    mesh.add_triangle(3, 0, 4);
    painter.add(mesh);
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
    let t = Translator::new(ui_state.applied_settings.general.ui_language);
    let catalog = match SqliteHistoricalCatalog::open_default() {
        Ok(catalog) => catalog,
        Err(error) => {
            ui_state.message = t.text_args(
                "message-history-catalog-unavailable",
                &args([("error", error.to_string())]),
            );
            return None;
        }
    };
    let scenario_id = if ui_state.selected_scenario_id.is_empty() {
        catalog
            .scenarios()
            .ok()
            .and_then(|scenarios| scenarios.first().map(|scenario| scenario.id.clone()))
    } else {
        Some(ui_state.selected_scenario_id.clone())
    }?;
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
                Err(error) => {
                    ui_state.message = t.text_args(
                        "message-officer-catalog-load-failed",
                        &args([("error", error.to_string())]),
                    );
                }
            }
            Some(game)
        }
        Err(error) => {
            ui_state.message = t.text_args(
                "message-officer-data-load-failed",
                &args([("error", error.to_string())]),
            );
            None
        }
    }
}

fn banner_logo(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    let Some(logo) = &ui_state.banner_logo else {
        ui.label(
            egui::RichText::new(t.text("app-title"))
                .size(42.0)
                .color(war_gold())
                .strong(),
        );
        ui.label(
            egui::RichText::new("Shogun")
                .size(18.0)
                .color(war_text_muted()),
        );
        return;
    };

    let max_width = ui.available_width().min(BANNER_LOGO_MAX_WIDTH);
    let scale = (max_width / logo.crop_size.x)
        .min(BANNER_LOGO_MAX_HEIGHT / logo.crop_size.y)
        .max(0.0);
    let size = logo.crop_size * scale;

    ui.add(egui::Image::from_texture((logo.texture.id(), size)).fit_to_exact_size(size));
}

fn register_banner_logo(ctx: &egui::Context, image: &BevyImage) -> Result<MenuBannerLogo, String> {
    let decoded = decode_banner_logo(image)?;
    let texture = ctx.load_texture(
        "main_menu_banner_logo",
        decoded.color_image,
        egui::TextureOptions::LINEAR,
    );

    Ok(MenuBannerLogo {
        crop_size: decoded.crop_size,
        texture,
    })
}

fn register_main_menu_illustration(
    contexts: &mut EguiContexts<'_, '_>,
    handle: &Handle<BevyImage>,
    image: &BevyImage,
    index: usize,
) -> Result<MenuIllustration, String> {
    let decoded = decode_main_menu_illustration(image, index)?;
    let texture_id = contexts.add_image(EguiTextureHandle::Strong(handle.clone()));

    Ok(MenuIllustration {
        texture_id,
        crop_uv: decoded.crop_uv,
        crop_size: decoded.crop_size,
    })
}

fn register_main_menu_cloud_pattern(
    contexts: &mut EguiContexts<'_, '_>,
    handle: &Handle<BevyImage>,
    image: &BevyImage,
) -> Result<MenuCloudPattern, String> {
    let decoded = decode_png_rgba(image, "main menu cloud pattern")?;
    let size = egui::vec2(decoded.width as f32, decoded.height as f32);
    let texture_id = contexts.add_image(EguiTextureHandle::Strong(handle.clone()));

    Ok(MenuCloudPattern { texture_id, size })
}

struct DecodedBannerLogo {
    crop_size: egui::Vec2,
    color_image: egui::ColorImage,
}

struct DecodedMenuIllustration {
    crop_uv: egui::Rect,
    crop_size: egui::Vec2,
}

struct DecodedPng {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

fn decode_banner_logo(image: &BevyImage) -> Result<DecodedBannerLogo, String> {
    let mut decoded = decode_png_rgba(image, "banner logo")?;
    remove_flat_logo_background(&mut decoded.rgba, decoded.width, decoded.height);
    let crop = alpha_crop_bounds(&decoded.rgba, decoded.width, decoded.height).unwrap_or((
        0,
        0,
        decoded.width - 1,
        decoded.height - 1,
    ));
    let cropped_rgba = crop_rgba(&decoded.rgba, decoded.width, crop);
    let crop_width = crop.2 - crop.0 + 1;
    let crop_height = crop.3 - crop.1 + 1;
    let crop_size = egui::vec2(crop_width as f32, crop_height as f32);
    let color_image =
        egui::ColorImage::from_rgba_unmultiplied([crop_width, crop_height], &cropped_rgba);

    Ok(DecodedBannerLogo {
        crop_size,
        color_image,
    })
}

fn decode_main_menu_illustration(
    image: &BevyImage,
    index: usize,
) -> Result<DecodedMenuIllustration, String> {
    let decoded = decode_png_rgba(image, &format!("main menu illustration {index}"))?;
    let crop = alpha_crop_bounds(&decoded.rgba, decoded.width, decoded.height).unwrap_or((
        0,
        0,
        decoded.width - 1,
        decoded.height - 1,
    ));
    let crop_uv = crop_uv(crop, decoded.width, decoded.height);
    let crop_size = egui::vec2((crop.2 - crop.0 + 1) as f32, (crop.3 - crop.1 + 1) as f32);

    Ok(DecodedMenuIllustration { crop_uv, crop_size })
}

fn decode_png_rgba(image: &BevyImage, description: &str) -> Result<DecodedPng, String> {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let format = image.texture_descriptor.format;
    let data = image
        .data
        .as_ref()
        .ok_or_else(|| format!("decoded {description} has no CPU pixel data"))?;
    let rgba = match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Uint => {
            data.clone()
        }
        TextureFormat::Rgba16Unorm | TextureFormat::Rgba16Uint => rgba16_to_rgba8(data)?,
        _ => {
            return Err(format!(
                "unsupported {description} texture format: {format:?}"
            ));
        }
    };

    Ok(DecodedPng {
        width,
        height,
        rgba,
    })
}

fn fit_contained_size(image_size: egui::Vec2, available_size: egui::Vec2) -> egui::Vec2 {
    if image_size.x <= 0.0
        || image_size.y <= 0.0
        || available_size.x <= 0.0
        || available_size.y <= 0.0
    {
        return egui::Vec2::ZERO;
    }

    let scale = (available_size.x / image_size.x)
        .min(available_size.y / image_size.y)
        .max(0.0);
    image_size * scale
}

fn rgba16_to_rgba8(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if !bytes.len().is_multiple_of(8) {
        return Err(format!(
            "16-bit RGBA logo buffer length must be divisible by 8, got {}",
            bytes.len()
        ));
    }

    let mut rgba = Vec::with_capacity(bytes.len() / 2);
    for channel in bytes.chunks_exact(2) {
        let value = u16::from_le_bytes([channel[0], channel[1]]);
        rgba.push((value / 257) as u8);
    }
    Ok(rgba)
}

fn remove_flat_logo_background(rgba: &mut [u8], width: usize, height: usize) {
    if width == 0 || height == 0 {
        return;
    }

    let background = estimate_edge_background_color(rgba, width, height);
    for pixel in rgba.chunks_exact_mut(4) {
        let distance = color_distance([pixel[0], pixel[1], pixel[2]], background);
        if distance >= BANNER_LOGO_BACKGROUND_ALPHA_THRESHOLD {
            continue;
        }
        let alpha = ((distance as u16 * 255) / BANNER_LOGO_BACKGROUND_ALPHA_THRESHOLD as u16) as u8;
        pixel[3] = pixel[3].min(alpha);
    }
}

fn estimate_edge_background_color(rgba: &[u8], width: usize, height: usize) -> [u8; 3] {
    let mut sample = RgbSample::default();

    for x in 0..width {
        sample.add(rgba, width, x, 0);
        sample.add(rgba, width, x, height - 1);
    }
    for y in 1..height.saturating_sub(1) {
        sample.add(rgba, width, 0, y);
        sample.add(rgba, width, width - 1, y);
    }

    if sample.count == 0 {
        return [0, 0, 0];
    }

    [
        (sample.red / sample.count) as u8,
        (sample.green / sample.count) as u8,
        (sample.blue / sample.count) as u8,
    ]
}

#[derive(Default)]
struct RgbSample {
    red: u64,
    green: u64,
    blue: u64,
    count: u64,
}

impl RgbSample {
    fn add(&mut self, rgba: &[u8], width: usize, x: usize, y: usize) {
        let offset = (y * width + x) * 4;
        self.red += rgba[offset] as u64;
        self.green += rgba[offset + 1] as u64;
        self.blue += rgba[offset + 2] as u64;
        self.count += 1;
    }
}

fn color_distance(a: [u8; 3], b: [u8; 3]) -> u8 {
    a[0].abs_diff(b[0])
        .max(a[1].abs_diff(b[1]))
        .max(a[2].abs_diff(b[2]))
}

fn alpha_crop_bounds(
    rgba: &[u8],
    width: usize,
    height: usize,
) -> Option<(usize, usize, usize, usize)> {
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found_pixel = false;

    for y in 0..height {
        for x in 0..width {
            let alpha = rgba[(y * width + x) * 4 + 3];
            if alpha <= BANNER_LOGO_ALPHA_THRESHOLD {
                continue;
            }
            found_pixel = true;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    found_pixel.then_some((
        min_x.saturating_sub(BANNER_LOGO_CROP_PADDING),
        min_y.saturating_sub(BANNER_LOGO_CROP_PADDING),
        (max_x + BANNER_LOGO_CROP_PADDING).min(width - 1),
        (max_y + BANNER_LOGO_CROP_PADDING).min(height - 1),
    ))
}

fn crop_rgba(rgba: &[u8], width: usize, crop: (usize, usize, usize, usize)) -> Vec<u8> {
    let crop_width = crop.2 - crop.0 + 1;
    let crop_height = crop.3 - crop.1 + 1;
    let mut cropped = Vec::with_capacity(crop_width * crop_height * 4);

    for y in crop.1..=crop.3 {
        let start = (y * width + crop.0) * 4;
        let end = start + crop_width * 4;
        cropped.extend_from_slice(&rgba[start..end]);
    }

    cropped
}

fn crop_uv(crop: (usize, usize, usize, usize), width: usize, height: usize) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2(crop.0 as f32 / width as f32, crop.1 as f32 / height as f32),
        egui::pos2(
            (crop.2 + 1) as f32 / width as f32,
            (crop.3 + 1) as f32 / height as f32,
        ),
    )
}

fn extend_game_with_catalog_officers(game: &mut GameState, profiles: Vec<OfficerProfile>) {
    let catalog_faction_id = "catalog".to_string();
    game.factions
        .entry(catalog_faction_id.clone())
        .or_insert_with(|| Faction {
            id: catalog_faction_id.clone(),
            name: "资料库".to_string(),
            ruler_id: String::new(),
            heir_id: None,
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
        birth_year: profile.birth_year.unwrap_or(0),
        gender: profile.gender.clone(),
        status: OfficerStatus::Unavailable,
        profile: Some(profile),
    }
}

pub(super) fn officer_settings_modal(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
) {
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
                if modal_title_bar(ui, t, &t.text("officer-settings-title")) {
                    ui_state.officer_settings_open = false;
                    ui_state.officer_detail_id = None;
                }
                ui.separator();
                if let Some(game) = &ui_state.officer_settings_game {
                    officer_browser_filters(
                        ui,
                        game,
                        &mut ui_state.officer_settings_filters,
                        "main_menu_officer_settings_filters",
                        t,
                    );
                    if !ui_state.officer_settings_editable {
                        ui.colored_label(war_text_muted(), t.text("officer-settings-readonly"));
                    }
                    ui.separator();
                    let response = officer_browser_table(
                        ui,
                        game,
                        &ui_state.officer_settings_filters,
                        OfficerBrowserTableOptions {
                            max_height: height - 118.0,
                            id_salt: "main_menu_officer_settings_table",
                            selected_officer_id: ui_state.officer_settings_selected_id.as_deref(),
                            editable: ui_state.officer_settings_editable,
                            retainer_faction_id: None,
                        },
                        t,
                    );
                    if let Some(officer_id) = response.selected_officer_id {
                        ui_state.officer_settings_selected_id = Some(officer_id);
                    }
                    if let Some(officer_id) = response.view_officer_id {
                        ui_state.officer_detail_id = Some(officer_id);
                    }
                    if let Some(officer_id) = response.edit_officer_id {
                        open_officer_profile_editor(ui_state, &officer_id);
                    }
                } else {
                    ui.label(t.text("officer-settings-empty"));
                }
            });
        });
}

fn open_officer_profile_editor(ui_state: &mut GameUiState, officer_id: &str) {
    if !ui_state.officer_settings_editable {
        let t = Translator::new(ui_state.applied_settings.general.ui_language);
        ui_state.message = t.text("officer-settings-readonly");
        return;
    }
    let profile = ui_state
        .officer_settings_game
        .as_ref()
        .and_then(|game| game.officers.get(officer_id))
        .and_then(|officer| officer.profile.as_ref());
    let Some(profile) = profile else {
        let t = Translator::new(ui_state.applied_settings.general.ui_language);
        ui_state.officer_edit_error = Some(t.text_args(
            "message-officer-profile-missing",
            &args([("id", officer_id.to_string())]),
        ));
        return;
    };
    ui_state.officer_settings_selected_id = Some(officer_id.to_string());
    ui_state.officer_edit_draft = Some(OfficerEditDraft::from_profile(profile));
    ui_state.officer_edit_error = None;
    ui_state.officer_edit_open = true;
}

fn officer_profile_edit_modal(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    async_runtime: &CoreAsyncRuntime,
) {
    let screen = ctx.content_rect();
    let width = (screen.width() * 0.72).clamp(760.0, 900.0);
    let height = (screen.height() * 0.78).clamp(460.0, 720.0);
    egui::Area::new(egui::Id::new("officer_profile_edit_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("officer-edit-title")) {
                    close_officer_profile_editor(ui_state);
                }
                ui.separator();
                if ui_state.officer_edit_draft.is_some() {
                    let content_height = height - 116.0;
                    ui.horizontal_top(|ui| {
                        let form_width =
                            (width - OFFICER_PORTRAIT_PANEL_WIDTH - OFFICER_PORTRAIT_GAP)
                                .max(420.0);
                        ui.vertical(|ui| {
                            ui.set_width(form_width);
                            egui::ScrollArea::vertical()
                                .id_salt("officer_profile_edit_scroll")
                                .max_height(content_height)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.set_width(form_width);
                                    officer_profile_edit_form(ui, ui_state, t);
                                });
                        });
                        ui.add_space(OFFICER_PORTRAIT_GAP);
                        officer_profile_portrait_panel(
                            ctx,
                            ui,
                            ui_state,
                            t,
                            async_runtime,
                            OFFICER_PORTRAIT_PANEL_WIDTH,
                            content_height,
                        );
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([108.0, 34.0], egui::Button::new(t.text("common-save")))
                            .clicked()
                        {
                            save_officer_profile_edit(ui_state);
                        }
                        if ui.button(t.text("common-cancel")).clicked() {
                            close_officer_profile_editor(ui_state);
                        }
                    });
                } else {
                    ui.label(t.text("officer-edit-none-selected"));
                }
            });
        });
}

fn officer_profile_edit_form(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    let Some(draft) = ui_state.officer_edit_draft.as_mut() else {
        return;
    };
    ui.label(egui::RichText::new(format!("ID: {}", draft.id)).color(war_text_muted()));
    ui.add_space(6.0);
    egui::Grid::new("officer_profile_edit_grid")
        .num_columns(2)
        .spacing(egui::vec2(18.0, 8.0))
        .show(ui, |ui| {
            ui.label(t.text("officer-field-name"));
            ui.text_edit_singleline(&mut draft.name);
            ui.end_row();

            ui.label(t.text("officer-field-courtesy-name"));
            ui.text_edit_singleline(&mut draft.courtesy_name);
            ui.end_row();

            ui.label(t.text("officer-field-native-place"));
            ui.text_edit_singleline(&mut draft.native_place);
            ui.end_row();

            ui.label(t.text("officer-field-birth-year"));
            ui.text_edit_singleline(&mut draft.birth_year);
            ui.end_row();

            ui.label(t.text("officer-field-death-year"));
            ui.text_edit_singleline(&mut draft.death_year);
            ui.end_row();

            ui.label(t.text("officer-field-gender"));
            egui::ComboBox::from_id_salt("officer_edit_gender")
                .selected_text(officer_gender_label(t, &draft.gender))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut draft.gender,
                        OfficerGender::Male,
                        t.text("gender-male"),
                    );
                    ui.selectable_value(
                        &mut draft.gender,
                        OfficerGender::Female,
                        t.text("gender-female"),
                    );
                });
            ui.end_row();

            ui.label(t.text("officer-field-confidence"));
            egui::ComboBox::from_id_salt("officer_edit_confidence")
                .selected_text(confidence_label(t, &draft.confidence))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut draft.confidence,
                        SourceConfidence::High,
                        t.text("confidence-high"),
                    );
                    ui.selectable_value(
                        &mut draft.confidence,
                        SourceConfidence::Medium,
                        t.text("confidence-medium"),
                    );
                    ui.selectable_value(
                        &mut draft.confidence,
                        SourceConfidence::Low,
                        t.text("confidence-low"),
                    );
                });
            ui.end_row();

            ui.label(t.text("officer-field-tags"));
            officer_profile_tag_selector(ui, &ui_state.officer_settings_game, draft, t);
            ui.end_row();
        });

    ui.add_space(10.0);
    ui.horizontal_wrapped(|ui| {
        ability_drag(ui, &t.text("stat-leadership"), &mut draft.leadership);
        ability_drag(ui, &t.text("stat-strength"), &mut draft.strength);
        ability_drag(ui, &t.text("stat-intelligence"), &mut draft.intelligence);
        ability_drag(ui, &t.text("stat-politics"), &mut draft.politics);
        ability_drag(ui, &t.text("stat-charm"), &mut draft.charm);
    });

    ui.add_space(10.0);
    ui.label(t.text("officer-field-biography"));
    ui.add_sized(
        [ui.available_width(), 150.0],
        egui::TextEdit::multiline(&mut draft.biography),
    );
    ui.add_space(8.0);
    ui.label(t.text("officer-field-notes"));
    ui.add_sized(
        [ui.available_width(), 72.0],
        egui::TextEdit::multiline(&mut draft.notes),
    );

    if let Some(error) = &ui_state.officer_edit_error {
        ui.add_space(8.0);
        ui.colored_label(egui::Color32::from_rgb(220, 92, 72), error);
    }
}

fn officer_profile_tag_selector(
    ui: &mut egui::Ui,
    game: &Option<GameState>,
    draft: &mut OfficerEditDraft,
    t: &Translator,
) {
    let Some(game) = game else {
        ui.colored_label(war_text_muted(), t.text("officer-edit-no-tag-definitions"));
        return;
    };
    if game.officer_tag_definitions.is_empty() {
        ui.colored_label(war_text_muted(), t.text("officer-edit-no-tag-definitions"));
        return;
    }
    ui.vertical(|ui| {
        for (category, definitions) in officer_tag_definitions_by_category(game) {
            ui.label(
                egui::RichText::new(officer_tag_category_label(category, t))
                    .color(war_text_muted()),
            );
            ui.horizontal_wrapped(|ui| {
                for definition in definitions {
                    let mut selected = draft.tag_ids.contains(&definition.id);
                    if ui
                        .checkbox(&mut selected, officer_tag_label(definition, t))
                        .changed()
                    {
                        if selected {
                            draft.tag_ids.insert(definition.id.clone());
                        } else {
                            draft.tag_ids.remove(&definition.id);
                        }
                    }
                }
            });
        }
    });
}

fn officer_profile_portrait_panel(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    async_runtime: &CoreAsyncRuntime,
    width: f32,
    max_height: f32,
) {
    let Some(draft) = ui_state.officer_edit_draft.clone() else {
        return;
    };
    let path = officer_portrait_path(&draft.id);
    let has_portrait = path.as_ref().is_ok_and(|path| path.is_file());
    let task_state = ui_state.officer_portraits.task_state(&draft.id);
    let generating = matches!(task_state, OfficerPortraitTaskState::Generating);
    let mut load_error = None;
    let texture = match &path {
        Ok(path) => match ui_state.officer_portraits.texture_for(ctx, &draft.id, path) {
            Ok(texture) => texture,
            Err(error) => {
                load_error = Some(error);
                None
            }
        },
        Err(error) => {
            load_error = Some(error.clone());
            None
        }
    };

    ui.allocate_ui_with_layout(
        egui::vec2(width, max_height),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.set_width(width);
            war_sub_panel_frame().show(ui, |ui| {
                ui.set_width((width - 20.0).max(0.0));
                ui.label(
                    egui::RichText::new(t.text("officer-portrait-title"))
                        .strong()
                        .color(war_gold()),
                );
                ui.add_space(4.0);

                let preview_width = ui.available_width().min(width - 20.0).max(0.0);
                let preview_height =
                    preview_width * OFFICER_PORTRAIT_ASPECT_HEIGHT / OFFICER_PORTRAIT_ASPECT_WIDTH;
                let preview_size = egui::vec2(preview_width, preview_height);
                let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
                paint_officer_portrait_preview(ui, rect, texture, generating, t);

                ui.add_space(8.0);
                officer_portrait_status_line(
                    ui,
                    t,
                    &task_state,
                    has_portrait,
                    load_error.as_deref(),
                );
                ui.add_space(6.0);

                let button_text = if generating {
                    t.text("officer-portrait-generating")
                } else if has_portrait {
                    t.text("officer-portrait-regenerate")
                } else {
                    t.text("officer-portrait-generate")
                };
                let clicked = ui
                    .add_enabled(
                        !generating,
                        egui::Button::new(button_text).min_size(egui::vec2(preview_width, 34.0)),
                    )
                    .clicked();
                if clicked {
                    ui_state.officer_edit_error = None;
                    ui_state.officer_portraits.start_generation(
                        async_runtime,
                        draft,
                        ui_state.applied_settings.ai.multimodal.api_key.clone(),
                        ui_state.applied_settings.ai.multimodal.model_name.clone(),
                        t.text("officer-portrait-api-key-required"),
                    );
                }
            });
        },
    );
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
            let t = Translator::new(ui_state.applied_settings.general.ui_language);
            ui_state.officer_edit_error = Some(localized_officer_edit_error(&t, &error));
            return;
        }
    };
    let result = SqliteHistoricalCatalog::open_default()
        .and_then(|catalog| catalog.update_officer_profile(&draft.id, &update));
    match result {
        Ok(profile) => {
            sync_updated_officer_profile(ui_state, profile);
            let t = Translator::new(ui_state.applied_settings.general.ui_language);
            ui_state.message = t.text_args(
                "message-officer-saved",
                &args([("name", draft.name.trim().to_string())]),
            );
            close_officer_profile_editor(ui_state);
        }
        Err(error) => {
            ui_state.officer_edit_error = Some(error.to_string());
        }
    }
}

fn draft_to_update(draft: &OfficerEditDraft) -> Result<OfficerProfileUpdate, String> {
    if draft.name.trim().is_empty() {
        return Err("officer-edit-name-required".to_string());
    }
    Ok(OfficerProfileUpdate {
        name: draft.name.trim().to_string(),
        courtesy_name: optional_text(&draft.courtesy_name),
        native_place: optional_text(&draft.native_place),
        birth_year: optional_year(&draft.birth_year, "officer-field-birth-year")?,
        death_year: optional_year(&draft.death_year, "officer-field-death-year")?,
        gender: draft.gender.clone(),
        stats: OfficerStats {
            leadership: draft.leadership,
            strength: draft.strength,
            intelligence: draft.intelligence,
            politics: draft.politics,
            charm: draft.charm,
        },
        tags: draft.tag_ids.iter().cloned().collect(),
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
            .map_err(|_| format!("officer-edit-year-invalid:{label}"))
    }
}

fn localized_officer_edit_error(t: &Translator, error: &str) -> String {
    if error == "officer-edit-name-required" {
        return t.text(error);
    }
    if let Some(label_key) = error.strip_prefix("officer-edit-year-invalid:") {
        return t.text_args(
            "officer-edit-year-invalid",
            &args([("label", t.text(label_key))]),
        );
    }
    error.to_string()
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
    use crate::core::settings::{GameSettings, GameSettingsStore, LoadedGameSettings};

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
            tags: vec!["role:ruler".to_string()],
            confidence: SourceConfidence::High,
            biography: "刘备生平".to_string(),
            relationships: Vec::new(),
            notes: "测试".to_string(),
        }
    }

    fn ui_state_with_officer_settings_profile() -> GameUiState {
        let mut state = GameUiState::new(
            GameSettingsStore::with_default_path(),
            LoadedGameSettings {
                settings: GameSettings::default(),
                message: None,
            },
        );
        let mut game = SqliteHistoricalCatalog::in_memory_from_seed()
            .unwrap()
            .build_game("ad200", "liu_bei")
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
    fn banner_logo_decodes_and_respects_display_bounds() {
        let image = decode_png_asset(BANNER_LOGO_ASSET_PATH, "banner logo").unwrap();
        let decoded = decode_banner_logo(&image).unwrap();

        assert!(decoded.crop_size.x > 0.0);
        assert!(decoded.crop_size.y > 0.0);

        let scale = (BANNER_LOGO_MAX_WIDTH / decoded.crop_size.x)
            .min(BANNER_LOGO_MAX_HEIGHT / decoded.crop_size.y);
        let display_size = decoded.crop_size * scale;

        assert!(display_size.x <= BANNER_LOGO_MAX_WIDTH + f32::EPSILON);
        assert!(display_size.y <= BANNER_LOGO_MAX_HEIGHT + f32::EPSILON);
    }

    #[test]
    fn banner_logo_background_is_removed_from_rendered_texture() {
        let image = decode_png_asset(BANNER_LOGO_ASSET_PATH, "banner logo").unwrap();
        let decoded = decode_banner_logo(&image).unwrap();

        let [width, height] = decoded.color_image.size;
        let corners = [
            (0, 0),
            (width - 1, 0),
            (0, height - 1),
            (width - 1, height - 1),
        ];

        for (x, y) in corners {
            assert_eq!(decoded.color_image[(x, y)].a(), 0);
        }
    }

    #[test]
    fn main_menu_cloud_pattern_decodes_at_expected_size() {
        let image = decode_png_asset(
            MAIN_MENU_CLOUD_PATTERN_ASSET_PATH,
            "main menu cloud pattern",
        )
        .unwrap();
        let decoded = decode_png_rgba(&image, "main menu cloud pattern").unwrap();

        assert_eq!(decoded.width, 1536);
        assert_eq!(decoded.height, 1024);
    }

    #[test]
    fn main_menu_illustrations_decode_at_expected_size() {
        for (index, asset_path) in MAIN_MENU_ILLUSTRATION_ASSET_PATHS.iter().enumerate() {
            let image =
                decode_png_asset(asset_path, &format!("main menu illustration {index}")).unwrap();
            let decoded =
                decode_png_rgba(&image, &format!("main menu illustration {index}")).unwrap();

            assert_eq!(decoded.width, 1152);
            assert_eq!(decoded.height, 1696);
        }
    }

    fn decode_png_asset(asset_path: &str, description: &str) -> Result<BevyImage, String> {
        let bytes = std::fs::read(std::path::Path::new("assets").join(asset_path))
            .map_err(|error| format!("failed to read {description} PNG asset: {error}"))?;
        BevyImage::from_buffer(
            &bytes,
            bevy::image::ImageType::Format(bevy::image::ImageFormat::Png),
            bevy::image::CompressedImageFormats::NONE,
            true,
            bevy::image::ImageSampler::Default,
            bevy::asset::RenderAssetUsages::MAIN_WORLD,
        )
        .map_err(|error| format!("failed to decode {description} PNG: {error}"))
    }

    #[test]
    fn main_menu_illustration_fit_respects_display_bounds() {
        let image_size = egui::vec2(1152.0, 1696.0);

        for available_size in [egui::vec2(804.0, 788.0), egui::vec2(804.0, 688.0)] {
            let display_size = fit_contained_size(image_size, available_size);

            assert!(display_size.x <= available_size.x + f32::EPSILON);
            assert!(display_size.y <= available_size.y + f32::EPSILON);
            assert!((display_size.x / display_size.y - image_size.x / image_size.y).abs() < 0.001);
        }
    }

    #[test]
    fn draft_to_update_normalizes_blank_optional_fields_and_keeps_tags() {
        let mut draft = OfficerEditDraft::from_profile(&test_profile("刘备"));
        draft.courtesy_name = " ".to_string();
        draft.native_place = " 涿郡 ".to_string();
        draft.birth_year.clear();
        draft.death_year = "223".to_string();
        draft.tag_ids.insert("source:manual_curated".to_string());

        let update = draft_to_update(&draft).unwrap();

        assert_eq!(update.courtesy_name, None);
        assert_eq!(update.native_place.as_deref(), Some("涿郡"));
        assert_eq!(update.birth_year, None);
        assert_eq!(update.death_year, Some(223));
        assert_eq!(update.tags, ["role:ruler", "source:manual_curated"]);
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
