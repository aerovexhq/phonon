//! Parallel parameter sweep benchmark suite for cavity magnon-polariton frequency combs.

use crate::cavity_magnon_polariton_comb::CavityMagnonPolaritonCombSolver;
use phonon_models::cavity_magnon_polariton_comb::{
    MagnonPolaritonCombMetrics, MagnonPolaritonCombParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for magnon-polariton comb sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnonPolaritonCombBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_threshold_power_mw: f64,
    pub max_threshold_power_mw: f64,
    pub mean_comb_octave_span: f64,
    pub min_comb_octave_span: f64,
    pub mean_sub_shot_noise_db: f64,
    pub min_sub_shot_noise_db: f64,
    pub mean_polariton_log_negativity: f64,
    pub min_polariton_log_negativity: f64,
    pub mean_comb_teeth_count: f64,
    pub min_comb_teeth_count: usize,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct MagnonPolaritonCombBenchmarkRunner;

impl MagnonPolaritonCombBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> MagnonPolaritonCombBenchmarkResult {
        let sweep_params: Vec<MagnonPolaritonCombParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let f_c_ghz = 8.0 + 6.0 * frac; // 8.0 to 14.0 GHz
                let f_b_mhz = 15.0 + 35.0 * ((i % 100) as f64 / 100.0); // 15 to 50 MHz
                let k_hz = 0.40 + 0.80 * ((i % 60) as f64 / 60.0); // 0.40 to 1.20 Hz
                let g_mb_mhz = 10.0 + 10.0 * frac; // 10.0 to 20.0 MHz
                let p_pump_mw = 1.20 + 2.00 * ((i % 80) as f64 / 80.0); // 1.20 to 3.20 mW
                let kappa_c_mhz = 0.80 + 0.80 * frac; // 0.80 to 1.60 MHz
                let kappa_m_mhz = 0.60 + 0.60 * ((i % 50) as f64 / 50.0); // 0.60 to 1.20 MHz
                let r = 0.75 + 0.45 * ((i % 70) as f64 / 70.0); // 0.75 to 1.20
                let temp_k = 0.015 + 0.025 * frac; // 15 to 40 mK

                MagnonPolaritonCombParams::new(
                    f_c_ghz,
                    f_b_mhz,
                    k_hz,
                    g_mb_mhz,
                    p_pump_mw,
                    kappa_c_mhz,
                    kappa_m_mhz,
                    r,
                    temp_k,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<MagnonPolaritonCombMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = CavityMagnonPolaritonCombSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_pth = 0.0;
        let mut max_pth = 0.0;
        let mut sum_oct = 0.0;
        let mut min_oct = f64::MAX;
        let mut sum_ssn = 0.0;
        let mut min_ssn = f64::MAX;
        let mut sum_en = 0.0;
        let mut min_en = f64::MAX;
        let mut sum_teeth = 0.0;
        let mut min_teeth = usize::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_pth += m.comb_threshold_power_mw;
            if m.comb_threshold_power_mw > max_pth {
                max_pth = m.comb_threshold_power_mw;
            }

            sum_oct += m.comb_octave_span;
            if m.comb_octave_span < min_oct {
                min_oct = m.comb_octave_span;
            }

            sum_ssn += m.sub_shot_noise_improvement_db;
            if m.sub_shot_noise_improvement_db < min_ssn {
                min_ssn = m.sub_shot_noise_improvement_db;
            }

            sum_en += m.polariton_log_negativity;
            if m.polariton_log_negativity < min_en {
                min_en = m.polariton_log_negativity;
            }

            sum_teeth += m.comb_teeth_count as f64;
            if m.comb_teeth_count < min_teeth {
                min_teeth = m.comb_teeth_count;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        MagnonPolaritonCombBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_threshold_power_mw: sum_pth / n,
            max_threshold_power_mw: max_pth,
            mean_comb_octave_span: sum_oct / n,
            min_comb_octave_span: min_oct,
            mean_sub_shot_noise_db: sum_ssn / n,
            min_sub_shot_noise_db: min_ssn,
            mean_polariton_log_negativity: sum_en / n,
            min_polariton_log_negativity: min_en,
            mean_comb_teeth_count: sum_teeth / n,
            min_comb_teeth_count: min_teeth,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
