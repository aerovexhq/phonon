#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological optomechanical polariton switchyards and multi-channel routing networks.

use phonon_models::optomechanical_switchyard::OptomechanicalSwitchyardParams;
use phonon_solver::optomechanical_switchyard::OptomechanicalSwitchyardSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = OptomechanicalSwitchyardParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.05,  // below 0.1 C
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.optomechanical_coupling_energy_mev, 1.0);
    assert_eq!(underflow.topological_polariton_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.polariton_routing_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.optical_control_pump_power_uw, 0.5);
    assert_eq!(underflow.synthetic_optomechanical_cooperativity_c, 0.1);
    assert_eq!(underflow.switchyard_channel_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = OptomechanicalSwitchyardParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        8.0,    // above 5.0 C
        35.0,   // above 20.0 um
    );
    assert_eq!(overflow.optomechanical_coupling_energy_mev, 35.0);
    assert_eq!(overflow.topological_polariton_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.polariton_routing_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.optical_control_pump_power_uw, 30.0);
    assert_eq!(overflow.synthetic_optomechanical_cooperativity_c, 5.0);
    assert_eq!(overflow.switchyard_channel_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = OptomechanicalSwitchyardParams::default();
    assert_eq!(params.optomechanical_coupling_energy_mev, 16.5);
    assert_eq!(params.topological_polariton_gap_mev, 22.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.8);
    assert_eq!(params.polariton_routing_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.optical_control_pump_power_uw, 5.8);
    assert_eq!(params.synthetic_optomechanical_cooperativity_c, 1.6);
    assert_eq!(params.switchyard_channel_pitch_um, 4.8);

    let solver = OptomechanicalSwitchyardSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.routing_fidelity >= 0.9980,
        "Routing fidelity must be >= 0.9980, got {:.6}",
        metrics.routing_fidelity
    );
    assert!(
        metrics.polariton_state_retention_fraction >= 0.9970,
        "Polariton state retention fraction must be >= 0.9970, got {:.6}",
        metrics.polariton_state_retention_fraction
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
fn test_optomechanical_coupling_energy_scaling() {
    let mut low_p = OptomechanicalSwitchyardParams::default();
    low_p.optomechanical_coupling_energy_mev = 2.0;
    let mut high_p = OptomechanicalSwitchyardParams::default();
    high_p.optomechanical_coupling_energy_mev = 30.0;

    let low_m = OptomechanicalSwitchyardSolver::new(low_p).evaluate_metrics();
    let high_m = OptomechanicalSwitchyardSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.polariton_state_retention_fraction > low_m.polariton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_polariton_gap_scaling() {
    let mut low_p = OptomechanicalSwitchyardParams::default();
    low_p.topological_polariton_gap_mev = 5.0;
    let mut high_p = OptomechanicalSwitchyardParams::default();
    high_p.topological_polariton_gap_mev = 40.0;

    let low_m = OptomechanicalSwitchyardSolver::new(low_p).evaluate_metrics();
    let high_m = OptomechanicalSwitchyardSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.polariton_state_retention_fraction > low_m.polariton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = OptomechanicalSwitchyardParams::default();
    low_p.acoustic_drive_frequency_ghz = 2.0;
    let mut high_p = OptomechanicalSwitchyardParams::default();
    high_p.acoustic_drive_frequency_ghz = 10.0;

    let low_m = OptomechanicalSwitchyardSolver::new(low_p).evaluate_metrics();
    let high_m = OptomechanicalSwitchyardSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.polariton_state_retention_fraction > low_m.polariton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_polariton_routing_speed_scaling() {
    let mut low_p = OptomechanicalSwitchyardParams::default();
    low_p.polariton_routing_speed_m_per_s = 400.0;
    let mut high_p = OptomechanicalSwitchyardParams::default();
    high_p.polariton_routing_speed_m_per_s = 2600.0;

    let low_m = OptomechanicalSwitchyardSolver::new(low_p).evaluate_metrics();
    let high_m = OptomechanicalSwitchyardSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.polariton_state_retention_fraction > low_m.polariton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold_p = OptomechanicalSwitchyardParams::default();
    cold_p.cryogenic_temperature_mk = 2.0;
    let mut warm_p = OptomechanicalSwitchyardParams::default();
    warm_p.cryogenic_temperature_mk = 45.0;

    let cold_m = OptomechanicalSwitchyardSolver::new(cold_p).evaluate_metrics();
    let warm_m = OptomechanicalSwitchyardSolver::new(warm_p).evaluate_metrics();

    assert!(cold_m.routing_fidelity > warm_m.routing_fidelity);
    assert!(cold_m.polariton_state_retention_fraction > warm_m.polariton_state_retention_fraction);
    assert!(cold_m.topological_protection_gap_mhz > warm_m.topological_protection_gap_mhz);
    assert!(cold_m.inter_channel_crosstalk_isolation_db > warm_m.inter_channel_crosstalk_isolation_db);
    assert!(cold_m.topological_mode_dephasing_rate_hz < warm_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_optical_control_pump_power_scaling() {
    let mut low_p = OptomechanicalSwitchyardParams::default();
    low_p.optical_control_pump_power_uw = 1.0;
    let mut high_p = OptomechanicalSwitchyardParams::default();
    high_p.optical_control_pump_power_uw = 25.0;

    let low_m = OptomechanicalSwitchyardSolver::new(low_p).evaluate_metrics();
    let high_m = OptomechanicalSwitchyardSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.polariton_state_retention_fraction > low_m.polariton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synthetic_optomechanical_cooperativity_scaling() {
    let mut low_p = OptomechanicalSwitchyardParams::default();
    low_p.synthetic_optomechanical_cooperativity_c = 0.3;
    let mut high_p = OptomechanicalSwitchyardParams::default();
    high_p.synthetic_optomechanical_cooperativity_c = 4.5;

    let low_m = OptomechanicalSwitchyardSolver::new(low_p).evaluate_metrics();
    let high_m = OptomechanicalSwitchyardSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.polariton_state_retention_fraction > low_m.polariton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_switchyard_channel_pitch_scaling() {
    let mut low_p = OptomechanicalSwitchyardParams::default();
    low_p.switchyard_channel_pitch_um = 1.0;
    let mut high_p = OptomechanicalSwitchyardParams::default();
    high_p.switchyard_channel_pitch_um = 18.0;

    let low_m = OptomechanicalSwitchyardSolver::new(low_p).evaluate_metrics();
    let high_m = OptomechanicalSwitchyardSolver::new(high_p).evaluate_metrics();

    assert!(high_m.routing_fidelity > low_m.routing_fidelity);
    assert!(high_m.polariton_state_retention_fraction > low_m.polariton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
