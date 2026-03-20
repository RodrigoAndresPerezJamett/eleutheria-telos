# Eleutheria Telos — Ideas, Desirables & Future Thinking

This is the project's idea bin. Nothing here is scheduled or committed. These are thoughts, "wouldn't it be nice if..." items, community suggestions, and future possibilities. Items graduate to ROADMAP.md when they are prioritized and scoped.

Claude Code: do not implement anything from this file unless it has been explicitly moved to ROADMAP.md and is in the current phase.

---

## UI / UX Ideas

- **Search: recent searches history** — below the search bar, show the last 5–8 searches as clickable chips. Dismissable individually or all at once. Persisted in SQLite (`search_history` table). Helps users re-run common queries without retyping. Phase 5.

- **Clipboard: privacy blur mode** — a toggle in Settings (or directly in the Clipboard panel header) that blurs all clipboard history items. Useful when screen sharing or recording. When enabled, items show a blurred overlay that reveals on hover/click. State persisted in `settings` table (`clipboard_blur: bool`). Phase 5.

- **Clipboard: rich content preview** — beyond plain text, show a thumbnail if the copied item is an image (arboard already supports `get_image()`), and a waveform or file icon if the source is an audio file path. Requires clipboard content-type detection at capture time and storing a `content_type` column alongside `content` in the `clipboard` table. Text remains the default; images and audio are opt-in previews. Phase 5.

- **OCR: automatic language detection by default** — Tesseract supports `osd` (orientation and script detection) and language packs can be combined (`-l eng+spa`). The current default is hardcoded to `eng`. Better UX: default to `auto` which runs `tesseract --psm 0` first to detect the script, then selects the best installed language pack. Manual override remains available for edge cases. Requires testing across installed language packs. Phase 5.

- **Translation: automatic source language detection by default** — the `from_lang` select currently defaults to English. The `auto` option exists in the select but is not the default. Change the default to `auto` (ctranslate2 supports source language detection via sentencepiece when `from_lang` is omitted or set to `auto`). Both OCR inline translate and the standalone Translate panel should default to `auto`. Phase 5.

- **Keybindings section in Settings** — a dedicated section in the Settings panel where the user can see all keyboard shortcuts (Ctrl+K for command palette, global hotkey, etc.) and remap them. Could extend to Quick Actions triggers: assign a hotkey to run a specific pipeline directly. Natural evolution of the current hardcoded Ctrl+K listener. Phase 5.

- **Draw on screen (Screen Annotation)** — draw, highlight, and annotate directly on top of live screen content. Overlay window using Tauri's transparent always-on-top window. Proposed by user during Phase 4.5 Q&A as a wanted feature.

- **Dynamic color adaptation from wallpaper** — inspired by Caelestia Shell: the app UI palette adapts in real time when the desktop background changes color or a new app is opened. Optional feature the user can activate. Requires reading the dominant color from the wallpaper or sampling the screen behind the window. Architecturally complex (OS-level color sampling). Phase 6+.

- **Panel navigation history (back / forward)** — browser-like back/forward arrows in the shell header. Every tool navigation is pushed onto an in-memory stack. Back re-navigates to the previous tool; forward re-navigates if the user went back. Useful when accidentally clicking the wrong tool. Implementation: a JS `navHistory[]` array + current index, updated on every `htmx:afterSwap` of `#tool-panel`. Chevron buttons shown/hidden based on stack state. Phase 5.

- **Resizable panel dividers** — the vertical separator between the notes list and notes editor (and any other two-column panel like Quick Actions) should be draggable to resize, same as the sidebar. User sets their preferred split and it persists in settings. Low implementation cost — same drag logic already used for the sidebar. Phase 5.

- **Sidebar customisation: remove / restore built-in tools** — right-clicking a sidebar item (or hovering to reveal an X button) lets the user hide it from the sidebar. Hidden tools are not deleted — their data is preserved. They can be re-enabled from the Plugin Store, where built-in tools appear as a first section ("Built-in tools") above the community plugin listing. The Plugin Store has a search bar (non-functional for now — no community registry yet; bar is present for structural completeness and future use). State stored in `settings` table as `sidebar_hidden_tools: JSON array`. Phase 5.

- **Arc-style sidebar: user-creatable groups + stacking** — users can create named groups in the sidebar (like Arc Spaces), drag tools into groups, and collapse/expand them. Tools can be "stacked" (multiple tools share one sidebar slot with a mini-tab selector). Requires a backend data model for group persistence (SQLite). Phase 5.

