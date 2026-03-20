# Eleutheria Telos — Changelog

This file is the project's memory between sessions. It is updated at the end of every work session by Claude Code. Before starting any session, read the most recent entry.

Format per entry:
- **Date** — what was completed, what changed, what was decided, what's next

---

## [2026-03-19] — UI Design System Overhaul (DESIGN.md / Ethereal Command Center)

### Completed

**Design direction:** `DESIGN.md` added to repo (Stitch/Google reference). Direction: "Ethereal Command Center" — lavender accent, mint glass, Space Grotesk editorial titles, pill buttons, tonal layering, no hard borders. Applied as hybrid (light + dark), not literal.

**Themes (`ui/assets/themes/*.css`)**
- Dark: accent shifted to lavender `#8b74d4`, `--accent-dim` added, `--shadow-accent` tinted glow token
- Light: palette aligned to DESIGN.md — base `#f0f2f8`, mint glass `rgba(204,250,245,0.55)`, lavender `#7049b3` primary, lavender-tinted shadows
- All themes: `--radius-lg` 14→20px, `--radius-xl` 18→28px, `--radius-pill: 9999px` added, `--shadow-accent` added

**Typography (`ui/assets/fonts/`, `ui/assets/base.css`)**
- Space Grotesk variable font downloaded and bundled offline (`space-grotesk-variable.woff2`)
- `.panel-title` uses Space Grotesk, `letter-spacing: -0.02em` — editorial authority per DESIGN.md

**Buttons (`base.css`)**
- Pill shape (`border-radius: var(--radius-pill)`) on all `.btn` variants
- `.btn-primary`: lavender gradient 135° + `--shadow-accent` glow, Telos Glow inset on `:active`
- `.btn-secondary`: `accent-subtle` background, no border (mint-style per DESIGN.md), accent color text
- `.btn-sm/.btn-lg` padding adjusted for pill shape

**Cards (`base.css`)**
- Padding increased to `16px 18px`
- Hover: `--shadow-accent` tinted shadow instead of flat shadow
- `.card-interactive:active`: inset accent glow (Telos Glow)

**Inputs (`base.css`)**
- `border-radius: var(--radius-lg)` (was `--radius-md`), padding `8px 14px`

**Panel transitions (`base.css`, `ui/index.html`)**
- `@keyframes panel-enter`: opacity 0→1 + `translateY(6px→0)`, 160ms `cubic-bezier(0.22,1,0.36,1)`
- Applied via `htmx:afterSwap` on `#tool-panel` — every panel navigation now fades in

**Ideas logged (`IDEAS.md`)**
- Search: recent searches history below search bar
- Clipboard: privacy blur mode
- Clipboard: rich content preview (images/audio)
- OCR/Translation: auto language detection
- Resizable panel dividers (Notes, Quick Actions)
- Keybindings section in Settings

### Next
Continue UI improvements — open `cargo tauri dev`, review every panel visually, identify what still feels "toy-like" now that the design system base is updated. Priority panels to audit: Notes editor, Clipboard list, Models, Settings form layout.

---

## [2026-03-19] — Phase 4.6 Complete

### Completed

- **Item 1 — Translation backend fix** (`scripts/translate.py`, `install_argos_package.py`, `uninstall_argos_package.py`, `requirements.txt`) — argostranslate replaced with ctranslate2 + sentencepiece. Eliminates Python 3.14 incompatibility and ~3GB dependency footprint. See D-036.
- **Item 2 — Contextual pipeline CTA** (`src-tauri/src/tools/ocr.rs`, `voice.rs`, `ui/tools/quick-actions/index.html`) — "Create pipeline from this" button on OCR and Voice result cards; navigates to Quick Actions and pre-selects the correct trigger via `window.__qaPreTrigger`.
- **Item 3 — Pipeline templates** (`src-tauri/src/tools/quick_actions.rs`, `ui/tools/quick-actions/index.html`) — 5 pre-built templates shown in right panel when no pipeline is selected. "Use this" creates pipeline + steps and opens editor in one click via HTMX OOB swap.
- **Item 4 — Problem-first empty states** (`clipboard.rs`, `notes.rs`, `translate.rs`, `quick_actions.rs`, `ui/tools/notes/index.html`, `ui/tools/search/index.html`) — all generic "nothing here" messages replaced with problem-framing copy.
- **Item 5 — First community plugin** — deferred to backlog. Plugin system already stress-tested in Phase 4; Obsidian Send and GitHub Issues creator logged in IDEAS.md.

### Also this session
- Command palette footer: added `Ctrl K` kbd hint (discoverability)
- Search panel: added "Tip: press Ctrl K to open quick search from anywhere" below search bar
- IDEAS.md: clipboard blur mode, rich content preview (images/audio), OCR auto language detection, translation auto source detection, keybindings settings section

### Phase 4.6 status
**COMPLETE.** Moving to Phase 5 — Monetization + Distribution.

### Next
Phase 5 — starting with **license key system** (Gumroad integration, asymmetric key verification, no server required) or **onboarding flow** (first-run wizard). Recommend onboarding first — it surfaces the "which tools do you want?" question before monetization makes sense.

---

## [2026-03-19] — Phase 4.6 Item 1: Translation backend — argostranslate → ctranslate2

### Completed

**Translation backend rewrite (D-036)**
- `scripts/translate.py` — rewritten: loads `ctranslate2.Translator` + two `SentencePieceProcessor`s from `~/.local/share/eleutheria-telos/models/translate/{from}-{to}/`; tokenizes with `source.spm`, translates, detokenizes with `target.spm`; zero argostranslate import
- `scripts/install_argos_package.py` — rewritten: fetches Argos model index JSON from GitHub via `urllib`, finds matching package, downloads `.argosmodel` ZIP (which is a standard ZIP containing CT2 model files), extracts `model.bin`, `source.spm`, `target.spm`, `config.json` to the local models dir; zero argostranslate import
- `scripts/uninstall_argos_package.py` — rewritten: `shutil.rmtree` on the model directory; ~30 lines down from the argostranslate version
- `scripts/requirements.txt` — `argostranslate>=1.11.0` replaced with `ctranslate2>=4.7.1` + `sentencepiece>=0.2.1`
- Axum routes, UI, and Rust subprocess invocation unchanged — same CLI interface, same model storage path

