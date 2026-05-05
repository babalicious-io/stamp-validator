# CSS3 Design Guide

A guide for structuring, naming, and writing CSS for a project. Follow these rules when setting up a new project or adding new classes to an existing one.

---

## File Structure

Organise the stylesheet into clearly labelled sections using block comment banners. Each section groups related rules together. The order should follow the document flow from global setup to specific components.

### Recommended section order

```
1.  External imports           (Google Fonts, third-party CSS)
2.  Root styles & variables    (`:root`, custom properties)
3.  Body styles                (*, html, body, [hidden])
4.  Layout / containers        (page regions: header, main, footer, nav; wrappers, grids, rows, columns)
5.  Typography                 (headings, links, labels, values)
6.  Form controls              (input, select, textarea, label)
7.  Icons
8.  Buttons & interactive controls
9.  Component patterns         (collapsible, tabs, cards, badges, dialogs, status, toasts, etc.)
10. Notifications / toasts     (status messages)
11. Feature-specific components (charts, media previews, etc.)
12. Responsive / media queries (always last)
```

### Section banner format

Use a consistent banner style to separate every section. This makes navigating large files fast.

```css
/* ===================================================================
   SECTION NAME
   =================================================================== */
```

Use a sub-banner for logical groups within a section:

```css
/* ===== Sub-group name ===== */
```

### Cascade layers (large projects)

For larger projects, CSS Cascade Layers (`@layer`) give explicit control over specificity order, preventing conflicts between resets, base styles, and components. Declare all layers at the top of the file before any rules:

```css
@layer reset, base, components, utilities;

@layer reset {
  *, *::before, *::after { box-sizing: border-box; }
}

@layer base {
  /* body, typography, form element defaults */
}

@layer components {
  /* buttons, cards, modals, etc. */
}

@layer utilities {
  /* single-purpose helpers */
}
```

Rules in a later-declared layer always win over rules in an earlier layer, regardless of specificity. This makes overrides predictable without resorting to `!important`.

---

## Custom Properties (CSS Variables)

All design tokens are defined in `:root`. Never hardcode a color, size, shadow, or border value outside of `:root` unless it is a one-off that has no design meaning.

### Token categories


| Category        | Prefix             | Example               |
| --------------- | ------------------ | --------------------- |
| Colors          | `--color-`         | `--color-primary-500` |
| Sizes / spacing | `--size-`          | `--size-16`           |
| Border widths   | `--border-width`   | `--border-width`      |
| Border radii    | `--border-radius-` | `--border-radius-2`   |
| Shadows         | `--shadow-`        | `--shadow-2`          |


### Sizing scale

Size tokens are named after their pixel equivalent (at a 16px base). This keeps mental conversion trivial.

```css
--size-2:  0.125rem;   /*  2px */
--size-4:  0.25rem;    /*  4px */
--size-8:  0.5rem;     /*  8px */
--size-12: 0.75rem;    /* 12px */
--size-14: 0.875rem;   /* 14px */
--size-16: 1rem;       /* 16px */
--size-20: 1.25rem;    /* 20px */
--size-24: 1.5rem;     /* 24px */
--size-36: 2.25rem;    /* 36px */
--size-40: 2.5rem;     /* 40px */
--size-48: 3rem;       /* 48px */
```

Add new size tokens only when a value is used in **more than one place**. One-off magic numbers can live inline.

### Border radius scale

Use a numbered scale, based on the container layer the border lives in. From most rounded to least. `round` is the suffix used for fully circular/pill shapes.

```css
--border-radius-round: 999px;  /* pill / circle */
--border-radius-1: 2.25rem;    /* container-1 */
--border-radius-2: 1.5rem;     /* container-2 */
--border-radius-3: 0.75rem;    /* container-3, inputs, buttons, small panels */
```

### Shadow scale

Shadow tokens are numbered to match the container layer they are primarily used on. Outermost containers cast the strongest shadow; inner layers use progressively lighter shadows.

```css
--shadow-1: 0 8px 32px rgba(0, 0, 0, 0.6);    /* container-1, page-level panels */
--shadow-2: 0 4px 16px rgba(0, 0, 0, 0.4);    /* container-2, modals, dialogs */
--shadow-3: 0 2px 8px  rgba(0, 0, 0, 0.25);   /* container-3, cards, small panels */
```

### Color tokens

See `DESIGN_COLOR_SCHEME.md` for how to define the full color palette. In `:root`, include:

- Primary scale (`--color-primary-50` → `--color-primary-950`) + gradient
- Secondary scale if applicable (+ gradient)
- Neutral scale (`--color-neutral-0` → `--color-neutral-1000`) + gradient
- Status colors (`--color-status-red/green/orange/neutral`)

Always reference tokens by variable in all rules — never use a raw hex value outside `:root`.

```css
/* Correct */
color: var(--color-neutral-300);

/* Wrong */
color: #d4d4d8;
```

---

## Naming Conventions

### Class names

Use **kebab-case** for all class names. Never use camelCase, PascalCase, or underscores.

```css
/* Correct */
.container-grid-2 {}
.button-flat-primary {}
.result-list-row-header {}

/* Wrong */
.containerGrid2 {}
.ButtonFlatPrimary {}
.result_list {}
```

### BEM-inspired naming

Classes follow a loose BEM-style pattern using only single dashes throughout. No double dashes. Use this structure:

```
.block {}                   Base component
.block-element {}           A part of the component
.block-modifier {}          A variant of the component
.block-element-modifier {}  A variant of an element within a component
```

Examples:

```css
.container-row {}            /* block */
.container-row-center {}     /* modifier — centers items */

.button {}                   /* block */
.button-flat-primary {}      /* element-modifier — flat style, primary color */
.button-outline-primary {}   /* element-modifier — outline style, primary color */
.button-block {}             /* modifier — full width */

.collapsible {}              /* block */
.collapsible-header {}       /* element */
.collapsible-toggle {}       /* element */
.collapsible-toggle-icon {}  /* nested element */
```

### Utility classes

Single-purpose classes use a descriptive name that reflects what they do, not how they look:

```css
.container-gap {}       /* adds standard gap */
.container-gap-sm {}    /* adds small gap */
.container-isolate {}   /* sets isolation context */
```

