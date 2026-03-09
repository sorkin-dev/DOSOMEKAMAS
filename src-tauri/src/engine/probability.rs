use crate::engine::sink::{max_sink_for_level, sink_gained_from_failure};
use crate::models::{
    item::ItemState,
    probability::{ConfidenceInterval, ProbabilityResult},
    rune::{Rune, RuneTier},
};

/// Tier multiplier for base success rate.
/// Ra (x10) > Pa (x3) > Ba (x1) regarding unit weight, but tier also influences
/// the probability model: heavier runes are harder to land successfully.
fn tier_success_modifier(tier: &RuneTier) -> f64 {
    match tier {
        RuneTier::Ba => 1.0,
        RuneTier::Pa => 0.95,
        RuneTier::Ra => 0.85,
    }
}

/// Fraction of max sink capacity remaining for rune placement.
/// Returns a value in [0, 1]; 0 means the item is already at max capacity,
/// 1 means the puits is completely empty (worst case for success).
fn remaining_capacity_ratio(item: &ItemState, rune_weight: f64) -> f64 {
    let max_sink = if item.max_sink_capacity > 0.0 {
        item.max_sink_capacity
    } else {
        max_sink_for_level(item.level)
    };
    // Available weight budget: max_sink minus what is already over-maged.
    // If sink_value is > 0 the puits is partially filled, which is favorable,
    // but the "remaining capacity" for the rune itself is what matters.
    let over_maged_weight: f64 = item.stats.iter().map(|s| {
        let delta = s.current_value - s.max_value;
        if delta > 0.0 {
            let w = if s.weight > 0.0 { s.weight } else { 1.0 };
            delta * w
        } else {
            0.0
        }
    }).sum();
    let used = over_maged_weight;
    let available = (max_sink - used).max(0.0);
    if max_sink > 0.0 {
        ((available - rune_weight) / max_sink).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Determine if the targeted stat is already over-maged (above max_value).
fn is_stat_over_maged(item: &ItemState, rune: &Rune) -> bool {
    item.stats
        .iter()
        .find(|s| s.stat_id == rune.stat_id)
        .map(|s| s.current_value >= s.max_value)
        .unwrap_or(false)
}

/// Core probability calculation for a rune application attempt.
///
/// Returns the 5-outcome probability distribution:
/// - `critical_success_rate`: rune applies + bonus
/// - `success_rate`: rune applies normally
/// - `neutral_failure_rate`: nothing happens
/// - `failure_rate`: rune doesn't apply, minor stat loss
/// - `critical_failure_rate`: rune doesn't apply, significant stat loss
///
/// Probabilities are influenced by:
/// - Sink percentage (more puits → higher success)
/// - Rune weight vs. remaining capacity
/// - Rune tier
/// - Whether the stat is over-maged
pub fn calculate_probability(
    item: &ItemState,
    rune: &Rune,
    sample_size: u64,
) -> ProbabilityResult {
    let sink_pct = if item.max_sink_capacity > 0.0 {
        (item.sink_value / item.max_sink_capacity).clamp(0.0, 1.0)
    } else {
        let max = max_sink_for_level(item.level);
        if max > 0.0 {
            (item.sink_value / max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    };

    let tier_mod = tier_success_modifier(&rune.tier);
    let capacity_ratio = remaining_capacity_ratio(item, rune.weight);
    let over_maged = is_stat_over_maged(item, rune);

    // Base success probability: starts at 40%, boosted by sink, capped at 90%.
    // Sink contribution: a full puits (100%) can add up to +45% success.
    let base_success = 0.40_f64;
    let sink_bonus = sink_pct * 0.45;
    // Capacity penalty: if we're near max capacity the rune is harder to apply.
    let capacity_penalty = capacity_ratio * 0.05;
    // Over-maged penalty: landing a rune on an already over-maged stat is harder.
    let over_mage_penalty = if over_maged { 0.15 } else { 0.0 };

    let raw_success = (base_success + sink_bonus - capacity_penalty - over_mage_penalty)
        * tier_mod;
    let total_success = raw_success.clamp(0.05, 0.90);

    // Critical success: ~10% of successes become critical (bonus effect).
    let critical_success_rate = (total_success * 0.10).clamp(0.0, 0.15);
    let success_rate = total_success - critical_success_rate;

    // Remaining probability distributed among failure types.
    let failure_pool = 1.0 - total_success;

    // Critical failure: ~10% of failures (more impactful with heavy runes).
    let heavy_rune_factor = (rune.weight / 100.0).clamp(0.0, 0.3);
    let critical_failure_rate = (failure_pool * (0.05 + heavy_rune_factor)).clamp(0.0, 0.20);

    // Neutral failure: rises when sink is already high (item absorbs the attempt).
    let neutral_failure_rate = (failure_pool * 0.40 * (1.0 + sink_pct * 0.5)).clamp(0.0, failure_pool);

    let failure_rate =
        (failure_pool - critical_failure_rate - neutral_failure_rate).max(0.0);

    // Expected value: weighted sum of stat gain across outcomes.
    // Success → full flat_value, critical success → flat_value * 1.5
    let ev = critical_success_rate * rune.flat_value * 1.5
        + success_rate * rune.flat_value
        + neutral_failure_rate * 0.0
        + failure_rate * (-rune.flat_value * 0.1)
        + critical_failure_rate * (-rune.flat_value * 0.3);

    // Expected kama cost per net stat-point gained.
    let success_prob = critical_success_rate + success_rate;
    let expected_kama_cost = if success_prob > 0.0 {
        rune.price_estimate / success_prob
    } else {
        f64::INFINITY
    };

    let confidence_interval = wilson_confidence_interval(total_success, sample_size, 1.96);

    ProbabilityResult {
        success_rate,
        critical_success_rate,
        neutral_failure_rate,
        failure_rate,
        critical_failure_rate,
        expected_value: ev,
        expected_kama_cost,
        confidence_interval,
        sample_size,
    }
}

/// Wilson score confidence interval for a proportion `p` with `n` observations.
/// `z` is the z-score for the desired confidence level (1.96 ≈ 95%).
pub fn wilson_confidence_interval(p: f64, n: u64, z: f64) -> ConfidenceInterval {
    if n == 0 {
        return ConfidenceInterval {
            lower: 0.0,
            upper: 1.0,
            confidence_level: 0.95,
        };
    }
    let nf = n as f64;
    let z2 = z * z;
    let denom = 1.0 + z2 / nf;
    let center = (p + z2 / (2.0 * nf)) / denom;
    let spread = (p * (1.0 - p) / nf + z2 / (4.0 * nf * nf)).sqrt() * z / denom;

    ConfidenceInterval {
        lower: (center - spread).clamp(0.0, 1.0),
        upper: (center + spread).clamp(0.0, 1.0),
        confidence_level: 0.95,
    }
}

/// Estimate the sink consumed on a successful application of a rune.
/// A success "uses up" some accumulated puits proportional to the rune weight.
pub fn sink_consumed_on_success(rune_weight: f64, sink_pct: f64) -> f64 {
    rune_weight * sink_pct
}

/// Estimate the sink gained on a failure.
pub fn sink_gained_on_failure(rune_weight: f64) -> f64 {
    sink_gained_from_failure(rune_weight)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        item::{ItemCategory, ItemStat, ItemState},
        rune::{Rune, RuneTier},
    };

    fn make_rune(weight: f64, flat_value: f64, tier: RuneTier) -> Rune {
        Rune {
            rune_id: "r1".into(),
            name: "Force Ba".into(),
            stat_id: "force".into(),
            stat_name: "Force".into(),
            tier,
            flat_value,
            weight,
            price_estimate: 100.0,
            last_price_update: 0,
        }
    }

    fn make_item(level: u32, sink: f64, max_sink: f64) -> ItemState {
        ItemState {
            item_id: 1,
            name: "Item".into(),
            level,
            category: ItemCategory::Hat,
            stats: vec![ItemStat {
                stat_id: "force".into(),
                name: "Force".into(),
                current_value: 40.0,
                base_value: 50.0,
                min_value: 40.0,
                max_value: 60.0,
                is_negative: false,
                weight: 1.0,
            }],
            sink_value: sink,
            max_sink_capacity: max_sink,
            is_identified: true,
            last_updated: 0,
        }
    }

    #[test]
    fn test_probabilities_sum_to_one() {
        let item = make_item(100, 200.0, 500.0);
        let rune = make_rune(5.0, 1.0, RuneTier::Ba);
        let result = calculate_probability(&item, &rune, 0);
        let total = result.success_rate
            + result.critical_success_rate
            + result.neutral_failure_rate
            + result.failure_rate
            + result.critical_failure_rate;
        assert!((total - 1.0).abs() < 1e-9, "Probabilities must sum to 1, got {total}");
    }

    #[test]
    fn test_high_sink_increases_success() {
        let low_sink = make_item(100, 0.0, 500.0);
        let high_sink = make_item(100, 490.0, 500.0);
        let rune = make_rune(5.0, 1.0, RuneTier::Ba);
        let low_result = calculate_probability(&low_sink, &rune, 0);
        let high_result = calculate_probability(&high_sink, &rune, 0);
        let low_total = low_result.success_rate + low_result.critical_success_rate;
        let high_total = high_result.success_rate + high_result.critical_success_rate;
        assert!(
            high_total > low_total,
            "High sink should increase success rate: {high_total} > {low_total}"
        );
    }

    #[test]
    fn test_wilson_ci_zero_samples() {
        let ci = wilson_confidence_interval(0.5, 0, 1.96);
        assert_eq!(ci.lower, 0.0);
        assert_eq!(ci.upper, 1.0);
    }

    #[test]
    fn test_wilson_ci_large_sample() {
        // For p=0.7 and n=10,000, the 95% CI half-width is z*sqrt(p*(1-p)/n) ≈ 1.96*sqrt(0.21/10000) ≈ 0.009.
        // So the interval should be approximately [0.691, 0.709]. Using slightly wider bounds [0.68, 0.72]
        // to account for the Wilson correction factor.
        let ci = wilson_confidence_interval(0.7, 10_000, 1.96);
        assert!(ci.lower > 0.68 && ci.upper < 0.72, "95% CI around p=0.7 with n=10000 should be tight");
    }

    #[test]
    fn test_expected_kama_cost_positive() {
        let item = make_item(100, 100.0, 500.0);
        let rune = make_rune(5.0, 1.0, RuneTier::Ba);
        let result = calculate_probability(&item, &rune, 0);
        assert!(result.expected_kama_cost > 0.0);
    }
}

