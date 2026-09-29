#![deny(unsafe_code)]

use phonon_models::skyrmion_braiding_memory::SkyrmionMemoryParams;
use phonon_solver::skyrmion_braiding_memory::SkyrmionMemorySolver;

#[test]
fn test_default_skyrmion_memory() {
    let params = SkyrmionMemoryParams::default();
    let solver = SkyrmionMemorySolver::new(params);
    let metrics = solver.solve();

    // Drift velocity >= 250.0 m/s
    assert!(
        metrics.drift_velocity_m_s >= 250.0,
        "Velocity {} m/s < 250.0 m/s",
        metrics.drift_velocity_m_s
    );

    // Bit error rate <= 1.0e-12
    assert!(
        metrics.bit_error_rate <= 1.0e-12,
        "BER {} > 1.0e-12",
        metrics.bit_error_rate
    );

    // Retention time >= 15.0 years
    assert!(
        metrics.retention_years >= 15.0,
        "Retention {} years < 15.0 years",
        metrics.retention_years
    );

    // Write energy <= 0.5 fJ
    assert!(
        metrics.write_energy_fj <= 0.5,
        "Write energy {} fJ > 0.5 fJ",
        metrics.write_energy_fj
    );

    // Braiding fidelity >= 99.5%
    assert!(
        metrics.braiding_fidelity_pct >= 99.5,
        "Braiding fidelity {}% < 99.5%",
        metrics.braiding_fidelity_pct
    );

    // Hall angle suppression >= 90.0%
    assert!(
        metrics.hall_angle_suppression_pct >= 90.0,
        "Hall suppression {}% < 90.0%",
        metrics.hall_angle_suppression_pct
    );
}

#[test]
fn test_drift_velocity_and_retention_scaling() {
    let p_low_coupling = SkyrmionMemoryParams {
        magnetoelastic_coupling_j_m3: 3.5e6,
        ..Default::default()
    };
    let s_low = SkyrmionMemorySolver::new(p_low_coupling);
    let m_low = s_low.solve();

    let p_high_coupling = SkyrmionMemoryParams {
        magnetoelastic_coupling_j_m3: 15.0e6,
        ..Default::default()
    };
    let s_high = SkyrmionMemorySolver::new(p_high_coupling);
    let m_high = s_high.solve();

    assert!(m_high.drift_velocity_m_s > m_low.drift_velocity_m_s);
    assert!(m_low.drift_velocity_m_s >= 250.0);

    // Retention vs thermal stability
    let p_low_stab = SkyrmionMemoryParams {
        thermal_stability_factor: 55.0,
        ..Default::default()
    };
    let p_high_stab = SkyrmionMemoryParams {
        thermal_stability_factor: 75.0,
        ..Default::default()
    };

    let m_low_stab = SkyrmionMemorySolver::new(p_low_stab).solve();
    let m_high_stab = SkyrmionMemorySolver::new(p_high_stab).solve();

    assert!(m_high_stab.retention_years > m_low_stab.retention_years);
    assert!(m_low_stab.retention_years >= 15.0);
}