Avoid naming by appearance (`.blue-text`, `.big-font`). Name by role or function (`.value-lg`, `.body-empty-state`).

### Numbered variants

When a component has multiple structural tiers (e.g. nested container depths), use a numeric suffix:

```css
.container-1 {}   /* outermost card */
.container-2 {}   /* inner panel */
.container-3 {}   /* innermost panel */
```

The container number directly maps to its border-radius and shadow tokens. Each layer is slightly less rounded than the one wrapping it, which prevents the inner radius from visually overflowing the outer one. Shadow depth decreases with each inner layer.

```css
.container-1 { border-radius: var(--border-radius-1); box-shadow: var(--shadow-1); }
.container-2 { border-radius: var(--border-radius-2); box-shadow: var(--shadow-2); }
.container-3 { border-radius: var(--border-radius-3); box-shadow: var(--shadow-3); }
```

Elements that live inside a `container-3` (inputs, buttons, small panels) also use `--border-radius-3` so they sit flush with their parent's curvature.

---

## Writing Rules

### Property order (within a rule)

Consistent ordering makes rules easier to scan. Follow this sequence:

```
1. Positioning       (position, top, right, bottom, left, z-index)
2. Display & layout  (display, flex-*, grid-*, place-*, align-*, justify-*)
3. Box model         (width, height, min-*, max-*, padding, margin, border, border-radius)
4. Typography        (font-*, line-height, letter-spacing, text-*, white-space)
5. Visual            (color, background, box-shadow, opacity, overflow)
6. Interaction       (cursor, pointer-events, transition, transform)
```

Example:

```css
.button {
  /* box */
  padding: var(--size-8) var(--size-16);
  border: var(--border-width) solid var(--color-primary-500);
  border-radius: var(--border-radius-3);   /* matches container-3 layer */
  /* typography */
  font-weight: 700;
  letter-spacing: 0.05rem;
  text-align: center;
  white-space: nowrap;
  /* visual */
  color: var(--color-neutral-900);
  background: var(--color-primary-500);
  /* interaction */
  cursor: pointer;
  transition: background-color 0.2s, color 0.2s, border-color 0.2s, opacity 0.2s;
}
```

### Shorthand vs. longhand

Prefer shorthand properties where all sides/values are intentional. Use longhand when only one side is being set to avoid accidentally overriding other values.

```css
/* Good — all four sides intentional */
padding: var(--size-12) var(--size-24);

/* Good — only block-end should change */
margin-block-end: var(--size-8);

/* Avoid — shorthand would reset unintended sides */
margin: 0 0 var(--size-8) 0;
```

### Logical properties

Prefer CSS logical properties over physical ones for better internationalisation support.

```css
/* Preferred */
margin-block-start: 0;
margin-block-end: var(--size-4);
padding-inline-end: var(--size-16);

/* Avoid for new rules */
margin-top: 0;
margin-bottom: var(--size-4);
padding-right: var(--size-16);
```

### Transitions

Always list specific properties in `transition` rather than using `all`. This prevents unintended animation of layout properties (width, height, display).

```css
/* Correct */
transition: background-color 0.2s, color 0.2s, border-color 0.2s, opacity 0.2s;

/* Avoid */
transition: all 0.2s;
```

### Gradient borders

A gradient cannot be applied directly to `border-color`. Instead, use two `background` layers: one clipped to `padding-box` for the fill, and one clipped to `border-box` for the gradient. Set the `border` colour to `transparent` so the gradient shows through.

```css
.container-2 {
  border: var(--border-width) solid transparent;
  border-radius: var(--border-radius-2);
  background:
    /* Fill — clipped inside the border so the gradient rim stays visible */
    linear-gradient(to bottom, var(--color-neutral-1000) 50%, var(--color-neutral-900) 100%) padding-box,
    /* Gradient border — visible only in the transparent border area */
    var(--color-neutral-gradient) border-box;
}
```

Constraints:
- `border-radius` works correctly alongside this technique; the gradient clips to the corner curve.
- Do **not** use `border-image` with `border-radius` — they are incompatible. Always use the two-layer `background` approach instead.
- List the `padding-box` fill layer first; `border-box` is the default clip so the second layer fills the full box including the border area.

### color-mix()

Use `color-mix()` when transparency is semantically required and no solid color works — the primary use case is overlay backdrops (e.g. a modal scrim).

```css
/* Modal scrim — semi-transparent dark overlay */
.modal::backdrop {
  background-color: color-mix(in srgb, var(--color-neutral-900) 80%, transparent);
}
```

Do not use `color-mix()` as a shortcut for a recurring tinted color. If a tint is used in multiple places, define it as a dedicated token in `:root` instead.

---

## Comments

### When to comment

Comments are for intent and context — not narration. Do not describe what the code obviously does.

```css
/* Wrong — narrates the obvious */
/* Set the border radius */
border-radius: var(--border-radius-3);

/* Correct — explains the why or the constraint */
/* Prevents text overflow on narrow viewports */
overflow-wrap: anywhere;
```

### Acceptable comment types

**Sub-group labels** — visually separate logical groups inside a section:

```css
/* ===== Flat primary button ===== */
```

**Inline unit comments** — clarify the px equivalent of rem values:

```css
--size-16: 1rem; /* 16px */
```

**ARIA / state comments** — explain why a rule targets a specific attribute:

```css
/* Invalid state driven by JS setting aria-invalid */
input[aria-invalid="true"] {
  border-color: var(--color-status-red);
}
```

**Workaround comments** — explain browser quirks or hack justifications:

```css
/* Hides native details marker across browsers */
.collapsible-header::-webkit-details-marker {
  display: none;
}
```

---

## Selectors

### Specificity

Keep specificity as low as possible. Prefer class selectors over element or ID selectors. Avoid nesting selectors more than two levels deep.

```css
/* Good — low specificity */
.result-list > div {}

/* Avoid — unnecessarily high specificity */
main .container-2 div.result-list div {}
```

### Element selectors

Use element selectors only for global resets or base typography that should apply universally.

```css
/* Acceptable — universal reset */
* { box-sizing: border-box; }

/* Acceptable — base typography defaults */
h1, h2, h3, h4 { margin-block-start: 0; }

/* Acceptable — form element font inheritance */
button, input, select { font: inherit; }
```

