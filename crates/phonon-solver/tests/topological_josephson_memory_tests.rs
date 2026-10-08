#![deny(unsafe_code)]

//! Integration tests for Topological Josephson Memory & Quantum Phase-Slip Crossbar Module.

use phonon_solver::topological_josephson_memory::{
    ChiralSotParams, ChiralSotSolver, QuantumPhaseSlipParams, QuantumPhaseSlipSolver,
    SuperconductingCrossbarParams, SuperconductingCrossbarSolver,
    TopologicalJosephsonMemoryProcessor, TopologicalJosephsonParams,
    TopologicalJosephsonSolver, CROSSBAR_DIMENSION, TOTAL_MEMORY_CELLS,
};
use std::f64::consts::PI;

#[test]
fn test_topological_josephson_cpr_anomalous_phase() {
    let params = TopologicalJosephsonParams::default();
    let solver = TopologicalJosephsonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify anomalous phase offset phi_0
    assert!(metrics.anomalous_phase_phi0_rad > 0.45 * PI);
    assert!(metrics.anomalous_phase_phi0_rad < 0.65 * PI);

    // Verify fractional 4pi supercurrent component
    assert!(metrics.fractional_4pi_current_ua > 0.0);
    assert!((metrics.fractional_4pi_current_ua / metrics.conventional_critical_current_ua - 0.35).abs() < 1e-4);

    // Verify memory double-well energy barrier Delta U >= 40.0 ueV
    assert!(
        metrics.memory_energy_barrier_uev >= 40.0,
        "Double-well barrier {} must be >= 40.0 ueV",
        metrics.memory_energy_barrier_uev
    );

    // Verify non-volatile retention lifetime tau >= 1.0 us
    assert!(
        metrics.retention_lifetime_us >= 1.0,
        "Retention lifetime {} must be >= 1.0 us",
        metrics.retention_lifetime_us
    );

    // Verify CPR curve generation
    let cpr = solver.compute_cpr_curve(50);
    assert_eq!(cpr.len(), 50);

    // Verify Andreev spectrum
    let andreev = solver.compute_andreev_spectrum(30);
    assert_eq!(andreev.len(), 30);
    for (_phi, e_plus, e_minus) in andreev {
        assert!(e_plus > 0.0);
        assert!(e_minus < 0.0);
    }
}

#[test]
fn test_chiral_sot_switching_dynamics() {
    let params = ChiralSotParams::default();
    let solver = ChiralSotSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify switching speed <= 150 ps
    assert!(
        metrics.switching_time_ps <= 150.0,
        "Switching time {} ps must be <= 150 ps",
        metrics.switching_time_ps
    );

    // Verify ultra-low energy dissipation <= 1.0 aJ
    assert!(
        metrics.switching_energy_aj <= 1.0,
        "Switching energy {} aJ must be <= 1.0 aJ",
        metrics.switching_energy_aj
    );

    // Verify low stochastic switching error rate <= 1.0e-5
    assert!(
        metrics.switching_error_rate <= 1.0e-5,
        "Switching error rate {} must be <= 1.0e-5",
        metrics.switching_error_rate
    );

    // Verify LLGS trajectory simulation
    let traj = solver.simulate_switching_trajectory(60);
    assert_eq!(traj.len(), 60);
    assert!(traj[0].mz > 0.8, "Initial mz must be positive");
    assert!(traj.last().unwrap().mz < 0.0, "Final mz must switch to negative");
}

#[test]
fn test_quantum_phase_slip_bloch_and_rabi() {
    let params = QuantumPhaseSlipParams::default();
    let solver = QuantumPhaseSlipSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify coherent QPS tunneling amplitude E_QPS >= 1.0 GHz
    assert!(
        metrics.qps_amplitude_ghz >= 1.0,
        "QPS amplitude {} GHz must be >= 1.0 GHz",
        metrics.qps_amplitude_ghz
    );

    // Verify Bloch oscillation voltage
    assert!(metrics.bloch_voltage_uv > 0.0);
    assert!(metrics.critical_voltage_vc_uv > 0.0);

    // Verify Rabi gate fidelity >= 0.995
    assert!(
        metrics.rabi_gate_fidelity >= 0.995,
        "Rabi fidelity {} must be >= 0.995",
        metrics.rabi_gate_fidelity
    );

    // Verify IV curve and Rabi trajectory
    let iv = solver.compute_iv_curve(30);
    assert_eq!(iv.len(), 30);

    let rabi = solver.compute_rabi_trajectory(40, 20.0);
    assert_eq!(rabi.len(), 40);
    assert!(rabi[0].prob_ground > 0.95);
}

#[test]
fn test_superconducting_crossbar_readout_and_isolation() {
    let params = SuperconductingCrossbarParams::default();
    let mut solver = SuperconductingCrossbarSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify matrix cell count
    assert_eq!(TOTAL_MEMORY_CELLS, 64);
    assert_eq!(CROSSBAR_DIMENSION, 8);
    assert_eq!(solver.cells().len(), 64);

    // Verify bit read and write
    assert!(solver.write_cell(3, 4, 1));
    assert_eq!(solver.read_cell(3, 4), Some(1));

    assert!(solver.write_cell(3, 4, 0));
    assert_eq!(solver.read_cell(3, 4), Some(0));

    // Verify dispersive cavity readout SNR >= 20.0 dB
    assert!(
        metrics.readout_snr_db >= 20.0,
        "Readout SNR {} dB must be >= 20.0 dB",
        metrics.readout_snr_db
    );

    // Verify half-select crosstalk isolation >= 35.0 dB
    assert!(
        metrics.half_select_isolation_db >= 35.0,
        "Crosstalk isolation {} dB must be >= 35.0 dB",
        metrics.half_select_isolation_db
    );

    // Verify cryogenic coherence lifetimes
    assert!(metrics.dephasing_time_t2_star_us >= 12.0);
    assert!(metrics.relaxation_time_t1_us >= 25.0);
    assert!(metrics.access_latency_ns <= 2.0);

    // Verify spectrum generation
    let spec = solver.compute_readout_spectrum(35);
    assert_eq!(spec.len(), 35);
}

#[test]
fn test_topological_josephson_memory_10_point_audit() {
    let processor = TopologicalJosephsonMemoryProcessor::default();
    let report = processor.run_physics_audit();

    let (passed, total) = report.score();
    println!("{}", report.summary());

    assert_eq!(total, 10);
    assert_eq!(passed, 10);
    assert!(report.all_passed());
}
