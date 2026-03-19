use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Router,
};
use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::server::{AppError, AppState};

// ── Download tracking ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DownloadState {
    pub progress: u8,   // 0–100
    pub status: String, // "downloading" | "done" | "error"
    pub error: Option<String>,
}

pub type DownloadMap = Arc<Mutex<HashMap<String, DownloadState>>>;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// ~/.local/share/eleutheria-telos/models/
fn models_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".local/share/eleutheria-telos/models")
}

/// "whisper-tiny" → models_dir/whisper/ggml-tiny.bin
fn whisper_path(model_id: &str) -> PathBuf {
    let name = model_id.strip_prefix("whisper-").unwrap_or(model_id);
    models_dir()
        .join("whisper")
        .join(format!("ggml-{name}.bin"))
}

/// "argos-en-es" → Some(("en", "es"))
fn argos_lang_pair(model_id: &str) -> Option<(String, String)> {
    let rest = model_id.strip_prefix("argos-")?;
    let mut parts = rest.splitn(2, '-');
    let from = parts.next()?.to_string();
    let to = parts.next()?.to_string();
    Some((from, to))
}

/// Scripts directory: ../scripts/ relative to src-tauri/ at compile time.
/// Phase 5: replace with Tauri resource path from AppHandle.
fn scripts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("scripts")
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── DB row ────────────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow, Clone)]
struct ModelRow {
    id: String,
    name: String,
    tool: String,
    size_bytes: Option<i64>,
    path: Option<String>,
    downloaded: i64,
    url: Option<String>,
}

// ── Card rendering ────────────────────────────────────────────────────────────

