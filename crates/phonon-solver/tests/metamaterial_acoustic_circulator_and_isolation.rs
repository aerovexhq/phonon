//! Integration tests for topological acoustic metamaterial circulators,
//! non-reciprocal isolation, and insertion loss.

use phonon_models::metamaterial_circulator_cloak::MetamaterialCirculatorCloakParams;
use phonon_solver::metamaterial_circulator_cloak::MetamaterialCirculatorCloakSolver;

#[test]
fn test_default_circulator_metrics() {
    let params = MetamaterialCirculatorCloakParams::default();
    let solver = MetamaterialCirculatorCloakSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.circulator_isolation_db >= 25.0,
        "Isolation {} dB must be >= 25.0 dB",
        metrics.circulator_isolation_db
    );
    assert!(
        metrics.forward_insertion_loss_db <= 1.5,
        "Insertion loss {} dB must be <= 1.5 dB",
        metrics.forward_insertion_loss_db
    );
    assert!(
        metrics.non_reciprocal_contrast_ratio >= 100.0,
        "Non-reciprocal contrast ratio {} must be >= 100.0",
        metrics.non_reciprocal_contrast_ratio
    );
    assert_eq!(metrics.topological_chern_number, 1);
}

#[test]
fn test_fluid_bias_mach_isolation_scaling() {
    let p_low_bias = MetamaterialCirculatorCloakParams {
        fluid_bias_mach_number: 0.10,
        ..Default::default()
    };
    let p_high_bias = MetamaterialCirculatorCloakParams {
        fluid_bias_mach_number: 0.28,
        ..Default::default()
    };

    let solver_low = MetamaterialCirculatorCloakSolver::new(p_low_bias);
    let solver_high = MetamaterialCirculatorCloakSolver::new(p_high_bias);

    let iso_low = solver_low.compute_circulator_isolation_db();
    let iso_high = solver_high.compute_circulator_isolation_db();

    assert!(
        iso_high > iso_low,
        "Higher fluid bias Mach number must enhance circulator isolation (high={} vs low={})",
        iso_high,
        iso_low
    );
    assert!(iso_low >= 25.0);
    assert!(iso_high >= 25.0);
}

#[test]
fn test_resonator_q_insertion_loss_scaling() {
    let p_low_q = MetamaterialCirculatorCloakParams {
        resonator_q_factor: 200.0,
        ..Default::default()
    };
    let p_high_q = MetamaterialCirculatorCloakParams {
        resonator_q_factor: 1000.0,
        ..Default::default()
    };

    let solver_low = MetamaterialCirculatorCloakSolver::new(p_low_q);
    let solver_high = MetamaterialCirculatorCloakSolver::new(p_high_q);

    let loss_low = solver_low.compute_forward_insertion_loss_db();
    let loss_high = solver_high.compute_forward_insertion_loss_db();

    assert!(
        loss_high < loss_low,
        "Higher resonator Q must reduce forward insertion loss (high_q={} vs low_q={})",
        loss_high,
        loss_low
    );
    assert!(loss_low <= 1.5);
    assert!(loss_high <= 1.5);
}
