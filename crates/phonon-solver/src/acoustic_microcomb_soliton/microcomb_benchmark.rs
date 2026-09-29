//! Parallel parameter sweep benchmark suite for quantum acoustic frequency combs
//! and phononic microresonator soliton synthesizers.

#![deny(unsafe_code)]

use crate::acoustic_microcomb_soliton::AcousticMicrocombSolitonSolver;
use phonon_models::acoustic_microcomb_soliton::{
    AcousticMicrocombMetrics, AcousticMicrocombParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark summary report for acoustic microcomb soliton parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MicrocombBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_repetition_rate_ghz: f64,
    pub min_repetition_rate_ghz: f64,
    pub mean_spacing_stability: f64,
    pub max_spacing_stability: f64,
    pub mean_conversion_efficiency: f64,
    pub min_conversion_efficiency: f64,
    pub mean_phase_noise_dbc: f64,
    pub max_phase_noise_dbc: f64,
    pub mean_octave_span: f64,
    pub min_octave_span: f64,
    pub mean_timing_jitter_fs: f64,
    pub max_timing_jitter_fs: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct MicrocombBenchmarkRunner;

impl MicrocombBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> MicrocombBenchmarkResult {
        let sweep_params: Vec<AcousticMicrocombParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let radius_um = 40.0 + 160.0 * ((i % 100) as f64 / 100.0); // 40.0 to 200.0 um
                let fund_freq_ghz = 1.5 + 4.0 * ((i % 80) as f64 / 80.0); // 1.5 to 5.5 GHz
                let quality_factor = 2.5e6 + 5.0e6 * frac; // 2.5e6 to 7.5e6
                let kerr_hz = 1.0 + 8.0 * ((i % 60) as f64 / 60.0); // 1.0 to 9.0 Hz
                let d2_khz = 20.0 + 80.0 * ((i % 70) as f64 / 70.0); // 20.0 to 100.0 kHz
                let pump_power_mw = 8.0 + 16.0 * ((i % 50) as f64 / 50.0); // 8.0 to 24.0 mW
                let detuning = 1.5 + 2.0 * ((i % 90) as f64 / 90.0); // 1.5 to 3.5
                let temp_mk = 5.0 + 15.0 * ((i % 40) as f64 / 40.0); // 5.0 to 20.0 mK

                AcousticMicrocombParams::new(
                    radius_um,
                    fund_freq_ghz,
                    quality_factor,
                    kerr_hz,
                    d2_khz,
                    pump_power_mw,
                    detuning,
                    temp_mk,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<AcousticMicrocombMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = AcousticMicrocombSolitonSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_rep = 0.0;
        let mut min_rep = f64::MAX;
        let mut sum_stab = 0.0;
        let mut max_stab = f64::MIN;
        let mut sum_eff = 0.0;
        let mut min_eff = f64::MAX;
        let mut sum_pn = 0.0;
        let mut max_pn = f64::MIN; // highest (least negative) phase noise
        let mut sum_span = 0.0;
        let mut min_span = f64::MAX;
        let mut sum_jitter = 0.0;
        let mut max_jitter = f64::MIN;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_rep += m.comb_repetition_rate_ghz;
            if m.comb_repetition_rate_ghz < min_rep {
                min_rep = m.comb_repetition_rate_ghz;
            }

            sum_stab += m.comb_spacing_stability;
            if m.comb_spacing_stability > max_stab {
                max_stab = m.comb_spacing_stability;
            }

            sum_eff += m.conversion_efficiency;
            if m.conversion_efficiency < min_eff {
                min_eff = m.conversion_efficiency;
            }

            sum_pn += m.phase_noise_at_10khz_dbc;
            if m.phase_noise_at_10khz_dbc > max_pn {
                max_pn = m.phase_noise_at_10khz_dbc;
            }

            sum_span += m.comb_octave_span;
            if m.comb_octave_span < min_span {
                min_span = m.comb_octave_span;
            }

            sum_jitter += m.timing_jitter_fs;
            if m.timing_jitter_fs > max_jitter {
                max_jitter = m.timing_jitter_fs;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        MicrocombBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_repetition_rate_ghz: sum_rep / n,
            min_repetition_rate_ghz: min_rep,
            mean_spacing_stability: sum_stab / n,
            max_spacing_stability: max_stab,
            mean_conversion_efficiency: sum_eff / n,
            min_conversion_efficiency: min_eff,
            mean_phase_noise_dbc: sum_pn / n,
            max_phase_noise_dbc: max_pn,
            mean_octave_span: sum_span / n,
            min_octave_span: min_span,
            mean_timing_jitter_fs: sum_jitter / n,
            max_timing_jitter_fs: max_jitter,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
