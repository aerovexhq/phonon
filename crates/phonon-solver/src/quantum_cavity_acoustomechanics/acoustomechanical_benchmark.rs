//! Parallel parameter sweep benchmark suite for quantum cavity acoustomechanics.

use crate::quantum_cavity_acoustomechanics::QuantumCavityAcoustomechanicalSolver;
use phonon_models::quantum_cavity_acoustomechanics::{
    AcoustomechanicalSqueezingMetrics, AcoustomechanicalSqueezingParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for acoustomechanical squeezing sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomechanicalBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_squeezing_db: f64,
    pub min_squeezing_db: f64,
    pub mean_qnd_fidelity: f64,
    pub min_qnd_fidelity: f64,
    pub mean_decoherence_rate_hz: f64,
    pub max_decoherence_rate_hz: f64,
    pub mean_photons: f64,
    pub min_photons: f64,
    pub mean_bae_purity: f64,
    pub min_bae_purity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct AcoustomechanicalBenchmarkRunner;

impl AcoustomechanicalBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> AcoustomechanicalBenchmarkResult {
        let sweep_params: Vec<AcoustomechanicalSqueezingParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let f_m = 10.0 + 15.0 * frac; // 10.0 to 25.0 MHz
                let f_c = 190.0 + 5.0 * ((i % 50) as f64 / 50.0); // 190.0 to 195.0 THz
                let g0 = 180.0 + 180.0 * ((i % 100) as f64 / 100.0); // 180 to 360 Hz
                let n_c = 5.5e5 + 2.5e5 * frac; // 5.5e5 to 8.0e5
                let q_m = 8.0e7 + 1.2e8 * ((i % 75) as f64 / 75.0); // 0.8e8 to 2.0e8
                let kappa = 1.8 + 1.8 * frac; // 1.8 to 3.6 MHz
                let imb = 0.001 + 0.005 * ((i % 40) as f64 / 40.0); // 0.001 to 0.006
                let temp_mk = 10.0 + 15.0 * ((i % 60) as f64 / 60.0); // 10.0 to 25.0 mK

                AcoustomechanicalSqueezingParams::new(
                    f_m, f_c, g0, n_c, q_m, kappa, imb, temp_mk,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<AcoustomechanicalSqueezingMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = QuantumCavityAcoustomechanicalSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_sq = 0.0;
        let mut min_sq = f64::MAX;
        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut sum_decoh = 0.0;
        let mut max_decoh = 0.0;
        let mut sum_photons = 0.0;
        let mut min_photons = f64::MAX;
        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_sq += m.ponderomotive_squeezing_db;
            if m.ponderomotive_squeezing_db < min_sq {
                min_sq = m.ponderomotive_squeezing_db;
            }

            sum_fid += m.qnd_measurement_fidelity;
            if m.qnd_measurement_fidelity < min_fid {
                min_fid = m.qnd_measurement_fidelity;
            }

            sum_decoh += m.mechanical_decoherence_rate_hz;
            if m.mechanical_decoherence_rate_hz > max_decoh {
                max_decoh = m.mechanical_decoherence_rate_hz;
            }

            sum_photons += m.intracavity_photon_number;
            if m.intracavity_photon_number < min_photons {
                min_photons = m.intracavity_photon_number;
            }

            sum_purity += m.backaction_evasion_purity;
            if m.backaction_evasion_purity < min_purity {
                min_purity = m.backaction_evasion_purity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        AcoustomechanicalBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_squeezing_db: sum_sq / n,
            min_squeezing_db: min_sq,
            mean_qnd_fidelity: sum_fid / n,
            min_qnd_fidelity: min_fid,
            mean_decoherence_rate_hz: sum_decoh / n,
            max_decoherence_rate_hz: max_decoh,
            mean_photons: sum_photons / n,
            min_photons: min_photons,
            mean_bae_purity: sum_purity / n,
            min_bae_purity: min_purity,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
