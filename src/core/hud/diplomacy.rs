use crate::game::*;
use bevy_egui::egui;

use super::super::HUD_MARGIN;
use super::super::i18n::{Translator, args};
use super::super::labels::diplomacy_action_label;
use super::super::map::faction_color;
use super::super::state::GameUiState;
use super::super::style::{
    modal_title_bar, war_border, war_danger, war_gold, war_panel_frame, war_sub_panel_frame,
    war_success, war_text_muted, war_warning,
};

pub(super) fn diplomacy_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.diplomacy_open {
        return;
    }

    let Some(game) = ui_state.game.as_ref().cloned() else {
        return;
    };
    ensure_diplomacy_defaults(ui_state, &game);

    let width = (screen.width() * 0.84)
        .clamp(820.0, 1160.0)
        .min((screen.width() - HUD_MARGIN * 2.0).max(360.0));
    let height = (screen.height() * 0.78)
        .clamp(520.0, 740.0)
        .min((screen.height() - HUD_MARGIN * 2.0).max(360.0));

    egui::Area::new(egui::Id::new("hud_diplomacy"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("diplomacy-title")) {
                    ui_state.diplomacy_open = false;
                    return;
                }
                ui.separator();
                diplomacy_panel(ui, ui_state, &game, t, width, height);
            });
        });
}

fn diplomacy_panel(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    t: &Translator,
    width: f32,
    height: f32,
) {
    let body_height = (height - 150.0).max(320.0);
    ui.columns(3, |columns| {
        columns[0].set_width((width * 0.28).clamp(230.0, 310.0));
        columns[1].set_width((width * 0.30).clamp(250.0, 350.0));
        target_list_panel(&mut columns[0], ui_state, game, t, body_height);
        action_panel(&mut columns[1], ui_state, game, t, body_height);
        terms_panel(&mut columns[2], ui_state, game, t, body_height);
    });
    ui.add_space(8.0);
    pending_diplomacy_panel(ui, ui_state, game, t, width);
}

fn target_list_panel(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    t: &Translator,
    max_height: f32,
) {
    war_sub_panel_frame().show(ui, |ui| {
        panel_title(
            ui,
            egui_phosphor::regular::FLAG,
            &t.text("diplomacy-targets"),
        );
        let targets = diplomacy_targets(game);
        if targets.is_empty() {
            ui.colored_label(war_text_muted(), t.text("command-diplomacy-no-targets"));
            return;
        }
        egui::ScrollArea::vertical()
            .id_salt("diplomacy_targets")
            .max_height(max_height - 34.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for faction in targets {
                    target_row(ui, ui_state, game, faction, t);
                    ui.add_space(4.0);
                }
            });
    });
}

fn target_row(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    faction: &Faction,
    t: &Translator,
) {
    let selected = ui_state.selected_diplomacy_target.as_deref() == Some(faction.id.as_str());
    let relation = game.relation(&game.player_faction_id, &faction.id);
    let relation_score = relation.map(|relation| relation.score).unwrap_or_default();
    let hostile = relation.is_some_and(|relation| relation.is_hostile());
    let truce = relation.is_some_and(|relation| relation.has_active_truce(game.turn));
    let passage = relation
        .is_some_and(|relation| relation.has_passage_right(&game.player_faction_id, game.turn));
    let cities = game.cities_for_faction(&faction.id).len();
    let troops: u32 = game
        .cities
        .values()
        .filter(|city| city.faction_id == faction.id)
        .map(|city| city.troops.total())
        .sum();
    let status = if hostile {
        t.text("diplomacy-status-hostile")
    } else {
        match (truce, passage) {
            (true, true) => t.text("diplomacy-status-truce-passage"),
            (true, false) => t.text("diplomacy-status-truce"),
            (false, true) => t.text("diplomacy-status-passage"),
            (false, false) => t.text("diplomacy-status-open"),
        }
    };

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(180.0), 56.0),
        egui::Sense::click(),
    );
    if ui.is_rect_visible(rect) {
        let fill = if selected {
            egui::Color32::from_rgba_unmultiplied(113, 80, 42, 220)
        } else if response.hovered() {
            egui::Color32::from_rgba_unmultiplied(52, 42, 29, 220)
        } else {
            egui::Color32::from_rgba_unmultiplied(24, 21, 16, 96)
        };
        let stroke = egui::Stroke::new(
            if selected { 1.6 } else { 1.0 },
            if selected { war_gold() } else { war_border() },
        );
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect(
            rect.shrink(1.0),
            4.0,
            fill,
            stroke,
            egui::StrokeKind::Inside,
        );
        painter.rect_filled(
            egui::Rect::from_min_size(
                rect.left_top() + egui::vec2(7.0, 8.0),
                egui::vec2(5.0, 40.0),
            ),
            2.0,
            faction_color(faction),
        );

        let text_left = rect.left() + 22.0;
        painter.text(
            egui::pos2(text_left, rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            faction.name.as_str(),
            egui::FontId::proportional(15.0),
            war_gold(),
        );
        painter.text(
            egui::pos2(text_left, rect.top() + 30.0),
            egui::Align2::LEFT_TOP,
            format!(
                "{} / {} / {} {} / {}",
                t.text_args(
                    "faction-detail-header-cities",
                    &args([("count", cities.to_string())])
                ),
                t.text_args(
                    "diplomacy-relation-score",
                    &args([("score", relation_score.to_string())])
                ),
                t.text("resource-troops"),
                troops,
                status
            ),
            egui::FontId::proportional(12.0),
            if truce || passage {
                war_success()
            } else {
                war_text_muted()
            },
        );
    }

    if response.clicked() {
        ui_state.selected_diplomacy_target = Some(faction.id.clone());
    }
}

