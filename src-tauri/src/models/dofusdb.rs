use serde::{Deserialize, Serialize};

/// Localized string returned by DofusDB API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedString {
    pub fr: String,
    pub en: String,
    pub es: String,
    pub pt: String,
    pub de: String,
    pub it: String,
}

/// A single stat entry from a DofusDB item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DofusDbItemStat {
    pub stat_id: u32,
    pub name: LocalizedString,
    pub min: f64,
    pub max: f64,
    pub order: u32,
}

/// An item as returned by the DofusDB API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DofusDbItem {
    pub id: u32,
    pub name: LocalizedString,
    pub level: u32,
    pub type_id: u32,
    pub type_name: LocalizedString,
    pub icon_url: String,
    pub description: LocalizedString,
    pub stats: Vec<DofusDbItemStat>,
    pub set_id: Option<u32>,
    pub set_name: Option<LocalizedString>,
    pub conditions: Vec<String>,
    pub recipe_ids: Vec<u32>,
}

/// Rune info as returned by the DofusDB API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DofusDbRuneInfo {
    pub rune_id: u32,
    pub name: LocalizedString,
    pub stat_id: u32,
    pub stat_name: LocalizedString,
    pub weight: f64,
    pub value_ba: f64,
    pub value_pa: f64,
    pub value_ra: f64,
}

/// Generic paginated response from DofusDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DofusDbPaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub skip: u64,
    pub limit: u64,
}

/// API error from DofusDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DofusDbApiError {
    pub code: u32,
    pub message: String,
    pub details: Option<String>,
}

