//! Multi-threaded Rayon benchmark runner for topological soliton frequency combs across 10,000 parameter sweeps.

use super::soliton_comb_solver::TopologicalSolitonCombSolver;
use phonon_models::topological_soliton_comb::TopologicalSolitonCombParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for topological soliton combs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolitonCombSweepPoint {
    pub span_octaves: f64,
    pub jitter_fs: f64,
    pub snr_db: f64,
    pub duration_ps: f64,
    pub lines_count: usize,
    pub allan_floor: f64,
}

/// Comprehensive benchmark report for topological soliton frequency combs.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalSolitonCombBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean comb bandwidth span in octaves ($\ge 2.0\text{ octaves}$ required).
    pub mean_comb_span_octaves: f64,
    /// Minimum comb bandwidth span in octaves.
    pub min_comb_span_octaves: f64,
    /// Maximum comb bandwidth span in octaves.
    pub max_comb_span_octaves: f64,
    /// Mean timing jitter in femtoseconds ($\le 10.0\text{ fs}$ required).
    pub mean_timing_jitter_fs: f64,
    /// Maximum timing jitter in femtoseconds.
    pub max_timing_jitter_fs: f64,
    /// Mean beat-note SNR in decibels ($\ge 30.0\text{ dB}$ required).
    pub mean_beat_note_snr_db: f64,
    /// Minimum beat-note SNR in decibels.
    pub min_beat_note_snr_db: f64,
    /// Mean soliton pulse duration in picoseconds ($\le 15.0\text{ ps}$ required).
    pub mean_soliton_pulse_duration_ps: f64,
    /// Maximum soliton pulse duration in picoseconds.
    pub max_soliton_pulse_duration_ps: f64,
    /// Mean comb lines count ($\ge 100$ required).
    pub mean_comb_lines_count: usize,
    /// Minimum comb lines count.
    pub min_comb_lines_count: usize,
    /// Mean Allan deviation floor ($\le 1.0\times 10^{-12}$ required).
    pub mean_allan_deviation_floor: f64,
    /// Maximum Allan deviation floor.
    pub max_allan_deviation_floor: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for topological soliton combs.
#[derive(Debug, Clone)]
pub struct TopologicalSolitonCombBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for TopologicalSolitonCombBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl TopologicalSolitonCombBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> TopologicalSolitonCombBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<SolitonCombSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let f0 = 2.5 + 6.5 * frac; // 2.5 to 9.0 GHz
                let fsr = 60.0 + 180.0 * pseudo_hash; // 60 to 240 MHz
                let q_factor = 5.0e5 + 4.5e6 * frac; // 0.5M to 5.0M
                let d2 = 20.0 + 60.0 * pseudo_hash; // 20 to 80 kHz
                let kerr = 15.0 + 55.0 * frac; // 15 to 70 Hz
                let power = 1.0 + 9.0 * pseudo_hash; // 1.0 to 10.0 mW
                let detuning = 2.0 + 4.0 * frac; // 2.0 to 6.0

                let params = TopologicalSolitonCombParams {
                    cavity_resonance_freq_ghz: f0,
                    free_spectral_range_mhz: fsr,
                    loaded_q_factor: q_factor,
                    dispersion_d2_khz: d2,
                    kerr_coefficient_hz: kerr,
                    pump_power_mw: power,
                    normalized_detuning: detuning,
                };

                let solver = TopologicalSolitonCombSolver::new(params);
                let m = solver.solve();

                SolitonCombSweepPoint {
                    span_octaves: m.comb_span_octaves,
                    jitter_fs: m.timing_jitter_fs,
                    snr_db: m.beat_note_snr_db,
                    duration_ps: m.soliton_pulse_duration_ps,
                    lines_count: m.comb_lines_count,
                    allan_floor: m.allan_deviation_floor,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1.0e-6);

        let mut sum_span = 0.0;
        let mut min_span = f64::MAX;
        let mut max_span = f64::MIN;

        let mut sum_jitter = 0.0;
        let mut max_jitter = f64::MIN;

        let mut sum_snr = 0.0;
        let mut min_snr = f64::MAX;

        let mut sum_dur = 0.0;
        let mut max_dur = f64::MIN;

        let mut sum_lines = 0usize;
        let mut min_lines = usize::MAX;

        let mut sum_allan = 0.0;
        let mut max_allan = f64::MIN;

        let mut compliant_count = 0usize;

        for pt in &results {
            sum_span += pt.span_octaves;
            if pt.span_octaves < min_span {
                min_span = pt.span_octaves;
            }
            if pt.span_octaves > max_span {
                max_span = pt.span_octaves;
            }

            sum_jitter += pt.jitter_fs;
            if pt.jitter_fs > max_jitter {
                max_jitter = pt.jitter_fs;
            }

            sum_snr += pt.snr_db;
            if pt.snr_db < min_snr {
                min_snr = pt.snr_db;
            }

            sum_dur += pt.duration_ps;
            if pt.duration_ps > max_dur {
                max_dur = pt.duration_ps;
            }

            sum_lines += pt.lines_count;
            if pt.lines_count < min_lines {
                min_lines = pt.lines_count;
            }

            sum_allan += pt.allan_floor;
            if pt.allan_floor > max_allan {
                max_allan = pt.allan_floor;
            }

            // Strict compliance conditions:
            // 1. Comb span >= 2.0 octaves
            // 2. Timing jitter <= 10.0 fs
            // 3. Beat-note SNR >= 30.0 dB
            // 4. Soliton duration <= 15.0 ps
            // 5. Comb lines count >= 100
            // 6. Allan deviation floor <= 1.0e-12
            if pt.span_octaves >= 2.0
                && pt.jitter_fs <= 10.0
                && pt.snr_db >= 30.0
                && pt.duration_ps <= 15.0
                && pt.lines_count >= 100
                && pt.allan_floor <= 1.0e-12
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        TopologicalSolitonCombBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_comb_span_octaves: sum_span / n,
            min_comb_span_octaves: min_span,
            max_comb_span_octaves: max_span,
            mean_timing_jitter_fs: sum_jitter / n,
            max_timing_jitter_fs: max_jitter,
            mean_beat_note_snr_db: sum_snr / n,
            min_beat_note_snr_db: min_snr,
            mean_soliton_pulse_duration_ps: sum_dur / n,
            max_soliton_pulse_duration_ps: max_dur,
            mean_comb_lines_count: sum_lines / total,
            min_comb_lines_count: min_lines,
            mean_allan_deviation_floor: sum_allan / n,
            max_allan_deviation_floor: max_allan,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