**DECISIONS.md**
- D-036 updated: marked implemented, documents the three rewritten scripts
- D-025 updated: no longer references argostranslate's Python API; describes current urllib/zipfile approach

**ROADMAP.md** — Phase 4.6 translation checkbox marked `[x]`

### Next
Phase 4.6 item 2: **Contextual pipeline CTA** — "Create pipeline from this" button on OCR and Voice result cards, pre-filling the Quick Actions builder with the correct trigger

---

## [2026-03-19] — Phase 4.5 Complete: Playwright Review + OCR card fix

### Completed

**Playwright visual review infrastructure (`playwright-review/`)**
- `playwright-review/package.json` + `playwright.config.js` — Playwright 1.58 setup; serves `ui/` via Python HTTP server on port 9191; injects real session token from `~/.local/share/eleutheria-telos/server.json` via `addInitScript` before page scripts run
- `playwright-review/tests/visual.spec.js` — screenshots all 13 panels + command palette + sidebar states
- `.gitignore` — added Playwright node_modules, screenshots, test-results, playwright-report exclusions

**Review findings — all panels signed off:**
- Clipboard, Notes, Voice, Translate, Search, Screen Recorder, Audio Recorder, Photo Editor, Video Processor, Quick Actions, Models, Settings, Command Palette — all pass ✓

**Fix: OCR panel controls wrapped in `.card`**
- `ui/tools/ocr/index.html` — language select + action buttons now inside a `.card` div (consistent with Voice, Screen Recorder, Audio Recorder which all use `.card` for their control areas)
- Also moved indicators inside the card for a cleaner layout

### Phase 4.5 status
**COMPLETE.** Every panel reviewed and signed off. Moving to Phase 4.6 — Cohesion.

### Deferred to Phase 4.6
- Notes "Select a note to edit" → problem-first empty state
- Models section heading still says "TRANSLATION (ARGOS)" → update when ctranslate2 lands (D-036)

### Next
Phase 4.6 — Cohesion, starting with **translation backend fix**: replace argostranslate with ctranslate2 + Opus-MT models (D-036)

---

## [2026-03-19] — Phase 4.5 Step 2: Panel Polish, Emoji Removal, Drag-to-Resize

### Completed

**Sidebar improvements:**
- `ui/assets/base.css` — pinned icon height reduced from 36px to 28px; sidebar-scroll gets `overflow-x: hidden` to prevent horizontal scroll on pinned 3×3 grid
- `ui/index.html` — drag-to-resize: now shows live width during `mousemove` (disables CSS transition while dragging) then snaps to 56px or 200px on `mouseup` (threshold: finalWidth < 128px = collapse)
- `src-tauri/src/server.rs` — `GET /api/settings/ui` SQL query fixed to include `pinned` and `sidebar_collapsed` keys (bug: were always returning defaults because query only fetched `theme/glass/font`)

**Emoji removal (all panels):**
- `ui/tools/translate/index.html` — removed 🌐 from header, replaced with `panel-title/panel-subtitle`
- `ui/tools/quick-actions/index.html` — removed ⚡ from header, full redesign with design system
- `src-tauri/src/tools/quick_actions.rs` — removed ⚡/📷/🎙/📋 from `trigger_label()`; removed 🌐/📋/📝/⚙️ from `tool_icon()` (replaced with Lucide HTML icon strings); removed emojis from trigger select options and step select options in `render_editor()`
- `src-tauri/src/tools/translate.rs` — removed 📦 from empty-state "No language packs installed" (now uses `empty-state` CSS class + Lucide icon)

**Button redesign (Quick Actions):**
- `src-tauri/src/tools/quick_actions.rs` — all Tailwind `bg-blue-700/bg-gray-700/bg-gray-800` replaced with `btn btn-primary/secondary/ghost/danger btn-sm` design system classes; inputs use `.input` class; pipeline list items and step cards use CSS custom properties for color
- `ui/tools/quick-actions/index.html` — rewritten: New Pipeline / Create buttons use `btn btn-primary`; select options use `.input`; two-column layout uses inline CSS vars

**Panel redesigns (header + button polish):**
- `ui/tools/screen-recorder/index.html` — rewritten: `panel-title/panel-subtitle`, `.card` wrapper, Start/Stop use `btn btn-primary/btn-danger`, select uses `.input`, Tailwind color classes replaced with CSS vars
- `ui/tools/audio-recorder/index.html` — rewritten: same design system treatment
- `ui/tools/video-processor/index.html` — rewritten: operation tab buttons use Alpine `:class` binding to toggle `btn-primary/btn-secondary`, inputs use `.input`, submit uses `btn btn-primary`
- `ui/tools/photo-editor/index.html` — rewritten: toolbar integrated into `panel-header`, Brush/Eraser active state via `:class="tool==='brush' ? 'btn-primary' : 'btn-secondary'"`, Remove BG / Export use `.btn-disabled` when not available; layer strip buttons use `btn-primary` for active layer

**base.css additions:**
- `.btn-disabled` added alongside `.btn:disabled` (same opacity:0.45/pointer-events:none rule)

### Bug fixes
- Pinned items were never restored on app restart — SQL query was missing `pinned` key
- Sidebar collapsed state was never restored — SQL query was missing `sidebar_collapsed` key
- Translate empty-state emoji `📦` was rendered by Rust server-side, not the static HTML

