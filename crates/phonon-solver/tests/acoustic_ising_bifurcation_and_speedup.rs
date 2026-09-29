#![deny(unsafe_code)]

use phonon_models::phononic_neural_annealer::PhononicAnnealerParams;
use phonon_solver::phononic_neural_annealer::PhononicAnnealerSolver;

#[test]
fn test_speedup_vs_network_spin_count() {
    let spin_counts = [64, 128, 256, 512];
    for &spins in &spin_counts {
        let params = PhononicAnnealerParams {
            network_spin_count: spins,
            ..Default::default()
        };
        let solver = PhononicAnnealerSolver::new(params);
        let metrics = solver.solve();

        assert!(metrics.speedup_factor >= 100.0);
        assert!(metrics.energy_per_flip_fj <= 50.0);
    }
}

#[test]
fn test_bifurcation_contrast_vs_pump() {
    let p_low_pump = PhononicAnnealerParams {
        parametric_pump_rate_mhz: 20.0,
        ..Default::default()
    };
    let p_high_pump = PhononicAnnealerParams {
        parametric_pump_rate_mhz: 90.0,
        ..Default::default()
    };

    let m_low = PhononicAnnealerSolver::new(p_low_pump).solve();
    let m_high = PhononicAnnealerSolver::new(p_high_pump).solve();

    assert!(m_high.bifurcation_contrast_db > m_low.bifurcation_contrast_db);
    assert!(m_low.bifurcation_contrast_db >= 25.0);
}
