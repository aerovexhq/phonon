#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum non-Abelian holonomic
//! acoustic gate processors across multi-threaded Rayon workers.

use crate::holonomic_quantum_processor::holonomic_processor_solver::HolonomicQuantumProcessorSolver;
use phonon_models::holonomic_quantum_processor::{
    HolonomicQuantumProcessorMetrics, HolonomicQuantumProcessorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for holonomic quantum processor parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HolonomicProcessorBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_holonomic_gate_fidelity: f64,
    pub min_holonomic_gate_fidelity: f64,
    pub max_holonomic_gate_fidelity: f64,
    pub mean_two_qubit_gate_duration_ns: f64,
    pub min_two_qubit_gate_duration_ns: f64,
    pub max_two_qubit_gate_duration_ns: f64,
    pub mean_geometric_phase_error: f64,
    pub min_geometric_phase_error: f64,
    pub max_geometric_phase_error: f64,
    pub mean_fault_tolerant_logic_depth: f64,
    pub min_fault_tolerant_logic_depth: usize,
    pub max_fault_tolerant_logic_depth: usize,
    pub mean_crosstalk_isolation_db: f64,
    pub min_crosstalk_isolation_db: f64,
    pub max_crosstalk_isolation_db: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct HolonomicProcessorBenchmarkRunner;

impl HolonomicProcessorBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> HolonomicProcessorBenchmarkResult {
        let sweep_params: Vec<HolonomicQuantumProcessorParams> = (0..cycles)
            .map(|i| {
                let qubit_acoustic_freq_ghz = 5.0 + 1.0 * ((i % 13) as f64 / 13.0);
                let driving_field_amplitude_mhz = 85.0 + 15.0 * ((i % 17) as f64 / 17.0);
                let dynamical_phase_cancellation_depth = 0.9945 + 0.0040 * ((i % 19) as f64 / 19.0);
                let inter_qubit_coupling_mhz = 43.0 + 6.0 * ((i % 23) as f64 / 23.0);
                let acoustic_dephasing_rate_khz = 1.7 + 0.6 * ((i % 29) as f64 / 29.0);
                let operating_temp_m_k = 11.0 + 2.0 * ((i % 31) as f64 / 31.0);
                let pulse_shaping_truncation_ns = 3.2 + 0.6 * ((i % 37) as f64 / 37.0);
                let qubit_register_size = 6 + (i % 5);

                HolonomicQuantumProcessorParams::new(
                    qubit_acoustic_freq_ghz,
                    driving_field_amplitude_mhz,
                    dynamical_phase_cancellation_depth,
                    inter_qubit_coupling_mhz,
                    acoustic_dephasing_rate_khz,
                    operating_temp_m_k,
                    pulse_shaping_truncation_ns,
                    qubit_register_size,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<HolonomicQuantumProcessorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = HolonomicQuantumProcessorSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_tau = 0.0;
        let mut min_tau = f64::MAX;
        let mut max_tau = f64::MIN;

        let mut sum_err = 0.0;
        let mut min_err = f64::MAX;
        let mut max_err = f64::MIN;

        let mut sum_depth = 0.0;
        let mut min_depth = usize::MAX;
        let mut max_depth = usize::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut max_iso = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.holonomic_gate_fidelity;
            if m.holonomic_gate_fidelity < min_fid {
                min_fid = m.holonomic_gate_fidelity;
            }
            if m.holonomic_gate_fidelity > max_fid {
                max_fid = m.holonomic_gate_fidelity;
            }

            sum_tau += m.two_qubit_gate_duration_ns;
            if m.two_qubit_gate_duration_ns < min_tau {
                min_tau = m.two_qubit_gate_duration_ns;
            }
            if m.two_qubit_gate_duration_ns > max_tau {
                max_tau = m.two_qubit_gate_duration_ns;
            }

            sum_err += m.geometric_phase_error;
            if m.geometric_phase_error < min_err {
                min_err = m.geometric_phase_error;
            }
            if m.geometric_phase_error > max_err {
                max_err = m.geometric_phase_error;
            }

            sum_depth += m.fault_tolerant_logic_depth as f64;
            if m.fault_tolerant_logic_depth < min_depth {
                min_depth = m.fault_tolerant_logic_depth;
            }
            if m.fault_tolerant_logic_depth > max_depth {
                max_depth = m.fault_tolerant_logic_depth;
            }

            sum_iso += m.crosstalk_isolation_db;
            if m.crosstalk_isolation_db < min_iso {
                min_iso = m.crosstalk_isolation_db;
            }
            if m.crosstalk_isolation_db > max_iso {
                max_iso = m.crosstalk_isolation_db;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        HolonomicProcessorBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_holonomic_gate_fidelity: sum_fid / n,
            min_holonomic_gate_fidelity: min_fid,
            max_holonomic_gate_fidelity: max_fid,
            mean_two_qubit_gate_duration_ns: sum_tau / n,
            min_two_qubit_gate_duration_ns: min_tau,
            max_two_qubit_gate_duration_ns: max_tau,
            mean_geometric_phase_error: sum_err / n,
            min_geometric_phase_error: min_err,
            max_geometric_phase_error: max_err,
            mean_fault_tolerant_logic_depth: sum_depth / n,
            min_fault_tolerant_logic_depth: min_depth,
            max_fault_tolerant_logic_depth: max_depth,
            mean_crosstalk_isolation_db: sum_iso / n,
            min_crosstalk_isolation_db: min_iso,
            max_crosstalk_isolation_db: max_iso,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
