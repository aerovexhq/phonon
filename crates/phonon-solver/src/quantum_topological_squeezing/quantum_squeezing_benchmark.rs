//! Multi-threaded Rayon benchmark runner for quantum topological phonon squeezing
//! and sub-SQL acoustic metrology across 10,000 parameter sweeps.

use super::quantum_squeezing_solver::QuantumPhononSqueezingSolver;
use phonon_models::quantum_topological_squeezing::QuantumPhononSqueezingParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for quantum topological phonon squeezing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononSqueezingSweepPoint {
    pub squeezing_db: f64,
    pub fidelity_pct: f64,
    pub wigner_negativity: f64,
    pub force_sensitivity: f64,
    pub gravimetry_nano_g: f64,
    pub entanglement_ebits: f64,
}

/// Comprehensive benchmark report for quantum topological phonon squeezing.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumPhononSqueezingBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean quadrature squeezing below shot noise in decibels ($\ge 6.0\text{ dB}$ required).
    pub mean_squeezing_db: f64,
    /// Minimum quadrature squeezing below shot noise in decibels.
    pub min_squeezing_db: f64,
    /// Maximum quadrature squeezing below shot noise in decibels.
    pub max_squeezing_db: f64,
    /// Mean Schrödinger cat state fidelity in percent ($\ge 90.0\%$ required).
    pub mean_fidelity_pct: f64,
    /// Minimum Schrödinger cat state fidelity in percent.
    pub min_fidelity_pct: f64,
    /// Mean Wigner distribution negativity at origin ($\ge 0.15$ required).
    pub mean_wigner_negativity: f64,
    /// Minimum Wigner distribution negativity at origin.
    pub min_wigner_negativity: f64,
    /// Mean detectable force spectral density in attonewtons per $\sqrt{\text{Hz}}$ ($\le 25.0\text{ aN}/\sqrt{\text{Hz}}$ required).
    pub mean_force_sensitivity: f64,
    /// Maximum detectable force spectral density in attonewtons per $\sqrt{\text{Hz}}$.
    pub max_force_sensitivity: f64,
    /// Mean quantum gravimetry precision in nano-g ($\le 5.0\text{ nano-g}$ required).
    pub mean_gravimetry_nano_g: f64,
    /// Maximum quantum gravimetry precision in nano-g.
    pub max_gravimetry_nano_g: f64,
    /// Mean continuous-variable entanglement in ebits ($\ge 1.0\text{ ebits}$ required).
    pub mean_entanglement_ebits: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for quantum topological phonon squeezing.
#[derive(Debug, Clone)]
pub struct QuantumPhononSqueezingBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for QuantumPhononSqueezingBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl QuantumPhononSqueezingBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> QuantumPhononSqueezingBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<PhononSqueezingSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let f0 = 1.5 + 6.0 * frac; // 1.5 to 7.5 GHz
                let q_factor = 8.0e5 + 5.0e6 * pseudo_hash; // 0.8M to 5.8M
                let g4 = 10.0 + 45.0 * frac; // 10 to 55 kHz
                let power = 0.5 + 5.5 * pseudo_hash; // 0.5 to 6.0 mW
                let eta_esc = 0.80 + 0.16 * frac; // 0.80 to 0.96
                let detuning = 0.1 + 1.2 * pseudo_hash; // 0.1 to 1.3
                let alpha = 1.5 + 1.5 * frac; // 1.5 to 3.0
                let n_th = 0.02 + 0.10 * pseudo_hash; // 0.02 to 0.12

                let params = QuantumPhononSqueezingParams {
                    acoustic_resonance_freq_ghz: f0,
                    loaded_q_factor: q_factor,
                    parametric_coupling_rate_khz: g4,
                    pump_power_mw: power,
                    cavity_escape_efficiency: eta_esc,
                    normalized_detuning: detuning,
                    cat_coherent_amplitude: alpha,
                    bath_thermal_occupancy: n_th,
                };

                let solver = QuantumPhononSqueezingSolver::new(params);
                let m = solver.solve();

                PhononSqueezingSweepPoint {
                    squeezing_db: m.quadrature_squeezing_db,
                    fidelity_pct: m.cat_state_fidelity_pct,
                    wigner_negativity: m.wigner_negativity,
                    force_sensitivity: m.force_sensitivity_attonewtons,
                    gravimetry_nano_g: m.quantum_gravimetry_precision_nano_g,
                    entanglement_ebits: m.continuous_entanglement_ebits,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_sqz = 0.0;
        let mut min_sqz = f64::MAX;
        let mut max_sqz = f64::MIN;

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;

        let mut sum_wneg = 0.0;
        let mut min_wneg = f64::MAX;

        let mut sum_force = 0.0;
        let mut max_force = f64::MIN;

        let mut sum_grav = 0.0;
        let mut max_grav = f64::MIN;

        let mut sum_ebits = 0.0;
        let mut compliant_count = 0;

        for r in &results {
            sum_sqz += r.squeezing_db;
            if r.squeezing_db < min_sqz {
                min_sqz = r.squeezing_db;
            }
            if r.squeezing_db > max_sqz {
                max_sqz = r.squeezing_db;
            }

            sum_fid += r.fidelity_pct;
            if r.fidelity_pct < min_fid {
                min_fid = r.fidelity_pct;
            }

            sum_wneg += r.wigner_negativity;
            if r.wigner_negativity < min_wneg {
                min_wneg = r.wigner_negativity;
            }

            sum_force += r.force_sensitivity;
            if r.force_sensitivity > max_force {
                max_force = r.force_sensitivity;
            }

            sum_grav += r.gravimetry_nano_g;
            if r.gravimetry_nano_g > max_grav {
                max_grav = r.gravimetry_nano_g;
            }

            sum_ebits += r.entanglement_ebits;

            // Physical criteria validation:
            let is_compliant = r.squeezing_db >= 6.0
                && r.fidelity_pct >= 90.0
                && r.wigner_negativity >= 0.15
                && r.force_sensitivity <= 25.0
                && r.gravimetry_nano_g <= 5.0
                && r.entanglement_ebits >= 1.0;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        QuantumPhononSqueezingBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_squeezing_db: sum_sqz / n,
            min_squeezing_db: min_sqz,
            max_squeezing_db: max_sqz,
            mean_fidelity_pct: sum_fid / n,
            min_fidelity_pct: min_fid,
            mean_wigner_negativity: sum_wneg / n,
            min_wigner_negativity: min_wneg,
            mean_force_sensitivity: sum_force / n,
            max_force_sensitivity: max_force,
            mean_gravimetry_nano_g: sum_grav / n,
            max_gravimetry_nano_g: max_grav,
            mean_entanglement_ebits: sum_ebits / n,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
