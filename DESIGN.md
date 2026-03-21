# Design System Strategy: Eleutheria Telos

## 1. Overview & Creative North Star
**The Creative North Star: "The Ethereal Command Center"**

This design system moves beyond the rigid, utilitarian structures of traditional software to create a digital environment that feels breathable, intelligent, and editorial. Inspired by the translucency of Arc Browser and the geometric precision of Caelestia Shell, we are building an "Ethereal Command Center."

The goal is to replace "software widgets" with "composed modules." We achieve a premium, custom feel by rejecting standard borders in favor of **Tonal Layering** and **Atmospheric Depth**. By utilizing high-contrast typography scales and generous, hyper-rounded containers, the UI feels less like a tool and more like a curated workspace.

---

## 2. Colors & Atmospheric Layering
The palette is rooted in soft off-whites (`#f8f9fa`) and "Mint-Glass" transitions, punctuated by intentional Lavender (`#7049b3`) accents.

### The "No-Line" Rule
Standard 1px borders are strictly prohibited for sectioning. Structural definition must be achieved through:
- **Surface Shifts:** Using `surface-container-low` against a `surface` background.
- **Negative Space:** Leveraging the 16px (`4`) spacing token to define boundaries.

### Surface Hierarchy & Nesting
Treat the UI as a physical stack of semi-translucent materials.
- **Base Layer:** `surface` (#f8f9fa)
- **Primary Modules:** `surface-container-low` (#f2f4f5)
- **Interactive Elements/Cards:** `surface-container-lowest` (#ffffff) for maximum "lift."
- **In-App Overlays:** Use `secondary-container` (#ccfaf5) with a 60% opacity and a 20px backdrop-blur to create the signature "Mint Glass" effect.

### The "Glass & Gradient" Rule
To elevate CTAs from "flat" to "bespoke," use a linear gradient for primary actions:
* **Lavender Flow:** `primary` (#7049b3) to `primary-dim` (#633ca6) at a 135° angle.
* **Mint Glow:** Use a subtle `secondary-fixed-dim` (#beebe7) glow behind high-priority glass modules to simulate light passing through a prism.

---

## 3. Typography: Editorial Authority
We use typography to create a "Visual Pulse"—the headers command attention, while the UI remains invisible but highly functional.

* **Display & Headlines (Space Grotesk):** This is our "Editorial Voice." Use `display-lg` and `headline-md` with tight letter-spacing (-0.02em) to create a high-end, futuristic feel. Space Grotesk's geometric quirks provide the "Caelestia Shell" personality.
* **UI & Body (Inter/Sans-Serif):** High-legibility is paramount. Use `body-md` for standard density and `label-md` for metadata. Inter provides the "High-Performance" reliability required for complex workflows.
* **Hierarchy Tip:** Never use "Bold" for body text when "Medium" with a darker `on-surface` color will suffice. Let the scale (size) do the heavy lifting, not the weight.

---

## 4. Elevation & Depth
In this system, elevation is a property of light and opacity, not shadow-casting.

* **The Layering Principle:** Depth is achieved by "stacking." A `surface-container-lowest` card placed on a `surface-container-low` background creates a natural, soft lift.
* **Ambient Shadows:** If a floating element (like a Popover) requires a shadow, use a "Tinted Ambient Shadow":
- `Y: 20px, Blur: 40px, Color: rgba(112, 73, 179, 0.06)` (A Lavender-tinted shadow).
* **The Ghost Border:** For accessibility on white-on-white elements, use `outline-variant` (#aeb3b5) at **10% opacity**. It should be felt, not seen.
* **Glassmorphism:** Navigation rails and sidebars should use a background of `surface-container-lowest` at 70% opacity with a `blur(12px)` effect. This allows the user's wallpaper or background content to "ghost" through, mimicking the Arc Browser aesthetic.

---

## 5. Components & Primitive Styling

### Buttons: The "Pill" Aesthetic
All buttons use the `full` (9999px) rounding token.
- **Primary:** Lavender gradient with `on-primary` text.
- **Secondary:** `secondary-container` (Mint) with `on-secondary-container` text. No border.
- **Tertiary:** Ghost style. `on-surface` text with a `surface-variant` background appearing only on hover.

### Modules (Cards)
- **Rounding:** Strictly `xl` (3rem/48px) for outer containers, `lg` (2rem/32px) for nested inner cards.
- **Gaps:** Maintain a strict 16px (`4`) gap in the tiled grid.
- **No Dividers:** Never use `
` tags. Use `1.5rem` (`6`) of vertical white space to separate content blocks.


### Input Fields
- **Background:** `surface-container-high`.
- **Shape:** `md` (1.5rem) rounding.
- **Active State:** A 2px "Ghost Border" of `primary` at 40% opacity.

### Navigation Rail (The "Shell")
Inspired by Caelestia, the rail is a vertical tile sitting on the left. It should be a `surface-container-low` element with a `full` rounded top and bottom, separated from the main content by a `16px` gap.

---

## 6. Do’s and Don’ts

### Do:
- **Do** embrace asymmetry. Allow a "Hero" module to span 2/3 of the grid while smaller modules stack in the remaining 1/3.
- **Do** use `primary-fixed-dim` for subtle background washes behind text-heavy sections.
- **Do** prioritize "Breathability." If a layout feels cramped, increase the spacing from `4` (1rem) to `6` (1.5rem).

### Don’t:
- **Don’t** use pure black (#000000) for text. Always use `on-surface` (#2e3335) to maintain the soft, premium feel.
- **Don’t** use square corners. Even a tooltip must have at least a `sm` (0.5rem) radius.
- **Don’t** use high-contrast dividers. If you must separate items in a list, use a 1px line of `surface-container-highest` that fades out at the edges.

---

## 7. Signature Interaction: The "Telos" Glow
When a user interacts with a tile, use a subtle inner-glow (box-shadow: inset) using the `primary-container` color. This simulates the "shell" being energized by the user's presence, reinforcing the high-performance personality of Eleutheria Telos.
