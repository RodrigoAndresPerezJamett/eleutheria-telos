# Eleutheria Telos — UI Brief

This file is the contract for all UI work in Phase 4.5. Filled via Q&A session on 2026-03-19.

Claude Code: do not begin UI implementation until this file is marked **BRIEF STATUS: APPROVED**.

---

**BRIEF STATUS: APPROVED — 2026-03-19**

---

## 1. Aesthetic Direction

**Confirmed direction:** Soft glassmorphism, Arc × Caelestia Shell aesthetic.

Specific principles:
- **Glassmorphism as default:** sidebar, cards, and floating panels use frosted-glass backgrounds (`backdrop-filter: blur + semi-transparent fill`). Toggleable off in Settings.
- **Rounded corners everywhere:** generous radius on cards (12–16px), smaller on inputs/buttons (8px). No sharp rectangles.
- **Depth through layers:** content cards sit above a panel surface, which sits above the sidebar, which sits above the background. Each layer is a slightly lighter shade + subtle shadow.
- **Minimal chrome:** no heavy borders, no loud outlines. Separation is achieved via opacity and blur, not hard lines.
- **Transitions:** smooth panel-to-panel transitions (150ms ease-out). No jarring flashes.

**References apply to:** how panels/elements are disposed, how they layer over content, how sidebars collapse/group — not specifically to color palette.

**Out of scope for 4.5 (future phases):**
- Dynamic color adaptation from wallpaper/open apps (Caelestia feature → Phase 6)
- Panels sliding in from screen edges overlapping other apps (requires Tauri transparent overlay window → Phase 5)
- Floating radial mode / fan menu (Phase 5)

---

## 2. Visual References

| Reference | What to take from it |
|-----------|---------------------|
| **Arc Browser** | Sidebar grouping, collapsible sections, stacked entries, smooth active states, pill-shaped highlights |
| **Caelestia Shell + Hyprland** | Glassmorphism layering, frosted panels, soft blur, how elements coexist without competing |

---

## 3. Current Pain Points (from screenshots)

| Pain point | Location |
|-----------|----------|
| Cards blend into background — no depth, no separation | Clipboard, Models, Quick Actions |
| Buttons are inconsistent — some filled blue, some dark outlined, some text-only | Everywhere |
| Sidebar has no grouping between Core Tools / Media / Plugins | Sidebar |
| Emoji system icons are inconsistent and not visually uniform | Sidebar, panel headers |
| Panels are bare and feel abandoned on empty state | OCR, Translate, Voice, Screen Rec |
| Typography is completely flat — no visual hierarchy beyond size | All panels |
| "ELEUTHERIA" sidebar label is small and characterless | Sidebar header |
| Active sidebar item highlight looks like a colored rectangle, not a pill | Sidebar |
| Photo Editor / Quick Actions / Video controls feel arcaic and cramped | Those panels |
| Native title bar has no personality | App shell |

---

## 4. Color Palette

**Theme system (multiple themes, not just dark):**

The app ships with built-in themes, stored in `ui/assets/themes/`. Theme is selected in Settings and saved to SQLite. The active theme CSS file is swapped at runtime.

**Built-in themes to ship in Phase 4.5:**

| Theme name | Base | Notes |
|-----------|------|-------|
| `dark` (default) | `#13151a` base, `#1e2030` surface, `#24273a` elevated | Soft dark, not pure black |
| `light` | `#eff1f5` base, `#ffffff` surface, `#f2f4f8` elevated | Clean light |
| `catppuccin-mocha` | Catppuccin Mocha palette | Popular in Hyprland/Linux community |
| `catppuccin-latte` | Catppuccin Latte palette | Light variant |
| `tokyo-night` | Tokyo Night palette | Common VS Code / Hyprland theme |

**CSS custom properties (defined per-theme):**
```css
--bg-base       /* deepest layer — window background */
--bg-surface    /* panel content area */
--bg-elevated   /* cards, inputs */
--bg-overlay    /* glassmorphism fill: semi-transparent */
--text-primary
--text-secondary
--text-muted
--accent         /* primary action color — per-theme */
--accent-subtle  /* accent at low opacity — hover states, badges */
--border         /* subtle divider color */
--shadow         /* box-shadow color */
--radius-sm: 8px
--radius-md: 12px
--radius-lg: 16px
```

