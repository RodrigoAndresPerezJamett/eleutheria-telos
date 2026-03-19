use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Json},
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::event_bus::Event;
use crate::server::AppState;

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn content_hash(text: &str) -> u64 {
    let mut h = DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

fn render_entry_card(id: &str, content: &str, created_at: i64) -> String {
    let preview = if content.len() > 200 {
        format!("{}…", &content[..200])
    } else {
        content.to_string()
    };
    let escaped = html_escape(&preview);
    let ts = format_timestamp(created_at);
    // r##"..."## used because the HTML contains "# sequences (HTMX target="#clip-id")
    format!(
        r##"<div id="clip-{id}" class="group relative bg-gray-800 rounded-lg p-3 mb-2 hover:bg-gray-750">
  <pre class="text-sm text-gray-200 whitespace-pre-wrap break-words font-sans leading-relaxed">{escaped}</pre>
  <div class="flex items-center justify-between mt-2">
    <span class="text-xs text-gray-500">{ts}</span>
    <div class="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
      <button class="text-xs text-blue-400 hover:text-blue-300"
              hx-post="/api/clipboard/{id}/recopy"
              title="Copy to clipboard">Copy</button>
      <button class="text-xs text-red-400 hover:text-red-300"
              hx-delete="/api/clipboard/{id}"
              hx-target="#clip-{id}"
              hx-swap="outerHTML"
              hx-confirm="Delete this entry?"
              title="Delete">Delete</button>
    </div>
  </div>
</div>"##
    )
}

fn render_list(entries: &[(String, String, i64)]) -> String {
    if entries.is_empty() {
        return r#"<p class="text-gray-500 text-sm">Nothing copied yet.</p>"#.to_string();
    }
    entries
        .iter()
        .map(|(id, content, ts)| render_entry_card(id, content, *ts))
        .collect()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn format_timestamp(ts: i64) -> String {
    // ts is Unix seconds; display as a simple relative label
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(ts);
    let diff = now - ts;
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    let rows: Vec<(String, String, i64)> = if params.q.is_empty() {
        sqlx::query_as(
            "SELECT id, content, created_at FROM clipboard
             ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    } else {
        let pattern = format!("%{}%", params.q);
        sqlx::query_as(
            "SELECT id, content, created_at FROM clipboard
             WHERE content LIKE ?
             ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(&pattern)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    };

    Html(render_list(&rows))
}

pub async fn recopy_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let row: Option<(String,)> = sqlx::query_as("SELECT content FROM clipboard WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    let Some((content,)) = row else {
        return Json(json!({ "ok": false, "error": "not found" }));
    };

    // Suppress the monitor so it won't re-insert this copy
    let hash = content_hash(&content);
    let _ = state.clipboard_suppress_tx.send(hash);

    // Write to system clipboard (blocking — must be on a thread, not async executor)
    let result = tokio::task::spawn_blocking(move || {
        arboard::Clipboard::new()
            .and_then(|mut cb| cb.set_text(&content))
            .is_ok()
    })
    .await
    .unwrap_or(false);

    Json(json!({ "ok": result }))
}

pub async fn delete_one_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM clipboard WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();
    // Return empty string — HTMX outerHTML swap removes the card
    Html(String::new())
}

pub async fn clear_all_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    sqlx::query("DELETE FROM clipboard")
        .execute(&state.db)
        .await
        .ok();
    Json(json!({ "ok": true }))
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/clipboard",
            get(list_handler).delete(clear_all_handler),
        )
        .route("/api/clipboard/:id/recopy", post(recopy_handler))
        .route("/api/clipboard/:id", delete(delete_one_handler))
}

// ── Clipboard monitor ─────────────────────────────────────────────────────────

