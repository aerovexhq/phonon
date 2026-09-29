//! Integration tests for chiral magnetic domain wall solitons,
//! topological magnetoplasmons (TMP), and non-reciprocal isolation waveguides.

use phonon_models::axion_electrodynamics::TopologicalMagnetoplasmonParams;
use phonon_solver::axion_electrodynamics::TopologicalMagnetoplasmonSolver;

#[test]
fn test_chiral_magnetic_domain_wall_soliton() {
    let params = TopologicalMagnetoplasmonParams::default();
    let solver = TopologicalMagnetoplasmonSolver::new(params);

    // Soliton profile checks:
    // theta(0) = 2 * atan(1) = pi/2
    let theta_mid = solver.evaluate_soliton_profile(0.0);
    assert!(
        (theta_mid - std::f64::consts::FRAC_PI_2).abs() < 1.0e-5,
        "Expected theta(0) = pi/2, got {:.5}",
        theta_mid
    );

    // Asymptotic values
    let theta_plus = solver.evaluate_soliton_profile(200.0);
    let theta_minus = solver.evaluate_soliton_profile(-200.0);
    assert!(
        (theta_plus - std::f64::consts::PI).abs() < 1.0e-4,
        "Expected theta(+inf) -> pi, got {:.5}",
        theta_plus
    );
    assert!(
        theta_minus < 1.0e-4,
        "Expected theta(-inf) -> 0, got {:.5}",
        theta_minus
    );

    // Topological charge
    let charge = solver.compute_soliton_topological_charge();
    assert_eq!(charge, 1, "Topological charge must be +1 for positive DMI");

    // Confinement depth
    let conf_depth = solver.compute_confinement_depth_nm();
    assert!(
        conf_depth >= params.domain_wall_width_nm
            && conf_depth <= params.domain_wall_width_nm * 2.0,
        "Expected transverse confinement in [{}, {}] nm, got {:.2} nm",
        params.domain_wall_width_nm,
        params.domain_wall_width_nm * 2.0,
        conf_depth
    );
}

#[test]
fn test_topological_magnetoplasmon_non_reciprocal_isolation() {
    let params = TopologicalMagnetoplasmonParams::default();
    let solver = TopologicalMagnetoplasmonSolver::new(params);
    let metrics = solver.solve();

    // Forward loss must be minimal
    assert!(
        metrics.forward_transmission_db > -2.0,
        "Forward transmission loss should be < 2.0 dB, got {:.2} dB",
        metrics.forward_transmission_db
    );

    // Backward attenuation must be strong
    assert!(
        metrics.backward_isolation_db <= -25.0,
        "Backward isolation should be <= -25.0 dB, got {:.2} dB",
        metrics.backward_isolation_db
    );

    // Isolation contrast must satisfy >= 25.0 dB
    assert!(
        metrics.non_reciprocal_isolation_db >= 25.0,
        "Non-reciprocal isolation contrast must be >= 25.0 dB, got {:.2} dB",
        metrics.non_reciprocal_isolation_db
    );

    // Edge velocity
    assert!(
        metrics.chiral_edge_velocity_m_s >= 1.0e5,
        "Chiral edge velocity must be >= 1e5 m/s, got {:.2e} m/s",
        metrics.chiral_edge_velocity_m_s
    );
}
