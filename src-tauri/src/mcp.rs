// mcp.rs — MCP server implementation
//
// Phase 4.1: JSON API routes at /api/mcp/... consumed by the eleutheria-mcp stdio binary.
// Phase 4.2: SSE transport at /mcp (stubs kept below, full implementation in next item).

use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{get, post, put},
    Form, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt as _;

use crate::server::AppState;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn scripts_dir() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has no parent")
        .join("scripts")
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn video_out(label: &str) -> std::io::Result<String> {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = PathBuf::from(&home).join("Videos").join("Eleutheria");
    std::fs::create_dir_all(&dir)?;
    Ok(dir
        .join(format!("video-{}-{}.mp4", label, now_secs() as u64))
        .to_string_lossy()
        .into_owned())
}

fn audio_out(ext: &str) -> std::io::Result<String> {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = PathBuf::from(&home).join("Music").join("Eleutheria");
    std::fs::create_dir_all(&dir)?;
    Ok(dir
        .join(format!("audio-{}.{}", now_secs() as u64, ext))
        .to_string_lossy()
        .into_owned())
}

fn trim_stderr(raw: &[u8]) -> String {
    let s = String::from_utf8_lossy(raw);
    let lines: Vec<&str> = s.lines().collect();
    let start = lines.len().saturating_sub(10);
    lines[start..].join("\n")
}

// ── Request structs ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ClipboardQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Deserialize)]
pub struct CopyForm {
    pub content: String,
}

#[derive(Deserialize)]
pub struct NoteQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Deserialize)]
pub struct NoteCreateForm {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tags: String,
}

#[derive(Deserialize)]
pub struct NoteUpdateForm {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
}

#[derive(Deserialize)]
pub struct OcrForm {
    pub path: String,
    #[serde(default = "default_lang_eng")]
    pub lang: String,
}

#[derive(Deserialize)]
pub struct VoiceForm {
    pub path: String,
    #[serde(default = "default_lang_auto")]
    pub lang: String,
}

#[derive(Deserialize)]
pub struct TranslateForm {
    pub text: String,
    pub from_lang: String,
    pub to_lang: String,
}

#[derive(Deserialize)]
pub struct VideoForm {
    pub operation: String,
    pub input_path: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub end: String,
    #[serde(default = "default_audio_format")]
    pub audio_format: String,
    #[serde(default = "default_crf")]
    pub crf: u32,
    #[serde(default)]
    pub compress_resolution: String,
    #[serde(default)]
    pub resize_resolution: String,
}

#[derive(Deserialize)]
pub struct RembgForm {
    pub path: String,
}

fn default_limit() -> i64 {
    50
}
fn default_lang_eng() -> String {
    "eng".to_string()
}
fn default_lang_auto() -> String {
    "auto".to_string()
}
fn default_audio_format() -> String {
    "mp3".to_string()
}
fn default_crf() -> u32 {
    28
}

// ── Clipboard handlers ────────────────────────────────────────────────────────

/// GET /api/mcp/clipboard  → { "items": [{ "id", "content", "created_at" }] }
async fn clipboard_list_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ClipboardQuery>,
) -> impl IntoResponse {
    let rows: Vec<(String, String, i64)> = if params.q.is_empty() {
        sqlx::query_as(
            "SELECT id, content, created_at FROM clipboard
             ORDER BY created_at DESC LIMIT ?",
        )
        .bind(params.limit)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    } else {
        let pattern = format!("%{}%", params.q);
        sqlx::query_as(
            "SELECT id, content, created_at FROM clipboard
             WHERE content LIKE ?
             ORDER BY created_at DESC LIMIT ?",
        )
        .bind(&pattern)
        .bind(params.limit)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    };

    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, content, ts)| json!({ "id": id, "content": content, "created_at": ts }))
        .collect();

    Json(json!({ "items": items }))
}

/// POST /api/mcp/clipboard/copy  (form: content)
/// Writes text to the system clipboard. The monitor will record it in DB.
async fn clipboard_copy_handler(
    State(_state): State<Arc<AppState>>,
    Form(form): Form<CopyForm>,
) -> impl IntoResponse {
    let content = form.content.clone();
    // arboard is not Send; must run in spawn_blocking
    let ok = tokio::task::spawn_blocking(move || {
        arboard::Clipboard::new()
            .and_then(|mut cb| cb.set_text(&content))
            .is_ok()
    })
    .await
    .unwrap_or(false);

    Json(json!({ "ok": ok }))
}

