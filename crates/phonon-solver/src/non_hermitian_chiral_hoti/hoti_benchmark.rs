//! Multi-threaded Rayon benchmark runner for non-Hermitian chiral HOTI systems,
//! corner localization contrast >= 30.0 dB, and acoustoelectric rectification >= 20.0 dB across 10,000 sweeps.

use super::hoti_corner_solver::NonHermitianHotiSolver;
use phonon_models::non_hermitian_chiral_hoti::NonHermitianHotiParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point in non-Hermitian chiral HOTI lattices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HotiSweepPoint {
    pub localization_contrast_db: f64,
    pub skin_depth_cells: f64,
    pub quadrupole_polarization: f64,
    pub acoustoelectric_current_a_m2: f64,
    pub rectification_db: f64,
    pub corner_snr_db: f64,
    pub corner_modes_count: usize,
}

/// Comprehensive benchmark report for non-Hermitian chiral HOTI systems.
#[derive(Debug, Clone, PartialEq)]
pub struct NonHermitianHotiBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean corner localization contrast in decibels ($\ge 30.0\text{ dB}$ required).
    pub mean_localization_contrast_db: f64,
    /// Minimum corner localization contrast in decibels.
    pub min_localization_contrast_db: f64,
    /// Maximum corner localization contrast in decibels.
    pub max_localization_contrast_db: f64,
    /// Mean corner skin depth in unit cells ($\le 2.0\text{ cells}$ required).
    pub mean_skin_depth_cells: f64,
    /// Maximum corner skin depth in unit cells.
    pub max_skin_depth_cells: f64,
    /// Mean non-reciprocal acoustoelectric rectification in decibels ($\ge 20.0\text{ dB}$ required).
    pub mean_rectification_db: f64,
    /// Minimum non-reciprocal acoustoelectric rectification in decibels.
    pub min_rectification_db: f64,
    /// Mean corner sensor SNR in decibels ($\ge 20.0\text{ dB}$ required).
    pub mean_sensor_snr_db: f64,
    /// Minimum corner sensor SNR in decibels.
    pub min_sensor_snr_db: f64,
    /// Mean acoustoelectric current density in $\text{A/m}^2$.
    pub mean_acoustoelectric_current_a_m2: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for non-Hermitian chiral HOTIs.
#[derive(Debug, Clone)]
pub struct NonHermitianHotiBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for NonHermitianHotiBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl NonHermitianHotiBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> NonHermitianHotiBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<HotiSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Varied lattice dimensions & hoppings
                let lx = 14 + (idx % 16); // 14 to 29 cells
                let ly = 14 + ((idx / 16) % 16);
                let gamma = 0.25 + 0.35 * (1.0 - frac); // 0.25 to 0.60 MHz
                let lambda = 1.40 + 1.20 * frac; // 1.40 to 2.60 MHz
                let eta = 0.25 + 0.50 * pseudo_hash; // 0.25 to 0.75
                let p_flux = 1.5 + 8.5 * frac; // 1.5 to 10.0 mW/mm
                let k2 = 1.8 + 3.2 * pseudo_hash; // 1.8% to 5.0%
                let mu = 2000.0 + 8000.0 * frac; // 2000 to 10000 cm^2/Vs
                let freq = 0.8 + 2.4 * pseudo_hash; // 0.8 to 3.2 GHz

                let params = NonHermitianHotiParams {
                    grid_dim_x: lx,
                    grid_dim_y: ly,
                    intra_cell_hopping_mhz: gamma,
                    inter_cell_hopping_mhz: lambda,
                    non_reciprocal_asymmetry_eta: eta,
                    saw_power_flux_mw_mm: p_flux,
                    piezoelectric_coupling_k2_pct: k2,
                    carrier_mobility_cm2_v_s: mu,
                    saw_frequency_ghz: freq,
                };

                let solver = NonHermitianHotiSolver::new(params);
                let metrics = solver.solve();

                HotiSweepPoint {
                    localization_contrast_db: metrics.corner_localization_contrast_db,
                    skin_depth_cells: metrics.corner_skin_depth_cells,
                    quadrupole_polarization: metrics.quantized_quadrupole_polarization,
                    acoustoelectric_current_a_m2: metrics.acoustoelectric_current_density_a_m2,
                    rectification_db: metrics.acoustoelectric_rectification_db,
                    corner_snr_db: metrics.corner_sensor_snr_db,
                    corner_modes_count: metrics.topological_corner_mode_count,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1.0e-6);

        let mut sum_contrast = 0.0;
        let mut min_contrast = f64::MAX;
        let mut max_contrast = f64::MIN;

        let mut sum_xi = 0.0;
        let mut max_xi = f64::MIN;

        let mut sum_rect = 0.0;
        let mut min_rect = f64::MAX;

        let mut sum_snr = 0.0;
        let mut min_snr = f64::MAX;

        let mut sum_jae = 0.0;
        let mut compliant_count = 0usize;

        for pt in &results {
            sum_contrast += pt.localization_contrast_db;
            if pt.localization_contrast_db < min_contrast {
                min_contrast = pt.localization_contrast_db;
            }
            if pt.localization_contrast_db > max_contrast {
                max_contrast = pt.localization_contrast_db;
            }

            sum_xi += pt.skin_depth_cells;
            if pt.skin_depth_cells > max_xi {
                max_xi = pt.skin_depth_cells;
            }

            sum_rect += pt.rectification_db;
            if pt.rectification_db < min_rect {
                min_rect = pt.rectification_db;
            }

            sum_snr += pt.corner_snr_db;
            if pt.corner_snr_db < min_snr {
                min_snr = pt.corner_snr_db;
            }

            sum_jae += pt.acoustoelectric_current_a_m2;

            // Strict compliance criteria:
            // 1. Localization contrast >= 30.0 dB
            // 2. Skin depth <= 2.0 cells
            // 3. Quadrupole polarization == 0.5
            // 4. Rectification >= 20.0 dB
            // 5. Sensor SNR >= 20.0 dB
            // 6. Corner modes count == 4
            if pt.localization_contrast_db >= 30.0
                && pt.skin_depth_cells <= 2.0
                && pt.quadrupole_polarization == 0.5
                && pt.rectification_db >= 20.0
                && pt.corner_snr_db >= 20.0
                && pt.corner_modes_count == 4
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        NonHermitianHotiBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_localization_contrast_db: sum_contrast / n,
            min_localization_contrast_db: min_contrast,
            max_localization_contrast_db: max_contrast,
            mean_skin_depth_cells: sum_xi / n,
            max_skin_depth_cells: max_xi,
            mean_rectification_db: sum_rect / n,
            min_rectification_db: min_rect,
            mean_sensor_snr_db: sum_snr / n,
            min_sensor_snr_db: min_snr,
            mean_acoustoelectric_current_a_m2: sum_jae / n,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
