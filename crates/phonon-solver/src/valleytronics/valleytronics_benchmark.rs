//! Multi-threaded Rayon benchmark for 2D quantum valleytronics,
//! Berry curvature dipole non-linear Hall effect, and valley Hall transistors.

use super::semiclassical_valley_boltzmann_solver::SemiclassicalValleyBoltzmannSolver;
use super::valley_hall_transistor_solver::ValleyHallTransistorSolver;
use phonon_models::valleytronics::TmdMaterialParams;
use rayon::prelude::*;
use std::time::Instant;

/// Single valleytronic sweep point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleytronicSweepPoint {
    pub electric_field_v_m: f64,
    pub chemical_potential_ev: f64,
    pub rectification_ratio_db: f64,
    pub on_off_ratio_db: f64,
    pub circular_dichroism: f64,
}

/// Comprehensive benchmark report for quantum valleytronics.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleytronicsBenchmarkReport {
    /// Total number of parallel sweeps executed.
    pub total_cycles: usize,
    /// Elapsed benchmark wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in cycles per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean non-linear Hall rectification ratio in decibels.
    pub mean_rectification_db: f64,
    /// Minimum non-linear Hall rectification ratio in decibels ($\ge 20\text{ dB}$ required).
    pub min_rectification_db: f64,
    /// Mean transistor ON/OFF switching ratio in decibels ($\ge 20\text{ dB}$ required).
    pub mean_on_off_ratio_db: f64,
    /// Fraction of sweeps meeting $\mathcal{R} \ge 20\text{ dB}$ and $\mathrm{ON/OFF} \ge 20\text{ dB}$.
    pub high_contrast_fraction: f64,
    /// Mean optical circular dichroism at band edge ($\ge 95\%$ required).
    pub mean_circular_dichroism: f64,
}

/// Parallel benchmark runner for quantum valleytronics.
#[derive(Debug, Clone)]
pub struct ValleytronicsBenchmarkRunner {
    pub total_cycles: usize,
    pub params: TmdMaterialParams,
}

impl Default for ValleytronicsBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl ValleytronicsBenchmarkRunner {
    pub fn new(total_cycles: usize) -> Self {
        let params = TmdMaterialParams::strained_tilted_tmd(1.2e5);
        Self {
            total_cycles,
            params,
        }
    }

    /// Runs parallel Rayon benchmark across `total_cycles` sweeps.
    pub fn run_parallel_benchmark(&self) -> ValleytronicsBenchmarkReport {
        let start = Instant::now();
        let params = self.params;

        let sweep_results: Vec<ValleytronicSweepPoint> = (0..self.total_cycles)
            .into_par_iter()
            .map(|idx| {
                // Vary electric field in high-contrast regime (30 kV/m to 120 kV/m)
                let e_field = 3.0e4 + (idx as f64 % 90.0) * 1.0e3;
                // Vary chemical potential near conduction band edge (0.62 eV to 0.72 eV)
                let mu = 0.62 + (idx as f64 % 20.0) * 0.005;

                let mut solver = SemiclassicalValleyBoltzmannSolver::new(params);
                let res = solver.solve(e_field, mu, 16);

                let mut trans_solver = ValleyHallTransistorSolver::new(params);
                let trans_res = trans_solver.solve(e_field);

                ValleytronicSweepPoint {
                    electric_field_v_m: e_field,
                    chemical_potential_ev: mu,
                    rectification_ratio_db: res.rectification_ratio_db,
                    on_off_ratio_db: trans_res.on_off_ratio_db,
                    circular_dichroism: res.circular_dichroism_edge,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (self.total_cycles as f64) / elapsed.max(1e-6);

        let mut sum_r = 0.0;
        let mut min_r = f64::MAX;
        let mut sum_on_off = 0.0;
        let mut sum_cd = 0.0;
        let mut high_contrast_count = 0;

        for pt in &sweep_results {
            sum_r += pt.rectification_ratio_db;
            if pt.rectification_ratio_db < min_r {
                min_r = pt.rectification_ratio_db;
            }
            sum_on_off += pt.on_off_ratio_db;
            sum_cd += pt.circular_dichroism.abs();

            if pt.rectification_ratio_db >= 20.0 && pt.on_off_ratio_db >= 20.0 {
                high_contrast_count += 1;
            }
        }

        let n = self.total_cycles as f64;
        let mean_r = sum_r / n;
        let mean_on_off = sum_on_off / n;
        let mean_cd = sum_cd / n;
        let high_contrast_frac = (high_contrast_count as f64) / n;

        ValleytronicsBenchmarkReport {
            total_cycles: self.total_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_rectification_db: mean_r,
            min_rectification_db: min_r,
            mean_on_off_ratio_db: mean_on_off,
            high_contrast_fraction: high_contrast_frac,
            mean_circular_dichroism: mean_cd,
        }
    }
}