### Next
- Voice panel polish (still uses old gray Tailwind classes)
- Notes, Clipboard, OCR, Search, Models panels — design system pass
- Phase 4.5 full completion: all panels consistent

---

## [2026-03-19] — Phase 4.5 Step 1: App Shell — Design System

### Completed

**Assets (bundled locally, offline-first)**
- `ui/assets/fonts/inter-variable.woff2` + `inter-variable-italic.woff2` — Inter variable font (latin, @fontsource-variable/inter 5.2.8)
- `ui/assets/lucide.min.js` — Lucide icons UMD bundle v0.577.0 (replaces all emojis)
- `ui/assets/themes/dark.css` (default) — soft dark: `#0f1117` base, indigo-periwinkle accent `#6d83f2`
- `ui/assets/themes/light.css`
- `ui/assets/themes/catppuccin-mocha.css` — Mauve accent `#cba6f7`
- `ui/assets/themes/catppuccin-latte.css`
- `ui/assets/themes/tokyo-night.css` — Blue accent `#7aa2f7`
- `ui/assets/base.css` — full component design system: fonts, scrollbar, sidebar, nav-item, btn-primary/secondary/ghost/danger, input, card, card-glass, badge, empty-state, skeleton, prose, HTMX indicator

**Theme system CSS variables (per-theme):**
`--bg-base`, `--bg-surface`, `--bg-elevated`, `--bg-overlay`, `--text-primary/secondary/muted`, `--accent`, `--accent-subtle/hover`, `--border`, `--border-focus`, `--shadow/shadow-lg`, `--glass-bg/blur/border`, `--destructive/success/warning` (+ subtle variants), `--radius-sm/md/lg/xl`

**Glassmorphism system:**
- Default: sidebar + cards use `backdrop-filter: blur(20px)` + semi-transparent fill
- Disabled: `html.no-glass` class → opaque fills, no blur

**App Shell (`ui/index.html`) — full rewrite:**
- Loads Inter, Lucide, theme CSS, base.css; Tailwind CDN kept for layout utilities (preflight disabled — base.css owns resets)
- `applyTheme(name)` + `applyGlass(enabled)` functions exposed on `window`
- `initApp()` fetches `/api/settings/ui` on startup to apply saved theme/glass before first render
- Lucide `createIcons()` called on DOMContentLoaded + on every `htmx:afterSwap` into `#tool-panel`
- Plugin sidebar entries use `<i data-lucide="puzzle">` instead of emojis
- Sidebar responsive layout owned by `base.css` media queries (no Tailwind responsive classes on `#sidebar`)
- Three sidebar groups: **Tools** (Clipboard, Notes, Voice, OCR, Translate, Search) / **Media** (Screen Rec, Audio Rec, Photo Edit, Video, Quick Actions) / **Plugins** (dynamic) + bottom: Models, Settings
- Pill-style active nav item: `--accent-subtle` background + `--accent` text
- "ELEUTHERIA" → `logo-dot` (8px accent circle) + "Eleutheria" label
- Command palette: glassmorphism box, Lucide search icon, styled input

**Backend (`src-tauri/src/server.rs`):**
- `GET /api/settings/ui` — returns `{theme, glass, font}` with defaults; used by `initApp()`
- `POST /api/settings/ui` — upserts theme/glass/font keys in settings table

**Settings panel (`ui/tools/settings/index.html`) — rewritten:**
- Theme dropdown (5 themes), glassmorphism toggle switch, font selector (Inter / system)
- Changes applied instantly to the shell + persisted via `/api/settings/ui`
- App info section (version, server port, phase)

### Architecture notes
- `base.css` is the single source of truth for all component visual styles
- Theme files only define CSS custom properties — zero layout/component rules
- Responsive sidebar visibility in `base.css` @media queries, not Tailwind
- `applyTheme()` and `applyGlass()` are global window functions so the Settings panel can call them after a fetch()

### Next: Priority 2 — Clipboard History panel polish

---

## [2026-03-19] — Phase 4.5 planning scaffolding

### Completed

- `ROADMAP.md` — added Phase 4.5 (UI Polish) with mandatory workflow: references → questions → execution → Playwright review → user feedback → iteration
- `UI_BRIEF.md` (new) — template document that must be filled via Q&A before any UI implementation begins; covers aesthetic direction, references, pain points, palette, typography, density, components, sidebar, empty states, priority order

### Next session should start with

Phase 4.5 — UI Polish. User must open the app and take screenshots of current state, plus gather 1-2 reference apps they find visually inspiring. Then Claude asks all questions to complete `UI_BRIEF.md` before touching any code.

---

## [2026-03-19] — Phase 4.7: Quick Actions (visual pipeline builder)

### Completed

- `src-tauri/migrations/004_phase4_pipelines.sql` (new) — `pipelines` and `pipeline_steps` tables; `pipeline_steps` has `ON DELETE CASCADE` referencing `pipelines(id)`
- `src-tauri/src/tools/quick_actions.rs` (new) — full pipeline CRUD + execution engine:
  - HTML renderers: `render_pipeline_list()`, `render_steps()`, `render_editor()`
  - Routes: `GET/POST /api/pipelines`, `GET /api/pipelines/:id/editor`, `PUT /api/pipelines/:id`, `DELETE /api/pipelines/:id`, `POST /api/pipelines/:id/steps`, `DELETE /api/pipelines/:id/steps/:step_id`, `POST /api/pipelines/:id/steps/:step_id/move`, `POST /api/pipelines/:id/run`
  - Step types: `translate` (calls `scripts/translate.py`), `copy_clipboard` (arboard), `save_note` (SQLite insert)
  - `start_pipeline_engine()` — background task subscribing to Event Bus, executes matching enabled pipelines when `OcrCompleted`, `TranscriptionCompleted`, or `ClipboardChanged` events fire
