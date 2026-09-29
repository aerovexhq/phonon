//! Integration tests for topological polariton vortices, quantized circulation,
//! acoustic black hole horizons, and all-optical logic switches.

use phonon_models::polariton_condensate::{
    PolaritonCondensateParams, PolaritonGateParams, PolaritonVortexParams,
};
use phonon_solver::polariton_condensate::{PolaritonGateSolver, PolaritonVortexSolver};

#[test]
fn test_quantized_vortex_circulation() {
    let cond = PolaritonCondensateParams::default();

    for &charge in &[1, -1, 2, -2] {
        let vortex = PolaritonVortexParams::new(charge, 1.5);
        let solver = PolaritonVortexSolver::new(cond, vortex);
        let metrics = solver.solve_vortex_metrics();

        assert!(
            (metrics.circulation_quantum_ratio - (charge as f64)).abs() < 1e-4,
            "Circulation quantum ratio must match topological charge {}, got {:.4}",
            charge,
            metrics.circulation_quantum_ratio
        );
    }
}

#[test]
fn test_vortex_core_density_depletion() {
    let cond = PolaritonCondensateParams::default();
    let vortex = PolaritonVortexParams::new(1, 1.5);
    let solver = PolaritonVortexSolver::new(cond, vortex);

    let radii = vec![0.0, 0.5, 1.5, 5.0, 10.0];
    let profile = solver.solve_radial_profile(&radii);

    let n_core = profile[0].1;
    let n_far = profile[4].1;

    assert_eq!(
        n_core, 0.0,
        "Vortex phase singularity requires strictly 0 density at r = 0"
    );
    assert!(
        n_far > n_core,
        "Density must monotonically recover away from vortex core"
    );
    assert!(
        n_far > 1.0,
        "Density at large r must approach background condensate density"
    );
}

#[test]
fn test_acoustic_black_hole_horizon() {
    let cond = PolaritonCondensateParams::default();
    let vortex = PolaritonVortexParams::new(1, 1.5);
    let solver = PolaritonVortexSolver::new(cond, vortex);
    let metrics = solver.solve_vortex_metrics();

    assert!(
        metrics.sonic_horizon_radius_um > 0.05 && metrics.sonic_horizon_radius_um < 5.0,
        "Sonic horizon radius must fall within 0.05 - 5.0 um, got {:.3} um",
        metrics.sonic_horizon_radius_um
    );
    assert!(
        metrics.hawking_temperature_k > 0.0,
        "Sonic Hawking temperature must be positive, got {:.4} K",
        metrics.hawking_temperature_k
    );
}

#[test]
fn test_polariton_transistor_switching_contrast() {
    let cond = PolaritonCondensateParams::default();
    let gate = PolaritonGateParams::new(1.8, 0.997);
    let solver = PolaritonGateSolver::new(cond, gate);
    let metrics = solver.solve_gate_metrics();

    assert!(
        metrics.switching_contrast_db >= 25.0,
        "Optical transistor switching contrast must exceed 25.0 dB, got {:.2} dB",
        metrics.switching_contrast_db
    );
    assert!(
        metrics.switching_latency_ps <= 10.0,
        "Gate switching latency must be sub-10 ps, got {:.2} ps",
        metrics.switching_latency_ps
    );
}
