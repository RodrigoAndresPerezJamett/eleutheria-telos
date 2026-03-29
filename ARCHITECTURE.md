# Eleutheria Telos — Architecture

**A cross-platform Swiss Army knife for everyday computing tasks.**
Lightweight, offline-first, extensible by the community, and usable as MCP infrastructure for AI agents.

---

## Stack

| Layer | Technology | Why |
|---|---|---|
| Desktop shell | Tauri 2.x | Native binary, cross-platform, ~15MB — no JVM, no Chromium |
| Mobile | Tauri 2.x (Android) | Same codebase, 4 tools exposed on mobile |
| Frontend | HTMX + Alpine.js + Tailwind CSS | MPA pattern, no SPA, minimal JS |
| Backend (local) | Rust + Axum | Internal HTTP server on localhost, fast, safe |
| Database | SQLite via sqlx | Local-only, FTS5 for search, no server |
| Transcription | Whisper.cpp (whisper-rs crate) | Offline, multiple model sizes |
| OCR | Tesseract (leptess crate) | Offline, multi-language |
| Translation | Argos Translate (Python subprocess) | Offline with online fallback |
| Background removal | rembg (Python subprocess) | Offline, U2Net model |
| Screen/audio recording | ffmpeg (binary subprocess) | Universal, LGPL-safe usage |
| Photo editing | Fabric.js (in WebView) | Canvas with layers, JS-extensible |
| MCP server | Axum SSE + stdio | Exposes tools to AI agents |

---

## Project Structure

```
eleutheria-telos/
│
├── ARCHITECTURE.md         ← This file
├── PRINCIPLES.md           ← Non-negotiable rules
├── ROADMAP.md              ← Current phase and upcoming phases
├── CLAUDE.md               ← Instructions for Claude Code
│
├── src-tauri/              ← Rust backend (OS access, HTTP server, tools)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         ← Entry point: init Tauri + Axum + plugins
│       ├── server.rs       ← Axum router, port detection, auth token
│       ├── db.rs           ← SQLite connection pool, migrations
│       ├── plugin_loader.rs← Reads manifests, spawns plugin processes
│       ├── event_bus.rs    ← Internal pub/sub between tools
│       ├── mcp.rs          ← MCP server (stdio + SSE)
│       ├── i18n.rs         ← String externalization loader
│       └── tools/
│           ├── mod.rs
│           ├── clipboard.rs    ← Clipboard monitoring + history
│           ├── notes.rs        ← CRUD notes with SQLite FTS5
│           ├── ocr.rs          ← Tesseract integration
│           ├── transcriber.rs  ← Whisper.cpp integration
│           ├── translator.rs   ← Argos offline + online fallback
│           ├── recorder.rs     ← Screen + audio recording via ffmpeg
│           ├── photo_editor.rs ← Fabric.js bridge + rembg for bg removal
│           ├── video_processor.rs ← ffmpeg operations (cut, join, extract)
│           └── search.rs       ← Full-text search across notes + clipboard
│
├── ui/                     ← Frontend (HTML, HTMX, Alpine, Tailwind)
│   ├── shell.html          ← App shell: sidebar (desktop) / bottom nav (mobile)
│   ├── assets/
│   │   ├── tailwind.min.css
│   │   ├── htmx.min.js
│   │   ├── alpine.min.js
│   │   └── fabric.min.js
│   ├── locales/
│   │   └── en.json         ← English strings (default)
│   └── tools/
│       ├── clipboard/
│       │   ├── index.html      ← Full panel
│       │   └── partials/
│       │       ├── list.html   ← HTMX fragment: clipboard list
│       │       └── item.html   ← HTMX fragment: single item
│       ├── notes/
│       ├── ocr/
│       ├── voice/
│       ├── translate/
│       ├── photo-editor/
│       ├── video-processor/
│       ├── search/
│       └── settings/
│
├── plugins/                ← Community plugins (user-installed)
│   └── example-plugin/     ← Reference implementation
│       ├── manifest.json
│       ├── plugin.py
│       └── ui/
│           ├── index.html
│           └── partials/
│
├── models/                 ← AI models downloaded on-demand (gitignored)
│   ├── whisper-tiny.bin    ← ~75MB
│   ├── whisper-base.bin    ← ~142MB
│   └── argos-*.pkg         ← ~100MB per language pair
│
└── eleutheria.db           ← User database (gitignored)
```

---

## Architecture Layers

### Layer 1 — Platform (Tauri 2.x)

