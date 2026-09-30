#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic non-Hermitian
//! Floquet exceptional-ring synthesizers and chiral skin sensors across multi-threaded Rayon workers.

use crate::floquet_exceptional_ring_sensor::FloquetExceptionalRingSensorSolver;
use phonon_models::floquet_exceptional_ring_sensor::{
    FloquetExceptionalRingSensorMetrics, FloquetExceptionalRingSensorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for Floquet exceptional ring sensor parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExceptionalRingBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_skin_mode_localization_ratio: f64,
    pub min_skin_mode_localization_ratio: f64,
    pub max_skin_mode_localization_ratio: f64,
    pub mean_sensitivity_enhancement_factor: f64,
    pub min_sensitivity_enhancement_factor: f64,
    pub max_sensitivity_enhancement_factor: f64,
    pub mean_reverse_backscattering_suppression_db: f64,
    pub min_reverse_backscattering_suppression_db: f64,
    pub max_reverse_backscattering_suppression_db: f64,
    pub mean_sensor_noise_figure_db: f64,
    pub min_sensor_noise_figure_db: f64,
    pub max_sensor_noise_figure_db: f64,
    pub mean_exceptional_ring_topological_charge: f64,
    pub min_exceptional_ring_topological_charge: f64,
    pub max_exceptional_ring_topological_charge: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct ExceptionalRingBenchmarkRunner;

impl ExceptionalRingBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> ExceptionalRingBenchmarkResult {
        let sweep_params: Vec<FloquetExceptionalRingSensorParams> = (0..cycles)
            .map(|i| {
                let floquet_drive_amplitude_mhz = 15.0 + 55.0 * (((i * 7) % 50) as f64 / 50.0);
                let floquet_modulation_frequency_ghz = 2.5 + 4.5 * (((i * 13) % 45) as f64 / 45.0);
                let non_reciprocal_hopping_asymmetry = 0.35 + 0.55 * (((i * 11) % 40) as f64 / 40.0);
                let cavity_loss_contrast_khz = 50.0 + 350.0 * (((i * 17) % 35) as f64 / 35.0);
                let sensor_array_elements = 12 + (((i * 19) % 45) * 48 / 45);
                let perturbation_coupling_strength_hz = 50.0 + 600.0 * (((i * 23) % 30) as f64 / 30.0);
                let operating_temperature_mk = 2.0 + 35.0 * (((i * 29) % 32) as f64 / 32.0);
                let piezoelectric_gain_db = 15.0 + 25.0 * (((i * 31) % 25) as f64 / 25.0);

                FloquetExceptionalRingSensorParams::new(
                    floquet_drive_amplitude_mhz,
                    floquet_modulation_frequency_ghz,
                    non_reciprocal_hopping_asymmetry,
                    cavity_loss_contrast_khz,
                    sensor_array_elements,
                    perturbation_coupling_strength_hz,
                    operating_temperature_mk,
                    piezoelectric_gain_db,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<FloquetExceptionalRingSensorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = FloquetExceptionalRingSensorSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_localization = 0.0;
        let mut min_localization = f64::MAX;
        let mut max_localization = f64::MIN;

        let mut sum_sensitivity = 0.0;
        let mut min_sensitivity = f64::MAX;
        let mut max_sensitivity = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_noise = 0.0;
        let mut min_noise = f64::MAX;
        let mut max_noise = f64::MIN;

        let mut sum_charge = 0.0;
        let mut min_charge = f64::MAX;
        let mut max_charge = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_localization += m.skin_mode_localization_ratio;
            if m.skin_mode_localization_ratio < min_localization {
                min_localization = m.skin_mode_localization_ratio;
            }
            if m.skin_mode_localization_ratio > max_localization {
                max_localization = m.skin_mode_localization_ratio;
            }

            sum_sensitivity += m.sensitivity_enhancement_factor;
            if m.sensitivity_enhancement_factor < min_sensitivity {
                min_sensitivity = m.sensitivity_enhancement_factor;
            }
            if m.sensitivity_enhancement_factor > max_sensitivity {
                max_sensitivity = m.sensitivity_enhancement_factor;
            }

            sum_isolation += m.reverse_backscattering_suppression_db;
            if m.reverse_backscattering_suppression_db < min_isolation {
                min_isolation = m.reverse_backscattering_suppression_db;
            }
            if m.reverse_backscattering_suppression_db > max_isolation {
                max_isolation = m.reverse_backscattering_suppression_db;
            }

            sum_noise += m.sensor_noise_figure_db;
            if m.sensor_noise_figure_db < min_noise {
                min_noise = m.sensor_noise_figure_db;
            }
            if m.sensor_noise_figure_db > max_noise {
                max_noise = m.sensor_noise_figure_db;
            }

            sum_charge += m.exceptional_ring_topological_charge;
            if m.exceptional_ring_topological_charge < min_charge {
                min_charge = m.exceptional_ring_topological_charge;
            }
            if m.exceptional_ring_topological_charge > max_charge {
                max_charge = m.exceptional_ring_topological_charge;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        let physical_compliance_fraction = (compliant_count as f64) / n;

        ExceptionalRingBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_skin_mode_localization_ratio: sum_localization / n,
            min_skin_mode_localization_ratio: min_localization,
            max_skin_mode_localization_ratio: max_localization,
            mean_sensitivity_enhancement_factor: sum_sensitivity / n,
            min_sensitivity_enhancement_factor: min_sensitivity,
            max_sensitivity_enhancement_factor: max_sensitivity,
            mean_reverse_backscattering_suppression_db: sum_isolation / n,
            min_reverse_backscattering_suppression_db: min_isolation,
            max_reverse_backscattering_suppression_db: max_isolation,
            mean_sensor_noise_figure_db: sum_noise / n,
            min_sensor_noise_figure_db: min_noise,
            max_sensor_noise_figure_db: max_noise,
            mean_exceptional_ring_topological_charge: sum_charge / n,
            min_exceptional_ring_topological_charge: min_charge,
            max_exceptional_ring_topological_charge: max_charge,
            physical_compliance_fraction,
        }
    }
}
