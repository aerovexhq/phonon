---
name: architecture_codestructure
description: High-performance Rust workspace architecture, Data-Oriented Design (DOD), cache locality, zero-allocation inner loops, SIMD vectorization, lock-free ring buffers, and multi-threading.
---

# Code Architecture & High-Performance Rust Design

## 1. Cargo Workspace Topology & Crate Hierarchy

Phonon is partitioned into modular, independently testable, high-cohesion crates within a single unified Cargo workspace:

```
phonon/
├── Cargo.toml                 # Workspace root manifest (linter configurations, profiles, dependencies)
├── crates/
│   ├── phonon-core/           # Mathematical primitives, units, graph representation, errors, SI constants
│   ├── phonon-solver/         # Sparse LU decomposition, MNA assembly, TR-BDF2, NEGF, LLGS, QEC, SOEN
│   ├── phonon-models/         # Transistor compact physics (BSIM, BJT), diodes, spintronics, cryo, optics
│   ├── phonon-thermal/        # Thermal PDE/FDM discretization, Cauer/Foster ladders, cryogenic budgets
│   ├── phonon-netlist/        # Lexer, SPICE 3f5 / HSPICE AST parser, netlist validator, ERC
│   ├── phonon-cli/            # Headless CLI binary with clap, parametric sweeps, Arrow/VCD telemetry
│   └── phonon-gui/            # GPU-accelerated interactive CAD GUI (egui + wgpu) & WASM runner
```

### 1.1 Architectural Dependency Graph
```
                          [ phonon-core ]
                           ^     ^     ^
                          /      |      \
               [ phonon-models ] |    [ phonon-thermal ]
                          \      |      /
                           v     |     v
                         [ phonon-solver ]
                                ^
                          ______|______
                         /             \
                 [ phonon-netlist ]     \
                        |                \
                        v                 v
                  [ phonon-cli ]     [ phonon-gui ]
```

- **Strict Unidirectional Layering**: Lower crates have zero knowledge of higher crates. Cyclic dependencies are structurally prohibited by Cargo.
- **Headless Physical Core**: `phonon-core`, `phonon-models`, `phonon-thermal`, and `phonon-solver` have **zero dependencies on windowing, GUI, or desktop display libraries**. They compile seamlessly to native bare-metal, Linux server farms, and `wasm32-unknown-unknown`.
- **Pure Safe Rust Mandate**: `#![deny(unsafe_code)]` is enforced across all crates. Where micro-optimizations occur, they rely on compiler vectorization, contiguous array layouts, and safe slicing.

---

## 2. Modular Subsystem Layout in Domain Crates

### 2.1 `phonon-models` Physical Hierarchy
The `phonon-models` crate houses physical compact models spanning classical, cryogenic, spintronic, and quantum regimes:

```
crates/phonon-models/src/
├── lib.rs                     # Re-exports and Unified Device Traits
├── passives/                  # Resistors, Capacitors, Inductors (with parasitic ESR/ESL, thermal drift)
├── diodes/                    # Shockley PN junctions, Schottky barrier, Zener/avalanche breakdown
├── transistors/               # BSIM3v3.3, BSIM4, BSIM-CMG FinFET, Gummel-Poon BJT
├── cryogenic/                 # Sub-4K carrier freeze-out, incomplete ionization, hopping conduction
├── quantum/                   # Surface-code geometry, Pauli syndrome generators, stabilizer bitmasks
├── spintronics/               # LLGS micromagnetics, MTJ resistance (Julliere/Slonczewski), PMA/IMA, NML
├── superconducting/           # Josephson junctions (RCSJ), RSFQ logic gates (DFF, AND, INV, JTL), SOEN
├── optoelectronics/           # Micro-ring resonators (MRR), Mach-Zehnder interferometers, waveguides
├── memristors/                # RRAM (filamentary), PCM (phase change), FeFET (polarization switching)
└── negf/                      # Non-Equilibrium Green's Function 1D tight-binding ballistic models
```

