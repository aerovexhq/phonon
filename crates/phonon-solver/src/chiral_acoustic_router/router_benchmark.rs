#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for chiral quantum acoustic
//! metamaterial circulators and multi-terminal non-reciprocal router networks across Rayon workers.

use crate::chiral_acoustic_router::ChiralAcousticRouterSolver;
use phonon_models::chiral_acoustic_router::{
    ChiralAcousticRouterMetrics, ChiralAcousticRouterParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for chiral acoustic router parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouterBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_non_reciprocal_isolation_db: f64,
    pub min_non_reciprocal_isolation_db: f64,
    pub max_non_reciprocal_isolation_db: f64,
    pub mean_insertion_loss_db: f64,
    pub min_insertion_loss_db: f64,
    pub max_insertion_loss_db: f64,
    pub mean_phase_coherence_fidelity: f64,
    pub min_phase_coherence_fidelity: f64,
    pub max_phase_coherence_fidelity: f64,
    pub mean_cross_talk_rejection_db: f64,
    pub min_cross_talk_rejection_db: f64,
    pub max_cross_talk_rejection_db: f64,
    pub mean_operating_bandwidth_mhz: f64,
    pub min_operating_bandwidth_mhz: f64,
    pub max_operating_bandwidth_mhz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct RouterBenchmarkRunner;

impl RouterBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> RouterBenchmarkResult {
        let sweep_params: Vec<ChiralAcousticRouterParams> = (0..cycles)
            .map(|i| {
                let center_frequency_ghz = 4.0 + 2.0 * ((i % 29) as f64 / 29.0);
                let synthetic_angular_momentum_mhz = 75.0 + 25.0 * ((i % 31) as f64 / 31.0);
                let ports_count = 3 + (i % 4);
                let odd_viscosity_coefficient = 0.12 + 0.08 * ((i % 37) as f64 / 37.0);
                let resonator_q_factor = 1.5e7 + 1.0e7 * ((i % 23) as f64 / 23.0);
                let waveguide_coupling_rate_mhz = 22.0 + 8.0 * ((i % 19) as f64 / 19.0);
                let operating_temp_m_k = 10.0 + 8.0 * ((i % 43) as f64 / 43.0);
                let fabrication_disorder_fraction = 0.01 + 0.02 * ((i % 17) as f64 / 17.0);

                ChiralAcousticRouterParams::new(
                    center_frequency_ghz,
                    synthetic_angular_momentum_mhz,
                    ports_count,
                    odd_viscosity_coefficient,
                    resonator_q_factor,
                    waveguide_coupling_rate_mhz,
                    operating_temp_m_k,
                    fabrication_disorder_fraction,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ChiralAcousticRouterMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ChiralAcousticRouterSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_is = 0.0;
        let mut min_is = f64::MAX;
        let mut max_is = f64::MIN;

        let mut sum_il = 0.0;
        let mut min_il = f64::MAX;
        let mut max_il = f64::MIN;

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_cr = 0.0;
        let mut min_cr = f64::MAX;
        let mut max_cr = f64::MIN;

        let mut sum_bw = 0.0;
        let mut min_bw = f64::MAX;
        let mut max_bw = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_is += m.non_reciprocal_isolation_db;
            if m.non_reciprocal_isolation_db < min_is {
                min_is = m.non_reciprocal_isolation_db;
            }
            if m.non_reciprocal_isolation_db > max_is {
                max_is = m.non_reciprocal_isolation_db;
            }

            sum_il += m.insertion_loss_db;
            if m.insertion_loss_db < min_il {
                min_il = m.insertion_loss_db;
            }
            if m.insertion_loss_db > max_il {
                max_il = m.insertion_loss_db;
            }

            sum_fid += m.phase_coherence_fidelity;
            if m.phase_coherence_fidelity < min_fid {
                min_fid = m.phase_coherence_fidelity;
            }
            if m.phase_coherence_fidelity > max_fid {
                max_fid = m.phase_coherence_fidelity;
            }

            sum_cr += m.cross_talk_rejection_db;
            if m.cross_talk_rejection_db < min_cr {
                min_cr = m.cross_talk_rejection_db;
            }
            if m.cross_talk_rejection_db > max_cr {
                max_cr = m.cross_talk_rejection_db;
            }

            sum_bw += m.operating_bandwidth_mhz;
            if m.operating_bandwidth_mhz < min_bw {
                min_bw = m.operating_bandwidth_mhz;
            }
            if m.operating_bandwidth_mhz > max_bw {
                max_bw = m.operating_bandwidth_mhz;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let count = cycles.max(1) as f64;
        let compliance_fraction = (compliant_count as f64) / count;

        RouterBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_non_reciprocal_isolation_db: sum_is / count,
            min_non_reciprocal_isolation_db: min_is,
            max_non_reciprocal_isolation_db: max_is,
            mean_insertion_loss_db: sum_il / count,
            min_insertion_loss_db: min_il,
            max_insertion_loss_db: max_il,
            mean_phase_coherence_fidelity: sum_fid / count,
            min_phase_coherence_fidelity: min_fid,
            max_phase_coherence_fidelity: max_fid,
            mean_cross_talk_rejection_db: sum_cr / count,
            min_cross_talk_rejection_db: min_cr,
            max_cross_talk_rejection_db: max_cr,
            mean_operating_bandwidth_mhz: sum_bw / count,
            min_operating_bandwidth_mhz: min_bw,
            max_operating_bandwidth_mhz: max_bw,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
