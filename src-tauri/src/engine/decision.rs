use crate::engine::{
    probability::calculate_probability,
    sink::compute_sink_state,
};
use crate::errors::AppError;
use crate::models::{
    item::ItemState,
    probability::{AlternativeRune, ForgeRecommendation, ProbabilityResult},
    rune::Rune,
};

/// Score a rune for the purpose of ranking recommendations.
/// Higher score = better recommendation.
/// The score combines:
/// - EV efficiency (kamas per stat-point, inverted so lower kama cost → higher score)
/// - Risk profile (prefer lower variance outcomes)
/// - Goal proximity (how much the rune brings the item closer to its max stats)
fn score_rune(item: &ItemState, rune: &Rune, prob: &ProbabilityResult) -> f64 {
    // EV contribution (normalised): stat-gain weighted by success probability.
    let ev_score = prob.expected_value.max(0.0);

    // Risk penalty: high critical failure probability is penalised.
    let risk_penalty = prob.critical_failure_rate * 3.0 + prob.failure_rate * 1.0;

    // Goal proximity: how far below max is the targeted stat.
    let goal_gap = item
        .stats
        .iter()
        .find(|s| s.stat_id == rune.stat_id)
        .map(|s| (s.max_value - s.current_value).max(0.0))
        .unwrap_or(0.0);

    // Normalise goal_gap contribution (assume max reasonable gap = 100 stat points).
    let goal_score = (goal_gap / 100.0).clamp(0.0, 1.0);

    // Kama efficiency (inverted): prefer cheaper runes when EV is equal.
    let kama_efficiency = if prob.expected_kama_cost.is_finite() && prob.expected_kama_cost > 0.0 {
        1.0 / prob.expected_kama_cost
    } else {
        0.0
    };

    ev_score * 2.0 + goal_score * 1.5 + kama_efficiency * 0.5 - risk_penalty
}

/// Build a human-readable French reasoning string for the recommendation.
fn build_reasoning(
    item: &ItemState,
    rune: &Rune,
    prob: &ProbabilityResult,
    score: f64,
) -> String {
    let stat_name = &rune.stat_name;
    let success_pct = ((prob.success_rate + prob.critical_success_rate) * 100.0).round();
    let ev = prob.expected_value;
    let kama = prob.expected_kama_cost;

    let sink_info = {
        let max = item.max_sink_capacity;
        let pct = if max > 0.0 {
            (item.sink_value / max * 100.0).round()
        } else {
            0.0
        };
        if pct >= 30.0 {
            format!("Le puits est favorable ({pct:.0}%).")
        } else {
            format!("Le puits est faible ({pct:.0}%) — les chances de succès sont réduites.")
        }
    };

    let goal_gap = item
        .stats
        .iter()
        .find(|s| s.stat_id == rune.stat_id)
        .map(|s| s.max_value - s.current_value)
        .unwrap_or(0.0);

    let proximity_msg = if goal_gap <= 0.0 {
        format!("La stat {stat_name} est déjà au maximum — attention aux sur-magie.")
    } else if goal_gap <= rune.flat_value {
        format!("Un seul jet suffirait à atteindre le maximum de {stat_name}.")
    } else {
        format!(
            "Il manque encore {goal_gap:.0} points de {stat_name} pour atteindre le maximum."
        )
    };

    format!(
        "Rune recommandée : {name} (tier {tier:?}). \
         Probabilité de succès estimée : {success_pct:.0}%. \
         Espérance de gain : {ev:.2} point(s). \
         Coût estimé par succès : {kama:.0} kamas. \
         {sink_info} \
         {proximity_msg} \
         Score global : {score:.3}.",
        name = rune.name,
        tier = rune.tier,
    )
}

/// Evaluate all available runes for the item and return a ranked recommendation.
///
/// Returns an `AppError::Calculation` if no runes are provided.
pub fn get_recommendation(
    item: &ItemState,
    available_runes: &[Rune],
) -> Result<ForgeRecommendation, AppError> {
    if available_runes.is_empty() {
        return Err(AppError::Calculation(
            "Aucune rune disponible pour évaluation.".into(),
        ));
    }

    let sink_state = compute_sink_state(item);

    // Score all runes.
    let mut scored: Vec<(f64, &Rune, ProbabilityResult)> = available_runes
        .iter()
        .map(|rune| {
            let prob = calculate_probability(item, rune, 0);
            let score = score_rune(item, rune, &prob);
            (score, rune, prob)
        })
        .collect();

    // Sort descending by score.
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let (best_score, best_rune, best_prob) = scored.remove(0);
    let reasoning = build_reasoning(item, best_rune, &best_prob, best_score);

    // Build alternatives (up to 5 next best).
    let alternatives: Vec<AlternativeRune> = scored
        .into_iter()
        .take(5)
        .map(|(alt_score, alt_rune, alt_prob)| {
            let tradeoff = build_tradeoff_description(best_rune, alt_rune, &best_prob, &alt_prob, alt_score);
            AlternativeRune {
                rune: alt_rune.clone(),
                expected_outcome: alt_prob,
                tradeoff_description: tradeoff,
            }
        })
        .collect();

    Ok(ForgeRecommendation {
        recommended_rune: best_rune.clone(),
        expected_outcome: best_prob,
        reasoning,
        alternative_runes: alternatives,
        current_sink_state: sink_state,
    })
}

