use serde::{Deserialize, Serialize};

/// Rune tier (Ba, Pa, Ra).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuneTier {
    Ba,
    Pa,
    Ra,
}

/// A forgemagie rune.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rune {
    pub rune_id: String,
    pub name: String,
    pub stat_id: String,
    pub stat_name: String,
    pub tier: RuneTier,
    pub flat_value: f64,
    pub weight: f64,
    pub price_estimate: f64,
    pub last_price_update: u64,
}

/// Outcome of applying a rune.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuneOutcome {
    CriticalSuccess,
    Success,
    NeutralFailure,
    Failure,
    CriticalFailure,
}

/// Change in a single stat after a rune application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatChange {
    pub stat_id: String,
    pub previous_value: f64,
    pub new_value: f64,
    pub delta: f64,
}

/// Result of applying a rune.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuneApplicationResult {
    pub rune: Rune,
    pub outcome: RuneOutcome,
    pub stat_changes: Vec<StatChange>,
    pub sink_delta: f64,
    pub timestamp: u64,
}