### 2.2 `phonon-solver` Multi-Physics Engine Hierarchy
```
crates/phonon-solver/src/
├── lib.rs                     # Master solver orchestrator
├── matrix/                    # Compressed Sparse Column (CSC/CSR), Sparse LU factorization, KCL/KVL stamping
├── transient/                 # Adaptive TR-BDF2, Gear 2, Forward/Backward Euler integrators
├── dc/                        # Newton-Raphson, Gmin stepping, Source stepping, Pseudo-transient continuation
├── ac/                        # Small-signal frequency domain analysis, phasor decomposition
├── thermal/                   # Monolithic Jacobian stamping and partitioned waveform relaxation
├── spintronics/               # Norm-preserving RK4 integration of Landau-Lifshitz-Gilbert-Slonczewski
├── negf/                      # Self-consistent Poisson-Schrodinger and NEGF density matrix iteration
└── superconducting/           # Cryogenic QEC syndrome decoding, SOEN recurrent spiking network simulation
```

---

## 3. Data-Oriented Design (DOD) & Cache Locality

Standard object-oriented SPICE implementations use linked lists of heap-allocated device structs, generating unpredictable pointer chasing, CPU cache misses, and pipeline stalls during matrix assembly. Phonon adopts **Data-Oriented Design**:

### 3.1 Structure of Arrays (SoA) for Inner Loops
Instead of an Array of Structures (AoS) which mixes cold and hot parameters:
```rust
// AVOID: AoS causes cache pollution when iterating only over electrical connections
struct Resistor {
    node_a: usize,
    node_b: usize,
    resistance: f64,
    temp_coeff: f64,
    device_name: String,     // Cold data polluting cache line
    model_name: String,      // Cold data polluting cache line
}
```

Phonon organizes component collections in contiguous flat arrays (Structure of Arrays):
```rust
// PREFERRED: Contiguous flat slices maximize L1/L2 data cache prefetching
pub struct ResistorBank {
    pub node_a: Vec<u32>,
    pub node_b: Vec<u32>,
    pub conductance: Vec<f64>,
    pub temp_coeff: Vec<f64>,
}

impl ResistorBank {
    #[inline]
    pub fn stamp_all(&self, matrix: &mut SparseMatrix) {
        let len = self.node_a.len();
        for i in 0..len {
            let na = self.node_a[i] as usize;
            let nb = self.node_b[i] as usize;
            let g = self.conductance[i];
            matrix.add(na, na, g);
            matrix.add(nb, nb, g);
            matrix.add(na, nb, -g);
            matrix.add(nb, na, -g);
        }
    }
}
```

### 3.2 Cache-Line Alignment to Eliminate False Sharing
In multi-threaded parallel sweeps or hybrid coprocessor execution, worker threads write individual thread-local telemetry and convergence flags. If two adjacent thread-local states share a single 64-byte L1 cache line, CPU cache coherency protocols (MESI/MOESI) invalidate the cache line back and forth, degrading parallel scaling.

Phonon prevents false sharing via explicit 64-byte alignment:
```rust
#[repr(align(64))]
pub struct ThreadLocalMetrics {
    pub step_count: u64,
    pub rejected_steps: u64,
    pub last_residual_norm: f64,
    pub energy_joules: f64,
}
```

---

## 4. Zero-Allocation Inner Simulation Loops

In stiff multi-physics simulations running $10^6$ to $10^9$ transient time steps, dynamic memory allocation (`malloc`, `Vec::push`, `Box::new`) in the inner loop causes heap fragmentation and microsecond latency spikes.

Phonon enforces **zero dynamic heap allocations** during active simulation:

```rust
pub struct TransientWorkspace {
    pub x_current: Vec<f64>,
    pub x_stage: Vec<f64>,
    pub x_next: Vec<f64>,
    pub x_dot: Vec<f64>,
    pub residual: Vec<f64>,
    pub delta: Vec<f64>,
    pub lte_error: Vec<f64>,
    pub scratch_indices: Vec<usize>,
}

impl TransientWorkspace {
    pub fn new(dim: usize) -> Self {
        Self {
            x_current: vec![0.0; dim],
            x_stage: vec![0.0; dim],
            x_next: vec![0.0; dim],
            x_dot: vec![0.0; dim],
            residual: vec![0.0; dim],
            delta: vec![0.0; dim],
            lte_error: vec![0.0; dim],
            scratch_indices: vec![0; dim],
        }
    }

    #[inline(always)]
    pub fn clear_residuals(&mut self) {
        self.residual.fill(0.0);
    }
}
```

