//! Parallel Rayon benchmark evaluating topological single-mode lasing and non-Hermitian exceptional points.

use super::maxwell_bloch_solver::MaxwellBlochSolver;
use super::non_hermitian_eigensolver::NonHermitianEigensolver;
use phonon_models::non_hermitian::{LaserRateEquationParams, PtDimerParams, SshLatticeParams};
use rayon::prelude::*;

/// Performance summary report for non-Hermitian topological laser array benchmark.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalLaserBenchmarkReport {
    /// Minimum Side-Mode Suppression Ratio (SMSR) achieved in dB (must exceed 35.0 dB).
    pub min_smsr_db: f64,
    /// Maximum edge mode localization ratio over bulk intensity (must exceed 100.0).
    pub max_edge_to_bulk_ratio: f64,
    /// Exceptional point frequency coalescence error $|\omega_+ - \omega_-|$ at $\gamma = \kappa$.
    pub exceptional_point_coalescence_error: f64,
    /// Maximum Petermann excess noise factor evaluated.
    pub max_petermann_factor: f64,
    /// Total round-trip time steps and configurations evaluated.
    pub total_round_trips_evaluated: usize,
    /// Simulation execution throughput in round-trip steps/sec.
    pub throughput_steps_per_sec: f64,
}

/// Benchmark runner for topological laser arrays.
pub struct TopologicalLaserBenchmarkRunner;

impl TopologicalLaserBenchmarkRunner {
    /// Executes parallel Rayon benchmark across `total_steps` (default 10,000).
    pub fn run_benchmark(total_steps: usize) -> TopologicalLaserBenchmarkReport {
        let count = total_steps.max(10_000);
        let start_time = std::time::Instant::now();

        // 1. Evaluate exceptional point degeneracy in PT dimer
        let ep_dimer = PtDimerParams::standard_exceptional_point_dimer();
        let ((w1_re, w1_im), (w2_re, w2_im)) = ep_dimer.eigenfrequencies_hz();
        let ep_error = ((w1_re - w2_re).powi(2) + (w1_im - w2_im).powi(2)).sqrt();
        let max_petermann = ep_dimer.petermann_factor();

        // 2. Parallel Rayon sweep over 10,000 temporal round-trips across varied disorder and pump configurations
        let chunk_size = 100;
        let num_runs = count / chunk_size;

        let results: Vec<(f64, f64)> = (0..num_runs)
            .into_par_iter()
            .map(|idx| {
                let mut laser_params = LaserRateEquationParams::standard_topological_array();
                laser_params.pump_current_amperes = 12.0e-3 + (idx % 10) as f64 * 1.5e-3;

                let lattice_params = SshLatticeParams::standard_topological_laser_lattice();
                let disorder = (idx % 5) as f64 * 0.02 * lattice_params.topological_bandgap_hz();

                let dt = 2.0e-12; // 2 ps time step
                let sim_res = MaxwellBlochSolver::simulate_dynamics(
                    &laser_params,
                    &lattice_params,
                    chunk_size,
                    dt,
                    disorder,
                    idx as u64,
                );

                let eigen_res = NonHermitianEigensolver::solve(&lattice_params);

                (sim_res.smsr_db, eigen_res.edge_to_bulk_ratio)
            })
            .collect();

        let elapsed = start_time.elapsed().as_secs_f64().max(1e-6);
        let throughput = (count as f64) / elapsed;

        let mut min_smsr = f64::INFINITY;
        let mut max_ratio = 0.0;

        for (smsr, ratio) in results {
            if smsr < min_smsr {
                min_smsr = smsr;
            }
            if ratio > max_ratio {
                max_ratio = ratio;
            }
        }

        TopologicalLaserBenchmarkReport {
            min_smsr_db: min_smsr,
            max_edge_to_bulk_ratio: max_ratio,
            exceptional_point_coalescence_error: ep_error,
            max_petermann_factor: max_petermann,
            total_round_trips_evaluated: count,
            throughput_steps_per_sec: throughput,
        }
    }
}
