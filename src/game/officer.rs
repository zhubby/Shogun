use super::city::SourceConfidence;
use super::ids::{CityId, FactionId, OfficerId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OfficerStatus {
    Active,
    Wild,
    Unavailable,
    Dead,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OfficerProfile {
    pub id: OfficerId,
    pub name: String,
    pub courtesy_name: Option<String>,
    pub native_place: Option<String>,
    pub birth_year: Option<i32>,
    pub death_year: Option<i32>,
    pub stats: OfficerStats,
    pub tags: Vec<String>,
    pub confidence: SourceConfidence,
    pub notes: String,
}

pub trait OfficerProfileView {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn courtesy_name(&self) -> Option<&str>;
    fn native_place(&self) -> Option<&str>;
    fn birth_year(&self) -> Option<i32>;
    fn death_year(&self) -> Option<i32>;
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
