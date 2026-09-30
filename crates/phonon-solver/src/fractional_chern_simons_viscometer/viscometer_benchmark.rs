#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic chiral fractional
//! Chern-Simons hydrodynamics and anyonic holographic edge viscometers across multi-threaded
//! Rayon workers.

use crate::fractional_chern_simons_viscometer::FractionalChernSimonsViscometerSolver;
use phonon_models::fractional_chern_simons_viscometer::{
    FractionalChernSimonsViscometerMetrics, FractionalChernSimonsViscometerParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for fractional Chern-Simons edge viscometer parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChernSimonsViscometerBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_hall_viscosity_measurement_fidelity: f64,
    pub min_hall_viscosity_measurement_fidelity: f64,
    pub max_hall_viscosity_measurement_fidelity: f64,
    pub mean_edge_to_bulk_acoustic_isolation_db: f64,
    pub min_edge_to_bulk_acoustic_isolation_db: f64,
    pub max_edge_to_bulk_acoustic_isolation_db: f64,
    pub mean_edge_mode_velocity_stability_fraction: f64,
    pub min_edge_mode_velocity_stability_fraction: f64,
    pub max_edge_mode_velocity_stability_fraction: f64,
    pub mean_anomalous_edge_acoustic_dissipation_db_per_um: f64,
    pub min_anomalous_edge_acoustic_dissipation_db_per_um: f64,
    pub max_anomalous_edge_acoustic_dissipation_db_per_um: f64,
    pub mean_hydrodynamic_entropy_generation_rate_w_per_k: f64,
    pub min_hydrodynamic_entropy_generation_rate_w_per_k: f64,
    pub max_hydrodynamic_entropy_generation_rate_w_per_k: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct ChernSimonsViscometerBenchmarkRunner;

impl ChernSimonsViscometerBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> ChernSimonsViscometerBenchmarkResult {
        let sweep_params: Vec<FractionalChernSimonsViscometerParams> = (0..cycles)
            .map(|i| {
                let fractional_filling_factor_nu =
                    0.22 + 0.75 * (((i * 7) % 50) as f64 / 50.0);
                let magnetic_field_tesla =
                    2.5 + 13.0 * (((i * 13) % 45) as f64 / 45.0);
                let piezoelectric_stress_coupling_coefficient =
                    0.15 + 0.78 * (((i * 11) % 40) as f64 / 40.0);
                let acoustic_shear_frequency_ghz =
                    1.2 + 13.5 * (((i * 23) % 35) as f64 / 35.0);
                let cryogenic_temperature_mk =
                    2.0 + 46.0 * (((i * 19) % 45) as f64 / 45.0);
                let viscometer_channel_length_um =
                    2.5 + 27.0 * (((i * 31) % 40) as f64 / 40.0);
                let edge_channel_width_nm =
                    25.0 + 170.0 * (((i * 17) % 50) as f64 / 50.0);
                let electron_effective_mass_ratio =
                    0.06 + 0.42 * (((i * 29) % 45) as f64 / 45.0);

                FractionalChernSimonsViscometerParams::new(
                    fractional_filling_factor_nu,
                    magnetic_field_tesla,
                    piezoelectric_stress_coupling_coefficient,
                    acoustic_shear_frequency_ghz,
                    cryogenic_temperature_mk,
                    viscometer_channel_length_um,
                    edge_channel_width_nm,
                    electron_effective_mass_ratio,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<FractionalChernSimonsViscometerMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = FractionalChernSimonsViscometerSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_stability = 0.0;
        let mut min_stability = f64::MAX;
        let mut max_stability = f64::MIN;

        let mut sum_dissipation = 0.0;
        let mut min_dissipation = f64::MAX;
        let mut max_dissipation = f64::MIN;

        let mut sum_entropy = 0.0;
        let mut min_entropy = f64::MAX;
        let mut max_entropy = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.hall_viscosity_measurement_fidelity;
            min_fidelity = min_fidelity.min(m.hall_viscosity_measurement_fidelity);
            max_fidelity = max_fidelity.max(m.hall_viscosity_measurement_fidelity);

            sum_isolation += m.edge_to_bulk_acoustic_isolation_db;
            min_isolation = min_isolation.min(m.edge_to_bulk_acoustic_isolation_db);
            max_isolation = max_isolation.max(m.edge_to_bulk_acoustic_isolation_db);

            sum_stability += m.edge_mode_velocity_stability_fraction;
            min_stability = min_stability.min(m.edge_mode_velocity_stability_fraction);
            max_stability = max_stability.max(m.edge_mode_velocity_stability_fraction);

            sum_dissipation += m.anomalous_edge_acoustic_dissipation_db_per_um;
            min_dissipation = min_dissipation.min(m.anomalous_edge_acoustic_dissipation_db_per_um);
            max_dissipation = max_dissipation.max(m.anomalous_edge_acoustic_dissipation_db_per_um);

            sum_entropy += m.hydrodynamic_entropy_generation_rate_w_per_k;
            min_entropy = min_entropy.min(m.hydrodynamic_entropy_generation_rate_w_per_k);
            max_entropy = max_entropy.max(m.hydrodynamic_entropy_generation_rate_w_per_k);

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;

        ChernSimonsViscometerBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_hall_viscosity_measurement_fidelity: sum_fidelity / n,
            min_hall_viscosity_measurement_fidelity: min_fidelity,
            max_hall_viscosity_measurement_fidelity: max_fidelity,
            mean_edge_to_bulk_acoustic_isolation_db: sum_isolation / n,
            min_edge_to_bulk_acoustic_isolation_db: min_isolation,
            max_edge_to_bulk_acoustic_isolation_db: max_isolation,
            mean_edge_mode_velocity_stability_fraction: sum_stability / n,
            min_edge_mode_velocity_stability_fraction: min_stability,
            max_edge_mode_velocity_stability_fraction: max_stability,
            mean_anomalous_edge_acoustic_dissipation_db_per_um: sum_dissipation / n,
            min_anomalous_edge_acoustic_dissipation_db_per_um: min_dissipation,
            max_anomalous_edge_acoustic_dissipation_db_per_um: max_dissipation,
            mean_hydrodynamic_entropy_generation_rate_w_per_k: sum_entropy / n,
            min_hydrodynamic_entropy_generation_rate_w_per_k: min_entropy,
            max_hydrodynamic_entropy_generation_rate_w_per_k: max_entropy,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