**Glassmorphism variables:**
```css
--glass-bg       /* rgba version of --bg-surface at ~60% opacity */
--glass-blur: blur(16px)
--glass-border   /* 1px solid rgba(white, 0.08) */
```

Glassmorphism toggle in Settings saves `glass: true/false` to SQLite. When off, `--glass-bg` falls back to `--bg-surface` (opaque) and `--glass-blur` to `none`.

---

## 5. Typography

**Default font:** Inter (bundled locally under `ui/assets/fonts/inter/`)

**Size scale (Tailwind overrides):**
- `text-xs`: 11px — timestamps, labels
- `text-sm`: 13px — secondary content, sidebar labels
- `text-base`: 15px — body, card content
- `text-lg`: 17px — panel subheadings
- `text-xl`: 20px — panel titles
- `text-2xl`: 24px — (rarely used)

**Weight usage:**
- `font-normal (400)`: body text, descriptions
- `font-medium (500)`: labels, button text, sidebar items
- `font-semibold (600)`: panel titles, section headers
- `font-bold (700)`: accent numbers, status indicators

**Font change in Settings (Phase 4.5):**
- Dropdown to select system font vs Inter
- "Download font" feature → **Phase 5** (requires font management system)

---

## 6. Density & Spacing

**Core principle:** Compact and information-dense for simple panels (Clipboard, Notes list, Models). Visually richer and more spaced for complex workflow panels (Quick Actions, Photo Editor, Video Processor).

**Simple panel density:**
- Card padding: `px-4 py-3`
- Gap between cards: `gap-2`
- Section headers: `py-2 px-4`

**Complex panel density:**
- More whitespace between logical groups
- Larger step cards, clearer visual flow
- Action buttons more prominent (not just text links)

---

## 7. Component Patterns

**Buttons (3 variants only — standardize across all panels):**
- `btn-primary`: filled with `--accent`, white text, `radius-md`
- `btn-secondary`: `--bg-elevated` fill, `--text-primary`, subtle border
- `btn-ghost`: transparent, `--text-secondary`, `--accent-subtle` on hover
- **No more raw text links as action buttons** (Copy, Delete, Download must use btn variants)
- Destructive actions (Delete, Clear all): `btn-ghost` with red text, confirmation state on click

**Inputs:**
- `--bg-elevated` fill, `--border` outline at 1px, `radius-sm`
- Focus ring: `--accent` at 2px, no default browser outline
- Placeholder: `--text-muted`

**Cards:**
- `--glass-bg` fill (or `--bg-elevated` when glass off), `--glass-border`, `radius-md`
- Subtle `box-shadow: 0 1px 3px var(--shadow)`
- Hover: lift effect (`translateY(-1px)`, slightly brighter border)

**Sidebar item (active state):**
- Pill highlight: `--accent-subtle` background, `--accent` left border or pill shape
- Arc-style: the active item is a pill that fills the sidebar width with rounded ends

**Badges / Status indicators:**
- Recording: pulsing red dot
- Downloaded: green subtle chip
- Downloading: progress bar inside card (not a separate indicator)
- Error: red text, no toast (inline)

**Section separators in sidebar:**
- Thin `1px --border` line + small label (`text-xs --text-muted uppercase tracking-wider`)

---

## 8. Sidebar

**Structure (3 static groups for Phase 4.5):**
```
TOOLS
  Clipboard
  Notes
  Voice
  OCR
  Translate
  Search
─────────────
MEDIA
  Screen Rec
  Audio Rec
  Photo Edit
  Video
  Quick Actions
─────────────
PLUGINS
  [plugin entries]
  Models
  Settings
```

**Icons:** Lucide icons, replacing all emojis. Bundled locally as `ui/assets/lucide.min.js`. Called via `lucide.createIcons()` on DOMContentLoaded and after each HTMX swap.