Tauri wraps the app in a native binary for each OS. It provides:
- Native OS window management
- System tray integration (the app lives in the tray, never fully quits)
- Global hotkey registration (configurable by the user)
- File system access
- Shell command execution (for ffmpeg, Python subprocesses)
- Secure WebView (uses OS-native renderer: WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux)

**Android (phone):** Tauri 2.x supports Android officially. On phone layout (<640px), only 4 tools are surfaced: Voice-to-Text, OCR (camera as input), Notes, Clipboard. The rest are hidden via responsive layout rules. Background clipboard monitoring runs as an Android Foreground Service.

**Android (tablet):** Same 4 tools as phone, but uses the icon-only sidebar layout (640px–1023px breakpoint). If screen is large enough (≥1024px), full sidebar may be shown.

**iPad:** Future scope. Tauri iOS support is in beta. Architecture does not block it — the tablet layout (icon-only sidebar) maps naturally to iPad. No timeline committed.

**iPhone:** Future scope. Same as iPad. The mobile layout (<640px) already works for iPhone screen sizes once Tauri iOS is stable.

---

### Layer 2 — UI (WebView: HTMX + Alpine.js + Tailwind)

The WebView renders HTML served by the local Axum server.

**Navigation flow:**
```
User clicks "OCR" in sidebar
  → HTMX sends GET to https://localhost:{PORT}/tools/ocr
    with Authorization: Bearer {SESSION_TOKEN}
  → Axum returns HTML fragment for OCR panel
  → HTMX swaps the main content area
```

**Layout (responsive — 3 breakpoints):**
```
Desktop (≥1024px)             Tablet (≥640px <1024px)       Mobile (<640px)
┌──────────┬───────────────┐  ┌─────┬─────────────────┐    ┌───────────────────┐
│ Sidebar  │               │  │Icons│                 │    │                   │
│ full     │  Active Tool  │  │only │  Active Tool    │    │   Active Tool     │
│ 📋 Clip  │               │  │     │                 │    │                   │
│ 📝 Notes │               │  │ 📋  │                 │    │                   │
│ 🎙️ Voice │               │  │ 📝  │                 │    │                   │
│ 🔤 OCR   │               │  │ 🎙️  │                 │    ├───────────────────┤
│ 🌐 Trans │               │  │ 🔤  │                 │    │  📋  📝  🎙️  🔤  ⋯ │
│ 🔍 Search│               │  └─────┴─────────────────┘    └───────────────────┘
└──────────┴───────────────┘   Icon-only sidebar               Bottom Nav
                               (tooltip on hover)
```

Tailwind responsive classes handle the switch (`sm:`, `lg:` prefixes). No JavaScript layout logic.

**Breakpoints:**
- `< 640px` — mobile phones: bottom navigation bar, full-screen tool
- `640px–1023px` — tablets (Android tablet, iPad): icon-only collapsed sidebar with tooltips
- `≥ 1024px` — desktop: full sidebar with labels

**Command Palette** (`Ctrl+K` / `Cmd+K`): Global search across tools and notes. Implemented as an HTMX overlay triggered by Alpine.js keyboard listener. Available on all screens.

**Alpine.js usage rules:** Only for UI state that does not leave the component (open/close modals, active tab within a tool, toggle visibility). Never for data fetching (that's HTMX) or cross-tool state (that's the Event Bus).

---

### Layer 3 — Core Engine (Rust + Axum)

An HTTP server runs internally on `localhost:{PORT}`. Only localhost connections are accepted. Every request must include a `Authorization: Bearer {SESSION_TOKEN}` header. The token is generated at startup and injected into the WebView as a JS constant.

**Port selection:** The server tries port `47821` once. If occupied, it binds `127.0.0.1:0` and lets the OS assign a guaranteed-free port. The selected port is written to `app_data_dir()/server.port` so the WebView frontend can read it. No unbounded loop. See D-053.

**Axum router structure:**
```
GET  /                         → shell.html (app entry)
GET  /tools/{tool_name}        → tool main panel
POST /tools/{tool_name}/{action} → tool action handler
GET  /tools/{tool_name}/partials/{partial} → HTMX fragment
GET  /plugins/{plugin_id}/*    → proxied to plugin subprocess
GET  /mcp                      → MCP SSE endpoint
POST /mcp                      → MCP stdio-compatible endpoint
GET  /api/settings             → settings R/W
GET  /api/models               → model management
```

---

### Layer 4 — Database (SQLite)

