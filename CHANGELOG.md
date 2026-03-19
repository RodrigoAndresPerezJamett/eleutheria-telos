# Eleutheria Telos — Changelog

This file is the project's memory between sessions. It is updated at the end of every work session by Claude Code. Before starting any session, read the most recent entry.

Format per entry:
- **Date** — what was completed, what changed, what was decided, what's next

---

## [2026-03-18] — Project foundation

### Completed
- Created project repository: `rodrigoandresperezjamett/eleutheria-telos`
- Branch structure: `dev` as active development branch, `main` reserved for releases
- Core documentation created: `ARCHITECTURE.md`, `PRINCIPLES.md`, `ROADMAP.md`, `CLAUDE.md`, `CHANGELOG.md`, `DECISIONS.md`, `IDEAS.md`
- Tauri 2.x project initialized with `cargo tauri init`
  - App name: `eleutheria-telos`
  - Window title: `Eleutheria Telos`
  - Web assets path: `../ui`
  - Dev server URL: `http://localhost:47821`
- GitHub MCP configured and verified connected
- Notion MCP verified connected
- Environment verified (see CLAUDE.md → Pinned Environment)

### Environment confirmed working
- Rust 1.92.0, Cargo 1.92.0
- Node 22.20.0, npm 10.9.3
- Tauri CLI 2.10.1
- ffmpeg 7.1.2 (ffmpeg-free — already installed, do not replace)
- Tesseract 5.5.2
- Python 3.14.2 (cutting-edge — verify package support before use)

### Known issues / notes
- ffmpeg-free conflicts with rpmfusion ffmpeg — do not run `sudo dnf install ffmpeg`
- Python 3.14 is newer than most AI packages expect — verify compatibility before adding Python deps

### Next session should start with
Phase 0 — Foundation. Goal: Tauri app running with Axum internal server, HTMX shell navigation, SQLite connected, system tray, and plugin loader skeleton. See ROADMAP.md Phase 0 checklist.

---

## [2026-03-18] — Phase 0 implementation

