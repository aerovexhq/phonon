//! Multi-threaded Rayon benchmark for molecular spintronics, SMM quantum tunneling,
//! and spin-torque nano-oscillators.

use crate::stno::giant_spin_solver::GiantSpinSolver;
use crate::stno::negf_solver::NegfMolecularSolver;
use crate::stno::stochastic_llg_solver::{MacrospinState, StochasticLlgSolver};
use phonon_models::stno::{
    GiantSpinParams, InjectionLockingParams, NegfMolecularJunctionParams, StnoParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark performance and physics report for Phase 51.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StnoBenchmarkReport {
    /// SMM Fe8 ground state QTM tunnel splitting in Kelvin.
    pub fe8_qtm_splitting_k: f64,
    /// Molecular junction zero-bias Kondo conductance G / G_0.
    pub kondo_zero_bias_conductance: f64,
    /// Kondo Zeeman splitting at 2 Tesla in meV.
    pub kondo_zeeman_splitting_mev: f64,
    /// STNO oscillation frequency in GHz.
    pub stno_frequency_ghz: f64,
    /// Injection locking verification (true if Adler phase difference is bounded).
    pub injection_locking_verified: bool,
    /// Total precession cycles simulated.
    pub total_precession_cycles: usize,
    /// Overall simulation throughput in integration steps per second.
    pub throughput_steps_per_sec: f64,
    /// Elapsed wall-clock time in seconds.
    pub elapsed_seconds: f64,
}

/// Runner for high-throughput Phase 51 benchmarks.
pub struct StnoBenchmarkRunner;

impl StnoBenchmarkRunner {
    /// Executes the full Phase 51 benchmark suite across parallel Rayon threads.
    pub fn run_benchmark(oscillator_count: usize, cycles: usize) -> StnoBenchmarkReport {
        let start = Instant::now();

        // 1. SMM Giant Spin QTM eigensolver verification
        let fe8 = GiantSpinParams::fe8_molecular_magnet();
        let fe8_sol = GiantSpinSolver::solve(&fe8, 0.0, 0.0);

        // 2. NEGF molecular junction with Kondo splitting verification
        let adatom = NegfMolecularJunctionParams::standard_magnetic_adatom();
        let negf_sol = NegfMolecularSolver::solve(&adatom, 0.001, 2.0);

        // 3. STNO auto-oscillator and injection locking benchmark
        let stno_params = StnoParams::standard_cofeb_stno();
        let h_ext = [0.0, 0.0, 1.6e5]; // 0.2 Tesla external field
        let threshold_i = stno_params.threshold_current_amperes(h_ext[2]);
        let drive_current = 1.5 * threshold_i; // Above threshold -> sustained limit cycle

        let f_fmr =
            stno_params.kittel_fmr_frequency_rad_per_s(h_ext[2]) / (2.0 * std::f64::consts::PI);
        let dt = 1.0 / (f_fmr * 50.0); // 50 points per precession cycle

        let steps_per_osc = cycles * 50;
        let total_steps = oscillator_count * steps_per_osc;

        let results: Vec<(f64, bool)> = (0..oscillator_count)
            .into_par_iter()
            .map(|id| {
                let mut state = MacrospinState::new([0.1, 0.0, 0.995]);
                let mut seed = 0x5EED_0000_0000_0000_u64.wrapping_add(id as u64);
                let pol_dir = [0.0, 0.0, 1.0];

                let mut zero_crossings = 0;
                let mut prev_mx = state.m[0];

                for _ in 0..steps_per_osc {
                    StochasticLlgSolver::step(
                        &mut state,
                        &stno_params,
                        h_ext,
                        drive_current,
                        pol_dir,
                        300.0,
                        dt,
                        &mut seed,
                    );

                    if prev_mx <= 0.0 && state.m[0] > 0.0 {
                        zero_crossings += 1;
                    }
                    prev_mx = state.m[0];
                }

                let total_time = steps_per_osc as f64 * dt;
                let freq_ghz = (zero_crossings as f64 / total_time) * 1.0e-9;

                // Check injection locking
                let locking = InjectionLockingParams::new(
                    freq_ghz * 1.0e9,
                    freq_ghz * 1.0e9 + 10.0e6,
                    1.0e-6,
                    10.0e-6,
                    50.0,
                );

                (freq_ghz, locking.is_locked())
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64().max(1e-9);
        let throughput = total_steps as f64 / elapsed;

        let avg_freq = results.iter().map(|r| r.0).sum::<f64>() / oscillator_count as f64;
        let all_locked = results.iter().all(|r| r.1);

        StnoBenchmarkReport {
            fe8_qtm_splitting_k: fe8_sol.ground_state_splitting_k,
            kondo_zero_bias_conductance: negf_sol.zero_bias_conductance_2e2_over_h,
            kondo_zeeman_splitting_mev: negf_sol.zeeman_splitting_ev * 1.0e3,
            stno_frequency_ghz: avg_freq,
            injection_locking_verified: all_locked,
            total_precession_cycles: cycles * oscillator_count,
            throughput_steps_per_sec: throughput,
            elapsed_seconds: elapsed,
        }
    }
}
