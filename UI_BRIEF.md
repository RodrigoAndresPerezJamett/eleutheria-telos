# Eleutheria Telos — UI Brief

This file is the contract for all UI work in Phase 4.5. Approved 2026-03-19.

**BRIEF STATUS: APPROVED — 2026-03-19**

> **Note on DESIGN.md:** `DESIGN.md` (the "Ethereal Command Center" doc) has been absorbed into this file. The unique content (Creative North Star, Space Grotesk for panel titles, Telos Glow interaction) is in §1 and §7 below. `DESIGN.md` can be deleted — it is superseded by this file.

---

## 1. Creative North Star: "The Ethereal Command Center"

This app moves beyond rigid, utilitarian software to create a digital environment that feels breathable, intelligent, and editorial. Inspired by Arc Browser's translucency and Caelestia Shell's geometric precision.

The goal: replace "software widgets" with "composed modules." Achieve premium feel via **Tonal Layering** and **Atmospheric Depth** — reject hard borders in favor of surface shifts and negative space.

**Specific principles:**
- **Glassmorphism as default:** sidebar, cards, and floating panels use frosted-glass backgrounds (`backdrop-filter: blur + semi-transparent fill`). Toggleable off in Settings.
- **Rounded corners everywhere:** generous radius on cards (12–16px), smaller on inputs/buttons (8px). No sharp rectangles.
- **Depth through layers:** content cards sit above panel surface, which sits above sidebar, which sits above background. Each layer: slightly lighter shade + subtle shadow.
- **Minimal chrome:** no heavy borders, no loud outlines. Separation via opacity and blur, not hard lines.
- **Transitions:** smooth panel-to-panel (150ms ease-out). No jarring flashes.

**Visual references:**

| Reference | What to take from it |
|-----------|---------------------|
| **Arc Browser** | Sidebar grouping, collapsible sections, smooth active states, pill-shaped highlights |
| **Caelestia Shell + Hyprland** | Glassmorphism layering, frosted panels, soft blur, how elements coexist without competing |

---

## 2. Color Palette & Theme System

**5 built-in themes** stored in `ui/assets/themes/` (already implemented). Active theme persisted to SQLite. See D-038 for architecture.

| Theme | Base | Notes |
|-------|------|-------|
| `dark` (default) | `#0f1117` base, indigo-periwinkle accent `#6d83f2` | Soft dark, not pure black |
| `light` | `#eff1f5` base | Clean light |
| `catppuccin-mocha` | Catppuccin Mocha palette | Mauve accent `#cba6f7` |
| `catppuccin-latte` | Catppuccin Latte palette | Light variant |
| `tokyo-night` | Tokyo Night palette | Blue accent `#7aa2f7` |

**CSS custom properties (defined per-theme in every theme file):**
```css
--bg-base, --bg-surface, --bg-elevated, --bg-overlay
--text-primary, --text-secondary, --text-muted
--accent, --accent-subtle, --accent-hover
--border, --border-focus
--shadow, --shadow-lg
--glass-bg, --glass-blur, --glass-border
--destructive, --success, --warning (+ -subtle variants)
--radius-sm: 8px, --radius-md: 12px, --radius-lg: 16px, --radius-xl
```

**The "No-Line" rule:** Standard 1px borders are prohibited for structural sectioning. Use surface shifts and negative space instead.

**Glassmorphism toggle:** `html.no-glass` class → opaque fills, no blur.

---

## 3. Typography

**Display & headlines: Space Grotesk** — used for `.panel-title` class only. Tight letter-spacing (`-0.02em`) creates editorial authority and the "Caelestia Shell" futuristic personality. Already bundled.

**UI & body: Inter variable** — high legibility, all other text. Already bundled as `ui/assets/fonts/inter-variable.woff2`.

**Size scale (Tailwind overrides):**
- `text-xs` (11px) — timestamps, labels
- `text-sm` (13px) — secondary content, sidebar labels
- `text-base` (15px) — body, card content
- `text-lg` (17px) — panel subheadings
- `text-xl` (20px) — panel titles (pair with `.panel-title` for Space Grotesk)

