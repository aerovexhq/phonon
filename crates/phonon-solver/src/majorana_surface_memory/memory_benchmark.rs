#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for topological quantum acoustic memory
//! and Majorana surface code decoders across multi-threaded Rayon workers.

use crate::majorana_surface_memory::MajoranaSurfaceMemorySolver;
use phonon_models::majorana_surface_memory::{
    MajoranaSurfaceMemoryMetrics, MajoranaSurfaceMemoryParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for topological quantum acoustic memory sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_quantum_coherence_t2_ms: f64,
    pub min_quantum_coherence_t2_ms: f64,
    pub mean_fault_tolerant_threshold: f64,
    pub min_fault_tolerant_threshold: f64,
    pub mean_syndrome_decoding_latency_us: f64,
    pub max_syndrome_decoding_latency_us: f64,
    pub mean_logical_error_rate: f64,
    pub max_logical_error_rate: f64,
    pub mean_acoustic_qubit_storage_fidelity: f64,
    pub min_acoustic_qubit_storage_fidelity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct MemoryBenchmarkRunner;

impl MemoryBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> MemoryBenchmarkResult {
        let sweep_params: Vec<MajoranaSurfaceMemoryParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles.max(1) as f64);
                let code_distance = 3 + 2 * (i % 6); // 3, 5, 7, 9, 11, 13
                let cavity_resonance_ghz = 2.0 + 8.0 * ((i % 50) as f64 / 50.0);
                let acoustic_quality_factor = 1.0e7 + 4.0e7 * frac;
                let physical_error_rate = 0.0005 + 0.0020 * ((i % 40) as f64 / 40.0);
                let syndrome_extraction_time_ns = 100.0 + 300.0 * ((i % 30) as f64 / 30.0);
                let majorana_coupling_mhz = 15.0 + 40.0 * ((i % 60) as f64 / 60.0);
                let operating_temp_m_k = 4.0 + 16.0 * ((i % 25) as f64 / 25.0);
                let readout_dispersive_shift_mhz = 4.0 + 12.0 * ((i % 35) as f64 / 35.0);

                MajoranaSurfaceMemoryParams::new(
                    code_distance,
                    cavity_resonance_ghz,
                    acoustic_quality_factor,
                    physical_error_rate,
                    syndrome_extraction_time_ns,
                    majorana_coupling_mhz,
                    operating_temp_m_k,
                    readout_dispersive_shift_mhz,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<MajoranaSurfaceMemoryMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = MajoranaSurfaceMemorySolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_t2 = 0.0;
        let mut min_t2 = f64::MAX;
        let mut sum_thresh = 0.0;
        let mut min_thresh = f64::MAX;
        let mut sum_lat = 0.0;
        let mut max_lat = f64::MIN;
        let mut sum_err = 0.0;
        let mut max_err = f64::MIN;
        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_t2 += m.quantum_coherence_t2_ms;
            if m.quantum_coherence_t2_ms < min_t2 {
                min_t2 = m.quantum_coherence_t2_ms;
            }

            sum_thresh += m.fault_tolerant_threshold;
            if m.fault_tolerant_threshold < min_thresh {
                min_thresh = m.fault_tolerant_threshold;
            }

            sum_lat += m.syndrome_decoding_latency_us;
            if m.syndrome_decoding_latency_us > max_lat {
                max_lat = m.syndrome_decoding_latency_us;
            }

            sum_err += m.logical_error_rate;
            if m.logical_error_rate > max_err {
                max_err = m.logical_error_rate;
            }

            sum_fid += m.acoustic_qubit_storage_fidelity;
            if m.acoustic_qubit_storage_fidelity < min_fid {
                min_fid = m.acoustic_qubit_storage_fidelity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        MemoryBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_quantum_coherence_t2_ms: sum_t2 / n,
            min_quantum_coherence_t2_ms: min_t2,
            mean_fault_tolerant_threshold: sum_thresh / n,
            min_fault_tolerant_threshold: min_thresh,
            mean_syndrome_decoding_latency_us: sum_lat / n,
            max_syndrome_decoding_latency_us: max_lat,
            mean_logical_error_rate: sum_err / n,
            max_logical_error_rate: max_err,
            mean_acoustic_qubit_storage_fidelity: sum_fid / n,
            min_acoustic_qubit_storage_fidelity: min_fid,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
