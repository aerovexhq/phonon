//! Integration tests for many-body localization disorder protection
//! and robustness against Floquet drive perturbations.

use phonon_models::quantum_time_crystal::QuantumTimeCrystalParams;
use phonon_solver::quantum_time_crystal::FloquetTimeCrystalSolver;

#[test]
fn test_disorder_dependence_and_thermalization_protection() {
    let low_disorder_params = QuantumTimeCrystalParams {
        disorder_strength_w: 1.6,
        ..Default::default()
    };
    let high_disorder_params = QuantumTimeCrystalParams {
        disorder_strength_w: 3.8,
        ..Default::default()
    };

    let low_solver = FloquetTimeCrystalSolver::new(low_disorder_params);
    let high_solver = FloquetTimeCrystalSolver::new(high_disorder_params);

    let low_lifetime = low_solver.compute_time_crystal_lifetime_cycles();
    let high_lifetime = high_solver.compute_time_crystal_lifetime_cycles();

    assert!(
        high_lifetime > low_lifetime,
        "Higher disorder W must provide stronger MBL protection and longer lifetime: {:.1} vs {:.1}",
        high_lifetime,
        low_lifetime
    );

    // Robustness against pulse perturbation epsilon
    let perturbed_params = QuantumTimeCrystalParams {
        pulse_imperfection_epsilon: 0.08,
        ..Default::default()
    };
    let perturbed_solver = FloquetTimeCrystalSolver::new(perturbed_params);
    let perturbed_metrics = perturbed_solver.solve();

    assert!(
        perturbed_metrics.spectral_rigidity_contrast_db >= 20.0,
        "Perturbed time crystal must maintain rigidity contrast >= 20.0 dB, got {:.2} dB",
        perturbed_metrics.spectral_rigidity_contrast_db
    );
    assert!(
        perturbed_metrics.time_crystal_lifetime_cycles >= 1000.0,
        "Perturbed lifetime must still exceed 1000 cycles, got {:.1} cycles",
        perturbed_metrics.time_crystal_lifetime_cycles
    );
}
