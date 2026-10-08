#![deny(unsafe_code)]

//! Integration Test Suite for Non-Abelian Anyon Braiding & Topological Qubit Crossbar in Chiral Phononic Graphene (Phase 454).
//!
//! Validates:
//! 1. Honeycomb chiral phononic graphene bulk Chern gap (Delta >= 15.0 MHz) and edge modes.
//! 2. Artin Yang-Baxter braid relations and adiabatic Landau-Zener leakage protection.
//! 3. Single-qubit Clifford logic compilation (F >= 99.9%) and two-qubit CNOT concurrence (C >= 0.92).
//! 4. Cryogenic crossbar routing, dispersive parity readout (SNR >= 20.0 dB), and low thermal occupancy (n_th <= 0.05).
//! 5. 10-point rigorous physics audit scoring 10/10 PASS.

use phonon_solver::chiral_graphene_braiding::{
    ChiralGrapheneBraidingProcessor, ChiralGrapheneLatticeParams, ChiralGrapheneLatticeSolver,
    GrapheneAnyonBraidParams, GrapheneAnyonBraidSolver, GrapheneCrossbarParams,
    GrapheneCrossbarSolver, GrapheneQubitGateParams, GrapheneQubitGateSolver, GrapheneTargetGate,
};

#[test]
fn test_chiral_graphene_lattice_topology_and_gap() {
    let params = ChiralGrapheneLatticeParams::default();
    let solver = ChiralGrapheneLatticeSolver::new(params);

    let metrics = solver.evaluate_metrics();
    assert!(
        metrics.bulk_chern_bandgap_mhz >= 15.0,
        "Bulk Chern bandgap must be >= 15.0 MHz, got {} MHz",
        metrics.bulk_chern_bandgap_mhz
    );
    assert_eq!(metrics.lower_band_chern_number, 1);
    assert_eq!(metrics.upper_band_chern_number, -1);
    assert!(metrics.edge_penetration_depth_cells <= 2.2);
    assert!(metrics.corner_transmission_ratio >= 0.940);

    let dispersion = solver.compute_dispersion(40);
    assert_eq!(dispersion.len(), 40);

    let spatial = solver.compute_spatial_mode();
    assert_eq!(spatial.len(), 24);
}

#[test]
fn test_non_abelian_braiding_artin_relations() {
    let params = GrapheneAnyonBraidParams::default();
    let solver = GrapheneAnyonBraidSolver::new(params);

    let metrics = solver.evaluate_metrics();
    assert!(
        metrics.artin_relation_error <= 1.0e-5,
        "Artin braid relation error must be <= 1.0e-5, got {}",
        metrics.artin_relation_error
    );
    assert!(
        metrics.far_commutation_error <= 1.0e-5,
        "Far commutation error must be <= 1.0e-5, got {}",
        metrics.far_commutation_error
    );
    assert!(
        metrics.diabatic_leakage_probability <= 1.0e-4,
        "Diabatic transition probability must be <= 1.0e-4, got {}",
        metrics.diabatic_leakage_probability
    );
    assert!(metrics.adiabatic_parameter >= 10.0);
    assert!(metrics.braid_unitary_fidelity_pct >= 99.9);

    let worldlines = solver.compute_worldlines(50);
    assert_eq!(worldlines.len(), 50);
    for pt in worldlines {
        assert!(pt.instantaneous_gap_mhz >= 12.0);
    }
}

#[test]
fn test_topological_qubit_clifford_and_cnot_gates() {
    // 1. Single-qubit Hadamard gate
    let mut h_params = GrapheneQubitGateParams::default();
    h_params.target_gate = GrapheneTargetGate::Hadamard;
    let h_solver = GrapheneQubitGateSolver::new(h_params);
    let h_metrics = h_solver.evaluate_metrics();
    assert!(h_metrics.gate_process_fidelity_pct >= 99.9);
    assert!(h_metrics.logical_coherence_time_us >= 45.0);

    // 2. Single-qubit Phase S gate
    let mut s_params = GrapheneQubitGateParams::default();
    s_params.target_gate = GrapheneTargetGate::PhaseS;
    let s_solver = GrapheneQubitGateSolver::new(s_params);
    let s_metrics = s_solver.evaluate_metrics();
    assert!(s_metrics.gate_process_fidelity_pct >= 99.9);

    // 3. Two-qubit entangling CNOT gate
    let mut cnot_params = GrapheneQubitGateParams::default();
    cnot_params.target_gate = GrapheneTargetGate::Cnot;
    let cnot_solver = GrapheneQubitGateSolver::new(cnot_params);
    let cnot_metrics = cnot_solver.evaluate_metrics();
    assert!(
        cnot_metrics.gate_process_fidelity_pct >= 99.0,
        "CNOT gate process fidelity must be >= 99.0%, got {}%",
        cnot_metrics.gate_process_fidelity_pct
    );
    assert!(
        cnot_metrics.entanglement_concurrence >= 0.92,
        "CNOT concurrence must be >= 0.92, got {}",
        cnot_metrics.entanglement_concurrence
    );
    assert!(cnot_metrics.bell_state_fidelity_pct >= 99.0);

    let curve = cnot_solver.compute_process_curve(30);
    assert_eq!(curve.len(), 30);
}

#[test]
fn test_cryogenic_crossbar_readout_and_coherence() {
    let params = GrapheneCrossbarParams::default();
    let solver = GrapheneCrossbarSolver::new(params);

    let metrics = solver.evaluate_metrics();
    assert!(
        metrics.dispersive_frequency_splitting_mhz >= 8.0,
        "Dispersive frequency splitting 2*chi must be >= 8.0 MHz, got {} MHz",
        metrics.dispersive_frequency_splitting_mhz
    );
    assert!(
        metrics.parity_readout_snr_db >= 20.0,
        "Parity readout SNR must be >= 20.0 dB, got {} dB",
        metrics.parity_readout_snr_db
    );
    assert!(
        metrics.qnd_readout_fidelity_pct >= 99.5,
        "QND readout fidelity must be >= 99.5%, got {}%",
        metrics.qnd_readout_fidelity_pct
    );
    assert!(
        metrics.crossbar_waveguide_isolation_db >= 38.0,
        "Crossbar waveguide isolation must be >= 38.0 dB, got {} dB",
        metrics.crossbar_waveguide_isolation_db
    );
    assert!(
        metrics.cryogenic_thermal_noise_occupancy <= 0.05,
        "Thermal noise occupancy must be <= 0.05 quanta, got {}",
        metrics.cryogenic_thermal_noise_occupancy
    );
    assert!(
        metrics.quasiparticle_poisoning_lifetime_us >= 40.0,
        "Quasiparticle poisoning lifetime must be >= 40.0 us, got {} us",
        metrics.quasiparticle_poisoning_lifetime_us
    );

    let spectrum = solver.compute_readout_spectrum(40);
    assert_eq!(spectrum.len(), 40);
}

#[test]
fn test_chiral_graphene_braiding_10_point_audit() {
    let processor = ChiralGrapheneBraidingProcessor::new(
        ChiralGrapheneLatticeParams::default(),
        GrapheneAnyonBraidParams::default(),
        GrapheneQubitGateParams::default(),
        GrapheneCrossbarParams::default(),
    );

    let audit = processor.evaluate_audit();
    assert_eq!(
        audit.score(),
        (10, 10),
        "Audit report must pass 10/10 criteria, got {:?}",
        audit
    );
    assert!(audit.is_pass());
}
