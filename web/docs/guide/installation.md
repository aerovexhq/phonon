# Installation & Distribution Guide

Phonon provides multiple streamlined distribution packages designed for rapid setup, automated deployment, and seamless replacement on all major Linux distributions.

## Method 1: Universal User-Space Installer (Recommended)

Phonon is a desktop EDA CAD suite that runs purely in user-space and does not require superuser (`sudo`) privileges:

```bash
# User-space installation to ~/.local/bin (no sudo required):
curl -fsSL https://phonon.aerovex.net/install.sh | bash

# Or install rootlessly from a downloaded .deb package without sudo:
./install.sh phonon_0.1.0_amd64.deb

# Verify installation:
phonon
```

### What gets installed:
- `~/.local/bin/phonon` (Unified executable CLI + GUI launcher)
- `~/.local/share/applications/phonon.desktop` (Application menu launcher)
- `~/.local/share/icons/hicolor/scalable/apps/phonon.svg` (Scalable vector icon)
- `~/.local/share/bash-completion/completions/phonon` (Bash autocompletion)
- `~/.local/share/zsh/site-functions/_phonon` (Zsh autocompletion)

To uninstall:
```bash
./install.sh --uninstall
```

---

## Method 2: System-Wide Debian Package (.deb)

For system administrators configuring a multi-user machine or root environment:

```bash
# Download release package
wget https://github.com/aerovexhq/phonon/releases/download/v0.1.0/phonon_0.1.0_amd64.deb

# System-wide installation
sudo dpkg -i phonon_0.1.0_amd64.deb

# Or with custom system prefix:
sudo bash install.sh --system --prefix /opt/phonon
```

---

## Method 3: Standalone Portable Binary Executable

If you want a portable single binary without any system packages or root access:

```bash
# Download standalone x86_64 stripped binary
wget https://github.com/aerovexhq/phonon/releases/download/v0.1.0/phonon-x86_64

# Mark as executable
chmod +x phonon-x86_64

# Run immediately
./phonon-x86_64 --help
./phonon-x86_64 ui
```

---

## Method 4: Universal Distribution Tarball (.tar.gz)

```bash
wget https://github.com/aerovexhq/phonon/releases/download/v0.1.0/phonon-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
tar -xzf phonon-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
cd phonon-v0.1.0-x86_64-unknown-linux-gnu
sudo bash install.sh
```

---

## Method 5: Windows Setup Wizard & Enterprise MSI

### Graphical Setup Wizard (.exe)
Download the Inno Setup interactive installer:
- **File**: `phonon-setup-0.1.0-x64.exe`
- **Features**: Step-by-step graphical wizard, directory selection, automatic `PATH` environment registration, Start Menu & Desktop shortcuts, file associations (`.phonon`, `.sp`, `.cir`), and complete uninstallation support.
- **Silent Install Switch**: `phonon-setup-0.1.0-x64.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART`

### Enterprise MSI Package (.msi)
For IT administrators, Microsoft Intune, Active Directory Group Policy (GPO), and SCCM:
- **File**: `phonon-0.1.0-x64.msi`
- **Features**: Native Windows Installer database compiled via WiX Toolset v4/v5 with standard Windows Installer properties and clean UAC elevation.
- **Silent Deployment**:
  ```cmd
  msiexec /i phonon-0.1.0-x64.msi /qn /l*v install.log
  ```

---

## Method 6: macOS Drag-and-Drop DMG & Guided PKG Wizard

### Styled Disk Image (.dmg)
- **File**: `Phonon-0.1.0.dmg`
- **Installation**: Double-click to mount the disk image, then drag `Phonon Studio.app` into your `Applications` directory.
- Supports Apple Silicon (`aarch64-apple-darwin`) and Intel (`x86_64-apple-darwin`).

### Guided Installer Package (.pkg)
- **File**: `Phonon-0.1.0.pkg`
- **Features**: Interactive Apple multi-step guided installer wizard with license acceptance, target volume selection, `/usr/local/bin/phonon` CLI symlink creation, and zsh shell completions setup.
- **Silent Command-Line Deployment**:
  ```bash
  sudo installer -pkg Phonon-0.1.0.pkg -target /
  ```

---

## Method 7: Linux Universal AppImage & RPM Package

### Portable AppImage (.AppImage)
Runs on any modern Linux distribution without installation:
```bash
wget https://github.com/aerovexhq/phonon/releases/download/v0.1.0/Phonon-0.1.0-x86_64.AppImage
chmod +x Phonon-0.1.0-x86_64.AppImage
./Phonon-0.1.0-x86_64.AppImage
```

### Fedora / RHEL / openSUSE RPM (.rpm)
```bash
sudo dnf install https://github.com/aerovexhq/phonon/releases/download/v0.1.0/phonon-0.1.0-1.x86_64.rpm
```

### Interactive Linux Setup Wizard (GUI / TUI)
For an interactive installation experience with automatic Zenity (GTK) / Whiptail (terminal) detection:
```bash
bash packaging/linux/setup_wizard.sh
```

---

## Method 8: Build from Source

Requirements:
- Rust 1.80+ (`cargo`, `rustc`)
- C compiler (`gcc` or `clang`)
- Linux libraries: `libasound2-dev`, `libudev-dev`, `pkg-config`

```bash
# Clone the repository
git clone https://github.com/aerovexhq/phonon.git
cd phonon

# Build optimized release binary
cargo build --release -p phonon-cli

# Run local binary
./target/release/phonon gui
```

