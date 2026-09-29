//! Multi-threaded Rayon benchmark runner for Terahertz magnon polaritons,
//! quantum paramagnons, and antiferromagnetic spintronics across 10,000 parameter sweeps.

use super::afm_polariton_solver::AfmPolaritonSolver;
use super::neel_domain_wall_solver::NeelDomainWallSolver;
use phonon_models::afm_spintronics::{
    AfmMaterialParams, NeelDomainWallParams, ThzCavityPolaritonParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point in AFM spintronics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AfmSweepPoint {
    pub rabi_splitting_ghz: f64,
    pub dw_velocity_m_s: f64,
    pub transit_time_ps: f64,
    pub diode_rectification_db: f64,
    pub cooperativity: f64,
}

/// Comprehensive benchmark report for AFM spintronics.
#[derive(Debug, Clone, PartialEq)]
pub struct AfmBenchmarkReport {
    /// Total number of sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean vacuum Rabi splitting $\Omega_R$ in $\text{GHz}$ ($> 100\text{ GHz}$ required).
    pub mean_rabi_splitting_ghz: f64,
    /// Minimum vacuum Rabi splitting in $\text{GHz}$.
    pub min_rabi_splitting_ghz: f64,
    /// Mean domain wall velocity in $\text{m/s}$ ($> 5000\text{ m/s}$ required).
    pub mean_dw_velocity_m_s: f64,
    /// Minimum domain wall velocity in $\text{m/s}$.
    pub min_dw_velocity_m_s: f64,
    /// Mean synaptic transit time in picoseconds ($< 1.0\text{ ps}$ required).
    pub mean_transit_time_ps: f64,
    /// Maximum synaptic transit time in picoseconds.
    pub max_transit_time_ps: f64,
    /// Mean magnon diode rectification in decibels ($\ge 15.0\text{ dB}$ required).
    pub mean_diode_rectification_db: f64,
    /// Minimum magnon diode rectification in decibels.
    pub min_diode_rectification_db: f64,
    /// Mean polariton cooperativity ($> 100$ required).
    pub mean_cooperativity: f64,
    /// Fraction of parameter sweeps complying with physical criteria.
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for AFM spintronics.
#[derive(Debug, Clone)]
pub struct AfmMagnonBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for AfmMagnonBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl AfmMagnonBenchmarkRunner {
    /// Creates a new benchmark runner with specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes parallel Rayon benchmark across all parameter sweeps.
    pub fn run_parallel_benchmark(&self) -> AfmBenchmarkReport {
        let start = Instant::now();

        let sweep_results: Vec<AfmSweepPoint> = (0..self.total_cycles)
            .into_par_iter()
            .map(|idx| {
                // Vary exchange field between 250.0 and 550.0 T
                let hex = 250.0 + (idx as f64 % 31.0) * 10.0;
                // Vary anisotropy field between 0.8 and 2.5 T
                let ha = 0.8 + (idx as f64 % 18.0) * 0.1;
                // Vary vacuum coupling between 120.0 and 220.0 GHz
                let g_ghz = 120.0 + (idx as f64 % 21.0) * 5.0;
                // Vary drive current density between 4.0e11 and 8.0e11 A/m^2
                let je = 4.0e11 + (idx as f64 % 21.0) * 2.0e10;
                // Vary track length between 5.0 and 10.0 nm
                let l_track = (5.0 + (idx as f64 % 6.0) * 1.0) * 1.0e-9;

                let afm_params = AfmMaterialParams::new(hex, ha, 1.0e-11, 5.0e-4);
                let cavity_params = ThzCavityPolaritonParams::new(1.0, 250.0, g_ghz);
                let dw_params = NeelDomainWallParams::new(3.0e-9, l_track, 0.30);

                let polariton_solver = AfmPolaritonSolver::new(afm_params, cavity_params);
                let rabi_splitting = polariton_solver.solve_rabi_splitting_ghz();
                let pol_metrics = polariton_solver.solve_polaritons_at_k(0.0);

                let dw_solver = NeelDomainWallSolver::new(afm_params, dw_params);
                let v_dw = dw_solver.solve_dw_velocity(je);
                let tau_ps = dw_params
                    .evaluate_transit(&afm_params, je, 0.5)
                    .transit_time_ps;
                let diode_rec = dw_solver.solve_magnon_diode_rectification();

                AfmSweepPoint {
                    rabi_splitting_ghz: rabi_splitting,
                    dw_velocity_m_s: v_dw,
                    transit_time_ps: tau_ps,
                    diode_rectification_db: diode_rec,
                    cooperativity: pol_metrics.cooperativity,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (self.total_cycles as f64) / elapsed.max(1e-6);

        let mut sum_rabi = 0.0;
        let mut min_rabi = f64::MAX;
        let mut sum_v = 0.0;
        let mut min_v = f64::MAX;
        let mut sum_tau = 0.0;
        let mut max_tau = f64::MIN;
        let mut sum_rec = 0.0;
        let mut min_rec = f64::MAX;
        let mut sum_coop = 0.0;
        let mut compliant_count = 0;

        for pt in &sweep_results {
            sum_rabi += pt.rabi_splitting_ghz;
            if pt.rabi_splitting_ghz < min_rabi {
                min_rabi = pt.rabi_splitting_ghz;
            }

            sum_v += pt.dw_velocity_m_s;
            if pt.dw_velocity_m_s < min_v {
                min_v = pt.dw_velocity_m_s;
            }

            sum_tau += pt.transit_time_ps;
            if pt.transit_time_ps > max_tau {
                max_tau = pt.transit_time_ps;
            }

            sum_rec += pt.diode_rectification_db;
            if pt.diode_rectification_db < min_rec {
                min_rec = pt.diode_rectification_db;
            }

            sum_coop += pt.cooperativity;

            if pt.rabi_splitting_ghz > 100.0
                && pt.dw_velocity_m_s > 5000.0
                && pt.transit_time_ps < 1.0
                && pt.diode_rectification_db >= 15.0
                && pt.cooperativity > 100.0
            {
                compliant_count += 1;
            }
        }

        let n = self.total_cycles as f64;

        AfmBenchmarkReport {
            total_cycles: self.total_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_rabi_splitting_ghz: sum_rabi / n,
            min_rabi_splitting_ghz: min_rabi,
            mean_dw_velocity_m_s: sum_v / n,
            min_dw_velocity_m_s: min_v,
            mean_transit_time_ps: sum_tau / n,
            max_transit_time_ps: max_tau,
            mean_diode_rectification_db: sum_rec / n,
            min_diode_rectification_db: min_rec,
            mean_cooperativity: sum_coop / n,
            compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
