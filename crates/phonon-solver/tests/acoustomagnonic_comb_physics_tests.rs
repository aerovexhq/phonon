#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for chiral phonon-magnon
//! polariton frequency combs and quantum topological acoustomagnonics.

use phonon_models::acoustomagnonic_comb::AcoustomagnonicCombParams;
use phonon_solver::acoustomagnonic_comb::AcoustomagnonicCombSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = AcoustomagnonicCombParams::new(
        2.0,    // below 5.0 GHz
        10.0,   // below 20.0 MHz
        0.5,    // below 1.0 kHz
        1.0e-6, // below 1.0e-5
        0.01,   // below 0.05 MHz
        0.5,    // below 1.0 mK
        0.5,    // below 1.0 mW
        0.30,   // below 0.50
    );
    assert!((underflow.pump_frequency_ghz - 5.0).abs() < 1e-9);
    assert!((underflow.magnetoelastic_coupling_mhz - 20.0).abs() < 1e-9);
    assert!((underflow.kerr_nonlinearity_khz - 1.0).abs() < 1e-9);
    assert!((underflow.gilbert_damping_alpha - 1.0e-5).abs() < 1e-12);
    assert!((underflow.acoustic_loss_rate_mhz - 0.05).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.rf_drive_power_mw - 1.0).abs() < 1e-9);
    assert!((underflow.chiral_asymmetry_ratio - 0.50).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = AcoustomagnonicCombParams::new(
        40.0,   // above 30.0 GHz
        250.0,  // above 200.0 MHz
        80.0,   // above 50.0 kHz
        5.0e-3, // above 1.0e-3
        10.0,   // above 5.0 MHz
        150.0,  // above 100.0 mK
        150.0,  // above 100.0 mW
        1.50,   // above 0.99
    );
    assert!((overflow.pump_frequency_ghz - 30.0).abs() < 1e-9);
    assert!((overflow.magnetoelastic_coupling_mhz - 200.0).abs() < 1e-9);
    assert!((overflow.kerr_nonlinearity_khz - 50.0).abs() < 1e-9);
    assert!((overflow.gilbert_damping_alpha - 1.0e-3).abs() < 1e-12);
    assert!((overflow.acoustic_loss_rate_mhz - 5.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 100.0).abs() < 1e-9);
    assert!((overflow.rf_drive_power_mw - 100.0).abs() < 1e-9);
    assert!((overflow.chiral_asymmetry_ratio - 0.99).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AcoustomagnonicCombParams::default();
    let solver = AcoustomagnonicCombSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.comb_spectral_span_ghz >= 60.0,
        "Default comb spectral span must be >= 60.0 GHz, got {:.4} GHz",
        metrics.comb_spectral_span_ghz
    );
    assert!(
        metrics.phase_noise_at_10khz_dbc <= -125.0,
        "Default phase noise at 10 kHz must be <= -125.0 dBc/Hz, got {:.4} dBc/Hz",
        metrics.phase_noise_at_10khz_dbc
    );
    assert!(
        metrics.polariton_conversion_efficiency >= 0.880,
        "Default conversion efficiency must be >= 0.880, got {:.4}",
        metrics.polariton_conversion_efficiency
    );
    assert!(
        metrics.inter_modal_isolation_db >= 32.0,
        "Default inter-modal isolation must be >= 32.0 dB, got {:.4} dB",
        metrics.inter_modal_isolation_db
    );
    assert!(
        metrics.polariton_cooperativity >= 80.0,
        "Default polariton cooperativity must be >= 80.0, got {:.4}",
        metrics.polariton_cooperativity
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_coupling_scaling() {
    let base = AcoustomagnonicCombParams::default();
    let solver_base = AcoustomagnonicCombSolver::new(base);
    let span_base = solver_base.compute_comb_spectral_span_ghz();
    let eff_base = solver_base.compute_polariton_conversion_efficiency();
    let coop_base = solver_base.compute_polariton_cooperativity();
    let pn_base = solver_base.compute_phase_noise_at_10khz_dbc();

    // Increase magnetoelastic coupling
    let higher_coupling = AcoustomagnonicCombParams::new(
        base.pump_frequency_ghz,
        110.0, // increased from 85.0 MHz
        base.kerr_nonlinearity_khz,
        base.gilbert_damping_alpha,
        base.acoustic_loss_rate_mhz,
        base.operating_temp_m_k,
        base.rf_drive_power_mw,
        base.chiral_asymmetry_ratio,
    );
    let solver_higher = AcoustomagnonicCombSolver::new(higher_coupling);
    let span_higher = solver_higher.compute_comb_spectral_span_ghz();
    let eff_higher = solver_higher.compute_polariton_conversion_efficiency();
    let coop_higher = solver_higher.compute_polariton_cooperativity();
    let pn_higher = solver_higher.compute_phase_noise_at_10khz_dbc();

    assert!(
        span_higher > span_base,
        "Higher magnetoelastic coupling must increase comb spectral span: {:.4} > {:.4}",
        span_higher,
        span_base
    );
    assert!(
        eff_higher > eff_base,
        "Higher magnetoelastic coupling must increase polariton conversion efficiency: {:.4} > {:.4}",
        eff_higher,
        eff_base
    );
    assert!(
        coop_higher > coop_base,
        "Higher magnetoelastic coupling must increase polariton cooperativity: {:.4} > {:.4}",
        coop_higher,
        coop_base
    );
    assert!(
        pn_higher <= pn_base,
        "Higher magnetoelastic coupling must improve phase noise (more negative): {:.4} <= {:.4}",
        pn_higher,
        pn_base
    );
}

#[test]
fn test_power_scaling() {
    let base = AcoustomagnonicCombParams::default();
    let solver_base = AcoustomagnonicCombSolver::new(base);
    let span_base = solver_base.compute_comb_spectral_span_ghz();

    // Increase RF drive power
    let higher_power = AcoustomagnonicCombParams::new(
        base.pump_frequency_ghz,
        base.magnetoelastic_coupling_mhz,
        base.kerr_nonlinearity_khz,
        base.gilbert_damping_alpha,
        base.acoustic_loss_rate_mhz,
        base.operating_temp_m_k,
        45.0, // increased from 25.0 mW
        base.chiral_asymmetry_ratio,
    );
    let solver_higher = AcoustomagnonicCombSolver::new(higher_power);
    let span_higher = solver_higher.compute_comb_spectral_span_ghz();

    assert!(
        span_higher > span_base,
        "Higher RF drive power must increase comb spectral span: {:.4} > {:.4}",
        span_higher,
        span_base
    );
}

#[test]
fn test_temperature_degradation() {
    let base = AcoustomagnonicCombParams::default();
    let solver_base = AcoustomagnonicCombSolver::new(base);
    let span_base = solver_base.compute_comb_spectral_span_ghz();
    let pn_base = solver_base.compute_phase_noise_at_10khz_dbc();
    let eff_base = solver_base.compute_polariton_conversion_efficiency();
    let iso_base = solver_base.compute_inter_modal_isolation_db();
    let coop_base = solver_base.compute_polariton_cooperativity();

    // Elevated temperature
    let hot = AcoustomagnonicCombParams::new(
        base.pump_frequency_ghz,
        base.magnetoelastic_coupling_mhz,
        base.kerr_nonlinearity_khz,
        base.gilbert_damping_alpha,
        base.acoustic_loss_rate_mhz,
        35.0, // increased from 20.0 mK
        base.rf_drive_power_mw,
        base.chiral_asymmetry_ratio,
    );
    let solver_hot = AcoustomagnonicCombSolver::new(hot);
    let span_hot = solver_hot.compute_comb_spectral_span_ghz();
    let pn_hot = solver_hot.compute_phase_noise_at_10khz_dbc();
    let eff_hot = solver_hot.compute_polariton_conversion_efficiency();
    let iso_hot = solver_hot.compute_inter_modal_isolation_db();
    let coop_hot = solver_hot.compute_polariton_cooperativity();

    assert!(
        span_hot < span_base,
        "Elevated temperature must decrease comb spectral span: {:.4} < {:.4}",
        span_hot,
        span_base
    );
    assert!(
        pn_hot > pn_base,
        "Elevated temperature must degrade phase noise (less negative): {:.4} > {:.4}",
        pn_hot,
        pn_base
    );
    assert!(
        eff_hot < eff_base,
        "Elevated temperature must decrease polariton conversion efficiency: {:.4} < {:.4}",
        eff_hot,
        eff_base
    );
    assert!(
        iso_hot < iso_base,
        "Elevated temperature must decrease modal isolation: {:.4} < {:.4}",
        iso_hot,
        iso_base
    );
    assert!(
        coop_hot < coop_base,
        "Elevated temperature must degrade cooperativity: {:.4} < {:.4}",
        coop_hot,
        coop_base
    );
}

#[test]
fn test_physical_compliance_thresholds() {
    let default_params = AcoustomagnonicCombParams::default();
    let default_solver = AcoustomagnonicCombSolver::new(default_params);
    let default_metrics = default_solver.evaluate_metrics();
    assert!(
        default_metrics.is_physically_compliant,
        "Default configuration must be physically compliant"
    );

    // Extreme degradation configuration should not be fully compliant
    let severe_params = AcoustomagnonicCombParams::new(
        14.0,
        25.0,   // very low coupling
        15.0,
        8.0e-4, // high Gilbert damping
        4.0,    // high acoustic loss
        90.0,   // hot 90 mK
        5.0,    // low power
        0.55,   // low chiral asymmetry
    );
    let severe_solver = AcoustomagnonicCombSolver::new(severe_params);
    let severe_metrics = severe_solver.evaluate_metrics();
    assert!(
        !severe_metrics.is_physically_compliant,
        "Severe degraded configuration must not be physically compliant"
    );
}
