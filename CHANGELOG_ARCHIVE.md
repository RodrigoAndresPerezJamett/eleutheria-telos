# Eleutheria Telos — Changelog Archive

Sessions before 2026-03-19. For recent work see `CHANGELOG.md`.

---

## [2026-03-19] — Phase 4 complete: Plugin system bug fixes + sidebar wiring

### Summary
Plugin system was implemented but had 5 bugs preventing it from working end-to-end: port collision, proxy 500 on subpaths, permission check logic inverted, `host_reachable: false`, and sidebar edits applied to `shell.html` instead of `index.html` (app loads `index.html`, not `shell.html`).

### Bug fixes
| Bug | Root cause | Fix |
|-----|-----------|-----|
| All plugins same port | `find_free_port_sync()` rescanned from DEFAULT_PORT each call | `find_free_port_from(next_port)` with counter in `plugin_loader.rs` |
| Proxy 500 on subpaths | `Path<String>` extractor incompatible with 2-segment routes | Extract from `req.uri().path()` directly |
| Permission check never 403 | Logic inverted | Check request path against each declared route |
| `host_reachable: false` | Plugins called `/api/clipboard` (HTML, not JSON) | Call `/health` (JSON) |
| Sidebar plugins not visible | Edits applied to `shell.html`; app loads `index.html` | Apply to `index.html` |
| `cargo tauri dev` binary error | Two `[[bin]]` entries, no `default-run` | Added `default-run = "app"` to `Cargo.toml` |

### Also completed
- `src-tauri/src/api.rs` — `list_sidebar_plugins` Tauri command
- `src-tauri/src/lib.rs` — `window.__SIDEBAR_PLUGINS__` injected via `initialization_script`
- `ui/index.html` — plugin sidebar loaded from injected JSON at startup

### Verified working
- Both hello-python and hello-node appear in sidebar
- Echo form works end-to-end for both plugins
- Plugins run on separate ports; proxy correctly routes subpaths

---

## [2026-03-19] — Phase 4.5: Node.js example plugin

**`plugins/hello-node/`** — Node.js stdlib-only HTTP server. Routes: `GET /`, `GET /api/hello`, `POST /api/echo`. Graceful SIGTERM shutdown. Smoke-tested standalone.

---

## [2026-03-19] — Phase 4.4: Python example plugin

**`plugins/hello-python/`** — Pure stdlib Python HTTP server. Reads env vars from host. Routes: `GET /`, `GET /api/hello`, `POST /api/echo`. Optional host callback for connectivity check. Smoke-tested standalone.

---

## [2026-03-19] — Phase 4.3: Plugin system full implementation

- `plugin_loader.rs` — `start_plugins()` allocates ports, spawns subprocesses (python/node/binary), injects env vars, returns PluginRegistry + child handles
- `src-tauri/src/plugins.rs` (new) — `GET /api/plugins`, `GET /api/plugins/sidebar`, full reverse proxy at `/plugins/:id/*path` (permission check, prefix strip, forward headers, return response or 502)
- AppState extended: `plugin_registry`, `plugin_processes`
- CI: all tests pass

---

## [2026-03-19] — Phase 4.2: MCP SSE transport

- `mcp.rs` — `GET /mcp` (SSE stream), `POST /mcp?sessionId=` (JSON-RPC), full session map, 11 tools dispatched via loopback HTTP (D-034)
- `AppState` — `mcp_sessions: McpSessions` added
- `tokio-stream` crate added for `ReceiverStream`
- CI: all tests pass

---

## [2026-03-19] — Phase 4.1: MCP stdio transport

- `src-tauri/src/mcp_stdio.rs` — `[[bin]]` target `eleutheria-mcp`; JSON-RPC over stdin/stdout; `initialize`, `tools/list`, `tools/call` for all 11 tools; `McpClient` wraps reqwest with bearer auth
- `Cargo.toml` — `tokio io-std` feature, `reqwest json` feature added

