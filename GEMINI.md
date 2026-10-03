# Phonon — Engineering Guidelines & Agentic Workflow Guide

## Initial Protocol & Project Identity
- **Project Identity**: Phonon is an industry-grade, physically rigorous, universal multi-scale electro-thermal circuit simulator, quantum acoustic EDA, and transistor-level SPICE engine written in 100% pure safe Rust.
- **Standalone Governance**: Phonon is hosted as an independent, fully decoupled open-source project under `aerovexhq/phonon`. It maintains zero hard dependencies on proprietary simulation engines or closed-source frameworks.
- **Always Consult Analysis Files First**: At the start of a conversation or when receiving an initial prompt, always read and check the documentation in `analysis/` (especially `analysis/analysis.md`) to thoroughly understand the project architecture, numerical solvers, and physical models.

---

## Mandatory Engineering Rules & Constraints

### 1. Pure Safe Rust Mandate
- `#![deny(unsafe_code)]` MUST be present at line 1 of every new or modified Rust source (`.rs`) and test file across all crates in the workspace.
- Unsafe code blocks (`unsafe { ... }`) are strictly forbidden. All numerical algorithms, memory layouts, and data parsing must be implemented using memory-safe abstractions.

### 2. Strictly Zero Unicode Emojis
- Strictly ZERO Unicode emojis are permitted anywhere in the repository:
  - No emojis in Rust source code, docstrings, or inline comments.
  - No emojis in test output, assertion messages, or terminal logs.
  - No emojis in commit messages or git tags.
  - No emojis in markdown documentation (`todo.md`, `walkthrough.md`, `README.md`, `GEMINI.md`, etc.).

### 3. Ecosystem Decoupling & Ghost Kernel Architecture
- **Reference Dynamics Autonomy**: Phonon ships with a lightweight, self-contained reference 6-DOF Runge-Kutta 4th-order (RK4) rigid-body dynamics engine (`crates/phonon-core/src/dynamics_reference.rs`) that compiles and runs out-of-the-box on all platforms without external services.
- **Zero Closed-Source Leaks**: Proprietary Aerovex simulation math, kernels, or symbols (`world_manager`, `featherstone`, `engine_bullet`, `aerovex_sim::`) must NEVER be committed, referenced, or linked into the Phonon repository.
- **Optional Environmental Connectors**: Phonon provides non-blocking presence detection (`AerovexPresenceProbe`) and atomic lock-free shared memory readers (`AerovexShmBackend`) that dynamically connect to external multi-physics buffers if present on `/dev/shm`, while gracefully falling back to reference dynamics when absent.

### 4. Canonical Command Naming
- The native desktop CAD studio command is strictly named `phonon gui`.
- Legacy command aliases such as `phonon ui` are deprecated and strictly removed from the CLI, packaging scripts, desktop entries, and documentation.

### 5. Git Policy
- **Authorized Operations**: You are explicitly permitted to stage (`git add`), commit (`git commit`), update branch refs (`git update-ref refs/heads/main HEAD`), and push (`git push origin main`) changes responsibly upon completion of each prompt or phase.
- **Strictly Prohibited Operations**: Never execute destructive git operations such as `git reset`, `git checkout`, `git restore`, `git clean`, `git stash`, `git rebase`, or force push (`--force`).
- **Commit Message Hygiene**: Use standard Conventional Commits format (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`). Commit titles and bodies must NEVER include phase numbers (e.g. strictly do NOT mention "Phase <num>", "Phase 324", "Phase 325", etc.) or arbitrary milestone tags. Provide clear, descriptive, professional, and human-readable information explaining what the commit accomplishes.
- **Commit and Push After Each Phase**: Always stage, commit, and push changes to `origin/main` upon completion of each roadmap phase.

### 6. Documentation & Autonomous Lifecycle Tracking
- **Phonon Autonomous Roadmap**: All Phonon-specific tasks and roadmap phases are tracked exclusively inside `todo.md`.
- **Three-Tier Lifecycle**:
  - `## Future`: Upcoming phases waiting to be activated (5-10 line specifications).
  - `## Current`: Active phase being planned or executed.
  - `## Done`: Completed phases permanently retained with deliverables and test summaries.
- **Verification Walkthrough**: Update `walkthrough.md` after every phase with benchmark throughput metrics, test results, and verification proofs.

### 7. Periodic Transistor Speed Regression Protocol
- Every 5 roadmap phases (e.g., Phase 305, Phase 310, Phase 315, Phase 320), execute an automated 6-tier transistor speed regression audit covering:
  - Tier 1: Microscopic TCAD 1D Mesh Drift-Diffusion
  - Tier 2a: Inverse Design Single Genome Fitness
  - Tier 2b: Full NSGA-II + Adjoint Optimization
  - Tier 3a: Compact BSIM4 MOSFET + Ward-Dutton Charges
  - Tier 3b: Compact Gummel-Poon BJT
  - Tier 3c: Full MNA Circuit Newton-Raphson DC Solve
  - Tier 4: Cryo-CMOS 4.2K Freeze-Out & Central-Diff Jacobians
  - Tier 5: Coupled Electro-Thermal Monolithic Steady-State
  - Tier 6: SIMD 4-Lane Vectorized Batch Devices
- Document the regression table against previous baselines in `analysis/history/engineering_decisions_log.md` to guarantee zero performance regression.

### 8. Continuous Release Asset Packaging
- Whenever release-impacting commits or milestones are completed, compile the optimized release binary (`cargo build --release -p phonon-cli`), execute `packaging/build_packages.sh`, and upload distribution assets (`install.sh`, `phonon_0.1.0_amd64.deb`, `phonon-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`, `phonon-x86_64`, `setup_wizard.sh`, `SHA256SUMS`) to the `v0.1.0` GitHub Release on `aerovexhq/phonon` via `gh release upload ... --clobber`.

---

## Architecture & Workspace Crate Topology

Phonon is organized as a high-performance modular Cargo workspace:

- **`phonon-core`**: Core mathematical types, units, physical constants, abstract `PhysicsDynamicsBackend` trait, reference RK4 dynamics engine, presence probes, and error handling.
- **`phonon-solver`**: Modified Nodal Analysis (MNA) matrix assembler, sparse LU solver, non-linear Newton-Raphson, adaptive time-step integrator (TR-BDF2), and mixed-signal co-simulation kernel.
- **`phonon-models`**: Physical semiconductor models (BSIM3v3.3, BSIM4 with Ward-Dutton charge conservation, BSIM-CMG FinFET, Gummel-Poon BJT, cryogenic 4.2K freeze-out, and genetic parameter extraction).
- **`phonon-thermal`**: Discretized Cauer RC thermal ladder networks and monolithic electro-thermal coupling.
- **`phonon-netlist`**: SPICE 3f5 / HSPICE netlist parser, AST compiler, and bidirectional netlist synchronization engine.
- **`phonon-cli`**: Command-line interface for headless validation, batch simulation, and native CAD launch via `phonon gui`.
- **`phonon-gui`**: Cross-platform desktop and web visual CAD studio built on `egui` and `wgpu`, featuring interactive schematic capture, 7-category component drawer, full undo/redo history stack, custom window frame, and oscilloscope visualizers.
