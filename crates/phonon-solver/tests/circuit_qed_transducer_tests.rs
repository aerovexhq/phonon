#![deny(unsafe_code)]

//! Automated test suite for Phase 435:
//! Phonon Studio Topological Higher-Order Acoustic Superconducting Circuit QED Quantum Transducer & Multi-Qubit Crossbar.

use phonon_solver::circuit_qed_transducer::{
    AcousticCornerParams, AcousticCornerSolver, CircuitQedTransducerProcessor, CrossbarParams,
    MultiQubitCrossbarSolver, TransmonCircuitQedSolver, TransmonParams,
};

#[test]
fn test_topological_acoustic_corner_mode_and_confinement() {
    let params = AcousticCornerParams::default();
    let solver = AcousticCornerSolver::new(params);
    let (metrics, points) = solver.solve_corner_mode();

    assert!(
        metrics.is_topological,
        "Metamaterial must be in topological phase"
    );
    assert!(
        metrics.confinement_ratio >= 0.85,
        "Corner mode confinement ratio must be >= 85%, got {:.3}",
        metrics.confinement_ratio
    );
    assert!(
        metrics.bulk_gap_mhz >= 15.0,
        "Bulk bandgap must be >= 15 MHz, got {:.1} MHz",
        metrics.bulk_gap_mhz
    );
    assert!(
        metrics.transduction_rate_mhz >= 10.0,
        "Piezoelectric transduction rate must be >= 10 MHz"
    );
    assert_eq!(
        points.len(),
        36,
        "Grid with size 6 must produce 36 spatial points"
    );

    let corner_points_count = points.iter().filter(|p| p.is_corner).count();
    assert_eq!(corner_points_count, 4, "Must have exactly 4 corner cells");
}

#[test]
fn test_transmon_circuit_qed_and_state_transfer() {
    let params = TransmonParams::default();
    let solver = TransmonCircuitQedSolver::new(params);
    let (metrics, trajectory) = solver.solve_dynamics(15.0);

    assert!(
        metrics.qubit_freq_ghz > 4.5 && metrics.qubit_freq_ghz < 7.0,
        "Qubit transition frequency must be in 4.5-7.0 GHz range, got {:.3} GHz",
        metrics.qubit_freq_ghz
    );
    assert!(
        metrics.anharmonicity_mhz <= -200.0,
        "Negative anharmonicity must be <= -200 MHz, got {:.1} MHz",
        metrics.anharmonicity_mhz
    );
    assert!(
        metrics.vacuum_rabi_mhz >= 20.0,
        "Vacuum Rabi splitting must be >= 20 MHz, got {:.1} MHz",
        metrics.vacuum_rabi_mhz
    );
    assert!(
        metrics.tau_swap_ns > 10.0 && metrics.tau_swap_ns < 30.0,
        "State transfer swap time must be in 10-30 ns, got {:.2} ns",
        metrics.tau_swap_ns
    );
    assert!(
        metrics.state_transfer_fidelity >= 0.990,
        "State transfer fidelity must be >= 0.990, got {:.5}",
        metrics.state_transfer_fidelity
    );
    assert!(
        metrics.dispersive_shift_mhz >= 1.0,
        "Dispersive shift must be >= 1.0 MHz, got {:.2} MHz",
        metrics.dispersive_shift_mhz
    );
    assert!(
        metrics.qnd_readout_snr_db >= 15.0,
        "Dispersive QND readout SNR must be >= 15.0 dB, got {:.2} dB",
        metrics.qnd_readout_snr_db
    );
    assert_eq!(
        trajectory.len(),
        100,
        "Rabi trajectory must have 100 sample points"
    );
}

#[test]
fn test_multi_qubit_crossbar_routing_and_entangling_gate() {
    let params = CrossbarParams::default();
    let solver = MultiQubitCrossbarSolver::new(params);
    let metrics = solver.solve_crossbar(15.0, 150.0, 65.0);

    assert!(
        metrics.crosstalk_isolation_db >= 35.0,
        "Crosstalk isolation must be >= 35.0 dB, got {:.1} dB",
        metrics.crosstalk_isolation_db
    );
    assert!(
        metrics.insertion_loss_db <= 0.40,
        "Insertion loss must be <= 0.40 dB, got {:.2} dB",
        metrics.insertion_loss_db
    );
    assert!(
        metrics.two_qubit_dynamics.gate_fidelity >= 0.985,
        "Two-qubit gate fidelity must be >= 0.985, got {:.4}",
        metrics.two_qubit_dynamics.gate_fidelity
    );
    assert!(
        metrics.two_qubit_dynamics.bell_concurrence >= 0.95,
        "Bell concurrence must be >= 0.95, got {:.4}",
        metrics.two_qubit_dynamics.bell_concurrence
    );
    assert_eq!(
        metrics.routing_matrix.len(),
        16,
        "4x4 crossbar must have 16 S-matrix elements"
    );

    let active_routes = metrics
        .routing_matrix
        .iter()
        .filter(|el| el.is_active_route)
        .count();
    assert_eq!(
        active_routes, 2,
        "Must have forward and reverse active paths"
    );
}

#[test]
fn test_ten_point_circuit_qed_transducer_audit_full_pass() {
    let processor = CircuitQedTransducerProcessor::default();
    let report = processor.audit_transducer();

    assert!(
        report.corner_localization_pass,
        "Audit 1: Corner mode localization failed"
    );
    assert!(
        report.piezoelectric_transduction_pass,
        "Audit 2: Piezoelectric transduction failed"
    );
    assert!(
        report.transmon_anharmonicity_pass,
        "Audit 3: Transmon anharmonicity failed"
    );
    assert!(
        report.vacuum_rabi_splitting_pass,
        "Audit 4: Vacuum Rabi splitting failed"
    );
    assert!(
        report.dispersive_shift_pass,
        "Audit 5: Dispersive shift failed"
    );
    assert!(
        report.state_transfer_fidelity_pass,
        "Audit 6: State transfer fidelity failed"
    );
    assert!(
        report.crossbar_crosstalk_isolation_pass,
        "Audit 7: Crossbar crosstalk isolation failed"
    );
    assert!(
        report.two_qubit_gate_fidelity_pass,
        "Audit 8: Two-qubit gate fidelity failed"
    );
    assert!(
        report.dispersive_qnd_readout_pass,
        "Audit 9: Dispersive QND readout failed"
    );
    assert!(
        report.cryogenic_coherence_pass,
        "Audit 10: Cryogenic coherence failed"
    );

    assert_eq!(
        report.total_score, 10,
        "Total audit score must be strictly 10/10, got {}/10",
        report.total_score
    );
    assert!(report.all_passed, "All 10 physics audit checks must pass");
}
