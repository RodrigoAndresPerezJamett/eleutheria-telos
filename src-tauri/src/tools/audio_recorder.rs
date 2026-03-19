use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::server::AppState;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Holds the ffmpeg child process, output path, and start timestamp.
pub type AudioRecording = Arc<Mutex<Option<(tokio::process::Child, String, u64)>>>;

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

/// Returns ~/Music/Eleutheria/recording-{timestamp}.{ext}, creating dir if needed.
fn recording_path(ext: &str) -> std::io::Result<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = std::path::PathBuf::from(&home)
        .join("Music")
        .join("Eleutheria");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("recording-{}.{}", now_secs(), ext));
    Ok(path.to_string_lossy().into_owned())
}

/// ffmpeg codec args for each supported format.
fn codec_args(format: &str) -> &'static [&'static str] {
    match format {
        "wav" => &["-c:a", "pcm_s16le"],
        "ogg" => &["-c:a", "libvorbis"],
        "flac" => &["-c:a", "flac"],
        _ => &["-c:a", "libmp3lame"], // mp3 default
    }
}

// ── Request structs ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StartParams {
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "mp3".to_string()
}

// ── Route handlers ────────────────────────────────────────────────────────────

/// GET /api/audio/state
/// Returns JSON { "recording": bool, "started_at": u64 } for panel state restore.
pub async fn state_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let recording = state.audio_recording.lock().await;
    match &*recording {
        Some((_, _, started_at)) => Json(json!({ "recording": true, "started_at": started_at })),
        None => Json(json!({ "recording": false, "started_at": 0 })),
    }
}

/// GET /api/audio/status — HTML badge.
pub async fn status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let recording = state.audio_recording.lock().await;
    if recording.is_some() {
        Html(
            r#"<span class="text-red-400 font-medium text-sm animate-pulse">● Recording</span>"#
                .to_string(),
        )
    } else {
        Html(r#"<span class="text-gray-500 text-sm">Idle</span>"#.to_string())
    }
}

/// POST /api/audio/record/start  (form-encoded, field: format)
/// Spawns ffmpeg recording from the default PulseAudio source.
pub async fn record_start_handler(
    State(state): State<Arc<AppState>>,
    Form(params): Form<StartParams>,
) -> impl IntoResponse {
    let mut recording = state.audio_recording.lock().await;
    if recording.is_some() {
        return Html(r#"<p class="text-yellow-400 text-sm">Already recording.</p>"#.to_string())
            .into_response();
    }

    let ext = match params.format.as_str() {
        "wav" | "ogg" | "flac" => params.format.as_str(),
        _ => "mp3",
    };

    let output_path = match recording_path(ext) {
        Ok(p) => p,
        Err(e) => {
            return Html(format!(
                r##"<p class="text-red-400 text-sm">Could not create output directory: {}</p>"##,
                html_escape(&e.to_string())
            ))
            .into_response();
        }
    };

    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.args(["-y", "-f", "pulse", "-i", "default"])
        .args(codec_args(ext))
        .arg(&output_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    match cmd.spawn() {
        Ok(child) => {
            let started_at = now_secs();
            *recording = Some((child, output_path.clone(), started_at));
            Html(format!(
                r##"<p class="text-red-400 text-sm font-medium animate-pulse">● Recording {ext}… press Stop when done.</p>"##
            ))
            .into_response()
        }
        Err(e) => Html(format!(
            r##"<p class="text-red-400 text-sm">Failed to start ffmpeg: {}</p>"##,
            html_escape(&e.to_string())
        ))
        .into_response(),
    }
}

/// POST /api/audio/record/stop
/// Stops ffmpeg gracefully via stdin 'q\n' (same pattern as voice.rs).
pub async fn record_stop_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut recording = state.audio_recording.lock().await;
    let (child, output_path, _) = match recording.take() {
        Some(p) => p,
        None => {
            return Html(
                r#"<p class="text-gray-400 text-sm">No recording in progress.</p>"#.to_string(),
            )
            .into_response();
        }
    };

    let mut child = child;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"q\n").await;
        drop(stdin);
    }
    let _ = child.wait().await;

    match tokio::fs::metadata(&output_path).await {
        Ok(m) if m.len() > 0 => {}
        _ => {
            return Html(
                r#"<p class="text-gray-400 text-sm">Recording was empty or was not saved.</p>"#
                    .to_string(),
            )
            .into_response();
        }
    }

    let escaped_path = html_escape(&output_path);
    let ext = std::path::Path::new(&output_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("audio")
        .to_uppercase();

    Html(format!(
        r##"<div class="mt-4 flex flex-col gap-3">
  <span class="text-green-400 text-sm font-medium">✓ {ext} saved</span>
  <code class="text-xs text-gray-300 bg-gray-800 rounded px-3 py-2 break-all select-all">{escaped_path}</code>
  <p class="text-xs text-gray-500">Saved to ~/Music/Eleutheria/</p>
</div>"##
    ))
    .into_response()
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/audio/state", get(state_handler))
        .route("/api/audio/status", get(status_handler))
        .route("/api/audio/record/start", post(record_start_handler))
        .route("/api/audio/record/stop", post(record_stop_handler))
}
