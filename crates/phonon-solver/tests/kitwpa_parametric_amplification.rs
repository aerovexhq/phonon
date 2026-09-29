//! Integration tests for Superconducting KITWPA 4WM Parametric Amplification.

use phonon_models::kitwpa::{KitwpaTransmissionLine, SuperconductingFilm};
use phonon_solver::kitwpa::KitwpaWaveSolver;

#[test]
fn test_kitwpa_nonlinear_inductance() {
    let film = SuperconductingFilm::default();
    assert_eq!(film.material_name, "NbTiN");
    assert!((film.critical_temperature_k - 14.5).abs() < 1e-6);
    assert!((film.kinetic_inductance_fraction - 0.95).abs() < 1e-6);

    let line = KitwpaTransmissionLine::default();
    let l0 = line.linear_inductance_per_m;
    let i_star = line.film.characteristic_current_scale_a;

    // At I = 0: L = L0
    let l_zero = line.non_linear_inductance_per_m(0.0);
    assert!((l_zero - l0).abs() < 1e-12);

    // At I = I_star: L = L0 * (1 + xi_nl) = L0 * 1.95
    let l_star = line.non_linear_inductance_per_m(i_star);
    let expected = l0 * (1.0 + 0.95);
    assert!((l_star - expected).abs() < 1e-12);

    // Phase velocity and characteristic impedance
    let vp = line.phase_velocity_m_per_s();
    let z0 = line.characteristic_impedance_ohms();
    assert!(vp > 4.0e7 && vp < 5.0e7); // ~ 0.157 c
    assert!(z0 > 65.0 && z0 < 75.0); // ~ 70.7 Ohms
}

#[test]
fn test_kitwpa_gain_and_manley_rowe_conservation() {
    let line = KitwpaTransmissionLine::default();
    let solver = KitwpaWaveSolver::new();

    // Peak gain at pump frequency 8 GHz
    let gain_linear = line.analytical_signal_power_gain(8.0e9, true);
    let gain_db = 10.0 * gain_linear.log10();
    assert!(
        gain_db >= 20.0,
        "KITWPA gain must exceed 20 dB, got {:.2} dB",
        gain_db
    );

    // RK4 spatial integration and Manley-Rowe photon conservation check
    let (state, max_mr_error) = solver.integrate_spatial_profile(&line, 8.0e9, true, 200);
    let rk4_gain_s = state.signal_power_gain();
    let rk4_gain_i = state.idler_power_gain();
    let rk4_gain_db = 10.0 * rk4_gain_s.log10();

    assert!(
        rk4_gain_db >= 20.0,
        "RK4 integrated gain must exceed 20 dB, got {:.2} dB",
        rk4_gain_db
    );
    assert!(
        (rk4_gain_s - gain_linear).abs() / gain_linear < 1e-3,
        "RK4 and analytical gains must match"
    );
    assert!(
        max_mr_error < 1e-6,
        "Manley-Rowe error must be < 1e-6, got {:e}",
        max_mr_error
    );
    assert!(
        (rk4_gain_s - rk4_gain_i - 1.0).abs() < 1e-6,
        "Manley-Rowe photon balance: G_s - G_i = 1"
    );
}

#[test]
fn test_kitwpa_bandwidth_and_saturation_power() {
    let line = KitwpaTransmissionLine::default();
    let solver = KitwpaWaveSolver::new();

    let spectrum = solver.sweep_gain_spectrum(&line, 4.0e9, 12.0e9, 41, true);
    assert!(spectrum.peak_gain_db >= 20.0, "Peak gain must be >= 20 dB");
    assert!(
        spectrum.three_db_bandwidth_hz >= 4.0e9,
        "3-dB bandwidth must be >= 4.0 GHz, got {:.2} GHz",
        spectrum.three_db_bandwidth_hz / 1e9
    );
    assert!(
        spectrum.manley_rowe_max_error < 1e-6,
        "Max Manley-Rowe error across spectrum must be < 1e-6"
    );

    let p_sat_dbm = line.one_db_compression_power_dbm();
    assert!(
        p_sat_dbm > -50.0,
        "Saturation power P_-1dB must exceed -50 dBm, got {:.2} dBm",
        p_sat_dbm
    );
}
