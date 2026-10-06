#![deny(unsafe_code)]

//! Comprehensive Automated Test Suite for Phase 407:
//! Quantum Metamaterial Non-Abelian Majorana Braid Interconnect & Fault-Tolerant Surface Code Co-Processor.

use phonon_solver::majorana_surface_code::{
    CodeDistance, DispersiveParityReadoutParams, MagicDistillationEngine, MagicDistillationParams,
    MajoranaBraidingCrossbar, MajoranaBraidingCrossbarParams, MajoranaSurfaceCodeCoprocessor,
    PauliOperator, SurfaceCodePatch, SurfaceRng, TargetCliffordGate,
};

#[test]
fn test_artin_braid_relations() {
    let crossbar = MajoranaBraidingCrossbar::new(MajoranaBraidingCrossbarParams::default());

    // 1. Non-Abelian Yang-Baxter / Artin braid relation: B_1 B_2 B_1 == B_2 B_1 B_2
    let (artin_pass, artin_diff) = crossbar.verify_artin_braid_relation();
    assert!(
        artin_pass,
        "Artin braid relation failed: difference norm = {:.3e}",
        artin_diff
    );
    assert!(
        artin_diff < 1.0e-10,
        "Artin relation residual norm too high: {:.3e}",
        artin_diff
    );

    // 2. Distant commutativity: B_1 B_3 == B_3 B_1
    let (comm_pass, comm_diff) = crossbar.verify_distant_commutativity();
    assert!(
        comm_pass,
        "Distant commutativity failed: difference norm = {:.3e}",
        comm_diff
    );
    assert!(
        comm_diff < 1.0e-10,
        "Distant commutativity residual norm too high: {:.3e}",
        comm_diff
    );
}

#[test]
fn test_clifford_gate_compilation() {
    let crossbar = MajoranaBraidingCrossbar::new(MajoranaBraidingCrossbarParams::default());

    // Compile Phase S gate
    let s_gate = crossbar.compile_clifford_gate(TargetCliffordGate::PhaseS);
    assert_eq!(s_gate.braid_sequence, vec![1]);
    assert!(
        s_gate.process_fidelity >= 0.9990,
        "Phase S gate fidelity too low: {:.5}",
        s_gate.process_fidelity
    );

    // Compile Hadamard gate: B_1 B_2 B_1
    let h_gate = crossbar.compile_clifford_gate(TargetCliffordGate::Hadamard);
    assert_eq!(h_gate.braid_sequence, vec![1, 2, 1]);
    assert!(
        h_gate.process_fidelity >= 0.9990,
        "Hadamard gate fidelity too low: {:.5}",
        h_gate.process_fidelity
    );

    // Compile Pauli X gate: B_2 B_2
    let x_gate = crossbar.compile_clifford_gate(TargetCliffordGate::PauliX);
    assert_eq!(x_gate.braid_sequence, vec![2, 2]);
    assert!(
        x_gate.process_fidelity >= 0.9990,
        "Pauli X gate fidelity too low: {:.5}",
        x_gate.process_fidelity
    );

    // Compile Pauli Z gate: B_1 B_1
    let z_gate = crossbar.compile_clifford_gate(TargetCliffordGate::PauliZ);
    assert_eq!(z_gate.braid_sequence, vec![1, 1]);
    assert!(
        z_gate.process_fidelity >= 0.9990,
        "Pauli Z gate fidelity too low: {:.5}",
        z_gate.process_fidelity
    );
}

#[test]
fn test_diabatic_transition_error() {
    let crossbar = MajoranaBraidingCrossbar::new(MajoranaBraidingCrossbarParams::default());
    let p_diabatic = crossbar.compute_diabatic_error();

    assert!(
        p_diabatic < 1.0e-4,
        "Landau-Zener diabatic error must be < 1e-4: got {:.3e}",
        p_diabatic
    );
}

