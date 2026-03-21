# Eleutheria Telos — Roadmap

---

## How to read this file

Phases are sequential **except 4.5 and 4.7**, which run in parallel:
- **Phase 4.5** (UI Polish) — Cursor's lane. HTML, CSS, theming.
- **Phase 4.7** (Backlog Sprint) — Claude Code's lane. Rust, features, fixes.

Both must be complete before Phase 5.

**Current position: Phase 4.7 active (Claude Code) + Phase 4.5 starting (Cursor).**

---

## North star (read this before every session)

> **The cross-platform utility layer that works on every device, keeps everything yours, and connects to whatever AI you already use.**

Three pillars:
1. **Offline-first, always** — every feature works without internet. AI is additive, not required.
2. **You control the AI** — local model, your API key, or self-hosted. The app connects to your choice and degrades gracefully when none is configured.
3. **Your data, your devices** — nothing leaves unless you say so. Sync is peer-to-peer on your network. No required accounts.

This framing survives platform consolidation (Apple/Google/Microsoft absorbing individual features) because it's not competing on AI quality. It's competing on ownership, portability, and the fact that no platform will ever build a tool that works equally well on your iPhone, Android tablet, Windows work laptop, and Linux home machine.

---

## Phase 0 — Foundation ✓ Complete

Running Tauri app, Axum HTTP server, HTMX navigation, SQLite, plugin loader, CI/CD (ci.yml, build.yml, release.yml). All complete.

---

## Phase 1 — Core Tools ✓ Complete

Clipboard history, Notes (full tag/backlink/trash system), Local search (FTS5). Complete including: `#tag` inline extraction, Bear-style tag sidebar, `[[Note Title]]` backlinks, trash bin with 30-day retention, date-chunked grid.

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

**Phase 4.5 is complete when** a non-technical user shown the app would not describe any part of it as unfinished. Judgment call, Rodrigo as final arbiter.

### Foundation (do first — already partially complete from Phase 4.5 Step 1)
- [x] 5 theme CSS files (`dark.css`, `light.css`, `catppuccin-mocha.css`, `catppuccin-latte.css`, `tokyo-night.css`)
- [x] Inter + Space Grotesk fonts bundled locally
- [x] Lucide icons bundled (`ui/assets/lucide.min.js`)
- [x] `base.css` — full component design system (btn, card, card-glass, input, badge, empty-state)
- [x] App shell — sidebar groups (TOOLS / MEDIA / PLUGINS), pill active state, glassmorphism
- [ ] Claude Code: inline `<script>` in shell `<head>` to set theme from `window.__ACTIVE_THEME__` before first paint (FOUC prevention)

### Panels (priority order — apply design system to panels still using old classes)
- [ ] **Clipboard History** — card depth + hover lift, `btn-*` variants on all actions, date bucket separators
- [ ] **OCR** — rich empty state, result card with `btn-*` actions
- [ ] **Screen / Audio Recorder** — pulsing red dot while recording, clear status in header
- [ ] **Notes** — tag pill chips at card bottom, editor chrome consistency
- [ ] **Models** — CSS progress bar inside model card, installed vs available distinction
- [ ] **Quick Actions** — pill/diamond node shapes per D-039, pipeline list polish
- [ ] **Video Processor / Photo Editor** — operation tab layout, toolbar polish
- [ ] **Translate / Voice** — empty state redesign
- [ ] **Settings** — theme selector (calls `setTheme()` per D-038), glass toggle, font selector
- [ ] **Plugin panels** — consistent chrome

---

## Phase 4.6 — Cohesion (Partially Complete — Claude Code lane)

- [x] **Translation backend fix** — ctranslate2 + Opus-MT replacing argostranslate (D-036)
- [ ] **Contextual pipeline CTA** — "Create pipeline from this" on OCR and Voice result cards
- [ ] **Pipeline templates** — 3-5 built-in templates in Quick Actions
- [ ] **Problem-first empty states** — replace generic "nothing here" with problem-framing CTAs

**Phase 4.6 is complete when** translation works end-to-end offline, pipelines are discoverable from result cards, and every major empty state has a problem-first CTA.

---

## Phase 4.7 — Backlog Sprint (In Progress — Claude Code lane)

