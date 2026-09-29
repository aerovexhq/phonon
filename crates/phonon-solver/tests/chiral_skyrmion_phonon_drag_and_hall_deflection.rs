//! Integration tests for chiral skyrmion-phonon drag, Thiele equation dynamics,
//! and topological Skyrmion Hall angle deflection.

use phonon_models::skyrmion_phonon_drag::SkyrmionPhononParams;
use phonon_solver::skyrmion_phonon_drag::SkyrmionPhononSolver;

#[test]
fn test_default_skyrmion_drag_and_hall_angle() {
    let params = SkyrmionPhononParams::default();
    let solver = SkyrmionPhononSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.skyrmion_drift_velocity_m_s > 100.0,
        "Drift velocity {} m/s must exceed 100.0 m/s",
        metrics.skyrmion_drift_velocity_m_s
    );
    assert!(
        metrics.topological_hall_angle_deg >= 10.0 && metrics.topological_hall_angle_deg <= 70.0,
        "Hall angle {} deg must be between 10.0 and 70.0 deg",
        metrics.topological_hall_angle_deg
    );
    assert!(
        metrics.acoustic_drag_force_pn > 0.0,
        "Acoustic drag force {} pN must be positive",
        metrics.acoustic_drag_force_pn
    );
    assert!(
        metrics.topological_stability_factor >= 1.0,
        "Stability factor {} must be >= 1.0",
        metrics.topological_stability_factor
    );
}

#[test]
fn test_drag_force_linear_strain_scaling() {
    let p_base = SkyrmionPhononParams::default();
    let p_double_strain = SkyrmionPhononParams {
        saw_strain_amplitude_ppm: p_base.saw_strain_amplitude_ppm * 2.0,
        ..p_base
    };

    let solver_base = SkyrmionPhononSolver::new(p_base);
    let solver_double = SkyrmionPhononSolver::new(p_double_strain);

    let f_base = solver_base.compute_acoustic_drag_force_pn();
    let f_double = solver_double.compute_acoustic_drag_force_pn();

    assert!(
        (f_double - 2.0 * f_base).abs() < 1e-6,
        "Acoustic drag force must scale linearly with strain amplitude"
    );
}

#[test]
fn test_racetrack_confinement_hall_deflection() {
    // Tighter racetrack confinement suppresses transverse Magnus deflection:
    let p_loose = SkyrmionPhononParams {
        racetrack_confinement_factor: 0.85,
        ..Default::default()
    };
    let p_tight = SkyrmionPhononParams {
        racetrack_confinement_factor: 0.98,
        ..Default::default()
    };

    let solver_loose = SkyrmionPhononSolver::new(p_loose);
    let solver_tight = SkyrmionPhononSolver::new(p_tight);

    let theta_loose = solver_loose.compute_topological_hall_angle_deg();
    let theta_tight = solver_tight.compute_topological_hall_angle_deg();

    assert!(
        theta_tight < theta_loose,
        "Tighter boundary confinement must reduce Skyrmion Hall angle (tight={} vs loose={})",
        theta_tight,
        theta_loose
    );
    assert!(theta_tight >= 10.0);
    assert!(theta_loose <= 70.0);
}
