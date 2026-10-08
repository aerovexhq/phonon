#![deny(unsafe_code)]

use phonon_solver::read_rezayi_fibonacci::{
    EnclosedTopologicalCharge, FibonacciBraidingParams, FibonacciBraidingSolver,
    FibonacciTargetGate, QuantumAcousticBusParams, QuantumAcousticBusSolver,
    ReadRezayiFibonacciProcessor, ReadRezayiFilling, ReadRezayiStateParams,
    ReadRezayiStateSolver, SawInterferometerParams, SawInterferometerSolver,
};

#[test]
fn test_read_rezayi_state_metrics_and_golden_ratio() {
    let params = ReadRezayiStateParams {
        filling_factor: ReadRezayiFilling::Nu12Over5,
        magnetic_field_t: 5.80,
        topological_gap_kelvin: 0.048,
        ..Default::default()
    };
    let solver = ReadRezayiStateSolver::new(params);
    let metrics = solver.evaluate_metrics();

    let phi_expected = (1.0 + 5.0_f64.sqrt()) / 2.0;
    assert!((metrics.fibonacci_quantum_dimension - phi_expected).abs() < 1e-6);
    assert!((metrics.filling_factor_val - 2.40).abs() < 1e-4);
    assert!(metrics.neutral_mode_velocity_ms > 1.0e4);
    assert!(metrics.charge_mode_velocity_ms > 5.0e4);

    let dispersion = solver.compute_edge_dispersion(20);
    assert_eq!(dispersion.len(), 20);
    assert!(dispersion[19].charge_branch_ghz > dispersion[0].charge_branch_ghz);

    let corr = solver.compute_pair_correlation(25);
    assert_eq!(corr.len(), 25);
    assert!(corr[0].pair_correlation_g_r < corr[24].pair_correlation_g_r);
}

#[test]
fn test_fibonacci_f_matrix_unitarity_and_artin_braid() {
    let params = FibonacciBraidingParams {
        target_gate: FibonacciTargetGate::Hadamard,
        ..Default::default()
    };
    let solver = FibonacciBraidingSolver::new(params);

    let (f_pass, f_err) = solver.verify_f_matrix_unitarity();
    assert!(f_pass, "F-matrix unitarity error too high: {}", f_err);
    assert!(f_err < 1.0e-11);

    let (artin_pass, artin_err) = solver.verify_artin_braid_relation();
    assert!(artin_pass, "Artin braid relation error too high: {}", artin_err);
    assert!(artin_err < 1.0e-10);

    let metrics = solver.evaluate_metrics();
    assert!(metrics.compiled_gate_fidelity >= 0.999);
    assert!(!metrics.magic_state_distillation_required);

    let trajectories = solver.generate_worldline_trajectories(15);
    assert_eq!(trajectories.len(), 15);
}

#[test]
fn test_saw_interferometer_non_abelian_suppression() {
    let mut params_vac = SawInterferometerParams::default();
    params_vac.enclosed_charge = EnclosedTopologicalCharge::Vacuum1;
    let solver_vac = SawInterferometerSolver::new(params_vac);
    let m_vac = solver_vac.evaluate_metrics();

    let mut params_tau = SawInterferometerParams::default();
    params_tau.enclosed_charge = EnclosedTopologicalCharge::SingleTau;
    let solver_tau = SawInterferometerSolver::new(params_tau);
    let m_tau = solver_tau.evaluate_metrics();

    assert!(m_vac.vacuum_visibility_percent >= 80.0);
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let expected_suppression = 1.0 / phi; // ~0.618034
    assert!(
        (m_tau.visibility_suppression_ratio - expected_suppression).abs() < 0.02,
        "Observed suppression ratio {} deviates from 1/phi {}",
        m_tau.visibility_suppression_ratio,
        expected_suppression
    );

    let flux_pts = solver_tau.compute_flux_oscillations(30);
    assert_eq!(flux_pts.len(), 30);
    let spec = solver_tau.compute_acoustic_spectrum(25);
    assert_eq!(spec.len(), 25);
}

#[test]
fn test_quantum_acoustic_bus_and_audit() {
    let bus_solver = QuantumAcousticBusSolver::new(QuantumAcousticBusParams::default());
    let bm = bus_solver.evaluate_metrics();

    assert!(bm.state_transfer_fidelity >= 0.990);
    assert!(bm.chiral_isolation_db >= 30.0);
    assert!(bm.thermal_phonon_occupancy < 0.01);
    assert!(bm.multi_node_entanglement_concurrence >= 0.94);

    let pulses = bus_solver.compute_pulse_dynamics(20);
    assert_eq!(pulses.len(), 20);

    let processor = ReadRezayiFibonacciProcessor::default();
    let audit = processor.audit_processor();
    assert_eq!(audit.total_score, 10);
    assert!(audit.all_passed);
}
