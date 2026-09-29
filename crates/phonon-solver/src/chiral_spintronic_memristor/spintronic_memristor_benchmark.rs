//! Multi-threaded Rayon benchmark runner for chiral spintronic memristors
//! and neuromorphic crossbars across 10,000 parameter sweeps.

use super::spintronic_memristor_solver::SpintronicMemristorSolver;
use phonon_models::chiral_spintronic_memristor::SpintronicMemristorParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for chiral spintronic memristors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpintronicMemristorSweepPoint {
    pub energy_fj: f64,
    pub retention_years: f64,
    pub on_off_ratio: f64,
    pub stdp_fidelity_pct: f64,
    pub efficiency_topsw: f64,
    pub linearity_error_pct: f64,
}

/// Comprehensive benchmark report for chiral spintronic memristors.
#[derive(Debug, Clone, PartialEq)]
pub struct SpintronicMemristorBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean programming energy in femtojoules ($\le 10.0\text{ fJ}$ required).
    pub mean_energy_fj: f64,
    /// Maximum programming energy in femtojoules.
    pub max_energy_fj: f64,
    /// Mean non-volatile retention time in years ($\ge 10.0\text{ years}$ required).
    pub mean_retention_years: f64,
    /// Minimum non-volatile retention time in years.
    pub min_retention_years: f64,
    /// Mean conductance on/off ratio ($\ge 10.0$ required).
    pub mean_on_off_ratio: f64,
    /// Minimum conductance on/off ratio.
    pub min_on_off_ratio: f64,
    /// Mean STDP learning fidelity in percent ($\ge 95.0\%$ required).
    pub mean_stdp_fidelity_pct: f64,
    /// Minimum STDP learning fidelity in percent.
    pub min_stdp_fidelity_pct: f64,
    /// Mean crossbar compute efficiency in TOPS/W ($\ge 150.0\text{ TOPS/W}$ required).
    pub mean_efficiency_topsw: f64,
    /// Minimum crossbar compute efficiency in TOPS/W.
    pub min_efficiency_topsw: f64,
    /// Mean weight programming non-linearity error in percent ($\le 2.5\%$ required).
    pub mean_linearity_error_pct: f64,
    /// Maximum weight programming non-linearity error in percent.
    pub max_linearity_error_pct: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for chiral spintronic memristors.
#[derive(Debug, Clone)]
pub struct SpintronicMemristorBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for SpintronicMemristorBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl SpintronicMemristorBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> SpintronicMemristorBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<SpintronicMemristorSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let f_saw = 1.5 + 4.5 * frac; // 1.5 to 6.0 GHz
                let length = 0.8 + 1.6 * pseudo_hash; // 0.8 to 2.4 um
                let width = 25.0 + 40.0 * frac; // 25 to 65 nm
                let astt = 0.45 + 0.45 * pseudo_hash; // 0.45 to 0.90
                let pulse_t = 0.8 + 2.2 * frac; // 0.8 to 3.0 ns
                let pulse_p = 0.4 + 2.0 * pseudo_hash; // 0.4 to 2.4 uW
                let stability = 48.0 + 24.0 * frac; // 48 to 72
                let tmr = 180.0 + 220.0 * pseudo_hash; // 180 to 400%
                let dim = match idx % 4 {
                    0 => 32,
                    1 => 64,
                    2 => 128,
                    _ => 256,
                };

                let params = SpintronicMemristorParams {
                    acoustic_frequency_ghz: f_saw,
                    nanowire_length_um: length,
                    nanowire_width_nm: width,
                    astt_efficiency: astt,
                    pulse_duration_ns: pulse_t,
                    pulse_power_uw: pulse_p,
                    thermal_stability_factor: stability,
                    tmr_ratio_pct: tmr,
                    crossbar_dimension: dim,
                };

                let solver = SpintronicMemristorSolver::new(params);
                let m = solver.solve();

                SpintronicMemristorSweepPoint {
                    energy_fj: m.programming_energy_fj,
                    retention_years: m.retention_time_years,
                    on_off_ratio: m.conductance_on_off_ratio,
                    stdp_fidelity_pct: m.stdp_learning_fidelity_pct,
                    efficiency_topsw: m.crossbar_energy_efficiency_topsw,
                    linearity_error_pct: m.weight_linearity_error_pct,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_e = 0.0;
        let mut max_e = f64::MIN;

        let mut sum_ret = 0.0;
        let mut min_ret = f64::MAX;

        let mut sum_on_off = 0.0;
        let mut min_on_off = f64::MAX;

        let mut sum_stdp = 0.0;
        let mut min_stdp = f64::MAX;

        let mut sum_eff = 0.0;
        let mut min_eff = f64::MAX;

        let mut sum_lin = 0.0;
        let mut max_lin = f64::MIN;

        let mut compliant_count = 0;

        for r in &results {
            sum_e += r.energy_fj;
            if r.energy_fj > max_e {
                max_e = r.energy_fj;
            }

            sum_ret += r.retention_years;
            if r.retention_years < min_ret {
                min_ret = r.retention_years;
            }

            sum_on_off += r.on_off_ratio;
            if r.on_off_ratio < min_on_off {
                min_on_off = r.on_off_ratio;
            }

            sum_stdp += r.stdp_fidelity_pct;
            if r.stdp_fidelity_pct < min_stdp {
                min_stdp = r.stdp_fidelity_pct;
            }

            sum_eff += r.efficiency_topsw;
            if r.efficiency_topsw < min_eff {
                min_eff = r.efficiency_topsw;
            }

            sum_lin += r.linearity_error_pct;
            if r.linearity_error_pct > max_lin {
                max_lin = r.linearity_error_pct;
            }

            // Physical criteria validation:
            let is_compliant = r.energy_fj <= 10.0
                && r.retention_years >= 10.0
                && r.on_off_ratio >= 10.0
                && r.stdp_fidelity_pct >= 95.0
                && r.efficiency_topsw >= 150.0
                && r.linearity_error_pct <= 2.5;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        SpintronicMemristorBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_energy_fj: sum_e / n,
            max_energy_fj: max_e,
            mean_retention_years: sum_ret / n,
            min_retention_years: min_ret,
            mean_on_off_ratio: sum_on_off / n,
            min_on_off_ratio: min_on_off,
            mean_stdp_fidelity_pct: sum_stdp / n,
            min_stdp_fidelity_pct: min_stdp,
            mean_efficiency_topsw: sum_eff / n,
            min_efficiency_topsw: min_eff,
            mean_linearity_error_pct: sum_lin / n,
            max_linearity_error_pct: max_lin,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
