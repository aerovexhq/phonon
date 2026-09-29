//! Multi-threaded Rayon benchmark runner for topological acoustic axion polaritons
//! and quantum dark matter resonant transducers across 10,000 parameter sweeps.

use super::acoustic_axion_solver::AcousticAxionSolver;
use phonon_models::topological_acoustic_axion::AcousticAxionParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for topological acoustic axion polaritons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticAxionSweepPoint {
    pub isolation_db: f64,
    pub cooperativity: f64,
    pub snr_db: f64,
    pub purity_pct: f64,
    pub insertion_loss_db: f64,
    pub gap_ghz: f64,
}

/// Comprehensive benchmark report for topological acoustic axion polaritons.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticAxionBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean magnetoelectric isolation in dB ($\\ge 30.0\\text{ dB}$ required).
    pub mean_isolation_db: f64,
    /// Minimum magnetoelectric isolation in dB.
    pub min_isolation_db: f64,
    /// Mean axion coupling cooperativity ($\\ge 50.0$ required).
    pub mean_cooperativity: f64,
    /// Minimum axion coupling cooperativity.
    pub min_cooperativity: f64,
    /// Mean dark matter readout SNR in dB ($\\ge 25.0\\text{ dB}$ required).
    pub mean_snr_db: f64,
    /// Minimum dark matter readout SNR in dB.
    pub min_snr_db: f64,
    /// Mean chiral anomaly mode purity in percent ($\\ge 95.0\\%$ required).
    pub mean_purity_pct: f64,
    /// Minimum chiral anomaly mode purity in percent.
    pub min_purity_pct: f64,
    /// Mean insertion loss in dB ($\\le 1.0\\text{ dB}$ required).
    pub mean_insertion_loss_db: f64,
    /// Maximum insertion loss in dB.
    pub max_insertion_loss_db: f64,
    /// Mean anti-crossing gap in GHz ($\\ge 1.5\\text{ GHz}$ required).
    pub mean_gap_ghz: f64,
    /// Minimum anti-crossing gap in GHz.
    pub min_gap_ghz: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for topological acoustic axion polaritons.
#[derive(Debug, Clone)]
pub struct AcousticAxionBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for AcousticAxionBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl AcousticAxionBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> AcousticAxionBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<AcousticAxionSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let freq = 2.0 + 6.0 * frac; // 2.0 to 8.0 GHz
                let bias = 4.0 + 8.0 * pseudo_hash; // 4.0 to 12.0 T
                let coupling = 2.5e-3 + 4.5e-3 * frac; // 2.5e-3 to 7.0e-3
                let q_m = 60_000.0 + 180_000.0 * pseudo_hash; // 60k to 240k
                let q_e = 30_000.0 + 60_000.0 * frac; // 30k to 90k
                let width = 16.0 + 24.0 * pseudo_hash; // 16 to 40 nm

                let params = AcousticAxionParams {
                    acoustic_frequency_ghz: freq,
                    static_magnetic_bias_t: bias,
                    axion_strain_coupling: coupling,
                    cavity_acoustic_q: q_m,
                    cavity_em_q: q_e,
                    domain_wall_width_nm: width,
                };

                let solver = AcousticAxionSolver::new(params);
                let m = solver.solve();

                AcousticAxionSweepPoint {
                    isolation_db: m.magnetoelectric_isolation_db,
                    cooperativity: m.axion_cooperativity,
                    snr_db: m.dark_matter_snr_db,
                    purity_pct: m.chiral_anomaly_purity_pct,
                    insertion_loss_db: m.insertion_loss_db,
                    gap_ghz: m.anticrossing_gap_ghz,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;

        let mut sum_coop = 0.0;
        let mut min_coop = f64::MAX;

        let mut sum_snr = 0.0;
        let mut min_snr = f64::MAX;

        let mut sum_pur = 0.0;
        let mut min_pur = f64::MAX;

        let mut sum_loss = 0.0;
        let mut max_loss = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;

        let mut compliant_count = 0;

        for r in &results {
            sum_iso += r.isolation_db;
            if r.isolation_db < min_iso {
                min_iso = r.isolation_db;
            }

            sum_coop += r.cooperativity;
            if r.cooperativity < min_coop {
                min_coop = r.cooperativity;
            }

            sum_snr += r.snr_db;
            if r.snr_db < min_snr {
                min_snr = r.snr_db;
            }

            sum_pur += r.purity_pct;
            if r.purity_pct < min_pur {
                min_pur = r.purity_pct;
            }

            sum_loss += r.insertion_loss_db;
            if r.insertion_loss_db > max_loss {
                max_loss = r.insertion_loss_db;
            }

            sum_gap += r.gap_ghz;
            if r.gap_ghz < min_gap {
                min_gap = r.gap_ghz;
            }

            // Physical criteria validation:
            let is_compliant = r.isolation_db >= 30.0
                && r.cooperativity >= 50.0
                && r.snr_db >= 25.0
                && r.purity_pct >= 95.0
                && r.insertion_loss_db <= 1.0
                && r.gap_ghz >= 1.5;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        AcousticAxionBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_isolation_db: sum_iso / n,
            min_isolation_db: min_iso,
            mean_cooperativity: sum_coop / n,
            min_cooperativity: min_coop,
            mean_snr_db: sum_snr / n,
            min_snr_db: min_snr,
            mean_purity_pct: sum_pur / n,
            min_purity_pct: min_pur,
            mean_insertion_loss_db: sum_loss / n,
            max_insertion_loss_db: max_loss,
            mean_gap_ghz: sum_gap / n,
            min_gap_ghz: min_gap,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
