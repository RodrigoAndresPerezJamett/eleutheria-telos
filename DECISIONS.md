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

**Decision:** The app lives in the system tray. Closing the window hides it, does not quit.

**Rejected alternatives:**
- Normal window lifecycle (close = quit) — clipboard history requires the app to be running in the background at all times. A normal window lifecycle makes this impossible without a separate daemon.

**Reason:** The clipboard monitor must run continuously. System tray is the standard pattern for this class of app (cf. 1Password, Raycast, Alfred).

**Date:** 2026-03-18

---

## D-009 — Monetization: open source + 1 ad/day + $5 lifetime

**Decision:** App is open source. Free users see 1 non-intrusive ad per day (shown at app open, auto-dismissed). Paid users pay $5 once via Gumroad for a lifetime license key. License verified locally with asymmetric cryptography.

**Rejected alternatives:**
- Monthly subscription — contradicts the "simple, no friction" ethos.
- Fully paid — reduces community adoption and plugin ecosystem growth.
- Fully free — unsustainable for long-term maintenance.

**Reason:** The $5 one-time model is the lowest-friction ethical monetization for a developer tool. Users who compile from source can remove ads — this is acceptable. Gumroad handles payment and key generation with zero server infrastructure.

**Date:** 2026-03-18

---

## D-010 — No Notion integration in core workflow

**Decision:** Notion is not used for project tracking. GitHub Issues + CHANGELOG.md + DECISIONS.md + IDEAS.md cover all needs.

**Reason:** Everything related to the project should live in the repo itself so both Claude Code and Cursor can read it directly. Notion requires a separate MCP call and creates a split source of truth.

**Date:** 2026-03-18

---

## D-011 — Shell served as static file (frontendDist), not from Axum

**Decision:** `ui/index.html` is served by Tauri directly via `frontendDist`. HTMX requests are rewritten at runtime via `htmx:configRequest` to prepend the Axum base URL. A `CorsLayer` on Axum allows the WebView origin to reach the API server.

**Rejected alternatives:**
- `devUrl: http://localhost:47821` — Tauri CLI polls this URL before compiling the Rust binary. First build takes ~2min; Tauri CLI's timeout is 180s. Axum can't respond until the binary is compiled. Irresolvable chicken-and-egg on first run.

**Reason:** With `frontendDist`, Tauri serves the shell instantly. Axum still starts in the background. Dynamic HTMX requests reach Axum via absolute URL rewrite.

**Date:** 2026-03-18

**Revisit if:** Tauri CLI exposes a configurable devUrl poll timeout.

---

## D-012 — FTS5 sync via SQL triggers

**Decision:** Sync the `notes_fts` virtual table via SQL triggers, not in Rust handler code.

**Reason:** SQL triggers are atomic with the DML that fires them. FTS5 is always consistent with the notes table. Zero Rust code required to maintain sync.

**Date:** 2026-03-18

---

## D-013 — Clipboard dedup via DefaultHasher hash

**Decision:** Dedup clipboard entries in the monitor using an in-memory `u64` hash via `std::hash::DefaultHasher`.

**Reason:** Zero I/O per poll cycle. Hash fits in a register. DefaultHasher is fast and stdlib-only.

**Date:** 2026-03-18

---

## D-014 — Clipboard suppress channel via `tokio::sync::watch`

**Decision:** Use `tokio::sync::watch::Sender<u64>` in AppState to suppress the clipboard monitor from re-inserting content that was just recopied.

**Reason:** `watch` is the idiomatic tokio primitive for "broadcast the latest value to interested readers". Monitor uses `has_changed()` + `borrow_and_update()` for non-blocking check.

**Date:** 2026-03-18

---

## D-015 — marked.js bundled under ui/assets/

**Decision:** Bundle `marked.min.js` under `ui/assets/` rather than loading from CDN.

**Reason:** Offline-first is a core principle. All static assets must be bundled.

**Date:** 2026-03-18

---

## D-016 — Tower oneshot + direct handler calls for tests

