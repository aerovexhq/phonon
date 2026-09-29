//! Integration tests for non-Hermitian exceptional points in topological polariton condensates,
//! square-root sensitivity enhancement, and macroscopic condensation thresholds.

use phonon_models::polariton_exceptional_point::PolaritonEpParams;
use phonon_solver::polariton_exceptional_point::PolaritonEpSolver;

#[test]
fn test_default_polariton_ep_metrics() {
    let params = PolaritonEpParams::default();
    let solver = PolaritonEpSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.exceptional_sensitivity_db >= 30.0,
        "Exceptional sensitivity {} dB must be >= 30.0 dB",
        metrics.exceptional_sensitivity_db
    );
    assert!(
        metrics.condensation_threshold_mw <= 5.0,
        "Condensation threshold {} mW must be <= 5.0 mW",
        metrics.condensation_threshold_mw
    );
    assert!(
        metrics.polariton_laser_linewidth_mhz <= 50.0,
        "Laser linewidth {} MHz must be <= 50.0 MHz",
        metrics.polariton_laser_linewidth_mhz
    );
    assert_eq!(
        metrics.topological_winding_charge, 0.5,
        "Topological winding charge must equal 0.5 (EP2 branch-cut singularity)"
    );
}

#[test]
fn test_perturbation_strain_sensitivity_scaling() {
    // Square-root sensitivity enhancement: smaller perturbation yields higher relative enhancement
    let p_small_strain = PolaritonEpParams {
        perturbation_strain_ppm: 1.0,
        ..Default::default()
    };
    let p_large_strain = PolaritonEpParams {
        perturbation_strain_ppm: 25.0,
        ..Default::default()
    };

    let solver_small = PolaritonEpSolver::new(p_small_strain);
    let solver_large = PolaritonEpSolver::new(p_large_strain);

    let sens_small = solver_small.compute_exceptional_sensitivity_db();
    let sens_large = solver_large.compute_exceptional_sensitivity_db();

    assert!(
        sens_small > sens_large,
        "Smaller strain perturbation should yield higher relative sensitivity enhancement"
    );
    assert!(sens_small >= 30.0);
    assert!(sens_large >= 30.0);
}

#[test]
fn test_cavity_decay_linewidth_scaling() {
    let p_high_q = PolaritonEpParams {
        cavity_photon_decay_mhz: 15.0,
        ..Default::default()
    };
    let p_mod_q = PolaritonEpParams {
        cavity_photon_decay_mhz: 45.0,
        ..Default::default()
    };

    let solver_high = PolaritonEpSolver::new(p_high_q);
    let solver_mod = PolaritonEpSolver::new(p_mod_q);

    let lw_high = solver_high.compute_polariton_laser_linewidth_mhz();
    let lw_mod = solver_mod.compute_polariton_laser_linewidth_mhz();

    assert!(
        lw_high < lw_mod,
        "Lower cavity photon decay must yield narrower polariton laser emission linewidth"
    );
    assert!(lw_high <= 50.0);
    assert!(lw_mod <= 50.0);
}