**Weight usage:** `font-normal (400)` body · `font-medium (500)` labels/buttons · `font-semibold (600)` panel titles/headers · `font-bold (700)` accent numbers only. Never bold body text when medium + darker color achieves the same hierarchy.

---

## 4. Elevation & Depth

Elevation is a property of **light and opacity**, not shadow-casting.

- **Layering principle:** `card-glass` on `--bg-surface` creates natural soft lift. Stack layers: base → surface → elevated → overlay.
- **Ambient shadows:** floating elements (popovers, tooltips) use tinted ambient shadow: `Y: 20px, Blur: 40px, Color: rgba(var(--accent-raw), 0.06)`.
- **Ghost border:** for accessibility on near-white elements, use `--border` at 10% opacity — felt, not seen.
- **Glassmorphism:** nav rail and sidebar use `--glass-bg` (60–70% opacity fill) + `--glass-blur` (blur 16–20px). Wallpaper ghosts through, mimicking Arc Browser.

---

## 5. Density & Spacing

**Simple panels** (Clipboard, Notes list, Models) — compact and information-dense:
- Card padding: `px-4 py-3`
- Gap between cards: `gap-2`
- Section headers: `py-2 px-4`

**Complex panels** (Quick Actions, Photo Editor, Video Processor) — more whitespace, larger step cards, clearer visual flow, more prominent action buttons.

---

## 6. Component Patterns

All implemented in `ui/assets/base.css`.

**Buttons (3 variants — standardize everywhere):**
- `btn btn-primary` — `--accent` fill, white text, `--radius-md`. For primary actions.
- `btn btn-secondary` — `--bg-elevated` fill, `--text-primary`, subtle border. For secondary actions.
- `btn btn-ghost` — transparent, `--text-secondary`, `--accent-subtle` on hover. For tertiary/text-link actions.
- `btn btn-danger` — destructive. `btn-ghost` treatment with `--destructive` color + confirmation state on click.
- All buttons: `--radius-pill` (9999px) shape — the "Pill Aesthetic."

**Cards:**
- `.card` — opaque, `--bg-elevated` fill
- `.card-glass` — `--glass-bg` fill, `--glass-border`, `--radius-md`, `box-shadow: 0 1px 3px var(--shadow)`
- Hover: `translateY(-1px)`, slightly brighter border, subtle `--shadow-accent` tinted glow
- `.card-interactive:active` — inset accent glow (the **Telos Glow** — simulates the shell being energized by user presence)

**Inputs:** `.input` class. `--bg-elevated` fill, `1px --border`, `--radius-sm`. Focus: `2px --border-focus` outline. Placeholder: `--text-muted`.

**Sidebar active item:** `.nav-item.active` pill style — `--accent-subtle` background, `--accent` text. Arc-style pill fills sidebar width with rounded ends.

**Section separators in sidebar:** thin `1px --border` line + `text-xs --text-muted uppercase tracking-wider` label. No `<hr>`.

**Badges / Status:**
- Recording: pulsing red dot (`@keyframes pulse`, CSS only via `.badge-recording`)
- Downloaded: green subtle chip
- Downloading: CSS progress bar inside card (no separate indicator)
- Error: `--destructive` inline text, no toast

**Empty states:** `.empty-state` class. Lucide icon (48px, `--text-muted`), title (`text-base font-medium`), subtitle (`text-sm --text-muted`), optional `btn btn-primary` CTA.

**Loading states:**
- Skeleton shimmer: CSS animation via `.skeleton` class
- HTMX indicator: Lucide `loader-2` rotating via `.htmx-indicator`

---

## 7. The "Telos Glow" — Signature Interaction

When a user interacts with an interactive card or tile, apply a subtle inset glow using `--accent-subtle`:

```css
.card-interactive:active {
  box-shadow: inset 0 0 0 2px var(--accent-subtle), 0 1px 3px var(--shadow);
}
```

