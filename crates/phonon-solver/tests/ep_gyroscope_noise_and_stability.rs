#![deny(unsafe_code)]

use phonon_models::non_hermitian_ep_gyroscope::EpGyroscopeParams;
use phonon_solver::non_hermitian_ep_gyroscope::EpGyroscopeSolver;

#[test]
fn test_noise_scaling_with_temperature_and_q() {
    let p_room = EpGyroscopeParams {
        thermal_noise_temp_k: 300.0,
        quality_factor: 20000.0,
        ..Default::default()
    };
    let s_room = EpGyroscopeSolver::new(p_room);
    let m_room = s_room.solve();

    let p_cryo = EpGyroscopeParams {
        thermal_noise_temp_k: 77.0,
        quality_factor: 45000.0,
        ..Default::default()
    };
    let s_cryo = EpGyroscopeSolver::new(p_cryo);
    let m_cryo = s_cryo.solve();

    assert!(
        m_cryo.angle_random_walk_deg_sqrthr < m_room.angle_random_walk_deg_sqrthr,
        "Cryo ARW {} must be lower than room ARW {}",
        m_cryo.angle_random_walk_deg_sqrthr,
        m_room.angle_random_walk_deg_sqrthr
    );

    assert!(
        m_cryo.bias_stability_deg_hr < m_room.bias_stability_deg_hr,
        "Cryo bias {} must be more stable than room bias {}",
        m_cryo.bias_stability_deg_hr,
        m_room.bias_stability_deg_hr
    );
}

#[test]
fn test_dynamic_range_across_radii() {
    for &radius in &[100.0, 200.0, 500.0, 1000.0] {
        let params = EpGyroscopeParams {
            ring_radius_um: radius,
            ..Default::default()
        };
        let solver = EpGyroscopeSolver::new(params);
        let metrics = solver.solve();

        assert!(
            metrics.dynamic_range_db >= 120.0,
            "Dynamic range at R={} um: {} dB < 120.0 dB",
            radius,
            metrics.dynamic_range_db
        );
    }
}
