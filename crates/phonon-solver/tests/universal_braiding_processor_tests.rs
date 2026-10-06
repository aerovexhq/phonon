#![deny(unsafe_code)]

//! Comprehensive test suite for Phase 400: Universal Non-Abelian Anyon Braiding
//! & Topological Quantum Acoustic Co-Processor Super-Engine.
//!
//! Verifies:
//! - Artin non-Abelian braid relations: sigma_1 sigma_2 sigma_1 == sigma_2 sigma_1 sigma_2.
//! - Distant braid commutation: sigma_1 sigma_3 == sigma_3 sigma_1.
//! - Single-qubit Clifford gate compilation and fidelity >= 0.999 (H, S, X, Z).
//! - T-gate magic state distillation protocol simulation and Solovay-Kitaev precision.
//! - Dispersive parity readout cavity splitting and SNR >= 18 dB.
//! - QND parity readout fidelity >= 0.998.
//! - Bell state synthesis, concurrence >= 0.95, and CHSH violation S_CHSH > 2.70.
//! - Crossbar crosstalk isolation >= 40 dB and insertion loss <= 0.5 dB.
//! - 10-point physics audit 10/10 full pass.

use std::f64::consts::PI;
use phonon_solver::universal_braiding_processor::{
    verify_artin_relation, verify_distant_commutation, BellStateKind, BraidingParams,
    CliffordTGateCompiler, CrossbarMatrixRouter, CrossbarParams, DispersiveCavityResponse,
    ElementaryBraid, EntanglementSynthesizer, FermionParity, InterferometerParams, TargetGate,
    UniversalBraidingProcessor,
};

#[test]
fn test_artin_non_abelian_braid_relations() {
    // Test Yang-Baxter Artin relation: sigma_1 * sigma_2 * sigma_1 == sigma_2 * sigma_1 * sigma_2
    let (artin_pass, diff) = verify_artin_relation(1, 4);
    assert!(
        artin_pass,
        "Artin relation sigma_1 sigma_2 sigma_1 == sigma_2 sigma_1 sigma_2 must hold (diff = {:.2e})",
        diff
    );
    assert!(diff < 1e-12, "Residual norm must be < 1e-12");

    // Test second braid generator pair: sigma_2 * sigma_3 * sigma_2 == sigma_3 * sigma_2 * sigma_3
    let (artin_pass2, diff2) = verify_artin_relation(2, 6);
    assert!(
        artin_pass2,
        "Artin relation sigma_2 sigma_3 sigma_2 == sigma_3 sigma_2 sigma_3 must hold (diff = {:.2e})",
        diff2
    );
    assert!(diff2 < 1e-12);

    // Test distant braid commutativity: sigma_1 * sigma_3 == sigma_3 * sigma_1 for |1 - 3| >= 2
    let (comm_pass, comm_diff) = verify_distant_commutation(1, 3, 6);
    assert!(
        comm_pass,
        "Distant braids must commute: sigma_1 sigma_3 == sigma_3 sigma_1 (diff = {:.2e})",
        comm_diff
    );
    assert!(comm_diff < 1e-12);

    // Test SU(2) unitary representation of sigma_1 * sigma_2 * sigma_1 acting as Hadamard
    let b1 = ElementaryBraid::new(1);
    let b2 = ElementaryBraid::new(2);
    let u1 = b1.unitary_2x2();
    let u2 = b2.unitary_2x2();

    // Compute LHS = u1 * u2 * u1
    let u12 = phonon_solver::universal_braiding_processor::mat_mul_2x2(&u1, &u2);
    let lhs = phonon_solver::universal_braiding_processor::mat_mul_2x2(&u12, &u1);

    // Compute RHS = u2 * u1 * u2
    let u21 = phonon_solver::universal_braiding_processor::mat_mul_2x2(&u2, &u1);
    let rhs = phonon_solver::universal_braiding_processor::mat_mul_2x2(&u21, &u2);

    // Verify LHS == RHS in SU(2)
    for r in 0..2 {
        for c in 0..2 {
            let d_re = (lhs[r][c].re - rhs[r][c].re).abs();
            let d_im = (lhs[r][c].im - rhs[r][c].im).abs();
            assert!(
                d_re < 1e-12 && d_im < 1e-12,
                "SU(2) Artin equality failed at ({}, {})",
                r,
                c
            );
        }
    }
}

