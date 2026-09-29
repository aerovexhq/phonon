#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for programmable chiral
//! phonon networks and high-dimensional quantum acoustic graph states.

use phonon_models::programmable_chiral_graph::ProgrammableChiralGraphParams;
use phonon_solver::programmable_chiral_graph::ProgrammableChiralGraphSolver;

#[test]
fn test_graph_entanglement_fidelity_bounds() {
    let params = ProgrammableChiralGraphParams::default();
    let solver = ProgrammableChiralGraphSolver::new(params);
    let fid = solver.compute_graph_entanglement_fidelity();

    // Target graph entanglement fidelity >= 94.0%
    assert!(
        fid >= 0.940,
        "Graph entanglement fidelity must be >= 0.940, got {:.4}",
        fid
    );
    assert!(
        fid <= 1.0,
        "Entanglement fidelity cannot exceed unity, got {:.4}",
        fid
    );
}

#[test]
fn test_topological_edge_purity_bounds() {
    let params = ProgrammableChiralGraphParams::default();
    let solver = ProgrammableChiralGraphSolver::new(params);
    let purity = solver.compute_topological_edge_purity();

    // Target topological edge purity >= 96.0%
    assert!(
        purity >= 0.960,
        "Topological edge channel purity must be >= 0.960, got {:.4}",
        purity
    );
    assert!(
        purity <= 1.0,
        "Edge purity cannot exceed unity, got {:.4}",
        purity
    );
}

#[test]
fn test_network_nodes_count_bounds() {
    let params = ProgrammableChiralGraphParams::default();
    let solver = ProgrammableChiralGraphSolver::new(params);
    let nodes = solver.compute_network_nodes_count();

    // Target network nodes count >= 64
    assert!(
        nodes >= 64,
        "Network nodes count must be >= 64, got {}",
        nodes
    );
}

#[test]
fn test_switching_time_bounds() {
    let params = ProgrammableChiralGraphParams::default();
    let solver = ProgrammableChiralGraphSolver::new(params);
    let switch_time = solver.compute_switching_time_ns();

    // Target reconfigurable switching time <= 20.0 ns
    assert!(
        switch_time <= 20.0,
        "Reconfigurable switching time must be <= 20.0 ns, got {:.2} ns",
        switch_time
    );
    assert!(
        switch_time >= 0.0,
        "Switching time must be non-negative, got {:.2} ns",
        switch_time
    );
}

#[test]
fn test_nullifier_variance_bounds() {
    let params = ProgrammableChiralGraphParams::default();
    let solver = ProgrammableChiralGraphSolver::new(params);
    let null_var = solver.compute_nullifier_variance_db();

    // Target continuous-variable nullifier variance <= -4.5 dB
    assert!(
        null_var <= -4.5,
        "Nullifier variance must be <= -4.5 dB, got {:.2} dB",
        null_var
    );
}

#[test]
fn test_stabilizer_generator_fidelity_bounds() {
    let params = ProgrammableChiralGraphParams::default();
    let solver = ProgrammableChiralGraphSolver::new(params);
    let stab_fid = solver.compute_stabilizer_generator_fidelity();

    // Target stabilizer generator fidelity >= 95.0%
    assert!(
        stab_fid >= 0.950,
        "Stabilizer generator fidelity must be >= 0.950, got {:.4}",
        stab_fid
    );
    assert!(
        stab_fid <= 1.0,
        "Stabilizer fidelity cannot exceed unity, got {:.4}",
        stab_fid
    );
}

#[test]
fn test_full_metrics_physical_compliance() {
    let params = ProgrammableChiralGraphParams::default();
    let solver = ProgrammableChiralGraphSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default programmable chiral graph configuration must satisfy all physical compliance thresholds: {:?}",
        metrics
    );
    assert!(metrics.graph_entanglement_fidelity >= 0.940);
    assert!(metrics.topological_edge_purity >= 0.960);
    assert!(metrics.network_nodes_count >= 64);
    assert!(metrics.switching_time_ns <= 20.0);
    assert!(metrics.nullifier_variance_db <= -4.5);
    assert!(metrics.stabilizer_generator_fidelity >= 0.950);
}

#[test]
fn test_parameter_clamping() {
    let params = ProgrammableChiralGraphParams::new(
        8,      // clamped to 16
        0.1,    // clamped to 0.5
        1.0,    // clamped to 3.0
        2.0,    // clamped to 5.0
        0.5,    // clamped to 1.0
        10.0,   // clamped to 20.0
        0.5,    // clamped to 1.0
        0.001,  // clamped to 0.005
    );

    assert_eq!(params.network_nodes_count, 16);
    assert_eq!(params.acoustic_resonance_ghz, 0.5);
    assert_eq!(params.initial_squeezing_db, 3.0);
    assert_eq!(params.inter_site_coupling_mhz, 5.0);
    assert_eq!(params.phase_shifter_switching_time_ns, 1.0);
    assert_eq!(params.chiral_isolation_db, 20.0);
    assert_eq!(params.operating_temp_m_k, 1.0);
    assert_eq!(params.waveguide_propagation_loss_db_per_cm, 0.005);

    let params_upper = ProgrammableChiralGraphParams::new(
        500,    // clamped to 256
        25.0,   // clamped to 15.0
        25.0,   // clamped to 18.0
        200.0,  // clamped to 100.0
        100.0,  // clamped to 50.0
        100.0,  // clamped to 60.0
        500.0,  // clamped to 100.0
        0.50,   // clamped to 0.10
    );

    assert_eq!(params_upper.network_nodes_count, 256);
    assert_eq!(params_upper.acoustic_resonance_ghz, 15.0);
    assert_eq!(params_upper.initial_squeezing_db, 18.0);
    assert_eq!(params_upper.inter_site_coupling_mhz, 100.0);
    assert_eq!(params_upper.phase_shifter_switching_time_ns, 50.0);
    assert_eq!(params_upper.chiral_isolation_db, 60.0);
    assert_eq!(params_upper.operating_temp_m_k, 100.0);
    assert_eq!(params_upper.waveguide_propagation_loss_db_per_cm, 0.10);
}
