---
name: hardware_components
description: Physical modeling of realistic hardware components, passive parasitics (ESR, ESL), core saturation, non-ideal diodes, transmission lines, and thermal port interfaces.
---

# Physical Hardware Component Modeling

## 1. Non-Ideal Passive Components

In high-frequency power electronics and high-speed digital systems, idealized R, L, and C components do not exist. Phonon implements physically accurate multi-element subcircuit representations.

### 1.1 Non-Ideal Capacitor Model
A physical capacitor (electrolytic, ceramic MLCC, or tantalum) exhibits high-frequency parasitic resonances and dielectric absorption:

```
        +---[ ESR ]---[ ESL ]---+---[ C_nominal ]---+
        |                       |                   |
Pin 1 --+                       +---[ R_leakage ]---+-- Pin 2
                                |                   |
                                +---[ R_da ]-[ C_da ]+
```
- **ESR (Equivalent Series Resistance)**: Ohmic losses in electrodes and electrolyte; dominates dissipation at self-resonant frequency (SRF): $P_{diss} = I_{rms}^2 \cdot \text{ESR}$.
- **ESL (Equivalent Series Inductance)**: Parasitic lead and plate loop inductance; causes capacitor impedance to become inductive above the SRF:
  $$f_{SRF} = \frac{1}{2\pi \sqrt{C \cdot \text{ESL}}}$$
- **$R_{leakage}$**: High resistance ($> 10^8\,\Omega$) representing DC dielectric insulation breakdown.
- **Dielectric Absorption ($R_{da}, C_{da}$)**: Models residual voltage recovery after rapid discharge (memory effect) due to dipole relaxation in ferroelectric ceramic dielectrics (e.g., X7R, Y5V).

### 1.2 Non-Ideal Inductor & Core Saturation
A physical power inductor is limited by copper winding resistance, distributed capacitance, and non-linear magnetic flux saturation:
- **DC Resistance (DCR)**: Copper wire resistance with temperature coefficient:
  $$R_{cu}(T) = R_{cu0} \left[ 1 + 0.00393 (T - 293.15) \right]$$
- **EPC (Equivalent Parallel Capacitance)**: Turn-to-turn winding capacitance creating parallel resonance.
- **Magnetic Core Saturation**:
  Phonon models the non-linear $B-H$ curve of ferrite and iron-powder cores using a continuous hyperbolic tangent formulation:
  $$B(H) = B_{sat} \tanh\left( \frac{\mu_r \mu_0 H}{B_{sat}} \right)$$
  The differential inductance $L_{diff}(i)$ drops sharply as current approaches saturation:
  $$L_{diff}(i) = N \frac{d\Phi}{di} = N A_e \frac{dB}{dH} \frac{dH}{di} = L_0 \cdot \text{sech}^2\left( \frac{i}{I_{sat}} \right)$$

---

## 2. Non-Ideal Diode Modeling

### 2.1 Complete Current Equation
$$\begin{aligned}
I_D = I_S(T) \left[ \exp\left( \frac{V_D - I_D R_S}{N V_t} \right) - 1 \right] &- I_{BV}(T) \left[ \exp\left( -\frac{V_D + V_{BV}}{N_V V_t} \right) \right]
\end{aligned}$$
Where:
- $R_S$: Bulk semiconductor and contact series resistance.
- $V_{BV}$: Reverse breakdown voltage (Zener / Avalanche).
- $I_{BV}$: Current at onset of breakdown.

### 2.2 Charge-Conservative Junction & Diffusion Capacitance
The dynamic charge stored in the diode is split into depletion (space-charge) and diffusion (minority carrier) storage:

$$Q_D(V_D) = Q_{dep}(V_D) + Q_{diff}(V_D)$$

#### 1. Depletion Charge ($Q_{dep}$):
For $V_D < FC \cdot V_J$ (where $FC \approx 0.5$ is the forward bias coefficient):
$$Q_{dep}(V_D) = \frac{C_{J0} V_J}{1 - M} \left[ 1 - \left( 1 - \frac{V_D}{V_J} \right)^{1-M} \right]$$
For $V_D \ge FC \cdot V_J$, a linearized quadratic expansion guarantees continuous first and second derivatives without mathematical divergence at $V_D = V_J$.

#### 2. Diffusion Charge ($Q_{diff}$):
Minority carrier transit time $\tau_T$ stores charge under forward bias:
$$Q_{diff}(V_D) = \tau_T \cdot I_D(V_D)$$
This accurately models **reverse recovery time ($t_{rr}$)**: when voltage reverses rapidly, current spikes negatively until stored minority carriers recombine.