This simulates the shell being "energized" by the user's presence. It reinforces the high-performance personality of Eleutheria Telos. Applied via `.card-interactive` class in `base.css`.

---

## 8. Sidebar

**Structure (3 static groups, Phase 4.5):**
```
TOOLS
  Clipboard · Notes · Voice · OCR · Translate · Search
──────────────────────────────
MEDIA
  Screen Rec · Audio Rec · Photo Edit · Video · Quick Actions
──────────────────────────────
PLUGINS
  [plugin entries]
  Models · Settings
```

**Icons:** Lucide icons via `ui/assets/lucide.min.js`. `lucide.createIcons()` on DOMContentLoaded + every HTMX swap. All emojis replaced.

**Sidebar width:** 200px desktop (icons + labels). Icon-only collapsed at tablet. Bottom nav on mobile.

**User-creatable groups + Arc-style stacking → Phase 5.**

---

## 9. Panel Layout

**Header per panel:** `<h1>` with `.panel-title` class (Space Grotesk) + optional subtitle (`.text-sm .text-muted`). Actions right-aligned in header row using `btn` variants.

**Content area:** `px-6 py-5` padding. Max width: none for list panels; `max-w-2xl` for form-heavy panels (OCR, Voice, Translate).

**Split panels** (Notes, Quick Actions): left panel fixed width + scrollable; right panel fills remaining space.

---

## 10. Per-Panel Empty States

| Panel | Copy |
|-------|------|
| Clipboard | "Nothing copied yet. Start copying text to see your history here." |
| Notes | "No notes yet. Press + New to create your first note." |
| OCR | Drag zone + "Capture screen area" as large `btn-primary` |
| Voice | Microphone icon + "Press Record to start transcribing" |
| Translate | "No language packs installed." + link to Models |
| Quick Actions | "No pipelines. Create one to automate your workflow." |
| Models | Never empty (catalog always populated) |

---

## 11. Panel Polish Priority Order

1. **App shell** ✓ (Phase 4.5 Step 1 — complete)
2. **Clipboard History** — card depth, hover lift, consistent action buttons
3. **OCR** — richer empty state, result card
4. **Screen Recorder** — recording state clarity
5. **Audio Recorder** — same pattern as Screen Rec
6. **Notes** — editor chrome, tag pills at card bottom
7. **Models** — download progress bar, installed vs available
8. **Quick Actions** — D-039 node shapes, pipeline list polish
9. **Video Processor** — operation tab layout
10. **Photo Editor** — toolbar polish
11. **Translate / Voice** — empty state work
12. **Settings** — theme selector, font selector, glass toggle
13. **Plugin panels** — consistent chrome

---

## 12. What to Keep (Do Not Change)

- Dark theme as default
- 2-column split layout for Notes and Quick Actions
- HTMX-driven navigation (no SPA, no full-page reloads)
- Sidebar structure (desktop: icons + labels; tablet: icons only; mobile: bottom nav)
- `Ctrl+K` command palette overlay
- Axum/HTMX/Alpine/Tailwind stack — visual changes are CSS + HTML only, no architecture changes

---

## Design Decisions Log

| Decision | Rationale | Date |
|----------|-----------|------|
| Lucide icons bundled locally | Offline-first (D-018); consistent visual language | 2026-03-19 |
| Inter bundled locally | Offline-first; best neutral high-quality UI font | 2026-03-19 |
| Space Grotesk for panel titles | Editorial authority; Caelestia Shell personality | 2026-03-19 |
| Separate CSS file per theme | Community scalability — copy one file, change values, done (D-038) | 2026-03-20 |
| Static sidebar groups for Phase 4.5 | Dynamic groups require backend data model; styling done without it | 2026-03-19 |
| Glassmorphism default on, toggleable | Matches Caelestia reference; users on lower-end hardware may disable | 2026-03-19 |
| Pill/diamond node shapes for Quick Actions canvas | Flowchart-classic — universally understood without color alone (D-039) | 2026-03-20 |
