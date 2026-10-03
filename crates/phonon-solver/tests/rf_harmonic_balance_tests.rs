#![deny(unsafe_code)]

//! Integration test suite for Phase 328: Phonon Studio RF & Microwave S-Parameter Extraction
//! and Harmonic Balance Frequency-Domain Engine.

use phonon_solver::rf::{
    constant_reactance_arc, constant_reactance_circle, constant_resistance_circle,
    gamma_to_normalized_z, gamma_to_z, load_stability_circle, normalized_z_to_gamma,
    source_stability_circle, z_to_gamma, Complex64, FrequencySweep, HarmonicBalanceSolver,
    MultiPortSSolver, NonlinearDevice, SweepType, TwoPortSParameters,
};
use std::f64::consts::PI;

#[test]
fn test_frequency_sweep_log_and_linear_span() {
    // 100 kHz to 100 GHz sweep test
    let start_hz = 100_000.0;
    let stop_hz = 100_000_000_000.0;
    let points = 101;

    let log_sweep = FrequencySweep::logarithmic(start_hz, stop_hz, points);
    assert_eq!(log_sweep.sweep_type, SweepType::Logarithmic);
    assert_eq!(log_sweep.points, points);

    let freqs = log_sweep.points();
    assert_eq!(freqs.len(), points);
    assert!((freqs[0] - start_hz).abs() < 1e-3);
    assert!((freqs[points - 1] - stop_hz).abs() < 1e3);

    // Verify monotonic log progression
    for i in 1..points {
        assert!(freqs[i] > freqs[i - 1]);
        let ratio = freqs[i] / freqs[i - 1];
        // 6 decades across 100 steps: 10^(6/100) ~ 1.148
        assert!((ratio - 1.1481536).abs() < 1e-4);
    }

    // Linear sweep test
    let lin_sweep = FrequencySweep::linear(1.0e9, 10.0e9, 10);
    assert_eq!(lin_sweep.sweep_type, SweepType::Linear);
    let lin_freqs = lin_sweep.points();
    assert_eq!(lin_freqs.len(), 10);
    assert!((lin_freqs[0] - 1.0e9).abs() < 1e-3);
    assert!((lin_freqs[9] - 10.0e9).abs() < 1e-3);
    for i in 1..10 {
        assert!((lin_freqs[i] - lin_freqs[i - 1] - 1.0e9).abs() < 1e-3);
    }
}

#[test]
fn test_multiport_solver_transmission_line_and_rlc() {
    let solver = MultiPortSSolver::default()
        .with_z0(50.0)
        .with_sweep(FrequencySweep::logarithmic(1.0e6, 10.0e9, 50));

    // Lossless matched transmission line (Zc = 50 Ohm, tau = 100 ps)
    let res = solver.solve_transmission_line(50.0, 100.0e-12);
    assert_eq!(res.len(), 50);

    for s in &res.s_parameters {
        // Matched line should exhibit zero reflection (|S11| < 1e-10) and unity transmission (|S21| = 1)
        assert!(s.s11_mag() < 1e-10);
        assert!(s.s22_mag() < 1e-10);
        assert!((s.s21_mag() - 1.0).abs() < 1e-10);
        assert!((s.s12_mag() - 1.0).abs() < 1e-10);
        assert!((s.vswr() - 1.0).abs() < 1e-6);
    }

    // Series RLC bandpass resonator (R = 5 Ohm, L = 2.533 nH, C = 1 pF -> f0 ~ 3.162 GHz)
    let r: f64 = 5.0;
    let l: f64 = 2.53303e-9;
    let c: f64 = 1.0e-12;
    let f0: f64 = 1.0 / (2.0 * PI * (l * c).sqrt());
    assert!((f0 - 3.162e9).abs() < 1e7);

    let rlc_solver = MultiPortSSolver::default()
        .with_z0(50.0)
        .with_sweep(FrequencySweep::linear(2.0e9, 4.5e9, 51));
    let rlc_res = rlc_solver.solve_series_rlc(r, l, c);

    // Find frequency point closest to f0
    let mut min_diff = f64::INFINITY;
    let mut resonant_s = rlc_res.s_parameters[0];
    for s in &rlc_res.s_parameters {
        let diff = (s.freq_hz - f0).abs();
        if diff < min_diff {
            min_diff = diff;
            resonant_s = *s;
        }
    }

    // At series resonance, reactance vanishes and Z_series = R = 5 Ohm.
    // S21 = 2*Z0 / (R + 2*Z0) = 100 / 105 ~ 0.95238
    let expected_s21 = 100.0 / 105.0;
    assert!((resonant_s.s21_mag() - expected_s21).abs() < 0.02);
}

