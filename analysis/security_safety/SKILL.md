---
name: security_safety
description: Numerical safety, IEEE 754 floating-point hygiene, FTZ/DAZ subnormal handling, unsafe code auditing, netlist DoS mitigation, and parser security in Rust.
---

# Numerical Safety, Memory Invariants & Parser Security

## 1. Floating-Point Hygiene & Numerical Robustness

In numerical simulation, unhandled IEEE 754 anomalies (`NaN`, `+Inf`, `-Inf`, and subnormals) can silently corrupt state vectors, corrupting results or triggering infinite loops.

### 1.1 NaN and Infinity Trapping
Phonon enforces strict invariant checking after every linear solve and Newton-Raphson update:

```rust
#[inline]
pub fn validate_state_vector(vec: &[f64]) -> Result<(), SolverError> {
    for (i, &val) in vec.iter().enumerate() {
        if val.is_nan() {
            return Err(SolverError::NumericalAnomaly {
                node_idx: i,
                kind: AnomalyKind::NotANumber,
            });
        }
        if val.is_infinite() {
            return Err(SolverError::NumericalAnomaly {
                node_idx: i,
                kind: AnomalyKind::InfiniteVoltageOrCurrent,
            });
        }
    }
    Ok(())
}
```

### 1.2 Flush-To-Zero (FTZ) & Denormals-Are-Zero (DAZ)
In semiconductor off-states (e.g., reverse diode leakage $I_s \sim 10^{-15}\text{ A}$), values can drop into the IEEE 754 subnormal range ($< 2.22 \times 10^{-308}$).
*Hardware Penalty*: Modern x86 and ARM CPUs handle subnormal floating-point numbers in microcode traps, causing operations to run **$10\times$ to $100\times$ slower**.

Phonon configures hardware FTZ and DAZ modes at the start of simulation runs:
```rust
#[cfg(target_arch = "x86_64")]
pub fn enable_fast_subnormal_handling() {
    use std::arch::x86_64::*;
    unsafe {
        // Set Flush-To-Zero (FTZ) and Denormals-Are-Zero (DAZ) bits in MXCSR
        let mxcsr = _mm_getcsr();
        _mm_setcsr(mxcsr | 0x8040);
    }
}
```
Subnormal currents are flushed cleanly to true zero, maintaining full processor pipeline speed without sacrificing electrical simulation accuracy.

---

## 2. Memory Safety & Unsafe Code Policy

Phonon adheres to idiomatic Rust memory safety:
1. **Workspace Policy**: `#![deny(unsafe_code)]` is enforced across `phonon-core`, `phonon-models`, `phonon-thermal`, `phonon-netlist`, `phonon-cli`, and `phonon-gui`.
2. **Encapsulated Hotspots**: In the rare instances where `unsafe` is required (e.g. unchecked indexing inside the innermost sparse LU solve kernel or hardware SIMD intrinsics):
   - Unsafe blocks must be isolated in a dedicated internal module (`phonon-solver::internal::fast_ops`).
   - Every `unsafe` block must be accompanied by a `// SAFETY:` comment proving why invariants (e.g. bounds checking proved by prior graph validation) cannot be violated.
   - All unsafe operations are validated continuously using `cargo-miri` and AddressSanitizer (ASan) in CI.

---

## 3. Netlist Parser Security & DoS Protection

When Phonon is exposed as a web application (via WASM) or used to parse netlists from third-party sources, malformed or malicious inputs must not crash the host or consume unbounded system resources.

### 3.1 Recursive Subcircuit Bombs
A malicious netlist can define mutually recursive subcircuits:
```spice
.SUBCKT BOMB_A 1 2
X1 1 2 BOMB_B
.ENDS
.SUBCKT BOMB_B 1 2
X2 1 2 BOMB_A
.ENDS
```
- **Depth Limiting**: The parser enforces a strict maximum subcircuit hierarchy depth (default: `MAX_SUBCKT_DEPTH = 64`).
- **Cycle Detection**: The elaboration pass constructs a directed acyclic graph (DAG) of subcircuit definitions. If a cycle is detected, elaboration aborts immediately with `Err(NetlistError::CircularSubcircuitDefinition)`.

### 3.2 Allocation Quotas
- Maximum node count limit (e.g., $10^6$ nodes in native, $5 \times 10^4$ in WASM).
- Maximum memory allocation bounds for sparse matrix assembly.
- Execution timeout bounds for Newton-Raphson DC operating point calculations.

### 3.3 Electrical Sanity Pre-Checks
Before attempting to factorize the MNA matrix, Phonon scans the circuit topology:
- **Voltage Source Loops**: Detects closed loops composed entirely of ideal voltage sources or zero-ohm inductors (which produce singular, unsolvable matrices).
- **Current Source Cutsets**: Detects nodes connected only to ideal current sources that violate KCL ($\sum I_{source} \ne 0$).
- **Floating Nodes**: Identifies nodes with zero DC path to ground, warning the user and automatically connecting a high-impedance bleeder resistor ($G_{shunt} = 10^{-12}\text{ S}$) if auto-remediation is enabled.