### Quick Actions — Canvas ✓ Complete
- [x] H0–H4 — Nav history, graph schema, canvas render, pan/zoom, node connect, config panel, undo/redo, graph execution engine with cycle detection

### Pipeline organisation ✓ Complete
- [x] P1–P3 — Pipeline folders, YAML export, YAML import

### Notes + Clipboard ✓ Complete
- [x] Trash bin, date-chunked grid, `[[backlinks]]`, `#tag` extraction, lazy-load pagination

### Active backlog (ordered by impact)

**Safety / correctness**
- [ ] **Loop quality checks** — 60s toast warn + 120s auto-kill in Quick Actions engine. Configurable per-pipeline (`timeout_warn_secs` / `timeout_kill_secs`). Files: `quick_actions.rs`, `event_bus.rs`, `server.rs`, panel HTML.
- [ ] Fix `tools::translate::tests::test_langs_no_models` — pre-existing HTML mismatch
- [ ] Fix `clippy::unnecessary_closure` in `quick_actions.rs`

**Usability**
- [ ] **Quick Actions: opt-in/opt-out toast** — non-blocking toast when pipeline auto-fires: name + Accept / Dismiss / "Don't ask again"; auto-dismissed 8s; per-pipeline "Always run" vs "Ask me first".
- [ ] **Clipboard: copy button shows `{okay:true}`** — fix endpoint to return HTML fragment.
- [ ] **Clipboard + Notes: full-content DOM bloat** — truncate server-side, add `GET /api/clipboard/:id` + `GET /api/notes/:id/content`, modal fetches on demand.
- [ ] **Name sanity check** — strip invalid chars from pipeline and folder names, client + server.

**Features**
- [ ] **Voice: live recording waveform** — Web Audio API `AnalyserNode` + canvas. Frontend only.
- [ ] **Clipboard: pin entries + content-type icons** — `is_pinned` column migration, content-type badge.
- [ ] **Quick Actions: pipeline tree visualizer** — read-only collapsible tree as alternative to canvas.
- [ ] **Command palette learns** — `command_history` table; Ctrl+K surfaces most-used first.

**Research (no code — documented outcomes only)**
- [ ] R1 — CDP licensing → `RESEARCH_CDP.md` + DECISIONS.md
- [ ] R2 — Audacity 4 UI study → `RESEARCH_AUDACITY4.md`
- [ ] R3 — SoundThread feasibility → `RESEARCH_SOUNDTHREAD.md` + DECISIONS.md

**Phase 4.7 is complete when** all non-deferred items are checked, R1–R3 research is written, and `cargo test` + `cargo clippy -- -D warnings` pass clean.

---

## Phase 5 — Distribution + AI Layer + Audio

**Gate: Phases 4.5, 4.6, and 4.7 must all be complete.**

### Distribution
- [ ] License key system — Gumroad + asymmetric local verification, no server required
- [ ] Ad integration — ethical-ads.io or Carbon Ads; one ad per day, auto-dismissed after 5s
- [ ] Onboarding flow — first-run wizard: choose tools, download models, set global hotkey, configure AI tier
- [ ] Auto-updater — Tauri built-in, check on startup (respects offline mode)
- [ ] Installers — signed: Windows `.msi`, macOS `.dmg`, Linux `.AppImage` + `.deb`
- [ ] Code signing — Apple Developer + Windows EV certificate
- [ ] Backup/restore — export/import all user data as ZIP

### AI tier system (critical — unlocks value for non-technical users)
- [ ] **"Your AI" section in Settings** — one place to configure the AI tier. No technical knowledge required. Three options presented simply: "Use my device (free, private)", "Use my existing subscription (paste key)", "Use local server (advanced)".
- [ ] **Cloud API key support** — accept an API key for OpenAI, Anthropic, Gemini, or Mistral. Route heavy tasks (summarization, smart search, document understanding) to the configured provider automatically.
- [ ] **Self-hosted endpoint support** — accept any OpenAI-compatible endpoint URL (Ollama, LM Studio, llama.cpp server). Same UI as cloud key, different field.
- [ ] **Graceful fallback chain** — local model → cloud key → self-hosted → offer to configure. Never fails silently. Clear messaging at each step.
- [ ] **Bundled venv + first-run AI setup** — auto-create `~/.local/share/eleutheria-telos/venv/` on first launch, pip install in background, progress screen; all Python subprocesses use venv interpreter.

