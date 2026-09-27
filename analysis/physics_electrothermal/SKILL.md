---
name: physics_electrothermal
description: In-depth engineering specification and physical equations for coupled electro-thermal simulation, Fourier heat diffusion, Cauer/Foster networks, and semiconductor self-heating.
---

# Coupled Electro-Thermal Physics & Modeling

## 1. Physical Principles & Coupled Equations

In advanced semiconductor devices and power electronics, electrical currents and temperatures are mutually coupled. Current flow induces Joule dissipation, which elevates device temperature, which in turn degrades carrier mobility, alters threshold voltages, shifts PN-junction built-in potentials, and can trigger catastrophic thermal runaway.

```
       +---------------------------------------------+
       |             Electrical Domain               |
       |  G x + C dx/dt + f(x, T) = u(t)             |
       +---------------------------------------------+
                       |             ^
        Joule Power    |             |  Modulates:
        P_diss = V * I |             |  - V_th(T)
                       v             |  - Mobility mu(T)
       +-----------------------------+               |
       |              Thermal Domain                 |
       |  C_th dT/dt + G_th T = P_diss + G_th T_amb  |
       +---------------------------------------------+
```

### 1.1 Fourier Heat Conduction in Semiconductor Dies
Within a 3D silicon substrate $\Omega \subset \mathbb{R}^3$, the heat diffusion equation is:

$$\rho c_p(T) \frac{\partial T(\mathbf{r}, t)}{\partial t} - \nabla \cdot \left(\kappa(T) \nabla T(\mathbf{r}, t)\right) = p_{gen}(\mathbf{r}, t)$$

Where:
- $\rho$: Mass density of Silicon ($2330 \text{ kg/m}^3$) or Silicon Carbide ($3210 \text{ kg/m}^3$).
- $c_p(T)$: Temperature-dependent specific heat capacity:
  $$c_p(T) = c_{p0} \left[ 1 + a_1 \left( \frac{T - 300}{300} \right) \right]$$
  For Silicon at $300\text{ K}$, $c_p \approx 712 \text{ J/(kg}\cdot\text{K)}$.
- $\kappa(T)$: Non-linear thermal conductivity:
  $$\kappa(T) = \kappa_0 \left( \frac{T}{300\text{ K}} \right)^{-\alpha_\kappa}$$
  For pure Silicon, $\kappa_0 \approx 148 \text{ W/(m}\cdot\text{K)}$ with $\alpha_\kappa \approx 1.33$.
- $p_{gen}(\mathbf{r}, t)$: Volumetric heat generation source ($W/m^3$), calculated directly from active channel and junction currents:
  $$p_{gen}(\mathbf{r}, t) = \mathbf{J}(\mathbf{r}, t) \cdot \mathbf{E}(\mathbf{r}, t)$$

### 1.2 Boundary Conditions
Three types of thermal boundary conditions must be enforced:
1. **Dirichlet Boundary (Ideal Heatsink)**:
   $$T(\mathbf{r}, t)\Big|_{\Gamma_D} = T_{sink}$$
2. **Neumann Boundary (Adiabatic / Insulated Surfaces)**:
   $$-\kappa(T) \frac{\partial T}{\partial \mathbf{n}}\Big|_{\Gamma_N} = 0$$
3. **Robin Boundary (Convective / Radiative Cooling)**:
   $$-\kappa(T) \frac{\partial T}{\partial \mathbf{n}}\Big|_{\Gamma_R} = h_{conv} \left( T - T_{ambient} \right) + \epsilon \sigma_{SB} \left( T^4 - T_{ambient}^4 \right)$$
   Where $h_{conv}$ is the convection coefficient ($\text{W}/(\text{m}^2\cdot\text{K})$), $\epsilon$ is emissivity, and $\sigma_{SB} = 5.67037 \times 10^{-8} \text{ W/(m}^2\cdot\text{K}^4)$ is the Stefan-Boltzmann constant.

---

## 2. Lumped Thermal Networks: Cauer vs. Foster Formulations

