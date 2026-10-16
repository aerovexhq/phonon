export interface Subsystem {
  id: string;
  name: string;
  domain: string;
  tierLabel: string;
  category: string;
  status: string;
  statusTone: 'emerald' | 'cyan' | 'amber' | 'purple';
  tagline: string;
  metric: string;
  metricDetail: string;
  description: string;
  highlights: string[];
  tags: string[];
  docSectionId: string;
  architectureDetails: {
    pipelineDiagram: string;
    tradeoffs: { choice: string; why: string }[];
    codeSnippet: {
      language: string;
      title: string;
      code: string;
    };
  };
}

export const PHONON_SUBSYSTEMS: Subsystem[] = [
  {
    id: 'mna_solver',
    name: 'Modified Nodal Analysis (MNA) Solver',
    domain: 'mna',
    tierLabel: 'TIER 1 // NUMERICAL CORE',
    category: 'Circuit Analysis',
    status: 'ACTIVE',
    statusTone: 'cyan',
    tagline: 'High-speed sparse LU matrix decomposition with damped Newton-Raphson convergence.',
    metric: '<0.42 μs',
    metricDetail: 'Mean Transient Step Latency across 500-Node Circuits',
    description: 'Solves coupled non-linear differential-algebraic equations (DAEs) governing electrical node voltages and branch currents using compressed sparse row (CSR) matrices.',
    highlights: [
      'Sparse LU factorization with Markowitz pivot reordering for minimal fill-in',
      'Adaptive damping Newton-Raphson nonlinear solver preventing convergence divergence',
      'Autonomous Jacobian matrix caching for linear and quasi-linear sub-blocks'
    ],
    tags: ['MNA Solver', 'Sparse LU', 'Newton-Raphson', 'Sub-μs Step'],
    docSectionId: 'mna_solver_architecture',
    architectureDetails: {
      pipelineDiagram: `SPICE Netlist Graph (.cir / .sp)
           │
           ▼
[Conductance & Capacitance Engine] ── Sparse CSR Topology
           │
           ▼
[Nonlinear MNA Solver] ── Robust Iterative Convergence
           │
           ▼
[Dynamic Solution Stream] ── Instantaneous Node Voltages & Currents`,
      tradeoffs: [
        { choice: 'Sparse CSR vs Dense Matrix', why: 'Circuit matrices are >95% sparse; CSR yields O(N) operations instead of O(N^3).' },
        { choice: 'Damped Newton-Raphson vs Standard', why: 'Guarantees global convergence in stiff high-gain amplifier regimes.' }
      ],
      codeSnippet: {
        language: 'spice',
        title: 'Phonon High-Efficiency Transient SPICE Netlist',
        code: `* Phonon Power Converter Transient Simulation
.title High-Voltage Synchronous Buck Converter
.param VIN=48.0V VOUT=5.0V FSW=2.0MEG

Vin in 0 DC {VIN}
Vclk hs_gate 0 PULSE(0 10 0 2n 2n 240n 500n)
M1 in hs_gate sw 0 POWER_NMOS W=120u L=0.18u
L1 sw out 2.2u
Cout out 0 22u
Rload out 0 2.5

.model POWER_NMOS NMOS (level=54 vth0=1.2 u0=380 rd=0.015)
.tran 50ps 2us
.options reltol=1e-4 method=trbdf2`
      }
    }
  },
  {
    id: 'cryo_cmos',
    name: 'Cryo-CMOS TCAD (4.2K - 300K)',
    domain: 'cryo',
    tierLabel: 'TIER 1 // QUANTUM HARDWARE',
    category: 'Semiconductor TCAD',
    status: 'OPTIMIZED',
    statusTone: 'emerald',
    tagline: 'Deep cryogenic semiconductor physics modeling carrier freeze-out and subthreshold steepening.',
    metric: '4.2 Kelvin',
    metricDetail: 'Full Multi-Scale Cryogenic TCAD Physical Accuracy Range',
    description: 'Accurately predicts transconductance collapse, dopant freeze-out, kink effect, and sub-60mV/dec subthreshold swings for cryogenic qubit control chips.',
    highlights: [
      'Fermi-Dirac carrier statistics replacing Maxwell-Boltzmann at sub-77K',
      'Incomplete dopant ionization dynamics modeling shallow donor/acceptor freeze-out',
      'Direct interface with quantum acoustic and superconducting coplanar waveguides'
    ],
    tags: ['Cryo-CMOS', '4.2K Physics', 'Carrier Freeze-Out', 'Qubit Control'],
    docSectionId: 'cryo_cmos_physics',
    architectureDetails: {
      pipelineDiagram: `Cryogenic Operating Profile (4.2 K to 300 K)
           │
           ▼
[Cryogenic Semiconductor TCAD] ── Non-Degenerate Fermi-Dirac Physics
           │
           ▼
[Carrier Transport Evaluator] ── Low-Temperature Mobility & Freeze-Out
           │
           ▼
[Qubit Readout Interface] ── Cryogenic LNA & Multiplexer Verification`,
      tradeoffs: [
        { choice: 'Full Cryogenic TCAD vs Ambient Approximation', why: 'Standard SPICE simulators produce divide-by-zero errors and unphysical results below 77K.' },
        { choice: 'Continuous Temperature Scaling vs Segmented Lookups', why: 'Guarantees smooth thermal sweeps from liquid helium (4.2K) to ambient room temperature.' }
      ],
      codeSnippet: {
        language: 'spice',
        title: 'Cryo-CMOS 4.2K Transistor Model Card & Sweep',
        code: `* Phonon 4.2K Cryogenic Low-Noise Amplifier Stage
.temp 4.2

Vdd vdd 0 DC 1.0V
Vbias gate 0 DC 0.65V
Vin in gate SIN(0 5mV 500MEG)
M1 out in 0 0 CRYO_TRANSISTOR W=10u L=28n
Rd vdd out 1.5k

.model CRYO_TRANSISTOR NMOS (
+ level=54
+ tref=4.2
+ vth0=0.55
+ u0=480
+ freeze_out=1
)
.ac dec 100 100MEG 10G`
      }
    }
  },
  {
    id: 'quantum_resonators',
    name: 'Quantum Acoustic Resonators',
    domain: 'quantum',
    tierLabel: 'TIER 2 // QUANTUM EDA',
    category: 'Acoustics & Microwave',
    status: 'ACTIVE',
    statusTone: 'purple',
    tagline: 'Coupled piezoelectric Surface Acoustic Wave (SAW) and Bulk Acoustic Wave (BAW) modeling.',
    metric: '8.2 GHz',
    metricDetail: 'Peak Resonant Frequency for Ultra-High Q Acoustic Filters',
    description: 'Simulates piezoelectric elastodynamics coupled directly with microwave transmission lines, modeling mechanical displacement, phase velocity, and insertion loss.',
    highlights: [
      'Constitutive piezoelectric model parameter extraction for RF filter design',
      'Electromechanical coupling coefficient optimization and spurious mode suppression',
      'Harmonic acoustic wave frequency response analysis'
    ],
    tags: ['SAW/BAW', 'Piezoelectric', 'RF Resonator', 'Quantum Acoustic'],
    docSectionId: 'quantum_resonators',
    architectureDetails: {
      pipelineDiagram: `Acoustic Resonator Topology & Materials
           │
           ▼
[Piezoelectric Transducer Interface] ── Mechanical-Electrical Transduction
           │
           ▼
[Microwave Waveguide Coupling] ── Insertion Loss & Dispersion
           │
           ▼
[Scattering S-Parameters] ── S11 Reflection & S21 Passband Metrics`,
      tradeoffs: [
        { choice: 'Coupled Equivalent Network vs Full 3D FEM', why: 'Provides instant interactive design sweeps with sub-1% discrepancy compared to full mesh solvers.' },
        { choice: 'Lossy Acoustic Attenuation vs Ideal Lossless', why: 'Essential for predicting real-world high-frequency insertion loss and filter selectivity.' }
      ],
      codeSnippet: {
        language: 'spice',
        title: 'High-Q Acoustic Filter Netlist Definition',
        code: `* 8.2 GHz SAW Bandpass Resonator Filter
.title Quantum Acoustic Delay Line
Port1 in 0 50
Port2 out 0 50

Xres1 in out 0 ACOUSTIC_SAW_RESONATOR
+ f0=8.2G Q_mech=8500 k2=0.065 C0=180f

.subckt ACOUSTIC_SAW_RESONATOR p n gnd params: f0=8.2G Q_mech=8500 k2=0.065 C0=180f
Cp p n {C0}
L_m p m {1.0 / (4 * 3.14159^2 * f0^2 * (C0 * k2))}
C_m m k {C0 * k2}
R_m k n {(2 * 3.14159 * f0 * (1.0 / (4 * 3.14159^2 * f0^2 * (C0 * k2)))) / Q_mech}
.ends

.sp dec 200 7.5G 9.0G`
      }
    }
  },
  {
    id: 'adaptive_transient',
    name: 'Adaptive Transient Stepping',
    domain: 'transient',
    tierLabel: 'TIER 2 // ADAPTIVE SOLVER',
    category: 'Numerical Stability',
    status: 'ACTIVE',
    statusTone: 'cyan',
    tagline: 'Autonomous Gear-2 & TR-BDF2 time integration with local truncation error control.',
    metric: 'TR-BDF2',
    metricDetail: 'Composite L-Stable Time-Stepping with Zero Artificial Damping',
    description: 'Eliminates trapezoidal ringing on inductive switching spikes while accelerating time steps by up to 40x during smooth DC transitions.',
    highlights: [
      'Autonomous local truncation error (LTE) step-size adaptation',
      'A-stable and L-stable Gear-2 integration for stiff magnetic components',
      'Zero numerical oscillation on fast power MOSFET switching edges'
    ],
    tags: ['TR-BDF2', 'Gear-2', 'LTE Control', 'Zero Ringing'],
    docSectionId: 'transient_stepping',
    architectureDetails: {
      pipelineDiagram: `Dynamic Circuit State Trajectory
           │
           ▼
[TR-BDF2 Composite Stage] ── Ringing-Free Time Discretization
           │
           ▼
[Autonomous Step-Size Controller] ── Adaptive Expansion / Contraction
           │
           ▼
[High-Fidelity Transient Trajectory] ── Zero Non-Physical Oscillations`,
      tradeoffs: [
        { choice: 'TR-BDF2 vs Standard Trapezoidal', why: 'Trapezoidal integration suffers from non-physical ringing on steep switching edges; TR-BDF2 is strictly L-stable.' },
        { choice: 'Adaptive Step-Size vs Fixed Time-Step', why: 'Reduces total required steps by up to 20x while maintaining picosecond accuracy during switching events.' }
      ],
      codeSnippet: {
        language: 'spice',
        title: 'Adaptive Transient Solver Directives',
        code: `* Phonon Adaptive Step Configuration
.title High-Speed Gate Driver Simulation
Vgate in 0 PULSE(0 15 0 100p 100p 20n 50n)

* Specify TR-BDF2 integration with adaptive local error tolerance
.options method=trbdf2
.options reltol=1e-4 abstol=1e-9 vntol=1e-6
.options trtol=1.0 chgtol=1e-14

* Run simulation from 0 to 100ns with autonomous step scaling
.tran 10ps 100ns 0 500ps`
      }
    }
  },
  {
    id: 'thermal_coupling',
    name: 'Thermal Runaway & Chronos Coupling',
    domain: 'thermal',
    tierLabel: 'TIER 3 // MULTI-PHYSICS',
    category: 'Robotics Integration',
    status: 'ACTIVE',
    statusTone: 'amber',
    tagline: 'Bidirectional multi-physics coupling with Chronos flight dynamics and battery thermals.',
    metric: 'In-Process IPC',
    metricDetail: 'Sub-Microsecond Lockstep Exchange with Chronos Multi-Physics Kernel',
    description: 'Calculates instantaneous Joule heating across drone ESCs, motors, and lithium-polymer cells, injecting thermal resistance degradation directly into flight simulation.',
    highlights: [
      'Dynamic electro-thermal feedback modeling temperature-dependent resistance',
      'Lithium-ion cell internal resistance runaway threshold warning',
      'Direct shared memory coupling with Aerovex Workstation 3D cockpit'
    ],
    tags: ['Thermal Runaway', 'Chronos Bridge', 'Joule Heating', 'LiPo Degradation'],
    docSectionId: 'thermal_co_simulation',
    architectureDetails: {
      pipelineDiagram: `Chronos Flight Motor Current & Mechanical Load
           │
           ▼
[Phonon Electro-Thermal Solver] ── Real-Time Joule Dissipation
           │
           ▼
[Thermal Model Network] ── Junction & Cell Heat Dynamics
           │
           ▼
[Degraded Electrical Parameters] ── Torque & Voltage Feedback to Chronos`,
      tradeoffs: [
        { choice: 'In-Process Lockstep Coupling vs Offline Analysis', why: 'Predicts real-world thermal runaway and motor power collapse during aggressive flight maneuvers.' },
        { choice: 'Lumped Thermal RC Network vs Full 3D CFD', why: 'Executes thousands of times faster than CFD while providing high thermal prediction accuracy.' }
      ],
      codeSnippet: {
        language: 'python',
        title: 'Phonon-Chronos Co-Simulation Python Bridge',
        code: `import phonon
import chronos

# Connect Phonon circuit simulator to Chronos physics session
sim = phonon.CircuitSession("power_distribution_esc.cir")
drone = chronos.SimulationWorld("drone_quad_heavy.json")

# Run coupled lockstep stepping
for step in range(1000):
    motor_current = drone.get_motor_current(motor_idx=0)
    sim.set_source_current("I_motor", motor_current)
    sim.step_transient(dt=0.001)
    
    # Read junction temperature and update motor resistance in Chronos
    junction_temp = sim.get_node_voltage("T_junction")
    drone.update_motor_temperature(motor_idx=0, temp_k=junction_temp)`
      }
    }
  }
];
