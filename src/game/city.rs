use super::ids::{CityId, FactionId, OfficerId};
use super::model::MapPosition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CityScale {
    County,
    Commandery,
    RegionalCapital,
    ImperialCapital,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceConfidence {
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CityProfile {
    pub id: CityId,
    pub name: String,
    pub province: String,
    pub commandery: String,
    pub position: MapPosition,
    pub scale: CityScale,
    pub strategic_rank: u8,
    pub agriculture_base: u16,
    pub commerce_base: u16,
    pub defense_base: u16,
    pub population_min: u32,
    pub population_max: u32,
    pub confidence: SourceConfidence,
    pub notes: String,
}

pub trait CityProfileView {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn province(&self) -> &str;
    fn commandery(&self) -> &str;
    fn scale(&self) -> &CityScale;
    fn strategic_rank(&self) -> u8;
    fn confidence(&self) -> &SourceConfidence;
}

impl CityProfileView for CityProfile {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn province(&self) -> &str {
        &self.province
    }

    fn commandery(&self) -> &str {
        &self.commandery
    }

    fn scale(&self) -> &CityScale {
        &self.scale
    }

    fn strategic_rank(&self) -> u8 {
        self.strategic_rank
    }

    fn confidence(&self) -> &SourceConfidence {
        &self.confidence
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct City {
    pub id: CityId,
    pub name: String,
    pub faction_id: FactionId,
    pub position: MapPosition,
    pub population: u32,
    pub gold: i32,
    pub food: i32,
    pub troops: u32,
    pub training: u8,
    pub agriculture: u16,
    pub commerce: u16,
    pub defense: u16,
    pub order: u8,
    pub governor_id: Option<OfficerId>,
    pub profile: Option<CityProfile>,
}

impl City {
    pub fn clamp_fields(&mut self) {
        self.training = self.training.min(100);
        self.order = self.order.min(100);
        self.agriculture = self.agriculture.min(999);
        self.commerce = self.commerce.min(999);
        self.defense = self.defense.min(999);
        self.gold = self.gold.max(0);
        self.food = self.food.max(0);
    }
}

pub trait CityStateView {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn faction_id(&self) -> &str;
    fn population(&self) -> u32;
    fn troops(&self) -> u32;
    fn governor_id(&self) -> Option<&str>;
}

impl CityStateView for City {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn faction_id(&self) -> &str {
        &self.faction_id
    }

    fn population(&self) -> u32 {
        self.population
    }

    fn troops(&self) -> u32 {
        self.troops
    }

    fn governor_id(&self) -> Option<&str> {
        self.governor_id.as_deref()
    }
}

pub trait CityCatalog {
    type Error;

    fn city_profiles(&self) -> Result<Vec<CityProfile>, Self::Error>;
    fn city_profile(&self, city_id: &str) -> Result<Option<CityProfile>, Self::Error>;
}
