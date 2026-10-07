#!/usr/bin/env bash
# ==============================================================================
# Phonon Universal Installer
# Phonon Universal Multi-Scale Visual CAD Studio & Semiconductor Solver
# https://phonon.aerovex.net
# ==============================================================================

set -euo pipefail

VERSION="0.1.0"
REPO="aerovexhq/phonon"
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
Usage: install.sh [OPTIONS] [DEB_FILE_OR_PATH]

Options:
  -h, --help            Show this help message and exit
  -v, --version         Show installer version
  --deb                 Install via Debian (.deb) package
  --local-deb <PATH>    Install directly from a local .deb package without sudo
  --tarball             Install via universal tarball
  --prefix <PATH>       Install directory prefix (default: ~/.local or /usr/local)
  --rootless            Force user-local non-root installation (~/.local, no sudo)
  --system              Request system-wide installation (/usr/local, requires root/sudo)
  --uninstall           Uninstall Phonon from the target prefix
  --no-verify           Skip SHA-256 cryptographic verification

Examples:
  # User-space install without sudo (recommended):
  curl -fsSL https://phonon.aerovex.net/install.sh | bash

  # Install from local .deb package into ~/.local without sudo:
  ./install.sh phonon_0.1.0_amd64.deb
  ./install.sh --local-deb phonon_0.1.0_amd64.deb

  # Custom prefix:
  ./install.sh --prefix /opt/phonon

  # Uninstall user-space installation:
  ./install.sh --uninstall
EOF
}

# Default options
FORCE_DEB=false
FORCE_TARBALL=false
PREFIX=""
ROOTLESS=false
FORCE_SYSTEM=false
UNINSTALL=false
NO_VERIFY=false
LOCAL_DEB=""

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
        --local-deb)
            if [[ $# -lt 2 ]]; then
                log_error "--local-deb requires a file path argument."
                exit 1
            fi
            LOCAL_DEB="$2"
            FORCE_DEB=true
            shift 2
            ;;
        --tarball)
            FORCE_TARBALL=true
            shift
            ;;
        --prefix)
            if [[ $# -lt 2 ]]; then
                log_error "--prefix requires a path argument."
                exit 1
            fi
            PREFIX="$2"
            shift 2
            ;;
        --rootless)
            ROOTLESS=true
            shift
            ;;
        --system)
            FORCE_SYSTEM=true
            shift
            ;;
        --uninstall)
            UNINSTALL=true
            shift
            ;;
        --no-verify)
            NO_VERIFY=true
            shift
            ;;
        *.deb)
            LOCAL_DEB="$1"
            FORCE_DEB=true
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
    DISTRO_ID="$(grep -E '^ID=' /etc/os-release | head -n1 | cut -d= -f2 | tr -d '\"' || echo "unknown")"
    DISTRO_LIKE="$(grep -E '^ID_LIKE=' /etc/os-release | head -n1 | cut -d= -f2 | tr -d '\"' || echo "")"
fi

log_info "Detected Platform: Linux ${ARCH} (${DISTRO_ID})"

# Check root and sudo capabilities
IS_ROOT=false
if [[ $EUID -eq 0 ]]; then
    IS_ROOT=true
fi

can_sudo_passwordless() {
    command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null
}

# Determine installation mode and target prefix
# Phonon is an EDA CAD studio that runs purely in user space and does not require system-level privileges or daemons.
# Unless running as root ($EUID -eq 0) or explicitly requested with system credentials,
# default to user space (~/.local) so users are never forced to use sudo.
if [[ "$ROOTLESS" == true ]]; then
    INSTALL_MODE="user"
    DEFAULT_PREFIX="${HOME}/.local"
elif [[ "$IS_ROOT" == true ]]; then
    INSTALL_MODE="system"
    DEFAULT_PREFIX="/usr/local"
