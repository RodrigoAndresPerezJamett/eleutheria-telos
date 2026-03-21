#/home/rodrigopj/Downloads/CLAUDE_ADDITIONS.md  Status — Eleutheria Telos

**This file is the single handoff document between Claude Code, Cursor, and Rodrigo.**
Every session starts by reading this file. Every session ends by updating it.

---

## Current Situation

| | |
|---|---|
| **Active phases** | 4.5 UI Polish (Cursor lane) + 4.7 Backlog Sprint (Claude Code lane) |
| **Claude Code limit** | Resets 2026-03-25 ~14:00 Santiago |
| **Last Claude Code session** | 2026-03-20 — trash bin, date chunks, note references, drag ghost fix |
| **Last Cursor session** | None yet — sprint starting |
| **Last dev→cursor sync** | Not yet — run worktree setup below first |
| **Last cursor→dev merge** | N/A |

---

## Worktree Setup (run once if not done)

```bash
cd /home/rodrigopj/Projects/eleutheria-telos
git worktree add -b cursor-sprint ../eleutheria-telos-cursor dev
git worktree list   # verify both appear
```

---

## Rodrigo's Role — The Three Rituals

You are the conductor. The tools do not coordinate with each other automatically. Your job is three small rituals, each takes under a minute.

### Ritual 1 — Starting a Cursor session
```bash
cd /home/rodrigopj/Projects/eleutheria-telos-cursor
git fetch origin && git merge origin/dev --no-ff -m "chore: sync cursor-sprint with dev"
# Open Cursor in that directory
# First message to Cursor: "Read CURSOR.md and STATUS.md. Tell me your plan."
```

### Ritual 2 — Starting a Claude Code session
```bash
cd /home/rodrigopj/Projects/eleutheria-telos
# Open Claude Code there (your normal flow)
# First message: "Read CLAUDE.md and STATUS.md. Tell me what Cursor has done and your plan."
```

### Ritual 3 — Merging after a Cursor task is done
```bash
# When Cursor marks a task [DONE] and you're happy with the result:
cd /home/rodrigopj/Projects/eleutheria-telos
git merge cursor-sprint --no-ff -m "feat(ui): [describe what Cursor did]"

# Then re-sync Cursor's branch so it has the latest Claude Code work too:
cd /home/rodrigopj/Projects/eleutheria-telos-cursor
git merge origin/dev --no-ff -m "chore: sync after merge"
```

**When to merge:** After each self-contained task (e.g. "sidebar redesign done"), not after every commit. Merge when the feature is testable, not merely written.

**Conflict resolution:** Conflicts will only happen in `STATUS.md` and `CHANGELOG.md`. For both: keep all content from both sides — there is never a logical conflict, only a git text conflict. Accept both sections in full.

---

## How the Tools Update This File

**Both tools update STATUS.md at the end of every session.**
They only update the relevant rows — they do not rewrite the whole file.
They must append a "Last updated" timestamp at the top when they do.

---

## Active Tasks

### 🟡 Cursor Queue — Phase 4.5 UI Polish

Work in this exact order. Mark `[DONE YYYY-MM-DD]` when complete and committed.

**Foundation — do these first, everything depends on them**
- [ ] `[CURSOR]` Create `ui/assets/themes/dark.css` — all CSS vars from UI_BRIEF.md §4 + D-038
- [ ] `[CURSOR]` Create `ui/assets/themes/light.css`
- [ ] `[CURSOR]` Create `ui/assets/themes/catppuccin-mocha.css`
- [ ] `[CURSOR]` Create `ui/assets/themes/catppuccin-latte.css`
- [ ] `[CURSOR]` Create `ui/assets/themes/tokyo-night.css`
- [ ] `[CURSOR]` Bundle Inter font under `ui/assets/fonts/inter/` (self-host, no CDN)
- [ ] `[CURSOR]` Confirm `ui/assets/lucide.min.js` present; if not, download and bundle locally
- [ ] `[CURSOR]` Update `ui/shell.html` — sidebar groups (TOOLS/MEDIA/PLUGINS separators), pill active state, glassmorphism, Lucide icons replacing all emojis, `lucide.createIcons()` on DOMContentLoaded + every HTMX swap

