use bevy::{
    image::Image as BevyImage,
    prelude::{Assets, Handle, Res, ResMut, Resource},
    render::render_resource::TextureFormat,
};
use bevy_asset_loader::prelude::AssetCollection;
use bevy_egui::{EguiTextureHandle, EguiUserTextures, egui};

use crate::build_info::menu_build_label;
use crate::game::{
    Controller, Faction, GameState, HistoricalCatalog, Officer, OfficerCatalog, OfficerGender,
    OfficerProfile, OfficerProfileUpdate, OfficerStats, OfficerStatus, SourceConfidence,
    SqliteHistoricalCatalog,
};

use super::HUD_MARGIN;
use super::actions::{enter_game, refresh_saves, start_history_game, start_json_game};
use super::hud::{OfficerBrowserTableOptions, officer_browser_filters, officer_browser_table};
use super::labels::{confidence_label, officer_gender_label};
use super::settings::{refresh_audio_output_devices, settings_modal};
use super::state::{
    GameUiState, MenuBannerLogo, MenuCloudPattern, MenuIllustration, OfficerEditDraft,
    refresh_history_factions, refresh_history_menu,
};
use super::style::{
    modal_title_bar, war_gold, war_panel_frame, war_sub_panel_frame, war_text_muted,
};

#[cfg(test)]
const BANNER_LOGO_ASSET_PATH: &str = "icons/banner_logo.png";
const BANNER_LOGO_ALPHA_THRESHOLD: u8 = 8;
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
    ApplyGameSettings,
    Exit,
}

pub(super) fn main_menu(ctx: &egui::Context, ui_state: &mut GameUiState) -> MainMenuAction {
    let mut action = MainMenuAction::None;
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            action = main_menu_columns(ui, ui_state);
        });

    if ui_state.main_menu_new_game_open {
        new_game_modal(ctx, ui_state);
    }
    if ui_state.main_menu_load_game_open {
        load_game_modal(ctx, ui_state);
    }
    if ui_state.settings_open {
        if settings_modal(ctx, ui_state) {
            action = MainMenuAction::ApplyGameSettings;
        }
    }
    if ui_state.officer_settings_open {
        officer_settings_modal(ctx, ui_state);
    }
    if ui_state.officer_edit_open {
        officer_profile_edit_modal(ctx, ui_state);
    }
    action
}

pub(super) fn prepare_main_menu_assets_for_egui(
    mut ui_state: ResMut<GameUiState>,
    menu_assets: Option<Res<MainMenuAssets>>,
    images: Res<Assets<BevyImage>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
) {
    let Some(menu_assets) = menu_assets else {
        return;
    };

    if ui_state.banner_logo.is_none()
        && ui_state.banner_logo_error.is_none()
        && let Some(image) = images.get(&menu_assets.banner_logo)
    {
        match register_banner_logo(&mut egui_user_textures, &menu_assets.banner_logo, image) {
            Ok(logo) => ui_state.banner_logo = Some(logo),
            Err(error) => ui_state.banner_logo_error = Some(error),
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
        match register_main_menu_illustration(&mut egui_user_textures, handle, image, index) {
            Ok(illustration) => ui_state.main_menu_illustrations[index] = Some(illustration),
            Err(error) => ui_state.main_menu_illustration_errors[index] = Some(error),
        }
    }

    if ui_state.main_menu_cloud_pattern.is_none()
        && ui_state.main_menu_cloud_pattern_error.is_none()
        && let Some(image) = images.get(&menu_assets.cloud_pattern)
    {
        match register_main_menu_cloud_pattern(
            &mut egui_user_textures,
            &menu_assets.cloud_pattern,
            image,
        ) {
            Ok(pattern) => ui_state.main_menu_cloud_pattern = Some(pattern),
            Err(error) => ui_state.main_menu_cloud_pattern_error = Some(error),
        }
    }
}

fn main_menu_columns(ui: &mut egui::Ui, ui_state: &mut GameUiState) -> MainMenuAction {
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
    let action = draw_main_menu_left_panel(ui, ui_state, left_rect);
    draw_main_menu_bgm_button(ui, ui_state, content_rect);
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
            banner_logo(ui, ui_state);
            ui.add(
                egui::Label::new(
                    egui::RichText::new(menu_build_label())
                        .size(12.0)
                        .color(war_text_muted()),
                )
                .truncate(),
            );
            ui.add_space((inner.height() * 0.04).clamp(12.0, 24.0));

            action = draw_main_menu_buttons(ui, ui_state);

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

fn draw_main_menu_buttons(ui: &mut egui::Ui, ui_state: &mut GameUiState) -> MainMenuAction {
    let button_width = ui.available_width().min(MAIN_MENU_BUTTON_WIDTH);
    let mut action = MainMenuAction::None;
    let mut hovered_illustration_index = None;

    let new_game_response = main_menu_button(ui, button_width, "新的开始");
    if new_game_response.hovered() {
        hovered_illustration_index = Some(1);
    }
    if new_game_response.clicked() {
        close_main_menu_popups(ui_state);
        ui_state.main_menu_new_game_open = true;
    }
    ui.add_space(MAIN_MENU_BUTTON_SPACING);
    let load_game_response = main_menu_button(ui, button_width, "继续征途");
    if load_game_response.hovered() {
        hovered_illustration_index = Some(2);
    }
    if load_game_response.clicked() {
        close_main_menu_popups(ui_state);
        refresh_saves(ui_state);
        ui_state.main_menu_load_game_open = true;
    }
    ui.add_space(MAIN_MENU_BUTTON_SPACING);
    let officer_response = main_menu_button(ui, button_width, "武将设置");
    if officer_response.hovered() {
        hovered_illustration_index = Some(3);
    }
    if officer_response.clicked() {
        close_main_menu_popups(ui_state);
        open_officer_settings(ui_state);
    }
    ui.add_space(MAIN_MENU_BUTTON_SPACING);
    let settings_response = main_menu_button(ui, button_width, "游戏设置");
    if settings_response.hovered() {
        hovered_illustration_index = Some(4);
    }
    if settings_response.clicked() {
        close_main_menu_popups(ui_state);
        refresh_audio_output_devices(ui_state);
        ui_state.settings_open = true;
    }
    ui.add_space(MAIN_MENU_BUTTON_SPACING);
    let exit_response = main_menu_button(ui, button_width, "退出游戏");
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
        (egui_phosphor::regular::SPEAKER_HIGH, "关闭背景音乐")
    } else {
        (egui_phosphor::regular::SPEAKER_X, "打开背景音乐")
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

fn new_game_modal(ctx: &egui::Context, ui_state: &mut GameUiState) {
    main_menu_scrim(ctx, ui_state);

    let screen = ctx.content_rect();
    let width = (screen.width() * 0.48).clamp(440.0, 640.0);
    let height = (screen.height() - HUD_MARGIN * 2.0).clamp(500.0, 680.0);
    egui::Area::new(egui::Id::new("main_menu_new_game_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, "新的开始") {
                    ui_state.main_menu_new_game_open = false;
                }
                ui.separator();
                new_game_menu(ui, ui_state);
            });
        });
}

