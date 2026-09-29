#![deny(unsafe_code)]

use phonon_models::topological_acoustic_axion::AcousticAxionParams;
use phonon_solver::topological_acoustic_axion::AcousticAxionSolver;

#[test]
fn test_dark_matter_transduction_and_cooperativity() {
    let q_factors = [30_000.0, 60_000.0, 120_000.0, 240_000.0];
    for &q in &q_factors {
        let params = AcousticAxionParams {
            cavity_acoustic_q: q,
            ..Default::default()
        };
        let solver = AcousticAxionSolver::new(params);
        let metrics = solver.solve();

        assert!(metrics.axion_cooperativity >= 50.0);
        assert!(metrics.dark_matter_snr_db >= 25.0);
        assert!(metrics.insertion_loss_db <= 1.0);
    }
}

#[test]
fn test_chiral_anomaly_purity_vs_domain_wall() {
    let p_narrow = AcousticAxionParams {
        domain_wall_width_nm: 15.0,
        ..Default::default()
    };
    let p_wide = AcousticAxionParams {
        domain_wall_width_nm: 45.0,
        ..Default::default()
    };

    let m_narrow = AcousticAxionSolver::new(p_narrow).solve();
    let m_wide = AcousticAxionSolver::new(p_wide).solve();

    assert!(m_wide.chiral_anomaly_purity_pct > m_narrow.chiral_anomaly_purity_pct);
    assert!(m_narrow.chiral_anomaly_purity_pct >= 95.0);
}
