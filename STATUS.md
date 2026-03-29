# Status — Eleutheria Telos

**This file is the single handoff document between Claude Code, Cursor, and Rodrigo.**
Every session starts by reading this file. Every session ends by updating it.

---

## Current Situation

| | |
|---|---|
| **Mode** | **Solo (Claude Code limit resets 2026-03-25 ~14:00 Santiago)** |
| **Active phases** | 4.5 UI Polish + 4.7 Backlog Sprint |
| **Last Claude Code session** | 2026-03-20 — trash bin, date chunks, note references, drag ghost fix |
| **Last Cursor session** | None yet — sprint starting |
| **Last dev→cursor sync** | Run setup commands in WORKTREE.md if not done |
| **Last cursor→dev merge** | N/A |

**Mode explanation for Cursor:**
- **Solo** = Claude Code limit is active. You handle all tasks regardless of lane. Follow Solo Mode Rules in CURSOR.md.
- **Parallel** = Both tools active. Strict lane separation applies.

Rodrigo updates this field at the start of each week.

---

## Rodrigo's Role — The Three Rituals

### Ritual 1 — Starting any session
```bash
# For a Cursor session:
cd /home/rodrigopj/Projects/eleutheria-telos-cursor
git fetch origin && git merge origin/dev --no-ff -m "chore: sync cursor-sprint with dev"
# Open Cursor. First message: "Read CURSOR.md and STATUS.md. Mode is [Solo/Parallel]. Tell me your plan."

# For a Claude Code session:
cd /home/rodrigopj/Projects/eleutheria-telos
# Open Claude Code. First message: "Read CLAUDE.md and STATUS.md. Tell me what Cursor has done and your plan."
```

### Ritual 2 — Merging after a Cursor task is done
```bash
cd /home/rodrigopj/Projects/eleutheria-telos
git merge cursor-sprint --no-ff -m "feat: [describe what was done]"

# Re-sync cursor branch:
cd /home/rodrigopj/Projects/eleutheria-telos-cursor
git merge origin/dev --no-ff -m "chore: sync after merge"
```

**When to merge:** After each self-contained task, not after every commit. Merge when it's testable.

### Ritual 3 — Triggering the archive
After a merge, if CHANGELOG.md is getting long, tell the active tool: "The merge is done. Archive entries older than 14 days." The tool moves those entries to `CHANGELOG_ARCHIVE.md` and commits.

**Conflict resolution:** Conflicts will only happen in `STATUS.md` and `CHANGELOG.md`. Always keep content from both sides — accept both sections in full.

---

## ⚠️ Important: What Phase 4.5 Already Has

Phase 4.5 Step 1 was **completed on 2026-03-19**. The following already exists — do not recreate:

- `ui/assets/themes/dark.css`, `light.css`, `catppuccin-mocha.css`, `catppuccin-latte.css`, `tokyo-night.css` ✓
- `ui/assets/fonts/inter-variable.woff2` + italic ✓
- `ui/assets/lucide.min.js` (v0.577.0) ✓
- `ui/assets/base.css` — full component system (btn, card, card-glass, input, badge, empty-state, etc.) ✓
- `applyTheme()` and `applyGlass()` global functions in shell ✓
- Sidebar groups (Tools / Media / Plugins), pill active state ✓
- `GET /api/settings/ui` and `POST /api/settings/ui` routes ✓

**What is still needed** is applying the design system to individual panels that still use old Tailwind color classes or don't match the UI_BRIEF spec. See tasks below.

---

## Active Tasks

### Cursor Queue — Phase 4.5 Panel Polish

Work in priority order. Mark `[DONE YYYY-MM-DD]` when complete and committed.

**Audit first (do this before any panel work)**
- [ ] `[CURSOR]` Open `cargo tauri dev`, review each panel visually. For each panel: does it use `btn btn-primary/secondary/ghost/danger` consistently? Does it use `.card`/`.card-glass`? Does it use `.empty-state` for empty states? Note any that still have raw Tailwind color classes (`bg-blue-700`, `bg-gray-800`, etc.) and fix them in priority order below.

