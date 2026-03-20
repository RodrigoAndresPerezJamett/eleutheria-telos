# Eleutheria Telos — Roadmap

---

## Current Phase: Phase 0 — Foundation

**Goal:** A running Tauri app with internal HTTP server, HTMX navigation, SQLite, and plugin loader. No tool functionality yet — only the skeleton that everything else will be built on.

**Phase 0 is complete when:**
- [ ] Tauri 2.x app builds and runs on Windows, macOS, and Linux
- [ ] Axum HTTP server starts on an auto-detected port (starting at 47821)
- [ ] Session token generated at startup, injected into WebView
- [ ] Shell UI renders with sidebar (desktop) and bottom nav (mobile)
- [ ] Navigation between empty tool placeholders works via HTMX
- [ ] SQLite connected with initial migrations (all tables from ARCHITECTURE.md)
- [ ] System tray icon and global hotkey (configurable) to show/hide window
- [ ] Plugin loader reads `/plugins/*/manifest.json` and logs detected plugins
- [ ] i18n loader reads `ui/locales/en.json` and resolves strings in templates
- [ ] CI/CD pipeline configured (GitHub Actions):
  - `ci.yml` — fmt + clippy + tests on every push
  - `build.yml` — cross-platform compile check on push to dev/main
  - `release.yml` — full build + draft release on `v*` tags
- [ ] `TAURI_SIGNING_PRIVATE_KEY` secret added to GitHub repository settings

---

## Phase 1 — Core Tools (Offline, No AI)

**Goal:** The 3 most universally useful tools, working fully offline.

- [ ] **Clipboard History** — monitor system clipboard, store in SQLite, display list with search, click to re-copy
- [ ] **Notes** — create/edit/delete notes in Markdown, tag support, pin support, local-only
- [ ] **Local Search** — FTS5 search across notes and clipboard history via Command Palette (`Ctrl+K`)

**Phase 1 is complete when** a user can open the app, see their clipboard history, write and find notes, and search across everything — with zero internet connection.

---

## Phase 2 — AI Tools (Offline Models)

**Goal:** Add the AI-powered tools with local models. Introduce the Models manager.

- [ ] **Models panel** — download/delete/update AI models with progress indicator
- [ ] **OCR from capture** — select screen area, extract text, copy or save to note
- [ ] **Voice-to-text** — record mic or open audio file, transcribe with Whisper.cpp
- [ ] **Translation** — translate any text via Argos Translate offline, DeepL/Google as optional online fallback
- [ ] **OCR + Translation pipeline** — after OCR, offer one-click translation

**Phase 2 is complete when** a user can capture text from screen, transcribe audio, and translate — all without internet.

---

## Phase 3 — Media Tools

**Goal:** Screen recording and basic photo editing.

- [x] **Screen recorder** — record full screen or selected window, optional mic audio overlay, save as mp4
- [x] **Audio recorder** — record microphone to mp3/wav
- [x] **Photo editor** — open image, paint/erase (for manual background removal), layer a second image on top, export
- [x] **Background removal** — remove image background with rembg (AI, offline), or manually with eraser brush
- [x] **Video processor** — cut/trim video, extract audio, compress, change resolution — all via ffmpeg, no visual timeline

**Phase 3 is complete when** a user can record their screen, do basic photo compositing, and process videos without a third-party app.

---

## Phase 4 — MCP + Plugin Ecosystem

**Goal:** Make the app usable as AI agent infrastructure and open it to community developers.

- [x] **MCP server — stdio transport** — all built-in tools accessible as MCP tools
- [x] **MCP server — SSE transport** — accessible from web-based agent clients
- [x] **Plugin system — full implementation** — plugins run, routes proxied, permissions enforced, sidebar entry added
- [x] **Example plugin (Python)** — reference implementation with full manifest
- [x] **Example plugin (Node.js)** — reference implementation
- [x] **Plugin developer documentation** — how to build, manifest schema, available permissions, event bus events
- [x] **Quick Actions (basic)** — connect tool events to actions without code (e.g., auto-translate after OCR)

**Phase 4 is complete when** an AI agent can use the app's tools, and a community developer can build and install a working plugin by following the docs.

---

## Phase 4.5 — UI Polish

**Goal:** Transform the current functional-but-bare UI into something visually polished that a non-technical user would feel proud to use daily.

**Workflow (mandatory — follow this order every session):**

1. **References** — user provides screenshots of the current state + 1-2 apps they find visually inspiring. Claude reads these before asking anything.
2. **Questions** — Claude asks everything needed: aesthetic direction, component inventory, pain points, priorities, what to keep vs redesign. All answers saved to `UI_BRIEF.md`.
3. **Execution** — Claude implements based on the brief. Works panel by panel, not all at once.
4. **Playwright review** — Claude navigates the running app with Playwright MCP, screenshots every panel, adjusts based on what it sees. App must be running before this step.
5. **User feedback** — Claude signals "ready for review". User opens the app, tests, and gives specific feedback.
6. **Iteration** — repeat steps 4–5 until the user is satisfied.

**Scope:**

