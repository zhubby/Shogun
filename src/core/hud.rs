use crate::game::*;
use bevy_egui::egui;

use super::actions::{
    clear_pending_commands, enter_game, finish_current_turn, open_city, refresh_saves,
};
use super::city_intel::city_summary_intel;
use super::city_panel::selected_city_panel;
use super::labels::{officer_gender_label, technology_branch_label};
use super::map::{map_panel, reset_map_view, zoom_map};
use super::state::{
    GameUiState, OfficerBrowserFilters, OfficerGenderFilter, OfficerStatusFilter, Screen,
};
use super::style::{
    modal_title_bar, war_bar_frame, war_border, war_danger, war_gold, war_panel_frame,
    war_sub_panel_frame, war_success, war_text_muted, war_warning,
};
use super::{HUD_MARGIN, HUD_TOP_HEIGHT, HUD_TOP_OFFSET, MAP_ZOOM_STEP};

pub(super) fn in_game(ctx: &egui::Context, ui_state: &mut GameUiState) {
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            map_panel(ui, ui_state);
        });

    in_game_hud(ctx, ui_state);
}

pub(super) fn in_game_hud(ctx: &egui::Context, ui_state: &mut GameUiState) {
    let screen = ctx.content_rect();
    top_status_hud(ctx, ui_state, screen);
    map_controls_hud(ctx, ui_state);
    left_city_summary_hud(ctx, ui_state);
    city_list_hud(ctx, ui_state, screen);
    save_hud(ctx, ui_state, screen);
    city_drawer_hud(ctx, ui_state, screen);
    report_hud(ctx, ui_state, screen);
    bottom_map_actions_hud(ctx, ui_state);
    officer_browser_hud(ctx, ui_state, screen);
    retainer_hud(ctx, ui_state, screen);
    technology_hud(ctx, ui_state, screen);
}

pub(super) fn top_status_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    let width = (screen.width() - HUD_MARGIN * 2.0).max(320.0);
    let summary = ui_state.game.as_ref().map(|game| {
        let faction_name = game
            .factions
            .get(&game.player_faction_id)
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| "未知势力".to_string());
        let status = match &game.status {
            GameStatus::Running => None,
            GameStatus::Victory { reason } => Some(format!("胜利: {reason}")),
            GameStatus::Defeat { reason } => Some(format!("失败: {reason}")),
        };
        (
            game.scenario_name.clone(),
            game.year,
            game.month,
            game.turn,
            faction_name,
            status,
        )
    });

    egui::Area::new(egui::Id::new("hud_top_status"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(HUD_MARGIN, HUD_TOP_OFFSET),
        )
        .show(ctx, |ui| {
            war_bar_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("三国争霸")
                            .size(24.0)
                            .color(war_gold())
                            .strong(),
                    );
                    ui.separator();
                    if let Some((scenario, year, month, turn, faction_name, status)) = summary {
                        ui.label(format!("{scenario}  {year}年{month}月  第{turn}回合"));
                        ui.label(format!("玩家: {faction_name}"));
                        if let Some(status) = status {
                            ui.colored_label(egui::Color32::from_rgb(200, 72, 52), status);
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("主菜单").clicked() {
                            ui_state.screen = Screen::MainMenu;
                        }
                        if ui.button("存档").clicked() {
                            ui_state.save_panel_open = !ui_state.save_panel_open;
                        }
                        if ui.button("清空命令").clicked() {
                            clear_pending_commands(ui_state);
                        }
                        if ui.button("结束本月").clicked() {
                            finish_current_turn(ui_state);
                        }
                    });
                });
            });
        });
}

pub(super) fn map_controls_hud(ctx: &egui::Context, ui_state: &mut GameUiState) {
    egui::Area::new(egui::Id::new("hud_map_controls"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_TOP,
            egui::vec2(-HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(270.0);
                map_controls(ui, ui_state);
            });
        });
}

pub(super) fn left_city_summary_hud(ctx: &egui::Context, ui_state: &mut GameUiState) {
    egui::Area::new(egui::Id::new("hud_left_city_summary"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(285.0);
                selected_city_summary(ui, ui_state);
            });
        });
}

pub(super) fn bottom_map_actions_hud(ctx: &egui::Context, ui_state: &mut GameUiState) {
    egui::Area::new(egui::Id::new("hud_bottom_map_actions"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_BOTTOM,
            egui::vec2(-HUD_MARGIN, -HUD_MARGIN),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    let city_label = if ui_state.city_list_open {
                        "收起城池"
                    } else {
                        "城池"
                    };
                    if ui.button(city_label).clicked() {
                        ui_state.city_list_open = !ui_state.city_list_open;
                    }

                    let officer_label = if ui_state.officer_browser_open {
                        "收起武将"
                    } else {
                        "武将"
                    };
                    if ui.button(officer_label).clicked() {
                        ui_state.officer_browser_open = !ui_state.officer_browser_open;
                    }

                    let retainer_label = if ui_state.retainers_open {
                        "收起幕僚"
                    } else {
                        "幕僚"
                    };
                    if ui.button(retainer_label).clicked() {
                        ui_state.retainers_open = !ui_state.retainers_open;
                    }

                    let technology_label = if ui_state.technology_open {
                        "收起科技"
                    } else {
                        "科技"
                    };
                    if ui.button(technology_label).clicked() {
                        ui_state.technology_open = !ui_state.technology_open;
                    }
                });
            });
        });
}

pub(super) fn technology_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    if !ui_state.technology_open {
        return;
    }

    let width = (screen.width() * 0.84).clamp(780.0, 1120.0);
    let height = (screen.height() * 0.78).clamp(500.0, 720.0);
    egui::Area::new(egui::Id::new("hud_technology"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, "科技") {
                    ui_state.technology_open = false;
                }
                ui.separator();
                technology_panel(ui, ui_state, width, height);
            });
        });
}

