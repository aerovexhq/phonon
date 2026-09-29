//! Parallel parameter sweep benchmark suite for superconducting optomechanical
//! quantum teleportation across phononic crystal waveguides.

use crate::quantum_teleportation_waveguide::QuantumTeleportationSolver;
use phonon_models::quantum_teleportation_waveguide::{
    QuantumTeleportationMetrics, QuantumTeleportationParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark summary report for phononic waveguide quantum teleportation parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumTeleportationBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_fidelity: f64,
    pub min_fidelity: f64,
    pub mean_distillation_purity: f64,
    pub min_distillation_purity: f64,
    pub mean_waveguide_loss_db_per_cm: f64,
    pub max_waveguide_loss_db_per_cm: f64,
    pub mean_quantum_memory_t2_ms: f64,
    pub min_quantum_memory_t2_ms: f64,
    pub mean_bell_concurrence: f64,
    pub min_bell_concurrence: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct QuantumTeleportationBenchmarkRunner;

impl QuantumTeleportationBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> QuantumTeleportationBenchmarkResult {
        let sweep_params: Vec<QuantumTeleportationParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let f_q = 4.0 + 3.0 * frac; // 4.0 to 7.0 GHz
                let length_cm = 0.5 + 4.5 * ((i % 100) as f64 / 100.0); // 0.5 to 5.0 cm
                let loss_db = 0.010 + 0.035 * frac; // 0.010 to 0.045 dB/cm
                let coop = 25.0 + 55.0 * ((i % 80) as f64 / 80.0); // 25.0 to 80.0
                let squeezing = 1.0 + 1.5 * ((i % 60) as f64 / 60.0); // 1.0 to 2.5
                let bsm_eff = 0.91 + 0.08 * (1.0 - frac); // 0.91 to 0.99
                let t2_ms = 1.5 + 8.5 * ((i % 120) as f64 / 120.0); // 1.5 to 10.0 ms
                let temp_mk = 10.0 + 15.0 * ((i % 50) as f64 / 50.0); // 10.0 to 25.0 mK

                QuantumTeleportationParams::new(
                    f_q,
                    length_cm,
                    loss_db,
                    coop,
                    squeezing,
                    bsm_eff,
                    t2_ms,
                    temp_mk,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<QuantumTeleportationMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = QuantumTeleportationSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;
        let mut sum_loss = 0.0;
        let mut max_loss = f64::MIN;
        let mut sum_t2 = 0.0;
        let mut min_t2 = f64::MAX;
        let mut sum_conc = 0.0;
        let mut min_conc = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.teleportation_fidelity;
            if m.teleportation_fidelity < min_fid {
                min_fid = m.teleportation_fidelity;
            }

            sum_purity += m.entanglement_distillation_purity;
            if m.entanglement_distillation_purity < min_purity {
                min_purity = m.entanglement_distillation_purity;
            }

            sum_loss += m.waveguide_propagation_loss_db_per_cm;
            if m.waveguide_propagation_loss_db_per_cm > max_loss {
                max_loss = m.waveguide_propagation_loss_db_per_cm;
            }

            sum_t2 += m.quantum_memory_coherence_time_ms;
            if m.quantum_memory_coherence_time_ms < min_t2 {
                min_t2 = m.quantum_memory_coherence_time_ms;
            }

            sum_conc += m.bell_state_concurrence;
            if m.bell_state_concurrence < min_conc {
                min_conc = m.bell_state_concurrence;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        QuantumTeleportationBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_fidelity: sum_fid / n,
            min_fidelity: min_fid,
            mean_distillation_purity: sum_purity / n,
            min_distillation_purity: min_purity,
            mean_waveguide_loss_db_per_cm: sum_loss / n,
            max_waveguide_loss_db_per_cm: max_loss,
            mean_quantum_memory_t2_ms: sum_t2 / n,
            min_quantum_memory_t2_ms: min_t2,
            mean_bell_concurrence: sum_conc / n,
            min_bell_concurrence: min_conc,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