pub async fn start_monitor(state: Arc<AppState>) {
    // arboard is not Send; run entirely inside spawn_blocking
    let state_clone = state.clone();
    tokio::task::spawn_blocking(move || {
        // Seed last_hash from the most recent DB entry to avoid re-inserting on restart
        let last_content: Option<(String,)> = tauri::async_runtime::block_on(
            sqlx::query_as("SELECT content FROM clipboard ORDER BY created_at DESC LIMIT 1")
                .fetch_optional(&state_clone.db),
        )
        .unwrap_or(None);

        let mut last_hash: u64 = last_content
            .as_ref()
            .map(|(c,)| content_hash(c))
            .unwrap_or(0);

        let mut suppress_rx = state_clone.clipboard_suppress_tx.subscribe();

        let mut cb = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Clipboard init failed: {e}");
                return;
            }
        };

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));

            // Check suppress channel (non-blocking)
            if suppress_rx.has_changed().unwrap_or(false) {
                last_hash = *suppress_rx.borrow_and_update();
                continue;
            }

            let text = match cb.get_text() {
                Ok(t) => t,
                Err(_) => continue, // Normal on Wayland when no window has focus
            };

            if text.is_empty() {
                continue;
            }

            let hash = content_hash(&text);
            if hash == last_hash {
                continue;
            }
            last_hash = hash;

            let id = uuid::Uuid::new_v4().to_string();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let insert_result = tauri::async_runtime::block_on(
                sqlx::query(
                    "INSERT INTO clipboard (id, content, content_type, created_at)
                     VALUES (?, ?, 'text', ?)",
                )
                .bind(&id)
                .bind(&text)
                .bind(now)
                .execute(&state_clone.db),
            );

            if let Err(e) = insert_result {
                log::error!("Clipboard insert failed: {e}");
            } else {
                state_clone.event_bus.publish(Event::ClipboardChanged {
                    content: text,
                    content_type: "text".to_string(),
                });
            }
        }
    })
    .await
    .ok();
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
        let audio_recording = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let mcp_sessions =
            std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
        let plugin_registry =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let plugin_processes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        Arc::new(AppState {
            db,
            session_token: "test-token".to_string(),
            port: 0,
            event_bus: crate::event_bus::EventBus::new(),
            clipboard_suppress_tx,
            download_states,
            voice_recording,
            screen_recording,
            audio_recording,
            mcp_sessions,
            plugin_registry,
            plugin_processes,
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

    async fn call(app: axum::Router, method: Method, uri: &str) -> (StatusCode, String) {
        let req = Request::builder()
            .method(method)
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
    async fn list_empty() {
        let state = make_test_state().await;
        let (status, body) = call(test_app(state), Method::GET, "/api/clipboard").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Nothing copied yet."));
    }

    #[tokio::test]
    async fn list_entries_desc_order() {
        let state = make_test_state().await;
        sqlx::query(
            "INSERT INTO clipboard (id, content, content_type, created_at) VALUES (?, ?, 'text', ?)",
        )
        .bind("id-old")
        .bind("first item")
        .bind(1000i64)
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO clipboard (id, content, content_type, created_at) VALUES (?, ?, 'text', ?)",
        )
        .bind("id-new")
        .bind("second item")
        .bind(2000i64)
        .execute(&state.db)
        .await
        .unwrap();

        let (status, body) = call(test_app(state), Method::GET, "/api/clipboard").await;
        assert_eq!(status, StatusCode::OK);
        let pos_new = body.find("second item").unwrap();
        let pos_old = body.find("first item").unwrap();
        assert!(pos_new < pos_old, "newer entry should appear first");
    }

    #[tokio::test]
    async fn search_filter() {
        let state = make_test_state().await;
        sqlx::query(
            "INSERT INTO clipboard (id, content, content_type, created_at) VALUES (?, ?, 'text', ?)",
        )
        .bind("id-a")
        .bind("hello world")
        .bind(1000i64)
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO clipboard (id, content, content_type, created_at) VALUES (?, ?, 'text', ?)",
        )
        .bind("id-b")
        .bind("something else")
        .bind(2000i64)
        .execute(&state.db)
        .await
        .unwrap();

        let (status, body) = call(test_app(state), Method::GET, "/api/clipboard?q=hello").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("hello world"));
        assert!(!body.contains("something else"));
    }

    #[tokio::test]
    async fn delete_one() {
        let state = make_test_state().await;
        sqlx::query(
            "INSERT INTO clipboard (id, content, content_type, created_at) VALUES (?, ?, 'text', ?)",
        )
        .bind("del-id")
        .bind("to delete")
        .bind(1000i64)
        .execute(&state.db)
        .await
        .unwrap();

        // Call handler directly to bypass router (tests business logic, not routing)
        delete_one_handler(State(state.clone()), Path("del-id".to_string())).await;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clipboard WHERE id = 'del-id'")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(count.0, 0);
    }

    #[tokio::test]
    async fn clear_all() {
        let state = make_test_state().await;
        for i in 0..3 {
            sqlx::query(
                "INSERT INTO clipboard (id, content, content_type, created_at) VALUES (?, ?, 'text', ?)",
            )
            .bind(format!("id-{i}"))
            .bind(format!("item {i}"))
            .bind(i as i64 * 1000)
            .execute(&state.db)
            .await
            .unwrap();
        }

        let (status, _) = call(test_app(state.clone()), Method::DELETE, "/api/clipboard").await;
        assert_eq!(status, StatusCode::OK);

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clipboard")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(count.0, 0);
    }
}
