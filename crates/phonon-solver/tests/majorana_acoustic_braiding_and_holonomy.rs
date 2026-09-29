//! Integration tests for non-Abelian braiding of Majorana bound states
//! and non-Abelian Berry geometric phase holonomies in chiral acoustic networks.

use phonon_models::majorana_chiral_phonon::MajoranaBraidingParams;
use phonon_solver::majorana_chiral_phonon::MajoranaBraidingSolver;

#[test]
fn test_default_majorana_braiding_metrics() {
    let params = MajoranaBraidingParams::default();
    let solver = MajoranaBraidingSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.braiding_fidelity_pct >= 99.0,
        "Braiding fidelity {}% must be >= 99.0%",
        metrics.braiding_fidelity_pct
    );
    assert!(
        metrics.phase_error_rad <= 0.05,
        "Phase error {} rad must be <= 0.05 rad",
        metrics.phase_error_rad
    );
    assert!(
        metrics.landau_zener_leakage <= 1.0e-3,
        "Landau-Zener leakage {} must be <= 1.0e-3",
        metrics.landau_zener_leakage
    );
    assert!(
        (metrics.non_abelian_phase_rad - std::f64::consts::FRAC_PI_2).abs() <= 0.05,
        "Non-Abelian Berry phase {} rad must be close to pi/2",
        metrics.non_abelian_phase_rad
    );
}

#[test]
fn test_topological_gap_adiabatic_scaling() {
    let p_small_gap = MajoranaBraidingParams {
        topological_gap_muev: 80.0,
        ..Default::default()
    };
    let p_large_gap = MajoranaBraidingParams {
        topological_gap_muev: 300.0,
        ..Default::default()
    };

    let solver_small = MajoranaBraidingSolver::new(p_small_gap);
    let solver_large = MajoranaBraidingSolver::new(p_large_gap);

    let lz_small = solver_small.compute_landau_zener_leakage();
    let lz_large = solver_large.compute_landau_zener_leakage();

    assert!(
        lz_large < lz_small,
        "Larger topological gap must suppress Landau-Zener transition leakage (large={} vs small={})",
        lz_large,
        lz_small
    );
    assert!(lz_small <= 1.0e-3);
    assert!(lz_large <= 1.0e-3);
}

#[test]
fn test_braiding_duration_adiabatic_fidelity() {
    let p_fast = MajoranaBraidingParams {
        braiding_duration_ns: 20.0,
        ..Default::default()
    };
    let p_adiabatic = MajoranaBraidingParams {
        braiding_duration_ns: 80.0,
        ..Default::default()
    };

    let solver_fast = MajoranaBraidingSolver::new(p_fast);
    let solver_adiabatic = MajoranaBraidingSolver::new(p_adiabatic);

    let lz_fast = solver_fast.compute_landau_zener_leakage();
    let lz_adiabatic = solver_adiabatic.compute_landau_zener_leakage();

    assert!(
        lz_adiabatic < lz_fast,
        "Longer braiding duration should provide smoother adiabatic transport"
    );
    assert!(lz_fast <= 1.0e-3);
    assert!(lz_adiabatic <= 1.0e-3);
}
