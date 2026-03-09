use serde::{Deserialize, Serialize};

use super::rune::Rune;

/// Confidence interval around a probability estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfidenceInterval {
    pub lower: f64,
    pub upper: f64,
    pub confidence_level: f64,
}

/// Result of a probability calculation for a rune application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbabilityResult {
    pub success_rate: f64,
    pub critical_success_rate: f64,
    pub neutral_failure_rate: f64,
    pub failure_rate: f64,
    pub critical_failure_rate: f64,
    pub expected_value: f64,
    pub expected_kama_cost: f64,
    pub confidence_interval: ConfidenceInterval,
    pub sample_size: u64,
}

/// Current sink (puits) state of an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SinkState {
    pub current_sink: f64,
    pub max_sink: f64,
    pub sink_percentage: f64,
    pub estimated_sink_from_failure: f64,
    pub is_sink_favorable: bool,
}

/// An alternative rune suggestion with its expected outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlternativeRune {
    pub rune: Rune,
    pub expected_outcome: ProbabilityResult,
    pub tradeoff_description: String,
}

/// Full recommendation from the optimization engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgeRecommendation {
    pub recommended_rune: Rune,
    pub expected_outcome: ProbabilityResult,
    pub reasoning: String,
    pub alternative_runes: Vec<AlternativeRune>,
    pub current_sink_state: SinkState,
}