fn technology_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState, width: f32, height: f32) {
    let Some(game) = ui_state.game.as_ref().cloned() else {
        ui.label("当前没有剧本局面。");
        return;
    };
    let branch = ui_state.selected_technology_branch;
    if technology_spec(ui_state.selected_technology_id).branch != branch {
        ui_state.selected_technology_id = technology_specs_for_branch(branch)
            .next()
            .map(|spec| spec.id)
            .unwrap_or(TechnologyId::MilitiaDrill);
    }

    ui.horizontal(|ui| {
        for branch in [TechnologyBranch::Military, TechnologyBranch::Domestic] {
            if ui
                .selectable_label(
                    ui_state.selected_technology_branch == branch,
                    technology_branch_label(branch),
                )
                .clicked()
            {
                ui_state.selected_technology_branch = branch;
                ui_state.selected_technology_id = technology_specs_for_branch(branch)
                    .next()
                    .map(|spec| spec.id)
                    .unwrap_or(ui_state.selected_technology_id);
            }
        }
    });
    ui.add_space(8.0);

    ui.columns(2, |columns| {
        columns[0].set_width(width * 0.46);
        war_sub_panel_frame().show(&mut columns[0], |ui| {
            ui.label(egui::RichText::new("科技树").color(war_gold()).strong());
            ui.add_space(6.0);
            technology_tree(ui, ui_state, &game, height - 118.0);
        });

        war_sub_panel_frame().show(&mut columns[1], |ui| {
            ui.label(egui::RichText::new("详情").color(war_gold()).strong());
            ui.add_space(6.0);
            technology_detail(ui, ui_state, &game);
        });
    });
}

fn technology_tree(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    max_height: f32,
) {
    let faction_state = faction_technology_state(game, &game.player_faction_id);
    egui::ScrollArea::vertical()
        .id_salt("technology_tree")
        .max_height(max_height.max(320.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for spec in technology_specs_for_branch(ui_state.selected_technology_branch) {
                let selected = ui_state.selected_technology_id == spec.id;
                let status = technology_node_status(game, faction_state, spec);
                let response = technology_tree_node(ui, spec, selected, status, faction_state);
                if response.clicked() {
                    ui_state.selected_technology_id = spec.id;
                }
                response.on_hover_text(spec.effect);
                ui.add_space(7.0);
            }
        });
}

fn technology_tree_node(
    ui: &mut egui::Ui,
    spec: &TechnologySpec,
    selected: bool,
    status: TechnologyNodeStatus,
    faction_state: Option<&FactionTechnologyState>,
) -> egui::Response {
    const ROW_HEIGHT: f32 = 48.0;
    let depth = technology_depth(spec);
    let indent = 18.0 + depth as f32 * 34.0;
    let available = ui.available_width().max(360.0);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(available, ROW_HEIGHT), egui::Sense::click());
    let painter = ui.painter_at(rect);
    let visuals = technology_status_visuals(status);
    let node_center = egui::pos2(rect.left() + indent + 12.0, rect.center().y);
    let has_parent = !spec.prerequisites.is_empty();

    if selected || response.hovered() {
        let fill = if selected {
            egui::Color32::from_rgba_unmultiplied(122, 59, 39, 170)
        } else {
            egui::Color32::from_rgba_unmultiplied(74, 56, 35, 135)
        };
        painter.rect(
            rect.shrink2(egui::vec2(1.0, 3.0)),
            5.0,
            fill,
            egui::Stroke::new(1.0, if selected { war_gold() } else { war_border() }),
            egui::StrokeKind::Inside,
        );
    }

    if depth > 0 {
        let trunk_x = rect.left() + indent - 18.0;
        let top = egui::pos2(trunk_x, rect.top() - 7.0);
        let bottom = egui::pos2(trunk_x, node_center.y);
        painter.line_segment(
            [top, bottom],
            egui::Stroke::new(1.0, tree_line_color(status)),
        );
        painter.line_segment(
            [bottom, egui::pos2(node_center.x - 14.0, node_center.y)],
            egui::Stroke::new(1.0, tree_line_color(status)),
        );
    }
    if has_parent {
        painter.circle_filled(
            egui::pos2(rect.left() + indent - 18.0, node_center.y),
            2.0,
            tree_line_color(status),
        );
    }

    painter.circle_filled(node_center, 14.0, visuals.fill);
    painter.circle_stroke(node_center, 14.0, egui::Stroke::new(1.2, visuals.stroke));
    painter.text(
        node_center,
        egui::Align2::CENTER_CENTER,
        technology_icon(spec),
        egui::FontId::proportional(18.0),
        visuals.icon_color,
    );

    let text_left = node_center.x + 23.0;
    painter.text(
        egui::pos2(text_left, rect.top() + 6.0),
        egui::Align2::LEFT_TOP,
        spec.name,
        egui::FontId::proportional(18.0),
        if status == TechnologyNodeStatus::Locked {
            war_text_muted()
        } else {
            egui::Color32::from_rgb(238, 225, 193)
        },
    );

    let progress = faction_state
        .map(|state| technology_progress(state, spec.id))
        .unwrap_or_default();
    let meta = if status == TechnologyNodeStatus::Active {
        format!("{progress}/{}回合  {}金", spec.turns, spec.gold_cost)
    } else {
        format!("{}回合 / {}金", spec.turns, spec.gold_cost)
    };
    painter.text(
        egui::pos2(text_left, rect.top() + 29.0),
        egui::Align2::LEFT_TOP,
        meta,
        egui::FontId::proportional(13.0),
        war_text_muted(),
    );

    let badge_text = visuals.label;
    let badge_width = (badge_text.chars().count() as f32 * 13.0 + 18.0).clamp(54.0, 90.0);
    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(rect.right() - badge_width - 8.0, rect.center().y - 13.0),
        egui::vec2(badge_width, 26.0),
    );
    painter.rect(
        badge_rect,
        4.0,
        visuals.badge_fill,
        egui::Stroke::new(1.0, visuals.stroke),
        egui::StrokeKind::Inside,
    );
    painter.text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        badge_text,
        egui::FontId::proportional(13.0),
        visuals.badge_text,
    );

    response
}

