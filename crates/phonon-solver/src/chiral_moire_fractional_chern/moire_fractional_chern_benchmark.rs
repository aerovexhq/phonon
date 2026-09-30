#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for chiral acoustic moire fractional
//! Chern insulators and anyonic interferometric braiding networks across multi-threaded Rayon workers.

use crate::chiral_moire_fractional_chern::ChiralMoireFractionalChernSolver;
use phonon_models::chiral_moire_fractional_chern::{
    ChiralMoireFractionalChernMetrics, ChiralMoireFractionalChernParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for chiral moire fractional Chern insulator parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoireFractionalChernBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_anyonic_braiding_phase_fidelity: f64,
    pub min_anyonic_braiding_phase_fidelity: f64,
    pub max_anyonic_braiding_phase_fidelity: f64,
    pub mean_moire_flatband_coherence_ms: f64,
    pub min_moire_flatband_coherence_ms: f64,
    pub max_moire_flatband_coherence_ms: f64,
    pub mean_non_adiabatic_braiding_leakage: f64,
    pub min_non_adiabatic_braiding_leakage: f64,
    pub max_non_adiabatic_braiding_leakage: f64,
    pub mean_quasiparticle_parity_poisoning_immunity_db: f64,
    pub min_quasiparticle_parity_poisoning_immunity_db: f64,
    pub max_quasiparticle_parity_poisoning_immunity_db: f64,
    pub mean_braiding_phase_stability_error_rad: f64,
    pub min_braiding_phase_stability_error_rad: f64,
    pub max_braiding_phase_stability_error_rad: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct MoireFractionalChernBenchmarkRunner;

impl MoireFractionalChernBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> MoireFractionalChernBenchmarkResult {
        let sweep_params: Vec<ChiralMoireFractionalChernParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let twist_angle_deg = 1.08 + 0.35 * ((i as f64 * 0.19).sin());
                let moire_potential_depth_mev = 12.0 + 25.0 * (((i * 7) % 50) as f64 / 50.0);
                let fractional_filling_factor = 0.333333 + 0.12 * (((i * 13) % 45) as f64 / 45.0 - 0.5);
                let acoustic_interferometer_arms = 3 + ((i * 19) % 5);
                let topological_flatband_width_khz = 50.0 + 80.0 * frac;
                let braiding_drive_frequency_ghz = 4.0 + 2.5 * (((i * 11) % 40) as f64 / 40.0);
                let cryogenic_temperature_mk = 5.0 + 20.0 * (((i * 17) % 35) as f64 / 35.0);
                let chiral_damping_rate_hz = 5.0 + 25.0 * (((i * 23) % 30) as f64 / 30.0);

                ChiralMoireFractionalChernParams::new(
                    twist_angle_deg,
                    moire_potential_depth_mev,
                    fractional_filling_factor,
                    acoustic_interferometer_arms,
                    topological_flatband_width_khz,
                    braiding_drive_frequency_ghz,
                    cryogenic_temperature_mk,
                    chiral_damping_rate_hz,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ChiralMoireFractionalChernMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ChiralMoireFractionalChernSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_coherence = 0.0;
        let mut min_coherence = f64::MAX;
        let mut max_coherence = f64::MIN;

        let mut sum_leakage = 0.0;
        let mut min_leakage = f64::MAX;
        let mut max_leakage = f64::MIN;

        let mut sum_immunity = 0.0;
        let mut min_immunity = f64::MAX;
        let mut max_immunity = f64::MIN;

        let mut sum_error = 0.0;
        let mut min_error = f64::MAX;
        let mut max_error = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.anyonic_braiding_phase_fidelity;
            if m.anyonic_braiding_phase_fidelity < min_fidelity {
                min_fidelity = m.anyonic_braiding_phase_fidelity;
            }
            if m.anyonic_braiding_phase_fidelity > max_fidelity {
                max_fidelity = m.anyonic_braiding_phase_fidelity;
            }

            sum_coherence += m.moire_flatband_coherence_ms;
            if m.moire_flatband_coherence_ms < min_coherence {
                min_coherence = m.moire_flatband_coherence_ms;
            }
            if m.moire_flatband_coherence_ms > max_coherence {
                max_coherence = m.moire_flatband_coherence_ms;
            }

            sum_leakage += m.non_adiabatic_braiding_leakage;
            if m.non_adiabatic_braiding_leakage < min_leakage {
                min_leakage = m.non_adiabatic_braiding_leakage;
            }
            if m.non_adiabatic_braiding_leakage > max_leakage {
                max_leakage = m.non_adiabatic_braiding_leakage;
            }

            sum_immunity += m.quasiparticle_parity_poisoning_immunity_db;
            if m.quasiparticle_parity_poisoning_immunity_db < min_immunity {
                min_immunity = m.quasiparticle_parity_poisoning_immunity_db;
            }
            if m.quasiparticle_parity_poisoning_immunity_db > max_immunity {
                max_immunity = m.quasiparticle_parity_poisoning_immunity_db;
            }

            sum_error += m.braiding_phase_stability_error_rad;
            if m.braiding_phase_stability_error_rad < min_error {
                min_error = m.braiding_phase_stability_error_rad;
            }
            if m.braiding_phase_stability_error_rad > max_error {
                max_error = m.braiding_phase_stability_error_rad;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        let physical_compliance_fraction = (compliant_count as f64) / n;

        MoireFractionalChernBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_anyonic_braiding_phase_fidelity: sum_fidelity / n,
            min_anyonic_braiding_phase_fidelity: min_fidelity,
            max_anyonic_braiding_phase_fidelity: max_fidelity,
            mean_moire_flatband_coherence_ms: sum_coherence / n,
            min_moire_flatband_coherence_ms: min_coherence,
            max_moire_flatband_coherence_ms: max_coherence,
            mean_non_adiabatic_braiding_leakage: sum_leakage / n,
            min_non_adiabatic_braiding_leakage: min_leakage,
            max_non_adiabatic_braiding_leakage: max_leakage,
            mean_quasiparticle_parity_poisoning_immunity_db: sum_immunity / n,
            min_quasiparticle_parity_poisoning_immunity_db: min_immunity,
            max_quasiparticle_parity_poisoning_immunity_db: max_immunity,
            mean_braiding_phase_stability_error_rad: sum_error / n,
            min_braiding_phase_stability_error_rad: min_error,
            max_braiding_phase_stability_error_rad: max_error,
            physical_compliance_fraction,
        }
    }
}
