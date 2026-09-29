//! Integration tests for acoustoelectric moiré flat phonon bands and bandwidth quenching.

use phonon_models::acoustoelectric_moire::AcoustoelectricMoireParams;
use phonon_solver::acoustoelectric_moire::AcoustoelectricMoireSolver;

#[test]
fn test_default_moire_bandwidth_quenching_and_minigap() {
    let params = AcoustoelectricMoireParams::default();
    let solver = AcoustoelectricMoireSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.bandwidth_quenching_ratio >= 10.0,
        "Bandwidth quenching {} must be >= 10.0x",
        metrics.bandwidth_quenching_ratio
    );
    assert!(
        metrics.moire_minigap_mev >= 5.0,
        "Moiré minigap {} meV must be >= 5.0 meV",
        metrics.moire_minigap_mev
    );
    assert!(
        metrics.quantum_simulation_fidelity_pct >= 98.0,
        "Simulation fidelity {}% must be >= 98.0%",
        metrics.quantum_simulation_fidelity_pct
    );
}

#[test]
fn test_twist_angle_and_strain_tuning() {
    let angles = [1.02, 1.08, 1.15, 1.25];
    for &th in &angles {
        let params = AcoustoelectricMoireParams {
            twist_angle_deg: th,
            dynamic_strain_amplitude_1e4: 2.5,
            ..Default::default()
        };
        let solver = AcoustoelectricMoireSolver::new(params);
        let q = solver.compute_bandwidth_quenching_ratio();
        let gap = solver.compute_moire_minigap_mev();

        assert!(
            q >= 10.0,
            "Quenching ratio at theta={} was {} < 10.0x",
            th,
            q
        );
        assert!(gap >= 5.0, "Minigap at theta={} was {} < 5.0 meV", th, gap);
    }
}
