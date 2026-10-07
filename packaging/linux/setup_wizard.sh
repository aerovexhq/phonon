#!/usr/bin/env bash
# ==============================================================================
# Phonon Universal Multi-Scale Visual Studio Linux Setup Wizard
# Interactive GUI (Zenity/KDialog) and TUI (Whiptail/Dialog) Setup Wizard
# Supports non-root user-space (~/.local) and system-wide (/usr/local) installs
# ==============================================================================

set -euo pipefail

VERSION="0.1.0"
APP_TITLE="Phonon Simulation Studio Setup Wizard"

# Detect display capabilities
has_gui() {
    [[ -n "${DISPLAY:-}" || -n "${WAYLAND_DISPLAY:-}" ]] && command -v zenity >/dev/null 2>&1
}

has_tui() {
    command -v whiptail >/dev/null 2>&1 || command -v dialog >/dev/null 2>&1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Source binary discovery
BIN_SOURCE="${ROOT_DIR}/dist/phonon-x86_64"
if [[ ! -f "$BIN_SOURCE" ]]; then
    BIN_SOURCE="${ROOT_DIR}/target/release/phonon"
fi

# ------------------------------------------------------------------------------
# GUI Implementation (Zenity)
# ------------------------------------------------------------------------------
run_gui_wizard() {
    # 1. Welcome Screen
    zenity --info \
        --title="${APP_TITLE}" \
        --width=480 \
        --height=220 \
        --text="Welcome to the Phonon Simulation Studio v${VERSION} Setup Wizard!\n\nThis wizard will guide you through installing the Phonon multi-scale quantum CAD simulation engine and desktop studio on your Linux system."

    # 2. License Agreement
    LICENSE_TEXT="Phonon Simulation Studio is licensed under Apache-2.0 / MIT.\n\nPermitted: Commercial use, modification, distribution, private use.\nConditions: License and copyright notices must be preserved.\nWarranty: Provided AS-IS without warranty of any kind."
    if ! echo -e "$LICENSE_TEXT" | zenity --text-info \
        --title="${APP_TITLE} - License Terms" \
        --width=520 \
        --height=280 \
        --checkbox="I accept the license terms and conditions."; then
        zenity --error --text="Installation cancelled: License terms were not accepted."
        exit 1
    fi

    # 3. Installation Destination
    INSTALL_SCOPE=$(zenity --list \
        --title="${APP_TITLE} - Installation Scope" \
        --radiolist \
        --column="Select" --column="Scope" --column="Destination Path" \
        TRUE "User Space (Recommended)" "$HOME/.local/bin (No root/sudo required)" \
        FALSE "System-Wide" "/usr/local/bin (Requires administrative privileges)" \
        --width=560 --height=220)

    if [[ "$INSTALL_SCOPE" == *"System-Wide"* ]]; then
        if [[ $EUID -ne 0 ]] && ! (command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null); then
            zenity --warning \
                --title="${APP_TITLE} - Elevation Notice" \
                --width=450 \
                --text="System-wide installation requires administrative privileges, but sudo is unavailable or requires a password.\n\nPhonon does not require sudo. Falling back to User Space ($HOME/.local/bin)." || true
            TARGET_BIN="$HOME/.local/bin"
            TARGET_DESKTOP="$HOME/.local/share/applications"
            TARGET_ICON="$HOME/.local/share/icons/hicolor/scalable/apps"
            TARGET_BASH="$HOME/.local/share/bash-completion/completions"
            TARGET_ZSH="$HOME/.local/share/zsh/site-functions"
            SUDO_CMD=""
        else
            TARGET_BIN="/usr/local/bin"
            TARGET_DESKTOP="/usr/share/applications"
            TARGET_ICON="/usr/share/icons/hicolor/scalable/apps"
            TARGET_BASH="/usr/share/bash-completion/completions"
            TARGET_ZSH="/usr/share/zsh/site-functions"
            SUDO_CMD="sudo"
        fi
    else
        TARGET_BIN="$HOME/.local/bin"
        TARGET_DESKTOP="$HOME/.local/share/applications"
        TARGET_ICON="$HOME/.local/share/icons/hicolor/scalable/apps"
        TARGET_BASH="$HOME/.local/share/bash-completion/completions"
        TARGET_ZSH="$HOME/.local/share/zsh/site-functions"
        SUDO_CMD=""
    fi

    # 4. Installation Progress
    (
        echo "10"; echo "# Creating destination directories..."
        $SUDO_CMD mkdir -p "$TARGET_BIN" "$TARGET_DESKTOP" "$TARGET_ICON" "$TARGET_BASH" "$TARGET_ZSH"
        sleep 0.2

        echo "40"; echo "# Installing Phonon unified binary..."
        if [[ -f "$BIN_SOURCE" ]]; then
            $SUDO_CMD cp -f "$BIN_SOURCE" "$TARGET_BIN/phonon"
            $SUDO_CMD chmod 755 "$TARGET_BIN/phonon"
        fi
        sleep 0.2

        echo "70"; echo "# Registering Desktop Studio launcher and icons..."
        if [[ -f "${ROOT_DIR}/packaging/phonon.desktop" ]]; then
            $SUDO_CMD cp -f "${ROOT_DIR}/packaging/phonon.desktop" "$TARGET_DESKTOP/phonon.desktop"
            $SUDO_CMD chmod 644 "$TARGET_DESKTOP/phonon.desktop"
        fi
        if [[ -f "${ROOT_DIR}/packaging/phonon.svg" ]]; then
            $SUDO_CMD cp -f "${ROOT_DIR}/packaging/phonon.svg" "$TARGET_ICON/phonon.svg"
            $SUDO_CMD chmod 644 "$TARGET_ICON/phonon.svg"
        fi
        sleep 0.2

        echo "90"; echo "# Configuring shell autocompletions..."
        if [[ -f "${ROOT_DIR}/packaging/completions/phonon.bash" ]]; then
            $SUDO_CMD cp -f "${ROOT_DIR}/packaging/completions/phonon.bash" "$TARGET_BASH/phonon"
        fi
        if [[ -f "${ROOT_DIR}/packaging/completions/_phonon.zsh" ]]; then
            $SUDO_CMD cp -f "${ROOT_DIR}/packaging/completions/_phonon.zsh" "$TARGET_ZSH/_phonon"
        fi
        sleep 0.2

        echo "100"; echo "# Updating desktop application database..."
        command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$TARGET_DESKTOP" 2>/dev/null || true
    ) | zenity --progress \
        --title="${APP_TITLE}" \
        --text="Installing Phonon Simulation Studio..." \
        --percentage=0 \
        --auto-close \
        --width=450

    # 5. Success Dialog with Launch Option
    if zenity --question \
        --title="${APP_TITLE} - Complete" \
        --text="Phonon Studio v${VERSION} has been successfully installed to:\n${TARGET_BIN}/phonon\n\nWould you like to launch Phonon Studio now?" \
        --ok-label="Launch Studio" \
        --cancel-label="Finish"; then
        "${TARGET_BIN}/phonon" gui &
    fi
}

# ------------------------------------------------------------------------------
# TUI Implementation (Whiptail / Dialog)
# ------------------------------------------------------------------------------
run_tui_wizard() {
    TUI_BIN="whiptail"
    command -v whiptail >/dev/null 2>&1 || TUI_BIN="dialog"

    # 1. Welcome
    $TUI_BIN --title "${APP_TITLE}" \
             --msgbox "Welcome to Phonon Simulation Studio v${VERSION} Setup Wizard.\n\nThis tool will configure the unified 'phonon' CLI and 'phonon gui' Studio on your system." 12 65

    # 2. Scope
    SCOPE_CHOICE=$($TUI_BIN --title "${APP_TITLE} - Scope" \
                            --menu "Select installation target directory:" 14 65 2 \
                            "1" "User Space (~/.local/bin) [No root needed]" \
                            "2" "System Wide (/usr/local/bin) [Requires sudo]" 3>&1 1>&2 2>&3)

    if [[ "$SCOPE_CHOICE" == "2" ]]; then
        if [[ $EUID -ne 0 ]] && ! (command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null); then
            $TUI_BIN --title "${APP_TITLE} - Elevation Notice" \
                     --msgbox "System-wide installation requires administrative privileges, but sudo requires a password or is unavailable.\n\nPhonon does not require sudo. Falling back to User Space (~/.local/bin)." 12 65
            TARGET_BIN="$HOME/.local/bin"
            TARGET_DESKTOP="$HOME/.local/share/applications"
            TARGET_ICON="$HOME/.local/share/icons/hicolor/scalable/apps"
            SUDO_CMD=""
        else
            TARGET_BIN="/usr/local/bin"
            TARGET_DESKTOP="/usr/share/applications"
            TARGET_ICON="/usr/share/icons/hicolor/scalable/apps"
            SUDO_CMD="sudo"
        fi
    else
        TARGET_BIN="$HOME/.local/bin"
        TARGET_DESKTOP="$HOME/.local/share/applications"
        TARGET_ICON="$HOME/.local/share/icons/hicolor/scalable/apps"
        SUDO_CMD=""
    fi

    # 3. Perform copy
    $SUDO_CMD mkdir -p "$TARGET_BIN" "$TARGET_DESKTOP" "$TARGET_ICON"
    if [[ -f "$BIN_SOURCE" ]]; then
        $SUDO_CMD cp -f "$BIN_SOURCE" "$TARGET_BIN/phonon"
        $SUDO_CMD chmod 755 "$TARGET_BIN/phonon"
    fi
    if [[ -f "${ROOT_DIR}/packaging/phonon.desktop" ]]; then
        $SUDO_CMD cp -f "${ROOT_DIR}/packaging/phonon.desktop" "$TARGET_DESKTOP/phonon.desktop"
    fi
    if [[ -f "${ROOT_DIR}/packaging/phonon.svg" ]]; then
        $SUDO_CMD cp -f "${ROOT_DIR}/packaging/phonon.svg" "$TARGET_ICON/phonon.svg"
    fi

    $TUI_BIN --title "${APP_TITLE} - Complete" \
             --msgbox "Installation complete!\n\nBinary installed to: ${TARGET_BIN}/phonon\n\nRun 'phonon' for CLI or 'phonon gui' for Desktop Studio." 12 65
}

# ------------------------------------------------------------------------------
# CLI Fallback
# ------------------------------------------------------------------------------
run_cli_fallback() {
    echo "================================================================================"
    echo "  Phonon Simulation Studio v${VERSION} - Interactive Setup"
    echo "================================================================================"
    echo "1) User Space (~/.local/bin) [Recommended]"
    echo "2) System Wide (/usr/local/bin)"
    read -rp "Select installation scope [1/2, default 1]: " choice
    if [[ "$choice" == "2" ]]; then
        if [[ $EUID -ne 0 ]] && ! (command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null); then
            echo "[WARN] System-wide installation requires administrative privileges, but sudo requires a password or is unavailable."
            echo "[INFO] Phonon does not require sudo. Falling back to User Space (~/.local/bin)..."
            TARGET_BIN="$HOME/.local/bin"
            TARGET_DESKTOP="$HOME/.local/share/applications"
            TARGET_ICON="$HOME/.local/share/icons/hicolor/scalable/apps"
            SUDO_CMD=""
        else
            TARGET_BIN="/usr/local/bin"
            TARGET_DESKTOP="/usr/share/applications"
            TARGET_ICON="/usr/share/icons/hicolor/scalable/apps"
            SUDO_CMD="sudo"
        fi
    else
        TARGET_BIN="$HOME/.local/bin"
        TARGET_DESKTOP="$HOME/.local/share/applications"
        TARGET_ICON="$HOME/.local/share/icons/hicolor/scalable/apps"
        SUDO_CMD=""
    fi

    $SUDO_CMD mkdir -p "$TARGET_BIN" "$TARGET_DESKTOP" "$TARGET_ICON"
    if [[ -f "$BIN_SOURCE" ]]; then
        $SUDO_CMD cp -f "$BIN_SOURCE" "$TARGET_BIN/phonon"
        $SUDO_CMD chmod 755 "$TARGET_BIN/phonon"
    fi
    echo "Installed successfully to ${TARGET_BIN}/phonon"
}

# Main Execution Routing
if has_gui; then
    run_gui_wizard
elif has_tui; then
    run_tui_wizard
else
    run_cli_fallback
fi
