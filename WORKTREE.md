# Worktree Coordination Protocol — Eleutheria Telos

This file governs how Claude Code and Cursor share the repository without stepping on each other. Read it at the start of every session.

---

## Why worktrees

Claude Code has a weekly usage limit. When it resets, both tools run in the same week. Git worktrees let them operate on different branches simultaneously — same repository, same git history, different working directories. No stashing. No mid-task branch-switching.

---

## Directory layout

```
/home/rodrigopj/Projects/
├── eleutheria-telos/           ← dev branch — Claude Code's worktree
└── eleutheria-telos-cursor/    ← cursor-sprint branch — Cursor's worktree
```

**One-time setup:**
```bash
cd /home/rodrigopj/Projects/eleutheria-telos
git worktree add -b cursor-sprint ../eleutheria-telos-cursor dev
git worktree list   # verify both appear
```

---

## Branch rules

| Branch | Owner | Rule |
|--------|-------|------|
| `main` | — | Production only. Tagged releases. Never commit directly. |
| `dev` | Claude Code | Active development base. Cursor reads but never commits here. |
| `cursor-sprint` | Cursor | Cursor's working branch. Branched from `dev`. Merged back to `dev` when a task completes. |

---

## Lane ownership

This is the most important section. Conflicts happen when both tools touch the same file.

### Claude Code owns
- `src-tauri/src/` — all Rust source
- `src-tauri/migrations/` — SQLite migrations
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `.github/workflows/`
- `plugins/`
- `ui/locales/` — i18n files (Claude Code adds new keys; Cursor only uses existing ones)
- `ARCHITECTURE.md`, `DECISIONS.md`, `PRINCIPLES.md`, `ROADMAP.md`

### Cursor owns
- `ui/assets/themes/` — all theme CSS files
- `ui/assets/fonts/` — font files
- `ui/tools/*/index.html` — panel HTML
- `ui/tools/*/partials/` — HTMX fragment HTML
- `ui/shell.html` — app shell layout
- `ui/assets/lucide.min.js` — icon library (update only)

### Both tools update (append only, never overwrite)
- `STATUS.md` — sprint handoff
- `CHANGELOG.md` — session history
- `IDEAS.md` — new ideas captured during sessions

---

## Sync protocol

### Step 1 — Cursor pulls from dev at sprint start
```bash
cd /home/rodrigopj/Projects/eleutheria-telos-cursor
git fetch origin
git merge origin/dev --no-ff -m "chore: sync cursor-sprint with dev before sprint"
```
Do this at the start of every Cursor session — never skip it.

### Step 2 — Work normally
Cursor commits to `cursor-sprint`. Claude Code commits to `dev`. Both append to `CHANGELOG.md` and update `STATUS.md`.

### Step 3 — Merge cursor-sprint into dev (Rodrigo runs this)
When Cursor finishes a self-contained task and Rodrigo is happy with the result:

```bash
cd /home/rodrigopj/Projects/eleutheria-telos
git merge cursor-sprint --no-ff -m "feat(ui): [describe what was done]"
```

Then immediately re-sync Cursor's branch:
```bash
cd /home/rodrigopj/Projects/eleutheria-telos-cursor
git merge origin/dev --no-ff -m "chore: sync cursor-sprint after merge"
```

### Conflict resolution
Conflicts will occur in `STATUS.md` and `CHANGELOG.md` — both tools edit them regularly.

**Rule:** Keep all content from both sides. There is never a logical conflict in these files, only a git text conflict. In the editor, select "Accept Both" for every conflicted section, then clean up any duplicate blank lines.

---

## Shared file editing rules

When Claude Code changes a Rust handler's HTML fragment output (e.g. adds a new field to a rendered card), it must:
1. Make the minimal structural HTML change needed
2. Add a note in `CHANGELOG.md`: "Rust fragment updated — Cursor may apply styling to the new element"
3. Not attempt to apply Tailwind classes — leave that to Cursor

When Cursor needs a new Axum route or a schema change to complete a UI task:
1. Do not implement it
2. Add the task to STATUS.md under Blocked: "Cursor task X needs route Y — Claude Code implements"
3. Continue with other tasks that don't require it

---

## Running the app during a Cursor session

The app runs with `cargo tauri dev` from Claude Code's worktree:
```bash
cd /home/rodrigopj/Projects/eleutheria-telos
cargo tauri dev
```

Rodrigo runs this in a terminal. Cursor edits HTML/CSS files and asks Rodrigo to reload the relevant panel in the running app to verify changes. Cursor cannot run the app independently from its own worktree because `src-tauri/` is not in its lane.

For layout and styling work that doesn't need real data, HTML files can be opened directly in a browser for visual preview.