### Product quality
- [ ] OCR: click-to-capture — screengrab from selection marquee inside app; no file dialog
- [ ] Screen Recorder: global hotkey start/stop — `Super+Shift+R` default, configurable
- [ ] Sidebar customisation — hide/show built-in tools; Plugin Store with search
- [ ] Session recovery — Photo Editor, Video Processor, Quick Actions auto-save to `session_drafts`
- [ ] Photo Editor: undo/redo — command stack (max 50), Ctrl+Z / Ctrl+Shift+Z
- [ ] Notes: text highlighting — `==text==` syntax, multi-colour chip toolbar on selection
- [ ] **Freehand notes** — draw, annotate, and write with mouse, stylus, or finger anywhere in the Notes panel. Canvas layer underneath or above text content. Pressure sensitivity where hardware supports it. (Fabric.js canvas already present from photo editor — reuse.)

### Audio tools (gated on R1–R3 from Phase 4.7)
- [ ] **Audio Editor** — trim, fade, normalize, cut/paste; Wavesurfer.js waveform; ffmpeg backend ± CDP if licensing cleared (R1)
- [ ] **Virtual Microphone / Voice Effects** — real-time DSP chain (CPAL capture → effects → PipeWire/PulseAudio null sink on Linux, VB-Cable on Windows, BlackHole on macOS). Pitch shift, reverb, noise gate, radio filter.
- [ ] **Soundboard** — sound button grid, playback routed through virtual mic for streaming/calls
- [ ] **Audio player through virtual mic** — play any audio file and route it through the virtual mic so it appears as your microphone output to other apps (Discord, Zoom, OBS, games)

### Community theming
- [ ] Theme contribution guide — document the CSS variable contract, show how to derive values from any VS Code / terminal theme palette
- [ ] Ship 2-3 additional community-sourced themes

**Phase 5 is complete when** the app is distributable to non-technical users on all three desktop platforms, the AI tier system works end-to-end, monetization is live, and audio tool scope is decided.

---

## Phase 6 — Mobile + Sync + Memory + Photo Library

This phase makes the app genuinely cross-device and adds the features with the highest daily-use ceiling for non-technical users.

### Mobile (Android + tablet)
- [ ] Tauri Android build running and signed
- [ ] Phone layout (<640px): bottom nav, core tools (Voice-to-Text, OCR via camera, Notes, Clipboard)
- [ ] Tablet layout (640–1023px): icon-only sidebar
- [ ] Background Foreground Service for clipboard monitoring on Android
- [ ] OCR: device camera as capture source
- [ ] APK for sideloading; eventually F-Droid
- [ ] iOS/iPad: blocked on Tauri iOS stable — responsive layouts already correct, no architecture changes needed when ready

### Local-network sync (P2P, no server required)
- [ ] **Device discovery** — devices on the same WiFi see each other automatically via mDNS/Bonjour. No IP configuration required.
- [ ] **Clipboard sync** — anything copied on one device appears on all others within ~1 second
- [ ] **Notes sync** — notes created or edited on any device propagate to all others (last-write-wins for Phase 6; conflict resolution in Phase 7)
- [ ] **Captures sync** — OCR results, transcriptions, and translated text sync across devices
- [ ] **Settings sync** — theme, glass preference, and tool configuration sync across devices
- [ ] **No server, no account** — pure peer-to-peer on the local network. When devices are on different networks, nothing syncs (optional cloud sync is Phase 7+).

### Personal memory layer
- [ ] **Captures table** — everything the app processes (clipboard entries, OCR results, transcriptions, notes) is stored in a unified `captures` table with timestamps, content type, and source tool
- [ ] **Smart search** — when a cloud API key or local model is configured, Ctrl+K can answer natural-language queries: "find that article about X I copied last week", "what did I transcribe from the meeting about Y"
- [ ] **Memory timeline** — a chronological view of everything captured, filterable by tool and content type. The app's memory of what you did, yours to keep.
- [ ] **Quick Actions trigger: on capture** — any capture can trigger a pipeline automatically (e.g., every OCR result → translate if not in English → save to notes)

