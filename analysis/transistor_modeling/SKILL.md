---
name: transistor_modeling
description: Advanced semiconductor physics, compact transistor models (BSIM3/4, BSIM-CMG, Gummel-Poon BJT), charge-conservative Ward-Dutton formulations, and short-channel effects in Rust.
---

# Transistor Modeling & Semiconductor Physics

## 1. Transistor Physics: From Microscopic to Compact Models

In nanometer-scale circuits and power switching devices, idealized textbook transistor equations (e.g., Shichman-Hodges square-law) fail catastrophically. Real devices are governed by high-field quantum and transport effects:
1. **Velocity Saturation**: Longitudinal electric fields accelerate carriers until they reach the scattering saturation velocity $v_{sat} \approx 10^7 \text{ cm/s}$, causing drain current to scale linearly with $V_{gs} - V_{th}$ rather than quadratically.
2. **Drain-Induced Barrier Lowering (DIBL)**: High drain voltages pull down the source-channel electrostatic potential barrier, degrading subthreshold slope and shifting $V_{th}$ downward.
3. **Channel-Length Modulation & Hot Carrier Effects**: The pinch-off region widens with drain voltage, modulating effective length $L_{eff}$.
4. **Quantum Confinement**: Thin oxide layers ($t_{ox} < 2\text{ nm}$) cause carrier quantization in the inversion layer, increasing effective oxide thickness $t_{ox, eff}$ and shifting threshold voltages.

---

## 2. Charge Conservation & Ward-Dutton Formulation

### 2.1 The Failure of Legacy Capacitance Models
Early SPICE simulators (e.g. SPICE2 with Meyer models) approximated dynamic charge storage by calculating small-signal capacitances:
$$C_{gs} = \frac{\partial Q_G}{\partial V_{GS}}, \quad C_{gd} = \frac{\partial Q_G}{\partial V_{GD}}$$
*Flaw*: Because $C_{ij} \ne C_{ji}$ in general multi-terminal non-linear devices, integrating $i = C \frac{dv}{dt}$ is not conservative over closed voltage cycles:
$$\oint i(t) dt \ne 0$$
This created artificial **charge pumping**, causing ring oscillators and switched-capacitor filters to violate the 1st Law of Thermodynamics (spontaneous energy generation or dissipation).

### 2.2 Ward-Dutton Charge Formulation
Phonon enforces **strict charge conservation** via the Ward-Dutton charge partitioning method. The state of the transistor is defined by independent terminal charges:
$$\mathbf{Q} = \begin{bmatrix} Q_G \\ Q_D \\ Q_S \\ Q_B \end{bmatrix}$$
Subject to the physical constraint of charge neutrality across the device:
$$Q_G + Q_D + Q_S + Q_B = 0$$

Dynamic terminal currents are computed strictly as the time derivatives of these charges:
$$i_j(t) = \frac{dQ_j}{dt}, \quad j \in \{G, D, S, B\}$$

The dynamic companion Jacobian matrix consists of the 16 non-reciprocal transcapacitances:
$$C_{jk} = -\frac{\partial Q_j}{\partial V_k}, \quad \text{for } j \ne k, \quad C_{jj} = \frac{\partial Q_j}{\partial V_j}$$
Where each row and column sums to zero:
$$\sum_{j} C_{jk} = 0, \quad \sum_{k} C_{jk} = 0$$

---

## 3. MOSFET Model Hierarchy

### 3.1 Unified DC Current Model (BSIM3v3 / BSIM4 Baseline)
To eliminate derivative discontinuities (which destroy Newton-Raphson quadratic convergence), Phonon utilizes unified smoothing functions.

#### 1. Unified Threshold Voltage Formulation:
$$V_{th} = V_{th0} + \gamma \left( \sqrt{\phi_s - V_{bs}} - \sqrt{\phi_s} \right) - \Delta V_{th, DIBL} - \Delta V_{th, SCE}$$
Where:
- $\Delta V_{th, DIBL} = \left( \eta_{DIBL} + \frac{\theta_{DIBL}}{L_{eff}} \right) V_{ds}$
- $\gamma = \frac{\sqrt{2 q \epsilon_{si} N_A}}{C_{ox}}$ is the body effect coefficient.

