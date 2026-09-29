//! Multi-threaded Rayon benchmark runner for quantum topological polariton condensates,
//! optomechanical vortices, and non-equilibrium superfluids across 10,000 sweeps.

use super::polariton_condensate_solver::PolaritonCondensateSolver;
use super::polariton_gate_solver::PolaritonGateSolver;
use super::polariton_vortex_solver::PolaritonVortexSolver;
use phonon_models::polariton_condensate::{
    PolaritonCondensateParams, PolaritonGateParams, PolaritonVortexParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point in polariton condensation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonSweepPoint {
    pub superfluid_fraction: f64,
    pub sound_speed_m_s: f64,
    pub circulation_quantum_ratio: f64,
    pub switching_contrast_db: f64,
    pub healing_length_um: f64,
}

/// Comprehensive benchmark report for polariton condensation.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonBenchmarkReport {
    /// Total number of sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean superfluid fraction ($\ge 0.80$ required).
    pub mean_superfluid_fraction: f64,
    /// Minimum superfluid fraction.
    pub min_superfluid_fraction: f64,
    /// Mean Bogoliubov sound speed in $\text{m/s}$.
    pub mean_sound_speed_m_s: f64,
    /// Mean circulation quantum ratio ($= 1.0$ for $|\ell|=1$).
    pub mean_circulation_ratio: f64,
    /// Mean switching contrast in decibels ($\ge 25.0\text{ dB}$ required).
    pub mean_switching_contrast_db: f64,
    /// Minimum switching contrast in decibels.
    pub min_switching_contrast_db: f64,
    /// Fraction of parameter sweeps complying with physical criteria.
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for polariton condensation.
#[derive(Debug, Clone)]
pub struct PolaritonCondensateBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for PolaritonCondensateBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl PolaritonCondensateBenchmarkRunner {
    /// Creates a benchmark runner with specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes parallel Rayon benchmark across all parameter sweeps.
    pub fn run_parallel_benchmark(&self) -> PolaritonBenchmarkReport {
        let start = Instant::now();

        let sweep_results: Vec<PolaritonSweepPoint> = (0..self.total_cycles)
            .into_par_iter()
            .map(|idx| {
                // Vary effective mass ratio between 1.5e-4 and 4.0e-4
                let m_ratio = 1.5e-4 + (idx as f64 % 26.0) * 1.0e-5;
                // Vary interaction g between 0.015 and 0.035 meV*um^2
                let g = 0.015 + (idx as f64 % 21.0) * 0.001;
                // Vary pump power between 0.35 and 0.77 um^-2*ps^-1 (above threshold 0.04)
                let pump = 0.35 + (idx as f64 % 21.0) * 0.02;
                // Vary extinction efficiency between 0.997 and 0.9995
                let ext = 0.997 + (idx as f64 % 26.0) * 0.0001;

                let cond_params = PolaritonCondensateParams::new(m_ratio, g, pump);
                let vortex_params = PolaritonVortexParams::new(1, 1.5);
                let gate_params = PolaritonGateParams::new(1.8, ext);

                let cond_solver = PolaritonCondensateSolver::new(cond_params);
                let cond_metrics = cond_solver.solve_condensate_metrics();

                let vortex_solver = PolaritonVortexSolver::new(cond_params, vortex_params);
                let vortex_metrics = vortex_solver.solve_vortex_metrics();

                let gate_solver = PolaritonGateSolver::new(cond_params, gate_params);
                let gate_metrics = gate_solver.solve_gate_metrics();

                PolaritonSweepPoint {
                    superfluid_fraction: cond_metrics.superfluid_fraction,
                    sound_speed_m_s: cond_metrics.sound_speed_m_s,
                    circulation_quantum_ratio: vortex_metrics.circulation_quantum_ratio,
                    switching_contrast_db: gate_metrics.switching_contrast_db,
                    healing_length_um: cond_metrics.healing_length_um,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (self.total_cycles as f64) / elapsed.max(1e-6);

        let mut sum_sf = 0.0;
        let mut min_sf = f64::MAX;
        let mut sum_cs = 0.0;
        let mut sum_circ = 0.0;
        let mut sum_contrast = 0.0;
        let mut min_contrast = f64::MAX;
        let mut compliant_count = 0;

        for pt in &sweep_results {
            sum_sf += pt.superfluid_fraction;
            if pt.superfluid_fraction < min_sf {
                min_sf = pt.superfluid_fraction;
            }

            sum_cs += pt.sound_speed_m_s;
            sum_circ += pt.circulation_quantum_ratio;

            sum_contrast += pt.switching_contrast_db;
            if pt.switching_contrast_db < min_contrast {
                min_contrast = pt.switching_contrast_db;
            }

            if pt.superfluid_fraction >= 0.80
                && pt.sound_speed_m_s > 1.0e5
                && (pt.circulation_quantum_ratio - 1.0).abs() < 1e-4
                && pt.switching_contrast_db >= 25.0
            {
                compliant_count += 1;
            }
        }

        let n = self.total_cycles as f64;

        PolaritonBenchmarkReport {
            total_cycles: self.total_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_superfluid_fraction: sum_sf / n,
            min_superfluid_fraction: min_sf,
            mean_sound_speed_m_s: sum_cs / n,
            mean_circulation_ratio: sum_circ / n,
            mean_switching_contrast_db: sum_contrast / n,
            min_switching_contrast_db: min_contrast,
            compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
