<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/github/github-logo-dark-2000x875.png">
  <source media="(prefers-color-scheme: light)" srcset="images/github/github-logo-light-2000x875.png">
  <img alt="Stamp Validator" src="images/github/github-logo-light-2000x875.png">
</picture>

# Stamp Validator

A small, self-contained browser app for looking up Bitcoin Stamps on-chain metadata by Bitcoin
transaction hash.

## Prerequisites

- **Python 3** (stdlib only)—used for a minimal local HTTP server.
- **A recent desktop browser** with WebAssembly enabled.
- **Internet access** — the app fetches raw transactions from public Bitcoin explorer APIs ([`app.js`](app.js)).
- **Rust toolchain** (`rustup`, Cargo) plus the **`wasm32-unknown-unknown`** target — only needed if you [rebuild the indexer](#build-wasm).

The repository includes **`app.wasm`** so you can run the UI after cloning without building Rust first.

## Quickstart

Clone the repo, `cd` into the project root (the directory that contains `index.html`), then serve that folder:

```sh
git clone <repository-url>
cd <repo-folder>   # clone directory name varies
python3 -m http.server 8000 --bind 127.0.0.1 --directory .
```

Open **`http://localhost:8000/`** in your browser.

Avoid opening **`index.html` via `file://`**: the loader uses `fetch("./app.wasm")`, which reliably works when the app is served over HTTP/S from the same directory.

## Using the app

1. Confirm the status line reads **Stamp indexer loaded** — if not, rebuild or restore [`app.wasm`](app.wasm) (see [Build Wasm](#build-wasm)).
2. Paste a valid **64-character hexadecimal** Bitcoin transaction id.
3. Choose a **Bitcoin node** provider (default **Mempool** → `mempool.space`).
4. Submit the form to fetch the tx, parse it locally in Wasm, and inspect stamp metadata and media.

## Files

- `index.html` — semantic page structure.
- `app.css` — design tokens and responsive UI styles.
- `app.js` — validation, provider selection, transaction fetching, UI state, and Wasm bridging.
- `tx-flow-chart.js` — transaction flow diagram controls (owned separately; `app.js` resets the chart when clearing results).
- `app.wasm` — compiled Rust/Wasm stamp indexer (rebuild via `build-wasm.sh`).
- `indexer/` — Rust source for local transaction parsing, stamp payload extraction, metadata normalization, and media detection.
- `images/` — static assets including GitHub readme header art under `images/github/`.

## Local preview

From the repository root (`index.html` and `app.wasm` should be alongside this file):

```sh
python3 -m http.server 8000 --bind 127.0.0.1 --directory "$(pwd)"
```

Stop the foreground server with **Ctrl+C**.

### Optional: background server

```sh
nohup python3 -m http.server 8000 --bind 127.0.0.1 --directory "$(pwd)" > ~/.localhost-8000.log 2>&1 &
echo $! > ~/.localhost-8000.pid
```

Stop and clean up:

```sh
kill "$(cat ~/.localhost-8000.pid)"
rm -f ~/.localhost-8000.pid ~/.localhost-8000.log
```

Check whether port **8000** is already listening:

```sh
lsof -nP -iTCP:8000 -sTCP:LISTEN
```

## Build Wasm

Install Rust if needed, add the Wasm target, then run the script **from the directory that contains [`build-wasm.sh`](build-wasm.sh)** (same folder as [`app.js`](app.js)):

```sh
rustup target add wasm32-unknown-unknown
sh ./build-wasm.sh
```

This builds [`indexer/`](indexer/) and writes **`app.wasm`** next to the script.

If initialization shows **Build the stamp indexer to enable lookup** or search reports the indexer is unavailable, **`app.wasm` is missing, blocked, or failed to load** (check the browser network tab for `./app.wasm`, rebuild from source, or ensure you are serving the correct directory).

---

The app uses **`mempool.space`** as the default Bitcoin node API, with other public nodes as alternate read-only transaction data sources. Stamp metadata and media are processed locally by the Rust/Wasm indexer.