**Decision:** Integration tests use `tower::ServiceExt::oneshot()` for non-path-parameterized routes, and direct handler function calls for path-parameterized routes.

**Reason:** Direct handler calls bypass routing and test the business logic (DB operations, FTS sync). HTTP routing is implicitly tested by running `cargo tauri dev` and using the app.

**Date:** 2026-03-18

---

## D-017 — `htmx.config.selfRequestsOnly = false`

**Decision:** Set `htmx.config.selfRequestsOnly = false` in the shell HTML.

**Reason:** Shell is served from `tauri://localhost`; Axum runs on `http://127.0.0.1:{PORT}`. These are different origins. HTMX 2.0.4 defaults to blocking cross-origin requests silently.

**Date:** 2026-03-18

---

## D-018 — HTMX, Alpine.js, and Lucide bundled locally

**Decision:** Bundle `htmx.min.js`, `alpine.min.js`, and `lucide.min.js` under `ui/assets/`.

**Reason:** Offline-first principle. WebKitGTK on Linux can be slow or blocked reaching CDNs. All static assets must be available without internet.

**Date:** 2026-03-18

---

## D-019 — Initial panel load via `htmx.ajax()` in `initApp()`

**Decision:** Load the default tool panel using `htmx.ajax()` with a full absolute URL inside `initApp()` on `DOMContentLoaded`.

**Rejected alternatives:**
- `hx-trigger="load"` on `#tool-panel` — fires before `initialization_script` has set `window.__SESSION_TOKEN__`; token may be undefined on first request.

**Reason:** `initApp()` awaits `window.__TAURI__.core.invoke('get_session_token')` to confirm the real token before making any request.

**Date:** 2026-03-18

---

## D-020 — Axum 0.7 route params use `:param` syntax, not `{param}`

**Decision:** All Axum route definitions use `:param` syntax (e.g. `/api/notes/:id`), not `{param}`.

**Reason:** Axum 0.7.9 depends on matchit 0.7.3 which uses `:param` for named parameters. `{param}` compiles without error but routes return 404 at runtime. This is a silent failure with no warning.

**Date:** 2026-03-18

**Revisit if:** Axum is upgraded to 0.8+ (which uses `{param}` syntax natively).

---

## D-021 — HTMX handlers use `Form<T>`, not `Json<T>`

**Decision:** Axum handlers that receive data from HTMX form submissions or `hx-vals` use `Form<T>` (application/x-www-form-urlencoded), not `Json<T>`.

**Reason:** HTMX's default content type for POST is `application/x-www-form-urlencoded`. Mismatching with `Json<T>` fails silently — Axum returns 415 with no HTMX error event surfaced. This is the most common silent failure in the UI.

**Exception:** Handlers called by Alpine `fetch()` with explicit `Content-Type: application/json` correctly use `Json<T>`.

**Date:** 2026-03-18

---

## D-022 — arboard requires `wayland-data-control` feature on Linux

**Decision:** `arboard = { version = "3", features = ["wayland-data-control"] }` in Cargo.toml.

**Reason:** Without this feature, arboard compiles with X11-only backend. On Wayland + Hyprland, every `get_text()` call fails silently. The monitor loop swallows the error and clipboard history is always empty.

**Date:** 2026-03-18

---

## D-023 — Raw string `r#"..."#` terminates at first `"#` in content

**Decision:** Never put `"#` sequences inside `r#"..."#` raw strings. Pre-compute any string containing `"#` before the `format!` call.

**Reason:** `hx-target="#model-card-{id}"` inside `r#"..."#` is silently parsed as the raw string ending at the first `"#`. The format! macro sees malformed syntax with a confusing error. Pre-computing avoids the `"#` sequence entirely.

**Date:** 2026-03-18

---

## D-024 — Whisper model download via reqwest streaming

**Decision:** Download Whisper ggml model files via `reqwest` with the `stream` feature, using `Response::chunk()` for byte-level progress tracking.

**Reason:** `reqwest` is already added; `chunk()` is the idiomatic async chunk reader that doesn't require `futures::StreamExt`, keeping deps minimal.

