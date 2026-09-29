//! Multi-threaded Rayon parallel benchmark runner for quantum valley acoustic cavities,
//! pseudomagnetic fields, and phonon valleytronics across 10,000 parameter sweeps.

use super::valley_cavity_solver::ValleyCavitySolver;
use super::valley_router_solver::ValleyRouterSolver;
use phonon_models::valley_acoustic::{
    StrainGaugeParams, ValleyAcousticRouter, ValleyCavityParams, ValleyIndex,
};
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter point in the valley phononics sweep.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyPhononSweepPoint {
    pub strain_coeff: f64,
    pub quality_factor: f64,
    pub pseudomagnetic_field_t: f64,
    pub valley_contrast_db: f64,
    pub purcell_factor: f64,
    pub corner_transmission: f64,
}

/// Comprehensive benchmark report for quantum valley phononics.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyPhononBenchmarkReport {
    /// Total number of sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Sweep execution throughput in cycles per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean pseudomagnetic field in Tesla ($T$) ($> 100\text{ T}$ required).
    pub mean_pseudomagnetic_field_t: f64,
    /// Minimum pseudomagnetic field in Tesla ($T$).
    pub min_pseudomagnetic_field_t: f64,
    /// Mean valley polarization contrast in decibels ($\ge 20\text{ dB}$ required).
    pub mean_valley_contrast_db: f64,
    /// Minimum valley polarization contrast in decibels.
    pub min_valley_contrast_db: f64,
    /// Mean Purcell enhancement factor ($> 10.0$ required).
    pub mean_purcell_factor: f64,
    /// Minimum Purcell enhancement factor.
    pub min_purcell_factor: f64,
    /// Mean corner bend transmission ($T_{\mathrm{bend}} \ge 0.90$ required).
    pub mean_corner_transmission: f64,
    /// Fraction of parameter sweeps complying with $B_{\mathrm{ps}} > 100\text{ T}$, $\mathcal{R}_{\mathrm{valley}} \ge 20\text{ dB}$, $F_P > 10$, and $T_{\mathrm{bend}} \ge 0.90$.
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for valley phononics.
#[derive(Debug, Clone)]
pub struct ValleyPhononBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for ValleyPhononBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl ValleyPhononBenchmarkRunner {
    /// Creates a new benchmark runner with specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes parallel Rayon benchmark across all parameter sweeps.
    pub fn run_parallel_benchmark(&self) -> ValleyPhononBenchmarkReport {
        let start = Instant::now();

        let sweep_results: Vec<ValleyPhononSweepPoint> = (0..self.total_cycles)
            .into_par_iter()
            .map(|idx| {
                // Vary strain coefficient between 6.0e6 and 15.6e6 m^-1 (B_ps in 110 - 287 T)
                let strain_c = 6.0e6 + (idx as f64 % 25.0) * 4.0e5;
                // Vary quality factor between 5,000 and 15,000
                let q = 5000.0 + (idx as f64 % 21.0) * 500.0;
                // Corner bend angle between 60.0 and 120.0 deg
                let bend_angle = 60.0 + (idx as f64 % 13.0) * 5.0;

                let gauge_params = StrainGaugeParams::new(1.0e-9, 3000.0, 3.5, strain_c);
                let bps = gauge_params.pseudomagnetic_field_magnitude_tesla();

                let cavity_params = ValleyCavityParams::new(50.0e-9, 20.0e-9, q, gauge_params);
                let cavity_solver = ValleyCavitySolver::new(cavity_params);
                let mode_k = cavity_solver.solve_mode(ValleyIndex::ValleyK);

                let router = ValleyAcousticRouter::new(0.95, bend_angle, 25.0);
                let router_solver = ValleyRouterSolver::new(router);
                let corner_t = router_solver.solve_corner_transmission();

                ValleyPhononSweepPoint {
                    strain_coeff: strain_c,
                    quality_factor: q,
                    pseudomagnetic_field_t: bps,
                    valley_contrast_db: mode_k.valley_contrast_db,
                    purcell_factor: mode_k.purcell_factor,
                    corner_transmission: corner_t,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (self.total_cycles as f64) / elapsed.max(1e-6);

        let mut sum_bps = 0.0;
        let mut min_bps = f64::MAX;
        let mut sum_contrast = 0.0;
        let mut min_contrast = f64::MAX;
        let mut sum_purcell = 0.0;
        let mut min_purcell = f64::MAX;
        let mut sum_corner = 0.0;
        let mut compliant_count = 0;

        for pt in &sweep_results {
            sum_bps += pt.pseudomagnetic_field_t;
            if pt.pseudomagnetic_field_t < min_bps {
                min_bps = pt.pseudomagnetic_field_t;
            }

            sum_contrast += pt.valley_contrast_db;
            if pt.valley_contrast_db < min_contrast {
                min_contrast = pt.valley_contrast_db;
            }

            sum_purcell += pt.purcell_factor;
            if pt.purcell_factor < min_purcell {
                min_purcell = pt.purcell_factor;
            }

            sum_corner += pt.corner_transmission;

            if pt.pseudomagnetic_field_t >= 100.0
                && pt.valley_contrast_db >= 20.0
                && pt.purcell_factor > 10.0
                && pt.corner_transmission >= 0.90
            {
                compliant_count += 1;
            }
        }

        let n = self.total_cycles as f64;
        let mean_bps = sum_bps / n;
        let mean_contrast = sum_contrast / n;
        let mean_purcell = sum_purcell / n;
        let mean_corner = sum_corner / n;
        let compliance_fraction = (compliant_count as f64) / n;

        ValleyPhononBenchmarkReport {
            total_cycles: self.total_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_pseudomagnetic_field_t: mean_bps,
            min_pseudomagnetic_field_t: min_bps,
            mean_valley_contrast_db: mean_contrast,
            min_valley_contrast_db: min_contrast,
            mean_purcell_factor: mean_purcell,
            min_purcell_factor: min_purcell,
            mean_corner_transmission: mean_corner,
            compliance_fraction,
        }
    }
}