fn action_panel(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    t: &Translator,
    max_height: f32,
) {
    war_sub_panel_frame().show(ui, |ui| {
        panel_title(
            ui,
            egui_phosphor::regular::SCALES,
            &t.text("diplomacy-proposal"),
        );
        ui.horizontal_wrapped(|ui| {
            for action in DiplomacyActionKind::ALL {
                let selected = ui_state.selected_diplomacy_action == action;
                if ui
                    .selectable_label(selected, diplomacy_action_label(t, action))
                    .clicked()
                {
                    ui_state.selected_diplomacy_action = action;
                    normalize_diplomacy_terms(ui_state);
                }
            }
        });
        ui.separator();
        selected_relation_summary(ui, ui_state, game, t);
        ui.add_space(8.0);
        proposal_preview(ui, ui_state, game, t, max_height);
    });
}

fn selected_relation_summary(
    ui: &mut egui::Ui,
    ui_state: &GameUiState,
    game: &GameState,
    t: &Translator,
) {
    let Some(target_id) = ui_state.selected_diplomacy_target.as_deref() else {
        ui.colored_label(war_text_muted(), t.text("command-diplomacy-no-targets"));
        return;
    };
    let relation = game.relation(&game.player_faction_id, target_id);
    let score = relation.map(|relation| relation.score).unwrap_or_default();
    metric_line(
        ui,
        egui_phosphor::regular::HEART,
        &t.text("diplomacy-relation"),
        score.to_string(),
        relation_score_color(score),
    );
    let truce_label = relation
        .and_then(|relation| relation.truce_until_turn)
        .filter(|until| *until >= game.turn)
        .map(|until| {
            t.text_args(
                "faction-detail-truce-until",
                &args([("turn", until.to_string())]),
            )
        })
        .unwrap_or_else(|| t.text("faction-detail-no-truce"));
    metric_line(
        ui,
        egui_phosphor::regular::SCROLL,
        &t.text("diplomacy-truce-status"),
        truce_label,
        war_text_muted(),
    );
    let passage_label = relation
        .and_then(|relation| {
            relation
                .passage_rights
                .get(&game.player_faction_id)
                .copied()
        })
        .filter(|until| *until >= game.turn)
        .map(|until| {
            t.text_args(
                "diplomacy-passage-until",
                &args([("turn", until.to_string())]),
            )
        })
        .unwrap_or_else(|| t.text("diplomacy-no-passage"));
    metric_line(
        ui,
        egui_phosphor::regular::MAP_TRIFOLD,
        &t.text("diplomacy-passage-status"),
        passage_label,
        war_text_muted(),
    );
}

