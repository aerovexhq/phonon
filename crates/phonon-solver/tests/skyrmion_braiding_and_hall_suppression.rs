#![deny(unsafe_code)]

use phonon_models::skyrmion_braiding_memory::SkyrmionMemoryParams;
use phonon_solver::skyrmion_braiding_memory::SkyrmionMemorySolver;

#[test]
fn test_braiding_fidelity_and_damping() {
    let dampings = [0.008, 0.015, 0.025, 0.040];
    for &alpha in &dampings {
        let params = SkyrmionMemoryParams {
            gilbert_damping: alpha,
            ..Default::default()
        };
        let solver = SkyrmionMemorySolver::new(params);
        let metrics = solver.solve();

        assert!(metrics.braiding_fidelity_pct >= 99.5);
        assert!(metrics.drift_velocity_m_s >= 250.0);
    }
}

#[test]
fn test_track_width_and_hall_suppression() {
    let p_narrow = SkyrmionMemoryParams {
        nanowire_track_width_nm: 35.0,
        ..Default::default()
    };
    let p_wide = SkyrmionMemoryParams {
        nanowire_track_width_nm: 90.0,
        ..Default::default()
    };

    let m_narrow = SkyrmionMemorySolver::new(p_narrow).solve();
    let m_wide = SkyrmionMemorySolver::new(p_wide).solve();

    assert!(m_wide.hall_angle_suppression_pct > m_narrow.hall_angle_suppression_pct);
    assert!(m_narrow.hall_angle_suppression_pct >= 90.0);
}
