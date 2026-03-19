use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
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

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn video_out(label: &str) -> std::io::Result<String> {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = PathBuf::from(&home).join("Videos").join("Eleutheria");
    std::fs::create_dir_all(&dir)?;
    Ok(dir
        .join(format!("video-{}-{}.mp4", label, now_secs()))
        .to_string_lossy()
        .into_owned())
}

fn audio_out(ext: &str) -> std::io::Result<String> {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = PathBuf::from(&home).join("Music").join("Eleutheria");
    std::fs::create_dir_all(&dir)?;
    Ok(dir
        .join(format!("audio-{}.{}", now_secs(), ext))
        .to_string_lossy()
        .into_owned())
}

fn success_card(output_path: &str, label: &str) -> String {
    let escaped = html_escape(output_path);
    format!(
        r##"<div class="flex flex-col gap-2">
  <span class="text-green-400 text-sm font-medium">✓ {label} complete</span>
  <code class="text-xs text-gray-300 bg-gray-800 rounded px-3 py-2 break-all select-all">{escaped}</code>
</div>"##
    )
}

fn error_card(msg: &str) -> String {
    let escaped = html_escape(msg);
    format!(r##"<pre class="text-red-400 text-xs whitespace-pre-wrap break-all">{escaped}</pre>"##)
}

/// Trims ffmpeg's verbose stderr to the last N lines.
fn trim_stderr(raw: &[u8]) -> String {
    let s = String::from_utf8_lossy(raw);
    let lines: Vec<&str> = s.lines().collect();
    let start = lines.len().saturating_sub(25);
    lines[start..].join("\n")
}

fn ffmpeg_result(
    result: Result<std::process::Output, std::io::Error>,
    output_path: &str,
    label: &str,
) -> axum::response::Response {
    match result {
        Ok(o) if o.status.success() => Html(success_card(output_path, label)).into_response(),
        Ok(o) => Html(error_card(&trim_stderr(&o.stderr))).into_response(),
        Err(e) => Html(error_card(&e.to_string())).into_response(),
    }
}

// ── Request struct ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ProcessForm {
    pub path: String,
    pub operation: String,
    // trim
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub end: String,
    // extract_audio
    #[serde(default)]
    pub audio_format: String,
    // compress — separate name from resize to avoid duplicate-field ambiguity
    #[serde(default = "default_qp")]
    pub qp: u32,
    #[serde(default)]
    pub compress_resolution: String,
    // resize
    #[serde(default)]
    pub resize_resolution: String,
}

fn default_qp() -> u32 {
    28
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// POST /api/video/process  (form-urlencoded)
/// Dispatches to the right ffmpeg command based on `operation`.
pub async fn process_handler(
    State(_state): State<Arc<AppState>>,
    Form(form): Form<ProcessForm>,
) -> impl IntoResponse {
    let input = form.path.trim().to_string();

    if input.is_empty() {
        return Html(error_card("Input path is required.")).into_response();
    }
    if !std::path::Path::new(&input).is_file() {
        return Html(error_card(&format!("File not found: {input}"))).into_response();
    }

    match form.operation.as_str() {
        // ── Trim ─────────────────────────────────────────────────────────────
        // Uses stream copy: no re-encoding, nearly instant.
        "trim" => {
            let start = form.start.trim().to_string();
            let end = form.end.trim().to_string();
            if start.is_empty() || end.is_empty() {
                return Html(error_card("Start and end times are required.")).into_response();
            }
            let output = match video_out("trim") {
                Ok(p) => p,
                Err(e) => return Html(error_card(&e.to_string())).into_response(),
            };
            let result = tokio::process::Command::new("ffmpeg")
                .args([
                    "-y", "-i", &input, "-ss", &start, "-to", &end, "-c", "copy", &output,
                ])
                .output()
                .await;
            ffmpeg_result(result, &output, "Trim")
        }

        // ── Extract audio ─────────────────────────────────────────────────────
        "extract_audio" => {
            let (codec, ext) = match form.audio_format.as_str() {
                "wav" => ("pcm_s16le", "wav"),
                "flac" => ("flac", "flac"),
                _ => ("libmp3lame", "mp3"),
            };
            let output = match audio_out(ext) {
                Ok(p) => p,
                Err(e) => return Html(error_card(&e.to_string())).into_response(),
            };
            let result = tokio::process::Command::new("ffmpeg")
                .args(["-y", "-i", &input, "-vn", "-c:a", codec, &output])
                .output()
                .await;
            ffmpeg_result(result, &output, "Extract audio")
        }

        // ── Compress ──────────────────────────────────────────────────────────
        // Re-encodes with h264_vaapi (GPU). QP 18–40 (lower = better quality).
        // Optionally downscales if compress_resolution is set.
        "compress" => {
            let qp = form.qp.clamp(18, 40).to_string();
            let vf = match form.compress_resolution.as_str() {
                "1080" | "720" | "480" => {
                    format!("scale=-2:{},format=nv12,hwupload", form.compress_resolution)
                }
                _ => "format=nv12,hwupload".to_string(),
            };
            let output = match video_out("compressed") {
                Ok(p) => p,
                Err(e) => return Html(error_card(&e.to_string())).into_response(),
            };
            let result = tokio::process::Command::new("ffmpeg")
                .args([
                    "-y",
                    "-vaapi_device",
                    "/dev/dri/renderD128",
                    "-i",
                    &input,
                    "-vf",
                    &vf,
                    "-c:v",
                    "h264_vaapi",
                    "-qp",
                    &qp,
                    &output,
                ])
                .output()
                .await;
            ffmpeg_result(result, &output, "Compress")
        }

        // ── Resize ────────────────────────────────────────────────────────────
        // Changes resolution with h264_vaapi, preserving aspect ratio.
        "resize" => {
            let height = form.resize_resolution.trim().to_string();
            if !matches!(height.as_str(), "1080" | "720" | "480" | "360" | "240") {
                return Html(error_card("Select a valid target resolution.")).into_response();
            }
            let vf = format!("scale=-2:{height},format=nv12,hwupload");
            let output = match video_out(&format!("{height}p")) {
                Ok(p) => p,
                Err(e) => return Html(error_card(&e.to_string())).into_response(),
            };
            let result = tokio::process::Command::new("ffmpeg")
                .args([
                    "-y",
                    "-vaapi_device",
                    "/dev/dri/renderD128",
                    "-i",
                    &input,
                    "-vf",
                    &vf,
                    "-c:v",
                    "h264_vaapi",
                    "-qp",
                    "28",
                    &output,
                ])
                .output()
                .await;
            ffmpeg_result(result, &output, "Resize")
        }

        other => Html(error_card(&format!("Unknown operation: {other}"))).into_response(),
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/video/process", post(process_handler))
}