---

## 4. Physical Invariant Preservation & Conservation Laws

To ensure multi-scale multi-physics solvers do not drift into non-physical states due to numerical truncation errors, Phonon enforces invariant guards:

### 4.1 Spintronics: Unit Magnetization Norm
For Landau-Lifshitz-Gilbert-Slonczewski (LLGS) dynamics:
$$\frac{d\vec{m}}{dt} = -\gamma \vec{m} \times \vec{H}_{eff} + \alpha \vec{m} \times \frac{d\vec{m}}{dt} + \vec{\tau}_{STT}$$
Because $\frac{d}{dt} |\vec{m}|^2 = 2 \vec{m} \cdot \frac{d\vec{m}}{dt} \equiv 0$ in the absence of numerical error, standard Runge-Kutta integration can cause $|\vec{m}|$ to drift.
Phonon monitors the norm after every step:
$$\big| |\vec{m}| - 1.0 \big| < 10^{-12}$$
If drift exceeds $10^{-6}$, the state is projected back to the unit sphere: $\vec{m}_{proj} = \vec{m} / |\vec{m}|$.

### 4.2 Superconducting Flux Quantization
In closed superconducting loops containing Josephson junctions:
$$\oint \vec{A} \cdot d\vec{l} + \frac{m^*}{q^*} \oint \vec{v}_s \cdot d\vec{l} = n \Phi_0, \quad \Phi_0 = \frac{h}{2e} \approx 2.0678 \times 10^{-15}\text{ Wb}$$
The integrated terminal voltage across phase-slip junctions must satisfy:
$$\Delta \Phi = \int_{t_1}^{t_2} V(t) \, dt = \frac{\Phi_0}{2\pi} \Delta \phi = n \Phi_0 \pm \delta \Phi_{leak}$$
Any deviation in closed loops without normal resistance triggers an assertion failure.

### 4.3 Quantum Mechanics: Norm & Trace Preservation
- **Wavefunction Normalization**: $\sum_{i} |\psi_i|^2 = 1.0 \pm 10^{-14}$.
- **Density Matrix Invariants**: $\text{Tr}(\rho) = 1.0$ and eigenvalues $\lambda_i(\rho) \ge 0$ (positive semi-definiteness).
- **Stabilizer Commutation**: In surface-code lattices, all stabilizer generators must commute: $[S_i, S_j] = 0$.

---

## 5. Matrix Ill-Conditioning, Regularization & Singularity Trapping

When circuits transition between extreme operating points (e.g. subthreshold to deep inversion, or superconducting to normal state), the Jacobian condition number can explode:
$$\kappa(\mathbf{J}) = \|\mathbf{J}\|_1 \|\mathbf{J}^{-1}\|_1 > 10^{16}$$

Phonon implements a 4-tier convergence fallback engine:
1. **Adaptive Damped Newton-Raphson**: If $\|F(x_{k+1})\| > \|F(x_k)\|$, step size is cut by half: $\Delta x = \alpha \mathbf{J}^{-1} F(x_k)$ where $\alpha \in \{0.5, 0.25, 0.125\}$.
2. **Dynamic Gmin Stepping**: High-impedance reverse-biased junctions receive temporary parallel conductances $G_{min} = 10^{-3}\text{ S}$, which are iteratively stepped down by orders of magnitude to $10^{-12}\text{ S}$.
3. **Independent Source Stepping**: Independent DC supplies ($V_{DD}, I_{bias}$) are scaled by $\alpha \in [0.0, 1.0]$. The solution from step $\alpha_k$ seeds the initial guess for $\alpha_{k+1}$.
4. **Pseudo-Transient Continuation (PTC)**: Stubborn algebraic constraints are augmented with fictitious transient capacitors ($C_{pseudo} \frac{dx}{dt}$), solving a stable pseudo-time ODE until $\frac{dx}{dt} \to 0$.

---

## 6. WebAssembly Sandboxing & Multi-Tenant Hardening

When running in browser WebAssembly or multi-tenant cloud environments:
1. **Linear Memory Isolation**: WASM linear memory is constrained to 4 GB with strict bounds checks enforced by the WebAssembly runtime.
2. **Execution Fuel / Cycle Budgeting**: The solver loop inspects an atomic instruction/step counter. If simulation iterations exceed the allocated quota (e.g., $10^7$ steps in browser demo mode), the engine yields execution gracefully:
   ```rust
   if self.cycle_count >= self.max_allowed_cycles {
       return Err(SolverError::CycleBudgetExceeded);
   }
   ```
3. **Panic Containment**: Uncaught Rust panics are intercepted at the FFI/WASM boundary via `std::panic::catch_unwind`, converting panics into structured JSON error payloads without tearing down the host process.

