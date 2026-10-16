# Tier 4: Cryogenic Cryo-CMOS Physics

Tier 4 extends semiconductor modeling down to deep cryogenic temperatures (4.2 K liquid Helium down to sub-100 mK dilution refrigerator stages), essential for quantum computing qubit control ASICs, cryogenic low-noise amplifiers (LNAs), and deep-space autonomous sensor systems.

::: tip Cryogenic Physical Regimes
- **Incomplete Dopant Ionization & Freeze-Out**: Resolves non-linear reductions in mobile carrier concentration.
- **Subthreshold Swing Steepening**: Captures steep sub-15 mV/decade subthreshold conduction at 4.2K.
- **Bandgap Widening & Mobility Surge**: Models suppression of acoustic phonon scattering and Varshni bandgap expansion.
- **Non-Ohmic Contact Resistances**: Calibrates Schottky barrier tunneling and freeze-out contact resistance.
:::

## Incomplete Dopant Ionization Formulation

At room temperature (300 K), thermal energy $k_B T \approx 25.9\,\text{meV}$ is significantly greater than donor binding energies ($\sim 45\,\text{meV}$ for Phosphorus in Silicon), resulting in complete ionization ($n \approx N_D$).

At cryogenic temperatures (4.2 K), $k_B T \approx 0.362\,\text{meV}$, causing the vast majority of dopants to remain neutral (frozen out). Phonon evaluates the self-consistent Fermi-Dirac carrier statistics:

```
N_D^+ = N_D / (1 + g_D * exp((E_F - E_D) / (k_B * T)))
N_A^- = N_A / (1 + g_A * exp((E_A - E_F) / (k_B * T)))
```

Where:
- $g_D = 2$ and $g_A = 4$ are the donor and acceptor degeneracy factors.
- $E_D$ and $E_A$ are the dopant ground-state energy levels relative to conduction and valence band edges.
- $E_F$ is the self-consistent Fermi level determined via Newton-Raphson iteration of the charge neutrality condition.

## Cryo-CMOS Empirical Metrics Across Temperature

| Parameter | 300 K (Room) | 77 K (Liquid N2) | 4.2 K (Liquid He) | Unit |
| :--- | :--- | :--- | :--- | :--- |
| **Thermal Energy ($k_B T$)** | 25.86 | 6.64 | 0.362 | meV |
| **Subthreshold Swing ($S$)** | 68.2 | 18.5 | 12.8 | mV/decade |
| **Silicon Bandgap ($E_g$)** | 1.124 | 1.162 | 1.170 | eV |
| **Ionized Fraction ($N_D^+ / N_D$)** | 99.8 | 62.4 | 0.412 | % |
| **Low-Field Electron Mobility** | 420 | 1,480 | 2,860 | cm^2/V-s |
| **Threshold Voltage ($V_{th}$)** | 0.380 | 0.485 | 0.542 | V |

::: info Numerical Convergence Guarantee
Legacy SPICE simulators evaluate carrier density using the classical Boltzmann approximation $n = n_i \exp(qV/k_BT)$, leading to catastrophic floating-point overflow when $T \to 4.2\,\text{K}$. Phonon employs rational Chebyshev approximations of the Fermi-Dirac integral $F_{1/2}(\eta)$, ensuring unconditional stability down to 100 mK.
:::

## Cryogenic SPICE Model Netlist

```ini
* 4.2K Cryo-CMOS Readout LNA for Qubit Control
.title 4.2K Cryogenic Low-Noise Readout Stage

.param T_CRYO = 4.200
.param VDD_VAL = 1.000

Vdd vdd 0 DC {VDD_VAL}
Vgate in 0 DC 0.650 AC 1.0m

* Cryogenic MOSFET with Incomplete Ionization Flag
M_lna out in 0 0 PHONON_CRYO_NMOS W=10u L=28n T_OP={T_CRYO}
R_load vdd out 2.4k

.model PHONON_CRYO_NMOS NMOS (
+ LEVEL=54 VERSION=4.8.2
+ TCAD_CRYO=1
+ E_DONOR=45m
+ G_DONOR=2.0
+ S_MIN=12.0m
+ VTH0_CRYO=0.540
+ U0_CRYO=0.280
)

.dc Vgate 0.0 1.0 10mV
.print dc V(out) I(M_lna)
.end
```

## Running Cryogenic Analyses

```bash
# Verify netlist topology and cryogenic model parameter ranges
phonon validate cryo_lna.cir

# Execute DC sweep to inspect cryogenic subthreshold slope
phonon run cryo_lna.cir --format json --output cryo_results.json
```
