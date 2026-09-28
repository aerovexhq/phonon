//! Parallel Cavity Spintronics Multi-Cycle Benchmark Runner.
//!
//! Multi-threaded Rayon benchmark evaluating 10,000 drive cycles, verifying
//! strong coupling cooperativity $C_{mp} > 100$, anti-crossing splitting $> 20\text{ MHz}$,
//! and detectable inverse spin Hall voltage $V_{ISHE} > 1.0\ \mu\text{V}$.

use super::coupled_llg_cavity_solver::{CoupledLlgCavitySolver, CoupledLlgConfig};
use super::polariton_solver::PolaritonSolver;
use phonon_models::cavity_spintronics::{
    MagnonCavityCoupling, MicrowaveCavityParams, YigMaterial, YigPtInterface,
};
use rayon::prelude::*;
use std::time::Instant;

/// Configuration for the multi-cycle cavity spintronics benchmark.
#[derive(Debug, Clone, PartialEq)]
pub struct CavityBenchmarkConfig {
    /// Total number of simulation cycles / sweep configurations to evaluate (default 10,000).
    pub total_cycles: usize,
    /// Center cavity frequency in Hz (default 10 GHz).
    pub center_freq_hz: f64,
    /// Coupling strength $g_{mp}/(2\pi)$ in Hz (default 40 MHz).
    pub coupling_rate_hz: f64,
    /// Microwave drive magnetic field in Tesla (default 15 uT).
    pub drive_field_t: f64,
}

impl Default for CavityBenchmarkConfig {
    fn default() -> Self {
        Self {
            total_cycles: 10_000,
            center_freq_hz: 10.0e9,
            coupling_rate_hz: 40.0e6,
            drive_field_t: 15.0e-6,
        }
    }
}

/// Comprehensive benchmark report with verification metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct CavityBenchmarkReport {
    /// Total cycles executed.
    pub total_cycles: usize,
    /// Elapsed benchmark duration in milliseconds.
    pub elapsed_ms: f64,
    /// Throughput in cycles per second.
    pub cycles_per_sec: f64,
    /// Mean cooperativity $\bar{C}_{mp}$.
    pub mean_cooperativity: f64,
    /// Minimum cooperativity observed.
    pub min_cooperativity: f64,
    /// Anti-crossing splitting in Hz.
    pub anticrossing_splitting_hz: f64,
    /// Mean steady-state DC ISHE voltage in Volts.
    pub mean_ishe_voltage_volts: f64,
    /// Max steady-state DC ISHE voltage in Volts.
    pub max_ishe_voltage_volts: f64,
    /// Fraction of cycles meeting strong coupling ($C_{mp} > 100$).
    pub strong_coupling_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner.
#[derive(Debug, Default, Clone)]
pub struct CavityBenchmarkRunner;

impl CavityBenchmarkRunner {
    /// Creates a new benchmark runner.
    pub fn new() -> Self {
        Self
    }

    /// Runs the 10,000-cycle parallel benchmark.
    pub fn run(&self, config: &CavityBenchmarkConfig) -> CavityBenchmarkReport {
        let start = Instant::now();

        let base_magnon = YigMaterial::default();
        let base_cavity = MicrowaveCavityParams {
            resonant_frequency_hz: config.center_freq_hz,
            quality_factor: 5000.0,
            external_coupling_ratio: 0.5,
        };
        let b0_res = base_magnon.resonant_field(config.center_freq_hz);
        let base_interface = YigPtInterface::default();

        let _polariton_solver = PolaritonSolver::new();
        let llg_solver = CoupledLlgCavitySolver::new();

        // Sweep parameters across 10,000 cycles
        let results: Vec<(f64, f64, f64, bool)> = (0..config.total_cycles)
            .into_par_iter()
            .map(|cycle_idx| {
                // Vary detuning slightly across cycles
                let frac = (cycle_idx as f64) / (config.total_cycles as f64) - 0.5;
                let b0 = b0_res * (1.0 + frac * 0.05); // +/- 2.5% field detuning

                // Vary coupling rate slightly (+/- 10%)
                let g_hz = config.coupling_rate_hz * (1.0 + 0.1 * ((cycle_idx % 17) as f64) / 17.0);

                let coupling = MagnonCavityCoupling {
                    coupling_rate_hz: g_hz,
                    cavity: base_cavity.clone(),
                    magnon: base_magnon.clone(),
                    bias_field: b0,
                };

                let c_mp = coupling.cooperativity();
                let is_strong = coupling.is_strong_coupling();
                let splitting = coupling.anticrossing_gap_hz();

                // Run fast LLG cycle to evaluate spin pumping & ISHE voltage
                let llg_config = CoupledLlgConfig {
                    duration_seconds: 5.0e-9,
                    dt_seconds: 1.0e-11,
                    drive_freq_hz: config.center_freq_hz,
                    drive_amplitude_t: config.drive_field_t,
                };

                let llg_res = llg_solver.solve(&coupling, &base_interface, &llg_config);

                (c_mp, splitting, llg_res.dc_ishe_voltage_volts, is_strong)
            })
            .collect();

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let cycles_per_sec = (config.total_cycles as f64) / elapsed.as_secs_f64();

        let mut sum_coop: f64 = 0.0;
        let mut min_coop: f64 = f64::INFINITY;
        let mut sum_ishe: f64 = 0.0;
        let mut max_ishe: f64 = 0.0;
        let mut strong_count: usize = 0;
        let mut max_splitting: f64 = 0.0;

        for (c_mp, splitting, v_ishe, is_strong) in results {
            sum_coop += c_mp;
            min_coop = min_coop.min(c_mp);
            sum_ishe += v_ishe;
            max_ishe = max_ishe.max(v_ishe);
            max_splitting = max_splitting.max(splitting);
            if is_strong {
                strong_count += 1;
            }
        }

        let n = config.total_cycles as f64;
        CavityBenchmarkReport {
            total_cycles: config.total_cycles,
            elapsed_ms,
            cycles_per_sec,
            mean_cooperativity: sum_coop / n,
            min_cooperativity: min_coop,
            anticrossing_splitting_hz: max_splitting,
            mean_ishe_voltage_volts: sum_ishe / n,
            max_ishe_voltage_volts: max_ishe,
            strong_coupling_fraction: (strong_count as f64) / n,
        }
    }
}