fn load_game_modal(ctx: &egui::Context, ui_state: &mut GameUiState) {
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
                if modal_title_bar(ui, "继续征途") {
                    ui_state.main_menu_load_game_open = false;
                }
                ui.separator();
                load_game_menu(ui, ui_state);
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

fn banner_logo(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let Some(logo) = &ui_state.banner_logo else {
        ui.label(
            egui::RichText::new("三国争霸")
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

    ui.add(
        egui::Image::from_texture((logo.texture_id, size))
            .uv(logo.crop_uv)
            .fit_to_exact_size(size),
    );
}

fn register_banner_logo(
    egui_user_textures: &mut EguiUserTextures,
    handle: &Handle<BevyImage>,
    image: &BevyImage,
) -> Result<MenuBannerLogo, String> {
    let decoded = decode_banner_logo(image)?;
    let texture_id = egui_user_textures.add_image(EguiTextureHandle::Strong(handle.clone()));

    Ok(MenuBannerLogo {
        texture_id,
        crop_uv: decoded.crop_uv,
        crop_size: decoded.crop_size,
    })
}

fn register_main_menu_illustration(
    egui_user_textures: &mut EguiUserTextures,
    handle: &Handle<BevyImage>,
    image: &BevyImage,
    index: usize,
) -> Result<MenuIllustration, String> {
    let decoded = decode_main_menu_illustration(image, index)?;
    let texture_id = egui_user_textures.add_image(EguiTextureHandle::Strong(handle.clone()));

    Ok(MenuIllustration {
        texture_id,
        crop_uv: decoded.crop_uv,
        crop_size: decoded.crop_size,
    })
}

fn register_main_menu_cloud_pattern(
    egui_user_textures: &mut EguiUserTextures,
    handle: &Handle<BevyImage>,
    image: &BevyImage,
) -> Result<MenuCloudPattern, String> {
    let decoded = decode_png_rgba(image, "main menu cloud pattern")?;
    let size = egui::vec2(decoded.width as f32, decoded.height as f32);
    let texture_id = egui_user_textures.add_image(EguiTextureHandle::Strong(handle.clone()));

    Ok(MenuCloudPattern { texture_id, size })
}

struct DecodedBannerLogo {
    crop_uv: egui::Rect,
    crop_size: egui::Vec2,
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
    let decoded = decode_png_rgba(image, "banner logo")?;
    let crop = alpha_crop_bounds(&decoded.rgba, decoded.width, decoded.height).unwrap_or((
        0,
        0,
        decoded.width - 1,
        decoded.height - 1,
    ));
    let crop_uv = crop_uv(crop, decoded.width, decoded.height);
    let crop_size = egui::vec2((crop.2 - crop.0 + 1) as f32, (crop.3 - crop.1 + 1) as f32);

    Ok(DecodedBannerLogo { crop_uv, crop_size })
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
        TextureFormat::Rgba16Unorm | TextureFormat::Rgba16Uint => rgba16_to_rgba8(&data)?,
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
                        OfficerBrowserTableOptions {
                            max_height: height - 118.0,
                            id_salt: "main_menu_officer_settings_table",
                            selected_officer_id: ui_state.officer_settings_selected_id.as_deref(),
                            editable: ui_state.officer_settings_editable,
                            retainer_faction_id: None,
                        },
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
    use crate::core::display_settings::{GameSettings, GameSettingsStore, LoadedGameSettings};
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
            GameSettingsStore::with_default_path(),
            LoadedGameSettings {
                settings: GameSettings::default(),
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