- **Floating radial mode (fan menu)** — a small always-on-top floating button that, when clicked, fans out into a radial menu of the user's most-used tools. Designed for use overlaid on other apps without opening a full window. Requires a Tauri transparent always-on-top overlay window + radial CSS layout. Phase 5. (See also: "Mini mode" below.)

- **Font system in Settings** — download and install new fonts directly from within the app. Browse a curated list, download to `ui/assets/fonts/`, and switch the active font. Phase 5.

- **Command Palette (`Ctrl+K`)** — global search across all tools and notes, launcher for quick actions. Already planned for MVP but worth noting here as a high-priority idea.
- **Quick Actions / Pipelines** — user-defined chains of tool events without code. Example: "after OCR, auto-translate and copy to clipboard". Event Bus already supports this — just needs a UI. *(Implemented in Phase 4.7 — ideas below are for future enhancement.)*
- **Quick Actions: keybinds per pipeline** — assign a keyboard shortcut to a pipeline so it can be triggered manually without auto-running after every OCR/voice/clipboard event. Useful when the user only sometimes wants to run a pipeline after a tool completes, e.g. "I want to translate this particular OCR result but not all of them". Requires `tauri-plugin-global-shortcut` for global hotkeys, or a local hotkey approach within the app window.
- **Quick Actions: opt-in / opt-out for auto-triggered pipelines** — when a pipeline trigger fires (e.g. `OcrCompleted`), instead of executing automatically, show a toast or overlay asking the user "Run pipeline X?" with Accept/Dismiss buttons. This prevents pipelines from silently running on every event. Could be a per-pipeline setting: "Always run" vs "Ask me first". (Surfaced in sprint review 2026-03-19 — exact UX: small non-blocking toast bottom-right, auto-dismissed after 8s if ignored, "Don't ask again for this pipeline" checkbox.)
- **Persist tool outputs by default — open product question** — OCR results and voice transcriptions are currently transient (shown in result cards, gone on navigation). Open question: should they be saved automatically to a `captures` table, or only when the user explicitly saves to Notes/Clipboard? If yes: requires a new UI surface (captures browser, search, delete). If no: FTS across notes + clipboard already covers the use case. Decision deferred to after beta user feedback. Do not build infrastructure until the product question is answered. (See D-035.)
- **Quick Actions: full visual canvas editor** — replace the current list-based step editor with a drag-and-drop canvas (like n8n or Zapier): boxes for each step connected by arrows, support for conditional branches (if/else), cycles (loops), and multiple trigger inputs. High effort but the ideal end state for non-technical users building automation workflows.
- **Themes** — community CSS themes for the shell UI. Since everything is HTML + Tailwind, a theme is just an override CSS file.
- **Mini mode** — a compact floating mode for the app (like a widget) that shows only the most-used tool, always on top.
- **Keyboard-first mode** — navigate all tools entirely with keyboard shortcuts, no mouse required.
- **Global hotkey for show/hide (Phase 5)** — configurable keybind to show/hide the app window without relying on the system tray. The tray is invisible in some shell environments (e.g. Hyprland + Noctalia). A global hotkey via `tauri-plugin-global-shortcut` makes the app accessible everywhere regardless of tray support. Should be user-configurable in Settings, default e.g. `Super+E`.

---

## New Tools (Not in Current Roadmap)

- **Smart Copy** — previously considered and descoped. An OCR overlay that lets the user screenshot any part of the screen and extracts text, links, and media URLs from it. Useful for copying text from videos or images on web pages.
- **Quick File Converter** — convert image to PDF, PDF to text, video to audio, etc. ffmpeg already handles most of this — just needs a UI. Low implementation cost.
- **Reader Mode / Article Saver** — save a URL as clean readable text locally (like Pocket but offline). Useful with translation tool.
- **Quick Timer / Stopwatch** — minimal but surprisingly often-needed. Trivial to implement. High MCP value: "set a 25-minute timer".
- **Pomodoro** — extends the timer with work/break cycles. Community plugin candidate.
- **Color Picker** — pick any color from anywhere on screen, copy hex/rgb/hsl. Common in developer toolkits.
- **Regex Tester** — test regular expressions with live matching. Community plugin candidate.
- **Diff Tool** — compare two pieces of text. Community plugin candidate.

---

## Media Tools (Phase 3+)

