#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Abelian quantum acoustic
//! holonomic gates and geometric phase processors across multi-threaded Rayon workers.

use crate::acoustic_holonomic_processor::AcousticHolonomicProcessorSolver;
use phonon_models::acoustic_holonomic_processor::{
    AcousticHolonomicProcessorMetrics, AcousticHolonomicProcessorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for acoustic holonomic processor parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HolonomicBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_holonomic_gate_fidelity: f64,
    pub min_holonomic_gate_fidelity: f64,
    pub max_holonomic_gate_fidelity: f64,
    pub mean_gate_operation_time_ns: f64,
    pub min_gate_operation_time_ns: f64,
    pub max_gate_operation_time_ns: f64,
    pub mean_gate_error_rate: f64,
    pub min_gate_error_rate: f64,
    pub max_gate_error_rate: f64,
    pub mean_two_qubit_entangling_fidelity: f64,
    pub min_two_qubit_entangling_fidelity: f64,
    pub max_two_qubit_entangling_fidelity: f64,
    pub mean_geometric_purity: f64,
    pub min_geometric_purity: f64,
    pub max_geometric_purity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct HolonomicBenchmarkRunner;

impl HolonomicBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> HolonomicBenchmarkResult {
        let sweep_params: Vec<AcousticHolonomicProcessorParams> = (0..cycles)
            .map(|i| {
                let acoustic_resonance_ghz = 5.0 + 2.0 * ((i % 29) as f64 / 29.0);
                let piezoelectric_drive_amplitude_mhz = 32.0 + 16.0 * ((i % 31) as f64 / 31.0);
                let wilczek_zee_phase_rad = 1.45 + 0.20 * ((i % 23) as f64 / 23.0);
                let dynamical_phase_cancellation_ratio = 0.985 + 0.014 * ((i % 37) as f64 / 37.0);
                let acoustic_damping_rate_khz = 4.0 + 2.5 * ((i % 41) as f64 / 41.0);
                let operating_temp_m_k = 12.0 + 5.0 * ((i % 43) as f64 / 43.0);
                let qubit_coupling_rate_mhz = 18.0 + 8.0 * ((i % 33) as f64 / 33.0);
                let pulse_rise_time_ns = 4.0 + 2.0 * ((i % 27) as f64 / 27.0);

                AcousticHolonomicProcessorParams::new(
                    acoustic_resonance_ghz,
                    piezoelectric_drive_amplitude_mhz,
                    wilczek_zee_phase_rad,
                    dynamical_phase_cancellation_ratio,
                    acoustic_damping_rate_khz,
                    operating_temp_m_k,
                    qubit_coupling_rate_mhz,
                    pulse_rise_time_ns,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<AcousticHolonomicProcessorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = AcousticHolonomicProcessorSolver::new(*p);
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

        let mut sum_2q = 0.0;
        let mut min_2q = f64::MAX;
        let mut max_2q = f64::MIN;

        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;
        let mut max_purity = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.holonomic_gate_fidelity;
            if m.holonomic_gate_fidelity < min_fid {
                min_fid = m.holonomic_gate_fidelity;
            }
            if m.holonomic_gate_fidelity > max_fid {
                max_fid = m.holonomic_gate_fidelity;
            }

            sum_tau += m.gate_operation_time_ns;
            if m.gate_operation_time_ns < min_tau {
                min_tau = m.gate_operation_time_ns;
            }
            if m.gate_operation_time_ns > max_tau {
                max_tau = m.gate_operation_time_ns;
            }

            sum_err += m.gate_error_rate;
            if m.gate_error_rate < min_err {
                min_err = m.gate_error_rate;
            }
            if m.gate_error_rate > max_err {
                max_err = m.gate_error_rate;
            }

            sum_2q += m.two_qubit_entangling_fidelity;
            if m.two_qubit_entangling_fidelity < min_2q {
                min_2q = m.two_qubit_entangling_fidelity;
            }
            if m.two_qubit_entangling_fidelity > max_2q {
                max_2q = m.two_qubit_entangling_fidelity;
            }

            sum_purity += m.geometric_purity;
            if m.geometric_purity < min_purity {
                min_purity = m.geometric_purity;
            }
            if m.geometric_purity > max_purity {
                max_purity = m.geometric_purity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let count = cycles.max(1) as f64;
        let compliance_fraction = (compliant_count as f64) / count;

        HolonomicBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_holonomic_gate_fidelity: sum_fid / count,
            min_holonomic_gate_fidelity: min_fid,
            max_holonomic_gate_fidelity: max_fid,
            mean_gate_operation_time_ns: sum_tau / count,
            min_gate_operation_time_ns: min_tau,
            max_gate_operation_time_ns: max_tau,
            mean_gate_error_rate: sum_err / count,
            min_gate_error_rate: min_err,
            max_gate_error_rate: max_err,
            mean_two_qubit_entangling_fidelity: sum_2q / count,
            min_two_qubit_entangling_fidelity: min_2q,
            max_two_qubit_entangling_fidelity: max_2q,
            mean_geometric_purity: sum_purity / count,
            min_geometric_purity: min_purity,
            max_geometric_purity: max_purity,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
