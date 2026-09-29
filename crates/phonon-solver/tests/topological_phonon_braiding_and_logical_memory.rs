//! Integration tests for topological phonon braiding gate fidelity,
//! logical state readout, and anyon core separation.

use phonon_models::floquet_anyon_braiding::FloquetAnyonParams;
use phonon_solver::floquet_anyon_braiding::FloquetAnyonSolver;

#[test]
fn test_fidelity_and_readout_bounds() {
    let params = FloquetAnyonParams::default();
    let solver = FloquetAnyonSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.braiding_gate_fidelity_pct >= 99.50,
        "Braiding fidelity {}% must be >= 99.50%",
        metrics.braiding_gate_fidelity_pct
    );
    assert!(
        metrics.logical_readout_snr_db >= 25.0,
        "Logical readout SNR {} dB must be >= 25.0 dB",
        metrics.logical_readout_snr_db
    );
}

#[test]
fn test_anyon_core_separation_fidelity_scaling() {
    let p_close = FloquetAnyonParams {
        anyon_core_separation_um: 2.0,
        ..Default::default()
    };
    let p_far = FloquetAnyonParams {
        anyon_core_separation_um: 7.0,
        ..Default::default()
    };

    let solver_close = FloquetAnyonSolver::new(p_close);
    let solver_far = FloquetAnyonSolver::new(p_far);

    let fid_close = solver_close.compute_braiding_gate_fidelity_pct();
    let fid_far = solver_far.compute_braiding_gate_fidelity_pct();

    assert!(
        fid_far >= fid_close,
        "Greater anyon separation should minimize wavefunction overlap errors (far={} vs close={})",
        fid_far,
        fid_close
    );
    assert!(fid_close >= 99.50);
    assert!(fid_far >= 99.50);
}

#[test]
fn test_sub_kelvin_thermal_protection() {
    let p_cold = FloquetAnyonParams {
        ambient_temperature_mk: 15.0,
        ..Default::default()
    };
    let p_warm = FloquetAnyonParams {
        ambient_temperature_mk: 60.0,
        ..Default::default()
    };

    let solver_cold = FloquetAnyonSolver::new(p_cold);
    let solver_warm = FloquetAnyonSolver::new(p_warm);

    let fid_cold = solver_cold.compute_braiding_gate_fidelity_pct();
    let fid_warm = solver_warm.compute_braiding_gate_fidelity_pct();

    assert!(
        fid_cold >= fid_warm,
        "Lower temperature should enhance topological braiding fidelity"
    );
    assert!(fid_warm >= 99.50);
}
