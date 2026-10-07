#![deny(unsafe_code)]

//! Comprehensive automated physics verification test suite for Phase 416:
//! Phonon Studio Quantum Metamaterial Non-Abelian Holonomic Geometric Braiding
//! & Monolithic CMOS-MEMS Co-Processor.

use phonon_solver::holonomic_braiding_coprocessor::{
    CmosMemsParams, CmosMemsSolver, HoloBraidingParams, HoloBraidingSolver, HolonomicBraidStep,
    HolonomicBraidingCoprocessor, HolonomicGateKind, HolonomicGateParams, HolonomicGateSolver,
};

#[test]
fn test_holonomic_gate_unitarity_and_fidelity() {
    let gate_kinds = [
        HolonomicGateKind::Hadamard,
        HolonomicGateKind::PhaseS,
        HolonomicGateKind::PauliX,
        HolonomicGateKind::PauliY,
        HolonomicGateKind::PauliZ,
        HolonomicGateKind::TGate,
        HolonomicGateKind::ArbitraryRotation {
            theta_deg: 45.0,
            phi_deg: 30.0,
        },
    ];

    for kind in gate_kinds {
        let params = HolonomicGateParams {
            gate_kind: kind,
            ..Default::default()
        };
        let solver = HolonomicGateSolver::new(params);
        let u = solver.target_unitary();

        assert!(
            u.is_unitary(1e-6),
            "Gate {:?} must synthesize an exact unitary matrix",
            kind
        );

        let metrics = solver.evaluate_metrics();
        assert!(
            metrics.process_fidelity >= 0.999,
            "Gate {:?} process fidelity must be >= 0.999, got {:.5}",
            kind,
            metrics.process_fidelity
        );
    }
}

#[test]
fn test_pure_geometric_phase_and_dynamical_cancellation() {
    let solver = HolonomicGateSolver::new(HolonomicGateParams::default());
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.residual_dynamical_phase_rad <= 1e-4,
        "Residual dynamical phase must be strictly <= 1e-4 rad, got {:.2e}",
        metrics.residual_dynamical_phase_rad
    );
    assert!(
        metrics.is_purely_geometric,
        "Gate must be verified as purely geometric"
    );

    let traj = solver.compute_trajectory(30);
    assert_eq!(traj.len(), 31);
    for pt in &traj {
        let r_sq = pt.bloch_x.powi(2) + pt.bloch_y.powi(2) + pt.bloch_z.powi(2);
        assert!(
            (r_sq - 1.0).abs() < 1e-4,
            "Bloch sphere trajectory must remain on the surface, got r^2 = {:.5}",
            r_sq
        );
    }
}

#[test]
fn test_non_abelian_commutation() {
    let solver_x = HolonomicGateSolver::new(HolonomicGateParams {
        gate_kind: HolonomicGateKind::PauliX,
        ..Default::default()
    });

    // Test [X, Z] != 0
    let (is_non_abelian_z, comm_z) =
        solver_x.verify_non_abelian_commutation(HolonomicGateKind::PauliZ);
    assert!(is_non_abelian_z, "Pauli X and Z must not commute");
    assert!(
        comm_z > 1.0,
        "Commutator norm must be significant, got {:.3}",
        comm_z
    );

    // Test [H, S] != 0
    let solver_h = HolonomicGateSolver::new(HolonomicGateParams {
        gate_kind: HolonomicGateKind::Hadamard,
        ..Default::default()
    });
    let (is_non_abelian_s, comm_s) =
        solver_h.verify_non_abelian_commutation(HolonomicGateKind::PhaseS);
    assert!(is_non_abelian_s, "Hadamard and Phase S must not commute");
    assert!(
        comm_s > 0.5,
        "Commutator norm must be non-zero, got {:.3}",
        comm_s
    );
}

