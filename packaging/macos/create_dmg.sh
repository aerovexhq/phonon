#!/usr/bin/env bash
# ==============================================================================
# Phonon macOS DMG Disk Image Generator
# Creates styled drag-and-drop disk image (.dmg) for Apple Silicon and Intel Macs
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
VERSION="${1:-${PHONON_VERSION:-0.1.0}}"
DMG_NAME="Phonon-${VERSION}.dmg"
DMG_PATH="${DIST_DIR}/${DMG_NAME}"
APP_BUNDLE="${DIST_DIR}/macos/Phonon Studio.app"

if [[ ! -d "${APP_BUNDLE}" ]]; then
    echo "Application bundle not found at ${APP_BUNDLE}. Building first..."
    bash "${SCRIPT_DIR}/build_app_bundle.sh"
fi

echo "[1/3] Preparing disk image staging area..."
STAGING_DIR="$(mktemp -d /tmp/phonon-dmg.XXXXXX)"
cp -R "${APP_BUNDLE}" "${STAGING_DIR}/"
ln -s /Applications "${STAGING_DIR}/Applications"

echo "[2/3] Building disk image..."
mkdir -p "${DIST_DIR}"
rm -f "${DMG_PATH}"

if command -v create-dmg >/dev/null 2>&1; then
    echo "Using create-dmg for styled presentation..."
    create-dmg \
        --volname "Phonon Studio" \
        --window-pos 200 120 \
        --window-size 660 400 \
        --icon-size 128 \
        --icon "Phonon Studio.app" 180 190 \
        --hide-extension "Phonon Studio.app" \
        --app-drop-link 480 190 \
        --no-internet-enable \
        "${DMG_PATH}" \
        "${STAGING_DIR}" || {
            echo "create-dmg exited with non-zero code, falling back to hdiutil..."
            hdiutil create -volname "Phonon Studio" \
                           -srcfolder "${STAGING_DIR}" \
                           -ov -format UDZO \
                           "${DMG_PATH}"
        }
elif command -v hdiutil >/dev/null 2>&1; then
    echo "Using native macOS hdiutil..."
    hdiutil create -volname "Phonon Studio" \
                   -srcfolder "${STAGING_DIR}" \
                   -ov -format UDZO \
                   "${DMG_PATH}"
else
    echo "Neither create-dmg nor hdiutil available (non-macOS environment)."
    echo "Packaging bundle into compressed archive for macOS distribution..."
    tar -czf "${DIST_DIR}/Phonon-v${VERSION}-macos.tar.gz" -C "${DIST_DIR}/macos" "Phonon Studio.app"
    rm -rf "${STAGING_DIR}"
    exit 0
fi

echo "[3/3] Cleaning up staging area..."
rm -rf "${STAGING_DIR}"

echo "DMG disk image successfully generated at: ${DMG_PATH}"
