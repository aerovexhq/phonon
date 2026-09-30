#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for topological acoustic chiral skyrmion-lattice
//! transducers and non-reciprocal magnon-polaron interconnects across multi-threaded Rayon workers.

use crate::chiral_skyrmion_magnon_polaron::ChiralSkyrmionMagnonPolaronSolver;
use phonon_models::chiral_skyrmion_magnon_polaron::{
    ChiralSkyrmionMagnonPolaronMetrics, ChiralSkyrmionMagnonPolaronParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for chiral skyrmion magnon-polaron parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionPolaronBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_topological_hall_angle_deg: f64,
    pub min_topological_hall_angle_deg: f64,
    pub max_topological_hall_angle_deg: f64,
    pub mean_magnon_polaron_transfer_fidelity: f64,
    pub min_magnon_polaron_transfer_fidelity: f64,
    pub max_magnon_polaron_transfer_fidelity: f64,
    pub mean_non_reciprocal_acoustic_isolation_db: f64,
    pub min_non_reciprocal_acoustic_isolation_db: f64,
    pub max_non_reciprocal_acoustic_isolation_db: f64,
    pub mean_skyrmion_drift_velocity_mps: f64,
    pub min_skyrmion_drift_velocity_mps: f64,
    pub max_skyrmion_drift_velocity_mps: f64,
    pub mean_topological_charge_stability_ratio: f64,
    pub min_topological_charge_stability_ratio: f64,
    pub max_topological_charge_stability_ratio: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct SkyrmionPolaronBenchmarkRunner;

impl SkyrmionPolaronBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> SkyrmionPolaronBenchmarkResult {
        let sweep_params: Vec<ChiralSkyrmionMagnonPolaronParams> = (0..cycles)
            .map(|i| {
                let dmi_exchange_strength_mj_m2 = 1.0 + 4.5 * (((i * 7) % 50) as f64 / 50.0);
                let acoustic_strain_drive_amplitude_ppm = 50.0 + 500.0 * (((i * 13) % 45) as f64 / 45.0);
                let magnon_polaron_coupling_mhz = 10.0 + 80.0 * (((i * 11) % 40) as f64 / 40.0);
                let skyrmion_lattice_constant_nm = 40.0 + 180.0 * (((i * 17) % 35) as f64 / 35.0);
                let gilbert_damping_alpha = 0.003 + 0.035 * (((i * 19) % 45) as f64 / 45.0);
                let acoustic_frequency_ghz = 2.0 + 11.0 * (((i * 23) % 30) as f64 / 30.0);
                let cryogenic_temperature_mk = 2.0 + 40.0 * (((i * 29) % 32) as f64 / 32.0);
                let heterostructure_thickness_nm = 4.0 + 40.0 * (((i * 31) % 25) as f64 / 25.0);

                ChiralSkyrmionMagnonPolaronParams::new(
                    dmi_exchange_strength_mj_m2,
                    acoustic_strain_drive_amplitude_ppm,
                    magnon_polaron_coupling_mhz,
                    skyrmion_lattice_constant_nm,
                    gilbert_damping_alpha,
                    acoustic_frequency_ghz,
                    cryogenic_temperature_mk,
                    heterostructure_thickness_nm,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ChiralSkyrmionMagnonPolaronMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ChiralSkyrmionMagnonPolaronSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_hall_angle = 0.0;
        let mut min_hall_angle = f64::MAX;
        let mut max_hall_angle = f64::MIN;

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_velocity = 0.0;
        let mut min_velocity = f64::MAX;
        let mut max_velocity = f64::MIN;

        let mut sum_stability = 0.0;
        let mut min_stability = f64::MAX;
        let mut max_stability = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_hall_angle += m.topological_hall_angle_deg;
            if m.topological_hall_angle_deg < min_hall_angle {
                min_hall_angle = m.topological_hall_angle_deg;
            }
            if m.topological_hall_angle_deg > max_hall_angle {
                max_hall_angle = m.topological_hall_angle_deg;
            }

            sum_fidelity += m.magnon_polaron_transfer_fidelity;
            if m.magnon_polaron_transfer_fidelity < min_fidelity {
                min_fidelity = m.magnon_polaron_transfer_fidelity;
            }
            if m.magnon_polaron_transfer_fidelity > max_fidelity {
                max_fidelity = m.magnon_polaron_transfer_fidelity;
            }

            sum_isolation += m.non_reciprocal_acoustic_isolation_db;
            if m.non_reciprocal_acoustic_isolation_db < min_isolation {
                min_isolation = m.non_reciprocal_acoustic_isolation_db;
            }
            if m.non_reciprocal_acoustic_isolation_db > max_isolation {
                max_isolation = m.non_reciprocal_acoustic_isolation_db;
            }

            sum_velocity += m.skyrmion_drift_velocity_mps;
            if m.skyrmion_drift_velocity_mps < min_velocity {
                min_velocity = m.skyrmion_drift_velocity_mps;
            }
            if m.skyrmion_drift_velocity_mps > max_velocity {
                max_velocity = m.skyrmion_drift_velocity_mps;
            }

            sum_stability += m.topological_charge_stability_ratio;
            if m.topological_charge_stability_ratio < min_stability {
                min_stability = m.topological_charge_stability_ratio;
            }
            if m.topological_charge_stability_ratio > max_stability {
                max_stability = m.topological_charge_stability_ratio;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        let physical_compliance_fraction = (compliant_count as f64) / n;

        SkyrmionPolaronBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_topological_hall_angle_deg: sum_hall_angle / n,
            min_topological_hall_angle_deg: min_hall_angle,
            max_topological_hall_angle_deg: max_hall_angle,
            mean_magnon_polaron_transfer_fidelity: sum_fidelity / n,
            min_magnon_polaron_transfer_fidelity: min_fidelity,
            max_magnon_polaron_transfer_fidelity: max_fidelity,
            mean_non_reciprocal_acoustic_isolation_db: sum_isolation / n,
            min_non_reciprocal_acoustic_isolation_db: min_isolation,
            max_non_reciprocal_acoustic_isolation_db: max_isolation,
            mean_skyrmion_drift_velocity_mps: sum_velocity / n,
            min_skyrmion_drift_velocity_mps: min_velocity,
            max_skyrmion_drift_velocity_mps: max_velocity,
            mean_topological_charge_stability_ratio: sum_stability / n,
            min_topological_charge_stability_ratio: min_stability,
            max_topological_charge_stability_ratio: max_stability,
            physical_compliance_fraction,
        }
    }
}
