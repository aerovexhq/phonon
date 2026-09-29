//! Multi-threaded Rayon benchmark runner for chiral phonon spin-mechanics
//! and OAM multiplexing across 10,000 parameter sweeps.

use super::chiral_phonon_spin_solver::ChiralPhononSpinSolver;
use phonon_models::chiral_phonon_spin_mechanics::ChiralPhononSpinParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for chiral phonon spin-mechanics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralPhononSpinSweepPoint {
    pub isolation_db: f64,
    pub crosstalk_db: f64,
    pub efficiency_pct: f64,
    pub insertion_loss_db: f64,
    pub purity_pct: f64,
    pub router_extinction_db: f64,
    pub capacity_gbps: f64,
}

/// Comprehensive benchmark report for chiral phonon spin-mechanics.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralPhononSpinBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean OAM mode isolation in decibels ($\ge 25.0\text{ dB}$ required).
    pub mean_isolation_db: f64,
    /// Minimum OAM mode isolation in decibels.
    pub min_isolation_db: f64,
    /// Maximum OAM mode isolation in decibels.
    pub max_isolation_db: f64,
    /// Mean inter-channel modal crosstalk in decibels ($\le -20.0\text{ dB}$ required).
    pub mean_crosstalk_db: f64,
    /// Maximum inter-channel modal crosstalk in decibels (worst-case).
    pub max_crosstalk_db: f64,
    /// Mean transduction efficiency in percent ($\ge 70.0\%$ required).
    pub mean_efficiency_pct: f64,
    /// Minimum transduction efficiency in percent.
    pub min_efficiency_pct: f64,
    /// Mean forward insertion loss in decibels ($\le 2.0\text{ dB}$ required).
    pub mean_insertion_loss_db: f64,
    /// Maximum forward insertion loss in decibels.
    pub max_insertion_loss_db: f64,
    /// Mean spin-to-orbital angular momentum conversion purity in percent ($\ge 90.0\%$ required).
    pub mean_purity_pct: f64,
    /// Minimum spin-to-orbital conversion purity in percent.
    pub min_purity_pct: f64,
    /// Mean topological router extinction ratio in decibels ($\ge 25.0\text{ dB}$ required).
    pub mean_router_extinction_db: f64,
    /// Minimum topological router extinction ratio in decibels.
    pub min_router_extinction_db: f64,
    /// Mean multiplexed transmission capacity in Gbps ($\ge 10.0\text{ Gbps}$ required).
    pub mean_capacity_gbps: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for chiral phonon spin-mechanics.
#[derive(Debug, Clone)]
pub struct ChiralPhononSpinBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for ChiralPhononSpinBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl ChiralPhononSpinBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> ChiralPhononSpinBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<ChiralPhononSpinSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let freq = 1.5 + 7.0 * frac; // 1.5 to 8.5 GHz
                let velocity = 2800.0 + 1400.0 * pseudo_hash; // 2800 to 4200 m/s
                let soc = 15.0 + 60.0 * frac; // 15 to 75 MHz
                let charge = match (idx % 6) as i32 {
                    0 => -3,
                    1 => -2,
                    2 => -1,
                    3 => 1,
                    4 => 2,
                    _ => 3,
                };
                let pairs = 25 + ((idx * 7) % 85); // 25 to 110 pairs
                let k2 = 0.035 + 0.050 * pseudo_hash; // 0.035 to 0.085
                let radius = 8.0 + 35.0 * frac; // 8 to 43 um
                let pitch = 0.6 + 1.8 * pseudo_hash; // 0.6 to 2.4 um
                let power = 1.0 + 15.0 * frac; // 1.0 to 16.0 mW

                let params = ChiralPhononSpinParams {
                    acoustic_frequency_ghz: freq,
                    acoustic_velocity_m_per_s: velocity,
                    spin_orbit_coupling_mhz: soc,
                    topological_oam_charge: charge,
                    transducer_finger_pairs: pairs,
                    electromechanical_coupling_k2: k2,
                    waveguide_radius_um: radius,
                    spiral_pitch_um: pitch,
                    pump_rf_power_mw: power,
                };

                let solver = ChiralPhononSpinSolver::new(params);
                let m = solver.solve();

                ChiralPhononSpinSweepPoint {
                    isolation_db: m.oam_mode_isolation_db,
                    crosstalk_db: m.channel_crosstalk_db,
                    efficiency_pct: m.transduction_efficiency_pct,
                    insertion_loss_db: m.insertion_loss_db,
                    purity_pct: m.spin_orbit_purity_pct,
                    router_extinction_db: m.router_extinction_ratio_db,
                    capacity_gbps: m.multiplexed_capacity_gbps,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut max_iso = f64::MIN;

        let mut sum_xtalk = 0.0;
        let mut max_xtalk = f64::MIN;

        let mut sum_eff = 0.0;
        let mut min_eff = f64::MAX;

        let mut sum_il = 0.0;
        let mut max_il = f64::MIN;

        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;

        let mut sum_ext = 0.0;
        let mut min_ext = f64::MAX;

        let mut sum_cap = 0.0;
        let mut compliant_count = 0;

        for r in &results {
            sum_iso += r.isolation_db;
            if r.isolation_db < min_iso {
                min_iso = r.isolation_db;
            }
            if r.isolation_db > max_iso {
                max_iso = r.isolation_db;
            }

            sum_xtalk += r.crosstalk_db;
            if r.crosstalk_db > max_xtalk {
                max_xtalk = r.crosstalk_db;
            }

            sum_eff += r.efficiency_pct;
            if r.efficiency_pct < min_eff {
                min_eff = r.efficiency_pct;
            }

            sum_il += r.insertion_loss_db;
            if r.insertion_loss_db > max_il {
                max_il = r.insertion_loss_db;
            }

            sum_purity += r.purity_pct;
            if r.purity_pct < min_purity {
                min_purity = r.purity_pct;
            }

            sum_ext += r.router_extinction_db;
            if r.router_extinction_db < min_ext {
                min_ext = r.router_extinction_db;
            }

            sum_cap += r.capacity_gbps;

            // Physical criteria validation:
            let is_compliant = r.isolation_db >= 25.0
                && r.crosstalk_db <= -20.0
                && r.efficiency_pct >= 70.0
                && r.insertion_loss_db <= 2.0
                && r.purity_pct >= 90.0
                && r.router_extinction_db >= 25.0
                && r.capacity_gbps >= 10.0;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        ChiralPhononSpinBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_isolation_db: sum_iso / n,
            min_isolation_db: min_iso,
            max_isolation_db: max_iso,
            mean_crosstalk_db: sum_xtalk / n,
            max_crosstalk_db: max_xtalk,
            mean_efficiency_pct: sum_eff / n,
            min_efficiency_pct: min_eff,
            mean_insertion_loss_db: sum_il / n,
            max_insertion_loss_db: max_il,
            mean_purity_pct: sum_purity / n,
            min_purity_pct: min_purity,
            mean_router_extinction_db: sum_ext / n,
            min_router_extinction_db: min_ext,
            mean_capacity_gbps: sum_cap / n,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
