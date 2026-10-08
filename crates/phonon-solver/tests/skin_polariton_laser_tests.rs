#![deny(unsafe_code)]

use phonon_solver::skin_polariton_laser::{
    ChiralSagnacGyroscopeSolver, GyroscopeSagnacParams, PolaritonGainMediumSolver,
    PolaritonGainParams, RiemannEnergyWindingSolver, RiemannWindingParams,
    SkinEffectLasingParams, SkinEffectLasingSolver, SkinPolaritonLaserProcessor,
};

#[test]
fn test_skin_effect_lasing_nhse_metrics() {
    let params = SkinEffectLasingParams::default();
    let solver = SkinEffectLasingSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Verify asymmetric hopping contrast J_R / J_L >= 3.0
    assert!(
        metrics.hopping_asymmetry_ratio >= 3.0,
        "Hopping asymmetry ratio {} should be >= 3.0",
        metrics.hopping_asymmetry_ratio
    );

    // Verify skin penetration depth xi <= 3.5 cells
    assert!(
        metrics.skin_depth_cells <= 3.5,
        "Skin depth {} should be <= 3.5 cells",
        metrics.skin_depth_cells
    );

    // Verify boundary modal localization ratio >= 88.0%
    assert!(
        metrics.boundary_localization_ratio >= 0.88,
        "Boundary localization {} should be >= 0.88",
        metrics.boundary_localization_ratio
    );

    // Verify side-mode suppression ratio SMSR >= 32.0 dB
    assert!(
        metrics.side_mode_suppression_ratio_db >= 32.0,
        "SMSR {} dB should be >= 32.0 dB",
        metrics.side_mode_suppression_ratio_db
    );

    // Verify net modal gain >= 2.5 MHz
    assert!(
        metrics.net_modal_gain_mhz >= 2.5,
        "Net modal gain {} MHz should be >= 2.5 MHz",
        metrics.net_modal_gain_mhz
    );

    // Verify spatial mode profile generation
    let spatial = solver.compute_spatial_mode_profile();
    assert_eq!(spatial.len(), params.lattice_site_count);
    // Boundary site (last site) should have higher energy density than first site
    assert!(spatial.last().unwrap().energy_density > spatial.first().unwrap().energy_density);
}

#[test]
fn test_riemann_energy_winding_and_gbz() {
    let params = RiemannWindingParams::default();
    let solver = RiemannEnergyWindingSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify point-gap topological winding number W == +1
    assert_eq!(
        metrics.point_gap_winding_number, 1,
        "Point-gap winding number {} must be +1",
        metrics.point_gap_winding_number
    );

    // Verify GBZ deformation magnitude |r_GBZ - 1.0| >= 0.30
    assert!(
        metrics.gbz_deformation_magnitude >= 0.30,
        "GBZ deformation magnitude {} should be >= 0.30",
        metrics.gbz_deformation_magnitude
    );

    // Verify GBZ radius r_GBZ < 1.0 (localized at boundary)
    assert!(
        metrics.gbz_radius < 1.0,
        "GBZ radius {} should be < 1.0",
        metrics.gbz_radius
    );

    // Verify Riemann spectrum points
    let points = solver.compute_complex_energy_loop(180);
    assert_eq!(points.len(), 180);
    for pt in points {
        assert!(pt.bloch_momentum_rad >= -std::f64::consts::PI - 1e-6);
        assert!(pt.bloch_momentum_rad <= std::f64::consts::PI + 1e-6);
    }
}

#[test]
fn test_chiral_sagnac_gyroscope_enhancement() {
    let params = GyroscopeSagnacParams::default();
    let solver = ChiralSagnacGyroscopeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify sensitivity enhancement factor eta >= 45.0x
    assert!(
        metrics.sensitivity_enhancement_factor >= 45.0,
        "Sensitivity enhancement factor {} should be >= 45.0x",
        metrics.sensitivity_enhancement_factor
    );

    // Verify Angle Random Walk ARW <= 0.008 deg / sqrt(h)
    assert!(
        metrics.angle_random_walk_deg_sqrt_h <= 0.008,
        "ARW {} deg/sqrt(h) should be <= 0.008",
        metrics.angle_random_walk_deg_sqrt_h
    );

    // Verify bias instability <= 0.05 deg / h
    assert!(
        metrics.bias_instability_deg_h <= 0.05,
        "Bias instability {} deg/h should be <= 0.05",
        metrics.bias_instability_deg_h
    );

    // Verify scale factor stability <= 15.0 ppm
    assert!(
        metrics.scale_factor_stability_ppm <= 15.0,
        "Scale factor stability {} ppm should be <= 15.0",
        metrics.scale_factor_stability_ppm
    );

    // Verify dynamic range >= 120.0 dB
    assert!(
        metrics.dynamic_range_db >= 120.0,
        "Dynamic range {} dB should be >= 120.0",
        metrics.dynamic_range_db
    );

    // Verify rotation response curve
    let curve = solver.compute_response_curve(120);
    assert_eq!(curve.len(), 120);
}

#[test]
fn test_polariton_gain_medium_kinetics() {
    let params = PolaritonGainParams::default();
    let solver = PolaritonGainMediumSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify lasing threshold power P_th <= 1.80 mW
    assert!(
        metrics.threshold_power_mw <= 1.80,
        "Lasing threshold {} mW should be <= 1.80 mW",
        metrics.threshold_power_mw
    );

    // Verify slope efficiency >= 42.0%
    assert!(
        metrics.slope_efficiency_pct >= 42.0,
        "Slope efficiency {} % should be >= 42.0%",
        metrics.slope_efficiency_pct
    );

    // Verify Schawlow-Townes linewidth Delta nu <= 12.0 kHz
    assert!(
        metrics.schawlow_townes_linewidth_khz <= 12.0,
        "Linewidth {} kHz should be <= 12.0 kHz",
        metrics.schawlow_townes_linewidth_khz
    );

    // Verify relative intensity noise RIN <= -145.0 dBc/Hz
    assert!(
        metrics.relative_intensity_noise_dbc_hz <= -145.0,
        "RIN {} dBc/Hz should be <= -145.0",
        metrics.relative_intensity_noise_dbc_hz
    );

    // Verify power curve
    let power_curve = solver.compute_power_curve(100);
    assert_eq!(power_curve.len(), 100);
}

#[test]
fn test_skin_polariton_laser_10_point_audit() {
    let processor = SkinPolaritonLaserProcessor::default();
    let report = processor.evaluate_audit();

    println!("{}", report.summary());

    assert!(report.asymmetric_hopping_contrast_pass, "Criterion 1 failed");
    assert!(report.skin_depth_pass, "Criterion 2 failed");
    assert!(report.boundary_localization_pass, "Criterion 3 failed");
    assert!(report.side_mode_suppression_pass, "Criterion 4 failed");
    assert!(report.point_gap_winding_pass, "Criterion 5 failed");
    assert!(report.gbz_deformation_pass, "Criterion 6 failed");
    assert!(report.gyro_enhancement_pass, "Criterion 7 failed");
    assert!(report.angle_random_walk_pass, "Criterion 8 failed");
    assert!(report.laser_threshold_pass, "Criterion 9 failed");
    assert!(report.linewidth_narrowing_pass, "Criterion 10 failed");

    assert!(report.all_passed(), "Expected all 10 physics audit criteria to pass");
    assert_eq!(report.score(), (10, 10));
}