- `src-tauri/src/tools/ocr.rs` — emits `Event::OcrCompleted` after successful Tesseract run
- `src-tauri/src/tools/voice.rs` — emits `Event::TranscriptionCompleted` after successful Whisper transcription
- `src-tauri/src/tools/mod.rs` — added `pub mod quick_actions;`
- `src-tauri/src/server.rs` — merged `quick_actions::router()`
- `src-tauri/src/lib.rs` — spawned `start_pipeline_engine` as background tokio task
- `ui/tools/quick-actions/index.html` (new) — two-column layout: pipeline list + step editor
- `ui/index.html` — added ⚡ Quick Actions entry to desktop and tablet sidebars; added `overflow-y-auto` to sidebar `<ul>` elements

### Bug fixes

- **Trigger select not saving** — `<option value='{"type":"OcrCompleted"}'>` inner quotes terminated the HTML attribute early; browser sent only `{` to server. Fixed by applying `html_escape()` in `render_editor()` and `&quot;` entities in the static create form.
- **Quick Actions not visible in sidebar** — sidebar `<ul class="flex-1">` without `overflow-y-auto` silently clipped items below viewport height. Fixed by adding `overflow-y-auto`.

### Future ideas added to IDEAS.md
- Keybinds per pipeline (manual trigger via hotkey)
- Opt-in/opt-out for auto-triggered pipeline execution (toast prompt before running)
- Full visual canvas editor with drag-and-drop boxes, arrow connectors, cycles, conditions

### Next session should start with
Phase 5 — Monetization + Distribution (license key, onboarding flow, auto-updater, installers). Or confirm with user whether to address any remaining Phase 4 gaps first.

---

## [2026-03-19] — Phase 4.6: Plugin developer documentation

### Completed

- `plugins/README.md` (new) — full plugin developer guide covering:
  - Manifest schema (all fields, `sidebar` config, `routes` permission declarations)
  - Runtimes: `python`, `node`, `binary` — command used for each
  - Environment variables injected by host (`ELEUTHERIA_APP_PORT`, `ELEUTHERIA_TOKEN`, `ELEUTHERIA_PLUGIN_ID`, `ELEUTHERIA_PLUGIN_PORT`) with Python + Node.js code examples
  - Routing: how `/plugins/<id>/subpath` maps to `/subpath` at the plugin server; permission enforcement
  - Calling the host API: auth pattern, available endpoints table
  - HTMX UI conventions: fragment structure, absolute paths, Tailwind + Alpine already loaded
  - HTML escaping: Python `html.escape` and JS helper
  - Graceful shutdown: `SIGTERM` handling for Python and Node.js
  - Local development: how to run standalone with env vars set manually, curl test examples
  - Reference implementations table (hello-python, hello-node)
  - New plugin checklist

### Next session should start with
Phase 4.7: Quick Actions (basic) — global keyboard shortcut to trigger a quick paste/note/search action without opening the full window.

---

## [2026-03-19] — Phase 4 complete: Plugin system + sidebar + bug fixes

### Completed

**Phase 4.3 – Plugin system bug fixes (this session)**

- `src-tauri/src/plugins.rs` — fixed raw string literals: `r#"..."#` → `r##"..."##` (the `"#` in `hx-target="#tool-panel"` was terminating the raw string causing a parse error); removed `axum::extract::Path` extractor from `plugin_proxy_handler`, now extracts `plugin_id` from `req.uri().path()` directly (fixes "Wrong number of path arguments" 500 on `/plugins/:id/*path`); fixed permission check logic (was checking declared routes against the URL prefix; now checks request path against each declared route)
- `src-tauri/src/server.rs` — added `find_free_port_from(start: u16) -> u16`; `find_free_port_sync()` now delegates to it; fixed plugin port collision (all plugins were allocated the same port because each call to `find_free_port_sync()` scanned from `DEFAULT_PORT` before the server had bound)
- `src-tauri/src/plugin_loader.rs` — fixed port allocation: tracks `next_port = app_port + 1`, increments `next_port = plugin_port + 1` after each allocation via `find_free_port_from(next_port)`
- `src-tauri/Cargo.toml` — added `default-run = "app"` to `[package]` (fixes "could not determine which binary to run" when two `[[bin]]` entries exist)
- `src-tauri/src/api.rs` — added `list_sidebar_plugins` Tauri command (returns sorted list of plugins with sidebar entries)
- `src-tauri/src/lib.rs` — `initialization_script` now injects `window.__SIDEBAR_PLUGINS__` (sorted JSON array of plugin sidebar entries) before any page script runs

**Plugin sidebar in UI**

- `ui/index.html` — added plugin sidebar loading to `initApp()` (reads `window.__SIDEBAR_PLUGINS__`, creates `<li>` elements via `document.createElement`, calls `htmx.process()` on each); added `<ul id="plugin-sidebar-desktop">` after the main tool list; added `<ul id="plugin-sidebar-tablet">` in the tablet icon sidebar — both populated at startup from the injected plugin list

**Note:** `ui/shell.html` is NOT loaded by the app (Tauri loads `ui/index.html` via `WebviewUrl::App("index.html")`). Shell.html is kept as a standalone browser-preview artifact only.

### Verified working (end-to-end)
- 🐍 Hello Python and 🟩 Hello Node appear in the sidebar below the main tools
- Echo form works: typing a message and clicking "Echo" returns the message (both plugins)
- "Fetch plugin info" shows `host_reachable: true` and correct plugin metadata (both plugins)
- Plugins run on separate ports (47854, 47863 in latest run) — no port collision
- Plugin proxy correctly routes `/plugins/hello-python/api/echo` → Python process → response back to WebView

