# Changelog

All notable releases of **Stamp Validator** are documented here.

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
