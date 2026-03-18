# Instructions for Claude Code — Eleutheria Telos

Read this file, then ARCHITECTURE.md, PRINCIPLES.md, and CHANGELOG.md before writing any code.

---

## What this project is

Eleutheria Telos is a cross-platform Swiss Army knife desktop app. It bundles everyday tools (clipboard history, notes, OCR, voice-to-text, translation, screen recording, photo editing, video processing) in a single lightweight native app. Offline-first, extensible via plugins, exposes tools to AI agents via MCP.

---

## Pinned Environment — Never Deviate From These

These are the exact versions running on this machine. All dependency decisions must be validated against them.

```
OS:            Nobara Linux (Fedora 43 base), Wayland / Hyprland
Rust:          1.92.0 (edition 2021)
Cargo:         1.92.0
Node.js:       22.20.0
npm:           10.9.3
Tauri CLI:     2.10.1
Tauri:         2.x (stay on 2.x — do NOT suggest upgrades without explicit approval)
ffmpeg:        7.1.2 (ffmpeg-free build — LGPL codecs only, no GPL)
Tesseract:     5.5.2
Python:        3.14.2
Claude Code:   2.1.78
```

**ffmpeg note:** The installed build is `ffmpeg-free`. It excludes some patent-encumbered decoders (H.264 decode disabled, but h264_vaapi encoder available). When implementing the video processor, verify codec availability before assuming support. Do NOT attempt to replace ffmpeg-free — it conflicts with Nobara system packages.

**Python 3.14 note:** Python 3.14 is cutting-edge. Many packages (including some AI libraries) may not officially support it yet. Always verify Python package compatibility with 3.14 before recommending. If a package doesn't support 3.14, propose a `venv` with a pinned version via `pyenv` — never suggest a system-wide downgrade.

---

## Compatibility Protocol — Run This Before Every Dependency Addition

This project has zero dependency conflicts. Keep it that way.

**Before adding any crate to Cargo.toml:**
1. State the crate name, intended pinned version, and why it is needed
2. Verify it compiles with Rust 1.92.0 / edition 2021
3. Verify it is compatible with Tauri 2.x (many crates still target Tauri 1.x)
4. Check for conflicts with crates already in Cargo.toml
5. Confirm the crate was updated in the last 12 months
6. Confirm license: MIT, Apache 2.0, or BSD only — no GPL in core app code
7. Add to Cargo.toml with comment: `# Used for: {reason} | pinned: {date}`

**Never:**
- Add a crate and discover a conflict after writing code that depends on it
- Use `*` or overly loose version ranges (`>=` without upper bound)
- Add a crate speculatively — only add what is needed right now
- Upgrade a pinned version without reading the crate's changelog for breaking changes

---

## Session Workflow

### Starting a session
1. Read CHANGELOG.md — know what was done last and what comes next
2. Read DECISIONS.md — know the constraints that are already locked in
3. Confirm current phase from ROADMAP.md
4. Say: "Ready. Phase X. Last session: [Y]. Today: [Z]."

### Before writing any code
1. List every file that will be created or modified, and why
2. For any new dependency, run the compatibility protocol above first
3. Wait for approval before proceeding
4. If anything is ambiguous, ask — never assume

### During implementation
- Work in small verifiable steps — one thing, confirm it works, then the next
- If something fails, report the exact error and the proposed fix before trying anything
- Never rewrite working code to "improve" it unless explicitly asked
- Never implement Phase N+1 features while working on Phase N

### Ending a session
1. Update CHANGELOG.md — specific: which files changed, what was added/removed/fixed
2. Add new architectural decisions to DECISIONS.md
3. Move any ideas that surfaced to IDEAS.md
4. Create GitHub issues for anything identified but not completed
5. State the recommended next step for the following session

---

## Rules — No Exceptions

### Stack
- Tauri 2.x — desktop shell
- Rust + Axum — internal HTTP server on localhost
- HTMX — frontend navigation (MPA, not SPA)
- Alpine.js — micro-interactions only (toggle visibility, local UI state)
- Tailwind CSS — all styling, no inline styles
- SQLite via sqlx — all local storage
- No React, Vue, Svelte, or any SPA framework — ever

### Architecture
- Tools live in `src-tauri/src/tools/{tool_name}.rs` and `ui/tools/{tool_name}/`
- Tools never import each other — cross-tool communication via Event Bus only
- All user-facing strings in `ui/locales/en.json` — never hardcoded in HTML
- Every HTTP route validates the session token: `Authorization: Bearer {TOKEN}`
- The session token is generated at startup in `server.rs` and injected into WebView

