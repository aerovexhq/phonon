---
name: verification_validation
description: Comprehensive verification methodology, conservation law probes (KCL, KVL, energy balance), industry SPICE/NIST benchmarks, and automated regression testing.
---

# Verification, Physical Validation & Benchmark Methodology

## 1. Physical Conservation Law Probes

A simulator can produce smooth, stable numerical curves that are completely unphysical. Phonon enforces physical validity by evaluating fundamental conservation laws at runtime.

### 1.1 Kirchhoff's Current Law (KCL) Residual Probe
At every accepted time-step $t_n$, the solver computes the algebraic residual norm across all $N$ circuit nodes:

$$R_i(t_n) = \sum_{k \in \text{Connected}(i)} I_{ki}(t_n) - I_{external, i}(t_n)$$

$$\|\mathbf{R}(t_n)\|_\infty = \max_{1 \le i \le N} |R_i(t_n)|$$

- **Acceptance Criterion**: $\|\mathbf{R}(t_n)\|_\infty < \text{reltol} \cdot I_{max} + \text{abstol}$ (typically $< 10^{-11}\text{ A}$).
- If the KCL residual violates tolerance, the time-step is flagged and rejected.

### 1.2 First Law of Thermodynamics: Energy Balance Verification
In any closed electrical and thermal network, total energy supplied by excitation sources must equal total energy dissipated as Joule heat plus the change in stored electromagnetic and thermal energy:

$$\int_0^t P_{supply}(\tau) d\tau = \int_0^t P_{joule}(\tau) d\tau + \Delta E_{EM}(t) + \Delta E_{thermal}(t)$$

Where:
- $P_{supply}(t) = \sum_{s \in \text{Sources}} v_s(t) \cdot i_s(t)$
- $P_{joule}(t) = \sum_{r \in \text{Resistors}} \frac{v_r(t)^2}{R_r} + \sum_{d \in \text{Active}} v_d(t) \cdot i_d(t)$
- $\Delta E_{EM}(t) = \sum \frac{1}{2} C_k [v_k(t)^2 - v_k(0)^2] + \sum \frac{1}{2} L_m [i_m(t)^2 - i_m(0)^2]$
- $\Delta E_{thermal}(t) = \sum C_{th, j} [T_j(t) - T_j(0)]$

#### Energy Error Metric:
$$\text{Error}_{Energy}(t) = \frac{\left| \int_0^t P_{supply} d\tau - \left( \int_0^t P_{joule} d\tau + \Delta E_{EM} + \Delta E_{thermal} \right) \right|}{\max\left( \int_0^t |P_{supply}| d\tau, \, 10^{-9} \right)}$$
Phonon logs an automated verification assertion if $\text{Error}_{Energy}(t) > 0.001$ ($0.1\%$).

---

## 2. Analytical Closed-Form Reference Tests

Unit tests compare simulation output against exact mathematical solutions:

### 2.1 First-Order RC Transient
- Circuit: $V_{step} = 5.0\text{V}$, $R = 10\text{ k}\Omega$, $C = 100\text{ nF}$ ($\tau = RC = 1.0\text{ ms}$).
- Analytical Solution: $V_C(t) = 5.0 \left(1 - e^{-t / \tau}\right)$.
- Expected L2 Error Norm: $\|V_{sim}(t) - V_C(t)\|_2 < 10^{-4}\text{ V}$.

### 2.2 Second-Order RLC Damped Oscillator
- Governing Equation: $\frac{d^2 v}{dt^2} + 2\zeta \omega_0 \frac{dv}{dt} + \omega_0^2 v = 0$
- Tested across three physical regimes:
  1. **Underdamped ($\zeta < 1$)**: Verifies frequency accuracy and decaying envelope.
  2. **Critically Damped ($\zeta = 1$)**: Verifies fastest non-oscillatory settling time.
  3. **Overdamped ($\zeta > 1$)**: Verifies two distinct decaying exponential rates without artificial numerical oscillations.

---

## 3. Industry SPICE & NIST Benchmark Suite

Phonon integrates standard benchmark netlists used to qualify industrial circuit simulators:

```
tests/benchmarks/
├── spice3f5/
│   ├── ring_oscillator_17stage.cir   # Benchmark for charge conservation & speed
│   ├── bjt_differential_pair.cir     # Small-signal & high-frequency distortion
│   ├── bandgap_reference.cir         # Electro-thermal stability across -40C to 125C
│   └── operational_amplifier_741.cir # Slew rate, CMRR, and DC gain
├── nist/
│   ├── stiff_diode_decay.cir         # Tests L-stability under severe stiffness
│   └── transmission_line_pulse.cir   # Tests high-frequency reflection dynamics
└── power/
    └── synchronous_buck_converter.cir# Stiff inductive switching & thermal runaway
```

