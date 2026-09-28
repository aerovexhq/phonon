//! Multi-threaded Rayon benchmark for JTWPA quantum-limited readout:
//! gain spectra across 4-8 GHz, Caves noise limit verification, and throughput.

use crate::jtwpa::coupled_mode_solver::CoupledModeSolver;
use crate::jtwpa::quantum_noise_solver::QuantumNoiseSolver;
use phonon_models::jtwpa::{
    DispersionEngineeringParams, ParametricProcessParams, SnailElementParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark performance and physics report for Phase 52.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JtwpaBenchmarkReport {
    /// Peak signal power gain in dB across 4-8 GHz (> 20 dB).
    pub peak_gain_db: f64,
    /// 3-dB gain amplification bandwidth in GHz (>= 3.0 GHz).
    pub gain_bandwidth_ghz: f64,
    /// Added noise quanta N_add at peak gain (Caves limit <= 0.505).
    pub added_noise_quanta: f64,
    /// Added noise temperature T_add in Kelvin.
    pub added_noise_temperature_k: f64,
    /// 1-dB compression saturation power in dBm (> -90 dBm).
    pub saturation_power_1db_dbm: f64,
    /// Maximum Manley-Rowe photon conservation error (< 1e-6).
    pub max_manley_rowe_error: f64,
    /// Total signal readout pulses simulated.
    pub total_pulses_evaluated: usize,
    /// Simulation throughput in pulse evaluations per second (> 50,000 / sec).
    pub throughput_pulses_per_sec: f64,
    /// Elapsed wall-clock time in seconds.
    pub elapsed_seconds: f64,
}

/// Runner for high-throughput Phase 52 benchmarks.
pub struct JtwpaBenchmarkRunner;

impl JtwpaBenchmarkRunner {
    /// Executes the 10,000-readout-pulse benchmark suite across parallel Rayon threads.
    pub fn run_benchmark(total_pulses: usize) -> JtwpaBenchmarkReport {
        let start = Instant::now();

        // 1. Dispersion engineering & SNAIL parameter verification
        let _dispersion = DispersionEngineeringParams::standard_50ohm_jtwpa();
        let snail = SnailElementParams::standard_kerr_free_snail();
        let p_1db = snail.saturation_power_1db_dbm();

        // 2. Base parametric process configuration (3WM at 6 GHz, 12 GHz pump)
        let base_params = ParametricProcessParams::standard_3wm_amplifier();

        // Compute bandwidth
        let (peak_gain, bw_hz, _f_low) =
            CoupledModeSolver::compute_gain_bandwidth(&base_params, 4.0e9, 8.0e9, 40);

        // 3. Parallel Rayon simulation of multi-frequency multiplexed readout pulses
        let pulse_count = total_pulses.max(10_000);
        let f_min = 4.5e9;
        let f_max = 7.5e9;

        let results: Vec<(f64, f64)> = (0..pulse_count)
            .into_par_iter()
            .map(|i| {
                let frac = (i % 100) as f64 / 99.0;
                let f_sig = f_min + frac * (f_max - f_min);

                // Small frequency-dependent phase mismatch
                let df = f_sig - 6.0e9;
                let dk = 2.0 + 3.0e-18 * df.powi(2);

                let mut p = base_params;
                p.signal_freq_hz = f_sig;
                p.phase_mismatch_rad_per_m = dk;

                let mode_res = CoupledModeSolver::solve(&p, 40);
                (mode_res.signal_gain_linear, mode_res.manley_rowe_error)
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64().max(1e-9);
        let throughput = pulse_count as f64 / elapsed;

        let max_mr_err = results
            .iter()
            .map(|r| r.1)
            .fold(0.0_f64, |acc, err| acc.max(err));

        let avg_gain = results.iter().map(|r| r.0).sum::<f64>() / pulse_count as f64;
        let noise_eval = QuantumNoiseSolver::evaluate(&base_params, avg_gain);

        JtwpaBenchmarkReport {
            peak_gain_db: peak_gain,
            gain_bandwidth_ghz: bw_hz * 1.0e-9,
            added_noise_quanta: noise_eval.added_noise_quanta,
            added_noise_temperature_k: noise_eval.added_noise_temperature_k,
            saturation_power_1db_dbm: p_1db,
            max_manley_rowe_error: max_mr_err,
            total_pulses_evaluated: pulse_count,
            throughput_pulses_per_sec: throughput,
            elapsed_seconds: elapsed,
        }
    }
}
