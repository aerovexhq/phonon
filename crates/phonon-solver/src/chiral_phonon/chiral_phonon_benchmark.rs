//! Multi-threaded Rayon benchmark for topological chiral phonon heat transport,
//! non-equilibrium thermal rectification, and corner reflection immunity.

use super::negf_phonon_solver::NegfPhononSolver;
use super::thermal_hall_boltzmann_solver::ThermalHallBoltzmannSolver;
use phonon_models::chiral_phonon::{
    ChiralPhononMaterialParams, HoneycombChiralLattice, TopologicalPhononDiode,
};
use rayon::prelude::*;
use std::time::Instant;

/// Single thermal cycle sweep result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralPhononSweepPoint {
    pub t_hot_k: f64,
    pub t_cold_k: f64,
    pub rectification_ratio: f64,
    pub corner_transmission: f64,
    pub backscattering_prob: f64,
    pub thermal_hall_angle_rad: f64,
}

/// Comprehensive benchmark report for topological chiral phononics.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralPhononBenchmarkReport {
    /// Total number of parallel thermal cycles executed.
    pub total_cycles: usize,
    /// Elapsed benchmark wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in cycles per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean thermal rectification ratio across all sweeps.
    pub mean_rectification_ratio: f64,
    /// Minimum thermal rectification ratio observed ($\ge 10.0\times$ required).
    pub min_rectification_ratio: f64,
    /// Mean corner transmission $T_{bend}$ across all sweeps ($\ge 0.90$ required).
    pub mean_corner_transmission: f64,
    /// Maximum backscattering reflection probability observed ($\le 0.10$ required).
    pub max_backscattering_prob: f64,
    /// High-contrast fraction (cycles with $\mathcal{R} \ge 10.0\times$).
    pub high_contrast_fraction: f64,
}

/// Parallel benchmark runner for chiral phonon transport.
#[derive(Debug, Clone)]
pub struct ChiralPhononBenchmarkRunner {
    pub total_cycles: usize,
    pub diode: TopologicalPhononDiode,
    pub lattice: HoneycombChiralLattice,
}

impl Default for ChiralPhononBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl ChiralPhononBenchmarkRunner {
    pub fn new(total_cycles: usize) -> Self {
        let diode = TopologicalPhononDiode::new(200.0, 100.0, 5000.0);
        let params = ChiralPhononMaterialParams::fe2mo3o8_standard();
        let lattice = HoneycombChiralLattice::new(params);
        Self {
            total_cycles,
            diode,
            lattice,
        }
    }

    /// Runs parallel Rayon benchmark across `total_cycles` sweeps.
    pub fn run_parallel_benchmark(&self) -> ChiralPhononBenchmarkReport {
        let start = Instant::now();
        let diode = self.diode.clone();
        let lattice = self.lattice.clone();

        let sweep_results: Vec<ChiralPhononSweepPoint> = (0..self.total_cycles)
            .into_par_iter()
            .map(|idx| {
                // Vary hot temperature between 50 K and 300 K
                // Vary temperature gradient Delta T between 5 K and 50 K
                let t_hot = 50.0 + (idx as f64 % 250.0);
                let delta_t = 5.0 + (idx as f64 % 45.0);
                let t_cold = (t_hot - delta_t).max(10.0);
                let bend_angle = 60.0 + (idx as f64 % 60.0); // 60 to 120 degrees

                let solver = NegfPhononSolver::new(diode.clone());
                let result = solver.solve(t_hot, t_cold, bend_angle);

                let hall_solver = ThermalHallBoltzmannSolver::new(lattice.clone());
                // Evaluate quick Hall point
                let hall_result = hall_solver.solve(t_cold, 8);

                ChiralPhononSweepPoint {
                    t_hot_k: t_hot,
                    t_cold_k: t_cold,
                    rectification_ratio: result.rectification_ratio,
                    corner_transmission: result.corner_bend_transmission,
                    backscattering_prob: result.corner_backscattering_prob,
                    thermal_hall_angle_rad: hall_result.thermal_hall_angle_rad,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (self.total_cycles as f64) / elapsed.max(1e-6);

        let mut sum_r = 0.0;
        let mut min_r = f64::MAX;
        let mut sum_t_bend = 0.0;
        let mut max_r_back = 0.0;
        let mut high_contrast_count = 0;

        for pt in &sweep_results {
            sum_r += pt.rectification_ratio;
            if pt.rectification_ratio < min_r {
                min_r = pt.rectification_ratio;
            }
            sum_t_bend += pt.corner_transmission;
            if pt.backscattering_prob > max_r_back {
                max_r_back = pt.backscattering_prob;
            }
            if pt.rectification_ratio >= 10.0 {
                high_contrast_count += 1;
            }
        }

        let mean_r = sum_r / (self.total_cycles as f64);
        let mean_t_bend = sum_t_bend / (self.total_cycles as f64);
        let high_contrast_frac = (high_contrast_count as f64) / (self.total_cycles as f64);

        ChiralPhononBenchmarkReport {
            total_cycles: self.total_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_rectification_ratio: mean_r,
            min_rectification_ratio: min_r,
            mean_corner_transmission: mean_t_bend,
            max_backscattering_prob: max_r_back,
            high_contrast_fraction: high_contrast_frac,
        }
    }
}
