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
ffmpeg:        7.1.2 (ffmpeg-free, LGPL codecs only)
Python:        3.14.2 — verify package compatibility before any suggestion
```

---

## Worktree — Read This First

You work exclusively in:
```
/home/rodrigopj/Projects/eleutheria-telos-cursor/   ← cursor-sprint branch
```

Never touch `/home/rodrigopj/Projects/eleutheria-telos/` — that is Claude Code's worktree on `dev`.

Full protocol: **WORKTREE.md**.

---

## Session Workflow

### Starting a session
1. Run `git fetch origin && git merge origin/dev --no-ff -m "chore: sync"` — always sync first
2. Read `STATUS.md` — find the next unchecked `[CURSOR]` task in priority order
3. Confirm the task is in your lane (UI/HTML/CSS — see lane ownership below)
4. Say: "Synced from dev [hash]. Task: [X]. Files I will touch: [Y]."

### Before writing any code
1. List every file you will create or modify
2. Confirm none are in Claude Code's lane (`src-tauri/`, migrations, CI, docs)
3. If a task requires a new Axum route or schema change: add it to STATUS.md Blocked section and pick a different task

### During implementation
- One panel or component at a time — commit after each completes
- Never touch `src-tauri/` for any reason
- If you find a bug in HTML rendered by Rust (wrong structure, missing field): document it in STATUS.md and CHANGELOG.md — do not patch the Rust yourself

### Ending a session
1. Mark tasks done in `STATUS.md` with `[DONE YYYY-MM-DD]`
2. Append to `CHANGELOG.md` — files changed, what was added/removed/fixed
3. Commit with a conventional commit message
4. State what is recommended next

---

## Your Lane

**You own:**
- `ui/assets/themes/` — all theme CSS files
- `ui/assets/fonts/` — font files
- `ui/tools/*/index.html` — panel HTML
- `ui/tools/*/partials/` — HTMX fragments
- `ui/shell.html` — app shell

**Do not touch without explicit instruction:**
- `src-tauri/` — anything in here
- `ui/locales/` — add new key needs via STATUS.md for Claude Code
- `.github/workflows/`
- `ARCHITECTURE.md`, `DECISIONS.md`, `PRINCIPLES.md`, `ROADMAP.md`

---

## CSS Theming Architecture (D-038)

Each theme is a **separate CSS file** in `ui/assets/themes/`. The active theme is applied by swapping the `href` of `<link id="theme-link">` in `shell.html`. This is the community-scalable approach: a contributor adds a theme by copying one file, renaming it, and changing the color values — no other files touched.

**File naming:** `dark.css`, `light.css`, `catppuccin-mocha.css`, `catppuccin-latte.css`, `tokyo-night.css`

**Each theme file defines only `:root` custom properties:**
```css
/* ui/assets/themes/dark.css */
:root {
  --bg-base: #13151a;        /* deepest layer — window background */
  --bg-surface: #1e2030;     /* panel content area */
  --bg-elevated: #24273a;    /* cards, inputs */
  --bg-overlay: rgba(30, 32, 48, 0.6);
  --text-primary: #cad3f5;
  --text-secondary: #a5adcb;
  --text-muted: #6e738d;
  --accent: #8aadf4;
  --accent-subtle: rgba(138, 173, 244, 0.15);
  --border: rgba(255, 255, 255, 0.08);
  --shadow: rgba(0, 0, 0, 0.4);
  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --glass-bg: rgba(30, 32, 48, 0.6);
  --glass-blur: blur(16px);
  --glass-border: 1px solid rgba(255, 255, 255, 0.08);
}