// ── Notes handlers ────────────────────────────────────────────────────────────

/// GET /api/mcp/notes  → { "notes": [{ "id", "title", "content", "tags", "pinned", "updated_at" }] }
async fn notes_list_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NoteQuery>,
) -> impl IntoResponse {
    let rows: Vec<(String, String, String, String, i64, i64)> = if params.q.is_empty() {
        sqlx::query_as(
            "SELECT id, title, content, tags, pinned, updated_at FROM notes
             ORDER BY pinned DESC, updated_at DESC LIMIT ?",
        )
        .bind(params.limit)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    } else {
        // FTS5 MATCH search — joins notes_fts rowid to notes rowid
        sqlx::query_as(
            "SELECT n.id, n.title, n.content, n.tags, n.pinned, n.updated_at
             FROM notes n
             JOIN notes_fts f ON f.rowid = n.rowid
             WHERE notes_fts MATCH ?
             ORDER BY n.pinned DESC, rank
             LIMIT ?",
        )
        .bind(&params.q)
        .bind(params.limit)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    };

    let notes: Vec<Value> = rows
        .into_iter()
        .map(|(id, title, content, tags, pinned, updated_at)| {
            json!({
                "id": id, "title": title, "content": content,
                "tags": tags, "pinned": pinned == 1, "updated_at": updated_at
            })
        })
        .collect();

    Json(json!({ "notes": notes }))
}