- **Video Timeline Editor** — a visual NLE with multiple tracks. Explicitly a major community plugin project, not a core feature. Estimated effort: months. Would use the Video Processor as its ffmpeg backend.
- **Video Processor UX improvements (Phase 5 polish):**
  - **Drag-and-drop file input** — drag a video file onto the panel to populate the path field automatically, avoiding manual copy-paste of filesystem paths.
  - **Click-to-browse file picker** — a "Browse…" button that opens the system file manager via `tauri-plugin-dialog` (`dialog.open()` with video file filters). Requires adding `tauri-plugin-dialog` as a dependency with capability configuration.
  - **Video preview / thumbnail** — show a static thumbnail or short preview of the loaded video before processing, so the user can confirm they selected the right file.
  - **Visual timeline** — a minimal waveform + keyframe strip showing the video duration, with draggable in/out handles for the Trim operation. Eliminates manual `HH:MM:SS` typing. Could use `ffprobe` for duration metadata and `ffmpeg -vf thumbnail` for frame extraction.
- **Audio Editor** — trim, fade, normalize audio. ffmpeg-based. Smaller scope than video editor.
- **Batch Image Processing** — resize, convert, compress multiple images at once. rembg for batch background removal.
- **Photo Editor: undo / redo** — Ctrl+Z / Ctrl+Shift+Z (or Ctrl+Y) to step backwards and forwards through edit history. Requires a command stack per session: each operation (paint stroke, erase, layer move, crop, filter) is pushed onto the stack. Undo pops and reverses; redo re-applies. Stack can be capped at 50 steps. Phase 5.

- **Photo Editor: crop tool** — crop an image by dragging a selection box over the canvas, with aspect ratio lock options (free, 1:1, 16:9, 4:3). The crop applies non-destructively until confirmed; original is preserved until the user explicitly saves. Phase 5.
- **Photo Editor: move image within layer** — pan the image inside the canvas/layer without resizing it. Useful for repositioning after scaling. Requires a transform matrix per layer (translateX, translateY, scale, rotation) stored separately from the canvas dimensions. Phase 5.
- **Photo Editor: resize layer without affecting photo** — scale the canvas/layer bounding box independently of the image content, cropping or adding transparent/filled padding around the image. Equivalent to Photoshop "Canvas Size" vs "Image Size". Allows adding space around an image for text overlays or border effects. Phase 5.
- **Screen Annotation** — draw, highlight, and annotate on top of live screen content. Overlay window using Tauri's transparent window capabilities.
- **GIF Recorder** — record a short screen region as a GIF. ffmpeg can do this. High demand for sharing demos.
- **MKV output format for Screen Recorder** — wf-recorder supports `-f output.mkv` natively. MKV is a recoverable container (mp4 is not — a crash loses the file). Add a format selector (mp4 / mkv) to the Screen Recorder panel. Low effort, high value for long recordings.

---

## Community / Ecosystem Ideas

- **Plugin: Obsidian Send** — from any result card (OCR, voice, note), send the text directly to an Obsidian vault as a new `.md` file. Configured with vault path in plugin settings. Good reference implementation for plugins that write to the filesystem and integrate with a third-party app.

- **Plugin: GitHub Issues creator** — select text from any result and create a GitHub issue in a configured repository with that content pre-filled. Auth via personal access token stored in plugin settings. Good reference implementation for plugins that call external APIs.

- **Plugin Registry** — a hosted JSON file listing community plugins with name, author, version, download URL. The app shows a browsable "Plugin Store" and installs with one click. Requires minimal server infrastructure (static JSON on GitHub Pages would work).
- **Plugin Templates** — `cargo generate` or `npx` templates for scaffolding a new plugin in Python or Node in seconds.
- **Plugin Sandboxing V2** — more granular permissions (e.g., `network.outbound.allowlist: ["api.example.com"]`) and resource limits (CPU, memory) per plugin.
- **Community Translation Files** — i18n contributions from the community. Spanish, Portuguese, French, German, Japanese as first targets given likely user base.

---

## Session Recovery

- **Crash / accidental navigation recovery for stateful tools** — if the user closes the app, force-quits, or accidentally navigates away while working in the Photo Editor, Video Processor, or Quick Actions pipeline builder, their in-progress work should be restored when they return to that tool. Mechanism: each tool periodically serialises its current state to a `session_drafts` SQLite table (key = tool name, value = JSON snapshot). On panel load, if a draft exists, show a non-intrusive banner: "You have unsaved work from [time] — restore it?" with Restore / Discard buttons. Auto-saves every 30 seconds while the tool is active. Phase 5.

---

## Voice Tool Enhancements

