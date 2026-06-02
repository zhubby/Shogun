use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct ImportedOfficer {
    id: String,
    external_id: String,
    name: String,
    courtesy_name: Option<String>,
    native_place: Option<String>,
    birth_year: Option<i32>,
    death_year: Option<i32>,
    gender: String,
    stats: [u8; 5],
    tags: Vec<String>,
    confidence: String,
    biography: String,
    notes: String,
    source_url: String,
    family: Value,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct RelationshipSeed {
    source_id: String,
    target_id: String,
    kind: &'static str,
    confidence: &'static str,
    notes: String,
    source: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct FamilyRelationshipSpec {
    key: &'static str,
    kind: &'static str,
    label: &'static str,
    symmetric: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let source_dir = args
        .next()
        .map(PathBuf::from)
        .ok_or("usage: import_three_kingdoms <characters-json-dir> <output-sql-path>")?;
    let output_path = args
        .next()
        .map(PathBuf::from)
        .ok_or("usage: import_three_kingdoms <characters-json-dir> <output-sql-path>")?;

    let officers = load_officers(&source_dir)?;
    let sql = render_migration_fragment(&officers);
    fs::write(&output_path, sql)?;
    println!(
        "generated {} from {} imported officers",
        output_path.display(),
        officers.len()
    );
    Ok(())
}

fn load_officers(source_dir: &Path) -> Result<Vec<ImportedOfficer>, Box<dyn std::error::Error>> {
    let mut files = fs::read_dir(source_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    files.retain(|path| path.extension().is_some_and(|ext| ext == "json"));
    files.sort();

    let id_overrides = id_overrides();
    let mut officers = Vec::new();
    for path in files {
        let body = fs::read_to_string(&path)?;
        let value: Value = serde_json::from_str(&body)?;
        let Some(name) = value.get("name").and_then(Value::as_str) else {
            continue;
        };
        let external_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(name)
            .to_string();
        let id = id_overrides
            .get(name)
            .cloned()
            .unwrap_or_else(|| ascii_id("ctk", &external_id));
        let faction = value
            .get("faction")
            .and_then(Value::as_str)
            .unwrap_or("未详");
        let positions = value
            .get("position")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default();
        let biography = value
            .get("historicalBriefIIntroduction")
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .or_else(|| {
                value
                    .get("novelisticBriefIIntroduction")
                    .and_then(Value::as_str)
                    .filter(|text| !text.trim().is_empty())
            })
            .unwrap_or("")
            .trim()
            .to_string();

        officers.push(ImportedOfficer {
            id,
            external_id: external_id.clone(),
            name: name.to_string(),
            courtesy_name: value
                .get("courtesyName")
                .and_then(Value::as_str)
                .filter(|text| !text.trim().is_empty())
                .map(str::to_string),
            native_place: value
                .get("birthplace")
                .and_then(Value::as_str)
                .filter(|text| !text.trim().is_empty())
                .map(str::to_string),
            birth_year: value
                .get("birthdate")
                .and_then(Value::as_str)
                .and_then(extract_year),
            death_year: value
                .get("deathdate")
                .and_then(Value::as_str)
                .and_then(extract_year),
            gender: match value.get("gender").and_then(Value::as_i64) {
                Some(0) => "Female".to_string(),
                _ => "Male".to_string(),
            },
            stats: inferred_stats(name, faction, &positions),
            tags: tags_for(faction, &positions),
            confidence: if biography.is_empty() {
                "Medium".to_string()
            } else {
                "High".to_string()
            },
            biography,
            notes: format!(
                "来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 {}",
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(external_id.as_str())
            ),
            source_url: format!(
                "https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/{}.json",
                external_id
            ),
            family: value.get("family").cloned().unwrap_or(Value::Null),
        });
    }

    Ok(officers)
}

fn render_migration_fragment(officers: &[ImportedOfficer]) -> String {
    let mut output = String::new();
    output.push_str("-- Source-backed import migration fragment generated from the MIT-licensed\n");
    output.push_str("-- fthux/Characters_of_the_Three_Kingdoms character JSON corpus.\n");
    output.push_str("--\n");
    output.push_str("-- Rebuild with:\n");
    output.push_str("--   rtk cargo run --bin import_three_kingdoms -- <source-characters-dir> /tmp/ctk_import_fragment.sql\n\n");

    let core_ids = core_officer_ids();
    for officer in officers {
        let [leadership, strength, intelligence, politics, charm] = officer.stats;
        output.push_str("INSERT INTO officers\n");
        output.push_str("(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, confidence, biography, notes)\n");
        output.push_str("VALUES (");
        output.push_str(&sql_string(&officer.id));
        output.push_str(", ");
        output.push_str(&sql_string(&officer.name));
        output.push_str(", ");
        output.push_str(&sql_optional_string(officer.courtesy_name.as_deref()));
        output.push_str(", ");
        output.push_str(&sql_optional_string(officer.native_place.as_deref()));
        output.push_str(", ");
        output.push_str(&sql_optional_i32(officer.birth_year));
        output.push_str(", ");
        output.push_str(&sql_optional_i32(officer.death_year));
        output.push_str(", ");
        output.push_str(&sql_string(&officer.gender));
        output.push_str(&format!(
            ", {leadership}, {strength}, {intelligence}, {politics}, {charm}, "
        ));
        output.push_str(&sql_string(&officer.confidence));
        output.push_str(", ");
        output.push_str(&sql_string(&officer.biography));
        output.push_str(", ");
        output.push_str(&sql_string(&officer.notes));
        output.push_str(")\nON CONFLICT(id) DO UPDATE SET\n");
        output.push_str(
            "    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),\n",
        );
        output.push_str(
            "    native_place = COALESCE(excluded.native_place, officers.native_place),\n",
        );
        output.push_str("    birth_year = COALESCE(officers.birth_year, excluded.birth_year),\n");
        output.push_str("    death_year = COALESCE(officers.death_year, excluded.death_year),\n");
        output.push_str("    gender = excluded.gender,\n");
        output.push_str("    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,\n");
        output.push_str("    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;\n\n");

        for tag_id in officer.tags.iter().filter_map(|tag| canonical_tag_id(tag)) {
            output.push_str("INSERT OR IGNORE INTO officer_tags (officer_id, tag_id) VALUES (");
            output.push_str(&sql_string(&officer.id));
            output.push_str(", ");
            output.push_str(&sql_string(tag_id));
            output.push_str(");\n");
        }
        output.push('\n');

        output.push_str("INSERT OR REPLACE INTO officer_external_ids\n");
        output.push_str("(officer_id, source, external_id, source_url, confidence, notes)\n");
        output.push_str("VALUES (");
        output.push_str(&sql_string(&officer.id));
        output.push_str(", 'characters_of_the_three_kingdoms', ");
        output.push_str(&sql_string(&officer.external_id));
        output.push_str(", ");
        output.push_str(&sql_string(&officer.source_url));
        output.push_str(", ");
        output.push_str(&sql_string(&officer.confidence));
        output.push_str(", 'MIT licensed game-oriented character corpus');\n\n");

        if !core_ids.contains(&officer.id) {
            output.push_str("INSERT OR IGNORE INTO officer_life_events\n");
            output.push_str("(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)\n");
            output.push_str("VALUES (");
            output.push_str(&sql_string(&format!("ctk_start_{}", officer.id)));
            output.push_str(", ");
            output.push_str(&sql_string(&officer.id));
            output.push_str(", ");
            output.push_str(&officer.birth_year.map_or(190, |year| year + 18).to_string());
            output.push_str(", 1, 'Appear', NULL, NULL, ");
            output.push_str(&initial_loyalty(&officer.confidence).to_string());
            output.push_str(", 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');\n\n");
        }
    }

    let relationships = imported_relationships(officers);
    for relationship in relationships {
        output.push_str("INSERT OR IGNORE INTO officer_relationships\n");
        output.push_str("(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)\n");
        output.push_str("VALUES (");
        output.push_str(&sql_string(&relationship.source_id));
        output.push_str(", ");
        output.push_str(&sql_string(&relationship.target_id));
        output.push_str(", ");
        output.push_str(&sql_string(relationship.kind));
        output.push_str(", ");
        output.push_str(&sql_string(relationship.confidence));
        output.push_str(", ");
        output.push_str(&sql_string(&relationship.notes));
        output.push_str(", ");
        output.push_str(&sql_string(relationship.source));
        output.push_str(");\n");
    }

    output
}

fn initial_loyalty(confidence: &str) -> u8 {
    match confidence {
        "High" => 76,
        "Low" => 62,
        _ => 70,
    }
}

fn imported_relationships(officers: &[ImportedOfficer]) -> BTreeSet<RelationshipSeed> {
    let mut name_to_id = id_overrides();
    let mut known_ids = core_officer_ids();
    for officer in officers {
        name_to_id.insert(officer.name.clone(), officer.id.clone());
        known_ids.insert(officer.id.clone());
    }

    let mut relationships = BTreeSet::new();
    for officer in officers {
        for spec in [
            FamilyRelationshipSpec {
                key: "father",
                kind: "ParentChild",
                label: "父子/父女",
                symmetric: false,
            },
            FamilyRelationshipSpec {
                key: "mother",
                kind: "ParentChild",
                label: "母子/母女",
                symmetric: false,
            },
            FamilyRelationshipSpec {
                key: "sons",
                kind: "ParentChild",
                label: "亲子",
                symmetric: false,
            },
            FamilyRelationshipSpec {
                key: "daughters",
                kind: "ParentChild",
                label: "亲子",
                symmetric: false,
            },
            FamilyRelationshipSpec {
                key: "spouse",
                kind: "Spouse",
                label: "夫妻",
                symmetric: true,
            },
            FamilyRelationshipSpec {
                key: "brothers",
                kind: "Sibling",
                label: "兄弟",
                symmetric: true,
            },
            FamilyRelationshipSpec {
                key: "sisters",
                kind: "Sibling",
                label: "兄妹/姊妹",
                symmetric: true,
            },
        ] {
            add_family_relationships(
                &mut relationships,
                &name_to_id,
                &known_ids,
                &officer.id,
                &officer.family,
                spec,
            );
        }
    }

    relationships
}

fn add_family_relationships(
    relationships: &mut BTreeSet<RelationshipSeed>,
    name_to_id: &BTreeMap<String, String>,
    known_ids: &BTreeSet<String>,
    source_id: &str,
    family: &Value,
    spec: FamilyRelationshipSpec,
) {
    let Some(entries) = family
        .get(spec.key)
        .and_then(|item| item.get("character"))
        .and_then(Value::as_array)
    else {
        return;
    };

    for entry in entries {
        let Some(name) = entry.get("name").and_then(Value::as_str) else {
            continue;
        };
        if name.contains("不详") {
            continue;
        }
        let Some(target_id) = name_to_id.get(name) else {
            continue;
        };
        if !known_ids.contains(target_id) {
            continue;
        }
        if target_id == source_id {
            continue;
        }
        let desc = entry
            .get("desc")
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .unwrap_or(spec.label);
        let notes = format!("{}: {desc}", spec.label);
        relationships.insert(RelationshipSeed {
            source_id: source_id.to_string(),
            target_id: target_id.clone(),
            kind: spec.kind,
            confidence: "Medium",
            notes: notes.clone(),
            source: "characters_of_the_three_kingdoms",
        });
        if spec.symmetric {
            relationships.insert(RelationshipSeed {
                source_id: target_id.clone(),
                target_id: source_id.to_string(),
                kind: spec.kind,
                confidence: "Medium",
                notes,
                source: "characters_of_the_three_kingdoms",
            });
        }
    }
}

fn core_officer_ids() -> BTreeSet<String> {
    BTreeSet::from([
        "cao_cao".to_string(),
        "cao_hong".to_string(),
        "cao_pi".to_string(),
        "cao_ren".to_string(),
        "chen_dao".to_string(),
        "chen_gong".to_string(),
        "cheng_pu".to_string(),
        "cheng_yu".to_string(),
        "dian_wei".to_string(),
        "dong_zhuo".to_string(),
        "fa_zheng".to_string(),
        "gan_ning".to_string(),
        "gao_lan".to_string(),
        "gao_shun".to_string(),
        "gongsun_zan".to_string(),
        "guan_yu".to_string(),
        "guo_jia".to_string(),
        "han_sui".to_string(),
        "han_xian_di".to_string(),
        "huang_gai".to_string(),
        "huang_zhong".to_string(),
        "huang_zu".to_string(),
        "jia_xu".to_string(),
        "jian_yong".to_string(),
        "jiang_wei".to_string(),
        "ji_ling".to_string(),
        "ju_shou".to_string(),
        "kong_rong".to_string(),
        "kuai_liang".to_string(),
        "kuai_yue".to_string(),
        "li_dian".to_string(),
        "li_ru".to_string(),
        "liu_bei".to_string(),
        "liu_biao".to_string(),
        "liu_shan".to_string(),
        "liu_yao".to_string(),
        "liu_zhang".to_string(),
        "lu_bu".to_string(),
        "lu_su".to_string(),
        "lu_xun".to_string(),
        "ma_chao".to_string(),
        "ma_dai".to_string(),
        "ma_teng".to_string(),
        "man_chong".to_string(),
        "mi_zhu".to_string(),
        "pang_tong".to_string(),
        "shen_pei".to_string(),
        "shi_xie".to_string(),
        "sima_yi".to_string(),
        "sun_ce".to_string(),
        "sun_jian".to_string(),
        "sun_qian".to_string(),
        "sun_quan".to_string(),
        "taishi_ci".to_string(),
        "tao_qian".to_string(),
        "tian_feng".to_string(),
        "wei_yan".to_string(),
        "wen_chou".to_string(),
        "xiahou_dun".to_string(),
        "xiahou_yuan".to_string(),
        "xun_you".to_string(),
        "xun_yu".to_string(),
        "xu_chu".to_string(),
        "xu_huang".to_string(),
        "yan_liang".to_string(),
        "yan_yan".to_string(),
        "yang_song".to_string(),
        "yuan_shang".to_string(),
        "yuan_shao".to_string(),
        "yuan_shu".to_string(),
        "yuan_tan".to_string(),
        "yue_jin".to_string(),
        "yu_jin".to_string(),
        "zhang_fei".to_string(),
        "zhang_he".to_string(),
        "zhang_hong".to_string(),
        "zhang_liao".to_string(),
        "zhang_lu".to_string(),
        "zhang_ren".to_string(),
        "zhang_zhao".to_string(),
        "zhao_yun".to_string(),
        "zhao_yun_early".to_string(),
        "zhou_tai".to_string(),
        "zhou_yu".to_string(),
        "zhuge_liang".to_string(),
    ])
}

fn id_overrides() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("刘备".to_string(), "liu_bei".to_string()),
        ("刘禅".to_string(), "liu_shan".to_string()),
        ("刘表".to_string(), "liu_biao".to_string()),
        ("刘璋".to_string(), "liu_zhang".to_string()),
        ("刘繇".to_string(), "liu_yao".to_string()),
        ("刘协".to_string(), "han_xian_di".to_string()),
        ("孙坚".to_string(), "sun_jian".to_string()),
        ("孙策".to_string(), "sun_ce".to_string()),
        ("孙权".to_string(), "sun_quan".to_string()),
        ("曹操".to_string(), "cao_cao".to_string()),
        ("曹丕".to_string(), "cao_pi".to_string()),
        ("吕布".to_string(), "lu_bu".to_string()),
        ("董卓".to_string(), "dong_zhuo".to_string()),
        ("袁绍".to_string(), "yuan_shao".to_string()),
        ("袁术".to_string(), "yuan_shu".to_string()),
        ("马腾".to_string(), "ma_teng".to_string()),
        ("马超".to_string(), "ma_chao".to_string()),
        ("张鲁".to_string(), "zhang_lu".to_string()),
        ("张辽".to_string(), "zhang_liao".to_string()),
        ("张郃".to_string(), "zhang_he".to_string()),
        ("张飞".to_string(), "zhang_fei".to_string()),
        ("张昭".to_string(), "zhang_zhao".to_string()),
        ("张纮".to_string(), "zhang_hong".to_string()),
        ("张任".to_string(), "zhang_ren".to_string()),
        ("张松".to_string(), "zhang_song".to_string()),
        ("关羽".to_string(), "guan_yu".to_string()),
        ("赵云".to_string(), "zhao_yun".to_string()),
        ("诸葛亮".to_string(), "zhuge_liang".to_string()),
        ("司马懿".to_string(), "sima_yi".to_string()),
        ("周瑜".to_string(), "zhou_yu".to_string()),
        ("鲁肃".to_string(), "lu_su".to_string()),
        ("吕蒙".to_string(), "lu_meng".to_string()),
        ("陆逊".to_string(), "lu_xun".to_string()),
        ("甘夫人".to_string(), "lady_gan".to_string()),
        ("糜夫人".to_string(), "lady_mi".to_string()),
        ("麋夫人".to_string(), "lady_mi".to_string()),
        ("孙夫人".to_string(), "lady_sun".to_string()),
        ("穆皇后".to_string(), "empress_mu".to_string()),
    ])
}

