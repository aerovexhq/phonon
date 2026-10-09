#![deny(unsafe_code)]

//! Comprehensive test suite for Phase 457: Topological Acoustic Floquet Corner Spin-Orbit Polariton Laser & Non-Hermitian Quantum Sensor.

use phonon_solver::floquet_corner_sensor::*;

#[test]
fn test_floquet_corner_polariton_laser_threshold_and_confinement() {
    let params = CornerPolaritonLaserParams::default();
    let solver = CornerPolaritonLaserSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Invariant 1: Lasing threshold P_th <= 15.0 mW
    assert!(
        metrics.lasing_threshold_mw <= 15.0,
        "Expected lasing threshold <= 15.0 mW, got {} mW",
        metrics.lasing_threshold_mw
    );

    // Invariant 2: Corner spatial confinement >= 85.0%
    assert!(
        metrics.corner_confinement_pct >= 85.0,
        "Expected corner confinement >= 85.0%, got {}%",
        metrics.corner_confinement_pct
    );

    // Invariant 3: Degree of circular polarization DOCP >= 90.0%
    assert!(
        metrics.circular_polarization_pct >= 90.0,
        "Expected DOCP >= 90.0%, got {}%",
        metrics.circular_polarization_pct
    );

    // Invariant 4: Emission linewidth Delta_nu <= 50.0 kHz
    assert!(
        metrics.emission_linewidth_khz <= 50.0,
        "Expected linewidth <= 50.0 kHz, got {} kHz",
        metrics.emission_linewidth_khz
    );

    // Test L-I sweep curve
    let li_curve = solver.sweep_pump_power(40);
    assert_eq!(li_curve.len(), 40);

    let sub_thresh = li_curve.iter().find(|p| p.pump_power_mw < metrics.lasing_threshold_mw * 0.5).unwrap();
    let above_thresh = li_curve.iter().find(|p| p.pump_power_mw > metrics.lasing_threshold_mw * 1.5).unwrap();

    assert!(
        above_thresh.output_power_mw > sub_thresh.output_power_mw * 10.0,
        "Expected strong lasing turn-on above threshold"
    );
    assert!(
        above_thresh.linewidth_khz < sub_thresh.linewidth_khz,
        "Expected linewidth narrowing above threshold"
    );

    // Test 2D spatial intensity profile
    let spatial_map = solver.compute_spatial_intensity(20);
    assert_eq!(spatial_map.len(), 400);

    let corner_pt = spatial_map.iter().find(|p| (p.x - 1.0).abs() < 0.1 && (p.y - 1.0).abs() < 0.1).unwrap();
    let center_pt = spatial_map.iter().find(|p| p.x.abs() < 0.1 && p.y.abs() < 0.1).unwrap();

    assert!(
        corner_pt.intensity > center_pt.intensity * 2.0,
        "Corner intensity must dominate bulk center: corner={}, center={}",
        corner_pt.intensity,
        center_pt.intensity
    );
}

#[test]
fn test_non_hermitian_mode_selector_and_smsr() {
    let params = NonHermitianModeSelectorParams::default();
    let solver = NonHermitianModeSelectorSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Invariant 1: Side-mode suppression ratio SMSR >= 35.0 dB
    assert!(
        metrics.side_mode_suppression_ratio_db >= 35.0,
        "Expected SMSR >= 35.0 dB, got {} dB",
        metrics.side_mode_suppression_ratio_db
    );

    // Invariant 2: Corner-to-edge gain contrast >= 12.0 dB
    assert!(
        metrics.corner_to_edge_gain_contrast_db >= 12.0,
        "Expected gain contrast >= 12.0 dB, got {} dB",
        metrics.corner_to_edge_gain_contrast_db
    );

    // Invariant 3: Corner mode net growth rate > 0
    assert!(
        metrics.corner_net_growth_rate_mhz > 0.0,
        "Expected corner mode to have net positive growth"
    );

    // Invariant 4: Nearest competing mode net decay rate < 0
    assert!(
        metrics.competing_mode_decay_rate_mhz < 0.0,
        "Expected competing modes to have net negative growth"
    );

    // Invariant 5: Single-mode selection invariant
    assert!(
        metrics.single_mode_selection_invariant,
        "Expected single-mode selection invariant to be true"
    );

    // Test eigenvalue spectrum
    let spectrum = solver.evaluate_eigenvalues();
    assert_eq!(spectrum.len(), 24);

    for mode in &spectrum {
        if mode.mode_label.starts_with("Corner") {
            assert!(mode.is_lasing, "Corner mode must be lasing");
            assert!(mode.imag_rate_mhz > 0.0);
        } else {
            assert!(!mode.is_lasing, "Non-corner mode must not be lasing");
            assert!(mode.imag_rate_mhz < 0.0);
        }
    }
}

