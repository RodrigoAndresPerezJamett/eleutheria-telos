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

---

## D-012 — FTS5 sync via SQL triggers

**Decision:** Sync the `notes_fts` virtual table via SQL triggers defined in migration 002, not in Rust handler code.

**Rejected alternatives:**
- In-Rust sync (INSERT/DELETE into notes_fts after every notes CRUD operation) — adds boilerplate to every handler, risks divergence if any handler is updated without also updating FTS sync
- Periodic rebuild (`INSERT INTO notes_fts(notes_fts) VALUES ('rebuild')`) — stale search results between rebuilds

**Reason:** SQL triggers are atomic with the DML that fires them. FTS5 is always consistent with the notes table. Zero Rust code required to maintain sync.

**Date:** 2026-03-18

**Revisit if:** FTS5 trigger behavior causes issues across SQLite versions, or if content_fts needs Markdown stripping (Phase 2 — at that point, in-Rust pre-processing before INSERT is still compatible with triggers).

---

## D-013 — Clipboard dedup via DefaultHasher hash

**Decision:** Dedup clipboard entries in the monitor using an in-memory `u64` hash of the clipboard text via `std::hash::DefaultHasher`.

**Rejected alternatives:**
- DB query on every poll (`SELECT content FROM clipboard ORDER BY created_at DESC LIMIT 1`) — adds async overhead to a tight polling loop running every 500ms inside `spawn_blocking`
- SHA-256 — overkill for dedup; collisions are acceptable (worst case: duplicate entry)

**Reason:** Zero I/O per poll cycle after startup seed. Hash fits in a register. DefaultHasher is fast and stdlib-only.

**Date:** 2026-03-18

**Revisit if:** Collision false-positives cause real duplicate suppression (very unlikely with short clipboard text).

---

## D-014 — Clipboard suppress channel via `tokio::sync::watch`

**Decision:** Use `tokio::sync::watch::Sender<u64>` in AppState to suppress the clipboard monitor from re-inserting content that was just recopied via the recopy handler.

**Rejected alternatives:**
- `Mutex<u64>` — works but adds lock contention; watch is designed for single-writer multi-reader broadcast of the latest value
- Skip dedup on recopy entirely — would insert duplicate entries every time a clipboard item is recopied

**Reason:** `watch` is the idiomatic tokio primitive for "broadcast the latest value to interested readers". Monitor uses `has_changed()` + `borrow_and_update()` for non-blocking check.

**Date:** 2026-03-18

**Revisit if:** Multiple simultaneous recopy calls cause a race (only the last hash would be suppressed); acceptable for Phase 1.

---

## D-015 — marked.js bundled under ui/assets/

**Decision:** Bundle `marked.min.js` under `ui/assets/marked.min.js` rather than loading from CDN.

**Rejected alternatives:**
- CDN load (`<script src="https://cdn.jsdelivr.net/npm/marked/...">`) — violates offline-first principle; app would fail to render Markdown previews without internet

**Reason:** Offline-first is a core principle. All static assets must be bundled. The file is ~40KB — negligible.

**Date:** 2026-03-18

**Revisit if:** Asset bundling pipeline is introduced in Phase 5 (at that point, npm/bundler will manage this automatically).

---

## D-016 — Tower oneshot + direct handler calls for tests

**Decision:** Integration tests use `tower::ServiceExt::oneshot()` for non-path-parameterized routes, and direct handler function calls for path-parameterized routes.

**Rejected alternatives:**
- `axum-test v15` — path-parameterized routes return 404 when using `{id}` syntax with axum 0.7 (root cause: matchit 0.7.3 uses `:param` not `{param}`; production routes now use `:id` but tests still use direct handler calls for simplicity)
- `axum-test v19` — targets axum 0.8; would require upgrading axum (forbidden without explicit approval)

**Reason:** Direct handler calls bypass routing and test the business logic (DB operations, FTS sync) which is the meaningful test target. HTTP routing is implicitly tested by running `cargo tauri dev` and using the app.