- [x] **Visual design brief** — `UI_BRIEF.md` capturing aesthetic direction, color palette, typography, spacing, component patterns
- [x] **App shell** — sidebar, header, transitions, empty states
- [x] **Clipboard History panel** — list density, item preview, search bar
- [x] **Notes panel** — editor chrome, tag display, pin indicator
- [x] **OCR panel** — capture button affordance, result display; card wrapper added
- [x] **Voice panel** — recording state feedback, waveform or indicator
- [x] **Translation panel** — language selector UX, result layout; empty state redesigned
- [x] **Screen / Audio / Video panels** — control layout, progress feedback
- [x] **Photo editor panel** — toolbar, canvas area, export flow
- [x] **Quick Actions panel** — pipeline list readability, step editor clarity
- [x] **Settings panel** — form layout, section grouping
- [x] **Models panel** — download progress, installed vs available states
- [x] **Plugin panels** — consistent chrome so plugin UIs feel native

**Phase 4.5 is complete.** Playwright review done 2026-03-19. Every panel screenshotted and signed off. Design system in `UI_BRIEF.md`. Next: Phase 4.6 — Cohesion.

---

## Phase 4.6 — Cohesion

**Goal:** Close the product gaps that make features feel isolated. Fix the translation blocker, make pipelines discoverable, prove the plugin system with a real plugin, and improve empty states. This phase must be complete before Phase 5 distribution work begins.

- [x] **Translation backend fix** — replace argostranslate with ctranslate2 + Opus-MT models directly; eliminates Python 3.14 incompatibility and ~3GB dependency footprint (see D-036)
- [ ] **Contextual pipeline CTA** — add "Create pipeline from this" button to OCR and Voice result cards, pre-filling the Quick Actions builder with the correct trigger
- [ ] **Pipeline templates** — 3-5 built-in templates featured prominently in the Quick Actions panel (e.g., "OCR → Translate → Copy", "Voice → Save as Note", "Clipboard → Translate")
- [ ] **First real community plugin** — build and open-source one non-trivial plugin (Obsidian send or GitHub Issues creator); stress-tests the plugin API and serves as reference implementation *(deferred to backlog — plugin system already stress-tested in Phase 4; moved to IDEAS.md)*
- [ ] **Problem-first empty states** — replace generic "nothing here" messages with problem-framing CTAs (e.g., "Lost something you copied? Your clipboard history lives here.", "Need text from an image? Capture a screen region.")

**Phase 4.6 is complete when** translation works end-to-end, pipelines are discoverable from result cards, at least one real plugin is published, and every major empty state has a problem-first CTA.

---

## Phase 4.7 — Backlog Sprint (current)

**Goal:** Implement high-impact features from IDEAS.md that were deferred from previous phases. No monetization, no distribution. Pure product value. Each hito is tested by the user before merging and updating CHANGELOG.md.

**Order: highest cross-cutting impact first.**

### Quick Actions — Canvas visual (replacing list editor)
- [ ] **H0 — Panel navigation history (back/forward)** — shell-level feature; back/forward chevrons in header; in-memory nav stack updated on every `htmx:afterSwap` of `#tool-panel`. Applies to all tools, not just Quick Actions.
- [ ] **H1 — DB migration: graph schema** — new `pipeline_nodes` + `pipeline_edges` tables; auto-migrate existing `pipeline_steps` to linear node chains; new API routes `/graph`, `/nodes`, `/edges`.
- [ ] **H2 — Canvas render + persistence** — nodes as draggable HTML divs, SVG connections overlay, node positions auto-saved to DB on every move (no "unsaved work" — canvas state is always persisted).
- [ ] **H3 — Node palette + connect/disconnect + undo/redo** — toolbar with node types (Trigger, Action, Condition, Loop, End); click output port → drag → click input port to connect; Ctrl+Z / Ctrl+Shift+Z for canvas operations.
- [ ] **H4 — Graph-aware execution engine** — replaces linear step runner; graph traversal; condition node evaluation; backward-compatible with migrated pipelines.
- [ ] **H5 — Loop node with timeout** — loop node with configurable `max_iterations` and `timeout_secs` (default 60); cycle detection via loop node counter.

### Remaining backlog items (ordered by impact)
- [ ] **Notes: inline #tag extraction** — `#tag` tokens parsed at save time → `tags` table → clickable chips in notes list → filter by tag. Touches: `notes.rs`, SQLite migration, notes list UI, search.
- [ ] **Quick Actions: opt-in/opt-out for auto-triggered pipelines** — small non-blocking toast bottom-right when a pipeline trigger fires; Accept / Dismiss; auto-dismissed after 8s; "Don't ask again for this pipeline" checkbox. Touches: `event_bus.rs`, `quick_actions.rs`, result cards (OCR/Voice).
- [ ] **Command palette learns** — `command_history` table with access counts and last-used timestamps; Ctrl+K surfaces most-used items first.
- [ ] **Voice: live recording waveform** — Web Audio API `AnalyserNode` + canvas waveform while mic is active. Frontend only.
- [ ] **Clipboard: pin entries + content-type icons** — `is_pinned` column, pinned items float to top; content-type badge (URL, image, code) per item.
- [ ] **Video: multi-track NLE** — audio + video tracks, trim handles, concatenate clips. Major feature; deferred to end of sprint.

