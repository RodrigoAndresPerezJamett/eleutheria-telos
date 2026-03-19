use serde_json::json;
use std::sync::Arc;
use tauri::command;

use crate::server::{AppError, AppState};

/// Returns the list of plugins that have sidebar entries, sorted by order.
/// Called from shell.html via Tauri invoke — avoids HTTP entirely for reliability.
#[command]
pub fn list_sidebar_plugins(
    state: tauri::State<'_, Arc<AppState>>,
) -> Vec<serde_json::Value> {
    let registry = state.plugin_registry.lock().unwrap();
    let mut plugins: Vec<_> = registry
        .values()
        .filter(|p| p.manifest.sidebar.as_ref().map(|s| s.show).unwrap_or(false))
        .map(|p| {
            json!({
                "id": p.manifest.id,
                "name": p.manifest.name,
                "icon": p.manifest.icon,
                "label": p.manifest.sidebar.as_ref().map(|s| s.label.clone()).unwrap_or_default(),
                "order": p.manifest.sidebar.as_ref().and_then(|s| s.order).unwrap_or(u32::MAX),
            })
        })
        .collect();
    plugins.sort_by_key(|p| p["order"].as_u64().unwrap_or(u64::MAX));
    plugins
}

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
