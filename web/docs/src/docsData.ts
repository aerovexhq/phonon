export interface DocSectionContent {
  title: string;
  badge: string;
  description: string;
  spiceCode?: string;
  verilogCode?: string;
  pythonCode?: string;
  callout?: {
    type: 'note' | 'tip' | 'warning' | 'spec';
    title: string;
    text: string;
  };
  specs?: { param: string; val: string; note: string }[];
  steps?: { title: string; desc: string; cmd?: string }[];
}

export interface DocSection {
  id: string;
  title: string;
  badge: string;
  content: DocSectionContent;
}

export interface DocCategory {
  id: string;
  title: string;
  badge: string;
  sections: DocSection[];
}

export const PHONON_DOC_TREE: DocCategory[] = [
  {
    id: 'getting_started',
    title: '1. Getting Started',
    badge: 'CORE',
    sections: [
      {
        id: 'why_phonon',
        title: 'Why Phonon',
        badge: 'OVERVIEW',
        content: {
          title: 'Universal Electro-Thermal SPICE & Cryo-CMOS EDA',
          badge: 'ARCHITECTURE',
          description: 'Phonon is a modern multi-scale circuit and semiconductor physics simulator built in pure safe Rust, supporting standard SPICE netlists, deep cryogenic temperatures down to 4.2 Kelvin, and real-time co-simulation with the Chronos robotics flight kernel.',
          callout: {
            type: 'spec',
            title: 'Cryogenic Invariant',
            text: 'Phonon uses non-degenerate Fermi-Dirac integrals and incomplete dopant ionization physics, remaining unconditionally stable and accurate at 4.2K where legacy SPICE fails.'
          },
          specs: [
            { param: 'Transient Step Latency', val: '<0.42 μs (Mean)', note: '500-node sparse CSR matrix solve' },
            { param: 'Cryogenic Range', val: '4.2 K to 450 K', note: 'Full temperature continuity' },
            { param: 'Time Integration', val: 'TR-BDF2 & Gear-2', note: 'Zero numerical ringing on fast edges' },
            { param: 'Chronos Coupling', val: 'In-Process Seqlock SHM', note: 'Sub-microsecond electro-thermal feedback' }
          ],
          spiceCode: `* Phonon Sample: Cryogenic Low-Noise Amplifier (4.2K)
.temp 4.2
.param VDD_VAL=1.2V

Vdd vdd 0 DC {VDD_VAL}
Vin in 0 SIN(0 10mV 1GHz)
Rin in gate 50
M1 out gate 0 0 CRYO_NMOS W=10u L=45n
Rd vdd out 1k
Cout out 0 20f

.model CRYO_NMOS NMOS (level=54 vth0=0.55 u0=420 tref=4.2 freeze_out=1)
.tran 10ps 10ns
.print tran v(in) v(out)`,
          verilogCode: `// Verilog-A Behavioral Thermal Resistor Model
\`include "disciplines.vams"

module thermal_resistor(p, n, t_junction);
    inout p, n, t_junction;
    electrical p, n;
    thermal t_junction;
    parameter real r0 = 100.0;
    parameter real tc1 = 0.0039; // Copper temperature coefficient

    analog begin
        V(p, n) <+ I(p, n) * r0 * (1.0 + tc1 * (Temp(t_junction) - 298.15));
        // Power dissipation into thermal node
        P(t_junction) <+ V(p, n) * I(p, n);
    end
endmodule`,
          pythonCode: `# Python Simulation Automation via Phonon C-ABI
import ctypes
from phonon import PhononSession

session = PhononSession()
session.load_netlist("cryo_lna.cir")
session.set_temperature(4.2) # Set cryogenic stage to 4.2 Kelvin
results = session.run_transient(stop_time=10e-9, step=10e-12)

print(f"Peak V(out): {max(results['v(out)'])} V")
print(f"Joule Dissipation: {results.total_joule_energy} Joules")`
        }
      },
      {
        id: 'installation',
        title: 'Installation & Toolchain',
        badge: 'SETUP',
        content: {
          title: 'Installing Phonon SPICE & TCAD',
          badge: 'BINARIES',
          description: 'Phonon is distributed as a single native executable or WebAssembly module with zero runtime dependencies.',
          steps: [
            { title: 'Download via Universal Installer', desc: 'Installs native phonon binary.', cmd: 'curl -fsSL https://phonon.aerovex.net/install.sh | bash' },
            { title: 'Check Active Solver Engines', desc: 'Inspects sparse matrix backend and SIMD features.', cmd: 'phonon --version --features' },
            { title: 'Run Benchmark Netlist', desc: 'Simulates 500-node switching ESC in 42ms.', cmd: 'phonon sim examples/esc_power.cir --tran 1ns 10us' }
          ]
        }
      }
    ]
  },
  {
    id: 'cryo_cmos',
    title: '2. Cryo-CMOS Physics (4.2K)',
    badge: 'CRYO',
    sections: [
      {
        id: 'freeze_out',
        title: 'Dopant Carrier Freeze-Out',
        badge: 'PHYSICS',
        content: {
          title: 'Carrier Freeze-Out & Subthreshold Inversion at 4.2K',
          badge: 'QUANTUM TCAD',
          description: 'At liquid Helium temperatures (4.2K), the thermal energy kT (~0.36 meV) is significantly smaller than typical dopant ionization energies (30-50 meV), causing carriers to freeze out into donor and acceptor levels.',
          callout: {
            type: 'spec',
            title: 'Physical Invariant',
            text: 'Axiom solves the incomplete ionization state equation self-consistently with the Poisson equation, correctly predicting the plateau and kink phenomena in cryo-CMOS output characteristics.'
          },
          specs: [
            { param: 'Ionization Energy Ed', val: '45 meV (Phosphorus in Si)', note: 'Shallow donor binding energy' },
            { param: 'Subthreshold Swing', val: '<12 mV/decade @ 4.2K', note: 'Extreme subthreshold steepening' },
            { param: 'Carrier Mobility Gain', val: '2.4x to 3.8x over 300K', note: 'Phonon scattering suppressed' }
          ]
        }
      }
    ]
  },
  {
    id: 'transient_coupling',
    title: '3. Multi-Physics & Transient',
    badge: 'SOLVER',
    sections: [
      {
        id: 'tr_bdf2',
        title: 'TR-BDF2 Time-Stepping',
        badge: 'L-STABLE',
        content: {
          title: 'Composite L-Stable Adaptive Transient Integration',
          badge: 'INTEGRATION',
          description: 'Combines the accuracy of the Trapezoidal rule with the high-frequency damping of BDF-2, eliminating non-physical voltage oscillations during power MOSFET switching in drone ESCs.',
          specs: [
            { param: 'Stability Model', val: 'L-Stable (A-Stable with zero infinity pole)', note: 'Damps stiff inductive spikes' },
            { param: 'Local Truncation Error', val: 'O(h^3) error estimator', note: 'Autonomous step-size modulation' },
            { param: 'Switching Speedup', val: '5x to 40x over fixed step', note: 'Adapts dt from 1ps to 100ns' }
          ]
        }
      },
      {
        id: 'chronos_bridge',
        title: 'Chronos Robotics Coupling',
        badge: 'ROBOTICS',
        content: {
          title: 'Electro-Thermal Feedback with Chronos Physics Kernel',
          badge: 'IN-PROCESS IPC',
          description: 'Phonon exchanges motor currents and battery terminal voltages with the Chronos parallel flight simulation kernel every physics tick, calculating dynamic junction temperatures and resistance degradation.',
          specs: [
            { param: 'IPC Transport', val: 'POSIX Shared Memory Atomic Seqlock', note: 'Zero-copy lockstep exchange' },
            { param: 'Feedback Latency', val: '<150 ns per tick', note: 'Seamless parallel execution' },
            { param: 'Thermal Safeguard', val: 'Over-temperature ESC throttling', note: 'Prevents motor burn-out' }
          ]
        }
      }
    ]
  }
];
