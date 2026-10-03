#![deny(unsafe_code)]

//! Test suite for Phase 338: Discrete Non-Linear Josephson Transmission Line & 4WM Kernel.
//!
//! Verifies:
//! - Discrete Josephson cell non-linear inductance L_J(I) > L_J0 under current bias.
//! - Dispersion relation and cutoff frequency omega_c = 2 / sqrt(L_J0 * C_g).
//! - 4WM phase mismatch cancellation using periodic RPM stubs.
//! - Exponential parametric power gain G >= 20 dB over octave bandwidth 4-8 GHz.
//! - Quantum noise squeezing below SQL (S_xx < 0.25) and quantum-limited added noise n_add <= 0.55.

use phonon_solver::jtwpa_simulator::{
    GainSpectrumPoint, JosephsonCellParams, JosephsonTransmissionLine, JtwpaParams, JtwpaSolver,
    QuantumSqueezing, RpmStubParams,
};
use std::f64::consts::PI;

#[test]
fn test_josephson_cell_non_linear_inductance() {
    let cell = JosephsonCellParams::default();
    let line = JosephsonTransmissionLine::new(1000, cell, None);

    let l_j0 = cell.l_j0;
    assert!((l_j0 - 80.0e-12).abs() < 1.0e-15);

    // At zero current, inductance must equal linear Josephson inductance L_J0
    let l_at_zero = line.non_linear_inductance(0.0);
    assert!(
        (l_at_zero - l_j0).abs() < 1.0e-15,
        "Zero-current inductance must be L_J0: got {}, expected {}",
        l_at_zero,
        l_j0
    );

    // Under current bias I = 0.5 * I_c: L_J(I) = L_J0 / sqrt(1 - 0.25) = L_J0 / sqrt(0.75) > L_J0
    let bias_current = 0.5 * cell.i_c;
    let l_biased = line.non_linear_inductance(bias_current);
    let expected_l = l_j0 / (1.0 - 0.25f64).sqrt();
    assert!(
        l_biased > l_j0,
        "Biased inductance must exceed small-signal inductance: L_biased={}, L_J0={}",
        l_biased,
        l_j0
    );
    assert!(
        (l_biased - expected_l).abs() < 1.0e-15,
        "Biased inductance mismatch: got {}, expected {}",
        l_biased,
        expected_l
    );

    // Near critical current I = 0.90 * I_c: L_J(I) increases significantly
    let l_high_bias = line.non_linear_inductance(0.90 * cell.i_c);
    assert!(
        l_high_bias > l_biased,
        "Higher current bias must produce larger non-linear inductance"
    );

    // Clamping safety check at and above critical current I >= I_c
    let l_clamped = line.non_linear_inductance(1.5 * cell.i_c);
    let max_safe = l_j0 / (1.0 - 0.95 * 0.95f64).sqrt();
    assert!(
        (l_clamped - max_safe).abs() < 1.0e-15,
        "Inductance must safely clamp at 0.95 * I_c without infinity: got {}",
        l_clamped
    );
}

#[test]
fn test_dispersion_relation_and_cutoff_frequency() {
    let cell = JosephsonCellParams::default();
    let line = JosephsonTransmissionLine::new(1000, cell, None);

    // Cutoff angular frequency omega_c = 2 / sqrt(L_J0 * C_g)
    let expected_omega_c = 2.0 / (cell.l_j0 * cell.c_g).sqrt();
    let omega_c = line.cutoff_angular_frequency();
    assert!(
        (omega_c - expected_omega_c).abs() < 1.0e-6,
        "Cutoff angular frequency must match 2 / sqrt(L_J0 * C_g): got {}, expected {}",
        omega_c,
        expected_omega_c
    );

    // Cutoff frequency f_cutoff = 1 / (pi * sqrt(L_J0 * C_g))
    let expected_f_cutoff = 1.0 / (PI * (cell.l_j0 * cell.c_g).sqrt());
    let f_cutoff = line.cutoff_frequency();
    assert!(
        (f_cutoff - expected_f_cutoff).abs() < 1.0e-3,
        "Cutoff frequency must match 1 / (pi * sqrt(L_J0 * C_g)): got {}, expected {}",
        f_cutoff,
        expected_f_cutoff
    );

    // Characteristic impedance Z_0 = sqrt(L_J0 / C_g) = sqrt(80e-12 / 50e-15) = 40.0 Ohms
    let z0 = line.characteristic_impedance();
    assert!(
        (z0 - 40.0).abs() < 1.0e-6,
        "Characteristic impedance must be 40 Ohms: got {}",
        z0
    );

    // Dispersion relation at low frequency (omega << omega_c): k approx omega / v_p
    let f_low = 1.0e9; // 1 GHz
    let k_low = line.dispersion_k(f_low, false);
    let omega_low = 2.0 * PI * f_low;
    let v_p = cell.a / (cell.l_j0 * cell.c_g).sqrt();
    let k_expected = omega_low / v_p;
    let relative_err = (k_low - k_expected).abs() / k_expected;
    assert!(
        relative_err < 0.01,
        "Low-frequency dispersion must be linear with phase velocity v_p: rel_err={}",
        relative_err
    );
}

