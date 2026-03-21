# Eleutheria Telos — Roadmap

---

## How to read this file

Phases are sequential **except 4.5 and 4.7**, which run in parallel:

- **Phase 4.5** (UI Polish) — Cursor's lane. HTML, CSS, theming.
- **Phase 4.7** (Backlog Sprint) — Claude Code's lane. Rust, features, fixes.

Both must be complete before Phase 5.

**Current position: Phase 4.7 active (Claude Code) + Phase 4.5 starting (Cursor).**

---

## Phase 0 — Foundation ✓ Complete

Running Tauri app, Axum HTTP server, HTMX navigation, SQLite, plugin loader, CI/CD (ci.yml, build.yml, release.yml). All items complete.

---

## Phase 1 — Core Tools ✓ Complete

Clipboard history, Notes (full tag/backlink/trash system), Local search (FTS5). Complete including: `#tag` inline extraction, Bear-style tag sidebar, `[[Note Title]]` backlinks, trash bin with 30-day retention, date-chunked grid, drag-and-drop reordering.

---

## Phase 2 — AI Tools ✓ Complete (translation blocked at runtime)

OCR (Tesseract), Voice-to-text (Whisper.cpp), Translation (argostranslate — blocked by Python 3.14 incompatibility; fixed in Phase 4.6 via ctranslate2), Models manager. Structurally complete.

---

## Phase 3 — Media Tools ✓ Complete

Screen recorder, audio recorder, photo editor (background removal via rembg), video processor (ffmpeg). All complete.

---

## Phase 4 — MCP + Plugin Ecosystem ✓ Complete

MCP server (stdio + SSE), full plugin system, example plugins (Python + Node), plugin developer docs, Quick Actions basic pipeline. All complete.

---

## Phase 4.5 — UI Polish (In Progress — Cursor lane)

**Goal:** Transform the functional-but-bare UI into something a non-technical user would feel proud to use daily.

**Design spec:** `UI_BRIEF.md` (approved 2026-03-19).
**CSS architecture:** `DECISIONS.md` D-038 — separate CSS file per theme, swapped via `<link>` href.

**Phase 4.5 is complete when** a non-technical user shown the app would not describe any part of it as unfinished. No fixed panel checklist — judgment call, with Rodrigo as the final arbiter.

### Foundation (do first — everything depends on it)
- [ ] `ui/assets/themes/dark.css` — full CSS variable set per D-038
- [ ] `ui/assets/themes/light.css`
- [ ] `ui/assets/themes/catppuccin-mocha.css`
- [ ] `ui/assets/themes/catppuccin-latte.css`
- [ ] `ui/assets/themes/tokyo-night.css`
- [ ] Inter font bundled locally under `ui/assets/fonts/inter/` (no CDN)
- [ ] Lucide icons bundled as `ui/assets/lucide.min.js`; `lucide.createIcons()` on DOMContentLoaded + every HTMX swap
- [ ] `ui/shell.html` — sidebar groups (TOOLS / MEDIA / PLUGINS), pill active state, glassmorphism, Lucide icons replacing all emojis
- [ ] Claude Code: inline `<script>` in `shell.html` `<head>` to set theme from `window.__ACTIVE_THEME__` before first paint (FOUC prevention); `window.__ACTIVE_THEME__` injected by Axum at shell render time

### Panels (priority order)
- [ ] **Clipboard History** — card depth + hover lift, `btn-*` variants on all actions, date bucket separator styling
- [ ] **OCR** — rich empty state (large Lucide icon + prominent `btn-primary` CTA), result card
- [ ] **Screen Recorder** — pulsing red dot while recording (CSS `@keyframes`), clear status in header
- [ ] **Audio Recorder** — same pattern as Screen Recorder
- [ ] **Notes** — tag pill chips at card bottom, editor chrome consistency
- [ ] **Models** — CSS progress bar inside model card, installed vs available distinction
- [ ] **Quick Actions** — pill/diamond node shapes per D-039, step card left-border accent, pipeline list polish
- [ ] **Video Processor** — operation tab layout polish
- [ ] **Photo Editor** — toolbar polish
- [ ] **Translate / Voice** — empty state redesign
- [ ] **Settings** — theme selector (calls `setTheme()` per D-038), glass toggle, font selector
- [ ] **Plugin panels** — consistent chrome

---

## Phase 4.6 — Cohesion (Partially Complete — Claude Code lane)

**Goal:** Close product gaps that make features feel isolated.

- [x] **Translation backend fix** — ctranslate2 + Opus-MT replacing argostranslate (D-036)
- [ ] **Contextual pipeline CTA** — "Create pipeline from this" on OCR and Voice result cards
- [ ] **Pipeline templates** — 3-5 built-in templates in Quick Actions ("OCR → Translate → Copy", "Voice → Save as Note", "Clipboard → Translate")
- [ ] **Problem-first empty states** — replace generic "nothing here" with problem-framing CTAs

