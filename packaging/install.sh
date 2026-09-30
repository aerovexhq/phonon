#!/usr/bin/env bash
# ==============================================================================
# Phonon Universal Installer
# Phonon Universal Multi-Scale Visual CAD Studio & Semiconductor Solver
# https://phonon.aerovex.net
# ==============================================================================

set -euo pipefail

VERSION="0.1.0"
REPO="aerovexsim/phonon"
BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"
PAGES_URL="https://phonon.aerovex.net"

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m' # No Color

log_info() {
    printf "${BLUE}[INFO]${NC} %s\n" "$1"
}

log_success() {
    printf "${GREEN}[SUCCESS]${NC} %s\n" "$1"
}

log_warn() {
    printf "${YELLOW}[WARN]${NC} %s\n" "$1"
}

log_error() {
    printf "${RED}[ERROR]${NC} %s\n" "$1" >&2
}

print_banner() {
    printf "${CYAN}${BOLD}"
    cat << "EOF"
  ____  _   _  ___  _   _  ___  _   _ 
 |  _ \| | | |/ _ \| \ | |/ _ \| \ | |
 | |_) | |_| | | | |  \| | | | |  \| |
 |  __/|  _  | |_| | |\  | |_| | |\  |
 |_|   |_| |_|\___/|_| \_|\___/|_| \_|
EOF
    printf "${NC}"
    printf "${BOLD}Phonon Universal Multi-Scale Visual CAD Studio & Semiconductor Solver${NC}\n"
    printf "Version: %s | Release: https://github.com/%s\n" "$VERSION" "$REPO"
    printf "Documentation: https://phonon.aerovex.net\n\n"
}

show_help() {
    print_banner
    cat << EOF
Usage: install.sh [OPTIONS]

Options:
  -h, --help            Show this help message and exit
  -v, --version         Show installer version
  --deb                 Force installation via Debian (.deb) package
  --tarball             Force installation via universal tarball
  --prefix <PATH>       Install directory prefix (default: /usr/local or ~/.local)
  --rootless            Force user-local non-root installation (~/.local)
  --no-verify           Skip SHA-256 cryptographic verification

Examples:
  curl -fsSL https://phonon.aerovex.net/install.sh | bash
  curl -fsSL https://phonon.aerovex.net/install.sh | bash -s -- --rootless
  curl -fsSL https://phonon.aerovex.net/install.sh | bash -s -- --prefix /opt/phonon
EOF
}

# Default options
FORCE_DEB=false
FORCE_TARBALL=false
PREFIX=""
ROOTLESS=false
NO_VERIFY=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--version)
            echo "Phonon Installer v${VERSION}"
            exit 0
            ;;
        --deb)
            FORCE_DEB=true
            shift
            ;;
        --tarball)
            FORCE_TARBALL=true
            shift
            ;;
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --rootless)
            ROOTLESS=true
            shift
            ;;
        --no-verify)
            NO_VERIFY=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

print_banner

# OS and Architecture Detection
OS="$(uname -s)"
ARCH="$(uname -m)"

if [[ "$OS" != "Linux" ]]; then
    log_error "Phonon currently officially supports Linux x86_64. Detected OS: $OS"
    exit 1
fi

if [[ "$ARCH" != "x86_64" && "$ARCH" != "amd64" ]]; then
    log_error "Phonon release binaries are compiled for x86_64 / amd64. Detected architecture: $ARCH"
    exit 1
fi

# Detect Linux Distribution
DISTRO_ID="unknown"
DISTRO_LIKE=""
if [[ -f /etc/os-release ]]; then
    # shellcheck disable=SC1091
    source /etc/os-release
    DISTRO_ID="${ID:-unknown}"
    DISTRO_LIKE="${ID_LIKE:-}"
fi

log_info "Detected Platform: Linux ${ARCH} (${DISTRO_ID})"

# Determine installation mode and prefix
IS_ROOT=false
if [[ $EUID -eq 0 ]]; then
    IS_ROOT=true
fi

if [[ "$ROOTLESS" == true ]] || [[ "$IS_ROOT" == false && -z "$PREFIX" && "$FORCE_DEB" == false && ! -w /usr/local/bin ]]; then
    INSTALL_MODE="user"
    DEFAULT_PREFIX="${HOME}/.local"
else
    INSTALL_MODE="system"
    DEFAULT_PREFIX="/usr/local"
