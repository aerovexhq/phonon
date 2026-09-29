#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Hermitian skin-topological phonon
//! diodes and unidirectional quantum acoustic amplifiers across Rayon workers.

use crate::non_hermitian_skin_amplifier::NonHermitianSkinAmplifierSolver;
use phonon_models::non_hermitian_skin_amplifier::{
    NonHermitianSkinAmplifierMetrics, NonHermitianSkinAmplifierParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for non-Hermitian skin amplifier parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkinAmplifierBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_forward_gain_db: f64,
    pub min_forward_gain_db: f64,
    pub max_forward_gain_db: f64,
    pub mean_reverse_isolation_db: f64,
    pub min_reverse_isolation_db: f64,
    pub max_reverse_isolation_db: f64,
    pub mean_added_noise_quanta: f64,
    pub min_added_noise_quanta: f64,
    pub max_added_noise_quanta: f64,
    pub mean_power_saturation_threshold_dbm: f64,
    pub min_power_saturation_threshold_dbm: f64,
    pub max_power_saturation_threshold_dbm: f64,
    pub mean_skin_mode_localization_ratio: f64,
    pub min_skin_mode_localization_ratio: f64,
    pub max_skin_mode_localization_ratio: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct SkinAmplifierBenchmarkRunner;

impl SkinAmplifierBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> SkinAmplifierBenchmarkResult {
        let sweep_params: Vec<NonHermitianSkinAmplifierParams> = (0..cycles)
            .map(|i| {
                let center_frequency_ghz = 3.5 + 1.5 * ((i % 23) as f64 / 23.0);
                let lattice_sites_count = 25 + (i % 15);
                let forward_coupling_mhz = 40.0 + 10.0 * ((i % 29) as f64 / 29.0);
                let reverse_coupling_mhz = 3.5 + 2.5 * ((i % 31) as f64 / 31.0);
                let parametric_pump_rate_mhz = 18.0 + 6.0 * ((i % 37) as f64 / 37.0);
                let dissipation_gradient_mhz = 10.0 + 4.0 * ((i % 19) as f64 / 19.0);
                let operating_temp_m_k = 10.0 + 6.0 * ((i % 43) as f64 / 43.0);
                let input_signal_power_dbm = -40.0 + 10.0 * ((i % 17) as f64 / 17.0);

                NonHermitianSkinAmplifierParams::new(
                    center_frequency_ghz,
                    lattice_sites_count,
                    forward_coupling_mhz,
                    reverse_coupling_mhz,
                    parametric_pump_rate_mhz,
                    dissipation_gradient_mhz,
                    operating_temp_m_k,
                    input_signal_power_dbm,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<NonHermitianSkinAmplifierMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = NonHermitianSkinAmplifierSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_gain = 0.0;
        let mut min_gain = f64::MAX;
        let mut max_gain = f64::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut max_iso = f64::MIN;

        let mut sum_noise = 0.0;
        let mut min_noise = f64::MAX;
        let mut max_noise = f64::MIN;

        let mut sum_psat = 0.0;
        let mut min_psat = f64::MAX;
        let mut max_psat = f64::MIN;

        let mut sum_loc = 0.0;
        let mut min_loc = f64::MAX;
        let mut max_loc = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_gain += m.forward_gain_db;
            if m.forward_gain_db < min_gain {
                min_gain = m.forward_gain_db;
            }
            if m.forward_gain_db > max_gain {
                max_gain = m.forward_gain_db;
            }

            sum_iso += m.reverse_isolation_db;
            if m.reverse_isolation_db < min_iso {
                min_iso = m.reverse_isolation_db;
            }
            if m.reverse_isolation_db > max_iso {
                max_iso = m.reverse_isolation_db;
            }

            sum_noise += m.added_noise_quanta;
            if m.added_noise_quanta < min_noise {
                min_noise = m.added_noise_quanta;
            }
            if m.added_noise_quanta > max_noise {
                max_noise = m.added_noise_quanta;
            }

            sum_psat += m.power_saturation_threshold_dbm;
            if m.power_saturation_threshold_dbm < min_psat {
                min_psat = m.power_saturation_threshold_dbm;
            }
            if m.power_saturation_threshold_dbm > max_psat {
                max_psat = m.power_saturation_threshold_dbm;
            }

            sum_loc += m.skin_mode_localization_ratio;
            if m.skin_mode_localization_ratio < min_loc {
                min_loc = m.skin_mode_localization_ratio;
            }
            if m.skin_mode_localization_ratio > max_loc {
                max_loc = m.skin_mode_localization_ratio;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let count = cycles.max(1) as f64;
        let compliance_fraction = (compliant_count as f64) / count;

        SkinAmplifierBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_forward_gain_db: sum_gain / count,
            min_forward_gain_db: min_gain,
            max_forward_gain_db: max_gain,
            mean_reverse_isolation_db: sum_iso / count,
            min_reverse_isolation_db: min_iso,
            max_reverse_isolation_db: max_iso,
            mean_added_noise_quanta: sum_noise / count,
            min_added_noise_quanta: min_noise,
            max_added_noise_quanta: max_noise,
            mean_power_saturation_threshold_dbm: sum_psat / count,
            min_power_saturation_threshold_dbm: min_psat,
            max_power_saturation_threshold_dbm: max_psat,
            mean_skin_mode_localization_ratio: sum_loc / count,
            min_skin_mode_localization_ratio: min_loc,
            max_skin_mode_localization_ratio: max_loc,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
