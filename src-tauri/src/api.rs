use serde_json::json;
use std::sync::Arc;
use tauri::command;

use crate::server::{AppError, AppState};

#[command]
pub async fn get_session_token(state: tauri::State<'_, Arc<AppState>>) -> Result<String, AppError> {
    Ok(state.session_token.clone())
}

#[command]
pub async fn get_api_port(state: tauri::State<'_, Arc<AppState>>) -> Result<u16, AppError> {
    Ok(state.port)
}

#[command]
pub async fn health_check() -> Result<serde_json::Value, AppError> {
    Ok(json!({ "status": "ok", "message": "Eleutheria Telos API is running" }))
}

#[command]
pub async fn get_config() -> Result<serde_json::Value, AppError> {
    let config = json!({
        "app_name": "Eleutheria Telos",
        "version": env!("CARGO_PKG_VERSION"),
        "phase": 1,
        "environment": {
            "rust_version": env!("CARGO_PKG_RUST_VERSION"),
            "tauri_version": "2.10.3"
        }
    });
    Ok(config)
}
