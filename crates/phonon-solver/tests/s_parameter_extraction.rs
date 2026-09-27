//! Integration test: 2-port RF network parameter analysis and S-parameter conversions.
//! Validates:
//! 1. Analytical S-parameters for a matched distributed transmission line (phase delay, zero reflection, unity transmission).
//! 2. Z-matrix to S-matrix transformation on a calibrated 6 dB resistive attenuator T-pad.
//! 3. Y-matrix to S-matrix transformation and network parameter reciprocity ($S_{12} = S_{21}$).
//! 4. Round-trip identity ($S \to Z \to S$) and RF metrics (Return Loss, Insertion Loss, VSWR).

use phonon_solver::rf::{Complex64, TwoPortSParameters};

#[test]
fn test_matched_transmission_line_s_parameters() {
    let z0 = 50.0;
    let tau = 1.0e-9; // 1 ns delay
    let z_ref = 50.0; // Matched reference

    // Test at f = 250 MHz: lambda/4 electrical length (phase delay = -90 deg = -pi/2 rad)
    let f1 = 250.0e6;
    let s_f1 = TwoPortSParameters::for_transmission_line(z0, tau, z_ref, f1);

    // S11 and S22 must be exactly zero (matched)
    assert!(s_f1.s11.norm() < 1e-12);
    assert!(s_f1.s22.norm() < 1e-12);

    // |S21| must be 1.0 (lossless)
    assert!((s_f1.s21.norm() - 1.0).abs() < 1e-12);
    assert!((s_f1.s12.norm() - 1.0).abs() < 1e-12);

    // Phase of S21 at 250 MHz: exp(-j * 2*pi * 0.25) = exp(-j * pi/2) = 0 - j
    assert!(s_f1.s21.re.abs() < 1e-12);
    assert!((s_f1.s21.im - (-1.0)).abs() < 1e-12);

    // Metrics
    assert!((s_f1.vswr() - 1.0).abs() < 1e-6);
    assert!(s_f1.return_loss_db() > 200.0);
    assert!(s_f1.insertion_loss_db().abs() < 1e-12);

    // Test at f = 500 MHz: lambda/2 electrical length (phase delay = -180 deg = -pi rad)
    let f2 = 500.0e6;
    let s_f2 = TwoPortSParameters::for_transmission_line(z0, tau, z_ref, f2);
    // exp(-j * pi) = -1 + 0j
    assert!((s_f2.s21.re - (-1.0)).abs() < 1e-12);
    assert!(s_f2.s21.im.abs() < 1e-12);
}

#[test]
fn test_resistive_attenuator_z_to_s_conversion() {
    // 6 dB T-pad attenuator designed for 50 Ohm system:
    // Voltage attenuation factor A = 10^(6.0206 / 20) = 2.0
    // R1 = R2 = Z0 * (A - 1) / (A + 1) = 50 * (1/3) = 16.6666667 Ohm
    // R3 = Z0 * (2 * A) / (A^2 - 1) = 50 * (4/3) = 66.6666667 Ohm
    let z0 = 50.0;
    let r1 = z0 * (1.0 / 3.0);
    let r2 = r1;
    let r3 = z0 * (4.0 / 3.0);

    // Z-parameters:
    // Z11 = R1 + R3
    // Z22 = R2 + R3
    // Z12 = Z21 = R3
    let z11 = Complex64::from_real(r1 + r3);
    let z22 = Complex64::from_real(r2 + r3);
    let z12 = Complex64::from_real(r3);
    let z21 = Complex64::from_real(r3);

    let z = [[z11, z12], [z21, z22]];
    let s_params = TwoPortSParameters::from_z_matrix(z, z0, 1.0e6);

    // Matched attenuator: S11 == 0, S22 == 0
    assert!(
        s_params.s11.norm() < 1e-6,
        "Expected S11 ~ 0, got {}",
        s_params.s11.norm()
    );
    assert!(
        s_params.s22.norm() < 1e-6,
        "Expected S22 ~ 0, got {}",
        s_params.s22.norm()
    );

    // 6 dB attenuation -> S21 = 0.50
    assert!(
        (s_params.s21.norm() - 0.5).abs() < 1e-5,
        "Expected S21 ~ 0.5, got {}",
        s_params.s21.norm()
    );
    assert!(
        (s_params.s12.norm() - 0.5).abs() < 1e-5,
        "Expected S12 ~ 0.5, got {}",
        s_params.s12.norm()
    );

    // Insertion loss: -20*log10(0.5) = 6.0206 dB
    let il = s_params.insertion_loss_db();
    assert!(
        (il - 6.0206).abs() < 1e-3,
        "Expected 6.02 dB insertion loss, got {il}"
    );

    // VSWR should be 1.0 (ideally matched)
    assert!(
        (s_params.vswr() - 1.0).abs() < 1e-4,
        "VSWR was {}",
        s_params.vswr()
    );
}

#[test]
fn test_s_to_z_to_s_round_trip_identity() {
    let z0 = 50.0;
    // An arbitrary passive reciprocal 2-port S-matrix
    let s11 = Complex64::new(0.1, 0.2);
    let s22 = Complex64::new(-0.05, 0.15);
    let s21 = Complex64::new(0.6, -0.3);
    let s12 = s21; // Reciprocal

    let original = TwoPortSParameters::new(1.0e6, z0, s11, s12, s21, s22);

    // Convert S -> Z
    let z = original
        .to_z_matrix()
        .expect("Failed to convert S to Z matrix");

    // Convert Z -> S
    let reconstructed = TwoPortSParameters::from_z_matrix(z, z0, 1.0e6);

    assert!((original.s11.re - reconstructed.s11.re).abs() < 1e-10);
    assert!((original.s11.im - reconstructed.s11.im).abs() < 1e-10);
    assert!((original.s21.re - reconstructed.s21.re).abs() < 1e-10);
    assert!((original.s21.im - reconstructed.s21.im).abs() < 1e-10);
    assert!((original.s12.re - reconstructed.s12.re).abs() < 1e-10);
    assert!((original.s12.im - reconstructed.s12.im).abs() < 1e-10);
    assert!((original.s22.re - reconstructed.s22.re).abs() < 1e-10);
    assert!((original.s22.im - reconstructed.s22.im).abs() < 1e-10);
}

#[test]
fn test_y_matrix_to_s_parameters() {
    let z0 = 50.0;

    // 100 Ohm shunt resistor to ground at Port 1:
    // Y11 = 1/100 = 0.01 S, Y12 = 0, Y21 = 0, Y22 = 0
    let g_shunt = 1.0 / 100.0;
    let y11 = Complex64::from_real(g_shunt);
    let y12 = Complex64::ZERO;
    let y21 = Complex64::ZERO;
    let y22 = Complex64::ZERO;

    let y = [[y11, y12], [y21, y22]];
    let s = TwoPortSParameters::from_y_matrix(y, z0, 1.0e6);

    // Analytical S11 for 100 Ohm shunt (Y11 = 0.01 S) in 50 Ohm reference (Y0 = 0.02 S):
    // Gamma = (Y0 - Y11) / (Y0 + Y11) = (0.02 - 0.01) / (0.02 + 0.01) = 0.01 / 0.03 = 1/3
    assert!(
        (s.s11.re - (1.0 / 3.0)).abs() < 1e-6,
        "Expected S11 = 1/3, got {}",
        s.s11.re
    );
    assert!(s.s11.im.abs() < 1e-6);
}