fn technology_detail(ui: &mut egui::Ui, ui_state: &mut GameUiState, game: &GameState) {
    let faction_id = game.player_faction_id.clone();
    let spec = technology_spec(ui_state.selected_technology_id);
    let faction_state = faction_technology_state(game, &faction_id);
    let progress = faction_state
        .map(|state| technology_progress(state, spec.id))
        .unwrap_or_default();
    let total_gold = faction_total_gold(game, &faction_id);
    let cost = effective_technology_cost(game, &faction_id, spec.id);
    let missing = missing_prerequisite_names(faction_state, spec.id);
    let is_completed = faction_state.is_some_and(|state| state.completed.contains(&spec.id));
    let is_funded = faction_state.is_some_and(|state| state.funded.contains(&spec.id));
    let is_active = faction_state.is_some_and(|state| state.active == Some(spec.id));

    ui.heading(egui::RichText::new(spec.name).color(war_gold()));
    ui.label(format!(
        "{}科技  ·  {} 回合  ·  立项 {} 金",
        technology_branch_label(spec.branch),
        spec.turns,
        cost
    ));
    if cost != spec.gold_cost {
        ui.colored_label(
            war_success(),
            format!("度支尚书减免后原价 {} 金", spec.gold_cost),
        );
    }
    ui.label(format!("当前势力金钱: {total_gold}"));
    ui.label(format!("研发进度: {progress}/{}", spec.turns));
    ui.separator();

    if spec.prerequisites.is_empty() {
        ui.label("前置: 无");
    } else if missing.is_empty() {
        ui.colored_label(
            war_success(),
            format!("前置: {}", prerequisite_names(spec).join("、")),
        );
    } else {
        ui.colored_label(war_warning(), format!("缺少前置: {}", missing.join("、")));
    }
    ui.add_space(6.0);
    ui.label("效果");
    ui.colored_label(war_text_muted(), spec.effect);
    ui.separator();

    if is_completed {
        ui.add_enabled(false, egui::Button::new("已完成"));
        return;
    }
    if is_active {
        ui.add_enabled(
            false,
            egui::Button::new(format!("研发中 {progress}/{}", spec.turns)),
        );
        return;
    }
    if !missing.is_empty() {
        ui.add_enabled(false, egui::Button::new("前置未完成"));
        return;
    }
    if is_funded {
        if ui.button("继续研发").clicked() {
            start_player_research(ui_state, spec.id);
        }
        return;
    }
    if total_gold < cost {
        ui.colored_label(war_danger(), format!("还缺 {} 金", cost - total_gold));
        ui.add_enabled(false, egui::Button::new("金钱不足，无法立项"));
        return;
    }
    if ui.button("立项研发").clicked() {
        start_player_research(ui_state, spec.id);
    }
}

