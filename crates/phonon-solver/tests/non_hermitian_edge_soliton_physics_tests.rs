#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Hermitian
//! topological acoustic edge solitons and dissipationless phononic shockwave routers.

use phonon_models::non_hermitian_edge_soliton::NonHermitianEdgeSolitonParams;
use phonon_solver::non_hermitian_edge_soliton::NonHermitianEdgeSolitonSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = NonHermitianEdgeSolitonParams::new(
        0.5,    // below 1.0 GHz
        2.0,    // below 5.0 kHz
        0.1,    // below 0.5 Hz
        0.5,    // below 1.0 MHz
        0.5,    // below 1.0 MHz
        5.0,    // below 10.0 Pa
        0.5,    // below 1.0 mK
        20.0,   // below 50.0 um
    );
    assert!((underflow.carrier_frequency_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.dispersion_parameter_d2_khz - 5.0).abs() < 1e-9);
    assert!((underflow.kerr_nonlinearity_hz - 0.5).abs() < 1e-9);
    assert!((underflow.non_hermitian_gain_mhz - 1.0).abs() < 1e-9);
    assert!((underflow.non_hermitian_loss_mhz - 1.0).abs() < 1e-9);
    assert!((underflow.soliton_amplitude_pa - 10.0).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.waveguide_length_um - 50.0).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = NonHermitianEdgeSolitonParams::new(
        20.0,   // above 12.0 GHz
        150.0,  // above 100.0 kHz
        80.0,   // above 50.0 Hz
        60.0,   // above 40.0 MHz
        60.0,   // above 40.0 MHz
        800.0,  // above 500.0 Pa
        100.0,  // above 50.0 mK
        1500.0, // above 1000.0 um
    );
    assert!((overflow.carrier_frequency_ghz - 12.0).abs() < 1e-9);
    assert!((overflow.dispersion_parameter_d2_khz - 100.0).abs() < 1e-9);
    assert!((overflow.kerr_nonlinearity_hz - 50.0).abs() < 1e-9);
    assert!((overflow.non_hermitian_gain_mhz - 40.0).abs() < 1e-9);
    assert!((overflow.non_hermitian_loss_mhz - 40.0).abs() < 1e-9);
    assert!((overflow.soliton_amplitude_pa - 500.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.waveguide_length_um - 1000.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = NonHermitianEdgeSolitonParams::default();
    let solver = NonHermitianEdgeSolitonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.soliton_transmission_fidelity >= 0.9920,
        "Default soliton transmission fidelity must be >= 0.9920, got {:.6}",
        metrics.soliton_transmission_fidelity
    );
    assert!(
        metrics.harmonic_distortion_db <= -45.0,
        "Default harmonic distortion must be <= -45.0 dB, got {:.4} dB",
        metrics.harmonic_distortion_db
    );
    assert!(
        metrics.backscattering_immunity_db >= 35.0,
        "Default backscattering immunity must be >= 35.0 dB, got {:.4} dB",
        metrics.backscattering_immunity_db
    );
    assert!(
        metrics.soliton_pulse_width_ns <= 15.0,
        "Default soliton pulse width must be <= 15.0 ns, got {:.4} ns",
        metrics.soliton_pulse_width_ns
    );
    assert!(
        metrics.lyapunov_stability_exponent <= 0.050,
        "Default Lyapunov stability exponent must be <= 0.050, got {:.6}",
        metrics.lyapunov_stability_exponent
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_soliton_amplitude_scaling() {
    let base = NonHermitianEdgeSolitonParams::default();
    let solver_base = NonHermitianEdgeSolitonSolver::new(base);

    let high_amp = NonHermitianEdgeSolitonParams::new(
        base.carrier_frequency_ghz,
        base.dispersion_parameter_d2_khz,
        base.kerr_nonlinearity_hz,
        base.non_hermitian_gain_mhz,
        base.non_hermitian_loss_mhz,
        200.0, // increased from 120.0 Pa
        base.operating_temp_m_k,
        base.waveguide_length_um,
    );
    let solver_high_amp = NonHermitianEdgeSolitonSolver::new(high_amp);

    let f_base = solver_base.compute_soliton_transmission_fidelity();
    let f_high = solver_high_amp.compute_soliton_transmission_fidelity();
    assert!(
        f_high >= f_base,
        "Higher amplitude must enhance or maintain transmission fidelity: f_high={:.6}, f_base={:.6}",
        f_high,
        f_base
    );

    let tau_base = solver_base.compute_soliton_pulse_width_ns();
    let tau_high = solver_high_amp.compute_soliton_pulse_width_ns();
    assert!(
        tau_high < tau_base,
        "Higher amplitude must compress soliton temporal pulse width: tau_high={:.4} ns, tau_base={:.4} ns",
        tau_high,
        tau_base
    );

    let hd_base = solver_base.compute_harmonic_distortion_db();
    let hd_high = solver_high_amp.compute_harmonic_distortion_db();
    assert!(
        hd_high >= hd_base,
        "Higher amplitude increases non-linear harmonic generation: hd_high={:.4} dB, hd_base={:.4} dB",
        hd_high,
        hd_base
    );
}

#[test]
fn test_dispersion_parameter_scaling() {
    let base = NonHermitianEdgeSolitonParams::default();
    let solver_base = NonHermitianEdgeSolitonSolver::new(base);

    let high_d2 = NonHermitianEdgeSolitonParams::new(
        base.carrier_frequency_ghz,
        50.0, // increased dispersion from 32.0 kHz
        base.kerr_nonlinearity_hz,
        base.non_hermitian_gain_mhz,
        base.non_hermitian_loss_mhz,
        base.soliton_amplitude_pa,
        base.operating_temp_m_k,
        base.waveguide_length_um,
    );
    let solver_high_d2 = NonHermitianEdgeSolitonSolver::new(high_d2);

    let hd_base = solver_base.compute_harmonic_distortion_db();
    let hd_high_d2 = solver_high_d2.compute_harmonic_distortion_db();
    assert!(
        hd_high_d2 < hd_base,
        "Higher dispersion improves harmonic distortion suppression (more negative dB): hd_high={:.4} dB, hd_base={:.4} dB",
        hd_high_d2,
        hd_base
    );

    let tau_base = solver_base.compute_soliton_pulse_width_ns();
    let tau_high_d2 = solver_high_d2.compute_soliton_pulse_width_ns();
    assert!(
        tau_high_d2 > tau_base,
        "Higher dispersion broadens soliton pulse width: tau_high={:.4} ns, tau_base={:.4} ns",
        tau_high_d2,
        tau_base
    );
}

#[test]
fn test_non_hermitian_gain_and_pt_symmetry() {
    let base = NonHermitianEdgeSolitonParams::default();
    let solver_pt_symmetric = NonHermitianEdgeSolitonSolver::new(base);

    // Broken PT-symmetry: gain != loss
    let broken_pt = NonHermitianEdgeSolitonParams::new(
        base.carrier_frequency_ghz,
        base.dispersion_parameter_d2_khz,
        base.kerr_nonlinearity_hz,
        25.0, // gain detuned from loss (18.0)
        18.0,
        base.soliton_amplitude_pa,
        base.operating_temp_m_k,
        base.waveguide_length_um,
    );
    let solver_broken_pt = NonHermitianEdgeSolitonSolver::new(broken_pt);

    let lyap_pt = solver_pt_symmetric.compute_lyapunov_stability_exponent();
    let lyap_broken = solver_broken_pt.compute_lyapunov_stability_exponent();
    assert!(
        lyap_broken > lyap_pt,
        "PT symmetry detuning must increase Lyapunov stability exponent: lyap_broken={:.6}, lyap_pt={:.6}",
        lyap_broken,
        lyap_pt
    );

    let bi_pt = solver_pt_symmetric.compute_backscattering_immunity_db();
    let bi_higher_gain = solver_broken_pt.compute_backscattering_immunity_db();
    assert!(
        bi_higher_gain > bi_pt,
        "Higher gain must enhance topological backscattering immunity: bi_higher_gain={:.4} dB, bi_pt={:.4} dB",
        bi_higher_gain,
        bi_pt
    );
}

#[test]
fn test_temperature_degradation() {
    let low_temp = NonHermitianEdgeSolitonParams::new(
        3.8,
        32.0,
        12.0,
        18.0,
        18.0,
        120.0,
        5.0, // 5.0 mK
        250.0,
    );
    let high_temp = NonHermitianEdgeSolitonParams::new(
        3.8,
        32.0,
        12.0,
        18.0,
        18.0,
        120.0,
        45.0, // 45.0 mK
        250.0,
    );

    let solver_low = NonHermitianEdgeSolitonSolver::new(low_temp);
    let solver_high = NonHermitianEdgeSolitonSolver::new(high_temp);

    let f_low = solver_low.compute_soliton_transmission_fidelity();
    let f_high = solver_high.compute_soliton_transmission_fidelity();
    assert!(
        f_low > f_high,
        "Elevated temperature must degrade transmission fidelity: f_low={:.6}, f_high={:.6}",
        f_low,
        f_high
    );

    let bi_low = solver_low.compute_backscattering_immunity_db();
    let bi_high = solver_high.compute_backscattering_immunity_db();
    assert!(
        bi_low > bi_high,
        "Elevated temperature must reduce backscattering immunity: bi_low={:.4} dB, bi_high={:.4} dB",
        bi_low,
        bi_high
    );

    let lyap_low = solver_low.compute_lyapunov_stability_exponent();
    let lyap_high = solver_high.compute_lyapunov_stability_exponent();
    assert!(
        lyap_high > lyap_low,
        "Elevated temperature must increase Lyapunov exponent: lyap_high={:.6}, lyap_low={:.6}",
        lyap_high,
        lyap_low
    );
}

#[test]
fn test_physical_compliance_thresholds() {
    // Parameters yielding full compliance
    let compliant_params = NonHermitianEdgeSolitonParams::default();
    let compliant_solver = NonHermitianEdgeSolitonSolver::new(compliant_params);
    assert!(compliant_solver.evaluate_metrics().is_physically_compliant);

    // Extreme temperature degradation violating fidelity / distortion / Lyapunov limits
    let non_compliant_temp = NonHermitianEdgeSolitonParams {
        operating_temp_m_k: 500.0, // intentionally exceeding normal operational range
        ..NonHermitianEdgeSolitonParams::default()
    };
    let solver_uncooled = NonHermitianEdgeSolitonSolver::new(non_compliant_temp);
    let metrics_uncooled = solver_uncooled.evaluate_metrics();
    assert!(
        !metrics_uncooled.is_physically_compliant || metrics_uncooled.lyapunov_stability_exponent >= 0.050,
        "Extreme temperature uncooled system must violate roadmap physical compliance"
    );
}