fn proposal_preview(
    ui: &mut egui::Ui,
    ui_state: &GameUiState,
    game: &GameState,
    t: &Translator,
    max_height: f32,
) {
    egui::ScrollArea::vertical()
        .id_salt("diplomacy_preview")
        .max_height((max_height - 160.0).max(120.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let Some(order) = candidate_order(ui_state, game) else {
                ui.colored_label(war_text_muted(), t.text("command-incomplete"));
                return;
            };
            let net = diplomacy_net_value(&order);
            let acceptance = diplomacy_acceptance_score(game, &order);
            ui.label(
                egui::RichText::new(t.text("command-preview-title"))
                    .color(war_gold())
                    .strong(),
            );
            ui.colored_label(
                if net >= 0 {
                    war_success()
                } else {
                    war_warning()
                },
                t.text_args("diplomacy-net-value", &args([("value", net.to_string())])),
            );
            ui.colored_label(
                acceptance_color(order.kind, acceptance),
                t.text_args(
                    "diplomacy-acceptance-score",
                    &args([("score", acceptance.to_string())]),
                ),
            );
            match diplomacy_submission_status(game, &order) {
                Ok(true) => ui.colored_label(war_success(), t.text("diplomacy-preview-accepted")),
                Ok(false) => ui.colored_label(war_warning(), t.text("diplomacy-preview-rejected")),
                Err(error) => ui.colored_label(war_danger(), error.to_string()),
            };
            ui.add_space(6.0);
            ui.colored_label(war_text_muted(), diplomacy_effect_text(game, &order, t));
        });
}

fn terms_panel(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    t: &Translator,
    max_height: f32,
) {
    war_sub_panel_frame().show(ui, |ui| {
        panel_title(
            ui,
            egui_phosphor::regular::COINS,
            &t.text("diplomacy-terms"),
        );
        city_selector(
            ui,
            "diplomacy_source_city",
            &t.text("diplomacy-source-city"),
            &mut ui_state.selected_diplomacy_source_city,
            game,
        );
        city_selector(
            ui,
            "diplomacy_receive_city",
            &t.text("diplomacy-receive-city"),
            &mut ui_state.selected_diplomacy_receive_city,
            game,
        );
        ui.separator();
        let source_limit = ui_state
            .selected_diplomacy_source_city
            .as_deref()
            .and_then(|city_id| game.cities.get(city_id))
            .map(city_resource_bundle)
            .unwrap_or_default();
        let target_limit = ui_state
            .selected_diplomacy_target
            .as_deref()
            .map(|faction_id| faction_resource_bundle(game, faction_id))
            .unwrap_or_default();
        let allow_offer = ui_state.selected_diplomacy_action != DiplomacyActionKind::DeclareWar;
        let allow_request = !matches!(
            ui_state.selected_diplomacy_action,
            DiplomacyActionKind::ImproveRelations | DiplomacyActionKind::DeclareWar
        );
        resource_editor(
            ui,
            t,
            &t.text("diplomacy-offer"),
            &mut ui_state.diplomacy_offer,
            source_limit,
            allow_offer,
        );
        ui.add_space(8.0);
        resource_editor(
            ui,
            t,
            &t.text("diplomacy-request"),
            &mut ui_state.diplomacy_request,
            target_limit,
            allow_request,
        );
        ui.separator();
        submit_diplomacy_button(ui, ui_state, game, t, max_height);
    });
}

fn submit_diplomacy_button(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    t: &Translator,
    _max_height: f32,
) {
    let Some(order) = candidate_order(ui_state, game) else {
        ui.add_enabled(false, egui::Button::new(t.text("diplomacy-submit")));
        return;
    };
    let status = diplomacy_submission_status(game, &order);
    let can_submit = status.is_ok();
    let button = ui.add_enabled(can_submit, egui::Button::new(t.text("diplomacy-submit")));
    if let Err(error) = status {
        ui.colored_label(war_danger(), error.to_string());
    }
    if button.clicked() {
        let Some(game) = &mut ui_state.game else {
            ui_state.message = t.text("message-game-not-started");
            return;
        };
        match queue_player_diplomacy(game, order) {
            Ok(()) => {
                ui_state.message = t.text("diplomacy-submitted");
                ui_state.diplomacy_offer = ResourceBundle::default();
                ui_state.diplomacy_request = ResourceBundle::default();
            }
            Err(error) => ui_state.message = error.to_string(),
        }
    }
}

fn pending_diplomacy_panel(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    t: &Translator,
    width: f32,
) {
    war_sub_panel_frame().show(ui, |ui| {
        ui.set_width(width - 28.0);
        panel_title(
            ui,
            egui_phosphor::regular::CLIPBOARD_TEXT,
            &t.text("diplomacy-pending"),
        );
        if game.pending_diplomacy.is_empty() {
            ui.colored_label(war_text_muted(), t.text("diplomacy-pending-empty"));
            return;
        }
        let mut remove_index = None;
        egui::ScrollArea::vertical()
            .id_salt("diplomacy_pending")
            .max_height(88.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (index, order) in game.pending_diplomacy.iter().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(order_summary(game, order, t));
                        if ui
                            .button(t.text("command-recruit-officer-cancel"))
                            .clicked()
                        {
                            remove_index = Some(index);
                        }
                    });
                }
            });
        if let Some(index) = remove_index
            && let Some(game) = &mut ui_state.game
            && index < game.pending_diplomacy.len()
        {
            game.pending_diplomacy.remove(index);
            ui_state.message = t.text("diplomacy-withdrawn");
        }
    });
}

