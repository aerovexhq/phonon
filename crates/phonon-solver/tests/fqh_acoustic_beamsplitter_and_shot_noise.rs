#![deny(unsafe_code)]

use phonon_models::fqh_acoustic_interferometer::FqhInterferometerParams;
use phonon_solver::fqh_acoustic_interferometer::FqhInterferometerSolver;

#[test]
fn test_default_fqh_interferometer() {
    let params = FqhInterferometerParams::default();
    let solver = FqhInterferometerSolver::new(params);
    let metrics = solver.solve();

    // Charge precision error <= 1.0e-4
    assert!(
        metrics.charge_precision_error <= 1.0e-4,
        "Precision error {} > 1.0e-4",
        metrics.charge_precision_error
    );

    // Fano factor 0.25 +/- 1.0e-4
    assert!(
        (metrics.fano_factor - 0.25).abs() <= 1.0e-4,
        "Fano factor {} != 0.25",
        metrics.fano_factor
    );

    // Fringe visibility >= 90.0%
    assert!(
        metrics.fringe_visibility_pct >= 90.0,
        "Visibility {}% < 90.0%",
        metrics.fringe_visibility_pct
    );

    // Coherence length >= 25.0 um
    assert!(
        metrics.coherence_length_um >= 25.0,
        "Coherence length {} um < 25.0 um",
        metrics.coherence_length_um
    );

    // Cross correlation <= -20.0 dB
    assert!(
        metrics.cross_correlation_db <= -20.0,
        "Cross correlation {} dB > -20.0 dB",
        metrics.cross_correlation_db
    );

    // Readout SNR >= 25.0 dB
    assert!(
        metrics.readout_snr_db >= 25.0,
        "Readout SNR {} dB < 25.0 dB",
        metrics.readout_snr_db
    );
}

#[test]
fn test_shot_noise_vs_temperature_and_drive() {
    let p_cold = FqhInterferometerParams {
        temperature_mk: 10.0,
        ..Default::default()
    };
    let s_cold = FqhInterferometerSolver::new(p_cold);
    let m_cold = s_cold.solve();

    let p_warm = FqhInterferometerParams {
        temperature_mk: 50.0,
        ..Default::default()
    };
    let s_warm = FqhInterferometerSolver::new(p_warm);
    let m_warm = s_warm.solve();

    assert!(m_cold.charge_precision_error < m_warm.charge_precision_error);
    assert!(m_warm.charge_precision_error <= 1.0e-4);

    assert!(m_cold.fringe_visibility_pct > m_warm.fringe_visibility_pct);
    assert!(m_warm.fringe_visibility_pct >= 90.0);
}