#[test]
fn test_braid_trajectory_generation() {
    let crossbar = MajoranaBraidingCrossbar::new(MajoranaBraidingCrossbarParams::default());
    let steps = crossbar.generate_braid_trajectory(12);

    assert_eq!(steps.len(), 12);
    assert_eq!(steps.first().unwrap().step_index, 0);
    assert_eq!(steps.last().unwrap().step_index, 11);

    // Initial position of gamma_1 should be in West arm (-X, 0)
    let p0 = steps.first().unwrap().mzm_positions[0];
    assert!(p0.0 < -5.0 && p0.1.abs() < 1.0e-3);

    // Final position of gamma_1 should be in North arm (0, +Y)
    let p_final = steps.last().unwrap().mzm_positions[0];
    assert!(p_final.0.abs() < 1.0e-3 && p_final.1 > 5.0);
}

#[test]
fn test_surface_code_stabilizer_commutativity() {
    // Distance-3 patch
    let patch_d3 = SurfaceCodePatch::new(CodeDistance::Distance3);
    let (comm_d3, pairs_d3) = patch_d3.verify_stabilizer_commutativity();
    assert!(comm_d3, "Distance-3 stabilizers must mutually commute");
    assert_eq!(pairs_d3, 16); // 4 Star X * 4 Plaquette Z = 16 pairs

    // Distance-5 patch
    let patch_d5 = SurfaceCodePatch::new(CodeDistance::Distance5);
    let (comm_d5, pairs_d5) = patch_d5.verify_stabilizer_commutativity();
    assert!(comm_d5, "Distance-5 stabilizers must mutually commute");
    assert_eq!(pairs_d5, 144); // 12 Star X * 12 Plaquette Z = 144 pairs
}

#[test]
fn test_syndrome_extraction_and_recovery() {
    let patch = SurfaceCodePatch::new(CodeDistance::Distance3);
    let mut rng = SurfaceRng::new(999);

    // Clean code space: 0 defects
    let clean_errors = vec![PauliOperator::Identity; 9];
    let clean_res = patch.extract_syndromes(&clean_errors, 0.0, &mut rng);
    assert_eq!(clean_res.defect_count, 0);

    // Inject single Pauli X (bit-flip) on data qubit 0
    let mut err_x0 = vec![PauliOperator::Identity; 9];
    err_x0[0] = PauliOperator::PauliX;
    let syn_x0 = patch.extract_syndromes(&err_x0, 0.0, &mut rng);
    assert!(syn_x0.defect_count > 0, "Defect must be detected for Pauli X");

    // Recover using decoder
    let rec_x0 = patch.decode_and_correct(&syn_x0);
    assert!(rec_x0.syndromes_cleared, "Syndromes must be cleared post-correction");
    assert!(rec_x0.logical_success, "Single-qubit X defect must be corrected successfully");

    // Inject single Pauli Z (phase-flip) on data qubit 4 (bulk qubit)
    let mut err_z4 = vec![PauliOperator::Identity; 9];
    err_z4[4] = PauliOperator::PauliZ;
    let syn_z4 = patch.extract_syndromes(&err_z4, 0.0, &mut rng);
    assert!(syn_z4.defect_count > 0, "Defect must be detected for Pauli Z");

    let rec_z4 = patch.decode_and_correct(&syn_z4);
    assert!(rec_z4.syndromes_cleared, "Syndromes must be cleared post-correction");
    assert!(rec_z4.logical_success, "Single-qubit Z defect must be corrected successfully");
}

