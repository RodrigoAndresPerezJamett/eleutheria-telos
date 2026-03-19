use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    response::{Html, IntoResponse, Response},
    routing::post,
    Form, Router,
};
use serde::Deserialize;

use crate::event_bus::Event;
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

async fn run_tesseract(image_path: &str, lang: &str, state: &Arc<AppState>) -> Response {
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
                    r#"<p style="font-size:13px;color:var(--text-muted);margin-top:16px;">No text found in image.</p>"#
                        .to_string(),
                )
                .into_response();
            }
            state.event_bus.publish(Event::OcrCompleted {
                text: text.to_string(),
                source: image_path.to_string(),
            });
            render_result(text).into_response()
        }
        Ok(o) => {
            let err = html_escape(String::from_utf8_lossy(&o.stderr).trim());
            Html(format!(
                r#"<p style="font-size:13px;color:var(--destructive);margin-top:16px;">Tesseract error: {err}</p>"#
            ))
            .into_response()
        }
        Err(e) => Html(format!(
            r#"<p style="font-size:13px;color:var(--destructive);margin-top:16px;">Could not run tesseract: {}</p>"#,
            html_escape(&e.to_string())
        ))
        .into_response(),
    }
}

/// Build the result HTML shown after a successful OCR run.
/// Uses r##"..."## to allow `"#` inside (e.g. hx-include="#ocr-text-form"). (D-023)
fn render_result(text: &str) -> Html<String> {
    let escaped = html_escape(text);
    // Pre-compute ID refs to avoid "# terminating r##"..."## (D-023).
    let ocr_text_form = "#ocr-text-form";
    let ocr_feedback = "#ocr-feedback";
    let ocr_translate_result = "#ocr-translate-result";
    Html(format!(
        r##"<div style="margin-top:16px;display:flex;flex-direction:column;gap:12px;">
  <pre style="font-size:13px;color:var(--text-primary);background:var(--bg-elevated);border-radius:var(--radius-md);padding:16px;white-space:pre-wrap;word-break:break-words;max-height:256px;overflow-y:auto;font-family:inherit;line-height:1.6;">{escaped}</pre>
  <form id="ocr-text-form">
    <textarea name="text" style="display:none;">{escaped}</textarea>
  </form>
  <div style="display:flex;gap:8px;flex-wrap:wrap;">
    <button class="btn btn-primary btn-sm"
            hx-post="/api/ocr/copy"
            hx-include="{ocr_text_form}"
            hx-target="{ocr_feedback}"
            hx-swap="innerHTML">Copy to Clipboard</button>
    <button class="btn btn-secondary btn-sm"
            hx-post="/api/ocr/save-note"
            hx-include="{ocr_text_form}"
            hx-target="{ocr_feedback}"
            hx-swap="innerHTML">Save as Note</button>
  </div>
  <div id="ocr-feedback" style="font-size:12px;"></div>

  <!-- ── OCR → Translate pipeline ───────────────────────────────────────── -->
  <div style="border-top:1px solid var(--border);padding-top:12px;" x-data="{{ showTranslate: false }}">
    <button class="btn btn-ghost btn-sm"
            @click="showTranslate = !showTranslate">
      Translate…
    </button>
    <div x-show="showTranslate" x-cloak style="margin-top:12px;display:flex;flex-direction:column;gap:8px;">
      <form hx-post="/api/translate/text"
            hx-target="{ocr_translate_result}"
            hx-swap="innerHTML"
            hx-indicator="#ocr-translate-spinner"
            style="display:flex;align-items:flex-end;gap:8px;flex-wrap:wrap;">
        <textarea name="text" style="display:none;">{escaped}</textarea>
        <div>
          <label style="display:block;font-size:11px;color:var(--text-muted);margin-bottom:4px;">From</label>
          <select name="from_lang" class="input" style="font-size:12px;padding:4px 8px;">
            <option value="en">English</option>
            <option value="es">Spanish</option>
            <option value="fr">French</option>
            <option value="de">German</option>
            <option value="pt">Portuguese</option>
          </select>
        </div>
        <span style="color:var(--text-muted);padding-bottom:4px;">→</span>
        <div>
          <label style="display:block;font-size:11px;color:var(--text-muted);margin-bottom:4px;">To</label>
          <select name="to_lang" class="input" style="font-size:12px;padding:4px 8px;">
            <option value="es">Spanish</option>
            <option value="en">English</option>
            <option value="fr">French</option>
            <option value="de">German</option>
            <option value="pt">Portuguese</option>
          </select>
        </div>
        <button type="submit" class="btn btn-primary btn-sm">Translate</button>
        <span id="ocr-translate-spinner" class="htmx-indicator" style="font-size:12px;color:var(--text-muted);font-style:italic;">Translating…</span>
      </form>
      <div id="ocr-translate-result" style="font-size:13px;"></div>
    </div>
  </div>
</div>"##,
        escaped = escaped,
        ocr_text_form = ocr_text_form,
        ocr_feedback = ocr_feedback,
        ocr_translate_result = ocr_translate_result,
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
    State(state): State<Arc<AppState>>,
    Form(params): Form<CaptureParams>,
) -> impl IntoResponse {
    // Step 1: slurp — shows a Wayland-native crosshair/selection overlay.
    // Blocks until the user draws a region or presses Escape to cancel.
    let slurp = tokio::process::Command::new("slurp").output().await;

    let slurp = match slurp {
        Ok(s) => s,
        Err(e) => {
            return Html(format!(
                r#"<p style="font-size:13px;color:var(--destructive);margin-top:16px;">slurp not found: {}</p>"#,
                html_escape(&e.to_string())
            ))
            .into_response();
        }
    };

    if !slurp.status.success() {
        // User pressed Escape to cancel
        return Html(r#"<p style="font-size:13px;color:var(--text-muted);margin-top:16px;">Capture cancelled.</p>"#.to_string())
            .into_response();
    }

    let geometry = String::from_utf8_lossy(&slurp.stdout).trim().to_string();
    if geometry.is_empty() {
        return Html(r#"<p style="font-size:13px;color:var(--text-muted);margin-top:16px;">Capture cancelled.</p>"#.to_string())
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
                r#"<p style="font-size:13px;color:var(--destructive);margin-top:16px;">grim error: {err}</p>"#
            ))
            .into_response();
        }
        Err(e) => {
            return Html(format!(
                r#"<p style="font-size:13px;color:var(--destructive);margin-top:16px;">grim not found: {}</p>"#,
                html_escape(&e.to_string())
            ))
            .into_response();
        }
    }

    // Step 3: tesseract — OCR the captured image.
    run_tesseract(CAPTURE_TMP, &params.lang, &state).await
}

/// POST /api/ocr/file  (multipart/form-data, field name: "image", field name: "lang")
/// Saves the uploaded image to a temp file, then runs tesseract.
pub async fn file_handler(
    State(state): State<Arc<AppState>>,
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
                            r#"<p style="font-size:13px;color:var(--destructive);margin-top:16px;">Upload error: {}</p>"#,
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
                r#"<p style="font-size:13px;color:var(--text-muted);margin-top:16px;">No image received.</p>"#.to_string(),
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
            r#"<p style="font-size:13px;color:var(--destructive);margin-top:16px;">Write error: {}</p>"#,
            html_escape(&e.to_string())
        ))
        .into_response();
    }

    run_tesseract(&tmp_path, lang, &state).await
}

/// POST /api/ocr/copy  (form-encoded, field: text)
/// Copies the OCR result to the system clipboard via arboard.
pub async fn copy_handler(
    State(_state): State<Arc<AppState>>,
    Form(body): Form<TextBody>,
) -> impl IntoResponse {
    if body.text.is_empty() {
        return Html(r#"<span style="color:var(--text-muted);">Nothing to copy.</span>"#.to_string())
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

    Html(r#"<span style="color:var(--success);">Copied to clipboard!</span>"#.to_string()).into_response()
}

/// POST /api/ocr/save-note  (form-encoded, field: text)
/// Saves the OCR result as a new Note in SQLite.
pub async fn save_note_handler(
    State(state): State<Arc<AppState>>,
    Form(body): Form<TextBody>,
) -> impl IntoResponse {
    if body.text.is_empty() {
        return Html(r#"<span style="color:var(--text-muted);">Nothing to save.</span>"#.to_string())
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
        Ok(_) => Html(r#"<span style="color:var(--success);">Saved to Notes!</span>"#.to_string())
            .into_response(),
        Err(e) => Html(format!(
            r#"<span style="color:var(--destructive);">DB error: {}</span>"#,
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
