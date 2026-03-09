#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dosomekamas_lib::engine::{
    calculate_ev, calculate_probability, compute_sink_state, get_recommendation,
};
use dosomekamas_lib::errors::AppError;
use dosomekamas_lib::models::item::ItemState;
use dosomekamas_lib::models::probability::{ForgeRecommendation, ProbabilityResult, SinkState};
use dosomekamas_lib::models::rune::Rune;
use dosomekamas_lib::ocr::parser::ParsedItemStats;

#[tauri::command]
fn get_app_info() -> Result<serde_json::Value, AppError> {
    Ok(serde_json::json!({
        "name": "DOSOMEKAMAS",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Assistant Forgemagie — Dofus 3.5"
    }))
}

/// Calculate the 5-outcome probability distribution for a rune applied to an item.
#[tauri::command]
fn calculate_probability_cmd(
    item_state: ItemState,
    rune: Rune,
) -> Result<ProbabilityResult, AppError> {
    Ok(calculate_probability(&item_state, &rune, 0))
}

/// Compute the current sink (puits) state for an item.
#[tauri::command]
fn calculate_sink_state(item_state: ItemState) -> Result<SinkState, AppError> {
    Ok(compute_sink_state(&item_state))
}

/// Return the best rune recommendation and ranked alternatives for an item.
#[tauri::command]
fn get_recommendation_cmd(
    item_state: ItemState,
    available_runes: Vec<Rune>,
) -> Result<ForgeRecommendation, AppError> {
    get_recommendation(&item_state, &available_runes)
}

/// Calculate the expected value (stat gain per attempt) for applying a rune.
#[tauri::command]
fn calculate_ev_cmd(item_state: ItemState, rune: Rune) -> Result<f64, AppError> {
    Ok(calculate_ev(&item_state, &rune))
}

/// Capture a screen region and return it as a base64-encoded PNG.
#[tauri::command]
fn capture_screen_region(x: i32, y: i32, width: u32, height: u32) -> Result<String, AppError> {
    dosomekamas_lib::ocr::capture::capture_region(x, y, width, height)
}

/// Accept a base64-encoded PNG image and return the OCR text.
#[tauri::command]
fn recognize_text_from_image(image_base64: String) -> Result<String, AppError> {
    dosomekamas_lib::ocr::recognize::recognize_text_from_image(image_base64)
}

/// Parse an OCR text string and return the structured item stats.
#[tauri::command]
fn parse_stats_from_text(text: String) -> Result<ParsedItemStats, AppError> {
    dosomekamas_lib::ocr::parser::parse_stats_from_text(text)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            calculate_probability_cmd,
            calculate_sink_state,
            get_recommendation_cmd,
            calculate_ev_cmd,
            capture_screen_region,
            recognize_text_from_image,
            parse_stats_from_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

