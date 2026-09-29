#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Hermitian
//! skin-topological phonon diodes and unidirectional quantum acoustic amplifiers.

use phonon_models::non_hermitian_skin_amplifier::NonHermitianSkinAmplifierParams;
use phonon_solver::non_hermitian_skin_amplifier::NonHermitianSkinAmplifierSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = NonHermitianSkinAmplifierParams::new(
        0.5,    // below 1.0 GHz
        5,      // below 10 sites
        5.0,    // below 10.0 MHz
        0.1,    // below 0.5 MHz
        2.0,    // below 5.0 MHz
        0.5,    // below 1.0 MHz
        0.5,    // below 1.0 mK
        -70.0,  // below -60.0 dBm
    );
    assert!((underflow.center_frequency_ghz - 1.0).abs() < 1e-9);
    assert_eq!(underflow.lattice_sites_count, 10);
    assert!((underflow.forward_coupling_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.reverse_coupling_mhz - 0.5).abs() < 1e-9);
    assert!((underflow.parametric_pump_rate_mhz - 5.0).abs() < 1e-9);
    assert!((underflow.dissipation_gradient_mhz - 1.0).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.input_signal_power_dbm - (-60.0)).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = NonHermitianSkinAmplifierParams::new(
        15.0,   // above 12.0 GHz
        150,    // above 100 sites
        150.0,  // above 100.0 MHz
        30.0,   // above 20.0 MHz
        70.0,   // above 50.0 MHz
        45.0,   // above 30.0 MHz
        75.0,   // above 50.0 mK
        -5.0,   // above -10.0 dBm
    );
    assert!((overflow.center_frequency_ghz - 12.0).abs() < 1e-9);
    assert_eq!(overflow.lattice_sites_count, 100);
    assert!((overflow.forward_coupling_mhz - 100.0).abs() < 1e-9);
    assert!((overflow.reverse_coupling_mhz - 20.0).abs() < 1e-9);
    assert!((overflow.parametric_pump_rate_mhz - 50.0).abs() < 1e-9);
    assert!((overflow.dissipation_gradient_mhz - 30.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.input_signal_power_dbm - (-10.0)).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = NonHermitianSkinAmplifierParams::default();
    let solver = NonHermitianSkinAmplifierSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.forward_gain_db >= 28.0,
        "Default forward gain must be >= 28.0 dB, got {:.4} dB",
        metrics.forward_gain_db
    );
    assert!(
        metrics.reverse_isolation_db >= 42.0,
        "Default reverse isolation must be >= 42.0 dB, got {:.4} dB",
        metrics.reverse_isolation_db
    );
    assert!(
        metrics.added_noise_quanta <= 0.250,
        "Default added noise must be <= 0.250 quanta, got {:.4} quanta",
        metrics.added_noise_quanta
    );
    assert!(
        metrics.power_saturation_threshold_dbm >= -15.0,
        "Default power saturation threshold must be >= -15.0 dBm, got {:.4} dBm",
        metrics.power_saturation_threshold_dbm
    );
    assert!(
        metrics.skin_mode_localization_ratio >= 0.900,
        "Default skin mode localization ratio must be >= 0.900, got {:.4}",
        metrics.skin_mode_localization_ratio
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_coupling_asymmetry_scaling() {
    let base = NonHermitianSkinAmplifierParams::default();
    let solver_base = NonHermitianSkinAmplifierSolver::new(base);
    let gain_base = solver_base.compute_forward_gain_db();
    let iso_base = solver_base.compute_reverse_isolation_db();
    let loc_base = solver_base.compute_skin_mode_localization_ratio();

    // Increase forward coupling and decrease reverse coupling (higher asymmetry)
    let high_asymmetry = NonHermitianSkinAmplifierParams::new(
        base.center_frequency_ghz,
        base.lattice_sites_count,
        60.0, // increased from 45.0 MHz
        2.5,  // decreased from 5.0 MHz
        base.parametric_pump_rate_mhz,
        base.dissipation_gradient_mhz,
        base.operating_temp_m_k,
        base.input_signal_power_dbm,
    );
    let solver_high = NonHermitianSkinAmplifierSolver::new(high_asymmetry);
    let gain_high = solver_high.compute_forward_gain_db();
    let iso_high = solver_high.compute_reverse_isolation_db();
    let loc_high = solver_high.compute_skin_mode_localization_ratio();

    assert!(
        gain_high > gain_base,
        "Higher coupling asymmetry must increase forward gain: {:.4} dB vs {:.4} dB",
        gain_high, gain_base
    );
    assert!(
        iso_high > iso_base,
        "Higher coupling asymmetry must increase reverse isolation: {:.4} dB vs {:.4} dB",
        iso_high, iso_base
    );
    assert!(
        loc_high > loc_base,
        "Higher coupling asymmetry must enhance skin mode localization: {:.4} vs {:.4}",
        loc_high, loc_base
    );
}

#[test]
fn test_temperature_degradation() {
    let base = NonHermitianSkinAmplifierParams::default();
    let solver_base = NonHermitianSkinAmplifierSolver::new(base);
    let gain_base = solver_base.compute_forward_gain_db();
    let iso_base = solver_base.compute_reverse_isolation_db();
    let psat_base = solver_base.compute_power_saturation_threshold_dbm();
    let loc_base = solver_base.compute_skin_mode_localization_ratio();

    // Elevated cryogenic temperature (30.0 mK vs 15.0 mK)
    let warmer = NonHermitianSkinAmplifierParams::new(
        base.center_frequency_ghz,
        base.lattice_sites_count,
        base.forward_coupling_mhz,
        base.reverse_coupling_mhz,
        base.parametric_pump_rate_mhz,
        base.dissipation_gradient_mhz,
        30.0, // increased from 15.0 mK
        base.input_signal_power_dbm,
    );
    let solver_warmer = NonHermitianSkinAmplifierSolver::new(warmer);
    let gain_warmer = solver_warmer.compute_forward_gain_db();
    let iso_warmer = solver_warmer.compute_reverse_isolation_db();
    let psat_warmer = solver_warmer.compute_power_saturation_threshold_dbm();
    let loc_warmer = solver_warmer.compute_skin_mode_localization_ratio();

    assert!(
        gain_warmer < gain_base,
        "Elevated temperature must degrade forward gain: {:.4} dB vs {:.4} dB",
        gain_warmer, gain_base
    );
    assert!(
        iso_warmer < iso_base,
        "Elevated temperature must degrade reverse isolation: {:.4} dB vs {:.4} dB",
        iso_warmer, iso_base
    );
    assert!(
        psat_warmer < psat_base,
        "Elevated temperature must lower power saturation threshold: {:.4} dBm vs {:.4} dBm",
        psat_warmer, psat_base
    );
    assert!(
        loc_warmer < loc_base,
        "Elevated temperature must reduce skin mode localization: {:.4} vs {:.4}",
        loc_warmer, loc_base
    );
}

#[test]
fn test_noise_scaling() {
    let base = NonHermitianSkinAmplifierParams::default();
    let solver_base = NonHermitianSkinAmplifierSolver::new(base);
    let noise_base = solver_base.compute_added_noise_quanta();

    // Warmer bath temperature
    let warmer = NonHermitianSkinAmplifierParams::new(
        base.center_frequency_ghz,
        base.lattice_sites_count,
        base.forward_coupling_mhz,
        base.reverse_coupling_mhz,
        base.parametric_pump_rate_mhz,
        base.dissipation_gradient_mhz,
        25.0, // warmer
        base.input_signal_power_dbm,
    );
    let solver_warmer = NonHermitianSkinAmplifierSolver::new(warmer);
    let noise_warmer = solver_warmer.compute_added_noise_quanta();

    assert!(
        noise_warmer > noise_base,
        "Higher temperature must increase added noise quanta: {:.4} vs {:.4}",
        noise_warmer, noise_base
    );

    // Higher reverse coupling
    let higher_rev = NonHermitianSkinAmplifierParams::new(
        base.center_frequency_ghz,
        base.lattice_sites_count,
        base.forward_coupling_mhz,
        10.0, // increased reverse coupling
        base.parametric_pump_rate_mhz,
        base.dissipation_gradient_mhz,
        base.operating_temp_m_k,
        base.input_signal_power_dbm,
    );
    let solver_rev = NonHermitianSkinAmplifierSolver::new(higher_rev);
    let noise_rev = solver_rev.compute_added_noise_quanta();

    assert!(
        noise_rev > noise_base,
        "Higher reverse coupling must increase added noise quanta: {:.4} vs {:.4}",
        noise_rev, noise_base
    );
}

#[test]
fn test_skin_mode_localization_ratio_bounds() {
    let p_min = NonHermitianSkinAmplifierParams::new(1.0, 10, 10.0, 20.0, 5.0, 1.0, 50.0, -60.0);
    let solver_min = NonHermitianSkinAmplifierSolver::new(p_min);
    let loc_min = solver_min.compute_skin_mode_localization_ratio();
    assert!(loc_min >= 0.900 && loc_min <= 0.995, "Localization must be bounded: {:.4}", loc_min);

    let p_max = NonHermitianSkinAmplifierParams::new(12.0, 100, 100.0, 0.5, 50.0, 30.0, 1.0, -10.0);
    let solver_max = NonHermitianSkinAmplifierSolver::new(p_max);
    let loc_max = solver_max.compute_skin_mode_localization_ratio();
    assert!(loc_max >= 0.900 && loc_max <= 0.995, "Localization must be bounded: {:.4}", loc_max);
}
