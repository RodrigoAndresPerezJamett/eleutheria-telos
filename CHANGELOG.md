# Eleutheria Telos — Changelog

Contains the **last ~2 weeks of sessions only**. Everything older lives in `CHANGELOG_ARCHIVE.md`.

**Archive policy:** After any `cursor-sprint → dev` merge, Rodrigo tells the active tool "the merge is done." The tool then moves all entries older than 14 days to `CHANGELOG_ARCHIVE.md` and commits. Neither tool archives without that explicit confirmation.

Read the most recent entry before starting any session.

---

## [2026-03-20] — Phase 4.7: Trash bin, date chunks, note references, drag ghost fix

### Completed

**`src-tauri/migrations/010_trash.sql`** — `deleted_at INTEGER DEFAULT NULL` on `notes` and `clipboard`; indexes

**`src-tauri/migrations/011_note_links.sql`** — `note_links (from_id, to_id)` join table; `ON DELETE CASCADE`; derived index rebuilt from `[[Note Title]]` tokens on every save

**`src-tauri/src/tools/notes.rs`**
- `date_bucket()` + `bucket_separator()` — buckets notes by recency
- `render_note_list` — takes `show_buckets: bool`; emits grid-spanning separators between buckets
- `render_trash_note_card()` — dimmed card with Restore + Delete Forever buttons
- `extract_note_refs()` + `sync_note_links()` — scans `[[...]]` tokens, resolves to IDs, rebuilds `note_links`
- All list/tag/filter queries: `AND deleted_at IS NULL`
- New handlers: `restore_handler`, `purge_handler`, `trash_list_handler`, `links_handler`, `resolve_by_title_handler`
- Router: added `/api/notes/trash`, `/api/notes/resolve`, `/api/notes/:id/restore`, `/api/notes/:id/purge`, `/api/notes/:id/links`
- Tests updated; 28 pass

**`src-tauri/src/tools/clipboard.rs`** — same trash/date-bucket pattern; `restore_clipboard_handler`, `purge_clipboard_handler`, `trash_clipboard_handler` added

**`ui/tools/notes/index.html`** — trash button fixed to sidebar bottom; `notesDragStart` creates semi-transparent ghost; `notesApp.init()` handles `notes:find-by-title`

**`ui/tools/clipboard/index.html`** — Trash button added to header

### Known pre-existing failures
- `tools::translate::tests::test_langs_no_models` — HTML mismatch (prior session)
- `clippy::unnecessary_closure` in `quick_actions.rs`

### Next session
Test all features in running app. See STATUS.md for full task queue.

---

## [2026-03-20] — Phase 4.7: Notes tag UX polish

### Completed

**`src-tauri/src/tools/notes.rs`**
- `render_tag_tree` — `data-tag-card` attribute for active highlight; `oncontextmenu` on all tag nodes
- `DeleteTagQuery` + `delete_tag_handler` (`DELETE /api/notes/tags?name=`) — strips tag from all affected notes, calls `sync_note_tags`
- All 15 tests pass

**`ui/tools/notes/index.html`**
- `notesDragStart` — deferred fade so ghost captures at full opacity
- `_applyActiveTag` — applies outline to `[data-tag-card]` container
- `notesTagContextMenu` + `notesDeleteTag` — floating context menu; "Delete #tag"; auto-dismissed on next click

---

## [2026-03-20] — Phase 4.7: Notes Bear-style inline tags

### Completed

**`src-tauri/migrations/009_note_tags.sql`** — `note_tags (note_id, tag)` join table

**`src-tauri/src/tools/notes.rs`**
- `extract_tags(content)` — byte-scan parser; `#tagname`/`#parent/child`; unique lowercase
- `sync_note_tags()` — rebuilds `note_tags` rows + `notes.tags` JSON blob
- `render_tag_tree()` — collapsible sidebar, 2-level hierarchy, Alpine chevron, `hx-get` per tag
- `tags_handler`, tag-filter branch in `list_handler`, `sync_note_tags` called after create/update

---

## [2026-03-20] — Phase 4.7 P1–P3: Pipeline folders, YAML export/import

### Completed
- `migrations/008_pipeline_folders.sql` — `pipeline_folders` table + `folder_id FK` on `pipelines`
- `quick_actions.rs` — folder CRUD; pipeline list groups by folder; move-to-folder; `export_pipeline_handler`; `import_pipeline_handler` (parses YAML, generates fresh UUIDs, remaps edges)
- `serde_yaml = "0.9"` added (MIT, Rust 1.92 compatible)
- Canvas toolbar: Export + Import buttons; Import uses hidden file input + `hx-encoding="multipart/form-data"`

> **YAML format:** `name`, `trigger`, `nodes` (id slug, type, config, pos_x, pos_y), `edges` (source slug, target slug, label). Human-readable slugs in YAML; real UUIDs on import.

---

## [2026-03-20] — Workflow + documentation overhaul

### Completed
- `CURSOR.md`, `WORKTREE.md`, `STATUS.md` added — multi-tool parallel/solo workflow
- `ROADMAP.md` restructured — parallel phase lanes explicit; Phase 4.5 completion criteria defined
- `DECISIONS.md` — D-038 (CSS theming), D-039 (canvas node style) added; D-027 (superseded Spanish duplicate) removed
- `CHANGELOG_ARCHIVE.md` created — all sessions before 2026-03-19 moved there
- `.gitignore` — `__pycache__/`, `*.pyc` added; pycache removed from tracking
- `tests/fixtures/test-pipeline-h4.yaml` — moved from repo root

