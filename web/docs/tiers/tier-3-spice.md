# Tier 3: SPICE Circuit Electronics

Tier 3 delivers an industry-grade electronic circuit simulation engine powered by high-performance Modified Nodal Analysis (MNA), sparse matrix factorization, and physical semiconductor compact models.

::: tip Key Architectural Capabilities
- **64-Bit KLU Sparse LU Decomposition**: Rapid linear system solution using Gilbert-Peierls fill-in minimization.
- **BSIM4 Unified Overdrive Model**: Industry-accurate drain current, subthreshold slope, and velocity saturation.
- **Ward-Dutton Charge Conservation**: Physically consistent terminal charge derivatives preventing artificial energy generation.
- **Adaptive TR-BDF2 Integration**: Second-order composite integration unconditionally suppressing inductive switching spikes.
:::

## Mathematical Formulation of MNA

Phonon transforms electrical networks of arbitrary topology into differential algebraic equations (DAEs):

```
G * x(t) + C * d/dt[x(t)] = i(t)
```

Where:
- `G` is the static conductance matrix containing linear resistors, dependent sources, and linearized transistor transconductances.
- `C` is the dynamic capacitance matrix containing linear capacitors, inductors, and non-linear semiconductor charges.
- `x(t)` is the unknown node voltage and independent branch current vector.
- `i(t)` represents independent voltage and current excitations.

### Non-Linear Newton-Raphson Iteration

For non-linear circuits, Phonon evaluates the linearized Jacobian system at each Newton iteration `k`:

```
J(x^(k)) * delta_x^(k+1) = - (G * x^(k) + C * d/dt[x^(k)] - i(t))
```

Damped updates ensure global convergence across stiff exponential regions:

```
x^(k+1) = x^(k) + omega * delta_x^(k+1)   (where omega in (0, 1] is dynamically adapted)
```

## Supported Device Compact Models

| Device Type | SPICE Key | Model Standard | Primary Mathematical Formulation |
| :--- | :--- | :--- | :--- |
| **MOSFET** | `M` | BSIM4 v4.8.2 | Velocity saturation, DIBL, gate tunneling, halo doping |
| **BJT** | `Q` | Gummel-Poon | Early effect, base push-out, temperature tracking |
| **Diode** | `D` | Shockley Ideal | Diffusion and depletion capacitance, reverse breakdown |
| **Transmission Line** | `T`, `O` | Lossy / W-Element | Frequency-dependent attenuation and skin effect dispersion |
| **Mutual Inductors** | `K` | Linear Magnetic | Coupled flux linkages with symmetric mutual inductance |

## BSIM4 Core Parameters Reference

| Parameter | Default | Unit | Description |
| :--- | :--- | :--- | :--- |
| `VTH0` | 0.40 | V | Zero-bias long-channel threshold voltage |
| `U0` | 0.045 | m^2/V-s | Low-field electron mobility at ambient 300K |
| `TOXE` | 1.8e-9 | m | Electrical gate equivalent oxide thickness |
| `VSAT` | 8.0e4 | m/s | Saturation velocity in inversion channel |
| `DROUT` | 0.56 | - | DIBL parameter coefficient for channel length modulation |
| `KT1` | -0.11 | V | Temperature coefficient for threshold voltage shift |

::: info Charge Conservation Directive
Phonon enforces strict Ward-Dutton charge conservation across all transistor capacitance formulations. Intrinsic terminal charges $Q_g$, $Q_d$, $Q_s$, and $Q_b$ sum identically to zero ($\sum Q_i = 0$), preventing unphysical drift in high-speed clock generators.
:::

## Example Transient Netlist

```ini
* CMOS Inverter Dynamic Switching Benchmark
.title High-Speed Inverter Transient Response

.param VDD_VAL = 1.8V
.param T_RISE = 25ps

Vdd vdd 0 DC {VDD_VAL}
Vin in 0 PULSE(0 {VDD_VAL} 100ps {T_RISE} {T_RISE} 1ns 2ns)

M1 out in vdd vdd PHONON_PMOS W=1.2u L=45n
M2 out in 0   0   PHONON_NMOS W=0.6u L=45n
Cload out 0 15fF

.model PHONON_NMOS NMOS (LEVEL=54 VERSION=4.8.2 VTH0=0.38 U0=0.048 TOXE=1.6n)
.model PHONON_PMOS PMOS (LEVEL=54 VERSION=4.8.2 VTH0=-0.36 U0=0.016 TOXE=1.6n)

.tran 1ps 4ns TRBDF2 LTE_TOL=1e-7
.probe V(out) V(in) I(Vdd)
.end
```

## Running Transient Simulations

Execute the unified CLI executable:

```bash
# Validate circuit topology and check for floating nodes
phonon validate inverter.cir

# Run transient simulation with CSV telemetry output
phonon run inverter.cir --format csv --output transient_out.csv
```
