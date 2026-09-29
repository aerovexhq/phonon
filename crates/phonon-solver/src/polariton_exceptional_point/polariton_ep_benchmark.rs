//! Multi-threaded Rayon benchmark runner for non-Hermitian polariton exceptional points across 10,000 parameter sweeps.

use super::polariton_ep_solver::PolaritonEpSolver;
use phonon_models::polariton_exceptional_point::PolaritonEpParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for polariton EP dynamics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonEpSweepPoint {
    pub sensitivity_db: f64,
    pub purity_pct: f64,
    pub threshold_mw: f64,
    pub linewidth_mhz: f64,
    pub gyro_enhancement: f64,
    pub winding_charge: f64,
}

/// Comprehensive benchmark report for non-Hermitian polariton EP systems.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonEpBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean non-Hermitian square-root sensitivity in decibels ($\ge 30.0\text{ dB}$ required).
    pub mean_exceptional_sensitivity_db: f64,
    /// Minimum non-Hermitian square-root sensitivity in decibels.
    pub min_exceptional_sensitivity_db: f64,
    /// Maximum non-Hermitian square-root sensitivity in decibels.
    pub max_exceptional_sensitivity_db: f64,
    /// Mean chiral mode purity in percent ($\ge 99.0\%$ required).
    pub mean_chiral_mode_purity_pct: f64,
    /// Minimum chiral mode purity in percent.
    pub min_chiral_mode_purity_pct: f64,
    /// Mean macroscopic condensation threshold in milliwatts ($\le 5.0\text{ mW}$ required).
    pub mean_condensation_threshold_mw: f64,
    /// Maximum macroscopic condensation threshold in milliwatts.
    pub max_condensation_threshold_mw: f64,
    /// Mean polariton laser spectral linewidth in $\text{MHz}$ ($\le 50.0\text{ MHz}$ required).
    pub mean_polariton_laser_linewidth_mhz: f64,
    /// Maximum polariton laser spectral linewidth in $\text{MHz}$.
    pub max_polariton_laser_linewidth_mhz: f64,
    /// Mean gyroscopic Sagnac scale-factor enhancement ratio ($\ge 10.0\times$ required).
    pub mean_gyro_scale_factor_enhancement: f64,
    /// Minimum gyroscopic Sagnac scale-factor enhancement ratio.
    pub min_gyro_scale_factor_enhancement: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for polariton exceptional points.
#[derive(Debug, Clone)]
pub struct PolaritonEpBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for PolaritonEpBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl PolaritonEpBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> PolaritonEpBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<PolaritonEpSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let decay = 15.0 + 30.0 * frac; // 15 to 45 MHz
                let rabi = 5.0 + 9.0 * pseudo_hash; // 5 to 14 meV
                let freq = 1.2 + 2.8 * frac; // 1.2 to 4.0 GHz
                let j_coup = 30.0 + 70.0 * frac; // 30 to 100 MHz
                let gamma = 2.0 * j_coup; // At or near EP2
                let pump = 2.5 + 8.5 * pseudo_hash; // 2.5 to 11.0 mW
                let t_encirc = 30.0 + 90.0 * frac; // 30 to 120 ns
                let strain = 1.0 + 35.0 * pseudo_hash; // 1.0 to 36.0 ppm

                let params = PolaritonEpParams {
                    cavity_photon_decay_mhz: decay,
                    exciton_coupling_rabi_mev: rabi,
                    acoustic_phonon_frequency_ghz: freq,
                    inter_cavity_coupling_j_mhz: j_coup,
                    gain_loss_contrast_gamma_mhz: gamma,
                    pump_power_mw: pump,
                    encirclement_period_ns: t_encirc,
                    perturbation_strain_ppm: strain,
                };

                let solver = PolaritonEpSolver::new(params);
                let m = solver.solve();

                PolaritonEpSweepPoint {
                    sensitivity_db: m.exceptional_sensitivity_db,
                    purity_pct: m.chiral_mode_purity_pct,
                    threshold_mw: m.condensation_threshold_mw,
                    linewidth_mhz: m.polariton_laser_linewidth_mhz,
                    gyro_enhancement: m.gyro_scale_factor_enhancement,
                    winding_charge: m.topological_winding_charge,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1.0e-6);

        let mut sum_sens = 0.0;
        let mut min_sens = f64::MAX;
        let mut max_sens = f64::MIN;

        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;

        let mut sum_th = 0.0;
        let mut max_th = f64::MIN;

        let mut sum_lw = 0.0;
        let mut max_lw = f64::MIN;

        let mut sum_gyro = 0.0;
        let mut min_gyro = f64::MAX;

        let mut compliant_count = 0usize;

        for pt in &results {
            sum_sens += pt.sensitivity_db;
            if pt.sensitivity_db < min_sens {
                min_sens = pt.sensitivity_db;
            }
            if pt.sensitivity_db > max_sens {
                max_sens = pt.sensitivity_db;
            }

            sum_purity += pt.purity_pct;
            if pt.purity_pct < min_purity {
                min_purity = pt.purity_pct;
            }

            sum_th += pt.threshold_mw;
            if pt.threshold_mw > max_th {
                max_th = pt.threshold_mw;
            }

            sum_lw += pt.linewidth_mhz;
            if pt.linewidth_mhz > max_lw {
                max_lw = pt.linewidth_mhz;
            }

            sum_gyro += pt.gyro_enhancement;
            if pt.gyro_enhancement < min_gyro {
                min_gyro = pt.gyro_enhancement;
            }

            // Strict compliance conditions:
            // 1. Exceptional sensitivity >= 30.0 dB
            // 2. Chiral mode purity >= 99.0%
            // 3. Condensation threshold <= 5.0 mW
            // 4. Laser linewidth <= 50.0 MHz
            // 5. Gyro scale factor enhancement >= 10.0x
            // 6. Topological charge == 0.5
            if pt.sensitivity_db >= 30.0
                && pt.purity_pct >= 99.0
                && pt.threshold_mw <= 5.0
                && pt.linewidth_mhz <= 50.0
                && pt.gyro_enhancement >= 10.0
                && pt.winding_charge == 0.5
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        PolaritonEpBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_exceptional_sensitivity_db: sum_sens / n,
            min_exceptional_sensitivity_db: min_sens,
            max_exceptional_sensitivity_db: max_sens,
            mean_chiral_mode_purity_pct: sum_purity / n,
            min_chiral_mode_purity_pct: min_purity,
            mean_condensation_threshold_mw: sum_th / n,
            max_condensation_threshold_mw: max_th,
            mean_polariton_laser_linewidth_mhz: sum_lw / n,
            max_polariton_laser_linewidth_mhz: max_lw,
            mean_gyro_scale_factor_enhancement: sum_gyro / n,
            min_gyro_scale_factor_enhancement: min_gyro,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