---

## [2026-03-19] — Phase 4.7: Notes grid card UI + lazy-load pagination

### Completed
- `notes.rs` — responsive card grid (`auto-fill minmax(210px,1fr)`); modal preview with copy-on-click; paginated list (24/page); `IntersectionObserver` sentinel; search bypasses pagination (200 max)
- `clipboard.rs` — paginated list (20/page); sentinel div

---

## [2026-03-19] — Phase 4.7 H4: Graph-aware execution engine

### Completed
- `quick_actions.rs` — full graph traversal (BFS); condition node evaluation; cycle detection with configurable per-pipeline timeout (default 60s warn / 120s kill); backward-compatible with migrated pipelines

> **Loop design (D-037):** No dedicated Loop node. Loops are back-edges. Engine enforces timeout.

---

## [2026-03-19] — Phase 4.7 H3a–H3d: Canvas pan/zoom, node connect, config panel, undo/redo

### Completed
- **H3a** — transform-based canvas; scroll-to-zoom toward cursor; fit-all ⌖; zoom ± buttons
- **H3b** — output/input ports; drag port-to-port to create edge; back-edges for cycles; click-edge-Delete
- **H3c** — 280px right config panel; trigger/action/condition forms; Save PUTs to DB; canvas QoL (spawn position, camera reset on pipeline switch)
- **H3d** — 50-op undo stack; Ctrl+Z/Y/Shift+Z; toolbar ↩↪ with disabled state; covers add/delete node+edges, move node

---

## [2026-03-19] — Phase 4.7 H0–H2: Nav history + Quick Actions canvas foundation

### Completed
- **H0** — back/forward nav (Alt+←/→, mouse side buttons, chevrons in shell header)
- **H1** — `migrations/006_pipeline_graph.sql`: `pipeline_nodes` + `pipeline_edges`; auto-migration of existing `pipeline_steps` to linear chains; full graph CRUD routes
- **H2** — `qaApp()` Alpine canvas component: nodes as draggable divs on 3000×2000 dot-grid, SVG bezier edges, 5 node types, drag-to-reposition, toolbar, run result bar, empty state

---

## [2026-03-19] — Phase 4.5 Complete (Playwright review + OCR fix)

### Completed
- Playwright review infrastructure (`playwright-review/`) — screenshots all 13 panels
- All panels signed off ✓
- OCR panel: controls wrapped in `.card` for consistency

**Phase 4.5 COMPLETE.** → Phase 4.6.

---

## [2026-03-19] — Phase 4.5 Step 2: Panel polish, emoji removal

### Completed
- Emoji removal: translate, quick-actions, screen-recorder, audio-recorder, photo-editor, video-processor panels
- `quick_actions.rs` — all Tailwind color classes replaced with `btn`/`input` design system classes; `trigger_label()`, `tool_icon()` use Lucide HTML strings
- Screen Rec, Audio Rec, Video Processor, Photo Editor — full design system pass (panel-title, card, btn variants, input class)
- `base.css` — `.btn-disabled` added
- Bug fix: `GET /api/settings/ui` SQL fixed to include `pinned` and `sidebar_collapsed` keys (were always returning defaults)

---

## [2026-03-19] — Phase 4.5 Step 1: App Shell + Design System

### Completed

**Assets (all bundled locally — offline-first)**
- `ui/assets/fonts/inter-variable.woff2` + italic
- `ui/assets/lucide.min.js` — Lucide v0.577.0 UMD
- `ui/assets/themes/dark.css`, `light.css`, `catppuccin-mocha.css`, `catppuccin-latte.css`, `tokyo-night.css`
- `ui/assets/base.css` — full component design system

**Theme CSS variables per-theme:** `--bg-base/surface/elevated/overlay`, `--text-primary/secondary/muted`, `--accent/subtle/hover`, `--border/border-focus`, `--shadow/shadow-lg`, `--glass-bg/blur/border`, `--destructive/success/warning` (+subtle), `--radius-sm/md/lg/xl`

**App shell (`ui/index.html`) rewritten:**
- `applyTheme(name)` + `applyGlass(enabled)` global functions
- `initApp()` fetches `/api/settings/ui` on startup to apply saved theme/glass
- Lucide `createIcons()` on DOMContentLoaded + every `htmx:afterSwap`
- Three sidebar groups: Tools / Media / Plugins; pill active nav item; "Eleutheria" logo-dot
- Command palette: glassmorphism box, Lucide search icon

**Backend (`server.rs`):** `GET /api/settings/ui` and `POST /api/settings/ui` added

**Settings panel rewritten:** theme dropdown, glass toggle, font selector; changes applied instantly + persisted

---

## [2026-03-19] — Phase 4.6 Complete

### Completed
- Translation backend: argostranslate → ctranslate2 + sentencepiece (D-036)
- Contextual pipeline CTA: "Create pipeline from this" on OCR + Voice result cards
- Pipeline templates: 5 built-in templates in Quick Actions right panel
- Problem-first empty states across all panels

**Phase 4.6 COMPLETE.**

---

## [2026-03-19] — Phase 4.5 planning + UI_BRIEF.md

### Completed
- `UI_BRIEF.md` created and approved — full design contract (aesthetic, palette, typography, components, priority order)
- `ROADMAP.md` — Phase 4.5 workflow added

---

*Entries before 2026-03-19 are in `CHANGELOG_ARCHIVE.md`*
