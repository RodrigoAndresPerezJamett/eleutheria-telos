use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    response::{Html, IntoResponse, Response},
    routing::post,
    Form, Router,
};
use serde::Deserialize;

use crate::server::AppState;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

const CAPTURE_TMP: &str = "/tmp/eleutheria-ocr-capture.png";
const UPLOAD_TMP_BASE: &str = "/tmp/eleutheria-ocr-upload";

// ── Shared: run tesseract on a file and render the result HTML ────────────────

async fn run_tesseract(image_path: &str, lang: &str) -> Response {
    let out = tokio::process::Command::new("tesseract")
        .arg(image_path)
        .arg("stdout")
        .arg("-l")
        .arg(lang)
        .output()
        .await;

    match out {
        Ok(o) if o.status.success() => {
            let raw = String::from_utf8_lossy(&o.stdout);
            let text = raw.trim();
            if text.is_empty() {
                return Html(
                    r#"<p class="text-gray-400 text-sm mt-4">No text found in image.</p>"#
                        .to_string(),
                )
                .into_response();
            }
            render_result(text).into_response()
        }
        Ok(o) => {
            let err = html_escape(String::from_utf8_lossy(&o.stderr).trim());
            Html(format!(
                r##"<p class="text-red-400 text-sm mt-4">Tesseract error: {err}</p>"##
            ))
            .into_response()
        }
        Err(e) => Html(format!(
            r##"<p class="text-red-400 text-sm mt-4">Could not run tesseract: {}</p>"##,
            html_escape(&e.to_string())
        ))
        .into_response(),
    }
}

/// Build the result HTML shown after a successful OCR run.
/// Uses r##"..."## to allow `"#` inside (e.g. hx-include="#ocr-text-form"). (D-023)
fn render_result(text: &str) -> Html<String> {
    let escaped = html_escape(text);
    Html(format!(
        r##"<div class="mt-4 flex flex-col gap-3">
  <pre class="text-sm text-gray-200 bg-gray-800 rounded-lg p-4 whitespace-pre-wrap break-words max-h-64 overflow-y-auto font-sans leading-relaxed">{escaped}</pre>
  <form id="ocr-text-form">
    <textarea name="text" class="hidden">{escaped}</textarea>
  </form>
  <div class="flex gap-2">
    <button class="text-xs text-blue-400 hover:text-blue-300 border border-blue-700 rounded px-3 py-1.5"
            hx-post="/api/ocr/copy"
            hx-include="#ocr-text-form"
            hx-target="#ocr-feedback"
            hx-swap="innerHTML">Copy to Clipboard</button>
    <button class="text-xs text-green-400 hover:text-green-300 border border-green-700 rounded px-3 py-1.5"
            hx-post="/api/ocr/save-note"
            hx-include="#ocr-text-form"
            hx-target="#ocr-feedback"
            hx-swap="innerHTML">Save as Note</button>
  </div>
  <div id="ocr-feedback" class="text-xs"></div>
</div>"##
    ))
}

// ── Request structs ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TextBody {
    pub text: String,
}

#[derive(Deserialize)]
pub struct CaptureParams {
    #[serde(default = "default_lang")]
    pub lang: String,
}

fn default_lang() -> String {
    "eng".to_string()
}

// ── Route handlers ────────────────────────────────────────────────────────────