**Date:** 2026-03-18

---

## D-025 — Translation models managed via Python subprocess

**Decision:** Use `python3 scripts/install_argos_package.py {from} {to}` to install translation language packs. Python handles index fetching, download, and extraction.

**Reason:** Python subprocess isolates failures. Scripts use `urllib` + `zipfile` — no argostranslate import. Downloads `.argosmodel` ZIP from the Argos model index, extracts CT2 files to `~/.local/share/eleutheria-telos/models/translate/{from}-{to}/`.

**Date:** 2026-03-18 (updated 2026-03-19 — argostranslate replaced by ctranslate2 direct; see D-036)

---

## D-026 — scripts/ directory path via compile-time `env!("CARGO_MANIFEST_DIR")`

**Decision:** Resolve Python scripts path using `PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().join("scripts")` at compile time.

**Reason:** In dev mode, `CARGO_MANIFEST_DIR` is `src-tauri/` — correct path to `../scripts/`. Phase 5 will switch to Tauri's `app.path().resource_dir()` for production bundles.

**Date:** 2026-03-18

**Revisit if:** App is built for production (Phase 5) — must switch to Tauri resource path.

---

## D-028 — `wf-recorder` as screen recording backend on Wayland/Hyprland

**Decision:** Use `wf-recorder` as a subprocess. Stop via `kill -TERM {pid}` so it writes the mp4 trailer cleanly before exiting.

**Rejected alternatives:**
- `ffmpeg -f pipewire` — not compiled in ffmpeg-free build
- `ffmpeg -f kmsgrab` — requires `CAP_SYS_ADMIN`
- `ffmpeg -f x11grab` — X11 only; machine runs Wayland

**Reason:** `wf-recorder` uses the wlroots `wlr-screencopy-v1` protocol, which Hyprland implements natively.

**Date:** 2026-03-18

**Revisit if:** Adding Windows/macOS support.

---

## D-029 — Photo editor layer system using off-screen canvases outside Alpine