Avoid element selectors inside component rules. Use a class instead.

### Attribute selectors

Use attribute selectors for ARIA-driven state changes. This reinforces accessible markup requirements.

```css
input[aria-invalid="true"] { border-color: var(--color-status-red); }
[hidden] { display: none !important; }
```

### The :has() relational pseudo-class

`:has()` lets a parent element respond to its descendants' state — without JavaScript. It is baseline supported across all modern browsers (Chrome 105+, Firefox 121+, Safari 15.4+).

```css
/* Form field dims its label when the input is disabled */
.form-field:has(input:disabled) label {
  opacity: 0.5;
}

/* Card highlights when its checkbox is checked */
.card:has(input[type="checkbox"]:checked) {
  border-color: var(--color-primary-500);
}

/* Nav shows a submenu when a descendant link is the current page */
.nav-item:has([aria-current="page"]) .nav-submenu {
  display: block;
}
```

Keep `:has()` selectors shallow — target a direct child or one level deep. Deeply nested `:has()` rules are harder to reason about and may have performance implications on large DOMs.

### Focus styles

Always style `:focus-visible` (not `:focus`) to show a focus indicator only for keyboard users, not mouse clicks. Apply it globally in the `:root` section.

```css
:focus-visible {
  outline: var(--size-2) solid var(--color-primary-400);
  outline-offset: var(--size-2);
  border-radius: var(--border-radius-3);
}
```

---

## Typography

### Base reset

Zero out heading margins globally so spacing is always applied intentionally via layout containers.

```css
h1, h2, h3, h4, h5, h6 {
  margin-block-start: 0;
  margin-block-end: 0;
  font-weight: 700;
  line-height: 1.2;
}

p {
  margin-block-start: 0;
  margin-block-end: var(--size-16);
  line-height: 1.6;
  max-width: 65ch;   /* keeps line length comfortable for reading */
}

p:last-child {
  margin-block-end: 0;
}
```

### Heading scale

Map each heading level to a size token. Adjust values to suit the project's type scale.

```css
h1 { font-size: var(--size-48); }
h2 { font-size: var(--size-36); }
h3 { font-size: var(--size-24); }
h4 { font-size: var(--size-16); letter-spacing: 0.03rem; }
```

Add `text-wrap: balance` to headings to prevent awkward single-word last lines. Supported in Chrome 114+, Firefox 121+, Safari 17.4+.

```css
h1, h2, h3 { text-wrap: balance; }
```

### Links

```css
a {
  color: var(--color-primary-500);
  text-decoration: underline;
  text-underline-offset: 0.2em;
  transition: color 0.2s;
}

a:hover  { color: var(--color-primary-400); }
a:visited { color: var(--color-primary-600); }
```

Do not style links as buttons. Use `<a>` only when the destination is a URL; use `<button>` for actions.

### Text utility classes

```css
/* ===== Overline — small uppercase label above a data group ===== */
.overline {
  margin: 0 0 calc(var(--size-4) * -1);
  font-family: Montserrat, "Arial Black", Arial, sans-serif;
  font-size: var(--size-12);
  font-weight: 800;
  color: var(--color-primary-300);
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

/* ===== Value — primary-colored data display ===== */
.value {
  font-size: var(--size-14);
  font-weight: 500;
  color: var(--color-primary-500);
  line-height: 1.6;
  text-transform: uppercase;
}

/* ===== Value large — high-emphasis metric or stat ===== */
.value-lg {
  font-size: var(--size-24);
  font-weight: 800;
  color: var(--color-primary-500);
  line-height: 1.2;
}

/* ===== Label — secondary descriptive text beside a value ===== */
.label {
  font-size: var(--size-14);
  font-weight: 400;
  color: var(--color-neutral-300);
  line-height: 1.6;
}
```

---

## Semantic HTML

### Use proper tags

Prefer semantic elements over generic `<div>`s. Semantic tags carry meaning, reduce required ARIA scaffolding, and give browsers and assistive technologies the context they need.

| Purpose | Use | Avoid |
| --- | --- | --- |
| Disclosure / accordion | `<details>` + `<summary>` | `<div>` + click handler |
| Form group | `<fieldset>` + `<legend>` | `<div class="group">` |
| Field label | `<label for="id">` | `<span>` or `placeholder` only |
| Key-value data | `<dl>` + `<dt>` + `<dd>` | `<div class="label">` + `<div class="value">` |
| Navigation | `<nav>` | `<div class="nav">` |
| Page regions | `<main>`, `<header>`, `<footer>`, `<section>`, `<article>` | `<div>` |
| Interactive control | `<button>` | `<div onclick>` or `<a>` without `href` |

### Page region CSS

Apply base styles directly to the semantic elements rather than wrapping them in extra `<div>`s.

```css
/* ===== Page shell ===== */
body {
  display: flex;
  flex-direction: column;
  min-height: 100dvh;
  margin: 0;
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-block: var(--size-16);
  padding-inline: var(--size-24);
}

main {
  flex: 1;
  padding-block: var(--size-24);
  padding-inline: var(--size-24);
}

footer {
  margin-block-start: auto;
  padding-block: var(--size-16);
  padding-inline: var(--size-24);
  font-size: var(--size-12);
  color: var(--color-neutral-500);
}
```

### Nav link styles

Use `aria-current="page"` on the active link — CSS reads the attribute directly. No JS class toggling needed.

```html
<nav>
  <a class="nav-link" href="/dashboard" aria-current="page">Dashboard</a>
  <a class="nav-link" href="/settings">Settings</a>
</nav>
```

```css
nav {
  display: flex;
  align-items: center;
  gap: var(--size-24);
}

.nav-link {
  color: var(--color-neutral-500);
  font-size: var(--size-14);
  font-weight: 400;
  text-decoration: none;
  white-space: nowrap;
  transition: color 0.2s;
}

.nav-link:hover { color: var(--color-primary-500); }

.nav-link[aria-current="page"] {
  color: var(--color-primary-500);
  cursor: default;
  pointer-events: none;
}
```

JS sets `aria-current="page"` on navigation after each route change; all other links have the attribute removed or set to `false`.

---

### Prefer CSS over JavaScript for UI state

JavaScript should respond to events and update data — not maintain visual state. Most UI patterns can be driven entirely by CSS.