#[test]
fn test_rollett_stability_k_delta_mu1_and_gain_metrics() {
    // Calibrated unconditionally stable RF transistor at 2 GHz:
    // S11 = 0.65 /_ -140 deg, S21 = 2.8 /_ 50 deg, S12 = 0.08 /_ 20 deg, S22 = 0.45 /_ -80 deg
    let z0 = 50.0;
    let s11 = Complex64::from_polar(0.65, -140.0_f64.to_radians());
    let s21 = Complex64::from_polar(2.80, 50.0_f64.to_radians());
    let s12 = Complex64::from_polar(0.08, 20.0_f64.to_radians());
    let s22 = Complex64::from_polar(0.45, -80.0_f64.to_radians());

    let s = TwoPortSParameters::new(2.0e9, z0, s11, s12, s21, s22);

    let delta = s.delta();
    let delta_mag = delta.abs();
    assert!(delta_mag < 1.0, "Delta magnitude must be < 1 for unconditional stability");

    let k = s.stability_factor_k();
    assert!(k > 1.0, "Rollett K factor must be > 1.0 for unconditional stability");

    let mu1 = s.mu1();
    assert!(mu1 > 1.0, "Edwards-Sinsky mu1 must be > 1.0 for unconditional stability");
    assert!(s.is_unconditionally_stable());

    // Gain metrics
    let msg = s.maximum_stable_gain();
    let msg_expected = 2.80 / 0.08; // 35.0
    assert!((msg - msg_expected).abs() < 1e-6);
    let msg_db = s.msg_db();
    assert!((msg_db - 10.0 * 35.0_f64.log10()).abs() < 1e-4);

    let mag = s.maximum_available_gain();
    assert!(mag.is_some());
    let mag_val = mag.unwrap();
    assert!(mag_val > 0.0 && mag_val <= msg);
    let mag_db = s.mag_db().unwrap();
    assert!(mag_db < msg_db);

    // Conditionally unstable case (increased feedback S12 = 0.35)
    let s12_unstable = Complex64::from_polar(0.35, 20.0_f64.to_radians());
    let s_unstable = TwoPortSParameters::new(2.0e9, z0, s11, s12_unstable, s21, s22);

    assert!(!s_unstable.is_unconditionally_stable());
    assert!(s_unstable.stability_factor_k() < 1.0);
    assert!(s_unstable.mu1() < 1.0);
    assert!(s_unstable.maximum_available_gain().is_none());
}

#[test]
fn test_touchstone_s2p_export_format() {
    let solver = MultiPortSSolver::default()
        .with_z0(50.0)
        .with_sweep(FrequencySweep::linear(1.0e9, 3.0e9, 3));

    let sweep_res = solver.solve_tee_attenuator(16.6667, 66.6667);
    let s2p_str = sweep_res.to_touchstone_s2p();

    // Verify Touchstone format requirements
    assert!(s2p_str.contains("# HZ S RI R 50"));
    assert!(s2p_str.contains("Touchstone 2-port S-parameters file"));

    // Check row count (header lines + 3 data lines)
    let non_comment_lines: Vec<&str> = s2p_str
        .lines()
        .filter(|l| !l.starts_with('!') && !l.starts_with('#') && !l.trim().is_empty())
        .collect();
    assert_eq!(non_comment_lines.len(), 3);

    // Check first data line contains 9 numbers: freq s11_re s11_im s21_re s21_im s12_re s12_im s22_re s22_im
    let tokens: Vec<&str> = non_comment_lines[0].split_whitespace().collect();
    assert_eq!(tokens.len(), 9);
    let f_parsed: f64 = tokens[0].parse().unwrap();
    assert!((f_parsed - 1.0e9).abs() < 1.0);

    // Also test single-point Touchstone export on TwoPortSParameters
    let single_s2p = sweep_res.s_parameters[0].to_touchstone_s2p();
    assert!(single_s2p.contains("# HZ S RI R 50"));
}

