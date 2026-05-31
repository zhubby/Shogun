use super::city::SourceConfidence;
use super::ids::{CityId, FactionId, OfficerId};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OfficerStatus {
    Active,
    Wild,
    Unavailable,
    Dead,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub enum OfficerGender {
    #[default]
    Male,
    Female,
}

impl<'de> Deserialize<'de> for OfficerGender {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "Female" => Self::Female,
            _ => Self::Male,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OfficerRelationshipKind {
    RulerSubject,
    ParentChild,
    AdoptiveParentChild,
    Spouse,
    Sibling,
    SwornSibling,
    Enemy,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OfficerRelationship {
    pub target_id: OfficerId,
    pub target_name: String,
    pub kind: OfficerRelationshipKind,
    pub confidence: SourceConfidence,
    pub notes: String,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OfficerProfile {
    pub id: OfficerId,
    pub name: String,
    pub courtesy_name: Option<String>,
    pub native_place: Option<String>,
    pub birth_year: Option<i32>,
    pub death_year: Option<i32>,
    #[serde(default)]
    pub gender: OfficerGender,
    pub stats: OfficerStats,
    pub tags: Vec<String>,
    pub confidence: SourceConfidence,
    #[serde(default)]
    pub biography: String,
    #[serde(default)]
    pub relationships: Vec<OfficerRelationship>,
    pub notes: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OfficerProfileUpdate {
    pub name: String,
    pub courtesy_name: Option<String>,
    pub native_place: Option<String>,
    pub birth_year: Option<i32>,
    pub death_year: Option<i32>,
    pub gender: OfficerGender,
    pub stats: OfficerStats,
    pub tags: Vec<String>,
    pub confidence: SourceConfidence,
    pub biography: String,
    pub notes: String,
}

impl OfficerProfileUpdate {
    pub fn from_profile(profile: &OfficerProfile) -> Self {
        Self {
            name: profile.name.clone(),
            courtesy_name: profile.courtesy_name.clone(),
            native_place: profile.native_place.clone(),
            birth_year: profile.birth_year,
            death_year: profile.death_year,
            gender: profile.gender.clone(),
            stats: profile.stats,
            tags: profile.tags.clone(),
            confidence: profile.confidence.clone(),
            biography: profile.biography.clone(),
            notes: profile.notes.clone(),
        }
    }
}

pub trait OfficerProfileView {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn courtesy_name(&self) -> Option<&str>;
    fn native_place(&self) -> Option<&str>;
    fn birth_year(&self) -> Option<i32>;
    fn death_year(&self) -> Option<i32>;
    fn gender(&self) -> &OfficerGender;
    fn confidence(&self) -> &SourceConfidence;
}

impl OfficerProfileView for OfficerProfile {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn courtesy_name(&self) -> Option<&str> {
        self.courtesy_name.as_deref()
    }

    fn native_place(&self) -> Option<&str> {
        self.native_place.as_deref()
    }

    fn birth_year(&self) -> Option<i32> {
        self.birth_year
    }

    fn death_year(&self) -> Option<i32> {
        self.death_year
    }

    fn gender(&self) -> &OfficerGender {
        &self.gender
    }

    fn confidence(&self) -> &SourceConfidence {
        &self.confidence
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Officer {
    pub id: OfficerId,
    pub name: String,
    pub faction_id: FactionId,
    pub city_id: Option<CityId>,
    pub stats: OfficerStats,
    pub loyalty: u8,
    #[serde(default)]
    pub gender: OfficerGender,
    pub status: OfficerStatus,
    pub profile: Option<OfficerProfile>,
}

impl Officer {
    pub fn is_active(&self) -> bool {
        self.status == OfficerStatus::Active
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OfficerStats {
    pub leadership: u8,
    pub strength: u8,
    pub intelligence: u8,
    pub politics: u8,
    pub charm: u8,
}

pub trait OfficerAbilityView {
    fn leadership(&self) -> u8;
    fn strength(&self) -> u8;
    fn intelligence(&self) -> u8;
    fn politics(&self) -> u8;
    fn charm(&self) -> u8;
}

impl OfficerAbilityView for OfficerStats {
    fn leadership(&self) -> u8 {
        self.leadership
    }

    fn strength(&self) -> u8 {
        self.strength
    }

    fn intelligence(&self) -> u8 {
        self.intelligence
    }

    fn politics(&self) -> u8 {
        self.politics
    }

    fn charm(&self) -> u8 {
        self.charm
    }
}

impl OfficerAbilityView for Officer {
    fn leadership(&self) -> u8 {
        self.stats.leadership
    }

    fn strength(&self) -> u8 {
        self.stats.strength
    }

    fn intelligence(&self) -> u8 {
        self.stats.intelligence
    }

    fn politics(&self) -> u8 {
        self.stats.politics
    }

    fn charm(&self) -> u8 {
        self.stats.charm
    }
}

pub trait OfficerCatalog {
    type Error;

    fn officer_profiles(&self) -> Result<Vec<OfficerProfile>, Self::Error>;
    fn officer_profile(&self, officer_id: &str) -> Result<Option<OfficerProfile>, Self::Error>;
}