/// POST /api/mcp/notes  (form: title, content, tags) → { "ok", "id", "title", ... }
async fn notes_create_handler(
    State(state): State<Arc<AppState>>,
    Form(form): Form<NoteCreateForm>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_secs();

    // Convert "tag1,tag2" to JSON array, or use "[]" if empty
    let tags = if form.tags.is_empty() {
        "[]".to_string()
    } else {
        let arr: Vec<Value> = form
            .tags
            .split(',')
            .map(|t| Value::String(t.trim().to_string()))
            .collect();
        serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
    };

    let result = sqlx::query(
        "INSERT INTO notes (id, title, content, content_fts, tags, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&form.title)
    .bind(&form.content)
    .bind(&form.content) // content_fts mirrors content (FTS5 triggers sync notes_fts)
    .bind(&tags)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Json(json!({
            "ok": true, "id": id, "title": form.title,
            "content": form.content, "tags": tags, "created_at": now
        })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// PUT /api/mcp/notes/:id  (form: title?, content?, tags?) → { "ok": true }
async fn notes_update_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Form(form): Form<NoteUpdateForm>,
) -> impl IntoResponse {
    let now = now_secs();

    // Dynamic SET clause — same pattern as notes::update_handler
    let mut set_parts = vec!["updated_at = ?"];
    let mut title_val: Option<String> = None;
    let mut content_val: Option<String> = None;
    let mut tags_val: Option<String> = None;

    if form.title.is_some() {
        set_parts.push("title = ?");
        title_val = form.title;
    }
    if form.content.is_some() {
        set_parts.push("content = ?");
        set_parts.push("content_fts = ?");
        content_val = form.content;
    }
    if let Some(ref raw_tags) = form.tags {
        set_parts.push("tags = ?");
        let arr: Vec<Value> = raw_tags
            .split(',')
            .map(|t| Value::String(t.trim().to_string()))
            .collect();
        tags_val = Some(serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string()));
    }

    if set_parts.len() == 1 {
        // Only updated_at — nothing meaningful to update
        return Json(json!({ "ok": true, "message": "nothing to update" }));
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
    q = q.bind(&id);

    match q.execute(&state.db).await {
        Ok(_) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// DELETE /api/mcp/notes/:id  → { "ok": true }
async fn notes_delete_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();
    Json(json!({ "ok": true }))
}

// ── OCR handler ───────────────────────────────────────────────────────────────

/// POST /api/mcp/ocr/file  (form: path, lang?) → { "ok", "text" }
async fn ocr_file_handler(
    State(_state): State<Arc<AppState>>,
    Form(form): Form<OcrForm>,
) -> impl IntoResponse {
    let path = form.path.trim().to_string();
    if !std::path::Path::new(&path).is_file() {
        return Json(json!({ "ok": false, "error": format!("File not found: {path}") }));
    }
    let lang = form.lang.clone();
    let out = tokio::process::Command::new("tesseract")
        .arg(&path)
        .arg("stdout")
        .arg("-l")
        .arg(&lang)
        .output()
        .await;

    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
            Json(json!({ "ok": true, "text": text }))
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            Json(json!({ "ok": false, "error": err }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ── Voice handler ─────────────────────────────────────────────────────────────

/// POST /api/mcp/voice/transcribe  (form: path, lang?) → { "ok", "text" }
async fn voice_transcribe_handler(
    State(_state): State<Arc<AppState>>,
    Form(form): Form<VoiceForm>,
) -> impl IntoResponse {
    let path = form.path.trim().to_string();
    if !std::path::Path::new(&path).is_file() {
        return Json(json!({ "ok": false, "error": format!("File not found: {path}") }));
    }
    let script = scripts_dir().join("transcribe.py");
    let lang = form.lang.clone();
    let mut cmd = tokio::process::Command::new("python3");
    cmd.arg(&script).arg(&path);
    if !lang.is_empty() && lang != "auto" {
        cmd.arg("--lang").arg(&lang);
    }
    match cmd.output().await {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
            Json(json!({ "ok": true, "text": text }))
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            Json(json!({ "ok": false, "error": err }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ── Translate handler ─────────────────────────────────────────────────────────

/// POST /api/mcp/translate  (form: text, from_lang, to_lang) → { "ok", "text" }
async fn translate_handler(
    State(_state): State<Arc<AppState>>,
    Form(form): Form<TranslateForm>,
) -> impl IntoResponse {
    let script = scripts_dir().join("translate.py");
    let text = form.text.clone();
    let from = form.from_lang.clone();
    let to = form.to_lang.clone();

    let result = tokio::spawn(async move {
        tokio::process::Command::new("python3")
            .arg(&script)
            .arg(&text)
            .arg(&from)
            .arg(&to)
            .output()
            .await
    })
    .await;

    match result {
        Ok(Ok(o)) if o.status.success() => {
            let translated = String::from_utf8_lossy(&o.stdout).trim().to_string();
            Json(json!({ "ok": true, "text": translated }))
        }
        Ok(Ok(o)) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            Json(json!({ "ok": false, "error": err }))
        }
        Ok(Err(e)) => Json(json!({ "ok": false, "error": e.to_string() })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ── Video handler ─────────────────────────────────────────────────────────────

/// POST /api/mcp/video/process  (form: operation, input_path, ...) → { "ok", "output" }
async fn video_process_handler(
    State(_state): State<Arc<AppState>>,
    Form(form): Form<VideoForm>,
) -> impl IntoResponse {
    let input = form.input_path.trim().to_string();
    if !std::path::Path::new(&input).is_file() {
        return Json(json!({ "ok": false, "error": format!("File not found: {input}") }));
    }

    // Build (output_path, ffmpeg_args) per operation
    let (output_path, args): (String, Vec<String>) = match form.operation.as_str() {
        "trim" => {
            let start = form.start.trim().to_string();
            let end = form.end.trim().to_string();
            if start.is_empty() || end.is_empty() {
                return Json(
                    json!({ "ok": false, "error": "start and end are required for trim" }),
                );
            }
            let out = match video_out("trim") {
                Ok(p) => p,
                Err(e) => return Json(json!({ "ok": false, "error": e.to_string() })),
            };
            let a = vec![
                "-y".into(),
                "-i".into(),
                input,
                "-ss".into(),
                start,
                "-to".into(),
                end,
                "-c".into(),
                "copy".into(),
                out.clone(),
            ];
            (out, a)
        }

        "extract_audio" => {
            let (codec, ext) = match form.audio_format.as_str() {
                "wav" => ("pcm_s16le", "wav"),
                "flac" => ("flac", "flac"),
                _ => ("libmp3lame", "mp3"),
            };
            let out = match audio_out(ext) {
                Ok(p) => p,
                Err(e) => return Json(json!({ "ok": false, "error": e.to_string() })),
            };
            let a = vec![
                "-y".into(),
                "-i".into(),
                input,
                "-vn".into(),
                "-c:a".into(),
                codec.into(),
                out.clone(),
            ];
            (out, a)
        }

        "compress" => {
            let crf = form.crf.clamp(18, 40).to_string();
            let out = match video_out("compressed") {
                Ok(p) => p,
                Err(e) => return Json(json!({ "ok": false, "error": e.to_string() })),
            };
            let mut a: Vec<String> = vec!["-y".into(), "-i".into(), input];
            if !form.compress_resolution.is_empty() && form.compress_resolution != "original" {
                a.extend([
                    "-vf".into(),
                    format!("scale=-2:{}", form.compress_resolution),
                ]);
            }
            a.extend([
                "-c:v".into(),
                "libx264".into(),
                "-crf".into(),
                crf,
                "-preset".into(),
                "fast".into(),
                out.clone(),
            ]);
            (out, a)
        }

        "resize" => {
            let height = if form.resize_resolution.is_empty() {
                "720".to_string()
            } else {
                form.resize_resolution.clone()
            };
            let crf = form.crf.clamp(18, 40).to_string();
            let out = match video_out("resized") {
                Ok(p) => p,
                Err(e) => return Json(json!({ "ok": false, "error": e.to_string() })),
            };
            let a = vec![
                "-y".into(),
                "-i".into(),
                input,
                "-vf".into(),
                format!("scale=-2:{height}"),
                "-c:v".into(),
                "libx264".into(),
                "-crf".into(),
                crf,
                "-preset".into(),
                "fast".into(),
                out.clone(),
            ];
            (out, a)
        }

        op => return Json(json!({ "ok": false, "error": format!("Unknown operation: {op}") })),
    };

    let cmd_result = tokio::process::Command::new("ffmpeg")
        .args(&args)
        .output()
        .await;

    match cmd_result {
        Ok(o) if o.status.success() => Json(json!({ "ok": true, "output": output_path })),
        Ok(o) => Json(json!({ "ok": false, "error": trim_stderr(&o.stderr) })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ── Photo / rembg handler ─────────────────────────────────────────────────────

/// POST /api/mcp/photo/rembg  (form: path) → { "ok", "output" }
/// Accepts a file path, runs rembg_remove.py, saves PNG to ~/Pictures/Eleutheria/.
async fn photo_rembg_handler(
    State(_state): State<Arc<AppState>>,
    Form(form): Form<RembgForm>,
) -> impl IntoResponse {
    let path = form.path.trim().to_string();
    if !std::path::Path::new(&path).is_file() {
        return Json(json!({ "ok": false, "error": format!("File not found: {path}") }));
    }

    let script = scripts_dir().join("rembg_remove.py");
    let result = tokio::process::Command::new("python3")
        .arg(&script)
        .arg(&path)
        .output()
        .await;

    match result {
        Ok(o) if o.status.success() => {
            let b64 = String::from_utf8_lossy(&o.stdout).trim().to_string();
            use base64::Engine;
            match base64::engine::general_purpose::STANDARD.decode(&b64) {
                Ok(bytes) => {
                    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    let out_dir = PathBuf::from(&home).join("Pictures").join("Eleutheria");
                    let _ = std::fs::create_dir_all(&out_dir);
                    let output = out_dir
                        .join(format!("rembg-{}.png", now_secs() as u64))
                        .to_string_lossy()
                        .into_owned();
                    match tokio::fs::write(&output, &bytes).await {
                        Ok(_) => Json(json!({ "ok": true, "output": output })),
                        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
                    }
                }
                Err(e) => {
                    Json(json!({ "ok": false, "error": format!("base64 decode error: {e}") }))
                }
            }
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            Json(json!({ "ok": false, "error": err }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

// ── MCP tool manifest (shared between SSE and stdio transports) ───────────────

fn mcp_tools() -> Value {
    json!([
        {
            "name": "clipboard_list",
            "description": "List recent clipboard history. Optionally filter by search query.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "search": { "type": "string", "description": "Filter by content (optional)" },
                    "limit": { "type": "integer", "description": "Max results (default 50)" }
                }
            }
        },
        {
            "name": "clipboard_copy",
            "description": "Add text to the system clipboard and save to clipboard history.",
            "inputSchema": {
                "type": "object",
                "required": ["content"],
                "properties": {
                    "content": { "type": "string", "description": "Text to copy to clipboard" }
                }
            }
        },
        {
            "name": "note_create",
            "description": "Create a new note.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "content": { "type": "string" },
                    "tags": { "type": "string", "description": "Comma-separated tags" }
                }
            }
        },
        {
            "name": "note_list",
            "description": "List notes. Optionally filter by full-text search query.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "search": { "type": "string", "description": "Full-text search query (FTS5)" },
                    "limit": { "type": "integer", "description": "Max results (default 50)" }
                }
            }
        },
        {
            "name": "note_update",
            "description": "Update a note by ID. Only provided fields are updated.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "string", "description": "Note UUID" },
                    "title": { "type": "string" },
                    "content": { "type": "string" },
                    "tags": { "type": "string", "description": "Comma-separated tags" }
                }
            }
        },
        {
            "name": "note_delete",
            "description": "Delete a note by ID.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "string", "description": "Note UUID" }
                }
            }
        },
        {
            "name": "ocr_file",
            "description": "Extract text from an image file using OCR (Tesseract). Supports PNG, JPG, TIFF.",
            "inputSchema": {
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to the image file" },
                    "lang": { "type": "string", "description": "Tesseract language code: eng (default) or spa" }
                }
            }
        },
        {
            "name": "voice_transcribe",
            "description": "Transcribe an audio file to text using Whisper (offline AI).",
            "inputSchema": {
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to the audio file (wav, mp3, ogg, flac, m4a)" },
                    "lang": { "type": "string", "description": "Language code (e.g. en, es) or 'auto' for auto-detect (default)" }
                }
            }
        },
        {
            "name": "translate_text",
            "description": "Translate text between languages using the offline translation engine.",
            "inputSchema": {
                "type": "object",
                "required": ["text", "from_lang", "to_lang"],
                "properties": {
                    "text": { "type": "string" },
                    "from_lang": { "type": "string", "description": "Source language code: en, es, fr, de, pt" },
                    "to_lang": { "type": "string", "description": "Target language code: en, es, fr, de, pt" }
                }
            }
        },
        {
            "name": "video_process",
            "description": "Process a video file: trim, extract audio, compress, or resize.",
            "inputSchema": {
                "type": "object",
                "required": ["operation", "input_path"],
                "properties": {
                    "operation": { "type": "string", "enum": ["trim", "extract_audio", "compress", "resize"] },
                    "input_path": { "type": "string", "description": "Absolute path to the input video file" },
                    "start": { "type": "string", "description": "Trim start time (HH:MM:SS)" },
                    "end": { "type": "string", "description": "Trim end time (HH:MM:SS)" },
                    "audio_format": { "type": "string", "enum": ["mp3", "wav", "flac"] },
                    "crf": { "type": "integer", "description": "CRF quality 18–40 (default 28)" },
                    "compress_resolution": { "type": "string", "enum": ["original", "1080", "720", "480"] },
                    "resize_resolution": { "type": "string", "enum": ["1080", "720", "480", "360"] }
                }
            }
        },
        {
            "name": "photo_rembg",
            "description": "Remove the background from an image using AI (rembg, offline). Saves result as PNG to ~/Pictures/Eleutheria/.",
            "inputSchema": {
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to the image file (PNG, JPG, WEBP)" }
                }
            }
        }
    ])
}

// ── SSE HTTP client (mirrors McpClient in the stdio binary) ──────────────────

struct SseHttpClient {
    base: String,
    auth: String,
    http: reqwest::Client,
}

impl SseHttpClient {
    fn from_state(state: &AppState) -> Self {
        Self {
            base: format!("http://127.0.0.1:{}", state.port),
            auth: format!("Bearer {}", state.session_token),
            http: reqwest::Client::new(),
        }
    }

    async fn get_query(&self, path: &str, params: &[(&str, &str)]) -> Result<Value, String> {
        self.http
            .get(format!("{}{}", self.base, path))
            .header("authorization", &self.auth)
            .query(params)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())
    }

    async fn post_form(&self, path: &str, form: &[(&str, &str)]) -> Result<Value, String> {
        self.http
            .post(format!("{}{}", self.base, path))
            .header("authorization", &self.auth)
            .form(form)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())
    }

    async fn put_form(&self, path: &str, form: &[(&str, &str)]) -> Result<Value, String> {
        self.http
            .put(format!("{}{}", self.base, path))
            .header("authorization", &self.auth)
            .form(form)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())
    }

    async fn delete(&self, path: &str) -> Result<Value, String> {
        self.http
            .delete(format!("{}{}", self.base, path))
            .header("authorization", &self.auth)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())
    }
}

// ── Tool dispatch (same logic as call_tool in the stdio binary) ───────────────

async fn call_tool_sse(client: &SseHttpClient, name: &str, args: &Value) -> Result<Value, String> {
    let empty = json!({});
    let a = if args.is_null() || !args.is_object() {
        &empty
    } else {
        args
    };

    match name {
        "clipboard_list" => {
            let search = a.get("search").and_then(|v| v.as_str()).unwrap_or("");
            let limit = a
                .get("limit")
                .and_then(|v| v.as_i64())
                .unwrap_or(50)
                .to_string();
            if search.is_empty() {
                client
                    .get_query("/api/mcp/clipboard", &[("limit", &limit)])
                    .await
            } else {
                client
                    .get_query("/api/mcp/clipboard", &[("q", search), ("limit", &limit)])
                    .await
            }
        }

        "clipboard_copy" => {
            let content = a.get("content").and_then(|v| v.as_str()).unwrap_or("");
            client
                .post_form("/api/mcp/clipboard/copy", &[("content", content)])
                .await
        }

        "note_create" => {
            let title = a.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let content = a.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let tags = a.get("tags").and_then(|v| v.as_str()).unwrap_or("");
            client
                .post_form(
                    "/api/mcp/notes",
                    &[("title", title), ("content", content), ("tags", tags)],
                )
                .await
        }

        "note_list" => {
            let search = a.get("search").and_then(|v| v.as_str()).unwrap_or("");
            let limit = a
                .get("limit")
                .and_then(|v| v.as_i64())
                .unwrap_or(50)
                .to_string();
            if search.is_empty() {
                client
                    .get_query("/api/mcp/notes", &[("limit", &limit)])
                    .await
            } else {
                client
                    .get_query("/api/mcp/notes", &[("q", search), ("limit", &limit)])
                    .await
            }
        }

        "note_update" => {
            let id = a
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "note_update requires 'id'".to_string())?;
            let mut fields: Vec<(String, String)> = vec![];
            if let Some(t) = a.get("title").and_then(|v| v.as_str()) {
                fields.push(("title".into(), t.into()));
            }
            if let Some(c) = a.get("content").and_then(|v| v.as_str()) {
                fields.push(("content".into(), c.into()));
            }
            if let Some(t) = a.get("tags").and_then(|v| v.as_str()) {
                fields.push(("tags".into(), t.into()));
            }
            let refs: Vec<(&str, &str)> = fields
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            client
                .put_form(&format!("/api/mcp/notes/{id}"), &refs)
                .await
        }

        "note_delete" => {
            let id = a
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "note_delete requires 'id'".to_string())?;
            client.delete(&format!("/api/mcp/notes/{id}")).await
        }

        "ocr_file" => {
            let path = a.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let lang = a.get("lang").and_then(|v| v.as_str()).unwrap_or("eng");
            client
                .post_form("/api/mcp/ocr/file", &[("path", path), ("lang", lang)])
                .await
        }

        "voice_transcribe" => {
            let path = a.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let lang = a.get("lang").and_then(|v| v.as_str()).unwrap_or("auto");
            client
                .post_form(
                    "/api/mcp/voice/transcribe",
                    &[("path", path), ("lang", lang)],
                )
                .await
        }

        "translate_text" => {
            let text = a.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let from = a.get("from_lang").and_then(|v| v.as_str()).unwrap_or("en");
            let to = a.get("to_lang").and_then(|v| v.as_str()).unwrap_or("es");
            client
                .post_form(
                    "/api/mcp/translate",
                    &[("text", text), ("from_lang", from), ("to_lang", to)],
                )
                .await
        }

        "video_process" => {
            let op = a.get("operation").and_then(|v| v.as_str()).unwrap_or("");
            let input = a.get("input_path").and_then(|v| v.as_str()).unwrap_or("");
            let mut fields: Vec<(String, String)> = vec![
                ("operation".into(), op.into()),
                ("input_path".into(), input.into()),
            ];
            for key in &[
                "start",
                "end",
                "audio_format",
                "compress_resolution",
                "resize_resolution",
            ] {
                if let Some(v) = a.get(*key).and_then(|v| v.as_str()) {
                    fields.push(((*key).into(), v.into()));
                }
            }
            if let Some(v) = a.get("crf").and_then(|v| v.as_i64()) {
                fields.push(("crf".into(), v.to_string()));
            }
            let refs: Vec<(&str, &str)> = fields
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            client.post_form("/api/mcp/video/process", &refs).await
        }

        "photo_rembg" => {
            let path = a.get("path").and_then(|v| v.as_str()).unwrap_or("");
            client
                .post_form("/api/mcp/photo/rembg", &[("path", path)])
                .await
        }

        unknown => Err(format!("Unknown tool: {unknown}")),
    }
}

// ── JSON-RPC message processor ────────────────────────────────────────────────

async fn process_sse_message(state: &Arc<AppState>, body: Value) -> Option<Value> {
    let method = body.get("method").and_then(|v| v.as_str())?;
    let id = body.get("id").cloned().unwrap_or(Value::Null);

    match method {
        // Notifications — no JSON-RPC response.
        "initialized" | "notifications/initialized" => None,

        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "eleutheria-telos",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })),

        "ping" => Some(json!({ "jsonrpc": "2.0", "id": id, "result": {} })),

        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "tools": mcp_tools() }
        })),

        "tools/call" => {
            let params = body.get("params")?;
            let name = params.get("name").and_then(|v| v.as_str())?;
            let args = params.get("arguments").cloned().unwrap_or(Value::Null);
            let client = SseHttpClient::from_state(state);
            match call_tool_sse(&client, name, &args).await {
                Ok(result) => {
                    let text = serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| result.to_string());
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": { "content": [{ "type": "text", "text": text }] }
                    }))
                }
                Err(e) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32603, "message": format!("Tool error: {e}") }
                })),
            }
        }

        other => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Method not found: {other}") }
        })),
    }
}

