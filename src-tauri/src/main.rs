#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dosomekamas_lib::errors::AppError;

#[tauri::command]
fn get_app_info() -> Result<serde_json::Value, AppError> {
    Ok(serde_json::json!({
        "name": "DOSOMEKAMAS",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Assistant Forgemagie — Dofus 3.5"
    }))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_app_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