**Date:** 2026-03-18

**Revisit if:** A version of axum-test is found that correctly handles `:id` path params with axum 0.7.

---

## D-017 — `htmx.config.selfRequestsOnly = false`

**Decision:** Set `htmx.config.selfRequestsOnly = false` in the shell HTML.

**Rejected alternatives:**
- Leave default (`true`) — HTMX 2.0.4 silently drops all cross-origin requests with no error; every HTMX call is blocked

**Reason:** Shell is served from `tauri://localhost` (static file via Tauri frontendDist); Axum runs on `http://127.0.0.1:{PORT}`. These are different origins. HTMX 2.0.4 defaults to blocking cross-origin requests.

**Date:** 2026-03-18

**Revisit if:** Shell and API server are ever on the same origin.

---

## D-018 — HTMX and Alpine.js bundled locally

**Decision:** Bundle `htmx.min.js` and `alpine.min.js` under `ui/assets/`.

**Rejected alternatives:**
- CDN load — violates offline-first principle; WebKitGTK on Linux can be slow or blocked from reaching CDN

**Reason:** Same principle as D-015 (marked.js). All JS dependencies must be available without internet.

**Date:** 2026-03-18

**Revisit if:** Asset bundling pipeline in Phase 5 manages this automatically.

---

## D-019 — Initial panel load via `htmx.ajax()` in `initApp()`

**Decision:** Load the default tool panel using `htmx.ajax()` with a full absolute URL inside `initApp()` on `DOMContentLoaded`.

**Rejected alternatives:**
- `hx-trigger="load"` on `#tool-panel` — fires before `initialization_script` is guaranteed to have set `window.__SESSION_TOKEN__`; token may be undefined on first request

**Reason:** `initApp()` awaits `window.__TAURI__.core.invoke('get_session_token')` to confirm the real token before making any request.

**Date:** 2026-03-18

**Revisit if:** Tauri exposes a synchronous token mechanism.

---

## D-020 — Axum 0.7 route params use `:param` syntax, not `{param}`

**Decision:** All Axum route definitions use `:param` syntax (e.g. `/api/notes/:id`), not `{param}`.

**Rejected alternatives:**
- `{param}` syntax — compiles without error but routes return 404 at runtime; matchit 0.7.3 (the version Axum 0.7.9 depends on) uses `:param` syntax; `{param}` is only supported in matchit 0.8+

**Reason:** Axum 0.7.9 depends on matchit 0.7 which uses `:param` for named parameters. The `{param}` brace syntax was introduced in matchit 0.8. Axum passes route strings directly to matchit without transformation, so using braces results in matchit treating the entire `{param}` as a literal string segment.

**Date:** 2026-03-18

**Revisit if:** Axum is upgraded to 0.8+ (which depends on matchit 0.8+ and uses `{param}` syntax natively).

---

## D-021 — HTMX handlers use `Form<T>`, not `Json<T>`

**Decision:** Axum handlers that receive data from HTMX form submissions or `hx-vals` use `Form<T>` (application/x-www-form-urlencoded), not `Json<T>`.

**Rejected alternatives:**
- `Json<T>` — HTMX sends `hx-vals` and standard form submissions as form-encoded data by default; `Json<T>` returns 415 silently (no HTMX error event is surfaced by default)
- `hx-ext="json-enc"` — would allow JSON, but requires bundling the json-enc HTMX extension; adds complexity for no benefit

**Reason:** HTMX's default content type for POST/PUT is `application/x-www-form-urlencoded`. Mismatching with `Json<T>` fails silently (no visible error in the UI), making it extremely hard to diagnose.

**Exception:** Handlers called by Alpine `fetch()` (like `update_handler` for notes auto-save) explicitly set `Content-Type: application/json` and correctly use `Json<T>`.

**Date:** 2026-03-18

**Revisit if:** The project adopts `hx-ext="json-enc"` globally.

---

## D-023 — Raw string `r#"..."#` terminates at first `"#` in content

