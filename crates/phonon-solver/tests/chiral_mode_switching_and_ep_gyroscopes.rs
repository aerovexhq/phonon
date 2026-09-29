//! Integration tests for chiral mode switching under EP encirclement and
//! non-Hermitian acoustic gyroscopes.

use phonon_models::polariton_exceptional_point::PolaritonEpParams;
use phonon_solver::polariton_exceptional_point::PolaritonEpSolver;

#[test]
fn test_chiral_mode_purity_and_gyro_bounds() {
    let params = PolaritonEpParams::default();
    let solver = PolaritonEpSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.chiral_mode_purity_pct >= 99.0,
        "Chiral mode purity {}% must be >= 99.0%",
        metrics.chiral_mode_purity_pct
    );
    assert!(
        metrics.gyro_scale_factor_enhancement >= 10.0,
        "Gyro scale factor enhancement {}x must be >= 10.0x",
        metrics.gyro_scale_factor_enhancement
    );
}

#[test]
fn test_encirclement_period_purity_scaling() {
    let p_fast_encirc = PolaritonEpParams {
        encirclement_period_ns: 30.0,
        ..Default::default()
    };
    let p_slow_encirc = PolaritonEpParams {
        encirclement_period_ns: 120.0,
        ..Default::default()
    };

    let solver_fast = PolaritonEpSolver::new(p_fast_encirc);
    let solver_slow = PolaritonEpSolver::new(p_slow_encirc);

    let purity_fast = solver_fast.compute_chiral_mode_purity_pct();
    let purity_slow = solver_slow.compute_chiral_mode_purity_pct();

    assert!(
        purity_slow >= purity_fast,
        "Longer encirclement period should enhance chiral state selection purity"
    );
    assert!(purity_fast >= 99.0);
    assert!(purity_slow >= 99.0);
}

#[test]
fn test_inter_cavity_coupling_gyro_enhancement() {
    let p_low_j = PolaritonEpParams {
        inter_cavity_coupling_j_mhz: 30.0,
        ..Default::default()
    };
    let p_high_j = PolaritonEpParams {
        inter_cavity_coupling_j_mhz: 100.0,
        ..Default::default()
    };

    let solver_low = PolaritonEpSolver::new(p_low_j);
    let solver_high = PolaritonEpSolver::new(p_high_j);

    let gyro_low = solver_low.compute_gyro_scale_factor_enhancement();
    let gyro_high = solver_high.compute_gyro_scale_factor_enhancement();

    assert!(
        gyro_high > gyro_low,
        "Higher coherent inter-cavity coupling should enhance gyroscopic scale factor"
    );
    assert!(gyro_low >= 10.0);
    assert!(gyro_high >= 10.0);
}
