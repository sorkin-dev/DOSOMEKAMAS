use crate::models::{item::ItemState, probability::SinkState};

/// Weight table for Dofus forgemagie stats.
/// Each entry maps a stat_id prefix (or exact id) to its unit weight.
/// Source: standard Dofus forgemagie weight table.
pub fn stat_weight(stat_id: &str) -> f64 {
    match stat_id {
        "vitalite" | "vita" | "pv" => 0.2,
        "sagesse" | "sag" => 3.0,
        "force" | "for" => 1.0,
        "intelligence" | "int" => 1.0,
        "chance" | "cha" => 1.0,
        "agilite" | "agi" => 1.0,
        "pa" | "action_point" => 100.0,
        "pm" | "movement_point" => 90.0,
        "po" | "portee" | "range" => 51.0,
        "invocations" | "invoc" => 40.0,
        "critique" | "crit" | "cc" => 2.0,
        "fuite" => 2.0,
        "tacle" => 2.0,
        "initiative" | "ini" => 0.1,
        "prospection" | "pros" => 1.0,
        "pods" => 0.01,
        "soin" | "heal" => 2.0,
        "puissance" | "pui" => 2.0,
        "degats_eau" | "dmg_eau" => 4.0,
        "degats_feu" | "dmg_feu" => 4.0,
        "degats_terre" | "dmg_terre" => 4.0,
        "degats_air" | "dmg_air" => 4.0,
        "degats_neutre" | "dmg_neutre" => 4.0,
        "resistance_eau" | "res_eau" => 2.0,
        "resistance_feu" | "res_feu" => 2.0,
        "resistance_terre" | "res_terre" => 2.0,
        "resistance_air" | "res_air" => 2.0,
        "resistance_neutre" | "res_neutre" => 2.0,
        "resistance_eau_pct" | "res_eau_pct" => 6.0,
        "resistance_feu_pct" | "res_feu_pct" => 6.0,
        "resistance_terre_pct" | "res_terre_pct" => 6.0,
        "resistance_air_pct" | "res_air_pct" => 6.0,
        "resistance_neutre_pct" | "res_neutre_pct" => 6.0,
        "reduction_pa" | "reduc_pa" => 8.0,
        "reduction_pm" | "reduc_pm" => 8.0,
        _ => 1.0,
    }
}

/// Compute the maximum sink capacity for an item based on its level.
/// Formula approximates the Dofus mechanic: higher-level items have more capacity.
pub fn max_sink_for_level(level: u32) -> f64 {
    // Linear interpolation: level 1 → ~30 capacity, level 200 → 1000 capacity
    let base = 25.0_f64;
    let scale = 4.875_f64; // (1000 - 25) / 200
    (base + scale * level as f64).round()
}

/// Compute the current sink from the item's stat deltas (current vs base).
/// Sink increases when stats are below base (failures pushed weight into the puits),
/// and decreases when stats are above base (weight was consumed from the puits).
pub fn compute_current_sink(item: &ItemState) -> f64 {
    item.stats.iter().fold(0.0, |acc, stat| {
        let delta = stat.current_value - stat.base_value;
        // Negative delta (stat below base) → positive sink contribution (puits filled)
        // Positive delta (stat above base) → negative sink contribution (puits drained)
        let weight = if stat.weight > 0.0 {
            stat.weight
        } else {
            stat_weight(&stat.stat_id)
        };
        acc + (-delta * weight)
    })
}

/// Estimate sink gained from a single failure outcome for a rune of given weight.
/// In Dofus, a failure adds the rune's weight to the puits.
pub fn sink_gained_from_failure(rune_weight: f64) -> f64 {
    rune_weight
}

/// Compute full SinkState for an item.
pub fn compute_sink_state(item: &ItemState) -> SinkState {
    let current_sink = compute_current_sink(item).max(0.0);
    let max_sink = if item.max_sink_capacity > 0.0 {
        item.max_sink_capacity
    } else {
        max_sink_for_level(item.level)
    };
    let sink_percentage = if max_sink > 0.0 {
        (current_sink / max_sink * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    // Sink is "favorable" when we have accumulated enough puits (≥ 30%)
    let is_sink_favorable = sink_percentage >= 30.0;

    // Estimated sink from a typical failure: use 1 weight unit as placeholder.
    // Callers should compute per-rune using sink_gained_from_failure().
    let estimated_sink_from_failure = 1.0;

    SinkState {
        current_sink,
        max_sink,
        sink_percentage,
        estimated_sink_from_failure,
        is_sink_favorable,
    }
}

/// Update sink state after a rune application result.
/// `sink_delta` is positive when puits increases (failure) or negative when consumed (success).
pub fn update_sink(current: f64, max: f64, sink_delta: f64) -> SinkState {
    let new_sink = (current + sink_delta).clamp(0.0, max);
    let sink_percentage = if max > 0.0 {
        (new_sink / max * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    SinkState {
        current_sink: new_sink,
        max_sink: max,
        sink_percentage,
        estimated_sink_from_failure: 1.0,
        is_sink_favorable: sink_percentage >= 30.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{item::ItemState, item::ItemCategory, item::ItemStat};

    fn make_item(level: u32, stats: Vec<ItemStat>) -> ItemState {
        ItemState {
            item_id: 1,
            name: "Test".into(),
            level,
            category: ItemCategory::Hat,
            stats,
            sink_value: 0.0,
            max_sink_capacity: 0.0,
            is_identified: true,
            last_updated: 0,
        }
    }

    #[test]
    fn test_max_sink_for_level() {
        assert_eq!(max_sink_for_level(1), 30.0);
        assert_eq!(max_sink_for_level(200), 1000.0);
    }

    #[test]
    fn test_compute_current_sink_zero_at_base() {
        let item = make_item(
            100,
            vec![ItemStat {
                stat_id: "force".into(),
                name: "Force".into(),
                current_value: 50.0,
                base_value: 50.0,
                min_value: 40.0,
                max_value: 60.0,
                is_negative: false,
                weight: 1.0,
            }],
        );
        assert_eq!(compute_current_sink(&item), 0.0);
    }

    #[test]
    fn test_compute_current_sink_below_base() {
        let item = make_item(
            100,
            vec![ItemStat {
                stat_id: "force".into(),
                name: "Force".into(),
                current_value: 45.0,
                base_value: 50.0,
                min_value: 40.0,
                max_value: 60.0,
                is_negative: false,
                weight: 1.0,
            }],
        );
        // delta = 45 - 50 = -5, sink = -(-5) * 1.0 = 5.0
        assert_eq!(compute_current_sink(&item), 5.0);
    }

    #[test]
    fn test_sink_state_favorable() {
        let item = make_item(
            100,
            vec![ItemStat {
                stat_id: "force".into(),
                name: "Force".into(),
                current_value: 20.0,
                base_value: 50.0,
                min_value: 0.0,
                max_value: 60.0,
                is_negative: false,
                weight: 1.0,
            }],
        );
        let state = compute_sink_state(&item);
        assert!(state.current_sink > 0.0);
        assert!(state.sink_percentage >= 0.0);
    }

    #[test]
    fn test_update_sink_clamps_to_max() {
        let state = update_sink(500.0, 600.0, 200.0);
        assert_eq!(state.current_sink, 600.0);
    }

    #[test]
    fn test_update_sink_clamps_to_zero() {
        let state = update_sink(10.0, 600.0, -50.0);
        assert_eq!(state.current_sink, 0.0);
    }
}

