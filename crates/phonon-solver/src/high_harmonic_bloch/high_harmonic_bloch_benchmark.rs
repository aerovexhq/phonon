//! Multi-threaded Rayon benchmark runner for high-harmonic acoustic Bloch oscillations
//! and phononic frequency synthesizers across 10,000 parameter sweeps.

use super::high_harmonic_bloch_solver::HighHarmonicBlochSolver;
use phonon_models::high_harmonic_bloch::HighHarmonicBlochParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for high-harmonic acoustic Bloch oscillations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighHarmonicBlochSweepPoint {
    pub bloch_freq_ghz: f64,
    pub cutoff_order: usize,
    pub spectral_purity_db: f64,
    pub oscillations_count: f64,
    pub efficiency_pct: f64,
    pub zener_leakage: f64,
}

/// Comprehensive benchmark report for high-harmonic acoustic Bloch oscillations.
#[derive(Debug, Clone, PartialEq)]
pub struct HighHarmonicBlochBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean acoustic Bloch frequency in GHz ($\\ge 50.0\\text{ GHz}$ required).
    pub mean_bloch_freq_ghz: f64,
    /// Minimum acoustic Bloch frequency in GHz.
    pub min_bloch_freq_ghz: f64,
    /// Mean harmonic emission cutoff order ($\\ge 25$ required).
    pub mean_cutoff_order: f64,
    /// Minimum harmonic emission cutoff order.
    pub min_cutoff_order: usize,
    /// Mean spectral purity in dB ($\\ge 45.0\\text{ dB}$ required).
    pub mean_spectral_purity_db: f64,
    /// Minimum spectral purity in dB.
    pub min_spectral_purity_db: f64,
    /// Mean coherent oscillation count ($\\ge 3.0$ required).
    pub mean_oscillations_count: f64,
    /// Minimum coherent oscillation count.
    pub min_oscillations_count: f64,
    /// Mean synthesizer conversion efficiency in percent ($\\ge 15.0\\%$ required).
    pub mean_efficiency_pct: f64,
    /// Minimum synthesizer conversion efficiency in percent.
    pub min_efficiency_pct: f64,
    /// Mean Zener leakage probability ($\\le 0.05$ required).
    pub mean_zener_leakage: f64,
    /// Maximum Zener leakage probability.
    pub max_zener_leakage: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for high-harmonic acoustic Bloch oscillations.
#[derive(Debug, Clone)]
pub struct HighHarmonicBlochBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for HighHarmonicBlochBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl HighHarmonicBlochBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> HighHarmonicBlochBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<HighHarmonicBlochSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let period = 12.0 + 20.0 * frac; // 12 to 32 nm
                let width = 1.0 + 3.0 * pseudo_hash; // 1.0 to 4.0 meV
                let gap = 5.0 + 8.0 * frac; // 5.0 to 13.0 meV
                let force = 0.08 + 0.20 * pseudo_hash; // 0.08 to 0.28 meV/nm
                let deph = 5.0 + 12.0 * frac; // 5.0 to 17.0 ps
                let drive = 1.1 + 1.2 * pseudo_hash; // 1.1 to 2.3

                let params = HighHarmonicBlochParams {
                    superlattice_period_nm: period,
                    miniband_width_mev: width,
                    miniband_gap_mev: gap,
                    effective_force_field_mev_nm: force,
                    dephasing_time_ps: deph,
                    non_linear_drive_factor: drive,
                };

                let solver = HighHarmonicBlochSolver::new(params);
                let m = solver.solve();

                HighHarmonicBlochSweepPoint {
                    bloch_freq_ghz: m.bloch_frequency_ghz,
                    cutoff_order: m.harmonic_cutoff_order,
                    spectral_purity_db: m.spectral_purity_db,
                    oscillations_count: m.coherent_oscillations_count,
                    efficiency_pct: m.synthesizer_efficiency_pct,
                    zener_leakage: m.zener_leakage_prob,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_f = 0.0;
        let mut min_f = f64::MAX;

        let mut sum_cut = 0;
        let mut min_cut = usize::MAX;

        let mut sum_pur = 0.0;
        let mut min_pur = f64::MAX;

        let mut sum_osc = 0.0;
        let mut min_osc = f64::MAX;

        let mut sum_eff = 0.0;
        let mut min_eff = f64::MAX;

        let mut sum_zen = 0.0;
        let mut max_zen = f64::MIN;

        let mut compliant_count = 0;

        for r in &results {
            sum_f += r.bloch_freq_ghz;
            if r.bloch_freq_ghz < min_f {
                min_f = r.bloch_freq_ghz;
            }

            sum_cut += r.cutoff_order;
            if r.cutoff_order < min_cut {
                min_cut = r.cutoff_order;
            }

            sum_pur += r.spectral_purity_db;
            if r.spectral_purity_db < min_pur {
                min_pur = r.spectral_purity_db;
            }

            sum_osc += r.oscillations_count;
            if r.oscillations_count < min_osc {
                min_osc = r.oscillations_count;
            }

            sum_eff += r.efficiency_pct;
            if r.efficiency_pct < min_eff {
                min_eff = r.efficiency_pct;
            }

            sum_zen += r.zener_leakage;
            if r.zener_leakage > max_zen {
                max_zen = r.zener_leakage;
            }

            // Physical criteria validation:
            let is_compliant = r.bloch_freq_ghz >= 50.0
                && r.cutoff_order >= 25
                && r.spectral_purity_db >= 45.0
                && r.oscillations_count >= 3.0
                && r.efficiency_pct >= 15.0
                && r.zener_leakage <= 0.05;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        HighHarmonicBlochBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_bloch_freq_ghz: sum_f / n,
            min_bloch_freq_ghz: min_f,
            mean_cutoff_order: sum_cut as f64 / n,
            min_cutoff_order: min_cut,
            mean_spectral_purity_db: sum_pur / n,
            min_spectral_purity_db: min_pur,
            mean_oscillations_count: sum_osc / n,
            min_oscillations_count: min_osc,
            mean_efficiency_pct: sum_eff / n,
            min_efficiency_pct: min_eff,
            mean_zener_leakage: sum_zen / n,
            max_zener_leakage: max_zen,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
