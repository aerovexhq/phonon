#!/usr/bin/env bash
# ==============================================================================
# Phonon Release Packaging Script
# Builds .deb package, universal .tar.gz archive, standalone binary, and checksums
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
VERSION="0.1.0"

echo "[1/6] Preparing output directories..."
mkdir -p "${DIST_DIR}"

RELEASE_BIN="${ROOT_DIR}/target/release/phonon"
if [[ ! -f "$RELEASE_BIN" ]]; then
    echo "Release binary not found at $RELEASE_BIN. Compiling..."
    cargo build --release -p phonon-cli --manifest-path "${ROOT_DIR}/Cargo.toml"
fi

echo "[2/6] Generating standalone stripped binary..."
cp -f "$RELEASE_BIN" "${DIST_DIR}/phonon-x86_64"
chmod 755 "${DIST_DIR}/phonon-x86_64"
strip --strip-unneeded "${DIST_DIR}/phonon-x86_64" 2>/dev/null || true

echo "[3/6] Packaging Debian (.deb) package..."
DEB_STAGING="${SCRIPT_DIR}/debian"
mkdir -p "${DEB_STAGING}/DEBIAN" \
         "${DEB_STAGING}/usr/bin" \
         "${DEB_STAGING}/usr/share/applications" \
         "${DEB_STAGING}/usr/share/icons/hicolor/scalable/apps" \
         "${DEB_STAGING}/usr/share/bash-completion/completions" \
         "${DEB_STAGING}/usr/share/zsh/site-functions"

# Copy binary
cp -f "${DIST_DIR}/phonon-x86_64" "${DEB_STAGING}/usr/bin/phonon"
chmod 755 "${DEB_STAGING}/usr/bin/phonon"

# Copy desktop and icon
cp -f "${SCRIPT_DIR}/phonon.desktop" "${DEB_STAGING}/usr/share/applications/phonon.desktop"
chmod 644 "${DEB_STAGING}/usr/share/applications/phonon.desktop"

cp -f "${SCRIPT_DIR}/phonon.svg" "${DEB_STAGING}/usr/share/icons/hicolor/scalable/apps/phonon.svg"
chmod 644 "${DEB_STAGING}/usr/share/icons/hicolor/scalable/apps/phonon.svg"

# Copy completions
cp -f "${SCRIPT_DIR}/completions/phonon.bash" "${DEB_STAGING}/usr/share/bash-completion/completions/phonon"
chmod 644 "${DEB_STAGING}/usr/share/bash-completion/completions/phonon"

cp -f "${SCRIPT_DIR}/completions/_phonon.zsh" "${DEB_STAGING}/usr/share/zsh/site-functions/_phonon"
chmod 644 "${DEB_STAGING}/usr/share/zsh/site-functions/_phonon"

chmod 644 "${DEB_STAGING}/DEBIAN/control"

dpkg-deb --build --root-owner-group "${DEB_STAGING}" "${DIST_DIR}/phonon_${VERSION}_amd64.deb"

echo "[4/6] Packaging universal distribution tarball..."
TAR_STAGING="$(mktemp -d /tmp/phonon-tar.XXXXXX)"
mkdir -p "${TAR_STAGING}/bin" \
         "${TAR_STAGING}/share/applications" \
         "${TAR_STAGING}/share/icons/hicolor/scalable/apps" \
         "${TAR_STAGING}/share/completions"

cp -f "${DIST_DIR}/phonon-x86_64" "${TAR_STAGING}/bin/phonon"
cp -f "${SCRIPT_DIR}/phonon.desktop" "${TAR_STAGING}/share/applications/phonon.desktop"
cp -f "${SCRIPT_DIR}/phonon.svg" "${TAR_STAGING}/share/icons/hicolor/scalable/apps/phonon.svg"
cp -f "${SCRIPT_DIR}/completions/phonon.bash" "${TAR_STAGING}/share/completions/phonon.bash"
cp -f "${SCRIPT_DIR}/completions/_phonon.zsh" "${TAR_STAGING}/share/completions/_phonon.zsh"
cp -f "${SCRIPT_DIR}/install.sh" "${TAR_STAGING}/install.sh"
cp -f "${ROOT_DIR}/README.md" "${TAR_STAGING}/README.md"
cp -f "${ROOT_DIR}/LICENSE" "${TAR_STAGING}/LICENSE"
chmod +x "${TAR_STAGING}/install.sh"

tar -czf "${DIST_DIR}/phonon-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz" -C "${TAR_STAGING}" .
rm -rf "${TAR_STAGING}"

echo "[5/6] Copying installer script..."
cp -f "${SCRIPT_DIR}/install.sh" "${DIST_DIR}/install.sh"
chmod +x "${DIST_DIR}/install.sh"

echo "[6/6] Computing SHA256 checksums..."
cd "${DIST_DIR}"
sha256sum phonon_${VERSION}_amd64.deb \
          phonon-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz \
          phonon-x86_64 \
          install.sh > SHA256SUMS

echo "Packaging complete! Built assets in ${DIST_DIR}:"
ls -lh "${DIST_DIR}"
cat "${DIST_DIR}/SHA256SUMS"
