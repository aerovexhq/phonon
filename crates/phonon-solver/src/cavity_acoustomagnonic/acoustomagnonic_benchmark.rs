//! Parallel parameter sweep benchmark suite for cavity acoustomagnonics.

use crate::cavity_acoustomagnonic::CavityAcoustomagnonicSolver;
use phonon_models::cavity_acoustomagnonic::{AcoustomagnonicMetrics, AcoustomagnonicParams};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for cavity acoustomagnonic sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomagnonicBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_cooperativity: f64,
    pub min_cooperativity: f64,
    pub mean_conversion_gain_db: f64,
    pub min_conversion_gain_db: f64,
    pub mean_snr_db: f64,
    pub min_snr_db: f64,
    pub mean_scan_rate_ghz_per_day: f64,
    pub min_scan_rate_ghz_per_day: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct AcoustomagnonicBenchmarkRunner;

impl AcoustomagnonicBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> AcoustomagnonicBenchmarkResult {
        let sweep_params: Vec<AcoustomagnonicParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let f_c = 8.0 + 6.0 * frac; // 8.0 to 14.0 GHz
                let f_m = f_c + 0.05 * (frac - 0.5); // near-resonant Kittel mode
                let f_b = f_c; // acoustic mode on resonance
                let g_cm = 20.0 + 10.0 * ((i % 50) as f64 / 50.0); // 20 to 30 MHz
                let g_ma = 10.0 + 6.0 * ((i % 60) as f64 / 60.0); // 10 to 16 MHz
                let q_c = 1.0e5 + 1.0e5 * frac; // 1.0e5 to 2.0e5
                let alpha = 1.0e-4 + 0.5e-4 * (1.0 - frac); // 1.0e-4 to 1.5e-4
                let q_b = 2.0e7 + 1.0e7 * ((i % 80) as f64 / 80.0); // 2.0e7 to 3.0e7
                let d_um = 400.0 + 200.0 * ((i % 70) as f64 / 70.0); // 400 to 600 um
                let t_mk = 15.0 + 20.0 * ((i % 40) as f64 / 40.0); // 15 to 35 mK
                let gain_db = 20.0 + 5.0 * frac; // 20 to 25 dB
                let hemt_k = 2.5 + 0.6 * (1.0 - frac); // 2.5 to 3.1 K
                let tau_s = 0.8 + 0.4 * ((i % 90) as f64 / 90.0); // 0.8 to 1.2 s

                AcoustomagnonicParams::new(
                    f_c, f_m, f_b, g_cm, g_ma, q_c, alpha, q_b, d_um, t_mk, gain_db, hemt_k, tau_s,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<AcoustomagnonicMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = CavityAcoustomagnonicSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_coop = 0.0;
        let mut min_coop = f64::MAX;
        let mut sum_gain = 0.0;
        let mut min_gain = f64::MAX;
        let mut sum_snr = 0.0;
        let mut min_snr = f64::MAX;
        let mut sum_rate = 0.0;
        let mut min_rate = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_coop += m.acoustomagnonic_cooperativity;
            if m.acoustomagnonic_cooperativity < min_coop {
                min_coop = m.acoustomagnonic_cooperativity;
            }

            sum_gain += m.conversion_gain_db;
            if m.conversion_gain_db < min_gain {
                min_gain = m.conversion_gain_db;
            }

            sum_snr += m.haloscope_readout_snr_db;
            if m.haloscope_readout_snr_db < min_snr {
                min_snr = m.haloscope_readout_snr_db;
            }

            sum_rate += m.exclusion_scan_rate_ghz_per_day;
            if m.exclusion_scan_rate_ghz_per_day < min_rate {
                min_rate = m.exclusion_scan_rate_ghz_per_day;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        AcoustomagnonicBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_cooperativity: sum_coop / n,
            min_cooperativity: min_coop,
            mean_conversion_gain_db: sum_gain / n,
            min_conversion_gain_db: min_gain,
            mean_snr_db: sum_snr / n,
            min_snr_db: min_snr,
            mean_scan_rate_ghz_per_day: sum_rate / n,
            min_scan_rate_ghz_per_day: min_rate,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
