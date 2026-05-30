use super::city::SourceConfidence;
use super::ids::CityId;
use super::model::MapPosition;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub const MAP_BOUNDARY_ASSET_PATH: &str = "assets/data/map_boundaries.json";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapBoundaryCatalog {
    pub version: u32,
    #[serde(default)]
    pub notes: String,
    pub boundaries: Vec<MapBoundary>,
}

impl MapBoundaryCatalog {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, MapBoundaryError> {
        let input = fs::read_to_string(path).map_err(MapBoundaryError::Io)?;
        Self::from_json_str(&input)
    }

    pub fn from_json_str(input: &str) -> Result<Self, MapBoundaryError> {
        let catalog: Self = serde_json::from_str(input).map_err(MapBoundaryError::Parse)?;
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn boundaries_for_year(&self, year: i32) -> impl Iterator<Item = &MapBoundary> {
        self.boundaries
            .iter()
            .filter(move |boundary| boundary.includes_year(year))
    }

    pub fn validate(&self) -> Result<(), MapBoundaryError> {
        let mut ids = BTreeSet::new();
        let mut levels = BTreeMap::new();

        for boundary in &self.boundaries {
            if boundary.id.trim().is_empty() {
                return Err(MapBoundaryError::Invalid("边界 id 不能为空".to_string()));
            }
            if !ids.insert(boundary.id.as_str()) {
                return Err(MapBoundaryError::Invalid(format!(
                    "边界 id {} 重复",
                    boundary.id
                )));
            }
            if boundary.points.len() < 3 {
                return Err(MapBoundaryError::Invalid(format!(
                    "边界 {} 至少需要 3 个点",
                    boundary.id
                )));
            }
            if let (Some(from), Some(to)) = (boundary.valid_from_year, boundary.valid_to_year) {
                if from > to {
                    return Err(MapBoundaryError::Invalid(format!(
                        "边界 {} 年份范围无效",
                        boundary.id
                    )));
                }
            }
            if boundary
                .points
                .iter()
                .any(|point| !point.x.is_finite() || !point.y.is_finite())
            {
                return Err(MapBoundaryError::Invalid(format!(
                    "边界 {} 包含无效坐标",
                    boundary.id
                )));
            }
            if boundary.level == MapBoundaryLevel::Commandery && boundary.parent_id.is_none() {
                return Err(MapBoundaryError::Invalid(format!(
                    "郡域边界 {} 缺少父级州",
                    boundary.id
                )));
            }
            levels.insert(boundary.id.as_str(), boundary.level);
        }

        for boundary in &self.boundaries {
            if let Some(parent_id) = &boundary.parent_id {
                match levels.get(parent_id.as_str()) {
                    Some(MapBoundaryLevel::Province) => {}
                    Some(MapBoundaryLevel::Commandery) => {
                        return Err(MapBoundaryError::Invalid(format!(
                            "边界 {} 的父级 {} 不是州界",
                            boundary.id, parent_id
                        )));
                    }
                    None => {
                        return Err(MapBoundaryError::Invalid(format!(
                            "边界 {} 引用了不存在的父级 {}",
                            boundary.id, parent_id
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum MapBoundaryLevel {
    Province,
    Commandery,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapBoundary {
    pub id: String,
    pub name: String,
    pub level: MapBoundaryLevel,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub valid_from_year: Option<i32>,
    #[serde(default)]
    pub valid_to_year: Option<i32>,
    pub confidence: SourceConfidence,
    pub source: String,
    #[serde(default)]
    pub city_ids: Vec<CityId>,
    #[serde(deserialize_with = "deserialize_boundary_points")]
    pub points: Vec<MapPosition>,
}

impl MapBoundary {
    pub fn includes_year(&self, year: i32) -> bool {
        self.valid_from_year.is_none_or(|from| year >= from)
            && self.valid_to_year.is_none_or(|to| year <= to)
    }
}

#[derive(Debug)]
pub enum MapBoundaryError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Invalid(String),
}

impl std::fmt::Display for MapBoundaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapBoundaryError::Io(error) => write!(f, "读取地图边界失败: {error}"),
            MapBoundaryError::Parse(error) => write!(f, "解析地图边界失败: {error}"),
            MapBoundaryError::Invalid(message) => write!(f, "地图边界数据无效: {message}"),
        }
    }
}

impl std::error::Error for MapBoundaryError {}

#[derive(Deserialize)]
#[serde(untagged)]
enum BoundaryPointSeed {
    Named { x: f32, y: f32 },
    Tuple([f32; 2]),
}

fn deserialize_boundary_points<'de, D>(deserializer: D) -> Result<Vec<MapPosition>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let points = Vec::<BoundaryPointSeed>::deserialize(deserializer)?;
    Ok(points
        .into_iter()
        .map(|point| match point {
            BoundaryPointSeed::Named { x, y } => MapPosition { x, y },
            BoundaryPointSeed::Tuple([x, y]) => MapPosition { x, y },
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_JSON: &str = r#"
    {
      "version": 1,
      "boundaries": [
        {
          "id": "province_si",
          "name": "司隶",
          "level": "province",
          "valid_from_year": 184,
          "valid_to_year": 220,
          "confidence": "Medium",
          "source": "test",
          "points": [
            { "x": -10.0, "y": -10.0 },
            { "x": 10.0, "y": -10.0 },
            { "x": 10.0, "y": 10.0 },
            { "x": -10.0, "y": 10.0 }
          ]
        },
        {
          "id": "commandery_henan",
          "name": "河南尹",
          "level": "commandery",
          "parent_id": "province_si",
          "valid_from_year": 184,
          "valid_to_year": 220,
          "confidence": "Medium",
          "source": "test",
          "city_ids": ["luoyang"],
          "points": [
            { "x": -5.0, "y": -5.0 },
            { "x": 5.0, "y": -5.0 },
            { "x": 5.0, "y": 5.0 }
          ]
        }
      ]
    }
    "#;

    #[test]
    fn valid_catalog_loads_and_filters_by_year() {
        let catalog = MapBoundaryCatalog::from_json_str(VALID_JSON).unwrap();
        assert_eq!(catalog.boundaries.len(), 2);
        assert_eq!(catalog.boundaries_for_year(190).count(), 2);
        assert_eq!(catalog.boundaries_for_year(230).count(), 0);
    }

    #[test]
    fn invalid_catalog_rejects_short_polygons() {
        let input = VALID_JSON.replace(
            r#",
            { "x": 5.0, "y": 5.0 }"#,
            "",
        );

        let error = MapBoundaryCatalog::from_json_str(&input).unwrap_err();
        assert!(error.to_string().contains("至少需要 3 个点"));
    }

    #[test]
    fn invalid_catalog_rejects_bad_year_range() {
        let input = VALID_JSON.replace(
            r#""valid_from_year": 184,
          "valid_to_year": 220"#,
            r#""valid_from_year": 221,
          "valid_to_year": 220"#,
        );

        let error = MapBoundaryCatalog::from_json_str(&input).unwrap_err();
        assert!(error.to_string().contains("年份范围无效"));
    }

    #[test]
    fn invalid_catalog_rejects_commandery_without_parent() {
        let input = VALID_JSON.replace(
            r#""parent_id": "province_si",
          "#,
            "",
        );

        let error = MapBoundaryCatalog::from_json_str(&input).unwrap_err();
        assert!(error.to_string().contains("缺少父级州"));
    }
}