#[test]
fn test_synthetic_gauge_rotation_sensor() {
    let params = SyntheticGaugeRotationParams::default();
    let solver = SyntheticGaugeRotationSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Invariant 1: Minimum detectable rotation Omega_min <= 1.0e-5 rad/s / sqrt(Hz)
    assert!(
        metrics.minimum_detectable_rotation_rad_s_sqrt_hz <= 1.0e-5,
        "Expected rotation sensitivity <= 1.0e-5 rad/s/sqrt(Hz), got {}",
        metrics.minimum_detectable_rotation_rad_s_sqrt_hz
    );

    // Invariant 2: Scale factor stability <= 10.0 ppm
    assert!(
        metrics.scale_factor_stability_ppm <= 10.0,
        "Expected scale factor stability <= 10.0 ppm, got {} ppm",
        metrics.scale_factor_stability_ppm
    );

    // Invariant 3: Dynamic range >= 80.0 dB
    assert!(
        metrics.dynamic_range_db >= 80.0,
        "Expected dynamic range >= 80.0 dB, got {} dB",
        metrics.dynamic_range_db
    );

    // Test rotation sweep anti-symmetry
    let sweep = solver.sweep_rotation_rate(31);
    assert_eq!(sweep.len(), 31);

    let zero_pt = &sweep[15]; // center index for 31 points is 15
    assert!(zero_pt.rotation_rate_deg_s.abs() < 1e-6);
    assert!(zero_pt.frequency_split_hz.abs() < 1e-6);

    let pos_pt = &sweep[25];
    let neg_pt = &sweep[5];
    assert!(
        (pos_pt.frequency_split_hz + neg_pt.frequency_split_hz).abs() < 1e-6,
        "Expected anti-symmetric frequency splitting"
    );
}

#[test]
fn test_sub_picotesla_magnetometer() {
    let params = SubPicoteslaMagnetometerParams::default();
    let solver = SubPicoteslaMagnetometerSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Invariant 1: Minimum detectable magnetic field B_min <= 0.80 pT / sqrt(Hz)
    assert!(
        metrics.minimum_detectable_field_pt_sqrt_hz <= 0.80,
        "Expected magnetic sensitivity <= 0.80 pT/sqrt(Hz), got {} pT/sqrt(Hz)",
        metrics.minimum_detectable_field_pt_sqrt_hz
    );

    // Invariant 2: Dynamic range >= 75.0 dB
    assert!(
        metrics.dynamic_range_db >= 75.0,
        "Expected dynamic range >= 75.0 dB, got {} dB",
        metrics.dynamic_range_db
    );

    // Invariant 3: Linearity error <= 0.10%
    assert!(
        metrics.linearity_error_pct <= 0.10,
        "Expected linearity error <= 0.10%, got {}%",
        metrics.linearity_error_pct
    );

    // Test magnetic field sweep
    let sweep = solver.sweep_field(25);
    assert_eq!(sweep.len(), 25);

    let zero_pt = &sweep[12]; // center index for 25 points is 12
    assert!(zero_pt.field_nt.abs() < 1e-6);
    assert!(zero_pt.frequency_shift_khz.abs() < 1e-6);
}

#[test]
fn test_10_point_physics_audit() {
    let processor = FloquetCornerSensorProcessor::new(
        CornerPolaritonLaserParams::default(),
        NonHermitianModeSelectorParams::default(),
        SyntheticGaugeRotationParams::default(),
        SubPicoteslaMagnetometerParams::default(),
    );

    let audit = processor.evaluate_audit();
    let (passed, total) = audit.score();

    println!("Phase 457 Floquet Corner Sensor Audit Score: {}/{}", passed, total);
    assert_eq!(total, 10, "Expected 10 audit criteria");
    assert_eq!(passed, 10, "Expected all 10 audit criteria to pass, got: {:?}", audit);
    assert!(audit.is_pass(), "Audit must pass 10/10");
}
