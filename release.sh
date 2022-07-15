#!/bin/sh -ue

cargo build -q --release --target wasm32-unknown-unknown

wasm-bindgen --out-dir pkg --target web --reference-types --no-typescript --omit-default-module-path \
	     target/wasm32-unknown-unknown/release/index.wasm

echo "The directory $(pwd) should now be usable as a complete wasm web app. Serve this directory and load \"index.htm\" in your web browser."