- **Notes: text highlighting / marker** — select a word or sentence and apply a highlight colour (like a marker pen). Multiple colours available (yellow, green, pink, blue) via a small colour chip toolbar that appears on text selection. Highlights stored as markdown-compatible annotations (`==text==` syntax, rendered via a custom CSS rule as `background: <color>`). Useful for study notes, review workflows, and meeting notes. Phase 5.

- **Audio playback in Voice panel** — after recording or uploading, show an HTML5 `<audio>` player so the user can listen back to the recording before or after transcription. Includes play/pause controls.
- **Save audio file** — "Save Recording" button that copies the WAV/audio file from `/tmp/` to a user-chosen location (or a default `~/Documents/eleutheria-recordings/`). Currently only the transcript is saved; the audio is discarded after transcription.

---

## Python Dependency Management (Phase 5)

- **`argostranslate` is broken on Python 3.14 — hard blocker for translation** — discovered during Phase 2 Step 4 testing. Two compounding problems:

  1. **Python 3.14 incompatibility:** `argostranslate` → `spacy` → `thinc` → `confection` → `pydantic.v1`. Pydantic V1 is not compatible with Python 3.14+. Runtime error: `Core Pydantic V1 functionality isn't compatible with Python 3.14 or greater / unable to infer type for attribute "REGEX"`. Translation fails even after installation.

  2. **Massive dependency footprint:** `pip3 install argostranslate` pulls in ~3 GB of packages: PyTorch 2.10 (915 MB), full CUDA stack (nvidia-cublas, nvidia-cudnn, nvidia-cufft, triton, etc.), spacy, stanza, onnxruntime, and 50+ other packages. Completely disproportionate for a text translation feature.

  **Recommended alternatives to evaluate before Phase 5:**
  - **`ctranslate2` directly** — argostranslate uses ctranslate2 under the hood. Use it directly with Opus-MT `.ctranslate2` models, bypassing the spacy/stanza/pydantic chain entirely. Much lighter. Already confirmed `ctranslate2==4.7.1` has a cp314 manylinux wheel.
  - **LibreTranslate (local)** — self-hosted REST API, no Python dependency, translates via HTTP. Heavier to set up but fully offline and Python-version-agnostic.
  - **pyenv venv pinned to Python 3.12** — run the translation subprocess in a 3.12 venv where argostranslate works. Avoids rewriting the integration but adds venv management complexity.
  - Current status: translation UI and routes are implemented and working structurally; the Python subprocess fails at runtime on this machine. Feature is non-functional until one of the above paths is chosen.

- **Bundled venv + first-run setup** — instead of requiring `pip install` manually, the app should:
  1. On first launch, detect if `~/.local/share/eleutheria-telos/venv/` exists
  2. If not, show a "Setting up AI tools…" screen with a progress indicator
  3. Run `python3 -m venv` + `pip install -r requirements.txt` automatically in the background
  4. All subsequent Python subprocess calls use the venv's Python interpreter (`venv/bin/python3`) instead of the system one
  - This means the user never sees `pip` — the app is self-contained from their perspective
  - Alternative to evaluate: replace `pywhispercpp` with `whisper-rs` (Rust crate wrapping whisper.cpp) to eliminate Python entirely for voice transcription

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

## Intelligence & Cohesion — Making the App Feel "Smart"

This section captures the result of a deliberate introspection: what would it take for Eleutheria Telos to feel genuinely *intelligent* — where each tool knows about the others, actions surface at the right moment, and the app competes with best-in-class specialized tools on their own ground?

### Cross-tool awareness (the "context layer")

The current architecture is tool-isolated (by design). But intelligence requires a shared context layer on top of that isolation. Ideas:

- **Global recents feed** — a lightweight `captures` table records every tool output (OCR text, transcription, translation result, clipboard entry) with timestamps. Any tool can query "what was the user doing 5 minutes ago?" This enables: translate the last OCR result, save the last transcription to notes, re-use any recent capture as input to a new pipeline. The feed is *not* shown to the user as a standalone panel — it's the invisible backbone that makes other things possible.

- **Context-aware Quick Actions suggestions** — when the user opens Quick Actions, the app surfaces *relevant* template suggestions based on what they just did. If the last tool used was OCR: show OCR-related templates first. If the clipboard top entry is a URL: surface "Open in Reader Mode" or "Translate this page". Requires the global recents feed + a tiny scoring heuristic (no ML needed).

- **Tool output routing via drag** — drag a result card from OCR/Voice/Translation onto the sidebar icon of Notes, Clipboard, or Translation to trigger that tool with the result as input. No pipelines needed — discoverable, immediate, gestural. Requires Tauri drag-and-drop events between WebView elements and sidebar.