All buffers are allocated once during circuit compilation/elaboration and reused indefinitely across all transient time steps.

---

## 5. SIMD Hardware Vectorization & Dynamic Dispatch Elimination

### 5.1 Static Enum Dispatch Over Dynamic Virtual Tables
Calling dynamic trait objects (`Box<dyn Device>` or `&dyn Device`) introduces vtable dereferences that prevent function inlining and vectorization.

Phonon employs closed enum dispatch:
```rust
pub enum DeviceKind {
    Resistor(ResistorModel),
    Capacitor(CapacitorModel),
    Diode(DiodeModel),
    Bsim4(Bsim4Model),
    JosephsonJunction(JosephsonJunctionModel),
    SpinTorqueMtj(MtjModel),
}

impl DeviceKind {
    #[inline(always)]
    pub fn stamp(&self, state: &[f64], matrix: &mut SparseMatrix, rhs: &mut [f64]) {
        match self {
            Self::Resistor(r) => r.stamp(state, matrix, rhs),
            Self::Capacitor(c) => c.stamp(state, matrix, rhs),
            Self::Diode(d) => d.stamp(state, matrix, rhs),
            Self::Bsim4(b) => b.stamp(state, matrix, rhs),
            Self::JosephsonJunction(j) => j.stamp(state, matrix, rhs),
            Self::SpinTorqueMtj(m) => m.stamp(state, matrix, rhs),
        }
    }
}
```
This enables LLVM to inline device stamping directly into the solver loop.

### 5.2 Explicit SIMD Vectorization
Semiconductor transport functions (e.g. BSIM4 or Diode exponential currents) are evaluated across 4 or 8 devices simultaneously:
```rust
use std::simd::{f64x4, SimdFloat};

#[inline(always)]
pub fn batch_diode_current_simd(
    vd: f64x4,
    is: f64x4,
    vt: f64x4,
) -> (f64x4, f64x4) {
    let arg = vd / vt;
    let exp_term = arg.exp();
    let current = is * (exp_term - f64x4::splat(1.0));
    let conductance = (is / vt) * exp_term;
    (current, conductance)
}
```

---

## 6. Concurrency Architecture & Inter-Thread Telemetry

### 6.1 Lock-Free Waveform Streaming (SPSC Ring Buffer)
The numerical simulation engine runs on a high-priority background worker thread, while the GUI or headless file writer runs on separate threads:

```
[ Solver Worker Thread ]
          |
          | (Zero-allocation lock-free push: `rtrb::Producer`)
          v
[ SPSC Ring Buffer: Atomic read/write pointers ]
          |
          | (Decimated pop: `rtrb::Consumer`)
          v
[ GUI Render Thread / Arrow Telemetry Streamer ]
```

- **Zero Lock Contention**: The numerical solver thread never blocks waiting for a mutex. If the buffer is full, the producer drops or decimates samples based on the user's fidelity configuration.
- **Microsecond Latency**: Guaranteed sub-microsecond handoff between solver and visualizer.

### 6.2 Rayon Parallelism for Multi-Scale Sweeps
1. **Parametric & Monte Carlo Sweeps**: Embarrassingly parallel across CPU cores using `rayon::par_iter()`. Each worker owns an independent `Solver` instance with its own pre-allocated workspace.
2. **SOEN Neural Networks**: Recurrent optical synaptic matrix evaluation is partitioned across Rayon parallel chunks:
   $$\vec{I}_{syn}[j] = \sum_{i} W_{ij} P_{opt}[i]$$
3. **Quantum Error Correction Rounds**: Syndrome decoding iterations across thousands of independent stabilizer cycles run concurrently across worker threads with linear CPU scaling.

