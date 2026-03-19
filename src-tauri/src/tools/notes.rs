use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::event_bus::Event;
use crate::server::AppState;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_note_card(id: &str, title: &str, content: &str, pinned: i64, updated_at: i64) -> String {
    let display_title = if title.is_empty() { "Untitled" } else { title };
    let preview = if content.len() > 120 {
        format!("{}…", &content[..120])
    } else {
        content.to_string()
    };
    let pin_icon = if pinned == 1 { "📌" } else { "" };
    let ts = format_timestamp(updated_at);
    // r##"..."## used because the HTML contains "# sequences (HTMX targets like "#note-id")
    format!(
        r##"<div id="note-{id}" class="group bg-gray-800 rounded-lg p-3 mb-2 cursor-pointer hover:bg-gray-750"
             hx-get="/api/notes/{id}"
             hx-target="#note-editor"
             hx-swap="innerHTML">
  <div class="flex items-start justify-between gap-2">
    <h3 class="text-sm font-medium text-gray-100 truncate">{pin_icon}{escaped_title}</h3>
    <button class="shrink-0 text-xs text-gray-500 hover:text-yellow-400 opacity-0 group-hover:opacity-100"
            hx-post="/api/notes/{id}/pin"
            hx-target="#note-{id}"
            hx-swap="outerHTML"
            title="Toggle pin"
            @click.stop="">Pin</button>
  </div>
  <p class="text-xs text-gray-400 mt-1 line-clamp-2">{escaped_preview}</p>
  <div class="flex items-center justify-between mt-2">
    <span class="text-xs text-gray-600">{ts}</span>
    <button class="text-xs text-red-400 hover:text-red-300 opacity-0 group-hover:opacity-100"
            hx-delete="/api/notes/{id}"
            hx-target="#note-{id}"
            hx-swap="outerHTML"
            hx-confirm="Delete this note?"
            @click.stop="">Delete</button>
  </div>
</div>"##,
        id = id,
        pin_icon = pin_icon,
        escaped_title = html_escape(display_title),
        escaped_preview = html_escape(&preview),
        ts = ts,
    )
}

fn render_note_list(entries: &[(String, String, String, i64, i64)]) -> String {
    if entries.is_empty() {
        return r#"<p class="text-gray-500 text-sm">No notes yet.</p>"#.to_string();
    }
    entries
        .iter()
        .map(|(id, title, content, pinned, updated_at)| {
            render_note_card(id, title, content, *pinned, *updated_at)
        })
        .collect()
}

fn render_editor(id: &str, title: &str, content: &str) -> String {
    let escaped_content = html_escape(content);
    format!(
        r#"<div x-data="notesEditor('{id}')" class="flex flex-col h-full">
  <div class="flex items-center gap-2 mb-3">
    <input type="text"
           x-model="title"
           @input.debounce.800ms="save()"
           placeholder="Note title…"
           class="flex-1 bg-transparent text-lg font-semibold text-gray-100 outline-none border-b border-gray-700 pb-1 focus:border-blue-500"/>
    <button @click="preview = !preview"
            class="text-xs px-2 py-1 rounded bg-gray-700 hover:bg-gray-600 text-gray-300">
      Preview
    </button>
    <span x-show="saving" class="text-xs text-gray-500">Saving…</span>
    <span x-show="saved && !saving" x-transition class="text-xs text-green-500">Saved</span>
  </div>
  <div x-show="!preview" class="flex-1 min-h-0">
    <textarea x-model="content"
              @input.debounce.800ms="save()"
              placeholder="Start writing…"
              class="w-full h-full bg-transparent text-sm text-gray-200 resize-none outline-none leading-relaxed font-mono">{escaped_content}</textarea>
  </div>
  <div x-show="preview" class="flex-1 overflow-y-auto prose prose-invert prose-sm max-w-none"
       x-html="renderMarkdown()"></div>
</div>
<script>
function notesEditor(noteId) {{
  return {{
    noteId: noteId,
    title: {title_json},
    content: {content_json},
    saving: false,
    saved: false,
    preview: false,
    async save() {{
      this.saving = true;
      this.saved = false;
      try {{
        await fetch('http://127.0.0.1:' + window.__API_PORT__ + '/api/notes/' + this.noteId, {{
          method: 'PUT',
          headers: {{
            'Content-Type': 'application/json',
            'Authorization': 'Bearer ' + window.__SESSION_TOKEN__,
          }},
          body: JSON.stringify({{ title: this.title, content: this.content }}),
        }});
        this.saved = true;
        htmx.trigger(document.body, 'noteUpdated');
      }} finally {{
        this.saving = false;
      }}
    }},
    renderMarkdown() {{
      if (typeof marked !== 'undefined') return marked.parse(this.content || '');
      return '<pre>' + this.content + '</pre>';
    }},
  }};
}}
</script>"#,
        id = id,
        escaped_content = escaped_content,
        title_json = serde_json::to_string(title).unwrap_or_else(|_| "\"\"".to_string()),
        content_json = serde_json::to_string(content).unwrap_or_else(|_| "\"\"".to_string()),
    )
}

