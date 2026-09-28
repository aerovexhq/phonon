//! Integration tests for Langevin SDE trajectories and parallel Rayon benchmarks.

use phonon_models::optomechanics::SidebandCoolingParams;
use phonon_solver::optomechanics::{CavityOptomechanicsBenchmarkRunner, LangevinSdeSolver};

#[test]
fn test_langevin_sde_trajectory_cooling() {
    let cooling = SidebandCoolingParams::standard_ground_state_nanobeam();
    let dt = 1.0 / (20.0 * cooling.system.mechanical_frequency_hz);

    let traj = LangevinSdeSolver::simulate_trajectory(&cooling, 500, dt, 42);

    assert!(traj.displacement_variance_m2 > 0.0);
    assert!(traj.momentum_variance_kg2_m2_s2 > 0.0);
    assert!(
        traj.simulated_phonon_occupancy < 1.0,
        "Simulated phonon occupancy should be in sub-single-phonon regime, got {}",
        traj.simulated_phonon_occupancy
    );
    assert_eq!(traj.steps_integrated, 500);
}

#[test]
fn test_parallel_10k_optomechanics_benchmark() {
    let report = CavityOptomechanicsBenchmarkRunner::run_benchmark(10_000);

    assert!(
        report.min_phonon_occupancy < 0.1,
        "Benchmark minimum phonon occupancy must reach ground state (< 0.1), got {}",
        report.min_phonon_occupancy
    );

    assert!(
        report.max_cooperativity > 100.0,
        "Max cooperativity must exceed 100, got {}",
        report.max_cooperativity
    );

    assert!(
        report.max_squeezing_db > 3.0,
        "Max squeezing must exceed 3.0 dB, got {} dB",
        report.max_squeezing_db
    );

    assert!(
        report.omit_contrast > 0.5,
        "OMIT contrast must exceed 0.5, got {}",
        report.omit_contrast
    );

    assert_eq!(report.total_trajectories_evaluated, 10_000);

    assert!(
        report.throughput_trajectories_per_sec > 20_000.0,
        "Throughput must exceed 20,000 trajectories/sec, got {}",
        report.throughput_trajectories_per_sec
    );
}
