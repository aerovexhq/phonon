#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for programmable chiral phonon networks
//! and high-dimensional quantum acoustic graph states.

use crate::programmable_chiral_graph::ProgrammableChiralGraphSolver;
use phonon_models::programmable_chiral_graph::{
    ProgrammableChiralGraphMetrics, ProgrammableChiralGraphParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for programmable chiral graph sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_graph_entanglement_fidelity: f64,
    pub min_graph_entanglement_fidelity: f64,
    pub mean_topological_edge_purity: f64,
    pub min_topological_edge_purity: f64,
    pub mean_network_nodes_count: f64,
    pub min_network_nodes_count: usize,
    pub mean_switching_time_ns: f64,
    pub max_switching_time_ns: f64,
    pub mean_nullifier_variance_db: f64,
    pub max_nullifier_variance_db: f64,
    pub mean_stabilizer_generator_fidelity: f64,
    pub min_stabilizer_generator_fidelity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct GraphBenchmarkRunner;

impl GraphBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> GraphBenchmarkResult {
        let sweep_params: Vec<ProgrammableChiralGraphParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles.max(1) as f64);
                let network_nodes_count = 64 + (i % 65);
                let acoustic_resonance_ghz = 2.0 + 4.0 * ((i % 60) as f64 / 60.0);
                let initial_squeezing_db = 12.0 + 4.0 * frac;
                let inter_site_coupling_mhz = 20.0 + 20.0 * ((i % 50) as f64 / 50.0);
                let phase_shifter_switching_time_ns = 8.0 + 8.0 * ((i % 40) as f64 / 40.0);
                let chiral_isolation_db = 32.0 + 16.0 * frac;
                let operating_temp_m_k = 10.0 + 20.0 * ((i % 30) as f64 / 30.0);
                let waveguide_propagation_loss_db_per_cm = 0.015 + 0.010 * ((i % 80) as f64 / 80.0);

                ProgrammableChiralGraphParams::new(
                    network_nodes_count,
                    acoustic_resonance_ghz,
                    initial_squeezing_db,
                    inter_site_coupling_mhz,
                    phase_shifter_switching_time_ns,
                    chiral_isolation_db,
                    operating_temp_m_k,
                    waveguide_propagation_loss_db_per_cm,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ProgrammableChiralGraphMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ProgrammableChiralGraphSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;
        let mut sum_nodes = 0.0;
        let mut min_nodes = usize::MAX;
        let mut sum_switch = 0.0;
        let mut max_switch = f64::MIN;
        let mut sum_null = 0.0;
        let mut max_null = f64::MIN;
        let mut sum_stab = 0.0;
        let mut min_stab = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.graph_entanglement_fidelity;
            if m.graph_entanglement_fidelity < min_fid {
                min_fid = m.graph_entanglement_fidelity;
            }

            sum_purity += m.topological_edge_purity;
            if m.topological_edge_purity < min_purity {
                min_purity = m.topological_edge_purity;
            }

            sum_nodes += m.network_nodes_count as f64;
            if m.network_nodes_count < min_nodes {
                min_nodes = m.network_nodes_count;
            }

            sum_switch += m.switching_time_ns;
            if m.switching_time_ns > max_switch {
                max_switch = m.switching_time_ns;
            }

            sum_null += m.nullifier_variance_db;
            if m.nullifier_variance_db > max_null {
                max_null = m.nullifier_variance_db;
            }

            sum_stab += m.stabilizer_generator_fidelity;
            if m.stabilizer_generator_fidelity < min_stab {
                min_stab = m.stabilizer_generator_fidelity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        GraphBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_graph_entanglement_fidelity: sum_fid / n,
            min_graph_entanglement_fidelity: min_fid,
            mean_topological_edge_purity: sum_purity / n,
            min_topological_edge_purity: min_purity,
            mean_network_nodes_count: sum_nodes / n,
            min_network_nodes_count: min_nodes,
            mean_switching_time_ns: sum_switch / n,
            max_switching_time_ns: max_switch,
            mean_nullifier_variance_db: sum_null / n,
            max_nullifier_variance_db: max_null,
            mean_stabilizer_generator_fidelity: sum_stab / n,
            min_stabilizer_generator_fidelity: min_stab,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