#[test]
fn test_4wm_phase_mismatch_cancellation_with_rpm() {
    let mut params = JtwpaParams::default();
    params.pump_freq_hz = 6.0e9;
    params.pump_power_dbm = -50.0;

    let f_s_hz = 5.0e9;
    // Point without RPM
    let pt_bare: GainSpectrumPoint = JtwpaSolver::solve_point(&params, f_s_hz, false);
    // Point with RPM
    let pt_rpm: GainSpectrumPoint = JtwpaSolver::solve_point(&params, f_s_hz, true);

    // Verify RpmStubParams
    let rpm = RpmStubParams::default();
    assert_eq!(rpm.m_cells, 16);
    assert!(rpm.resonant_frequency_hz() > 1.0e9);

    // Without RPM: phase mismatch Delta_k is large and negative due to Kerr shift -2*gamma_NL*P_p
    assert!(
        pt_bare.delta_k.abs() > 500.0,
        "Bare phase mismatch must be significant (> 500 rad/m) due to Kerr shift: got {}",
        pt_bare.delta_k
    );

    // With RPM: periodic RPM stubs cancel the nonlinear Kerr phase shift
    assert!(
        pt_rpm.delta_k_rpm.abs() < 20.0,
        "RPM phase mismatch must be compensated near zero (< 20 rad/m): got {}",
        pt_rpm.delta_k_rpm
    );

    // Cancellation efficiency: |Delta_k_rpm| << |Delta_k_bare|
    assert!(
        pt_rpm.delta_k_rpm.abs() < pt_bare.delta_k.abs() * 0.05,
        "RPM must cancel at least 95% of bare phase mismatch: bare={}, rpm={}",
        pt_bare.delta_k,
        pt_rpm.delta_k_rpm
    );
}

#[test]
fn test_parametric_power_gain_over_octave_bandwidth() {
    let mut params = JtwpaParams::default();
    params.pump_freq_hz = 6.0e9;
    params.pump_power_dbm = -50.0;
    params.signal_freq_start_hz = 4.0e9;
    params.signal_freq_stop_hz = 8.0e9;
    params.sample_points = 41; // 100 MHz resolution across 4-8 GHz octave

    let spectrum = JtwpaSolver::solve_spectrum(&params);
    assert_eq!(spectrum.len(), 41);

    // Verify exponential power gain G >= 20 dB across the entire octave bandwidth [4.0, 8.0] GHz
    for pt in &spectrum {
        assert!(
            pt.gain_s_db >= 20.0,
            "Parametric signal gain G_s at {} GHz must achieve >= 20.0 dB: got {:.2} dB",
            pt.f_s,
            pt.gain_s_db
        );
        assert!(
            pt.gain_s_linear >= 100.0,
            "Linear signal gain G_s at {} GHz must achieve >= 100: got {:.2}",
            pt.f_s,
            pt.gain_s_linear
        );
        // Idler gain should be approximately G_s - 1
        assert!(
            pt.gain_i_db >= 19.9,
            "Idler conversion gain at {} GHz must follow signal gain: got {:.2} dB",
            pt.f_i,
            pt.gain_i_db
        );
    }

    let (peak_gain_db, bw_ghz) = JtwpaSolver::bandwidth_3db(&spectrum);
    assert!(
        peak_gain_db >= 20.0,
        "Peak gain must exceed 20 dB: got {:.2} dB",
        peak_gain_db
    );
    assert!(
        bw_ghz >= 3.9,
        "3-dB bandwidth must span nearly full octave band: got {:.2} GHz",
        bw_ghz
    );
}

#[test]
fn test_quantum_noise_squeezing_and_added_noise() {
    let params = JtwpaParams::default();
    let squeezing: QuantumSqueezing = JtwpaSolver::quantum_squeezing(&params, 5.5e9);

    // Squeezed quadrature variance must be below the Standard Quantum Limit (0.25)
    assert!(
        squeezing.s_xx < 0.25,
        "Squeezed quadrature variance S_xx must be below SQL (0.25): got {}",
        squeezing.s_xx
    );

    // Anti-squeezed quadrature variance must satisfy Heisenberg minimum uncertainty S_xx * S_yy = 0.0625
    let uncertainty_product = squeezing.s_xx * squeezing.s_yy;
    assert!(
        (uncertainty_product - 0.0625).abs() < 1.0e-6,
        "Uncertainty product S_xx * S_yy must equal 1/16 = 0.0625: got {}",
        uncertainty_product
    );

    // Squeezing in dB must exceed 6 dB below SQL
    assert!(
        squeezing.squeezing_db > 6.0,
        "Squeezing must exceed 6 dB below SQL: got {:.2} dB",
        squeezing.squeezing_db
    );

    // Added noise quanta n_add = 0.5 * (1 - 1/G_s) must satisfy n_add <= 0.55
    assert!(
        squeezing.n_add <= 0.55,
        "Added noise quanta n_add must be quantum-limited (<= 0.55): got {}",
        squeezing.n_add
    );
    assert!(
        squeezing.n_add >= 0.49,
        "Added noise quanta n_add must approach 0.5 quanta: got {}",
        squeezing.n_add
    );
}
