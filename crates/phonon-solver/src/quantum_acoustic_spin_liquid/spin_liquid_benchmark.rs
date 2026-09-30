#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Abelian quantum acoustic fractional
//! spin liquids and topological resonating valence bond networks across multi-threaded Rayon workers.

use crate::quantum_acoustic_spin_liquid::QuantumAcousticSpinLiquidSolver;
use phonon_models::quantum_acoustic_spin_liquid::{
    QuantumAcousticSpinLiquidMetrics, QuantumAcousticSpinLiquidParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for quantum acoustic spin liquid parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpinLiquidBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_spinon_excitation_fidelity: f64,
    pub min_spinon_excitation_fidelity: f64,
    pub max_spinon_excitation_fidelity: f64,
    pub mean_topological_entanglement_entropy: f64,
    pub min_topological_entanglement_entropy: f64,
    pub max_topological_entanglement_entropy: f64,
    pub mean_topological_entropy_error: f64,
    pub min_topological_entropy_error: f64,
    pub max_topological_entropy_error: f64,
    pub mean_spin_mechanical_crosstalk_isolation_db: f64,
    pub min_spin_mechanical_crosstalk_isolation_db: f64,
    pub max_spin_mechanical_crosstalk_isolation_db: f64,
    pub mean_ground_state_degeneracy_protection_db: f64,
    pub min_ground_state_degeneracy_protection_db: f64,
    pub max_ground_state_degeneracy_protection_db: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct SpinLiquidBenchmarkRunner;

impl SpinLiquidBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> SpinLiquidBenchmarkResult {
        let sweep_params: Vec<QuantumAcousticSpinLiquidParams> = (0..cycles)
            .map(|i| {
                let heisenberg_exchange_coupling_mhz = 20.0 + 110.0 * (((i * 7) % 50) as f64 / 50.0);
                let frustration_ratio_j2_j1 = 0.10 + 0.40 * (((i * 13) % 45) as f64 / 45.0);
                let spinon_phonon_coupling_mhz = 3.0 + 22.0 * (((i * 11) % 40) as f64 / 40.0);
                let chiral_three_spin_scalar_chirality = 1.0 + 16.0 * (((i * 17) % 35) as f64 / 35.0);
                let kagome_plaquette_count = 12 + ((i * 19) % 45);
                let acoustic_driving_frequency_ghz = 2.0 + 8.0 * (((i * 23) % 30) as f64 / 30.0);
                let cryogenic_temperature_mk = 3.0 + 35.0 * (((i * 29) % 32) as f64 / 32.0);
                let lattice_geometry_type = i % 2;

                QuantumAcousticSpinLiquidParams::new(
                    heisenberg_exchange_coupling_mhz,
                    frustration_ratio_j2_j1,
                    spinon_phonon_coupling_mhz,
                    chiral_three_spin_scalar_chirality,
                    kagome_plaquette_count,
                    acoustic_driving_frequency_ghz,
                    cryogenic_temperature_mk,
                    lattice_geometry_type,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<QuantumAcousticSpinLiquidMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = QuantumAcousticSpinLiquidSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_entropy = 0.0;
        let mut min_entropy = f64::MAX;
        let mut max_entropy = f64::MIN;

        let mut sum_error = 0.0;
        let mut min_error = f64::MAX;
        let mut max_error = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_protection = 0.0;
        let mut min_protection = f64::MAX;
        let mut max_protection = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.spinon_excitation_fidelity;
            if m.spinon_excitation_fidelity < min_fidelity {
                min_fidelity = m.spinon_excitation_fidelity;
            }
            if m.spinon_excitation_fidelity > max_fidelity {
                max_fidelity = m.spinon_excitation_fidelity;
            }

            sum_entropy += m.topological_entanglement_entropy;
            if m.topological_entanglement_entropy < min_entropy {
                min_entropy = m.topological_entanglement_entropy;
            }
            if m.topological_entanglement_entropy > max_entropy {
                max_entropy = m.topological_entanglement_entropy;
            }

            sum_error += m.topological_entropy_error;
            if m.topological_entropy_error < min_error {
                min_error = m.topological_entropy_error;
            }
            if m.topological_entropy_error > max_error {
                max_error = m.topological_entropy_error;
            }

            sum_isolation += m.spin_mechanical_crosstalk_isolation_db;
            if m.spin_mechanical_crosstalk_isolation_db < min_isolation {
                min_isolation = m.spin_mechanical_crosstalk_isolation_db;
            }
            if m.spin_mechanical_crosstalk_isolation_db > max_isolation {
                max_isolation = m.spin_mechanical_crosstalk_isolation_db;
            }

            sum_protection += m.ground_state_degeneracy_protection_db;
            if m.ground_state_degeneracy_protection_db < min_protection {
                min_protection = m.ground_state_degeneracy_protection_db;
            }
            if m.ground_state_degeneracy_protection_db > max_protection {
                max_protection = m.ground_state_degeneracy_protection_db;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        let physical_compliance_fraction = (compliant_count as f64) / n;

        SpinLiquidBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_spinon_excitation_fidelity: sum_fidelity / n,
            min_spinon_excitation_fidelity: min_fidelity,
            max_spinon_excitation_fidelity: max_fidelity,
            mean_topological_entanglement_entropy: sum_entropy / n,
            min_topological_entanglement_entropy: min_entropy,
            max_topological_entanglement_entropy: max_entropy,
            mean_topological_entropy_error: sum_error / n,
            min_topological_entropy_error: min_error,
            max_topological_entropy_error: max_error,
            mean_spin_mechanical_crosstalk_isolation_db: sum_isolation / n,
            min_spin_mechanical_crosstalk_isolation_db: min_isolation,
            max_spin_mechanical_crosstalk_isolation_db: max_isolation,
            mean_ground_state_degeneracy_protection_db: sum_protection / n,
            min_ground_state_degeneracy_protection_db: min_protection,
            max_ground_state_degeneracy_protection_db: max_protection,
            physical_compliance_fraction,
        }
    }
}
