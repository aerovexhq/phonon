#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Hermitian topological
//! acoustic edge solitons and dissipationless phononic shockwave routers across Rayon workers.

use crate::non_hermitian_edge_soliton::NonHermitianEdgeSolitonSolver;
use phonon_models::non_hermitian_edge_soliton::{
    NonHermitianEdgeSolitonMetrics, NonHermitianEdgeSolitonParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for non-Hermitian edge soliton parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeSolitonBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_soliton_transmission_fidelity: f64,
    pub min_soliton_transmission_fidelity: f64,
    pub max_soliton_transmission_fidelity: f64,
    pub mean_harmonic_distortion_db: f64,
    pub min_harmonic_distortion_db: f64,
    pub max_harmonic_distortion_db: f64,
    pub mean_backscattering_immunity_db: f64,
    pub min_backscattering_immunity_db: f64,
    pub max_backscattering_immunity_db: f64,
    pub mean_soliton_pulse_width_ns: f64,
    pub min_soliton_pulse_width_ns: f64,
    pub max_soliton_pulse_width_ns: f64,
    pub mean_lyapunov_stability_exponent: f64,
    pub min_lyapunov_stability_exponent: f64,
    pub max_lyapunov_stability_exponent: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct EdgeSolitonBenchmarkRunner;

impl EdgeSolitonBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> EdgeSolitonBenchmarkResult {
        let sweep_params: Vec<NonHermitianEdgeSolitonParams> = (0..cycles)
            .map(|i| {
                let carrier_frequency_ghz = 3.5 + 0.6 * ((i % 17) as f64 / 17.0);
                let dispersion_parameter_d2_khz = 30.0 + 4.0 * ((i % 19) as f64 / 19.0);
                let kerr_nonlinearity_hz = 10.0 + 4.0 * ((i % 23) as f64 / 23.0);
                let non_hermitian_gain_mhz = 17.0 + 2.0 * ((i % 29) as f64 / 29.0);
                let non_hermitian_loss_mhz = 17.0 + 2.0 * ((i % 31) as f64 / 31.0);
                let soliton_amplitude_pa = 115.0 + 10.0 * ((i % 37) as f64 / 37.0);
                let operating_temp_m_k = 13.0 + 4.0 * ((i % 41) as f64 / 41.0);
                let waveguide_length_um = 230.0 + 40.0 * ((i % 43) as f64 / 43.0);

                NonHermitianEdgeSolitonParams::new(
                    carrier_frequency_ghz,
                    dispersion_parameter_d2_khz,
                    kerr_nonlinearity_hz,
                    non_hermitian_gain_mhz,
                    non_hermitian_loss_mhz,
                    soliton_amplitude_pa,
                    operating_temp_m_k,
                    waveguide_length_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<NonHermitianEdgeSolitonMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = NonHermitianEdgeSolitonSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_f = 0.0;
        let mut min_f = f64::MAX;
        let mut max_f = f64::MIN;

        let mut sum_hd = 0.0;
        let mut min_hd = f64::MAX;
        let mut max_hd = f64::MIN;

        let mut sum_bi = 0.0;
        let mut min_bi = f64::MAX;
        let mut max_bi = f64::MIN;

        let mut sum_tau = 0.0;
        let mut min_tau = f64::MAX;
        let mut max_tau = f64::MIN;

        let mut sum_lyap = 0.0;
        let mut min_lyap = f64::MAX;
        let mut max_lyap = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_f += m.soliton_transmission_fidelity;
            if m.soliton_transmission_fidelity < min_f {
                min_f = m.soliton_transmission_fidelity;
            }
            if m.soliton_transmission_fidelity > max_f {
                max_f = m.soliton_transmission_fidelity;
            }

            sum_hd += m.harmonic_distortion_db;
            if m.harmonic_distortion_db < min_hd {
                min_hd = m.harmonic_distortion_db;
            }
            if m.harmonic_distortion_db > max_hd {
                max_hd = m.harmonic_distortion_db;
            }

            sum_bi += m.backscattering_immunity_db;
            if m.backscattering_immunity_db < min_bi {
                min_bi = m.backscattering_immunity_db;
            }
            if m.backscattering_immunity_db > max_bi {
                max_bi = m.backscattering_immunity_db;
            }

            sum_tau += m.soliton_pulse_width_ns;
            if m.soliton_pulse_width_ns < min_tau {
                min_tau = m.soliton_pulse_width_ns;
            }
            if m.soliton_pulse_width_ns > max_tau {
                max_tau = m.soliton_pulse_width_ns;
            }

            sum_lyap += m.lyapunov_stability_exponent;
            if m.lyapunov_stability_exponent < min_lyap {
                min_lyap = m.lyapunov_stability_exponent;
            }
            if m.lyapunov_stability_exponent > max_lyap {
                max_lyap = m.lyapunov_stability_exponent;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        EdgeSolitonBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_soliton_transmission_fidelity: sum_f / n,
            min_soliton_transmission_fidelity: min_f,
            max_soliton_transmission_fidelity: max_f,
            mean_harmonic_distortion_db: sum_hd / n,
            min_harmonic_distortion_db: min_hd,
            max_harmonic_distortion_db: max_hd,
            mean_backscattering_immunity_db: sum_bi / n,
            min_backscattering_immunity_db: min_bi,
            max_backscattering_immunity_db: max_bi,
            mean_soliton_pulse_width_ns: sum_tau / n,
            min_soliton_pulse_width_ns: min_tau,
            max_soliton_pulse_width_ns: max_tau,
            mean_lyapunov_stability_exponent: sum_lyap / n,
            min_lyapunov_stability_exponent: min_lyap,
            max_lyapunov_stability_exponent: max_lyap,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