**Phase 4.6 is complete when** translation works end-to-end offline, pipelines are discoverable from result cards, and every major empty state has a problem-first CTA.

---

## Phase 4.7 — Backlog Sprint (In Progress — Claude Code lane)

**Goal:** High-impact features deferred from previous phases. Pure product value.

### Quick Actions — Canvas ✓ Complete
- [x] H0 — Panel navigation history (back/forward, Alt+←/→, mouse side buttons)
- [x] H1 — DB migration: graph schema (`pipeline_nodes`, `pipeline_edges`)
- [x] H2 — Canvas render + persistence (draggable nodes, SVG bezier edges, dot-grid)
- [x] H3a — Canvas pan + zoom (transform-based, scroll-to-zoom, fit-all ⌖)
- [x] H3b — Connect nodes (ports, back-edges for cycles, click-edge-Delete)
- [x] H3c — Node config panel (280px right panel, all node type forms, Save PUTs to DB)
- [x] Canvas QoL (spawn position, config closes on delete, camera reset on pipeline switch)
- [x] H3d — Undo/redo (50-op stack, Ctrl+Z/Y/Shift+Z, toolbar ↩↪)
- [x] H4 — Graph-aware execution engine (graph traversal, condition eval, cycle detection + timeout)

> **Loop design (2026-03-19):** No dedicated Loop node. Loops are back-edges. Engine enforces per-pipeline timeout (default 60s warn / 120s kill). D-037.

### Pipeline organisation and portability ✓ Complete
- [x] P1 — Pipeline folders (group, collapse/expand, move pipeline into folder)
- [x] P2 — Export pipeline as YAML ("Export" button, downloads `.yaml`)
- [x] P3 — Import pipeline from YAML (fresh UUIDs, edges remapped)

> **YAML format (2026-03-20):** `name`, `trigger`, `nodes` (id slug, type, config, pos_x, pos_y), `edges` (source slug, target slug, label). Slugs in YAML; real UUIDs on import.

### Notes + Clipboard improvements ✓ Complete
- [x] Trash bin with 30-day retention (soft delete, restore, Purge Forever)
- [x] Date-chunked grid (Today / Yesterday / This Week / This Month / Older)
- [x] Notes: `[[Note Title]]` backlinks (join table, backlink panel, clickable in preview)
- [x] Notes: Bear-style `#tag` extraction, collapsible sidebar, right-click delete
- [x] Notes: drag ghost fix, `user-select:none`
- [x] Notes: grid card UI, lazy-load pagination (24/page)
- [x] Clipboard: lazy-load pagination (20/page)

### Active backlog (Claude Code — ordered by impact)

**Safety / correctness — start here**
- [ ] **Loop quality checks** — 60s toast warn + 120s auto-kill in the Quick Actions execution engine. Both thresholds configurable per-pipeline (`timeout_warn_secs` / `timeout_kill_secs`) and globally in Settings. Files: `quick_actions.rs`, `event_bus.rs`, `server.rs`, Quick Actions panel HTML.
- [ ] Fix `tools::translate::tests::test_langs_no_models` — pre-existing HTML mismatch
- [ ] Fix `clippy::unnecessary_closure` in `quick_actions.rs`

