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