| Pattern | CSS approach | JavaScript role |
| --- | --- | --- |
| Collapsible / accordion | `<details>[open]` attribute + `::details-content` | None required |
| Toggle active state | `button[aria-pressed="true"]` selector | Flip `aria-pressed` attribute only |
| Validation error | `input[aria-invalid="true"]` selector | Set `aria-invalid` attribute only |
| Hover / focus styles | `:hover`, `:focus-visible` | None |
| Disabled state | `button:disabled` | Set `disabled` attribute only |

---

## Component Patterns

### Row layout helpers

`.container-row` and `.container-row-center` are composable modifiers used to build flex rows. They are used as companion classes on container elements throughout the guide.

```css
/* ===== Row layout helpers ===== */
.container-row {
  display: flex;
  justify-content: space-between;
}

.container-row-center {
  align-items: center;
}
```

---

### Collapsible / disclosure

Use `<details>` and `<summary>` — the browser toggles the `[open]` attribute natively. CSS animates the panel via `::details-content`; no JavaScript is needed for open/close.

```html
<details class="collapsible container-3">
  <summary class="collapsible-header container-row container-row-center">
    <span>Section title</span>
    <button class="collapsible-toggle" type="button" tabindex="-1" aria-hidden="true">
      <img class="collapsible-toggle-icon" src="./images/icons/plus.svg" alt="">
    </button>
  </summary>
  <div class="collapsible-body">
    <p>Content goes here.</p>
  </div>
</details>
```

```css
/* Animate the disclosure panel */
.collapsible { transition-behavior: allow-discrete; }

.collapsible::details-content {
  block-size: 0;
  overflow: hidden;
  opacity: 0;
  transition:
    block-size 400ms ease-in-out,
    content-visibility 400ms ease-in-out allow-discrete,
    opacity 400ms ease-in-out;
}

.collapsible[open]::details-content { block-size: auto; opacity: 1; }

/* Icon rotates when open — driven by [open], no JS class toggle needed */
.collapsible[open] .collapsible-toggle-icon { transform: rotate(45deg); }

/* Remove the native browser marker */
.collapsible-header { list-style: none; }
.collapsible-header::-webkit-details-marker { display: none; }
```

The icon rotation is purely CSS — the `[open]` attribute selector handles it. The `tabindex="-1"` and `aria-hidden="true"` on the icon button keep it out of the focus order since `<summary>` itself is already the interactive element.

