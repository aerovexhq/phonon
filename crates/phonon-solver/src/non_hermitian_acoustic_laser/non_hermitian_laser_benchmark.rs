//! Parallel parameter sweep benchmark suite for non-Hermitian topological acoustic lasers.

use crate::non_hermitian_acoustic_laser::NonHermitianAcousticLaserSolver;
use phonon_models::non_hermitian_acoustic_laser::{
    NonHermitianLaserMetrics, NonHermitianLaserParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for non-Hermitian acoustic laser sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianLaserBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_skin_localization: f64,
    pub min_skin_localization: f64,
    pub mean_laser_smsr_db: f64,
    pub min_laser_smsr_db: f64,
    pub mean_laser_output_power_mw: f64,
    pub min_laser_output_power_mw: f64,
    pub mean_non_reciprocal_isolation_db: f64,
    pub min_non_reciprocal_isolation_db: f64,
    pub mean_corner_fidelity: f64,
    pub min_corner_fidelity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct NonHermitianLaserBenchmarkRunner;

impl NonHermitianLaserBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> NonHermitianLaserBenchmarkResult {
        let sweep_params: Vec<NonHermitianLaserParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let n = 16 + (i % 16); // lattice size 16 to 31
                let f_0_mhz = 350.0 + 300.0 * frac; // 350 to 650 MHz
                let t_r = 1.80 + 0.80 * ((i % 50) as f64 / 50.0); // 1.80 to 2.60
                let t_l = 0.040 + 0.040 * ((i % 60) as f64 / 60.0); // 0.040 to 0.080
                let gamma = 0.65 + 0.40 * frac; // 0.65 to 1.05
                let omega_p_mhz = 35.0 + 25.0 * ((i % 70) as f64 / 70.0); // 35 to 60 MHz
                let p_mw = 25.0 + 35.0 * frac; // 25 to 60 mW
                let q_cav = 4.0e3 + 6.0e3 * ((i % 80) as f64 / 80.0); // 4e3 to 10e3
                let temp_k = 280.0 + 30.0 * ((i % 40) as f64 / 40.0); // 280 to 310 K

                NonHermitianLaserParams::new(
                    n,
                    f_0_mhz,
                    t_r,
                    t_l,
                    gamma,
                    omega_p_mhz,
                    p_mw,
                    q_cav,
                    temp_k,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<NonHermitianLaserMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = NonHermitianAcousticLaserSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_skin = 0.0;
        let mut min_skin = f64::MAX;
        let mut sum_smsr = 0.0;
        let mut min_smsr = f64::MAX;
        let mut sum_pwr = 0.0;
        let mut min_pwr = f64::MAX;
        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_skin += m.skin_mode_localization_ratio;
            if m.skin_mode_localization_ratio < min_skin {
                min_skin = m.skin_mode_localization_ratio;
            }

            sum_smsr += m.laser_smsr_db;
            if m.laser_smsr_db < min_smsr {
                min_smsr = m.laser_smsr_db;
            }

            sum_pwr += m.laser_output_power_mw;
            if m.laser_output_power_mw < min_pwr {
                min_pwr = m.laser_output_power_mw;
            }

            sum_iso += m.non_reciprocal_isolation_db;
            if m.non_reciprocal_isolation_db < min_iso {
                min_iso = m.non_reciprocal_isolation_db;
            }

            sum_fid += m.corner_mode_fidelity;
            if m.corner_mode_fidelity < min_fid {
                min_fid = m.corner_mode_fidelity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        NonHermitianLaserBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_skin_localization: sum_skin / n,
            min_skin_localization: min_skin,
            mean_laser_smsr_db: sum_smsr / n,
            min_laser_smsr_db: min_smsr,
            mean_laser_output_power_mw: sum_pwr / n,
            min_laser_output_power_mw: min_pwr,
            mean_non_reciprocal_isolation_db: sum_iso / n,
            min_non_reciprocal_isolation_db: min_iso,
            mean_corner_fidelity: sum_fid / n,
            min_corner_fidelity: min_fid,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
