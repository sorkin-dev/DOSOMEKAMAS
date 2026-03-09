use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::errors::AppError;

/// A single parsed Dofus stat extracted from OCR text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedStat {
    /// Stat label (e.g. "Force", "Intelligence", "% Résistance Feu").
    pub name: String,
    /// Numeric value. For range patterns ("10 à 20 Sagesse") this is the midpoint.
    pub value: f64,
    /// `true` when the stat prefix is `"-"`.
    pub is_negative: bool,
}

/// A complete set of parsed stats for a Dofus item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedItemStats {
    /// Item name extracted from the first non-stat line, or `None` if not detected.
    pub item_name: Option<String>,
    /// All recognised stat entries.
    pub stats: Vec<ParsedStat>,
}

// ---------------------------------------------------------------------------
// Regex helpers
// ---------------------------------------------------------------------------

/// Compiled regex for a simple `+15 Force` / `-5 Intelligence` pattern.
fn re_simple() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?m)^\s*([+\-])?\s*(\d+(?:\.\d+)?)\s+([[:alpha:]%][[:alpha:]% éàâêîôùûçèïœæ/()'-]+?)\s*$",
        )
        .expect("invalid simple stat regex")
    })
}

/// Compiled regex for a range pattern `10 à 20 Sagesse`.
fn re_range() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?m)^\s*([+\-])?\s*(\d+(?:\.\d+)?)\s+à\s+(\d+(?:\.\d+)?)\s+([[:alpha:]%][[:alpha:]% éàâêîôùûçèïœæ/()'-]+?)\s*$",
        )
        .expect("invalid range stat regex")
    })
}

// ---------------------------------------------------------------------------
// Known Dofus stat labels (canonical forms for normalisation)
// ---------------------------------------------------------------------------
const KNOWN_STATS: &[&str] = &[
    "Force",
    "Intelligence",
    "Chance",
    "Agilité",
    "Vitalité",
    "Sagesse",
    "Puissance",
    "Dommages",
    "Soins",
    "PA",
    "PM",
    "Portée",
    "Invocations",
    "Prospection",
    "Initiative",
    "Pods",
    "% Résistance Terre",
    "% Résistance Feu",
    "% Résistance Eau",
    "% Résistance Air",
    "% Résistance Neutre",
    "Résistance Terre",
    "Résistance Feu",
    "Résistance Eau",
    "Résistance Air",
    "Résistance Neutre",
    "Esquive PA",
    "Esquive PM",
    "Retrait PA",
    "Retrait PM",
    "Tacle",
    "Fuite",
];

/// Normalise a raw stat name from OCR to the canonical form when possible.
fn normalise_stat_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let lower = trimmed.to_lowercase();
    // Case-insensitive match against known stat labels.
    for known in KNOWN_STATS {
        if known.eq_ignore_ascii_case(trimmed) || known.to_lowercase().contains(&lower) {
            return known.to_string();
        }
    }
    trimmed.to_string()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse raw OCR text and extract Dofus item stats.
///
/// The function attempts to identify an item name from the first line that
/// does not match any stat pattern, then extracts all recognisable stat lines.
pub fn parse_stats(text: &str) -> ParsedItemStats {
    let mut stats: Vec<ParsedStat> = Vec::new();
    let mut item_name: Option<String> = None;

    // First pass: collect range patterns.
    for cap in re_range().captures_iter(text) {
        let sign = cap.get(1).map_or("", |m| m.as_str());
        let lo: f64 = cap[2].parse().unwrap_or(0.0);
        let hi: f64 = cap[3].parse().unwrap_or(0.0);
        let name = normalise_stat_name(&cap[4]);
        let value = (lo + hi) / 2.0;
        let is_negative = sign == "-";
        stats.push(ParsedStat { name, value, is_negative });
    }

    // Second pass: simple patterns (+/- number stat_name).
    for cap in re_simple().captures_iter(text) {
        let sign = cap.get(1).map_or("", |m| m.as_str());
        let value: f64 = cap[2].parse().unwrap_or(0.0);
        let name = normalise_stat_name(&cap[3]);
        let is_negative = sign == "-";

        // Avoid duplicating stats already captured by the range pass.
        let already_captured = stats.iter().any(|s| s.name == name);
        if !already_captured {
            stats.push(ParsedStat { name, value, is_negative });
        }
    }

    // Item name detection: first non-empty, non-stat line.
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let is_stat = re_simple().is_match(trimmed) || re_range().is_match(trimmed);
        if !is_stat && item_name.is_none() {
            // Heuristic: item names don't start with a digit.
            if !trimmed.starts_with(|c: char| c.is_ascii_digit()) {
                item_name = Some(trimmed.to_string());
            }
        }
    }

    ParsedItemStats { item_name, stats }
}

/// Tauri command: parse an OCR text string and return the structured item stats.
#[tauri::command]
pub fn parse_stats_from_text(text: String) -> Result<ParsedItemStats, AppError> {
    Ok(parse_stats(&text))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_positive_stat() {
        let text = "+15 Force";
        let result = parse_stats(text);
        assert_eq!(result.stats.len(), 1);
        let s = &result.stats[0];
        assert_eq!(s.name, "Force");
        assert!((s.value - 15.0).abs() < f64::EPSILON);
        assert!(!s.is_negative);
    }

    #[test]
    fn test_parse_simple_negative_stat() {
        let text = "-5 Intelligence";
        let result = parse_stats(text);
        assert_eq!(result.stats.len(), 1);
        let s = &result.stats[0];
        assert_eq!(s.name, "Intelligence");
        assert!((s.value - 5.0).abs() < f64::EPSILON);
        assert!(s.is_negative);
    }

    #[test]
    fn test_parse_range_stat_midpoint() {
        let text = "10 à 20 Sagesse";
        let result = parse_stats(text);
        assert_eq!(result.stats.len(), 1);
        let s = &result.stats[0];
        assert_eq!(s.name, "Sagesse");
        assert!((s.value - 15.0).abs() < f64::EPSILON);
        assert!(!s.is_negative);
    }

    #[test]
    fn test_parse_multiple_stats() {
        let text = "+15 Force\n-5 Intelligence\n10 à 20 Sagesse";
        let result = parse_stats(text);
        assert_eq!(result.stats.len(), 3);
    }

    #[test]
    fn test_item_name_detected() {
        let text = "Chapeau du Bouftou\n+15 Force\n+10 Vitalité";
        let result = parse_stats(text);
        assert_eq!(result.item_name, Some("Chapeau du Bouftou".to_string()));
    }

    #[test]
    fn test_empty_text_returns_empty_stats() {
        let result = parse_stats("");
        assert!(result.stats.is_empty());
        assert!(result.item_name.is_none());
    }

    #[test]
    fn test_parse_stats_from_text_command() {
        let result = parse_stats_from_text("+20 Chance".into()).unwrap();
        assert_eq!(result.stats.len(), 1);
        assert_eq!(result.stats[0].name, "Chance");
    }

    #[test]
    fn test_negative_range_stat() {
        let text = "-10 à 20 Sagesse";
        let result = parse_stats(text);
        assert_eq!(result.stats.len(), 1);
        let s = &result.stats[0];
        assert!(s.is_negative);
        assert!((s.value - 15.0).abs() < f64::EPSILON);
    }
}
