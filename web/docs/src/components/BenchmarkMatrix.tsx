import React from 'react';
import { BarChart3, Zap } from 'lucide-react';

interface MatrixRow {
  metric: string;
  category: string;
  phonon: { text: string; highlight?: boolean };
  ngspice: { text: string };
  ltspice: { text: string };
  spectre: { text: string };
}

const COMPARISON_ROWS: MatrixRow[] = [
  {
    metric: 'Transient Step Latency (500 Nodes)',
    category: 'Solver Speed',
    phonon: { text: '<0.42 μs (Sparse CSR + LU)', highlight: true },
    ngspice: { text: '3.80 μs (Sparse1.3 solver)' },
    ltspice: { text: '1.20 μs (Proprietary matrix)' },
    spectre: { text: '0.85 μs (Cadence parallel)' }
  },
  {
    metric: 'Deep Cryo-CMOS Physics (4.2K)',
    category: 'Cryogenic Accuracy',
    phonon: { text: 'Native (Carrier freeze-out & subthreshold)', highlight: true },
    ngspice: { text: 'Fails below 77K (Division by zero)' },
    ltspice: { text: 'Inaccurate (Boltzmann model only)' },
    spectre: { text: 'Requires custom Verilog-A' }
  },
  {
    metric: 'Robotics Flight Multi-Physics Coupling',
    category: 'Co-Simulation',
    phonon: { text: 'In-Process Lockstep (Chronos Bridge)', highlight: true },
    ngspice: { text: 'None (Offline batch only)' },
    ltspice: { text: 'None (GUI desktop only)' },
    spectre: { text: 'Heavyweight Unix socket FFI' }
  },
  {
    metric: 'Piezoelectric Quantum Acoustics (SAW/BAW)',
    category: 'Acoustic EDA',
    phonon: { text: 'Built-in (P-Matrix 8.2 GHz S-params)', highlight: true },
    ngspice: { text: 'Unsupported' },
    ltspice: { text: 'Equivalent RLC approximation only' },
    spectre: { text: 'Requires RF license module' }
  },
  {
    metric: 'Numerical Damping & Ringing Control',
    category: 'Integration Method',
    phonon: { text: 'TR-BDF2 & Gear-2 (L-Stable, 0 Ringing)', highlight: true },
    ngspice: { text: 'Trapezoidal (Severe ringing)' },
    ltspice: { text: 'Modified Trap (Heuristic damping)' },
    spectre: { text: 'Gear-2 / Euler selectable' }
  },
  {
    metric: 'Browser & Edge Deployment',
    category: 'Platform Architecture',
    phonon: { text: 'Zero-GC WebAssembly (60 FPS Canvas)', highlight: true },
    ngspice: { text: 'Heavy Emscripten port' },
    ltspice: { text: 'Windows x86 binary only' },
    spectre: { text: 'Linux RHEL enterprise server' }
  }
];

export function BenchmarkMatrix() {
  return (
    <section className="py-20 px-6 max-w-7xl mx-auto w-full z-10 relative">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 mb-12">
        <div>
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white/[0.04] border border-white/[0.08] text-xs font-mono text-sky-400 mb-3">
            <BarChart3 className="w-3.5 h-3.5" />
            <span>EMPIRICAL BENCHMARKS // SPICE &amp; TCAD PERFORMANCE</span>
          </div>
          <h2 className="text-3xl sm:text-4xl font-extrabold text-white tracking-tight font-sans">
            Phonon vs Classical SPICE Engines
          </h2>
          <p className="text-sm text-slate-400 mt-2 max-w-2xl leading-relaxed">
            Direct comparison across solver throughput, 4.2K cryo-CMOS accuracy, multi-scale acoustic resonators, and autonomous robotics electro-thermal coupling.
          </p>
        </div>

        <div className="flex items-center gap-2 font-mono text-xs text-slate-400 bg-[#0f1012] p-2 rounded-xl border border-white/[0.08]">
          <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
          <span>BENCHMARK SUITE: 500-NODE POWER ESC + CRYO-LNA</span>
        </div>
      </div>

      <div className="bg-[#0f1012] border border-white/[0.08] rounded-2xl overflow-hidden shadow-2xl">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse text-xs">
            <thead>
              <tr className="border-b border-white/[0.08] bg-[#15171a] font-mono text-[11px] text-slate-300">
                <th className="py-4 px-6 font-bold uppercase tracking-wider">Evaluation Dimension</th>
                <th className="py-4 px-6 font-bold text-sky-400 uppercase tracking-wider bg-sky-500/[0.06] border-x border-sky-500/20">
                  Phonon TCAD
                </th>
                <th className="py-4 px-6 font-semibold uppercase tracking-wider text-slate-400">ngspice v42</th>
                <th className="py-4 px-6 font-semibold uppercase tracking-wider text-slate-400">LTspice XVII</th>
                <th className="py-4 px-6 font-semibold uppercase tracking-wider text-slate-400">Cadence Spectre</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/[0.04] font-mono">
              {COMPARISON_ROWS.map((row, idx) => (
                <tr key={idx} className="hover:bg-white/[0.02] transition">
                  <td className="py-4 px-6 text-slate-300 font-sans font-medium">
                    <div className="text-sm font-semibold text-white">{row.metric}</div>
                    <div className="text-xs text-slate-500 font-mono mt-0.5">{row.category}</div>
                  </td>

                  <td className="py-4 px-6 text-white font-bold bg-sky-500/[0.04] border-x border-sky-500/10">
                    <div className="flex items-center gap-2 text-sky-400">
                      <Zap className="w-3.5 h-3.5 fill-current shrink-0" />
                      <span>{row.phonon.text}</span>
                    </div>
                  </td>

                  <td className="py-4 px-6 text-slate-400">{row.ngspice.text}</td>
                  <td className="py-4 px-6 text-slate-400">{row.ltspice.text}</td>
                  <td className="py-4 px-6 text-slate-400">{row.spectre.text}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}
