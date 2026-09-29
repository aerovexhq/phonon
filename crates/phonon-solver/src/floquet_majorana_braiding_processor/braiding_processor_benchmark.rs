#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for topological phononic
//! Floquet-Majorana braiding processors across multi-threaded Rayon workers.

use crate::floquet_majorana_braiding_processor::FloquetMajoranaBraidingProcessorSolver;
use phonon_models::floquet_majorana_braiding_processor::{
    FloquetMajoranaBraidingProcessorMetrics, FloquetMajoranaBraidingProcessorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for Floquet-Majorana braiding processor parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BraidingProcessorBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_braiding_gate_fidelity: f64,
    pub min_braiding_gate_fidelity: f64,
    pub max_braiding_gate_fidelity: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_operation_latency_ns: f64,
    pub min_operation_latency_ns: f64,
    pub max_operation_latency_ns: f64,
    pub mean_edge_state_isolation_db: f64,
    pub min_edge_state_isolation_db: f64,
    pub max_edge_state_isolation_db: f64,
    pub mean_non_abelian_state_purity: f64,
    pub min_non_abelian_state_purity: f64,
    pub max_non_abelian_state_purity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct BraidingProcessorBenchmarkRunner;

impl BraidingProcessorBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> BraidingProcessorBenchmarkResult {
        let sweep_params: Vec<FloquetMajoranaBraidingProcessorParams> = (0..cycles)
            .map(|i| {
                let floquet_drive_freq_ghz = 6.0 + 4.0 * ((i % 29) as f64 / 29.0);
                let floquet_modulation_amplitude_mhz = 55.0 + 30.0 * ((i % 31) as f64 / 31.0);
                let synthetic_gauge_flux_rad = 1.30 + 0.50 * ((i % 23) as f64 / 23.0);
                let phononic_waveguide_length_um = 20.0 + 8.0 * ((i % 37) as f64 / 37.0);
                let majorana_coupling_gap_mhz = 25.0 + 15.0 * ((i % 41) as f64 / 41.0);
                let acoustic_loss_rate_khz = 3.5 + 2.5 * ((i % 33) as f64 / 33.0);
                let operating_temp_m_k = 10.0 + 6.0 * ((i % 43) as f64 / 43.0);
                let braiding_nodes_count = 3 + (i % 4);

                FloquetMajoranaBraidingProcessorParams::new(
                    floquet_drive_freq_ghz,
                    floquet_modulation_amplitude_mhz,
                    synthetic_gauge_flux_rad,
                    phononic_waveguide_length_um,
                    majorana_coupling_gap_mhz,
                    acoustic_loss_rate_khz,
                    operating_temp_m_k,
                    braiding_nodes_count,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<FloquetMajoranaBraidingProcessorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = FloquetMajoranaBraidingProcessorSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_tau = 0.0;
        let mut min_tau = f64::MAX;
        let mut max_tau = f64::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut max_iso = f64::MIN;

        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;
        let mut max_purity = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.braiding_gate_fidelity;
            if m.braiding_gate_fidelity < min_fid {
                min_fid = m.braiding_gate_fidelity;
            }
            if m.braiding_gate_fidelity > max_fid {
                max_fid = m.braiding_gate_fidelity;
            }

            sum_gap += m.topological_protection_gap_mhz;
            if m.topological_protection_gap_mhz < min_gap {
                min_gap = m.topological_protection_gap_mhz;
            }
            if m.topological_protection_gap_mhz > max_gap {
                max_gap = m.topological_protection_gap_mhz;
            }

            sum_tau += m.operation_latency_ns;
            if m.operation_latency_ns < min_tau {
                min_tau = m.operation_latency_ns;
            }
            if m.operation_latency_ns > max_tau {
                max_tau = m.operation_latency_ns;
            }

            sum_iso += m.edge_state_isolation_db;
            if m.edge_state_isolation_db < min_iso {
                min_iso = m.edge_state_isolation_db;
            }
            if m.edge_state_isolation_db > max_iso {
                max_iso = m.edge_state_isolation_db;
            }

            sum_purity += m.non_abelian_state_purity;
            if m.non_abelian_state_purity < min_purity {
                min_purity = m.non_abelian_state_purity;
            }
            if m.non_abelian_state_purity > max_purity {
                max_purity = m.non_abelian_state_purity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let count = cycles.max(1) as f64;
        let compliance_fraction = (compliant_count as f64) / count;

        BraidingProcessorBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_braiding_gate_fidelity: sum_fid / count,
            min_braiding_gate_fidelity: min_fid,
            max_braiding_gate_fidelity: max_fid,
            mean_topological_protection_gap_mhz: sum_gap / count,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_operation_latency_ns: sum_tau / count,
            min_operation_latency_ns: min_tau,
            max_operation_latency_ns: max_tau,
            mean_edge_state_isolation_db: sum_iso / count,
            min_edge_state_isolation_db: min_iso,
            max_edge_state_isolation_db: max_iso,
            mean_non_abelian_state_purity: sum_purity / count,
            min_non_abelian_state_purity: min_purity,
            max_non_abelian_state_purity: max_purity,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
