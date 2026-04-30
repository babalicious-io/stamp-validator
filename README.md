![Stamp Validator](images/stamp-validator-hero.png)

# Stamp Validator - Beta version

A small, self-contained browser app for looking up Bitcoin Stamps on-chain metadata by Bitcoin
transaction hash.

## Files

- `index.html` contains the semantic page structure.
- `app.css` contains the design tokens and responsive UI styles.
- `app.js` handles input validation, node selection, raw transaction fetches, UI
  state, and Wasm calls.
- `app.wasm` is the compiled Rust/Wasm stamp indexer.
- `indexer/` contains the Rust source for local transaction parsing, stamp
  payload extraction, metadata normalization, and media detection.

## Local Preview

Run from any directory (replace the path if needed):

```sh
python3 -m http.server 8000 --bind 127.0.0.1 --directory ~/Development/Apps/"Stamp Validator"/App
```

Then open **`http://localhost:8000/`** — loads `index.html` directly.

To keep the server running after the terminal exits:

```sh
nohup python3 -m http.server 8000 --bind 127.0.0.1 --directory ~/Development/Apps/"Stamp Validator"/App > ~/.localhost-8000.log 2>&1 &
echo $! > ~/.localhost-8000.pid
```

To stop it:

```sh
kill "$(cat ~/.localhost-8000.pid)"
rm -f ~/.localhost-8000.pid ~/.localhost-8000.log
```

To check whether port `8000` is already in use:

```sh
lsof -nP -iTCP:8000 -sTCP:LISTEN
```

## Build Wasm

Install Rust with the `wasm32-unknown-unknown` target, then run:

```sh
rustup target add wasm32-unknown-unknown
sh ./build-wasm.sh
```

The app uses `mempool.space` as the default Bitcoin node, with other public nodes as alternate read-only transaction data sources. Stamp metadata and media are processed by the local Rust/Wasm indexer.