**Usability**
- [ ] **Quick Actions: opt-in/opt-out toast** — when a pipeline fires automatically, show non-blocking toast bottom-right: pipeline name + Accept / Dismiss / "Don't ask again" checkbox. Auto-dismissed after 8s. Per-pipeline setting: "Always run" vs "Ask me first". Files: `event_bus.rs`, `quick_actions.rs`, OCR/Voice result card HTML.
- [ ] **Clipboard: copy button shows `{okay:true}`** — fix endpoint to return HTML fragment (button in "copied" state) or `hx-swap="none"` + `htmx:afterRequest`. See IDEAS.md Bugs.
- [ ] **Clipboard + Notes: full-content DOM bloat** — truncate to 2KB attr / 300 char preview server-side; add `GET /api/clipboard/:id` and `GET /api/notes/:id/content`; modal fetches on demand. Files: `clipboard.rs`, `notes.rs`, both index.html modal handlers.
- [ ] **Name sanity check** — strip/reject invalid chars (`/`, `\`, `<`, `>`, `"`, null bytes) from pipeline and folder names, client + server.

**Features**
- [ ] **Voice: live recording waveform** — Web Audio API `AnalyserNode` + canvas. Frontend only. Fast win (~1 session).
- [ ] **Clipboard: pin entries + content-type icons** — `is_pinned` column (migration), pinned float to top, content-type badge (URL globe, image chip, code `{}` icon).
- [ ] **Quick Actions: pipeline tree visualizer** — read-only collapsible tree as alternative to canvas; toggle in toolbar.
- [ ] **Command palette learns** — `command_history` table; Ctrl+K surfaces most-used first.

**Research (no code — documented outcomes only)**
- [ ] **R1 — CDP licensing** → `RESEARCH_CDP.md` + DECISIONS.md. Commercial redistribution? CLI API surface? ffmpeg overlap? Full spec in IDEAS.md.
- [ ] **R2 — Audacity 4 UI study** → `RESEARCH_AUDACITY4.md`. New UX patterns; GPL boundary confirmation. Full spec in IDEAS.md.
- [ ] **R3 — SoundThread feasibility** → `RESEARCH_SOUNDTHREAD.md` + DECISIONS.md. License, embed terms, offline viability, vs in-house Wavesurfer.js. Full spec in IDEAS.md.

**Deferred**
- [ ] Video: multi-track NLE — Phase 5
- [ ] Quick Actions: pipeline keybinds — requires `tauri-plugin-global-shortcut`; Phase 5

**Phase 4.7 is complete when** all non-deferred items are checked, R1–R3 research is written, and `cargo test` + `cargo clippy -- -D warnings` pass clean.

---

## Phase 5 — Monetization + Distribution

**Gate: Phases 4.5, 4.6, and 4.7 must all be complete.**

### Distribution
- [ ] License key system — Gumroad + asymmetric local verification, no server required after purchase
- [ ] Ad integration — ethical-ads.io or Carbon Ads; one ad per day on app open, auto-dismissed after 5s
- [ ] Onboarding flow — first-run wizard: choose tools, download models, set global hotkey
- [ ] Auto-updater — Tauri built-in updater, check on startup (respects offline mode)
- [ ] Installers — signed: Windows `.msi`, macOS `.dmg`, Linux `.AppImage` + `.deb`
- [ ] Code signing — Apple Developer + Windows EV certificate (or document workaround for MVP)
- [ ] Backup/restore — export/import user data as ZIP from Settings

### Product quality
- [ ] OCR: click-to-capture — screengrab from selection marquee inside app; no file dialog
- [ ] Screen Recorder: global hotkey start/stop — `Super+Shift+R` default, configurable
- [ ] Sidebar customisation — hide/show built-in tools; Plugin Store with search
- [ ] Session recovery — Photo Editor, Video Processor, Quick Actions auto-save to `session_drafts`; restore banner on re-open
- [ ] Photo Editor: undo/redo — command stack (max 50), Ctrl+Z / Ctrl+Shift+Z
- [ ] Notes: text highlighting — `==text==` syntax, multi-colour chip toolbar on selection
- [ ] Bundled venv + first-run AI setup — auto-create venv on first launch, pip install in background, progress screen; all Python subprocesses use venv interpreter

### Community theming (natural continuation of D-038)
- [ ] Theme contribution guide — document the 16-variable contract, show how to derive values from any terminal/VS Code theme palette
- [ ] Ship 2-3 additional community-sourced themes as examples beyond the 5 built-ins

### Audio tools (gated on R1–R3 from Phase 4.7)
- [ ] Soundboard — sound button grid, virtual mic routing (PipeWire on Linux, VB-Cable on Windows, BlackHole on macOS)
- [ ] Virtual Microphone / Voice Effects — real-time DSP chain (CPAL capture → effects → null sink loopback)
- [ ] Audio Editor — trim, fade, normalize, cut/paste; Wavesurfer.js waveform; ffmpeg ± CDP (if licensing cleared by R1)

**Phase 5 is complete when** the app is distributable to non-technical users on all three platforms, monetization is live, and audio tool scope is decided from R1–R3 research.

---

## Phase 6 — Mobile (Android + tablet)

- [ ] Tauri Android build running and signed
- [ ] Phone layout (<640px): bottom nav, 4 tools (Voice-to-Text, OCR via camera, Notes, Clipboard)
- [ ] Tablet layout (640–1023px): icon-only sidebar, same 4 tools
- [ ] Background Foreground Service for clipboard monitoring
- [ ] APK for sideloading; eventually F-Droid

iOS/iPad: not scheduled. Blocked on Tauri iOS stable. Responsive layouts already correct for iOS screen sizes.

---

## Phase 7 — Screen Sharing (P2P)

- [ ] WebRTC P2P via PeerJS, 6-character room code, no accounts
- [ ] Audio sharing toggle
- [ ] 2-5 people: SFU via LiveKit Cloud free tier or self-hosted mediasoup
- [ ] Screen annotation during share (draw in real time)
- [ ] Recorded session saving (local, automatic)

---

## Future / Community Ideas

Not scheduled. Architecturally supported. Full specs in `IDEAS.md`.

- Plugin registry (browsable, one-click install)
- Video timeline editor (community plugin — Video Processor as ffmpeg backend)
- Cloud sync (user-controlled, optional)
- iOS support (Tauri iOS when stable)
- Community i18n translations
- Local LLM via Ollama (notes summarization, smart search)
- MCP over LAN (expose MCP server on local network for mobile ↔ desktop)
- Dynamic color adaptation from wallpaper (Phase 6+)
- Floating radial mode / fan menu (Phase 5, requires Tauri transparent overlay window)