fn start_player_research(ui_state: &mut GameUiState, technology_id: TechnologyId) {
    let Some(game) = ui_state.game.as_mut() else {
        ui_state.message = "尚未开始游戏".to_string();
        return;
    };
    let faction_id = game.player_faction_id.clone();
    let spec = technology_spec(technology_id);
    match start_research(game, &faction_id, technology_id) {
        Ok(outcome) if outcome.resumed => {
            ui_state.message = format!("继续研发 {}", spec.name);
        }
        Ok(outcome) => {
            ui_state.message = format!("已立项研发 {}，消耗 {} 金", spec.name, outcome.cost_paid);
        }
        Err(error) => ui_state.message = error.to_string(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TechnologyNodeStatus {
    Completed,
    Active,
    Funded,
    Available,
    Unaffordable,
    Locked,
}

#[derive(Clone, Copy, Debug)]
struct TechnologyNodeVisuals {
    label: &'static str,
    fill: egui::Color32,
    stroke: egui::Color32,
    icon_color: egui::Color32,
    badge_fill: egui::Color32,
    badge_text: egui::Color32,
}

fn technology_node_status(
    game: &GameState,
    faction_state: Option<&FactionTechnologyState>,
    spec: &TechnologySpec,
) -> TechnologyNodeStatus {
    if faction_state.is_some_and(|state| state.completed.contains(&spec.id)) {
        return TechnologyNodeStatus::Completed;
    }
    if faction_state.is_some_and(|state| state.active == Some(spec.id)) {
        return TechnologyNodeStatus::Active;
    }
    if faction_state.is_some_and(|state| state.funded.contains(&spec.id)) {
        return TechnologyNodeStatus::Funded;
    }
    if !missing_prerequisite_names(faction_state, spec.id).is_empty() {
        return TechnologyNodeStatus::Locked;
    }
    if faction_total_gold(game, &game.player_faction_id)
        >= effective_technology_cost(game, &game.player_faction_id, spec.id)
    {
        TechnologyNodeStatus::Available
    } else {
        TechnologyNodeStatus::Unaffordable
    }
}

fn technology_status_visuals(status: TechnologyNodeStatus) -> TechnologyNodeVisuals {
    match status {
        TechnologyNodeStatus::Completed => TechnologyNodeVisuals {
            label: "已完成",
            fill: egui::Color32::from_rgba_unmultiplied(39, 86, 51, 220),
            stroke: war_success(),
            icon_color: egui::Color32::from_rgb(226, 244, 218),
            badge_fill: egui::Color32::from_rgba_unmultiplied(45, 92, 55, 180),
            badge_text: egui::Color32::from_rgb(226, 244, 218),
        },
        TechnologyNodeStatus::Active => TechnologyNodeVisuals {
            label: "研发中",
            fill: egui::Color32::from_rgba_unmultiplied(122, 59, 39, 230),
            stroke: war_gold(),
            icon_color: egui::Color32::from_rgb(255, 235, 180),
            badge_fill: egui::Color32::from_rgba_unmultiplied(117, 65, 35, 200),
            badge_text: egui::Color32::from_rgb(255, 236, 190),
        },
        TechnologyNodeStatus::Funded => TechnologyNodeVisuals {
            label: "已付款",
            fill: egui::Color32::from_rgba_unmultiplied(72, 65, 39, 220),
            stroke: war_warning(),
            icon_color: egui::Color32::from_rgb(245, 216, 145),
            badge_fill: egui::Color32::from_rgba_unmultiplied(87, 67, 34, 180),
            badge_text: egui::Color32::from_rgb(246, 222, 160),
        },
        TechnologyNodeStatus::Available => TechnologyNodeVisuals {
            label: "可立项",
            fill: egui::Color32::from_rgba_unmultiplied(54, 48, 34, 220),
            stroke: war_gold(),
            icon_color: war_gold(),
            badge_fill: egui::Color32::from_rgba_unmultiplied(66, 50, 28, 170),
            badge_text: egui::Color32::from_rgb(247, 224, 173),
        },
        TechnologyNodeStatus::Unaffordable => TechnologyNodeVisuals {
            label: "金不足",
            fill: egui::Color32::from_rgba_unmultiplied(44, 39, 32, 190),
            stroke: war_warning(),
            icon_color: war_warning(),
            badge_fill: egui::Color32::from_rgba_unmultiplied(61, 45, 28, 150),
            badge_text: egui::Color32::from_rgb(236, 196, 122),
        },
        TechnologyNodeStatus::Locked => TechnologyNodeVisuals {
            label: "未解锁",
            fill: egui::Color32::from_rgba_unmultiplied(34, 31, 27, 170),
            stroke: egui::Color32::from_rgba_unmultiplied(118, 105, 81, 150),
            icon_color: war_text_muted(),
            badge_fill: egui::Color32::from_rgba_unmultiplied(42, 37, 31, 140),
            badge_text: war_text_muted(),
        },
    }
}

fn tree_line_color(status: TechnologyNodeStatus) -> egui::Color32 {
    match status {
        TechnologyNodeStatus::Completed => {
            egui::Color32::from_rgba_unmultiplied(118, 186, 122, 150)
        }
        TechnologyNodeStatus::Active | TechnologyNodeStatus::Available => {
            egui::Color32::from_rgba_unmultiplied(215, 162, 72, 155)
        }
        TechnologyNodeStatus::Funded | TechnologyNodeStatus::Unaffordable => {
            egui::Color32::from_rgba_unmultiplied(218, 174, 88, 130)
        }
        TechnologyNodeStatus::Locked => egui::Color32::from_rgba_unmultiplied(118, 105, 81, 92),
    }
}

fn technology_depth(spec: &TechnologySpec) -> usize {
    spec.prerequisites
        .iter()
        .map(|id| technology_depth(technology_spec(*id)) + 1)
        .max()
        .unwrap_or_default()
}

fn technology_icon(spec: &TechnologySpec) -> &'static str {
    match spec.id {
        TechnologyId::MilitiaDrill => egui_phosphor::regular::USERS,
        TechnologyId::ArsenalLogistics => egui_phosphor::regular::STACK,
        TechnologyId::ScoutRoads => egui_phosphor::regular::MAP_TRIFOLD,
        TechnologyId::IronWeapons => egui_phosphor::regular::SWORD,
        TechnologyId::StrictDiscipline => egui_phosphor::regular::FLAG,
        TechnologyId::FortifiedGarrisons => egui_phosphor::regular::SHIELD,
        TechnologyId::SupplyEscort => egui_phosphor::regular::BARN,
        TechnologyId::CombinedArms => egui_phosphor::regular::ARROWS_OUT_CARDINAL,
        TechnologyId::GateFireTactics => egui_phosphor::regular::FIRE,
        TechnologyId::RotatingDefense => egui_phosphor::regular::ARROWS_CLOCKWISE,
        TechnologyId::MilitaryGranaries => egui_phosphor::regular::WAREHOUSE,
        TechnologyId::SiegeEngines => egui_phosphor::regular::HAMMER,
        TechnologyId::OfficerMerit => egui_phosphor::regular::MEDAL,
        TechnologyId::GrandCommandery => egui_phosphor::regular::CROWN,
        TechnologyId::HouseholdRegisters => egui_phosphor::regular::IDENTIFICATION_CARD,
        TechnologyId::IrrigationSurvey => egui_phosphor::regular::DROP,
        TechnologyId::MarketRegisters => egui_phosphor::regular::COINS,
        TechnologyId::GranarySystem => egui_phosphor::regular::BARN,
        TechnologyId::PriceStabilization => egui_phosphor::regular::SCALES,
        TechnologyId::ArtisanRegisters => egui_phosphor::regular::HAMMER,
        TechnologyId::CanalRestoration => egui_phosphor::regular::WAVES,
        TechnologyId::TradePasses => egui_phosphor::regular::BRIDGE,
        TechnologyId::BureaucraticRecords => egui_phosphor::regular::SCROLL,
        TechnologyId::EverNormalGranary => egui_phosphor::regular::WAREHOUSE,
        TechnologyId::WorkshopGuilds => egui_phosphor::regular::GEAR,
        TechnologyId::CommanderyReviews => egui_phosphor::regular::CLIPBOARD_TEXT,
        TechnologyId::CanalTaxation => egui_phosphor::regular::BANK,
        TechnologyId::MinistryOfFinance => egui_phosphor::regular::SEAL_CHECK,
    }
}

fn prerequisite_names(spec: &TechnologySpec) -> Vec<&'static str> {
    spec.prerequisites
        .iter()
        .map(|id| technology_spec(*id).name)
        .collect()
}

pub(super) fn city_list_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    if !ui_state.city_list_open {
        return;
    }
    let max_height = (screen.height() - HUD_TOP_HEIGHT - 170.0).clamp(240.0, 520.0);
    egui::Area::new(egui::Id::new("hud_city_list"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 338.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(285.0);
                ui.set_max_height(max_height);
                city_list(ui, ui_state);
            });
        });
}

pub(super) fn save_hud(ctx: &egui::Context, ui_state: &mut GameUiState, _screen: egui::Rect) {
    if !ui_state.save_panel_open {
        return;
    }
    egui::Area::new(egui::Id::new("hud_save_panel"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_TOP,
            egui::vec2(-HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(330.0);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("存档").color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("收起").clicked() {
                            ui_state.save_panel_open = false;
                        }
                    });
                });
                save_controls(ui, ui_state);
            });
        });
}

pub(super) fn city_drawer_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    if !ui_state.city_drawer_open {
        return;
    }
    let modal_width = (screen.width() - HUD_MARGIN * 2.0).clamp(760.0, 1120.0);
    let modal_height = (screen.height() - HUD_MARGIN * 2.0).clamp(520.0, 760.0);
    egui::Area::new(egui::Id::new("hud_city_drawer"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(modal_width);
                ui.set_min_height(modal_height);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("中军帐").color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("收起").clicked() {
                            ui_state.city_drawer_open = false;
                        }
                    });
                });
                ui.separator();
                ui.allocate_ui_with_layout(
                    egui::vec2(modal_width, modal_height - 54.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        selected_city_panel(ui, ui_state);
                    },
                );
            });
        });
}

