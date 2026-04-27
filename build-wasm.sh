#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/rust-indexer"
CARGO_TARGET_DIR=target cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/stamp_explorer_indexer.wasm ../app.wasm
