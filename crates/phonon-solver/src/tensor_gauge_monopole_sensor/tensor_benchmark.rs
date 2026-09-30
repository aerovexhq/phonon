#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for topological acoustic higher-rank
//! tensor gauge fields and chiral monopole-plaquette phononic sensors across Rayon workers.

use crate::tensor_gauge_monopole_sensor::TensorGaugeMonopoleSensorSolver;
use phonon_models::tensor_gauge_monopole_sensor::{
    TensorGaugeMonopoleSensorMetrics, TensorGaugeMonopoleSensorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for topological acoustic higher-rank tensor gauge parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TensorGaugeBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_tensor_charge_sensitivity_enhancement: f64,
    pub min_tensor_charge_sensitivity_enhancement: f64,
    pub max_tensor_charge_sensitivity_enhancement: f64,
    pub mean_plaquette_phase_stability_error_rad: f64,
    pub min_plaquette_phase_stability_error_rad: f64,
    pub max_plaquette_phase_stability_error_rad: f64,
    pub mean_sub_dimensional_leakage: f64,
    pub min_sub_dimensional_leakage: f64,
    pub max_sub_dimensional_leakage: f64,
    pub mean_topological_monopole_lifetime_ms: f64,
    pub min_topological_monopole_lifetime_ms: f64,
    pub max_topological_monopole_lifetime_ms: f64,
    pub mean_tensor_gauge_flux_quantization_fidelity: f64,
    pub min_tensor_gauge_flux_quantization_fidelity: f64,
    pub max_tensor_gauge_flux_quantization_fidelity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct TensorGaugeBenchmarkRunner;

impl TensorGaugeBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> TensorGaugeBenchmarkResult {
        let sweep_params: Vec<TensorGaugeMonopoleSensorParams> = (0..cycles)
            .map(|i| {
                let tensor_gauge_coupling_constant =
                    0.30 + 4.50 * (((i * 7) % 50) as f64 / 50.0);
                let chiral_plaquette_coupling_energy_mev =
                    2.0 + 27.0 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_sensor_frequency_ghz =
                    1.5 + 13.0 * (((i * 11) % 40) as f64 / 40.0);
                let cryogenic_temperature_mk =
                    2.0 + 45.0 * (((i * 19) % 45) as f64 / 45.0);
                let lattice_cell_dimension_nm =
                    50.0 + 340.0 * (((i * 23) % 35) as f64 / 35.0);
                let dipole_conservation_constraint_weight =
                    0.55 + 0.42 * (((i * 29) % 30) as f64 / 30.0);
                let monopole_pinning_field_tesla =
                    0.8 + 8.8 * (((i * 31) % 40) as f64 / 40.0);
                let sensing_cavity_quality_factor =
                    2.0e4 + 4.5e5 * (((i * 17) % 50) as f64 / 50.0);

                TensorGaugeMonopoleSensorParams::new(
                    tensor_gauge_coupling_constant,
                    chiral_plaquette_coupling_energy_mev,
                    acoustic_sensor_frequency_ghz,
                    cryogenic_temperature_mk,
                    lattice_cell_dimension_nm,
                    dipole_conservation_constraint_weight,
                    monopole_pinning_field_tesla,
                    sensing_cavity_quality_factor,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<TensorGaugeMonopoleSensorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TensorGaugeMonopoleSensorSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_sensitivity = 0.0;
        let mut min_sensitivity = f64::MAX;
        let mut max_sensitivity = f64::MIN;

        let mut sum_error = 0.0;
        let mut min_error = f64::MAX;
        let mut max_error = f64::MIN;

        let mut sum_leakage = 0.0;
        let mut min_leakage = f64::MAX;
        let mut max_leakage = f64::MIN;

        let mut sum_lifetime = 0.0;
        let mut min_lifetime = f64::MAX;
        let mut max_lifetime = f64::MIN;

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_sensitivity += m.tensor_charge_sensitivity_enhancement;
            if m.tensor_charge_sensitivity_enhancement < min_sensitivity {
                min_sensitivity = m.tensor_charge_sensitivity_enhancement;
            }
            if m.tensor_charge_sensitivity_enhancement > max_sensitivity {
                max_sensitivity = m.tensor_charge_sensitivity_enhancement;
            }

            sum_error += m.plaquette_phase_stability_error_rad;
            if m.plaquette_phase_stability_error_rad < min_error {
                min_error = m.plaquette_phase_stability_error_rad;
            }
            if m.plaquette_phase_stability_error_rad > max_error {
                max_error = m.plaquette_phase_stability_error_rad;
            }

            sum_leakage += m.sub_dimensional_leakage;
            if m.sub_dimensional_leakage < min_leakage {
                min_leakage = m.sub_dimensional_leakage;
            }
            if m.sub_dimensional_leakage > max_leakage {
                max_leakage = m.sub_dimensional_leakage;
            }

            sum_lifetime += m.topological_monopole_lifetime_ms;
            if m.topological_monopole_lifetime_ms < min_lifetime {
                min_lifetime = m.topological_monopole_lifetime_ms;
            }
            if m.topological_monopole_lifetime_ms > max_lifetime {
                max_lifetime = m.topological_monopole_lifetime_ms;
            }

            sum_fidelity += m.tensor_gauge_flux_quantization_fidelity;
            if m.tensor_gauge_flux_quantization_fidelity < min_fidelity {
                min_fidelity = m.tensor_gauge_flux_quantization_fidelity;
            }
            if m.tensor_gauge_flux_quantization_fidelity > max_fidelity {
                max_fidelity = m.tensor_gauge_flux_quantization_fidelity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        TensorGaugeBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_tensor_charge_sensitivity_enhancement: sum_sensitivity / n,
            min_tensor_charge_sensitivity_enhancement: min_sensitivity,
            max_tensor_charge_sensitivity_enhancement: max_sensitivity,
            mean_plaquette_phase_stability_error_rad: sum_error / n,
            min_plaquette_phase_stability_error_rad: min_error,
            max_plaquette_phase_stability_error_rad: max_error,
            mean_sub_dimensional_leakage: sum_leakage / n,
            min_sub_dimensional_leakage: min_leakage,
            max_sub_dimensional_leakage: max_leakage,
            mean_topological_monopole_lifetime_ms: sum_lifetime / n,
            min_topological_monopole_lifetime_ms: min_lifetime,
            max_topological_monopole_lifetime_ms: max_lifetime,
            mean_tensor_gauge_flux_quantization_fidelity: sum_fidelity / n,
            min_tensor_gauge_flux_quantization_fidelity: min_fidelity,
            max_tensor_gauge_flux_quantization_fidelity: max_fidelity,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