**Sidebar width:** 200px desktop (unchanged). Active item: pill style.

**User-creatable groups + Arc-style stacking → Phase 5** (requires backend data model for group persistence).

**Glassmorphism on sidebar:** yes, sidebar uses `--glass-bg` + `--glass-blur` by default.

---

## 9. Panel Layout

**Header per panel:**
- Each panel has a consistent header zone: `<h1>` title (text-xl, font-semibold) + optional subtitle (text-sm, text-muted)
- Actions (e.g., "Clear all", "+ New") live in the header row, right-aligned, using btn variants

**Content area:**
- `px-6 py-5` padding (slightly more breathing room than current)
- Max content width: none (full width) for list panels; `max-w-2xl` for form-heavy panels (OCR, Voice, Translate)

**Split panels (Notes, Quick Actions):**
- Left panel: fixed width, scrollable list
- Right panel: fills remaining space, content area

**No breadcrumbs** — single-level navigation via sidebar is sufficient.

---

## 10. Empty States & Feedback

**Empty state anatomy:**
- Lucide icon: `text-muted`, large (48px)
- Title: `text-base font-medium`
- Subtitle: `text-sm text-muted`
- Optional CTA button

**Per-panel empty states:**
- Clipboard: "Nothing copied yet. Start copying text to see your history here."
- Notes: "No notes yet. Press + New to create your first note."
- OCR: visual prompt — drag zone + "Capture screen area" big action button
- Voice: microphone icon, "Press Record to start transcribing"
- Translate: "No language packs installed." + link to Models (already exists, needs styling)
- Models: never empty (catalog is always populated)
- Quick Actions: "No pipelines. Create one to automate your workflow."

**Loading states:**
- Skeleton shimmer (CSS animation) for lists that are loading
- `hx-indicator` spinner: replace current CSS spinner with a subtle Lucide `loader-2` rotating icon

**Error states:**
- Inline below the form/action that failed
- Red text, `text-sm`, no modal/toast

---

## 11. Priority Order

Implement in this order (most impactful / most-used first):

1. **App shell** — CSS variables, Inter font, Lucide icons, sidebar redesign (groups, pill active state)
2. **Clipboard History** — card depth, hover lift, consistent action buttons
3. **OCR** — richer empty state, better button layout, result card
4. **Screen Recorder** — status feedback, recording state clarity
5. **Audio Recorder** — same pattern as Screen Rec
6. **Notes** — editor chrome, note list card polish
7. **Models** — download progress bar, installed vs available distinction
8. **Quick Actions** — step cards richer, pipeline list more visual
9. **Video Processor** — better operation tab layout
10. **Photo Editor** — toolbar polish
11. **Translate / Voice** — mostly empty-state work
12. **Settings** — theme selector, font selector, glass toggle
13. **Plugin panels** — consistent chrome so they feel native

---

## 12. What to Keep

- Dark theme as default
- 2-column split layout for Notes and Quick Actions
- HTMX-driven navigation (no SPA, no full-page reloads)
- Sidebar structure (desktop: icons + labels; tablet: icons only)
- The `Ctrl+K` command palette overlay
- Axum/HTMX/Alpine stack — all visual changes are CSS + HTML only, no architecture changes

---

## Design Decisions (filled during execution)

| Decision | Rationale | Date |
|----------|-----------|------|
| Lucide icons bundled locally | Offline-first (D-018 principle); consistent visual language vs mixed emojis | 2026-03-19 |
| Inter bundled locally | Offline-first; Inter is the most neutral high-quality UI font | 2026-03-19 |
| CSS custom properties for theming | Clean separation; theme swap = single class change on `<html>` | 2026-03-19 |
| Static sidebar groups for Phase 4.5 | Dynamic groups require backend data model; styling can be done without that | 2026-03-19 |
| Glassmorphism default on, toggleable | Matches Caelestia reference; users on lower-end hardware may want to disable | 2026-03-19 |

---

## Playwright Review Notes (filled during Playwright step)

| Panel | Issues found | Fixed |
|-------|-------------|-------|
| | | |
