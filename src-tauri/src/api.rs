use serde_json::json;
use tauri::command;
use uuid::Uuid;

use crate::server::AppError;

#[command]
pub async fn get_session_token() -> Result<String, AppError> {
    Ok(Uuid::new_v4().to_string())
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
        "phase": 0,
        "environment": {
            "rust_version": env!("CARGO_PKG_RUST_VERSION"),
            "tauri_version": "2.10.3"
        }
    });
    Ok(config)
}
