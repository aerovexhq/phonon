# Phonon — Engineering & Agentic Workflow Guide

Welcome to **Phonon**. This document serves as the authoritative context, architectural mandate, and operational standard for all AI agents and engineers contributing to the Phonon project.

---

## 1. Project Purpose & Vision

Phonon is an **industry-grade, physically rigorous electro-thermal circuit simulator and transistor-level solver** written in modern Rust. It is engineered to bridge the gap between abstract SPICE circuit simulation and microscopic physical semiconductor physics.

### Core Capabilities:
1. **Raw Circuitry & Transistor-Level Rigor**:
   - Supports discrete components, raw silicon circuits, and ultra-deep sub-micron semiconductor devices.
   - Transistor models (BSIM3v3.3, BSIM4, BSIM-CMG FinFET, Gummel-Poon BJT) are formulated from physical transport equations with charge conservation ($\text{Ward-Dutton}$ formulation), velocity saturation, drain-induced barrier lowering (DIBL), and subthreshold swing.
2. **Coupled Electro-Thermal Physics**:
   - Every active and passive element exhibits real-time temperature dependence ($V_{th}(T), \mu(T), R(T)$).
   - Dynamic self-heating is solved monolithically or via tightly coupled relaxation: electrical Joule dissipation heats the thermal grid (discretized Fourier conduction / Cauer ladder network), which in turn dynamically modulates electrical characteristics.
3. **OS-Independent CLI & GUI**:
   - **Headless Engine & CLI**: 100% platform-agnostic, zero-GUI core suitable for high-throughput headless batch simulations, parametric sweeps, Monte Carlo analysis, and CI/CD automation.
   - **Modern Interactive GUI**: High-performance, GPU-accelerated immediate-mode CAD with schematic capture, live node probing, virtual oscilloscopes, and 2D/3D thermal heatmaps.
   - **WebAssembly Ready**: Compilable to WASM with browser filesystem support (IndexedDB) for browser-based interactive simulation without sacrificing native desktop throughput.
4. **Professional Rust Tooling**:
   - Zero-allocation inner simulation loops, cache-conscious Data-Oriented Design (DOD), SIMD vectorization, and deterministic IEEE 754 floating-point handling without panics.

---

## 2. Development Workflow: Future-Current-Done