#### 2. Continuous Inversion Charge & Subthreshold Conduction:
The effective gate drive $V_{gsteff}$ smoothly interpolates between exponential subthreshold conduction and linear/saturation strong inversion:
$$V_{gsteff} = \frac{2 n V_t \ln\left(1 + \exp\left(\frac{V_{gs} - V_{th}}{2 n V_t}\right)\right)}{1 + 2 n \frac{C_{ox}}{\mu C_d} \exp\left(-\frac{V_{gs} - V_{th}}{2 n V_t}\right)}$$
Subthreshold ideality factor:
$$n = 1 + \frac{C_d}{C_{ox}} + \frac{C_{it}}{C_{ox}}$$

#### 3. Continuous Saturation Voltage ($V_{dseff}$):
$$V_{dsat} = \frac{E_{sat} L \cdot (V_{gsteff} / A_{bulk})}{E_{sat} L + (V_{gsteff} / A_{bulk})}$$
$$V_{dseff} = V_{dsat} - \frac{1}{2} \left[ V_{dsat} - V_{ds} - \delta + \sqrt{(V_{dsat} - V_{ds} - \delta)^2 + 4 \delta V_{dsat}} \right]$$
Where $\delta$ is a small smoothing parameter ($10^{-3}$). As $V_{ds} \to \infty$, $V_{dseff} \to V_{dsat}$ with infinite continuously differentiable derivatives $C^\infty$.

#### 4. Total Channel Current:
$$I_{ds} = \frac{\mu_{eff}(T) C_{ox} \frac{W}{L} V_{gsteff} \left(1 - \frac{A_{bulk} V_{dseff}}{2(V_{gsteff} + 2 V_t)}\right) V_{dseff}}{1 + \frac{V_{dseff}}{E_{sat} L}} \cdot \left(1 + \lambda_{CLM} \ln\left(1 + \frac{V_{ds} - V_{dseff}}{V_A}\right)\right)$$

### 3.2 FinFET Multi-Gate Modeling (BSIM-CMG Target)
For sub-7nm FinFET and GAA (Gate-All-Around) architectures:
- Surface potential $\psi_s$ is solved using the 1D Poisson-Boltzmann relation across the 3D fin:
  $$\frac{d^2 \psi}{dx^2} = \frac{q n_i}{\epsilon_{si}} \exp\left( \frac{\psi - V_{ch}}{V_t} \right)$$
- Volume inversion: in ultra-thin fins ($T_{fin} < 10\text{ nm}$), inversion charge forms throughout the fin body, not merely at the oxide interfaces.
- Fin quantum confinement shifts the conduction band upward:
  $$\Delta E_c \approx \frac{\pi^2 \hbar^2}{2 m^* T_{fin}^2}$$

---

## 4. Bipolar Junction Transistor (BJT) Modeling: Gummel-Poon

For analog signal processing, bandgap references, and RF circuits, the Gummel-Poon model represents BJT physics with high fidelity:

### 4.1 Normalized Base Charge $q_b$
$$q_b = \frac{q_1}{2} + \sqrt{\left(\frac{q_1}{2}\right)^2 + q_2}$$
Where:
- $q_1 = 1 + \frac{V_{bc}}{V_{AF}} + \frac{V_{be}}{V_{AR}}$ (Early voltage base-width modulation).
- $q_2 = \frac{I_{S}}{I_{KF}} \left( \exp\left(\frac{V_{be}}{N_F V_t}\right) - 1 \right) + \frac{I_{S}}{I_{KR}} \left( \exp\left(\frac{V_{bc}}{N_R V_t}\right) - 1 \right)$ (High-injection knee rolloff).

### 4.2 Terminal Currents:
$$I_C = \frac{I_S}{q_b} \left( \exp\left(\frac{V_{be}}{N_F V_t}\right) - \exp\left(\frac{V_{bc}}{N_R V_t}\right) \right) - \frac{I_S}{\beta_R} \left( \exp\left(\frac{V_{bc}}{N_R V_t}\right) - 1 \right) - I_{SC} \left( \exp\left(\frac{V_{bc}}{N_C V_t}\right) - 1 \right)$$
$$I_B = \frac{I_S}{\beta_F} \left( \exp\left(\frac{V_{be}}{N_F V_t}\right) - 1 \right) + I_{SE} \left( \exp\left(\frac{V_{be}}{N_E V_t}\right) - 1 \right) + \frac{I_S}{\beta_R} \left( \exp\left(\frac{V_{bc}}{N_R V_t}\right) - 1 \right) + I_{SC} \left( \exp\left(\frac{V_{bc}}{N_C V_t}\right) - 1 \right)$$

