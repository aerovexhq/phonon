//! Integration tests for correlated electron-phonon pairing and acoustic Wigner crystallization.

use phonon_models::acoustoelectric_moire::AcoustoelectricMoireParams;
use phonon_solver::acoustoelectric_moire::AcoustoelectricMoireSolver;

#[test]
fn test_electron_phonon_pairing_and_correlation() {
    let params = AcoustoelectricMoireParams::default();
    let solver = AcoustoelectricMoireSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.electron_phonon_pairing_ratio >= 3.0,
        "Pairing enhancement {} must be >= 3.0x",
        metrics.electron_phonon_pairing_ratio
    );
    assert!(
        metrics.correlation_ratio_u_over_w >= 3.0,
        "Correlation ratio U/W {} must be >= 3.0",
        metrics.correlation_ratio_u_over_w
    );
    assert!(
        metrics.wigner_crystal_melting_temp_k >= 20.0,
        "Wigner melting temp {} K must be >= 20.0 K",
        metrics.wigner_crystal_melting_temp_k
    );
}

#[test]
fn test_coulomb_u_and_deformation_potential_scaling() {
    let u_values = [30.0, 50.0, 70.0];
    for &u in &u_values {
        let params = AcoustoelectricMoireParams {
            coulomb_correlation_u_mev: u,
            deformation_potential_ev: 6.0,
            ..Default::default()
        };
        let solver = AcoustoelectricMoireSolver::new(params);
        let pairing = solver.compute_electron_phonon_pairing_ratio();
        let u_over_w = solver.compute_correlation_ratio_u_over_w();
        let t_melt = solver.compute_wigner_crystal_melting_temp_k();

        assert!(pairing >= 3.0, "Pairing at U={} was {} < 3.0x", u, pairing);
        assert!(u_over_w >= 3.0, "U/W at U={} was {} < 3.0", u, u_over_w);
        assert!(
            t_melt >= 20.0,
            "Melting temp at U={} was {} < 20.0 K",
            u,
            t_melt
        );
    }
}
