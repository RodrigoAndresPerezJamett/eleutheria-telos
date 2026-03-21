# Instructions for Cursor — Eleutheria Telos

Read this file, then `ARCHITECTURE.md`, `PRINCIPLES.md`, `CHANGELOG.md`, `WORKTREE.md`, and `STATUS.md` before writing any code.

---

## What this project is

Eleutheria Telos is a cross-platform desktop app: a Swiss Army knife for everyday computing tasks. It bundles clipboard history, notes (tags, backlinks, trash), OCR, voice-to-text, translation, screen/audio recording, photo editing, video processing, and a visual automation pipeline system — all offline-first, in a single ~15MB native binary.

**Stack:** Tauri 2.x · Rust + Axum (local HTTP server) · HTMX + Alpine.js + Tailwind CSS (MPA frontend, no SPA) · SQLite via sqlx. No React. No Vue. No Svelte. Ever.

---

## Pinned Environment — Never Deviate

```
OS:            Nobara Linux (Fedora 43 base), Wayland / Hyprland
Rust:          1.92.0 (edition 2021)
Node.js:       22.20.0
Tauri CLI:     2.10.1
Tauri:         2.x — do NOT suggest upgrades
ffmpeg:        7.1.2 (ffmpeg-free, LGPL codecs only — no H.264 decode)
Tesseract:     5.5.2
Python:        3.14.2 — always verify package compatibility before suggesting pip install
```

---

## Two Modes — Read STATUS.md to Know Which Applies

### Mode A: Parallel (Claude Code is also active this week)
Lane separation is strict. You own UI/HTML/CSS. Claude Code owns Rust/migrations/CI.
Full rules in the "Lane Ownership" section below.
STATUS.md will say: **"Mode: Parallel"**

### Mode B: Solo (Claude Code limit is active — you are the only tool)
Lane separation is suspended. You can work on any task in STATUS.md, including backend Rust work.
Extra caution rules apply — see "Solo Mode Rules" below.
STATUS.md will say: **"Mode: Solo (Claude Code limit resets [date])"**

Rodrigo tells you which mode at the start of every session. If STATUS.md doesn't say, ask before touching any Rust files.

---

## Solo Mode Rules (Mode B only)

When Claude Code is unavailable, you handle everything. These rules keep you safe in unfamiliar territory:

1. **Read DECISIONS.md in full before any Rust work.** The hard-won lessons are there. Skipping this is how bugs get introduced.
2. **Dependency protocol before any Cargo.toml change:**
   - State the crate name, version, and why it's needed
   - Verify it compiles with Rust 1.92.0 / edition 2021
   - Verify it is compatible with Tauri 2.x
   - Confirm license: MIT, Apache 2.0, or BSD only — no GPL
   - Check crates.io: last updated within 12 months?
   - Add with comment: `# Used for: {reason} | pinned: {date}`
3. **Never rewrite working Rust code to "improve" it.** Fix the specific issue. Leave everything else alone.
4. **Use `tokio::spawn` for any blocking operation** (Whisper, Tesseract, ffmpeg, Python subprocess). Never block an Axum handler thread.
5. **Route params are `:param` not `{param}`** — `{param}` compiles but returns 404 at runtime. This is D-020 and has burned us before.
6. **HTMX handlers use `Form<T>` not `Json<T>`** — HTMX POSTs are form-encoded. `Json<T>` returns 415 silently. D-021.
7. **Run `cargo clippy -- -D warnings` and `cargo test` before committing any Rust change.** If tests fail, fix them before ending the session.
8. **Document every non-obvious decision in DECISIONS.md** before ending the session. Future Claude Code needs to understand what you did.
9. Pick tasks from STATUS.md in order. Don't skip to Phase N+1 tasks while Phase N tasks are unfinished.

---

## Session Workflow

### Starting a session
1. Run `git fetch origin && git merge origin/dev --no-ff -m "chore: sync"` — always sync first
2. Read `STATUS.md` — confirm the mode (Parallel or Solo) and find the next unchecked task
3. If Parallel: confirm the task is in your lane
4. If Solo: pick the highest-priority task regardless of lane
5. Say: "Mode: [A/B]. Synced from dev [hash]. Task: [X]. Files I will touch: [Y]."

### Before writing any code
1. List every file you will create or modify
2. If touching Rust: run the dependency protocol and state your plan before writing
3. If a task needs clarification: ask Rodrigo before starting

### During implementation
- One task at a time — commit after each completes
- Small verifiable steps — one thing, confirm it works, then the next
- If something fails: report the exact error and proposed fix before trying anything

