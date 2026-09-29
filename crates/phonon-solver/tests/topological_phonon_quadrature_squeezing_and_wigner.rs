//! Integration tests for topological phonon quadrature squeezing and macroscopic cat state Wigner functions.

use phonon_models::quantum_topological_squeezing::QuantumPhononSqueezingParams;
use phonon_solver::quantum_topological_squeezing::QuantumPhononSqueezingSolver;

#[test]
fn test_default_phonon_squeezing_and_cat_fidelity() {
    let params = QuantumPhononSqueezingParams::default();
    let solver = QuantumPhononSqueezingSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.quadrature_squeezing_db >= 6.0,
        "Squeezing {} dB must be >= 6.0 dB below shot noise",
        metrics.quadrature_squeezing_db
    );
    assert!(
        metrics.cat_state_fidelity_pct >= 90.0,
        "Cat state fidelity {}% must be >= 90.0%",
        metrics.cat_state_fidelity_pct
    );
    assert!(
        metrics.wigner_negativity >= 0.15,
        "Wigner negativity {} must be >= 0.15",
        metrics.wigner_negativity
    );
}

#[test]
fn test_power_and_q_factor_scaling() {
    let powers = [0.5, 1.5, 3.5, 6.0];
    for &p_mw in &powers {
        let params = QuantumPhononSqueezingParams {
            pump_power_mw: p_mw,
            loaded_q_factor: 2.5e6,
            ..Default::default()
        };
        let solver = QuantumPhononSqueezingSolver::new(params);
        let sqz = solver.compute_quadrature_squeezing_db();
        let fid = solver.compute_cat_state_fidelity_pct();
        let w_neg = solver.compute_wigner_negativity();

        assert!(sqz >= 6.0, "Squeezing at {} mW should be >= 6.0 dB", p_mw);
        assert!(fid >= 90.0, "Fidelity at {} mW should be >= 90.0%", p_mw);
        assert!(
            w_neg >= 0.15,
            "Wigner negativity at {} mW should be >= 0.15",
            p_mw
        );
    }
}