### Proactive suggestions (non-intrusive)

- **"Did you mean to…?" toast** — after OCR completes, if the text appears to be in a language that isn't the user's default, show a small non-blocking toast: "Looks like French — translate to English?" with Accept/Dismiss. Same pattern for "looks like a URL — save to Reader Mode?" or "looks like a phone number — copy to clipboard?". Detection is heuristic (lingua-rs or simple charset/regex), never network-dependent.

- **Smart clipboard deduplication** — when a clipboard entry is captured that is very similar (Levenshtein distance < 10%) to an entry captured in the last 30 minutes, show a subtle indicator rather than adding a duplicate. Reduces noise in the history without losing anything.

- **Pattern-based pipeline auto-suggestion** — if the user manually performs the same sequence of actions 3+ times (OCR → open translation → paste result), offer to create a pipeline for it. Requires action event logging in the DB. No ML — just frequency counting over a session window.

### Competing with specialized apps

Each core tool benchmarked against its best-in-class competitor:

| Tool | Best-in-class competitor | Gap to close |
|---|---|---|
| Clipboard | Raycast, Maccy, Pasta | Smart deduplication, content-type icons, pinning, fast fuzzy search |
| Notes | Bear, Obsidian | Backlinks between notes, inline tagging (`#tag`, `[[link]]`), markdown export |
| OCR | TextSniper (macOS) | Click-to-capture (no file needed — screengrab from selection), table extraction |
| Voice | Whisper.app, Speak AI | Real-time streaming transcription (Whisper streaming mode), speaker labels |
| Translation | DeepL | Quality gap is in the model; mitigation: support multiple models (NLLB, Opus-MT-Big), let user pick |
| Photo Editor | Canva, Photopea | Layer panel, blend modes, text on image, export to PNG/JPEG/WebP |
| Video Processor | HandBrake | Preset system, batch queue, hardware acceleration (VAAPI) |
| Screen Recorder | OBS, Kooha | Region selection resize, hotkey start/stop without clicking the UI |
| Quick Actions | Raycast Workflows, n8n | Conditional branches, schedule triggers, webhook triggers |

The fastest wins (low effort, high credibility impact):

1. **Clipboard:** add pinning (`is_pinned` column, float to top), and content-type icons (URL globe, image thumbnail, code `{}` badge). Already have the data — just UI.
2. **OCR:** click-to-capture (screengrab inline, no file dialog). Tauri screenshot API + Tesseract. Eliminates the #1 friction point.
3. **Voice:** show a live waveform while recording (Web Audio API, no native code). Purely cosmetic but makes recording feel responsive.
4. **Notes:** inline `#tag` extraction stored to a `tags` table. No backlinks yet — just tags. Enables "show all notes tagged #meeting".
5. **Screen Recorder:** global hotkey to start/stop (`tauri-plugin-global-shortcut`). The current "click Start then click Stop" flow breaks the screencast use case.

### "Intelligent" UI patterns

- **Adaptive empty states** — empty states that change based on prior behavior. First session: onboarding hint. After 5 sessions: power-user tip. After the user has used OCR 3 times: "Try setting up a pipeline to auto-translate OCR results". Requires a `usage_count` per tool in `settings`.

- **Command palette learns** — Ctrl+K results should surface the most recently and most frequently used items first. A `command_history` table with access counts and last-used timestamps. Already a near-zero effort add given the palette exists.

- **Inline translation in any panel** — a right-click context menu on any selected text anywhere in the app (notes, OCR result, clipboard preview) with "Translate selection" → opens translation result inline below the selection without navigating away. Requires a context menu event listener at the shell level + a dedicated small `/api/tools/translate/inline` endpoint.

- **Keyboard-first everything** — every action reachable without mouse. Focus moves logically through panels. Tool results are navigable with arrow keys. Escape always collapses. This is the deepest differentiator against Electron apps where keyboard navigation is bolted on.

All of the above are Phase 5+ ideas. None require new architecture — they extend what's already in place. The intelligence is emergent from the data that already flows through the app.

- **Local LLM integration** — connect to Ollama (already has a config entry in Claude Code's config) for AI-powered notes summarization, smart search, etc. Fully offline.
- **AI-powered Quick Actions** — instead of user-defined pipelines, use a small local model to suggest "next action" after a tool completes (e.g., after OCR: "translate this? save to notes?").
- **MCP over LAN** — expose the MCP server on the local network (not just localhost) so other devices on the same WiFi can use this machine's tools. Useful for mobile ↔ desktop workflows.
