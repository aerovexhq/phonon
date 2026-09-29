//! Integration tests for non-reciprocal acoustic cloaking cross-section reduction
//! and topological edge transmission through sharp bends.

use phonon_models::metamaterial_circulator_cloak::MetamaterialCirculatorCloakParams;
use phonon_solver::metamaterial_circulator_cloak::MetamaterialCirculatorCloakSolver;

#[test]
fn test_cloaking_and_bend_bounds() {
    let params = MetamaterialCirculatorCloakParams::default();
    let solver = MetamaterialCirculatorCloakSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.cloaking_cross_section_reduction_db >= 20.0,
        "Cloaking reduction {} dB must be >= 20.0 dB",
        metrics.cloaking_cross_section_reduction_db
    );
    assert!(
        metrics.topological_bend_transmission_pct >= 90.0,
        "Topological bend transmission {}% must be >= 90.0%",
        metrics.topological_bend_transmission_pct
    );
}

#[test]
fn test_shell_radius_ratio_cloaking_scaling() {
    let p_thin = MetamaterialCirculatorCloakParams {
        cloak_inner_radius_mm: 50.0,
        cloak_outer_radius_mm: 75.0, // ratio 1.5
        ..Default::default()
    };
    let p_thick = MetamaterialCirculatorCloakParams {
        cloak_inner_radius_mm: 50.0,
        cloak_outer_radius_mm: 150.0, // ratio 3.0
        ..Default::default()
    };

    let solver_thin = MetamaterialCirculatorCloakSolver::new(p_thin);
    let solver_thick = MetamaterialCirculatorCloakSolver::new(p_thick);

    let red_thin = solver_thin.compute_cloaking_cross_section_reduction_db();
    let red_thick = solver_thick.compute_cloaking_cross_section_reduction_db();

    assert!(
        red_thick > red_thin,
        "Thicker metamaterial cloaking shell must provide greater scattering cross-section reduction"
    );
    assert!(red_thin >= 20.0);
    assert!(red_thick >= 20.0);
}
