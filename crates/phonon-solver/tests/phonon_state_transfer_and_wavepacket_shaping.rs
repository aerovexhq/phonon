//! Integration tests for quantum phonon state transfer, propagation delay,
//! cryogenic attenuation, and wavepacket shaping.

use phonon_models::quantum_phonon_teleportation::PhononTeleportationParams;
use phonon_solver::quantum_phonon_teleportation::PhononTeleportationSolver;

#[test]
fn test_acoustic_delay_and_thermal_occupancy() {
    let params = PhononTeleportationParams::default();
    let solver = PhononTeleportationSolver::new(params);

    // L = 320 um, v = 3488 m/s => tau ~ 91.74 ns
    let delay_ns = solver.compute_propagation_delay_ns();
    assert!(
        delay_ns > 80.0 && delay_ns < 110.0,
        "Propagation delay out of bounds: {}",
        delay_ns
    );

    // At 15 mK and 4.8 GHz, thermal phonon occupancy n_th < 1e-5
    let n_th = solver.compute_thermal_phonon_occupancy();
    assert!(n_th < 1.0e-5, "Thermal phonon occupancy too high: {}", n_th);
}

#[test]
fn test_state_transfer_fidelity_and_loss() {
    let params = PhononTeleportationParams::default();
    let solver = PhononTeleportationSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.state_transfer_fidelity >= 0.960,
        "Fidelity below threshold: {}",
        metrics.state_transfer_fidelity
    );
    assert!(
        metrics.phonon_loss_probability <= 0.020,
        "Phonon loss probability above limit: {}",
        metrics.phonon_loss_probability
    );
    assert!(
        metrics.quantum_link_bandwidth_mhz >= 50.0,
        "Quantum link bandwidth below 50 MHz: {}",
        metrics.quantum_link_bandwidth_mhz
    );
    assert!(
        metrics.is_physically_compliant,
        "Solver metrics must be compliant"
    );
}

#[test]
fn test_wavepacket_shaping_variations() {
    for shaping_eff in [0.990, 0.995, 0.999] {
        let p = PhononTeleportationParams {
            wavepacket_shaping_efficiency: shaping_eff,
            ..Default::default()
        };
        let solver = PhononTeleportationSolver::new(p);
        let m = solver.evaluate_metrics();

        assert!(m.state_transfer_fidelity >= 0.960);
        assert!(m.phonon_loss_probability <= 0.020);
        assert!(m.is_physically_compliant);
    }
}