/// POST /api/ocr/capture
/// Runs slurp (interactive region selector) → grim (Wayland screenshot) → tesseract.
pub async fn capture_handler(
    State(_state): State<Arc<AppState>>,
    Form(params): Form<CaptureParams>,
) -> impl IntoResponse {
    // Step 1: slurp — shows a Wayland-native crosshair/selection overlay.
    // Blocks until the user draws a region or presses Escape to cancel.
    let slurp = tokio::process::Command::new("slurp").output().await;

    let slurp = match slurp {
        Ok(s) => s,
        Err(e) => {
            return Html(format!(
                r##"<p class="text-red-400 text-sm mt-4">slurp not found: {}</p>"##,
                html_escape(&e.to_string())
            ))
            .into_response();
        }
    };

    if !slurp.status.success() {
        // User pressed Escape to cancel
        return Html(r#"<p class="text-gray-500 text-sm mt-4">Capture cancelled.</p>"#.to_string())
            .into_response();
    }

    let geometry = String::from_utf8_lossy(&slurp.stdout).trim().to_string();
    if geometry.is_empty() {
        return Html(r#"<p class="text-gray-500 text-sm mt-4">Capture cancelled.</p>"#.to_string())
            .into_response();
    }

    // Step 2: grim — capture the selected screen region to a temp PNG.
    let grim = tokio::process::Command::new("grim")
        .arg("-g")
        .arg(&geometry)
        .arg(CAPTURE_TMP)
        .output()
        .await;

    match grim {
        Ok(g) if g.status.success() => {}
        Ok(g) => {
            let err = html_escape(String::from_utf8_lossy(&g.stderr).trim());
            return Html(format!(
                r##"<p class="text-red-400 text-sm mt-4">grim error: {err}</p>"##
            ))
            .into_response();
        }
        Err(e) => {
            return Html(format!(
                r##"<p class="text-red-400 text-sm mt-4">grim not found: {}</p>"##,
                html_escape(&e.to_string())
            ))
            .into_response();
        }
    }

    // Step 3: tesseract — OCR the captured image.
    run_tesseract(CAPTURE_TMP, &params.lang).await
}

/// POST /api/ocr/file  (multipart/form-data, field name: "image", field name: "lang")
/// Saves the uploaded image to a temp file, then runs tesseract.
pub async fn file_handler(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut image_bytes: Option<Vec<u8>> = None;
    let mut content_type = "image/png".to_string();
    let mut lang = "eng".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("image") => {
                if let Some(ct) = field.content_type() {
                    content_type = ct.to_string();
                }
                match field.bytes().await {
                    Ok(b) => image_bytes = Some(b.to_vec()),
                    Err(e) => {
                        return Html(format!(
                            r##"<p class="text-red-400 text-sm mt-4">Upload error: {}</p>"##,
                            html_escape(&e.to_string())
                        ))
                        .into_response();
                    }
                }
            }
            Some("lang") => {
                if let Ok(val) = field.text().await {
                    lang = val.trim().to_string();
                }
            }
            _ => {}
        }
    }

    let bytes = match image_bytes {
        Some(b) if !b.is_empty() => b,
        _ => {
            return Html(
                r#"<p class="text-gray-400 text-sm mt-4">No image received.</p>"#.to_string(),
            )
            .into_response();
        }
    };

    let ext = match content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/tiff" => "tif",
        _ => "png",
    };

    // Sanitise lang: only allow known installed values
    let lang = if lang == "spa" { "spa" } else { "eng" };

    let tmp_path = format!("{UPLOAD_TMP_BASE}.{ext}");
    if let Err(e) = tokio::fs::write(&tmp_path, &bytes).await {
        return Html(format!(
            r##"<p class="text-red-400 text-sm mt-4">Write error: {}</p>"##,
            html_escape(&e.to_string())
        ))
        .into_response();
    }

    run_tesseract(&tmp_path, lang).await
}

/// POST /api/ocr/copy  (form-encoded, field: text)
/// Copies the OCR result to the system clipboard via arboard.
pub async fn copy_handler(
    State(_state): State<Arc<AppState>>,
    Form(body): Form<TextBody>,
) -> impl IntoResponse {
    if body.text.is_empty() {
        return Html(r#"<span class="text-gray-400">Nothing to copy.</span>"#.to_string())
            .into_response();
    }

    let text = body.text.clone();
    // Do NOT suppress: OCR text is new content — it should appear in clipboard history.
    // Suppress is only for recopy (content already in history, D-014).
    tokio::task::spawn_blocking(move || {
        if let Ok(mut board) = arboard::Clipboard::new() {
            let _ = board.set_text(&text);
        }
    });

    Html(r#"<span class="text-green-400">Copied to clipboard!</span>"#.to_string()).into_response()
}

/// POST /api/ocr/save-note  (form-encoded, field: text)
/// Saves the OCR result as a new Note in SQLite.
pub async fn save_note_handler(
    State(state): State<Arc<AppState>>,
    Form(body): Form<TextBody>,
) -> impl IntoResponse {
    if body.text.is_empty() {
        return Html(r#"<span class="text-gray-400">Nothing to save.</span>"#.to_string())
            .into_response();
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = now_secs();
    // Use first non-empty line as title (up to 60 chars)
    let title: String = body
        .text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("OCR Extract")
        .chars()
        .take(60)
        .collect();

    let result = sqlx::query(
        "INSERT INTO notes (id, title, content, content_fts, tags, pinned, created_at, updated_at)
         VALUES (?, ?, ?, ?, '[]', 0, ?, ?)",
    )
    .bind(&id)
    .bind(&title)
    .bind(&body.text)
    .bind(&body.text) // FTS5 triggers sync notes_fts automatically (D-012)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Html(r#"<span class="text-green-400">Saved to Notes!</span>"#.to_string())
            .into_response(),
        Err(e) => Html(format!(
            r##"<span class="text-red-400">DB error: {}</span>"##,
            html_escape(&e.to_string())
        ))
        .into_response(),
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ocr/capture", post(capture_handler))
        .route("/api/ocr/file", post(file_handler))
        .route("/api/ocr/copy", post(copy_handler))
        .route("/api/ocr/save-note", post(save_note_handler))
}