#[test]
fn test_single_qubit_clifford_gate_compilation_and_fidelity() {
    let params = BraidingParams::default();
    let compiler = CliffordTGateCompiler::new(params);

    // 1. Identity Gate
    let id_res = compiler.compile_gate(TargetGate::Identity);
    assert!(id_res.process_fidelity >= 0.999);
    assert!(id_res.leakage_error < 1e-4);
    assert_eq!(id_res.magic_state_count, 0);

    // 2. Hadamard Gate
    let h_res = compiler.compile_gate(TargetGate::Hadamard);
    assert!(
        h_res.process_fidelity >= 0.999,
        "Hadamard fidelity must be >= 0.999, got {}",
        h_res.process_fidelity
    );
    assert!(h_res.leakage_error < 1e-4);
    assert_eq!(h_res.braid_sequence.len(), 3);
    assert_eq!(h_res.magic_state_count, 0);

    // 3. Phase S Gate
    let s_res = compiler.compile_gate(TargetGate::PhaseS);
    assert!(
        s_res.process_fidelity >= 0.999,
        "Phase S fidelity must be >= 0.999, got {}",
        s_res.process_fidelity
    );
    assert!(s_res.leakage_error < 1e-4);
    assert_eq!(s_res.braid_sequence.len(), 1);

    // 4. Pauli X Gate
    let x_res = compiler.compile_gate(TargetGate::PauliX);
    assert!(
        x_res.process_fidelity >= 0.999,
        "Pauli X fidelity must be >= 0.999, got {}",
        x_res.process_fidelity
    );
    assert!(x_res.leakage_error < 1e-4);
    assert_eq!(x_res.braid_sequence.len(), 2);

    // 5. Pauli Z Gate
    let z_res = compiler.compile_gate(TargetGate::PauliZ);
    assert!(
        z_res.process_fidelity >= 0.999,
        "Pauli Z fidelity must be >= 0.999, got {}",
        z_res.process_fidelity
    );
    assert!(z_res.leakage_error < 1e-4);
    assert_eq!(z_res.braid_sequence.len(), 2);
}

#[test]
fn test_t_gate_magic_state_distillation_and_solovay_kitaev() {
    let params = BraidingParams::default();
    let compiler = CliffordTGateCompiler::new(params);

    // 1. T-Gate Compilation with 15-to-1 Distillation
    let t_res = compiler.compile_gate(TargetGate::TGate);
    assert_eq!(t_res.magic_state_count, 1);
    assert!(
        t_res.distilled_fidelity >= 0.999,
        "15-to-1 distilled magic state fidelity must be >= 0.999, got {}",
        t_res.distilled_fidelity
    );
    assert!(
        t_res.process_fidelity >= 0.999,
        "T-gate process fidelity must be >= 0.999, got {}",
        t_res.process_fidelity
    );
    assert!(t_res.leakage_error < 1e-4);

    // 2. Solovay-Kitaev arbitrary rotation Rz(theta) decomposition
    let theta_target = PI / 3.0; // 60 degrees rotation
    let sk_res = compiler.compile_gate(TargetGate::ArbitraryRz(theta_target));
    assert!(
        sk_res.sk_approximation_error < 1e-3,
        "Solovay-Kitaev precision error must be < 1e-3, got {}",
        sk_res.sk_approximation_error
    );
    assert!(
        sk_res.process_fidelity >= 0.999,
        "Synthesized Rz fidelity must be >= 0.999, got {}",
        sk_res.process_fidelity
    );
    assert!(sk_res.magic_state_count >= 1);
}

#[test]
fn test_dispersive_parity_readout_cavity_splitting_and_snr() {
    let params = InterferometerParams::default();
    let response = DispersiveCavityResponse::new(params.clone());

    // In strong dispersive regime: chi (4.5 MHz) > kappa (1.2 MHz)
    assert!(params.dispersive_shift_chi_mhz > params.cavity_linewidth_mhz);

    // Parity doublet peak frequencies
    let chi = params.dispersive_shift_chi_mhz;
    let power_even_peak = response.transmission_power(chi, FermionParity::Even);
    let power_odd_peak = response.transmission_power(-chi, FermionParity::Odd);

    // On-resonance peak transmission is maximal (approx 1.0)
    assert!((power_even_peak - 1.0).abs() < 1e-4);
    assert!((power_odd_peak - 1.0).abs() < 1e-4);

    // Parity contrast at even peak frequency: power for odd parity should be deeply suppressed
    let power_odd_at_even_peak = response.transmission_power(chi, FermionParity::Odd);
    let contrast_db = 10.0 * (power_even_peak / power_odd_at_even_peak.max(1e-12)).log10();
    assert!(
        contrast_db > 20.0,
        "Transmission parity doublet contrast must be > 20.0 dB, got {:.2} dB",
        contrast_db
    );

    // SNR in dB must be >= 18.0 dB
    let snr_db = response.snr_db();
    assert!(
        snr_db >= 18.0,
        "Measurement SNR must be >= 18.0 dB, got {:.2} dB",
        snr_db
    );

    // Dephasing rate Gamma_meas check
    let gamma_meas = response.measurement_dephasing_rate_mhz();
    assert!(gamma_meas > 100.0);
}

