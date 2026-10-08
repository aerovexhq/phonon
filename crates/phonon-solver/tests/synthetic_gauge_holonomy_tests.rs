#![deny(unsafe_code)]

//! Integration test suite for Phase 442:
//! Topological Acoustic Synthetic Gauge Field & Non-Abelian Holonomic Quantum Gate Processor.

use phonon_solver::synthetic_gauge_holonomy::{
    HolonomicGateKind, HolonomicGateParams, HolonomicGateProcessor, ParameterLoopProfile,
    SyntheticGaugeFieldSolver, SyntheticGaugeParams, TopologicalSyntheticGaugeProcessor,
    WilczekZeeParams, WilczekZeeSolver,
};

#[test]
fn test_synthetic_gauge_flux_and_edge_dispersion() {
    let mut params = SyntheticGaugeParams::default();
    params.synthetic_flux_ratio = 0.25; // Peierls flux pi/2
    params.has_edge_defect = false;

    let solver = SyntheticGaugeFieldSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // 1. Verify synthetic Chern number C = 1.0
    assert_eq!(metrics.synthetic_chern_number, 1.0);
    assert!(metrics.bulk_bandgap_mhz >= 2.5);
    assert_eq!(metrics.defect_immunity_ratio, 1.0);
    assert!(metrics.edge_confinement_factor >= 0.85);

    // 2. Introduce boundary defect vacancy and verify backscattering immunity T_defect / T_clean >= 0.95
    params.has_edge_defect = true;
    let defect_solver = SyntheticGaugeFieldSolver::new(params);
    let defect_metrics = defect_solver.evaluate_metrics();
    assert!(
        defect_metrics.defect_immunity_ratio >= 0.95,
        "Defect immunity {:.3} must be >= 0.95",
        defect_metrics.defect_immunity_ratio
    );

    // 3. Verify Hofstadter butterfly spectrum generation
    let spectrum = solver.compute_hofstadter_spectrum(30);
    assert!(spectrum.len() >= 120);

    // 4. Verify chiral edge dispersion points
    let dispersion = solver.compute_edge_dispersion(32);
    let forward_modes: Vec<_> = dispersion.iter().filter(|p| p.is_forward_edge).collect();
    assert_eq!(forward_modes.len(), 32);
}

#[test]
fn test_wilczek_zee_holonomy_non_commutativity_and_invariance() {
    let params = WilczekZeeParams::default();
    let solver = WilczekZeeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // 1. Verify non-Abelian Wilczek-Zee non-commutativity: ||[U(C1), U(C2)]|| >= 0.50
    assert!(
        metrics.commutator_norm >= 0.50,
        "Commutator norm {:.4} must be >= 0.50",
        metrics.commutator_norm
    );

    // 2. Verify geometric path speed invariance: residual < 1e-4
    assert!(
        metrics.speed_invariance_residual < 1.0e-4,
        "Speed invariance residual {:.2e} must be < 1e-4",
        metrics.speed_invariance_residual
    );

    // 3. Verify dark-state leakage suppression P_leak < 1e-4
    assert!(
        metrics.bright_state_leakage < 1.0e-4,
        "Bright state leakage {:.2e} must be < 1e-4",
        metrics.bright_state_leakage
    );
    assert!(metrics.dark_state_fidelity >= 0.999);

    // 4. Verify holonomy unitaries for different loop profiles
    let u1 = solver.compute_holonomy_unitary(ParameterLoopProfile::LongitudinalCircle);
    let u2 = solver.compute_holonomy_unitary(ParameterLoopProfile::LatitudinalCircle);
    let u3 = solver.compute_holonomy_unitary(ParameterLoopProfile::TiltedGeodesic);

    assert!(u1.norm() > 0.99);
    assert!(u2.norm() > 0.99);
    assert!(u3.norm() > 0.99);

    let traj = solver.compute_trajectory(30);
    assert_eq!(traj.len(), 30);
}

#[test]
fn test_holonomic_quantum_gate_synthesis_and_readout() {
    let mut params = HolonomicGateParams::default();
    params.target_gate = HolonomicGateKind::Hadamard;

    let processor = HolonomicGateProcessor::new(params.clone());
    let metrics = processor.evaluate_metrics();

    // 1. Single-qubit Hadamard gate fidelity F >= 0.999
    assert!(
        metrics.process_fidelity >= 0.999,
        "Single-qubit fidelity {:.5} must be >= 0.999",
        metrics.process_fidelity
    );
    assert!(metrics.bright_state_leakage < 1.0e-4);
    assert!(metrics.execution_latency_ns <= 50.0);

    // 2. Two-qubit CZ entangling gate fidelity and concurrence
    params.target_gate = HolonomicGateKind::ControlledPhaseCZ;
    let cz_processor = HolonomicGateProcessor::new(params);
    let cz_metrics = cz_processor.evaluate_metrics();

    assert!(
        cz_metrics.process_fidelity >= 0.999,
        "Two-qubit CZ fidelity {:.5} must be >= 0.999",
        cz_metrics.process_fidelity
    );
    assert!(
        cz_metrics.entangling_concurrence >= 0.95,
        "Concurrence {:.4} must be >= 0.95",
        cz_metrics.entangling_concurrence
    );

    // 3. Dispersive cavity readout SNR >= 18.0 dB and QND fidelity >= 0.998
    assert!(
        cz_metrics.readout_snr_db >= 18.0,
        "Readout SNR {:.2} dB must be >= 18.0 dB",
        cz_metrics.readout_snr_db
    );
    assert!(
        cz_metrics.qnd_readout_fidelity >= 0.998,
        "QND fidelity {:.4} must be >= 0.998",
        cz_metrics.qnd_readout_fidelity
    );

    // 4. Verify readout spectrum generation
    let spectrum = cz_processor.compute_readout_spectrum(40);
    assert_eq!(spectrum.len(), 40);
}

#[test]
fn test_topological_synthetic_gauge_10_point_audit_all_passed() {
    let processor = TopologicalSyntheticGaugeProcessor::new();
    let audit = processor.audit_processor();

    assert!(audit.trs_breaking_gauge_flux_passed, "Audit item 1 failed");
    assert!(audit.synthetic_chern_quantization_passed, "Audit item 2 failed");
    assert!(audit.chiral_defect_immunity_passed, "Audit item 3 failed");
    assert!(audit.non_abelian_non_commutativity_passed, "Audit item 4 failed");
    assert!(audit.geometric_speed_invariance_passed, "Audit item 5 failed");
    assert!(audit.dark_state_leakage_suppression_passed, "Audit item 6 failed");
    assert!(audit.single_qubit_gate_fidelity_passed, "Audit item 7 failed");
    assert!(audit.two_qubit_entangling_fidelity_passed, "Audit item 8 failed");
    assert!(audit.sub_50ns_gate_latency_passed, "Audit item 9 failed");
    assert!(audit.cryogenic_dispersive_readout_passed, "Audit item 10 failed");

    assert_eq!(audit.total_score, 10);
    assert!(audit.all_passed);
}