**Decision:** Never put `"#` sequences inside `r#"..."#` raw strings. Pre-compute any string that would contain `"#` (e.g. CSS selectors like `#model-card-ID`) before the format! call.

**Rejected alternatives:**
- Use `r##"..."##` (double-hash raw strings) — valid, but requires ensuring no `"##` in content; pre-computing is simpler and more obvious.

**Reason:** `hx-target="#model-card-{id}"` inside `r#"..."#` is silently parsed as: the raw string ends at the first `"#`, and everything after is outside the string. The format! macro then sees malformed syntax and emits "expected `,` found `-`" (a confusing error). Pre-computing `let target = format!("#model-card-{id}")` and using `{target}` in the format string avoids the `"#` sequence inside the raw literal.

**Date:** 2026-03-18

---

## D-024 — Whisper model download via reqwest streaming

**Decision:** Download Whisper ggml model files directly from HuggingFace via `reqwest` with the `stream` feature, using `Response::chunk()` for byte-level progress tracking.

**Rejected alternatives:**
- subprocess download (curl/wget) — no byte-level progress; harder to track and report percentage
- `futures::StreamExt::next()` — requires adding the `futures` crate; `reqwest::Response::chunk()` provides the same streaming without extra deps

**Reason:** `reqwest` is already added for downloads; `chunk()` is the idiomatic async chunk reader that doesn't require `futures::StreamExt`, keeping deps minimal.

**Date:** 2026-03-18

---

## D-025 — Argos Translate models managed via Python subprocess

**Decision:** Use `python3 scripts/install_argos_package.py {from} {to}` to install Argos language packs. Python handles package index fetching and installation. Rust only tracks `downloaded` status in the DB.

**Rejected alternatives:**
- Direct `.argosmodel` file download from Rust — Argos package URLs are managed by their own index (JSON at GitHub); duplicating that logic in Rust is fragile

**Reason:** argostranslate's Python API handles package discovery, download, and installation. Python 3.14 compatible: argostranslate 1.11.0 (pure Python), ctranslate2 4.7.1 (cp314 manylinux wheel), sentencepiece 0.2.1 (cp314 manylinux wheel) — all verified.

**Date:** 2026-03-18

---

## D-026 — scripts/ directory path via compile-time `env!("CARGO_MANIFEST_DIR")`

**Decision:** Resolve Python scripts path using `PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().join("scripts")` at compile time.

**Rejected alternatives:**
- Runtime env var — not set when running as Tauri app binary
- `std::env::current_exe().parent()` — unreliable; the binary location varies across dev/release/Tauri bundle

**Reason:** In dev mode (`cargo tauri dev`), `CARGO_MANIFEST_DIR` is the `src-tauri/` directory — correct path to `../scripts/`. Phase 5 will replace with Tauri's `app.path().resource_dir()` which correctly resolves bundled resources.

**Date:** 2026-03-18

**Revisit if:** App is built for production (Phase 5) — must switch to Tauri resource path and bundle scripts as Tauri resources.

---

## D-027 — argostranslate descartado como backend de traducción en producción

**Decision:** `argostranslate` se usa como backend en Phase 2 (implementado, rutas y scripts en su lugar) pero **no puede ser el backend final** para producción. Será reemplazado en Phase 5.

**Motivo del descarte — dos problemas comprobados en 2026-03-18:**
1. **Incompatibilidad con Python 3.14:** la cadena de dependencias `argostranslate → spacy → thinc → confection → pydantic.v1` falla en runtime. Pydantic V1 no soporta Python 3.14+. Error: `"Core Pydantic V1 functionality isn't compatible with Python 3.14 or greater / unable to infer type for attribute 'REGEX'"`.
2. **Footprint desproporcionado:** `pip3 install argostranslate` descarga ~3 GB (PyTorch 915 MB, CUDA stack completo, spacy, stanza, onnxruntime, 50+ paquetes). Inaceptable para un usuario final.