---

## 5. Rust Implementation & Companion Jacobian Stamps

```rust
pub struct MosfetEvaluator {
    // Model parameters
    pub vth0: f64,
    pub tox: f64,
    pub mu0: f64,
    pub w: f64,
    pub l: f64,
    pub gamma: f64,
    pub phi_s: f64,
    pub vsat: f64,
}

pub struct MosfetState {
    pub ids: f64,
    pub gm: f64,    // dIds / dVgs
    pub gds: f64,   // dIds / dVds
    pub gmbs: f64,  // dIds / dVbs
    pub qg: f64,
    pub qd: f64,
    pub qs: f64,
    pub qb: f64,
    pub c_matrix: [[f64; 4]; 4], // Non-reciprocal transcapacitances
}
```

### 5.1 Safe Exponential Limiting
To prevent floating-point overflow (`f64::INFINITY` or `NaN`) during Newton-Raphson trial steps:
```rust
#[inline(always)]
pub fn safe_exp(v: f64) -> (f64, f64) {
    const MAX_EXP_ARG: f64 = 80.0;
    if v < -MAX_EXP_ARG {
        (0.0, 0.0)
    } else if v > MAX_EXP_ARG {
        let exp_max = MAX_EXP_ARG.exp();
        // Linearized extrapolation past boundary to preserve non-zero gradient
        let val = exp_max * (1.0 + (v - MAX_EXP_ARG));
        let deriv = exp_max;
        (val, deriv)
    } else {
        let val = v.exp();
        (val, val)
    }
}
```
This linearized boundary guarantees continuous first derivatives and prevents matrix solver crashes even if extreme voltages (e.g. $+100\text{V}$) occur during intermediate iterations.

---

## 6. Multi-Scale Modeling: Microscopic TCAD Synthesis vs. Optimized Analytical Compact Models

To support both ultra-rigorous first-principles semiconductor physics and large-scale high-throughput circuit simulation, Phonon implements a **dual-tier multi-scale modeling architecture**.

```
+---------------------------------------------------------------------------------------+
|                                    PHONON CIRCUIT MNA                                 |
+---------------------------------------------------------------------------------------+
            ^                                                           ^
            | (Terminal I/V Stamped)                                    | (Terminal I/V Stamped)
            |                                                           |
+---------------------------------------+           +-----------------------------------+
|  TIER 1: Low-Level Structural TCAD    |           |  TIER 2: High-Level Compact Model |
|  - Raw Silicon / Oxide / Metal Mesh   |           |  - BSIM4 / BSIM-CMG / Gummel-Poon |
|  - Poisson-Drift-Diffusion PDEs       |           |  - Closed-form analytical curves  |
|  - Scharfetter-Gummel discretization  |           |  - O(1) evaluation latency        |
|  - Doping profiles (N_A, N_D)         |           |  - Charge-conservative Q(V)       |
|  - Direct microscopic device physics  |           |  - Ultra-high throughput circuits |
+---------------------------------------+           +-----------------------------------+
                    |                                           ^
                    +--- Automated Parameter Extraction & ------+
                         Surrogate Model Reduction (AI/Fit)
```

### 6.1 Low-Level Structural TCAD (Poisson-Drift-Diffusion)

For raw silicon devices built from scratch, the internal electrostatic potential $\psi(\mathbf{r})$ and quasi-Fermi potentials $(\phi_n, \phi_p)$ or carrier concentrations $(n, p)$ are governed by the semiconductor continuum partial differential equations:

1. **Non-linear Poisson Equation**:
   $$\nabla \cdot (\epsilon \nabla \psi) = -q (p - n + N_D^+ - N_A^-)$$
   Where:
   $$n = n_i \exp\left(\frac{\psi - \phi_n}{V_t}\right), \quad p = n_i \exp\left(\frac{\phi_p - \psi}{V_t}\right)$$