**Panels (in order)**
- [ ] `[CURSOR]` Clipboard History — card depth + hover lift, `btn-*` variants on all actions, date bucket separator styling
- [ ] `[CURSOR]` OCR — rich empty state (large Lucide icon + prominent CTA), result card with `btn-*` actions
- [ ] `[CURSOR]` Screen Recorder — pulsing red dot while recording, clear status in header
- [ ] `[CURSOR]` Audio Recorder — same pattern as Screen Recorder
- [ ] `[CURSOR]` Notes — tag pill chips at card bottom, editor chrome consistency
- [ ] `[CURSOR]` Models — CSS progress bar inside model card, installed vs available card distinction
- [ ] `[CURSOR]` Quick Actions — step card richer appearance (left-border accent, node type label), pipeline list polish
- [ ] `[CURSOR]` Video Processor — operation tab layout polish
- [ ] `[CURSOR]` Photo Editor — toolbar polish
- [ ] `[CURSOR]` Translate / Voice — empty state redesign
- [ ] `[EITHER]` Settings — theme selector (swaps `<link>` href per D-038), glass toggle, font selector
- [ ] `[CURSOR]` Plugin panels — consistent chrome (header, empty state, `btn-*` actions)

---

### 🔵 Claude Code Queue — Phase 4.7 Backlog

Pick from here when the limit resets (2026-03-25). Ordered by impact.

**Safety / correctness**
- [ ] `[CLAUDE]` **Loop quality checks** — 60s toast warn + 120s auto-kill in Quick Actions. Spec in ROADMAP.md Phase 4.7. Files: `quick_actions.rs`, `event_bus.rs`, `server.rs` (settings), Quick Actions panel HTML.
- [ ] `[CLAUDE]` Fix `tools::translate::tests::test_langs_no_models` — pre-existing HTML mismatch
- [ ] `[CLAUDE]` Fix `clippy::unnecessary_closure` in `quick_actions.rs`

**Usability**
- [ ] `[CLAUDE]` Quick Actions: opt-in/opt-out toast for auto-triggered pipelines. Full spec in ROADMAP.md Phase 4.7.
- [ ] `[CLAUDE]` Clipboard: copy button shows `{okay:true}` — fix to return HTML fragment or `hx-swap="none"`. See IDEAS.md Bugs.
- [ ] `[CLAUDE]` Clipboard + Notes: full-content DOM bloat fix — truncate server-side, add `GET /api/clipboard/:id` + `GET /api/notes/:id/content`. Spec in ROADMAP.md.

**Features**
- [ ] `[CLAUDE]` Voice: live recording waveform — Web Audio API `AnalyserNode` + canvas. Frontend only, no Rust changes needed.
- [ ] `[CLAUDE]` Clipboard: pin entries + `is_pinned` column migration, content-type badge.
- [ ] `[CLAUDE]` Quick Actions: pipeline tree visualizer — read-only collapsible tree, toggle button in canvas toolbar.
- [ ] `[CLAUDE]` Command palette learns — `command_history` table, Ctrl+K surfaces most-used first.

**Research (no code — written outcomes only)**
- [ ] `[CLAUDE]` R1 — CDP licensing → `RESEARCH_CDP.md` + DECISIONS.md entry
- [ ] `[CLAUDE]` R2 — Audacity 4 UI study → `RESEARCH_AUDACITY4.md`
- [ ] `[CLAUDE]` R3 — SoundThread feasibility → `RESEARCH_SOUNDTHREAD.md` + DECISIONS.md entry

---

## Blocked

| Task | Blocked by | Who unblocks |
|------|-----------|--------------|
| Glass sidebar visual test | `transparent: true` needed in `tauri.conf.json` | Claude Code sets it; Cursor then tests |
| Notes: `#tag` vs `##heading` conflict fix | Design decision needed first (see IDEAS.md Open Design Questions) | Rodrigo decides; both tools may be affected |
| Audio tools (Soundboard, Virtual Mic, Audio Editor) | R1–R3 research must complete first | Claude Code does research |
| Translation (end-to-end) | Phase 4.6 ctranslate2 fix not yet deployed | Claude Code (Phase 4.6) |

---

## Completed

### 2026-03-20 (latest)
- [x] Notes + Clipboard: trash bin with 30-day retention (soft delete, restore, Purge Forever)
- [x] Notes + Clipboard: date-chunked grid (Today / Yesterday / This Week / This Month / Older)
- [x] Notes: `[[Note Title]]` backlinks — join table, backlink panel, clickable in preview
- [x] Notes: tag drag ghost fix, `user-select:none`

### 2026-03-20 (earlier)
- [x] Notes: Bear-style `#tag` inline extraction, collapsible tag sidebar, right-click delete
- [x] Quick Actions: pipeline folders (P1), YAML export (P2), YAML import (P3)

### 2026-03-19
- [x] UI_BRIEF.md approved (full design spec, all panels, component patterns)
- [x] Quick Actions canvas H0–H4 (pan/zoom, node connect, config panel, undo/redo, graph execution engine with cycle detection)
- [x] Notes: grid card UI, lazy-load pagination
- [x] Clipboard: lazy-load pagination

### Foundation (earlier phases — full detail in CHANGELOG.md)
- [x] Phase 0–4: Foundation, Core Tools, AI Tools, Media Tools, MCP + Plugins
