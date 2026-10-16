# Tier 5: Coupled Electro-Thermal MNA

Tier 5 models monolithic bidirectional electro-thermal feedback, dynamically coupling electrical power dissipation directly into Cauer RC thermal lattices during transient simulation.

::: tip Monolithic Coupling Rationale
In high-power robotics avionics (such as motor ESC inverters and BLDC drive gates), power dissipation $P_{\text{diss}}(t) = V_{ds} \cdot I_{ds}$ heats the silicon die. As temperature rises, carrier mobility degrades and leakage current surges. Pipelined or uncoupled simulators miss sub-microsecond thermal spikes. Phonon solves electrical node voltages and junction temperatures simultaneously within the same Newton-Raphson Jacobian matrix.
:::

## Monolithic Jacobian Formulation

Phonon embeds thermal nodes directly into the Modified Nodal Analysis system as virtual thermal voltages ($1\,\text{V} \equiv 1\,\text{K}$) and heat flux currents ($1\,\text{A} \equiv 1\,\text{W}$):

```
[ J_EE   J_ET ] [ delta_V ] = - [ Residual_Electrical ]
[ J_TE   J_TT ] [ delta_T ]   - [ Residual_Thermal    ]
```

Where:
- `J_EE`: Classical electrical Jacobian with respect to node voltages.
- `J_ET`: Electro-thermal sensitivity matrix: $\partial I_i / \partial T_j$.
- `J_TE`: Power dissipation gradient with respect to voltage: $\partial P_{\text{joule}} / \partial V_i$.
- `J_TT`: Thermal conductance and capacitance matrix: $G_{th} + \frac{C_{th}}{\Delta t}$.

## Cauer 4-Stage Thermal RC Network

Phonon implements genuine Cauer networks reflecting physical physical heat flow from die to ambient heat sink:

```
Die Junction (T_j) 
   |
  [R_th1] --- (Die Substrate T_sub)
   |                |
  [C_th1]          [R_th2] --- (Package Case T_pkg)
   |                |                |
  GND              [C_th2]          [R_th3] --- (Heat Sink T_sink)
                    |                |                |
                   GND              [C_th3]          [R_th4] --- Ambient (T_amb)
                                     |                |
                                    GND              [C_th4]
                                                      |
                                                     GND
```

### Physical Layer Thermal Properties

| Layer Stage | Physical Component | Thermal Resistance ($R_{th}$) | Thermal Capacitance ($C_{th}$) | Time Constant ($\tau$) |
| :--- | :--- | :--- | :--- | :--- |
| **Stage 1** | Silicon Die Junction | 0.24 K/W | 1.20 mJ/K | 0.288 ms |
| **Stage 2** | Die Attach Solder | 0.48 K/W | 4.80 mJ/K | 2.304 ms |
| **Stage 3** | Copper Heat Slug & Package | 1.10 K/W | 32.0 mJ/K | 35.20 ms |
| **Stage 4** | Aluminum Extruded Heat Sink | 2.40 K/W | 480.0 mJ/K | 1,152 ms |

::: warning Thermal Runaway Protection
In high-current MOSFETs, when $\frac{\partial P_{\text{diss}}}{\partial T} > \frac{1}{R_{\text{th, total}}}$, thermal runaway initiates: heat generation exceeds dissipation capacity, leading to exponential temperature runaway. Phonon automatically triggers thermal divergence fail-safes and outputs exact critical margin metrics.
:::

## In-Process Chronos Robotics Co-Simulation

Phonon interfaces directly with Aerovex Chronos multi-world flight simulation via POSIX shared memory ring buffers operating at 400 Hz hard real-time:

```rust
// In-Process Co-Simulation Step
let motor_torque_nm = chronos_shm.read_torque_demand();
let motor_current_a = torque_to_current(motor_torque_nm);

// Step electro-thermal solver
let t_junction_k = phonon_engine.step_electro_thermal(motor_current_a, dt);

// Write telemetry back to Chronos flight controller
chronos_shm.write_esc_telemetry(EscState {
    t_junction_k,
    r_on_eff: r_on_at_temp(t_junction_k),
    overheat_alert: t_junction_k > 440.0,
});
```

## Coupled SPICE Netlist Example

```ini
* Motor Inverter Half-Bridge with Cauer Thermal Feedback
.title Brushless Motor ESC Electro-Thermal Coupled Transient

.param T_AMBIENT = 300.0
.param V_BUS = 24.0V

Vbus vbus 0 DC {V_BUS}
Vgate_hi ghi 0 PULSE(0 10V 0 10ns 10ns 25us 50us)

* Power MOSFET with dedicated thermal node T_DIE
M_high vbus ghi phase 0 PHONON_POWER_NFET W=20m L=0.18u T_NODE=t_die
L_motor phase 0 120uH RSER=0.08

* Cauer Thermal Stages (1V = 1K)
R_th1 t_die t_sub  0.24
C_th1 t_die 0      1.20m
R_th2 t_sub t_pkg  0.48
C_th2 t_sub 0      4.80m
R_th3 t_pkg t_sink 1.10
C_th3 t_pkg 0      32.0m
R_th4 t_sink 0     2.40
C_th4 t_sink 0     480.0m

.model PHONON_POWER_NFET NMOS (LEVEL=54 VTH0=2.20 U0=0.065 TCAD_THERMAL=1)

.tran 100ns 10ms TRBDF2
.probe V(phase) I(L_motor) TEMP(t_die)
.end
```