fi

TARGET_PREFIX="${PREFIX:-$DEFAULT_PREFIX}"
BIN_DIR="${TARGET_PREFIX}/bin"
SHARE_DIR="${TARGET_PREFIX}/share"
APPS_DIR="${SHARE_DIR}/applications"
ICONS_DIR="${SHARE_DIR}/icons/hicolor/scalable/apps"
BASH_COMP_DIR="${SHARE_DIR}/bash-completion/completions"
ZSH_COMP_DIR="${SHARE_DIR}/zsh/site-functions"

TMP_DIR="$(mktemp -d /tmp/phonon-install.XXXXXX)"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Download tool selection
download_file() {
    local url="$1"
    local dest="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --retry 3 --retry-delay 2 "$url" -o "$dest"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$dest" "$url"
    else
        log_error "Neither curl nor wget was found on your system. Please install curl or wget."
        exit 1
    fi
}

# Determine if we should install via Debian package
USE_DEB=false
if [[ "$FORCE_DEB" == true ]]; then
    USE_DEB=true
elif [[ "$FORCE_TARBALL" == false && "$INSTALL_MODE" == "system" ]]; then
    if command -v dpkg >/dev/null 2>&1; then
        case "$DISTRO_ID" in
            ubuntu|debian|linuxmint|pop|elementary|zorin|kali)
                USE_DEB=true
                ;;
            *)
                if [[ "$DISTRO_LIKE" == *"debian"* || "$DISTRO_LIKE" == *"ubuntu"* ]]; then
                    USE_DEB=true
                fi
                ;;
        esac
    fi
fi

if [[ "$USE_DEB" == true ]]; then
    DEB_NAME="phonon_${VERSION}_amd64.deb"
    DEB_URL="${BASE_URL}/${DEB_NAME}"
    log_info "Installing Debian package: ${DEB_NAME}"
    log_info "Fetching package from: ${DEB_URL}"

    download_file "$DEB_URL" "${TMP_DIR}/${DEB_NAME}"

    if [[ "$NO_VERIFY" == false ]]; then
        log_info "Verifying SHA-256 checksum..."
        if download_file "${BASE_URL}/SHA256SUMS" "${TMP_DIR}/SHA256SUMS" 2>/dev/null; then
            EXPECTED_HASH="$(grep "${DEB_NAME}" "${TMP_DIR}/SHA256SUMS" | awk '{print $1}')"
            if [[ -n "$EXPECTED_HASH" ]]; then
                ACTUAL_HASH="$(sha256sum "${TMP_DIR}/${DEB_NAME}" | awk '{print $1}')"
                if [[ "$EXPECTED_HASH" != "$ACTUAL_HASH" ]]; then
                    log_error "SHA-256 verification failed for ${DEB_NAME}!"
                    log_error "Expected: ${EXPECTED_HASH}"
                    log_error "Actual:   ${ACTUAL_HASH}"
                    exit 1
                fi
                log_success "SHA-256 verification passed: ${ACTUAL_HASH}"
            fi
        fi
    fi

    log_info "Executing dpkg installation..."
    if [[ "$IS_ROOT" == true ]]; then
        dpkg -i "${TMP_DIR}/${DEB_NAME}"
    elif command -v sudo >/dev/null 2>&1; then
        sudo dpkg -i "${TMP_DIR}/${DEB_NAME}"
    else
        log_warn "dpkg requires superuser privileges, but sudo is unavailable. Falling back to user-space tarball installation."
        USE_DEB=false
    fi
fi

