//! Integration tests for JTWPA periodic dispersion engineering and Kerr-free SNAIL elements.

use phonon_models::jtwpa::{DispersionEngineeringParams, SnailElementParams};

#[test]
fn test_dispersion_engineering_parameters() {
    let disp = DispersionEngineeringParams::standard_50ohm_jtwpa();
    assert!((disp.characteristic_impedance_ohms() - 50.0).abs() < 1e-4);

    let vp = disp.phase_velocity_cells_per_s();
    assert!(vp > 1.0e9);

    let omega_s = 2.0 * std::f64::consts::PI * 6.0e9;
    let k0 = disp.linear_wavenumber_rad_per_cell(omega_s);
    assert!(k0 > 0.0);

    let k_eng = disp.engineered_wavenumber_rad_per_cell(omega_s);
    assert!(k_eng > 0.0);

    // Test 4WM phase mismatch near pump frequency
    let omega_p = 2.0 * std::f64::consts::PI * 8.0e9;
    let omega_i = 2.0 * std::f64::consts::PI * 10.0e9;
    let dk = disp.four_wave_phase_mismatch(omega_p, omega_s, omega_i);
    assert!(dk.is_finite());
}

#[test]
fn test_snail_element_kerr_free_operation() {
    let snail = SnailElementParams::standard_kerr_free_snail();
    assert_eq!(snail.num_large_junctions, 3);
    assert_eq!(snail.junction_ratio_alpha, 0.29);

    let phi0 = snail.minimum_phase_rad();
    assert!(phi0 > 0.0 && phi0 < std::f64::consts::PI);

    let c2 = snail.c2_coefficient();
    assert!(c2 > 0.0, "c2 must be positive for inductive stability");

    let c3 = snail.c3_coefficient();
    assert!(c3.abs() > 0.01, "c3 (3WM non-linearity) must be non-zero");

    let c4 = snail.c4_coefficient();
    assert!(
        snail.is_kerr_free(),
        "SNAIL at optimal flux must be Kerr-free (|c4| < 0.05), got c4 = {}",
        c4
    );

    let p_1db = snail.saturation_power_1db_dbm();
    assert!(
        p_1db > -90.0,
        "Kerr-free SNAIL saturation power must exceed -90 dBm, got {} dBm",
        p_1db
    );
}