fn format_timestamp(ts: i64) -> String {
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

// ── Query/body params ─────────────────────────────────────────────────────────

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

#[derive(Deserialize)]
pub struct CreateBody {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tags: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateBody {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
    pub pinned: Option<i64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    let rows: Vec<(String, String, String, i64, i64)> = if params.q.is_empty() {
        sqlx::query_as(
            "SELECT id, title, content, pinned, updated_at FROM notes
             ORDER BY pinned DESC, updated_at DESC LIMIT ? OFFSET ?",
        )
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    } else {
        // FTS5 MATCH search — returns rowids joined to notes
        sqlx::query_as(
            "SELECT n.id, n.title, n.content, n.pinned, n.updated_at
             FROM notes n
             JOIN notes_fts f ON f.rowid = n.rowid
             WHERE notes_fts MATCH ?
             ORDER BY n.pinned DESC, rank
             LIMIT ? OFFSET ?",
        )
        .bind(&params.q)
        .bind(params.limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    };

    Html(render_note_list(&rows))
}

pub async fn create_handler(
    State(state): State<Arc<AppState>>,
    Form(body): Form<CreateBody>,
) -> Response {
    let id = uuid::Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let tags = body.tags.unwrap_or_else(|| "[]".to_string());

    let result = sqlx::query(
        "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&body.title)
    .bind(&body.content)
    .bind(&body.content) // Phase 1: content_fts = content (Markdown stripping in Phase 2)
    .bind(&tags)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            state.event_bus.publish(Event::NoteCreated {
                id: id.clone(),
                title: body.title.clone(),
            });
            let card = render_note_card(&id, &body.title, &body.content, 0, now);
            (StatusCode::CREATED, [("X-Note-Id", id)], Html(card)).into_response()
        }
        Err(e) => {
            log::error!("Note create failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "insert failed" })),
            )
                .into_response()
        }
    }
}

pub async fn get_handler(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT title, content FROM notes WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

    match row {
        Some((title, content)) => Html(render_editor(&id, &title, &content)).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" }))).into_response(),
    }
}

pub async fn update_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateBody>,
) -> impl IntoResponse {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // Build SET clause dynamically — raw query (no macros), with comment per CLAUDE.md.
    // This is the one place dynamic SQL is used; fields are bound parameters, not interpolated.
    let mut set_parts = vec!["updated_at = ?"];
    let mut title_val: Option<String> = None;
    let mut content_val: Option<String> = None;
    let mut tags_val: Option<String> = None;
    let mut pinned_val: Option<i64> = None;

    if body.title.is_some() {
        set_parts.push("title = ?");
        title_val = body.title;
    }
    if body.content.is_some() {
        // content_fts mirrors content (Phase 1; Markdown stripping deferred to Phase 2)
        set_parts.push("content = ?");
        set_parts.push("content_fts = ?");
        content_val = body.content.clone();
    }
    if body.tags.is_some() {
        set_parts.push("tags = ?");
        tags_val = body.tags;
    }
    if body.pinned.is_some() {
        set_parts.push("pinned = ?");
        pinned_val = body.pinned;
    }

    let sql = format!("UPDATE notes SET {} WHERE id = ?", set_parts.join(", "));

    let mut q = sqlx::query(&sql).bind(now);
    if let Some(v) = title_val {
        q = q.bind(v);
    }
    if let Some(v) = content_val {
        q = q.bind(v.clone()).bind(v); // content + content_fts
    }
    if let Some(v) = tags_val {
        q = q.bind(v);
    }
    if let Some(v) = pinned_val {
        q = q.bind(v);
    }
    q = q.bind(&id);

    match q.execute(&state.db).await {
        Ok(_) => {
            state.event_bus.publish(Event::NoteUpdated { id });
            Json(json!({ "ok": true }))
        }
        Err(e) => {
            log::error!("Note update failed: {e}");
            Json(json!({ "ok": false, "error": e.to_string() }))
        }
    }
}

pub async fn delete_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();
    Html(String::new())
}