### Bug fixes summary
| Bug | Root cause | Fix |
|-----|-----------|-----|
| All plugins same port | `find_free_port_sync()` rescans from DEFAULT_PORT each call | `find_free_port_from(next_port)` with counter |
| Proxy 500 on subpaths | `Path<String>` extractor doesn't work with 2-segment routes | Extract from `req.uri().path()` directly |
| Permission check never 403 | Logic inverted (routes checked against request prefix) | Check request path against each declared route |
| `host_reachable: false` | Plugins called `/api/clipboard` (returns HTML, not JSON) | Call `/health` (returns JSON) |
| Sidebar plugins not visible | All edits were applied to `shell.html`; app loads `index.html` | Apply changes to `index.html` |
| `cargo tauri dev` binary error | Two `[[bin]]` entries, no `default-run` | Added `default-run = "app"` to `Cargo.toml` |

### Files changed this session
- `src-tauri/Cargo.toml` — `default-run = "app"`
- `src-tauri/src/api.rs` — `list_sidebar_plugins` command
- `src-tauri/src/lib.rs` — `__SIDEBAR_PLUGINS__` injection + `list_sidebar_plugins` in invoke_handler
- `src-tauri/src/server.rs` — `find_free_port_from(start)`
- `src-tauri/src/plugin_loader.rs` — port counter fix
- `src-tauri/src/plugins.rs` — proxy handler fix + permission check fix + raw string fix
- `ui/index.html` — plugin sidebar loading in `initApp()` + sidebar `<ul>` containers
- `ui/shell.html` — same changes (for browser-preview parity, but not loaded by app)

### CI status
- Tests pass locally (all prior tests still green)
- `cargo fmt --check` ✓ (no new formatting issues)

### Next session should start with
Phase 4.6: Plugin developer documentation — `plugins/README.md` covering manifest schema, env vars, routing/permissions, HTMX UI conventions, and local dev workflow.

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

## [2026-03-19] — Phase 4.5: Example plugin (Node.js)

### Completed

**`plugins/hello-node/` (new directory)**

- `manifest.json` — full plugin manifest:
  - `id`: `hello-node`, `runtime`: `node`, `entry`: `main.js`
  - `routes`: `["/plugins/hello-node"]`
  - `sidebar`: `{ show: true, label: "Hello Node", order: 101, icon: "🟩" }`

- `main.js` — Node.js stdlib-only HTTP server (no npm packages):
  - Uses `node:http`, `node:url`, `node:querystring`
  - `GET /` or `GET /plugins/hello-node` → HTMX UI fragment
  - `GET /api/hello` → HTML `<pre>` with JSON info (id, port, node version, host reachability)
  - `POST /api/echo` → echoes `message` form field back as HTML
  - Optional host callback via `http.request` with Bearer auth
  - Graceful shutdown on `SIGTERM`

**Verified smoke test (standalone):**
- `GET /` → HTMX fragment ✓
- `GET /api/hello` → JSON with `host_reachable: false`, `node_version: v22.20.0` ✓
- `POST /api/echo message=Hola+Node` → `<p>Plugin echoes: Hola Node</p>` ✓
- `GET /unknown` → `{"error":"not found"}` ✓

### Next session should start with
Phase 4.6: Plugin developer documentation — `plugins/README.md` covering manifest schema, available env vars, routing, permissions, HTMX UI conventions, and how to run plugins in dev.

---

## [2026-03-19] — Phase 4.4: Example plugin (Python)

### Completed

**`plugins/hello-python/` (new directory)**

- `manifest.json` — full plugin manifest:
  - `id`: `hello-python`, `runtime`: `python`, `entry`: `main.py`
  - `routes`: `["/plugins/hello-python"]` (permission declaration for proxy)
  - `sidebar`: `{ show: true, label: "Hello Python", order: 100, icon: "🐍" }`

- `main.py` — pure stdlib HTTP server (no third-party packages):
  - Reads `ELEUTHERIA_APP_PORT`, `ELEUTHERIA_TOKEN`, `ELEUTHERIA_PLUGIN_ID`, `ELEUTHERIA_PLUGIN_PORT` from env
  - `GET /` or `GET /plugins/hello-python` → HTMX UI fragment (echo form + info panel)
  - `GET /api/hello` or `GET /plugins/hello-python/api/hello` → JSON plugin info (id, port, python version, host reachability)
  - `POST /api/echo` or `POST /plugins/hello-python/api/echo` → echoes `message` form field back as HTML
  - Optional host callback: calls `GET /api/clipboard?limit=1` via Bearer auth to verify host connectivity
  - Graceful shutdown on `KeyboardInterrupt`

**Verified smoke test (standalone, no host running):**
- `GET /` → correct HTMX fragment ✓
- `GET /api/hello` → JSON with `host_reachable: false` (expected — host not running) ✓
- `POST /api/echo message=Hola+mundo` → `<p>Plugin echoes: Hola mundo</p>` ✓
- `GET /unknown` → `{"error": "not found"}` ✓

### Next session should start with
Phase 4.5: Example plugin (Node.js) — same structure as hello-python but Node runtime, using only Node stdlib (`http` module).

---

## [2026-03-19] — Phase 4.3: Plugin system — full implementation

### Completed

**Plugin process management (`plugin_loader.rs`)**
- Added `#[derive(Clone)]` to `PluginManifest` and `SidebarConfig`
- Added `PluginInfo { manifest: PluginManifest, port: u16 }` struct (Clone)
- Added `PluginRegistry = Arc<std::sync::Mutex<HashMap<String, PluginInfo>>>` type alias
- Added `start_plugins(manifests, app_port, token) -> (PluginRegistry, Vec<std::process::Child>)`:
  - Allocates a free port per plugin via `find_free_port_sync()`
  - Spawns each plugin as a subprocess via `std::process::Command` (python3/node/binary runtimes)
  - Injects env vars: `ELEUTHERIA_APP_PORT`, `ELEUTHERIA_TOKEN`, `ELEUTHERIA_PLUGIN_ID`, `ELEUTHERIA_PLUGIN_PORT`
  - Returns populated registry + child handles (held alive to avoid orphaning)