**Panels still needing polish (based on last Playwright review)**
- [ ] `[CURSOR]` **Clipboard History** — apply `.card-glass` to items, `btn-ghost` on Copy/Delete/Pin actions, date bucket separator styling (`.text-xs .text-muted uppercase tracking-wider`), hover lift
- [ ] `[CURSOR]` **OCR** — verify empty state uses `.empty-state` class + Lucide icon; result card uses `.card`; action buttons use `btn` variants
- [ ] `[CURSOR]` **Notes** — tag pill chips at card bottom (tags JSON is already in DOM); editor chrome consistency; backlinks panel styling
- [ ] `[CURSOR]` **Models** — progress bar inside card (CSS width from inline style); installed vs available visual distinction
- [ ] `[CURSOR]` **Quick Actions** — apply D-039 node shapes (pill/diamond); step card left-border accent; pipeline list card polish
- [ ] `[CURSOR]` **Video Processor** — operation tab active state using `btn-primary/secondary` toggle
- [ ] `[CURSOR]` **Photo Editor** — toolbar uses `btn` variants consistently; layer strip polish
- [ ] `[CURSOR]` **Translate / Voice** — empty state uses `.empty-state`; result cards use `.card`
- [ ] `[EITHER]` **Settings** — verify theme selector, glass toggle, font selector all work end-to-end; add any missing UI polish
- [ ] `[CURSOR]` **Plugin panels** — consistent chrome (header with `.panel-title`, `.empty-state` for empty, `btn` variants on actions)

---

### Claude Code Queue — Phase 4.7 Backlog

Pick from here when limit resets (2026-03-25). Ordered by impact.

**Safety / correctness — start here**
- [ ] `[CLAUDE]` **Loop quality checks** — 60s toast warn + 120s auto-kill in Quick Actions engine. Both thresholds configurable per-pipeline (`timeout_warn_secs`/`timeout_kill_secs`) and globally in Settings. Files: `quick_actions.rs`, `event_bus.rs`, `server.rs`, Quick Actions panel HTML.
- [ ] `[CLAUDE]` **Plugin permission enforcement** — Axum middleware that validates plugin requests against declared permissions. Currently manifests are loaded but never enforced. See D-040. Files: `plugin_loader.rs`, `server.rs` (new middleware layer).
- [x] `[CLAUDE]` **Port selection fix** — replace unbounded loop with try-47821/fallback-to-0. Write port to `app_data_dir()/server.port`. See D-053. Files: `server.rs`. `[DONE 2026-03-29]`
- [x] `[CLAUDE]` **LIKE wildcard escaping** — escape `%` and `_` in user search input. Affects `mcp.rs`, `clipboard.rs`, `search.rs`, `notes.rs`. Added `like_escape()` in `tools/mod.rs`. See D-052. `[DONE 2026-03-29]`
- [x] `[CLAUDE]` **Replace DefaultHasher in clipboard dedup** — direct string equality for text, blake3 first-4KB for images. `clipboard_suppress_tx` changed to `watch::Sender<String>`. Fixed in `clipboard.rs`, `quick_actions.rs`, `server.rs`, `lib.rs`. See D-051. `[DONE 2026-03-29]`
- [x] `[CLAUDE]` **Trash 30-day TTL job** — `start_trash_ttl_worker` in `db.rs`; runs at startup + every hour; registered in `lib.rs`. `[DONE 2026-03-29]`
- [x] `[CLAUDE]` Fix `tools::translate::tests::test_langs_no_models` — updated assert to match current empty-state HTML. `[DONE 2026-03-29]`
- [x] `[CLAUDE]` Fix `clippy::unnecessary_lazy_evaluations` in `quick_actions.rs` — `.or_else(|| x)` → `.or(x)` at 2 sites. `[DONE 2026-03-29]`

**Usability**
- [ ] `[CLAUDE]` **Quick Actions: opt-in/opt-out toast** — non-blocking toast bottom-right when pipeline auto-fires: name + Accept / Dismiss / "Don't ask again" checkbox; auto-dismissed 8s; per-pipeline "Always run" vs "Ask me first" setting. Files: `event_bus.rs`, `quick_actions.rs`, OCR/Voice result card HTML.
- [x] `[CLAUDE]` **Clipboard: copy button shows `{okay:true}`** — `hx-swap="none"` already present; added `hx-on:htmx:after-request` to show "Copied!" feedback for 1.5s. `[DONE 2026-03-29]`
- [x] `[CLAUDE]` **Clipboard + Notes: full-content DOM bloat** — already fully implemented: 2KB/300-char truncation, `data-*-truncated` flags, `GET /api/clipboard/:id` and `GET /api/notes/:id/content`, fetch on-demand in both modals. `[DONE — was already implemented]`
- [x] `[CLAUDE]` **Name sanity check** — `validate_name()` in `quick_actions.rs` rejects `/\<>"\0`; applied to create/update/folder handlers. `pattern` attribute added to all 3 name inputs (2 in HTML, 1 server-rendered). `[DONE 2026-03-29]`

