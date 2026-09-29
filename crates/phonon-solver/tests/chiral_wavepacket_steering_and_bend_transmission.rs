#![deny(unsafe_code)]

use phonon_models::floquet_acoustic_chern::FloquetAcousticChernParams;
use phonon_solver::floquet_acoustic_chern::FloquetAcousticChernSolver;

#[test]
fn test_forward_bend_efficiency_vs_bend_angle() {
    for &angle in &[30.0, 60.0, 90.0, 120.0] {
        let params = FloquetAcousticChernParams {
            bend_angle_deg: angle,
            ..Default::default()
        };
        let solver = FloquetAcousticChernSolver::new(params);
        let metrics = solver.solve();

        assert!(
            metrics.forward_bend_efficiency_pct >= 92.0,
            "Bend eff at {} deg: {}% < 92.0%",
            angle,
            metrics.forward_bend_efficiency_pct
        );

        assert!(
            metrics.reverse_isolation_db >= 30.0,
            "Reverse isolation at {} deg: {} dB < 30.0 dB",
            angle,
            metrics.reverse_isolation_db
        );
    }
}

#[test]
fn test_beam_steering_angle_tuning() {
    let p_phase0 = FloquetAcousticChernParams {
        drive_phase_rad: 0.0,
        ..Default::default()
    };
    let s0 = FloquetAcousticChernSolver::new(p_phase0);
    let m0 = s0.solve();

    let p_phase_pi2 = FloquetAcousticChernParams {
        drive_phase_rad: std::f64::consts::PI / 2.0,
        ..Default::default()
    };
    let s_pi2 = FloquetAcousticChernSolver::new(p_phase_pi2);
    let m_pi2 = s_pi2.solve();

    assert!(
        (m0.beam_steering_angle_deg - m_pi2.beam_steering_angle_deg).abs() > 1.0,
        "Drive phase must tune steering angle: {} vs {}",
        m0.beam_steering_angle_deg,
        m_pi2.beam_steering_angle_deg
    );
}