### Ending a session
1. Mark tasks `[DONE YYYY-MM-DD]` in STATUS.md
2. Append to CHANGELOG.md — files changed, what was added/removed/fixed
3. Commit with a conventional commit message
4. State what is recommended next

---

## Lane Ownership (Mode A — Parallel only)

### You own (Cursor)
- `ui/assets/themes/` — CSS theme files
- `ui/assets/fonts/` — font files
- `ui/tools/*/index.html` — panel HTML
- `ui/tools/*/partials/` — HTMX fragments
- `ui/shell.html` — app shell layout (note: may be `ui/index.html` — check ARCHITECTURE.md)

### Claude Code owns
- `src-tauri/src/` — all Rust source
- `src-tauri/migrations/` — SQLite migrations
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `.github/workflows/`
- `plugins/`
- `ui/locales/` — i18n (Claude Code adds keys; you only use existing ones)
- `ARCHITECTURE.md`, `DECISIONS.md`, `PRINCIPLES.md`, `ROADMAP.md`

### Both tools update (append only, never overwrite)
- `STATUS.md`, `CHANGELOG.md`, `IDEAS.md`

### In Parallel mode: if a UI task requires a new Axum route or schema change
1. Do NOT implement it yourself
2. Add it to STATUS.md Blocked section: "Task X needs route Y — Claude Code implements"
3. Move on to the next task

---

## CSS Theming Architecture (D-038)

Each theme is a **separate CSS file** in `ui/assets/themes/`. The active theme is applied by swapping the `href` of `<link id="theme-link">` in the shell HTML. Community contributors add a theme by copying one file, renaming it, and changing color values — no other files touched.

**Already implemented (Phase 4.5 Step 1, 2026-03-19):** `dark.css`, `light.css`, `catppuccin-mocha.css`, `catppuccin-latte.css`, `tokyo-night.css` exist. `base.css` owns all component styles. `applyTheme()` and `applyGlass()` are global window functions.

**Each theme file defines only `:root` custom properties:**
```css
:root {
  --bg-base, --bg-surface, --bg-elevated, --bg-overlay
  --text-primary, --text-secondary, --text-muted
  --accent, --accent-subtle, --accent-hover
  --border, --border-focus, --shadow, --shadow-lg
  --glass-bg, --glass-blur, --glass-border
  --destructive, --success, --warning (+ -subtle variants)
  --radius-sm: 8px, --radius-md: 12px, --radius-lg: 16px, --radius-xl
}

/* Glass off — in every theme file */
html.no-glass {
  --glass-bg: var(--bg-elevated);
  --glass-blur: none;
  --glass-border: 1px solid var(--border);
}
```

**Switching theme:** `applyTheme(name)` swaps the `<link id="theme-link">` href and POSTs to `/api/settings/ui`.

---

## Quick Actions Canvas — Node Visual Style (D-039)

Nodes use flowchart-classic shapes. All use `--glass-bg` + `--glass-border` as base surface.

| Node type | Shape | Key style |
|-----------|-------|-----------|
| **Trigger** | Rounded rect + left accent bar | 4px `--accent` left border |
| **Action** | Pill (`border-radius: 999px`) | `--accent-subtle` fill, `--accent` border |
| **Condition** | Diamond (`transform: rotate(45deg)`) | Counter-rotated label inside |
| **End** | Small filled circle | `--text-muted` fill |

Edges: smooth bezier, `--border` stroke, `--accent` on hover. Condition edges: "true"/"false" labels in `text-xs --text-muted`. Selected: `--accent` 2px border + `box-shadow: 0 0 0 3px var(--accent-subtle)`.

---

## Component Patterns

**3 button variants only — standardize everywhere:**
- `btn btn-primary` — `--accent` fill, white text, `--radius-md`
- `btn btn-secondary` — `--bg-elevated` fill, `--text-primary`, 1px `--border`
- `btn btn-ghost` — transparent, `--text-secondary`, `--accent-subtle` bg on hover
- Destructive: `btn btn-danger` or `btn-ghost` with `--destructive` color, confirmation state on click

**Cards:** `card` or `card-glass` class from `base.css`. `--glass-bg` fill, `--glass-border`, `--radius-md`, subtle shadow. Hover: `translateY(-1px)`, brighter border.

**Sidebar active item:** pill highlight — `--accent-subtle` bg, `--accent` text. No rectangular highlight.

