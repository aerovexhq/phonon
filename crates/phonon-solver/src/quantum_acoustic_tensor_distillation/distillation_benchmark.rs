#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic tensor
//! network simulators and continuous-variable fault-tolerant magic state
//! distillation across multi-threaded Rayon workers.

use crate::quantum_acoustic_tensor_distillation::QuantumAcousticTensorDistillationSolver;
use phonon_models::quantum_acoustic_tensor_distillation::{
    QuantumAcousticTensorDistillationMetrics, QuantumAcousticTensorDistillationParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for quantum acoustic tensor distillation sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistillationBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_magic_state_fidelity: f64,
    pub min_magic_state_fidelity: f64,
    pub max_magic_state_fidelity: f64,
    pub mean_photon_subtraction_prob: f64,
    pub min_photon_subtraction_prob: f64,
    pub max_photon_subtraction_prob: f64,
    pub mean_distillation_cycle_latency_us: f64,
    pub min_distillation_cycle_latency_us: f64,
    pub max_distillation_cycle_latency_us: f64,
    pub mean_non_gaussian_gate_fidelity: f64,
    pub min_non_gaussian_gate_fidelity: f64,
    pub max_non_gaussian_gate_fidelity: f64,
    pub mean_acoustic_error_threshold: f64,
    pub min_acoustic_error_threshold: f64,
    pub max_acoustic_error_threshold: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct DistillationBenchmarkRunner;

impl DistillationBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> DistillationBenchmarkResult {
        let sweep_params: Vec<QuantumAcousticTensorDistillationParams> = (0..cycles)
            .map(|i| {
                let resonator_modes = 8 + (i % 5) * 4;
                let bond_dimension = 16 + (i % 6) * 4;
                let acoustic_frequency_ghz = 3.0 + 6.0 * ((i % 40) as f64 / 40.0);
                let cavity_q_factor = 1.0e7 + 3.0e7 * ((i % 50) as f64 / 50.0);
                let squeezing_param_r = 1.1 + 0.8 * ((i % 35) as f64 / 35.0);
                let non_linear_coupling_mhz = 14.0 + 16.0 * ((i % 45) as f64 / 45.0);
                let operating_temp_m_k = 5.0 + 10.0 * ((i % 30) as f64 / 30.0);
                let photon_subtraction_efficiency = 0.80 + 0.15 * ((i % 25) as f64 / 25.0);

                QuantumAcousticTensorDistillationParams::new(
                    resonator_modes,
                    bond_dimension,
                    acoustic_frequency_ghz,
                    cavity_q_factor,
                    squeezing_param_r,
                    non_linear_coupling_mhz,
                    operating_temp_m_k,
                    photon_subtraction_efficiency,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<QuantumAcousticTensorDistillationMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = QuantumAcousticTensorDistillationSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_magic = 0.0;
        let mut min_magic = f64::MAX;
        let mut max_magic = f64::MIN;

        let mut sum_sub = 0.0;
        let mut min_sub = f64::MAX;
        let mut max_sub = f64::MIN;

        let mut sum_lat = 0.0;
        let mut min_lat = f64::MAX;
        let mut max_lat = f64::MIN;

        let mut sum_gate = 0.0;
        let mut min_gate = f64::MAX;
        let mut max_gate = f64::MIN;

        let mut sum_th = 0.0;
        let mut min_th = f64::MAX;
        let mut max_th = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_magic += m.magic_state_fidelity;
            if m.magic_state_fidelity < min_magic {
                min_magic = m.magic_state_fidelity;
            }
            if m.magic_state_fidelity > max_magic {
                max_magic = m.magic_state_fidelity;
            }

            sum_sub += m.photon_subtraction_prob;
            if m.photon_subtraction_prob < min_sub {
                min_sub = m.photon_subtraction_prob;
            }
            if m.photon_subtraction_prob > max_sub {
                max_sub = m.photon_subtraction_prob;
            }

            sum_lat += m.distillation_cycle_latency_us;
            if m.distillation_cycle_latency_us < min_lat {
                min_lat = m.distillation_cycle_latency_us;
            }
            if m.distillation_cycle_latency_us > max_lat {
                max_lat = m.distillation_cycle_latency_us;
            }

            sum_gate += m.non_gaussian_gate_fidelity;
            if m.non_gaussian_gate_fidelity < min_gate {
                min_gate = m.non_gaussian_gate_fidelity;
            }
            if m.non_gaussian_gate_fidelity > max_gate {
                max_gate = m.non_gaussian_gate_fidelity;
            }

            sum_th += m.acoustic_error_threshold;
            if m.acoustic_error_threshold < min_th {
                min_th = m.acoustic_error_threshold;
            }
            if m.acoustic_error_threshold > max_th {
                max_th = m.acoustic_error_threshold;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        DistillationBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_magic_state_fidelity: sum_magic / n,
            min_magic_state_fidelity: min_magic,
            max_magic_state_fidelity: max_magic,
            mean_photon_subtraction_prob: sum_sub / n,
            min_photon_subtraction_prob: min_sub,
            max_photon_subtraction_prob: max_sub,
            mean_distillation_cycle_latency_us: sum_lat / n,
            min_distillation_cycle_latency_us: min_lat,
            max_distillation_cycle_latency_us: max_lat,
            mean_non_gaussian_gate_fidelity: sum_gate / n,
            min_non_gaussian_gate_fidelity: min_gate,
            max_non_gaussian_gate_fidelity: max_gate,
            mean_acoustic_error_threshold: sum_th / n,
            min_acoustic_error_threshold: min_th,
            max_acoustic_error_threshold: max_th,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
