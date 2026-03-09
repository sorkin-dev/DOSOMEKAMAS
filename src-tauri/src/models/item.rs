use serde::{Deserialize, Serialize};

/// Forgemagie item category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemCategory {
    Weapon,
    Hat,
    Cloak,
    Belt,
    Boots,
    Ring,
    Amulet,
    Shield,
    Dofus,
    Trophy,
    Pet,
    PetMount,
    Mount,
}

/// A single stat on an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemStat {
    pub stat_id: String,
    pub name: String,
    pub current_value: f64,
    pub base_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub is_negative: bool,
    pub weight: f64,
}

/// Full state of an item being forgemaged.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemState {
    pub item_id: u32,
    pub name: String,
    pub level: u32,
    pub category: ItemCategory,
    pub stats: Vec<ItemStat>,
    pub sink_value: f64,
    pub max_sink_capacity: f64,
    pub is_identified: bool,
    pub last_updated: u64,
}

