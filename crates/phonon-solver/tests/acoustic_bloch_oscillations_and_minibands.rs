#![deny(unsafe_code)]

use phonon_models::high_harmonic_bloch::HighHarmonicBlochParams;
use phonon_solver::high_harmonic_bloch::HighHarmonicBlochSolver;

#[test]
fn test_default_acoustic_bloch_oscillations() {
    let params = HighHarmonicBlochParams::default();
    let solver = HighHarmonicBlochSolver::new(params);
    let metrics = solver.solve();

    // Fundamental Bloch frequency >= 50.0 GHz
    assert!(
        metrics.bloch_frequency_ghz >= 50.0,
        "Bloch freq {} GHz < 50.0 GHz",
        metrics.bloch_frequency_ghz
    );

    // Harmonic emission cutoff order >= 25
    assert!(
        metrics.harmonic_cutoff_order >= 25,
        "Harmonic cutoff {} < 25",
        metrics.harmonic_cutoff_order
    );

    // Spectral purity >= 45.0 dB
    assert!(
        metrics.spectral_purity_db >= 45.0,
        "Spectral purity {} dB < 45.0 dB",
        metrics.spectral_purity_db
    );

    // Coherent oscillation count >= 3.0
    assert!(
        metrics.coherent_oscillations_count >= 3.0,
        "Oscillation count {} < 3.0",
        metrics.coherent_oscillations_count
    );

    // Synthesizer efficiency >= 15.0%
    assert!(
        metrics.synthesizer_efficiency_pct >= 15.0,
        "Synthesizer efficiency {}% < 15.0%",
        metrics.synthesizer_efficiency_pct
    );

    // Zener leakage prob <= 0.05
    assert!(
        metrics.zener_leakage_prob <= 0.05,
        "Zener leakage {} > 0.05",
        metrics.zener_leakage_prob
    );
}

#[test]
fn test_bloch_frequency_scaling_and_zener_suppression() {
    let p_low_force = HighHarmonicBlochParams {
        effective_force_field_mev_nm: 0.08,
        ..Default::default()
    };
    let s_low = HighHarmonicBlochSolver::new(p_low_force);
    let m_low = s_low.solve();

    let p_high_force = HighHarmonicBlochParams {
        effective_force_field_mev_nm: 0.24,
        ..Default::default()
    };
    let s_high = HighHarmonicBlochSolver::new(p_high_force);
    let m_high = s_high.solve();

    assert!(m_high.bloch_frequency_ghz > m_low.bloch_frequency_ghz);
    assert!(m_low.bloch_frequency_ghz >= 50.0);

    // Test Zener leakage vs bandgap
    let p_small_gap = HighHarmonicBlochParams {
        miniband_gap_mev: 4.0,
        ..Default::default()
    };
    let p_large_gap = HighHarmonicBlochParams {
        miniband_gap_mev: 12.0,
        ..Default::default()
    };

    let m_small = HighHarmonicBlochSolver::new(p_small_gap).solve();
    let m_large = HighHarmonicBlochSolver::new(p_large_gap).solve();

    assert!(m_small.zener_leakage_prob > m_large.zener_leakage_prob);
    assert!(m_small.zener_leakage_prob <= 0.05);
}
