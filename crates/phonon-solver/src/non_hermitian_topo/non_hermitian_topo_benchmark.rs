//! Multi-threaded Rayon benchmark runner for non-Hermitian skin effect,
//! exceptional point sensors, and topological phonon lasers across 10,000 sweeps.

use super::non_bloch_skin_solver::NonBlochSkinSolver;
use super::topological_phonon_laser_solver::TopologicalPhononLaserSolver;
use phonon_models::non_hermitian_topo::NonHermitianSkinParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point in non-Hermitian topology.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianTopoSweepPoint {
    pub skin_depth_unit_cells: f64,
    pub skin_localization_contrast_db: f64,
    pub sensitivity_enhancement: f64,
    pub laser_threshold_mw: f64,
    pub smsr_db: f64,
    pub directional_gain_db: f64,
}

/// Comprehensive benchmark report for non-Hermitian topological systems.
#[derive(Debug, Clone, PartialEq)]
pub struct NonHermitianTopoBenchmarkReport {
    /// Total sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean skin depth localization length in unit cells ($\le 3.0    ext{ cells}$ required).
    pub mean_skin_depth: f64,
    /// Maximum skin depth in unit cells.
    pub max_skin_depth: f64,
    /// Mean skin localization contrast in decibels ($\ge 25.0    ext{ dB}$ required).
    pub mean_skin_contrast_db: f64,
    /// Minimum skin localization contrast in decibels.
    pub min_skin_contrast_db: f64,
    /// Mean exceptional point sensitivity enhancement ($\ge 10.0    imes$ required).
    pub mean_sensitivity_enhancement: f64,
    /// Minimum exceptional point sensitivity enhancement.
    pub min_sensitivity_enhancement: f64,
    /// Mean laser threshold power in milliwatts ($\le 5.0    ext{ mW}$ required).
    pub mean_laser_threshold_mw: f64,
    /// Maximum laser threshold power in milliwatts.
    pub max_laser_threshold_mw: f64,
    /// Mean side-mode suppression ratio in decibels ($\ge 25.0    ext{ dB}$ required).
    pub mean_smsr_db: f64,
    /// Minimum side-mode suppression ratio in decibels.
    pub min_smsr_db: f64,
    /// Mean directional amplification gain in decibels ($\ge 25.0    ext{ dB}$ required).
    pub mean_directional_gain_db: f64,
    /// Minimum directional amplification gain in decibels.
    pub min_directional_gain_db: f64,
    /// Fraction of sweeps satisfying all physical criteria.
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for non-Hermitian topology.
#[derive(Debug, Clone)]
pub struct NonHermitianTopoBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for NonHermitianTopoBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl NonHermitianTopoBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> NonHermitianTopoBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<NonHermitianTopoSweepPoint> = (0..total)
            .into_par_iter()
            .map(|i| {
                let u = i as f64 / total.max(1) as f64;

                // Parameter sweeping across physical ranges:
                // Forward hopping t_R in [2.0, 3.5] MHz
                let tr = 2.0 + 1.5 * u;
                // Backward hopping t_L in [0.30, 0.65] MHz
                let tl = 0.30 + 0.35 * ((i * 7) % total) as f64 / total as f64;
                // Onsite gain/loss gamma in [0.6, 1.8] MHz
                let gamma = 0.6 + 1.2 * ((i * 11) % total) as f64 / total as f64;
                // Perturbation epsilon in [5e-4, 5e-3]
                let eps = (5.0 + 45.0 * ((i * 17) % total) as f64 / total as f64) * 1.0e-4;
                // Lattice sites N in [25, 45]
                let sites = 25 + (20 * ((i * 23) % total)) / total;

                let params = NonHermitianSkinParams {
                    lattice_sites_count: sites,
                    forward_hopping_mhz: tr,
                    backward_hopping_mhz: tl,
                    onsite_gain_loss_mhz: gamma,
                    perturbation_epsilon: eps,
                    pump_power_mw: 6.0,
                };

                let skin_solver = NonBlochSkinSolver::new(params);
                let skin_metrics = skin_solver.solve_skin_metrics();

                let laser_solver = TopologicalPhononLaserSolver::new(params);
                let laser_metrics = laser_solver.solve_laser_metrics();

                NonHermitianTopoSweepPoint {
                    skin_depth_unit_cells: skin_metrics.skin_depth_unit_cells,
                    skin_localization_contrast_db: skin_metrics.skin_localization_contrast_db,
                    sensitivity_enhancement: skin_metrics.sensitivity_enhancement_factor,
                    laser_threshold_mw: laser_metrics.laser_threshold_power_mw,
                    smsr_db: laser_metrics.side_mode_suppression_ratio_db,
                    directional_gain_db: laser_metrics.directional_amplification_gain_db,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_depth = 0.0;
        let mut max_depth = f64::NEG_INFINITY;
        let mut sum_contrast = 0.0;
        let mut min_contrast = f64::INFINITY;
        let mut sum_sens = 0.0;
        let mut min_sens = f64::INFINITY;
        let mut sum_th = 0.0;
        let mut max_th = f64::NEG_INFINITY;
        let mut sum_smsr = 0.0;
        let mut min_smsr = f64::INFINITY;
        let mut sum_dir = 0.0;
        let mut min_dir = f64::INFINITY;
        let mut compliant_count = 0usize;

        for p in &results {
            sum_depth += p.skin_depth_unit_cells;
            if p.skin_depth_unit_cells > max_depth {
                max_depth = p.skin_depth_unit_cells;
            }

            sum_contrast += p.skin_localization_contrast_db;
            if p.skin_localization_contrast_db < min_contrast {
                min_contrast = p.skin_localization_contrast_db;
            }

            sum_sens += p.sensitivity_enhancement;
            if p.sensitivity_enhancement < min_sens {
                min_sens = p.sensitivity_enhancement;
            }

            sum_th += p.laser_threshold_mw;
            if p.laser_threshold_mw > max_th {
                max_th = p.laser_threshold_mw;
            }

            sum_smsr += p.smsr_db;
            if p.smsr_db < min_smsr {
                min_smsr = p.smsr_db;
            }

            sum_dir += p.directional_gain_db;
            if p.directional_gain_db < min_dir {
                min_dir = p.directional_gain_db;
            }

            // Criteria:
            // 1. Skin depth <= 3.0 unit cells
            // 2. Skin localization contrast >= 25.0 dB
            // 3. Sensitivity enhancement >= 10.0x
            // 4. Laser threshold <= 5.0 mW
            // 5. SMSR >= 25.0 dB
            // 6. Directional gain >= 25.0 dB
            if p.skin_depth_unit_cells <= 3.0
                && p.skin_localization_contrast_db >= 25.0
                && p.sensitivity_enhancement >= 10.0
                && p.laser_threshold_mw <= 5.0
                && p.smsr_db >= 25.0
                && p.directional_gain_db >= 25.0
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        NonHermitianTopoBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_skin_depth: sum_depth / n,
            max_skin_depth: max_depth,
            mean_skin_contrast_db: sum_contrast / n,
            min_skin_contrast_db: min_contrast,
            mean_sensitivity_enhancement: sum_sens / n,
            min_sensitivity_enhancement: min_sens,
            mean_laser_threshold_mw: sum_th / n,
            max_laser_threshold_mw: max_th,
            mean_smsr_db: sum_smsr / n,
            min_smsr_db: min_smsr,
            mean_directional_gain_db: sum_dir / n,
            min_directional_gain_db: min_dir,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