// ── SSE transport handlers ────────────────────────────────────────────────────

/// `GET /mcp` — open an SSE stream.
///
/// Immediately sends an `endpoint` event with the URL for the client to POST
/// JSON-RPC messages. Responses are pushed back over this stream as `message`
/// events.
pub async fn mcp_sse_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let session_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel::<String>(64);

    // Pre-fill the endpoint event so the client receives it as the first SSE
    // item. The channel buffer (64) is large enough that try_send never blocks.
    let _ = tx.try_send(format!("E:/mcp?sessionId={}", session_id));

    {
        let mut sessions = state.mcp_sessions.lock().await;
        sessions.insert(session_id, tx);
    }

    let stream = ReceiverStream::new(rx).map(|raw| {
        let event = if let Some(url) = raw.strip_prefix("E:") {
            Event::default().event("endpoint").data(url)
        } else {
            Event::default().event("message").data(raw)
        };
        Ok::<_, Infallible>(event)
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// `POST /mcp?sessionId={id}` — send a JSON-RPC message to an open SSE session.
///
/// Returns `202 Accepted` immediately; the JSON-RPC response is pushed
/// asynchronously over the corresponding SSE stream.
pub async fn mcp_post_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let session_id = match params.get("sessionId") {
        Some(id) => id.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "missing sessionId query parameter" })),
            )
                .into_response()
        }
    };

    let tx = {
        let sessions = state.mcp_sessions.lock().await;
        sessions.get(&session_id).cloned()
    };

    let Some(tx) = tx else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "session not found or expired" })),
        )
            .into_response();
    };

    // Process in a background task so the POST returns 202 immediately.
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Some(resp) = process_sse_message(&state_clone, body).await {
            let json = serde_json::to_string(&resp).unwrap_or_default();
            let _ = tx.send(json).await;
        }
    });

    (StatusCode::ACCEPTED, Json(json!({ "ok": true }))).into_response()
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Clipboard
        .route("/api/mcp/clipboard", get(clipboard_list_handler))
        .route("/api/mcp/clipboard/copy", post(clipboard_copy_handler))
        // Notes
        .route(
            "/api/mcp/notes",
            get(notes_list_handler).post(notes_create_handler),
        )
        .route(
            "/api/mcp/notes/:id",
            put(notes_update_handler).delete(notes_delete_handler),
        )
        // OCR
        .route("/api/mcp/ocr/file", post(ocr_file_handler))
        // Voice
        .route("/api/mcp/voice/transcribe", post(voice_transcribe_handler))
        // Translate
        .route("/api/mcp/translate", post(translate_handler))
        // Video
        .route("/api/mcp/video/process", post(video_process_handler))
        // Photo
        .route("/api/mcp/photo/rembg", post(photo_rembg_handler))
}