/* Glass off — add this block to every theme file */
[data-glass="off"] {
  --bg-overlay: var(--bg-elevated);
  --glass-bg: var(--bg-elevated);
  --glass-blur: none;
  --glass-border: 1px solid var(--border);
}
```

**Theme switching in the Settings panel (Alpine):**
```javascript
async function setTheme(name) {
  document.getElementById('theme-link').href = `/assets/themes/${name}.css`;
  await fetch('/api/settings', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${window.__SESSION_TOKEN__}`,
      'Content-Type': 'application/x-www-form-urlencoded'
    },
    body: `key=active_theme&value=${name}`
  });
}
```

**Glass toggle:**
```javascript
function setGlass(enabled) {
  document.documentElement.dataset.glass = enabled ? 'on' : 'off';
  // also POST to /api/settings key=glass_enabled value=true/false
}
```

**FOUC prevention** — already handled by Claude Code in `shell.html` via an inline script that sets the theme before first paint (reads `window.__ACTIVE_THEME__` injected by Axum). You do not need to implement this — just ensure your theme files are in the correct location.

---

## Quick Actions Canvas — Node Visual Style (D-039)

Nodes use flowchart-classic shapes. All nodes use `--glass-bg` + `--glass-border` as their base surface.

| Node type | Shape | Key style |
|-----------|-------|-----------|
| **Trigger** | Rounded rectangle + left accent bar | `--accent` left border (4px), `--glass-bg` fill |
| **Action** | Pill (fully rounded) | `border-radius: 999px`, `--accent-subtle` fill, `--accent` border |
| **Condition** | Diamond | Container `transform: rotate(45deg)`, label counter-rotated inside |
| **End** | Small filled circle | `--text-muted` fill, no border |

**Edges:** smooth bezier, `--border` stroke, `--accent` on hover. Condition edges labelled "true" / "false" in `text-xs --text-muted`. Selected node: `--accent` border 2px + `box-shadow: 0 0 0 3px var(--accent-subtle)`.

You will apply this styling when working on the Quick Actions panel task. Do not deviate from these shapes without a DECISIONS.md update.

---

## Component Patterns — Use These, Invent Nothing New

**Buttons (3 variants only):**
- `btn-primary` — `--accent` fill, white text, `--radius-md`
- `btn-secondary` — `--bg-elevated` fill, `--text-primary`, 1px `--border`
- `btn-ghost` — transparent, `--text-secondary`, `--accent-subtle` bg on hover
- Destructive: `btn-ghost` with `color: var(--color-danger)`, confirmation state on click

**Cards:**
- `--glass-bg` fill, `--glass-border`, `--radius-md`, `box-shadow: 0 1px 3px var(--shadow)`
- Hover: `translateY(-1px)`, slightly brighter border

**Sidebar active item:** pill highlight — `--accent-subtle` bg, `--accent` left micro-border. No rectangular highlight.

**Section separators in sidebar:** thin `1px --border` line + `text-xs --text-muted uppercase tracking-wider` label. No `<hr>`.

**Inputs:** `--bg-elevated` fill, `1px --border`, `--radius-sm`. Focus: `2px --accent` outline.

**Status badges:**
- Recording: pulsing red dot (`@keyframes pulse`, CSS only)
- Downloaded: green subtle chip
- Downloading: CSS progress bar inside card (no separate indicator)
- Error: red inline text, no toast

---

## HTMX Rules (Hard-Won From Phase 0)

- **Always explicit:** `hx-target`, `hx-swap`, `hx-trigger` — never rely on defaults
- **`selfRequestsOnly = false` is already set** in shell — do not change it
- **After any `htmx.ajax()` swap:** call `htmx.process(target)` to re-process child `hx-*` attributes
- **HTMX POSTs are form-encoded** — Rust handlers use `Form<T>`, not `Json<T>`. If a button does nothing after clicking, this is the first thing to check.
- **Call `lucide.createIcons()`** after every HTMX swap that loads HTML containing Lucide data attributes
- **No CDN assets** — HTMX, Alpine, Lucide are all in `ui/assets/`, bundled locally

---

## Typography

**Font:** Inter, bundled locally under `ui/assets/fonts/inter/`. Never load from Google Fonts or any CDN.

| Class | Size | Use |
|-------|------|-----|
| `text-xs` | 11px | Timestamps, metadata labels |
| `text-sm` | 13px | Secondary content, sidebar labels |
| `text-base` | 15px | Body, card content |
| `text-lg` | 17px | Panel subheadings |
| `text-xl` | 20px | Panel titles |

Weights: `font-normal` body · `font-medium` labels/buttons · `font-semibold` panel titles/section headers · `font-bold` accent numbers only.

---

## Rules — No Exceptions

- No hardcoded colors — always CSS variables
- No pure black `#000000` — use `--text-primary`
- No square corners — minimum `--radius-sm` everywhere, including tooltips
- No `<hr>` dividers — use spacing + surface shifts (DESIGN.md "no-line rule")
- No inline `style=""` — Tailwind classes only, except where a CSS variable needs to be set dynamically (e.g. progress bar width)
- No `hx-boost` — breaks tool isolation
- No Alpine `$store` for app state — SQLite only (via existing HTMX routes)
- No CDN assets — everything must work offline

---

## Git

```
cursor-sprint  ← your branch
dev            ← Claude Code's branch (read-only for you)
```

Commit format (Conventional Commits):
```
feat(ui): redesign sidebar with pill active state and section groups
fix(ui): correct glass-blur fallback when glass setting is disabled
chore(ui): bundle Inter font files under ui/assets/fonts/inter/
style(ui): apply btn-primary variant consistently across all panels
```

---

## Wayland / Hyprland

`WEBKIT_DISABLE_DMABUF_RENDERER=1` is set in hyprland.conf. If the WebView goes blank, that's the fix.

`backdrop-filter: blur()` works in WebKitGTK on Wayland only when the Tauri window has `transparent: true` in `tauri.conf.json`. Claude Code sets this (D-038). If glass backgrounds are not rendering visually, check STATUS.md Blocked section before debugging CSS.
