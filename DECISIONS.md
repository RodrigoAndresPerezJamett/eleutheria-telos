# Eleutheria Telos — Architectural Decisions

Every significant decision lives here: what was chosen, what was rejected, and why. Before proposing an alternative to anything listed here, read the reasoning — it was probably already considered.

Format:
- **Decision:** what was decided
- **Rejected alternatives:** what else was considered
- **Reason:** why this choice was made
- **Date:** when decided
- **Revisit if:** condition that would warrant reconsidering

---

## D-001 — Tauri 2.x as the desktop shell

**Decision:** Use Tauri 2.x for cross-platform desktop packaging.

**Rejected alternatives:**
- Electron — ~150-200MB binary, includes full Chromium, contradicts the lightweight principle
- Java / JavaFX — JVM startup cost, poor system API interop, largely abandoned ecosystem
- Python + PySide6 (Qt) — viable but harder to ship a small binary, less clean cross-platform story

**Reason:** Tauri produces ~15MB binaries, uses the OS-native WebView (no bundled browser), has native access to system APIs, and supports Android in 2.x. Plugin developers don't need to know Rust — they only interact via HTTP.

**Date:** 2026-03-18

**Revisit if:** Tauri drops Android support or becomes unmaintained.

---

## D-002 — Internal HTTP server (Axum on localhost) instead of Tauri IPC

**Decision:** Run an Axum HTTP server internally on localhost as the bridge between the WebView and the Rust backend.

**Rejected alternatives:**
- Tauri's built-in IPC (invoke commands) — works well for Rust↔JS but plugins in Python/Node cannot use it. HTTP is the universal interface.

**Reason:** Plugins in any language (Python, Node, binary) can make HTTP requests. This makes the plugin system language-agnostic. The same endpoints also serve as the MCP server's foundation. Security is handled via a session token generated at startup.

**Date:** 2026-03-18

**Revisit if:** A plugin system that doesn't require HTTP becomes standardized.

---

## D-003 — HTMX + Alpine.js + Tailwind (no SPA)

**Decision:** Frontend uses HTMX for navigation, Alpine.js for micro-interactions, Tailwind for styling. No SPA framework.

**Rejected alternatives:**
- React — component model encourages shared state, which violates tool independence. Bundle size. Complexity for community contributors.
- Svelte — better than React but still a compiler step. Community devs contributing to a tool shouldn't need to learn a framework.

**Reason:** Each tool is a page. Community plugin developers only need to know HTML and HTTP. HTMX fragments are easy to read, easy to debug, and trivially cacheable. Tailwind keeps the CSS bundle small (~15KB compiled).

**Date:** 2026-03-18

**Revisit if:** A plugin developer experience requiring a JS framework becomes clearly necessary.

---

## D-004 — SQLite as the only local storage

**Decision:** SQLite (via sqlx) for all persistence: notes, clipboard history, settings, plugin data.

**Rejected alternatives:**
- Flat files (Markdown files for notes) — FTS5 full-text search would be impossible without an index. Concurrent access from plugins is error-prone.
- IndexedDB (browser storage) — not accessible from Rust, not accessible by plugins running outside the WebView.
- sled / redb (embedded Rust DBs) — no FTS, no SQL, no ecosystem of tools for inspection.

**Reason:** SQLite has FTS5 for full-text search, is accessible from any language via drivers, has excellent tooling (DB Browser for SQLite), and is a single portable file. Every plugin gets a sandboxed partition via `plugin_data(plugin_id, key, value)`.

**Date:** 2026-03-18

**Revisit if:** A tool requires a data model that fundamentally doesn't fit a relational model.

---

## D-005 — ffmpeg as a subprocess (not a Rust binding)

**Decision:** Invoke ffmpeg as a subprocess via Rust's `std::process::Command`. Do not use Rust ffmpeg bindings.

**Rejected alternatives:**
- `ffmpeg-sys` / `ffmpeg-next` crates — require compiling ffmpeg from source or linking against system libs. Complex build process, license complications, fragile across OS versions.

**Reason:** The system already has ffmpeg 7.1.2 (ffmpeg-free) installed on Nobara. Subprocess invocation is simple, reliable, and lets plugins also call ffmpeg without Rust knowledge. The ffmpeg-free build has all codecs needed for the video processor use case.

**Date:** 2026-03-18

**Revisit if:** Performance requirements demand frame-level access that subprocess can't provide.

---