---

## 3. Distributed Transmission Lines

For high-speed digital interconnects and RF buses, signals propagate as electromagnetic waves governed by the Telegrapher's partial differential equations:

$$\frac{\partial v(x, t)}{\partial x} = -R' i(x, t) - L' \frac{\partial i(x, t)}{\partial t}$$
$$\frac{\partial i(x, t)}{\partial x} = -G' v(x, t) - C' \frac{\partial v(x, t)}{\partial t}$$

### 3.1 Method of Characteristics (Lossless / Low-Loss)
Phonon transforms the PDE along characteristic curves into decoupled boundary equations:
- Characteristic Impedance: $Z_0 = \sqrt{\frac{L'}{C'}}$
- Propagation Delay: $\tau = d \sqrt{L' C'}$

At Port 1 ($x = 0$) and Port 2 ($x = d$):
$$v_1(t) - Z_0 i_1(t) = v_2(t - \tau) + Z_0 i_2(t - \tau)$$
$$v_2(t) - Z_0 i_2(t) = v_1(t - \tau) + Z_0 i_1(t - \tau)$$

The solver stores a rolling historical ring buffer of past terminal states to evaluate the delayed history terms with cubic Hermite spline interpolation.

---

## 4. Multi-Physics Thermal Port Interfaces

Every component in Phonon exposes a standardized thermal port interface to facilitate electro-thermal feedback:

```rust
pub trait Component {
    /// Returns the number of electrical terminals
    fn electrical_pins(&self) -> usize;

    /// Returns the thermal node index in the thermal grid (if any)
    fn thermal_node(&self) -> Option<usize>;

    /// Evaluates instantaneous power dissipation: P = sum(V_pin * I_pin)
    fn evaluate_power_dissipation(&self, v_pins: &[f64], i_pins: &[f64]) -> f64;

    /// Evaluates companion matrix stamps at local temperature T_local
    fn stamp(
        &self,
        v_pins: &[f64],
        t_local: f64,
        jacobian: &mut SparseMatrix,
        rhs: &mut [f64],
    );
}
```

This ensures seamless coupling between active silicon channels, passive dissipators, and the spatial thermal grid.

---

## 5. Superconducting Josephson Junctions & RSFQ Logic Components

### 5.1 RCSJ Equivalent Circuit & Dynamics
Superconducting circuits operate via the Josephson effect across superconductor-insulator-superconductor (SIS) barriers:
- Phase-voltage evolution: $\frac{d\phi}{dt} = \frac{2\pi}{\Phi_0} V(t)$.
- Supercurrent: $I_s = I_c \sin(\phi)$.
- Stewart-McCumber parameter: $\beta_c = \frac{2\pi I_c R_n^2 C_j}{\Phi_0}$. Overdamped junctions ($\beta_c \approx 1.0$) eliminate hysteretic latching.
- Subgap resistance $R_{sg} \approx 10 R_n$ models quasiparticle tunneling below the BCS superconducting gap $2\Delta(T)$.

### 5.2 RSFQ Logic Primitives
- **RSFQ D-Flip-Flop (`RsfqDff`)**: Stores a single flux quantum in a superconducting quantizing loop ($L I = \Phi_0$). Clock pulse triggers non-destructive readout and resets loop state.
- **RSFQ Concurrence / AND Gate (`RsfqAnd`)**: Coincidence detector emitting an output SFQ pulse only when both inputs arrive within the same clock period.
- **RSFQ Inverter (`RsfqInverter`)**: Timed negation emitting an SFQ pulse on clock tick only if no data pulse arrived.
- **Josephson Transmission Line (`RsfqJtl`)**: Low-jitter soliton delay stage ($\approx 2 - 5\text{ ps}$ propagation delay).

---

## 6. Cryogenic Optoelectronics & Dielectric Waveguides

### 6.1 Superconducting Nanowire Single-Photon Detector (SNSPD)
- Current-biased superconducting nanowire ($I_{bias} \approx 0.95 I_c$) operating at $4.2\text{ K}$.
- Hot-spot resistance relaxation: $R_{hs}(t) = R_{norm} \exp(-t / \tau_{th})$, with $\tau_{th} \approx 150\text{ ps}$.
- Current diversion: $I_{div}(t) = I_{bias} \frac{R_{hs}(t)}{R_{hs}(t) + R_{load}} (1 - e^{-t/\tau_{rise}})$.
- Diverted magnetic flux: $\Delta \Phi = \int V_{load}(t) dt \sim \Phi_0$.
- Microscopic Joule dissipation: strictly in the attojoule realm ($E_{diss} \sim 1\text{ aJ}$).

