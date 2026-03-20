use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use base64::Engine as _;
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

fn render_entry_card(
    id: &str,
    content: &str,
    created_at: i64,
    content_type: &str,
    image_thumb: Option<&str>,
    is_pinned: bool,
) -> String {
    let ts = format_timestamp(created_at);

    // Pin button — absolute top-right; always visible when pinned, hover-only when not
    let pin_btn = if is_pinned {
        format!(
            r#"<button class="btn btn-ghost btn-sm"
              style="position:absolute;top:6px;right:6px;z-index:2;display:inline-flex;padding:3px;color:#f59e0b;background:transparent;border:none;cursor:pointer;"
              hx-put="/api/clipboard/{id}/pin"
              hx-swap="none"
              hx-on::after-request="htmx.trigger(document.body, 'clipboardRefresh')"
              onclick="event.stopPropagation()"
              title="Unpin"><svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg></button>"#,
            id = id
        )
    } else {
        format!(
            r#"<button class="clip-pin-btn btn btn-ghost btn-sm"
              style="position:absolute;top:6px;right:6px;z-index:2;display:inline-flex;padding:3px;color:#f59e0b;opacity:0.55;background:transparent;border:none;cursor:pointer;"
              hx-put="/api/clipboard/{id}/pin"
              hx-swap="none"
              hx-on::after-request="htmx.trigger(document.body, 'clipboardRefresh')"
              onclick="event.stopPropagation()"
              title="Pin"><svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg></button>"#,
            id = id
        )
    };

    let (body_html, attr_content, truncated_attr, kind_attr, copy_btn) =
        if content_type == "image" {
            let body = match image_thumb {
                Some(thumb) => format!(
                    r#"<img src="data:image/png;base64,{thumb}" alt="Image" style="flex:1;min-height:0;width:100%;object-fit:cover;border-radius:calc(var(--radius-md) - 4px);margin:0 0 10px;display:block;">"#
                ),
                None => r#"<div style="flex:1;display:flex;align-items:center;justify-content:center;font-size:12px;color:var(--text-muted);margin:0 0 10px;">Image</div>"#.to_string(),
            };
            (body, String::new(), "", r#" data-clip-kind="image""#, "")
        } else {
            let truncated = content.len() > CONTENT_ATTR_MAX;
            let attr = html_escape(truncate_bytes(content, CONTENT_ATTR_MAX));
            let preview = html_escape(truncate_bytes(content, CONTENT_PREVIEW_MAX));
            let trunc_attr = if truncated { r#" data-clip-truncated="true""# } else { "" };
            let body = format!(
                r#"<pre style="font-size:12px;color:var(--text-primary);white-space:pre-wrap;word-break:break-all;font-family:inherit;line-height:1.5;margin:0 0 10px;flex:1;overflow:hidden;">{preview}</pre>"#
            );
            let copy = r#"<button class="clip-action btn btn-ghost btn-sm"
              style="display:none;font-size:11px;padding:2px 6px;"
              hx-post="/api/clipboard/{id}/recopy"
              hx-swap="none"
              onclick="event.stopPropagation()"
              title="Copy to clipboard">Copy</button>"#;
            (body, attr, trunc_attr, "", copy)
        };

    let copy_btn = copy_btn.replace("{id}", id);

    format!(
        r##"<div id="clip-{id}"
     data-clip-id="{id}"
     data-clip-content="{attr_content}"{truncated_attr}{kind_attr}
     style="background:var(--bg-elevated);border-radius:var(--radius-md);padding:14px 14px 12px;cursor:pointer;display:flex;flex-direction:column;overflow:hidden;outline:1px solid transparent;outline-offset:-1px;position:relative;"
     onmouseenter="this.style.outlineColor='var(--accent)';this.querySelectorAll('.clip-action').forEach(e=>e.style.display='inline-flex');const pb=this.querySelector('.clip-pin-btn');if(pb)pb.style.opacity='1'"
     onmouseleave="this.style.outlineColor='transparent';this.querySelectorAll('.clip-action').forEach(e=>e.style.display='none');const pb=this.querySelector('.clip-pin-btn');if(pb)pb.style.opacity='0.55'"
     onclick="clipboardOpenPreview(this)">
  {pin_btn}
  {body_html}
  <div style="display:flex;align-items:center;justify-content:space-between;flex-shrink:0;position:relative;z-index:1;">
    <span style="font-size:11px;color:var(--text-muted);">{ts}</span>
    <div style="display:flex;gap:4px;">
      {copy_btn}
      <button class="clip-action btn btn-ghost btn-sm"
              style="display:none;font-size:11px;padding:2px 6px;color:var(--destructive);"
              hx-delete="/api/clipboard/{id}"
              hx-target="#clip-{id}"
              hx-swap="outerHTML"
              hx-confirm="Delete this entry?"
              onclick="event.stopPropagation()">✕</button>
    </div>
  </div>
  <div style="position:absolute;bottom:0;left:0;right:0;height:38%;pointer-events:none;background:linear-gradient(to top, rgba(0,0,0,0.09) 0%, transparent 100%);border-radius:0 0 var(--radius-md) var(--radius-md);"></div>
</div>"##,
        id = id,
        attr_content = attr_content,
        truncated_attr = truncated_attr,
        kind_attr = kind_attr,
        pin_btn = pin_btn,
        body_html = body_html,
        ts = ts,
        copy_btn = copy_btn,
    )
}

const PAGE: i64 = 20;

#[allow(clippy::type_complexity)]
fn render_list(
    entries: &[(String, String, i64, Option<String>, String, bool)],
    has_more: bool,
    next_offset: i64,
) -> String {
    if entries.is_empty() {
        return r#"<div style="grid-column:1/-1;padding:48px 16px;text-align:center;">
  <p style="font-size:15px;font-weight:500;color:var(--text-primary);margin:0 0 8px;">Lost something you copied?</p>
  <p style="font-size:13px;color:var(--text-muted);margin:0;line-height:1.6;">Everything you copy appears here automatically.<br>Start copying and it will show up.</p>
</div>"#.to_string();
    }
    let mut html: String = entries
        .iter()
        .map(|(id, content, ts, image_thumb, content_type, is_pinned)| {
            render_entry_card(id, content, *ts, content_type, image_thumb.as_deref(), *is_pinned)
        })
        .collect();
    if has_more {
        html.push_str(&format!(
            r#"<div id="clip-sentinel" style="grid-column:1/-1;padding:12px;text-align:center;color:var(--text-muted);font-size:12px;"
     hx-get="/api/clipboard?offset={next_offset}&limit={PAGE}"
     hx-trigger="intersect once"
     hx-swap="outerHTML">Loading more…</div>"#
        ));
    }
    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// Max bytes stored in data-* attribute and shown in the card preview.
// Full content is fetched on-demand via GET /api/clipboard/:id when truncated.
const CONTENT_ATTR_MAX: usize = 2048;
const CONTENT_PREVIEW_MAX: usize = 300;

fn truncate_bytes(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

// Max thumbnail dimensions for image clipboard entries.
const THUMB_W: u32 = 640;
const THUMB_H: u32 = 360;

/// Quick fingerprint for an image: dimensions + first 256 bytes.
fn image_hash(img: &arboard::ImageData) -> u64 {
    let mut h = DefaultHasher::new();
    img.width.hash(&mut h);
    img.height.hash(&mut h);
    let n = img.bytes.len().min(256);
    img.bytes[..n].hash(&mut h);
    h.finish()
}

/// Nearest-neighbour downsample + PNG encode → base64 string.
/// Takes raw RGBA bytes so both arboard images and file-loaded images share the same path.
fn make_thumbnail(bytes: &[u8], src_w: usize, src_h: usize) -> String {
    if src_w == 0 || src_h == 0 {
        return String::new();
    }

    // Preserve aspect ratio, fit inside THUMB_W × THUMB_H
    let scale_w = THUMB_W as f64 / src_w as f64;
    let scale_h = THUMB_H as f64 / src_h as f64;
    let scale = scale_w.min(scale_h).min(1.0); // never upscale
    let dst_w = ((src_w as f64 * scale) as u32).max(1);
    let dst_h = ((src_h as f64 * scale) as u32).max(1);

    let mut out = vec![0u8; (dst_w * dst_h * 4) as usize];
    for dy in 0..dst_h as usize {
        for dx in 0..dst_w as usize {
            let sx = dx * src_w / dst_w as usize;
            let sy = dy * src_h / dst_h as usize;
            let src_i = (sy * src_w + sx) * 4;
            let dst_i = (dy * dst_w as usize + dx) * 4;
            if src_i + 4 <= bytes.len() {
                out[dst_i..dst_i + 4].copy_from_slice(&bytes[src_i..src_i + 4]);
            }
        }
    }

    let mut buf = Vec::new();
    {
        let mut enc = png::Encoder::new(std::io::Cursor::new(&mut buf), dst_w, dst_h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = match enc.write_header() {
            Ok(w) => w,
            Err(e) => {
                log::warn!("clipboard: PNG header error: {e}");
                return String::new();
            }
        };
        if let Err(e) = writer.write_image_data(&out) {
            log::warn!("clipboard: PNG data error: {e}");
            return String::new();
        }
    }

    base64::engine::general_purpose::STANDARD.encode(&buf)
}

/// If `text` is a local image file path (plain or file:// URI), load and thumbnail it.
/// Returns Some(base64_thumb) if successful, None otherwise.
fn thumb_from_path(text: &str) -> Option<String> {
    let text = text.trim();
    // Handle file:// URIs
    let path_str = if let Some(rest) = text.strip_prefix("file://") {
        rest.replace("%20", " ").replace("%28", "(").replace("%29", ")")
    } else {
        text.to_string()
    };
    let path = std::path::Path::new(&path_str);
    let ext = path.extension()?.to_str()?.to_lowercase();
    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp") {
        return None;
    }
    if !path.exists() {
        return None;
    }
    let img = image::open(path).ok()?;
    let rgba = img.to_rgba8();
    let w = rgba.width() as usize;
    let h = rgba.height() as usize;
    let thumb = make_thumbnail(rgba.as_raw(), w, h);
    if thumb.is_empty() { None } else { Some(thumb) }
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
    PAGE
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    // Fetch one extra row to detect whether a next page exists
    let fetch_limit = params.limit + 1;
    let mut rows: Vec<(String, String, i64, Option<String>, String, bool)> = if params.q.is_empty() {
        sqlx::query_as(
            "SELECT id, content, created_at, image_thumb, content_type, is_pinned FROM clipboard
             ORDER BY is_pinned DESC, created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(fetch_limit)
        .bind(params.offset)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    } else {
        // Search returns all matches — no pagination when filtering
        let pattern = format!("%{}%", params.q);
        sqlx::query_as(
            "SELECT id, content, created_at, image_thumb, content_type, is_pinned FROM clipboard
             WHERE content LIKE ?
             ORDER BY is_pinned DESC, created_at DESC LIMIT 200 OFFSET 0",
        )
        .bind(&pattern)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    };

    let has_more = rows.len() > params.limit as usize && params.q.is_empty();
    rows.truncate(params.limit as usize);
    let next_offset = params.offset + params.limit;

    Html(render_list(&rows, has_more, next_offset))
}

pub async fn get_one_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let row: Option<(String,)> = sqlx::query_as("SELECT content FROM clipboard WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);
    match row {
        Some((content,)) => Json(serde_json::json!({ "content": content })).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "not found" })),
        )
            .into_response(),
    }
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

pub async fn pin_toggle_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    sqlx::query(
        "UPDATE clipboard SET is_pinned = CASE WHEN is_pinned = 1 THEN 0 ELSE 1 END WHERE id = ?",
    )
    .bind(&id)
    .execute(&state.db)
    .await
    .ok();
    StatusCode::NO_CONTENT
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/clipboard",
            get(list_handler).delete(clear_all_handler),
        )
        .route("/api/clipboard/:id/recopy", post(recopy_handler))
        .route("/api/clipboard/:id/pin", put(pin_toggle_handler))
        .route(
            "/api/clipboard/:id",
            get(get_one_handler).delete(delete_one_handler),
        )
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
        let mut last_image_hash: u64 = 0;

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

            // ── Image detection ────────────────────────────────────────────
            if let Ok(img) = cb.get_image() {
                let hash = image_hash(&img);
                if hash != last_image_hash {
                    last_image_hash = hash;
                    last_hash = 0; // reset text hash so next text copy registers

                    let thumb = make_thumbnail(img.bytes.as_ref(), img.width, img.height);
                    let id = uuid::Uuid::new_v4().to_string();
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    let insert_result = tauri::async_runtime::block_on(
                        sqlx::query(
                            "INSERT INTO clipboard (id, content, content_type, image_thumb, created_at)
                             VALUES (?, '[image]', 'image', ?, ?)",
                        )
                        .bind(&id)
                        .bind(&thumb)
                        .bind(now)
                        .execute(&state_clone.db),
                    );
                    if let Err(e) = insert_result {
                        log::error!("Clipboard image insert failed: {e}");
                    } else {
                        state_clone.event_bus.publish(Event::ClipboardChanged {
                            content: "[image]".to_string(),
                            content_type: "image".to_string(),
                        });
                    }
                }
                continue; // clipboard contains an image — skip text check this cycle
            }

            // ── Text detection ─────────────────────────────────────────────
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
            last_image_hash = 0; // reset image hash so next image copy registers

            let id = uuid::Uuid::new_v4().to_string();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            // If text looks like a local image file path, store it as an image entry
            if let Some(thumb) = thumb_from_path(&text) {
                let insert_result = tauri::async_runtime::block_on(
                    sqlx::query(
                        "INSERT INTO clipboard (id, content, content_type, image_thumb, created_at)
                         VALUES (?, '[image]', 'image', ?, ?)",
                    )
                    .bind(&id)
                    .bind(&thumb)
                    .bind(now)
                    .execute(&state_clone.db),
                );
                if let Err(e) = insert_result {
                    log::error!("Clipboard image-from-path insert failed: {e}");
                } else {
                    state_clone.event_bus.publish(Event::ClipboardChanged {
                        content: "[image]".to_string(),
                        content_type: "image".to_string(),
                    });
                }
                continue;
            }

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
        assert!(body.contains("Lost something you copied?"));
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
    async fn pin_toggle() {
        let state = make_test_state().await;
        sqlx::query(
            "INSERT INTO clipboard (id, content, content_type, created_at) VALUES (?, ?, 'text', ?)",
        )
        .bind("pin-id")
        .bind("pinnable item")
        .bind(1000i64)
        .execute(&state.db)
        .await
        .unwrap();

        // Toggle on
        pin_toggle_handler(State(state.clone()), Path("pin-id".to_string())).await;
        let pinned: (bool,) =
            sqlx::query_as("SELECT is_pinned FROM clipboard WHERE id = 'pin-id'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert!(pinned.0, "should be pinned after first toggle");

        // Toggle off
        pin_toggle_handler(State(state.clone()), Path("pin-id".to_string())).await;
        let unpinned: (bool,) =
            sqlx::query_as("SELECT is_pinned FROM clipboard WHERE id = 'pin-id'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert!(!unpinned.0, "should be unpinned after second toggle");
    }

    #[tokio::test]
    async fn pinned_items_float_to_top() {
        let state = make_test_state().await;
        sqlx::query(
            "INSERT INTO clipboard (id, content, content_type, created_at) VALUES (?, ?, 'text', ?)",
        )
        .bind("id-old")
        .bind("older item")
        .bind(1000i64)
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO clipboard (id, content, content_type, created_at) VALUES (?, ?, 'text', ?)",
        )
        .bind("id-new")
        .bind("newer item")
        .bind(2000i64)
        .execute(&state.db)
        .await
        .unwrap();

        // Pin the older item
        pin_toggle_handler(State(state.clone()), Path("id-old".to_string())).await;

        let (status, body) = call(test_app(state), Method::GET, "/api/clipboard").await;
        assert_eq!(status, StatusCode::OK);
        let pos_old = body.find("older item").unwrap();
        let pos_new = body.find("newer item").unwrap();
        assert!(pos_old < pos_new, "pinned older item should appear before newer item");
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
