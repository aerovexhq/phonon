#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for topological acoustic higher-order
//! corner mode lasers and non-Hermitian phonon cavities across Rayon workers.

use crate::topological_corner_laser::TopologicalCornerLaserSolver;
use phonon_models::topological_corner_laser::{
    TopologicalCornerLaserMetrics, TopologicalCornerLaserParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for topological corner laser parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerLaserBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_corner_lasing_efficiency: f64,
    pub min_corner_lasing_efficiency: f64,
    pub max_corner_lasing_efficiency: f64,
    pub mean_threshold_power_uw: f64,
    pub min_threshold_power_uw: f64,
    pub max_threshold_power_uw: f64,
    pub mean_corner_mode_localization: f64,
    pub min_corner_mode_localization: f64,
    pub max_corner_mode_localization: f64,
    pub mean_mode_discrimination_db: f64,
    pub min_mode_discrimination_db: f64,
    pub max_mode_discrimination_db: f64,
    pub mean_emission_linewidth_khz: f64,
    pub min_emission_linewidth_khz: f64,
    pub max_emission_linewidth_khz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct CornerLaserBenchmarkRunner;

impl CornerLaserBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> CornerLaserBenchmarkResult {
        let sweep_params: Vec<TopologicalCornerLaserParams> = (0..cycles)
            .map(|i| {
                let acoustic_frequency_ghz = 4.0 + 1.5 * ((i % 23) as f64 / 23.0);
                let inter_cell_hopping_mhz = 42.0 + 12.0 * ((i % 29) as f64 / 29.0);
                let intra_cell_hopping_mhz = 10.0 + 4.0 * ((i % 31) as f64 / 31.0);
                let optical_pump_power_uw = 22.0 + 10.0 * ((i % 37) as f64 / 37.0);
                let non_hermitian_gain_mhz = 14.0 + 4.0 * ((i % 19) as f64 / 19.0);
                let acoustic_loss_rate_mhz = 2.2 + 0.6 * ((i % 41) as f64 / 41.0);
                let operating_temp_m_k = 12.0 + 6.0 * ((i % 43) as f64 / 43.0);
                let disorder_amplitude_percent = 1.0 + 2.0 * ((i % 17) as f64 / 17.0);

                TopologicalCornerLaserParams::new(
                    acoustic_frequency_ghz,
                    inter_cell_hopping_mhz,
                    intra_cell_hopping_mhz,
                    optical_pump_power_uw,
                    non_hermitian_gain_mhz,
                    acoustic_loss_rate_mhz,
                    operating_temp_m_k,
                    disorder_amplitude_percent,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<TopologicalCornerLaserMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TopologicalCornerLaserSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_eff = 0.0;
        let mut min_eff = f64::MAX;
        let mut max_eff = f64::MIN;

        let mut sum_pth = 0.0;
        let mut min_pth = f64::MAX;
        let mut max_pth = f64::MIN;

        let mut sum_loc = 0.0;
        let mut min_loc = f64::MAX;
        let mut max_loc = f64::MIN;

        let mut sum_md = 0.0;
        let mut min_md = f64::MAX;
        let mut max_md = f64::MIN;

        let mut sum_lw = 0.0;
        let mut min_lw = f64::MAX;
        let mut max_lw = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_eff += m.corner_lasing_efficiency;
            if m.corner_lasing_efficiency < min_eff {
                min_eff = m.corner_lasing_efficiency;
            }
            if m.corner_lasing_efficiency > max_eff {
                max_eff = m.corner_lasing_efficiency;
            }

            sum_pth += m.threshold_power_uw;
            if m.threshold_power_uw < min_pth {
                min_pth = m.threshold_power_uw;
            }
            if m.threshold_power_uw > max_pth {
                max_pth = m.threshold_power_uw;
            }

            sum_loc += m.corner_mode_localization;
            if m.corner_mode_localization < min_loc {
                min_loc = m.corner_mode_localization;
            }
            if m.corner_mode_localization > max_loc {
                max_loc = m.corner_mode_localization;
            }

            sum_md += m.mode_discrimination_db;
            if m.mode_discrimination_db < min_md {
                min_md = m.mode_discrimination_db;
            }
            if m.mode_discrimination_db > max_md {
                max_md = m.mode_discrimination_db;
            }

            sum_lw += m.emission_linewidth_khz;
            if m.emission_linewidth_khz < min_lw {
                min_lw = m.emission_linewidth_khz;
            }
            if m.emission_linewidth_khz > max_lw {
                max_lw = m.emission_linewidth_khz;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let count = cycles.max(1) as f64;
        let compliance_fraction = (compliant_count as f64) / count;

        CornerLaserBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_corner_lasing_efficiency: sum_eff / count,
            min_corner_lasing_efficiency: min_eff,
            max_corner_lasing_efficiency: max_eff,
            mean_threshold_power_uw: sum_pth / count,
            min_threshold_power_uw: min_pth,
            max_threshold_power_uw: max_pth,
            mean_corner_mode_localization: sum_loc / count,
            min_corner_mode_localization: min_loc,
            max_corner_mode_localization: max_loc,
            mean_mode_discrimination_db: sum_md / count,
            min_mode_discrimination_db: min_md,
            max_mode_discrimination_db: max_md,
            mean_emission_linewidth_khz: sum_lw / count,
            min_emission_linewidth_khz: min_lw,
            max_emission_linewidth_khz: max_lw,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
