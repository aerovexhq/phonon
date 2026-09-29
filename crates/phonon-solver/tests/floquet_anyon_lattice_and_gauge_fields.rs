//! Integration tests for Floquet-engineered non-Abelian anyon lattices,
//! synthetic gauge field commutators, and topological bandgaps.

use phonon_models::floquet_anyon_braiding::FloquetAnyonParams;
use phonon_solver::floquet_anyon_braiding::FloquetAnyonSolver;

#[test]
fn test_default_floquet_anyon_metrics() {
    let params = FloquetAnyonParams::default();
    let solver = FloquetAnyonSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.synthetic_gauge_commutator_norm >= 0.50,
        "Commutator norm {} must be >= 0.50",
        metrics.synthetic_gauge_commutator_norm
    );
    assert!(
        metrics.floquet_gap_khz >= 50.0,
        "Floquet gap {} kHz must be >= 50.0 kHz",
        metrics.floquet_gap_khz
    );
    assert!(
        metrics.leakage_error_rate <= 1.0e-4,
        "Leakage error rate {} must be <= 1.0e-4",
        metrics.leakage_error_rate
    );
    assert_eq!(
        metrics.anyon_qubit_dimension, 2,
        "Logical qubit dimension must equal 2"
    );
}

#[test]
fn test_modulation_depth_gauge_field_scaling() {
    let p_low_mod = FloquetAnyonParams {
        modulation_depth: 0.30,
        ..Default::default()
    };
    let p_high_mod = FloquetAnyonParams {
        modulation_depth: 0.65,
        ..Default::default()
    };

    let solver_low = FloquetAnyonSolver::new(p_low_mod);
    let solver_high = FloquetAnyonSolver::new(p_high_mod);

    let comm_low = solver_low.compute_synthetic_gauge_commutator_norm();
    let comm_high = solver_high.compute_synthetic_gauge_commutator_norm();

    assert!(
        comm_high > comm_low,
        "Higher modulation depth must generate stronger synthetic non-Abelian gauge field commutator"
    );
    assert!(comm_low >= 0.50);
    assert!(comm_high >= 0.50);
}

#[test]
fn test_floquet_gap_leakage_suppression() {
    let p_small_gap = FloquetAnyonParams {
        floquet_gap_khz: 100.0,
        ..Default::default()
    };
    let p_large_gap = FloquetAnyonParams {
        floquet_gap_khz: 400.0,
        ..Default::default()
    };

    let solver_small = FloquetAnyonSolver::new(p_small_gap);
    let solver_large = FloquetAnyonSolver::new(p_large_gap);

    let leak_small = solver_small.compute_leakage_error_rate();
    let leak_large = solver_large.compute_leakage_error_rate();

    assert!(
        leak_large < leak_small,
        "Larger Floquet topological gap must suppress non-adiabatic leakage"
    );
    assert!(leak_small <= 1.0e-4);
    assert!(leak_large <= 1.0e-4);
}