**Alternativas evaluadas para Phase 5:**
- **`ctranslate2` directo + modelos Opus-MT** — ctranslate2 4.7.1 ya tiene wheel cp314 y es lo que argostranslate usa internamente. Sin la cadena spacy/stanza/pydantic. **Primera opción a evaluar.**
- **LibreTranslate local** — REST API self-hosted, sin dependencias Python, agnóstico a la versión. Requiere que el usuario tenga el servidor corriendo.
- **venv pinned a Python 3.12** — workaround de compatibilidad usando pyenv. Evita reescribir la integración pero añade complejidad de gestión del venv.

**Estado actual:** la UI de traducción, las rutas Axum y `scripts/translate.py` están implementados correctamente. El pipeline OCR→Translate también. Solo falla el subprocess Python en runtime. En cuanto se sustituya el backend Python, todo lo demás funciona sin cambios.

**Date:** 2026-03-18

**Revisit:** Phase 5 — elegir entre ctranslate2 directo o LibreTranslate antes de la release pública.

---

## D-022 — arboard requires `wayland-data-control` feature on Linux

**Decision:** `arboard` is specified as `{ version = "3", features = ["wayland-data-control"] }` in Cargo.toml.

**Rejected alternatives:**
- `arboard = "3"` (no features) — compiles with X11-only backend; on Wayland the monitor silently fails on every poll cycle (`Err(_) => continue`); clipboard history always empty

**Reason:** The machine runs Wayland + Hyprland. Hyprland implements the `wlr-data-control` Wayland protocol. arboard's `wayland-data-control` feature enables the correct backend via `wl-clipboard-rs`. Without it, arboard falls back to X11/XWayland where the real system clipboard is not accessible.

**Date:** 2026-03-18