**Plugin proxy + sidebar (`src-tauri/src/plugins.rs` — new file)**
- `GET /api/plugins` — JSON list of all running plugins
- `GET /api/plugins/sidebar[?layout=tablet]` — HTMX `<li>` fragments sorted by `sidebar.order`, icon-only when `layout=tablet`
- `* /plugins/:plugin_id` and `* /plugins/:plugin_id/*path` — full reverse proxy:
  1. 404 if plugin not in registry
  2. 403 if route not declared in `manifest.routes`
  3. Strips `/plugins/{id}` prefix, builds `http://127.0.0.1:{port}/{subpath}` target
  4. Forwards all non-hop-by-hop headers + `x-session-token` + `x-plugin-id`
  5. Returns plugin response (status + headers + body) or 502 if unreachable

**AppState extended (`server.rs`, `lib.rs`)**
- Added `plugin_registry: PluginRegistry` and `plugin_processes: Arc<std::sync::Mutex<Vec<std::process::Child>>>` to `AppState`
- `lib.rs`: calls `plugin_loader::start_plugins()` at startup, stores registry and child handles in state
- `server.rs`: registers `plugins::router()` in `build_router()`

**Test constructors updated**
- `src-tauri/src/tools/clipboard.rs`, `notes.rs`, `search.rs`, `translate.rs` — added `plugin_registry` and `plugin_processes` fields to all `make_test_state()` functions

**Shell HTMX plugin sidebar (`ui/shell.html`)**
- Desktop sidebar: added `<ul id="plugin-sidebar-desktop">` after the main `<ul>`, loads via `hx-get="/api/plugins/sidebar"` on `load`
- Tablet sidebar: added `<ul id="plugin-sidebar-tablet">`, loads via `hx-get="/api/plugins/sidebar?layout=tablet"` on `load`
- Plugin entries appear below built-in tools, sorted by `sidebar.order` from manifest

**Bug fix**
- `plugins.rs`: raw string literals for HTML with `hx-target="#tool-panel"` changed from `r#"..."#` to `r##"..."##` — the `"#` sequence inside the HTML terminated the raw string early causing a parse error

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (19 tests, 0 failures)

### Next session should start with
Phase 4.4: Example plugin (Python) — a reference plugin implementation with `manifest.json`, HTTP server on assigned port, and at least one sidebar entry and one API route.

---

## [2026-03-19] — Phase 4.2: MCP SSE transport

### Completed

**MCP SSE transport (`GET /mcp`, `POST /mcp?sessionId=...`)**
- `src-tauri/src/mcp.rs` — replaced 501 stubs with full SSE implementation:
  - `mcp_sse_handler` (`GET /mcp`): creates a session ID, allocates a buffered mpsc channel (cap 64), pre-fills the `endpoint` event, stores the sender in `AppState::mcp_sessions`, returns an SSE stream via `ReceiverStream`
  - `mcp_post_handler` (`POST /mcp?sessionId=...`): looks up the session, spawns a background task that calls `process_sse_message()`, returns `202 Accepted` immediately
  - `process_sse_message()`: handles `initialize`, `initialized` (notification, no response), `ping`, `tools/list`, `tools/call`, and unknown-method errors
  - `call_tool_sse()`: dispatches all 11 tools via loopback HTTP (`http://127.0.0.1:{port}/api/mcp/...`) using `SseHttpClient` (mirrors `McpClient` in stdio binary)
  - `SseHttpClient`: struct wrapping `reqwest::Client` with bearer auth; `get_query`, `post_form`, `put_form`, `delete` methods
  - `mcp_tools()`: shared tool manifest (11 tools with JSON Schema) — also used by `tools/list` in `process_sse_message`
- `src-tauri/src/server.rs`:
  - Added `McpSessions = Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>` type alias
  - Added `mcp_sessions: McpSessions` field to `AppState`
- `src-tauri/src/lib.rs`: initializes `mcp_sessions: Arc::new(Mutex::new(HashMap::new()))` at startup
- `src-tauri/Cargo.toml`: added `tokio-stream = { version = "0.1" }` for `ReceiverStream`
- 4 test `make_test_state()` constructors updated (`clipboard.rs`, `notes.rs`, `search.rs`, `translate.rs`)

**Protocol:**
- `GET /mcp` → SSE stream; first event is `event: endpoint\ndata: /mcp?sessionId={uuid}`
- Client POSTs JSON-RPC to `POST /mcp?sessionId={uuid}` (with Bearer token)
- Responses arrive as `event: message\ndata: {json-rpc-response}` on the SSE stream
- Notifications (e.g. `initialized`) → no response event sent

### Architecture
- Session map keyed by UUID; sender cloned from map and moved into background task — receiver lives in the SSE stream
- Tool calls make loopback HTTP requests to the same Axum process rather than re-implementing handlers inline (single source of truth, same auth path)
- `SseHttpClient` is defined locally in `mcp.rs` (not shared with stdio binary) to keep binary free of lib dependencies (D-033)

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (19 tests, 0 failures)

### Next session should start with
Phase 4.3: Plugin system — full implementation. Plugins run their own process, routes are proxied through Axum, permissions are enforced, and a sidebar entry is added per plugin. Start with `plugin_loader.rs` (extend with process management) and `server.rs` (dynamic route proxying).

---

## [2026-03-19] — Phase 4.1: MCP stdio transport

### Completed

