#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Abelian quantum acoustic
//! fault-tolerant surface codes and chiral Majorana stabilizer simulators across Rayon workers.

use crate::quantum_acoustic_surface_code::QuantumAcousticSurfaceCodeSolver;
use phonon_models::quantum_acoustic_surface_code::{
    QuantumAcousticSurfaceCodeMetrics, QuantumAcousticSurfaceCodeParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for quantum acoustic surface code parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceCodeBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_logical_state_fidelity: f64,
    pub min_logical_state_fidelity: f64,
    pub max_logical_state_fidelity: f64,
    pub mean_fault_tolerant_threshold_error_rate: f64,
    pub min_fault_tolerant_threshold_error_rate: f64,
    pub max_fault_tolerant_threshold_error_rate: f64,
    pub mean_syndrome_decoding_latency_ns: f64,
    pub min_syndrome_decoding_latency_ns: f64,
    pub max_syndrome_decoding_latency_ns: f64,
    pub mean_uncorrectable_logical_error_rate: f64,
    pub min_uncorrectable_logical_error_rate: f64,
    pub max_uncorrectable_logical_error_rate: f64,
    pub mean_inter_stabilizer_crosstalk_isolation_db: f64,
    pub min_inter_stabilizer_crosstalk_isolation_db: f64,
    pub max_inter_stabilizer_crosstalk_isolation_db: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct SurfaceCodeBenchmarkRunner;

impl SurfaceCodeBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> SurfaceCodeBenchmarkResult {
        let sweep_params: Vec<QuantumAcousticSurfaceCodeParams> = (0..cycles)
            .map(|i| {
                let code_distance = 3.5 + 11.0 * (((i * 7) % 50) as f64 / 50.0);
                let physical_error_rate = 0.0002 + 0.018 * (((i * 13) % 45) as f64 / 45.0);
                let syndrome_extraction_time_ns = 15.0 + 270.0 * (((i * 11) % 40) as f64 / 40.0);
                let majorana_coupling_gap_mhz = 12.0 + 65.0 * (((i * 19) % 45) as f64 / 45.0);
                let cryogenic_temperature_mk = 2.0 + 45.0 * (((i * 23) % 35) as f64 / 35.0);
                let acoustic_stabilizer_frequency_ghz =
                    2.5 + 12.0 * (((i * 29) % 30) as f64 / 30.0);
                let inter_stabilizer_pitch_um = 1.5 + 13.0 * (((i * 31) % 40) as f64 / 40.0);
                let decoder_maximum_weight_iterations =
                    15.0 + 180.0 * (((i * 17) % 50) as f64 / 50.0);

                QuantumAcousticSurfaceCodeParams::new(
                    code_distance,
                    physical_error_rate,
                    syndrome_extraction_time_ns,
                    majorana_coupling_gap_mhz,
                    cryogenic_temperature_mk,
                    acoustic_stabilizer_frequency_ghz,
                    inter_stabilizer_pitch_um,
                    decoder_maximum_weight_iterations,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<QuantumAcousticSurfaceCodeMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = QuantumAcousticSurfaceCodeSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_threshold = 0.0;
        let mut min_threshold = f64::MAX;
        let mut max_threshold = f64::MIN;

        let mut sum_latency = 0.0;
        let mut min_latency = f64::MAX;
        let mut max_latency = f64::MIN;

        let mut sum_error_rate = 0.0;
        let mut min_error_rate = f64::MAX;
        let mut max_error_rate = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.logical_state_fidelity;
            if m.logical_state_fidelity < min_fidelity {
                min_fidelity = m.logical_state_fidelity;
            }
            if m.logical_state_fidelity > max_fidelity {
                max_fidelity = m.logical_state_fidelity;
            }

            sum_threshold += m.fault_tolerant_threshold_error_rate;
            if m.fault_tolerant_threshold_error_rate < min_threshold {
                min_threshold = m.fault_tolerant_threshold_error_rate;
            }
            if m.fault_tolerant_threshold_error_rate > max_threshold {
                max_threshold = m.fault_tolerant_threshold_error_rate;
            }

            sum_latency += m.syndrome_decoding_latency_ns;
            if m.syndrome_decoding_latency_ns < min_latency {
                min_latency = m.syndrome_decoding_latency_ns;
            }
            if m.syndrome_decoding_latency_ns > max_latency {
                max_latency = m.syndrome_decoding_latency_ns;
            }

            sum_error_rate += m.uncorrectable_logical_error_rate;
            if m.uncorrectable_logical_error_rate < min_error_rate {
                min_error_rate = m.uncorrectable_logical_error_rate;
            }
            if m.uncorrectable_logical_error_rate > max_error_rate {
                max_error_rate = m.uncorrectable_logical_error_rate;
            }

            sum_isolation += m.inter_stabilizer_crosstalk_isolation_db;
            if m.inter_stabilizer_crosstalk_isolation_db < min_isolation {
                min_isolation = m.inter_stabilizer_crosstalk_isolation_db;
            }
            if m.inter_stabilizer_crosstalk_isolation_db > max_isolation {
                max_isolation = m.inter_stabilizer_crosstalk_isolation_db;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let total = cycles as f64;
        SurfaceCodeBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_logical_state_fidelity: sum_fidelity / total,
            min_logical_state_fidelity: min_fidelity,
            max_logical_state_fidelity: max_fidelity,
            mean_fault_tolerant_threshold_error_rate: sum_threshold / total,
            min_fault_tolerant_threshold_error_rate: min_threshold,
            max_fault_tolerant_threshold_error_rate: max_threshold,
            mean_syndrome_decoding_latency_ns: sum_latency / total,
            min_syndrome_decoding_latency_ns: min_latency,
            max_syndrome_decoding_latency_ns: max_latency,
            mean_uncorrectable_logical_error_rate: sum_error_rate / total,
            min_uncorrectable_logical_error_rate: min_error_rate,
            max_uncorrectable_logical_error_rate: max_error_rate,
            mean_inter_stabilizer_crosstalk_isolation_db: sum_isolation / total,
            min_inter_stabilizer_crosstalk_isolation_db: min_isolation,
            max_inter_stabilizer_crosstalk_isolation_db: max_isolation,
            physical_compliance_fraction: (compliant_count as f64) / total,
        }
    }
}