if [[ "$USE_DEB" == false ]]; then
    TAR_NAME="phonon-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz"
    TAR_URL="${BASE_URL}/${TAR_NAME}"
    log_info "Installing universal multi-distro bundle to: ${TARGET_PREFIX}"
    log_info "Fetching archive from: ${TAR_URL}"

    download_file "$TAR_URL" "${TMP_DIR}/${TAR_NAME}"

    if [[ "$NO_VERIFY" == false ]]; then
        log_info "Verifying SHA-256 checksum..."
        if download_file "${BASE_URL}/SHA256SUMS" "${TMP_DIR}/SHA256SUMS" 2>/dev/null; then
            EXPECTED_HASH="$(grep "${TAR_NAME}" "${TMP_DIR}/SHA256SUMS" | awk '{print $1}')"
            if [[ -n "$EXPECTED_HASH" ]]; then
                ACTUAL_HASH="$(sha256sum "${TMP_DIR}/${TAR_NAME}" | awk '{print $1}')"
                if [[ "$EXPECTED_HASH" != "$ACTUAL_HASH" ]]; then
                    log_error "SHA-256 verification failed for ${TAR_NAME}!"
                    log_error "Expected: ${EXPECTED_HASH}"
                    log_error "Actual:   ${ACTUAL_HASH}"
                    exit 1
                fi
                log_success "SHA-256 verification passed: ${ACTUAL_HASH}"
            fi
        fi
    fi

    log_info "Extracting archive..."
    tar -xzf "${TMP_DIR}/${TAR_NAME}" -C "${TMP_DIR}"

    # Create target directories
    mkdir -p "$BIN_DIR" "$APPS_DIR" "$ICONS_DIR" "$BASH_COMP_DIR" "$ZSH_COMP_DIR"

    # Install files
    cp -f "${TMP_DIR}/bin/phonon" "${BIN_DIR}/phonon"
    chmod 755 "${BIN_DIR}/phonon"

    if [[ -f "${TMP_DIR}/share/applications/phonon.desktop" ]]; then
        cp -f "${TMP_DIR}/share/applications/phonon.desktop" "${APPS_DIR}/phonon.desktop"
        chmod 644 "${APPS_DIR}/phonon.desktop"
    fi

    if [[ -f "${TMP_DIR}/share/icons/hicolor/scalable/apps/phonon.svg" ]]; then
        cp -f "${TMP_DIR}/share/icons/hicolor/scalable/apps/phonon.svg" "${ICONS_DIR}/phonon.svg"
        chmod 644 "${ICONS_DIR}/phonon.svg"
    fi

    if [[ -f "${TMP_DIR}/share/completions/phonon.bash" ]]; then
        cp -f "${TMP_DIR}/share/completions/phonon.bash" "${BASH_COMP_DIR}/phonon"
        chmod 644 "${BASH_COMP_DIR}/phonon"
    fi

    if [[ -f "${TMP_DIR}/share/completions/_phonon.zsh" ]]; then
        cp -f "${TMP_DIR}/share/completions/_phonon.zsh" "${ZSH_COMP_DIR}/_phonon"
        chmod 644 "${ZSH_COMP_DIR}/_phonon"
    fi

    # Update desktop database if available
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$APPS_DIR" 2>/dev/null || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t "${SHARE_DIR}/icons/hicolor" 2>/dev/null || true
    fi
fi

# Verify executable
if command -v phonon >/dev/null 2>&1; then
    PHONON_EXEC="phonon"
elif [[ -x "${BIN_DIR}/phonon" ]]; then
    PHONON_EXEC="${BIN_DIR}/phonon"
else
    PHONON_EXEC="/usr/bin/phonon"
fi

printf "\n"
log_success "Phonon v${VERSION} has been successfully installed!"
printf "\n"
printf "${BOLD}Quick Start Guide:${NC}\n"
printf "  1. Launch Desktop CAD Studio:       ${CYAN}%s ui${NC}\n" "$PHONON_EXEC"
printf "  2. Display CLI Commands:            ${CYAN}%s --help${NC}\n" "$PHONON_EXEC"
printf "  3. Validate Circuit Netlist:        ${CYAN}%s validate &lt;netlist.cir&gt;${NC}\n" "$PHONON_EXEC"
printf "  4. Run Transient Simulation:        ${CYAN}%s run &lt;netlist.cir&gt;${NC}\n" "$PHONON_EXEC"
printf "  5. Web CAD Studio (Browser):        ${CYAN}https://phonon.aerovex.net/studio${NC}\n"
printf "  6. Documentation & Architecture:    ${CYAN}https://phonon.aerovex.net${NC}\n"
printf "\n"

if [[ "$INSTALL_MODE" == "user" && ":$PATH:" != *":$BIN_DIR:"* ]]; then
    log_warn "Notice: ${BIN_DIR} is not currently in your system PATH."
    printf "  Add it to your shell configuration by executing:\n"
    printf "    ${CYAN}echo 'export PATH=\"%s:\$PATH\"' >> ~/.bashrc${NC}\n" "$BIN_DIR"
    printf "    ${CYAN}source ~/.bashrc${NC}\n\n"
fi