**MCP JSON API (Axum — `/api/mcp/...`)**
- `src-tauri/src/mcp.rs` — full rewrite (was Phase 0 stubs):
  - `GET /api/mcp/clipboard` — list/search clipboard history, returns JSON
  - `POST /api/mcp/clipboard/copy` — write to clipboard (arboard)
  - `GET /api/mcp/notes` — list notes; FTS5 MATCH search via `?q=`
  - `POST /api/mcp/notes` — create note (form: title, content, tags)
  - `PUT /api/mcp/notes/:id` — partial update (dynamic SET, optional fields)
  - `DELETE /api/mcp/notes/:id` — delete note
  - `POST /api/mcp/ocr/file` — tesseract OCR from file path
  - `POST /api/mcp/voice/transcribe` — Whisper transcription from file path
  - `POST /api/mcp/translate` — translate via scripts/translate.py
  - `POST /api/mcp/video/process` — ffmpeg (trim/extract_audio/compress/resize)
  - `POST /api/mcp/photo/rembg` — rembg_remove.py, saves PNG to ~/Pictures/Eleutheria/
  - SSE stubs `/mcp` (GET/POST) kept as NOT_IMPLEMENTED for Phase 4.2
  - `pub fn router()` registered in `server.rs`
- `src-tauri/src/server.rs` — added `.merge(mcp::router())`

**MCP stdio binary**
- `src-tauri/src/bin/mcp_stdio.rs` — new: implements JSON-RPC 2.0 over stdin/stdout
  - Reads `~/.local/share/eleutheria-telos/server.json` (port + token written at app startup)
  - Handles: `initialize`, `initialized`, `tools/list`, `tools/call`, `ping`
  - 11 tools defined with full JSON Schema `inputSchema`
  - HTTP client (`McpClient`) proxies all tool calls to Axum via reqwest
- `src-tauri/src/lib.rs` — writes `server.json` at startup via `write_server_info(port, token)`

**Cargo.toml changes**
- `[[bin]]` entry for `eleutheria-mcp` (path: `src/bin/mcp_stdio.rs`)
- `tokio` — added `io-std` feature (async stdin/stdout for MCP binary)
- `reqwest` — added `json` feature (`Response::json()` for HTTP client in MCP binary)

### Architecture
- `reqwest` in `[dependencies]` is shared across all targets (lib + both binaries) — no separate deps needed (D-033)
- MCP binary is standalone: does NOT import `app_lib`. It only needs `serde_json`, `tokio`, `reqwest`
- JSON API routes are behind the same Bearer auth middleware as all other routes
- `photo_rembg` MCP route accepts a file path instead of multipart upload — consistent with video_processor (D-030), avoids base64-encoding large files over localhost
- Tags in MCP routes use comma-separated string input → stored as JSON array in DB

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (19 tests, 0 failures)

### Usage
Configure in Claude Desktop / Cursor:
```json
{
  "mcpServers": {
    "eleutheria": {
      "command": "/path/to/target/debug/eleutheria-mcp"
    }
  }
}
```

### Next session should start with
Phase 4.2: MCP server — SSE transport. Replace the `/mcp` 501 stubs with a real SSE implementation (Server-Sent Events stream for AI agent clients). Then Phase 4.3: Plugin system full implementation.

---

## [2026-03-19] — Phase 3 bugfix: Video Processor encoder

