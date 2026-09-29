#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum phononic non-Abelian
//! anyon colliders and multi-qubit topological braiding interferometers across
//! multi-threaded Rayon workers.

use crate::phononic_anyon_collider::PhononicAnyonColliderSolver;
use phonon_models::phononic_anyon_collider::{
    PhononicAnyonColliderMetrics, PhononicAnyonColliderParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for phononic anyon collider sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColliderBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_collision_visibility: f64,
    pub min_collision_visibility: f64,
    pub mean_cross_correlation_noise_suppression_db: f64,
    pub min_cross_correlation_noise_suppression_db: f64,
    pub mean_braiding_phase_error_rad: f64,
    pub max_braiding_phase_error_rad: f64,
    pub mean_topological_parity_readout_fidelity: f64,
    pub min_topological_parity_readout_fidelity: f64,
    pub mean_anyonic_fano_factor: f64,
    pub max_anyonic_fano_factor: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct ColliderBenchmarkRunner;

impl ColliderBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> ColliderBenchmarkResult {
        let sweep_params: Vec<PhononicAnyonColliderParams> = (0..cycles)
            .map(|i| {
                let acoustic_frequency_ghz = 2.0 + 10.0 * ((i % 50) as f64 / 50.0);
                let splitter_reflectivity = 0.20 + 0.60 * ((i % 40) as f64 / 40.0);
                let anyon_wavepacket_width_ps = 30.0 + 300.0 * ((i % 60) as f64 / 60.0);
                let interferometer_arm_length_um = 10.0 + 60.0 * ((i % 45) as f64 / 45.0);
                let topological_gap_mhz = 120.0 + 400.0 * ((i % 55) as f64 / 55.0);
                let dephasing_rate_khz = 1.0 + 8.0 * ((i % 35) as f64 / 35.0);
                let operating_temp_m_k = 3.0 + 20.0 * ((i % 30) as f64 / 30.0);
                let qubit_count = 2 + (i % 8) * 2; // 2, 4, 6, 8, 10, 12, 14, 16

                PhononicAnyonColliderParams::new(
                    acoustic_frequency_ghz,
                    splitter_reflectivity,
                    anyon_wavepacket_width_ps,
                    interferometer_arm_length_um,
                    topological_gap_mhz,
                    dephasing_rate_khz,
                    operating_temp_m_k,
                    qubit_count,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<PhononicAnyonColliderMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = PhononicAnyonColliderSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_vis = 0.0;
        let mut min_vis = f64::MAX;
        let mut sum_supp = 0.0;
        let mut min_supp = f64::MAX;
        let mut sum_err = 0.0;
        let mut max_err = f64::MIN;
        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut sum_fano = 0.0;
        let mut max_fano = f64::MIN;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_vis += m.collision_visibility;
            if m.collision_visibility < min_vis {
                min_vis = m.collision_visibility;
            }

            sum_supp += m.cross_correlation_noise_suppression_db;
            if m.cross_correlation_noise_suppression_db < min_supp {
                min_supp = m.cross_correlation_noise_suppression_db;
            }

            sum_err += m.braiding_phase_error_rad;
            if m.braiding_phase_error_rad > max_err {
                max_err = m.braiding_phase_error_rad;
            }

            sum_fid += m.topological_parity_readout_fidelity;
            if m.topological_parity_readout_fidelity < min_fid {
                min_fid = m.topological_parity_readout_fidelity;
            }

            sum_fano += m.anyonic_fano_factor;
            if m.anyonic_fano_factor > max_fano {
                max_fano = m.anyonic_fano_factor;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        ColliderBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_collision_visibility: sum_vis / n,
            min_collision_visibility: min_vis,
            mean_cross_correlation_noise_suppression_db: sum_supp / n,
            min_cross_correlation_noise_suppression_db: min_supp,
            mean_braiding_phase_error_rad: sum_err / n,
            max_braiding_phase_error_rad: max_err,
            mean_topological_parity_readout_fidelity: sum_fid / n,
            min_topological_parity_readout_fidelity: min_fid,
            mean_anyonic_fano_factor: sum_fano / n,
            max_anyonic_fano_factor: max_fano,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