> **Browser support:** `::details-content` requires Chromium 131+ and is not yet supported in Firefox (as of mid-2026). For broader support, use a `max-height` transition on the body element as a fallback, or check [caniuse.com](https://caniuse.com/mdn-css_selectors_details-content) before adopting in production.

---

### Form with input field

```html
<form>
  <fieldset>
    <legend>Verify stamp</legend>

    <div class="form-field">
      <label for="stamp-id">Stamp ID</label>
      <input
        id="stamp-id"
        type="text"
        name="stamp-id"
        placeholder="e.g. ABC-12345"
        autocomplete="off"
        required
      >
    </div>

    <div class="form-field">
      <label for="issue-date">Issue date</label>
      <input id="issue-date" type="date" name="issue-date">
    </div>

    <button class="button button-flat-primary button-block" type="submit">
      Validate
    </button>
  </fieldset>
</form>
```

**CSS:**

```css
/* ===== Form field wrapper ===== */
.form-field {
  display: flex;
  flex-direction: column;
  gap: var(--size-4);
}

.form-field label {
  font-size: var(--size-12);
  font-weight: 600;
  color: var(--color-neutral-500);
  text-transform: uppercase;
  letter-spacing: 0.05rem;
}

/* ===== Input field ===== */
.input-field {
  width: 100%;
  padding: var(--size-8) var(--size-12);
  border: var(--border-width) solid var(--color-neutral-500);
  border-radius: var(--border-radius-3);
  background-color: var(--color-neutral-900);
  color: var(--color-neutral-100);
  font-size: var(--size-14);
  font-weight: 400;
  font-family: inherit;
}

.input-field::placeholder {
  color: var(--color-neutral-600);
}

.input-field:focus {
  outline: none;
  border-color: var(--color-primary-500);
}
```

Rules:
- Always pair every `<input>` with a `<label for="id">` — never rely on `placeholder` alone.
- Use `<fieldset>` + `<legend>` to group related fields with a visible label.
- Specify `type="submit"` on the submit button explicitly.
- Validation error state is set by JS via `aria-invalid="true"` and styled in CSS:

```css
input[aria-invalid="true"] { border-color: var(--color-status-red); }
```

---

### Buttons

All buttons use `.button` as the base class plus a style modifier. Add `.button-block` for full width.

```html
<!-- Flat primary — filled, high emphasis -->
<button class="button button-flat-primary" type="button">Confirm</button>

<!-- Outline primary — bordered, medium emphasis -->
<button class="button button-outline-primary" type="button">Cancel</button>

<!-- Full-width submit -->
<button class="button button-flat-primary button-block" type="submit">Submit</button>

<!-- Disabled — JS sets the disabled attribute; CSS handles opacity -->
<button class="button button-flat-primary" type="button" disabled>Unavailable</button>
```

Never use `<div>` or `<a>` as a button. Use `<button>` for all controls that trigger an action rather than navigate. Use `<a>` only when the destination is a URL.

**CSS:**

```css
/* ===== Base ===== */
.button {
  padding: var(--size-8) var(--size-16);
  border-radius: var(--border-radius-3);
  font-size: var(--size-12);
  font-weight: 600;
  letter-spacing: 0.5px;
  white-space: nowrap;
  cursor: pointer;
  text-transform: uppercase;
  text-align: center;
  transition: background-color 0.2s, color 0.2s, border-color 0.2s, opacity 0.2s;
}

/* ===== Full width ===== */
.button-block { width: 100%; }

/* ===== Flat primary ===== */
.button-flat-primary {
  border: var(--border-width) solid var(--color-primary-500);
  background-color: var(--color-primary-500);
  color: var(--color-neutral-0);
}

.button-flat-primary:hover {
  background-color: transparent;
  color: var(--color-primary-500);
}

/* ===== Outline primary ===== */
.button-outline-primary {
  border: var(--border-width) solid var(--color-primary-500);
  background-color: transparent;
  color: var(--color-primary-500);
}

.button-outline-primary:hover {
  background-color: var(--color-primary-500);
  color: var(--color-neutral-0);
}

/* ===== Outline secondary ===== */
.button-outline-secondary {
  border: var(--border-width) solid var(--color-primary-700);
  background-color: transparent;
  color: var(--color-primary-700);
}

.button-outline-secondary:hover {
  background-color: var(--color-primary-700);
  color: var(--color-neutral-0);
}

/* ===== Outline neutral ===== */
.button-outline-neutral {
  border: var(--border-width) solid var(--color-neutral-500);
  background-color: transparent;
  color: var(--color-neutral-500);
}

.button-outline-neutral:hover {
  background-color: var(--color-neutral-500);
  color: var(--color-neutral-0);
}

/* ===== Disabled ===== */
.button:disabled {
  opacity: 0.5;
  cursor: wait;
}
```

---

### Toggle button

Use `aria-pressed` on a `<button>` to represent a binary on/off state. CSS reads the attribute to apply the active style — JavaScript only needs to flip the attribute value.

```html
<button class="button button-outline-primary toggle" type="button" aria-pressed="false">
  Show advanced
</button>
```

```css
.toggle[aria-pressed="true"] {
  background-color: var(--color-primary-500);
  color: var(--color-neutral-900);
  border-color: var(--color-primary-500);
}
```

```js
// JS only manages state — CSS handles the visual
toggleBtn.addEventListener('click', () => {
  const pressed = toggleBtn.getAttribute('aria-pressed') === 'true';
  toggleBtn.setAttribute('aria-pressed', String(!pressed));
});
```

---

### Toggle switch

A sliding track-and-thumb toggle for checkbox or radio inputs. The native `<input>` is visually hidden (but still in the DOM for accessibility) and the visual track and thumb are built from sibling elements driven by CSS `:checked`.

**HTML structure:**

```html
<label class="toggle-switch">
  <input class="toggle-switch-input" type="checkbox" role="switch">
  <span class="toggle-switch-slider">
    <span class="toggle-switch-track"></span>
    <span class="toggle-switch-thumb"></span>
  </span>
  <span class="toggle-switch-label">Enable notifications</span>
</label>
```

Use `type="checkbox"` for a single on/off toggle. Use `type="radio"` when part of a mutually exclusive group. Add `role="switch"` on the input so screen readers announce checked/unchecked as on/off.

**CSS:**

```css
/* ===== Toggle switch ===== */
.toggle-switch {
  display: inline-flex;
  align-items: center;
  gap: var(--size-12);
  cursor: pointer;
  user-select: none;
}

/* Hide the native input visually but keep it accessible */
.toggle-switch-input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-switch-slider {
  position: relative;
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
}

/* Track */
.toggle-switch-track {
  display: inline-block;
  width: 2.25rem;      /* 36px */
  height: 1.25rem;     /* 20px */
  border-radius: var(--border-radius-round);
  background-color: var(--color-neutral-300);
  transition: background-color 0.2s ease;
  overflow: hidden;
}

/* Thumb */
.toggle-switch-thumb {
  position: absolute;
  inset-block-start: 0.125rem;    /* 2px */
  inset-inline-start: 0.125rem;   /* 2px */
  width: 1rem;         /* 16px */
  height: 1rem;        /* 16px */
  border-radius: var(--border-radius-round);
  background-color: var(--color-neutral-0);
  transition: transform 0.2s ease;
  z-index: 1;
}

/* Checked state — track changes color */
.toggle-switch-input:checked + .toggle-switch-slider .toggle-switch-track {
  background-color: var(--color-neutral-700);
}

/* Checked state — thumb slides to the right and changes color */
.toggle-switch-input:checked + .toggle-switch-slider .toggle-switch-thumb {
  transform: translateX(100%);
  background-color: var(--color-neutral-900);
}

/* Disabled state */
.toggle-switch-input:disabled + .toggle-switch-slider {
  opacity: 0.5;
  cursor: not-allowed;
}

.toggle-switch:has(.toggle-switch-input:disabled) {
  cursor: not-allowed;
}

/* Focus ring on keyboard navigation */
.toggle-switch-input:focus-visible + .toggle-switch-slider .toggle-switch-track {
  outline: var(--size-2) solid var(--color-primary-400);
  outline-offset: var(--size-2);
}

/* Label text */
.toggle-switch-label {
  font-size: var(--size-14);
  font-weight: 500;
  color: var(--color-neutral-300);
  line-height: 1.3;
}
```

No JavaScript is needed for the visual. JS is only required if you need to react to the state change:

```js
document.querySelector('.toggle-switch-input').addEventListener('change', (e) => {
  console.log('Toggle is now:', e.target.checked ? 'on' : 'off');
});
```

---

### Tab system

Use `role="tablist"`, `role="tab"`, and `role="tabpanel"` so screen readers understand the tab relationship. CSS reads `aria-selected` for visual state — JavaScript only manages the attribute values and `hidden`.

**HTML structure:**

```html
<nav role="tablist" aria-label="Content sections">
  <button role="tab" class="tab" aria-selected="true"
          aria-controls="panel-posts" id="tab-posts">Posts</button>
  <button role="tab" class="tab" aria-selected="false"
          aria-controls="panel-media" id="tab-media">Media</button>
</nav>

<div role="tabpanel" id="panel-posts" aria-labelledby="tab-posts">
  <!-- panel content -->
</div>
<div role="tabpanel" id="panel-media" aria-labelledby="tab-media" hidden>
  <!-- panel content -->
</div>
```

**CSS:**

```css
/* ===== Tab buttons ===== */
.tab {
  cursor: pointer;
  transition: color 0.2s;
}

.tab[aria-selected="true"] {
  color: var(--color-neutral-600);
  cursor: default;
}

.tab[aria-selected="false"] {
  color: var(--color-neutral-300);
}

.tab[aria-selected="false"]:hover {
  color: var(--color-neutral-600);
}

/* ===== Tab panels ===== */
/* Each panel retains its own display value; hidden attribute removes it from view */
[role="tabpanel"][hidden] { display: none; }
```

Do not use `display: block` to show a panel — the panel keeps whatever `display` its layout requires. Only the `hidden` attribute controls visibility.

**JS contract:**

```js
function activateTab(selectedTab) {
  const tabs   = document.querySelectorAll('[role="tab"]');
  const panels = document.querySelectorAll('[role="tabpanel"]');

  tabs.forEach(tab => {
    tab.setAttribute('aria-selected', tab === selectedTab ? 'true' : 'false');
  });

  panels.forEach(panel => {
    const isActive = panel.id === selectedTab.getAttribute('aria-controls');
    panel.hidden = !isActive;
  });
}

document.querySelectorAll('[role="tab"]').forEach(tab => {
  tab.addEventListener('click', () => activateTab(tab));
});
```

JS only manages `aria-selected` and `hidden` — never adds or removes CSS classes.

#### Data-attribute conditional visibility

When a tab change should also affect elements scattered across the page (show/hide buttons, swap labels), set a `data-*` attribute on a common ancestor and target descendants with CSS. This avoids per-element JS toggling.

```html
<div class="container" data-active-tab="media">
  <button class="hide-on-media">Post</button>
  <button class="show-on-media">Upload</button>
</div>
```

```css
/* Default: hide elements that only appear in the media tab */
.show-on-media { display: none; }

/* When the media tab is active, flip visibility */
[data-active-tab="media"] .hide-on-media { display: none; }
[data-active-tab="media"] .show-on-media { display: block; }
```

JS sets the attribute on the ancestor; CSS handles all the show/hide logic.

```js
container.dataset.activeTab = selectedTab.getAttribute('aria-controls').replace('panel-', '');
```

---

### Custom scrollbar

The `.scrollbar` class is an opt-in utility. Apply it to any scrollable container to replace the browser default with a minimal, on-brand thumb.

```css
/* ===== Custom scrollbar ===== */
.scrollbar {
  scrollbar-width: thin;                                 /* Firefox */
  scrollbar-color: var(--color-neutral-100) transparent; /* Firefox: thumb track */
}

/* Webkit (Chrome, Safari, Edge) */
.scrollbar::-webkit-scrollbar        { width: 4px; }
.scrollbar::-webkit-scrollbar-track  { background: transparent; }
.scrollbar::-webkit-scrollbar-thumb  {
  background-color: var(--color-neutral-100);
  border-radius: 2px;
}
.scrollbar::-webkit-scrollbar-corner { background: transparent; }
```

- `scrollbar-width: thin` is the Firefox equivalent — the two-value `scrollbar-color` sets thumb then track.
- Do not apply globally; keep it as an opt-in class so layouts that should not scroll are unaffected.
- On mobile you may increase `::-webkit-scrollbar { width: 6px }` inside a media query for easier touch targeting.

---

### Toast notification

Fixed-position feedback messages that appear on top of the UI. Background and border both use the status color token directly.

**HTML:**

```html
<div class="message success" role="status" aria-live="polite">
  Post published successfully.
</div>

<div class="message error" role="alert">
  Connection failed. Please try again.
</div>
```

Use `role="status"` + `aria-live="polite"` for non-urgent confirmations. Use `role="alert"` for errors that need immediate attention.

**CSS:**

```css
/* ===== Toast base ===== */
.message {
  position: fixed;
  top: var(--size-20);
  left: var(--size-20);
  width: 320px;
  padding: var(--size-16) var(--size-20);
  border-radius: var(--border-radius-2);
  font-size: var(--size-14);
  color: var(--color-neutral-900);
  line-height: 1.4;
  letter-spacing: 0.03rem;
  white-space: pre-line;
  z-index: 999;
  pointer-events: none;
}

/* ===== Toast variants ===== */
.message.success {
  background-color: var(--color-status-green);
  border: var(--border-width) solid var(--color-status-green);
  pointer-events: auto;
}

.message.error {
  background-color: var(--color-status-red);
  border: var(--border-width) solid var(--color-status-red);
  pointer-events: auto;
}
```

`pointer-events: none` on the base prevents the invisible default state from blocking clicks. Re-enable it on active variants so users can dismiss the toast if needed.

---

### Status indicator

Use a CSS custom property cascade to theme a status sub-tree. A modifier class on a container sets `--status-color`; all children that use `var(--status-color)` inherit it automatically.

**HTML:**

```html
<div class="status-indicator status-green">
  <span class="status-dot"></span>
  <span class="status-text">Connected</span>
</div>

<div class="status-indicator status-red">
  <span class="status-dot"></span>
  <span class="status-text">Offline</span>
</div>
```

**CSS:**

```css
/* ===== Status color modifiers — set the cascade variable ===== */
.status-red    { --status-color: var(--color-status-red);    }
.status-green  { --status-color: var(--color-status-green);  }
.status-orange { --status-color: var(--color-status-orange); }

/* ===== Status indicator container ===== */
.status-indicator {
  --status-color: var(--color-status-red); /* default fallback */
  display: flex;
  align-items: center;
  gap: var(--size-8);
  color: var(--status-color);
  transition: color 0.3s ease;
}

/* ===== Status dot ===== */
.status-dot {
  width: 0.5rem;    /*  8px */
  height: 0.5rem;   /*  8px */
  border-radius: var(--border-radius-round);
  background-color: var(--status-color);
  flex-shrink: 0;
}

/* ===== Status text ===== */
.status-text {
  font-size: var(--size-12);
  font-weight: 500;
  letter-spacing: 0.03rem;
  line-height: 1;
  text-transform: uppercase;
}
```

The modifier classes (`.status-red`, `.status-green`) only set the custom property — they never repeat color values. Any new element added inside the container that uses `var(--status-color)` inherits the right color without extra rules.

To change status at runtime, JS swaps only the modifier class:

```js
indicator.classList.remove('status-red', 'status-green', 'status-orange');
indicator.classList.add('status-green');
```

---

### Figure

Use `<figure>` for self-contained media (images, diagrams, code blocks) that is referenced from the main content. Use `<figcaption>` for the associated caption.

```html
<figure class="figure">
  <img src="chart.png" alt="Monthly signups bar chart showing growth from Jan to Jun">
  <figcaption class="figure-caption">Monthly signups, Jan–Jun 2026</figcaption>
</figure>
```

```css
/* ===== Figure ===== */
.figure {
  margin: 0;
}

.figure img,
.figure video {
  display: block;
  width: 100%;
  height: auto;
  overflow: hidden;
  border-radius: var(--border-radius-3);
}

/* ===== Figure caption ===== */
.figure-caption {
  padding-block-start: var(--size-8);
  font-size: var(--size-12);
  color: var(--color-neutral-500);
  line-height: 1.4;
}
```

---

### Key-value list

Use `<dl>` (description list) for labelled data pairs. Each pair is wrapped in a `<div>` so CSS grid can align the term and value into columns.

```html
<dl class="data-list">
  <div class="data-list-row">
    <dt class="data-list-term">Status</dt>
    <dd class="data-list-value">Active</dd>
  </div>
  <div class="data-list-row">
    <dt class="data-list-term">Created</dt>
    <dd class="data-list-value">2 May 2026</dd>
  </div>
</dl>
```

```css
/* ===== Data list ===== */
.data-list {
  display: flex;
  flex-direction: column;
  gap: var(--size-8);
  margin: 0;
}

.data-list-row {
  display: grid;
  grid-template-columns: minmax(6rem, max-content) 1fr;
  gap: var(--size-16);
  align-items: baseline;
}

.data-list-term {
  font-size: var(--size-12);
  font-weight: 600;
  color: var(--color-neutral-500);
  text-transform: uppercase;
  letter-spacing: 0.05rem;
}

.data-list-value {
  margin: 0;
  font-size: var(--size-14);
  font-weight: 500;
  color: var(--color-neutral-200);
}
```

---

### Form with select and textarea

Extend the form input styles with `<select>` and `<textarea>`. Both share the same sizing and focus ring as `.input-field`.

**Select:**

```html
<div class="form-field">
  <label for="category">Category</label>
  <div class="select-wrapper">
    <select class="input-field" id="category" name="category">
      <option value="">Choose…</option>
      <option value="a">Option A</option>
      <option value="b">Option B</option>
    </select>
  </div>
</div>
```

```css
/* ===== Select wrapper — provides the custom chevron ===== */
.select-wrapper {
  position: relative;
}

.select-wrapper::after {
  content: "";
  position: absolute;
  inset-inline-end: var(--size-12);
  inset-block-start: 50%;
  transform: translateY(-50%);
  width: 0;
  height: 0;
  border-inline: 0.3rem solid transparent;
  border-block-start: 0.35rem solid var(--color-neutral-500);
  pointer-events: none;
}

/* ===== Select element ===== */
select.input-field {
  appearance: none;
  padding-inline-end: var(--size-36);   /* room for the chevron */
  cursor: pointer;
}
```

**Textarea:**

```html
<div class="form-field">
  <label for="notes">Notes</label>
  <textarea class="input-field" id="notes" name="notes"
            rows="4" placeholder="Enter notes…"></textarea>
</div>
```

```css
/* ===== Textarea ===== */
textarea.input-field {
  resize: vertical;
  min-height: 8rem;   /* 128px — at least 4 visible rows */
  font-family: inherit;
  line-height: 1.5;
}
```

The emerging `field-sizing: content` property (Chrome 123+) auto-grows a textarea to fit its content without JavaScript. Use as a progressive enhancement alongside `min-height`:

```css
textarea.input-field {
  field-sizing: content;   /* auto-grow where supported */
}
```

---

### Card

A card is a self-contained content item — use it when displaying a repeating list of independent items (results, products, posts). Use `<article>` as the element since each card is independently meaningful.

```html
<article class="card">
  <header class="card-header">
    <h3 class="card-title">Card title</h3>
  </header>
  <div class="card-body">
    <p>Card content goes here.</p>
  </div>
  <footer class="card-footer">
    <button class="button button-outline-primary" type="button">Action</button>
  </footer>
</article>
```

```css
/* ===== Card ===== */
.card {
  display: flex;
  flex-direction: column;
  border: var(--border-width) solid var(--color-neutral-800);
  border-radius: var(--border-radius-2);
  overflow: hidden;
}

.card-header {
  padding: var(--size-16) var(--size-16) 0 var(--size-16);
}

.card-title {
  font-size: var(--size-16);
  font-weight: 700;
  color: var(--color-neutral-100);
}

.card-body {
  flex: 1;
  padding: var(--size-16);
}

.card-footer {
  padding: 0 var(--size-16) var(--size-16) var(--size-16);
  display: flex;
  justify-content: flex-end;
  gap: var(--size-8);
}
```

Cards are distinct from `container-*` classes: `container-*` is a generic layout shell; a card is a semantic component representing one item in a collection. A card typically uses the same border and radius tokens as `container-2`.

---

### Badge / pill

Small inline labels for counts, statuses, and tags.

```html
<span class="badge">New</span>
<span class="badge badge-primary">12</span>
<span class="badge badge-success">Active</span>
<span class="badge badge-warning">Pending</span>
<span class="badge badge-error">Failed</span>
```

```css
/* ===== Badge base ===== */
.badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.15rem var(--size-8);
  border-radius: var(--border-radius-round);
  font-size: var(--size-12);
  font-weight: 600;
  letter-spacing: 0.04rem;
  text-transform: uppercase;
  line-height: 1.4;
  white-space: nowrap;
  color: var(--color-neutral-900);
  background-color: var(--color-neutral-700);
}

/* ===== Badge variants ===== */
.badge-primary { background-color: var(--color-primary-500); }
.badge-success  { background-color: var(--color-status-green); }
.badge-warning  { background-color: var(--color-status-orange); }
.badge-error    { background-color: var(--color-status-red); }
```

---

### Dialog / modal

Use the native `<dialog>` element. The browser provides the `open` attribute, `::backdrop`, and accessibility semantics out of the box — no custom overlay `<div>` is needed. JS only calls `dialog.showModal()` and `dialog.close()`.

```html
<dialog class="modal" id="confirm-dialog" aria-labelledby="modal-title">
  <header class="modal-header">
    <h2 class="modal-title" id="modal-title">Confirm action</h2>
  </header>
  <div class="modal-body">
    <p>Are you sure you want to delete this item? This cannot be undone.</p>
  </div>
  <footer class="modal-footer">
    <button class="button button-outline-neutral" type="button" data-close-modal>Cancel</button>
    <button class="button button-flat-primary" type="button">Confirm</button>
  </footer>
</dialog>
```

```css
/* ===== Modal backdrop ===== */
.modal::backdrop {
  background-color: color-mix(in srgb, var(--color-neutral-900) 80%, transparent);
  backdrop-filter: blur(var(--size-4));
}

/* ===== Modal container ===== */
.modal {
  width: min(480px, 90vw);
  padding: 0;
  border: var(--border-width) solid var(--color-neutral-800);
  border-radius: var(--border-radius-2);
  background-color: var(--color-neutral-900);
  color: var(--color-neutral-100);
  box-shadow: var(--shadow-2);
}

/* ===== Modal sections ===== */
.modal-header {
  padding: var(--size-24) var(--size-24) 0 var(--size-24);
}

.modal-title {
  font-size: var(--size-16);
  font-weight: 700;
}

.modal-body {
  padding: var(--size-16) var(--size-24);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--size-8);
  padding: 0 var(--size-24) var(--size-24) var(--size-24);
}
```

```js
const dialog = document.getElementById('confirm-dialog');

// Open
document.querySelector('[data-open-modal]').addEventListener('click', () => {
  dialog.showModal();
});

// Close via Cancel button or clicking the backdrop
document.querySelector('[data-close-modal]').addEventListener('click', () => {
  dialog.close();
});

dialog.addEventListener('click', (e) => {
  if (e.target === dialog) dialog.close();   /* click outside */
});
```

Pressing `Escape` closes the dialog automatically — no JS needed for that.

---

### Segmented progress bar

A step-based visual progress indicator. Use `role="progressbar"` and `aria-valuenow` / `aria-valuemin` / `aria-valuemax` so assistive technologies can announce progress.

```html
<div class="progress-bar" role="progressbar"
     aria-valuenow="3" aria-valuemin="0" aria-valuemax="5"
     aria-label="Step 3 of 5">
  <span class="progress-bar-step filled"></span>
  <span class="progress-bar-step filled"></span>
  <span class="progress-bar-step filled"></span>
  <span class="progress-bar-step"></span>
  <span class="progress-bar-step"></span>
</div>
```

```css
/* ===== Progress bar ===== */
.progress-bar {
  display: flex;
  width: 100%;
  height: 0.25rem;   /* 4px */
  gap: 2px;
}

/* ===== Steps ===== */
.progress-bar-step {
  flex: 1;
  background-color: var(--color-neutral-700);
}

.progress-bar-step:first-child {
  border-start-start-radius: var(--border-radius-round);
  border-end-start-radius: var(--border-radius-round);
}

.progress-bar-step:last-child {
  border-start-end-radius: var(--border-radius-round);
  border-end-end-radius: var(--border-radius-round);
}

.progress-bar-step.filled {
  background-color: var(--color-primary-500);
}
```

JS updates `aria-valuenow` and adds/removes the `.filled` class on each step when progress changes.

---

## Responsive Design

### Media queries placement

All media queries go at the **bottom of the file**, after all component rules. Do not scatter `@media` blocks inline within a component's rules.

### Breakpoints

Use `px`-based breakpoints for predictable gates that match design specs and common device widths.


| Name    | Value               | Typical use              |
| ------- | ------------------- | ------------------------ |
| Small   | `max-width: 768px`  | Tablet and below         |
| XSmall  | `max-width: 568px`  | Mobile                   |
| XXSmall | `max-width: 420px`  | Small mobile             |


Define breakpoints in order from largest to smallest (mobile-last) when writing desktop-first styles, or smallest to largest (mobile-first) if writing in a mobile-first approach. Be consistent throughout the project.

### Container queries

Container queries (`@container`) let a component respond to the size of its own parent container rather than the viewport. Use them for components that are placed at different widths in different parts of the layout.

```css
/* Mark the wrapper as a container */
.card-container {
  container-type: inline-size;
  container-name: card;
}

/* Component adapts to its container, not the viewport */
@container card (min-width: 400px) {
  .card {
    flex-direction: row;
  }
}
```

Container queries are baseline 2023 (Chrome 105+, Firefox 110+, Safari 16+). Prefer them over viewport `@media` queries when a component's layout should depend on where it is placed, not how wide the browser window is.

### What to put in media queries

Only override properties that genuinely differ at that breakpoint. Do not re-declare entire rule blocks.

```css
@media (max-width: 768px) {
  h1 { font-size: 4rem; }                              /* override only font-size */
  .container-grid-3 { grid-template-columns: 1fr; }    /* collapse to single column */
}
```

---

## Adding New Classes

When adding a new class to the project, follow this checklist:

1. **Does it belong in an existing section?** Place it there, not at the bottom of the file.
2. **Is it a variant of an existing component?** Add it adjacent to its parent class using the modifier naming pattern.
3. **Are any values hardcoded that should be tokens?** Add the token to `:root` first.
4. **Is the class name already taken or similar to an existing one?** Check before adding to avoid duplication.
5. **Is it truly reusable?** If it will only ever be used once, consider using an inline style or a more specific selector on an existing element instead of creating a new class.
6. **Add a sub-group comment** if the new class starts a new logical group within a section.

---

## General Recommendations

- **Use static `rem` or `px` values for sizing** and control layout changes through media query breakpoints rather than fluid `min()`, `max()`, or `clamp()` functions. For example: `width: 100%` at mobile, overridden to `width: 800px` at the appropriate breakpoint.
- **Use `fit-content`, `min-content`, `max-content`** for width/height where dimensions should derive from content.
- **Avoid `!important`** except for hard overrides that are universally applicable and intentional (e.g. `[hidden] { display: none !important; }`).
- **Avoid fixed pixel heights.** Let containers grow with content using `height: fit-content` or `min-height`.
- **Use `overflow: hidden` on containers** that clip child border-radius visuals (e.g. image frames inside rounded cards).
- **Use `isolation: isolate`** on containers that need a new stacking context for `z-index` management, rather than setting arbitrary high `z-index` values.
- **Use `gap` instead of margins** inside flex and grid containers for consistent spacing that does not bleed to the container edges.
- **Set `font: inherit`** on form elements (`button`, `input`, `select`) globally — browsers do not inherit font by default.
- **Prefix `--size-` comments with px equivalents** to keep the scale readable without mental maths.

