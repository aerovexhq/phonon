#![deny(unsafe_code)]

use phonon_models::fqh_acoustic_interferometer::FqhInterferometerParams;
use phonon_solver::fqh_acoustic_interferometer::FqhInterferometerSolver;

#[test]
fn test_coherence_length_scaling() {
    let dephasings = [0.8, 1.2, 2.0, 3.0];
    for &tau in &dephasings {
        let params = FqhInterferometerParams {
            dephasing_time_ns: tau,
            ..Default::default()
        };
        let solver = FqhInterferometerSolver::new(params);
        let metrics = solver.solve();

        assert!(metrics.coherence_length_um >= 25.0);
        assert!(metrics.fringe_visibility_pct >= 90.0);
    }
}

#[test]
fn test_saw_drive_impact_on_snr() {
    let p_low_saw = FqhInterferometerParams {
        saw_drive_amplitude_mv: 8.0,
        ..Default::default()
    };
    let p_high_saw = FqhInterferometerParams {
        saw_drive_amplitude_mv: 30.0,
        ..Default::default()
    };

    let m_low = FqhInterferometerSolver::new(p_low_saw).solve();
    let m_high = FqhInterferometerSolver::new(p_high_saw).solve();

    assert!(m_high.readout_snr_db > m_low.readout_snr_db);
    assert!(m_low.readout_snr_db >= 25.0);
    assert!(m_high.cross_correlation_db < m_low.cross_correlation_db);
}
