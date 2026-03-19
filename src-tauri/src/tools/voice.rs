use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::event_bus::Event;
use crate::server::AppState;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Holds the ffmpeg child process while recording is in progress.
pub type VoiceRecording = Arc<Mutex<Option<tokio::process::Child>>>;

// ── Constants ─────────────────────────────────────────────────────────────────

const RECORDING_TMP: &str = "/tmp/eleutheria-voice-recording.wav";
const UPLOAD_TMP_BASE: &str = "/tmp/eleutheria-voice-upload";

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

fn scripts_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has no parent")
        .join("scripts")
}

/// Run `scripts/transcribe.py` on an audio file and return the result HTML.
async fn run_transcription(audio_path: &str, lang: &str, state: &Arc<AppState>) -> Response {
    let script = scripts_dir().join("transcribe.py");
    let mut cmd = tokio::process::Command::new("python3");
    cmd.arg(&script).arg(audio_path);
    if !lang.is_empty() && lang != "auto" {
        cmd.arg("--lang").arg(lang);
    }
    match cmd.output().await {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if text.is_empty() {
                return Html(
                    r#"<p class="text-gray-400 text-sm mt-4">No speech detected in audio.</p>"#
                        .to_string(),
                )
                .into_response();
            }
            state.event_bus.publish(Event::TranscriptionCompleted {
                text: text.clone(),
                language: lang.to_string(),
            });
            render_result(&text).into_response()
        }
        Ok(o) => {
            let err = html_escape(String::from_utf8_lossy(&o.stderr).trim());
            Html(format!(
                r##"<p class="text-red-400 text-sm mt-4">Transcription error: {err}</p>"##
            ))
            .into_response()
        }
        Err(e) => Html(format!(
            r##"<p class="text-red-400 text-sm mt-4">Could not run python3: {}</p>"##,
            html_escape(&e.to_string())
        ))
        .into_response(),
    }
}

/// Build the result card shown after successful transcription.
fn render_result(text: &str) -> Html<String> {
    let escaped = html_escape(text);
    // Pre-compute CSS ID references to avoid "# terminating r#"..."# strings (D-023).
    let include_target = "#voice-text-form";
    let feedback_target = "#voice-feedback";
    Html(format!(
        r##"<div class="mt-4 flex flex-col gap-3">
  <pre class="text-sm text-gray-200 bg-gray-800 rounded-lg p-4 whitespace-pre-wrap break-words max-h-64 overflow-y-auto font-sans leading-relaxed">{escaped}</pre>
  <form id="voice-text-form">
    <textarea name="text" class="hidden">{escaped}</textarea>
  </form>
  <div class="flex gap-2">
    <button class="text-xs text-blue-400 hover:text-blue-300 border border-blue-700 rounded px-3 py-1.5"
            hx-post="/api/voice/copy"
            hx-include="{include_target}"
            hx-target="{feedback_target}"
            hx-swap="innerHTML">Copy to Clipboard</button>
    <button class="text-xs text-green-400 hover:text-green-300 border border-green-700 rounded px-3 py-1.5"
            hx-post="/api/voice/save-note"
            hx-include="{include_target}"
            hx-target="{feedback_target}"
            hx-swap="innerHTML">Save as Note</button>
  </div>
  <div id="voice-feedback" class="text-xs"></div>
</div>"##
    ))
}

// ── Request structs ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StopParams {
    #[serde(default = "default_lang")]
    pub lang: String,
}

#[derive(Deserialize)]
pub struct TextBody {
    pub text: String,
}

fn default_lang() -> String {
    "auto".to_string()
}

// ── Route handlers ────────────────────────────────────────────────────────────

