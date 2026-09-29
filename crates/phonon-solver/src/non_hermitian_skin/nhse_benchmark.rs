//! Multi-threaded Rayon benchmark for the Non-Hermitian Skin Effect (NHSE),
//! acoustic exceptional surfaces, and directional amplifiers.

use super::directional_amplifier_solver::DirectionalAmplifierSolver;
use super::non_bloch_transfer_matrix_solver::NonBlochTransferMatrixSolver;
use phonon_models::non_hermitian_skin::NhseLatticeParams;
use rayon::prelude::*;
use std::time::Instant;

/// Single NHSE sweep point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NhseSweepPoint {
    pub non_reciprocal_gamma: f64,
    pub num_sites: usize,
    pub skin_localization_db: f64,
    pub directional_contrast_db: f64,
    pub point_gap_winding: i32,
}

/// Comprehensive benchmark report for NHSE and Directional Amplification.
#[derive(Debug, Clone, PartialEq)]
pub struct NhseBenchmarkReport {
    /// Total number of parallel sweeps executed.
    pub total_cycles: usize,
    /// Elapsed benchmark wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in cycles per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean spatial skin localization factor in decibels ($\\ge 30\\text{ dB}$ required).
    pub mean_skin_localization_db: f64,
    /// Minimum spatial skin localization factor in decibels ($\\ge 30\\text{ dB}$ required).
    pub min_skin_localization_db: f64,
    /// Mean directional amplification contrast in decibels ($\\ge 30\\text{ dB}$ required).
    pub mean_directional_contrast_db: f64,
    /// Minimum directional amplification contrast in decibels ($\\ge 30\\text{ dB}$ required).
    pub min_directional_contrast_db: f64,
    /// Fraction of sweeps achieving $\\Lambda_{\\mathrm{skin}} \\ge 30\\text{ dB}$ and $\\mathcal{G}_{\\mathrm{dir}} \\ge 30\\text{ dB}$.
    pub high_contrast_fraction: f64,
}

/// Parallel benchmark runner for NHSE and directional amplifiers.
#[derive(Debug, Clone)]
pub struct NhseBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for NhseBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl NhseBenchmarkRunner {
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Runs parallel Rayon benchmark across `total_cycles` sweeps.
    pub fn run_parallel_benchmark(&self) -> NhseBenchmarkReport {
        let start = Instant::now();

        let sweep_results: Vec<NhseSweepPoint> = (0..self.total_cycles)
            .into_par_iter()
            .map(|idx| {
                // Vary gamma between 0.21 and 0.36
                let gamma = 0.21 + (idx as f64 % 30.0) * 0.005;
                // Vary chain length between 18 and 32 sites
                let sites = 18 + (idx % 15);

                let params = NhseLatticeParams {
                    num_sites: sites,
                    base_coupling_rad_s: 1000.0,
                    non_reciprocal_gamma: gamma,
                    onsite_frequency_rad_s: 5000.0,
                };

                let spec_solver = NonBlochTransferMatrixSolver::new(params);
                let spec_res = spec_solver.solve();

                let amp_solver = DirectionalAmplifierSolver::new(params);
                let amp_res = amp_solver.solve();

                NhseSweepPoint {
                    non_reciprocal_gamma: gamma,
                    num_sites: sites,
                    skin_localization_db: spec_res.skin_localization_db,
                    directional_contrast_db: amp_res.directional_contrast_db,
                    point_gap_winding: spec_res.point_gap_winding,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (self.total_cycles as f64) / elapsed.max(1e-6);

        let mut sum_skin = 0.0;
        let mut min_skin = f64::MAX;
        let mut sum_dir = 0.0;
        let mut min_dir = f64::MAX;
        let mut high_contrast_count = 0;

        for pt in &sweep_results {
            sum_skin += pt.skin_localization_db;
            if pt.skin_localization_db < min_skin {
                min_skin = pt.skin_localization_db;
            }

            sum_dir += pt.directional_contrast_db;
            if pt.directional_contrast_db < min_dir {
                min_dir = pt.directional_contrast_db;
            }

            if pt.skin_localization_db >= 30.0 && pt.directional_contrast_db >= 30.0 {
                high_contrast_count += 1;
            }
        }

        let n = self.total_cycles as f64;
        let mean_skin = sum_skin / n;
        let mean_dir = sum_dir / n;
        let high_contrast_frac = (high_contrast_count as f64) / n;

        NhseBenchmarkReport {
            total_cycles: self.total_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_skin_localization_db: mean_skin,
            min_skin_localization_db: min_skin,
            mean_directional_contrast_db: mean_dir,
            min_directional_contrast_db: min_dir,
            high_contrast_fraction: high_contrast_frac,
        }
    }
}
