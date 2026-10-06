#!/usr/bin/env bash
# ==============================================================================
# Phonon Studio WebAssembly Production Optimization & Cache Invalidation Pipeline
#
# 1. Compiles with LLVM Fat LTO, single codegen unit, and dead code elimination.
# 2. Generates WebAssembly JS bindings via wasm-bindgen.
# 3. Optimizes with wasm-opt -Oz (Binaryen) stripping debug info and producers.
# 4. Computes SHA-256 checksums and git commit hash for cache-busting headers.
# 5. Generates static deployment _headers file for instant cache invalidation.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${ROOT_DIR}/web/studio/public/wasm"

mkdir -p "${OUT_DIR}"

echo "======================================================================"
echo "[1/5] Compiling phonon-gui WebAssembly release with Fat LTO..."
echo "======================================================================"
cd "${ROOT_DIR}"
cargo build --release -p phonon-gui --target wasm32-unknown-unknown

WASM_TARGET="${ROOT_DIR}/target/wasm32-unknown-unknown/release/phonon_gui.wasm"
if [[ ! -f "${WASM_TARGET}" ]]; then
    echo "ERROR: Target WASM binary not found at ${WASM_TARGET}"
    exit 1
fi

RAW_SIZE=$(wc -c < "${WASM_TARGET}")
echo "Raw compiled WASM binary: ${RAW_SIZE} bytes ($(( RAW_SIZE / 1024 )) KB)"

echo "======================================================================"
echo "[2/5] Generating web bindings with wasm-bindgen..."
echo "======================================================================"
wasm-bindgen "${WASM_TARGET}" \
    --out-dir "${OUT_DIR}" \
    --target web \
    --no-typescript

BOUND_SIZE=$(wc -c < "${OUT_DIR}/phonon_gui_bg.wasm")
echo "Bound WASM binary: ${BOUND_SIZE} bytes ($(( BOUND_SIZE / 1024 )) KB)"

echo "======================================================================"
echo "[3/5] Optimizing binary size with wasm-opt -Oz..."
echo "======================================================================"
if command -v wasm-opt >/dev/null 2>&1; then
    WASM_OPT_CMD="wasm-opt"
else
    WASM_OPT_CMD="npx wasm-opt"
fi

${WASM_OPT_CMD} --all-features -Oz \
    --strip-debug \
    --strip-producers \
    --vacuum \
    "${OUT_DIR}/phonon_gui_bg.wasm" \
    -o "${OUT_DIR}/phonon_gui_bg.wasm"

OPT_SIZE=$(wc -c < "${OUT_DIR}/phonon_gui_bg.wasm")
echo "Optimized WASM binary: ${OPT_SIZE} bytes ($(( OPT_SIZE / 1024 )) KB)"
REDUCTION_PCT=$(awk -v r="${RAW_SIZE}" -v o="${OPT_SIZE}" 'BEGIN { printf "%.1f", (1.0 - o / r) * 100 }')
echo "Binary size reduction: ${REDUCTION_PCT}%"

echo "======================================================================"
echo "[4/5] Computing SHA-256 hashes and cache-busting keys..."
echo "======================================================================"
COMMIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "latest")
WASM_SHA256=$(sha256sum "${OUT_DIR}/phonon_gui_bg.wasm" | awk '{print $1}')
JS_SHA256=$(sha256sum "${OUT_DIR}/phonon_gui.js" | awk '{print $1}')
BUILD_ISO=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat <<EOF > "${OUT_DIR}/build_meta.json"
{
  "commit_hash": "${COMMIT_HASH}",
  "build_timestamp_iso": "${BUILD_ISO}",
  "raw_size_bytes": ${RAW_SIZE},
  "optimized_size_bytes": ${OPT_SIZE},
  "reduction_percent": ${REDUCTION_PCT},
  "wasm_sha256": "${WASM_SHA256}",
  "js_sha256": "${JS_SHA256}",
  "cache_query_key": "${COMMIT_HASH}",
  "lto_mode": "fat",
  "opt_level": "-Oz"
}
EOF

echo "Build metadata written to ${OUT_DIR}/build_meta.json:"
cat "${OUT_DIR}/build_meta.json"

echo "======================================================================"
echo "[5/5] Generating static hosting cache-invalidation headers (_headers)..."
echo "======================================================================"
cat <<'EOF' > "${ROOT_DIR}/web/studio/public/_headers"
# Cloudflare Pages / Netlify Cache-Control Header Policy for Phonon Web Studio

# 1. HTML Documents: Zero caching to guarantee instant update detection
/*
  Cache-Control: public, max-age=0, must-revalidate

# 2. Immutable WebAssembly & JS Bundles: Cache permanently when hashed with ?v=
/studio/wasm/*
  Cache-Control: public, max-age=31536000, immutable
  Access-Control-Allow-Origin: *

# 3. Static Icons & Manifests
/studio/favicon.svg
  Cache-Control: public, max-age=86400
EOF

cp -f "${ROOT_DIR}/web/studio/public/_headers" "${ROOT_DIR}/web/public_dist/_headers" 2>/dev/null || true

echo "WebAssembly optimization & cache-invalidation pipeline complete!"