fn ascii_id(prefix: &str, value: &str) -> String {
    let encoded = value
        .chars()
        .map(|ch| format!("{:x}", ch as u32))
        .collect::<Vec<_>>()
        .join("_");
    format!("{prefix}_{encoded}")
}

fn extract_year(value: &str) -> Option<i32> {
    let mut digits = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else if !digits.is_empty() {
            break;
        }
    }
    digits.parse().ok()
}

fn inferred_stats(name: &str, faction: &str, positions: &str) -> [u8; 5] {
    let seed = name.chars().map(|ch| ch as u32).sum::<u32>();
    let mut leadership = 45 + (seed % 31) as u8;
    let mut strength = 38 + ((seed / 3) % 35) as u8;
    let mut intelligence = 42 + ((seed / 5) % 38) as u8;
    let mut politics = 40 + ((seed / 7) % 40) as u8;
    let mut charm = 42 + ((seed / 11) % 36) as u8;

    if positions.contains("皇帝") || positions.contains("君主") || faction.contains('汉') {
        leadership = leadership.saturating_add(12).min(90);
        politics = politics.saturating_add(12).min(92);
        charm = charm.saturating_add(10).min(92);
    }
    if positions.contains('将') || positions.contains("都督") {
        leadership = leadership.saturating_add(10).min(92);
        strength = strength.saturating_add(10).min(92);
    }
    if positions.contains("太守") || positions.contains("刺史") || positions.contains("尚书")
    {
        politics = politics.saturating_add(10).min(90);
        intelligence = intelligence.saturating_add(6).min(90);
    }

    [leadership, strength, intelligence, politics, charm]
}