For fast circuit simulation without full 3D PDE mesh overhead, thermal transport is mapped into linear dynamical networks.

### 2.1 Cauer Ladder Model (Physically Meaningful)
The Cauer network represents physical thermal slices through die, leadframe, packaging, and heatsink:
```
P_in o----+--------[ R_th1 ]--------+--------[ R_th2 ]--------+---> T_ambient
          |                         |                         |
       [ C_th1 ]                 [ C_th2 ]                 [ C_th3 ]
          |                         |                         |
         GND                       GND                       GND
     (Junction)                  (Case)                   (Heatsink)
```
- **Physical Invariance**: Each capacitor $C_{th, i} = \rho c_p V_i$ corresponds to the actual thermal mass of volume $V_i$. Each resistor $R_{th, i} = \frac{L_i}{\kappa A_i}$ corresponds to the conduction path length.
- **Node Temperatures**: Every internal node voltage directly represents the physical temperature of that physical layer.
- **State Equations**:
  $$\mathbf{C}_{th} \frac{d\mathbf{T}}{dt} + \mathbf{A}_{cauer}^T \mathbf{G}_{th} \mathbf{A}_{cauer} \mathbf{T} = \mathbf{P}_{diss}(t)$$

### 2.2 Foster Ladder Model (Measurement Fit)
The Foster model is derived from multi-exponential curve fitting of transient thermal impedance curves $Z_{th}(t)$:
```
P_in o---[ R1 || C1 ]---[ R2 || C2 ]---[ R3 || C3 ]---> T_ambient
```
- **Limitation**: Internal nodes have **no physical meaning**. A Foster network **cannot** be truncated, tapped for intermediate temperatures, or coupled to spatial heat spreading without mathematical transformation into a Cauer equivalent via continued fraction expansion.

---

## 3. Electro-Thermal Feedback on Semiconductor Physics

### 3.1 Carrier Mobility Degradation
Lattice phonon scattering increases vigorously with temperature:
$$\mu_n(T) = \mu_{n0} \left( \frac{T}{T_0} \right)^{-\eta_n}, \quad \mu_p(T) = \mu_{p0} \left( \frac{T}{T_0} \right)^{-\eta_p}$$
Typical exponents: $\eta_n \approx 1.5 - 2.1$, $\eta_p \approx 1.8 - 2.3$. This causes MOSFET on-resistance $R_{DS(on)}$ to increase with temperature, providing positive thermal stability in power MOSFETs.

### 3.2 Threshold Voltage Drift
Thermal expansion of the Fermi level and intrinsic carrier concentration reduces the required band-bending for channel inversion:
$$V_{th}(T) = V_{th}(T_0) - \alpha_{Vth} (T - T_0)$$
Where $\alpha_{Vth} \approx 1.0 - 2.5 \text{ mV/K}$. A dropping $V_{th}$ at elevated temperatures increases drain current at low gate bias, which can provoke thermal runaway in linear-mode operation.

### 3.3 PN Junction Saturation Current & Thermal Runaway
In bipolar junctions and diodes, the reverse saturation current $I_S(T)$ scales exponentially:
$$I_S(T) = I_S(T_0) \left( \frac{T}{T_0} \right)^{\frac{XTI}{N}} \exp\left( -\frac{q E_g(T)}{N k_B T} \left( 1 - \frac{T}{T_0} \right) \right)$$
For silicon, $I_S$ approximately doubles every $8 - 10 \text{ K}$. If current is held constant, diode forward voltage drops at $-2.0 \text{ mV/K}$.

---

## 4. Multi-Physics Solver Algorithms in Rust

