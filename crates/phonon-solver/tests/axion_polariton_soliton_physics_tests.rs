#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological axion-polariton quantum simulators and non-linear anyonic soliton engines.

use phonon_models::axion_polariton_soliton::AxionPolaritonParams;
use phonon_solver::axion_polariton_soliton::AxionPolaritonSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AxionPolaritonParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.05,  // below 0.1 pm^2/V^2
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.axion_coupling_energy_mev, 1.0);
    assert_eq!(underflow.topological_polariton_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.soliton_propagation_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.optical_parametric_pump_power_uw, 0.5);
    assert_eq!(underflow.non_linear_kerr_coefficient_pm2_per_v2, 0.1);
    assert_eq!(underflow.polariton_waveguide_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = AxionPolaritonParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        8.0,    // above 5.0 pm^2/V^2
        35.0,   // above 20.0 um
    );
    assert_eq!(overflow.axion_coupling_energy_mev, 35.0);
    assert_eq!(overflow.topological_polariton_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.soliton_propagation_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.optical_parametric_pump_power_uw, 30.0);
    assert_eq!(overflow.non_linear_kerr_coefficient_pm2_per_v2, 5.0);
    assert_eq!(overflow.polariton_waveguide_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AxionPolaritonParams::default();
    assert_eq!(params.axion_coupling_energy_mev, 16.5);
    assert_eq!(params.topological_polariton_gap_mev, 22.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.8);
    assert_eq!(params.soliton_propagation_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.optical_parametric_pump_power_uw, 5.8);
    assert_eq!(params.non_linear_kerr_coefficient_pm2_per_v2, 1.6);
    assert_eq!(params.polariton_waveguide_pitch_um, 4.8);

    let solver = AxionPolaritonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.simulation_fidelity >= 0.9980,
        "Simulation fidelity must be >= 0.9980, got {:.6}",
        metrics.simulation_fidelity
    );
    assert!(
        metrics.soliton_state_retention_fraction >= 0.9970,
        "Soliton state retention fraction must be >= 0.9970, got {:.6}",
        metrics.soliton_state_retention_fraction
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
fn test_axion_coupling_energy_scaling() {
    let mut low_p = AxionPolaritonParams::default();
    low_p.axion_coupling_energy_mev = 2.0;

    let mut high_p = AxionPolaritonParams::default();
    high_p.axion_coupling_energy_mev = 34.0;

    let low_m = AxionPolaritonSolver::new(low_p).evaluate_metrics();
    let high_m = AxionPolaritonSolver::new(high_p).evaluate_metrics();

    assert!(high_m.simulation_fidelity > low_m.simulation_fidelity);
    assert!(high_m.soliton_state_retention_fraction > low_m.soliton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_polariton_gap_scaling() {
    let mut low_p = AxionPolaritonParams::default();
    low_p.topological_polariton_gap_mev = 3.0;

    let mut high_p = AxionPolaritonParams::default();
    high_p.topological_polariton_gap_mev = 43.0;

    let low_m = AxionPolaritonSolver::new(low_p).evaluate_metrics();
    let high_m = AxionPolaritonSolver::new(high_p).evaluate_metrics();

    assert!(high_m.simulation_fidelity > low_m.simulation_fidelity);
    assert!(high_m.soliton_state_retention_fraction > low_m.soliton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = AxionPolaritonParams::default();
    low_p.acoustic_drive_frequency_ghz = 1.5;

    let mut high_p = AxionPolaritonParams::default();
    high_p.acoustic_drive_frequency_ghz = 11.5;

    let low_m = AxionPolaritonSolver::new(low_p).evaluate_metrics();
    let high_m = AxionPolaritonSolver::new(high_p).evaluate_metrics();

    assert!(high_m.simulation_fidelity > low_m.simulation_fidelity);
    assert!(high_m.soliton_state_retention_fraction > low_m.soliton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_soliton_propagation_speed_scaling() {
    let mut low_p = AxionPolaritonParams::default();
    low_p.soliton_propagation_speed_m_per_s = 300.0;

    let mut high_p = AxionPolaritonParams::default();
    high_p.soliton_propagation_speed_m_per_s = 2900.0;

    let low_m = AxionPolaritonSolver::new(low_p).evaluate_metrics();
    let high_m = AxionPolaritonSolver::new(high_p).evaluate_metrics();

    assert!(high_m.simulation_fidelity > low_m.simulation_fidelity);
    assert!(high_m.soliton_state_retention_fraction > low_m.soliton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_p = AxionPolaritonParams::default();
    low_p.cryogenic_temperature_mk = 2.0;

    let mut high_p = AxionPolaritonParams::default();
    high_p.cryogenic_temperature_mk = 48.0;

    let low_m = AxionPolaritonSolver::new(low_p).evaluate_metrics();
    let high_m = AxionPolaritonSolver::new(high_p).evaluate_metrics();

    // Higher temperature degrades performance and increases dephasing
    assert!(low_m.simulation_fidelity > high_m.simulation_fidelity);
    assert!(low_m.soliton_state_retention_fraction > high_m.soliton_state_retention_fraction);
    assert!(low_m.topological_protection_gap_mhz > high_m.topological_protection_gap_mhz);
    assert!(low_m.inter_channel_crosstalk_isolation_db > high_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz > low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_optical_parametric_pump_power_scaling() {
    let mut low_p = AxionPolaritonParams::default();
    low_p.optical_parametric_pump_power_uw = 1.0;

    let mut high_p = AxionPolaritonParams::default();
    high_p.optical_parametric_pump_power_uw = 29.0;

    let low_m = AxionPolaritonSolver::new(low_p).evaluate_metrics();
    let high_m = AxionPolaritonSolver::new(high_p).evaluate_metrics();

    assert!(high_m.simulation_fidelity > low_m.simulation_fidelity);
    assert!(high_m.soliton_state_retention_fraction > low_m.soliton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_non_linear_kerr_coefficient_scaling() {
    let mut low_p = AxionPolaritonParams::default();
    low_p.non_linear_kerr_coefficient_pm2_per_v2 = 0.3;

    let mut high_p = AxionPolaritonParams::default();
    high_p.non_linear_kerr_coefficient_pm2_per_v2 = 4.8;

    let low_m = AxionPolaritonSolver::new(low_p).evaluate_metrics();
    let high_m = AxionPolaritonSolver::new(high_p).evaluate_metrics();

    assert!(high_m.simulation_fidelity > low_m.simulation_fidelity);
    assert!(high_m.soliton_state_retention_fraction > low_m.soliton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_polariton_waveguide_pitch_scaling() {
    let mut low_p = AxionPolaritonParams::default();
    low_p.polariton_waveguide_pitch_um = 1.0;

    let mut high_p = AxionPolaritonParams::default();
    high_p.polariton_waveguide_pitch_um = 19.0;

    let low_m = AxionPolaritonSolver::new(low_p).evaluate_metrics();
    let high_m = AxionPolaritonSolver::new(high_p).evaluate_metrics();

    assert!(high_m.simulation_fidelity > low_m.simulation_fidelity);
    assert!(high_m.soliton_state_retention_fraction > low_m.soliton_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
