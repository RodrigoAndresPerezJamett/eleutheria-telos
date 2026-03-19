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

- [ ] **MCP server — stdio transport** — all built-in tools accessible as MCP tools
- [ ] **MCP server — SSE transport** — accessible from web-based agent clients
- [ ] **Plugin system — full implementation** — plugins run, routes proxied, permissions enforced, sidebar entry added
- [ ] **Example plugin (Python)** — reference implementation with full manifest
- [ ] **Example plugin (Node.js)** — reference implementation
- [ ] **Plugin developer documentation** — how to build, manifest schema, available permissions, event bus events
- [ ] **Quick Actions (basic)** — connect tool events to actions without code (e.g., auto-translate after OCR)

**Phase 4 is complete when** an AI agent can use the app's tools, and a community developer can build and install a working plugin by following the docs.

---

## Phase 5 — Monetization + Distribution

**Goal:** Make the app distributable to non-technical users.

- [ ] **License key system** — Gumroad integration, asymmetric key verification, no server required
- [ ] **Ad integration** — ethical-ads.io or Carbon Ads, one ad per day on app open, auto-dismissed after 5 seconds
- [ ] **Onboarding flow** — first-run wizard: choose tools to enable, download models, set hotkey
- [ ] **Auto-updater** — Tauri's built-in updater, check for updates on startup (respects offline mode)
- [ ] **Installers** — signed installers for Windows (.msi), macOS (.dmg), Linux (.AppImage + .deb)
- [ ] **Code signing** — Apple Developer + Windows EV certificate (or document manual install workaround for free MVP)
- [ ] **Backup/restore** — export/import user data as ZIP from Settings panel

---

## Phase 6 — Mobile (Android + tablet)

**Goal:** Android app with the 4 most portable tools, working on both phone and tablet.

- [ ] Tauri Android build running and signed
- [ ] Phone layout (<640px): bottom nav with 4 tools: Voice-to-Text, OCR (camera input), Notes, Clipboard
- [ ] Tablet layout (640px–1023px): icon-only sidebar with same 4 tools
- [ ] Background Foreground Service for clipboard monitoring on Android
- [ ] OCR uses device camera as capture source (not screen area)
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