**Inputs:** `.input` class. `--bg-elevated` fill, `1px --border`, `--radius-sm`. Focus: `2px --border-focus` outline.

**Empty states:** `.empty-state` class. Lucide icon (48px `--text-muted`), title (`text-base font-medium`), subtitle (`text-sm --text-muted`), optional CTA button.

**Status badges:** `.badge` class. Recording: pulsing red dot (`@keyframes pulse`). Downloaded: green chip. Error: red inline text, no toast.

---

## HTMX Rules (Critical — Silent Failures Without These)

- **Always explicit:** `hx-target`, `hx-swap`, `hx-trigger` — never rely on defaults
- **`selfRequestsOnly = false` is already set** in shell — do not change or duplicate it
- **After any `htmx.ajax()` swap:** call `htmx.process(target)` to re-process child `hx-*` attributes
- **HTMX POSTs are form-encoded** — Rust handlers must use `Form<T>`, not `Json<T>`. Button does nothing after click? This is why. (D-021)
- **After any HTMX swap loading Lucide icons:** call `lucide.createIcons()` on the swapped target
- **No CDN assets** — HTMX, Alpine, Lucide are all in `ui/assets/`, bundled locally (D-018)

---

## Typography

**Font:** Inter variable, bundled locally as `ui/assets/fonts/inter-variable.woff2`. Never CDN.

Panel titles: `.panel-title` class (Space Grotesk, `letter-spacing: -0.02em`).
Body/UI: Inter via `base.css`.

| Class | Size | Use |
|-------|------|-----|
| `text-xs` | 11px | Timestamps, labels |
| `text-sm` | 13px | Secondary content, sidebar labels |
| `text-base` | 15px | Body, card content |
| `text-lg` | 17px | Subheadings |
| `text-xl` | 20px | Panel titles |

---

## Rules — No Exceptions

- No hardcoded colors — always CSS variables
- No pure black `#000000` — use `--text-primary`
- No square corners — minimum `--radius-sm` everywhere, including tooltips
- No `<hr>` dividers — use spacing + surface shifts
- No inline `style=""` except where a CSS variable must be set dynamically (e.g. progress bar width)
- No `hx-boost` — breaks tool isolation
- No Alpine `$store` for app state — SQLite only
- No CDN assets — everything must work offline

---

## Rust / Backend Quick Reference (Solo Mode)

Only read this section when in Solo Mode. When in Parallel Mode, these are Claude Code's concern.

**AppError:** All Axum handlers return `Result<impl IntoResponse, AppError>`. Use `?` for propagation.

**Route registration:** Add to the router in `server.rs` via `merge(tool::router())`. Routes use `:param` syntax (not `{param}` — D-020).

**Database:** `sqlx::query!` macros for compile-time checking. Never raw string queries. Migrations in `src-tauri/migrations/` as numbered SQL files.

**Blocking work:** `tokio::spawn` for anything that blocks (ffmpeg, Tesseract, Python subprocess, Whisper). Never block the Axum handler thread directly.

**Python subprocess pattern:**
```rust
tokio::spawn(async move {
    let output = tokio::process::Command::new("python3")
        .arg(script_path)
        .args(&[...])
        .output()
        .await?;
});
```

**HTMX fragment response:** Return `Html<String>` from handlers that HTMX will swap into the DOM.

---

## Git

```
cursor-sprint  ← your branch (commits go here)
dev            ← Claude Code's branch (read-only in Parallel mode; reference in Solo mode)
```

Commit format (Conventional Commits):
```
feat(ui): redesign sidebar with pill active state and section groups
fix(ui): correct glass-blur fallback when glass is disabled
feat(backend): add loop timeout quality checks to quick actions engine
fix(backend): clipboard copy button returns HTML fragment instead of JSON
chore: sync cursor-sprint with dev
```

---

## Wayland / Hyprland

`WEBKIT_DISABLE_DMABUF_RENDERER=1` is set in hyprland.conf. If the WebView goes blank, that's the fix.

`backdrop-filter: blur()` requires `transparent: true` in `tauri.conf.json` (D-038). If glass isn't rendering visually, check STATUS.md Blocked section.

---

## Running the App

The app runs with `cargo tauri dev` from the Claude Code worktree:
```bash
cd /home/rodrigopj/Projects/eleutheria-telos
cargo tauri dev
```

In Solo Mode, you can run this from your own worktree too — `src-tauri/` is present in `cursor-sprint` as it's branched from `dev`.

In Parallel Mode, ask Rodrigo to run it and reload the relevant panel to verify CSS/HTML changes.
