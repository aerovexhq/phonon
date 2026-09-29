//! Integration tests for phononic Lugiato-Lefever non-linear envelope dynamics,
//! dissipative Kerr soliton formation, and octave-spanning acoustic frequency combs.

use phonon_models::phononic_microcomb::PhononicMicrocombParams;
use phonon_solver::phononic_microcomb::LugiatoLefeverSolver;

#[test]
fn test_phononic_kerr_soliton_and_octave_comb() {
    let params = PhononicMicrocombParams {
        center_frequency_ghz: 3.0,
        free_spectral_range_mhz: 40.0,
        dispersion_d2_khz: 25.0,
        total_damping_khz: 50.0,
        kerr_nonlinearity_hz: 15.0,
        drive_power_mw: 8.0,
        normalized_detuning: 2.8,
    };
    let solver = LugiatoLefeverSolver::new(params);
    let metrics = solver.solve_microcomb_metrics();

    // Verify dissipative soliton duration in picosecond range (10 - 60 ps)
    assert!(
        metrics.soliton_duration_ps >= 10.0 && metrics.soliton_duration_ps <= 60.0,
        "Soliton duration {:.2} ps should be in 10 - 60 ps range",
        metrics.soliton_duration_ps
    );

    // Comb span must strictly be octave-spanning (>= 1.0 octave)
    assert!(
        metrics.comb_span_octaves >= 1.0,
        "Comb span {:.2} octaves must be >= 1.0 octave",
        metrics.comb_span_octaves
    );

    // Threshold power should be sub-10 mW
    assert!(
        metrics.threshold_power_mw > 0.1 && metrics.threshold_power_mw < 10.0,
        "Threshold power {:.2} mW should be in 0.1 - 10 mW",
        metrics.threshold_power_mw
    );

    // Verify frequency comb sech^2 spectrum
    let spectrum = solver.solve_comb_spectrum(20);
    assert_eq!(spectrum.len(), 41); // -20..=20

    // Center mode (mu = 0) must have highest power
    let center_power = spectrum.iter().find(|(mu, _)| *mu == 0).unwrap().1;
    let wing_power = spectrum.iter().find(|(mu, _)| *mu == 20).unwrap().1;
    assert!(
        center_power > wing_power,
        "Center power {:.1} dBm must exceed wing power {:.1} dBm",
        center_power,
        wing_power
    );

    // Verify temporal soliton profile (odd point count ensures exact zero sampling)
    let profile = solver.solve_soliton_temporal_profile(65);
    assert_eq!(profile.len(), 65);
    let peak_intensity = profile.iter().map(|(_, val)| *val).fold(0.0_f64, f64::max);
    assert!(
        (peak_intensity - 1.0).abs() < 1e-4,
        "Peak intensity must be normalized to 1.0"
    );
}

#[test]
fn test_acoustic_harmonic_generation() {
    let params = PhononicMicrocombParams::default();
    let solver = LugiatoLefeverSolver::new(params);
    let metrics = solver.solve_microcomb_metrics();

    // Highest harmonic generation order must be >= 10
    assert!(
        metrics.highest_harmonic_order >= 10,
        "Highest harmonic order {} must be >= 10",
        metrics.highest_harmonic_order
    );

    // Verify harmonic power monotonicity
    let p_2 = solver.solve_harmonic_power_ratio(2);
    let p_5 = solver.solve_harmonic_power_ratio(5);
    let p_10 = solver.solve_harmonic_power_ratio(10);
    assert!(p_2 > p_5, "Harmonic 2 power must exceed harmonic 5");
    assert!(p_5 > p_10, "Harmonic 5 power must exceed harmonic 10");
    assert!(p_10 > 0.0, "Harmonic 10 power must be non-zero");
}
