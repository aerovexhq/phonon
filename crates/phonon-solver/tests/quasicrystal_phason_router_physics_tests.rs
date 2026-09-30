#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological quasicrystal phason-defect routers and higher-dimensional
//! state concentrators.

use phonon_models::quasicrystal_phason_router::QuasicrystalPhasonRouterParams;
use phonon_solver::quasicrystal_phason_router::QuasicrystalPhasonRouterSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = QuasicrystalPhasonRouterParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        1.0,   // below 1.2
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.phason_strain_coupling_mev, 1.0);
    assert_eq!(underflow.quasicrystal_topological_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_phason_frequency_ghz, 1.0);
    assert_eq!(underflow.phason_flip_propagation_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_pump_power_uw, 0.5);
    assert_eq!(underflow.quasicrystal_inflation_ratio, 1.2);
    assert_eq!(underflow.router_channel_separation_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = QuasicrystalPhasonRouterParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        5.0,    // above 3.5
        35.0,   // above 20.0 um
    );
    assert_eq!(overflow.phason_strain_coupling_mev, 35.0);
    assert_eq!(overflow.quasicrystal_topological_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_phason_frequency_ghz, 12.0);
    assert_eq!(overflow.phason_flip_propagation_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_pump_power_uw, 30.0);
    assert_eq!(overflow.quasicrystal_inflation_ratio, 3.5);
    assert_eq!(overflow.router_channel_separation_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = QuasicrystalPhasonRouterParams::default();
    assert_eq!(params.phason_strain_coupling_mev, 16.5);
    assert_eq!(params.quasicrystal_topological_gap_mev, 22.0);
    assert_eq!(params.acoustic_phason_frequency_ghz, 5.7);
    assert_eq!(params.phason_flip_propagation_speed_m_per_s, 1420.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_pump_power_uw, 5.7);
    assert_eq!(params.quasicrystal_inflation_ratio, 1.618);
    assert_eq!(params.router_channel_separation_um, 5.0);

    let solver = QuasicrystalPhasonRouterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.routing_fidelity >= 0.9980,
        "Routing fidelity must be >= 0.9980, got {:.6}",
        metrics.routing_fidelity
    );
    assert!(
        metrics.phason_state_retention_fraction >= 0.9970,
        "Phason state retention fraction must be >= 0.9970, got {:.6}",
        metrics.phason_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_channel_crosstalk_isolation_db >= 55.0,
        "Inter-channel crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_channel_crosstalk_isolation_db
    );
    assert!(
        metrics.topological_mode_dephasing_rate_hz <= 12.0,
        "Topological mode dephasing rate must be <= 12.0 Hz, got {:.4} Hz",
        metrics.topological_mode_dephasing_rate_hz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_phason_strain_coupling_scaling() {
    let mut low_p = QuasicrystalPhasonRouterParams::default();
    low_p.phason_strain_coupling_mev = 2.0;

    let mut high_p = QuasicrystalPhasonRouterParams::default();
    high_p.phason_strain_coupling_mev = 34.0;

    let low_m = QuasicrystalPhasonRouterSolver::new(low_p).evaluate_metrics();
    let high_m = QuasicrystalPhasonRouterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.phason_state_retention_fraction > low_m.phason_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_quasicrystal_topological_gap_scaling() {
    let mut low_p = QuasicrystalPhasonRouterParams::default();
    low_p.quasicrystal_topological_gap_mev = 3.0;

    let mut high_p = QuasicrystalPhasonRouterParams::default();
    high_p.quasicrystal_topological_gap_mev = 43.0;

    let low_m = QuasicrystalPhasonRouterSolver::new(low_p).evaluate_metrics();
    let high_m = QuasicrystalPhasonRouterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.phason_state_retention_fraction > low_m.phason_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_phason_frequency_scaling() {
    let mut low_p = QuasicrystalPhasonRouterParams::default();
    low_p.acoustic_phason_frequency_ghz = 1.5;

    let mut high_p = QuasicrystalPhasonRouterParams::default();
    high_p.acoustic_phason_frequency_ghz = 11.5;

    let low_m = QuasicrystalPhasonRouterSolver::new(low_p).evaluate_metrics();
    let high_m = QuasicrystalPhasonRouterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.phason_state_retention_fraction > low_m.phason_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_phason_flip_propagation_speed_scaling() {
    let mut low_p = QuasicrystalPhasonRouterParams::default();
    low_p.phason_flip_propagation_speed_m_per_s = 300.0;

    let mut high_p = QuasicrystalPhasonRouterParams::default();
    high_p.phason_flip_propagation_speed_m_per_s = 2900.0;

    let low_m = QuasicrystalPhasonRouterSolver::new(low_p).evaluate_metrics();
    let high_m = QuasicrystalPhasonRouterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.phason_state_retention_fraction > low_m.phason_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_p = QuasicrystalPhasonRouterParams::default();
    low_p.cryogenic_temperature_mk = 2.0;

    let mut high_p = QuasicrystalPhasonRouterParams::default();
    high_p.cryogenic_temperature_mk = 48.0;

    let low_m = QuasicrystalPhasonRouterSolver::new(low_p).evaluate_metrics();
    let high_m = QuasicrystalPhasonRouterSolver::new(high_p).evaluate_metrics();

    // Higher temperature degrades performance and increases dephasing
    assert!(low_m.routing_fidelity > high_m.routing_fidelity);
    assert!(low_m.phason_state_retention_fraction > high_m.phason_state_retention_fraction);
    assert!(low_m.topological_protection_gap_mhz > high_m.topological_protection_gap_mhz);
    assert!(low_m.inter_channel_crosstalk_isolation_db > high_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz > low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_pump_power_scaling() {
    let mut low_p = QuasicrystalPhasonRouterParams::default();
    low_p.microwave_pump_power_uw = 1.0;

    let mut high_p = QuasicrystalPhasonRouterParams::default();
    high_p.microwave_pump_power_uw = 29.0;

    let low_m = QuasicrystalPhasonRouterSolver::new(low_p).evaluate_metrics();
    let high_m = QuasicrystalPhasonRouterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.phason_state_retention_fraction > low_m.phason_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_quasicrystal_inflation_ratio_scaling() {
    let mut low_p = QuasicrystalPhasonRouterParams::default();
    low_p.quasicrystal_inflation_ratio = 1.3;

    let mut high_p = QuasicrystalPhasonRouterParams::default();
    high_p.quasicrystal_inflation_ratio = 3.4;

    let low_m = QuasicrystalPhasonRouterSolver::new(low_p).evaluate_metrics();
    let high_m = QuasicrystalPhasonRouterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.phason_state_retention_fraction > low_m.phason_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_router_channel_separation_scaling() {
    let mut low_p = QuasicrystalPhasonRouterParams::default();
    low_p.router_channel_separation_um = 1.0;

    let mut high_p = QuasicrystalPhasonRouterParams::default();
    high_p.router_channel_separation_um = 19.0;

    let low_m = QuasicrystalPhasonRouterSolver::new(low_p).evaluate_metrics();
    let high_m = QuasicrystalPhasonRouterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.phason_state_retention_fraction > low_m.phason_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
