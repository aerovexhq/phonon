#!/usr/bin/env bash
# ==============================================================================
# Phonon macOS Application Bundle Generator
# Assembles "Phonon Studio.app" bundle for macOS (ARM64 / x86_64)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
APP_NAME="Phonon Studio.app"
APP_DIR="${DIST_DIR}/macos/${APP_NAME}"

echo "[1/4] Preparing bundle layout..."
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS" \
         "${APP_DIR}/Contents/Resources"

RELEASE_BIN="${1:-${ROOT_DIR}/target/release/phonon}"
if [[ ! -f "$RELEASE_BIN" ]]; then
    echo "Release binary not found at $RELEASE_BIN. Please specify binary path or run cargo build --release."
    exit 1
fi

echo "[2/4] Installing executable and launcher..."
cp -f "$RELEASE_BIN" "${APP_DIR}/Contents/MacOS/phonon"
chmod 755 "${APP_DIR}/Contents/MacOS/phonon"

# Dual-mode launcher: launches GUI if opened without arguments (Finder / Dock),
# otherwise executes CLI arguments directly.
cat <<'EOF' > "${APP_DIR}/Contents/MacOS/phonon-launcher"
#!/usr/bin/env bash
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ $# -eq 0 ]; then
    exec "${DIR}/phonon" ui
else
    exec "${DIR}/phonon" "$@"
fi
EOF
chmod 755 "${APP_DIR}/Contents/MacOS/phonon-launcher"

echo "[3/4] Copying metadata and resources..."
cp -f "${SCRIPT_DIR}/Info.plist" "${APP_DIR}/Contents/Info.plist"
echo -n "APPL????" > "${APP_DIR}/Contents/PkgInfo"

# If icns exists, copy it, else copy svg/png placeholder
if [[ -f "${SCRIPT_DIR}/phonon.icns" ]]; then
    cp -f "${SCRIPT_DIR}/phonon.icns" "${APP_DIR}/Contents/Resources/phonon.icns"
elif [[ -f "${ROOT_DIR}/packaging/phonon.svg" ]]; then
    cp -f "${ROOT_DIR}/packaging/phonon.svg" "${APP_DIR}/Contents/Resources/phonon.svg"
fi

if [[ -d "${ROOT_DIR}/packaging/completions" ]]; then
    mkdir -p "${APP_DIR}/Contents/Resources/completions"
    cp -r "${ROOT_DIR}/packaging/completions/"* "${APP_DIR}/Contents/Resources/completions/"
fi

echo "[4/4] Verifying application bundle..."
echo "Bundle successfully assembled at: ${APP_DIR}"
ls -la "${APP_DIR}/Contents"
ls -la "${APP_DIR}/Contents/MacOS"