**Decision:** Store each layer as a plain `HTMLCanvasElement` in `window.__peLayers[]` (outside Alpine's reactive proxy) and composite onto a single visible display canvas on every stroke.

**Rejected alternatives:**
- Storing canvases inside Alpine `x-data` — Alpine wraps objects in a Proxy; canvas elements proxied this way lose their `getContext()` method (returns null), breaking all drawing operations.

**Reason:** Off-screen canvases in `window.__peLayers[]` bypass Alpine proxying completely while keeping UI state reactive in Alpine.

**Date:** 2026-03-19

---

## D-030 — Video processor: file path input instead of file upload

**Decision:** Accept the video file as a filesystem path (text input) rather than uploading as multipart form data.

**Reason:** Uploading a 1–4GB video to localhost would buffer the entire file in memory inside Axum. Passing the path is simpler and more efficient for a desktop app.

**Date:** 2026-03-19

---

## D-031 — Video processor: separate form field names for compress vs resize resolution

**Decision:** Use `compress_resolution` and `resize_resolution` as distinct form field names.

**Reason:** Both `<select>` elements are in the DOM simultaneously (HTMX/`x-show` uses `display:none`, not `disabled`), so both values are submitted. Distinct field names make server-side deserialization unambiguous with zero JavaScript.

**Date:** 2026-03-19

---

## D-032 — Video processor: libx264 instead of h264_vaapi

**Decision:** Use `libx264 -crf` for compress and resize operations.

**Rejected alternatives:**
- h264_vaapi — `vainfo` returned empty output on this machine (AMD GPU, open-source mesa driver has no H.264 VAAPI entrypoints)
- libvpx-vp9 — very slow (10–30× slower than libx264 for HD video)

**Reason:** libx264 is present in Nobara's ffmpeg build, widely compatible, fast with `-preset fast`.

**Date:** 2026-03-19

**Revisit if:** A machine with confirmed VAAPI H.264 support is targeted.

---

## D-033 — MCP binary shares [dependencies] with the main Tauri package

**Decision:** The `eleutheria-mcp` stdio binary is a `[[bin]]` target within `src-tauri/` (same Cargo package as the Tauri app).

**Reason:** Adding a `[[bin]]` entry to the existing `Cargo.toml` is the simplest approach. Heavy deps like Tauri and Axum are present but not linked into `eleutheria-mcp` because no code in `mcp_stdio.rs` references them.

**Date:** 2026-03-19

---

## D-034 — MCP SSE: loopback HTTP for tool dispatch

**Decision:** The SSE `tools/call` handler dispatches tool calls by making HTTP requests to the same Axum process via `reqwest` rather than calling handler functions directly.

**Reason:** Handler functions use Axum extractors (`State`, `Form`, etc.) tightly coupled to the HTTP request lifecycle. Loopback HTTP reuses the exact same handler code path, including auth, serialization, and error handling.

**Date:** 2026-03-19

---

## D-035 — Captures table deferred pending product decision

**Decision:** Do not build a unified `captures` table. OCR results and voice transcriptions remain transient.

**Reason:** Persisting tool outputs by default requires a new UI surface. FTS5 search across `notes` + `clipboard` already covers the "find what I captured" use case. This is a product decision before it is an architecture decision.

**Date:** 2026-03-19

**Revisit if:** Beta user feedback shows users want to retrieve past OCR/voice results they didn't explicitly save.

---

## D-036 — Translation backend: ctranslate2 + Opus-MT replaces argostranslate

**Decision:** Replace argostranslate with ctranslate2 called directly, using Helsinki-NLP/Opus-MT `.ctranslate2` models.

**Rejected alternative:** argostranslate — blocked by two compounding problems: (1) Python 3.14 incompatibility via `spacy → thinc → confection → pydantic.v1`; (2) ~3GB dependency footprint from PyTorch + full CUDA stack.

**Reason:** ctranslate2 4.7.1 has a confirmed cp314 manylinux wheel. It is what argostranslate uses internally — using it directly eliminates the entire spacy/stanza/pydantic chain. Axum routes, UI, and CLI interface unchanged — only the Python implementation inside the script changes.

**Date:** 2026-03-19 — Implemented 2026-03-19.

Scripts rewritten: `scripts/translate.py`, `scripts/install_argos_package.py`, `scripts/uninstall_argos_package.py`, `scripts/requirements.txt`.

**Revisit if:** ctranslate2 drops Python 3.14 support.

---

## D-037 — Quick Actions: loops are back-edges, not a dedicated node type

**Decision:** No dedicated Loop node. Loops are created by drawing a back-edge from any output port to any previous node. The execution engine detects cycles and enforces a per-pipeline timeout (default 60s warn / 120s kill, configurable).

**Reason:** Matches the user's mental model and simplifies the node type system. A dedicated Loop node would require specifying iteration count, break condition, etc. — complexity that back-edges avoid.

**Date:** 2026-03-19

---

## D-038 — CSS theming: separate file per theme, swapped via `<link>` href

**Decision:** Each theme is a standalone CSS file in `ui/assets/themes/`. The active theme is applied by swapping the `href` of `<link id="theme-link">` in the shell HTML. Theme name persisted to SQLite via `POST /api/settings/ui`. On app load, an inline `<script>` in `<head>` reads `window.__ACTIVE_THEME__` (injected by Axum) and sets the `<link>` href before any paint, eliminating flash of unstyled content.

```html
<link id="theme-link" rel="stylesheet" href="/assets/themes/dark.css">
<script>
  const t = window.__ACTIVE_THEME__ || 'dark';
  document.getElementById('theme-link').href = '/assets/themes/' + t + '.css';
</script>
```

Each theme file defines only `:root` custom properties. The `[data-glass="off"]` block (or `html.no-glass`) is also present in every theme file to handle the glassmorphism toggle.

**Rejected alternatives:**
- `data-theme` attribute on `<html>` (all themes in one file) — scales poorly for community contributions: every contributor edits the same file, merge conflicts multiply, a community member cannot add a theme without understanding the whole file structure.

**Reason:** Community scalability is the primary driver. A contributor adds a theme by copying one file, renaming it, changing color values — zero knowledge of the rest of the codebase required. Theme templates from the internet (VS Code, terminal themes) map directly onto the CSS variable structure.

**Date:** 2026-03-20

---

## D-039 — Quick Actions canvas node visual style

**Decision:** Nodes use flowchart-classic shapes:
- **Action:** pill shape (`border-radius: 999px`), `--accent-subtle` fill, `--accent` border
- **Condition:** diamond (`transform: rotate(45deg)` on container, counter-rotated label inside), amber border variant
- **Trigger:** rounded rectangle + left-side `--accent` accent bar
- **End:** small filled circle in `--text-muted`
- **Edges:** smooth bezier, `--border` stroke, `--accent` on hover; condition edges labelled "true"/"false"

All nodes use `--glass-bg` + `--glass-border` as base surface.

**Rejected alternatives:**
- Rounded cards for all nodes — doesn't visually communicate node type at a glance
- All rounded rectangles (n8n style) — color alone carries too much cognitive load

**Reason:** The pill/diamond distinction is universally understood flowchart notation. Conditions visually "pop" as decision points without requiring color alone.

**Date:** 2026-03-20

**Revisit if:** User testing shows the diamond shape is confusing at small canvas scales.

---

## D-040 — Plugin permission enforcement via Axum middleware

**Decision:** Axum middleware extracts `plugin_id` from the request path, loads the plugin's declared permissions from a `Arc<RwLock<HashMap<String, Vec<String>>>>` cached at startup, and validates the request against a static mapping of `(path_prefix, HTTP_method) → required_permission`. Example: `POST /api/clipboard/*` requires `clipboard.write`, `GET /api/fs/*` requires `fs.user_dir`. Violations return `403 Forbidden` with a JSON body (see D-041).

**Full permission list (V1):**
- `db.read` — read own `plugin_data` rows
- `db.write` — write own `plugin_data` rows
- `clipboard.read` — read clipboard history
- `clipboard.write` — write to clipboard
- `event_bus.subscribe` — listen to Event Bus events
- `event_bus.publish` — publish events to Event Bus
- `fs.user_dir` — read/write/create subdirectories inside `~/eleutheria/plugins/{plugin_id}/` only; path traversal (`../`) rejected with 403
- `ocr.invoke` — call the OCR tool programmatically
- `tts.invoke` — call Voice-to-text programmatically
- `translate.invoke` — call the translation tool programmatically
- `notifications.show` — show system notifications
- `network.outbound` — make outbound HTTP requests (to declared domains only; undeclared targets rejected)

**Date:** 2026-03-29

**Revisit if:** Plugin API surface grows significantly or a granular `network.outbound.allowlist` is needed (Phase 6+).

---

## D-041 — Plugin permission denied response format

**Decision:** When a plugin request is denied due to missing permissions, return HTTP 403 with a JSON body:
```json
{ "error": "permission_denied", "required": "permission_name" }
```

**Rejected alternatives:**
- Silent 403 — harder to debug for plugin developers.

**Reason:** Permission names are public API documentation — exposing them does not reveal internal state. Descriptive errors accelerate plugin development. The attack surface in a localhost-only context is minimal.

**Date:** 2026-03-29

---

## D-042 — Plugin storage quota: 50MB per plugin

**Decision:** 50MB quota per plugin in `plugin_data`. On quota exceeded, return:
```json
{ "error": "storage_quota_exceeded", "used_bytes": N, "limit_bytes": 52428800 }
```
Expose `GET /api/db/plugin/quota` for proactive quota queries. Binary files must use `fs.user_dir`, not `plugin_data`.

**Reason:** Without a limit, a malicious or buggy plugin can exhaust disk space unboundedly. 50MB is generous for structured data and negligible for most use cases.

**Date:** 2026-03-29

---

## D-043 — Quick Actions condition DSL: visual mini-DSL, backend evaluation

**Decision:** Condition nodes use a visual mini-DSL with three fields: `[field] [operator] [value]`. Operators are in natural language: "equal to", "contains", "greater than", "starts with", "is empty". Multiple rules combinable with AND/OR. No JavaScript or expression-language runtime. Evaluation occurs in the backend Rust on the serialized execution context.

**Available contexts (dropdown-driven, not free-text):**
- `trigger.{field}` — payload of the triggering event
- `previous_step.result` — output of the immediately preceding node (hidden if node is first after trigger)
- `clipboard.current` — current clipboard content
- `variable.{name}` — pipeline-level variables defined by the user

The dropdown shows only variables available at that graph position — unavailable contexts are hidden, not grayed out.

**Rejected alternatives:**
- JavaScript eval in frontend — security risk, inconsistent behavior, opaque to non-technical users.
- Rust expression evaluator — requires learning a DSL syntax; visual fields are more accessible.

**Date:** 2026-03-29

---

## D-044 — Pipeline cycle detection: static DFS + runtime step counter

**Decision:** Two layers of cycle protection:
1. **Static DFS at save time** — if a cycle is detected, the save is rejected: cycle nodes are highlighted in red, a clear message is shown, and the save button is disabled until resolved.
2. **Runtime step counter** — cap at 1,000 steps per execution. If exceeded, abort the execution and log the event. This catches dynamic cycles that static analysis cannot predict (e.g., a cycle triggered by runtime data).

**Rejected alternatives:**
- Runtime-only detection with timeout — does not prevent obviously invalid pipeline saves.
- Static-only detection — cannot catch all dynamic execution cycles.

**Date:** 2026-03-29

---

## D-045 — Sync device private key stored in OS keychain

**Decision:** The sync identity private key is stored in the OS keychain (same mechanism as AI API keys: `secret-service` on Linux, Keychain on macOS, Credential Manager on Windows). If the keychain is unavailable on Linux (no gnome-keyring or kwallet), fall back to SQLite with a visible warning in the sync settings UI.

**Rejected alternatives:**
- Plaintext in `sync_identity.private_key TEXT` (original spec) — a private key in an SQLite file is readable by any process with filesystem access; inconsistent with how we treat AI API keys.

**Reason:** The private key is the device's cryptographic identity for peer trust. It must be protected with the same care as user credentials.

**Date:** 2026-03-29

---

## D-046 — Sync conflict resolution: logical sequence numbers primary, modified_at as tiebreak

**Decision:** Logical sequence numbers (per-device monotonic counters) are the primary ordering mechanism for sync. `modified_at` wall-clock timestamp is used **only** as a tiebreak when two changes have the same sequence number (true split-brain). Wall-clock time is never the primary conflict resolution mechanism.

**Rejected alternatives:**
- last-write-wins by `modified_at` as primary — clocks can skew between devices; not causally correct in a P2P system without a central time authority.

**Reason:** Sequence numbers are causally correct — they encode what happened before what, independent of clock drift. Modified_at is stored for display purposes and as a last-resort tiebreak only.

**Date:** 2026-03-29

---

## D-047 — Phase 6 sync scope: all-or-nothing per data type

**Decision:** Phase 6 sync is all-or-nothing per data type (clipboard, notes, captures, photos). Granular sharing by tag or filter is explicitly out of scope. The UI states clearly: "In this version, sync includes all content of the selected type. Filtering by tag will be available in a future update."

**Reason:** Granular sharing requires per-item metadata propagation and a data model (`sync_peers` per-item flags) that significantly increases complexity. The all-or-nothing model covers the most common use cases (personal multi-device and household sharing) without this complexity.

**Date:** 2026-03-29

**Revisit if:** Phase 7 adds conflict resolution UI — at that point, granular sharing metadata is naturally available.

---

## D-048 — Sync peer trust: Trust on First Use (TOFU) with fingerprint display

**Decision:** When a new device is discovered for the first time, the app shows the device name and the SHA-256 fingerprint of its self-signed certificate and asks for confirmation. The approved fingerprint is stored in `sync_peers.public_key`. On all future connections, the fingerprint is verified against the stored value. If it doesn't match, the connection is blocked and the user is alerted with a MITM warning.

**Reason:** TOFU is the same model as SSH (`The authenticity of host X can't be established`) — familiar to technical users, acceptable to non-technical users when explained simply ("confirm this is your other device"). A QR code or code comparison (Signal-style) is Phase 7+ if TOFU proves insufficient.

**Date:** 2026-03-29

---

## D-049 — Production resource paths via app_data_dir; CARGO_MANIFEST_DIR only in tests

**Decision:** `env!("CARGO_MANIFEST_DIR")` is removed from all production code paths. In production, Python scripts are:
1. Bundled as Tauri resources (read-only at install path)
2. Copied to `app.path().app_data_dir()/scripts/` on first launch
3. Referenced from `app_data_dir` at runtime — checked on every launch; if missing (reinstall, update), copied again from the bundle

`env!("CARGO_MANIFEST_DIR")` is kept only in `#[cfg(test)]` blocks.

**Rejected alternatives:**
- Always read from bundle path — bundle resources are read-only; Python scripts may need to be updated independently of the full app.

**Reason:** Tauri 2.x recommends `app_data_dir` for mutable user-local data. Scripts are logically mutable (they can be updated by the app without reinstalling). The copy-on-launch pattern handles reinstalls and updates cleanly.

**Date:** 2026-03-29

---

## D-050 — Event Bus startup: guaranteed init order + pre-ready buffer

**Decision:** Two layers:
1. **Guaranteed initialization order** — main.rs initializes in sequence: (1) Event Bus, (2) all tool subscribers, (3) publishers enabled, (4) system marked "ready". This order is documented with explicit comments in main.rs.
2. **Pre-ready buffer** — during the pre-ready phase, the Event Bus stores up to 50 events in an in-memory `VecDeque`. When a subscriber registers, the buffer is drained toward it. The buffer is discarded N seconds after startup regardless.

**Rejected alternatives:**
- Inflated broadcast channel capacity — doesn't solve the race, just makes it less likely.
- Ignore the problem — pipeline auto-triggers miss events that fire before the engine subscribes, silently.

**Date:** 2026-03-29

---

## D-051 — Replace DefaultHasher with direct comparison for clipboard dedup

**Decision:** Remove `std::hash::DefaultHasher` from the clipboard monitor dedup logic. For text: compare the last inserted item's content directly (string equality, O(n) but negligible for typical clipboard entries). For images: hash the first 4KB with `blake3` (already fast and stdlib-deterministic).

**Reason:** `DefaultHasher` is not guaranteed to be consistent across process restarts or Rust compiler versions. This is documented as acceptable for in-session dedup, but the direct comparison approach is correct and trivially implemented — no reason to keep the hash.

**Date:** 2026-03-29

---

## D-052 — SQL LIKE wildcard escaping with ESCAPE clause

**Decision:** User-supplied search strings are escaped before being wrapped in `%...%` for LIKE queries: replace `\` → `\\`, `%` → `\%`, `_` → `\_`. Query uses `LIKE ? ESCAPE '\'`. Applied to all free-text search endpoints.

**Reason:** Unescaped LIKE wildcards in user input cause incorrect search results (a search for `100%` matches any string containing 3+ characters). sqlx does not escape LIKE wildcards automatically.

**Date:** 2026-03-29

**Note:** For V1.1, consider migrating full-text search to FTS5 throughout, which makes this escaping unnecessary.

---

## D-053 — Port selection: try preferred port once, then OS-assigned port 0

**Decision:** Remove the unbounded port increment loop. Strategy:
1. Try binding `127.0.0.1:47821` once.
2. If it fails, bind `127.0.0.1:0` — the OS assigns a guaranteed-free port immediately.
3. Write the assigned port to `app_data_dir()/server.port` so the Tauri WebView frontend can read it.

**Rejected alternatives:**
- Unbounded loop — can hang indefinitely if the port range is exhausted.
- Random port — unpredictable, harder to debug, harder for MCP clients to discover.

**Reason:** Port 0 is the OS-correct way to request "any free port". The one-attempt-then-fallback pattern preserves compatibility with MCP clients that may have 47821 configured while eliminating the hang risk.

**Date:** 2026-03-29

---

## D-054 — License key format: Ed25519-signed JWT-like payload, per-user

**Decision:** License key payload (signed with Ed25519):
```json
{
  "license_id": "uuid-v4",
  "issued_to": "email@example.com",
  "issued_at": "ISO8601",
  "version": 1,
  "type": "lifetime"
}
```
License is **per-user** (portable across machines, not tied to hardware). The user saves the key string and re-enters it on reinstall. The Ed25519 public key is embedded in the app binary at compile time. No server required for verification.

**Reason:** Per-machine binding creates friction for legitimate reinstalls/upgrades — the most common case for a desktop app. The $5 price point does not justify the support burden of hardware-tied licenses. Key embedded in binary is standard practice for this price point.

**Date:** 2026-03-29

---

## D-055 — RNNoise licensing: BSD-3-Clause, legal for commercial use

**Decision:** RNNoise (BSD-3-Clause) is legal to use in the commercial app. Include attribution in README and the About screen. No additional legal action required.

**Reason:** The patent note in RNNoise's README has no documented enforcement history against any project. The risk for an indie app at this price point is effectively zero. BSD-3-Clause explicitly permits commercial use.

**Date:** 2026-03-29

---

## D-056 — In-memory last result per tool (no DB persistence in Phase 4.7)

**Decision:** Each tool (OCR, Voice) holds the last result in `Arc<Mutex<Option<T>>>` in the Axum `AppState`. Updated on each result, cleared on app exit. Not persisted to DB — this is not the `captures` table. The UI reads this via a `GET /api/{tool}/last` endpoint when the user navigates back to a tool.

**Rejected alternatives:**
- Discard results on navigation (current behavior) — frustrating UX when users accidentally navigate away.
- Full `captures` table (D-035) — premature until beta feedback confirms the need.

**Reason:** The minimum viable "don't lose my work" solution. Zero DB schema changes, zero migration, reversible. If beta feedback demands persistence, the captures table is the correct next step.

**Date:** 2026-03-29

---

## D-057 — /user-files/ contents and import/restore strategy

**Decision:**

**What goes in `/user-files/`:**
- Audio recordings (outputs from Screen/Audio Recorder)
- Processed videos (outputs from Video Processor)
- Exported photo edits (Photo Editor outputs)
- Clipboard images (see D-058)
- NOT: original user files opened for editing (those stay at their original path)

**Import (restore from backup):**
- Two modes: **Merge** (add what doesn't exist; sequence-number-higher version wins conflicts) and **Replace all** (requires explicit confirmation).
- Before merging: read `user_version` from the imported DB; apply pending sqlx migrations via `sqlx::migrate!()` on a copy. If the imported DB's version is newer than the app's current schema version, reject with a clear message. All migrations must be idempotent.

**Date:** 2026-03-29

---

## D-058 — Clipboard content types: 5 types, image as file in /user-files/

**Decision:**

**Final `content_type` values:** `text`, `url`, `html`, `image`, `file`

**Detection priority at capture time:** `image` > `html` > `url` > `file` > `text`

**URL detection:** if the full trimmed string matches a URL regex, `content_type = 'url'`. If the text contains URLs mixed with other content, `content_type = 'text'`.

**Image storage:** save as `user-files/clipboard/{uuid}.png` (not as SQLite blob). The `content` column stores the relative path `clipboard/{uuid}.png`. Retention policy: keep the 50 most recent images or 7 days, whichever is less — auto-delete older ones.

**HTML:** rich text copied from web pages (e.g., `text/html` from arboard) is stored as HTML string in `content`, `content_type = 'html'`. Rendered as sanitized HTML in the clipboard panel.

**Rejected alternatives:**
- Images as SQLite blobs — SQLite is not designed for binary blobs of this size; degrades query performance and backup size.

**Date:** 2026-03-29
