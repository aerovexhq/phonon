//! Multi-threaded Rayon benchmark runner for phononic Kerr microcombs,
//! octave-spanning acoustic combs, and phononic clock stability across 10,000 sweeps.

use super::lugiato_lefever_solver::LugiatoLefeverSolver;
use super::phononic_clock_solver::PhononicClockSolver;
use phonon_models::phononic_microcomb::PhononicMicrocombParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point in phononic microcombs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicMicrocombSweepPoint {
    pub comb_span_octaves: f64,
    pub comb_line_count: usize,
    pub timing_jitter_fs: f64,
    pub soliton_contrast_db: f64,
    pub highest_harmonic_order: usize,
}

/// Comprehensive benchmark report for phononic Kerr microcombs.
#[derive(Debug, Clone, PartialEq)]
pub struct PhononicMicrocombBenchmarkReport {
    /// Total sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean comb span in octaves ($\\ge 1.0\\text{ octave}$ required).
    pub mean_comb_span_octaves: f64,
    /// Minimum comb span in octaves.
    pub min_comb_span_octaves: f64,
    /// Mean number of coherent comb lines ($\\ge 50$ required).
    pub mean_comb_lines: f64,
    /// Minimum number of comb lines.
    pub min_comb_lines: usize,
    /// Mean timing jitter in femtoseconds ($\\le 100\\text{ fs}$ required).
    pub mean_timing_jitter_fs: f64,
    /// Maximum timing jitter in femtoseconds.
    pub max_timing_jitter_fs: f64,
    /// Mean soliton switching extinction contrast in decibels ($\\ge 20.0\\text{ dB}$ required).
    pub mean_soliton_contrast_db: f64,
    /// Minimum soliton switching extinction contrast in decibels.
    pub min_soliton_contrast_db: f64,
    /// Fraction of parameter sweeps satisfying all physical criteria.
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for phononic microcombs.
#[derive(Debug, Clone)]
pub struct PhononicMicrocombBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for PhononicMicrocombBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl PhononicMicrocombBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> PhononicMicrocombBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<PhononicMicrocombSweepPoint> = (0..total)
            .into_par_iter()
            .map(|i| {
                let u = i as f64 / total.max(1) as f64;

                // Parameter sweeping across physical ranges:
                // Center frequency f0 in [2.0, 4.5] GHz
                let f0_ghz = 2.0 + 2.5 * u;
                // FSR in [25.0, 65.0] MHz
                let fsr_mhz = 25.0 + 40.0 * ((i * 7) % total) as f64 / total as f64;
                // Dispersion D2 in [15.0, 45.0] kHz
                let d2_khz = 15.0 + 30.0 * ((i * 11) % total) as f64 / total as f64;
                // Drive power in [3.0, 15.0] mW
                let p_in_mw = 3.0 + 12.0 * ((i * 17) % total) as f64 / total as f64;
                // Detuning ratio Delta/kappa in [2.0, 4.0]
                let detuning = 2.0 + 2.0 * ((i * 23) % total) as f64 / total as f64;

                let params = PhononicMicrocombParams {
                    center_frequency_ghz: f0_ghz,
                    free_spectral_range_mhz: fsr_mhz,
                    dispersion_d2_khz: d2_khz,
                    total_damping_khz: 50.0,
                    kerr_nonlinearity_hz: 15.0,
                    drive_power_mw: p_in_mw,
                    normalized_detuning: detuning,
                };

                let lle_solver = LugiatoLefeverSolver::new(params);
                let comb_metrics = lle_solver.solve_microcomb_metrics();

                let clock_solver = PhononicClockSolver::new(params);
                let soliton_metrics = clock_solver.solve_soliton_metrics();

                PhononicMicrocombSweepPoint {
                    comb_span_octaves: comb_metrics.comb_span_octaves,
                    comb_line_count: soliton_metrics.comb_line_count,
                    timing_jitter_fs: comb_metrics.timing_jitter_fs,
                    soliton_contrast_db: soliton_metrics.soliton_contrast_db,
                    highest_harmonic_order: comb_metrics.highest_harmonic_order,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_span = 0.0;
        let mut min_span = f64::INFINITY;
        let mut sum_lines = 0.0;
        let mut min_lines = usize::MAX;
        let mut sum_jitter = 0.0;
        let mut max_jitter = f64::NEG_INFINITY;
        let mut sum_contrast = 0.0;
        let mut min_contrast = f64::INFINITY;
        let mut compliant_count = 0usize;

        for p in &results {
            sum_span += p.comb_span_octaves;
            if p.comb_span_octaves < min_span {
                min_span = p.comb_span_octaves;
            }

            sum_lines += p.comb_line_count as f64;
            if p.comb_line_count < min_lines {
                min_lines = p.comb_line_count;
            }

            sum_jitter += p.timing_jitter_fs;
            if p.timing_jitter_fs > max_jitter {
                max_jitter = p.timing_jitter_fs;
            }

            sum_contrast += p.soliton_contrast_db;
            if p.soliton_contrast_db < min_contrast {
                min_contrast = p.soliton_contrast_db;
            }

            // Criteria:
            // 1. Comb span >= 1.0 octave
            // 2. Comb line count >= 50
            // 3. Timing jitter <= 100.0 fs
            // 4. Soliton contrast >= 20.0 dB
            // 5. Highest harmonic order >= 10
            if p.comb_span_octaves >= 1.0
                && p.comb_line_count >= 50
                && p.timing_jitter_fs <= 100.0
                && p.soliton_contrast_db >= 20.0
                && p.highest_harmonic_order >= 10
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        PhononicMicrocombBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_comb_span_octaves: sum_span / n,
            min_comb_span_octaves: min_span,
            mean_comb_lines: sum_lines / n,
            min_comb_lines: min_lines,
            mean_timing_jitter_fs: sum_jitter / n,
            max_timing_jitter_fs: max_jitter,
            mean_soliton_contrast_db: sum_contrast / n,
            min_soliton_contrast_db: min_contrast,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