pub(super) fn report_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    let width = screen.width() * 0.5;
    egui::Area::new(egui::Id::new("hud_report_panel"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::LEFT_BOTTOM,
            egui::vec2(HUD_MARGIN, -HUD_MARGIN),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("回合报告").color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(if ui_state.reports_open {
                                "收起"
                            } else {
                                "展开"
                            })
                            .clicked()
                        {
                            ui_state.reports_open = !ui_state.reports_open;
                        }
                    });
                });
                if ui_state.reports_open {
                    ui.separator();
                    report_panel(ui, ui_state, screen);
                } else if !ui_state.message.is_empty() {
                    ui.label(&ui_state.message);
                }
            });
        });
}

pub(super) fn officer_browser_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    screen: egui::Rect,
) {
    if !ui_state.officer_browser_open {
        return;
    }

    let width = (screen.width() * 0.82).clamp(720.0, 1080.0);
    let height = (screen.height() * 0.76).clamp(420.0, 680.0);
    egui::Area::new(egui::Id::new("hud_officer_browser"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, "武将") {
                    ui_state.officer_browser_open = false;
                }
                ui.separator();
                if let Some(game) = &ui_state.game {
                    officer_browser_filters(
                        ui,
                        game,
                        &mut ui_state.officer_browser_filters,
                        "hud_officer_browser_filters",
                    );
                } else {
                    ui.label("当前没有剧本局面。");
                }
                ui.separator();
                if let Some(game) = &ui_state.game {
                    officer_browser_table(
                        ui,
                        game,
                        &ui_state.officer_browser_filters,
                        OfficerBrowserTableOptions {
                            max_height: height - 118.0,
                            id_salt: "hud_officer_browser_table",
                            selected_officer_id: None,
                            editable: false,
                            retainer_faction_id: None,
                        },
                    );
                }
            });
        });
}

pub(super) fn retainer_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    if !ui_state.retainers_open {
        return;
    }

    let width = (screen.width() * 0.82).clamp(760.0, 1120.0);
    let height = (screen.height() * 0.76).clamp(420.0, 680.0);
    egui::Area::new(egui::Id::new("hud_retainers"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, "幕僚") {
                    ui_state.retainers_open = false;
                }
                ui.separator();
                if let Some(game) = &ui_state.game {
                    officer_browser_filters(
                        ui,
                        game,
                        &mut ui_state.retainer_filters,
                        "hud_retainer_filters",
                    );
                } else {
                    ui.label("当前没有剧本局面。");
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
                        selected_officer_id: None,
                        editable: false,
                        retainer_faction_id: Some(player_faction_id.as_str()),
                    },
                );

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
                                ui_state.message = format!(
                                    "{officer_name} 被任命为 {office_name}，忠诚 {loyalty}"
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
                                ui_state.message = format!("{officer_name} 已免官，忠诚 {loyalty}");
                            }
                            Err(error) => ui_state.message = error.to_string(),
                        }
                    }
                }
            });
        });
}

pub(super) fn selected_city_summary(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let summary = ui_state.game.as_ref().and_then(|game| {
        let city = game.cities.get(ui_state.selected_city_id.as_deref()?)?;
        let faction_name = game
            .factions
            .get(&city.faction_id)
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| "未知".to_string());
        Some((city.id.clone(), city.clone(), faction_name))
    });

    let Some((city_id, city, faction_name)) = summary else {
        ui.label("未选择城池");
        return;
    };

    city_summary_intel(ui, &city, &faction_name);
    ui.add_space(8.0);
    if ui.button("打开中军帐").clicked() {
        open_city(ui_state, city_id);
    }
}

pub(super) fn map_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.horizontal(|ui| {
        ui.heading(egui::RichText::new("地图").color(war_gold()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_enabled(
                ui_state.map_boundaries.is_some(),
                egui::Checkbox::new(&mut ui_state.map_boundaries_enabled, "州郡边界"),
            );
        });
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button("-").clicked() {
            zoom_map(ui_state, 1.0 / MAP_ZOOM_STEP, None, None);
        }
        ui.add_sized(
            [58.0, 28.0],
            egui::Label::new(format!("{:.0}%", ui_state.map_zoom * 100.0)),
        );
        if ui.button("+").clicked() {
            zoom_map(ui_state, MAP_ZOOM_STEP, None, None);
        }
        if ui.button("重置").clicked() {
            reset_map_view(ui_state);
        }
    });
    if ui_state.map_boundaries.is_none() {
        ui.colored_label(war_text_muted(), "边界资产未加载");
    }
}

pub(super) fn city_list(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let Some(game) = &ui_state.game else {
        return;
    };
    let mut rows: Vec<_> = game
        .cities
        .values()
        .map(|city| {
            let faction_name = game
                .factions
                .get(&city.faction_id)
                .map(|faction| faction.name.clone())
                .unwrap_or_else(|| "未知".to_string());
            (city.id.clone(), city.name.clone(), faction_name)
        })
        .collect();
    rows.sort_by(|a, b| a.1.cmp(&b.1));

    ui.heading("城池");
    egui::ScrollArea::vertical()
        .id_salt("city_list")
        .max_height(460.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (city_id, city_name, faction_name) in rows {
                let selected = ui_state.selected_city_id.as_deref() == Some(city_id.as_str());
                let response =
                    ui.selectable_label(selected, format!("{} ({})", city_name, faction_name));
                if response.clicked() {
                    ui_state.selected_city_id = Some(city_id.clone());
                }
                if response.secondary_clicked() {
                    ui_state.selected_city_id = Some(city_id.clone());
                }
                response.context_menu(|ui| {
                    if ui.button("打开中军帐").clicked() {
                        open_city(ui_state, city_id.clone());
                        ui.close();
                    }
                });
            }
        });
}

