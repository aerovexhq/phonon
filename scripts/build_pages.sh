#!/usr/bin/env bash
# ==============================================================================
# Production Build Pipeline for Cloudflare Pages (Docs & Web Studio)
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== [1/4] Ensuring Rust wasm32 target ==="
if command -v rustup >/dev/null 2>&1; then
  rustup target add wasm32-unknown-unknown 2>/dev/null || true
fi

echo "=== [2/4] Building WebAssembly Studio Engine ==="
cd "${ROOT_DIR}"
cargo build --release -p phonon-gui --target wasm32-unknown-unknown

WASM_TARGET="${ROOT_DIR}/target/wasm32-unknown-unknown/release/phonon_gui.wasm"
OUT_DIR="${ROOT_DIR}/web/studio/public/wasm"
mkdir -p "${OUT_DIR}"

if command -v wasm-bindgen >/dev/null 2>&1; then
  BINDGEN_BIN="wasm-bindgen"
else
  echo "wasm-bindgen not found in PATH, fetching standalone binary..."
  curl -sSL https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.129/wasm-bindgen-0.2.129-x86_64-unknown-linux-musl.tar.gz | tar -xz
  BINDGEN_BIN="${ROOT_DIR}/wasm-bindgen-0.2.129-x86_64-unknown-linux-musl/wasm-bindgen"
fi

"${BINDGEN_BIN}" "${WASM_TARGET}" \
  --out-dir "${OUT_DIR}" \
  --target web \
  --no-typescript

if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt --all-features -Oz --strip-debug --strip-producers "${OUT_DIR}/phonon_gui_bg.wasm" -o "${OUT_DIR}/phonon_gui_bg.wasm" 2>/dev/null || true
fi

echo "=== [3/4] Building Web CAD Studio & VitePress Docs ==="
cd "${ROOT_DIR}/web/studio"
if [ -f package-lock.json ]; then npm ci; else npm install; fi
npm run build

cd "${ROOT_DIR}/web/docs"
if [ -f package-lock.json ]; then npm ci; else npm install; fi
npm run build

echo "=== [4/4] Assembling Static Site Distribution ==="
cd "${ROOT_DIR}"
mkdir -p web/public_dist
rm -rf web/public_dist/*
cp -r web/docs/.vitepress/dist/* web/public_dist/
mkdir -p web/public_dist/studio
cp -r web/studio/dist/* web/public_dist/studio/
touch web/public_dist/.nojekyll

if [ -f web/studio/public/_headers ]; then
  cp -f web/studio/public/_headers web/public_dist/_headers
fi

echo "Build complete! Static site ready in web/public_dist"