### 3.1 Comparison Metrics with ngspice
For every benchmark netlist, Phonon automatically executes comparative runs against `ngspice -b`:
1. **Waveform Root-Mean-Square Deviation (RMSD)**:
   $$\text{RMSD} = \sqrt{\frac{1}{M} \sum_{k=1}^M \left( V_{phonon}(t_k) - V_{ngspice}(t_k) \right)^2 } < 0.05\%$$
2. **Total Accepted Time Steps & Newton Iterations**: Measures algorithmic convergence efficiency.
3. **Wall-Clock Time & Memory Consumption**: Asserts that Phonon outperforms or matches legacy C simulators.

---

## 4. Property-Based Testing with `proptest`

To uncover edge cases that manual test circuits miss, Phonon uses randomized circuit fuzzing:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_random_linear_resistive_network(
        num_nodes in 3usize..50,
        resistor_values in prop::collection::vec(1.0f64..1e6, 5..100)
    ) {
        let circuit = generate_random_connected_network(num_nodes, &resistor_values);
        let solution = solve_dc_operating_point(&circuit).expect("Linear circuit must converge");
        
        // Assert KCL at every node
        for node in 1..num_nodes {
            let kcl_sum = calculate_node_kcl(&circuit, &solution, node);
            prop_assert!(kcl_sum.abs() < 1e-10, "KCL violated at node {}", node);
        }
    }
}
```
This guarantees solver stability across thousands of arbitrary topology permutations.

---

## 5. Superconducting & Quantum Flux Quantization Probes

### 5.1 Flux Quantization Residual Verification
In Single Flux Quantum (SFQ) and Josephson junction networks, voltage pulses are physical topological solitons transporting an exact quantum of magnetic flux $\Phi_0$:
$$\Phi = \int_{-\infty}^{\infty} V(t) dt = \Phi_0 = \frac{h}{2e} \approx 2.067833848 \times 10^{-15}\text{ Wb}$$
Phonon executes automated trapezoidal and Simpson integration over every emitted voltage pulse:
$$\text{RelError}_{\Phi} = \frac{\left| \int_{t_{start}}^{t_{end}} V(t) dt - \Phi_0 \right|}{\Phi_0} < 0.001 \quad (0.1\%)$$
Any unphysical numerical pulse broadening or dissipation that violates flux quantization triggers an automated test assertion.

### 5.2 Superconducting Loop Persistent Current Invariant
In closed superconducting loops ($R \to 0$):
$$\frac{d\Phi_{loop}}{dt} = L_{loop} \frac{dI_{loop}}{dt} = 0 \implies \Phi_{loop}(t) = \Phi_{loop}(0) = n \Phi_0$$
Asserted across multi-nanosecond transient simulations without numerical decay.

---

## 6. Spintronic Magnetization Norm & Energy Conservation Probes

### 6.1 Unit Sphere Projection Invariant ($|\vec{m}| = 1.0$)
The macrospin magnetization vector $\vec{m}(t) = \vec{M} / M_s$ must reside strictly on the unit sphere $S^2$:
$$\left| \|\vec{m}(t)\|_2 - 1.0 \right| < 10^{-14}$$
Tested at every RK4 evaluation stage to prevent unphysical damping or non-unitary norm amplification.

### 6.2 Dipolar Interaction Energy Invariant
In autonomous nanomagnetic logic arrays, total magnetostatic dipolar energy:
$$E_{dip} = -\frac{1}{2} \mu_0 \sum_{i} \vec{\mu}_i \cdot \vec{H}_{dip, i}$$
must monotonically decrease during adiabatic magnetic clock relaxation toward the true ground state.

---

## 7. Quantum Error Correction (QEC) Stabilizer Group Probes

In cryogenic surface code decoders (`CryoQecDecoder`):
1. **Stabilizer Commutation**: Every stabilizer generator must commute with all others:
   $$[S_a, S_b] = 0 \quad \forall a, b \in \{1, \dots, N_{ancilla}\}$$
2. **Logical Operator Protection**: Logical operators $\bar{X}$ and $\bar{Z}$ must commute with all stabilizers but anticommute with each other:
   $$[S_a, \bar{X}] = 0, \quad [S_a, \bar{Z}] = 0, \quad \{\bar{X}, \bar{Z}\} = 0$$
3. **100% Syndrome Decoding Fidelity**: Automated regression loops inject all $3 \times N_{data}$ single-qubit Pauli errors ($X, Z, Y$) into the surface code lattice, asserting that the decoded correction operators satisfy:
   $$P_{err} \cdot P_{corr} \in \mathcal{S} \implies \text{Fidelity} = 100.0\%$$