## D-006 — Python subprocess for Argos Translate and rembg

**Decision:** Run Argos Translate and rembg as Python subprocesses, not embedded via PyO3 or compiled bindings.

**Rejected alternatives:**
- PyO3 (embed Python in Rust) — complex build, version sensitivity, harder for community to modify translation behavior.

**Reason:** Subprocess is simpler, isolates failures (a crashing Python process doesn't crash the app), and lets community contributors modify or replace translation logic without touching Rust. Python 3.14.2 is installed on the system. **Note:** Python 3.14 compatibility of AI packages must be verified individually.

**Date:** 2026-03-18

**Revisit if:** Subprocess startup latency becomes a user-visible problem.

---

## D-007 — Port 47821 as default, with auto-detection fallback

**Decision:** The internal Axum server starts on port 47821. If occupied, it increments until a free port is found.

**Rejected alternatives:**
- Hardcoded port — crashes silently if port is taken.
- Random port — hard to debug, harder for plugins to discover.

**Reason:** 47821 is uncommon enough to rarely conflict. Auto-detection ensures the app always starts. The selected port is stored in session config so plugins and the MCP server can discover it.

**Date:** 2026-03-18

---

## D-008 — System tray as primary app lifecycle model

**Decision:** The app lives in the system tray. Closing the window hides it, does not quit. A global hotkey (configurable) shows/hides the window. Fully quitting is done via tray menu.

**Rejected alternatives:**
- Normal window lifecycle (close = quit) — clipboard history requires the app to be running in the background at all times to capture clipboard changes. A normal window lifecycle makes this impossible without a separate daemon.

**Reason:** The clipboard monitor must run continuously. System tray is the standard pattern for this class of app (cf. 1Password, Raycast, Alfred).

**Date:** 2026-03-18

---

## D-009 — Monetization: open source + 1 ad/day + $5 lifetime

**Decision:** App is open source (MIT or Apache 2.0). Free users see 1 non-intrusive ad per day (shown at app open, auto-dismissed). Paid users pay $5 once via Gumroad for a lifetime license key. License verified locally with asymmetric cryptography.

**Rejected alternatives:**
- Monthly subscription — contradicts the "simple, no friction" ethos.
- Fully paid — reduces community adoption and plugin ecosystem growth.
- Fully free — unsustainable for long-term maintenance.

**Reason:** The $5 one-time model is the lowest-friction ethical monetization for a developer tool. Users who compile from source can remove ads — this is acceptable. Users who pay are supporting the project, not paying to unlock features. Gumroad handles payment and key generation with zero server infrastructure.

**Date:** 2026-03-18

---

## D-010 — No Notion integration in core workflow

**Decision:** Notion is not used for project tracking. GitHub Issues + CHANGELOG.md + DECISIONS.md + IDEAS.md cover all needs.

**Reason:** Everything related to the project should live in the repo itself so Claude Code can read it directly. Notion requires a separate MCP call and creates a split source of truth. Ideas and notes go in IDEAS.md; decisions go in DECISIONS.md; progress goes in CHANGELOG.md; tasks go in GitHub Issues.

**Date:** 2026-03-18

---

## D-011 — Shell served as static file (frontendDist), not from Axum

**Decision:** `ui/index.html` is served by Tauri directly via `frontendDist` (loaded as `tauri://localhost/index.html`). HTMX requests are rewritten at runtime via a `htmx:configRequest` event handler that prepends the Axum base URL (`http://127.0.0.1:{PORT}`) to all relative paths. A `CorsLayer` (tower-http) on Axum allows the WebView origin (`tauri://localhost`) to reach the API server.

**Rejected alternatives:**
- `devUrl: http://localhost:47821` in `tauri.conf.json` — Tauri CLI polls this URL before compiling the Rust binary. First build takes ~2 min (600+ crates); Tauri CLI's hard-coded timeout is 180s. Since Axum is embedded inside the binary, it cannot respond until the binary is compiled. Irresolvable chicken-and-egg problem on first run.

**Reason:** With `frontendDist`, Tauri serves the shell instantly from the bundled binary without any external server dependency. Axum still starts in the background as designed. Dynamic HTMX requests reach Axum via absolute URL rewrite. CORS is required because `tauri://localhost` and `http://127.0.0.1` are different origins.

**Date:** 2026-03-18

**Revisit if:** Tauri CLI exposes a configurable devUrl poll timeout or a `--no-dev-server-wait` flag.