pub(super) fn officer_browser_filters(
    ui: &mut egui::Ui,
    game: &GameState,
    filters: &mut OfficerBrowserFilters,
    id_salt: &'static str,
) {
    const FILTER_HEIGHT: f32 = 30.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().interact_size.y = FILTER_HEIGHT;
        ui.spacing_mut().item_spacing.x = 12.0;
        ui.label("搜索");
        ui.add_sized(
            [260.0, FILTER_HEIGHT],
            egui::TextEdit::singleline(&mut filters.search).hint_text("姓名 / ID / 字 / 籍贯"),
        );

        egui::ComboBox::from_id_salt((id_salt, "gender"))
            .width(160.0)
            .selected_text(officer_gender_filter_label(filters.gender))
            .show_ui(ui, |ui| {
                for filter in [
                    OfficerGenderFilter::All,
                    OfficerGenderFilter::Male,
                    OfficerGenderFilter::Female,
                ] {
                    ui.selectable_value(
                        &mut filters.gender,
                        filter,
                        officer_gender_filter_label(filter),
                    );
                }
            });

        egui::ComboBox::from_id_salt((id_salt, "faction"))
            .width(170.0)
            .selected_text(
                filters
                    .faction_id
                    .as_deref()
                    .and_then(|id| game.factions.get(id))
                    .map(|faction| faction.name.as_str())
                    .unwrap_or("全部势力"),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut filters.faction_id, None, "全部势力");
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
            .selected_text(officer_status_filter_label(filters.status))
            .show_ui(ui, |ui| {
                for filter in [
                    OfficerStatusFilter::All,
                    OfficerStatusFilter::Active,
                    OfficerStatusFilter::Wild,
                    OfficerStatusFilter::Unavailable,
                    OfficerStatusFilter::Dead,
                ] {
                    ui.selectable_value(
                        &mut filters.status,
                        filter,
                        officer_status_filter_label(filter),
                    );
                }
            });

        egui::ComboBox::from_id_salt((id_salt, "city"))
            .width(170.0)
            .selected_text(
                filters
                    .city_id
                    .as_deref()
                    .and_then(|id| game.cities.get(id))
                    .map(|city| city.name.as_str())
                    .unwrap_or("全部城池"),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut filters.city_id, None, "全部城池");
                let mut cities: Vec<_> = game.cities.values().collect();
                cities.sort_by(|a, b| a.name.cmp(&b.name));
                for city in cities {
                    ui.selectable_value(&mut filters.city_id, Some(city.id.clone()), &city.name);
                }
            });

        if ui
            .add_sized([86.0, FILTER_HEIGHT], egui::Button::new("重置"))
            .clicked()
        {
            filters.reset();
        }
    });
}

pub(super) fn officer_browser_table(
    ui: &mut egui::Ui,
    game: &GameState,
    filters: &OfficerBrowserFilters,
    options: OfficerBrowserTableOptions<'_>,
) -> OfficerBrowserTableResponse {
    let rows = options.retainer_faction_id.map_or_else(
        || filtered_officer_rows(filters, game),
        |faction_id| retainer_officer_rows(filters, game, faction_id),
    );
    let mut table_response = OfficerBrowserTableResponse::default();
    ui.label(format!("共 {} 名武将", rows.len()));
    egui::ScrollArea::vertical()
        .id_salt(options.id_salt)
        .max_height(options.max_height.max(260.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new((options.id_salt, "grid"))
                .striped(true)
                .num_columns(13)
                .min_col_width(42.0)
                .spacing(egui::vec2(12.0, 6.0))
                .show(ui, |ui| {
                    ui.strong("姓名");
                    ui.strong("性别");
                    ui.strong("势力");
                    ui.strong("城池");
                    ui.strong("官职");
                    ui.strong("俸禄");
                    ui.strong("状态");
                    ui.strong("忠诚");
                    ui.strong("统");
                    ui.strong("武");
                    ui.strong("智");
                    ui.strong("政");
                    ui.strong("魅");
                    ui.end_row();

                    for row in rows {
                        let selected = options.selected_officer_id == Some(row.id.as_str());
                        let mut cell_context = OfficerRowCellContext {
                            ui,
                            game,
                            table_response: &mut table_response,
                            editable: options.editable,
                            retainer_faction_id: options.retainer_faction_id,
                        };
                        officer_row_cell(&mut cell_context, &row, selected, &row.name);
                        officer_row_cell(&mut cell_context, &row, selected, row.gender);
                        officer_row_cell(&mut cell_context, &row, selected, &row.faction_name);
                        officer_row_cell(&mut cell_context, &row, selected, &row.city_name);
                        officer_row_cell(&mut cell_context, &row, selected, &row.office_name);
                        officer_row_cell(&mut cell_context, &row, selected, row.salary.to_string());
                        officer_row_cell(&mut cell_context, &row, selected, row.status);
                        officer_row_cell(
                            &mut cell_context,
                            &row,
                            selected,
                            row.loyalty.to_string(),
                        );
                        officer_row_cell(
                            &mut cell_context,
                            &row,
                            selected,
                            row.leadership.to_string(),
                        );
                        officer_row_cell(
                            &mut cell_context,
                            &row,
                            selected,
                            row.strength.to_string(),
                        );
                        officer_row_cell(
                            &mut cell_context,
                            &row,
                            selected,
                            row.intelligence.to_string(),
                        );
                        officer_row_cell(
                            &mut cell_context,
                            &row,
                            selected,
                            row.politics.to_string(),
                        );
                        officer_row_cell(&mut cell_context, &row, selected, row.charm.to_string());
                        ui.end_row();
                    }
                });
        });
    table_response
}

