use crate::tools::models::DownloadMap;
use crate::tools::voice::VoiceRecording;
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde::Serialize;
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tower_http::cors::{Any, CorsLayer};

use crate::event_bus::EventBus;
use crate::mcp;
use crate::tools::{clipboard, models as models_tool, notes, ocr, search, voice};

pub const DEFAULT_PORT: u16 = 47821;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub session_token: String,
    pub port: u16,
    pub event_bus: EventBus,
    /// Used by the clipboard monitor to skip re-inserting just-recopied content.
    pub clipboard_suppress_tx: watch::Sender<u64>,
    /// Tracks in-progress model downloads (model_id → DownloadState).
    pub download_states: DownloadMap,
    /// Holds the ffmpeg child process while a voice recording is in progress.
    pub voice_recording: VoiceRecording,
}

#[derive(Debug, Serialize)]
pub struct AppError {
    pub message: String,
    pub code: u16,
}

#[allow(dead_code)]
impl AppError {
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code,
        }
    }
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(500, message)
    }
    pub fn unauthorized() -> Self {
        Self::new(401, "Unauthorized")
    }
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::new(404, format!("Not found: {}", resource.into()))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(json!({ "error": self.message }))).into_response()
    }
}

// Allow AppError to be used as a Tauri command error type
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// ── Auth middleware ──────────────────────────────────────────────────────────

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    let token_valid = token
        .map(|t| t == state.session_token.as_str())
        .unwrap_or(false);

    log::info!(
        "→ {} {} | auth: {}",
        method,
        path,
        token.map_or("none", |_| "present")
    );

    let resp = if token_valid {
        next.run(req).await
    } else {
        log::warn!("Auth failed — token mismatch or missing");
        AppError::unauthorized().into_response()
    };

    log::info!("← {} {} | status: {}", method, path, resp.status());
    resp
}

// ── Route handlers ───────────────────────────────────────────────────────────

async fn health_handler() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

async fn shell_handler(State(state): State<Arc<AppState>>) -> Response {
    // Shell is served as a static file; this route handles the root redirect.
    // The actual shell.html is loaded via Tauri's frontendDist in production
    // and devUrl in dev. This handler is a fallback for direct Axum access.
    let html = match tokio::fs::read_to_string("../ui/shell.html").await {
        Ok(content) => content
            .replace("{{SESSION_TOKEN}}", &state.session_token)
            .replace("{{API_PORT}}", &state.port.to_string()),
        Err(_) => format!(
            r#"<!DOCTYPE html><html><body><script>
            window.__SESSION_TOKEN__ = '{}';
            window.__API_PORT__ = {};
            </script><p>Loading...</p></body></html>"#,
            state.session_token, state.port
        ),
    };
    (StatusCode::OK, Html(html)).into_response()
}

async fn tool_panel_handler(Path(tool_name): Path<String>) -> impl IntoResponse {
    let path = format!("../ui/tools/{}/index.html", tool_name);
    match tokio::fs::read_to_string(&path).await {
        Ok(html) => (StatusCode::OK, Html(html)).into_response(),
        Err(_) => (
            StatusCode::OK,
            Html(format!(
                r#"<div id="tool-panel" class="p-6">
                  <h2 class="text-xl font-semibold capitalize">{}</h2>
                  <p class="text-gray-400 mt-2">Coming in a future phase.</p>
                </div>"#,
                tool_name
            )),
        )
            .into_response(),
    }
}

async fn settings_get_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT key, value FROM settings ORDER BY key")
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

    let obj: serde_json::Map<String, serde_json::Value> = rows
        .into_iter()
        .filter_map(|(k, v)| serde_json::from_str(&v).ok().map(|val| (k, val)))
        .collect();

    Json(json!({ "settings": obj }))
}

async fn settings_post_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(obj) = body.as_object() {
        for (key, value) in obj {
            let _ = sqlx::query(
                "INSERT INTO settings (key, value) VALUES (?, ?)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )
            .bind(key)
            .bind(value.to_string())
            .execute(&state.db)
            .await;
        }
    }
    Json(json!({ "ok": true }))
}

// ── Router ───────────────────────────────────────────────────────────────────

pub fn build_router(state: Arc<AppState>) -> Router {
    let protected = Router::new()
        .route(
            "/api/settings",
            get(settings_get_handler).post(settings_post_handler),
        )
        .route(
            "/mcp",
            get(mcp::mcp_sse_handler).post(mcp::mcp_post_handler),
        )
        .merge(clipboard::router())
        .merge(models_tool::router())
        .merge(notes::router())
        .merge(ocr::router())
        .merge(search::router())
        .merge(voice::router())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // CORS: allow Tauri WebView (tauri://localhost) and direct browser access to
    // reach Axum. Authorization header is exposed so the preflight passes.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/tools/:tool_name", get(tool_panel_handler))
        .route("/health", get(health_handler))
        .route("/", get(shell_handler))
        .merge(protected)
        .layer(cors)
        .with_state(state)
}

// ── Port detection ───────────────────────────────────────────────────────────

/// Synchronous port detection for use inside Tauri's setup hook.
pub fn find_free_port_sync() -> u16 {
    let mut port = DEFAULT_PORT;
    loop {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
        port += 1;
    }
}

pub async fn start_server(state: Arc<AppState>, port: u16) {
    let app = build_router(state);
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .expect("Failed to bind Axum server");
    tracing::info!("Axum server listening on http://127.0.0.1:{port}");
    axum::serve(listener, app)
        .await
        .expect("Axum server crashed");
}