**Features**
- [x] `[CLAUDE]` **Voice: live recording waveform** — Web Audio API `AnalyserNode` + canvas in `voice/index.html`. Fails silently if mic unavailable. `[DONE 2026-03-29]`
- [ ] `[CLAUDE]` **Clipboard: pin entries** — `is_pinned` column migration, pinned float to top, content-type badge (URL globe, image chip, code `{}` icon).
- [ ] `[CLAUDE]` **Quick Actions: pipeline tree visualizer** — read-only collapsible tree as alternative to canvas; toggle in toolbar.
- [ ] `[CLAUDE]` **Command palette learns** — `command_history` table; Ctrl+K surfaces most-used first.

**Research (no code — documented outcomes only)**
- [ ] `[CLAUDE]` R1 — CDP licensing → `RESEARCH_CDP.md` + DECISIONS.md entry
- [ ] `[CLAUDE]` R2 — Audacity 4 UI study → `RESEARCH_AUDACITY4.md`
- [ ] `[CLAUDE]` R3 — SoundThread feasibility → `RESEARCH_SOUNDTHREAD.md` + DECISIONS.md entry

---

## Blocked

| Task | Blocked by | Who unblocks |
|------|-----------|--------------|
| Glass sidebar visual test | Needs `transparent: true` in `tauri.conf.json` | Claude Code sets it; Cursor tests |
| Notes: `#tag` vs `##heading` parsing conflict | Design decision needed (see IDEAS.md Open Design Questions) | Rodrigo decides |
| Audio tools (Soundboard, Virtual Mic, Audio Editor) | R1–R3 research first | Claude Code |
| Translation end-to-end test | ctranslate2 model download needs verification | Claude Code (Phase 4.6 follow-up) |

---

## Completed

### 2026-03-20
- [x] Notes + Clipboard: trash bin, 30-day retention, restore, Purge Forever
- [x] Notes + Clipboard: date-chunked grid (Today / Yesterday / This Week / This Month / Older)
- [x] Notes: `[[Note Title]]` backlinks, join table, backlink panel, clickable in preview
- [x] Notes: tag drag ghost fix, `user-select:none`
- [x] Notes: Bear-style `#tag` extraction, collapsible tag sidebar, right-click delete
- [x] Quick Actions: pipeline folders (P1), YAML export (P2), YAML import (P3)
- [x] Workflow files added: CURSOR.md, WORKTREE.md, STATUS.md, CHANGELOG_ARCHIVE.md
- [x] .gitignore: `__pycache__/`, `*.pyc` added; pycache removed from tracking
- [x] test-pipeline-h4.yaml moved to `tests/fixtures/`

### 2026-03-19
- [x] **Phase 4.5 Step 1 (App Shell + Design System)** — Inter font, Lucide, 5 themes, base.css, sidebar groups, pill nav, glassmorphism, Settings panel, `applyTheme()`/`applyGlass()`
- [x] Phase 4.5 Step 2 — emoji removal from all panels, btn/input/card design system applied to Quick Actions, Screen Rec, Audio Rec, Video Processor, Photo Editor
- [x] Phase 4.5 complete — Playwright review, all 13 panels signed off
- [x] Phase 4.6 complete — translation backend (ctranslate2), contextual pipeline CTA, pipeline templates, problem-first empty states
- [x] Phase 4.7 H0–H4 — panel nav history, canvas render, pan/zoom, node connect, config panel, undo/redo, graph execution engine
- [x] Notes: grid card UI, lazy-load pagination
- [x] Clipboard: lazy-load pagination
- [x] UI_BRIEF.md approved

### Earlier phases (full detail in CHANGELOG_ARCHIVE.md)
- [x] Phase 0–4: Foundation, Core Tools, AI Tools, Media Tools, MCP + Plugins
