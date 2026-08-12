use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, poise::ChoiceParameter)]
pub enum CodexType {
    #[name = "operator"]
    Operator,
    #[name = "weapon"]
    Weapon,
    #[name = "equipment"]
    Equipment,
}

impl CodexType {
    pub fn path(self) -> &'static str {
        match self {
            Self::Operator => "operator",
            Self::Weapon => "weapon",
            Self::Equipment => "equipment",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub equipment: String,
    pub operators: ChunkIndex,
    pub weapons: ChunkIndex,
}

#[derive(Debug, Deserialize)]
pub struct ChunkIndex {
    pub files: HashMap<String, String>,
    pub index: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIndexEntry {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeaponIndexEntry {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentSet {
    pub id: String,
    pub image_url: Option<String>,
    pub name: String,
    pub pieces: Vec<EquipmentPiece>,
    pub set_bonuses: Vec<SetBonus>,
}

#[derive(Debug, Deserialize)]
pub struct EquipmentPiece {
    pub name: String,
    pub rarity: Option<u8>,
    #[serde(rename = "type")]
    pub equipment_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBonus {
    #[serde(default)]
    pub blackboard: Vec<BlackboardEntry>,
    pub description: String,
    pub name: String,
    pub pieces: u8,
}

#[derive(Debug, Deserialize)]
pub struct BlackboardEntry {
    pub key: String,
    pub value: Option<BlackboardValue>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BlackboardValue {
    Number(f64),
    Text(String),
}

#[derive(Debug, Deserialize)]
pub struct OperatorPayload {
    pub details: Option<OperatorDetails>,
    pub operator: Operator,
}

#[derive(Debug, Deserialize)]
pub struct OperatorDetails {
    pub description: Option<String>,
    pub element: Option<NamedEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {
    #[serde(default)]
    pub combat_skills: Vec<Skill>,
    pub max_level: u16,
    pub name: String,
    pub portrait_url: Option<String>,
    pub profession: String,
    pub rarity: Option<u8>,
    pub voice_actors: Vec<VoiceActor>,
    pub weapon_type: String,
}

#[derive(Debug, Deserialize)]
pub struct VoiceActor {
    pub language: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct NamedEntry {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Skill {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weapon {
    pub description: String,
    pub image_url: Option<String>,
    pub max_level: u16,
    pub name: String,
    pub rarity: Option<u8>,
    pub skills: Vec<Skill>,
    #[serde(rename = "type")]
    pub weapon_type: String,
}

pub trait CatalogEntry {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
}

impl CatalogEntry for OperatorIndexEntry {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl CatalogEntry for WeaponIndexEntry {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl CatalogEntry for EquipmentSet {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}
