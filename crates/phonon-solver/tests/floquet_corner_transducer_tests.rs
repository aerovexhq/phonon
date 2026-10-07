#![deny(unsafe_code)]

use phonon_solver::floquet_corner_transducer::{
    CornerStateTransducer, CornerTransducerParams, EntanglementRouterParams,
    FloquetCornerTransducerProcessor, NonReciprocalEntanglementRouter, TargetEntangledState,
};

#[test]
fn test_corner_modes_and_spatial_confinement() {
    let params = CornerTransducerParams::default();
    let transducer = CornerStateTransducer::new(params);

    let modes = transducer.solve_corner_modes();
    assert_eq!(modes.len(), 4, "Must resolve exactly 4 corner modes");

    for m in &modes {
        assert!(
            m.spatial_confinement_ratio >= 0.85,
            "Spatial confinement ratio must be >= 85%, got {}",
            m.spatial_confinement_ratio
        );
        assert!(
            m.decay_rate_khz > 0.0 && m.decay_rate_khz < 100.0,
            "Acoustic decay rate must be within reasonable range, got {}",
            m.decay_rate_khz
        );
    }
}

#[test]
fn test_quantum_transduction_efficiency_and_fidelity() {
    let params = CornerTransducerParams::default();
    let transducer = CornerStateTransducer::new(params);

    let metrics = transducer.evaluate_transduction_metrics();
    assert!(
        metrics.peak_efficiency >= 0.85,
        "Transduction efficiency must be >= 85%, got {}",
        metrics.peak_efficiency
    );
    assert!(
        metrics.transfer_fidelity >= 0.995,
        "State transfer fidelity must be >= 0.995, got {}",
        metrics.transfer_fidelity
    );
    assert!(
        metrics.optimal_duration_ns > 5.0 && metrics.optimal_duration_ns < 100.0,
        "Optimal transfer pulse duration must be physical, got {}",
        metrics.optimal_duration_ns
    );
    assert!(
        metrics.cooperativity > 5.0,
        "Cooperativity must exceed 5, got {}",
        metrics.cooperativity
    );
}

#[test]
fn test_transduction_trajectory_dynamics() {
    let params = CornerTransducerParams::default();
    let transducer = CornerStateTransducer::new(params);

    let trajectory = transducer.compute_transduction_trajectory(40);
    assert_eq!(trajectory.len(), 40);

    // Initial state: high transmon probability, zero phonon
    assert!(trajectory[0].transmon_prob > 0.95);
    assert!(trajectory[0].phonon_prob < 0.05);

    // Mid-swap: phonon probability reaches peak >= 85%
    let max_phonon = trajectory
        .iter()
        .map(|p| p.phonon_prob)
        .fold(0.0_f64, f64::max);
    assert!(
        max_phonon >= 0.85,
        "Peak phonon occupancy must be >= 85%, got {}",
        max_phonon
    );
}

#[test]
fn test_nonreciprocal_corner_routing_and_isolation() {
    let params = EntanglementRouterParams::default();
    let router = NonReciprocalEntanglementRouter::new(params);

    let metrics = router.evaluate_routing_metrics();
    assert!(
        metrics.forward_coupling_db >= -0.60,
        "Forward coupling must have low loss (>= -0.60 dB), got {}",
        metrics.forward_coupling_db
    );
    assert!(
        -metrics.reverse_isolation_db >= 30.0,
        "Reverse isolation depth must be >= 30 dB, got {}",
        -metrics.reverse_isolation_db
    );
    assert!(
        metrics.isolation_contrast_db >= 30.0,
        "Isolation contrast must be >= 30 dB, got {}",
        metrics.isolation_contrast_db
    );
    assert!(
        metrics.crosstalk_isolation_db >= 35.0,
        "Inter-corner crosstalk isolation must be >= 35 dB, got {}",
        metrics.crosstalk_isolation_db
    );
    assert!(
        metrics.chiral_directivity > 0.99,
        "Chiral directivity must be > 0.99, got {}",
        metrics.chiral_directivity
    );
}

#[test]
fn test_bell_state_synthesis_and_concurrence() {
    let params = EntanglementRouterParams::default();
    let router = NonReciprocalEntanglementRouter::new(params);

    for target in [
        TargetEntangledState::BellPhiPlus,
        TargetEntangledState::BellPhiMinus,
        TargetEntangledState::BellPsiPlus,
        TargetEntangledState::BellPsiMinus,
    ] {
        let report = router.synthesize_entangled_state(target);
        assert!(
            report.state_fidelity >= 0.990,
            "State fidelity must be >= 0.990, got {}",
            report.state_fidelity
        );
        assert!(
            report.concurrence >= 0.95,
            "Concurrence must be >= 0.95, got {}",
            report.concurrence
        );
        assert!(
            report.purity >= 0.985,
            "Quantum state purity must be >= 0.985, got {}",
            report.purity
        );

        let rho = router.compute_density_matrix_2qubit(target);
        let trace = rho[0][0] + rho[1][1] + rho[2][2] + rho[3][3];
        assert!((trace - 1.0).abs() < 1e-6, "Density matrix trace must equal 1.0");
    }
}

#[test]
fn test_chsh_bell_inequality_violation() {
    let params = EntanglementRouterParams::default();
    let router = NonReciprocalEntanglementRouter::new(params);

    let report = router.synthesize_entangled_state(TargetEntangledState::BellPhiPlus);
    assert!(
        report.chsh_parameter >= 2.75,
        "CHSH parameter must be >= 2.75 (violating 2.0 classical limit), got {}",
        report.chsh_parameter
    );
}

#[test]
fn test_master_processor_10_point_physics_audit() {
    let processor = FloquetCornerTransducerProcessor::default();
    let audit = processor.audit_corner_transducer();

    assert!(
        audit.all_passed,
        "All 10 physics audit criteria must pass, pass_count was {}",
        audit.pass_count
    );
    assert_eq!(audit.pass_count, 10);
    assert!(audit.pass_corner_confinement);
    assert!(audit.pass_bulk_gap);
    assert!(audit.pass_transduction_efficiency);
    assert!(audit.pass_transfer_fidelity);
    assert!(audit.pass_routing_isolation);
    assert!(audit.pass_routing_contrast);
    assert!(audit.pass_crosstalk_isolation);
    assert!(audit.pass_bell_concurrence);
    assert!(audit.pass_chsh_violation);
    assert!(audit.pass_quantum_purity);
}
