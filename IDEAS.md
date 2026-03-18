# Eleutheria Telos — Ideas, Desirables & Future Thinking

This is the project's idea bin. Nothing here is scheduled or committed. These are thoughts, "wouldn't it be nice if..." items, community suggestions, and future possibilities. Items graduate to ROADMAP.md when they are prioritized and scoped.

Claude Code: do not implement anything from this file unless it has been explicitly moved to ROADMAP.md and is in the current phase.

---

## UI / UX Ideas

- **Command Palette (`Ctrl+K`)** — global search across all tools and notes, launcher for quick actions. Already planned for MVP but worth noting here as a high-priority idea.
- **Quick Actions / Pipelines** — user-defined chains of tool events without code. Example: "after OCR, auto-translate and copy to clipboard". Event Bus already supports this — just needs a UI.
- **Themes** — community CSS themes for the shell UI. Since everything is HTML + Tailwind, a theme is just an override CSS file.
- **Mini mode** — a compact floating mode for the app (like a widget) that shows only the most-used tool, always on top.
- **Keyboard-first mode** — navigate all tools entirely with keyboard shortcuts, no mouse required.

---

## New Tools (Not in Current Roadmap)

- **Smart Copy** — previously considered and descoped. An OCR overlay that lets the user screenshot any part of the screen and extracts text, links, and media URLs from it. Useful for copying text from videos or images on web pages.
- **Quick File Converter** — convert image to PDF, PDF to text, video to audio, etc. ffmpeg already handles most of this — just needs a UI. Low implementation cost.
- **Reader Mode / Article Saver** — save a URL as clean readable text locally (like Pocket but offline). Useful with translation tool.
- **Password Generator** — simple offline password generator and local encrypted store. No sync, no cloud. SQLite + encryption.
- **Quick Timer / Stopwatch** — minimal but surprisingly often-needed. Trivial to implement. High MCP value: "set a 25-minute timer".
- **Pomodoro** — extends the timer with work/break cycles. Community plugin candidate.
- **Color Picker** — pick any color from anywhere on screen, copy hex/rgb/hsl. Common in developer toolkits.
- **Hash Calculator** — calculate MD5/SHA256/etc of files or text. Common developer utility.
- **JSON / YAML Formatter** — paste JSON, get it formatted. Community plugin candidate.
- **Regex Tester** — test regular expressions with live matching. Community plugin candidate.
- **Base64 Encoder/Decoder** — small utility, high daily use for developers.
- **Diff Tool** — compare two pieces of text. Community plugin candidate.

---

## Media Tools (Phase 3+)

- **Video Timeline Editor** — a visual NLE with multiple tracks. Explicitly a major community plugin project, not a core feature. Estimated effort: months. Would use the Video Processor as its ffmpeg backend.
- **Audio Editor** — trim, fade, normalize audio. ffmpeg-based. Smaller scope than video editor.
- **Batch Image Processing** — resize, convert, compress multiple images at once. rembg for batch background removal.
- **Screen Annotation** — draw, highlight, and annotate on top of live screen content. Overlay window using Tauri's transparent window capabilities.
- **GIF Recorder** — record a short screen region as a GIF. ffmpeg can do this. High demand for sharing demos.

---

## Community / Ecosystem Ideas

- **Plugin Registry** — a hosted JSON file listing community plugins with name, author, version, download URL. The app shows a browsable "Plugin Store" and installs with one click. Requires minimal server infrastructure (static JSON on GitHub Pages would work).
- **Plugin Templates** — `cargo generate` or `npx` templates for scaffolding a new plugin in Python or Node in seconds.
- **Plugin Sandboxing V2** — more granular permissions (e.g., `network.outbound.allowlist: ["api.example.com"]`) and resource limits (CPU, memory) per plugin.
- **Community Translation Files** — i18n contributions from the community. Spanish, Portuguese, French, German, Japanese as first targets given likely user base.

---

## Distribution & Monetization Ideas

- **"Supporter" tier** — optional $10-20 one-time for users who want to give more than the $5 minimum. No extra features — just a "supporter" badge in the about screen.
- **Plugin revenue sharing** — if a plugin marketplace grows, consider a mechanism for plugin developers to monetize their plugins through the same Gumroad-based system.
- **Homebrew / winget / AUR packages** — make installation a one-liner for technical users on each platform.

---

## Mobile-Specific Ideas (Phase 6+)

- **Share Sheet integration (Android)** — accept shared text/images from other apps directly into the app's tools (e.g., share a photo → opens in photo editor).
- **Widget (Android)** — home screen widget for quick note taking or clipboard access.
- **Notification for clipboard** — notify when something notable is copied (e.g., a URL, a phone number) with quick action buttons.

---

## Screen Sharing (Phase 7)

- **P2P via PeerJS** — 6-character room code, no accounts, no server. Works for 1-on-1.
- **Small group (2-5 people)** — LiveKit Cloud free tier or self-hosted mediasoup as SFU.
- **Screen annotation during share** — draw on the shared screen in real time. High value for teaching/tutoring use cases.
- **Recorded session sharing** — automatically save a screen share session locally for later reference.

---

## AI / MCP Ideas

- **Local LLM integration** — connect to Ollama (already has a config entry in Claude Code's config) for AI-powered notes summarization, smart search, etc. Fully offline.
- **AI-powered Quick Actions** — instead of user-defined pipelines, use a small local model to suggest "next action" after a tool completes (e.g., after OCR: "translate this? save to notes?").
- **MCP over LAN** — expose the MCP server on the local network (not just localhost) so other devices on the same WiFi can use this machine's tools. Useful for mobile ↔ desktop workflows.
