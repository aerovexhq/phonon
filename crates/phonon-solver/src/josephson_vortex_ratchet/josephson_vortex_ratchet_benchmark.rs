//! Parallel parameter sweep benchmark suite for quantum acoustoelectric Josephson vortex ratchets.

use crate::josephson_vortex_ratchet::JosephsonVortexRatchetSolver;
use phonon_models::josephson_vortex_ratchet::{
    JosephsonVortexRatchetMetrics, JosephsonVortexRatchetParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for Josephson vortex ratchet sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JosephsonVortexRatchetBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_rectification_efficiency: f64,
    pub min_rectification_efficiency: f64,
    pub mean_soliton_velocity: f64,
    pub min_soliton_velocity: f64,
    pub mean_threshold_power_uw: f64,
    pub max_threshold_power_uw: f64,
    pub mean_locking_precision: f64,
    pub max_locking_precision: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct JosephsonVortexRatchetBenchmarkRunner;

impl JosephsonVortexRatchetBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> JosephsonVortexRatchetBenchmarkResult {
        let sweep_params: Vec<JosephsonVortexRatchetParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let l_um = 180.0 + 150.0 * frac; // 180 to 330 um
                let lambda_j_um = 18.0 + 10.0 * ((i % 50) as f64 / 50.0); // 18 to 28 um
                let c_sw = 1.0e7 + 4.0e6 * frac; // 1.0e7 to 1.4e7 m/s
                let f_saw_ghz = 2.0 + 1.5 * ((i % 60) as f64 / 60.0); // 2.0 to 3.5 GHz
                let eps = 0.30 + 0.15 * frac; // 0.30 to 0.45
                let p_ac_uw = 0.25 + 0.30 * ((i % 80) as f64 / 80.0); // 0.25 to 0.55 uW
                let alpha = 0.015 + 0.020 * ((i % 40) as f64 / 40.0); // 0.015 to 0.035
                let j_c = 800.0 + 800.0 * frac; // 800 to 1600 A/cm^2
                let temp_k = 0.020 + 0.030 * ((i % 70) as f64 / 70.0); // 20 to 50 mK

                JosephsonVortexRatchetParams::new(
                    l_um,
                    lambda_j_um,
                    c_sw,
                    f_saw_ghz,
                    eps,
                    p_ac_uw,
                    alpha,
                    j_c,
                    temp_k,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<JosephsonVortexRatchetMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = JosephsonVortexRatchetSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_eta = 0.0;
        let mut min_eta = f64::MAX;
        let mut sum_v = 0.0;
        let mut min_v = f64::MAX;
        let mut sum_pth = 0.0;
        let mut max_pth = 0.0;
        let mut sum_lock = 0.0;
        let mut max_lock = 0.0;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_eta += m.ratchet_rectification_efficiency;
            if m.ratchet_rectification_efficiency < min_eta {
                min_eta = m.ratchet_rectification_efficiency;
            }

            sum_v += m.normalized_soliton_velocity;
            if m.normalized_soliton_velocity < min_v {
                min_v = m.normalized_soliton_velocity;
            }

            sum_pth += m.acoustic_threshold_power_uw;
            if m.acoustic_threshold_power_uw > max_pth {
                max_pth = m.acoustic_threshold_power_uw;
            }

            sum_lock += m.phase_slip_locking_precision;
            if m.phase_slip_locking_precision > max_lock {
                max_lock = m.phase_slip_locking_precision;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        JosephsonVortexRatchetBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_rectification_efficiency: sum_eta / n,
            min_rectification_efficiency: min_eta,
            mean_soliton_velocity: sum_v / n,
            min_soliton_velocity: min_v,
            mean_threshold_power_uw: sum_pth / n,
            max_threshold_power_uw: max_pth,
            mean_locking_precision: sum_lock / n,
            max_locking_precision: max_lock,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
