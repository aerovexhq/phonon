#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic chiral spin-mechanical
//! frequency-bin entanglement and phononic Bell state analyzers across multi-threaded Rayon workers.

use crate::chiral_frequency_bin_bell_analyzer::ChiralFrequencyBinBellAnalyzerSolver;
use phonon_models::chiral_frequency_bin_bell_analyzer::{
    ChiralFrequencyBinBellAnalyzerMetrics, ChiralFrequencyBinBellAnalyzerParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for chiral frequency-bin Bell state analyzer parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrequencyBinBellBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_bell_state_measurement_fidelity: f64,
    pub min_bell_state_measurement_fidelity: f64,
    pub max_bell_state_measurement_fidelity: f64,
    pub mean_frequency_bin_mode_indistinguishability: f64,
    pub min_frequency_bin_mode_indistinguishability: f64,
    pub max_frequency_bin_mode_indistinguishability: f64,
    pub mean_crosstalk_quantum_dephasing_rate_hz: f64,
    pub min_crosstalk_quantum_dephasing_rate_hz: f64,
    pub max_crosstalk_quantum_dephasing_rate_hz: f64,
    pub mean_dark_count_probability: f64,
    pub min_dark_count_probability: f64,
    pub max_dark_count_probability: f64,
    pub mean_two_phonon_entanglement_concurrence: f64,
    pub min_two_phonon_entanglement_concurrence: f64,
    pub max_two_phonon_entanglement_concurrence: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct FrequencyBinBellBenchmarkRunner;

impl FrequencyBinBellBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> FrequencyBinBellBenchmarkResult {
        let sweep_params: Vec<ChiralFrequencyBinBellAnalyzerParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let pump_amp = 10.0 + 20.0 * frac; // 10.0 to 30.0 MHz
                let bin_sep = 45.0 + 65.0 * ((i % 55) as f64 / 55.0); // 45.0 to 110.0 MHz
                let spin_coupling = 3.5 + 9.0 * frac; // 3.5 to 12.5 MHz
                let cavity_decay = 30.0 + 120.0 * ((i % 45) as f64 / 45.0); // 30.0 to 150.0 kHz
                let detector_eff = 0.88 + 0.10 * frac; // 0.88 to 0.98
                let chiral_iso = 30.0 + 20.0 * ((i % 65) as f64 / 65.0); // 30.0 to 50.0 dB
                let cryo_temp = 5.0 + 20.0 * ((i % 40) as f64 / 40.0); // 5.0 to 25.0 mK
                let meas_window = 1.0 + 3.0 * frac; // 1.0 to 4.0 us

                ChiralFrequencyBinBellAnalyzerParams::new(
                    pump_amp,
                    bin_sep,
                    spin_coupling,
                    cavity_decay,
                    detector_eff,
                    chiral_iso,
                    cryo_temp,
                    meas_window,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ChiralFrequencyBinBellAnalyzerMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ChiralFrequencyBinBellAnalyzerSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_indist = 0.0;
        let mut min_indist = f64::MAX;
        let mut max_indist = f64::MIN;

        let mut sum_dephasing = 0.0;
        let mut min_dephasing = f64::MAX;
        let mut max_dephasing = f64::MIN;

        let mut sum_dark_count = 0.0;
        let mut min_dark_count = f64::MAX;
        let mut max_dark_count = f64::MIN;

        let mut sum_concurrence = 0.0;
        let mut min_concurrence = f64::MAX;
        let mut max_concurrence = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.bell_state_measurement_fidelity;
            if m.bell_state_measurement_fidelity < min_fidelity {
                min_fidelity = m.bell_state_measurement_fidelity;
            }
            if m.bell_state_measurement_fidelity > max_fidelity {
                max_fidelity = m.bell_state_measurement_fidelity;
            }

            sum_indist += m.frequency_bin_mode_indistinguishability;
            if m.frequency_bin_mode_indistinguishability < min_indist {
                min_indist = m.frequency_bin_mode_indistinguishability;
            }
            if m.frequency_bin_mode_indistinguishability > max_indist {
                max_indist = m.frequency_bin_mode_indistinguishability;
            }

            sum_dephasing += m.crosstalk_quantum_dephasing_rate_hz;
            if m.crosstalk_quantum_dephasing_rate_hz < min_dephasing {
                min_dephasing = m.crosstalk_quantum_dephasing_rate_hz;
            }
            if m.crosstalk_quantum_dephasing_rate_hz > max_dephasing {
                max_dephasing = m.crosstalk_quantum_dephasing_rate_hz;
            }

            sum_dark_count += m.dark_count_probability;
            if m.dark_count_probability < min_dark_count {
                min_dark_count = m.dark_count_probability;
            }
            if m.dark_count_probability > max_dark_count {
                max_dark_count = m.dark_count_probability;
            }

            sum_concurrence += m.two_phonon_entanglement_concurrence;
            if m.two_phonon_entanglement_concurrence < min_concurrence {
                min_concurrence = m.two_phonon_entanglement_concurrence;
            }
            if m.two_phonon_entanglement_concurrence > max_concurrence {
                max_concurrence = m.two_phonon_entanglement_concurrence;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let total_f = cycles as f64;
        let compliance_fraction = (compliant_count as f64) / total_f;

        FrequencyBinBellBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_bell_state_measurement_fidelity: sum_fidelity / total_f,
            min_bell_state_measurement_fidelity: min_fidelity,
            max_bell_state_measurement_fidelity: max_fidelity,
            mean_frequency_bin_mode_indistinguishability: sum_indist / total_f,
            min_frequency_bin_mode_indistinguishability: min_indist,
            max_frequency_bin_mode_indistinguishability: max_indist,
            mean_crosstalk_quantum_dephasing_rate_hz: sum_dephasing / total_f,
            min_crosstalk_quantum_dephasing_rate_hz: min_dephasing,
            max_crosstalk_quantum_dephasing_rate_hz: max_dephasing,
            mean_dark_count_probability: sum_dark_count / total_f,
            min_dark_count_probability: min_dark_count,
            max_dark_count_probability: max_dark_count,
            mean_two_phonon_entanglement_concurrence: sum_concurrence / total_f,
            min_two_phonon_entanglement_concurrence: min_concurrence,
            max_two_phonon_entanglement_concurrence: max_concurrence,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
