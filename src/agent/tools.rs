use crate::agent::{FunctionAgentTool, ToolResult};
use crate::game::{
    GameState, Officer, OfficerGender, OfficerRelationshipKind, OfficerStatus, official_post_spec,
};
use serde_json::{Value, json};
use std::sync::Arc;

pub fn get_officer_info_tool(state: GameState) -> FunctionAgentTool {
    let state = Arc::new(state);
    FunctionAgentTool::new(
        "get_officer_info",
        "Read public in-game information for any officer by officer_id or exact name.",
        json!({
            "type": "object",
            "properties": {
                "officer_id": {
                    "type": "string",
                    "description": "Stable officer id when known."
                },
                "name": {
                    "type": "string",
                    "description": "Exact officer display name when id is unknown."
                }
            },
            "additionalProperties": false
        }),
        move |arguments| get_officer_info(&state, arguments),
    )
}

fn get_officer_info(state: &GameState, arguments: Value) -> ToolResult {
    let officer_id = arguments.get("officer_id").and_then(Value::as_str);
    let name = arguments.get("name").and_then(Value::as_str);
    let Some(officer) = find_officer(state, officer_id, name) else {
        return ToolResult::error(format!(
            "officer not found for officer_id={:?}, name={:?}",
            officer_id, name
        ));
    };

    ToolResult::success(officer_info_json(state, officer))
}

fn find_officer<'a>(
    state: &'a GameState,
    officer_id: Option<&str>,
    name: Option<&str>,
) -> Option<&'a Officer> {
    if let Some(officer_id) = officer_id.and_then(non_empty_trimmed)
        && let Some(officer) = state.officers.get(officer_id)
    {
        return Some(officer);
    }
    let name = name.and_then(non_empty_trimmed)?;
    let mut matches = state
        .officers
        .values()
        .filter(|officer| officer.name == name);
    let first = matches.next()?;
    if matches.next().is_some() {
        None
    } else {
        Some(first)
    }
}

fn officer_info_json(state: &GameState, officer: &Officer) -> Value {
    let profile = officer.profile.as_ref();
    json!({
        "id": officer.id,
        "name": officer.name,
        "gender": gender_label(&officer.gender),
        "status": status_label(&officer.status),
        "age": officer.age_at(state.year),
        "birth_year": officer.birth_year,
        "faction": officer.faction_id.as_str(),
        "faction_name": state.factions.get(&officer.faction_id).map(|faction| faction.name.as_str()),
        "city": officer.city_id.as_deref(),
        "city_name": officer.city_id.as_deref().and_then(|city_id| {
            state.cities.get(city_id).map(|city| city.name.as_str())
        }),
        "office": officer.office_id.as_deref(),
        "office_name": officer.office_id.as_deref().and_then(|office_id| {
            official_post_spec(office_id).map(|spec| spec.name)
        }),
        "stats": {
            "leadership": officer.stats.leadership,
            "strength": officer.stats.strength,
            "intelligence": officer.stats.intelligence,
            "politics": officer.stats.politics,
            "charm": officer.stats.charm
        },
        "profile": {
            "courtesy_name": profile.and_then(|profile| profile.courtesy_name.as_deref()),
            "native_place": profile.and_then(|profile| profile.native_place.as_deref()),
            "tags": profile.map(|profile| profile.tags.clone()).unwrap_or_default(),
            "biography": profile
                .map(|profile| truncate(&profile.biography, 240))
                .unwrap_or_default(),
            "notes": profile.map(|profile| truncate(&profile.notes, 120)).unwrap_or_default()
        },
        "relationships": relationship_summary(state, &officer.id)
    })
}

fn relationship_summary(state: &GameState, officer_id: &str) -> Vec<Value> {
    let mut relationships = Vec::new();
    for relationship in &state.family_relationships {
        if relationship.parent_id == officer_id {
            relationships.push(json!({
                "kind": "child",
                "officer_id": relationship.child_id,
                "name": officer_name(state, &relationship.child_id)
            }));
        } else if relationship.child_id == officer_id {
            relationships.push(json!({
                "kind": "parent",
                "officer_id": relationship.parent_id,
                "name": officer_name(state, &relationship.parent_id)
            }));
        }
    }
    for marriage in &state.marriages {
        if marriage.husband_id == officer_id {
            relationships.push(json!({
                "kind": "spouse",
                "officer_id": marriage.wife_id,
                "name": officer_name(state, &marriage.wife_id)
            }));
        } else if marriage.wife_id == officer_id {
            relationships.push(json!({
                "kind": "spouse",
                "officer_id": marriage.husband_id,
                "name": officer_name(state, &marriage.husband_id)
            }));
        }
    }
    if let Some(profile) = state
        .officers
        .get(officer_id)
        .and_then(|officer| officer.profile.as_ref())
    {
        for relationship in &profile.relationships {
            relationships.push(json!({
                "kind": historical_relationship_label(&relationship.kind),
                "officer_id": relationship.target_id,
                "name": relationship.target_name
            }));
        }
    }
    relationships
}

fn officer_name(state: &GameState, officer_id: &str) -> String {
    state
        .officers
        .get(officer_id)
        .map(|officer| officer.name.clone())
        .unwrap_or_else(|| officer_id.to_string())
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn gender_label(gender: &OfficerGender) -> &'static str {
    match gender {
        OfficerGender::Male => "male",
        OfficerGender::Female => "female",
    }
}

fn status_label(status: &OfficerStatus) -> &'static str {
    match status {
        OfficerStatus::Active => "active",
        OfficerStatus::Minor => "minor",
        OfficerStatus::Wild => "wild",
        OfficerStatus::Unavailable => "unavailable",
        OfficerStatus::Dead => "dead",
    }
}

fn historical_relationship_label(kind: &OfficerRelationshipKind) -> &'static str {
    match kind {
        OfficerRelationshipKind::RulerSubject => "ruler_subject",
        OfficerRelationshipKind::ParentChild => "parent_child",
        OfficerRelationshipKind::AdoptiveParentChild => "adoptive_parent_child",
        OfficerRelationshipKind::Spouse => "spouse",
        OfficerRelationshipKind::Sibling => "sibling",
        OfficerRelationshipKind::SwornSibling => "sworn_sibling",
        OfficerRelationshipKind::Enemy => "enemy",
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{HistoricalCatalog, SqliteHistoricalCatalog};

    #[test]
    fn get_officer_info_reads_public_officer_fields() {
        let game = SqliteHistoricalCatalog::in_memory_from_seed()
            .unwrap()
            .build_game("ad200", "liu_bei")
            .unwrap();
        let result = get_officer_info(&game, json!({"officer_id": "liu_bei"}));

        assert!(result.success);
        assert_eq!(result.output["id"], "liu_bei");
        assert_eq!(result.output["name"], "刘备");
        assert!(result.output["stats"]["leadership"].as_u64().unwrap() > 0);
    }

    #[test]
    fn get_officer_info_supports_exact_name_lookup() {
        let game = SqliteHistoricalCatalog::in_memory_from_seed()
            .unwrap()
            .build_game("ad200", "liu_bei")
            .unwrap();
        let result = get_officer_info(&game, json!({"name": "关羽"}));

        assert!(result.success);
        assert_eq!(result.output["id"], "guan_yu");
    }
}