---

## [2026-03-19] — Phase 4.6: Plugin developer documentation

- `plugins/README.md` — full guide: manifest schema, runtimes, env vars, routing/permissions, HTMX conventions, local dev workflow, reference implementations table, new plugin checklist

---

## [2026-03-19] — Phase 4.7: Quick Actions (basic pipeline system)

- `migrations/004_phase4_pipelines.sql` — `pipelines` + `pipeline_steps` tables
- `quick_actions.rs` — pipeline CRUD, step CRUD, execution engine (`translate`, `copy_clipboard`, `save_note`), `start_pipeline_engine()` subscribes to Event Bus
- `ocr.rs`, `voice.rs` — emit `OcrCompleted` / `TranscriptionCompleted` events
- `ui/tools/quick-actions/index.html` — two-column pipeline list + step editor

---

## [2026-03-19] — Phase 3 Step 4+5: Photo Editor + Background Removal + Video Processor

**Photo editor:** `photo_editor.rs`, `ui/tools/photo-editor/index.html` (Fabric.js canvas, layer system via `window.__peLayers[]` outside Alpine proxy — D-029). `POST /api/photo/layer`, `/api/photo/bg-remove`, `/api/photo/export`.

**Background removal:** Python `scripts/rembg_remove.py` via subprocess.

**Video processor:** `video_processor.rs`, `ui/tools/video-processor/index.html`. Operations: cut/trim, extract audio, compress (`libx264` — D-032), change resolution. Accepts file path input (D-030). Separate field names for compress vs resize resolution (D-031).

---

## [2026-03-19] — Phase 3 Step 2: Audio Recorder

- `audio_recorder.rs` — `POST /api/audio/start` (ffmpeg -f pulse), `POST /api/audio/stop` (stdin `q\n`), `GET /api/audio/state`
- Output: `~/Music/Eleutheria/recording-{timestamp}.{ext}` (mp3/wav/ogg/flac)
- `ui/tools/audio-recorder/index.html` — Alpine timer, format selector, Start/Stop

---

## [2026-03-19] — Phase 3 Step 1: Screen Recorder

- `screen_recorder.rs` — `POST /api/screen/start` (spawns wf-recorder), `POST /api/screen/stop` (SIGTERM — D-028), `GET /api/screen/status`
- `ui/tools/screen-recorder/index.html` — Alpine timer, audio toggle, Start/Stop, result card with file path

---

## [2026-03-18] — Phase 2 Step 5: OCR + Translation pipeline

- `ocr.rs` `render_result` — inline "Translate…" section in OCR result card using Alpine `x-show`
- Reuses `POST /api/translate/text` — no new routes
- Note: translation fails at runtime due to Python 3.14 / argostranslate incompatibility (D-027 → fixed in D-036)

---

## [2026-03-18] — Phase 2 Step 4: Translation tool

- `translate.rs` — `GET /api/translate/langs`, `POST /api/translate/text`, `POST /api/translate/copy`
- `scripts/translate.py` — `argostranslate.translate` (later replaced by ctranslate2 in D-036)
- `ui/tools/translate/index.html` — language selector, textarea, result card

---

## [2026-03-18] — Phase 2 Step 3: Voice tool

- `voice.rs` — `POST /api/voice/record/start` (ffmpeg -f pulse), `/stop` (stdin q, runs Whisper), `/file` (multipart upload), `/copy`, `/save-note`
- `scripts/transcribe.py` — pywhispercpp; auto-discovers ggml model; `--lang` flag
- `ui/tools/voice/index.html` — language selector, start/stop, file upload, result card

---

## [2026-03-18] — Phase 2 Step 2: OCR tool

- `ocr.rs` — `POST /api/ocr/capture` (slurp+grim+tesseract), `/file` (multipart), `/copy`, `/save-note`
- `ui/tools/ocr/index.html` — language selector, Capture button, file upload, result card
- `axum multipart` feature added

