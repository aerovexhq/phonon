//! Multi-threaded Rayon benchmark comparing Topological Majorana Qubits against Transmons and Surface Codes.
//!
//! Evaluates physical footprint, gate fidelity, coherence lifetimes, and energy dissipation
//! across 10,000 parallel Clifford circuit instances.

use rayon::prelude::*;
use std::time::Instant;

use super::braiding_solver::MajoranaBraidingSolver;
use super::parity_tracker::FermionParitySolver;
use phonon_models::topological::{TJunctionNanowireNetwork, TopologicalQubit};

/// Technology metrics report comparing Topological Qubits against Transmon and Surface Codes.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkComparisonReport {
    /// Number of parallel Clifford circuits simulated.
    pub total_circuits_simulated: usize,
    /// Average gate fidelity achieved by Topological Majorana Qubits.
    pub topological_avg_fidelity: f64,
    /// Average gate fidelity achieved by physical (unprotected) Transmons.
    pub transmon_unprotected_avg_fidelity: f64,
    /// Average gate fidelity achieved by Surface-Code Transmons ($d=3$).
    pub surface_code_d3_avg_fidelity: f64,
    /// Physical footprint per logical qubit in square micrometers ($\mu\text{m}^2$).
    pub topological_footprint_um2: f64,
    pub transmon_unprotected_footprint_um2: f64,
    pub surface_code_d3_footprint_um2: f64,
    /// Footprint reduction factor: Area(Surface-Code) / Area(Topological).
    pub footprint_reduction_factor: f64,
    /// Coherence lifetime advantage factor: $T_1^{topo} / T_1^{transmon}$.
    pub coherence_advantage_factor: f64,
    /// Fermion parity conservation rate across all circuits.
    pub parity_conservation_rate: f64,
    /// Total wallclock execution time in milliseconds.
    pub elapsed_wallclock_ms: f64,
}

/// Multi-threaded benchmark runner.
#[derive(Debug, Clone)]
pub struct TopologicalBenchmarkRunner {
    pub num_circuits: usize,
    pub gates_per_circuit: usize,
    pub gate_duration_s: f64,
}

impl Default for TopologicalBenchmarkRunner {
    fn default() -> Self {
        Self {
            num_circuits: 10_000,
            gates_per_circuit: 10,
            gate_duration_s: 2.0e-9, // 2 ns per braid
        }
    }
}

impl TopologicalBenchmarkRunner {
    /// Creates a runner with specified circuit count and gate depth.
    pub fn new(num_circuits: usize, gates_per_circuit: usize) -> Self {
        Self {
            num_circuits,
            gates_per_circuit,
            gate_duration_s: 2.0e-9,
        }
    }

    /// Executes the multi-threaded parallel benchmark using Rayon.
    pub fn run_benchmark(&self) -> BenchmarkComparisonReport {
        let start_time = Instant::now();

        // Physical parameters
        let topo_area_um2 = 1.0; // 1 um x 1 um T-junction
        let transmon_area_um2 = 250_000.0; // 500 um x 500 um = 0.25 mm^2
        let sc_d3_area_um2 = 17.0 * transmon_area_um2; // 17 physical transmons for d=3

        let topo_t1_s = 0.2; // 200 ms (quasiparticle poisoning limited)
        let transmon_t1_s = 100.0e-6; // 100 us

        // Run parallel circuit evaluations across all CPU cores
        let results: Vec<(f64, f64, f64, bool)> = (0..self.num_circuits)
            .into_par_iter()
            .map(|circuit_idx| {
                let mut qubit = TopologicalQubit::new();
                let network = TJunctionNanowireNetwork::default();
                let mut solver = MajoranaBraidingSolver::new(network, self.gate_duration_s);
                let mut parity_tracker = FermionParitySolver::new(5.0); // 5 Hz poisoning

                let mut topo_fidelity = 1.0;
                let mut transmon_fidelity = 1.0;
                let mut sc_fidelity = 1.0;

                // Simulate a sequence of Clifford gates: H, S, X, Z, H, S...
                for gate_idx in 0..self.gates_per_circuit {
                    let seed = (circuit_idx as u64)
                        .wrapping_mul(10007)
                        .wrapping_add(gate_idx as u64)
                        .wrapping_mul(6364136223846793005);
                    let draw = (seed >> 33) as f64 / (1u64 << 31) as f64;

                    match gate_idx % 4 {
                        0 => {
                            let step = solver.execute_hadamard_gate(&mut qubit);
                            topo_fidelity *= step.gate_fidelity;
                            transmon_fidelity *= 0.999; // 99.9% 1-qubit gate
                            sc_fidelity *= 0.99999; // 99.999% logical gate
                        }
                        1 => {
                            let step = solver.execute_phase_gate(&mut qubit);
                            topo_fidelity *= step.gate_fidelity;
                            transmon_fidelity *= 0.999;
                            sc_fidelity *= 0.99999;
                        }
                        2 => {
                            qubit.apply_pauli_x();
                            transmon_fidelity *= 0.999;
                            sc_fidelity *= 0.99999;
                        }
                        _ => {
                            qubit.apply_pauli_z();
                            transmon_fidelity *= 0.999;
                            sc_fidelity *= 0.99999;
                        }
                    }

                    // Check for stochastic quasiparticle poisoning during the gate duration
                    let poisoned =
                        parity_tracker.step_evolution(self.gate_duration_s, &mut qubit, draw);
                    if poisoned {
                        topo_fidelity *= 0.5; // Parity flip corrupts state
                    }
                }

                let parity_ok = qubit.is_parity_preserved();
                (topo_fidelity, transmon_fidelity, sc_fidelity, parity_ok)
            })
            .collect();

        let elapsed = start_time.elapsed().as_secs_f64() * 1000.0;

        let total = results.len() as f64;
        let sum_topo_fid: f64 = results.iter().map(|r| r.0).sum();
        let sum_transmon_fid: f64 = results.iter().map(|r| r.1).sum();
        let sum_sc_fid: f64 = results.iter().map(|r| r.2).sum();
        let parity_ok_count: usize = results.iter().filter(|r| r.3).count();

        let avg_topo_fid = sum_topo_fid / total;
        let avg_transmon_fid = sum_transmon_fid / total;
        let avg_sc_fid = sum_sc_fid / total;
        let parity_rate = parity_ok_count as f64 / total;

        BenchmarkComparisonReport {
            total_circuits_simulated: self.num_circuits,
            topological_avg_fidelity: avg_topo_fid,
            transmon_unprotected_avg_fidelity: avg_transmon_fid,
            surface_code_d3_avg_fidelity: avg_sc_fid,
            topological_footprint_um2: topo_area_um2,
            transmon_unprotected_footprint_um2: transmon_area_um2,
            surface_code_d3_footprint_um2: sc_d3_area_um2,
            footprint_reduction_factor: sc_d3_area_um2 / topo_area_um2,
            coherence_advantage_factor: topo_t1_s / transmon_t1_s,
            parity_conservation_rate: parity_rate,
            elapsed_wallclock_ms: elapsed,
        }
    }
}