### 4.1 Monolithic Jacobian Stamp
When solving electro-thermal systems monolithically in `phonon-solver`:
```rust
// Unified Jacobian matrix block structure
// [ J_ee   J_et ] [ delta_V ] = [ -F_elec ]
// [ J_te   J_tt ] [ delta_T ]   [ -F_therm ]
```
Where:
- $J_{ee} = \frac{\partial \mathbf{\Phi}_{elec}}{\partial \mathbf{V}}$: Standard electrical MNA Jacobian.
- $J_{et} = \frac{\partial \mathbf{\Phi}_{elec}}{\partial \mathbf{T}}$: Derivative of electrical branch currents with respect to temperature. For a diode: $\frac{\partial I_D}{\partial T} = \frac{I_D + I_S}{V_t} \left( \frac{V_D}{T} - \frac{E_g}{q T} \right)$.
- $J_{te} = -\frac{\partial P_{diss}}{\partial \mathbf{V}}$: Derivative of power dissipation with respect to voltage. For a two-terminal device: $\frac{\partial (V \cdot I)}{\partial V} = I + V \frac{\partial I}{\partial V}$.
- $J_{tt} = \mathbf{G}_{th} + \frac{\mathbf{C}_{th}}{\Delta t} - \frac{\partial P_{diss}}{\partial \mathbf{T}}$: Thermal conduction matrix augmented by dynamic capacitance and power-temperature derivatives.

### 4.2 Waveform Relaxation with Multirate Time Stepping
Because thermal time constants ($\tau_{th} \sim 10^{-3}\text{ s}$) are often 6 orders of magnitude larger than electrical switching speeds ($\tau_{elec} \sim 10^{-9}\text{ s}$), Phonon implements multirate subcycling:
1. Advance electrical states across fine time steps $\delta t$ keeping thermal state frozen at $T_k$.
2. Accumulate integrated energy dissipation: $\Delta E = \int_{t_k}^{t_k + \Delta T} P_{diss}(t) dt$.
3. Compute average power $\bar{P} = \frac{\Delta E}{\Delta T}$ and advance the thermal grid by coarse time step $\Delta T$.
4. Check convergence: if $\max |\Delta T_{node}| > \epsilon_{relax}$, backtrack and refine subcycling.

---

## 5. Cryogenic Thermal Regimes & Dilution Refrigerator Heat Budgets

In superconducting quantum computing and cryogenic RSFQ/SOEN coprocessing, circuits operate across distinct temperature stages inside a dilution refrigerator:

```
+-------------------------------------------------------------------------+
| Stage           | Nominal Temp | Cooling Power  | Dominant Heat Leak     |
|:----------------|:-------------|:---------------|:-----------------------|
| Room Temp (RT)  | 300 K        | Unlimited      | Ambient Convection     |
| 50 K Shield     | 50 K         | ~ 10 - 40 W    | Thermal Radiation      |
| 4 K Plate       | 4.2 K        | ~ 0.5 - 1.5 W  | Cu/SS Braids, Wiring   |
| Still Stage     | 800 mK       | ~ 50 - 150 mW  | He-3 Circulation       |
| Cold Plate      | 100 mK       | ~ 10 - 30 µW   | Coaxial Conductance    |
| Mixing Chamber  | 10 - 15 mK   | ~ 1 - 5 µW     | Attenuator Dissipation |
+-------------------------------------------------------------------------+
```

### 5.1 Thermal Conduction & The Wiedemann-Franz Law
For solid thermal links, Fourier conduction across temperature gradient $T_{hot} \to T_{cold}$ is:
$$Q_{cond} = \frac{A}{L} \int_{T_{cold}}^{T_{hot}} \kappa(T) \, dT$$

For metallic wires (copper, brass, cupronickel, stainless steel), electrical conductivity $\sigma(T)$ and thermal conductivity $\kappa(T)$ are coupled by the Wiedemann-Franz Law:
$$\frac{\kappa(T)}{\sigma(T)} = L_{Lorenz} T, \quad L_{Lorenz} \approx 2.44 \times 10^{-8} \text{ W}\cdot\Omega/\text{K}^2$$

- **Coaxial Cable Penalty**: A standard RG-405 semi-rigid copper coaxial line carries $\approx 2.5\text{ mW}$ of thermal conduction from 4K to the 100mK stage, quickly overwhelming the $20\,\mu\text{W}$ cooling power if hundreds of qubits are addressed.
- **Optical Ribbon Solution**: Multi-mode silica optical fibers have thermal conductivity $\kappa_{silica}(T) \approx 0.05 \text{ W/(m}\cdot\text{K)}$ at 4K, reducing conductive heat load to $< 0.8\,\mu\text{W}$ per link ($> 3000\times$ reduction).

