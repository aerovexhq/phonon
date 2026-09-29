//! Multi-threaded Rayon benchmark runner for chiral phonon-magnon skyrmion braiding
//! and non-volatile acoustic memory across 10,000 parameter sweeps.

use super::skyrmion_memory_solver::SkyrmionMemorySolver;
use phonon_models::skyrmion_braiding_memory::SkyrmionMemoryParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for skyrmion braiding and memory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionMemorySweepPoint {
    pub drift_velocity: f64,
    pub bit_error_rate: f64,
    pub retention_years: f64,
    pub write_energy_fj: f64,
    pub braiding_fidelity_pct: f64,
    pub hall_suppression_pct: f64,
}

/// Comprehensive benchmark report for chiral phonon-magnon skyrmion memory.
#[derive(Debug, Clone, PartialEq)]
pub struct SkyrmionMemoryBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean skyrmion drift velocity in m/s ($\\ge 250.0\\text{ m/s}$ required).
    pub mean_drift_velocity: f64,
    /// Minimum skyrmion drift velocity in m/s.
    pub min_drift_velocity: f64,
    /// Mean bit error rate ($\\le 1.0\\times 10^{-12}$ required).
    pub mean_bit_error_rate: f64,
    /// Maximum bit error rate.
    pub max_bit_error_rate: f64,
    /// Mean non-volatile retention time in years ($\\ge 15.0\\text{ years}$ required).
    pub mean_retention_years: f64,
    /// Minimum non-volatile retention time in years.
    pub min_retention_years: f64,
    /// Mean write energy in femtojoules ($\\le 0.5\\text{ fJ}$ required).
    pub mean_write_energy_fj: f64,
    /// Maximum write energy in femtojoules.
    pub max_write_energy_fj: f64,
    /// Mean braiding gate fidelity in percent ($\\ge 99.5\\%$ required).
    pub mean_braiding_fidelity_pct: f64,
    /// Minimum braiding gate fidelity in percent.
    pub min_braiding_fidelity_pct: f64,
    /// Mean skyrmion Hall suppression in percent ($\\ge 90.0\\%$ required).
    pub mean_hall_suppression_pct: f64,
    /// Minimum skyrmion Hall suppression in percent.
    pub min_hall_suppression_pct: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for skyrmion braiding and memory.
#[derive(Debug, Clone)]
pub struct SkyrmionMemoryBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for SkyrmionMemoryBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl SkyrmionMemoryBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> SkyrmionMemoryBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<SkyrmionMemorySweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let lambda = 80.0 + 200.0 * frac; // 80 to 280 nm
                let strain = 1.0e-3 + 1.8e-3 * pseudo_hash; // 1.0e-3 to 2.8e-3
                let coupling = 4.5e6 + 8.5e6 * frac; // 4.5e6 to 13.0e6 J/m3
                let damping = 0.008 + 0.025 * pseudo_hash; // 0.008 to 0.033
                let track = 40.0 + 60.0 * frac; // 40 to 100 nm
                let stability = 54.0 + 26.0 * pseudo_hash; // 54 to 80

                let params = SkyrmionMemoryParams {
                    acoustic_wavelength_nm: lambda,
                    acoustic_strain_amplitude: strain,
                    magnetoelastic_coupling_j_m3: coupling,
                    gilbert_damping: damping,
                    nanowire_track_width_nm: track,
                    thermal_stability_factor: stability,
                };

                let solver = SkyrmionMemorySolver::new(params);
                let m = solver.solve();

                SkyrmionMemorySweepPoint {
                    drift_velocity: m.drift_velocity_m_s,
                    bit_error_rate: m.bit_error_rate,
                    retention_years: m.retention_years,
                    write_energy_fj: m.write_energy_fj,
                    braiding_fidelity_pct: m.braiding_fidelity_pct,
                    hall_suppression_pct: m.hall_angle_suppression_pct,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_v = 0.0;
        let mut min_v = f64::MAX;

        let mut sum_ber = 0.0;
        let mut max_ber = f64::MIN;

        let mut sum_ret = 0.0;
        let mut min_ret = f64::MAX;

        let mut sum_e = 0.0;
        let mut max_e = f64::MIN;

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;

        let mut sum_supp = 0.0;
        let mut min_supp = f64::MAX;

        let mut compliant_count = 0;

        for r in &results {
            sum_v += r.drift_velocity;
            if r.drift_velocity < min_v {
                min_v = r.drift_velocity;
            }

            sum_ber += r.bit_error_rate;
            if r.bit_error_rate > max_ber {
                max_ber = r.bit_error_rate;
            }

            sum_ret += r.retention_years;
            if r.retention_years < min_ret {
                min_ret = r.retention_years;
            }

            sum_e += r.write_energy_fj;
            if r.write_energy_fj > max_e {
                max_e = r.write_energy_fj;
            }

            sum_fid += r.braiding_fidelity_pct;
            if r.braiding_fidelity_pct < min_fid {
                min_fid = r.braiding_fidelity_pct;
            }

            sum_supp += r.hall_suppression_pct;
            if r.hall_suppression_pct < min_supp {
                min_supp = r.hall_suppression_pct;
            }

            // Physical criteria validation:
            let is_compliant = r.drift_velocity >= 250.0
                && r.bit_error_rate <= 1.0e-12
                && r.retention_years >= 15.0
                && r.write_energy_fj <= 0.5
                && r.braiding_fidelity_pct >= 99.5
                && r.hall_suppression_pct >= 90.0;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        SkyrmionMemoryBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_drift_velocity: sum_v / n,
            min_drift_velocity: min_v,
            mean_bit_error_rate: sum_ber / n,
            max_bit_error_rate: max_ber,
            mean_retention_years: sum_ret / n,
            min_retention_years: min_ret,
            mean_write_energy_fj: sum_e / n,
            max_write_energy_fj: max_e,
            mean_braiding_fidelity_pct: sum_fid / n,
            min_braiding_fidelity_pct: min_fid,
            mean_hall_suppression_pct: sum_supp / n,
            min_hall_suppression_pct: min_supp,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