### 6.2 Cryogenic Semiconductor Optical Emitter
- Micro-LED or low-threshold nanolaser at $4\text{ K}$ driven by Josephson junction driver pulses.
- Rate equations coupling electron carrier density $N(t)$ and photon density $S(t)$:
  $$\frac{dN}{dt} = \frac{\eta_d I_{in}}{q V_{act}} - \frac{N}{\tau_n} - g_0 (N - N_{tr}) S$$
  $$\frac{dS}{dt} = \Gamma g_0 (N - N_{tr}) S - \frac{S}{\tau_p} + \Gamma \beta_{sp} \frac{N}{\tau_n}$$
- Differential quantum efficiency: $\eta_d > 85\%$. Optical pulse energy: $\sim 1 - 5\text{ aJ}$ per spike.

### 6.3 Dielectric Optical Waveguides & Zero-Crosstalk Interconnects
- Silicon Nitride ($\text{Si}_3\text{N}_4$) or Silicon-on-Insulator (SOI) dielectric waveguides at $4\text{ K}$.
- Propagation speed: $v = c / n_{eff}$ ($n_{eff} \approx 2.0$, delay $\approx 3.33\text{ ps/mm}$).
- Optical crosstalk isolation: $> 75\text{ dB}$ (eliminates capacitive/inductive coupling to sensitive qubit loops).
- High optical fanout: supports splitting optical pulse energy to $> 1000$ synaptic destinations without RC charging delay.

---

## 7. Non-Volatile Memristors & In-Memory Computing

### 7.1 Filamentary Resistive RAM (RRAM)
- Conductance state governed by oxygen vacancy migration in transition metal oxides ($\text{HfO}_x, \text{TaO}_x$):
  $$\frac{dg}{dt} = v_0 \exp\left(-\frac{E_a - \alpha_v q V}{k_B T}\right)$$
- Non-linear conduction via Poole-Frenkel emission and trap-assisted tunneling.

### 7.2 Phase-Change Memory (PCM)
- Reversible phase transition in chalcogenide alloys ($\text{Ge}_2\text{Sb}_2\text{Te}_5$, GST).
- Amorphization (RESET) via fast thermal quenching above melting temperature ($T_m \approx 600^\circ\text{C}$).
- Crystallization (SET) via prolonged Joule heating above glass transition ($T_g \approx 300^\circ\text{C}$).
- Ovonic Threshold Switching (OTS) enables low-voltage snapback in amorphous phase.

### 7.3 Ferroelectric Field-Effect Transistors (FeFET)
- Multi-domain spontaneous polarization in doped hafnia ($\text{Hf}_{0.5}\text{Zr}_{0.5}\text{O}_2$, HZO) modeled via the time-dependent Landau-Khalatnikov equation:
  $$\rho \frac{\partial P}{\partial t} + \alpha_F P + \beta_F P^3 + \gamma_F P^5 - \kappa_F \nabla^2 P = E_{ext}$$

---

## 8. Integrated Silicon Photonics Circuit Elements

### 8.1 Micro-Ring Resonators (MRR)
- All-pass and add-drop configurations:
  $$\mathcal{T}_{thru}(\lambda) = \frac{r_1^2 - 2 r_1 r_2 a e^{i \phi} + r_2^2 a^2}{1 - 2 r_1 r_2 a e^{i \phi} + (r_1 r_2 a)^2}$$
- Free Spectral Range: $\text{FSR} = \frac{\lambda_0^2}{n_g L}$. Thermo-optic resonance tuning: $\frac{d\lambda}{dT} = \frac{\lambda_0}{n_g} \frac{dn}{dT}$.

### 8.2 Mach-Zehnder Modulators (MZI)
- Electro-optic modulation via carrier plasma dispersion (Soref-Bennett equations):
  $$\Delta n = -8.8 \times 10^{-22} \Delta N_e - 8.5 \times 10^{-18} (\Delta N_h)^{0.8}$$
  $$\Delta \alpha = 8.5 \times 10^{-18} \Delta N_e + 6.0 \times 10^{-18} \Delta N_h$$
- Half-wave voltage $V_\pi$ and dynamic extinction ratio $> 30\text{ dB}$.
