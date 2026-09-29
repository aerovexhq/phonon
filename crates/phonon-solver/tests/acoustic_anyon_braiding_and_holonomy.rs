//! Integration tests for quantum acoustic anyon braiding fidelity and holonomy error.

use phonon_models::quantum_acoustic_anyons::QuantumAcousticAnyonParams;
use phonon_solver::quantum_acoustic_anyons::QuantumAcousticAnyonSolver;

#[test]
fn test_default_braiding_fidelity_and_leakage() {
    let params = QuantumAcousticAnyonParams::default();
    let solver = QuantumAcousticAnyonSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.braiding_fidelity_pct >= 99.90,
        "Braiding fidelity {}% must be >= 99.90%",
        metrics.braiding_fidelity_pct
    );
    assert!(
        metrics.non_adiabatic_leakage <= 1.0e-5,
        "Leakage {} must be <= 1.0e-5",
        metrics.non_adiabatic_leakage
    );
    assert!(
        metrics.braiding_phase_error_rad <= 0.005,
        "Phase error {} rad must be <= 0.005 rad",
        metrics.braiding_phase_error_rad
    );
}

#[test]
fn test_braiding_duration_scaling() {
    let durations = [80.0, 120.0, 200.0, 300.0];
    for &tau in &durations {
        let params = QuantumAcousticAnyonParams {
            braiding_duration_ns: tau,
            ..Default::default()
        };
        let solver = QuantumAcousticAnyonSolver::new(params);
        let fid = solver.compute_braiding_fidelity_pct();
        let leak = solver.compute_non_adiabatic_leakage();
        let err = solver.compute_braiding_phase_error_rad();

        assert!(fid >= 99.90, "Fidelity at tau={} was {} < 99.90%", tau, fid);
        assert!(
            leak <= 1.0e-5,
            "Leakage at tau={} was {} > 1.0e-5",
            tau,
            leak
        );
        assert!(
            err <= 0.005,
            "Phase error at tau={} was {} > 0.005",
            tau,
            err
        );
    }
}