### Code style
- Rust: rustfmt formatting, `?` for error propagation, `Result<T, AppError>` from all handlers
- HTML: semantic elements, Tailwind classes only, no inline styles
- HTMX: always explicit `hx-target`, `hx-swap`, `hx-trigger` — never rely on implicit defaults
- SQL: sqlx macros for compile-time checking — no raw string queries

### Offline-first in code
- Never make a network request without an explicit user action
- Online features check connectivity before attempting — degrade gracefully on failure
- Never crash or show a blank panel on network error

---

## Common Mistakes — Actively Avoid

- Do NOT use `hx-boost` on the app shell — breaks tool isolation
- Do NOT store app state in Alpine.js `$store` — use SQLite
- Do NOT block Axum handler threads — use `tokio::spawn` for Whisper, Tesseract, ffmpeg
- Do NOT hardcode the port — always read from config initialized in `server.rs`
- Do NOT bundle AI models — they are downloaded on-demand
- Do NOT assume ffmpeg codec availability — verify against the ffmpeg-free build
- Do NOT assume Python 3.14 package support — verify first

---

## Git & Versioning

```
main        → production-ready, tagged releases only
dev         → active development, all merges go here
feature/X   → one branch per phase or feature
fix/X       → bug fixes branched from dev
```

Commit format (Conventional Commits — always):
```
feat: add clipboard SQLite schema and monitor
fix: port detection fallback on Linux
chore: pin sqlx 0.8.2 for Rust 1.92 compatibility
docs: update CHANGELOG Phase 0 completion
refactor: extract session token to auth module
```

Versioning: all pre-release is `0.x.x`. `1.0.0` = Phase 5 complete.

Never commit: `eleutheria.db`, `/models/`, `.env`, `target/`, `__pycache__/`, `*.pyc`

---

## GitHub MCP

Available and configured. Use it to:
- Create issues: "create a GitHub issue for X"
- Open PRs: "open a PR from feature/X to dev"
- Check open issues at the start of each session

Labels: `bug`, `feature`, `phase-0` → `phase-7`, `good first issue`, `plugin`, `mcp`, `compatibility`

---

## How to Add a New Tool (Checklist)

1. `src-tauri/src/tools/{tool_name}.rs` — Axum route handlers
2. Register routes in `server.rs`
3. `ui/tools/{tool-name}/index.html` — main panel
4. `ui/tools/{tool-name}/partials/` — HTMX fragments
5. Add sidebar entry to `ui/shell.html`
6. Add Event Bus events to `event_bus.rs`
7. Add MCP tools to `mcp.rs`
8. Add strings to `ui/locales/en.json`
9. Write integration tests for each route handler
10. Update ROADMAP.md checklist and CHANGELOG.md

---

## Testing

- Every Axum route handler: at least one integration test
- DB migrations: tested against in-memory SQLite
- Plugin loader: tested with a mock manifest
- HTMX fragments: assert valid HTML structure

---

## Wayland / Hyprland

This machine uses Wayland + Hyprland. `WEBKIT_DISABLE_DMABUF_RENDERER=1` is already set in hyprland.conf. If the WebView goes blank in dev, that's the fix. Do not suggest X11 workarounds.

---

## GitHub Actions

Three workflows live in `.github/workflows/`. Do not modify them without understanding the impact.

**`ci.yml`** — runs on every push (except main) and every PR. Must pass before merging.
- `cargo fmt --check` — formatting
- `cargo clippy -- -D warnings` — linter, warnings are errors
- `cargo test` — all tests
- Runs on Linux only (fast, cheap)

**`build.yml`** — runs on push to `dev` and `main`.
- Compiles the app on Linux, Windows, and macOS
- Catches cross-platform compile errors before they accumulate
- Uploads a Linux binary artifact for inspection (7-day retention)

**`release.yml`** — runs only on `v*` tags pushed to `main`.
- Builds and bundles for all 3 platforms
- Creates a draft GitHub Release with assets attached
- You review the draft before publishing
- Code signing secrets are commented out until signing is configured

**To cut a release:**
```bash
git checkout main
git merge dev
git tag v0.1.0
git push origin main --tags
```
Then go to GitHub Releases, review the draft, and publish.

**Required GitHub repository secrets (add in repo Settings → Secrets):**
- `TAURI_SIGNING_PRIVATE_KEY` — for update signatures (generate with `cargo tauri signer generate`)
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — password for the above key
- Apple and Windows signing secrets — add when code signing is set up (Phase 5)
