use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::server::AppState;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn export_path() -> std::io::Result<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = std::path::PathBuf::from(&home)
        .join("Pictures")
        .join("Eleutheria");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("photo-{}.png", now_secs()));
    Ok(path.to_string_lossy().into_owned())
}

fn scripts_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has no parent")
        .join("scripts")
}

// ── Request structs ───────────────────────────────────────────────────────────

/// Receives the canvas PNG as a base64 dataURL: "data:image/png;base64,..."
#[derive(Deserialize)]
pub struct ExportBody {
    pub data: String,
}

// ── Route handlers ────────────────────────────────────────────────────────────

/// POST /api/photo/export  (JSON body: { "data": "data:image/png;base64,..." })
/// Strips the dataURL prefix, base64-decodes, saves as PNG to ~/Pictures/Eleutheria/.
pub async fn export_handler(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<ExportBody>,
) -> impl IntoResponse {
    // Strip the dataURL header: "data:image/png;base64,"
    let b64 = match body.data.split_once(',') {
        Some((_, b)) => b,
        None => {
            return Html(r#"<p class="text-red-400 text-sm">Invalid image data.</p>"#.to_string())
                .into_response();
        }
    };

    use base64::Engine;
    let bytes = match base64::engine::general_purpose::STANDARD.decode(b64) {
        Ok(b) => b,
        Err(e) => {
            return Html(format!(
                r##"<p class="text-red-400 text-sm">Decode error: {}</p>"##,
                html_escape(&e.to_string())
            ))
            .into_response();
        }
    };

    let path = match export_path() {
        Ok(p) => p,
        Err(e) => {
            return Html(format!(
                r##"<p class="text-red-400 text-sm">Could not create output directory: {}</p>"##,
                html_escape(&e.to_string())
            ))
            .into_response();
        }
    };

    if let Err(e) = tokio::fs::write(&path, &bytes).await {
        return Html(format!(
            r##"<p class="text-red-400 text-sm">Write error: {}</p>"##,
            html_escape(&e.to_string())
        ))
        .into_response();
    }

    let escaped = html_escape(&path);
    Html(format!(
        r##"<div class="flex flex-col gap-2">
  <span class="text-green-400 text-sm font-medium">✓ Exported</span>
  <code class="text-xs text-gray-300 bg-gray-800 rounded px-3 py-2 break-all select-all">{escaped}</code>
  <p class="text-xs text-gray-500">Saved to ~/Pictures/Eleutheria/</p>
</div>"##
    ))
    .into_response()
}

/// POST /api/photo/rembg  (multipart/form-data, field: "image")
/// Runs scripts/rembg_remove.py on the uploaded image.
/// Returns JSON { "ok": true, "png_b64": "..." } on success.
pub async fn rembg_handler(
    State(_state): State<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> impl IntoResponse {
    let mut image_bytes: Option<Vec<u8>> = None;
    let mut ext = "png".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            if let Some(fname) = field.file_name() {
                if let Some(e) = fname.rsplit('.').next() {
                    let e = e.to_lowercase();
                    if matches!(e.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp") {
                        ext = e;
                    }
                }
            }
            match field.bytes().await {
                Ok(b) => image_bytes = Some(b.to_vec()),
                Err(e) => {
                    return Json(json!({ "ok": false, "error": e.to_string() })).into_response();
                }
            }
        }
    }

    let bytes = match image_bytes {
        Some(b) if !b.is_empty() => b,
        _ => {
            return Json(json!({ "ok": false, "error": "No image received." })).into_response();
        }
    };

    let tmp_path = format!("/tmp/eleutheria-photo-rembg-input.{ext}");
    if let Err(e) = tokio::fs::write(&tmp_path, &bytes).await {
        return Json(json!({ "ok": false, "error": e.to_string() })).into_response();
    }

    let script = scripts_dir().join("rembg_remove.py");
    let result = tokio::process::Command::new("python3")
        .arg(&script)
        .arg(&tmp_path)
        .output()
        .await;

    match result {
        Ok(o) if o.status.success() => {
            let png_b64 = String::from_utf8_lossy(&o.stdout).trim().to_string();
            Json(json!({ "ok": true, "png_b64": png_b64 })).into_response()
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            Json(json!({ "ok": false, "error": err })).into_response()
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })).into_response(),
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/photo/export", post(export_handler))
        .route("/api/photo/rembg", post(rembg_handler))
}