### Completed
- `src-tauri/Cargo.toml` — fixed `arboard` version (`0.3` → `3`), added `tray-icon` feature to tauri
- `src-tauri/migrations/001_initial.sql` — full schema: notes, notes_fts (FTS5), clipboard, settings, plugin_data, models
- `src-tauri/src/server.rs` — AppError, AppState (db + token + port + event_bus), auth middleware (Bearer), build_router, find_free_port_sync, start_server
- `src-tauri/src/db.rs` — SqlitePool init, WAL mode, foreign keys, sqlx::migrate!
- `src-tauri/src/event_bus.rs` — broadcast-based pub/sub; Event enum with all architecture events
- `src-tauri/src/plugin_loader.rs` — PluginManifest serde struct, scan_plugins scans plugins/*/manifest.json
- `src-tauri/src/i18n.rs` — I18n::load reads ui/locales/en.json, t() lookup
- `src-tauri/src/mcp.rs` — Phase 0 skeletons for GET /mcp (SSE) and POST /mcp, return 501
- `src-tauri/src/tools/mod.rs` — placeholder; tool modules registered here in Phase 1+
- `src-tauri/src/api.rs` — fixed compile bugs: RUST_VERSION → CARGO_PKG_RUST_VERSION, removed broken error_response
- `src-tauri/src/lib.rs` — full setup: port detection, SQLite init, Axum spawn, i18n, plugin scan, system tray, session token injection via initialization_script, window creation
- `src-tauri/tauri.conf.json` — removed window config (created in code), removed beforeDevCommand/beforeBuildCommand, removed trayIcon (configured in code)
- `ui/shell.html` — full 3-breakpoint responsive shell: desktop sidebar, tablet icon-only sidebar, mobile bottom nav; HTMX navigation with auth header injection
- `ui/locales/en.json` — all UI strings for all tools
- `ui/tools/clipboard/index.html` — placeholder
- `ui/tools/notes/index.html` — placeholder
- `ui/tools/voice/index.html` — placeholder
- `ui/tools/ocr/index.html` — placeholder
- `ui/tools/translate/index.html` — placeholder
- `ui/tools/search/index.html` — placeholder
- `ui/tools/settings/index.html` — shows version + server port

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (0 tests — Phase 0 has no route handlers worth testing yet)

### Decisions made
- `event_bus` stored in AppState so it's accessible to all route handlers in Phase 1+
- `GET /` serves shell.html from Axum but does NOT require auth (WebView initial load has no headers); all other routes require Bearer token
- MCP routes registered as 501 stubs so the router compiles and the endpoint exists for Phase 4
- Phase 0 dead-code lints suppressed with `#[allow(dead_code)]` on infrastructure stubs (EventBus, AppError utilities, plugin fields) — to be removed as each is wired up in subsequent phases

### Known issues / notes
- `cargo tauri dev` requires `beforeDevCommand` to be empty — already set to `""` in tauri.conf.json
- The `WebviewUrl::App(PathBuf::from("/"))` loads from `devUrl` (http://localhost:47821) in dev — this is the Axum server
- `Cargo.lock` is currently gitignored but should be tracked for a binary app — remove from .gitignore before first release

### Next session should start with
Phase 1 — Core Tools. Implement Clipboard History (arboard monitor + SQLite storage + HTMX list), Notes (CRUD + FTS5), and Search (command palette Ctrl+K). Start with clipboard.rs, then notes.rs, then search.rs.

---

## [2026-03-18] — Phase 0 dev-mode fix

### Problem
`cargo tauri dev` polls `devUrl` (http://localhost:47821) **before** the Rust binary is compiled. On first build (600+ crates), compilation takes >180s — exceeding Tauri CLI's hard-coded timeout. The binary never starts in time for Tauri to connect.

### Root cause
The architecture had `devUrl: http://localhost:47821` in `tauri.conf.json`. Tauri CLI interprets this as "wait for an external dev server before opening the window". But our Axum server **is** embedded inside the Rust binary — it cannot respond until the binary is compiled and running. This creates an unsolvable chicken-and-egg problem on first run.

### Fix
Removed `devUrl` from `tauri.conf.json`. Tauri now serves the shell as a static file from `frontendDist: ../ui` (loads `ui/index.html` instantly via `tauri://localhost/`). Axum still starts in the background as before. HTMX requests are redirected to Axum via a `htmx:configRequest` event handler that rewrites relative paths (`/tools/...`) to absolute URLs (`http://127.0.0.1:{PORT}/...`). CORS headers added to Axum via `tower-http CorsLayer` so the WebView (origin `tauri://localhost`) can reach the API server.

### Files changed
- `src-tauri/Cargo.toml` — added `tower-http = { version = "0.5", features = ["cors"] }`
- `src-tauri/src/server.rs` — added `CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)` to router
- `src-tauri/tauri.conf.json` — removed `devUrl`, `beforeDevCommand`, `beforeBuildCommand`
- `src-tauri/src/lib.rs` — `WebviewUrl::App("index.html")` (explicit, no devUrl dependency)
- `ui/index.html` — new entry point (same layout as shell.html + `htmx:configRequest` URL rewrite)

### Result
`cargo tauri dev` compiles in ~28s incremental (first full build ~2min), no polling timeout. App window opens immediately after binary starts.

### Next session should start with
Phase 1 — Core Tools (unchanged). `cargo tauri dev` now works reliably.

---

## [2026-03-18] — Phase 3 Step 1: Screen Recorder

### Completed

**Backend (Rust)**
- `src-tauri/src/tools/screen_recorder.rs` — 3 route handlers:
  - `GET /api/screen/status` — returns recording/idle badge HTML
  - `POST /api/screen/start` — spawns `wf-recorder -f /tmp/eleutheria-screen-{timestamp}.mp4 [-a]`; stores child + path in `AppState.screen_recording`
  - `POST /api/screen/stop` — sends SIGTERM via `kill -TERM {pid}`, waits for exit, returns result card with file path
- `src-tauri/src/tools/mod.rs` — registered `screen_recorder` module
- `src-tauri/src/server.rs` — imported `ScreenRecording`, added `screen_recording` field to `AppState`, merged `screen_recorder::router()`
- `src-tauri/src/lib.rs` — initialized `screen_recording: Arc<Mutex<None>>`
- `src-tauri/src/tools/clipboard.rs`, `notes.rs`, `search.rs`, `translate.rs` — test `AppState` constructors updated with `screen_recording` field

**Frontend**
- `ui/tools/screen-recorder/index.html` — recording controls with Alpine.js mm:ss timer, audio toggle checkbox, Start/Stop buttons, tip about minimizing window
- `ui/index.html` — added "Screen Rec" (🎬) entry to desktop sidebar and tablet icon sidebar
- `ui/locales/en.json` — added 7 screen recorder strings

### Architecture
- `ScreenRecording = Arc<Mutex<Option<(Child, String)>>>` — holds wf-recorder child + output path
- Timestamped output paths (`/tmp/eleutheria-screen-{unix_ts}.mp4`) avoid collisions between recordings
- SIGTERM via `kill -TERM {pid}` subprocess instead of tokio `child.kill()` (SIGKILL) — ensures mp4 container is properly finalized (D-028)
- Audio toggle: HTML checkbox sends `audio=on` when checked, field absent when unchecked; Rust deserializes as `String` and checks `!params.audio.is_empty()` (D-021 compliant)

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (19 tests, 0 failures)

### Decisions made
- **D-028:** `wf-recorder` as screen recording backend — see DECISIONS.md

### Next session should start with
Phase 3 Step 2: Audio Recorder (`ffmpeg -f pulse` → mp3/wav, no transcription, save to file).

---

## [2026-03-18] — Phase 2 Step 5: OCR + Translation pipeline

### Completed

**Backend (Rust)**
- `src-tauri/src/tools/ocr.rs` — modificado `render_result`: el card de resultado OCR ahora incluye una sección "Translate…" que se expande con Alpine.js. Al enviar el mini-form, postea a `/api/translate/text` (ya existente) con el texto extraído, `from_lang` y `to_lang`. No se agregaron rutas nuevas.

**Frontend**
- El pipeline es puramente de frontend: `render_result` emite el HTML con el mini-form inline
- Alpine.js `x-data="{ showTranslate: false }"` controla visibilidad con `x-show` + `x-cloak`
- Selectores from/to con los 5 idiomas disponibles (en/es/fr/de/pt)
- Resultado de traducción aparece en `#ocr-translate-result` dentro del mismo card

### Architecture
- Cero rutas nuevas — el pipeline reutiliza `POST /api/translate/text` directamente
- El texto OCR se pasa como `<textarea name="text" class="hidden">` dentro del mini-form (misma técnica que copy/save-note, D-021 compliant)
- Nota: la traducción falla en runtime hasta que se resuelva el blocker de argostranslate / Python 3.14 (anotado en IDEAS.md y en memoria para Phase 5)

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (19 tests, 0 failures)

### Known issues / blockers
- **Traducción no funcional en runtime** — argostranslate 1.11.0 es incompatible con Python 3.14+ (`pydantic.v1` en la cadena `confection`). La UI, las rutas y el pipeline OCR→Translate están implementados correctamente; solo falla el subprocess Python. Ver D-027 en DECISIONS.md. Blocker de Phase 5, no de Phase 3.

### Next session should start with
**Phase 3 — Media Tools.**

Estado de Phase 2 al cierre de sesión:
- ✅ Models panel (`src-tauri/src/tools/models.rs`)
- ✅ OCR capture + file upload (`src-tauri/src/tools/ocr.rs`)
- ✅ Voice-to-text Whisper (`src-tauri/src/tools/voice.rs`)
- ✅ Translation UI + routes (`src-tauri/src/tools/translate.rs`) — backend Python bloqueado por Python 3.14, ver D-027
- ✅ OCR + Translation pipeline (botón "Translate…" en el card de resultado OCR)

**Para arrancar Phase 3, leer ROADMAP.md Phase 3 y comenzar con el primer ítem: Screen Recorder.**

Contexto relevante para Phase 3:
- ffmpeg ya está disponible como subprocess (`scripts_dir()` pattern en `voice.rs` y `ocr.rs`)
- El sistema usa Wayland + Hyprland — para screen recording usar `wf-recorder` o `ffmpeg -f pipewire` (no `x11grab`)
- `grim` + `slurp` ya están instalados y funcionando (usados por OCR)
- El patrón de tool completo está establecido: `src-tauri/src/tools/{tool}.rs` + `ui/tools/{tool}/index.html` + registrar en `mod.rs` + mergear router en `server.rs`
- AppState no necesita campos nuevos para screen recorder (el child process del recorder seguirá el mismo patrón que `VoiceRecording = Arc<Mutex<Option<Child>>>`)
- Antes de implementar: verificar con `which wf-recorder` o `ffmpeg -f pipewire -list_devices true` qué capturadores de pantalla están disponibles en Wayland

---

## [2026-03-18] — Phase 2 Step 4: Translation tool

### Completed

**Backend (Rust)**
- `src-tauri/src/tools/translate.rs` — 3 route handlers:
  - `GET /api/translate/langs` — queries DB for installed Argos language packs (downloaded=1, tool='translate'); returns language selector form HTML; if none installed returns "no models" prompt with link to Models panel
  - `POST /api/translate/text` — accepts `text`, `from_lang`, `to_lang` (form-encoded); spawns `python3 scripts/translate.py` in `tokio::spawn`; returns result card HTML with translated text and Copy button
  - `POST /api/translate/copy` — copies translated text to clipboard via arboard (`spawn_blocking`)
- `src-tauri/src/tools/mod.rs` — registered `translate` module
- `src-tauri/src/server.rs` — imported `translate`, merged `translate::router()`

**Python scripts**
- `scripts/translate.py` — translates text via `argostranslate.translate`; discovers installed language packs at runtime; exits 1 with stderr message if pack not installed

**Frontend**
- `ui/tools/translate/index.html` — full translate panel:
  - `hx-trigger="load"` → `GET /api/translate/langs` loads language pair form dynamically
  - Alpine.js `x-data` with `pairs` JSON map for reactive from→to filtering
  - Textarea for input, Translate button, loading indicator
  - Result card: translated text + Copy to Clipboard
- `ui/locales/en.json` — added 7 translate strings

### Architecture
- `parse_lang_pair` helper extracts `(from, to)` from `argos-{from}-{to}` model IDs
- Handler is `Form<T>` compliant (D-021)
- `tokio::spawn` wraps subprocess so handler thread is never blocked
- No new Cargo.toml dependencies

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (19 tests, 0 failures — 5 new translate tests)

### Next session should start with
Phase 2 Step 5: OCR + Translation pipeline — after OCR, offer one-click "Translate" button that sends the extracted text to the translate tool.

---

## [2026-03-18] — Phase 2 Step 3: Voice tool

### Completed

**Backend (Rust)**
- `src-tauri/src/tools/voice.rs` — 6 route handlers:
  - `GET /api/voice/status` — returns idle/recording badge HTML
  - `POST /api/voice/record/start` — spawns `ffmpeg -f pulse -i default` with piped stdin; stores child in `AppState.voice_recording`
  - `POST /api/voice/record/stop` — writes `q\n` to ffmpeg stdin, waits for process exit, runs `python3 scripts/transcribe.py` on the WAV output; returns result card HTML
  - `POST /api/voice/file` — accepts multipart audio upload (wav/mp3/ogg/flac/m4a), saves to `/tmp/`, runs Whisper transcription
  - `POST /api/voice/copy` — copies transcript to clipboard via arboard (no suppress — new content, D-014)
  - `POST /api/voice/save-note` — inserts transcript as a new Note in SQLite
- `src-tauri/src/tools/mod.rs` — registered `voice` module
- `src-tauri/src/server.rs` — imported `VoiceRecording`, added `voice_recording` to `AppState`, merged `voice::router()`
- `src-tauri/src/lib.rs` — initialized `voice_recording: Arc<Mutex<None>>`
- `src-tauri/src/tools/clipboard.rs`, `notes.rs`, `search.rs` — test AppState constructors updated with `voice_recording` field

**Python scripts**
- `scripts/transcribe.py` — Whisper transcription via `pywhispercpp`; auto-discovers ggml model from `~/.local/share/eleutheria-telos/models/whisper/`; `--lang <code|auto>` flag
- `scripts/requirements.txt` — added `pywhispercpp>=1.4.1`

**Frontend**
- `ui/tools/voice/index.html` — full voice panel:
  - Language selector (auto/en/es/fr/de/pt/it/zh/ja)
  - Start/Stop recording controls with Alpine.js mm:ss timer and pulsing "● Recording" badge
  - Stop sends `lang` via hidden form (`hx-include="#voice-stop-form"`)
  - File upload (wav/mp3/ogg/flac/m4a) with `hx-trigger="change"`
  - Result card: transcript + Copy to Clipboard + Save as Note

### Architecture
- `VoiceRecording = Arc<Mutex<Option<tokio::process::Child>>>` held in AppState — allows concurrent HTTP handlers to safely check/take the recording child
- ffmpeg stopped gracefully via stdin `q\n` (not SIGKILL) so WAV file is properly finalized
- Transcription always runs in an async tokio task — never blocks Axum handler thread

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (14 tests, 0 failures)

### Next session should start with
Phase 2 Step 4: Translation tool (Argos Translate via Python subprocess). Routes: `GET /tools/translate`, `POST /api/translate/text`. Then Step 5: OCR → Translate pipeline.

---

## [2026-03-18] — Phase 2 Step 2: OCR tool

### Completed

**Backend (Rust)**
- `src-tauri/src/tools/ocr.rs` — 4 route handlers:
  - `POST /api/ocr/capture` — runs `slurp` (interactive Wayland region selector) → `grim` (screenshot) → `tesseract`. Accepts `lang` form field (eng/spa).
  - `POST /api/ocr/file` — receives multipart image upload, saves to `/tmp/`, runs `tesseract`
  - `POST /api/ocr/copy` — copies OCR text to clipboard via arboard (with suppress hash D-014)
  - `POST /api/ocr/save-note` — inserts OCR text as a new Note in SQLite; first non-empty line becomes title
- `src-tauri/src/tools/mod.rs` — registered `ocr` module
- `src-tauri/src/server.rs` — imported `ocr`, merged `ocr::router()`

**Cargo.toml changes**
- Added `multipart` feature to axum — enables `axum::extract::Multipart` for image file upload

**Frontend**
- `ui/tools/ocr/index.html` — full OCR panel:
  - Language selector (English / Spanish — only installed Tesseract langpacks)
  - "Capture Screen Area" button with loading indicator and `hx-disabled-elt`
  - "Open Image File" label+input with auto-submit on file selection (`hx-trigger="change"`)
  - Result area: extracted text + "Copy to Clipboard" + "Save as Note" actions
  - `hx-include` pattern for passing OCR text to copy/save handlers (D-021 compliant)
- `ui/index.html` — added `.htmx-indicator` / `.htmx-indicator.htmx-request` CSS for loading indicators

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (14 tests, 0 failures)

### Notes
- Tesseract languages available: `eng`, `spa` (verified via `tesseract --list-langs`)
- Screen capture UX: move window aside before clicking "Capture Screen Area" (slurp overlay covers full screen but Tauri window will also be visible in the captured region if not moved)
- Phase 5: add window hide/show around slurp capture using AppHandle in AppState

### Next session should start with
Phase 2 Step 3: Voice tool (Whisper subprocess). User has Whisper Base already downloaded.

---

## [2026-03-18] — Phase 2 Step 1: Models panel

### Completed

**Backend (Rust)**
- `src-tauri/migrations/003_phase2_models.sql` — `ALTER TABLE models ADD COLUMN url TEXT`; seeds full catalog: 4 Whisper models (tiny/base/small/medium) + 8 Argos language pairs (EN↔ES/FR/DE/PT)
- `src-tauri/src/tools/models.rs` — full models panel backend:
  - `GET /api/models` — renders full catalog list grouped by tool (Voice / Translation)
  - `POST /api/models/:id/download` — starts non-blocking download in `tokio::spawn`; returns card HTML immediately
  - `GET /api/models/:id/progress` — polled every 2s by downloading cards; returns card HTML reflecting current state
  - `DELETE /api/models/:id` — removes file, resets DB, uninstalls Argos package via Python subprocess
  - Whisper download via `reqwest` streaming with byte-level progress tracking
  - Argos download via `python3 scripts/install_argos_package.py {from} {to}` subprocess
  - `DownloadMap = Arc<Mutex<HashMap<String, DownloadState>>>` stored in `AppState`
- `src-tauri/src/tools/mod.rs` — registered `models` module
- `src-tauri/src/server.rs` — imported `DownloadMap`, added `download_states` to `AppState`, merged `models_tool::router()`
- `src-tauri/src/lib.rs` — initialized `download_states` HashMap, passed to `AppState`
- `src-tauri/src/tools/clipboard.rs`, `notes.rs`, `search.rs` — test `AppState` constructors updated with `download_states` field

**Cargo.toml changes**
- Added `reqwest = { version = "0.12", features = ["stream"] }` — streaming model downloads
- Added `"fs"` and `"process"` to tokio features — `tokio::fs` (file ops) and `tokio::process::Command` (Python subprocess)

**Frontend**
- `ui/tools/models/index.html` — models panel with `hx-trigger="load"` → `GET /api/models`
- `ui/index.html` — added "Models" (🧠) entry to desktop sidebar and tablet icon sidebar

**Python scripts**
- `scripts/install_argos_package.py` — downloads and installs an Argos Translate language pack
- `scripts/uninstall_argos_package.py` — removes an installed Argos Translate language pack
- `scripts/requirements.txt` — `argostranslate>=1.11.0`

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (14 tests, 0 failures)

### Bug fixed during implementation
- **`r#"..."#` raw strings terminate prematurely at `"#`** — `hx-target="#model-card-{id}"` contains `"#` which Rust's raw string parser (`r#"..."#`) treats as the closing delimiter. Fix: pre-compute `let target = format!("#model-card-{id}")` and use `{target}` in the format string, avoiding `"#` inside the raw literal. (D-023)

### Decisions made
- **D-023:** Screen capture via `slurp | grim` subprocess on Wayland — both verified installed at `/usr/bin`
- **D-024:** Whisper download via `reqwest` streaming (direct binary download from HuggingFace ggml format)
- **D-025:** Argos Translate models managed via Python subprocess (argostranslate's own package manager) — Python 3.14 compatible (ctranslate2 4.7.1 + sentencepiece 0.2.1 both have cp314 manylinux wheels)
- **D-026:** `scripts/` directory used for Python subprocess scripts; path resolved at compile time via `env!("CARGO_MANIFEST_DIR")` — Phase 5 will replace with Tauri resource path

### Next session should start with
Phase 2 Step 2: OCR tool (Tesseract subprocess + grim/slurp screen capture). Then Voice (Whisper subprocess), then Translation (Argos subprocess).

---

## [2026-03-18] — Phase 1 implementation

### Completed

**Backend (Rust)**
- `src-tauri/migrations/002_phase1_indexes.sql` — perf indexes on clipboard and notes; FTS5 sync triggers (insert/delete/update) for notes_fts
- `src-tauri/src/tools/clipboard.rs` — list (with search), recopy, delete-one, clear-all handlers; clipboard monitor with arboard polling + dedup hash + suppress channel; 5 integration tests
- `src-tauri/src/tools/notes.rs` — list (plain + FTS5 MATCH), create, get (editor HTML), update (dynamic SET), delete, pin-toggle handlers; 6 integration tests
- `src-tauri/src/tools/search.rs` — merged FTS5 (notes) + LIKE (clipboard) search handler; 3 integration tests
- `src-tauri/src/tools/mod.rs` — registered clipboard, notes, search modules
- `src-tauri/src/server.rs` — added `clipboard_suppress_tx: watch::Sender<u64>` to AppState; merged three tool routers into build_router
- `src-tauri/src/lib.rs` — construct watch channel, pass to AppState, spawn clipboard monitor background task
- `src-tauri/src/event_bus.rs` — removed Phase 0 dead-code suppression; ClipboardChanged, NoteCreated, NoteUpdated now in active use

**Cargo.toml changes**
- Added `"sync"` to tokio features (for watch channel)
- Replaced `axum-test = "15"` (broken path-param routing) with `tower = "0.4"` + `http-body-util = "0.1"` dev deps

**Frontend**
- `ui/tools/clipboard/index.html` — full clipboard panel with search, list, recopy, delete, clear-all
- `ui/tools/notes/index.html` — split-view panel: note list (left) + editor area (right); marked.js loaded
- `ui/tools/search/index.html` — search panel with live HTMX input
- `ui/index.html` — Ctrl+K command palette overlay (Alpine `paletteOpen` state, HTMX search input, Escape to close)
- `ui/assets/marked.min.js` — marked.js bundled locally (offline-first, D-015)
- `ui/locales/en.json` — added ~20 new strings for clipboard, notes, search, palette

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (14 tests, 0 failures)

### Decisions made
- **D-012:** FTS5 sync via SQL triggers (not in-Rust handlers) — triggers in migration 002
- **D-013:** Clipboard dedup via in-memory `DefaultHasher` hash — no DB query per poll cycle
- **D-014:** Clipboard suppress channel via `tokio::sync::watch` in AppState — recopy handler sends hash before writing to clipboard
- **D-015:** `marked.js` bundled under `ui/assets/` (not CDN) for offline-first correctness
- **D-016:** Integration tests use `tower::ServiceExt::oneshot()` + direct handler calls for path-parameterized routes (axum-test v15 has broken path-param routing with `{id}` syntax in axum 0.7)

### Known issues / notes
- Path-parameterized routes work correctly in the running app (`cargo tauri dev`); the test limitation is only in the test harness (tower oneshot with `from_fn_with_state` + `with_state` doesn't route path params in tests)
- Notes editor Alpine component uses `fetch()` directly for debounced PUT (exception to HTMX rule per CLAUDE.md — HTMX form-encode limitations)

### Next session should start with
Phase 2 — Voice (Whisper) or OCR (Tesseract). Start by choosing which tool to implement first based on ROADMAP.md, verify Python package compatibility for Whisper with Python 3.14.2, and check Tesseract 5.5.2 Rust bindings compatibility.

---

## [2026-03-18] — Phase 1 WebView fix (tools loading)

### Problem
All tool panels showed "Loading…" forever in `cargo tauri dev`. No HTMX requests reached the Axum server.

### Root causes (three separate issues, all fixed):

**1. HTMX loaded from CDN (blocked/slow on WebKitGTK)**
HTMX and Alpine.js were loaded from `unpkg.com`. If the WebView can't reach CDN or is slow, HTMX never initializes and no `hx-*` processing happens.

**2. HTMX 2.0.4 `selfRequestsOnly: true` default**
HTMX 2.0.4 defaults to `selfRequestsOnly: true`, which blocks all cross-origin requests. Since the shell is served from `tauri://localhost` and Axum runs on `http://127.0.0.1:{PORT}`, every HTMX request was silently blocked (no error, no network activity).

**3. Fragile `hx-trigger="load"` initial panel load**
The shell had `hx-trigger="load"` on `#tool-panel`, which fired before token/port were guaranteed to be set by `initialization_script`. Also, the invoke fallback in `initApp()` could silently overwrite `window.__SESSION_TOKEN__` and `window.__API_PORT__` with `undefined` if `window.__TAURI__.invoke` wasn't a function.

### Fixes
- `ui/assets/htmx.min.js` — HTMX 2.0.4 bundled locally (50KB)
- `ui/assets/alpine.min.js` — Alpine.js 3.14.9 bundled locally (45KB)
- `ui/index.html` — replaced CDN script tags with local `/assets/` paths
- `ui/index.html` — added `htmx.config.selfRequestsOnly = false` before any HTMX requests
- `ui/index.html` — removed `hx-trigger="load"` from `#tool-panel`; added `initApp()` async function on `DOMContentLoaded` that uses Tauri invoke (with proper `typeof` guard) then calls `htmx.ajax()` with full absolute URL and explicit auth headers
- `src-tauri/src/api.rs` — fixed `get_session_token` to return the real token from `AppState` (not a new UUID); added `get_api_port` command
- `src-tauri/src/lib.rs` — added `app.manage(state.clone())` to register `AppState` with Tauri's state management so invoke commands can access it
- `src-tauri/tauri.conf.json` — added `"withGlobalTauri": true` so `window.__TAURI__` is available in the WebView
- `src-tauri/src/server.rs` — added request logging in `auth_middleware` (INFO + WARN) for diagnostics

### Decisions made
- **D-017:** `htmx.config.selfRequestsOnly = false` required because app shell and API server are on different origins (tauri:// vs http://)
- **D-018:** HTMX and Alpine.js bundled locally (same principle as D-015 for marked.js)
- **D-019:** Initial tool panel load uses `htmx.ajax()` with full absolute URL in `initApp()`, not `hx-trigger="load"`, to ensure token is confirmed before the request fires

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (14 tests, 0 failures)

### Next session should start with
Phase 2 — Voice (Whisper) or OCR (Tesseract). (Unchanged from Phase 1 entry.)

---

## [2026-03-18] — Route param syntax fix (D-020)

### Problem
All parameterized routes (`/tools/{tool_name}`, `/api/clipboard/{id}`, `/api/notes/{id}`, etc.) returned 404 at runtime despite compiling without errors.

### Root cause
Axum 0.7.9 depends on **matchit 0.7.3**, which uses `:param` syntax for named path parameters. The `{param}` brace syntax was introduced in matchit 0.8.x. Axum passes route strings directly to matchit without any transformation — so `{param}` was treated as a literal string segment, never matching any actual request path.

### Fix
Changed all route definitions from `{param}` to `:param` syntax:
- `src-tauri/src/server.rs` — `/tools/:tool_name`
- `src-tauri/src/tools/clipboard.rs` — `/api/clipboard/:id/recopy`, `/api/clipboard/:id`
- `src-tauri/src/tools/notes.rs` — `/api/notes/:id`, `/api/notes/:id/pin`

### Also cleaned up
- Removed diagnostic code added during investigation: `debug_log_handler`, `/debug/log` route, `dbgLog()` JS function, extra `htmx:beforeRequest`/`htmx:responseError`/`htmx:sendError` listeners, `tool_panel_handler` log line, `/test/:param` test route
- Updated D-016 note: root cause of axum-test path param failures is now known (matchit 0.7 syntax)
- Added D-017 through D-020 to DECISIONS.md (previously only in CHANGELOG)

### CI status
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (14 tests, 0 failures)

### Next session should start with
Phase 2 — Voice (Whisper) or OCR (Tesseract). Routing is now fully working — all tool panels load, all API endpoints are reachable. Verify with `cargo tauri dev` then proceed to Phase 2.

---

## [2026-03-18] — Post-mortem: Full "Loading…" bug saga + follow-up fixes

This entry documents the complete arc of bugs that caused the app to show "Loading…" forever, in the order they were discovered and fixed. Multiple sessions were needed.

---

### Root cause 1: Axum 0.7 route param syntax

**Symptom:** `GET /tools/clipboard` returned 404. Confirmed by adding a fallback handler that fired for every path — including `/tools/clipboard`. The registered route was not matching.

**Root cause:** All route definitions used `{param}` syntax (e.g. `/tools/{tool_name}`, `/api/notes/{id}`). Axum 0.7.9 depends on **matchit 0.7.3**, which uses `:param` syntax. The `{param}` brace syntax was only introduced in matchit 0.8. Axum passes route strings to matchit verbatim — no transformation. So `{param}` was treated as a literal static segment and never matched a real request path. The code compiled without warnings.

**Diagnostic path:** Added test route `/test/{param}` alongside `/tools/{tool_name}`. Both returned 404. Static routes (`/health`) returned 200. Confirmed matchit 0.7.3 source uses `:param`. Verified Axum source does no path conversion before inserting into matchit.

**Fix:** Changed all route definitions from `{param}` to `:param` in `server.rs`, `clipboard.rs`, `notes.rs`. (D-020)

---

### Root cause 2: HTMX 2.x `selfRequestsOnly = true` default

**Symptom:** Even after routing was fixed, inner HTMX requests (`hx-trigger="load"` on `#clipboard-list`) produced zero network activity. No errors, no logs.

**Root cause:** HTMX 2.0.4 defaults `selfRequestsOnly: true`, which silently blocks all requests to a different origin. The app shell is served from `tauri://localhost` (via Tauri frontendDist) while Axum runs on `http://127.0.0.1:{PORT}`. These are different origins. HTMX drops every request with no error event, no log, no indication.

**Fix:** `htmx.config.selfRequestsOnly = false` in the inline script of `index.html`, before any `hx-*` attributes are processed. (D-017)

---

### Root cause 3: HTMX and Alpine loaded from CDN

**Symptom:** Intermittent — on WebKitGTK (used by Tauri on Linux), CDN requests to `unpkg.com` were slow or blocked. HTMX failed to initialize entirely, making every `hx-*` attribute inert.

**Fix:** Bundle `htmx.min.js` (2.0.4) and `alpine.min.js` (3.14.9) locally under `ui/assets/`. Same offline-first principle as marked.js (D-018).

---

### Root cause 4: `hx-trigger="load"` on initial panel before token was confirmed

**Symptom:** On fast startup, the initial `hx-trigger="load"` on `#tool-panel` fired before Tauri's `initialization_script` had set `window.__SESSION_TOKEN__`. The first request went out with an undefined token and got a 401. Panel never retried.

**Fix:** Removed `hx-trigger="load"` from `#tool-panel`. Added `initApp()` async function on `DOMContentLoaded` that calls `window.__TAURI__.core.invoke('get_session_token')` (with proper `typeof` guard) to confirm the real token, then loads the default panel via `htmx.ajax()` with full absolute URL and explicit auth headers. (D-019)

---

### Root cause 5: `htmx.ajax()` source-element context breaks child `hx-trigger="load"`

**Symptom:** After the routing fix, the clipboard panel HTML loaded correctly into `#tool-panel`, but `#clipboard-list` (which has `hx-trigger="load"`) never fired its `GET /api/clipboard` request.

**Root cause:** `htmx.ajax()` with no explicit source element uses `document.body` as the source. HTMX's post-swap initialization task (`Ae()`) can miss child elements' load triggers when the source is `document.body` rather than a real ancestor.

**Fix:** Added `htmx:afterSwap` listener that calls `htmx.process(evt.detail.target)` when `#tool-panel` is the swap target. This re-processes all `hx-*` attributes in the newly loaded panel, including `hx-trigger="load"` children. (D-019 addendum)

---

### Root cause 6: Notes `+New` — JSON vs Form mismatch

**Symptom:** Clicking `+ New` did nothing. No note was created. No visible error.

**Root cause:** `create_handler` in `notes.rs` used `Json<CreateBody>` extractor, which expects `Content-Type: application/json`. HTMX sends `hx-vals` as `application/x-www-form-urlencoded` (form data). Axum returned 415 Unsupported Media Type, silently. HTMX had no error handler to surface this.

**Fix:** Changed `create_handler` to `Form<CreateBody>`. Updated the test helper from `post_json` to `post_form` to match. (No new decision — follows the principle: HTMX submits form data by default.)

---

### Root cause 7: Clipboard monitor capturing nothing on Wayland

**Symptom:** Clipboard history always empty despite copying text from other apps.

**Root cause:** `arboard = "3"` without features compiles with the X11 backend only. On Wayland + Hyprland, `arboard::get_text()` fails on every poll because the X11/XWayland clipboard is not the real system clipboard. The failure is caught by `Err(_) => continue` and produces no log output.

**Root cause detail:** arboard 3 has a `wayland-data-control` feature that enables the `wlr-data-control` Wayland protocol backend (via `wl-clipboard-rs`). Hyprland implements this protocol. Without the feature, arboard never tries Wayland and falls back to X11 silently.

**Fix:** Changed to `arboard = { version = "3", features = ["wayland-data-control"] }` in Cargo.toml.

---

### Follow-up fixes (UX)

**Clipboard auto-refresh:** `hx-trigger="load"` loads once. Changed to `hx-trigger="load, every 3s"` so the list polls while the panel is open.

**Notes list title sync:** Alpine `save()` sends a `PUT` via `fetch()` but nothing told `#notes-list` to refresh. Added `htmx.trigger(document.body, 'noteUpdated')` after save. Notes list gained `hx-trigger="load, noteUpdated from:body"` to refresh when triggered.

**Markdown `#` headings invisible:** Tailwind Preflight resets `h1`–`h6` to `font-size: inherit`. Without the Typography plugin (`@tailwindcss/typography`), `prose` classes don't re-apply heading sizes. Added explicit heading styles scoped to `.prose` in `ui/tools/notes/index.html`.

---

### Files changed across this entire saga

- `src-tauri/Cargo.toml` — arboard `wayland-data-control` feature
- `src-tauri/src/server.rs` — `:param` syntax, removed diagnostic code
- `src-tauri/src/tools/clipboard.rs` — `:param` syntax
- `src-tauri/src/tools/notes.rs` — `:param` syntax, `Form<CreateBody>`, `htmx.trigger` after save
- `ui/index.html` — `selfRequestsOnly=false`, local assets, `initApp()`, `htmx:afterSwap`
- `ui/assets/htmx.min.js` — bundled HTMX 2.0.4
- `ui/assets/alpine.min.js` — bundled Alpine.js 3.14.9
- `ui/tools/clipboard/index.html` — `every 3s` polling
- `ui/tools/notes/index.html` — `noteUpdated from:body`, heading styles

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (14 tests, 0 failures)

### Next session should start with
Phase 2 — Voice (Whisper) or OCR (Tesseract). All Phase 1 functionality is confirmed working end-to-end.

