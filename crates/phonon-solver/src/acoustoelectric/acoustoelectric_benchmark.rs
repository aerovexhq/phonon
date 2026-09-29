//! Multi-threaded Rayon benchmark for single-electron acoustoelectric pumping,
//! current quantization precision, and flying qubit entanglement.

use super::single_electron_pump_solver::SingleElectronPumpSolver;
use phonon_models::acoustoelectric::{DynamicQuantumDot, PiezoelectricSawParams, SplitGateChannel};
use rayon::prelude::*;
use std::time::Instant;

/// Single acoustic cycle sweep result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustoelectricSweepPoint {
    pub frequency_hz: f64,
    pub potential_amplitude_v: f64,
    pub gate_voltage_v: f64,
    pub pumped_current_a: f64,
    pub relative_quantization_error: f64,
    pub bell_state_fidelity: f64,
    pub concurrence: f64,
}

/// Comprehensive benchmark report for quantum acoustoelectric charge pumping.
#[derive(Debug, Clone, PartialEq)]
pub struct AcoustoelectricBenchmarkReport {
    /// Total number of parallel acoustic cycle sweeps executed.
    pub total_cycles: usize,
    /// Elapsed benchmark wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in cycles per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean current quantization error $|I / (e f) - 1|$.
    pub mean_quantization_error: f64,
    /// Maximum current quantization error observed across the plateau.
    pub max_quantization_error: f64,
    /// Fraction of sweeps meeting the precision goal $\epsilon_I < 10^{-4}$ (100% required).
    pub high_precision_fraction: f64,
    /// Mean flying qubit Bell state creation fidelity ($\ge 95\%$ required).
    pub mean_bell_fidelity: f64,
    /// Mean entanglement concurrence ($\ge 0.90$ required).
    pub mean_concurrence: f64,
}

/// Parallel benchmark runner for quantum acoustoelectric charge pumping.
#[derive(Debug, Clone)]
pub struct AcoustoelectricBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for AcoustoelectricBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl AcoustoelectricBenchmarkRunner {
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Runs parallel Rayon benchmark across `total_cycles` sweeps.
    pub fn run_parallel_benchmark(&self) -> AcoustoelectricBenchmarkReport {
        let start = Instant::now();

        let sweep_results: Vec<AcoustoelectricSweepPoint> = (0..self.total_cycles)
            .into_par_iter()
            .map(|idx| {
                // Vary SAW frequency from 2.0 GHz to 4.0 GHz
                let freq = 2.0e9 + (idx as f64 % 200.0) * 1.0e7;
                // Vary SAW potential amplitude from 40 mV to 80 mV
                let phi_0 = 0.040 + (idx as f64 % 40.0) * 0.001;
                // Gate voltage within single-electron plateau
                let v_gate = -0.55 + (idx as f64 % 50.0) * 0.002;

                let mut saw_params = PiezoelectricSawParams::gaas_standard_3ghz();
                saw_params.frequency_hz = freq;
                saw_params.potential_amplitude_v = phi_0;

                let channel = SplitGateChannel::new(v_gate, -0.80, 1.0e-6);
                let dot = DynamicQuantumDot::new(saw_params, channel);
                let solver = SingleElectronPumpSolver::new(dot);

                let res = solver.solve();

                AcoustoelectricSweepPoint {
                    frequency_hz: freq,
                    potential_amplitude_v: phi_0,
                    gate_voltage_v: v_gate,
                    pumped_current_a: res.pumped_current_a,
                    relative_quantization_error: res.relative_quantization_error,
                    bell_state_fidelity: res.bell_state_fidelity,
                    concurrence: res.entanglement_concurrence,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (self.total_cycles as f64) / elapsed.max(1e-6);

        let mut sum_err = 0.0;
        let mut max_err = 0.0f64;
        let mut high_precision_count = 0;
        let mut sum_bell = 0.0;
        let mut sum_conc = 0.0;

        for pt in &sweep_results {
            sum_err += pt.relative_quantization_error;
            if pt.relative_quantization_error > max_err {
                max_err = pt.relative_quantization_error;
            }
            if pt.relative_quantization_error < 1.0e-4 {
                high_precision_count += 1;
            }
            sum_bell += pt.bell_state_fidelity;
            sum_conc += pt.concurrence;
        }

        let n = self.total_cycles as f64;
        let mean_err = sum_err / n;
        let high_prec_frac = (high_precision_count as f64) / n;
        let mean_bell = sum_bell / n;
        let mean_conc = sum_conc / n;

        AcoustoelectricBenchmarkReport {
            total_cycles: self.total_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_quantization_error: mean_err,
            max_quantization_error: max_err,
            high_precision_fraction: high_prec_frac,
            mean_bell_fidelity: mean_bell,
            mean_concurrence: mean_conc,
        }
    }
}
