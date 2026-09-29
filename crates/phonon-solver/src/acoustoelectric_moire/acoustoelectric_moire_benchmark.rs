//! Multi-threaded Rayon benchmark runner for quantum acoustoelectric moiré superlattices
//! across 10,000 parameter sweeps.

use super::acoustoelectric_moire_solver::AcoustoelectricMoireSolver;
use phonon_models::acoustoelectric_moire::AcoustoelectricMoireParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for acoustoelectric moiré superlattices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustoelectricMoireSweepPoint {
    pub quenching_ratio: f64,
    pub pairing_ratio: f64,
    pub correlation_ratio: f64,
    pub minigap_mev: f64,
    pub simulation_fidelity_pct: f64,
    pub melting_temp_k: f64,
}

/// Comprehensive benchmark report for acoustoelectric moiré superlattices.
#[derive(Debug, Clone, PartialEq)]
pub struct AcoustoelectricMoireBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean bandwidth quenching ratio ($W_0 / W_{\mathrm{ph}} \ge 10.0\times$ required).
    pub mean_quenching_ratio: f64,
    /// Minimum bandwidth quenching ratio.
    pub min_quenching_ratio: f64,
    /// Maximum bandwidth quenching ratio.
    pub max_quenching_ratio: f64,
    /// Mean electron-phonon pairing ratio ($\ge 3.0\times$ required).
    pub mean_pairing_ratio: f64,
    /// Minimum electron-phonon pairing ratio.
    pub min_pairing_ratio: f64,
    /// Mean correlation ratio $U / W_{\mathrm{ph}}$ ($\ge 3.0$ required).
    pub mean_correlation_ratio: f64,
    /// Minimum correlation ratio.
    pub min_correlation_ratio: f64,
    /// Mean moiré minigap in $\text{meV}$ ($\ge 5.0\text{ meV}$ required).
    pub mean_minigap_mev: f64,
    /// Minimum moiré minigap in $\text{meV}$.
    pub min_minigap_mev: f64,
    /// Mean quantum simulation gate fidelity in percent ($\ge 98.0\%$ required).
    pub mean_simulation_fidelity_pct: f64,
    /// Minimum quantum simulation gate fidelity in percent.
    pub min_simulation_fidelity_pct: f64,
    /// Mean Wigner crystal melting temperature in Kelvin ($\ge 20.0\text{ K}$ required).
    pub mean_melting_temp_k: f64,
    /// Minimum Wigner crystal melting temperature in Kelvin.
    pub min_melting_temp_k: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for acoustoelectric moiré superlattices.
#[derive(Debug, Clone)]
pub struct AcoustoelectricMoireBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for AcoustoelectricMoireBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl AcoustoelectricMoireBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> AcoustoelectricMoireBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<AcoustoelectricMoireSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let theta = 0.95 + 0.35 * frac; // 0.95 to 1.30 deg
                let v0 = 20.0 + 45.0 * pseudo_hash; // 20 to 65 meV
                let f0 = 1.5 + 4.0 * frac; // 1.5 to 5.5 GHz
                let strain = 1.0 + 3.0 * pseudo_hash; // 1.0 to 4.0 (in 1e-4)
                let xi = 3.5 + 4.5 * frac; // 3.5 to 8.0 eV
                let w0 = 20.0 + 20.0 * pseudo_hash; // 20 to 40 meV
                let u = 35.0 + 40.0 * frac; // 35 to 75 meV
                let eps = 4.5 + 6.0 * pseudo_hash; // 4.5 to 10.5

                let params = AcoustoelectricMoireParams {
                    twist_angle_deg: theta,
                    moire_potential_amplitude_mev: v0,
                    saw_frequency_ghz: f0,
                    dynamic_strain_amplitude_1e4: strain,
                    deformation_potential_ev: xi,
                    bare_phonon_bandwidth_mev: w0,
                    coulomb_correlation_u_mev: u,
                    dielectric_constant: eps,
                };

                let solver = AcoustoelectricMoireSolver::new(params);
                let m = solver.solve();

                AcoustoelectricMoireSweepPoint {
                    quenching_ratio: m.bandwidth_quenching_ratio,
                    pairing_ratio: m.electron_phonon_pairing_ratio,
                    correlation_ratio: m.correlation_ratio_u_over_w,
                    minigap_mev: m.moire_minigap_mev,
                    simulation_fidelity_pct: m.quantum_simulation_fidelity_pct,
                    melting_temp_k: m.wigner_crystal_melting_temp_k,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_q = 0.0;
        let mut min_q = f64::MAX;
        let mut max_q = f64::MIN;

        let mut sum_pair = 0.0;
        let mut min_pair = f64::MAX;

        let mut sum_corr = 0.0;
        let mut min_corr = f64::MAX;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;

        let mut sum_melt = 0.0;
        let mut min_melt = f64::MAX;

        let mut compliant_count = 0;

        for r in &results {
            sum_q += r.quenching_ratio;
            if r.quenching_ratio < min_q {
                min_q = r.quenching_ratio;
            }
            if r.quenching_ratio > max_q {
                max_q = r.quenching_ratio;
            }

            sum_pair += r.pairing_ratio;
            if r.pairing_ratio < min_pair {
                min_pair = r.pairing_ratio;
            }

            sum_corr += r.correlation_ratio;
            if r.correlation_ratio < min_corr {
                min_corr = r.correlation_ratio;
            }

            sum_gap += r.minigap_mev;
            if r.minigap_mev < min_gap {
                min_gap = r.minigap_mev;
            }

            sum_fid += r.simulation_fidelity_pct;
            if r.simulation_fidelity_pct < min_fid {
                min_fid = r.simulation_fidelity_pct;
            }

            sum_melt += r.melting_temp_k;
            if r.melting_temp_k < min_melt {
                min_melt = r.melting_temp_k;
            }

            // Physical criteria validation:
            let is_compliant = r.quenching_ratio >= 10.0
                && r.pairing_ratio >= 3.0
                && r.correlation_ratio >= 3.0
                && r.minigap_mev >= 5.0
                && r.simulation_fidelity_pct >= 98.0
                && r.melting_temp_k >= 20.0;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        AcoustoelectricMoireBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_quenching_ratio: sum_q / n,
            min_quenching_ratio: min_q,
            max_quenching_ratio: max_q,
            mean_pairing_ratio: sum_pair / n,
            min_pairing_ratio: min_pair,
            mean_correlation_ratio: sum_corr / n,
            min_correlation_ratio: min_corr,
            mean_minigap_mev: sum_gap / n,
            min_minigap_mev: min_gap,
            mean_simulation_fidelity_pct: sum_fid / n,
            min_simulation_fidelity_pct: min_fid,
            mean_melting_temp_k: sum_melt / n,
            min_melting_temp_k: min_melt,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
