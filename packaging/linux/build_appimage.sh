#!/usr/bin/env bash
# ==============================================================================
# Phonon Universal Standalone AppImage Builder
# Assembles portable AppDir and generates "Phonon-<version>-x86_64.AppImage"
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
VERSION="0.1.0"
APP_DIR="${DIST_DIR}/AppDir"
OUTPUT_APPIMAGE="${DIST_DIR}/Phonon-${VERSION}-x86_64.AppImage"

echo "[1/4] Preparing AppDir filesystem structure..."
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/usr/bin" \
         "${APP_DIR}/usr/lib" \
         "${APP_DIR}/usr/share/applications" \
         "${APP_DIR}/usr/share/icons/hicolor/scalable/apps"

RELEASE_BIN="${DIST_DIR}/phonon-x86_64"
if [[ ! -f "$RELEASE_BIN" ]]; then
    RELEASE_BIN="${ROOT_DIR}/target/release/phonon"
fi

if [[ ! -f "$RELEASE_BIN" ]]; then
    echo "Phonon release binary not found at $RELEASE_BIN. Compiling..."
    cargo build --release -p phonon-cli --manifest-path "${ROOT_DIR}/Cargo.toml"
    RELEASE_BIN="${ROOT_DIR}/target/release/phonon"
fi

echo "[2/4] Copying binaries, desktop entries, and icons..."
cp -f "$RELEASE_BIN" "${APP_DIR}/usr/bin/phonon"
chmod 755 "${APP_DIR}/usr/bin/phonon"

cp -f "${ROOT_DIR}/packaging/phonon.desktop" "${APP_DIR}/usr/share/applications/phonon.desktop"
cp -f "${ROOT_DIR}/packaging/phonon.desktop" "${APP_DIR}/phonon.desktop"

cp -f "${ROOT_DIR}/packaging/phonon.svg" "${APP_DIR}/usr/share/icons/hicolor/scalable/apps/phonon.svg"
cp -f "${ROOT_DIR}/packaging/phonon.svg" "${APP_DIR}/phonon.svg"

echo "[3/4] Creating AppRun runtime entrypoint..."
cat <<'EOF' > "${APP_DIR}/AppRun"
#!/usr/bin/env bash
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH:-}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"

# Dual-mode execution: launch Desktop UI if no arguments provided,
# otherwise pass arguments directly to CLI engine.
if [[ $# -eq 0 ]]; then
    exec "${HERE}/usr/bin/phonon" ui
else
    exec "${HERE}/usr/bin/phonon" "$@"
fi
EOF
chmod 755 "${APP_DIR}/AppRun"

echo "[4/4] Building standalone AppImage..."
mkdir -p "${DIST_DIR}"

if command -v appimagetool >/dev/null 2>&1; then
    echo "Found appimagetool. Compiling squashfs AppImage..."
    ARCH=x86_64 appimagetool --appimage-extract-and-run "${APP_DIR}" "${OUTPUT_APPIMAGE}" 2>/dev/null || \
    ARCH=x86_64 appimagetool "${APP_DIR}" "${OUTPUT_APPIMAGE}"
    chmod +x "${OUTPUT_APPIMAGE}" 2>/dev/null || true
    echo "AppImage created successfully at: ${OUTPUT_APPIMAGE}"
else
    echo "appimagetool not found in PATH."
    echo "AppDir is fully assembled at: ${APP_DIR}"
    echo "To package into AppImage, run: ARCH=x86_64 appimagetool ${APP_DIR} ${OUTPUT_APPIMAGE}"
fi