#[derive(Clone, Copy, Debug)]
pub(super) struct OfficerBrowserTableOptions<'a> {
    pub(super) max_height: f32,
    pub(super) id_salt: &'static str,
    pub(super) selected_officer_id: Option<&'a str>,
    pub(super) editable: bool,
    pub(super) retainer_faction_id: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub(super) struct OfficerBrowserRow {
    id: OfficerId,
    name: String,
    faction_id: FactionId,
    gender: &'static str,
    faction_name: String,
    city_name: String,
    office_name: String,
    salary: i32,
    status: &'static str,
    loyalty: u8,
    leadership: u8,
    strength: u8,
    intelligence: u8,
    politics: u8,
    charm: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct OfficerBrowserTableResponse {
    pub(super) selected_officer_id: Option<OfficerId>,
    pub(super) edit_officer_id: Option<OfficerId>,
    pub(super) appoint_officer_id: Option<OfficerId>,
    pub(super) appoint_office_id: Option<OfficialPostId>,
    pub(super) dismiss_officer_id: Option<OfficerId>,
}

fn officer_row_cell(
    context: &mut OfficerRowCellContext<'_, '_>,
    row: &OfficerBrowserRow,
    selected: bool,
    text: impl Into<egui::WidgetText>,
) {
    let response = context.ui.selectable_label(selected, text);
    if response.clicked() || response.secondary_clicked() {
        context.table_response.selected_officer_id = Some(row.id.clone());
    }
    if context.editable || context.retainer_faction_id.is_some() {
        response.context_menu(|ui| {
            if context.editable && ui.button("编辑").clicked() {
                context.table_response.selected_officer_id = Some(row.id.clone());
                context.table_response.edit_officer_id = Some(row.id.clone());
                ui.close();
            }
            if context.retainer_faction_id.is_some() {
                ui.menu_button("授官", |ui| {
                    for spec in official_post_specs() {
                        let occupant = context.game.officers.values().find(|officer| {
                            officer.faction_id == row.faction_id
                                && officer.office_id.as_deref() == Some(spec.id)
                        });
                        let label = if let Some(occupant) = occupant {
                            if occupant.id == row.id {
                                format!("{} ({}, 当前)", spec.name, official_rank_label(spec.rank))
                            } else {
                                format!(
                                    "{} ({}, 现任: {})",
                                    spec.name,
                                    official_rank_label(spec.rank),
                                    occupant.name
                                )
                            }
                        } else {
                            format!("{} ({})", spec.name, official_rank_label(spec.rank))
                        };
                        if ui.button(label).clicked() {
                            context.table_response.selected_officer_id = Some(row.id.clone());
                            context.table_response.appoint_officer_id = Some(row.id.clone());
                            context.table_response.appoint_office_id = Some(spec.id.to_string());
                            ui.close();
                        }
                    }
                });
                if row.office_name != "无" && ui.button("免官").clicked() {
                    context.table_response.selected_officer_id = Some(row.id.clone());
                    context.table_response.dismiss_officer_id = Some(row.id.clone());
                    ui.close();
                }
            }
        });
    }
}

struct OfficerRowCellContext<'ui, 'data> {
    ui: &'ui mut egui::Ui,
    game: &'data GameState,
    table_response: &'ui mut OfficerBrowserTableResponse,
    editable: bool,
    retainer_faction_id: Option<&'data str>,
}

pub(super) fn filtered_officer_rows(
    filters: &OfficerBrowserFilters,
    game: &GameState,
) -> Vec<OfficerBrowserRow> {
    let search = filters.search.trim().to_lowercase();
    let mut officers: Vec<_> = game.officers.values().collect();
    officers.sort_by(|a, b| {
        let a_faction = game
            .factions
            .get(&a.faction_id)
            .map(|faction| faction.name.as_str())
            .unwrap_or("未知");
        let b_faction = game
            .factions
            .get(&b.faction_id)
            .map(|faction| faction.name.as_str())
            .unwrap_or("未知");
        let a_city = a
            .city_id
            .as_deref()
            .and_then(|city_id| game.cities.get(city_id))
            .map(|city| city.name.as_str())
            .unwrap_or("未配置");
        let b_city = b
            .city_id
            .as_deref()
            .and_then(|city_id| game.cities.get(city_id))
            .map(|city| city.name.as_str())
            .unwrap_or("未配置");
        (a_faction, a_city, a.name.as_str(), a.id.as_str()).cmp(&(
            b_faction,
            b_city,
            b.name.as_str(),
            b.id.as_str(),
        ))
    });

    officers
        .into_iter()
        .filter(|officer| officer_matches_filters(officer, game, filters, &search))
        .map(|officer| {
            let faction_name = game
                .factions
                .get(&officer.faction_id)
                .map(|faction| faction.name.clone())
                .unwrap_or_else(|| "未知".to_string());
            let city_name = officer
                .city_id
                .as_deref()
                .and_then(|city_id| game.cities.get(city_id))
                .map(|city| city.name.clone())
                .unwrap_or_else(|| "未配置".to_string());
            let office_name = officer
                .office_id
                .as_deref()
                .and_then(official_post_spec)
                .map(|spec| spec.name.to_string())
                .unwrap_or_else(|| "无".to_string());
            OfficerBrowserRow {
                id: officer.id.clone(),
                name: officer.name.clone(),
                faction_id: officer.faction_id.clone(),
                gender: officer_gender_label(&officer.gender),
                faction_name,
                city_name,
                office_name,
                salary: officer_monthly_salary(officer),
                status: officer_status_label(&officer.status),
                loyalty: officer.loyalty,
                leadership: officer.stats.leadership,
                strength: officer.stats.strength,
                intelligence: officer.stats.intelligence,
                politics: officer.stats.politics,
                charm: officer.stats.charm,
            }
        })
        .collect()
}

pub(super) fn retainer_officer_rows(
    filters: &OfficerBrowserFilters,
    game: &GameState,
    faction_id: &str,
) -> Vec<OfficerBrowserRow> {
    let mut locked_filters = filters.clone();
    locked_filters.faction_id = Some(faction_id.to_string());
    locked_filters.status = OfficerStatusFilter::Active;
    filtered_officer_rows(&locked_filters, game)
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
    search.is_empty() || officer_search_text(officer, game).contains(search)
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
        OfficerStatusFilter::Wild => *status == OfficerStatus::Wild,
        OfficerStatusFilter::Unavailable => *status == OfficerStatus::Unavailable,
        OfficerStatusFilter::Dead => *status == OfficerStatus::Dead,
    }
}

fn officer_gender_filter_label(filter: OfficerGenderFilter) -> &'static str {
    match filter {
        OfficerGenderFilter::All => "全部性别",
        OfficerGenderFilter::Male => "男",
        OfficerGenderFilter::Female => "女",
    }
}

