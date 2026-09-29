#![deny(unsafe_code)]

use phonon_models::chiral_spintronic_memristor::SpintronicMemristorParams;
use phonon_solver::chiral_spintronic_memristor::SpintronicMemristorSolver;

#[test]
fn test_default_chiral_spintronic_memristor() {
    let params = SpintronicMemristorParams::default();
    let solver = SpintronicMemristorSolver::new(params);
    let metrics = solver.solve();

    // Programming energy <= 10.0 fJ
    assert!(
        metrics.programming_energy_fj <= 10.0,
        "Energy {} fJ > 10.0 fJ",
        metrics.programming_energy_fj
    );
    assert!(metrics.programming_energy_fj > 0.0);

    // Retention time >= 10.0 years
    assert!(
        metrics.retention_time_years >= 10.0,
        "Retention {} years < 10.0 years",
        metrics.retention_time_years
    );

    // Conductance on/off ratio >= 10.0
    assert!(
        metrics.conductance_on_off_ratio >= 10.0,
        "On/Off ratio {} < 10.0",
        metrics.conductance_on_off_ratio
    );

    // STDP learning fidelity >= 95.0%
    assert!(
        metrics.stdp_learning_fidelity_pct >= 95.0,
        "STDP fidelity {} < 95.0%",
        metrics.stdp_learning_fidelity_pct
    );

    // Neuromorphic compute efficiency >= 150.0 TOPS/W
    assert!(
        metrics.crossbar_energy_efficiency_topsw >= 150.0,
        "Efficiency {} TOPS/W < 150.0",
        metrics.crossbar_energy_efficiency_topsw
    );

    // Weight linearity error <= 2.5%
    assert!(
        metrics.weight_linearity_error_pct <= 2.5,
        "Linearity error {}% > 2.5%",
        metrics.weight_linearity_error_pct
    );
}

#[test]
fn test_domain_wall_motion_energy_and_retention() {
    // Test scaling with thermal stability factor
    let p_low = SpintronicMemristorParams {
        thermal_stability_factor: 50.0,
        ..Default::default()
    };
    let s_low = SpintronicMemristorSolver::new(p_low);
    let m_low = s_low.solve();

    let p_high = SpintronicMemristorParams {
        thermal_stability_factor: 70.0,
        ..Default::default()
    };
    let s_high = SpintronicMemristorSolver::new(p_high);
    let m_high = s_high.solve();

    assert!(m_high.retention_time_years > m_low.retention_time_years);
    assert!(m_low.retention_time_years >= 10.0);

    // Test scaling with pulse power and duration
    let p_fast = SpintronicMemristorParams {
        pulse_power_uw: 0.5,
        pulse_duration_ns: 1.0,
        ..Default::default()
    };
    let s_fast = SpintronicMemristorSolver::new(p_fast);
    let m_fast = s_fast.solve();

    assert!(m_fast.programming_energy_fj < 1.0);
}
