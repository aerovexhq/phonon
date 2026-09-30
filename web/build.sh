#!/usr/bin/env bash
# ==============================================================================
# Phonon Static Web Dual-Deployment Build Pipeline
# Combines VitePress documentation (/) and Vite Web CAD Studio (/studio/)
# Writes CNAME for phonon.aerovex.net
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${SCRIPT_DIR}/public_dist"

echo "[1/5] Building Web CAD Studio (/studio)..."
cd "${SCRIPT_DIR}/studio"
if [[ ! -d "node_modules" ]]; then
    npm install
fi
npm run build

echo "[2/5] Building VitePress Documentation Portal (/)..."
cd "${SCRIPT_DIR}/docs"
if [[ ! -d "node_modules" ]]; then
    npm install
fi
npm run build

echo "[3/5] Assembling unified static deployment distribution..."
rm -rf "${OUTPUT_DIR}"
mkdir -p "${OUTPUT_DIR}"

# Copy VitePress static site to root
cp -r "${SCRIPT_DIR}/docs/.vitepress/dist/"* "${OUTPUT_DIR}/"

# Copy Web CAD Studio into /studio subpath
mkdir -p "${OUTPUT_DIR}/studio"
cp -r "${SCRIPT_DIR}/studio/dist/"* "${OUTPUT_DIR}/studio/"

# Copy packaging assets (install.sh, deb, tarball) so curl install works directly from domain
if [[ -d "${ROOT_DIR}/dist" ]]; then
    echo "[4/5] Copying release installer and package assets to web root..."
    cp -f "${ROOT_DIR}/dist/install.sh" "${OUTPUT_DIR}/install.sh" 2>/dev/null || true
    cp -f "${ROOT_DIR}/dist/SHA256SUMS" "${OUTPUT_DIR}/SHA256SUMS" 2>/dev/null || true
fi

echo "[5/5] Configuring CNAME and GitHub Pages markers..."
echo "phonon.aerovex.net" > "${OUTPUT_DIR}/CNAME"
touch "${OUTPUT_DIR}/.nojekyll"

echo "Build complete! Static site generated at ${OUTPUT_DIR}"
ls -la "${OUTPUT_DIR}"
