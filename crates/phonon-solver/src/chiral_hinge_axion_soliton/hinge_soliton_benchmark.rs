#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for 3D topological acoustic higher-order
//! axion insulators and chiral hinge soliton networks across multi-threaded Rayon workers.

use crate::chiral_hinge_axion_soliton::ChiralHingeAxionSolitonSolver;
use phonon_models::chiral_hinge_axion_soliton::{
    ChiralHingeAxionSolitonMetrics, ChiralHingeAxionSolitonParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for chiral hinge axion soliton parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HingeAxionBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_hinge_state_transmission_fidelity: f64,
    pub min_hinge_state_transmission_fidelity: f64,
    pub max_hinge_state_transmission_fidelity: f64,
    pub mean_topological_axion_gap_mhz: f64,
    pub min_topological_axion_gap_mhz: f64,
    pub max_topological_axion_gap_mhz: f64,
    pub mean_non_linear_harmonic_distortion_db: f64,
    pub min_non_linear_harmonic_distortion_db: f64,
    pub max_non_linear_harmonic_distortion_db: f64,
    pub mean_inter_hinge_crosstalk_isolation_db: f64,
    pub min_inter_hinge_crosstalk_isolation_db: f64,
    pub max_inter_hinge_crosstalk_isolation_db: f64,
    pub mean_hinge_soliton_group_velocity_mps: f64,
    pub min_hinge_soliton_group_velocity_mps: f64,
    pub max_hinge_soliton_group_velocity_mps: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct HingeAxionBenchmarkRunner;

impl HingeAxionBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> HingeAxionBenchmarkResult {
        let sweep_params: Vec<ChiralHingeAxionSolitonParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let axion_angle = std::f64::consts::PI + 0.25 * ((i as f64 * 0.17).sin());
                let coupling_alpha = 1.20 + 2.0 * frac;
                let bulk_gap_mhz = 32.0 + 30.0 * (((i * 7) % 50) as f64 / 50.0);
                let pulse_width_ns = 1.0 + 2.5 * (((i * 13) % 45) as f64 / 45.0);
                let non_linearity = 0.005 + 0.025 * frac;
                let operating_freq_ghz = 3.0 + 3.0 * (((i * 11) % 40) as f64 / 40.0);
                let cryo_temp_mk = 5.0 + 20.0 * (((i * 17) % 35) as f64 / 35.0);
                let lattice_dim = 10 + ((i * 19) % 15);

                ChiralHingeAxionSolitonParams::new(
                    axion_angle,
                    coupling_alpha,
                    bulk_gap_mhz,
                    pulse_width_ns,
                    non_linearity,
                    operating_freq_ghz,
                    cryo_temp_mk,
                    lattice_dim,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ChiralHingeAxionSolitonMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ChiralHingeAxionSolitonSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_distortion = 0.0;
        let mut min_distortion = f64::MAX;
        let mut max_distortion = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_velocity = 0.0;
        let mut min_velocity = f64::MAX;
        let mut max_velocity = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.hinge_state_transmission_fidelity;
            if m.hinge_state_transmission_fidelity < min_fidelity {
                min_fidelity = m.hinge_state_transmission_fidelity;
            }
            if m.hinge_state_transmission_fidelity > max_fidelity {
                max_fidelity = m.hinge_state_transmission_fidelity;
            }

            sum_gap += m.topological_axion_gap_mhz;
            if m.topological_axion_gap_mhz < min_gap {
                min_gap = m.topological_axion_gap_mhz;
            }
            if m.topological_axion_gap_mhz > max_gap {
                max_gap = m.topological_axion_gap_mhz;
            }

            sum_distortion += m.non_linear_harmonic_distortion_db;
            if m.non_linear_harmonic_distortion_db < min_distortion {
                min_distortion = m.non_linear_harmonic_distortion_db;
            }
            if m.non_linear_harmonic_distortion_db > max_distortion {
                max_distortion = m.non_linear_harmonic_distortion_db;
            }

            sum_isolation += m.inter_hinge_crosstalk_isolation_db;
            if m.inter_hinge_crosstalk_isolation_db < min_isolation {
                min_isolation = m.inter_hinge_crosstalk_isolation_db;
            }
            if m.inter_hinge_crosstalk_isolation_db > max_isolation {
                max_isolation = m.inter_hinge_crosstalk_isolation_db;
            }

            sum_velocity += m.hinge_soliton_group_velocity_mps;
            if m.hinge_soliton_group_velocity_mps < min_velocity {
                min_velocity = m.hinge_soliton_group_velocity_mps;
            }
            if m.hinge_soliton_group_velocity_mps > max_velocity {
                max_velocity = m.hinge_soliton_group_velocity_mps;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        let physical_compliance_fraction = (compliant_count as f64) / n;

        HingeAxionBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_hinge_state_transmission_fidelity: sum_fidelity / n,
            min_hinge_state_transmission_fidelity: min_fidelity,
            max_hinge_state_transmission_fidelity: max_fidelity,
            mean_topological_axion_gap_mhz: sum_gap / n,
            min_topological_axion_gap_mhz: min_gap,
            max_topological_axion_gap_mhz: max_gap,
            mean_non_linear_harmonic_distortion_db: sum_distortion / n,
            min_non_linear_harmonic_distortion_db: min_distortion,
            max_non_linear_harmonic_distortion_db: max_distortion,
            mean_inter_hinge_crosstalk_isolation_db: sum_isolation / n,
            min_inter_hinge_crosstalk_isolation_db: min_isolation,
            max_inter_hinge_crosstalk_isolation_db: max_isolation,
            mean_hinge_soliton_group_velocity_mps: sum_velocity / n,
            min_hinge_soliton_group_velocity_mps: min_velocity,
            max_hinge_soliton_group_velocity_mps: max_velocity,
            physical_compliance_fraction,
        }
    }
}
