use crate::engine::probability::{calculate_probability, sink_consumed_on_success, sink_gained_on_failure};
use crate::engine::sink::max_sink_for_level;
use crate::models::{
    item::ItemState,
    probability::ProbabilityResult,
    rune::{Rune, RuneTier},
};

/// Tier weight multiplier for EV scaling.
/// Pa runes apply 3× the flat value, Ra runes 10×.
fn tier_flat_multiplier(tier: &RuneTier) -> f64 {
    match tier {
        RuneTier::Ba => 1.0,
        RuneTier::Pa => 3.0,
        RuneTier::Ra => 10.0,
    }
}

/// Expected value of applying a rune to an item, measured in stat-points per attempt.
///
/// EV = Σ P(outcome_i) × stat_gain(outcome_i)
///
/// Returns a positive EV when the expected stat gain is favorable,
/// and a negative EV when the rune is more likely to hurt the item.
pub fn calculate_ev(item: &ItemState, rune: &Rune) -> f64 {
    let prob = calculate_probability(item, rune, 0);
    prob.expected_value
}

/// Expected kamas spent per net stat-point gained (efficiency metric).
///
/// Lower is better. Returns f64::INFINITY when success probability is zero.
pub fn calculate_ev_per_kama(item: &ItemState, rune: &Rune) -> f64 {
    let prob = calculate_probability(item, rune, 0);
    let ev = prob.expected_value;
    if ev > 0.0 {
        rune.price_estimate / ev
    } else {
        f64::INFINITY
    }
}

/// Compute the full EV breakdown including kama cost per stat-point and the
/// effective flat value considering rune tier.
#[derive(Debug, Clone)]
pub struct EvBreakdown {
    /// Raw expected stat gain per attempt (can be negative).
    pub expected_stat_gain: f64,
    /// Effective flat value adjusted for rune tier.
    pub effective_flat_value: f64,
    /// Expected kama cost per attempt.
    pub expected_kama_cost: f64,
    /// Kamas per net stat-point (efficiency).
    pub kama_per_stat_point: f64,
    /// Expected sink change per attempt (positive = sink increases).
    pub expected_sink_delta: f64,
    /// Probability result used to compute EV.
    pub probability: ProbabilityResult,
}

/// Full EV analysis for a rune application.
pub fn ev_breakdown(item: &ItemState, rune: &Rune) -> EvBreakdown {
    let prob = calculate_probability(item, rune, 0);
    let effective_flat = rune.flat_value * tier_flat_multiplier(&rune.tier);
    let success_prob = prob.success_rate + prob.critical_success_rate;

    let sink_pct = {
        let max = if item.max_sink_capacity > 0.0 {
            item.max_sink_capacity
        } else {
            max_sink_for_level(item.level)
        };
        if max > 0.0 { (item.sink_value / max).clamp(0.0, 1.0) } else { 0.0 }
    };

    // Expected sink delta per attempt:
    // Failures add rune.weight to puits; successes consume some puits.
    let sink_on_fail = sink_gained_on_failure(rune.weight);
    let sink_on_success = sink_consumed_on_success(rune.weight, sink_pct);
    let failure_prob = prob.failure_rate + prob.critical_failure_rate + prob.neutral_failure_rate;
    let expected_sink_delta =
        failure_prob * sink_on_fail - success_prob * sink_on_success;

    let kama_per_stat_point = if prob.expected_value > 0.0 {
        rune.price_estimate / prob.expected_value
    } else {
        f64::INFINITY
    };

    EvBreakdown {
        expected_stat_gain: prob.expected_value,
        effective_flat_value: effective_flat,
        expected_kama_cost: prob.expected_kama_cost,
        kama_per_stat_point,
        expected_sink_delta,
        probability: prob,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        item::{ItemCategory, ItemStat, ItemState},
        rune::{Rune, RuneTier},
    };

    fn make_item(sink: f64, max_sink: f64) -> ItemState {
        ItemState {
            item_id: 1,
            name: "Item".into(),
            level: 100,
            category: ItemCategory::Hat,
            stats: vec![ItemStat {
                stat_id: "force".into(),
                name: "Force".into(),
                current_value: 40.0,
                base_value: 50.0,
                min_value: 0.0,
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

    fn make_rune(tier: RuneTier, price: f64) -> Rune {
        Rune {
            rune_id: "r1".into(),
            name: "Force Ba".into(),
            stat_id: "force".into(),
            stat_name: "Force".into(),
            tier,
            flat_value: 1.0,
            weight: 1.0,
            price_estimate: price,
            last_price_update: 0,
        }
    }

    #[test]
    fn test_ev_returns_finite_value() {
        let item = make_item(200.0, 500.0);
        let rune = make_rune(RuneTier::Ba, 100.0);
        let ev = calculate_ev(&item, &rune);
        assert!(ev.is_finite(), "EV should be finite");
    }

    #[test]
    fn test_ra_has_higher_effective_flat() {
        let ra = make_rune(RuneTier::Ra, 100.0);
        let ba = make_rune(RuneTier::Ba, 100.0);
        assert_eq!(tier_flat_multiplier(&ra.tier), 10.0);
        assert_eq!(tier_flat_multiplier(&ba.tier), 1.0);
    }

    #[test]
    fn test_breakdown_expected_sink_delta_finite() {
        let item = make_item(100.0, 500.0);
        let rune = make_rune(RuneTier::Ba, 50.0);
        let bd = ev_breakdown(&item, &rune);
        assert!(bd.expected_sink_delta.is_finite());
    }

    #[test]
    fn test_kama_per_stat_point_infinity_when_negative_ev() {
        // Construct a scenario where EV should be near-zero or negative:
        // stat already over-maged, no sink, heavy rune.
        let item = ItemState {
            item_id: 1,
            name: "Item".into(),
            level: 1,
            category: ItemCategory::Hat,
            stats: vec![ItemStat {
                stat_id: "force".into(),
                name: "Force".into(),
                current_value: 60.0,
                base_value: 50.0,
                min_value: 0.0,
                max_value: 60.0,
                is_negative: false,
                weight: 1.0,
            }],
            sink_value: 0.0,
            max_sink_capacity: 100.0,
            is_identified: true,
            last_updated: 0,
        };
        let rune = make_rune(RuneTier::Ra, 1000.0);
        let bd = ev_breakdown(&item, &rune);
        // kama_per_stat_point should be infinity if EV ≤ 0
        if bd.expected_stat_gain <= 0.0 {
            assert_eq!(bd.kama_per_stat_point, f64::INFINITY);
        }
    }
}