2. **Electron and Hole Continuity Equations**:
   $$\nabla \cdot \mathbf{J}_n - q \frac{\partial n}{\partial t} = q R(n, p)$$
   $$\nabla \cdot \mathbf{J}_p + q \frac{\partial p}{\partial t} = -q R(n, p)$$
   Where $R(n, p)$ includes Shockley-Read-Hall (SRH) recombination and Auger recombination.

3. **Drift-Diffusion Current Fluxes**:
   $$\mathbf{J}_n = -q \mu_n n \nabla \psi + q D_n \nabla n = -q \mu_n n \nabla \phi_n$$
   $$\mathbf{J}_p = -q \mu_p p \nabla \psi - q D_p \nabla p = -q \mu_p p \nabla \phi_p$$

4. **Scharfetter-Gummel Spatial Discretization**:
   To prevent unphysical oscillations when drift dominates diffusion ($|\Delta \psi| > 2 V_t$), the current flux between adjacent spatial mesh nodes $i$ and $j$ along grid edge $h_{ij}$ is discretized using the Bernoulli function $B(x) = \frac{x}{e^x - 1}$:
   $$J_{n, ij} = \frac{q D_{n, ij}}{h_{ij}} \left[ n_j B\left(\frac{\psi_j - \psi_i}{V_t}\right) - n_i B\left(\frac{\psi_i - \psi_j}{V_t}\right) \right]$$

5. **Terminal Current Integration into MNA**:
   Terminal currents for contact $k$ (Source, Drain, Gate, Bulk) are evaluated by integrating total flux over contact boundary $\partial \Omega_k$:
   $$I_k = \int_{\partial \Omega_k} \left( \mathbf{J}_n + \mathbf{J}_p + \epsilon \frac{\partial \mathbf{E}}{\partial t} \right) \cdot \hat{\mathbf{n}} \, dA$$
   This enables full TCAD transistors to be instantiated directly as multi-terminal sub-elements inside the global circuit MNA matrix.

---

### 6.2 High-Level Analytical Compact Models

While low-level TCAD meshes require solving $3 \times N_{mesh}$ equations per device, high-level compact models (BSIM3/4, BSIM-CMG, Gummel-Poon) collapse internal spatial distributions into closed-form physical equations:
- **Evaluation Speed**: Evaluates in $\approx 10 - 50\text{ ns}$ per Newton iteration ($O(1)$ constant time) vs. $1 - 100\text{ ms}$ for fine TCAD meshes ($O(N_{mesh}^{1.5})$).
- **Exact Analytical Jacobians**: Direct transconductances ($g_m, g_{ds}, g_{mb}$) and transcapacitances ($C_{ij}$) with zero PDE discretization overhead.
- **Circuit Scaling**: Essential for circuits with thousands to millions of transistors (e.g. operational amplifiers, SRAM arrays, DSP logic).

---

### 6.3 Automated Parameter Extraction & Surrogate Reduction

Phonon bridges low-level physical devices and high-level circuit simulation via automated compact model extraction:
1. **Virtual TCAD Sweep**: Automated DC and small-signal AC characterization sweeps ($I_D-V_{GS}, I_D-V_{DS}, C-V$) are executed on the low-level structural device.
2. **Non-Linear Optimization Fitting**: Parameter extraction engines fit core physical coefficients ($V_{th0}, \mu_0, v_{sat}, \lambda, t_{ox}, \eta_{DIBL}$) to match TCAD terminal responses.
3. **Surrogate Compact Model Emission**: The fitted parameters instantiate a drop-in analytical compact model that runs at full SPICE throughput without sacrificing the metallurgical accuracy of the original custom device.

---

### 6.4 Programmatic Fluent Builder Architecture

Both structural TCAD devices and high-level circuit modules share a unified, programmatic Rust API:

```rust
// 1. Low-level structural TCAD transistor synthesis
let custom_nmos = StructureBuilder::new("custom_nmos")
    .silicon_substrate(SubstrateType::PType, 1e16 /* Na cm^-3 */)
    .well(WellType::NWell, Rect::new(0.0, 0.0, 1.0e-6, 0.2e-6), 1e20 /* Nd */)
    .gate_oxide(3.0e-9 /* tox (m) */, Material::SiO2)
    .gate_polysilicon(0.18e-6 /* L */, 1.0e-6 /* W */)
    .ohmic_contact("drain", ContactLocation::TopRight)
    .ohmic_contact("source", ContactLocation::TopLeft)
    .mesh_density(MeshResolution::Fine)
    .build()?;

// 2. High-level programmatic circuit wiring
let mut circuit = CircuitBuilder::new();
circuit.wire("vdd", 1.8);
circuit.add_compact_mosfet("M1", "out", "in", "gnd", "gnd", &nmos_model);
circuit.add_structural_device("M2_custom", custom_nmos, &["out", "in", "gnd", "gnd"]);
```

---

## 7. Cryogenic Semiconductor Physics & Dopant Freeze-Out

At liquid Helium temperatures ($4.2\text{ K}$) and intermediate cryogenic stages ($77\text{ K}$), standard thermal approximations ($n \approx N_D^+$) fail due to **carrier freeze-out** and extreme Fermi-Dirac degeneracy.

### 7.1 Incomplete Dopant Ionization
Thermal energy ($k_B T \approx 0.36\text{ meV}$ at $4.2\text{ K}$) is much smaller than shallow dopant ionization energies ($E_d \approx 45\text{ meV}$ for Phosphorus, $E_a \approx 45\text{ meV}$ for Boron):
$$N_D^+ = \frac{N_D}{1 + g_D \exp\left( \frac{E_F - E_D}{k_B T} \right)}$$
Where $g_D = 2$ is the donor ground-state degeneracy factor. Carrier concentration drops by orders of magnitude unless doping exceeds the Mott transition metal-insulator threshold ($N_{Mott} \approx 3.5 \times 10^{18}\text{ cm}^{-3}$).

### 7.2 Fermi-Dirac Degeneracy Statistics
The Boltzmann approximation $\exp((E_F - E_C)/k_B T)$ is invalid in strong inversion. Phonon evaluates exact Fermi-Dirac integrals of order 1/2:
$$n = N_C \mathcal{F}_{1/2}\left( \frac{E_F - E_C}{k_B T} \right) = N_C \frac{2}{\sqrt{\pi}} \int_0^\infty \frac{x^{1/2}}{1 + \exp\left(x - \frac{E_F - E_C}{k_B T}\right)} dx$$
Inverted numerically using the high-precision rational approximation of Aymerich-Humet.

---

## 8. Quantum Ballistic Transport & Atomistic Channels

### 8.1 Non-Equilibrium Green's Function (NEGF)
For ultra-confined channels (sub-5nm Gate-All-Around nanosheets, CNTs, and single molecules):
$$\mathbf{G}^R(E) = \left[ (E + i\eta)\mathbf{I} - \mathbf{H} - \mathbf{\Sigma}_L(E) - \mathbf{\Sigma}_R(E) \right]^{-1}$$
- Spectral function: $\mathbf{A}(E) = i \left( \mathbf{G}^R(E) - \mathbf{G}^A(E) \right)$.
- Transmission function: $\mathcal{T}(E) = \text{Tr}\left[ \mathbf{\Gamma}_L(E) \mathbf{G}^R(E) \mathbf{\Gamma}_R(E) \mathbf{G}^A(E) \right]$.
- Landauer current: $I = \frac{2e}{h} \int \mathcal{T}(E) [f_L(E) - f_R(E)] dE$.

### 8.2 Carbon Nanotubes & 2D TMD Monolayers
- **Carbon Nanotubes (CNTs)**: Chiral vector $\mathbf{C}_h = n \mathbf{a}_1 + m \mathbf{a}_2$. Bandgap $E_g = \frac{2 a_{cc} t}{\sqrt{3} d_t}$ for semiconducting tubes ($(n-m) \not\equiv 0 \pmod 3$). Quantum contact resistance $R_Q = \frac{h}{4e^2} \approx 6.45\text{ k}\Omega$.
- **2D Transition Metal Dichalcogenides ($\text{MoS}_2, \text{WS}_2$)**: Direct bandgap ($E_g \approx 1.8\text{ eV}$ for monolayer $\text{MoS}_2$), atomistic thickness ($t = 0.65\text{ nm}$), immunity to short-channel DIBL due to high natural length scale $\lambda = \sqrt{\frac{\epsilon_{ch}}{\epsilon_{ox}} t_{ch} t_{ox}}$.

