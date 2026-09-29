#![deny(unsafe_code)]

use phonon_models::topological_acoustic_axion::AcousticAxionParams;
use phonon_solver::topological_acoustic_axion::AcousticAxionSolver;

#[test]
fn test_default_topological_acoustic_axion() {
    let params = AcousticAxionParams::default();
    let solver = AcousticAxionSolver::new(params);
    let metrics = solver.solve();

    // Magnetoelectric isolation >= 30.0 dB
    assert!(
        metrics.magnetoelectric_isolation_db >= 30.0,
        "Isolation {} dB < 30.0 dB",
        metrics.magnetoelectric_isolation_db
    );

    // Axion cooperativity >= 50.0
    assert!(
        metrics.axion_cooperativity >= 50.0,
        "Cooperativity {} < 50.0",
        metrics.axion_cooperativity
    );

    // Dark matter readout SNR >= 25.0 dB
    assert!(
        metrics.dark_matter_snr_db >= 25.0,
        "SNR {} dB < 25.0 dB",
        metrics.dark_matter_snr_db
    );

    // Chiral anomaly purity >= 95.0%
    assert!(
        metrics.chiral_anomaly_purity_pct >= 95.0,
        "Purity {}% < 95.0%",
        metrics.chiral_anomaly_purity_pct
    );

    // Insertion loss <= 1.0 dB
    assert!(
        metrics.insertion_loss_db <= 1.0,
        "Loss {} dB > 1.0 dB",
        metrics.insertion_loss_db
    );

    // Anti-crossing gap >= 1.5 GHz
    assert!(
        metrics.anticrossing_gap_ghz >= 1.5,
        "Gap {} GHz < 1.5 GHz",
        metrics.anticrossing_gap_ghz
    );
}

#[test]
fn test_isolation_and_gap_vs_magnetic_bias() {
    let p_low_bias = AcousticAxionParams {
        static_magnetic_bias_t: 3.0,
        ..Default::default()
    };
    let s_low = AcousticAxionSolver::new(p_low_bias);
    let m_low = s_low.solve();

    let p_high_bias = AcousticAxionParams {
        static_magnetic_bias_t: 12.0,
        ..Default::default()
    };
    let s_high = AcousticAxionSolver::new(p_high_bias);
    let m_high = s_high.solve();

    assert!(m_high.magnetoelectric_isolation_db > m_low.magnetoelectric_isolation_db);
    assert!(m_low.magnetoelectric_isolation_db >= 30.0);

    assert!(m_high.anticrossing_gap_ghz > m_low.anticrossing_gap_ghz);
    assert!(m_low.anticrossing_gap_ghz >= 1.5);
}