fn tags_for(faction: &str, positions: &str) -> Vec<String> {
    let mut tags = BTreeSet::from(["ctk_import".to_string(), "source_backed".to_string()]);
    if !faction.trim().is_empty() && faction != "未详" {
        tags.insert(format!("faction:{faction}"));
    }
    if positions.contains("皇帝") || positions.contains("君主") {
        tags.insert("ruler".to_string());
    }
    if positions.contains('将') || positions.contains("都督") {
        tags.insert("general".to_string());
    }
    if positions.contains("太守") || positions.contains("刺史") || positions.contains("尚书")
    {
        tags.insert("administrator".to_string());
    }
    tags.into_iter().collect()
}

fn canonical_tag_id(tag: &str) -> Option<&'static str> {
    match tag {
        "ctk_import" => Some("source:ctk_import"),
        "source_backed" => Some("source:source_backed"),
        "ruler" => Some("role:ruler"),
        "general" => Some("role:general"),
        "administrator" => Some("role:administrator"),
        "faction:东汉" => Some("affiliation:han_court"),
        "faction:魏国" | "faction:魏" => Some("affiliation:cao_wei"),
        "faction:蜀" | "faction:蜀汉" => Some("affiliation:shu_han"),
        "faction:东吴" | "faction:吴" => Some("affiliation:eastern_wu"),
        "faction:西晋" => Some("affiliation:western_jin"),
        "faction:起义军" => Some("affiliation:rebel"),
        "faction:袁术" => Some("affiliation:yuan_shu"),
        _ => None,
    }
}

fn sql_optional_i32(value: Option<i32>) -> String {
    value.map_or_else(|| "NULL".to_string(), |value| value.to_string())
}

fn sql_optional_string(value: Option<&str>) -> String {
    value.map_or_else(|| "NULL".to_string(), sql_string)
}

fn sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
