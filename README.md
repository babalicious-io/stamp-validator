# Stamp Explorer

A small, self-contained browser app for looking up Bitcoin Stamps by Bitcoin
transaction hash.

## Files

- `index.html` contains the semantic page structure.
- `app.css` contains the design tokens and responsive UI styles.
- `app.js` handles input validation, node selection, raw transaction fetches, UI
  state, and Wasm calls.
- `app.wasm` is the compiled Rust/Wasm stamp indexer.
- `rust-indexer/` contains the Rust source for local transaction parsing, stamp
  payload extraction, metadata normalization, and media detection.

## Local Preview

Use any static file server from the project root, then open the served
`index.html` page in a browser.

## Build Wasm

Install Rust with the `wasm32-unknown-unknown` target, then run:

```sh
rustup target add wasm32-unknown-unknown
sh ./build-wasm.sh
```

The app uses `mempool.space` by default, with Blockstream as an alternate
read-only transaction data source. Stamp metadata and media are processed by the
local Rust/Wasm indexer.

