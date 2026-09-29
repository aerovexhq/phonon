#![deny(unsafe_code)]

use phonon_models::chiral_spintronic_memristor::SpintronicMemristorParams;
use phonon_solver::chiral_spintronic_memristor::SpintronicMemristorSolver;

#[test]
fn test_stdp_learning_and_crossbar_efficiency() {
    let dims = [32, 64, 128, 256];
    for &d in &dims {
        let params = SpintronicMemristorParams {
            crossbar_dimension: d,
            ..Default::default()
        };
        let solver = SpintronicMemristorSolver::new(params);
        let metrics = solver.solve();

        assert!(metrics.crossbar_energy_efficiency_topsw >= 150.0);
        assert!(metrics.stdp_learning_fidelity_pct >= 95.0);
        assert!(metrics.weight_linearity_error_pct <= 2.5);
    }
}

#[test]
fn test_conductance_ratio_vs_tmr() {
    let p1 = SpintronicMemristorParams {
        tmr_ratio_pct: 200.0,
        ..Default::default()
    };
    let p2 = SpintronicMemristorParams {
        tmr_ratio_pct: 400.0,
        ..Default::default()
    };

    let m1 = SpintronicMemristorSolver::new(p1).solve();
    let m2 = SpintronicMemristorSolver::new(p2).solve();

    assert!(m2.conductance_on_off_ratio > m1.conductance_on_off_ratio);
    assert!(m1.conductance_on_off_ratio >= 10.0);
}