---

## 9. Non-Transistor Material Switches

### 9.1 Atomic Relays & Solid-Electrolyte ECM Cells
- **Nanoscale Atomic Relays**: Actuated by electrostatic Maxwell stress:
  $$V_{pull-in} = \sqrt{\frac{8 k_{eff} g_0^3}{27 \epsilon_0 A}}$$
  True zero subthreshold leakage ($S = 0\text{ mV/dec}$, $I_{off} = 0.0\text{ A}$), finite contact resistance determined by Sharvin quantum point contact $R_c = \frac{h}{2e^2 M}$.
- **Electrochemical Metallization (ECM) Cells**: Electric-field-driven metallic ion migration ($Ag^+, Cu^+$) through amorphous solid electrolytes ($\text{SiO}_2, \text{Al}_2\text{O}_3$), forming a conductive nanofilament with quantized conductance steps $G = n G_0$.

### 9.2 Molecular Quantum Interference Switches
- Exploits destructive quantum interference (QI) anti-resonances in alternant conjugated hydrocarbon rings.
- **Para- vs Meta-Benzene**: Meta-connectivity causes total cancellation of transmission at the Fermi energy due to $\pi$ phase difference between transmission pathways, yielding an intrinsic on/off ratio $> 80,000\times$.
- **Conformational Dihedral Twist**: Hopping overlap scales as $t(\theta) = t_0 \cos(\theta)$, providing mechanical switching with sub-100 meV energy dissipation ($< 40\text{ meV}$ per logic gate).

---

## 10. Spintronic Nanomagnets & Magnetoresistive Logic

### 10.1 Landau-Lifshitz-Gilbert-Slonczewski (LLGS) Solver
The 3D magnetization vector $\vec{m} = \vec{M} / M_s$ ($|\vec{m}| = 1$) satisfies:
$$\frac{d\vec{m}}{dt} = -\frac{\gamma}{1+\alpha^2} \left[ \vec{m} \times \vec{B}_{eff} + \alpha\,\vec{m} \times (\vec{m} \times \vec{B}_{eff}) \right] + \vec{\tau}_{STT} + \vec{\tau}_{SOT}$$
Where:
- $\vec{B}_{eff} = \mu_0 \left( \vec{H}_{ext} + \vec{H}_{ani} + \vec{H}_{demag} + \vec{H}_{dip} + \vec{H}_{th} \right)$.
- Demagnetization field: $\vec{H}_{demag} = -M_s (N_x m_x \hat{x} + N_y m_y \hat{y} + N_z m_z \hat{z})$, with $\text{Tr}(\mathbf{N}) = 1$.
- Slonczewski Spin-Transfer Torque: $\vec{\tau}_{STT} = \gamma a_J [\vec{m}_p - (\vec{m} \cdot \vec{m}_p)\vec{m}]$.

### 10.2 Dipolar Stray-Field Logic
- Free-space magnetic stray field: $\vec{H}_{dip}(\vec{r}) = \frac{1}{4\pi r^3} [3(\vec{\mu} \cdot \hat{r})\hat{r} - \vec{\mu}]$.
- Side-by-side nanomagnets align anti-ferromagnetically, executing an inherent logical NOT without wires or charge transport.
- Universal Majority-3 logic: $M(A, B, C) = AB + BC + AC$. Synthesizes a 1-bit full adder with only 16 nanomagnets (3 Majority + 2 Inverters), eliminating $100\%$ of static standby leakage ($P_{static} = 0.0\text{ W}$).

---

## 11. Superconducting Optoelectronic Neurons (SOEN)