/// Build a French tradeoff description comparing an alternative rune to the best one.
fn build_tradeoff_description(
    best: &Rune,
    alt: &Rune,
    best_prob: &ProbabilityResult,
    alt_prob: &ProbabilityResult,
    alt_score: f64,
) -> String {
    let best_success = (best_prob.success_rate + best_prob.critical_success_rate) * 100.0;
    let alt_success = (alt_prob.success_rate + alt_prob.critical_success_rate) * 100.0;
    let diff_success = alt_success - best_success;

    let success_msg = if diff_success >= 1.0 {
        format!(
            "Taux de succès légèrement supérieur ({alt_success:.0}% vs {best_success:.0}% pour {best_name}).",
            best_name = best.name
        )
    } else if diff_success <= -1.0 {
        format!(
            "Taux de succès inférieur ({alt_success:.0}% vs {best_success:.0}% pour {best_name}).",
            best_name = best.name
        )
    } else {
        "Taux de succès similaire.".into()
    };

    let ev_diff = alt_prob.expected_value - best_prob.expected_value;
    let ev_msg = if ev_diff > 0.05 {
        format!("EV légèrement meilleure ({ev:.2} vs {best_ev:.2}).", ev = alt_prob.expected_value, best_ev = best_prob.expected_value)
    } else if ev_diff < -0.05 {
        format!("EV inférieure ({ev:.2} vs {best_ev:.2}).", ev = alt_prob.expected_value, best_ev = best_prob.expected_value)
    } else {
        "EV équivalente.".into()
    };

    format!(
        "Alternative : {name} (score {alt_score:.3}). {success_msg} {ev_msg}",
        name = alt.name,
    )
}

/// Evaluate a single rune and return its expected outcome without full ranking.
pub fn evaluate_rune(item: &ItemState, rune: &Rune) -> ProbabilityResult {
    calculate_probability(item, rune, 0)
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
            name: "Chapeau Test".into(),
            level: 100,
            category: ItemCategory::Hat,
            stats: vec![
                ItemStat {
                    stat_id: "force".into(),
                    name: "Force".into(),
                    current_value: 40.0,
                    base_value: 50.0,
                    min_value: 40.0,
                    max_value: 60.0,
                    is_negative: false,
                    weight: 1.0,
                },
                ItemStat {
                    stat_id: "vitalite".into(),
                    name: "Vitalité".into(),
                    current_value: 80.0,
                    base_value: 100.0,
                    min_value: 80.0,
                    max_value: 120.0,
                    is_negative: false,
                    weight: 0.2,
                },
            ],
            sink_value: sink,
            max_sink_capacity: max_sink,
            is_identified: true,
            last_updated: 0,
        }
    }

    fn make_rune(stat_id: &str, stat_name: &str, tier: RuneTier, price: f64) -> Rune {
        Rune {
            rune_id: format!("{stat_id}_{tier:?}"),
            name: format!("{stat_name} {:?}", tier),
            stat_id: stat_id.into(),
            stat_name: stat_name.into(),
            tier,
            flat_value: 1.0,
            weight: 1.0,
            price_estimate: price,
            last_price_update: 0,
        }
    }

    #[test]
    fn test_recommendation_returns_best_rune() {
        let item = make_item(200.0, 500.0);
        let runes = vec![
            make_rune("force", "Force", RuneTier::Ba, 100.0),
            make_rune("vitalite", "Vitalité", RuneTier::Ba, 50.0),
        ];
        let rec = get_recommendation(&item, &runes).unwrap();
        assert!(!rec.recommended_rune.rune_id.is_empty());
        assert!(!rec.reasoning.is_empty());
    }

    #[test]
    fn test_recommendation_empty_runes_returns_error() {
        let item = make_item(0.0, 500.0);
        let result = get_recommendation(&item, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_alternatives_capped_at_five() {
        let item = make_item(100.0, 500.0);
        let runes: Vec<Rune> = (0..10)
            .map(|i| Rune {
                rune_id: format!("r{i}"),
                name: format!("Rune {i}"),
                stat_id: "force".into(),
                stat_name: "Force".into(),
                tier: RuneTier::Ba,
                flat_value: 1.0,
                weight: 1.0,
                price_estimate: 100.0,
                last_price_update: 0,
            })
            .collect();
        let rec = get_recommendation(&item, &runes).unwrap();
        assert!(rec.alternative_runes.len() <= 5);
    }

    #[test]
    fn test_recommendation_reasoning_is_in_french() {
        let item = make_item(200.0, 500.0);
        let runes = vec![make_rune("force", "Force", RuneTier::Ba, 100.0)];
        let rec = get_recommendation(&item, &runes).unwrap();
        assert!(
            rec.reasoning.contains("Rune recommandée"),
            "Reasoning should be in French"
        );
    }

    #[test]
    fn test_score_rune_finite() {
        let item = make_item(200.0, 500.0);
        let rune = make_rune("force", "Force", RuneTier::Ba, 100.0);
        let prob = calculate_probability(&item, &rune, 0);
        let score = score_rune(&item, &rune, &prob);
        assert!(score.is_finite());
    }
}