fn candidate_order(ui_state: &GameUiState, game: &GameState) -> Option<DiplomacyOrder> {
    Some(DiplomacyOrder {
        issuer_faction_id: game.player_faction_id.clone(),
        target_faction_id: ui_state.selected_diplomacy_target.clone()?,
        kind: ui_state.selected_diplomacy_action,
        source_city_id: ui_state.selected_diplomacy_source_city.clone()?,
        receive_city_id: ui_state.selected_diplomacy_receive_city.clone()?,
        offer: ui_state.diplomacy_offer.clone(),
        request: ui_state.diplomacy_request.clone(),
        submitted_turn: game.turn,
    })
}

fn diplomacy_submission_status(
    game: &GameState,
    order: &DiplomacyOrder,
) -> Result<bool, CommandError> {
    validate_diplomacy_order(game, order)?;
    diplomacy_order_would_succeed(game, order)
}

fn ensure_diplomacy_defaults(ui_state: &mut GameUiState, game: &GameState) {
    let targets = diplomacy_targets(game);
    if !targets
        .iter()
        .any(|faction| Some(faction.id.as_str()) == ui_state.selected_diplomacy_target.as_deref())
    {
        ui_state.selected_diplomacy_target = targets.first().map(|faction| faction.id.clone());
    }
    let cities = player_cities(game);
    if !cities
        .iter()
        .any(|city| Some(city.id.as_str()) == ui_state.selected_diplomacy_source_city.as_deref())
    {
        ui_state.selected_diplomacy_source_city = cities.first().map(|city| city.id.clone());
    }
    if !cities
        .iter()
        .any(|city| Some(city.id.as_str()) == ui_state.selected_diplomacy_receive_city.as_deref())
    {
        ui_state.selected_diplomacy_receive_city = cities.first().map(|city| city.id.clone());
    }
    normalize_diplomacy_terms(ui_state);
}

fn normalize_diplomacy_terms(ui_state: &mut GameUiState) {
    match ui_state.selected_diplomacy_action {
        DiplomacyActionKind::ImproveRelations => {
            ui_state.diplomacy_request = ResourceBundle::default();
        }
        DiplomacyActionKind::DeclareWar => {
            ui_state.diplomacy_offer = ResourceBundle::default();
            ui_state.diplomacy_request = ResourceBundle::default();
        }
        _ => {}
    }
}

fn diplomacy_targets(game: &GameState) -> Vec<&Faction> {
    let mut targets: Vec<_> = game
        .factions
        .values()
        .filter(|faction| faction.id != game.player_faction_id && game.faction_alive(&faction.id))
        .collect();
    targets.sort_by(|a, b| a.name.cmp(&b.name));
    targets
}

fn player_cities(game: &GameState) -> Vec<&City> {
    let mut cities: Vec<_> = game
        .cities
        .values()
        .filter(|city| city.faction_id == game.player_faction_id)
        .collect();
    cities.sort_by(|a, b| a.name.cmp(&b.name));
    cities
}

fn city_selector(
    ui: &mut egui::Ui,
    id_salt: &str,
    label: &str,
    selected: &mut Option<CityId>,
    game: &GameState,
) {
    let cities = player_cities(game);
    let selected_text = selected
        .as_deref()
        .and_then(|city_id| game.cities.get(city_id))
        .map(|city| city.name.clone())
        .unwrap_or_else(|| label.to_string());
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(format!("{label}: {selected_text}"))
        .show_ui(ui, |ui| {
            for city in cities {
                ui.selectable_value(selected, Some(city.id.clone()), &city.name);
            }
        });
}

fn resource_editor(
    ui: &mut egui::Ui,
    t: &Translator,
    title: &str,
    bundle: &mut ResourceBundle,
    limit: ResourceBundle,
    enabled: bool,
) {
    ui.add_enabled_ui(enabled, |ui| {
        ui.label(egui::RichText::new(title).color(war_gold()).strong());
        resource_drag(
            ui,
            egui_phosphor::regular::COINS,
            &t.text("resource-gold"),
            &mut bundle.gold,
            limit.gold,
        );
        resource_drag(
            ui,
            egui_phosphor::regular::GRAINS,
            &t.text("resource-food"),
            &mut bundle.food,
            limit.food,
        );
        resource_drag(
            ui,
            egui_phosphor::regular::STACK,
            &t.text("resource-materials"),
            &mut bundle.materials,
            limit.materials,
        );
    });
    if !enabled {
        ui.colored_label(war_text_muted(), t.text("diplomacy-term-disabled"));
    }
}

