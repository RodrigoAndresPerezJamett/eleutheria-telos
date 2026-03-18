# Instructions for Claude Code

Read this file, ARCHITECTURE.md, PRINCIPLES.md, and ROADMAP.md before writing any code.

---

## What this project is

Eleutheria Telos is a cross-platform Swiss Army knife desktop app. It bundles multiple everyday tools (clipboard history, notes, OCR, voice-to-text, translation, screen recording, photo editing, video processing) in a single lightweight native app. It is offline-first, extensible via plugins, and exposes its tools to AI agents via the MCP protocol.

---

## Current phase

**Check ROADMAP.md** to confirm the current phase before doing anything. Only implement what is in scope for the current phase. If a feature is in a future phase, do not implement it — create a placeholder and document it.

---

## Before writing any code

1. State which files you will create or modify and why.
2. Wait for approval before proceeding.
3. If a decision is ambiguous, ask — do not assume.

---

## Rules you must always follow

### Stack (no exceptions)
- Tauri 2.x for the desktop shell
- Rust + Axum for the internal HTTP server
- HTMX for frontend navigation (MPA, not SPA)
- Alpine.js only for micro-interactions (toggling UI state, nothing else)
- Tailwind CSS for all styling
- SQLite via sqlx for all local storage
- No React, Vue, Svelte, or any SPA framework — ever

### Architecture
- Every tool lives in `src-tauri/src/tools/{tool_name}.rs` (Rust) and `ui/tools/{tool_name}/` (HTML)
- Tools never import each other — use the Event Bus for cross-tool communication
- All user-facing strings must be in `ui/locales/en.json`, never hardcoded in HTML
- Every HTTP route must validate the session token (`Authorization: Bearer {TOKEN}`)
- The session token is generated at startup in `server.rs` and injected into the WebView

### Dependencies
- Before adding a new crate, check if the standard library or an existing crate solves it
- Add a comment in `Cargo.toml` explaining why each crate is needed
- Never add a crate just because it is popular — it must solve a specific problem in this project

### Code style
- Rust: follow standard `rustfmt` formatting. Use `?` for error propagation. Return `Result<T, AppError>` from all tool handlers
- HTML: use semantic elements. No inline styles (use Tailwind classes only)
- HTMX: use `hx-target`, `hx-swap`, `hx-trigger` explicitly — never rely on implicit defaults
- SQL: all queries via `sqlx` macros for compile-time checking. No raw string queries

### What "offline-first" means in code
- Never make a network request without an explicit user action or a clearly communicated fallback
- Online features (translation APIs, model downloads) must be behind a check: "is network available?"
- If a network request fails, degrade gracefully — never crash, never show a blank panel

---

## Common mistakes to avoid

- **Do not** use `hx-boost` on the entire app shell — it breaks the tool isolation model
- **Do not** store application state in Alpine.js `$store` — use SQLite for persistence, Rust for state
- **Do not** spawn blocking operations on the Axum handler thread — use `tokio::spawn` for heavy tasks (Whisper, Tesseract, ffmpeg)
- **Do not** hardcode the port number anywhere — always read it from the app config initialized in `server.rs`
- **Do not** bundle AI models in the installer — they are downloaded on-demand
- **Do not** add any feature that requires an internet connection without a working offline fallback

---

## File naming conventions

```
Rust modules:        snake_case.rs
HTML tool panels:    ui/tools/{tool-name}/index.html
HTML partials:       ui/tools/{tool-name}/partials/{partial-name}.html
Plugin folder:       plugins/{plugin-id}/
Locale files:        ui/locales/{lang-code}.json
```

---

## How to add a new tool

1. Create `src-tauri/src/tools/{tool_name}.rs` with the tool's Axum route handlers
2. Register the tool's routes in `server.rs`
3. Create `ui/tools/{tool-name}/index.html` for the tool's main panel
4. Create `ui/tools/{tool-name}/partials/` for any HTMX fragments
5. Add the tool's sidebar entry to `ui/shell.html`
6. Add any new Event Bus events to `event_bus.rs`
7. Add any new MCP tools to `mcp.rs`
8. Add the tool's strings to `ui/locales/en.json`
9. Update ROADMAP.md to mark the tool as complete

---

## How to add a new dependency (crate)

1. Explain why it is needed and why existing crates don't solve it
2. Check its license is MIT, Apache 2.0, or BSD (no GPL in core app code)
3. Add it to `Cargo.toml` with a comment: `# Used for: {reason}`
4. If it requires a native system library, document the install requirement for all 3 platforms

---

## MCP tool registration

Every built-in tool that should be accessible to AI agents must be registered in `mcp.rs`. The registration includes:
- Tool name (snake_case)
- Description (clear enough for an AI agent to understand when to use it)
- JSON Schema for input parameters
- The Rust function that handles execution

Plugins register their MCP tools automatically via their `manifest.json` — no changes to `mcp.rs` required for plugin tools.

---

## Git & Versioning

**Branching model:**
```
main          → production-ready only, tagged releases
dev           → active development, all features merge here first
feature/X     → one branch per phase or feature (e.g. feature/phase-0-foundation)
fix/X         → bug fixes branched from dev
```

**Commit format (Conventional Commits — always):**
```
feat: add clipboard history panel
fix: correct port detection on Linux
chore: update sqlx to 0.8
docs: update ROADMAP phase 0 checklist
refactor: extract session token logic to auth module
```

**Versioning (Semantic Versioning):**
- All pre-release work is `0.x.x`
- `1.0.0` is tagged when Phase 5 (Monetization + Distribution) is complete
- Patch: `0.1.1` — bug fixes only
- Minor: `0.2.0` — new tool or significant feature
- Major: `1.0.0` — first public release

**Never commit:**
- `eleutheria.db`
- `/models/` directory
- `.env` files
- `target/` directory (Rust build artifacts)
- `node_modules/` if any JS tooling is added

---

## GitHub MCP

This project uses the GitHub MCP server. It is configured in the project — do not ask to set it up, it should already be available.

With the GitHub MCP you can be asked to:
- Create issues for bugs or tasks: "create an issue for the clipboard crash on Linux"
- Open pull requests: "open a PR from feature/phase-0-foundation to dev"
- List open issues: "what issues are open right now"
- Add labels or milestones to issues

When creating issues, use these labels consistently:
- `bug` — something broken
- `feature` — new functionality
- `phase-0` through `phase-7` — which phase the issue belongs to
- `good first issue` — suitable for community contributors
- `plugin` — related to the plugin system
- `mcp` — related to MCP integration

- Every Axum route handler must have at least one integration test
- Database migrations must be tested against an in-memory SQLite instance
- HTMX fragments must return valid HTML (test with a headless assertion, not visual)
- Plugin loader must be tested with a mock plugin manifest