Combines single-photon detection, cryogenic optical emission, and superconducting flux loops into sub-attojoule artificial neurons:
1. **SNSPD Hot-Spot Dynamics**: Photon absorption breaks Cooper pairs, forming a resistive hotspot $R_{hs}(t) = R_{norm} e^{-t/\tau_{th}}$ and diverting current $I_{div}(t)$ into an inductive loop with dissipation $E_{diss} \sim 1\text{ aJ}$.
2. **Cryogenic Optical Emitter**: Micro-LED or nanolaser modeled via cryogenic rate equations at $4\text{ K}$, emitting picosecond photon wavepackets when triggered by a driver pulse.
3. **Dendritic Flux Storage Loop**: Accumulates discrete magnetic flux quanta $\Phi = L I + n\Phi_0$ with leaky memory retention $\tau_{leak} = L / R_{damp} \approx 2.0\text{ ns}$.
4. **Somatic $2\pi$ Phase Slip**: When loop current pushes somatic Josephson junction past $I_c$, the junction undergoes a $2\pi$ phase slip, pulsing the optical emitter and self-resetting flux by $-\Phi_0$. Average synaptic energy dissipation: $\sim 3.5\text{ aJ}$ ($\approx 3.5\text{ fJ}$ wall-plug).

---

## 12. Topological Quantum Computing & Majorana Zero Modes

### 12.1 Tight-Binding Bogoliubov-de Gennes (BdG) Hamiltonian
In semiconductor nanowires (InAs, InSb) proximity-coupled to an s-wave superconductor (Al, Nb) with Rashba spin-orbit coupling and axial Zeeman splitting:
$$H_{BdG} = \sum_{i=1}^N \mathbf{H}_{ii} |i\rangle\langle i| + \sum_{i=1}^{N-1} \left( \mathbf{H}_{i, i+1} |i\rangle\langle i+1| + \mathbf{H}_{i, i+1}^T |i+1\rangle\langle i| \right)$$
In the Nambu spinor basis $\Psi_i = (c_{i\uparrow}, c_{i\downarrow}, c_{i\uparrow}^\dagger, c_{i\downarrow}^\dagger)^T$:
- On-site block $\mathbf{H}_{ii} = (2t - \mu) \tau_z + V_Z \sigma_x + \Delta \tau_x$.
- Hopping block $\mathbf{H}_{i, i+1} = -t \tau_z - i \alpha_R \tau_z \sigma_y$, where $\alpha_R = \frac{\alpha_{SO}}{2a}$.
- Particle-hole symmetry is exactly enforced: $\tau_x \mathbf{H} \tau_x = -\mathbf{H}$.

### 12.2 Topological Invariant & Boundary Localization
- **Phase Transition**: Occurs at $V_Z > V_{Z, c} = \sqrt{\Delta^2 + \mu^2}$.
- **Majorana Wavefunctions**: In the topological phase, bulk states remain gapped by $\Delta_{top}$, while an isolated pair of zero-energy bound states ($\gamma_1, \gamma_2$ with $E_0 \approx 0$) emerge, exponentially localized at the wire boundaries:
  $$|\psi_L(x)|^2 \propto \exp\left( -\frac{2x}{\xi_M} \right), \quad |\psi_R(x)|^2 \propto \exp\left( -\frac{2(L-x)}{\xi_M} \right)$$
  Where the Majorana coherence length is $\xi_M = \frac{\hbar v_F}{\Delta_{top}}$.

### 12.3 Resonant Andreev Reflection & Conductance Quantization
Tunneling from a normal lead into the Majorana wire end yields quantized zero-bias conductance:
$$G(0) = \frac{2 e^2}{h} \approx 77.48\,\mu\text{S}$$
At finite temperature, thermal broadening convolving with the Fermi-Dirac derivative reduces the peak height while preserving the integral.

### 12.4 4-Majorana Logical Qubits
A logical qubit is encoded in 4 Majoranas $(\gamma_1, \gamma_2, \gamma_3, \gamma_4)$ in the even fermion parity subspace ($P_{tot} = +1$):
- Logical basis: $|0_L\rangle$ ($P_{12}=+1, P_{34}=+1$), $|1_L\rangle$ ($P_{12}=-1, P_{34}=-1$).
- Clifford Phase Gate $S = \text{diag}(1, i)$ generated by braiding $B_{12} = \exp(\frac{\pi}{4}\gamma_2\gamma_1)$.
- Hadamard Gate $H$ generated by composite sequence $B_{12} B_{23} B_{12}$.
- Pauli flips $X_L, Z_L$ generated by double braids $B^2$.
- Hardware protection provides immunity against local charge and magnetic noise, with coherence lifetime limited only by non-equilibrium quasiparticle poisoning ($T_1^{topo} \approx 200\text{ ms}$).