All planning, execution, and tracking follow a strict three-tier lifecycle defined in [`todo.md`](file:///D/Projects/aerovex/modules/phonon/todo.md).

```
+-------------------------------------------------------------------+
|                            todo.md                                |
|                                                                   |
|   +-----------------------------------------------------------+   |
|   |  ## Future                                                |   |
|   |  Phases waiting to be activated (5-10 line descriptions)  |   |
|   +-----------------------------------------------------------+   |
|                                 |                                 |
|                                 v (Activate Phase)                |
|   +-----------------------------------------------------------+   |
|   |  ## Current                                               |   |
|   |  Active Phase being planned / executed                    |   |
|   +-----------------------------------------------------------+   |
|                                 |                                 |
|                                 v (Verification & Done)           |
|   +-----------------------------------------------------------+   |
|   |  ## Done                                                  |   |
|   |  Completed Phases (permanently retained)                  |   |
|   +-----------------------------------------------------------+   |
+-------------------------------------------------------------------+
```

### Workflow Rules for Agents:
1. **Phases as Atoms**: Every task in `todo.md` is a discrete, numbered **Phase** (e.g., Phase 1, Phase 2).
2. **5–10 Line Specifications**: Each phase description in `todo.md` must be exactly 5 to 10 lines long, detailing technical objectives, mathematical/solver requirements, and deliverables.
3. **Planning Before Execution**:
   - Never write code for a phase without first generating an `implementation_plan.md` artifact detailing the approach, architecture, file changes, and verification plan.
   - If there are architectural decisions or ambiguities, document them under **User Review Required** and **Open Questions** in the implementation plan.
   - Wait for explicit user confirmation before proceeding with modifications.
4. **Execution & Verification**:
   - Implement the code cleanly, with automated unit/integration tests and physical sanity checks.
   - Run tests, benchmarks, and lints (`cargo clippy`, `cargo test`).
5. **Advancement to Done**:
   - Once the phase objectives and verifications are met, move the phase item from `## Current` to `## Done` in `todo.md`.
   - Update `walkthrough.md` to document the delivered features and verification proofs.

---

## 3. Analysis & Specialized Domain Skills (`analysis/`)

The [`analysis/`](file:///D/Projects/aerovex/modules/phonon/analysis/) directory contains the project's permanent technical brain:

```
analysis/
├── analysis.md                        # Master architectural & physical-mathematical blueprint
├── physics_electrothermal/SKILL.md    # Coupled thermal dynamics, Fourier diffusion, self-heating
├── transistor_modeling/SKILL.md       # BSIM, Gummel-Poon, charge conservation, short-channel physics
├── circuit_solver/SKILL.md            # MNA formulation, Newton-Raphson, sparse LU, TR-BDF2/Gear
├── hardware_components/SKILL.md       # Real passives, non-linear diodes, parasitics, transmission lines
├── gui_ux/SKILL.md                    # CAD schematic capture, oscilloscope, thermal heatmaps
├── cli_interface/SKILL.md             # Headless engine, SPICE parser, telemetry streaming, CLI
├── architecture_codestructure/SKILL.md# Workspace topology, DOD, cache locality, SIMD, Rayon
├── security_safety/SKILL.md           # Memory safety, NaN/Inf handling, DoS protection, sandboxing
└── verification_validation/SKILL.md   # Physical conservation laws, SPICE benchmarks, test suites
```

### Rules for Maintaining Skills:
- **YAML Frontmatter**: Every `SKILL.md` must begin with standard YAML frontmatter (`name` and `description`).
- **Niche Detail**: Skills must not be generic summaries; they must provide in-depth mathematical formulations, circuit equations, numerical algorithms, code snippets, and edge-case handling.
- **Continuous Evolution**: As the codebase matures, agents must update these skill documents to reflect new design discoveries, algorithmic improvements, and benchmark figures.

---

## 4. Repository & Architecture Strategy

### Unified Multi-Crate Cargo Workspace
Phonon is organized as a unified Cargo workspace inside this repository. Splitting into separate git repositories or submodules should only occur if the GUI or web distribution becomes an entirely separate product with independent release schedules.

Expected Crate Structure:
- `phonon-core`: Common types, graph structures, node IDs, units, physical constants, error types.
- `phonon-solver`: Modified Nodal Analysis (MNA) matrix assembler, sparse LU solver, non-linear Newton-Raphson, adaptive time-step integrator (TR-BDF2).
- `phonon-models`: Physical models for passives (with parasitics), diodes, BJTs, and MOSFETs (BSIM3/4/CMG).
- `phonon-thermal`: Discretized thermal network solver (Cauer/Foster ladders, FEM/FDM grids, temperature-dependent coupling).
- `phonon-netlist`: SPICE 3f5 / HSPICE netlist lexer and AST parser, plus Phonon native formats.
- `phonon-cli`: Headless command-line binary with parametric sweeps, output formatters (CSV, Arrow, VCD).
- `phonon-gui`: Native CAD interface (schematic editor, oscilloscope visualizer, thermal heatmap) built with `egui` and `wgpu`.

---

## 5. Coding & Engineering Standards

1. **Rust Quality & Lints**:
   - Code must compile on stable Rust (`cargo build --workspace`).
   - Run `cargo clippy --workspace --all-targets -- -D warnings`.
   - Maintain idiomatic formatting via `cargo fmt --all`.
2. **Numerical Safety & Hygiene**:
   - Zero panics in the solver loop. All numerical anomalies (singular matrices, non-convergence, step rejections) must return structured `Result<T, SolverError>`.
   - Explicit handling of `f64` edge cases: test for `is_nan()` and `is_infinite()` on all state vector updates.
3. **Physical Validation**:
   - All models must adhere to physical conservation laws: Kirchhoff's Current Law ($\sum I = 0$), Kirchhoff's Voltage Law ($\sum V = 0$), and energy conservation ($\int P_{diss} dt = \Delta E_{stored}$).
   - Transistor capacitance models must be charge-conservative ($\sum q_i = 0$).
