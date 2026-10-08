#![deny(unsafe_code)]

//! Integration Test Suite for Genus-2 Parafermion Surface Code & Acoustic Processor.
//!
//! Validates:
//! 1. Genus-2 Riemann surface geometry, hyperbolic octagon embedding & D=9 code space.
//! 2. Transversal fault-tolerant quantum logic, Dehn twists & CZ_L entangling gates.
//! 3. Homological stabilizer syndrome extraction & MWHM decoding with P_L <= 1.0e-6.
//! 4. Cryogenic 12x12 routing crossbar array, half-select isolation >= 38 dB & SNR >= 22 dB.
//! 5. 10-point rigorous physics audit full 10/10 PASS verification.

use phonon_solver::genus2_parafermion_surface::{
    DefectKind, FaultTolerantLogicParams, Genus2LogicSolver, Genus2ParafermionProcessor,
    Genus2StabilizerParams, Genus2StabilizerSolver, Genus2SurfaceParams, Genus2SurfaceSolver,
    LogicalGateKind, QuantumCrossbarParams, CryogenicCrossbarSolver,
    GENUS2_CROSSBAR_DIMENSION, TOTAL_GENUS2_CROSSBAR_CELLS,
};

#[test]
fn test_genus2_surface_geometry_and_code_space() {
    let params = Genus2SurfaceParams::default();
    let solver = Genus2SurfaceSolver::new(params);

    // Dimension of code space for genus g=2 and clock order M=3: D = M^g = 9
    let dim = solver.compute_code_space_dimension();
    assert_eq!(dim, 9, "Code space dimension for Z_3 on genus-2 must be exactly 9");

    let metrics = solver.evaluate_metrics();
    assert_eq!(metrics.code_space_dimension, 9);
    assert_eq!(metrics.euler_characteristic, -2, "Euler characteristic of genus-2 must be -2");
    assert!(
        metrics.protection_gap_uev >= 10.0,
        "Bulk protection gap must be >= 10.0 ueV, got {:.2}",
        metrics.protection_gap_uev
    );
    assert!(
        metrics.poisoning_suppression_lifetime_us >= 10.0,
        "Quasiparticle poisoning suppression lifetime must be >= 10.0 us, got {:.2}",
        metrics.poisoning_suppression_lifetime_us
    );
    assert!(
        metrics.commutator_phase_error <= 1.0e-5,
        "Modular commutator phase error must be <= 1.0e-5, got {:e}",
        metrics.commutator_phase_error
    );

    // Poincaré disk hyperbolic embedding
    let embedding = solver.compute_poincare_disk_embedding(10);
    assert!(embedding.len() >= 80, "Embedding must contain boundary and interior points");
    for pt in &embedding {
        assert!(pt.r >= 0.0 && pt.r < 1.0, "Poincaré radius must be in [0, 1)");
        assert!(pt.wavepacket_density >= 0.0, "Wavepacket density must be non-negative");
    }

    // Dispersion curves
    let dispersion = solver.compute_dispersion_curves(30);
    assert_eq!(dispersion.len(), 30);
    for pt in &dispersion {
        assert!(pt.local_gap_uev > 0.0, "Local gap must remain open");
        assert!(
            pt.energy_excited_uev > pt.energy_ground_uev,
            "Excited branch must be strictly above ground branch"
        );
    }
}

#[test]
fn test_fault_tolerant_logic_and_dehn_twists() {
    let params = FaultTolerantLogicParams::default();
    let solver = Genus2LogicSolver::new(params);

    // Single qudit gates
    let gates = [
        LogicalGateKind::PauliX1,
        LogicalGateKind::PauliZ1,
        LogicalGateKind::Hadamard1,
        LogicalGateKind::PhaseS1,
        LogicalGateKind::PauliX2,
        LogicalGateKind::PauliZ2,
    ];

    for gate in gates {
        let m = solver.evaluate_gate(gate);
        assert!(
            m.process_fidelity >= 0.999,
            "Single qudit gate {:?} process fidelity must be >= 0.999, got {:.5}",
            gate,
            m.process_fidelity
        );
        assert!(
            m.diabatic_leakage_prob <= 1.0e-5,
            "Diabatic leakage for {:?} must be <= 1.0e-5, got {:e}",
            gate,
            m.diabatic_leakage_prob
        );
    }

    // Dehn twist along non-contractible cycle alpha_1
    let twist_m = solver.evaluate_gate(LogicalGateKind::DehnTwistAlpha1);
    assert!(twist_m.process_fidelity >= 0.999);
    assert!(
        twist_m.diabatic_leakage_prob <= 1.0e-5,
        "Dehn twist leakage must be <= 1.0e-5, got {:e}",
        twist_m.diabatic_leakage_prob
    );

    // Dehn twist trajectory
    let trajectory = solver.simulate_dehn_twist_trajectory(40);
    assert_eq!(trajectory.len(), 40);
    let last = trajectory.last().unwrap();
    assert!(
        (last.twist_angle_rad - 2.0 * std::f64::consts::PI).abs() < 1.0e-3,
        "Final twist angle must reach 2*pi"
    );
    for pt in &trajectory {
        assert!(pt.instantaneous_leakage <= 2.0e-5);
    }

    // Inter-handle Controlled-Z (CZ_L) entangling gate
    let cz_m = solver.evaluate_gate(LogicalGateKind::ControlledZ);
    assert!(
        cz_m.process_fidelity >= 0.999,
        "CZ_L gate fidelity must be >= 0.999, got {:.5}",
        cz_m.process_fidelity
    );
    assert!(
        cz_m.concurrence >= 0.95,
        "CZ_L entangling concurrence must be >= 0.95, got {:.4}",
        cz_m.concurrence
    );

    // Synthesize maximally entangled state
    let bell_state = solver.synthesize_maximally_entangled_state();
    assert!(
        (bell_state.norm() - 1.0).abs() < 1.0e-6,
        "State vector must be normalized to 1.0"
    );
    let entropy = bell_state.entanglement_entropy();
    assert!(
        entropy > 1.0,
        "Entanglement entropy of maximally entangled qutrits must exceed 1.0 nats, got {:.4}",
        entropy
    );
}

