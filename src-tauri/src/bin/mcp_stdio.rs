//! eleutheria-mcp — MCP stdio transport server
//!
//! Implements the Model Context Protocol (JSON-RPC 2.0 over stdio) and proxies
//! all tool calls to the running Eleutheria Telos app via its local HTTP API.
//!
//! Configure in Claude Desktop / Cursor / any MCP client:
//!   {
//!     "mcpServers": {
//!       "eleutheria": { "command": "/path/to/eleutheria-mcp" }
//!     }
//!   }
//!
//! Requires Eleutheria Telos to be running. On startup the app writes
//! ~/.local/share/eleutheria-telos/server.json with port and session token.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

// ── Server discovery ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ServerInfo {
    port: u16,
    token: String,
}

fn load_server_info() -> Option<ServerInfo> {
    let base = std::env::var("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            std::path::PathBuf::from(home).join(".local/share")
        });
    let path = base.join("eleutheria-telos/server.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

// ── JSON-RPC types ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn ok_response(id: Value, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

fn err_response(id: Value, code: i64, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(json!({ "code": code, "message": message })),
    }
}

// ── MCP tool manifest ─────────────────────────────────────────────────────────

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
                    "lang": {
                        "type": "string",
                        "description": "Tesseract language code: eng (default) or spa"
                    }
                }
            }
        },
        {
            "name": "voice_transcribe",
            "description": "Transcribe an audio file to text using Whisper (offline AI). Requires a Whisper model to be downloaded in the Models panel.",
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
            "description": "Translate text between languages using the offline translation engine. Requires a language pack to be downloaded in the Models panel.",
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
            "description": "Process a video file: trim to a time range, extract audio, compress (re-encode with libx264), or resize to a target height.",
            "inputSchema": {
                "type": "object",
                "required": ["operation", "input_path"],
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["trim", "extract_audio", "compress", "resize"]
                    },
                    "input_path": { "type": "string", "description": "Absolute path to the input video file" },
                    "start": { "type": "string", "description": "Trim start time (HH:MM:SS) — required for trim" },
                    "end": { "type": "string", "description": "Trim end time (HH:MM:SS) — required for trim" },
                    "audio_format": {
                        "type": "string",
                        "enum": ["mp3", "wav", "flac"],
                        "description": "Output audio format for extract_audio (default: mp3)"
                    },
                    "crf": {
                        "type": "integer",
                        "description": "CRF quality 18–40 for compress/resize (lower = better quality, default 28)"
                    },
                    "compress_resolution": {
                        "type": "string",
                        "enum": ["original", "1080", "720", "480"],
                        "description": "Optional downscale for compress (default: original)"
                    },
                    "resize_resolution": {
                        "type": "string",
                        "enum": ["1080", "720", "480", "360"],
                        "description": "Target height in pixels for resize (default: 720)"
                    }
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

// ── HTTP client ───────────────────────────────────────────────────────────────

struct McpClient {
    base_url: String,
    token: String,
    client: reqwest::Client,
}

impl McpClient {
    fn new(port: u16, token: String) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            token,
            client: reqwest::Client::new(),
        }
    }

    async fn get_query(&self, path: &str, params: &[(&str, &str)]) -> Result<Value, String> {
        self.client
            .get(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .query(params)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())
    }

    async fn post_form(&self, path: &str, form: &[(&str, &str)]) -> Result<Value, String> {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .form(form)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())
    }

    async fn put_form(&self, path: &str, form: &[(&str, &str)]) -> Result<Value, String> {
        self.client
            .put(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .form(form)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())
    }

    async fn delete(&self, path: &str) -> Result<Value, String> {
        self.client
            .delete(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())
    }
}

// ── Tool dispatch ─────────────────────────────────────────────────────────────

async fn call_tool(client: &McpClient, name: &str, args: &Value) -> Result<Value, String> {
    let empty = json!({});
    let a = if args.is_null() || !args.is_object() {
        &empty
    } else {
        args
    };

    match name {
        // ── Clipboard ─────────────────────────────────────────────────────────
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

        // ── Notes ─────────────────────────────────────────────────────────────
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

        // ── OCR ───────────────────────────────────────────────────────────────
        "ocr_file" => {
            let path = a.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let lang = a.get("lang").and_then(|v| v.as_str()).unwrap_or("eng");
            client
                .post_form("/api/mcp/ocr/file", &[("path", path), ("lang", lang)])
                .await
        }

        // ── Voice ─────────────────────────────────────────────────────────────
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

        // ── Translate ─────────────────────────────────────────────────────────
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

        // ── Video ─────────────────────────────────────────────────────────────
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

        // ── Photo ─────────────────────────────────────────────────────────────
        "photo_rembg" => {
            let path = a.get("path").and_then(|v| v.as_str()).unwrap_or("");
            client
                .post_form("/api/mcp/photo/rembg", &[("path", path)])
                .await
        }

        unknown => Err(format!("Unknown tool: {unknown}")),
    }
}

// ── Message handler ───────────────────────────────────────────────────────────

async fn handle_message(client: &McpClient, line: &str) -> Option<JsonRpcResponse> {
    let req: JsonRpcRequest = serde_json::from_str(line).ok()?;
    let id = req.id.clone().unwrap_or(Value::Null);

    let resp = match req.method.as_str() {
        "initialize" => ok_response(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "eleutheria-telos",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        ),

        // Notification — no response
        "initialized" | "notifications/initialized" => return None,

        "tools/list" => ok_response(id, json!({ "tools": mcp_tools() })),

        "tools/call" => {
            let params = req.params.as_ref()?;
            let name = params.get("name").and_then(|v| v.as_str())?;
            let args = params.get("arguments").unwrap_or(&Value::Null);
            match call_tool(client, name, args).await {
                Ok(result) => {
                    let text = serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| result.to_string());
                    ok_response(id, json!({ "content": [{ "type": "text", "text": text }] }))
                }
                Err(e) => err_response(id, -32603, &format!("Tool error: {e}")),
            }
        }

        "ping" => ok_response(id, json!({})),

        other => err_response(id, -32601, &format!("Method not found: {other}")),
    };

    Some(resp)
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let server = match load_server_info() {
        Some(s) => s,
        None => {
            eprintln!(
                "eleutheria-mcp: Could not read server info. \
                 Is Eleutheria Telos running? \
                 Expected: ~/.local/share/eleutheria-telos/server.json"
            );
            std::process::exit(1);
        }
    };

    let client = McpClient::new(server.port, server.token);

    let stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = BufWriter::new(tokio::io::stdout());
    let mut lines = stdin.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(resp) = handle_message(&client, &line).await {
            let json = serde_json::to_string(&resp).unwrap_or_default();
            let _ = stdout.write_all(json.as_bytes()).await;
            let _ = stdout.write_all(b"\n").await;
            let _ = stdout.flush().await;
        }
    }
}
