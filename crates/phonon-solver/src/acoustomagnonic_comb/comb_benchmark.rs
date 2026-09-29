#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for chiral phonon-magnon polariton
//! frequency combs and quantum topological acoustomagnonics across Rayon workers.

use crate::acoustomagnonic_comb::AcoustomagnonicCombSolver;
use phonon_models::acoustomagnonic_comb::{
    AcoustomagnonicCombMetrics, AcoustomagnonicCombParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for acoustomagnonic comb parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_comb_spectral_span_ghz: f64,
    pub min_comb_spectral_span_ghz: f64,
    pub max_comb_spectral_span_ghz: f64,
    pub mean_phase_noise_at_10khz_dbc: f64,
    pub min_phase_noise_at_10khz_dbc: f64,
    pub max_phase_noise_at_10khz_dbc: f64,
    pub mean_polariton_conversion_efficiency: f64,
    pub min_polariton_conversion_efficiency: f64,
    pub max_polariton_conversion_efficiency: f64,
    pub mean_inter_modal_isolation_db: f64,
    pub min_inter_modal_isolation_db: f64,
    pub max_inter_modal_isolation_db: f64,
    pub mean_polariton_cooperativity: f64,
    pub min_polariton_cooperativity: f64,
    pub max_polariton_cooperativity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct CombBenchmarkRunner;

impl CombBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> CombBenchmarkResult {
        let sweep_params: Vec<AcoustomagnonicCombParams> = (0..cycles)
            .map(|i| {
                let pump_frequency_ghz = 12.0 + 4.0 * ((i % 17) as f64 / 17.0);
                let magnetoelastic_coupling_mhz = 82.0 + 12.0 * ((i % 19) as f64 / 19.0);
                let kerr_nonlinearity_khz = 12.0 + 6.0 * ((i % 23) as f64 / 23.0);
                let gilbert_damping_alpha = 1.0e-4 + 0.12e-4 * ((i % 29) as f64 / 29.0);
                let acoustic_loss_rate_mhz = 0.30 + 0.04 * ((i % 31) as f64 / 31.0);
                let operating_temp_m_k = 15.0 + 4.5 * ((i % 37) as f64 / 37.0);
                let rf_drive_power_mw = 24.0 + 6.0 * ((i % 41) as f64 / 41.0);
                let chiral_asymmetry_ratio = 0.88 + 0.06 * ((i % 43) as f64 / 43.0);

                AcoustomagnonicCombParams::new(
                    pump_frequency_ghz,
                    magnetoelastic_coupling_mhz,
                    kerr_nonlinearity_khz,
                    gilbert_damping_alpha,
                    acoustic_loss_rate_mhz,
                    operating_temp_m_k,
                    rf_drive_power_mw,
                    chiral_asymmetry_ratio,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<AcoustomagnonicCombMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = AcoustomagnonicCombSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_span = 0.0;
        let mut min_span = f64::MAX;
        let mut max_span = f64::MIN;

        let mut sum_pn = 0.0;
        let mut min_pn = f64::MAX;
        let mut max_pn = f64::MIN;

        let mut sum_eff = 0.0;
        let mut min_eff = f64::MAX;
        let mut max_eff = f64::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut max_iso = f64::MIN;

        let mut sum_coop = 0.0;
        let mut min_coop = f64::MAX;
        let mut max_coop = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_span += m.comb_spectral_span_ghz;
            if m.comb_spectral_span_ghz < min_span {
                min_span = m.comb_spectral_span_ghz;
            }
            if m.comb_spectral_span_ghz > max_span {
                max_span = m.comb_spectral_span_ghz;
            }

            sum_pn += m.phase_noise_at_10khz_dbc;
            if m.phase_noise_at_10khz_dbc < min_pn {
                min_pn = m.phase_noise_at_10khz_dbc;
            }
            if m.phase_noise_at_10khz_dbc > max_pn {
                max_pn = m.phase_noise_at_10khz_dbc;
            }

            sum_eff += m.polariton_conversion_efficiency;
            if m.polariton_conversion_efficiency < min_eff {
                min_eff = m.polariton_conversion_efficiency;
            }
            if m.polariton_conversion_efficiency > max_eff {
                max_eff = m.polariton_conversion_efficiency;
            }

            sum_iso += m.inter_modal_isolation_db;
            if m.inter_modal_isolation_db < min_iso {
                min_iso = m.inter_modal_isolation_db;
            }
            if m.inter_modal_isolation_db > max_iso {
                max_iso = m.inter_modal_isolation_db;
            }

            sum_coop += m.polariton_cooperativity;
            if m.polariton_cooperativity < min_coop {
                min_coop = m.polariton_cooperativity;
            }
            if m.polariton_cooperativity > max_coop {
                max_coop = m.polariton_cooperativity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let count = cycles.max(1) as f64;
        let compliance_fraction = (compliant_count as f64) / count;

        CombBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_comb_spectral_span_ghz: sum_span / count,
            min_comb_spectral_span_ghz: min_span,
            max_comb_spectral_span_ghz: max_span,
            mean_phase_noise_at_10khz_dbc: sum_pn / count,
            min_phase_noise_at_10khz_dbc: min_pn,
            max_phase_noise_at_10khz_dbc: max_pn,
            mean_polariton_conversion_efficiency: sum_eff / count,
            min_polariton_conversion_efficiency: min_eff,
            max_polariton_conversion_efficiency: max_eff,
            mean_inter_modal_isolation_db: sum_iso / count,
            min_inter_modal_isolation_db: min_iso,
            max_inter_modal_isolation_db: max_iso,
            mean_polariton_cooperativity: sum_coop / count,
            min_polariton_cooperativity: min_coop,
            max_polariton_cooperativity: max_coop,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
