//! Integration tests for sub-SQL acoustic force sensing, gravimetry, and continuous entanglement.

use phonon_models::quantum_topological_squeezing::QuantumPhononSqueezingParams;
use phonon_solver::quantum_topological_squeezing::QuantumPhononSqueezingSolver;

#[test]
fn test_force_sensing_and_gravimetry_bounds() {
    let params = QuantumPhononSqueezingParams::default();
    let solver = QuantumPhononSqueezingSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.force_sensitivity_attonewtons <= 25.0,
        "Force sensitivity {} aN/sqrt(Hz) must be <= 25.0 aN/sqrt(Hz)",
        metrics.force_sensitivity_attonewtons
    );
    assert!(
        metrics.quantum_gravimetry_precision_nano_g <= 5.0,
        "Gravimetry precision {} nano-g must be <= 5.0 nano-g",
        metrics.quantum_gravimetry_precision_nano_g
    );
    assert!(
        metrics.continuous_entanglement_ebits >= 1.0,
        "Entanglement {} ebits must be >= 1.0 ebits",
        metrics.continuous_entanglement_ebits
    );
}

#[test]
fn test_thermal_occupancy_impact() {
    let n_ths = [0.01, 0.05, 0.10, 0.15];
    for &n_th in &n_ths {
        let params = QuantumPhononSqueezingParams {
            bath_thermal_occupancy: n_th,
            ..Default::default()
        };
        let solver = QuantumPhononSqueezingSolver::new(params);
        let force = solver.compute_force_sensitivity_attonewtons();
        let grav = solver.compute_quantum_gravimetry_precision_nano_g();
        let ebits = solver.compute_continuous_entanglement_ebits();

        assert!(
            force <= 25.0,
            "Force at n_th={} was {} > 25.0 aN",
            n_th,
            force
        );
        assert!(
            grav <= 5.0,
            "Gravimetry at n_th={} was {} > 5.0 nano-g",
            n_th,
            grav
        );
        assert!(
            ebits >= 1.0,
            "Entanglement at n_th={} was {} < 1.0 ebits",
            n_th,
            ebits
        );
    }
}
