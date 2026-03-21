<div align="center">

<br/>

```
███████╗██╗     ███████╗██╗   ██╗████████╗██╗  ██╗███████╗██████╗ ██╗ █████╗
██╔════╝██║     ██╔════╝██║   ██║╚══██╔══╝██║  ██║██╔════╝██╔══██╗██║██╔══██╗
█████╗  ██║     █████╗  ██║   ██║   ██║   ███████║█████╗  ██████╔╝██║███████║
██╔══╝  ██║     ██╔══╝  ██║   ██║   ██║   ██╔══██║██╔══╝  ██╔══██╗██║██╔══██║
███████╗███████╗███████╗╚██████╔╝   ██║   ██║  ██║███████╗██║  ██║██║██║  ██║
╚══════╝╚══════╝╚══════╝ ╚═════╝    ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝╚═╝  ╚═╝
                                    T E L O S
```

**Your tools. Your data. Your AI. Every device.**

[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache%202.0-blue?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20Android-lightgrey?style=flat-square)](ARCHITECTURE.md)
[![Phase](https://img.shields.io/badge/phase-4.7%20backlog%20sprint-orange?style=flat-square)](ROADMAP.md)
[![Binary size](https://img.shields.io/badge/binary-%3C20MB-green?style=flat-square)](PRINCIPLES.md)

</div>

---

## What is this?

Eleutheria Telos is a cross-platform desktop and mobile app that bundles the everyday tools people reach for constantly — clipboard history, notes, OCR, voice transcription, translation, screen recording, photo editing, video processing, and visual automation pipelines — in a single native binary under 20MB.

It works offline by default. It never sends your data anywhere without your permission. It connects to any AI you choose — a local model, your own API key, or a self-hosted server. And it exposes all its tools to AI agents via a built-in MCP server, so it becomes infrastructure rather than just a utility.

The name means "freedom through purpose."

---

## The problem it solves

Most people's digital lives are fragmented across platforms that don't talk to each other, tools that only run on one OS, and services that require subscriptions to keep your own data accessible.

- Your clipboard history lives on one device and disappears when you restart
- Your notes are in one app, your captured text in another, your transcriptions in a third
- OCR, translation, and voice tools either require internet or are locked to one platform
- Your photos are held hostage by Google and Apple with storage limits that grow more painful every year
- AI features make tools smarter — but only if you're willing to send everything to a cloud provider

Eleutheria Telos is the alternative: **one app, every device, nothing leaves unless you say so.**

---

## North star

> **The only cross-platform utility layer that works everywhere, gets smarter with whatever AI you already use, and keeps everything yours.**

This isn't an "offline AI" pitch. Local models are useful but limited — they can't compete with cloud models on complex reasoning, and pretending otherwise would be dishonest. The real position is simpler: **you control which AI.** Local when you want privacy and have the hardware. Your own API key when you want quality. A self-hosted model when your organization can't send data externally. The app connects to whichever you choose and degrades gracefully when none is configured.

Platforms — Apple, Google, Microsoft — have structural reasons they can never fully replicate this:
- Their revenue requires connectivity and data collection
- They're locked to their own ecosystems
- They won't let you bring your own AI model
- They won't build plugins they can't control

---

## Who it's for

**The privacy-conscious technical user** — developers, sysadmins, researchers. Understands the tradeoffs, wants control over their tools and data, will configure an API key or run Ollama. Underserved by every major platform. Willing to pay for something that respects their choices.

**The cross-platform everyday user** — uses a Windows work laptop, an Android phone, maybe a tablet. Constantly frustrated that their clipboard doesn't follow them, their notes aren't everywhere, their photos are scattered across devices and accumulating storage fees. Doesn't want to learn a new system — just wants things to work.

**The organization that can't use cloud** — legal, medical, research contexts where data leaving the network is not permitted. Needs AI-quality tooling with on-premise models. No current tool serves this without a complex server setup.

---

## Features

### Core tools
| Tool | What it does |
|------|-------------|
| **Clipboard history** | Monitors system clipboard continuously. Everything you copy is saved, searchable, pinnable. Syncs across your devices on the same network. |
| **Notes** | Markdown editor with Bear-style `#inline tags`, Obsidian-style `[[backlinks]]`, trash bin, full-text search (FTS5), tag hierarchy sidebar. |
| **Search** | Full-text search across notes and clipboard history via `Ctrl+K` command palette. |

### AI tools *(offline by default, cloud optional)*
| Tool | Local engine | Cloud fallback |
|------|-------------|----------------|
| **OCR** | Tesseract 5 | Any vision API |
| **Voice to text** | Whisper.cpp (tiny/base/small) | Any speech API |
| **Translation** | ctranslate2 + Opus-MT | DeepL / Google Translate |

### Media tools
| Tool | What it does |
|------|-------------|
| **Screen recorder** | Full screen or window capture with optional mic audio overlay |
| **Audio recorder** | Microphone recording to mp3/wav/ogg/flac |
| **Photo editor** | Layer-based canvas, background removal (rembg, offline), export |
| **Video processor** | Cut, trim, compress, resize, extract audio via ffmpeg — no timeline needed |

### Automation & AI infrastructure
| Feature | What it does |
|---------|-------------|
| **Quick Actions** | Visual drag-and-drop pipeline builder. Connect tools without code: OCR → translate → save to notes. Branches, conditions, cycles. |
| **MCP server** | Exposes all tools to AI agents via stdio and SSE. Claude Code, Cursor, any MCP-compatible agent can call your local OCR, clipboard, notes, and more. |
| **Plugin system** | Any developer can extend the app with a `manifest.json` and an HTTP server in Python, Node, or any binary. |

### Coming
- **Personal memory layer** — local AI queries everything you've ever captured
- **Local-network sync** — clipboard, notes, and captures sync P2P across your devices on the same WiFi, no server
- **Screen context awareness** — opt-in: understand what's on screen and act on it locally
- **Photo library** — local-first photo storage and organization; no storage limits, no subscription, no data leaving your device. For people paying Google Photos every month just to keep their memories.
- **Freehand notes** — draw, annotate, and take notes with mouse, stylus, or finger
- **Virtual microphone / voice effects** — real-time DSP, route processed audio to any app
- **Audio editor** — trim, fade, normalize, multi-track
- **Soundboard** — sound buttons routed through virtual mic for streaming/calls

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri 2.x shell                          │
│         window · tray · global hotkeys · OS permissions         │
│              Desktop (Win/Mac/Linux) + Android                  │
└────────────────────────┬────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────────┐
│                    WebView (frontend)                           │
│         HTMX (navigation) · Alpine.js (interactions)            │
│              Tailwind + base.css (theming system)               │
└────────────────────────┬────────────────────────────────────────┘
                         │  HTTP + session token
                         │  (localhost only)
┌────────────────────────▼───────────────────────────────────────┐
│                   Rust + Axum (backend)                        │
│                                                                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────┐   │
│  │  Core    │  │  AI      │  │  Media   │  │ Plugin proxy  │   │
│  │  tools   │  │  tools   │  │  tools   │  │ + MCP server  │   │
│  └──────────┘  └──────────┘  └──────────┘  └───────────────┘   │
│                                                                │
│              Event Bus · Session auth · i18n                   │
└──────┬──────────────┬───────────────┬──────────────────────┬───┘
       │              │               │                      │
  ┌────▼────┐   ┌─────▼──────┐  ┌─────▼──────┐    ┌──────────▼──┐
  │ SQLite  │   │   Local    │  │Subprocesses│    │   Plugin    │
  │ (sqlx)  │   │  models    │  │ffmpeg · py │    │ processes   │
  └─────────┘   └────────────┘  └────────────┘    └─────────────┘

  ↑ Your AI tier (automatic fallback chain)
  Local model → your API key → self-hosted → prompt to configure
```

**Stack:** Tauri 2.x · Rust · Axum · HTMX · Alpine.js · Tailwind CSS · SQLite (sqlx) · ffmpeg · Whisper.cpp · Tesseract · ctranslate2

**Binary size:** < 20MB · **RAM at idle:** < 80MB · **No bundled AI models** (downloaded on demand)

---

## AI model tiers

The app works across three tiers depending on what you have available. You decide — the app doesn't choose for you.

| Tier | What it uses | Quality | Cost | Privacy |
|------|-------------|---------|------|---------|
| **Local** | Whisper, Tesseract, Opus-MT | Good for transcription, OCR, basic translation | Free | Complete |
| **Your API key** | OpenAI, Anthropic, Gemini, Mistral — whatever you already pay for | Excellent | Your existing subscription | You control what you send |
| **Self-hosted** | Ollama, any OpenAI-compatible endpoint | Excellent on good hardware | Infrastructure cost | Complete |

No tier is required. Every feature degrades gracefully. If you have no API key and no local model for a task, the app tells you clearly and offers to help you set one up — it never fails silently.

---

## Cross-device sync

Sync works peer-to-peer on your local network — no server, no account, no data leaving your premises. Devices on the same WiFi discover each other and sync clipboard, notes, and captures automatically.

```
  your laptop ────────────────── your phone
       │         (same WiFi)         │
       │      P2P · no server        │
       └──────── your tablet ────────┘
```

Cloud sync is optional and user-controlled (Phase 6+). It will always be opt-in.

---

## Plugin system

Any developer can add a tool to Eleutheria Telos with a `manifest.json` and an HTTP server in any language.

```json
{
  "id": "my-tool",
  "name": "My Tool",
  "runtime": "python",
  "entry": "main.py",
  "permissions": ["db.read", "clipboard.read", "event_bus.subscribe"],
  "mcp_tools": [{ "name": "my_action", "description": "Does something useful" }]
}
```

Plugins get: a sandboxed SQLite partition, Event Bus access, an MCP tool slot, and a sidebar entry. No Rust or Tauri knowledge required — just HTTP.

---

## Roadmap summary

| Phase | Status | Focus |
|-------|--------|-------|
| 0 — Foundation | ✅ Complete | Tauri shell, Axum server, SQLite, plugin loader, CI/CD |
| 1 — Core tools | ✅ Complete | Clipboard history, Notes (full system), Search |
| 2 — AI tools | ✅ Complete | OCR, Voice, Translation, Models manager |
| 3 — Media tools | ✅ Complete | Screen/audio recorder, Photo editor, Video processor |
| 4 — MCP + Plugins | ✅ Complete | MCP server, plugin system, Quick Actions |
| 4.5 — UI polish | 🔄 In progress | Glassmorphism design system, 5 themes, panel polish |
| 4.7 — Backlog | 🔄 In progress | Loop safety, pipeline toasts, DOM perf, waveforms |
| 5 — Distribution | ⏳ Next | Installers, license key, onboarding, audio tools |
| 6 — Mobile | ⏳ Planned | Android + tablet, local-network sync, photo library |
| 7 — Screen sharing | ⏳ Planned | WebRTC P2P, screen annotation |

Full detail: [ROADMAP.md](ROADMAP.md)

---

## Core principles

1. **Offline-first** — every feature works without internet by default
2. **Lightweight** — binary under 20MB, idle RAM under 80MB
3. **You control the AI** — local, cloud, or self-hosted; the app connects to your choice
4. **Cross-platform** — same codebase, Windows + macOS + Linux + Android
5. **Your data stays yours** — no telemetry, no required accounts, no cloud dependencies
6. **Plugins are first-class** — anything built-in can be built as a plugin
7. **One language for contributors** — plugin developers need only HTTP and a `manifest.json`

Full detail: [PRINCIPLES.md](PRINCIPLES.md)

---

## Development

```bash
# Prerequisites
# Rust 1.92.0, Node 22.x, Tauri CLI 2.10.1

git clone https://github.com/RodrigoAndresPerezJamett/eleutheria-telos
cd eleutheria-telos
cargo tauri dev
```

For contributors and AI agents working on the project, read [CLAUDE.md](CLAUDE.md) and [ARCHITECTURE.md](ARCHITECTURE.md) first.

---

## License

MIT / Apache 2.0 — your choice. Open source. You can compile without ads. Users who pay do so to support the project, not because they have to.

---

<div align="center">
<sub>Built with Rust · Tauri · HTMX · SQLite · ffmpeg · Whisper · Tesseract</sub>
</div>
