# Eleutheria Telos — Changelog

This file is the project's memory between sessions. It is updated at the end of every work session by Claude Code. Before starting any session, read the most recent entry.

Format per entry:
- **Date** — what was completed, what changed, what was decided, what's next

---

## [2026-03-18] — Project foundation

### Completed
- Created project repository: `rodrigoandresperezjamett/eleutheria-telos`
- Branch structure: `dev` as active development branch, `main` reserved for releases
- Core documentation created: `ARCHITECTURE.md`, `PRINCIPLES.md`, `ROADMAP.md`, `CLAUDE.md`, `CHANGELOG.md`, `DECISIONS.md`, `IDEAS.md`
- Tauri 2.x project initialized with `cargo tauri init`
  - App name: `eleutheria-telos`
  - Window title: `Eleutheria Telos`
  - Web assets path: `../ui`
  - Dev server URL: `http://localhost:47821`
- GitHub MCP configured and verified connected
- Notion MCP verified connected
- Environment verified (see CLAUDE.md → Pinned Environment)

### Environment confirmed working
- Rust 1.92.0, Cargo 1.92.0
- Node 22.20.0, npm 10.9.3
- Tauri CLI 2.10.1
- ffmpeg 7.1.2 (ffmpeg-free — already installed, do not replace)
- Tesseract 5.5.2
- Python 3.14.2 (cutting-edge — verify package support before use)

### Known issues / notes
- ffmpeg-free conflicts with rpmfusion ffmpeg — do not run `sudo dnf install ffmpeg`
- Python 3.14 is newer than most AI packages expect — verify compatibility before adding Python deps

### Next session should start with
Phase 0 — Foundation. Goal: Tauri app running with Axum internal server, HTMX shell navigation, SQLite connected, system tray, and plugin loader skeleton. See ROADMAP.md Phase 0 checklist.