**Revisit if:** The project adds Windows or macOS support (those platforms don't need this feature; it is Linux-only and Cargo conditionally compiles it).

---

## D-028 — `wf-recorder` as screen recording backend on Wayland/Hyprland

**Decision:** Use `wf-recorder` as a subprocess to capture the screen on Wayland. Stop via `kill -TERM {pid}` so it writes the mp4 trailer cleanly before exiting.

**Rejected alternatives:**
- `ffmpeg -f pipewire` — not compiled in the system's ffmpeg-free build (pipewire input device unavailable)
- `ffmpeg -f kmsgrab` — requires `CAP_SYS_ADMIN` or root; not viable for a user-space app
- `ffmpeg -f x11grab` — X11 only; the machine runs Wayland + Hyprland with no XWayland for screen content

**Reason:** `wf-recorder` uses the wlroots `wlr-screencopy-v1` protocol, which Hyprland implements natively. It is the standard screen recorder for wlroots-based compositors. Available in the Nobara/Fedora repo (`dnf install wf-recorder`). Stopping with SIGTERM (not SIGKILL) ensures the mp4 container is properly finalized.

**Date:** 2026-03-18

**Revisit if:** Adding Windows/macOS support (use `ffmpeg -f gdigrab` on Windows, `ffmpeg -f avfoundation` on macOS — platform-conditional logic in start handler).

---

## D-029 — Photo editor layer system using off-screen canvases outside Alpine

**Decision:** Store each layer as a plain `HTMLCanvasElement` in `window.__peLayers[]` (outside Alpine's reactive proxy) and composite them onto a single visible display canvas on every stroke/redraw.

**Rejected alternatives:**
- Single canvas for all operations — no layer isolation; erasing on one image would destroy pixels from another
- Multiple stacked `<canvas>` elements in the DOM — requires CSS absolute positioning, z-index management, and per-layer pointer-event routing; complex to implement in an HTMX fragment
- Storing canvases inside Alpine `x-data` — Alpine wraps objects in a Proxy on assignment; canvas elements proxied this way lose their `getContext()` method (returns null), breaking all drawing operations

**Reason:** Off-screen canvases in `window.__peLayers[]` bypass Alpine proxying completely while keeping the UI state (layer names, active index) reactive in Alpine. Compositing on every stroke is fast enough for typical photo sizes since we only redraw the display canvas (~microseconds for 2 layers at 4K).

**Date:** 2026-03-19

**Revisit if:** More than ~5 layers are needed, or layer blending modes are added (at that point a proper render loop with `requestAnimationFrame` and dirty-rect compositing would be worth the complexity).

---

## D-030 — Video processor: file path input instead of file upload

**Decision:** Accept the video file as a filesystem path (text input) rather than uploading the file as multipart form data to the local Axum server.

**Rejected alternatives:**
- Multipart file upload — uploading a 1–4GB video file to `http://127.0.0.1` would buffer the entire file in memory inside Axum before ffmpeg can read it; unacceptable memory pressure and latency
- Tauri `dialog.open()` file picker — would require adding `tauri-plugin-dialog` as a dependency with capability configuration; adds complexity for marginal UX gain over a path text field

**Reason:** Since this is a desktop app and ffmpeg reads directly from disk, passing the path is both simpler and more efficient. Power users of a video processing tool are comfortable with file paths. The backend validates path existence before spawning ffmpeg.

**Date:** 2026-03-19

**Revisit if:** A non-technical user workflow is needed (Phase 5 polish), at which point a Tauri dialog plugin can be added cleanly alongside the existing path input.

---

## D-031 — Video processor: separate form field names for compress vs resize resolution

**Decision:** Use `compress_resolution` and `resize_resolution` as distinct form field names instead of a shared `resolution` field.

**Rejected alternatives:**
- Single `resolution` field with both selects — both `<select name="resolution">` elements are in the DOM simultaneously (HTMX/`x-show` uses `display:none`, not `disabled`), so both values are submitted; serde takes an unpredictable one
- JavaScript intercept on submit to disable hidden fields — adds JS complexity and is fragile across HTMX versions

**Reason:** Distinct field names make server-side deserialization unambiguous with zero JavaScript. The backend only reads the relevant field for each operation branch.

**Date:** 2026-03-19

**Revisit if:** A form refactor replaces the dual-select pattern with a single shared field driven by Alpine state.

---

## D-033 — MCP binary shares [dependencies] with the main Tauri package

**Decision:** The `eleutheria-mcp` stdio binary is a `[[bin]]` target within `src-tauri/` (the same Cargo package as the Tauri app). It shares all `[dependencies]` including `reqwest`, `serde_json`, and `tokio`.

**Rejected alternatives:**
- Separate workspace member with its own `Cargo.toml` and minimal deps — cleaner dependency graph, but requires setting up a Cargo workspace, changing build scripts, and complicating the Tauri build pipeline.
- Symlinked or script-based binary — not idiomatic Rust.

**Reason:** Adding a `[[bin]]` entry to the existing `Cargo.toml` is the simplest approach. The binary does NOT `use app_lib::...` anywhere, so the linker only includes what it actually uses (`serde_json`, `tokio`, `reqwest`). Heavy deps like Tauri and Axum are present in `[dependencies]` but not linked into `eleutheria-mcp` because no code in `mcp_stdio.rs` references them.

**Features added to existing deps** (not new crates):
- `tokio` — added `io-std` for async `stdin()`/`stdout()` in the binary
- `reqwest` — added `json` for `Response::json::<Value>()` in the HTTP client

**Date:** 2026-03-19

**Revisit if:** The binary grows significantly or needs deps that conflict with Tauri's deps, at which point extracting it to a separate workspace crate becomes justified.

---

## D-032 — Video processor: libx264 instead of h264_vaapi

**Decision:** Use `libx264 -crf` for compress and resize operations instead of `h264_vaapi -qp`.

**Rejected alternatives:**
- h264_vaapi — initial choice based on CLAUDE.md note ("h264_vaapi encoder available"), but `vainfo` returned empty output on this machine (AMD GPU with open-source mesa driver has no H.264 VAAPI entrypoints); runtime error: `No usable encoding entrypoint found for profile VAProfileH264High`.
- vp9_vaapi — same VAAPI availability issue.
- libvpx-vp9 (software) — available but very slow (10–30× slower than libx264 for HD video).

**Reason:** libx264 is present in Nobara's ffmpeg build (confirmed via `ffmpeg -encoders`), widely compatible, fast with `-preset fast`, and produces MP4 output natively. CRF 18–40 maps to the same quality range as the QP slider already in the UI — no UX change needed.

**Date:** 2026-03-19

**Revisit if:** A machine with confirmed VAAPI H.264 support is targeted (check `vainfo | grep VAProfileH264` before switching back).


---

## D-034 — MCP SSE: loopback HTTP for tool dispatch

**Decision:** The SSE `tools/call` handler dispatches tool calls by making HTTP requests to the same Axum process (`http://127.0.0.1:{port}/api/mcp/...`) using `reqwest`, rather than calling handler functions directly.

**Rejected alternatives:**
- Call handler functions directly — would require extracting them out of Axum's extractor system, duplicating the AppState parameter passing, and making them callable without an HTTP context. Complex and brittle.
- Share the tool dispatch logic as a lib function imported by both SSE and stdio handlers — cleaner in theory, but the tool handlers use Axum extractors (`State`, `Form`, etc.) which are tightly coupled to the HTTP request lifecycle.
- Implement a separate in-process RPC channel — over-engineered for the current scale.

**Reason:** Loopback HTTP reuses the exact same handler code path as external callers. Auth, JSON serialization, error handling, and any future middleware are all exercised consistently. The overhead is negligible (localhost TCP, no serialization mismatch).

**Date:** 2026-03-19

**Revisit if:** Tool calls over SSE show measurable latency (>50ms) from the loopback round-trip — at that point, consider extracting a `call_tool_inner(state, name, args)` function that bypasses HTTP.

---

## D-035 — Captures table deferred pending product decision

**Decision:** Do not build a unified `captures` table yet. OCR results and voice transcriptions remain transient (shown in result cards, discarded on navigation).

**Rejected alternative:** A shared `captures` table joining clipboard, OCR results, and voice transcriptions into a unified timeline. Evaluated during Phase 4.5 sprint review (2026-03-19).

**Reason:** Three blockers:
1. Persisting tool outputs by default requires a new UI surface (browsing, searching, deleting past captures) that doesn't exist and would be significant scope.
2. The existing FTS5 search across `notes` + `clipboard` already covers the "find what I captured" use case for persisted content.
3. This is a product decision ("should tool outputs persist by default?") before it is an architecture decision. Building infrastructure before the product question is answered creates throwaway work.

**Date:** 2026-03-19

**Revisit if:** Beta user feedback consistently shows users want to retrieve past OCR/voice results that they didn't explicitly save. At that point, define the UI surface first, then design the schema.

---

## D-036 — Translation backend: ctranslate2 + Opus-MT replaces argostranslate

**Decision:** Replace argostranslate with ctranslate2 called directly, using Helsinki-NLP/Opus-MT `.ctranslate2` models.

**Rejected alternative:** argostranslate — originally chosen as the offline translation backend (Phase 2). Blocked by two compounding problems discovered on 2026-03-18: (1) Python 3.14 incompatibility via `spacy → thinc → confection → pydantic.v1`; (2) ~3GB dependency footprint from PyTorch + full CUDA stack pulled in transitively.

**Also rejected:** Bundled Python venv with pinned argostranslate — simpler short-term but adds venv lifecycle management (first-run setup, cross-platform activation, path resolution) and still requires ~3GB download. Not worth it when ctranslate2 is a cleaner fix.

**Reason:** ctranslate2 4.7.1 has a confirmed cp314 manylinux wheel. It is what argostranslate uses internally — using it directly eliminates the entire spacy/stanza/pydantic chain. Opus-MT models in `.ctranslate2` format are available from Helsinki-NLP on HuggingFace. The existing `scripts/translate.py`, Axum routes, and UI are all correct — only the Python implementation inside the script changes.

**Date:** 2026-03-19

**Revisit if:** ctranslate2 drops Python 3.14 support or a better offline translation library emerges with a lighter footprint.