#[test]
fn test_smith_chart_conversions_and_circles() {
    let z0 = 50.0;

    // 1. Center of Smith chart: matched impedance Z = 50 Ohm (z = 1.0) -> Gamma = 0
    let gamma_center = z_to_gamma(Complex64::new(50.0, 0.0), z0);
    assert!(gamma_center.norm() < 1e-12);
    let z_back = gamma_to_z(gamma_center, z0);
    assert!((z_back.re - 50.0).abs() < 1e-10);
    assert!(z_back.im.abs() < 1e-10);

    // 2. Short circuit: Z = 0 -> Gamma = -1
    let gamma_short = z_to_gamma(Complex64::ZERO, z0);
    assert!((gamma_short.re - (-1.0)).abs() < 1e-12);
    assert!(gamma_short.im.abs() < 1e-12);
    let z_short = gamma_to_z(gamma_short, z0);
    assert!(z_short.abs() < 1e-10);

    // 3. Open circuit: Z -> inf -> Gamma = 1
    let gamma_open = normalized_z_to_gamma(Complex64::new(1e10, 0.0));
    assert!((gamma_open.re - 1.0).abs() < 1e-6);

    // 4. Pure reactances:
    // Z = +j50 (z = +j1) -> Gamma = +j
    let gamma_ind = normalized_z_to_gamma(Complex64::new(0.0, 1.0));
    assert!(gamma_ind.re.abs() < 1e-12);
    assert!((gamma_ind.im - 1.0).abs() < 1e-12);
    let z_ind = gamma_to_normalized_z(gamma_ind);
    assert!(z_ind.re.abs() < 1e-10);
    assert!((z_ind.im - 1.0).abs() < 1e-10);

    // 5. Arbitrary impedance round-trip
    let z_rand = Complex64::new(28.5, -45.2);
    let gamma_rand = z_to_gamma(z_rand, z0);
    let z_reconstructed = gamma_to_z(gamma_rand, z0);
    assert!((z_reconstructed.re - z_rand.re).abs() < 1e-9);
    assert!((z_reconstructed.im - z_rand.im).abs() < 1e-9);

    // 6. Constant resistance circles
    let r1_circ = constant_resistance_circle(1.0);
    assert!((r1_circ.center.re - 0.5).abs() < 1e-12);
    assert!(r1_circ.center.im.abs() < 1e-12);
    assert!((r1_circ.radius - 0.5).abs() < 1e-12);

    let r0_circ = constant_resistance_circle(0.0);
    assert!(r0_circ.center.norm() < 1e-12);
    assert!((r0_circ.radius - 1.0).abs() < 1e-12);

    // 7. Constant reactance circles
    let x1_circ = constant_reactance_circle(1.0);
    assert!((x1_circ.center.re - 1.0).abs() < 1e-12);
    assert!((x1_circ.center.im - 1.0).abs() < 1e-12);
    assert!((x1_circ.radius - 1.0).abs() < 1e-12);

    let xm2_circ = constant_reactance_circle(-2.0);
    assert!((xm2_circ.center.re - 1.0).abs() < 1e-12);
    assert!((xm2_circ.center.im - (-0.5)).abs() < 1e-12);
    assert!((xm2_circ.radius - 0.5).abs() < 1e-12);

    // 8. Constant reactance arc points remain inside the unit circle
    let arc_pts = constant_reactance_arc(1.0, 30);
    assert_eq!(arc_pts.len(), 30);
    for pt in arc_pts {
        assert!(pt.abs() <= 1.000_001, "Arc points must reside within |Gamma| <= 1");
    }

    // 9. Stability circles
    let s11 = Complex64::from_polar(0.6, -120.0_f64.to_radians());
    let s21 = Complex64::from_polar(2.5, 45.0_f64.to_radians());
    let s12 = Complex64::from_polar(0.1, 15.0_f64.to_radians());
    let s22 = Complex64::from_polar(0.5, -60.0_f64.to_radians());
    let s = TwoPortSParameters::new(2.4e9, 50.0, s11, s12, s21, s22);

    let load_circ = load_stability_circle(&s);
    let src_circ = source_stability_circle(&s);

    assert!(load_circ.radius > 0.0);
    assert!(src_circ.radius > 0.0);
    assert!(!load_circ.center.re.is_nan());
    assert!(!src_circ.center.re.is_nan());
}

