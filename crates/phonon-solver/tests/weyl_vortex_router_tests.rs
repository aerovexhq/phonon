#![deny(unsafe_code)]

//! Integration and verification test suite for Phase 460:
//! Topological Acoustic Floquet Higher-Order Weyl Semimetal Vortex Transceiver & Multi-Terminal Quantum Acoustic Router.

use phonon_solver::weyl_vortex_router::{
    HigherOrderWeylParams, HigherOrderWeylSolver, MultiTerminalRouterParams,
    MultiTerminalRouterSolver, WeylVortexParams, WeylVortexRouterProcessor, WeylVortexSolver,
};

#[test]
fn test_higher_order_weyl_semimetal_lattice() {
    let params = HigherOrderWeylParams {
        lattice_constant_um: 100.0,
        hopping_tx_mhz: 12.0,
        hopping_ty_mhz: 12.0,
        hopping_tz_mhz: 10.0,
        mass_m0_mhz: 4.5,
        floquet_drive_mhz: 6.0,
        grid_dim: 16,
    };
    let solver = HigherOrderWeylSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        (metrics.monopole_charge.abs() - 1.0).abs() <= 0.02,
        "Monopole charge {} is not quantized to 1.0",
        metrics.monopole_charge
    );
    assert!(
        metrics.monopole_quantization_error <= 0.02,
        "Monopole quantization error {} > 0.02",
        metrics.monopole_quantization_error
    );
    assert!(
        metrics.weyl_node_separation_inv_um >= 0.015,
        "Weyl node separation {} 1/um < 0.015 1/um",
        metrics.weyl_node_separation_inv_um
    );
    assert!(
        metrics.hinge_group_velocity_ms >= 500.0,
        "Hinge mode group velocity {} m/s < 500.0 m/s",
        metrics.hinge_group_velocity_ms
    );
    assert!(
        metrics.hinge_confinement_percent >= 85.0,
        "Hinge confinement {} % < 85.0 %",
        metrics.hinge_confinement_percent
    );
    assert!(metrics.bulk_gap_mhz > 0.0);

    // Verify 1D hinge dispersion along kz
    let dispersion = solver.compute_dispersion(25);
    assert_eq!(dispersion.len(), 25);
    let mid = &dispersion[12];
    assert!(mid.energy_bulk_upper_mhz > mid.energy_bulk_lower_mhz);

    // Verify 2D transverse spatial hinge mode profile
    let profile = solver.compute_spatial_hinge_profile();
    assert_eq!(profile.len(), 16 * 16);
    let hinge_points: Vec<_> = profile.iter().filter(|p| p.is_hinge).collect();
    assert_eq!(hinge_points.len(), 4);
    for hp in &hinge_points {
        assert!(hp.acoustic_intensity > 0.80);
    }
}

#[test]
fn test_acoustic_vortex_transceiver_metrics_and_profiles() {
    let params = WeylVortexParams {
        topological_charge_l: 1,
        beam_waist_um: 250.0,
        carrier_freq_mhz: 5.0,
        acoustic_velocity_ms: 1500.0,
        metasurface_efficiency: 0.92,
    };
    let solver = WeylVortexSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert_eq!(metrics.measured_topological_charge, 1);
    assert!(
        metrics.oam_mode_purity_percent >= 90.0,
        "OAM mode purity {} % < 90.0 %",
        metrics.oam_mode_purity_percent
    );
    assert!(
        metrics.vortex_generation_efficiency_percent >= 80.0,
        "Vortex generation efficiency {} % < 80.0 %",
        metrics.vortex_generation_efficiency_percent
    );
    assert!(
        metrics.core_null_depth_db >= 25.0,
        "Core null depth {} dB < 25.0 dB",
        metrics.core_null_depth_db
    );
    assert!(metrics.rayleigh_range_mm > 0.0);

    // Test l = 2 case
    let solver_l2 = WeylVortexSolver::new(WeylVortexParams {
        topological_charge_l: 2,
        ..Default::default()
    });
    let m_l2 = solver_l2.evaluate_metrics();
    assert_eq!(m_l2.measured_topological_charge, 2);
    assert!(m_l2.oam_mode_purity_percent >= 90.0);
    assert!(m_l2.vortex_generation_efficiency_percent >= 80.0);

    // Verify radial profile: zero at r = 0, peak at r > 0
    let radial = solver.generate_radial_profile(35);
    assert_eq!(radial.len(), 35);
    assert!(radial[0].intensity < 1e-4, "Expected zero core intensity");
    let max_intensity = radial.iter().map(|p| p.intensity).fold(0.0f64, f64::max);
    assert!(max_intensity > 0.95, "Peak intensity {} < 0.95", max_intensity);
}