Single file: `eleutheria.db`. Managed with `sqlx` in Rust with compile-time query checking.

**Schema:**

```sql
-- Notes
CREATE TABLE notes (
  id          TEXT PRIMARY KEY,  -- UUID
  title       TEXT NOT NULL DEFAULT '',
  content     TEXT NOT NULL DEFAULT '',
  content_fts TEXT,              -- stripped for FTS indexing
  tags        TEXT DEFAULT '[]', -- JSON array
  pinned      INTEGER DEFAULT 0,
  created_at  INTEGER NOT NULL,  -- Unix timestamp
  updated_at  INTEGER NOT NULL
);
CREATE VIRTUAL TABLE notes_fts USING fts5(title, content_fts, content='notes');

-- Clipboard History
CREATE TABLE clipboard (
  id          TEXT PRIMARY KEY,
  content     TEXT NOT NULL,     -- text/html/url content, or relative path for images
  content_type TEXT NOT NULL,    -- 'text' | 'url' | 'html' | 'image' | 'file'
  source_app  TEXT,
  created_at  INTEGER NOT NULL
);
-- Detection priority: image > html > url > file > text
-- Images stored as files in user-files/clipboard/{uuid}.png (not as blobs). See D-058.

-- Settings
CREATE TABLE settings (
  key         TEXT PRIMARY KEY,
  value       TEXT NOT NULL      -- JSON value
);

-- Plugin sandboxed storage
CREATE TABLE plugin_data (
  plugin_id   TEXT NOT NULL,
  key         TEXT NOT NULL,
  value       TEXT NOT NULL,
  PRIMARY KEY (plugin_id, key)
);

-- Model registry
CREATE TABLE models (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  tool        TEXT NOT NULL,    -- 'whisper' | 'argos' | etc.
  size_bytes  INTEGER,
  path        TEXT,
  downloaded  INTEGER DEFAULT 0,
  downloaded_at INTEGER
);
```

**Backup:** The user can export the full `eleutheria.db` and a `/user-files/` directory as a ZIP from the Settings panel. Import restores both.

**`/user-files/` contents:** audio recordings, processed video outputs, photo editor exports, clipboard images. Does NOT include original files opened by the user for editing (those stay at their original path).

