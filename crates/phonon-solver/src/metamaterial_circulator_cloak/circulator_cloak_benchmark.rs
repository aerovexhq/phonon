//! Multi-threaded Rayon benchmark runner for topological acoustic circulators and cloaks across 10,000 parameter sweeps.

use super::circulator_cloak_solver::MetamaterialCirculatorCloakSolver;
use phonon_models::metamaterial_circulator_cloak::MetamaterialCirculatorCloakParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for circulator and cloak metamaterials.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CirculatorCloakSweepPoint {
    pub cloaking_db: f64,
    pub isolation_db: f64,
    pub loss_db: f64,
    pub bend_pct: f64,
    pub contrast_ratio: f64,
    pub chern: i32,
}

/// Comprehensive benchmark report for topological circulator and cloak devices.
#[derive(Debug, Clone, PartialEq)]
pub struct MetamaterialCirculatorCloakBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean scattering cross-section reduction in decibels ($\ge 20.0\text{ dB}$ required).
    pub mean_cloaking_reduction_db: f64,
    /// Minimum scattering cross-section reduction in decibels.
    pub min_cloaking_reduction_db: f64,
    /// Maximum scattering cross-section reduction in decibels.
    pub max_cloaking_reduction_db: f64,
    /// Mean circulator isolation in decibels ($\ge 25.0\text{ dB}$ required).
    pub mean_circulator_isolation_db: f64,
    /// Minimum circulator isolation in decibels.
    pub min_circulator_isolation_db: f64,
    /// Mean forward transmission insertion loss in decibels ($\le 1.5\text{ dB}$ required).
    pub mean_forward_insertion_loss_db: f64,
    /// Maximum forward transmission insertion loss in decibels.
    pub max_forward_insertion_loss_db: f64,
    /// Mean topological bend transmission in percent ($\ge 90.0\%$ required).
    pub mean_topological_bend_transmission_pct: f64,
    /// Minimum topological bend transmission in percent.
    pub min_topological_bend_transmission_pct: f64,
    /// Mean linear non-reciprocal contrast ratio ($\ge 100.0$ required).
    pub mean_non_reciprocal_contrast_ratio: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for circulator and cloak devices.
#[derive(Debug, Clone)]
pub struct MetamaterialCirculatorCloakBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for MetamaterialCirculatorCloakBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl MetamaterialCirculatorCloakBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> MetamaterialCirculatorCloakBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<CirculatorCloakSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let freq = 3.0 + 20.0 * frac; // 3 to 23 kHz
                let r_in = 30.0 + 40.0 * frac; // 30 to 70 mm
                let r_out = r_in * (1.8 + 1.2 * pseudo_hash); // 54 to 210 mm
                let mach = 0.08 + 0.24 * frac; // 0.08 to 0.32 Mach
                let q_factor = 250.0 + 800.0 * pseudo_hash; // 250 to 1050

                let params = MetamaterialCirculatorCloakParams {
                    operating_frequency_khz: freq,
                    cloak_inner_radius_mm: r_in,
                    cloak_outer_radius_mm: r_out,
                    fluid_bias_mach_number: mach,
                    resonator_q_factor: q_factor,
                    sound_speed_m_s: 343.0,
                    waveguide_ports_count: 3,
                };

                let solver = MetamaterialCirculatorCloakSolver::new(params);
                let m = solver.solve();

                CirculatorCloakSweepPoint {
                    cloaking_db: m.cloaking_cross_section_reduction_db,
                    isolation_db: m.circulator_isolation_db,
                    loss_db: m.forward_insertion_loss_db,
                    bend_pct: m.topological_bend_transmission_pct,
                    contrast_ratio: m.non_reciprocal_contrast_ratio,
                    chern: m.topological_chern_number,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1.0e-6);

        let mut sum_cloak = 0.0;
        let mut min_cloak = f64::MAX;
        let mut max_cloak = f64::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;

        let mut sum_loss = 0.0;
        let mut max_loss = f64::MIN;

        let mut sum_bend = 0.0;
        let mut min_bend = f64::MAX;

        let mut sum_ratio = 0.0;
        let mut compliant_count = 0usize;

        for pt in &results {
            sum_cloak += pt.cloaking_db;
            if pt.cloaking_db < min_cloak {
                min_cloak = pt.cloaking_db;
            }
            if pt.cloaking_db > max_cloak {
                max_cloak = pt.cloaking_db;
            }

            sum_iso += pt.isolation_db;
            if pt.isolation_db < min_iso {
                min_iso = pt.isolation_db;
            }

            sum_loss += pt.loss_db;
            if pt.loss_db > max_loss {
                max_loss = pt.loss_db;
            }

            sum_bend += pt.bend_pct;
            if pt.bend_pct < min_bend {
                min_bend = pt.bend_pct;
            }

            sum_ratio += pt.contrast_ratio;

            // Strict compliance conditions:
            // 1. Cloaking cross-section reduction >= 20.0 dB
            // 2. Circulator isolation >= 25.0 dB
            // 3. Insertion loss <= 1.5 dB
            // 4. Topological bend transmission >= 90.0%
            // 5. Contrast ratio >= 100.0
            // 6. Chern number == 1
            if pt.cloaking_db >= 20.0
                && pt.isolation_db >= 25.0
                && pt.loss_db <= 1.5
                && pt.bend_pct >= 90.0
                && pt.contrast_ratio >= 100.0
                && pt.chern == 1
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        MetamaterialCirculatorCloakBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_cloaking_reduction_db: sum_cloak / n,
            min_cloaking_reduction_db: min_cloak,
            max_cloaking_reduction_db: max_cloak,
            mean_circulator_isolation_db: sum_iso / n,
            min_circulator_isolation_db: min_iso,
            mean_forward_insertion_loss_db: sum_loss / n,
            max_forward_insertion_loss_db: max_loss,
            mean_topological_bend_transmission_pct: sum_bend / n,
            min_topological_bend_transmission_pct: min_bend,
            mean_non_reciprocal_contrast_ratio: sum_ratio / n,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
