#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for cavity quantum acoustomagnonic polariton
//! condensation and chiral superfluid spin-phonon lasers across multi-threaded Rayon workers.

use crate::acoustomagnonic_polariton_laser::AcoustomagnonicPolaritonLaserSolver;
use phonon_models::acoustomagnonic_polariton_laser::{
    AcoustomagnonicPolaritonLaserMetrics, AcoustomagnonicPolaritonLaserParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for acoustomagnonic polariton laser parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonLaserBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_polariton_condensation_threshold_uw: f64,
    pub min_polariton_condensation_threshold_uw: f64,
    pub max_polariton_condensation_threshold_uw: f64,
    pub mean_condensate_coherence_lifetime_us: f64,
    pub min_condensate_coherence_lifetime_us: f64,
    pub max_condensate_coherence_lifetime_us: f64,
    pub mean_side_mode_suppression_ratio_db: f64,
    pub min_side_mode_suppression_ratio_db: f64,
    pub max_side_mode_suppression_ratio_db: f64,
    pub mean_linewidth_narrowing_factor: f64,
    pub min_linewidth_narrowing_factor: f64,
    pub max_linewidth_narrowing_factor: f64,
    pub mean_polariton_superfluid_fraction: f64,
    pub min_polariton_superfluid_fraction: f64,
    pub max_polariton_superfluid_fraction: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct PolaritonLaserBenchmarkRunner;

impl PolaritonLaserBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> PolaritonLaserBenchmarkResult {
        let sweep_params: Vec<AcoustomagnonicPolaritonLaserParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let magnon_kittel_ghz = 8.0 + 1.0 * frac; // 8.0 to 9.0 GHz
                let acoustic_freq_ghz = 8.0 + 1.0 * (((i * 7) % 50) as f64 / 50.0); // 8.0 to 9.0 GHz
                let coupling_mhz = 28.0 + 35.0 * frac; // 28.0 to 63.0 MHz
                let pump_power_uw = 15.0 + 40.0 * (((i * 13) % 50) as f64 / 50.0); // 15.0 to 55.0 uW
                let magnon_damping_mhz = 1.2 + 2.5 * (((i * 11) % 45) as f64 / 45.0); // 1.2 to 3.7 MHz
                let acoustic_decay_khz = 40.0 + 90.0 * frac; // 40.0 to 130.0 kHz
                let cryo_temp_mk = 5.0 + 20.0 * (((i * 17) % 40) as f64 / 40.0); // 5.0 to 25.0 mK
                let kerr_hz = 6.0 + 18.0 * frac; // 6.0 to 24.0 Hz

                AcoustomagnonicPolaritonLaserParams::new(
                    magnon_kittel_ghz,
                    acoustic_freq_ghz,
                    coupling_mhz,
                    pump_power_uw,
                    magnon_damping_mhz,
                    acoustic_decay_khz,
                    cryo_temp_mk,
                    kerr_hz,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<AcoustomagnonicPolaritonLaserMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = AcoustomagnonicPolaritonLaserSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_threshold = 0.0;
        let mut min_threshold = f64::MAX;
        let mut max_threshold = f64::MIN;

        let mut sum_coherence = 0.0;
        let mut min_coherence = f64::MAX;
        let mut max_coherence = f64::MIN;

        let mut sum_smsr = 0.0;
        let mut min_smsr = f64::MAX;
        let mut max_smsr = f64::MIN;

        let mut sum_narrowing = 0.0;
        let mut min_narrowing = f64::MAX;
        let mut max_narrowing = f64::MIN;

        let mut sum_superfluid = 0.0;
        let mut min_superfluid = f64::MAX;
        let mut max_superfluid = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_threshold += m.polariton_condensation_threshold_uw;
            if m.polariton_condensation_threshold_uw < min_threshold {
                min_threshold = m.polariton_condensation_threshold_uw;
            }
            if m.polariton_condensation_threshold_uw > max_threshold {
                max_threshold = m.polariton_condensation_threshold_uw;
            }

            sum_coherence += m.condensate_coherence_lifetime_us;
            if m.condensate_coherence_lifetime_us < min_coherence {
                min_coherence = m.condensate_coherence_lifetime_us;
            }
            if m.condensate_coherence_lifetime_us > max_coherence {
                max_coherence = m.condensate_coherence_lifetime_us;
            }

            sum_smsr += m.side_mode_suppression_ratio_db;
            if m.side_mode_suppression_ratio_db < min_smsr {
                min_smsr = m.side_mode_suppression_ratio_db;
            }
            if m.side_mode_suppression_ratio_db > max_smsr {
                max_smsr = m.side_mode_suppression_ratio_db;
            }

            sum_narrowing += m.linewidth_narrowing_factor;
            if m.linewidth_narrowing_factor < min_narrowing {
                min_narrowing = m.linewidth_narrowing_factor;
            }
            if m.linewidth_narrowing_factor > max_narrowing {
                max_narrowing = m.linewidth_narrowing_factor;
            }

            sum_superfluid += m.polariton_superfluid_fraction;
            if m.polariton_superfluid_fraction < min_superfluid {
                min_superfluid = m.polariton_superfluid_fraction;
            }
            if m.polariton_superfluid_fraction > max_superfluid {
                max_superfluid = m.polariton_superfluid_fraction;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        PolaritonLaserBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_polariton_condensation_threshold_uw: sum_threshold / n,
            min_polariton_condensation_threshold_uw: min_threshold,
            max_polariton_condensation_threshold_uw: max_threshold,
            mean_condensate_coherence_lifetime_us: sum_coherence / n,
            min_condensate_coherence_lifetime_us: min_coherence,
            max_condensate_coherence_lifetime_us: max_coherence,
            mean_side_mode_suppression_ratio_db: sum_smsr / n,
            min_side_mode_suppression_ratio_db: min_smsr,
            max_side_mode_suppression_ratio_db: max_smsr,
            mean_linewidth_narrowing_factor: sum_narrowing / n,
            min_linewidth_narrowing_factor: min_narrowing,
            max_linewidth_narrowing_factor: max_narrowing,
            mean_polariton_superfluid_fraction: sum_superfluid / n,
            min_polariton_superfluid_fraction: min_superfluid,
            max_polariton_superfluid_fraction: max_superfluid,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
