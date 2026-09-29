#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Hermitian higher-order
//! topological phononic lasers across multi-threaded Rayon workers.

use crate::non_hermitian_quadrupole_laser::quadrupole_laser_solver::NonHermitianQuadrupoleLaserSolver;
use phonon_models::non_hermitian_quadrupole_laser::{
    NonHermitianQuadrupoleLaserMetrics, NonHermitianQuadrupoleLaserParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for non-Hermitian quadrupole laser parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadrupoleLaserBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_corner_mode_lasing_fidelity: f64,
    pub min_corner_mode_lasing_fidelity: f64,
    pub max_corner_mode_lasing_fidelity: f64,
    pub mean_fractional_frequency_instability: f64,
    pub min_fractional_frequency_instability: f64,
    pub max_fractional_frequency_instability: f64,
    pub mean_side_mode_suppression_ratio_db: f64,
    pub min_side_mode_suppression_ratio_db: f64,
    pub max_side_mode_suppression_ratio_db: f64,
    pub mean_topological_corner_mode_lifetime_ms: f64,
    pub min_topological_corner_mode_lifetime_ms: f64,
    pub max_topological_corner_mode_lifetime_ms: f64,
    pub mean_pt_symmetry_confinement_ratio: f64,
    pub min_pt_symmetry_confinement_ratio: f64,
    pub max_pt_symmetry_confinement_ratio: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct QuadrupoleLaserBenchmarkRunner;

impl QuadrupoleLaserBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> QuadrupoleLaserBenchmarkResult {
        let sweep_params: Vec<NonHermitianQuadrupoleLaserParams> = (0..cycles)
            .map(|i| {
                let pump_gain_rate_khz = 115.0 + 10.0 * ((i % 11) as f64 / 11.0);
                let loss_dissipation_rate_khz = 105.0 + 10.0 * ((i % 13) as f64 / 13.0);
                let quadrupole_coupling_mhz = 26.0 + 4.0 * ((i % 17) as f64 / 17.0);
                let corner_confinement_factor = 0.86 + 0.04 * ((i % 19) as f64 / 19.0);
                let acoustic_resonator_frequency_ghz = 4.5 + 0.6 * ((i % 23) as f64 / 23.0);
                let cryogenic_temp_mk = 13.0 + 4.0 * ((i % 29) as f64 / 29.0);
                let non_linear_saturation_parameter = 0.010 + 0.004 * ((i % 31) as f64 / 31.0);
                let lattice_dimension = 10 + (i % 5);

                NonHermitianQuadrupoleLaserParams::new(
                    pump_gain_rate_khz,
                    loss_dissipation_rate_khz,
                    quadrupole_coupling_mhz,
                    corner_confinement_factor,
                    acoustic_resonator_frequency_ghz,
                    cryogenic_temp_mk,
                    non_linear_saturation_parameter,
                    lattice_dimension,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<NonHermitianQuadrupoleLaserMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = NonHermitianQuadrupoleLaserSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_instability = 0.0;
        let mut min_instability = f64::MAX;
        let mut max_instability = f64::MIN;

        let mut sum_smsr = 0.0;
        let mut min_smsr = f64::MAX;
        let mut max_smsr = f64::MIN;

        let mut sum_lifetime = 0.0;
        let mut min_lifetime = f64::MAX;
        let mut max_lifetime = f64::MIN;

        let mut sum_pt_ratio = 0.0;
        let mut min_pt_ratio = f64::MAX;
        let mut max_pt_ratio = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.corner_mode_lasing_fidelity;
            if m.corner_mode_lasing_fidelity < min_fidelity {
                min_fidelity = m.corner_mode_lasing_fidelity;
            }
            if m.corner_mode_lasing_fidelity > max_fidelity {
                max_fidelity = m.corner_mode_lasing_fidelity;
            }

            sum_instability += m.fractional_frequency_instability;
            if m.fractional_frequency_instability < min_instability {
                min_instability = m.fractional_frequency_instability;
            }
            if m.fractional_frequency_instability > max_instability {
                max_instability = m.fractional_frequency_instability;
            }

            sum_smsr += m.side_mode_suppression_ratio_db;
            if m.side_mode_suppression_ratio_db < min_smsr {
                min_smsr = m.side_mode_suppression_ratio_db;
            }
            if m.side_mode_suppression_ratio_db > max_smsr {
                max_smsr = m.side_mode_suppression_ratio_db;
            }

            sum_lifetime += m.topological_corner_mode_lifetime_ms;
            if m.topological_corner_mode_lifetime_ms < min_lifetime {
                min_lifetime = m.topological_corner_mode_lifetime_ms;
            }
            if m.topological_corner_mode_lifetime_ms > max_lifetime {
                max_lifetime = m.topological_corner_mode_lifetime_ms;
            }

            sum_pt_ratio += m.pt_symmetry_confinement_ratio;
            if m.pt_symmetry_confinement_ratio < min_pt_ratio {
                min_pt_ratio = m.pt_symmetry_confinement_ratio;
            }
            if m.pt_symmetry_confinement_ratio > max_pt_ratio {
                max_pt_ratio = m.pt_symmetry_confinement_ratio;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        QuadrupoleLaserBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_corner_mode_lasing_fidelity: sum_fidelity / n,
            min_corner_mode_lasing_fidelity: min_fidelity,
            max_corner_mode_lasing_fidelity: max_fidelity,
            mean_fractional_frequency_instability: sum_instability / n,
            min_fractional_frequency_instability: min_instability,
            max_fractional_frequency_instability: max_instability,
            mean_side_mode_suppression_ratio_db: sum_smsr / n,
            min_side_mode_suppression_ratio_db: min_smsr,
            max_side_mode_suppression_ratio_db: max_smsr,
            mean_topological_corner_mode_lifetime_ms: sum_lifetime / n,
            min_topological_corner_mode_lifetime_ms: min_lifetime,
            max_topological_corner_mode_lifetime_ms: max_lifetime,
            mean_pt_symmetry_confinement_ratio: sum_pt_ratio / n,
            min_pt_symmetry_confinement_ratio: min_pt_ratio,
            max_pt_symmetry_confinement_ratio: max_pt_ratio,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
