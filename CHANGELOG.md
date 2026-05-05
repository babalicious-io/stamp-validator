# Changelog

All notable releases of **Stamp Validator** are documented here.

---

## [1.2] — 2026-05-05

### Changed

- **Stylesheet audit against design spec** — Systematically compared `app.css` against `DESIGN_CSS_3.md` design rules and resolved all critical and high-priority discrepancies.

#### Tokens & variables
- Added missing size tokens: `--size-20` (20px), `--size-64` (64px), `--size-96` (96px).
- Corrected `--shadow-1` value to `0 8px 32px rgba(0, 0, 0, 0.6)`.
- Replaced `--size-0` with bare `0` literal in `nav` padding (undefined variable).
- Tokenised `h1` font-size using `--size-96`; `768px` breakpoint override uses `--size-64`.

#### Layout & structure
- Fixed `.container-row--center` double-dash naming to `.container-row-center` across `app.css` and all 10 occurrences in `index.html`.
- Corrected `body` minimum height from `100vh` to `100dvh` for accurate mobile viewport behaviour.
- Moved **Toggle Switch** and **Collapsible** sections to follow Buttons in the stylesheet, matching the recommended section order.

#### Typography
- Added global heading reset: `margin-block-end: 0`, `font-weight: 700`, `line-height: 1.2` on `h1`–`h6`.
- Added `text-wrap: balance` on `h1`, `h2`, `h3` to prevent orphaned words on narrow viewports.
- Added `p:last-child { margin-block-end: 0 }` to prevent unwanted bottom spacing on last paragraphs.
- Added `text-underline-offset: 0.2em` and `transition: color 0.2s` to `a` base styles.
- Removed unused `.value-lg` rule.

#### Buttons
- Added `font-size: var(--size-12)` and `text-transform: uppercase` to `.button` base.
- Corrected `.button` `font-weight` from `700` to `600`.

#### Forms
- Changed `.form-field` layout from `display: grid` to `display: flex; flex-direction: column` with `gap: var(--size-4)`.

#### Toast notifications
- Replaced no-op `color-mix(…, 100%, transparent)` calls in `.message` variants with direct token references.
- Fixed `.message` base: corrected `top`/`left` to `var(--size-20)`, `padding`, `color`, `line-height`, `letter-spacing`, `z-index`, and set `pointer-events: none` on base (re-enabled on active variants only).
- Replaced hardcoded `border-radius: 1.25rem` with `var(--border-radius-2)`.

### Notes

#### Light / dark theming
The app supports three theme states, all handled in CSS with no JavaScript beyond reading and writing a `localStorage` key on page load.

- **System default** — A `@media (prefers-color-scheme: dark)` block overrides the semantic theme tokens (prefixed `--theme-`) when the OS is set to dark mode. Light mode is the baseline defined in `:root`.
- **Manual override** — A `[data-theme="dark"]` / `[data-theme="light"]` attribute set on `<html>` takes explicit precedence over the system preference. This lets users pin their choice regardless of OS setting.
- **Toggle switch** — The nav toggle writes the chosen value to `localStorage` and sets `document.documentElement.dataset.theme`. On next load, an inline `<script>` in `<head>` reads the stored value before the first paint, preventing any flash of incorrect theme.
- **Semantic tokens** — All color rules reference `--theme-*` tokens rather than raw palette tokens, so swapping the theme requires only overriding those tokens in the relevant selector — no rule duplication needed.
- **Smooth transitions** — `html` carries `transition: background-color 0.2s ease, color 0.2s ease` so theme switches animate rather than snap.

---

## [1.1] — 2026-05-01

### Added

- **HTML stamp descriptions** — Extracts and displays `<meta name="description">` content beneath the title when available.

---

## [1.0] — 2026-04-30

**Status:** Production ready.

### Overview

Stamp Validator is a browser-based tool to look up **Bitcoin Stamps** on-chain by **Bitcoin transaction hash**. It fetches raw transaction data from public Bitcoin nodes, runs a local **Rust/WebAssembly** indexer in the browser, and shows decoded stamp metadata and media without relying on a backend for parsing.

### Features

- **Validate by tx hash** — Accepts a 64-character hex transaction ID with clear validation messages.
- **Multiple node providers** — Choose among public read-only APIs (e.g. Mempool, Blockstream, Blockchain.info, BlockCypher, Blockchair) for raw transaction data.
- **On-chain stamp decoding** — Extracts and normalizes stamp payloads; supports classic art Stamps and **SRC-20** token metadata where present in the transaction.
- **Rich result UI** — Media preview when applicable, optional image-encoding details, **stamp details** and **Bitcoin details** panels, and a **transaction flow** visualization for inputs and outputs.
- **Self-contained frontend** — Static HTML, CSS, and JS with Wasm; suitable for static hosting and local preview.

### Known limitations & roadmap

- **RC-101 metadata** — Structured parsing and presentation of **RC-101** stamp metadata is **not** included in this release; it is planned for a future version.
