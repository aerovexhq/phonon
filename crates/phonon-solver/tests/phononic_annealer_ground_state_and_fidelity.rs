#![deny(unsafe_code)]

use phonon_models::phononic_neural_annealer::PhononicAnnealerParams;
use phonon_solver::phononic_neural_annealer::PhononicAnnealerSolver;

#[test]
fn test_default_phononic_neural_annealer() {
    let params = PhononicAnnealerParams::default();
    let solver = PhononicAnnealerSolver::new(params);
    let metrics = solver.solve();

    // Convergence fidelity >= 98.0%
    assert!(
        metrics.convergence_fidelity_pct >= 98.0,
        "Fidelity {}% < 98.0%",
        metrics.convergence_fidelity_pct
    );

    // Speedup factor >= 100.0x
    assert!(
        metrics.speedup_factor >= 100.0,
        "Speedup {}x < 100.0x",
        metrics.speedup_factor
    );

    // Energy per flip <= 50.0 fJ
    assert!(
        metrics.energy_per_flip_fj <= 50.0,
        "Energy {} fJ > 50.0 fJ",
        metrics.energy_per_flip_fj
    );

    // Graph approximation ratio >= 0.95
    assert!(
        metrics.graph_approximation_ratio >= 0.95,
        "Approx ratio {} < 0.95",
        metrics.graph_approximation_ratio
    );

    // Solution time <= 10.0 us
    assert!(
        metrics.solution_time_us <= 10.0,
        "Solution time {} us > 10.0 us",
        metrics.solution_time_us
    );

    // Bifurcation contrast >= 25.0 dB
    assert!(
        metrics.bifurcation_contrast_db >= 25.0,
        "Contrast {} dB < 25.0 dB",
        metrics.bifurcation_contrast_db
    );
}

#[test]
fn test_ramp_duration_and_graph_approximation() {
    let p_fast_ramp = PhononicAnnealerParams {
        annealing_ramp_time_us: 1.0,
        ..Default::default()
    };
    let s_fast = PhononicAnnealerSolver::new(p_fast_ramp);
    let m_fast = s_fast.solve();

    let p_slow_ramp = PhononicAnnealerParams {
        annealing_ramp_time_us: 5.0,
        ..Default::default()
    };
    let s_slow = PhononicAnnealerSolver::new(p_slow_ramp);
    let m_slow = s_slow.solve();

    assert!(m_slow.graph_approximation_ratio > m_fast.graph_approximation_ratio);
    assert!(m_fast.graph_approximation_ratio >= 0.95);

    assert!(m_slow.convergence_fidelity_pct >= m_fast.convergence_fidelity_pct);
    assert!(m_fast.convergence_fidelity_pct >= 98.0);
}