elif [[ "$FORCE_SYSTEM" == true ]]; then
    if can_sudo_passwordless; then
        INSTALL_MODE="system"
        DEFAULT_PREFIX="/usr/local"
    else
        log_warn "System install requested, but sudo requires a password or root privileges are unavailable."
        log_info "Phonon does not require sudo. Falling back to user space (~/.local)..."
        INSTALL_MODE="user"
        DEFAULT_PREFIX="${HOME}/.local"
    fi
elif [[ -n "$PREFIX" ]]; then
    TARGET_PREFIX="$PREFIX"
    if [[ -w "$TARGET_PREFIX" || (! -e "$TARGET_PREFIX" && -w "$(dirname "$TARGET_PREFIX")") ]]; then
        INSTALL_MODE="user"
        DEFAULT_PREFIX="$TARGET_PREFIX"
    elif can_sudo_passwordless; then
        INSTALL_MODE="system"
        DEFAULT_PREFIX="$TARGET_PREFIX"
    else
        log_warn "Target prefix '${PREFIX}' is not writable and sudo requires a password."
        log_info "Falling back to user space (~/.local)..."
        INSTALL_MODE="user"
        DEFAULT_PREFIX="${HOME}/.local"
    fi
else
    # Default for normal users: User space install without sudo
    INSTALL_MODE="user"
    DEFAULT_PREFIX="${HOME}/.local"
fi

TARGET_PREFIX="${PREFIX:-$DEFAULT_PREFIX}"
BIN_DIR="${TARGET_PREFIX}/bin"
SHARE_DIR="${TARGET_PREFIX}/share"
APPS_DIR="${SHARE_DIR}/applications"
ICONS_DIR="${SHARE_DIR}/icons/hicolor/scalable/apps"
BASH_COMP_DIR="${SHARE_DIR}/bash-completion/completions"
ZSH_COMP_DIR="${SHARE_DIR}/zsh/site-functions"

# Handle --uninstall
if [[ "$UNINSTALL" == true ]]; then
    log_info "Uninstalling Phonon from ${TARGET_PREFIX}..."
    rm -f "${BIN_DIR}/phonon"
    rm -f "${APPS_DIR}/phonon.desktop"
    rm -f "${ICONS_DIR}/phonon.svg"
    rm -f "${BASH_COMP_DIR}/phonon"
    rm -f "${ZSH_COMP_DIR}/_phonon"

    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$APPS_DIR" 2>/dev/null || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t "${SHARE_DIR}/icons/hicolor" 2>/dev/null || true
    fi

    log_success "Phonon has been uninstalled from ${TARGET_PREFIX}."
    exit 0
fi

log_info "Installation Mode: ${INSTALL_MODE} (Prefix: ${TARGET_PREFIX})"

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

# Rootless Debian package extractor
extract_deb_rootless() {
    local deb_pkg="$1"
    local dest_dir="$2"
    mkdir -p "$dest_dir"

    if command -v dpkg-deb >/dev/null 2>&1; then
        dpkg-deb -x "$deb_pkg" "$dest_dir"
    elif command -v ar >/dev/null 2>&1 && command -v tar >/dev/null 2>&1; then
        local ar_tmp
        ar_tmp="$(mktemp -d /tmp/phonon-ar.XXXXXX)"
        (cd "$ar_tmp" && ar -x "$deb_pkg")
        if [[ -f "${ar_tmp}/data.tar.xz" ]]; then
            tar -xf "${ar_tmp}/data.tar.xz" -C "$dest_dir"
        elif [[ -f "${ar_tmp}/data.tar.gz" ]]; then
            tar -xf "${ar_tmp}/data.tar.gz" -C "$dest_dir"
        elif [[ -f "${ar_tmp}/data.tar.zst" ]]; then
            tar --zstd -xf "${ar_tmp}/data.tar.zst" -C "$dest_dir"
        else
            rm -rf "$ar_tmp"
            log_error "Could not find recognizable data archive inside ${deb_pkg}"
            return 1
        fi
        rm -rf "$ar_tmp"
    else
        log_error "Neither dpkg-deb nor ar+tar is available to extract ${deb_pkg}"
        return 1
    fi
}