#[test]
fn test_harmonic_balance_solver_convergence_harmonics_p1db_ip3() {
    // Non-linear amplifier model with cubic compression: i(v) = a1*v + a3*v^3
    let a1 = 0.02; // Small signal transconductance 20 mS
    let a2 = 0.00; // Zero quadratic term (symmetric odd non-linearity)
    let a3 = 0.05; // Cubic compression term 0.05 A/V^3
    let device = NonlinearDevice::Polynomial { a1, a2, a3 };

    let f0 = 1.0e9; // 1 GHz fundamental
    let h = 5; // 5 harmonics
    let solver = HarmonicBalanceSolver::new(f0, h, device)
        .with_z0(50.0)
        .with_max_iterations(50)
        .with_tolerance(1e-8);

    // Solve at moderate input power: Pin = 0 dBm
    let result = solver.solve_single_tone(0.0).expect("Harmonic balance should converge at 0 dBm");

    assert!(result.converged, "Solver must converge");
    assert!(result.iterations < 25, "Solver should converge within 25 iterations");
    assert!(result.residual_norm < 1e-8);
    assert_eq!(result.harmonics.len(), h + 1);

    // Fundamental output power
    let p_fund = result.fundamental_power_dbm();
    assert!(p_fund > -20.0 && p_fund < 10.0);

    // Second harmonic should be extremely small (-100 dBm or lower) due to zero quadratic term
    let p_harm2 = result.second_harmonic_power_dbm();
    assert!(p_harm2 < -60.0, "Second harmonic must be negligible for symmetric cubic nonlinearity");

    // Third harmonic should be generated by cubic non-linearity
    let p_harm3 = result.third_harmonic_power_dbm();
    assert!(p_harm3 > -50.0, "Third harmonic must be generated by a3*v^3 term");
    assert!(p_fund > p_harm3, "Fundamental must exceed third harmonic at 0 dBm");

    // Test non-linear compression and intercept extraction
    let metrics = solver
        .compute_compression_and_intercept(-20.0, 15.0, 15)
        .expect("Compression and IP3 computation should succeed");

    assert!(metrics.linear_gain_db.is_finite());
    assert!(metrics.p1db_in_dbm.is_finite());
    assert!(metrics.p1db_out_dbm.is_finite());
    assert!(metrics.ip3_in_dbm.is_finite());
    assert!(metrics.ip3_out_dbm.is_finite());

    // In non-linear RF stages, OIP3 is typically ~9.6 to 12 dB higher than P1dB_out
    assert!(metrics.ip3_out_dbm > metrics.p1db_out_dbm);

    // Test asymmetric non-linearity produces even harmonics (2*f0)
    let asym_device = NonlinearDevice::Polynomial {
        a1: 0.02,
        a2: 0.015,
        a3: 0.005,
    };
    let asym_solver = HarmonicBalanceSolver::new(f0, h, asym_device)
        .with_z0(50.0)
        .with_max_iterations(60);
    let asym_res = asym_solver.solve_single_tone(0.0).expect("Asymmetric HB should converge");

    assert!(asym_res.converged);
    // Asymmetric device must generate prominent second harmonic
    assert!(asym_res.second_harmonic_power_dbm() > -60.0);
}