**Phase 4.7 is complete when** all items above are checked, tested, and committed.

---

## Phase 5 — Monetization + Distribution

**Goal:** Make the app distributable to non-technical users and close the most impactful product gaps identified in the Phase 4.6 introspection.

### Distribution

- [ ] **License key system** — Gumroad integration, asymmetric key verification, no server required
- [ ] **Ad integration** — ethical-ads.io or Carbon Ads, one ad per day on app open, auto-dismissed after 5 seconds
- [ ] **Onboarding flow** — first-run wizard: choose tools to enable, download models, set hotkey
- [ ] **Auto-updater** — Tauri's built-in updater, check for updates on startup (respects offline mode)
- [ ] **Installers** — signed installers for Windows (.msi), macOS (.dmg), Linux (.AppImage + .deb)
- [ ] **Code signing** — Apple Developer + Windows EV certificate (or document manual install workaround for free MVP)
- [ ] **Backup/restore** — export/import user data as ZIP from Settings panel

### Product quality (fast wins from Phase 4.6 introspection)

- [ ] **Clipboard: pin entries + content-type icons** — `is_pinned` column, pinned items float to top; content-type badge (URL globe, image thumbnail chip, code `{}` icon) shown on each item. Data already captured — UI only.
- [ ] **OCR: click-to-capture** — screengrab directly from a selection marquee inside the app (Tauri screenshot API + region select overlay), no file dialog required. Eliminates the #1 friction point of the current OCR flow.
- [ ] **Voice: live recording waveform** — Web Audio API `AnalyserNode` feeds a canvas waveform visualisation during mic recording. Purely frontend, no native code. Makes recording feel responsive and confirms the mic is active.
- [ ] **Notes: inline #tag extraction** — `#tag` tokens in note body are parsed at save time, stored to a `tags` table, and rendered as clickable chips in the note list sidebar. Enables "show all notes tagged #meeting". No backlinks yet.
- [ ] **Screen Recorder: global hotkey start/stop** — `tauri-plugin-global-shortcut` binding (default `Super+Shift+R`) starts/stops recording without requiring the user to click inside the app. Essential for real screencasting use.
- [ ] **Panel navigation history (back / forward)** — back/forward chevrons in the shell header; in-memory nav stack updated on every panel swap. See IDEAS.md for implementation detail.
- [ ] **Sidebar customisation** — hide/show built-in tools from sidebar; plugin store lists built-ins first; store has search bar. See IDEAS.md for detail.
- [ ] **Session recovery for stateful tools** — photo editor, video processor, quick actions auto-save state to `session_drafts` table; on re-open show "restore unsaved work?" banner. See IDEAS.md for detail.
- [ ] **Command palette learns** — `command_history` table with access counts and last-used timestamps; Ctrl+K surfaces most-used items first.
- [ ] **Notes: text highlighting / markers** — `==text==` syntax, multi-colour chip toolbar on selection. See IDEAS.md for detail.
- [ ] **Photo Editor: undo / redo** — command stack (max 50 steps), Ctrl+Z / Ctrl+Shift+Z. See IDEAS.md for detail.

---

## Phase 6 — Mobile (Android + tablet)

**Goal:** Android app with the 4 most portable tools, working on both phone and tablet.

- [ ] Tauri Android build running and signed
- [ ] Phone layout (<640px): bottom nav with 4 tools: Voice-to-Text, OCR (camera input), Notes, Clipboard
- [ ] Tablet layout (640px–1023px): icon-only sidebar with same 4 tools
- [ ] Background Foreground Service for clipboard monitoring on Android
- [ ] OCR uses device camera as capture source (it also can use screen area, it has to be available on all platforms)
- [ ] APK available for sideloading, eventually submitted to F-Droid

**iOS / iPhone / iPad:** Not scheduled. Blocked on Tauri iOS reaching stable. The responsive layouts (mobile + tablet breakpoints) are already designed to work on iOS screen sizes — no architecture changes needed when Tauri iOS is ready.

---

## Phase 7 — Screen Sharing (P2P)

**Goal:** Simple peer-to-peer screen sharing for 1-5 people, no accounts required.

- [ ] WebRTC P2P screen sharing via PeerJS
- [ ] 6-character room code shared out-of-band (WhatsApp, email, etc.)
- [ ] Audio sharing toggle
- [ ] For 2-5 people: SFU via LiveKit Cloud free tier or self-hosted mediasoup
- [ ] No accounts, no server setup for the end user

---

## Future / Community Ideas

These are not scheduled but are architecturally supported:

- **Plugin registry** — browsable directory of community plugins, install with one click
- **Video timeline editor** — visual NLE built as a plugin (major community project)
- **Cloud sync (optional)** — user-controlled sync for notes and settings
- **iOS support** — Tauri iOS when stable enough
- **Themes** — community CSS themes for the UI shell
- **i18n translations** — community-contributed language files