#[test]
fn test_qnd_parity_readout_fidelity() {
    let params = InterferometerParams::default();
    let response = DispersiveCavityResponse::new(params);

    let fidelity = response.compute_readout_fidelity();
    assert!(
        fidelity >= 0.998,
        "QND parity readout fidelity must be >= 0.998, got {}",
        fidelity
    );

    // Simulate QND measurement trajectory trace
    let trace_even = response.simulate_qnd_trajectory(40, 5.0, FermionParity::Even);
    assert_eq!(trace_even.times_ns.len(), 40);
    assert_eq!(trace_even.parity, FermionParity::Even);
    // Integrated signal for Even must trend positive
    assert!(
        *trace_even.integrated_signal.last().unwrap() > 0.0,
        "Even parity trajectory must integrate positively"
    );

    let trace_odd = response.simulate_qnd_trajectory(40, 5.0, FermionParity::Odd);
    assert_eq!(trace_odd.parity, FermionParity::Odd);
    // Integrated signal for Odd must trend negative
    assert!(
        *trace_odd.integrated_signal.last().unwrap() < 0.0,
        "Odd parity trajectory must integrate negatively"
    );
}

#[test]
fn test_bell_state_synthesis_concurrence_and_chsh_violation() {
    let params = CrossbarParams::default();
    let synth = EntanglementSynthesizer::new(params);

    for bell_state in [
        BellStateKind::PhiPlus,
        BellStateKind::PhiMinus,
        BellStateKind::PsiPlus,
        BellStateKind::PsiMinus,
    ] {
        // State fidelity >= 0.990
        let fid = synth.compute_state_fidelity(bell_state);
        assert!(
            fid >= 0.990,
            "State fidelity for {:?} must be >= 0.990, got {}",
            bell_state,
            fid
        );

        // Density matrix trace must equal 1.0
        let rho = synth.compute_density_matrix_4x4(bell_state);
        let tr = rho[0][0].re + rho[1][1].re + rho[2][2].re + rho[3][3].re;
        assert!(
            (tr - 1.0).abs() < 1e-6,
            "Density matrix trace must be 1.0, got {}",
            tr
        );

        // Concurrence C(rho) >= 0.95
        let concurrence = synth.compute_concurrence(bell_state);
        assert!(
            concurrence >= 0.95,
            "Concurrence for {:?} must be >= 0.95, got {}",
            bell_state,
            concurrence
        );

        // CHSH parameter violation S_CHSH >= 2.75 > 2.0 (local realism bound)
        let s_chsh = synth.compute_chsh_parameter(bell_state);
        assert!(
            s_chsh >= 2.75,
            "CHSH parameter for {:?} must be >= 2.75, got {}",
            bell_state,
            s_chsh
        );
        assert!(
            s_chsh > 2.70,
            "CHSH parameter must strictly exceed 2.70"
        );
    }
}

#[test]
fn test_crossbar_crosstalk_isolation_and_insertion_loss() {
    let params = CrossbarParams::default();
    let mut router = CrossbarMatrixRouter::new(params);

    // Insertion loss <= 0.5 dB
    let il = router.insertion_loss_db();
    assert!(
        il <= 0.5,
        "Insertion loss must be <= 0.5 dB, got {:.3} dB",
        il
    );

    // Crosstalk suppression >= 40.0 dB
    let isolation = router.crosstalk_suppression_db();
    assert!(
        isolation >= 40.0,
        "Crosstalk isolation must be >= 40.0 dB, got {:.1} dB",
        isolation
    );

    // Check routing configuration update
    router.route_port(0, 3);
    assert_eq!(router.active_routes[0], 3);

    // S-matrix dimensions
    let smat = router.scattering_matrix();
    assert_eq!(smat.len(), 4);
    assert_eq!(smat[0].len(), 4);
}

#[test]
fn test_10_point_physics_audit_full_pass() {
    let processor = UniversalBraidingProcessor::default();
    let report = processor.audit_coprocessor();

    assert_eq!(report.total_count, 10, "Audit must evaluate 10 criteria");
    assert_eq!(
        report.passed_count, 10,
        "All 10 physics audit criteria must pass, failed: {:?}",
        report
            .criteria
            .iter()
            .filter(|c| !c.passed)
            .map(|c| c.name)
            .collect::<Vec<_>>()
    );
    assert!(report.overall_pass, "Overall audit must pass 10/10");
    assert!(
        report.cold_boot_latency_us < 2000.0,
        "Cold boot latency must be < 2000 us (2 ms)"
    );
}