fn render_card(row: &ModelRow, ds: Option<&DownloadState>) -> String {
    let size_str = row
        .size_bytes
        .map(|b| {
            if b >= 1_000_000_000 {
                format!("{:.1} GB", b as f64 / 1_000_000_000.0)
            } else {
                format!("{} MB", b / 1_000_000)
            }
        })
        .unwrap_or_else(|| "~100 MB".to_string());

    let name = html_escape(&row.name);
    let id = &row.id;

    // Pre-compute the CSS selector to avoid "# inside r#"..."# raw string termination.
    // `hx-target="#model-card-{id}"` contains `"#` which prematurely closes r#"..."#.
    let target = format!("#model-card-{id}");

    match ds {
        Some(state) if state.status == "downloading" => {
            // Card with progress bar — polls itself every 2s.
            let progress = state.progress;
            format!(
                r#"<div id="model-card-{id}" class="card" style="display:flex;align-items:center;justify-content:space-between;margin-bottom:8px;"
                        hx-get="/api/models/{id}/progress"
                        hx-trigger="every 2s"
                        hx-swap="outerHTML">
  <div style="flex:1;min-width:0;margin-right:12px;">
    <p style="font-size:13px;font-weight:500;color:var(--text-primary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{name}</p>
    <div style="margin-top:6px;width:100%;background:var(--border);border-radius:999px;height:4px;">
      <div style="background:var(--accent);height:4px;border-radius:999px;transition:width 300ms;width:{progress}%;"></div>
    </div>
    <p style="font-size:11px;color:var(--text-muted);margin-top:4px;">Downloading… {progress}%</p>
  </div>
  <button disabled class="btn btn-ghost btn-sm btn-disabled">Cancel</button>
</div>"#
            )
        }
        Some(state) if state.status == "error" => {
            let err_html = html_escape(state.error.as_deref().unwrap_or("Unknown error"));
            format!(
                r#"<div id="model-card-{id}" class="card" style="display:flex;align-items:center;justify-content:space-between;margin-bottom:8px;">
  <div>
    <p style="font-size:13px;font-weight:500;color:var(--text-primary);">{name}</p>
    <p style="font-size:11px;color:var(--destructive);margin-top:4px;">{err_html}</p>
  </div>
  <button class="btn btn-secondary btn-sm" style="margin-left:12px;"
          hx-post="/api/models/{id}/download"
          hx-target="{target}"
          hx-swap="outerHTML">Retry</button>
</div>"#
            )
        }
        _ if row.downloaded == 1 => {
            format!(
                r#"<div id="model-card-{id}" class="card" style="display:flex;align-items:center;justify-content:space-between;margin-bottom:8px;">
  <div>
    <p style="font-size:13px;font-weight:500;color:var(--text-primary);">{name}</p>
    <p style="font-size:11px;color:var(--success);margin-top:4px;">Downloaded</p>
  </div>
  <button class="btn btn-danger btn-sm"
          hx-delete="/api/models/{id}"
          hx-target="{target}"
          hx-swap="outerHTML"
          hx-confirm="Delete this model?">Delete</button>
</div>"#
            )
        }
        _ => {
            // Not downloaded, not in progress
            format!(
                r#"<div id="model-card-{id}" class="card" style="display:flex;align-items:center;justify-content:space-between;margin-bottom:8px;">
  <div>
    <p style="font-size:13px;font-weight:500;color:var(--text-primary);">{name}</p>
    <p style="font-size:11px;color:var(--text-muted);margin-top:4px;">{size_str}</p>
  </div>
  <button class="btn btn-primary btn-sm"
          hx-post="/api/models/{id}/download"
          hx-target="{target}"
          hx-swap="outerHTML">Download</button>
</div>"#
            )
        }
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rows: Vec<ModelRow> = sqlx::query_as(
        "SELECT id, name, tool, size_bytes, path, downloaded, url
         FROM models ORDER BY tool, name",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let dl = state.download_states.lock().await;

    let mut voice_html = String::new();
    let mut translate_html = String::new();

    for row in &rows {
        let ds = dl.get(&row.id);
        let card = render_card(row, ds);
        if row.tool == "voice" {
            voice_html.push_str(&card);
        } else {
            translate_html.push_str(&card);
        }
    }

    if voice_html.is_empty() {
        voice_html =
            r#"<p style="font-size:13px;color:var(--text-muted);">No voice models in catalog.</p>"#.to_string();
    }
    if translate_html.is_empty() {
        translate_html =
            r#"<p style="font-size:13px;color:var(--text-muted);">No translation models in catalog.</p>"#.to_string();
    }

    Html(format!(
        r#"<div id="models-list">
  <section style="margin-bottom:24px;">
    <h3 style="font-size:11px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;margin-bottom:12px;">Voice (Whisper)</h3>
    {voice_html}
  </section>
  <section>
    <h3 style="font-size:11px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.05em;margin-bottom:12px;">Translation (Argos)</h3>
    {translate_html}
  </section>
</div>"#
    ))
}

pub async fn progress_handler(
    Path(model_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let row: Option<ModelRow> = sqlx::query_as(
        "SELECT id, name, tool, size_bytes, path, downloaded, url FROM models WHERE id = ?",
    )
    .bind(&model_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let Some(row) = row else {
        return (
            StatusCode::NOT_FOUND,
            Html(format!(
                r#"<div id="model-card-{model_id}"><p style="font-size:13px;color:var(--destructive);">Model not found.</p></div>"#
            )),
        )
            .into_response();
    };

    let dl = state.download_states.lock().await;
    let ds = dl.get(&model_id);
    Html(render_card(&row, ds)).into_response()
}

pub async fn download_handler(
    Path(model_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Guard: already downloaded or already in progress
    {
        let dl = state.download_states.lock().await;
        if let Some(ds) = dl.get(&model_id) {
            if ds.status == "downloading" {
                let row: Option<ModelRow> = sqlx::query_as(
                    "SELECT id, name, tool, size_bytes, path, downloaded, url FROM models WHERE id = ?",
                )
                .bind(&model_id)
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None);
                if let Some(row) = row {
                    return Html(render_card(&row, Some(ds))).into_response();
                }
            }
        }
    }

    let row: Option<ModelRow> = sqlx::query_as(
        "SELECT id, name, tool, size_bytes, path, downloaded, url FROM models WHERE id = ?",
    )
    .bind(&model_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let Some(row) = row else {
        return AppError::not_found(&model_id).into_response();
    };

    if row.downloaded == 1 {
        return Html(render_card(&row, None)).into_response();
    }

    // Mark as downloading
    {
        let mut dl = state.download_states.lock().await;
        dl.insert(
            model_id.clone(),
            DownloadState {
                progress: 0,
                status: "downloading".to_string(),
                error: None,
            },
        );
    }

    // Spawn background download
    let states = state.download_states.clone();
    let db = state.db.clone();
    let mid = model_id.clone();
    let row_clone = row.clone();

    tokio::spawn(async move {
        if row_clone.tool == "voice" {
            download_whisper(mid, row_clone, states, db).await;
        } else {
            download_argos(mid, row_clone, states, db).await;
        }
    });

    // Return the "downloading" card immediately (it will start polling itself)
    let dl = state.download_states.lock().await;
    let ds = dl.get(&model_id);
    Html(render_card(&row, ds)).into_response()
}

pub async fn delete_handler(
    Path(model_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let row: Option<ModelRow> = sqlx::query_as(
        "SELECT id, name, tool, size_bytes, path, downloaded, url FROM models WHERE id = ?",
    )
    .bind(&model_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let Some(row) = row else {
        return AppError::not_found(&model_id).into_response();
    };

    // Remove the file if present
    if let Some(path_str) = &row.path {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }

    // For argos: remove via Python
    if row.tool == "translate" {
        if let Some((from, to)) = argos_lang_pair(&model_id) {
            let script = scripts_dir().join("uninstall_argos_package.py");
            let _ = tokio::process::Command::new("python3")
                .arg(&script)
                .arg(&from)
                .arg(&to)
                .output()
                .await;
        }
    }

    // Reset DB
    let _ = sqlx::query(
        "UPDATE models SET downloaded = 0, path = NULL, downloaded_at = NULL WHERE id = ?",
    )
    .bind(&model_id)
    .execute(&state.db)
    .await;

    // Clear download state
    state.download_states.lock().await.remove(&model_id);

    // Return the "not downloaded" card
    let updated: Option<ModelRow> = sqlx::query_as(
        "SELECT id, name, tool, size_bytes, path, downloaded, url FROM models WHERE id = ?",
    )
    .bind(&model_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    Html(render_card(updated.as_ref().unwrap_or(&row), None)).into_response()
}

// ── Background download tasks ─────────────────────────────────────────────────

async fn download_whisper(model_id: String, row: ModelRow, states: DownloadMap, db: SqlitePool) {
    let url = match &row.url {
        Some(u) => u.clone(),
        None => {
            set_error(&states, &model_id, "No download URL configured").await;
            return;
        }
    };

    let dest = whisper_path(&model_id);
    if let Some(parent) = dest.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            set_error(&states, &model_id, &e.to_string()).await;
            return;
        }
    }

    let client = reqwest::Client::new();
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            set_error(&states, &model_id, &format!("HTTP {}", r.status())).await;
            return;
        }
        Err(e) => {
            set_error(&states, &model_id, &e.to_string()).await;
            return;
        }
    };

    let total = resp.content_length().unwrap_or(0);
    let tmp = dest.with_extension("tmp");

    let mut file = match tokio::fs::File::create(&tmp).await {
        Ok(f) => f,
        Err(e) => {
            set_error(&states, &model_id, &e.to_string()).await;
            return;
        }
    };

    let mut downloaded: u64 = 0;
    let mut resp = resp;

    use tokio::io::AsyncWriteExt;
    loop {
        match resp.chunk().await {
            Ok(Some(chunk)) => {
                if let Err(e) = file.write_all(&chunk).await {
                    set_error(&states, &model_id, &e.to_string()).await;
                    let _ = tokio::fs::remove_file(&tmp).await;
                    return;
                }
                downloaded += chunk.len() as u64;
                let progress = if total > 0 {
                    (downloaded * 100 / total).min(99) as u8
                } else {
                    50
                };
                let mut dl = states.lock().await;
                dl.insert(
                    model_id.clone(),
                    DownloadState {
                        progress,
                        status: "downloading".to_string(),
                        error: None,
                    },
                );
            }
            Ok(None) => break,
            Err(e) => {
                set_error(&states, &model_id, &e.to_string()).await;
                let _ = tokio::fs::remove_file(&tmp).await;
                return;
            }
        }
    }

    if let Err(e) = file.flush().await {
        set_error(&states, &model_id, &e.to_string()).await;
        let _ = tokio::fs::remove_file(&tmp).await;
        return;
    }
    drop(file);

    if let Err(e) = tokio::fs::rename(&tmp, &dest).await {
        set_error(&states, &model_id, &e.to_string()).await;
        return;
    }

    let path_str = dest.to_string_lossy().to_string();
    let _ =
        sqlx::query("UPDATE models SET downloaded = 1, path = ?, downloaded_at = ? WHERE id = ?")
            .bind(&path_str)
            .bind(now_secs())
            .bind(&model_id)
            .execute(&db)
            .await;

    states.lock().await.insert(
        model_id,
        DownloadState {
            progress: 100,
            status: "done".to_string(),
            error: None,
        },
    );
}

async fn download_argos(model_id: String, _row: ModelRow, states: DownloadMap, db: SqlitePool) {
    let Some((from, to)) = argos_lang_pair(&model_id) else {
        set_error(&states, &model_id, "Invalid model ID for Argos").await;
        return;
    };

    let script = scripts_dir().join("install_argos_package.py");
    if !script.exists() {
        set_error(&states, &model_id, "install_argos_package.py not found").await;
        return;
    }

    let output = tokio::process::Command::new("python3")
        .arg(&script)
        .arg(&from)
        .arg(&to)
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            let _ = sqlx::query("UPDATE models SET downloaded = 1, downloaded_at = ? WHERE id = ?")
                .bind(now_secs())
                .bind(&model_id)
                .execute(&db)
                .await;

            states.lock().await.insert(
                model_id,
                DownloadState {
                    progress: 100,
                    status: "done".to_string(),
                    error: None,
                },
            );
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            set_error(&states, &model_id, &stderr).await;
        }
        Err(e) => {
            set_error(&states, &model_id, &e.to_string()).await;
        }
    }
}

async fn set_error(states: &DownloadMap, model_id: &str, msg: &str) {
    states.lock().await.insert(
        model_id.to_string(),
        DownloadState {
            progress: 0,
            status: "error".to_string(),
            error: Some(msg.to_string()),
        },
    );
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/models", get(list_handler))
        .route("/api/models/:id/download", post(download_handler))
        .route("/api/models/:id/progress", get(progress_handler))
        .route("/api/models/:id", delete(delete_handler))
}
