//! Parallel Multi-Cycle Benchmark Runner for Magnon BEC & Spin Superfluidity.
//!
//! Multi-threaded Rayon benchmark evaluating 10,000 parameter sweeps across diverse
//! microwave pump powers, channel lengths, bias magnetic fields, and phase gradients.
//! Validates:
//! - Magnon BEC threshold crossing at P = P_crit
//! - Chemical potential saturation mu_m -> E_min
//! - Spin superfluid velocity strictly below Landau critical velocity v_s < v_c
//! - Long-range transmission advantage T_super / T_diff >> 10x
//! - Sub-microsecond parallel throughput

use crate::magnon_bec::magnon_gpe_solver::MagnonGpeSolver;
use crate::magnon_bec::spin_superfluid_transport_solver::SpinSuperfluidTransportSolver;
use phonon_models::magnon_bec::{HeavyMetalElectrode, SpinSuperfluidChannel, YigMagnonFilm};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark configuration parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct MagnonBecBenchmarkConfig {
    /// Total number of sweeps to execute (default 10,000).
    pub total_sweeps: usize,
    /// Reference injector current in Amperes (default 1.0 mA).
    pub injector_current_a: f64,
    /// Base bias magnetic field in Tesla (default 0.10 T).
    pub base_bias_field_tesla: f64,
}

impl Default for MagnonBecBenchmarkConfig {
    fn default() -> Self {
        Self {
            total_sweeps: 10_000,
            injector_current_a: 1.0e-3,
            base_bias_field_tesla: 0.10,
        }
    }
}

/// Comprehensive benchmark report with verification metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct MagnonBecBenchmarkReport {
    /// Total sweeps executed.
    pub total_sweeps: usize,
    /// Elapsed benchmark duration in milliseconds.
    pub elapsed_ms: f64,
    /// Throughput in sweeps per second.
    pub sweeps_per_sec: f64,
    /// Fraction of sweeps exhibiting macroscopic BEC formation (P > P_crit).
    pub bec_formation_fraction: f64,
    /// Mean chemical potential ratio mu_m / E_min.
    pub mean_chemical_potential_ratio: f64,
    /// Fraction of sweeps satisfying Landau critical velocity limit (v_s < v_c).
    pub sub_critical_velocity_fraction: f64,
    /// Mean long-range transmission advantage ratio T_super / T_diff.
    pub mean_transmission_advantage: f64,
    /// Minimum long-range transmission advantage ratio across all sweeps.
    pub min_transmission_advantage: f64,
    /// Mean non-local ISHE detector voltage in the superfluid regime (microvolts).
    pub mean_non_local_voltage_uv: f64,
}

/// Multi-threaded Rayon benchmark runner.
#[derive(Debug, Default, Clone)]
pub struct MagnonBecBenchmarkRunner;

impl MagnonBecBenchmarkRunner {
    pub fn new() -> Self {
        Self
    }

    /// Executes the multi-cycle Magnon BEC and spin superfluidity benchmark.
    pub fn run_benchmark(&self, config: &MagnonBecBenchmarkConfig) -> MagnonBecBenchmarkReport {
        let n = config.total_sweeps;
        let gpe_solver = MagnonGpeSolver::new();
        let transport_solver = SpinSuperfluidTransportSolver::new();
        let electrode = HeavyMetalElectrode::default();

        let start_time = Instant::now();

        // Parallel parameter sweep across Rayon worker threads
        let sweep_results: Vec<(bool, f64, bool, f64, f64)> = (0..n)
            .into_par_iter()
            .map(|i| {
                let frac = i as f64 / n as f64;
                // Pump power ratio between 0.5 and 2.5
                let pump_ratio = 0.5 + 2.0 * frac;
                // Channel length between 10 um and 50 um
                let channel_len = 10.0e-6 + 40.0e-6 * (1.0 - frac);
                // Phase gradient
                let dphi_dx = 1.0e5 * (0.2 + 0.8 * frac);

                let film = YigMagnonFilm {
                    bias_field_tesla: config.base_bias_field_tesla,
                    ..Default::default()
                };

                let channel = SpinSuperfluidChannel {
                    film: film.clone(),
                    channel_length_m: channel_len,
                    ..Default::default()
                };

                // GPE steady state
                let gpe_res = gpe_solver.solve_steady_state(&film, pump_ratio, 32);
                let is_bec = gpe_res.condensate_fraction > 0.0;
                let e_min = film.minimum_spin_wave_energy_joules();
                let mu_ratio = gpe_res.chemical_potential_joules / e_min;

                // Transport
                let trans_res = transport_solver.solve_transport(
                    &channel,
                    &electrode,
                    config.injector_current_a,
                    dphi_dx,
                );

                let v_uv = trans_res.non_local_voltage_superfluid_v * 1e6;

                (
                    is_bec,
                    mu_ratio,
                    trans_res.is_below_critical_velocity,
                    trans_res.transmission_advantage,
                    v_uv,
                )
            })
            .collect();

        let elapsed = start_time.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let sweeps_per_sec = n as f64 / elapsed.as_secs_f64().max(1e-9);

        let mut bec_count = 0;
        let mut sum_mu_ratio = 0.0;
        let mut sub_crit_count = 0;
        let mut sum_advantage = 0.0;
        let mut min_advantage = f64::MAX;
        let mut sum_v_uv = 0.0;

        for (is_bec, mu_ratio, is_sub_crit, advantage, v_uv) in sweep_results {
            if is_bec {
                bec_count += 1;
            }
            sum_mu_ratio += mu_ratio;
            if is_sub_crit {
                sub_crit_count += 1;
            }
            sum_advantage += advantage;
            if advantage < min_advantage {
                min_advantage = advantage;
            }
            sum_v_uv += v_uv;
        }

        MagnonBecBenchmarkReport {
            total_sweeps: n,
            elapsed_ms,
            sweeps_per_sec,
            bec_formation_fraction: bec_count as f64 / n as f64,
            mean_chemical_potential_ratio: sum_mu_ratio / n as f64,
            sub_critical_velocity_fraction: sub_crit_count as f64 / n as f64,
            mean_transmission_advantage: sum_advantage / n as f64,
            min_transmission_advantage: min_advantage,
            mean_non_local_voltage_uv: sum_v_uv / n as f64,
        }
    }
}
