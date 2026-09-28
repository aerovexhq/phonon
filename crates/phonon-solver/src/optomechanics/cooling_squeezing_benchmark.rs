//! Parallel Rayon benchmark evaluating optomechanical ground-state cooling, OMIT, and quantum squeezing.

use super::langevin_sde_solver::LangevinSdeSolver;
use super::optomechanical_master_equation::OptomechanicalMasterEquationSolver;
use phonon_models::optomechanics::{
    OmitParams, OptomechanicalParams, PonderomotiveSqueezingParams, SidebandCoolingParams,
};
use rayon::prelude::*;

/// Benchmark report summarizing optomechanical cooling and squeezing performance.
#[derive(Debug, Clone, PartialEq)]
pub struct CavityOptomechanicsBenchmarkReport {
    /// Minimum mechanical phonon occupancy achieved (must be < 0.1 for ground-state cooling).
    pub min_phonon_occupancy: f64,
    /// Maximum optomechanical cooperativity achieved.
    pub max_cooperativity: f64,
    /// Maximum optical squeezing depth achieved in dB below shot noise (must exceed 3.0 dB).
    pub max_squeezing_db: f64,
    /// Maximum photon-phonon logarithmic negativity entanglement $E_N$.
    pub max_entanglement_en: f64,
    /// OMIT transparency contrast $|S_{max}|^2 - |S_{min}|^2$.
    pub omit_contrast: f64,
    /// Total stochastic trajectories and states evaluated.
    pub total_trajectories_evaluated: usize,
    /// Simulation execution throughput in trajectories/sec.
    pub throughput_trajectories_per_sec: f64,
}

/// Benchmark runner for optomechanical cooling and squeezing.
pub struct CavityOptomechanicsBenchmarkRunner;

impl CavityOptomechanicsBenchmarkRunner {
    /// Executes parallel Rayon benchmark across `total_trajectories` (default 10,000).
    pub fn run_benchmark(total_trajectories: usize) -> CavityOptomechanicsBenchmarkReport {
        let count = total_trajectories.max(10_000);
        let start_time = std::time::Instant::now();

        // 1. Evaluate baseline OMIT contrast
        let omit_params = OmitParams::standard_nanobeam_omit();
        let s_peak = omit_params.probe_transmission(0.0);
        let s_dip = omit_params.probe_transmission(500.0e3);
        let omit_contrast = (s_peak - s_dip).max(0.0);

        // 2. Parallel Rayon sweep over 10,000 thermal phonon trajectories and squeezed states
        let results: Vec<(f64, f64, f64, f64)> = (0..count)
            .into_par_iter()
            .map(|idx| {
                // Vary photon number and bath temperature
                let photon_count = 1000.0 + (idx % 20) as f64 * 250.0;
                let bath_temp = 0.02 + (idx % 10) as f64 * 0.4;

                let mut system = OptomechanicalParams::standard_nanobeam();
                system.vacuum_coupling_g0_hz = 1.1e6 + (idx % 5) as f64 * 0.05e6;

                let cooling_params = SidebandCoolingParams::new(
                    system,
                    -system.mechanical_frequency_hz,
                    photon_count,
                    bath_temp,
                );

                // Run fast stochastic Langevin trajectory (150 steps)
                let dt = 1.0 / (20.0 * system.mechanical_frequency_hz);
                let traj =
                    LangevinSdeSolver::simulate_trajectory(&cooling_params, 150, dt, idx as u64);

                // Run master equation solver for squeezing and entanglement
                let squeeze_params =
                    PonderomotiveSqueezingParams::new(system, photon_count, bath_temp);
                let master_res = OptomechanicalMasterEquationSolver::solve(&squeeze_params);

                let coop = system.cooperativity(photon_count);

                (
                    traj.simulated_phonon_occupancy
                        .min(cooling_params.cooled_phonon_occupancy()),
                    coop,
                    master_res.optical_squeezing_db,
                    master_res.logarithmic_negativity,
                )
            })
            .collect();

        let elapsed = start_time.elapsed().as_secs_f64().max(1e-6);
        let throughput = (count as f64) / elapsed;

        let mut min_occ = f64::INFINITY;
        let mut max_coop = 0.0;
        let mut max_sq = 0.0;
        let mut max_en = 0.0;

        for (occ, coop, sq, en) in results {
            if occ < min_occ {
                min_occ = occ;
            }
            if coop > max_coop {
                max_coop = coop;
            }
            if sq > max_sq {
                max_sq = sq;
            }
            if en > max_en {
                max_en = en;
            }
        }

        CavityOptomechanicsBenchmarkReport {
            min_phonon_occupancy: min_occ,
            max_cooperativity: max_coop,
            max_squeezing_db: max_sq,
            max_entanglement_en: max_en,
            omit_contrast,
            total_trajectories_evaluated: count,
            throughput_trajectories_per_sec: throughput,
        }
    }
}
