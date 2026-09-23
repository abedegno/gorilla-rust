#!/usr/bin/env bash
# Build the browser version into web/pkg.
#
# Needs the wasm32-unknown-unknown target and a wasm-bindgen CLI whose
# version matches the wasm-bindgen crate in Cargo.lock exactly. A mismatch
# builds fine and then fails when the page loads, so it is checked here.
set -euo pipefail
cd "$(dirname "$0")/.."

want="$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/"/, "", $3); print $3 }' Cargo.lock)"
have="$(wasm-bindgen --version 2>/dev/null | awk '{ print $2 }' || true)"
if [ "$want" != "$have" ]; then
  echo "wasm-bindgen CLI is '${have:-missing}' but Cargo.lock has $want." >&2
  echo "Install it with: cargo install wasm-bindgen-cli --version $want --locked" >&2
  exit 1
fi

# The library is an rlib everywhere else; only here is it a cdylib, the
# form wasm-bindgen needs, so native builds do not link it twice.
cargo rustc --release --lib --crate-type cdylib --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir web/pkg \
  target/wasm32-unknown-unknown/release/gorillas.wasm