#[test]
fn test_multi_terminal_chiral_router_s_parameters() {
    let params = MultiTerminalRouterParams {
        center_freq_mhz: 5.0,
        bandwidth_khz: 320.0,
        port_count: 6,
        corner_defect_ratio: 0.12,
        chiral_bias_phase_rad: std::f64::consts::FRAC_PI_3,
    };
    let solver = MultiTerminalRouterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.forward_insertion_loss_db <= 0.40,
        "Forward insertion loss {} dB > 0.40 dB",
        metrics.forward_insertion_loss_db
    );
    assert!(
        metrics.backward_isolation_db >= 38.0,
        "Backward isolation {} dB < 38.0 dB",
        metrics.backward_isolation_db
    );
    assert!(
        metrics.port_return_loss_db >= 22.0,
        "Port return loss {} dB < 22.0 dB",
        metrics.port_return_loss_db
    );
    assert!(
        metrics.corner_defect_retention_percent >= 95.0,
        "Defect retention {} % < 95.0 %",
        metrics.corner_defect_retention_percent
    );
    assert!(metrics.cyclic_symmetry_error < 1e-3);

    // Verify 6x6 S-matrix
    let s_matrix = solver.compute_s_matrix();
    assert_eq!(s_matrix.len(), 6);
    assert_eq!(s_matrix[0].len(), 6);

    // Forward transmission from port 0 to port 1
    assert!(
        s_matrix[1][0] > 0.90,
        "Forward transmission |S_21| too low: {}",
        s_matrix[1][0]
    );
    // Backward transmission from port 1 to port 0 (isolated)
    assert!(
        s_matrix[5][0] < 0.02,
        "Backward isolation |S_61| not isolated: {}",
        s_matrix[5][0]
    );
    // Return loss |S_11|
    assert!(
        s_matrix[0][0] < 0.10,
        "Return loss |S_11| too high: {}",
        s_matrix[0][0]
    );

    // Frequency sweep spectrum
    let spectrum = solver.sweep_frequency(25);
    assert_eq!(spectrum.len(), 25);
    let center = &spectrum[12];
    assert!(center.s_forward_db > -0.50);
    assert!(center.s_backward_db < -35.0);
}

#[test]
fn test_vortex_spatial_phase_and_doughnut_topology() {
    let solver = WeylVortexSolver::new(WeylVortexParams {
        topological_charge_l: 1,
        beam_waist_um: 200.0,
        ..Default::default()
    });
    let grid = solver.generate_2d_slice(21);
    assert_eq!(grid.len(), 21 * 21);

    // Center point (index 220 in 21x21 grid: ix = 10, iy = 10)
    let center = &grid[10 * 21 + 10];
    assert!(center.intensity < 1e-3);

    // Outer points on ring should have high intensity
    let ring_point = &grid[10 * 21 + 14]; // shifted in x
    assert!(ring_point.intensity > 0.30);
}

#[test]
fn test_10_point_physics_audit() {
    let processor = WeylVortexRouterProcessor::new(
        HigherOrderWeylParams::default(),
        WeylVortexParams::default(),
        MultiTerminalRouterParams::default(),
    );

    let audit = processor.audit_system();
    let (passed, total) = audit.score();

    assert_eq!(total, 10);
    assert_eq!(
        passed, 10,
        "Audit failed with score {}/{}: {:?}",
        passed, total, audit
    );
    assert!(audit.is_pass());
}
