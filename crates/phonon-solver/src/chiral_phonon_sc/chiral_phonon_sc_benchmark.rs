//! Multi-threaded Rayon benchmark runner for chiral phonon-driven superconductivity,
//! dynamic inversion breaking, and parametric Josephson modulators across 10,000 sweeps.

use super::chiral_eliashberg_solver::ChiralEliashbergSolver;
use super::parametric_josephson_solver::ParametricJosephsonSolver;
use phonon_models::chiral_phonon_sc::{
    ChiralPhononDriveParams, DynamicJosephsonParams, TransientPairingParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point in chiral phonon superconductivity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralPhononScSweepPoint {
    pub pairing_enhancement_percent: f64,
    pub parametric_gain_db: f64,
    pub modulation_contrast_db: f64,
    pub switching_time_ps: f64,
    pub induced_moment_bohr: f64,
}

/// Comprehensive benchmark report for chiral phonon-driven superconductivity.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralPhononScBenchmarkReport {
    /// Total sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean pairing enhancement percentage ($> 50.0\\%$ required).
    pub mean_pairing_enhancement_percent: f64,
    /// Minimum pairing enhancement percentage.
    pub min_pairing_enhancement_percent: f64,
    /// Mean parametric amplification gain in decibels ($\\ge 15.0\\text{ dB}$ required).
    pub mean_parametric_gain_db: f64,
    /// Minimum parametric amplification gain in decibels.
    pub min_parametric_gain_db: f64,
    /// Mean dynamic modulation contrast in decibels ($\\ge 20.0\\text{ dB}$ required).
    pub mean_modulation_contrast_db: f64,
    /// Minimum dynamic modulation contrast in decibels.
    pub min_modulation_contrast_db: f64,
    /// Mean modulation switching time in picoseconds ($\\le 0.5\\text{ ps}$ required).
    pub mean_switching_time_ps: f64,
    /// Maximum modulation switching time in picoseconds.
    pub max_switching_time_ps: f64,
    /// Fraction of parameter sweeps satisfying all physical criteria.
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for chiral phonon superconductivity.
#[derive(Debug, Clone)]
pub struct ChiralPhononScBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for ChiralPhononScBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl ChiralPhononScBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> ChiralPhononScBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<ChiralPhononScSweepPoint> = (0..total)
            .into_par_iter()
            .map(|i| {
                let u = i as f64 / total.max(1) as f64;

                // Parameter sweeping across physical ranges:
                // Amplitude Q0/Qzpf in [1.8, 3.2]
                let q0 = 1.8 + 1.4 * u;
                // Phonon frequency in [16.0, 24.0] THz
                let freq_thz = 16.0 + 8.0 * ((i * 7) % total) as f64 / total as f64;
                // Nonlinear coupling g12 in [12.0, 25.0] meV
                let g12 = 12.0 + 13.0 * ((i * 13) % total) as f64 / total as f64;
                // Modulation depth mu in [0.45, 0.65]
                let mu_mod = 0.45 + 0.20 * ((i * 17) % total) as f64 / total as f64;
                // Critical current Ic0 in [5.0, 25.0] uA
                let ic0 = (5.0 + 20.0 * ((i * 19) % total) as f64 / total as f64) * 1.0e-6;
                // Capacitance Cj in [5.0, 15.0] fF
                let cj = (5.0 + 10.0 * ((i * 23) % total) as f64 / total as f64) * 1.0e-15;
                // Pulse duration in [0.25, 0.50] ps
                let pulse_ps = 0.25 + 0.25 * ((i * 29) % total) as f64 / total as f64;

                let drive_params = ChiralPhononDriveParams {
                    phonon_frequency_thz: freq_thz,
                    normalized_amplitude_q0: q0,
                    nonlinear_coupling_g12_mev: g12,
                    effective_born_charge: 3.2,
                    pulse_duration_ps: pulse_ps,
                };

                let pairing_params = TransientPairingParams {
                    equilibrium_gap_mev: 15.0,
                    equilibrium_lambda: 0.45,
                    pairing_sensitivity: 0.18,
                    fermi_velocity_m_s: 2.0e5,
                };

                let josephson_params = DynamicJosephsonParams {
                    equilibrium_critical_current_a: ic0,
                    junction_capacitance_f: cj,
                    modulation_depth: mu_mod,
                };

                let eliashberg_solver = ChiralEliashbergSolver::new(drive_params, pairing_params);
                let pairing_metrics = eliashberg_solver.solve_transient_pairing();
                let drive_metrics = eliashberg_solver.solve_chiral_drive();

                let josephson_solver =
                    ParametricJosephsonSolver::new(josephson_params, pairing_params, drive_params);
                let josephson_metrics = josephson_solver.solve_dynamic_josephson();

                ChiralPhononScSweepPoint {
                    pairing_enhancement_percent: pairing_metrics.pairing_enhancement_fraction
                        * 100.0,
                    parametric_gain_db: josephson_metrics.parametric_gain_db,
                    modulation_contrast_db: josephson_metrics.modulation_contrast_db,
                    switching_time_ps: josephson_metrics.modulation_time_ps,
                    induced_moment_bohr: drive_metrics.induced_magnetic_moment_bohr,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_enhancement = 0.0;
        let mut min_enhancement = f64::INFINITY;
        let mut sum_gain = 0.0;
        let mut min_gain = f64::INFINITY;
        let mut sum_contrast = 0.0;
        let mut min_contrast = f64::INFINITY;
        let mut sum_time = 0.0;
        let mut max_time = f64::NEG_INFINITY;
        let mut compliant_count = 0usize;

        for p in &results {
            sum_enhancement += p.pairing_enhancement_percent;
            if p.pairing_enhancement_percent < min_enhancement {
                min_enhancement = p.pairing_enhancement_percent;
            }

            sum_gain += p.parametric_gain_db;
            if p.parametric_gain_db < min_gain {
                min_gain = p.parametric_gain_db;
            }

            sum_contrast += p.modulation_contrast_db;
            if p.modulation_contrast_db < min_contrast {
                min_contrast = p.modulation_contrast_db;
            }

            sum_time += p.switching_time_ps;
            if p.switching_time_ps > max_time {
                max_time = p.switching_time_ps;
            }

            // Criteria:
            // 1. pairing_enhancement > 50.0%
            // 2. parametric_gain >= 15.0 dB
            // 3. modulation_contrast >= 20.0 dB
            // 4. switching_time <= 0.50 ps
            if p.pairing_enhancement_percent > 50.0
                && p.parametric_gain_db >= 15.0
                && p.modulation_contrast_db >= 20.0
                && p.switching_time_ps <= 0.50
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        ChiralPhononScBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_pairing_enhancement_percent: sum_enhancement / n,
            min_pairing_enhancement_percent: min_enhancement,
            mean_parametric_gain_db: sum_gain / n,
            min_parametric_gain_db: min_gain,
            mean_modulation_contrast_db: sum_contrast / n,
            min_modulation_contrast_db: min_contrast,
            mean_switching_time_ps: sum_time / n,
            max_switching_time_ps: max_time,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
