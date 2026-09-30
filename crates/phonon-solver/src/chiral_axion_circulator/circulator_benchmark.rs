#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for chiral acoustic axion electrodynamics
//! and dynamic magnetoelectric phonon circulators across multi-threaded Rayon workers.

use crate::chiral_axion_circulator::ChiralAxionCirculatorSolver;
use phonon_models::chiral_axion_circulator::{
    ChiralAxionCirculatorMetrics, ChiralAxionCirculatorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for chiral axion circulator parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralAxionCirculatorBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_non_reciprocal_isolation_db: f64,
    pub min_non_reciprocal_isolation_db: f64,
    pub max_non_reciprocal_isolation_db: f64,
    pub mean_axion_polariton_transmission_fidelity: f64,
    pub min_axion_polariton_transmission_fidelity: f64,
    pub max_axion_polariton_transmission_fidelity: f64,
    pub mean_circulator_insertion_loss_db: f64,
    pub min_circulator_insertion_loss_db: f64,
    pub max_circulator_insertion_loss_db: f64,
    pub mean_axionic_phase_stability_error_rad: f64,
    pub min_axionic_phase_stability_error_rad: f64,
    pub max_axionic_phase_stability_error_rad: f64,
    pub mean_harmonic_distortion_suppression_db: f64,
    pub min_harmonic_distortion_suppression_db: f64,
    pub max_harmonic_distortion_suppression_db: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct ChiralAxionCirculatorBenchmarkRunner;

impl ChiralAxionCirculatorBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> ChiralAxionCirculatorBenchmarkResult {
        let sweep_params: Vec<ChiralAxionCirculatorParams> = (0..cycles)
            .map(|i| {
                let axion_coupling_constant_theta = 1.2 + 2.1 * (((i * 7) % 50) as f64 / 50.0);
                let magnetoelectric_polarizability_alpha =
                    0.10 + 0.80 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_circulation_frequency_ghz =
                    1.5 + 13.0 * (((i * 11) % 40) as f64 / 40.0);
                let cryogenic_temperature_mk = 2.0 + 45.0 * (((i * 23) % 35) as f64 / 35.0);
                let magnetic_heterostructure_thickness_nm =
                    25.0 + 210.0 * (((i * 19) % 45) as f64 / 45.0);
                let inter_port_angular_spacing_deg =
                    105.0 + 30.0 * (((i * 31) % 40) as f64 / 40.0);
                let acoustic_power_drive_uw = 0.5 + 45.0 * (((i * 17) % 50) as f64 / 50.0);
                let cavity_resonance_quality_factor =
                    2.0e4 + 4.5e5 * (((i * 29) % 30) as f64 / 30.0);

                ChiralAxionCirculatorParams::new(
                    axion_coupling_constant_theta,
                    magnetoelectric_polarizability_alpha,
                    acoustic_circulation_frequency_ghz,
                    cryogenic_temperature_mk,
                    magnetic_heterostructure_thickness_nm,
                    inter_port_angular_spacing_deg,
                    acoustic_power_drive_uw,
                    cavity_resonance_quality_factor,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ChiralAxionCirculatorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ChiralAxionCirculatorSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_loss = 0.0;
        let mut min_loss = f64::MAX;
        let mut max_loss = f64::MIN;

        let mut sum_error = 0.0;
        let mut min_error = f64::MAX;
        let mut max_error = f64::MIN;

        let mut sum_distortion = 0.0;
        let mut min_distortion = f64::MAX;
        let mut max_distortion = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_isolation += m.non_reciprocal_isolation_db;
            if m.non_reciprocal_isolation_db < min_isolation {
                min_isolation = m.non_reciprocal_isolation_db;
            }
            if m.non_reciprocal_isolation_db > max_isolation {
                max_isolation = m.non_reciprocal_isolation_db;
            }

            sum_fidelity += m.axion_polariton_transmission_fidelity;
            if m.axion_polariton_transmission_fidelity < min_fidelity {
                min_fidelity = m.axion_polariton_transmission_fidelity;
            }
            if m.axion_polariton_transmission_fidelity > max_fidelity {
                max_fidelity = m.axion_polariton_transmission_fidelity;
            }

            sum_loss += m.circulator_insertion_loss_db;
            if m.circulator_insertion_loss_db < min_loss {
                min_loss = m.circulator_insertion_loss_db;
            }
            if m.circulator_insertion_loss_db > max_loss {
                max_loss = m.circulator_insertion_loss_db;
            }

            sum_error += m.axionic_phase_stability_error_rad;
            if m.axionic_phase_stability_error_rad < min_error {
                min_error = m.axionic_phase_stability_error_rad;
            }
            if m.axionic_phase_stability_error_rad > max_error {
                max_error = m.axionic_phase_stability_error_rad;
            }

            sum_distortion += m.harmonic_distortion_suppression_db;
            if m.harmonic_distortion_suppression_db < min_distortion {
                min_distortion = m.harmonic_distortion_suppression_db;
            }
            if m.harmonic_distortion_suppression_db > max_distortion {
                max_distortion = m.harmonic_distortion_suppression_db;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        ChiralAxionCirculatorBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_non_reciprocal_isolation_db: sum_isolation / n,
            min_non_reciprocal_isolation_db: min_isolation,
            max_non_reciprocal_isolation_db: max_isolation,
            mean_axion_polariton_transmission_fidelity: sum_fidelity / n,
            min_axion_polariton_transmission_fidelity: min_fidelity,
            max_axion_polariton_transmission_fidelity: max_fidelity,
            mean_circulator_insertion_loss_db: sum_loss / n,
            min_circulator_insertion_loss_db: min_loss,
            max_circulator_insertion_loss_db: max_loss,
            mean_axionic_phase_stability_error_rad: sum_error / n,
            min_axionic_phase_stability_error_rad: min_error,
            max_axionic_phase_stability_error_rad: max_error,
            mean_harmonic_distortion_suppression_db: sum_distortion / n,
            min_harmonic_distortion_suppression_db: min_distortion,
            max_harmonic_distortion_suppression_db: max_distortion,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
