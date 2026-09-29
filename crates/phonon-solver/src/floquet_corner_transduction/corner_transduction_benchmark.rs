//! Parallel parameter sweep benchmark suite for Floquet corner state transduction.

use crate::floquet_corner_transduction::FloquetCornerTransductionSolver;
use phonon_models::floquet_corner_transduction::{
    CornerTransductionMetrics, CornerTransductionParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for Floquet corner transduction parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerTransductionBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_purity: f64,
    pub min_purity: f64,
    pub mean_efficiency: f64,
    pub min_efficiency: f64,
    pub mean_noise_photons: f64,
    pub max_noise_photons: f64,
    pub mean_quality_factor: f64,
    pub min_quality_factor: f64,
    pub mean_topological_invariant: f64,
    pub min_topological_invariant: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct CornerTransductionBenchmarkRunner;

impl CornerTransductionBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> CornerTransductionBenchmarkResult {
        let sweep_params: Vec<CornerTransductionParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let a = 1.2 + 1.2 * frac; // 1.2 to 2.4 um
                let f_m = 3.5 + 2.0 * ((i % 50) as f64 / 50.0); // 3.5 to 5.5 GHz
                let ratio = 1.8 + 1.6 * frac; // 1.8 to 3.4
                let c_em = 24.0 + 10.0 * ((i % 80) as f64 / 80.0); // 24.0 to 34.0
                let c_om = 22.0 + 10.0 * ((i % 70) as f64 / 70.0); // 22.0 to 32.0
                let q_m = 2.0e5 + 2.0e5 * frac; // 2.0e5 to 4.0e5
                let kappa = 35.0 + 25.0 * ((i % 40) as f64 / 40.0); // 35.0 to 60.0 MHz
                let temp_mk = 12.0 + 16.0 * ((i % 60) as f64 / 60.0); // 12.0 to 28.0 mK

                CornerTransductionParams::new(
                    a, f_m, ratio, c_em, c_om, q_m, kappa, temp_mk,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<CornerTransductionMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = FloquetCornerTransductionSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;
        let mut sum_eff = 0.0;
        let mut min_eff = f64::MAX;
        let mut sum_noise = 0.0;
        let mut max_noise = 0.0;
        let mut sum_q = 0.0;
        let mut min_q = f64::MAX;
        let mut sum_topo = 0.0;
        let mut min_topo = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_purity += m.corner_mode_localization_purity;
            if m.corner_mode_localization_purity < min_purity {
                min_purity = m.corner_mode_localization_purity;
            }

            sum_eff += m.bidirectional_transduction_efficiency;
            if m.bidirectional_transduction_efficiency < min_eff {
                min_eff = m.bidirectional_transduction_efficiency;
            }

            sum_noise += m.added_noise_photons;
            if m.added_noise_photons > max_noise {
                max_noise = m.added_noise_photons;
            }

            sum_q += m.corner_acoustic_quality_factor;
            if m.corner_acoustic_quality_factor < min_q {
                min_q = m.corner_acoustic_quality_factor;
            }

            sum_topo += m.quadrupole_topological_invariant;
            if m.quadrupole_topological_invariant < min_topo {
                min_topo = m.quadrupole_topological_invariant;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        CornerTransductionBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_purity: sum_purity / n,
            min_purity,
            mean_efficiency: sum_eff / n,
            min_efficiency: min_eff,
            mean_noise_photons: sum_noise / n,
            max_noise_photons: max_noise,
            mean_quality_factor: sum_q / n,
            min_quality_factor: min_q,
            mean_topological_invariant: sum_topo / n,
            min_topological_invariant: min_topo,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
