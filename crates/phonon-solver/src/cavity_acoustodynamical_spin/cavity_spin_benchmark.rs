#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for cavity quantum acoustodynamical
//! spin-phonon interfaces and chiral squeezed vacuum synthesizers across multi-threaded Rayon workers.

use crate::cavity_acoustodynamical_spin::cavity_spin_solver::CavityAcoustodynamicalSpinSolver;
use phonon_models::cavity_acoustodynamical_spin::{
    CavityAcoustodynamicalSpinMetrics, CavityAcoustodynamicalSpinParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for cavity acoustodynamical spin-phonon parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CavitySpinBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_acoustic_quadrature_squeezing_db: f64,
    pub min_acoustic_quadrature_squeezing_db: f64,
    pub max_acoustic_quadrature_squeezing_db: f64,
    pub mean_spin_phonon_fidelity: f64,
    pub min_spin_phonon_fidelity: f64,
    pub max_spin_phonon_fidelity: f64,
    pub mean_spin_coherence_lifetime_ms: f64,
    pub min_spin_coherence_lifetime_ms: f64,
    pub max_spin_coherence_lifetime_ms: f64,
    pub mean_thermal_phonon_occupancy: f64,
    pub min_thermal_phonon_occupancy: f64,
    pub max_thermal_phonon_occupancy: f64,
    pub mean_purcell_enhancement_factor: f64,
    pub min_purcell_enhancement_factor: f64,
    pub max_purcell_enhancement_factor: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct CavitySpinBenchmarkRunner;

impl CavitySpinBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> CavitySpinBenchmarkResult {
        let sweep_params: Vec<CavityAcoustodynamicalSpinParams> = (0..cycles)
            .map(|i| {
                let pump_power_mw = 4.0 + 1.2 * ((i % 11) as f64 / 11.0);
                let cavity_decay_rate_khz = 75.0 + 15.0 * ((i % 13) as f64 / 13.0);
                let spin_phonon_coupling_mhz = 5.4 + 1.0 * ((i % 17) as f64 / 17.0);
                let non_linear_gain_db = 15.5 + 2.0 * ((i % 19) as f64 / 19.0);
                let cryogenic_temp_mk = 16.0 + 6.0 * ((i % 23) as f64 / 23.0);
                let acoustic_frequency_ghz = 4.8 + 0.8 * ((i % 29) as f64 / 29.0);
                let spin_dephasing_rate_hz = 10.0 + 3.0 * ((i % 31) as f64 / 31.0);
                let chiral_isolation_db = 36.0 + 5.0 * ((i % 37) as f64 / 37.0);

                CavityAcoustodynamicalSpinParams::new(
                    pump_power_mw,
                    cavity_decay_rate_khz,
                    spin_phonon_coupling_mhz,
                    non_linear_gain_db,
                    cryogenic_temp_mk,
                    acoustic_frequency_ghz,
                    spin_dephasing_rate_hz,
                    chiral_isolation_db,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<CavityAcoustodynamicalSpinMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = CavityAcoustodynamicalSpinSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_squeezing = 0.0;
        let mut min_squeezing = f64::MAX;
        let mut max_squeezing = f64::MIN;

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_lifetime = 0.0;
        let mut min_lifetime = f64::MAX;
        let mut max_lifetime = f64::MIN;

        let mut sum_occupancy = 0.0;
        let mut min_occupancy = f64::MAX;
        let mut max_occupancy = f64::MIN;

        let mut sum_purcell = 0.0;
        let mut min_purcell = f64::MAX;
        let mut max_purcell = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_squeezing += m.acoustic_quadrature_squeezing_db;
            if m.acoustic_quadrature_squeezing_db < min_squeezing {
                min_squeezing = m.acoustic_quadrature_squeezing_db;
            }
            if m.acoustic_quadrature_squeezing_db > max_squeezing {
                max_squeezing = m.acoustic_quadrature_squeezing_db;
            }

            sum_fidelity += m.spin_phonon_fidelity;
            if m.spin_phonon_fidelity < min_fidelity {
                min_fidelity = m.spin_phonon_fidelity;
            }
            if m.spin_phonon_fidelity > max_fidelity {
                max_fidelity = m.spin_phonon_fidelity;
            }

            sum_lifetime += m.spin_coherence_lifetime_ms;
            if m.spin_coherence_lifetime_ms < min_lifetime {
                min_lifetime = m.spin_coherence_lifetime_ms;
            }
            if m.spin_coherence_lifetime_ms > max_lifetime {
                max_lifetime = m.spin_coherence_lifetime_ms;
            }

            sum_occupancy += m.thermal_phonon_occupancy;
            if m.thermal_phonon_occupancy < min_occupancy {
                min_occupancy = m.thermal_phonon_occupancy;
            }
            if m.thermal_phonon_occupancy > max_occupancy {
                max_occupancy = m.thermal_phonon_occupancy;
            }

            sum_purcell += m.purcell_enhancement_factor;
            if m.purcell_enhancement_factor < min_purcell {
                min_purcell = m.purcell_enhancement_factor;
            }
            if m.purcell_enhancement_factor > max_purcell {
                max_purcell = m.purcell_enhancement_factor;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        CavitySpinBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_acoustic_quadrature_squeezing_db: sum_squeezing / n,
            min_acoustic_quadrature_squeezing_db: min_squeezing,
            max_acoustic_quadrature_squeezing_db: max_squeezing,
            mean_spin_phonon_fidelity: sum_fidelity / n,
            min_spin_phonon_fidelity: min_fidelity,
            max_spin_phonon_fidelity: max_fidelity,
            mean_spin_coherence_lifetime_ms: sum_lifetime / n,
            min_spin_coherence_lifetime_ms: min_lifetime,
            max_spin_coherence_lifetime_ms: max_lifetime,
            mean_thermal_phonon_occupancy: sum_occupancy / n,
            min_thermal_phonon_occupancy: min_occupancy,
            max_thermal_phonon_occupancy: max_occupancy,
            mean_purcell_enhancement_factor: sum_purcell / n,
            min_purcell_enhancement_factor: min_purcell,
            max_purcell_enhancement_factor: max_purcell,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