pub async fn pin_toggle_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    // Toggle pinned and return updated card HTML
    sqlx::query("UPDATE notes SET pinned = CASE WHEN pinned = 1 THEN 0 ELSE 1 END WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();

    let row: Option<(String, String, String, i64, i64)> =
        sqlx::query_as("SELECT id, title, content, pinned, updated_at FROM notes WHERE id = ?")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

    match row {
        Some((id, title, content, pinned, updated_at)) => {
            Html(render_note_card(&id, &title, &content, pinned, updated_at)).into_response()
        }
        None => (StatusCode::NOT_FOUND, Html(String::new())).into_response(),
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/notes", get(list_handler).post(create_handler))
        .route(
            "/api/notes/:id",
            get(get_handler).put(update_handler).delete(delete_handler),
        )
        .route("/api/notes/:id/pin", post(pin_toggle_handler))
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

    async fn post_form(
        app: axum::Router,
        uri: &str,
        body: &[(&str, &str)],
    ) -> (StatusCode, String) {
        let encoded = body
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Authorization", "Bearer test-token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(encoded))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8_lossy(&bytes).to_string())
    }

    #[tokio::test]
    async fn create_and_fts_sync() {
        let state = make_test_state().await;
        let (status, _) = post_form(
            test_app(state.clone()),
            "/api/notes",
            &[("title", "Hello FTS"), ("content", "searchable content")],
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'searchable'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn update_and_fts_sync() {
        let state = make_test_state().await;
        let id = "note-upd";
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES (?, 'TestNote', 'xyzoldterm', 'xyzoldterm', '[]', ?, ?)",
        )
        .bind(id)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

        // Call handler directly to bypass router (tests business logic, not routing)
        update_handler(
            State(state.clone()),
            Path(id.to_string()),
            Json(UpdateBody {
                content: Some("xyzupdatedterm content".to_string()),
                title: None,
                tags: None,
                pinned: None,
            }),
        )
        .await;

        let old_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'xyzoldterm'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(old_count.0, 0);

        let new_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'xyzupdatedterm'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(new_count.0, 1);
    }

    #[tokio::test]
    async fn delete_and_fts_removal() {
        let state = make_test_state().await;
        let id = "note-del";
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES (?, 'DelTitle', 'del content', 'del content', '[]', ?, ?)",
        )
        .bind(id)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

        // Call handler directly to bypass router
        delete_handler(State(state.clone()), Path(id.to_string())).await;

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'DelTitle'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(count.0, 0);
    }

    #[tokio::test]
    async fn list_pinned_first() {
        let state = make_test_state().await;
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, pinned, created_at, updated_at)
             VALUES ('unpinned', 'B', '', '', '[]', 0, ?, ?)",
        )
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, pinned, created_at, updated_at)
             VALUES ('pinned', 'A', '', '', '[]', 1, ?, ?)",
        )
        .bind(now + 1)
        .bind(now + 1)
        .execute(&state.db)
        .await
        .unwrap();

        let (status, body) = get(test_app(state), "/api/notes").await;
        assert_eq!(status, StatusCode::OK);
        let pos_pinned = body.find("note-pinned").unwrap();
        let pos_unpinned = body.find("note-unpinned").unwrap();
        assert!(pos_pinned < pos_unpinned, "pinned note should appear first");
    }

    #[tokio::test]
    async fn pin_toggle() {
        let state = make_test_state().await;
        let id = "toggle-note";
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, pinned, created_at, updated_at)
             VALUES (?, 'Pin me', '', '', '[]', 0, ?, ?)",
        )
        .bind(id)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

        // Call handler directly to bypass router
        pin_toggle_handler(State(state.clone()), Path(id.to_string())).await;

        let row: (i64,) = sqlx::query_as("SELECT pinned FROM notes WHERE id = ?")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    async fn fts5_match_search() {
        let state = make_test_state().await;
        let now = 1000i64;
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES ('n1', 'Rust Programming', 'learn ownership', 'learn ownership', '[]', ?, ?)",
        )
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
             VALUES ('n2', 'Python Guide', 'async await syntax', 'async await syntax', '[]', ?, ?)",
        )
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

        let (status, body) = get(test_app(state), "/api/notes?q=ownership").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("Rust Programming"));
        assert!(!body.contains("Python Guide"));
    }
}
