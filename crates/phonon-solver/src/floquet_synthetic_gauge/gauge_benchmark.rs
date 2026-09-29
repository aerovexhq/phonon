#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for Floquet-Bloch synthetic gauge
//! acoustic fields and dynamically reconfigurable phononic quantum simulators across Rayon workers.

use crate::floquet_synthetic_gauge::FloquetSyntheticGaugeSolver;
use phonon_models::floquet_synthetic_gauge::{
    FloquetSyntheticGaugeMetrics, FloquetSyntheticGaugeParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for Floquet synthetic gauge parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GaugeBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_dynamical_state_fidelity: f64,
    pub min_dynamical_state_fidelity: f64,
    pub max_dynamical_state_fidelity: f64,
    pub mean_synthetic_magnetic_flux_ratio: f64,
    pub min_synthetic_magnetic_flux_ratio: f64,
    pub max_synthetic_magnetic_flux_ratio: f64,
    pub mean_flux_quantization_error: f64,
    pub min_flux_quantization_error: f64,
    pub max_flux_quantization_error: f64,
    pub mean_chern_switching_time_ns: f64,
    pub min_chern_switching_time_ns: f64,
    pub max_chern_switching_time_ns: f64,
    pub mean_topological_band_isolation_db: f64,
    pub min_topological_band_isolation_db: f64,
    pub max_topological_band_isolation_db: f64,
    pub physical_compliance_fraction: f64,
}

pub type FloquetSyntheticGaugeBenchmarkResult = GaugeBenchmarkResult;

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct GaugeBenchmarkRunner;

pub type FloquetSyntheticGaugeBenchmarkRunner = GaugeBenchmarkRunner;

impl GaugeBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> GaugeBenchmarkResult {
        let sweep_params: Vec<FloquetSyntheticGaugeParams> = (0..cycles)
            .map(|i| {
                let acoustic_center_freq_ghz = 4.2 + 0.8 * ((i % 17) as f64 / 17.0);
                let floquet_drive_freq_mhz = 75.0 + 15.0 * ((i % 19) as f64 / 19.0);
                let parametric_modulation_depth = 0.26 + 0.05 * ((i % 23) as f64 / 23.0);
                let lattice_plaquette_count = 14 + (i % 5);
                let synthetic_phase_gradient_rad = 1.50 + 0.15 * ((i % 29) as f64 / 29.0);
                let inter_site_coupling_mhz = 22.0 + 6.0 * ((i % 31) as f64 / 31.0);
                let operating_temp_m_k = 13.0 + 4.0 * ((i % 37) as f64 / 37.0);
                let acoustic_damping_rate_khz = 4.5 + 1.0 * ((i % 41) as f64 / 41.0);

                FloquetSyntheticGaugeParams::new(
                    acoustic_center_freq_ghz,
                    floquet_drive_freq_mhz,
                    parametric_modulation_depth,
                    lattice_plaquette_count,
                    synthetic_phase_gradient_rad,
                    inter_site_coupling_mhz,
                    operating_temp_m_k,
                    acoustic_damping_rate_khz,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<FloquetSyntheticGaugeMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = FloquetSyntheticGaugeSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_flux = 0.0;
        let mut min_flux = f64::MAX;
        let mut max_flux = f64::MIN;

        let mut sum_err = 0.0;
        let mut min_err = f64::MAX;
        let mut max_err = f64::MIN;

        let mut sum_tau = 0.0;
        let mut min_tau = f64::MAX;
        let mut max_tau = f64::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut max_iso = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.dynamical_state_fidelity;
            if m.dynamical_state_fidelity < min_fid {
                min_fid = m.dynamical_state_fidelity;
            }
            if m.dynamical_state_fidelity > max_fid {
                max_fid = m.dynamical_state_fidelity;
            }

            sum_flux += m.synthetic_magnetic_flux_ratio;
            if m.synthetic_magnetic_flux_ratio < min_flux {
                min_flux = m.synthetic_magnetic_flux_ratio;
            }
            if m.synthetic_magnetic_flux_ratio > max_flux {
                max_flux = m.synthetic_magnetic_flux_ratio;
            }

            sum_err += m.flux_quantization_error;
            if m.flux_quantization_error < min_err {
                min_err = m.flux_quantization_error;
            }
            if m.flux_quantization_error > max_err {
                max_err = m.flux_quantization_error;
            }

            sum_tau += m.chern_switching_time_ns;
            if m.chern_switching_time_ns < min_tau {
                min_tau = m.chern_switching_time_ns;
            }
            if m.chern_switching_time_ns > max_tau {
                max_tau = m.chern_switching_time_ns;
            }

            sum_iso += m.topological_band_isolation_db;
            if m.topological_band_isolation_db < min_iso {
                min_iso = m.topological_band_isolation_db;
            }
            if m.topological_band_isolation_db > max_iso {
                max_iso = m.topological_band_isolation_db;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        GaugeBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_dynamical_state_fidelity: sum_fid / n,
            min_dynamical_state_fidelity: min_fid,
            max_dynamical_state_fidelity: max_fid,
            mean_synthetic_magnetic_flux_ratio: sum_flux / n,
            min_synthetic_magnetic_flux_ratio: min_flux,
            max_synthetic_magnetic_flux_ratio: max_flux,
            mean_flux_quantization_error: sum_err / n,
            min_flux_quantization_error: min_err,
            max_flux_quantization_error: max_err,
            mean_chern_switching_time_ns: sum_tau / n,
            min_chern_switching_time_ns: min_tau,
            max_chern_switching_time_ns: max_tau,
            mean_topological_band_isolation_db: sum_iso / n,
            min_topological_band_isolation_db: min_iso,
            max_topological_band_isolation_db: max_iso,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
