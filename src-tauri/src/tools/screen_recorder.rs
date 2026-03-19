use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

use crate::server::AppState;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Holds the wf-recorder child process, output path, and start timestamp.
pub type ScreenRecording = Arc<Mutex<Option<(tokio::process::Child, String, u64)>>>;

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

/// Returns ~/Videos/Eleutheria/screen-{timestamp}.mp4, creating the dir if needed.
fn recording_path() -> std::io::Result<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = std::path::PathBuf::from(&home)
        .join("Videos")
        .join("Eleutheria");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("screen-{}.mp4", now_secs()));
    Ok(path.to_string_lossy().into_owned())
}

/// Runs `wlr-randr` and returns enabled output names (e.g. ["eDP-1", "HDMI-A-2"]).
/// Falls back to empty vec if wlr-randr is unavailable.
fn detect_outputs() -> Vec<String> {
    let Ok(out) = std::process::Command::new("wlr-randr").output() else {
        return vec![];
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Non-indented lines are output headers: "eDP-1 \"description\""
    stdout
        .lines()
        .filter(|l| !l.starts_with(' ') && !l.is_empty())
        .filter_map(|l| l.split_whitespace().next().map(str::to_string))
        .collect()
}

// ── Request structs ───────────────────────────────────────────────────────────

/// HTML checkboxes send "on" when checked and nothing when unchecked.
#[derive(Deserialize)]
pub struct StartParams {
    #[serde(default)]
    pub audio: String,
    #[serde(default)]
    pub output: String,
}

// ── Route handlers ────────────────────────────────────────────────────────────

/// GET /api/screen/outputs
/// Returns HTML <option> elements for the output selector, loaded on panel init.
pub async fn outputs_handler() -> impl IntoResponse {
    let outputs = detect_outputs();
    if outputs.is_empty() {
        return Html(r#"<option value="">Default output</option>"#.to_string());
    }
    let options: String = outputs
        .iter()
        .map(|o| {
            let e = html_escape(o);
            format!(r#"<option value="{e}">{e}</option>"#)
        })
        .collect();
    Html(options)
}

/// GET /api/screen/state
/// Returns JSON { "recording": bool, "started_at": u64 } for panel state restore.
pub async fn state_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let recording = state.screen_recording.lock().await;
    match &*recording {
        Some((_, _, started_at)) => Json(json!({ "recording": true, "started_at": started_at })),
        None => Json(json!({ "recording": false, "started_at": 0 })),
    }
}

/// GET /api/screen/status
/// Returns an HTML badge: "● Recording" or "Idle".
pub async fn status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let recording = state.screen_recording.lock().await;
    if recording.is_some() {
        Html(
            r#"<span style="color:var(--destructive);font-weight:500;">● Recording</span>"#
                .to_string(),
        )
    } else {
        Html(r#"<span style="color:var(--text-muted);">Idle</span>"#.to_string())
    }
}

/// POST /api/screen/start  (form-encoded, fields: audio="on", output="eDP-1")
/// Saves to ~/Videos/Eleutheria/screen-{ts}.mp4.
pub async fn record_start_handler(
    State(state): State<Arc<AppState>>,
    Form(params): Form<StartParams>,
) -> impl IntoResponse {
    let mut recording = state.screen_recording.lock().await;
    if recording.is_some() {
        return Html(r#"<p style="font-size:13px;color:var(--text-muted);">Already recording.</p>"#.to_string())
            .into_response();
    }

    let output_path = match recording_path() {
        Ok(p) => p,
        Err(e) => {
            return Html(format!(
                r#"<p style="font-size:13px;color:var(--destructive);">Could not create output directory: {}</p>"#,
                html_escape(&e.to_string())
            ))
            .into_response();
        }
    };

    let with_audio = !params.audio.is_empty();

    let mut cmd = tokio::process::Command::new("wf-recorder");
    cmd.arg("-f").arg(&output_path);
    // Specify output explicitly to avoid the interactive selection prompt
    // when multiple monitors are connected (D-028).
    if !params.output.is_empty() {
        cmd.arg("-o").arg(&params.output);
    }
    if with_audio {
        cmd.arg("-a");
    }
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    match cmd.spawn() {
        Ok(child) => {
            let started_at = now_secs();
            *recording = Some((child, output_path.clone(), started_at));
            let audio_note = if with_audio { " (with audio)" } else { "" };
            let out_note = html_escape(&params.output);
            Html(format!(
                r#"<p style="font-size:13px;color:var(--destructive);font-weight:500;">● Recording {out_note}{audio_note}… return to this panel to stop.</p>"#
            ))
            .into_response()
        }
        Err(e) => Html(format!(
            r#"<p style="font-size:13px;color:var(--destructive);">Failed to start wf-recorder: {}</p>"#,
            html_escape(&e.to_string())
        ))
        .into_response(),
    }
}

/// POST /api/screen/stop
/// Sends SIGTERM to wf-recorder so it finalizes the mp4 container (D-028).
pub async fn record_stop_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut recording = state.screen_recording.lock().await;
    let (mut child, output_path, _) = match recording.take() {
        Some(p) => p,
        None => {
            return Html(
                r#"<p style="font-size:13px;color:var(--text-muted);">No recording in progress.</p>"#.to_string(),
            )
            .into_response();
        }
    };

    if let Some(pid) = child.id() {
        let _ = std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
    }
    let _ = child.wait().await;

    match tokio::fs::metadata(&output_path).await {
        Ok(m) if m.len() > 0 => {}
        _ => {
            return Html(
                r#"<p style="font-size:13px;color:var(--text-muted);">Recording was empty or was not saved.</p>"#
                    .to_string(),
            )
            .into_response();
        }
    }

    let escaped_path = html_escape(&output_path);
    Html(format!(
        r##"<div style="margin-top:16px;display:flex;flex-direction:column;gap:12px;">
  <span style="font-size:13px;color:var(--success);font-weight:500;">✓ Recording saved</span>
  <code style="font-size:11px;color:var(--text-secondary);background:var(--bg-elevated);border-radius:var(--radius-sm);padding:8px 12px;word-break:break-all;user-select:all;">{escaped_path}</code>
  <p style="font-size:11px;color:var(--text-muted);">Saved to ~/Videos/Eleutheria/</p>
</div>"##
    ))
    .into_response()
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/screen/outputs", get(outputs_handler))
        .route("/api/screen/state", get(state_handler))
        .route("/api/screen/status", get(status_handler))
        .route("/api/screen/start", post(record_start_handler))
        .route("/api/screen/stop", post(record_stop_handler))
}
