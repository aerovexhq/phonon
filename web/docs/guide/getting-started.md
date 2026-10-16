# Getting Started with Phonon

Phonon is an industry-grade, multi-scale electro-thermal circuit simulation engine and visual CAD studio engineered for mission-critical semiconductor modeling, 4.2K cryogenic electronics, and hard real-time robotics co-simulation.

::: tip Architectural Overview
- **Unified Single Binary**: Integrates headless numerical solvers, parametric sweep engines, and GPU-accelerated desktop CAD in one executable.
- **Dual-Platform Studio**: Native desktop GUI powered by Rust (`eframe`/`egui`) alongside zero-dependency WebAssembly browser studio at `/studio`.
- **6 Multi-Scale Realism Tiers**: From microscopic 1D/2D drift-diffusion TCAD to topological non-Abelian acoustic phonon lattices.
- **Zero-Copy Robotics Co-Sim**: 400 Hz POSIX shared memory ring buffer integration with the Aerovex Chronos multi-world flight simulator.
:::

## Core CLI Command Reference

| Command | Syntax | Primary Operation |
| :--- | :--- | :--- |
| `phonon` | `phonon [OPTIONS]` | Displays runtime telemetry, hardware SIMD features, and version |
| `phonon validate` | `phonon validate <NETLIST>` | Executes Electrical Rules Checks (ERC), detects floating nets and DC loops |
| `phonon run` | `phonon run <NETLIST> [FLAGS]` | Executes DC operating point, AC frequency response, or transient analysis |
| `phonon sweep` | `phonon sweep <NETLIST> [ARGS]` | Multi-threaded Rayon parallel parametric sweep across components |
| `phonon monte-carlo` | `phonon monte-carlo <NETLIST>` | Vectorized statistical yield distribution and corner analysis |
| `phonon gui` | `phonon gui [NETLIST]` | Launches native GPU-accelerated desktop visual schematic editor |

## Quickstart Workflow in 60 Seconds

### 1. Validate Netlist Topology (ERC)

Before running transient simulation, execute Electrical Rules Checks (ERC) to verify pin counts, model parameters, and node ground references:

```bash
phonon validate examples/rc_filter.cir
```

### 2. Run High-Precision Transient Simulation

Execute adaptive TR-BDF2 transient analysis with CSV telemetry streaming:

```bash
phonon run examples/rc_filter.cir --format csv --output results.csv
```

### 3. Parallel Parametric Sweep

Execute a 20-step logarithmic resistance sweep across multi-core CPU threads:

```bash
phonon sweep examples/rc_filter.cir --param R1 --start 100 --stop 10000 --steps 20 --threads 8
```

### 4. Launch Desktop Visual CAD Studio

Launch the GPU-accelerated desktop workstation:

```bash
phonon gui examples/rc_filter.cir
```

::: info Static Web Studio Deployment
To evaluate circuit simulations without any local software installation, open the static WebAssembly environment directly in your browser:
**[Launch Phonon Web Studio](/studio/)**
:::