/// GET /api/voice/status
/// Returns a small HTML badge indicating whether recording is active.
pub async fn status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let recording = state.voice_recording.lock().await;
    if recording.is_some() {
        Html(r#"<span class="text-red-400 font-medium">● Recording</span>"#.to_string())
    } else {
        Html(r#"<span class="text-gray-500">Idle</span>"#.to_string())
    }
}

/// POST /api/voice/record/start
/// Spawns ffmpeg to record from the default PulseAudio source into a WAV file.
/// Returns an HTML fragment replacing the recording controls.
pub async fn record_start_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut recording = state.voice_recording.lock().await;
    if recording.is_some() {
        return Html(r#"<p class="text-yellow-400 text-sm">Already recording.</p>"#.to_string())
            .into_response();
    }

    let child = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "pulse",
            "-i",
            "default",
            "-ac",
            "1",
            "-ar",
            "16000",
            "-c:a",
            "pcm_s16le",
            RECORDING_TMP,
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    match child {
        Ok(c) => {
            *recording = Some(c);
            Html(
                r#"<p class="text-red-400 text-sm font-medium animate-pulse">● Recording… press Stop when done.</p>"#
                    .to_string(),
            )
            .into_response()
        }
        Err(e) => Html(format!(
            r##"<p class="text-red-400 text-sm">Failed to start ffmpeg: {}</p>"##,
            html_escape(&e.to_string())
        ))
        .into_response(),
    }
}

/// POST /api/voice/record/stop
/// Stops ffmpeg by writing 'q' to its stdin, waits for the process to exit,
/// then runs the Whisper transcription script and returns the result HTML.
pub async fn record_stop_handler(
    State(state): State<Arc<AppState>>,
    Form(params): Form<StopParams>,
) -> impl IntoResponse {
    let mut recording = state.voice_recording.lock().await;
    let child = match recording.take() {
        Some(c) => c,
        None => {
            return Html(
                r#"<p class="text-gray-400 text-sm">No recording in progress.</p>"#.to_string(),
            )
            .into_response();
        }
    };

    // Signal ffmpeg to stop gracefully: write 'q\n' to its stdin.
    let mut child = child;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"q\n").await;
        drop(stdin);
    }

    // Wait for ffmpeg to flush and close the WAV file.
    let _ = child.wait().await;

    // Check that the output file exists and is non-empty.
    match tokio::fs::metadata(RECORDING_TMP).await {
        Ok(m) if m.len() > 0 => {}
        _ => {
            return Html(
                r#"<p class="text-gray-400 text-sm">Recording was empty or was not saved.</p>"#
                    .to_string(),
            )
            .into_response();
        }
    }

    run_transcription(RECORDING_TMP, &params.lang, &state).await
}

/// POST /api/voice/file  (multipart/form-data, fields: "audio", "lang")
/// Saves the uploaded audio to a temp file and runs the Whisper transcription.
pub async fn file_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut audio_bytes: Option<Vec<u8>> = None;
    let mut ext = "wav".to_string();
    let mut lang = "auto".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("audio") => {
                if let Some(ct) = field.content_type() {
                    ext = match ct {
                        "audio/mpeg" | "audio/mp3" => "mp3",
                        "audio/ogg" => "ogg",
                        "audio/flac" => "flac",
                        "audio/x-m4a" | "audio/mp4" => "m4a",
                        _ => "wav",
                    }
                    .to_string();
                }
                if let Some(fname) = field.file_name() {
                    // Infer ext from filename if content-type is generic
                    let fname = fname.to_string();
                    if let Some(e) = fname.rsplit('.').next() {
                        let e = e.to_lowercase();
                        if matches!(e.as_str(), "mp3" | "ogg" | "flac" | "m4a" | "wav") {
                            ext = e;
                        }
                    }
                }
                match field.bytes().await {
                    Ok(b) => audio_bytes = Some(b.to_vec()),
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

    let bytes = match audio_bytes {
        Some(b) if !b.is_empty() => b,
        _ => {
            return Html(
                r#"<p class="text-gray-400 text-sm mt-4">No audio file received.</p>"#.to_string(),
            )
            .into_response();
        }
    };

    let tmp_path = format!("{UPLOAD_TMP_BASE}.{ext}");
    if let Err(e) = tokio::fs::write(&tmp_path, &bytes).await {
        return Html(format!(
            r##"<p class="text-red-400 text-sm mt-4">Write error: {}</p>"##,
            html_escape(&e.to_string())
        ))
        .into_response();
    }

    run_transcription(&tmp_path, &lang, &state).await
}

/// POST /api/voice/copy  (form-encoded, field: text)
/// Copies the transcript to the system clipboard via arboard.
/// Does NOT suppress clipboard history — voice transcripts are new content (D-014).
pub async fn copy_handler(
    State(_state): State<Arc<AppState>>,
    Form(body): Form<TextBody>,
) -> impl IntoResponse {
    if body.text.is_empty() {
        return Html(r#"<span class="text-gray-400">Nothing to copy.</span>"#.to_string())
            .into_response();
    }

    let text = body.text.clone();
    tokio::task::spawn_blocking(move || {
        if let Ok(mut board) = arboard::Clipboard::new() {
            let _ = board.set_text(&text);
        }
    });

    Html(r#"<span class="text-green-400">Copied to clipboard!</span>"#.to_string()).into_response()
}

/// POST /api/voice/save-note  (form-encoded, field: text)
/// Saves the transcript as a new Note in SQLite.
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
    let title: String = body
        .text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("Voice Transcript")
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
        .route("/api/voice/status", get(status_handler))
        .route("/api/voice/record/start", post(record_start_handler))
        .route("/api/voice/record/stop", post(record_stop_handler))
        .route("/api/voice/file", post(file_handler))
        .route("/api/voice/copy", post(copy_handler))
        .route("/api/voice/save-note", post(save_note_handler))
}
