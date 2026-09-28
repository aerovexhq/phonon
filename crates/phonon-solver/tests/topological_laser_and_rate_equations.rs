//! Integration tests for Maxwell-Bloch rate equations, single-mode lasing, and parallel benchmarks.

use phonon_models::non_hermitian::{LaserRateEquationParams, SshLatticeParams};
use phonon_solver::non_hermitian::{MaxwellBlochSolver, TopologicalLaserBenchmarkRunner};

#[test]
fn test_laser_rate_equations_threshold_and_smsr() {
    let laser_params = LaserRateEquationParams::standard_topological_array();
    let lattice_params = SshLatticeParams::standard_topological_laser_lattice();

    let i_th = laser_params.threshold_current_amperes();
    assert!(
        i_th > 1.0e-3 && i_th < 10.0e-3,
        "Threshold current should be few mA, got {} A",
        i_th
    );
    assert!(laser_params.is_above_threshold());

    // Run Maxwell-Bloch dynamic simulation for 200 round-trips
    let sim_res = MaxwellBlochSolver::simulate_dynamics(
        &laser_params,
        &lattice_params,
        200,
        1.0e-12,
        0.0, // Zero disorder
        123,
    );

    assert!(
        sim_res.smsr_db > 35.0,
        "Side-Mode Suppression Ratio must exceed 35 dB, got {} dB",
        sim_res.smsr_db
    );
    assert!(sim_res.edge_mode_power > sim_res.max_side_mode_power * 3000.0);
}

#[test]
fn test_topological_laser_defect_and_disorder_tolerance() {
    let laser_params = LaserRateEquationParams::standard_topological_array();
    let lattice_params = SshLatticeParams::standard_topological_laser_lattice();

    // Clean simulation
    let clean = MaxwellBlochSolver::simulate_dynamics(
        &laser_params,
        &lattice_params,
        150,
        1.0e-12,
        0.0,
        42,
    );

    // Simulation with 10% bandgap disorder
    let disorder_hz = 0.10 * lattice_params.topological_bandgap_hz();
    let disordered = MaxwellBlochSolver::simulate_dynamics(
        &laser_params,
        &lattice_params,
        150,
        1.0e-12,
        disorder_hz,
        42,
    );

    assert!(
        disordered.smsr_db > 35.0,
        "Disordered topological laser must still maintain SMSR > 35 dB, got {} dB",
        disordered.smsr_db
    );

    let degradation = (clean.smsr_db - disordered.smsr_db).abs();
    assert!(
        degradation < 1.5,
        "Topological protection restricts SMSR degradation to < 1.5 dB, got {} dB",
        degradation
    );
}

#[test]
fn test_parallel_10k_topological_laser_benchmark() {
    let report = TopologicalLaserBenchmarkRunner::run_benchmark(10_000);

    assert!(
        report.min_smsr_db > 35.0,
        "Benchmark minimum SMSR must exceed 35 dB, got {} dB",
        report.min_smsr_db
    );

    assert!(
        report.max_edge_to_bulk_ratio > 50.0,
        "Max edge-to-bulk ratio must exceed 50, got {}",
        report.max_edge_to_bulk_ratio
    );

    assert!(
        report.exceptional_point_coalescence_error < 1e-4,
        "EP coalescence error must be < 1e-4, got {}",
        report.exceptional_point_coalescence_error
    );

    assert_eq!(report.total_round_trips_evaluated, 10_000);

    assert!(
        report.throughput_steps_per_sec > 20_000.0,
        "Throughput must exceed 20,000 steps/sec, got {}",
        report.throughput_steps_per_sec
    );
}
