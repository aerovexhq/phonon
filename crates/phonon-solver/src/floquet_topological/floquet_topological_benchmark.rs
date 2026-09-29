//! Multi-threaded Rayon parallel benchmark runner for topological Floquet engineering,
//! ultrafast chiral light driving, and dynamic Hall states.

use super::floquet_magnus_solver::FloquetMagnusSolver;
use super::floquet_switch_solver::FloquetSwitchSolver;
use phonon_models::floquet_topological::{
    FloquetDiracMaterial, FloquetDriveParams, FloquetOpticalSwitch,
};
use rayon::prelude::*;
use std::time::Instant;

/// Single evaluated parameter point in the Floquet benchmark sweep.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetTopologicalSweepPoint {
    pub photon_energy_ev: f64,
    pub peak_field_v_m: f64,
    pub chirality: f64,
    pub gap_opening_mev: f64,
    pub switch_contrast_db: f64,
    pub unitarity_error: f64,
}

/// Comprehensive benchmark report for Floquet engineering.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetTopologicalBenchmarkReport {
    /// Total number of parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean dynamic bandgap opening in $\text{meV}$ ($\ge 50\text{ meV}$ required for circular drive).
    pub mean_gap_opening_mev: f64,
    /// Minimum dynamic bandgap opening in $\text{meV}$.
    pub min_gap_opening_mev: f64,
    /// Mean dynamic switching ON/OFF contrast in decibels ($\ge 30\text{ dB}$ required).
    pub mean_switch_contrast_db: f64,
    /// Minimum dynamic switching ON/OFF contrast in decibels ($\ge 30\text{ dB}$ required).
    pub min_switch_contrast_db: f64,
    /// Maximum Frobenius norm unitarity error $\|U^\dagger U - I\|_F$.
    pub max_unitarity_error: f64,
    /// Fraction of parameter sweeps complying with $\Delta_{\mathrm{gap}} \ge 50\text{ meV}$ and $\mathcal{R}_{\mathrm{switch}} \ge 30\text{ dB}$.
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for Floquet topological engineering.
#[derive(Debug, Clone)]
pub struct FloquetTopologicalBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for FloquetTopologicalBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl FloquetTopologicalBenchmarkRunner {
    /// Creates a benchmark runner with specified number of cycles.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes parallel Rayon benchmark across all parameter sweeps.
    pub fn run_parallel_benchmark(&self) -> FloquetTopologicalBenchmarkReport {
        let start = Instant::now();

        let sweep_results: Vec<FloquetTopologicalSweepPoint> = (0..self.total_cycles)
            .into_par_iter()
            .map(|idx| {
                // Vary photon energy between 0.80 and 0.99 eV
                let photon_energy = 0.80 + (idx as f64 % 20.0) * 0.01;
                // Vary peak electric field between 2.5e8 and 3.46e8 V/m
                let peak_field = 2.5e8 + (idx as f64 % 25.0) * 0.04e8;
                // Chirality alternating between +1.0 (RCP) and -1.0 (LCP)
                let chirality = if (idx % 2) == 0 { 1.0 } else { -1.0 };

                let drive = FloquetDriveParams::new_cw(photon_energy, peak_field, chirality);
                let material = FloquetDiracMaterial::default();

                let magnus_solver = FloquetMagnusSolver::new(material, drive);
                let gap_mev = magnus_solver.solve_dynamic_gap_mev();

                // Compute propagator over 16 time steps
                let u = magnus_solver.compute_one_period_propagator(16, 0.0, 0.0);
                let u_err = magnus_solver.unitarity_error(u);

                let switch = FloquetOpticalSwitch::new(drive, material, 1.0e4, 10e-15);
                let switch_solver = FloquetSwitchSolver::new(switch);
                let switch_resp = switch_solver.solve_steady_state();

                FloquetTopologicalSweepPoint {
                    photon_energy_ev: photon_energy,
                    peak_field_v_m: peak_field,
                    chirality,
                    gap_opening_mev: gap_mev,
                    switch_contrast_db: switch_resp.on_off_contrast_db,
                    unitarity_error: u_err,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (self.total_cycles as f64) / elapsed.max(1e-6);

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut sum_contrast = 0.0;
        let mut min_contrast = f64::MAX;
        let mut max_u_err = 0.0;
        let mut compliant_count = 0;

        for pt in &sweep_results {
            sum_gap += pt.gap_opening_mev;
            if pt.gap_opening_mev < min_gap {
                min_gap = pt.gap_opening_mev;
            }

            sum_contrast += pt.switch_contrast_db;
            if pt.switch_contrast_db < min_contrast {
                min_contrast = pt.switch_contrast_db;
            }

            if pt.unitarity_error > max_u_err {
                max_u_err = pt.unitarity_error;
            }

            if pt.gap_opening_mev >= 50.0 && pt.switch_contrast_db >= 30.0 {
                compliant_count += 1;
            }
        }

        let n = self.total_cycles as f64;
        let mean_gap = sum_gap / n;
        let mean_contrast = sum_contrast / n;
        let compliance_fraction = (compliant_count as f64) / n;

        FloquetTopologicalBenchmarkReport {
            total_cycles: self.total_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_gap_opening_mev: mean_gap,
            min_gap_opening_mev: min_gap,
            mean_switch_contrast_db: mean_contrast,
            min_switch_contrast_db: min_contrast,
            max_unitarity_error: max_u_err,
            compliance_fraction,
        }
    }
}
