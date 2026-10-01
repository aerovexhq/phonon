---
layout: home

hero:
  name: "Phonon"
  text: "Universal Multi-Scale Visual CAD Studio & Semiconductor Solver"
  tagline: "Industry-grade electro-thermal circuit simulator, cryogenic CMOS modeler, topological quantum acoustics engine, and visual studio CAD interface."
  image:
    src: /favicon.svg
    alt: Phonon CAD Logo
  actions:
    - theme: brand
      text: Launch Web Studio
      link: /studio/
    - theme: alt
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: Download & Install (All OS)
      link: /guide/installation

features:
  - title: 6 Realism Tiers Accessible Visually
    details: Seamlessly scale from microscopic atomistic/semiconductor TCAD (1D/2D mesh Poisson-Drift-Diffusion) and BSIM4 SPICE to 4.2K Cryo-CMOS and topological non-Abelian quantum acoustic braiding lattices.
  - title: Unified CLI & Desktop CAD Binary
    details: Single binary executable. Run 'phonon' for high-throughput headless simulation, parametric sweeps, and ERC validation, or 'phonon gui' to launch the native GPU-accelerated desktop CAD interface.
  - title: Instant One-Command Distro Setup
    details: Replace and install with a single Debian package ('phonon_0.1.0_amd64.deb') or run the universal single-command installer across Ubuntu, Debian, Fedora, Arch, RHEL, openSUSE, Alpine, and NixOS.
  - title: Dual-Platform Studio Deployment
    details: High-performance native desktop studio with direct multi-core Rayon execution, paired with a zero-dependency static WebAssembly CAD studio hosted in the browser at /studio.
  - title: Coupled Electro-Thermal Multi-Physics
    details: Monolithic Modified Nodal Analysis (MNA) coupled dynamically with Cauer RC thermal networks, capturing non-linear Joule self-heating and thermal runaway.
  - title: High-Throughput SIMD Acceleration
    details: 4-lane SIMD vectorization and parallel Rayon execution capable of processing millions of transistor and quantum state evaluations per second.
---

## Quick Start in 60 Seconds

### Linux Installation Options

```bash
# Option A: Fast universal one-line installer (all popular distros)
curl -fsSL https://phonon.aerovex.net/install.sh | bash

# Option B: Single Debian/Ubuntu package (.deb)
wget https://github.com/aerovexhq/phonon/releases/download/v0.1.0/phonon_0.1.0_amd64.deb
sudo dpkg -i phonon_0.1.0_amd64.deb

# Option C: Standalone portable executable
wget https://github.com/aerovexhq/phonon/releases/download/v0.1.0/phonon-x86_64
chmod +x phonon-x86_64 && ./phonon-x86_64 ui
```

### Windows & macOS Setup Wizards

- **Windows**: Download interactive setup wizard [`phonon-setup-0.1.0-x64.exe`](/guide/installation#method-5-windows-setup-wizard--enterprise-msi) or enterprise [`phonon-0.1.0-x64.msi`](/guide/installation#method-5-windows-setup-wizard--enterprise-msi).
- **macOS**: Download drag-and-drop disk image [`Phonon-0.1.0.dmg`](/guide/installation#method-6-macos-drag-and-drop-dmg--guided-pkg-wizard) or guided wizard [`Phonon-0.1.0.pkg`](/guide/installation#method-6-macos-drag-and-drop-dmg--guided-pkg-wizard).


### Open Static Web Studio in Browser

Access the complete browser-based CAD environment with zero local installation:
**[Launch Phonon Web Studio](/studio/)**