fn resource_drag(ui: &mut egui::Ui, icon: &str, label: &str, value: &mut i32, max: i32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).color(war_gold()));
        ui.add_sized([28.0, 20.0], egui::Label::new(label));
        ui.add(egui::DragValue::new(value).range(0..=max.max(0)).speed(10));
        ui.colored_label(war_text_muted(), format!("/ {}", max.max(0)));
    });
}

fn city_resource_bundle(city: &City) -> ResourceBundle {
    ResourceBundle {
        gold: city.gold.max(0),
        food: city.food.max(0),
        materials: city.materials.max(0),
    }
}

fn faction_resource_bundle(game: &GameState, faction_id: &str) -> ResourceBundle {
    let mut bundle = ResourceBundle::default();
    for city in game
        .cities
        .values()
        .filter(|city| city.faction_id == faction_id)
    {
        bundle.gold += city.gold.max(0);
        bundle.food += city.food.max(0);
        bundle.materials += city.materials.max(0);
    }
    bundle
}

fn diplomacy_effect_text(game: &GameState, order: &DiplomacyOrder, t: &Translator) -> String {
    match order.kind {
        DiplomacyActionKind::ImproveRelations => {
            let delta = 8 + order.offer.value() / 200;
            t.text_args(
                "diplomacy-effect-relations",
                &args([("delta", delta.to_string())]),
            )
        }
        DiplomacyActionKind::ResourceExchange => t.text("diplomacy-effect-exchange"),
        DiplomacyActionKind::Truce => t.text("diplomacy-effect-truce"),
        DiplomacyActionKind::RequestPeace => t.text("diplomacy-effect-peace"),
        DiplomacyActionKind::PassageRight => t.text("diplomacy-effect-passage"),
        DiplomacyActionKind::DeclareWar => {
            let target = faction_name(game, &order.target_faction_id);
            t.text_args("diplomacy-effect-war", &args([("target", target)]))
        }
    }
}

fn order_summary(game: &GameState, order: &DiplomacyOrder, t: &Translator) -> String {
    t.text_args(
        "diplomacy-order-summary",
        &args([
            ("target", faction_name(game, &order.target_faction_id)),
            ("action", diplomacy_action_label(t, order.kind)),
            ("offer", bundle_summary(&order.offer, t)),
            ("request", bundle_summary(&order.request, t)),
        ]),
    )
}

fn bundle_summary(bundle: &ResourceBundle, t: &Translator) -> String {
    if bundle.is_empty() {
        return t.text("common-none-selected");
    }
    t.text_args(
        "diplomacy-bundle-summary",
        &args([
            ("gold", bundle.gold.to_string()),
            ("food", bundle.food.to_string()),
            ("materials", bundle.materials.to_string()),
        ]),
    )
}

fn panel_title(ui: &mut egui::Ui, icon: &str, title: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).color(war_gold()));
        ui.label(egui::RichText::new(title).color(war_gold()).strong());
    });
    ui.add_space(4.0);
}

fn metric_line(ui: &mut egui::Ui, icon: &str, label: &str, value: String, color: egui::Color32) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(icon).color(war_gold()));
        ui.add_sized(
            [88.0, 20.0],
            egui::Label::new(egui::RichText::new(label).color(war_text_muted())),
        );
        ui.colored_label(color, value);
    });
}

fn relation_score_color(score: i16) -> egui::Color32 {
    match score {
        50..=i16::MAX => war_success(),
        0..=49 => war_gold(),
        -39..=-1 => war_warning(),
        _ => war_danger(),
    }
}

fn acceptance_color(kind: DiplomacyActionKind, score: i32) -> egui::Color32 {
    let threshold = match kind {
        DiplomacyActionKind::ImproveRelations | DiplomacyActionKind::DeclareWar => {
            return war_success();
        }
        DiplomacyActionKind::ResourceExchange => 0,
        DiplomacyActionKind::Truce => -40,
        DiplomacyActionKind::RequestPeace => -20,
        DiplomacyActionKind::PassageRight => 10,
    };
    if score >= threshold {
        war_success()
    } else {
        war_warning()
    }
}

fn faction_name(game: &GameState, faction_id: &str) -> String {
    game.factions
        .get(faction_id)
        .map(|faction| faction.name.clone())
        .unwrap_or_else(|| faction_id.to_string())
}