# Determine package type to install
USE_DEB=false
if [[ -n "$LOCAL_DEB" || "$FORCE_DEB" == true ]]; then
    USE_DEB=true
elif [[ "$FORCE_TARBALL" == false ]]; then
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

if [[ "$USE_DEB" == true ]]; then
    DEB_NAME="phonon_${VERSION}_amd64.deb"
    DEB_FILE=""

    if [[ -n "$LOCAL_DEB" ]]; then
        if [[ ! -f "$LOCAL_DEB" ]]; then
            log_error "Local Debian package not found at: ${LOCAL_DEB}"
            exit 1
        fi
        DEB_FILE="$(realpath "$LOCAL_DEB")"
        log_info "Using local Debian package: ${DEB_FILE}"
    else
        DEB_URL="${BASE_URL}/${DEB_NAME}"
        DEB_FILE="${TMP_DIR}/${DEB_NAME}"
        log_info "Downloading Debian package: ${DEB_NAME}"
        log_info "Fetching package from: ${DEB_URL}"
        download_file "$DEB_URL" "$DEB_FILE"

        if [[ "$NO_VERIFY" == false ]]; then
            log_info "Verifying SHA-256 checksum..."
            if download_file "${BASE_URL}/SHA256SUMS" "${TMP_DIR}/SHA256SUMS" 2>/dev/null; then
                EXPECTED_HASH="$(grep "${DEB_NAME}" "${TMP_DIR}/SHA256SUMS" | awk '{print $1}')"
                if [[ -n "$EXPECTED_HASH" ]]; then
                    ACTUAL_HASH="$(sha256sum "$DEB_FILE" | awk '{print $1}')"
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
    fi

    # Install or extract debian package
    if [[ "$INSTALL_MODE" == "system" && "$IS_ROOT" == true ]]; then
        log_info "Executing dpkg installation as root..."
        dpkg -i "$DEB_FILE"
    elif [[ "$INSTALL_MODE" == "system" && "$(can_sudo_passwordless; echo $?)" == "0" ]]; then
        log_info "Executing sudo dpkg installation..."
        sudo -n dpkg -i "$DEB_FILE"
    else
        # Rootless user-space installation
        log_info "Extracting Debian package rootlessly into: ${TARGET_PREFIX}"
        EXTRACT_DIR="${TMP_DIR}/deb_extracted"
        extract_deb_rootless "$DEB_FILE" "$EXTRACT_DIR"

        mkdir -p "$BIN_DIR" "$APPS_DIR" "$ICONS_DIR" "$BASH_COMP_DIR" "$ZSH_COMP_DIR"

        # Relocate binary
        if [[ -f "${EXTRACT_DIR}/usr/bin/phonon" ]]; then
            cp -f "${EXTRACT_DIR}/usr/bin/phonon" "${BIN_DIR}/phonon"
        elif [[ -f "${EXTRACT_DIR}/bin/phonon" ]]; then
            cp -f "${EXTRACT_DIR}/bin/phonon" "${BIN_DIR}/phonon"
        else
            log_error "Executable 'phonon' not found inside package!"
            exit 1
        fi
        chmod 755 "${BIN_DIR}/phonon"

        # Relocate desktop entry
        if [[ -f "${EXTRACT_DIR}/usr/share/applications/phonon.desktop" ]]; then
            cp -f "${EXTRACT_DIR}/usr/share/applications/phonon.desktop" "${APPS_DIR}/phonon.desktop"
            chmod 644 "${APPS_DIR}/phonon.desktop"
        fi

        # Relocate application icon
        if [[ -f "${EXTRACT_DIR}/usr/share/icons/hicolor/scalable/apps/phonon.svg" ]]; then
            cp -f "${EXTRACT_DIR}/usr/share/icons/hicolor/scalable/apps/phonon.svg" "${ICONS_DIR}/phonon.svg"
            chmod 644 "${ICONS_DIR}/phonon.svg"
        fi

        # Relocate shell autocompletions
        if [[ -f "${EXTRACT_DIR}/usr/share/bash-completion/completions/phonon" ]]; then
            cp -f "${EXTRACT_DIR}/usr/share/bash-completion/completions/phonon" "${BASH_COMP_DIR}/phonon"
            chmod 644 "${BASH_COMP_DIR}/phonon"
        fi
        if [[ -f "${EXTRACT_DIR}/usr/share/zsh/site-functions/_phonon" ]]; then
            cp -f "${EXTRACT_DIR}/usr/share/zsh/site-functions/_phonon" "${ZSH_COMP_DIR}/_phonon"
            chmod 644 "${ZSH_COMP_DIR}/_phonon"
        fi

        # Update desktop database and icon caches if tools exist
        if command -v update-desktop-database >/dev/null 2>&1; then
            update-desktop-database "$APPS_DIR" 2>/dev/null || true
        fi
        if command -v gtk-update-icon-cache >/dev/null 2>&1; then
            gtk-update-icon-cache -f -t "${SHARE_DIR}/icons/hicolor" 2>/dev/null || true
        fi
    fi
