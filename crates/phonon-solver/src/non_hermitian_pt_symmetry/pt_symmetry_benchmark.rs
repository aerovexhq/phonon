//! Multi-threaded Rayon benchmark runner for non-Hermitian acoustic PT-symmetry
//! and exceptional point sensors across 10,000 parameter sweeps.

use super::pt_symmetry_solver::PtSymmetrySolver;
use phonon_models::non_hermitian_pt_symmetry::PtSymmetryParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for acoustic PT-symmetry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PtSymmetrySweepPoint {
    pub enhancement_db: f64,
    pub threshold_power_mw: f64,
    pub reverse_isolation_db: f64,
    pub directional_absorption_pct: f64,
    pub phase_rigidity: f64,
    pub coalescence_fidelity_pct: f64,
}

/// Comprehensive benchmark report for acoustic PT-symmetry.
#[derive(Debug, Clone, PartialEq)]
pub struct PtSymmetryBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean sensitivity enhancement in decibels ($\ge 35.0\text{ dB}$ required).
    pub mean_enhancement_db: f64,
    /// Minimum sensitivity enhancement in decibels.
    pub min_enhancement_db: f64,
    /// Maximum sensitivity enhancement in decibels.
    pub max_enhancement_db: f64,
    /// Mean threshold power in milliwatts ($\le 2.0\text{ mW}$ required).
    pub mean_threshold_power_mw: f64,
    /// Maximum threshold power in milliwatts.
    pub max_threshold_power_mw: f64,
    /// Mean reverse isolation in decibels ($\ge 25.0\text{ dB}$ required).
    pub mean_reverse_isolation_db: f64,
    /// Minimum reverse isolation in decibels.
    pub min_reverse_isolation_db: f64,
    /// Mean directional absorption in percent ($\ge 90.0\%$ required).
    pub mean_directional_absorption_pct: f64,
    /// Minimum directional absorption in percent.
    pub min_directional_absorption_pct: f64,
    /// Mean Petermann phase rigidity ($\le 0.20$ required).
    pub mean_phase_rigidity: f64,
    /// Maximum Petermann phase rigidity.
    pub max_phase_rigidity: f64,
    /// Mean coalescence fidelity in percent ($\ge 95.0\%$ required).
    pub mean_coalescence_fidelity_pct: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for acoustic PT-symmetry.
#[derive(Debug, Clone)]
pub struct PtSymmetryBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for PtSymmetryBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl PtSymmetryBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> PtSymmetryBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<PtSymmetrySweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let f0 = 1.5 + 6.0 * frac; // 1.5 to 7.5 GHz
                let coupling = 15.0 + 35.0 * pseudo_hash; // 15 to 50 MHz
                let gain_loss = coupling * (0.94 + 0.10 * frac); // closely balanced near EP
                let detuning = 0.5 + 20.0 * pseudo_hash; // 0.5 to 20.5 kHz
                let q_factor = 2.0e5 + 1.2e6 * frac; // 0.2M to 1.4M
                let power = 0.3 + 1.2 * pseudo_hash; // 0.3 to 1.5 mW
                let phase = 0.8 + 1.5 * frac; // 0.8 to 2.3 rad

                let params = PtSymmetryParams {
                    acoustic_resonance_freq_ghz: f0,
                    intercavity_coupling_mhz: coupling,
                    gain_loss_rate_mhz: gain_loss,
                    perturbation_detuning_khz: detuning,
                    intrinsic_q_factor: q_factor,
                    active_gain_power_mw: power,
                    non_reciprocal_phase_rad: phase,
                };

                let solver = PtSymmetrySolver::new(params);
                let m = solver.solve();

                PtSymmetrySweepPoint {
                    enhancement_db: m.sensitivity_enhancement_db,
                    threshold_power_mw: m.threshold_power_mw,
                    reverse_isolation_db: m.reverse_isolation_db,
                    directional_absorption_pct: m.directional_absorption_pct,
                    phase_rigidity: m.pt_phase_rigidity,
                    coalescence_fidelity_pct: m.coalescence_fidelity_pct,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_enh = 0.0;
        let mut min_enh = f64::MAX;
        let mut max_enh = f64::MIN;

        let mut sum_pth = 0.0;
        let mut max_pth = f64::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;

        let mut sum_abs = 0.0;
        let mut min_abs = f64::MAX;

        let mut sum_rig = 0.0;
        let mut max_rig = f64::MIN;

        let mut sum_fid = 0.0;
        let mut compliant_count = 0;

        for r in &results {
            sum_enh += r.enhancement_db;
            if r.enhancement_db < min_enh {
                min_enh = r.enhancement_db;
            }
            if r.enhancement_db > max_enh {
                max_enh = r.enhancement_db;
            }

            sum_pth += r.threshold_power_mw;
            if r.threshold_power_mw > max_pth {
                max_pth = r.threshold_power_mw;
            }

            sum_iso += r.reverse_isolation_db;
            if r.reverse_isolation_db < min_iso {
                min_iso = r.reverse_isolation_db;
            }

            sum_abs += r.directional_absorption_pct;
            if r.directional_absorption_pct < min_abs {
                min_abs = r.directional_absorption_pct;
            }

            sum_rig += r.phase_rigidity;
            if r.phase_rigidity > max_rig {
                max_rig = r.phase_rigidity;
            }

            sum_fid += r.coalescence_fidelity_pct;

            // Physical criteria validation:
            let is_compliant = r.enhancement_db >= 35.0
                && r.threshold_power_mw <= 2.0
                && r.reverse_isolation_db >= 25.0
                && r.directional_absorption_pct >= 90.0
                && r.phase_rigidity <= 0.20
                && r.coalescence_fidelity_pct >= 95.0;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        PtSymmetryBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_enhancement_db: sum_enh / n,
            min_enhancement_db: min_enh,
            max_enhancement_db: max_enh,
            mean_threshold_power_mw: sum_pth / n,
            max_threshold_power_mw: max_pth,
            mean_reverse_isolation_db: sum_iso / n,
            min_reverse_isolation_db: min_iso,
            mean_directional_absorption_pct: sum_abs / n,
            min_directional_absorption_pct: min_abs,
            mean_phase_rigidity: sum_rig / n,
            max_phase_rigidity: max_rig,
            mean_coalescence_fidelity_pct: sum_fid / n,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