fn officer_status_filter_label(filter: OfficerStatusFilter) -> &'static str {
    match filter {
        OfficerStatusFilter::All => "全部状态",
        OfficerStatusFilter::Active => "在任",
        OfficerStatusFilter::Wild => "在野",
        OfficerStatusFilter::Unavailable => "不可用",
        OfficerStatusFilter::Dead => "死亡",
    }
}

fn officer_status_label(status: &OfficerStatus) -> &'static str {
    match status {
        OfficerStatus::Active => "在任",
        OfficerStatus::Wild => "在野",
        OfficerStatus::Unavailable => "不可用",
        OfficerStatus::Dead => "死亡",
    }
}

pub(super) fn save_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.horizontal(|ui| {
        ui.label("槽位");
        egui::ComboBox::from_id_salt("save_slot_combo")
            .selected_text(&ui_state.save_slot_id)
            .show_ui(ui, |ui| {
                for slot_id in ["slot1", "slot2", "slot3", "slot4", "slot5"] {
                    ui.selectable_value(&mut ui_state.save_slot_id, slot_id.to_string(), slot_id);
                }
            });
    });
    ui.horizontal(|ui| {
        ui.label("名称");
        ui.text_edit_singleline(&mut ui_state.save_display_name);
    });
    ui.horizontal(|ui| {
        if ui.button("保存").clicked()
            && let Some(game) = &ui_state.game
        {
            match ui_state.save_manager.save_slot(
                &ui_state.save_slot_id,
                &ui_state.save_display_name,
                game,
            ) {
                Ok(meta) => {
                    refresh_saves(ui_state);
                    ui_state.message = format!("保存到 {}", meta.display_name);
                }
                Err(error) => ui_state.message = error.to_string(),
            }
        }
        if ui.button("读取当前槽").clicked() {
            match ui_state.save_manager.load_slot(&ui_state.save_slot_id) {
                Ok(game) => {
                    enter_game(ui_state, game, "读取当前槽位".to_string());
                }
                Err(error) => {
                    let slot_id = ui_state.save_slot_id.clone();
                    let _ = ui_state.save_manager.delete_slot(&slot_id);
                    refresh_saves(ui_state);
                    ui_state.message = format!("存档已失效，已丢弃: {error}");
                }
            }
        }
    });
    if !ui_state.message.is_empty() {
        ui.label(&ui_state.message);
    }
}

pub(super) fn report_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState, screen: egui::Rect) {
    let Some(game) = &ui_state.game else {
        return;
    };
    let report_count = game.reports.len();
    let visible_start = report_count.saturating_sub(12);
    let report_height = (screen.height() * 0.32).clamp(220.0, 340.0);
    ui.set_min_height(report_height);
    egui::ScrollArea::vertical()
        .id_salt("turn_report_scroll")
        .max_height(report_height)
        .min_scrolled_height(report_height)
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if !ui_state.message.is_empty() {
                ui.colored_label(war_gold(), &ui_state.message);
                ui.separator();
            }
            if game.reports.is_empty() {
                ui.label("暂无报告");
            }
            for report in game.reports.iter().skip(visible_start) {
                ui.label(format!(
                    "{}年{}月 第{}回合",
                    report.year, report.month, report.turn
                ));
                for entry in &report.entries {
                    match entry.severity {
                        ReportSeverity::Info => {
                            ui.label(format!("· {}", entry.message));
                        }
                        ReportSeverity::Warning => {
                            ui.colored_label(egui::Color32::YELLOW, format!("! {}", entry.message));
                        }
                    }
                }
                ui.separator();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::display_settings::{DisplaySettings, LoadedDisplaySettings};
    use crate::core::state::{GameUiState, OfficerGenderFilter, OfficerStatusFilter};
    use crate::game::{OfficerGender, OfficerStatus, ScenarioData};

    fn ui_state_with_game() -> GameUiState {
        let mut state = GameUiState::new(
            crate::core::display_settings::DisplaySettingsStore::with_default_path(),
            LoadedDisplaySettings {
                settings: DisplaySettings::default(),
                message: None,
            },
        );
        state.game = Some(
            ScenarioData::default_scenario()
                .unwrap()
                .build_game("liu_bei")
                .unwrap(),
        );
        state
    }

    #[test]
    fn officer_browser_search_matches_name_id_faction_and_city() {
        let mut state = ui_state_with_game();
        let game = state.game.as_ref().unwrap();

        state.officer_browser_filters.search = "关羽".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game)
                .iter()
                .any(|row| row.name == "关羽")
        );

        state.officer_browser_filters.search = "guan_yu".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game)
                .iter()
                .any(|row| row.id == "guan_yu" && row.name == "关羽")
        );

        state.officer_browser_filters.search = "刘备军".to_string();
        assert!(!filtered_officer_rows(&state.officer_browser_filters, game).is_empty());

        state.officer_browser_filters.search = "平原".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game)
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
        let rows = filtered_officer_rows(&state.officer_browser_filters, game);

        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| {
            row.gender == "男"
                && row.faction_name == "刘备军"
                && row.status == "在任"
                && row.city_name == "平原"
        }));
    }

    #[test]
    fn officer_browser_empty_filters_return_all_officers_stably_sorted() {
        let state = ui_state_with_game();
        let game = state.game.as_ref().unwrap();

        let rows = filtered_officer_rows(&state.officer_browser_filters, game);
        let sorted_names = rows.windows(2).all(|pair| {
            (&pair[0].faction_name, &pair[0].city_name, &pair[0].name)
                <= (&pair[1].faction_name, &pair[1].city_name, &pair[1].name)
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
        let rows = filtered_officer_rows(&state.officer_browser_filters, game);

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
        let rows = filtered_officer_rows(&state.officer_browser_filters, game);

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
        let rows = retainer_officer_rows(&state.retainer_filters, game, &game.player_faction_id);

        assert!(!rows.is_empty());
        assert!(
            rows.iter()
                .all(|row| row.faction_id == game.player_faction_id && row.status == "在任")
        );
        assert!(rows.iter().all(|row| row.id != "cao_cao"));
        assert!(rows.iter().all(|row| row.id != "jian_yong"));
    }
}
