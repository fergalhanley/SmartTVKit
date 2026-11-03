#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
command -v cargo >/dev/null || { echo "Rust/Cargo is required: https://rustup.rs/" >&2; exit 1; }
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
echo
echo "Open http://localhost:8000/web/ in your browser."
echo "The prototype also loads the official Macroquad JavaScript loader from the internet."
python3 -m http.server 8000
