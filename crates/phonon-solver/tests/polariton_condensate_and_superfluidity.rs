//! Integration tests for microcavity exciton-polariton condensates,
//! non-equilibrium open-dissipative kinetics, and Bogoliubov superfluidity.

use phonon_models::polariton_condensate::PolaritonCondensateParams;
use phonon_solver::polariton_condensate::PolaritonCondensateSolver;

#[test]
fn test_polariton_condensation_threshold_and_density() {
    let params_sub = PolaritonCondensateParams::new(2.5e-4, 0.020, 0.01);
    let params_above = PolaritonCondensateParams::new(2.5e-4, 0.020, 0.50);

    let solver_sub = PolaritonCondensateSolver::new(params_sub);
    let solver_above = PolaritonCondensateSolver::new(params_above);

    let p_th = params_above.threshold_pump_power();
    assert!(
        p_th > 0.0 && p_th <= 0.05,
        "Threshold pump power must be positive and reasonable, got {:.5}",
        p_th
    );

    let n_sub = solver_sub.solve_condensate_metrics().condensate_density_um2;
    assert_eq!(
        n_sub, 0.0,
        "Below threshold, coherent condensate density must be strictly zero"
    );

    let n_above = solver_above
        .solve_condensate_metrics()
        .condensate_density_um2;
    assert!(
        n_above > 1.0,
        "Above threshold, macroscopic condensate density must form, got {:.3} um^-2",
        n_above
    );
}

#[test]
fn test_bogoliubov_sound_speed_and_healing_length() {
    let params = PolaritonCondensateParams::new(2.5e-4, 0.020, 0.50);
    let solver = PolaritonCondensateSolver::new(params);
    let metrics = solver.solve_condensate_metrics();

    assert!(
        metrics.sound_speed_m_s > 1.0e5,
        "Bogoliubov sound speed must exceed 100 km/s (1e5 m/s), got {:.1} m/s",
        metrics.sound_speed_m_s
    );
    assert!(
        metrics.sound_speed_m_s < 2.0e6,
        "Sound speed must remain sub-relativistic (< 2e6 m/s), got {:.1} m/s",
        metrics.sound_speed_m_s
    );

    assert!(
        metrics.healing_length_um >= 0.5 && metrics.healing_length_um <= 5.0,
        "Acoustic healing length must be within 0.5 - 5.0 um, got {:.3} um",
        metrics.healing_length_um
    );

    // Bogoliubov linear dispersion at low k: epsilon(k) ~ hbar * cs * k
    let energy_k = solver.solve_bogoliubov_energy_mev(0.1);
    assert!(
        energy_k > 0.01 && energy_k < 1.0,
        "Acoustic phonon-like Bogoliubov branch must have meV-scale excitation, got {:.4} meV",
        energy_k
    );
}

#[test]
fn test_superfluid_fraction_exceeds_80_percent() {
    let params = PolaritonCondensateParams::new(2.5e-4, 0.020, 0.50);
    let solver = PolaritonCondensateSolver::new(params);
    let metrics = solver.solve_condensate_metrics();

    assert!(
        metrics.superfluid_fraction >= 0.80,
        "Superfluid fraction must exceed 80% (0.80), got {:.4}",
        metrics.superfluid_fraction
    );
    assert!(
        metrics.superfluid_fraction <= 0.99,
        "Superfluid fraction must stay <= 99%, got {:.4}",
        metrics.superfluid_fraction
    );

    // Landau critical velocity test
    let cs = metrics.sound_speed_m_s;
    let drag_sub = solver.solve_superfluid_drag_reduction(cs * 0.10);
    let drag_sup = solver.solve_superfluid_drag_reduction(cs * 1.50);

    assert!(
        drag_sub < 0.35,
        "Sub-critical flow must experience dramatic drag suppression, got {:.3}",
        drag_sub
    );
    assert_eq!(
        drag_sup, 1.0,
        "Supersonic flow (v > cs) must recover full normal fluid drag (1.0)"
    );
}