### Photo library — personal photo management without subscriptions
**The problem:** Adults pay Google/Apple every month just to keep their own memories accessible. Phones fill up. Old photos are scattered across dead laptops, random USB drives, and phones people no longer own. Old people don't know how to manage this. Young people can't afford to. Nobody has time.

**The solution:** A local-first photo library that stores everything on your own devices, syncs across them on your network, and never imposes storage limits because it never touches a server.

- [ ] **Import** — scan a folder, a phone's DCIM directory, a USB drive, or an SD card. Detect and skip duplicates (perceptual hash). Import preserves original files and EXIF metadata.
- [ ] **Library view** — timeline grid (grouped by month/year), album view, search by date range. No face recognition in Phase 6 (privacy, complexity).
- [ ] **Automatic device sync** — when your phone connects to the same WiFi as your laptop, new photos appear on the laptop automatically. No cables. No iCloud. No Google Photos.
- [ ] **Storage anywhere** — photos live on whatever drive you point the app at. External HDD, NAS, the device itself — your choice, your hardware.
- [ ] **Export** — select photos, export to folder, ZIP, or directly to a USB drive. One-click "back up to this drive."
- [ ] **Basic editing** — rotate, crop, brightness/contrast. Not Lightroom — just the operations people actually need.
- [ ] **No compression** — photos are stored exactly as captured. No Google-style "storage saver" quality reduction unless the user explicitly opts in.

> **Why this matters:** This is the most concrete answer to "what problem does this solve for non-technical people right now?" The photo storage subscription is a monthly pain point that affects everyone with a smartphone. The answer isn't "here's a better cloud service" — it's "your photos live on your devices and sync between them, and that's all you need."

**Phase 6 is complete when** Android app is signed and installable, local-network sync works across at least 2 devices, and the photo library can import, display, and sync a real photo collection.

---

## Phase 7 — Screen Sharing + Context Awareness + Advanced Sync

### Screen sharing (P2P)
- [ ] WebRTC P2P via PeerJS, 6-character room code, no accounts
- [ ] Audio sharing toggle
- [ ] 2-5 people: SFU via LiveKit Cloud free tier or self-hosted mediasoup
- [ ] **Screen annotation during share** — draw on the shared screen in real time (reuses freehand canvas from Phase 5)
- [ ] Recorded session saving (local, automatic)

### Screen context awareness (opt-in, local processing only)
- [ ] **Screen understanding** — opt-in: the app can observe the active window and understand its context. User triggers explicitly (hotkey or button) — never continuous passive monitoring.
- [ ] **Context-aware Quick Actions** — when a pipeline is triggered, it knows what was on screen. "Summarize this article" reads the browser tab. "Add to notes" captures the visible content.
- [ ] **Smart paste** — clipboard entry is transformed based on where you're pasting. Pasting a URL into the Notes editor → auto-fetches title and description. Pasting into a translation field → auto-detects language.
- [ ] **Camera context awareness** (mobile) — point your camera at text, a whiteboard, or a document. The app understands what it sees and offers actions: translate, transcribe, save to notes, add to photo library.
- [ ] All context processing runs locally when possible. When a cloud model is configured, heavier understanding routes there — with explicit user confirmation before any screen content is sent.

### Advanced sync
- [ ] **Cloud sync (optional, user-controlled)** — encrypted sync to a user-configured S3-compatible bucket, Nextcloud instance, or similar. The app never operates its own sync server.
- [ ] **Conflict resolution** — for notes edited on two devices simultaneously, show a merge view rather than silently overwriting.
- [ ] **Photo library sync across networks** — when on different WiFi networks, sync via the configured cloud storage. Photos never go through Eleutheria servers.

---

## Future / Community Ideas

Not scheduled. Full specs in `IDEAS.md`.

- Plugin registry (browsable, one-click install)
- Video timeline editor (community plugin — Video Processor as ffmpeg backend)
- iOS support (Tauri iOS when stable)
- Community i18n translations
- Dynamic color adaptation from wallpaper
- Floating radial mode / fan menu
- MCP over LAN (expose MCP server on local network for other apps)
- "Second brain" onboarding — guided setup that frames the app as a memory layer from day one
- AR annotation (future mobile) — draw over the camera feed in real time
- Local LLM fine-tuning on your own notes (very future — requires significant hardware)
