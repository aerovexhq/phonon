#![deny(unsafe_code)]

//! Comprehensive automated test suite for Phase 411:
//! Topological Chiral Acoustic Edge-Magnetoplasmon Circulator & Non-Reciprocal Quantum Hall Router.

use phonon_solver::chiral_edge_magnetoplasmon::{
    ChiralDispersionSolver, ChiralEdgeMagnetoplasmonRouter, ChiralEmpParams, DefectParams,
    EmpCirculator, EmpCirculatorParams, QuantumHallRouter, QuantumHallRouterParams,
    RouterChannel, CONDUCTANCE_QUANTUM,
};

#[test]
fn test_quantized_hall_conductance_nu1_and_nu2() {
    let p_nu1 = ChiralEmpParams::preset_nu1_high_field();
    let solver_nu1 = ChiralDispersionSolver::new(p_nu1);
    let sigma_1 = solver_nu1.quantized_hall_conductance();
    assert!((sigma_1 - CONDUCTANCE_QUANTUM).abs() < 1e-12);

    let p_nu2 = ChiralEmpParams::preset_nu2_gaas();
    let solver_nu2 = ChiralDispersionSolver::new(p_nu2);
    let sigma_2 = solver_nu2.quantized_hall_conductance();
    assert!((sigma_2 - 2.0 * CONDUCTANCE_QUANTUM).abs() < 1e-12);
}

#[test]
fn test_cyclotron_frequency_and_gap() {
    let solver = ChiralDispersionSolver::new(ChiralEmpParams::default());
    let fc_ghz = solver.cyclotron_frequency_ghz();
    assert!(fc_ghz > 1.0e3, "Cyclotron frequency should be > 1.0 THz (1000 GHz), got {:.1}", fc_ghz);

    let gap_mev = solver.cyclotron_energy_mev();
    let th_mev = solver.thermal_energy_mev();
    assert!(gap_mev > 5.0 * th_mev, "Cyclotron gap must exceed 5 * k_B*T");
    assert!(solver.is_quantum_hall_regime_valid());
}

#[test]
fn test_acoustic_velocity_non_reciprocity() {
    let solver = ChiralDispersionSolver::new(ChiralEmpParams::default());
    let v_fwd = solver.acoustic_forward_velocity_ms();
    let v_bwd = solver.acoustic_backward_velocity_ms();

    assert!(v_fwd > v_bwd, "Forward velocity must exceed backward velocity in chiral regime");
    let eta_nr = solver.velocity_non_reciprocity_ratio();
    assert!(eta_nr >= 0.02, "Non-reciprocity ratio eta_nr should be >= 2.0%, got {:.4}", eta_nr);

    let curve = solver.compute_dispersion_curve(0.1, 10.0, 50);
    assert_eq!(curve.len(), 50);
    for pt in &curve {
        assert!(pt.emp_freq_ghz > 0.0);
        assert!(pt.hybrid_fwd_freq_ghz > pt.hybrid_bwd_freq_ghz);
    }
}

#[test]
fn test_circulator_s_matrix_isolation_and_return_loss() {
    let circ = EmpCirculator::new(EmpCirculatorParams::default());
    let sm = circ.evaluate_s_matrix(circ.params.center_freq_ghz);

    let il_db = circ.forward_insertion_loss_db();
    assert!(il_db <= 0.50, "Forward insertion loss must be <= 0.50 dB, got {:.2} dB", il_db);

    let iso_db = circ.reverse_isolation_db();
    assert!(iso_db >= 35.0, "Reverse isolation must be >= 35.0 dB, got {:.2} dB", iso_db);

    let rl_db = circ.return_loss_db();
    assert!(rl_db >= 25.0, "Return loss must be >= 25.0 dB, got {:.2} dB", rl_db);

    // Cyclic symmetry: S21 == S32 == S13, S12 == S23 == S31
    assert!((sm.s21_mag - sm.s32_mag).abs() < 1e-5);
    assert!((sm.s21_mag - sm.s13_mag).abs() < 1e-5);
    assert!((sm.s12_mag - sm.s23_mag).abs() < 1e-5);
    assert!((sm.s12_mag - sm.s31_mag).abs() < 1e-5);

    // Unitarity error check
    let u_err = sm.unitarity_error();
    assert!(u_err <= 0.20, "Frobenius unitarity error must be <= 0.20 for lossy circulator, got {:.4}", u_err);

    // Spectrum sweep
    let sweep = circ.compute_spectrum_sweep(2.5, 3.5, 40);
    assert_eq!(sweep.len(), 40);
}

#[test]
fn test_topological_corner_transmission() {
    let router = QuantumHallRouter::new(QuantumHallRouterParams::default());
    let t_corner = router.compute_corner_transmission();
    assert!(t_corner >= 0.95, "Corner transmission must be >= 95.0%, got {:.3}", t_corner);

    let rl_corner = router.compute_corner_return_loss_db();
    assert!(rl_corner >= 25.0, "Corner return loss must be >= 25.0 dB, got {:.2} dB", rl_corner);
}

#[test]
fn test_defect_backscattering_suppression() {
    let mut params = QuantumHallRouterParams::default();
    params.defect = DefectParams {
        defect_present: true,
        defect_depth_um: 2.5,
        barrier_height_mev: 18.0,
    };
    let router = QuantumHallRouter::new(params);
    let (t_defect, suppression_db) = router.compute_defect_metrics();

    assert!(t_defect >= 0.95, "Defect transmission must be >= 95.0%, got {:.3}", t_defect);
    assert!(suppression_db >= 30.0, "Backscattering suppression must be >= 30.0 dB, got {:.2} dB", suppression_db);
}

#[test]
fn test_split_gate_routing_and_isolation() {
    // Channel A configuration (gate 0.0V)
    let p_a = QuantumHallRouterParams {
        split_gate_voltage_v: 0.0,
        pinch_off_voltage_v: -1.8,
        ..Default::default()
    };
    let router_a = QuantumHallRouter::new(p_a);
    let metrics_a = router_a.evaluate_transport_metrics();
    assert_eq!(metrics_a.active_channel, RouterChannel::ChannelA);
    assert!(metrics_a.channel_a_power >= 0.94);
    assert!(metrics_a.cross_channel_isolation_db >= 35.0);

    // Channel B configuration (gate -2.5V, pinched off)
    let p_b = QuantumHallRouterParams {
        split_gate_voltage_v: -2.5,
        pinch_off_voltage_v: -1.8,
        ..Default::default()
    };
    let router_b = QuantumHallRouter::new(p_b);
    let metrics_b = router_b.evaluate_transport_metrics();
    assert_eq!(metrics_b.active_channel, RouterChannel::ChannelB);
    assert!(metrics_b.channel_b_power >= 0.94);
    assert!(metrics_b.cross_channel_isolation_db >= 35.0);
}

#[test]
fn test_10_point_physics_audit_pass() {
    let system = ChiralEdgeMagnetoplasmonRouter::default();
    let audit = system.audit_chiral_emp();

    assert_eq!(audit.total_count, 10);
    assert_eq!(audit.passed_count, 10, "Expected all 10 criteria to pass, got: {:?}", audit.criteria);
    assert!(audit.all_passed);
}