---

## [2026-03-18] — Phase 2 Step 1: Models panel

- `migrations/003_phase2_models.sql` — seeds Whisper + Argos model catalog
- `models.rs` — `GET /api/models`, `POST /api/models/:id/download` (non-blocking reqwest stream), `GET /api/models/:id/progress`, `DELETE /api/models/:id`
- `scripts/install_argos_package.py`, `uninstall_argos_package.py` added
- `reqwest stream` + `tokio fs + process` features added

---

## [2026-03-18] — Phase 1: Notes + Search

- `notes.rs` — CRUD (create/list/get/update/delete/pin/search), FTS5 via SQL triggers (D-012), `hx-trigger="noteUpdated from:body"` pattern
- `search.rs` — `GET /api/search?q=` queries both notes_fts and clipboard; renders unified result list
- `ui/tools/notes/index.html` — two-column editor (list + inline editor via Alpine)
- `ui/tools/search/index.html` — search bar, unified result list, `Ctrl+K` palette integration

---

## [2026-03-18] — Phase 1: Clipboard History

- `clipboard.rs` — `GET /api/clipboard`, `POST /api/clipboard/:id/recopy`, `DELETE /api/clipboard/:id`, clipboard monitor spawned in background (arboard + `wayland-data-control` feature — D-022)
- Dedup via `DefaultHasher` (D-013); suppress-channel via `tokio::sync::watch` (D-014)
- `ui/tools/clipboard/index.html` — list with `hx-trigger="load, every 3s"`, recopy + delete per item
- `migrations/002_fts5_triggers.sql` — FTS5 sync triggers for notes

---

## [2026-03-18] — Phase 0 dev-mode fix (D-011)

Removed `devUrl` from `tauri.conf.json`. Tauri CLI was timing out waiting for Axum before the binary compiled (chicken-and-egg). Fix: Tauri serves `ui/index.html` as static file via `frontendDist`. HTMX requests rewritten to absolute Axum URL via `htmx:configRequest`. CORS added via `tower-http CorsLayer`.

---

## [2026-03-18] — Phase 0: Foundation

- Full project repo created; branch structure; core docs (ARCHITECTURE, PRINCIPLES, ROADMAP, CLAUDE, CHANGELOG, DECISIONS, IDEAS)
- Tauri 2.x initialized; SQLite schema (`migrations/001_initial.sql`); Axum AppState + auth middleware; Event Bus (broadcast); Plugin loader (manifest scan); i18n loader; MCP stubs (501); system tray; session token injection via `initialization_script`; HTMX shell with 3-breakpoint responsive layout
- Environment verified: Rust 1.92, Node 22.20, Tauri CLI 2.10.1, ffmpeg-free 7.1.2, Tesseract 5.5.2, Python 3.14.2
- CI/CD: `ci.yml`, `build.yml`, `release.yml` configured

---

## [2026-03-18] — Route param syntax fix (D-020)

All parameterized routes used `{param}` syntax → 404 at runtime. Axum 0.7 uses matchit 0.7.3 which requires `:param`. Fixed across `server.rs`, `clipboard.rs`, `notes.rs`. Diagnostic code removed.

---

## [2026-03-18] — "Loading…" bug post-mortem

Seven root causes, all causing silent failure. Summary (full detail in DECISIONS.md D-017–D-022):
1. `{param}` route syntax → 404 (D-020)
2. HTMX 2.x `selfRequestsOnly: true` blocks cross-origin requests (D-017)
3. HTMX/Alpine loaded from CDN → WebKitGTK blocked them (D-018)
4. `hx-trigger="load"` fires before session token is available (D-019)
5. `htmx.ajax()` misses child `hx-trigger="load"` — fix: `htmx.process()` in `htmx:afterSwap`
6. Notes `+New` — `Json<T>` handler rejects HTMX form POST (D-021)
7. arboard on Wayland needs `wayland-data-control` feature (D-022)
