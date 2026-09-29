#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for topological acoustic skyrmion
//! lattices and chiral phononic neuromorphic processing engines across multi-threaded Rayon workers.

use crate::topological_acoustic_skyrmion::skyrmion_solver::TopologicalAcousticSkyrmionSolver;
use phonon_models::topological_acoustic_skyrmion::{
    TopologicalAcousticSkyrmionMetrics, TopologicalAcousticSkyrmionParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for topological acoustic skyrmion parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_synaptic_state_fidelity: f64,
    pub min_synaptic_state_fidelity: f64,
    pub max_synaptic_state_fidelity: f64,
    pub mean_skyrmion_propagation_velocity_mps: f64,
    pub min_skyrmion_propagation_velocity_mps: f64,
    pub max_skyrmion_propagation_velocity_mps: f64,
    pub mean_topological_charge_quantization_error: f64,
    pub min_topological_charge_quantization_error: f64,
    pub max_topological_charge_quantization_error: f64,
    pub mean_neuromorphic_energy_dissipation_aj: f64,
    pub min_neuromorphic_energy_dissipation_aj: f64,
    pub max_neuromorphic_energy_dissipation_aj: f64,
    pub mean_state_retention_isolation_db: f64,
    pub min_state_retention_isolation_db: f64,
    pub max_state_retention_isolation_db: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct SkyrmionBenchmarkRunner;

impl SkyrmionBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> SkyrmionBenchmarkResult {
        let sweep_params: Vec<TopologicalAcousticSkyrmionParams> = (0..cycles)
            .map(|i| {
                let dmi_strength_mj_m2 = 2.1 + 0.4 * ((i % 11) as f64 / 11.0);
                let exchange_stiffness_pj_m = 14.0 + 4.0 * ((i % 13) as f64 / 13.0);
                let anisotropy_mj_m3 = 0.75 + 0.3 * ((i % 17) as f64 / 17.0);
                let gilbert_damping_alpha = 0.012 + 0.006 * ((i % 19) as f64 / 19.0);
                let acoustic_drive_current_ma_um2 = 3.3 + 0.9 * ((i % 23) as f64 / 23.0);
                let lattice_constant_nm = 60.0 + 15.0 * ((i % 29) as f64 / 29.0);
                let skyrmion_diameter_nm = 40.0 + 8.0 * ((i % 31) as f64 / 31.0);
                let cryogenic_temp_k = 1.0 + 1.0 * ((i % 37) as f64 / 37.0);

                TopologicalAcousticSkyrmionParams::new(
                    dmi_strength_mj_m2,
                    exchange_stiffness_pj_m,
                    anisotropy_mj_m3,
                    gilbert_damping_alpha,
                    acoustic_drive_current_ma_um2,
                    lattice_constant_nm,
                    skyrmion_diameter_nm,
                    cryogenic_temp_k,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<TopologicalAcousticSkyrmionMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TopologicalAcousticSkyrmionSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_velocity = 0.0;
        let mut min_velocity = f64::MAX;
        let mut max_velocity = f64::MIN;

        let mut sum_error = 0.0;
        let mut min_error = f64::MAX;
        let mut max_error = f64::MIN;

        let mut sum_dissipation = 0.0;
        let mut min_dissipation = f64::MAX;
        let mut max_dissipation = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.synaptic_state_fidelity;
            if m.synaptic_state_fidelity < min_fidelity {
                min_fidelity = m.synaptic_state_fidelity;
            }
            if m.synaptic_state_fidelity > max_fidelity {
                max_fidelity = m.synaptic_state_fidelity;
            }

            sum_velocity += m.skyrmion_propagation_velocity_mps;
            if m.skyrmion_propagation_velocity_mps < min_velocity {
                min_velocity = m.skyrmion_propagation_velocity_mps;
            }
            if m.skyrmion_propagation_velocity_mps > max_velocity {
                max_velocity = m.skyrmion_propagation_velocity_mps;
            }

            sum_error += m.topological_charge_quantization_error;
            if m.topological_charge_quantization_error < min_error {
                min_error = m.topological_charge_quantization_error;
            }
            if m.topological_charge_quantization_error > max_error {
                max_error = m.topological_charge_quantization_error;
            }

            sum_dissipation += m.neuromorphic_energy_dissipation_aj;
            if m.neuromorphic_energy_dissipation_aj < min_dissipation {
                min_dissipation = m.neuromorphic_energy_dissipation_aj;
            }
            if m.neuromorphic_energy_dissipation_aj > max_dissipation {
                max_dissipation = m.neuromorphic_energy_dissipation_aj;
            }

            sum_isolation += m.state_retention_isolation_db;
            if m.state_retention_isolation_db < min_isolation {
                min_isolation = m.state_retention_isolation_db;
            }
            if m.state_retention_isolation_db > max_isolation {
                max_isolation = m.state_retention_isolation_db;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        SkyrmionBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_synaptic_state_fidelity: sum_fidelity / n,
            min_synaptic_state_fidelity: min_fidelity,
            max_synaptic_state_fidelity: max_fidelity,
            mean_skyrmion_propagation_velocity_mps: sum_velocity / n,
            min_skyrmion_propagation_velocity_mps: min_velocity,
            max_skyrmion_propagation_velocity_mps: max_velocity,
            mean_topological_charge_quantization_error: sum_error / n,
            min_topological_charge_quantization_error: min_error,
            max_topological_charge_quantization_error: max_error,
            mean_neuromorphic_energy_dissipation_aj: sum_dissipation / n,
            min_neuromorphic_energy_dissipation_aj: min_dissipation,
            max_neuromorphic_energy_dissipation_aj: max_dissipation,
            mean_state_retention_isolation_db: sum_isolation / n,
            min_state_retention_isolation_db: min_isolation,
            max_state_retention_isolation_db: max_isolation,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
