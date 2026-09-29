#![deny(unsafe_code)]

use phonon_models::non_hermitian_ep_gyroscope::EpGyroscopeParams;
use phonon_solver::non_hermitian_ep_gyroscope::EpGyroscopeSolver;

#[test]
fn test_default_ep_gyroscope() {
    let params = EpGyroscopeParams::default();
    let solver = EpGyroscopeSolver::new(params);
    let metrics = solver.solve();

    // Scale-factor enhancement >= 15.0x
    assert!(
        metrics.scale_factor_enhancement >= 15.0,
        "Scale enhancement {} < 15.0x",
        metrics.scale_factor_enhancement
    );

    // Dynamic range >= 120.0 dB
    assert!(
        metrics.dynamic_range_db >= 120.0,
        "Dynamic range {} dB < 120.0 dB",
        metrics.dynamic_range_db
    );

    // Angle random walk <= 0.001 deg/sqrt(hr)
    assert!(
        metrics.angle_random_walk_deg_sqrthr <= 0.001,
        "ARW {} > 0.001 deg/sqrt(hr)",
        metrics.angle_random_walk_deg_sqrthr
    );

    // Bias stability <= 0.005 deg/hr
    assert!(
        metrics.bias_stability_deg_hr <= 0.005,
        "Bias stability {} > 0.005 deg/hr",
        metrics.bias_stability_deg_hr
    );

    // Petermann factor >= 1.0
    assert!(
        metrics.petermann_factor >= 1.0,
        "Petermann factor {} < 1.0",
        metrics.petermann_factor
    );

    assert!(
        metrics.is_physically_compliant,
        "Default EP gyroscope must be compliant"
    );
}

#[test]
fn test_scale_enhancement_and_ep_proximity() {
    // Distant from EP: gamma/kappa = 0.90
    let p_far = EpGyroscopeParams {
        coupling_rate_mhz: 2.0,
        gain_loss_rate_mhz: 1.80,
        ..Default::default()
    };
    let s_far = EpGyroscopeSolver::new(p_far);
    let m_far = s_far.solve();

    // Very close to EP: gamma/kappa = 0.995
    let p_close = EpGyroscopeParams {
        coupling_rate_mhz: 2.0,
        gain_loss_rate_mhz: 1.99,
        ..Default::default()
    };
    let s_close = EpGyroscopeSolver::new(p_close);
    let m_close = s_close.solve();

    assert!(
        m_close.scale_factor_enhancement >= m_far.scale_factor_enhancement,
        "Operating closer to EP must increase scale enhancement: {} vs {}",
        m_close.scale_factor_enhancement,
        m_far.scale_factor_enhancement
    );

    assert!(
        m_close.petermann_factor > m_far.petermann_factor,
        "Petermann factor must increase closer to EP: {} vs {}",
        m_close.petermann_factor,
        m_far.petermann_factor
    );
}
