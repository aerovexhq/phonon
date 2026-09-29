#![deny(unsafe_code)]

use phonon_models::high_harmonic_bloch::HighHarmonicBlochParams;
use phonon_solver::high_harmonic_bloch::HighHarmonicBlochSolver;

#[test]
fn test_harmonic_cutoff_and_spectral_purity() {
    let drives = [1.0, 1.4, 1.8, 2.2];
    for &d in &drives {
        let params = HighHarmonicBlochParams {
            non_linear_drive_factor: d,
            ..Default::default()
        };
        let solver = HighHarmonicBlochSolver::new(params);
        let metrics = solver.solve();

        assert!(metrics.harmonic_cutoff_order >= 25);
        assert!(metrics.spectral_purity_db >= 45.0);
        assert!(metrics.synthesizer_efficiency_pct >= 15.0);
    }
}

#[test]
fn test_dephasing_time_impact() {
    let p_fast_deph = HighHarmonicBlochParams {
        dephasing_time_ps: 5.0,
        ..Default::default()
    };
    let p_slow_deph = HighHarmonicBlochParams {
        dephasing_time_ps: 18.0,
        ..Default::default()
    };

    let m_fast = HighHarmonicBlochSolver::new(p_fast_deph).solve();
    let m_slow = HighHarmonicBlochSolver::new(p_slow_deph).solve();

    assert!(m_slow.coherent_oscillations_count > m_fast.coherent_oscillations_count);
    assert!(m_slow.spectral_purity_db > m_fast.spectral_purity_db);
    assert!(m_fast.coherent_oscillations_count >= 3.0);
    assert!(m_fast.spectral_purity_db >= 45.0);
}