#[test]
fn test_surface_code_threshold_scaling() {
    let curve = SurfaceCodePatch::evaluate_threshold_curves(8);
    assert!(!curve.is_empty());

    // Below threshold (p = 0.005 < 0.010): Distance-5 logical error must be strictly lower than Distance-3
    let sub_point = curve.iter().find(|p| p.physical_error_rate < 0.007).unwrap();
    assert!(
        sub_point.logical_error_d5 < sub_point.logical_error_d3,
        "Distance-5 logical error ({:.4e}) must be lower than Distance-3 ({:.4e}) below threshold",
        sub_point.logical_error_d5,
        sub_point.logical_error_d3
    );

    // Above threshold (p = 0.020 > 0.010): Distance-5 logical error should exceed Distance-3
    let super_point = curve.iter().find(|p| p.physical_error_rate > 0.018).unwrap();
    assert!(
        super_point.logical_error_d5 > super_point.logical_error_d3,
        "Distance-5 logical error ({:.4e}) should exceed Distance-3 ({:.4e}) above threshold",
        super_point.logical_error_d5,
        super_point.logical_error_d3
    );
}

#[test]
fn test_magic_state_distillation() {
    let engine = MagicDistillationEngine::new(
        MagicDistillationParams {
            input_infidelity: 0.05,
            ..Default::default()
        },
        DispersiveParityReadoutParams::default(),
    );

    let metrics = engine.evaluate_distillation();

    // Verify 35 * eps_in^3 scaling: 35 * 0.05^3 = 0.004375
    assert!(
        metrics.output_infidelity_stage1 < 0.01,
        "Stage 1 infidelity too high: {:.5}",
        metrics.output_infidelity_stage1
    );
    assert!(
        metrics.error_suppression_factor >= 5.0,
        "Error suppression factor must be >= 5x: got {:.2}x",
        metrics.error_suppression_factor
    );
    assert!(
        metrics.acceptance_probability >= 0.25,
        "Acceptance probability must be >= 25%: got {:.3}",
        metrics.acceptance_probability
    );
}

#[test]
fn test_dispersive_parity_readout() {
    let engine = MagicDistillationEngine::new(
        MagicDistillationParams::default(),
        DispersiveParityReadoutParams {
            dispersive_shift_chi_mhz: 4.8,
            cavity_linewidth_mhz: 1.2,
            ..Default::default()
        },
    );

    let readout = engine.evaluate_parity_readout(41);
    assert!(
        readout.snr_db >= 18.0,
        "Readout SNR must be >= 18.0 dB: got {:.2} dB",
        readout.snr_db
    );
    assert!(
        readout.qnd_fidelity >= 0.995,
        "QND fidelity must be >= 0.995: got {:.5}",
        readout.qnd_fidelity
    );
    assert_eq!(readout.peak_splitting_mhz, 9.6);
    assert_eq!(readout.spectrum.len(), 41);
}

#[test]
fn test_interconnect_crossbar_isolation() {
    let engine = MagicDistillationEngine::new(
        MagicDistillationParams::default(),
        DispersiveParityReadoutParams::default(),
    );

    let xbar = engine.evaluate_interconnect_crossbar(4);
    assert!(
        xbar.crosstalk_isolation_db >= 40.0,
        "Crossbar crosstalk isolation must be >= 40.0 dB: got {:.2} dB",
        xbar.crosstalk_isolation_db
    );
    assert!(
        xbar.insertion_loss_db <= 0.50,
        "Insertion loss must be <= 0.50 dB: got {:.3} dB",
        xbar.insertion_loss_db
    );
    assert!(xbar.transduction_efficiency >= 0.90);
}

#[test]
fn test_comprehensive_10_point_physics_audit() {
    let coprocessor = MajoranaSurfaceCodeCoprocessor::default();
    let audit = coprocessor.audit_coprocessor();

    assert_eq!(audit.total_count, 10);
    assert_eq!(
        audit.passed_count, 10,
        "Expected all 10 criteria to pass, got {}/10. Failed: {:?}",
        audit.passed_count,
        audit
            .criteria
            .iter()
            .filter(|c| !c.passed)
            .map(|c| c.name)
            .collect::<Vec<_>>()
    );
    assert!(audit.overall_pass);
    assert!(
        audit.cold_boot_latency_us < 2000.0,
        "Audit cold boot latency too high: {:.2} us",
        audit.cold_boot_latency_us
    );
}