### Fixed
- **compress + resize failing** — `h264_vaapi` unavailable at runtime: AMD GPU open-source mesa driver has no H.264 VAAPI entrypoints (`vainfo` empty; error: `No usable encoding entrypoint found for profile VAProfileH264High`).
- Switched both operations to `libx264 -crf {value} -preset fast` (confirmed available via `ffmpeg -encoders` on Nobara's build). CRF range 18–40 matches the existing QP slider — no UX change needed beyond relabeling.
- UI label updated: "QP" → "CRF", description updated from h264_vaapi to libx264.
- D-032 added to DECISIONS.md documenting the switch and the vaapi failure root cause.

### CI status
- `cargo fmt --check` ✓ · `cargo clippy -- -D warnings` ✓ · `cargo test` ✓ (19 tests)

---

## [2026-03-19] — Phase 3 Step 4: Video Processor (Phase 3 complete)

### Completed

**Backend (Rust)**
- `src-tauri/src/tools/video_processor.rs` — 1 route handler:
  - `POST /api/video/process` — form-urlencoded body; dispatches to ffmpeg based on `operation` field
  - **Trim**: `ffmpeg -i input -ss start -to end -c copy output.mp4` (stream copy, lossless, near-instant)
  - **Extract audio**: `ffmpeg -i input -vn -c:a {libmp3lame|pcm_s16le|flac} output.{mp3|wav|flac}`
  - **Compress**: `ffmpeg -vaapi_device /dev/dri/renderD128 -i input -vf 'format=nv12,hwupload' -c:v h264_vaapi -qp {18–40} output.mp4` (optional downscale)
  - **Resize**: same pipeline with `scale=-2:{height},format=nv12,hwupload`; preserves aspect ratio
- `src-tauri/src/tools/mod.rs` — registered `video_processor` module
- `src-tauri/src/server.rs` — imported `video_processor`, merged `video_processor::router()`

**Frontend**
- `ui/tools/video-processor/index.html` — operation tab selector (Trim/Extract Audio/Compress/Resize), conditional field panels per operation, Alpine QP slider with quality label, `hx-indicator` for long-running ffmpeg jobs
- `ui/index.html` — added Video (🎞️) to desktop sidebar and tablet icon sidebar
- `ui/locales/en.json` — 12 video processor strings

### Architecture
- No new AppState fields — stateless handler (ffmpeg runs and completes within the HTTP request)
- Input: file path text field (avoids uploading GB-sized video files to localhost)
- Output: `~/Videos/Eleutheria/video-{op}-{timestamp}.mp4` or `~/Music/Eleutheria/audio-{timestamp}.{ext}`
- Codec choice: h264_vaapi for encode (confirmed available in ffmpeg-free on Nobara); trim uses `-c copy` (codec-agnostic); audio uses libmp3lame/pcm_s16le/flac
- Duplicate `resolution` field problem avoided by using `compress_resolution` and `resize_resolution` as separate form field names
- ffmpeg stderr truncated to last 25 lines in error responses (avoids overwhelming the UI)

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (19 tests, 0 failures)

### Phase 3 complete
All four media tools implemented: Screen Recorder, Audio Recorder, Photo Editor + Background Removal, Video Processor.

### Next session should start with
Phase 4 — MCP server (expose tools to AI agents) + Plugin system.

---

## [2026-03-19] — Phase 3 Step 3: Photo Editor + Background Removal

### Completed

**Backend (Rust)**
- `src-tauri/src/tools/photo_editor.rs` — 2 route handlers:
  - `POST /api/photo/export` — JSON body `{data: "data:image/png;base64,..."}`, strips dataURL prefix, base64-decodes, saves to `~/Pictures/Eleutheria/photo-{timestamp}.png`
  - `POST /api/photo/rembg` — multipart `image` field, writes to `/tmp/eleutheria-photo-rembg-input.{ext}`, spawns `python3 scripts/rembg_remove.py {path}`, returns JSON `{ok, png_b64}`
- `src-tauri/Cargo.toml` — added `base64 = "0.22"` for canvas PNG dataURL decoding
- `src-tauri/src/tools/mod.rs` — registered `photo_editor` module
- `src-tauri/src/server.rs` — imported `photo_editor`, merged `photo_editor::router()`

**Python script**
- `scripts/rembg_remove.py` — reads input image, runs `rembg.remove()`, outputs base64 PNG on stdout; exit 0 on success, 1 with stderr on error

**Frontend**
- `ui/tools/photo-editor/index.html` — canvas editor:
  - Off-screen canvas per layer (`window.__peLayers[]`), "Open image" resets all layers, "+ Layer" adds overlay image (scaled to contain)
  - Layer chip strip to switch active layer; brush/eraser/Remove BG act on active layer only
  - Brush interpolation: `moveTo(lastPt) + lineTo(currentPt)` with `lineCap:round` — no more disconnected dots
  - Canvas CSS-sized to fit container (`flex-1 min-h-0 overflow-hidden` + explicit `style.width/height` after load); internal resolution stays at full image size
  - Export composites all layers onto a temp canvas, sends dataURL to `/api/photo/export`
  - Checkerboard background via CSS gradient to visualize transparency
- `ui/index.html` — added Photo Edit (🖼️) to desktop sidebar and tablet icon sidebar
- `ui/locales/en.json` — 10 photo editor strings

### Bugs fixed during session
- **Canvas overflow on large images** — `max-width/max-height: 100%` on a canvas inside a flex container without `min-h-0` has no effect; the container expands to content size. Fix: `flex-1 min-h-0 overflow-hidden` on wrap + compute CSS scale explicitly after image load.
- **Brush dots instead of strokes** — original code drew an `arc` circle per pointer event; rapid movement left disconnected dots. Fix: track `window.__peLastPt`, draw `moveTo → lineTo` between consecutive events; `lineCap:round` gives smooth strokes and a correct single-click dot.
- **No layer support** — added multi-layer architecture using off-screen `HTMLCanvasElement` per layer stored outside Alpine (`window.__peLayers`) to avoid proxy issues; compositing on every redraw.

### Architecture
- No new AppState fields — photo editor is stateless on the server (no recording process to track)
- Output saved to `~/Pictures/Eleutheria/photo-{timestamp}.png`
- Layer system: off-screen canvases composited onto display canvas on every stroke; export uses a separate temp canvas at full resolution
- rembg subprocess: Python 3.14 compatible (rembg 2.0.73 is py3-none-any; pillow, onnxruntime have cp314 wheels)

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (19 tests, 0 failures)

### Next session should start with
Phase 3 Step 4: Video Processor (ffmpeg — trim, extract audio, compress, resize).

---

## [2026-03-19] — Phase 3 Step 2: Audio Recorder

### Completed

**Backend (Rust)**
- `src-tauri/src/tools/audio_recorder.rs` — 4 route handlers:
  - `GET /api/audio/state` — JSON `{recording, started_at}` for panel state restore
  - `GET /api/audio/status` — HTML badge (idle / recording)
  - `POST /api/audio/record/start` — form field `format` (mp3/wav/ogg/flac); spawns `ffmpeg -f pulse -i default -c:a {codec} output.{ext}`; stores child + path + timestamp in AppState
  - `POST /api/audio/record/stop` — graceful stop via `q\n` to ffmpeg stdin (same pattern as voice.rs); returns result card with file path
- `src-tauri/src/tools/mod.rs` — registered `audio_recorder` module
- `src-tauri/src/server.rs` — imported `AudioRecording`, added `audio_recording` field to `AppState`, merged `audio_recorder::router()`
- `src-tauri/src/lib.rs` — initialized `audio_recording: Arc<Mutex<None>>`
- `src-tauri/src/tools/{clipboard,notes,search,translate}.rs` — test constructors updated with `audio_recording` field

**Frontend**
- `ui/tools/audio-recorder/index.html` — radio selector (mp3/wav/ogg/flac), Start/Stop with Alpine timer, state restored on load via `x-init` fetch to `/api/audio/state`
- `ui/index.html` — added Audio Rec (🎙) to desktop sidebar and tablet icon sidebar
- `ui/locales/en.json` — 4 audio recorder strings

### Architecture
- Output saved to `~/Music/Eleutheria/recording-{timestamp}.{ext}` (permanent, not tmpfs)
- `AudioRecording = Arc<Mutex<Option<(Child, String, u64)>>>` — same pattern as ScreenRecording
- Stopped via `q\n` to stdin (ffmpeg graceful), not SIGTERM — ensures proper container finalization for all formats
- Codec mapping: mp3→libmp3lame, wav→pcm_s16le, ogg→libvorbis, flac→flac

### CI status
- `cargo fmt --check` ✓
- `cargo clippy -- -D warnings` ✓
- `cargo test` ✓ (19 tests, 0 failures)

### Next session should start with
Phase 3 Step 3: Photo Editor + Background Removal.

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

