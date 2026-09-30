# Getting Started with Phonon

Phonon is an industry-grade, multi-scale electro-thermal circuit simulation engine and CAD environment designed for modern semiconductor device modeling, cryogenic electronics, and topological quantum acoustics.

## Architectural Highlights

1. **Unified Binary Architecture**:
   Phonon packages both its headless numerical solver CLI and its GPU-accelerated desktop visual studio into a single optimized binary:
   - Run `phonon` to inspect available commands or run headless simulations.
   - Run `phonon ui` (or `phonon gui`) to launch the interactive desktop CAD interface.

2. **Dual-Platform Visual Deployment**:
   - **Desktop CAD Studio**: Native desktop GUI powered by Rust (`eframe`/`egui`) with low-latency rendering and multi-threaded Rayon execution.
   - **Static Web Studio**: Pure client-side WebAssembly environment hosted at `/studio` deployable to any static web server with zero backend dependencies.

3. **6 Realism Tiers**:
   From microscopic 1D/2D mesh TCAD drift-diffusion up to non-Abelian topological anyon braiding processors, Phonon provides seamless multi-abstraction physics.

## Quick CLI Usage

### Check Phonon Version and Banner

```bash
phonon
```

### Validate a Circuit Netlist (ERC)

```bash
phonon validate examples/rc_filter.cir
```

### Run Transient Simulation

```bash
phonon run examples/rc_filter.cir --format csv --output results.csv
```

### Perform Parallel Parametric Sweep

```bash
phonon sweep examples/rc_filter.cir --param R1 --start 100 --stop 10000 --steps 20
```

### Launch Native Desktop CAD Studio

```bash
phonon ui
```
