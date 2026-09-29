//! Parallel parameter sweep benchmark suite for quantum acoustic topological Chern circulators.

use crate::topological_chern_circulator::TopologicalChernCirculatorSolver;
use phonon_models::topological_chern_circulator::{
    TopologicalChernCirculatorMetrics, TopologicalChernCirculatorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for topological Chern circulator sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChernCirculatorBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_forward_transmission: f64,
    pub min_forward_transmission: f64,
    pub mean_isolation_db: f64,
    pub min_isolation_db: f64,
    pub mean_bandgap_ratio: f64,
    pub min_bandgap_ratio: f64,
    pub mean_backscattering_db: f64,
    pub max_backscattering_db: f64,
    pub mean_insertion_loss_db: f64,
    pub max_insertion_loss_db: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct ChernCirculatorBenchmarkRunner;

impl ChernCirculatorBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> ChernCirculatorBenchmarkResult {
        let sweep_params: Vec<TopologicalChernCirculatorParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let lattice_constant_um = 1.5 + 1.5 * frac;
                let center_frequency_ghz = 3.0 + 3.0 * ((i % 60) as f64 / 60.0);
                let synthetic_angular_momentum_mhz = 75.0 + 50.0 * frac;
                let inter_site_coupling_mhz = 30.0 + 20.0 * ((i % 50) as f64 / 50.0);
                let defect_disorder_fraction = 0.02 + 0.04 * ((i % 80) as f64 / 80.0);
                let operating_temp_m_k = 10.0 + 30.0 * ((i % 40) as f64 / 40.0);
                let circulator_ports_count = 3 + (i % 3);
                let acoustic_intrinsic_q = 4.0e5 + 4.0e5 * frac;

                TopologicalChernCirculatorParams::new(
                    lattice_constant_um,
                    center_frequency_ghz,
                    synthetic_angular_momentum_mhz,
                    inter_site_coupling_mhz,
                    defect_disorder_fraction,
                    operating_temp_m_k,
                    circulator_ports_count,
                    acoustic_intrinsic_q,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<TopologicalChernCirculatorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TopologicalChernCirculatorSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fwd = 0.0;
        let mut min_fwd = f64::MAX;
        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut sum_refl = 0.0;
        let mut max_refl = f64::MIN;
        let mut sum_il = 0.0;
        let mut max_il = f64::MIN;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_fwd += m.forward_transmission;
            if m.forward_transmission < min_fwd {
                min_fwd = m.forward_transmission;
            }

            sum_iso += m.non_reciprocal_isolation_db;
            if m.non_reciprocal_isolation_db < min_iso {
                min_iso = m.non_reciprocal_isolation_db;
            }

            sum_gap += m.topological_bandgap_ratio;
            if m.topological_bandgap_ratio < min_gap {
                min_gap = m.topological_bandgap_ratio;
            }

            sum_refl += m.backscattering_reflection_db;
            if m.backscattering_reflection_db > max_refl {
                max_refl = m.backscattering_reflection_db;
            }

            sum_il += m.insertion_loss_db;
            if m.insertion_loss_db > max_il {
                max_il = m.insertion_loss_db;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        ChernCirculatorBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_forward_transmission: sum_fwd / n,
            min_forward_transmission: min_fwd,
            mean_isolation_db: sum_iso / n,
            min_isolation_db: min_iso,
            mean_bandgap_ratio: sum_gap / n,
            min_bandgap_ratio: min_gap,
            mean_backscattering_db: sum_refl / n,
            max_backscattering_db: max_refl,
            mean_insertion_loss_db: sum_il / n,
            max_insertion_loss_db: max_il,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