else
    # Tarball installation
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

# Locate installed executable
if [[ -x "${BIN_DIR}/phonon" ]]; then
    PHONON_EXEC="${BIN_DIR}/phonon"
elif command -v phonon >/dev/null 2>&1; then
    PHONON_EXEC="$(command -v phonon)"
else
    PHONON_EXEC="${BIN_DIR}/phonon"
fi

printf "\n"
log_success "Phonon v${VERSION} has been successfully installed to ${PHONON_EXEC}!"
printf "\n"

# Verify active executable in PATH
if command -v phonon >/dev/null 2>&1; then
    RESOLVED_PATH="$(command -v phonon)"
    if [[ "$RESOLVED_PATH" == "${BIN_DIR}/phonon" ]]; then
        log_info "Active command: ${RESOLVED_PATH} (verified in PATH)"
    else
        log_warn "Active command in PATH resolves to: ${RESOLVED_PATH}"
        log_warn "Newly installed binary is located at: ${BIN_DIR}/phonon"
        log_info "Please ensure ${BIN_DIR} precedes other entries in your PATH."
    fi
else
    log_warn "Notice: ${BIN_DIR} is not currently in your system PATH."
    printf "  Add it to your shell configuration by executing:\n"
    printf "    ${CYAN}echo 'export PATH=\"%s:\$PATH\"' >> ~/.bashrc${NC}\n" "$BIN_DIR"
    printf "    ${CYAN}source ~/.bashrc${NC}\n\n"
fi

if [[ -f "/usr/bin/phonon" && "${BIN_DIR}/phonon" != "/usr/bin/phonon" ]]; then
    log_info "Note: A system binary is present at /usr/bin/phonon."
    if [[ "$(command -v phonon 2>/dev/null)" == "${BIN_DIR}/phonon" ]]; then
        log_info "Your user-space binary (${BIN_DIR}/phonon) takes precedence in PATH."
    fi
fi

printf "\n"
printf "${BOLD}Quick Start Guide:${NC}\n"
printf "  1. Launch Desktop CAD Studio:       ${CYAN}%s gui${NC}\n" "$PHONON_EXEC"
printf "  2. Display CLI Commands:            ${CYAN}%s --help${NC}\n" "$PHONON_EXEC"
printf "  3. Validate Circuit Netlist:        ${CYAN}%s validate <netlist.cir>${NC}\n" "$PHONON_EXEC"
printf "  4. Run Transient Simulation:        ${CYAN}%s run <netlist.cir>${NC}\n" "$PHONON_EXEC"
printf "  5. Web CAD Studio (Browser):        ${CYAN}https://phonon.aerovex.net/studio${NC}\n"
printf "  6. Documentation & Architecture:    ${CYAN}https://phonon.aerovex.net${NC}\n"
printf "\n"
