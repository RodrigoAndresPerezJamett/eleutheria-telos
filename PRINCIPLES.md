# Eleutheria Telos — Principles

These principles are non-negotiable. Every decision, every line of code, every feature must be evaluated against them. When in doubt, come back here.

---

## 1. Offline-First
Every feature must work without an internet connection by default. Online services (translation APIs, model downloads) are opt-in enhancements, never requirements. If a feature cannot work offline at all, it must clearly communicate this to the user before activation.

## 2. Lightweight
The installed binary must not exceed 20MB (excluding AI models). AI models are downloaded on-demand, never bundled. RAM usage at idle must stay under 80MB. Before adding any dependency, ask: is this strictly necessary, or can we solve this with what we already have?

## 3. No SPA
The frontend uses HTMX for navigation (MPA pattern). React, Vue, Svelte, and similar frameworks are forbidden in the core app. Alpine.js is allowed exclusively for micro-interactions (toggling UI state, small local behaviors) — never for application state management. Each tool is an independent page, not a component.

## 4. Tools are Independent
Built-in tools must not import or call each other directly. All cross-tool communication goes through the Event Bus. A tool must function correctly even if all other tools are disabled. This is what makes the plugin system possible — plugins follow the same contract as built-in tools.

## 5. Plugins are First-Class Citizens
Anything a built-in tool can do, a plugin must be able to do too. Plugins register HTTP routes, subscribe to the Event Bus, read/write their sandboxed SQLite partition, and expose MCP tools — all through the same APIs available to built-in tools. No special privileges for core code.

## 6. One Language for Plugin Developers
Plugin developers should only need to know HTTP to build a plugin. They write a `manifest.json`, serve HTML fragments, and handle POST requests. The runtime (Python, Node, binary) is their choice. The plugin SDK must never require knowledge of Rust or Tauri internals.

## 7. Security by Default
The internal HTTP server accepts connections only from localhost and requires a session token on every request. Plugins are sandboxed — they can only access the permissions declared in their `manifest.json`. No permission escalation at runtime.

## 8. Responsive by Design
The UI must work on desktop (Windows, macOS, Linux), tablet (Android tablet, iPad), and phone (Android, iPhone). Three layouts exist: desktop uses a full sidebar with labels (≥1024px), tablet uses an icon-only collapsed sidebar (640px–1023px), and mobile uses a bottom navigation bar (<640px). Tailwind responsive utilities handle all three — no separate codebases, no JavaScript layout logic.

## 9. English UI, i18n Ready
The UI is in English. All user-facing strings must be externalized into translation files from day one. Adding a new language must not require code changes — only a new translation file.

## 10. Conservative with Dependencies
Every new crate or library must be justified. Prefer standard library solutions, then well-maintained single-purpose crates, then large frameworks as a last resort. When a crate is added, document why in `Cargo.toml` as a comment.