#[test]
fn test_artin_braid_relations() {
    let solver = HoloBraidingSolver::new(HoloBraidingParams::default());
    assert!(
        solver.verify_artin_braid_relation(),
        "Artin braid relation B1 B2 B1 == B2 B1 B2 must hold"
    );

    let step = HolonomicBraidStep {
        step_index: 1,
        mode_a: 1,
        mode_b: 2,
        is_counter_clockwise: true,
    };
    let traj = solver.compute_braid_trajectory(step, 30);
    assert_eq!(traj.len(), 31);

    // Mode 1 and Mode 2 must exchange locations across the trajectory
    let init_a = traj[0].mode_positions[0];
    let init_b = traj[0].mode_positions[1];
    let final_a = traj[30].mode_positions[0];
    let final_b = traj[30].mode_positions[1];

    let dist_a_to_final_b = ((init_a.0 - final_b.0).powi(2) + (init_a.1 - final_b.1).powi(2)).sqrt();
    let dist_b_to_final_a = ((init_b.0 - final_a.0).powi(2) + (init_b.1 - final_a.1).powi(2)).sqrt();

    assert!(
        dist_a_to_final_b < 0.1,
        "Mode a must arrive at initial position of mode b"
    );
    assert!(
        dist_b_to_final_a < 0.1,
        "Mode b must arrive at initial position of mode a"
    );
}

#[test]
fn test_majorana_adiabatic_braiding_and_diabatic_suppression() {
    let solver = HoloBraidingSolver::new(HoloBraidingParams::default());
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.topological_gap_mhz >= 2.0,
        "Topological protection gap must be >= 2.0 MHz, got {:.2} MHz",
        metrics.topological_gap_mhz
    );
    assert!(
        metrics.diabatic_transition_error <= 1e-4,
        "Diabatic transition leakage error must be <= 1e-4, got {:.2e}",
        metrics.diabatic_transition_error
    );
}

#[test]
fn test_dispersive_parity_readout_and_qnd_fidelity() {
    let solver = HoloBraidingSolver::new(HoloBraidingParams::default());
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.parity_readout_snr_db >= 16.0,
        "Parity readout SNR must be >= 16.0 dB, got {:.1} dB",
        metrics.parity_readout_snr_db
    );
    assert!(
        metrics.qnd_readout_fidelity >= 0.995,
        "QND parity measurement fidelity must be >= 0.995, got {:.5}",
        metrics.qnd_readout_fidelity
    );
    assert!(
        metrics.cavity_splitting_mhz >= 5.0,
        "Cavity doublet splitting must be >= 5.0 MHz, got {:.2} MHz",
        metrics.cavity_splitting_mhz
    );

    let spectrum = solver.compute_parity_cavity_spectrum(40);
    assert!(!spectrum.is_empty());
}

#[test]
fn test_cmos_mems_actuation_speed_crosstalk_and_power() {
    let solver = CmosMemsSolver::new(CmosMemsParams::default());
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.rise_time_ns <= 5.0,
        "Actuator rise time must be <= 5.0 ns, got {:.1} ns",
        metrics.rise_time_ns
    );
    assert!(
        metrics.crosstalk_isolation_db >= 35.0,
        "Crosstalk isolation must be >= 35.0 dB, got {:.1} dB",
        metrics.crosstalk_isolation_db
    );
    assert!(
        metrics.total_cryogenic_dissipation_uw <= 50.0,
        "Total cryogenic power dissipation must be <= 50.0 uW, got {:.1} uW",
        metrics.total_cryogenic_dissipation_uw
    );
    assert!(
        metrics.piezo_electrostatic_efficiency >= 0.90,
        "Transduction efficiency must be >= 90%, got {:.1}%",
        metrics.piezo_electrostatic_efficiency * 100.0
    );

    let waveform = solver.compute_pulse_waveform(30);
    assert_eq!(waveform.len(), 31);
    let channels = solver.compute_channel_statuses();
    assert_eq!(channels.len(), 8);
}

#[test]
fn test_system_orchestrator_10_point_audit() {
    let coprocessor = HolonomicBraidingCoprocessor::default();
    let audit = coprocessor.audit_coprocessor();

    assert_eq!(
        audit.total_count, 10,
        "Audit must contain exactly 10 criteria"
    );
    assert_eq!(
        audit.passed_count, 10,
        "All 10 audit criteria must pass, failed: {:?}",
        audit
            .criteria
            .iter()
            .filter(|c| !c.passed)
            .map(|c| &c.name)
            .collect::<Vec<_>>()
    );
    assert!(audit.all_passed, "all_passed must be true");
}