#[test]
fn test_homological_stabilizer_code_and_mwhm_decoder() {
    let params = Genus2StabilizerParams::default();
    let solver = Genus2StabilizerSolver::new(params);

    // Hyperbolic metric on Poincaré disk
    let d0 = solver.hyperbolic_distance(0.0, 0.0, 0.0, 0.0);
    assert!(d0.abs() < 1.0e-9, "Distance between identical points must be 0");
    let d1 = solver.hyperbolic_distance(0.1, 0.2, 0.4, 0.5);
    assert!(d1 > 0.0, "Hyperbolic distance must be strictly positive");

    // Defect injection and MWHM error correction
    let syndrome = solver.inject_and_decode_errors(4);
    assert_eq!(syndrome.detected_defects.len(), 4);
    assert!(
        !syndrome.correction_chains.is_empty(),
        "Correction chains must be generated"
    );
    assert!(
        syndrome.logical_error_rate <= 1.0e-6,
        "Logical error rate must be suppressed to <= 1.0e-6, got {:e}",
        syndrome.logical_error_rate
    );
    assert!(
        syndrome.threshold_margin >= 10.0,
        "Threshold margin must be >= 10x, got {:.1}x",
        syndrome.threshold_margin
    );
    assert!(syndrome.correction_success);

    // Verify both defect kinds are modeled
    let has_charges = syndrome.detected_defects.iter().any(|d| d.kind == DefectKind::ParafermionCharge);
    let has_vortices = syndrome.detected_defects.iter().any(|d| d.kind == DefectKind::VortexFlux);
    assert!(has_charges && has_vortices, "Both parafermion charges and vortex defects must exist");

    // Threshold scaling comparison
    let curve = solver.compute_threshold_curve(25);
    assert_eq!(curve.len(), 25);
    // At low physical error, d=5 must have smaller logical error than d=3
    let low_error_pt = &curve[1];
    assert!(
        low_error_pt.logical_error_d5 < low_error_pt.logical_error_d3,
        "Higher distance d=5 must provide higher error suppression below threshold"
    );
}

#[test]
fn test_cryogenic_quantum_crossbar_and_dispersive_readout() {
    let params = QuantumCrossbarParams::default();
    let solver = CryogenicCrossbarSolver::new(params);

    assert_eq!(GENUS2_CROSSBAR_DIMENSION, 12);
    assert_eq!(TOTAL_GENUS2_CROSSBAR_CELLS, 144);

    let metrics = solver.evaluate_readout_metrics(0, 0);
    assert!(
        metrics.readout_snr_db >= 22.0,
        "Dispersive cavity readout SNR must be >= 22.0 dB, got {:.2} dB",
        metrics.readout_snr_db
    );
    assert!(
        metrics.crosstalk_isolation_db >= 38.0,
        "Half-select crosstalk isolation must be >= 38.0 dB, got {:.2} dB",
        metrics.crosstalk_isolation_db
    );
    assert!(
        metrics.retention_lifetime_us >= 50.0,
        "Quantum acoustic retention lifetime must be >= 50.0 us, got {:.2} us",
        metrics.retention_lifetime_us
    );
    assert!(
        metrics.readout_fidelity >= 0.999,
        "Parity state discrimination fidelity must be >= 0.999, got {:.5}",
        metrics.readout_fidelity
    );

    // Cavity transmission spectrum
    let spectrum = solver.compute_transmission_spectrum(50);
    assert_eq!(spectrum.len(), 50);
    let peak_p0 = spectrum
        .iter()
        .map(|pt| pt.transmission_p0_db)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        peak_p0 > -5.0,
        "Transmission peak for state P=0 must be well-resolved"
    );

    // 12x12 crosstalk map
    let grid = solver.compute_crosstalk_grid(3, 4);
    assert_eq!(grid.len(), 144);
    assert_eq!(grid[3 * 12 + 4], 0.0, "Selected cell has 0 dB attenuation");
    assert!(
        grid[3 * 12 + 5] >= 38.0,
        "Row half-selected cell must have >= 38 dB isolation"
    );
    assert!(
        grid[0 * 12 + 0] >= 56.0,
        "Unselected cell must have higher composite isolation"
    );
}

#[test]
fn test_genus2_parafermion_physics_audit_10_of_10_pass() {
    let processor = Genus2ParafermionProcessor::default();
    let audit = processor.evaluate_audit();

    let (passed, total) = audit.score();
    assert_eq!(
        (passed, total),
        (10, 10),
        "All 10 physics audit criteria must pass! Audit summary:\n{}",
        audit.summary()
    );
    assert!(audit.all_passed());

    assert!(audit.homology_commutator_pass);
    assert!(audit.code_space_dimension_pass);
    assert!(audit.topological_bulk_gap_pass);
    assert!(audit.poisoning_suppression_lifetime_pass);
    assert!(audit.dehn_twist_leakage_pass);
    assert!(audit.inter_handle_entangling_gate_pass);
    assert!(audit.code_distance_threshold_pass);
    assert!(audit.homological_matching_suppression_pass);
    assert!(audit.crossbar_crosstalk_isolation_pass);
    assert!(audit.dispersive_readout_snr_retention_pass);

    let summary = audit.summary();
    assert!(summary.contains("10/10 PASS"));
}
