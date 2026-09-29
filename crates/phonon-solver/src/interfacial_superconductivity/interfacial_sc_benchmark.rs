//! Multi-threaded Rayon benchmark runner for interfacial high-Tc superconductivity,
//! nematic fluctuations, and Josephson diode arrays across 10,000 parameter sweeps.

use super::interfacial_bdg_solver::InterfacialBdgSolver;
use super::josephson_diode_array_solver::JosephsonDiodeArraySolver;
use phonon_models::interfacial_superconductivity::{
    InterfacialScParams, JosephsonDiodeParams, NematicOrderParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point in interfacial superconductivity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterfacialScSweepPoint {
    pub critical_temperature_k: f64,
    pub bcs_strong_coupling_ratio: f64,
    pub diode_efficiency: f64,
    pub rectification_ratio_db: f64,
    pub gap_anisotropy_ratio: f64,
}

/// Comprehensive benchmark report for interfacial superconductivity.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfacialScBenchmarkReport {
    /// Total number of sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean enhanced critical temperature $T_c$ in Kelvin ($> 65\text{ K}$ required).
    pub mean_critical_temp_k: f64,
    /// Minimum enhanced critical temperature $T_c$ in Kelvin.
    pub min_critical_temp_k: f64,
    /// Mean non-reciprocal diode efficiency $\eta_{\mathrm{diode}}$ ($\ge 0.20$ required).
    pub mean_diode_efficiency: f64,
    /// Minimum non-reciprocal diode efficiency.
    pub min_diode_efficiency: f64,
    /// Mean non-reciprocal rectification ratio in decibels ($\ge 3.0\text{ dB}$ required).
    pub mean_rectification_ratio_db: f64,
    /// Minimum non-reciprocal rectification ratio in decibels.
    pub min_rectification_ratio_db: f64,
    /// Mean BCS strong-coupling ratio $2\Delta_0 / (k_B T_c)$ ($\ge 3.8$ required).
    pub mean_bcs_ratio: f64,
    /// Fraction of parameter sweeps complying with $T_c > 65\text{ K}$, $\eta_{\mathrm{diode}} \ge 0.20$, and $\mathcal{R}_{\mathrm{diode}} \ge 3.0\text{ dB}$.
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for interfacial high-Tc superconductivity.
#[derive(Debug, Clone)]
pub struct InterfacialScBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for InterfacialScBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl InterfacialScBenchmarkRunner {
    /// Creates a benchmark runner with specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes parallel Rayon benchmark across all parameter sweeps.
    pub fn run_parallel_benchmark(&self) -> InterfacialScBenchmarkReport {
        let start = Instant::now();

        let sweep_results: Vec<InterfacialScSweepPoint> = (0..self.total_cycles)
            .into_par_iter()
            .map(|idx| {
                // Vary optical phonon energy between 90.0 and 105.0 meV
                let omega_0 = 90.0 + (idx as f64 % 16.0) * 1.0;
                // Vary coupling lambda between 0.48 and 0.68
                let lambda = 0.48 + (idx as f64 % 21.0) * 0.01;
                // Vary second harmonic ratio between 0.30 and 0.45
                let r2 = 0.30 + (idx as f64 % 16.0) * 0.01;
                // Vary nematic order between 0.20 and 0.60
                let nematic = 0.20 + (idx as f64 % 21.0) * 0.02;

                let sc_params = InterfacialScParams::new(omega_0, 0.15, lambda, 8.0);
                let nem_params = NematicOrderParams::new(nematic, 70.0, 0.35);

                let bdg_solver = InterfacialBdgSolver::new(sc_params, nem_params);
                let tc = bdg_solver.solve_critical_temperature();
                let bcs_ratio = sc_params.strong_coupling_ratio();
                let gap_anisotropy = nem_params.gap_anisotropy_ratio();

                let diode_params = JosephsonDiodeParams::new(1.0e-5, r2, 0.1, 4);
                let diode_solver = JosephsonDiodeArraySolver::new(diode_params);
                let metrics = diode_solver.solve_diode_metrics();

                InterfacialScSweepPoint {
                    critical_temperature_k: tc,
                    bcs_strong_coupling_ratio: bcs_ratio,
                    diode_efficiency: metrics.diode_efficiency,
                    rectification_ratio_db: metrics.rectification_ratio_db,
                    gap_anisotropy_ratio: gap_anisotropy,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (self.total_cycles as f64) / elapsed.max(1e-6);

        let mut sum_tc = 0.0;
        let mut min_tc = f64::MAX;
        let mut sum_eta = 0.0;
        let mut min_eta = f64::MAX;
        let mut sum_rec = 0.0;
        let mut min_rec = f64::MAX;
        let mut sum_bcs = 0.0;
        let mut compliant_count = 0;

        for pt in &sweep_results {
            sum_tc += pt.critical_temperature_k;
            if pt.critical_temperature_k < min_tc {
                min_tc = pt.critical_temperature_k;
            }

            sum_eta += pt.diode_efficiency;
            if pt.diode_efficiency < min_eta {
                min_eta = pt.diode_efficiency;
            }

            sum_rec += pt.rectification_ratio_db;
            if pt.rectification_ratio_db < min_rec {
                min_rec = pt.rectification_ratio_db;
            }

            sum_bcs += pt.bcs_strong_coupling_ratio;

            if pt.critical_temperature_k > 65.0
                && pt.diode_efficiency >= 0.20
                && pt.rectification_ratio_db >= 3.0
                && pt.bcs_strong_coupling_ratio >= 3.8
            {
                compliant_count += 1;
            }
        }

        let n = self.total_cycles as f64;
        let mean_tc = sum_tc / n;
        let mean_eta = sum_eta / n;
        let mean_rec = sum_rec / n;
        let mean_bcs = sum_bcs / n;
        let compliance_fraction = (compliant_count as f64) / n;

        InterfacialScBenchmarkReport {
            total_cycles: self.total_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_critical_temp_k: mean_tc,
            min_critical_temp_k: min_tc,
            mean_diode_efficiency: mean_eta,
            min_diode_efficiency: min_eta,
            mean_rectification_ratio_db: mean_rec,
            min_rectification_ratio_db: min_rec,
            mean_bcs_ratio: mean_bcs,
            compliance_fraction,
        }
    }
}