**Import modes:** Merge (add what doesn't exist; sequence-number-higher version wins conflicts) or Replace All (requires explicit confirmation). Schema migration: the imported DB's `user_version` is checked; pending migrations are applied via `sqlx::migrate!()` on a copy before merging. If the imported DB is newer than the current schema, import is rejected with a clear message. See D-057.

---

### Layer 5 — Plugin System

A plugin is a folder inside `/plugins/` with a `manifest.json` and an entry point.

**manifest.json specification:**
```json
{
  "id": "my-plugin",              // unique, kebab-case
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "Community Dev",
  "description": "Does something useful",
  "entry": "plugin.py",           // or plugin.js, plugin.exe
  "runtime": "python",            // "python" | "node" | "binary"
  "min_app_version": "0.1.0",
  "icon": "🔧",
  "routes": ["/plugins/my-plugin"],
  "permissions": [
    "db.read",
    "db.write",
    "clipboard.read",
    "clipboard.write",
    "event_bus.subscribe",
    "event_bus.publish",
    "fs.user_dir",
    "ocr.invoke",
    "tts.invoke",
    "translate.invoke",
    "notifications.show",
    "network.outbound"
  ],
  "mcp_tools": [
    {
      "name": "my_tool",
      "description": "Description for AI agents",
      "input_schema": {
        "type": "object",
        "properties": {
          "input": { "type": "string" }
        },
        "required": ["input"]
      }
    }
  ],
  "sidebar": {
    "show": true,
    "label": "My Plugin",
    "order": 100
  }
}
```

**Plugin lifecycle:**
```
App starts
  → Plugin Loader scans /plugins/*/manifest.json
  → Validates manifest schema and permissions
  → Spawns plugin subprocess (Python/Node/binary)
  → Registers plugin routes in Axum (proxied)
  → Registers plugin MCP tools in MCP server
  → Adds plugin icon to sidebar
  → Plugin is ready
```

**Plugin HTTP contract:** Plugins receive a `X-Session-Token` header and a `X-Plugin-Id` header on proxied requests. They must respond with valid HTML fragments (for GET) or JSON (for POST actions).

**Permission enforcement:** Axum middleware extracts `plugin_id` from the request path, loads the plugin's declared permissions from a startup-cached `Arc<RwLock<HashMap>>`, and validates against a static `(path_prefix, HTTP_method) → required_permission` table. A plugin attempting an operation it hasn't declared receives `403 { "error": "permission_denied", "required": "permission_name" }`. `fs.user_dir` grants read/write/create-subdirectory access inside `~/eleutheria/plugins/{plugin_id}/` only — path traversal (`../`) is rejected. No permission escalation at runtime. See D-040, D-041.

---

### Layer 6 — AI Layer (Local Models + Online Fallback)

All AI features follow this fallback chain:

```
Local model available? → Use it (offline)
        ↓ no
User has configured online API? → Use it (online)
        ↓ no
Show download prompt for local model
```

**Model management:** Settings → Models panel shows all available models with size, quality rating, and download status. Models are downloaded via Tauri's HTTP client with progress reporting. Stored in `/models/` (gitignored).

| Tool | Engine | Model size | Quality |
|---|---|---|---|
| Voice-to-text | Whisper.cpp | tiny: 75MB / base: 142MB / small: 466MB | ⭐/⭐⭐/⭐⭐⭐ |
| OCR | Tesseract | ~20MB per language | ⭐⭐⭐ |
| Translation | Argos Translate | ~100MB per language pair | ⭐⭐ |
| Translation (online) | DeepL API | API key required | ⭐⭐⭐⭐⭐ |
| Background removal | rembg (U2Net) | ~170MB | ⭐⭐⭐⭐ |

---

### Layer 7 — MCP Server

Exposes all built-in tools and plugin tools as MCP-compatible tools for AI agents (Claude, Cursor, Windsurf, etc.).

**Transport options:**
- `stdio` — for direct agent integration (e.g., Claude Code)
- `SSE` — for web-based agent clients via `GET /mcp`

**Built-in MCP tools:**
```
ocr_screenshot       → captures screen area, returns extracted text
transcribe_audio     → transcribes audio file or live recording
create_note          → creates a note with title, content, optional tags
search_notes         → full-text search in notes
get_clipboard        → returns recent clipboard entries
set_clipboard        → sets clipboard content
translate_text       → translates text with specified source/target language
process_video        → runs ffmpeg operation on a video file
```

**Plugin MCP tools** are registered automatically from `manifest.json` → `mcp_tools` field. No extra configuration required.

---

### Event Bus

Internal pub/sub for cross-tool communication. Tools subscribe to event types; the core engine routes events to subscribers.

**Built-in events:**
```
ocr.completed        → { text: string, source: string }
transcription.completed → { text: string, language: string }
clipboard.changed    → { content: string, type: string }
note.created         → { id: string, title: string }
note.updated         → { id: string }
translation.completed → { original: string, translated: string, target_lang: string }
recording.started    → { type: 'screen' | 'audio' }
recording.stopped    → { file_path: string }
```

**Quick Actions (Phase 2):** User-defined pipelines connecting events to actions. Example: `on ocr.completed → translate_text(target: 'en') → set_clipboard`. No code required.

---

## Security Model

| Concern | Solution |
|---|---|
| External apps accessing localhost server | Session token required on every request |
| Plugin accessing unauthorized resources | Permissions declared in manifest, enforced in Rust |
| Plugin data isolation | SQLite rows scoped by `plugin_id` |
| Model downloads | HTTPS only, checksum verified before use |
| Ads (monetization) | Served in isolated iframe, no access to app data |

---

## Monetization

- **Free:** 1 non-intrusive ad shown once per day (on app open, auto-dismissed after 5 seconds, no UI pollution during session)
- **Paid:** $5 USD one-time via Gumroad → generates a license key → verified locally with asymmetric cryptography → no server required after purchase
- **Ad network:** ethical-ads.io or Carbon Ads (developer-focused, no aggressive trackers)

The app is open source. Users can compile without ads. This is acceptable — users who pay do so to support the project, not because they can't avoid it.

---

## Platform-Specific Notes

**macOS:** Requires user permission for screen recording and microphone. Tauri handles permission request dialogs.

**Windows:** Requires `WebView2` runtime (ships with Windows 11, downloadable for Windows 10). ffmpeg bundled as a separate binary in the installer.

**Linux:** Requires `WebKitGTK` (available in all major distros). ffmpeg installable via package manager or bundled.

**Android:** Only 4 tools active: Voice-to-Text, OCR, Notes, Clipboard. OCR uses device camera as input source. Bottom nav layout. System tray not available — app uses Android notification for background clipboard monitoring.
