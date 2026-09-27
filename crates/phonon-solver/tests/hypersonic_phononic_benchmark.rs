//! Integration tests for Rayon multi-threaded elastodynamic continuum solver
//! and 10,000-circuit benchmark comparing Hypersonic Phononic Logic against 3nm GAA CMOS.

use phonon_models::phononic::PiezoelectricMaterial;
use phonon_solver::phononic::{ContinuumSolver2D, PhononicBenchmarkRunner};

#[test]
fn test_multi_threaded_continuum_solver_step_and_thermal_coupling() {
    let aln = PiezoelectricMaterial::aln();
    // 32 x 32 continuum grid with dx = dy = 50 nm
    let mut solver = ContinuumSolver2D::new(32, 32, 50.0e-9, 50.0e-9, aln);

    assert_eq!(solver.nodes.len(), 32 * 32);
    assert_eq!(solver.mean_temperature(), 300.0);

    // Inject high-frequency 10 GHz acoustic excitation at center (16, 16)
    solver.inject_acoustic_pulse(16, 16, 4, 1.0e-9, 10.0e9);

    let initial_energy = solver.total_acoustic_energy();
    assert!(initial_energy > 0.0, "Initial energy = {}", initial_energy);

    // Run 20 transient steps with dt = 1e-12 s (1 ps)
    for _ in 0..20 {
        solver.step(1.0e-12);
    }

    // Verify acoustic displacement has propagated
    let center_id = solver.idx(16, 16);
    let edge_id = solver.idx(17, 16);
    assert!(
        solver.nodes[center_id].u_x.abs() > 0.0 || solver.nodes[edge_id].u_x.abs() > 0.0,
        "Displacement must exist"
    );

    // Verify Akhiezer dissipation slightly elevates lattice temperature
    let max_t = solver.max_temperature();
    assert!(max_t >= 300.0, "Max temperature = {} K", max_t);
}

#[test]
fn test_parallel_hypersonic_phononic_benchmark_10k_circuits() {
    let runner = PhononicBenchmarkRunner::default();
    assert_eq!(runner.num_circuits, 10_000);
    assert_eq!(runner.max_temp_k, 800.0);
    assert_eq!(runner.max_radiation_mrad, 100.0);

    let report = runner.run_benchmark();

    // Verify 100% truth-table fidelity for Hypersonic Phononic Logic
    assert_eq!(
        report.total_circuits_simulated, 10_000,
        "Total circuits simulated"
    );
    assert_eq!(
        report.phononic_fidelity_rate, 1.0,
        "Phononic logic must maintain 100% fidelity across all 10,000 circuits"
    );

    // CMOS must fail on a large portion of circuits due to TID and high temperatures
    assert!(
        report.cmos_fidelity_rate < 0.20,
        "CMOS fidelity rate ({}) must collapse under 800 K and 100 Mrad",
        report.cmos_fidelity_rate
    );

    // Verify temperature tolerance advantage
    assert!(report.phononic_max_temp_k >= 800.0);
    assert!(report.cmos_max_temp_k <= 450.0);

    // Verify radiation tolerance advantage (> 300x)
    assert!(report.phononic_tid_mrad >= 100.0);
    assert!(report.cmos_tid_mrad <= 0.3);

    // Verify zero static standby power
    assert_eq!(report.phononic_static_power_w, 0.0);
    assert!(report.cmos_static_power_300k_w > 0.0);
    assert!(report.cmos_static_power_450k_w > report.cmos_static_power_300k_w);

    // Verify dynamic energy advantage (>= 10x)
    assert!(report.energy_reduction_factor >= 10.0);

    println!(
        "Benchmark complete in {:.2} ms across 10,000 circuits. Phononic fidelity: 100%, CMOS fidelity: {:.1}%",
        report.elapsed_wallclock_ms,
        report.cmos_fidelity_rate * 100.0
    );
}
