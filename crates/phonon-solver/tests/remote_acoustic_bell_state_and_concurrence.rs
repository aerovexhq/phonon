//! Integration tests for remote acoustic Bell state creation, entanglement concurrence,
//! and cryogenic dephasing resilience.

use phonon_models::quantum_phonon_teleportation::PhononTeleportationParams;
use phonon_solver::quantum_phonon_teleportation::PhononTeleportationSolver;

#[test]
fn test_remote_acoustic_bell_concurrence() {
    let params = PhononTeleportationParams::default();
    let solver = PhononTeleportationSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.acoustic_bell_concurrence >= 0.920,
        "Acoustic Bell concurrence below threshold: {}",
        metrics.acoustic_bell_concurrence
    );
}

#[test]
fn test_cryogenic_temperature_sweep() {
    for temp_mk in [10.0, 20.0, 40.0, 60.0] {
        let p = PhononTeleportationParams {
            cryogenic_temperature_mk: temp_mk,
            ..Default::default()
        };
        let solver = PhononTeleportationSolver::new(p);
        let m = solver.evaluate_metrics();

        assert!(
            m.acoustic_bell_concurrence >= 0.920,
            "Concurrence at {} mK below threshold: {}",
            temp_mk,
            m.acoustic_bell_concurrence
        );
        assert!(
            m.state_transfer_fidelity >= 0.960,
            "Fidelity at {} mK below threshold: {}",
            temp_mk,
            m.state_transfer_fidelity
        );
        assert!(m.is_physically_compliant);
    }
}

#[test]
fn test_coupling_rate_sweep() {
    for g_mhz in [25.0, 30.0, 35.0, 40.0] {
        let p = PhononTeleportationParams {
            electromechanical_coupling_mhz: g_mhz,
            ..Default::default()
        };
        let solver = PhononTeleportationSolver::new(p);
        let m = solver.evaluate_metrics();

        assert!(m.quantum_link_bandwidth_mhz >= 50.0);
        assert!(m.acoustic_bell_concurrence >= 0.920);
        assert!(m.is_physically_compliant);
    }
}
