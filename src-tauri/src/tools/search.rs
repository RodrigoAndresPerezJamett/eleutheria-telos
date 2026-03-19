use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::server::AppState;

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    20
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_results(
    notes: &[(String, String)], // (id, title)
    clips: &[(String, String)], // (id, content_preview)
) -> String {
    if notes.is_empty() && clips.is_empty() {
        return r#"<p class="text-gray-500 text-sm px-3 py-2">No results.</p>"#.to_string();
    }

    let mut out = String::new();

    if !notes.is_empty() {
        out.push_str(
            "<p class=\"text-xs text-gray-500 uppercase tracking-wider px-3 pt-2 pb-1\">Notes</p>",
        );
        for (_id, title) in notes {
            let display = if title.is_empty() {
                "Untitled"
            } else {
                title.as_str()
            };
            // r##"..."## used because the HTML contains "# sequences (hx-target="#tool-panel")
            out.push_str(&format!(
                r##"<button class="w-full text-left px-3 py-2 hover:bg-gray-700 rounded text-sm text-gray-100 flex items-center gap-2"
                          hx-get="/tools/notes"
                          hx-target="#tool-panel"
                          hx-swap="innerHTML"
                          @click="paletteOpen = false"
                          >📝 {title}</button>"##,
                title = html_escape(display),
            ));
        }
    }

    if !clips.is_empty() {
        out.push_str("<p class=\"text-xs text-gray-500 uppercase tracking-wider px-3 pt-2 pb-1\">Clipboard</p>");
        for (id, preview) in clips {
            let truncated = if preview.len() > 80 {
                format!("{}…", &preview[..80])
            } else {
                preview.clone()
            };
            out.push_str(&format!(
                "<button class=\"w-full text-left px-3 py-2 hover:bg-gray-700 rounded text-sm text-gray-300 flex items-center gap-2\" \
                          hx-post=\"/api/clipboard/{id}/recopy\" \
                          @click=\"paletteOpen = false\" \
                          >📋 {preview}</button>",
                id = id,
                preview = html_escape(&truncated),
            ));
        }
    }

    out
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    if params.q.is_empty() {
        return Html(String::new());
    }

    // Notes: FTS5 MATCH
    let notes: Vec<(String, String)> = sqlx::query_as(
        "SELECT n.id, n.title
         FROM notes n
         JOIN notes_fts f ON f.rowid = n.rowid
         WHERE notes_fts MATCH ?
         ORDER BY n.pinned DESC, rank
         LIMIT ?",
    )
    .bind(&params.q)
    .bind(params.limit / 2 + params.limit % 2) // give notes the extra slot if odd
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Clipboard: LIKE search
    let pattern = format!("%{}%", params.q);
    let clips: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, content FROM clipboard
         WHERE content LIKE ?
         ORDER BY created_at DESC
         LIMIT ?",
    )
    .bind(&pattern)
    .bind(params.limit / 2)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Html(render_results(&notes, &clips))
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/search", get(search_handler))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        middleware,
    };
    use http_body_util::BodyExt;
    use tokio::sync::watch;
    use tower::ServiceExt;

    async fn make_test_state() -> Arc<AppState> {
        let db = sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("in-memory DB");
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("migrations");
        let (clipboard_suppress_tx, _) = watch::channel::<u64>(0);
        let download_states =
            std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
        let voice_recording = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let screen_recording = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        Arc::new(AppState {
            db,
            session_token: "test-token".to_string(),
            port: 0,
            event_bus: crate::event_bus::EventBus::new(),
            clipboard_suppress_tx,
            download_states,
            voice_recording,
            screen_recording,
        })
    }

    fn test_app(state: Arc<AppState>) -> axum::Router {
        use crate::server::auth_middleware;
        router()
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    async fn get(app: axum::Router, uri: &str) -> (StatusCode, String) {
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("Authorization", "Bearer test-token")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8_lossy(&bytes).to_string())
    }

    #[tokio::test]
    async fn empty_query_returns_empty() {
        let state = make_test_state().await;
        let (status, body) = get(test_app(state), "/api/search?q=").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn notes_and_clipboard_merged() {
        let state = make_test_state().await;
        let now = 1000i64;

        // Insert a note
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES ('n1', 'Rust ownership', 'borrow checker', 'borrow checker', '[]', ?, ?)",
        )
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

        // Insert a clipboard entry
        sqlx::query(
            "INSERT INTO clipboard (id, content, content_type, created_at)
             VALUES ('c1', 'borrow checker note', 'text', ?)",
        )
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

        let (status, body) = get(test_app(state), "/api/search?q=borrow").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Rust ownership"), "note should appear");
        assert!(
            body.contains("borrow checker note"),
            "clipboard should appear"
        );
        let pos_notes = body.find("Notes").unwrap();
        let pos_clips = body.find("Clipboard").unwrap();
        assert!(pos_notes < pos_clips, "notes ranked before clipboard");
    }

    #[tokio::test]
    async fn no_results_returns_message() {
        let state = make_test_state().await;
        let (status, body) = get(test_app(state), "/api/search?q=xyzzy_nonexistent").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("No results."));
    }
}
