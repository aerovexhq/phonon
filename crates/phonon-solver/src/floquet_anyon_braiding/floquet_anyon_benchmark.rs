//! Multi-threaded Rayon benchmark runner for Floquet anyon braiding across 10,000 parameter sweeps.

use super::floquet_anyon_solver::FloquetAnyonSolver;
use phonon_models::floquet_anyon_braiding::FloquetAnyonParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for Floquet anyon braiding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetAnyonSweepPoint {
    pub fidelity_pct: f64,
    pub leakage: f64,
    pub commutator_norm: f64,
    pub gap_khz: f64,
    pub readout_snr_db: f64,
    pub qubit_dim: usize,
}

/// Comprehensive benchmark report for Floquet anyon braiding systems.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetAnyonBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean topological braiding gate fidelity in percent ($\ge 99.5\%$ required).
    pub mean_braiding_gate_fidelity_pct: f64,
    /// Minimum topological braiding gate fidelity in percent.
    pub min_braiding_gate_fidelity_pct: f64,
    /// Maximum topological braiding gate fidelity in percent.
    pub max_braiding_gate_fidelity_pct: f64,
    /// Mean Landau-Zener-Floquet leakage error rate ($P_{\mathrm{leak}} \le 1.0\times 10^{-4}$ required).
    pub mean_leakage_error_rate: f64,
    /// Maximum Landau-Zener-Floquet leakage error rate.
    pub max_leakage_error_rate: f64,
    /// Mean synthetic non-Abelian commutator norm ($\ge 0.50$ required).
    pub mean_synthetic_gauge_commutator_norm: f64,
    /// Minimum synthetic non-Abelian commutator norm.
    pub min_synthetic_gauge_commutator_norm: f64,
    /// Mean Floquet topological bandgap in $\text{kHz}$ ($\ge 50.0\text{ kHz}$ required).
    pub mean_floquet_gap_khz: f64,
    /// Mean logical topological state readout SNR in decibels ($\ge 25.0\text{ dB}$ required).
    pub mean_logical_readout_snr_db: f64,
    /// Minimum logical topological state readout SNR in decibels.
    pub min_logical_readout_snr_db: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for Floquet anyon braiding.
#[derive(Debug, Clone)]
pub struct FloquetAnyonBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for FloquetAnyonBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl FloquetAnyonBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> FloquetAnyonBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<FloquetAnyonSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let dim = 16 + (idx % 32); // 16 to 47
                let drive_mhz = 20.0 + 35.0 * frac; // 20 to 55 MHz
                let mod_depth = 0.35 + 0.35 * pseudo_hash; // 0.35 to 0.70
                let gap = 120.0 + 300.0 * frac; // 120 to 420 kHz
                let duration = 2.0 + 8.0 * pseudo_hash; // 2.0 to 10.0 us
                let sep = 3.0 + 5.0 * frac; // 3.0 to 8.0 um
                let temp = 12.0 + 48.0 * pseudo_hash; // 12 to 60 mK

                let params = FloquetAnyonParams {
                    lattice_dimension: dim,
                    floquet_drive_freq_mhz: drive_mhz,
                    modulation_depth: mod_depth,
                    floquet_gap_khz: gap,
                    braid_duration_us: duration,
                    anyon_core_separation_um: sep,
                    ambient_temperature_mk: temp,
                };

                let solver = FloquetAnyonSolver::new(params);
                let m = solver.solve();

                FloquetAnyonSweepPoint {
                    fidelity_pct: m.braiding_gate_fidelity_pct,
                    leakage: m.leakage_error_rate,
                    commutator_norm: m.synthetic_gauge_commutator_norm,
                    gap_khz: m.floquet_gap_khz,
                    readout_snr_db: m.logical_readout_snr_db,
                    qubit_dim: m.anyon_qubit_dimension,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1.0e-6);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_leak = 0.0;
        let mut max_leak = f64::MIN;

        let mut sum_comm = 0.0;
        let mut min_comm = f64::MAX;

        let mut sum_gap = 0.0;

        let mut sum_snr = 0.0;
        let mut min_snr = f64::MAX;

        let mut compliant_count = 0usize;

        for pt in &results {
            sum_fid += pt.fidelity_pct;
            if pt.fidelity_pct < min_fid {
                min_fid = pt.fidelity_pct;
            }
            if pt.fidelity_pct > max_fid {
                max_fid = pt.fidelity_pct;
            }

            sum_leak += pt.leakage;
            if pt.leakage > max_leak {
                max_leak = pt.leakage;
            }

            sum_comm += pt.commutator_norm;
            if pt.commutator_norm < min_comm {
                min_comm = pt.commutator_norm;
            }

            sum_gap += pt.gap_khz;

            sum_snr += pt.readout_snr_db;
            if pt.readout_snr_db < min_snr {
                min_snr = pt.readout_snr_db;
            }

            // Strict compliance conditions:
            // 1. Braiding gate fidelity >= 99.5%
            // 2. Leakage error rate <= 1.0e-4
            // 3. Commutator norm >= 0.50
            // 4. Floquet gap >= 50.0 kHz
            // 5. Logical readout SNR >= 25.0 dB
            // 6. Qubit dimension == 2
            if pt.fidelity_pct >= 99.50
                && pt.leakage <= 1.0e-4
                && pt.commutator_norm >= 0.50
                && pt.gap_khz >= 50.0
                && pt.readout_snr_db >= 25.0
                && pt.qubit_dim == 2
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        FloquetAnyonBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_braiding_gate_fidelity_pct: sum_fid / n,
            min_braiding_gate_fidelity_pct: min_fid,
            max_braiding_gate_fidelity_pct: max_fid,
            mean_leakage_error_rate: sum_leak / n,
            max_leakage_error_rate: max_leak,
            mean_synthetic_gauge_commutator_norm: sum_comm / n,
            min_synthetic_gauge_commutator_norm: min_comm,
            mean_floquet_gap_khz: sum_gap / n,
            mean_logical_readout_snr_db: sum_snr / n,
            min_logical_readout_snr_db: min_snr,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