### 5.2 Passive RF Attenuator Dissipation
Microwave control lines addressing superconducting qubits (5–6 GHz) require cascaded attenuators (e.g., 20 dB at 4K, 10 dB at 100mK, 30 dB at 10mK) to attenuate 300K thermal Johnson-Nyquist noise ($P_{noise} = k_B T \Delta f$).
The power dissipated in each cryogenic attenuator is:
$$P_{diss, att} = P_{in} \left(1 - 10^{-A_{dB} / 10}\right)$$
For continuous drive pulses ($P_{in} \sim 10\text{ dBm} = 10\text{ mW}$), the 4K stage must sink $\sim 9.9\text{ mW}$ per line.

---

## 6. Nanoscale Non-Equilibrium Heat & Hotspot Dynamics

### 6.1 Acoustic Mismatch & Kapitza Boundary Resistance
At metal-dielectric or superconductor-substrate interfaces below 10K, phonon acoustic mismatch creates an interfacial temperature discontinuity $\Delta T$:
$$Q = \frac{A}{R_K} \left( T_1^4 - T_2^4 \right) \approx 4 \frac{A}{R_K} T^3 \Delta T$$
Where $R_K$ is the Kapitza boundary resistance ($\text{m}^2\cdot\text{K}^4/\text{W}$). Because thermal conductance scales as $T^3$, heat extraction from nanoscale devices into the substrate drops exponentially as $T \to 0$.

### 6.2 Two-Temperature Model (2TM) for SNSPD Hotspots
In superconducting nanowire single-photon detectors (SNSPDs), absorption of a single photon creates a localized non-equilibrium hotspot where the electron temperature $T_e$ decouples from the phonon temperature $T_{ph}$:

$$\begin{aligned}
C_e(T_e) \frac{\partial T_e}{\partial t} &= \nabla \cdot \left(\kappa_e \nabla T_e\right) - G_{e-ph} \left(T_e^n - T_{ph}^n\right) + \frac{J^2}{\sigma_n} \\
C_{ph}(T_{ph}) \frac{\partial T_{ph}}{\partial t} &= \nabla \cdot \left(\kappa_{ph} \nabla T_{ph}\right) + G_{e-ph} \left(T_e^n - T_{ph}^n\right) - \frac{C_{ph}}{\tau_{esc}} (T_{ph} - T_{sub})
\end{aligned}$$

Where:
- $G_{e-ph}$: Electron-phonon thermal coupling constant ($\text{W}/(\text{m}^3\cdot\text{K}^n)$, with $n \approx 3 - 5$ for disordered thin films).
- $\tau_{esc}$: Phonon escape time from the thin film into the dielectric substrate ($\approx 50 - 200\text{ ps}$).
- $J^2 / \sigma_n$: Local Joule heating in the resistive hotspot.

Once $T_e > T_c$, superconductivity is suppressed across the nanowire cross-section, diverting the bias current $I_{bias}$ into the readout load ($50\,\Omega$ or RSFQ gate) within tens of picoseconds.

---

## 7. Thermal Stability & Runaway Criterion

In power semiconductors and cryogenic Josephson circuits, thermal runaway occurs when internal heat generation increases faster with temperature than the package or cooling system can extract it:

$$\frac{d P_{gen}}{d T} > \frac{d P_{diss}}{d T} = \frac{1}{R_{th}}$$

1. **Bipolar Devices**: Forward voltage decreases with temperature ($\frac{d V_F}{d T} < 0$), causing current crowding and destructive runaway in parallel BJT fingers.
2. **Cryogenic RSFQ Circuits**: If local dissipation heats junctions above $T_c$ ($9.2\text{ K}$ for Nb), critical current $I_c(T) \to 0$, causing full latching into the normal resistive state and catastrophic loss of flux quantization.

