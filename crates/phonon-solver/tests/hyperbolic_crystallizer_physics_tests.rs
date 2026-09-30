#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological hyperbolic lattice anyon crystallizers and fractal boundary engines.

use phonon_models::hyperbolic_crystallizer::HyperbolicCrystallizerParams;
use phonon_solver::hyperbolic_crystallizer::HyperbolicCrystallizerSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = HyperbolicCrystallizerParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.05,  // below 0.1 um
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.hyperbolic_coupling_energy_mev, 1.0);
    assert_eq!(underflow.topological_fractal_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.crystallization_drift_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_pinning_power_uw, 0.5);
    assert_eq!(underflow.synthetic_curvature_radius_um, 0.1);
    assert_eq!(underflow.hyperbolic_tessellation_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = HyperbolicCrystallizerParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        8.0,    // above 5.0 um
        35.0,   // above 20.0 um
    );
    assert_eq!(overflow.hyperbolic_coupling_energy_mev, 35.0);
    assert_eq!(overflow.topological_fractal_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.crystallization_drift_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_pinning_power_uw, 30.0);
    assert_eq!(overflow.synthetic_curvature_radius_um, 5.0);
    assert_eq!(overflow.hyperbolic_tessellation_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = HyperbolicCrystallizerParams::default();
    assert_eq!(params.hyperbolic_coupling_energy_mev, 16.5);
    assert_eq!(params.topological_fractal_gap_mev, 22.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.8);
    assert_eq!(params.crystallization_drift_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_pinning_power_uw, 5.8);
    assert_eq!(params.synthetic_curvature_radius_um, 1.6);
    assert_eq!(params.hyperbolic_tessellation_pitch_um, 4.8);

    let solver = HyperbolicCrystallizerSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.crystallization_fidelity >= 0.9980,
        "Crystallization fidelity must be >= 0.9980, got {:.6}",
        metrics.crystallization_fidelity
    );
    assert!(
        metrics.crystallized_state_retention_fraction >= 0.9970,
        "Crystallized state retention fraction must be >= 0.9970, got {:.6}",
        metrics.crystallized_state_retention_fraction
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
fn test_hyperbolic_coupling_energy_scaling() {
    let mut low_p = HyperbolicCrystallizerParams::default();
    low_p.hyperbolic_coupling_energy_mev = 2.0;

    let mut high_p = HyperbolicCrystallizerParams::default();
    high_p.hyperbolic_coupling_energy_mev = 34.0;

    let low_m = HyperbolicCrystallizerSolver::new(low_p).evaluate_metrics();
    let high_m = HyperbolicCrystallizerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.crystallization_fidelity > low_m.crystallization_fidelity);
    assert!(high_m.crystallized_state_retention_fraction > low_m.crystallized_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_fractal_gap_scaling() {
    let mut low_p = HyperbolicCrystallizerParams::default();
    low_p.topological_fractal_gap_mev = 3.0;

    let mut high_p = HyperbolicCrystallizerParams::default();
    high_p.topological_fractal_gap_mev = 43.0;

    let low_m = HyperbolicCrystallizerSolver::new(low_p).evaluate_metrics();
    let high_m = HyperbolicCrystallizerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.crystallization_fidelity > low_m.crystallization_fidelity);
    assert!(high_m.crystallized_state_retention_fraction > low_m.crystallized_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = HyperbolicCrystallizerParams::default();
    low_p.acoustic_drive_frequency_ghz = 1.5;

    let mut high_p = HyperbolicCrystallizerParams::default();
    high_p.acoustic_drive_frequency_ghz = 11.5;

    let low_m = HyperbolicCrystallizerSolver::new(low_p).evaluate_metrics();
    let high_m = HyperbolicCrystallizerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.crystallization_fidelity > low_m.crystallization_fidelity);
    assert!(high_m.crystallized_state_retention_fraction > low_m.crystallized_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_crystallization_drift_speed_scaling() {
    let mut low_p = HyperbolicCrystallizerParams::default();
    low_p.crystallization_drift_speed_m_per_s = 300.0;

    let mut high_p = HyperbolicCrystallizerParams::default();
    high_p.crystallization_drift_speed_m_per_s = 2900.0;

    let low_m = HyperbolicCrystallizerSolver::new(low_p).evaluate_metrics();
    let high_m = HyperbolicCrystallizerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.crystallization_fidelity > low_m.crystallization_fidelity);
    assert!(high_m.crystallized_state_retention_fraction > low_m.crystallized_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_p = HyperbolicCrystallizerParams::default();
    low_p.cryogenic_temperature_mk = 2.0;

    let mut high_p = HyperbolicCrystallizerParams::default();
    high_p.cryogenic_temperature_mk = 48.0;

    let low_m = HyperbolicCrystallizerSolver::new(low_p).evaluate_metrics();
    let high_m = HyperbolicCrystallizerSolver::new(high_p).evaluate_metrics();

    // Higher temperature degrades performance and increases dephasing
    assert!(low_m.crystallization_fidelity > high_m.crystallization_fidelity);
    assert!(low_m.crystallized_state_retention_fraction > high_m.crystallized_state_retention_fraction);
    assert!(low_m.topological_protection_gap_mhz > high_m.topological_protection_gap_mhz);
    assert!(low_m.inter_channel_crosstalk_isolation_db > high_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz > low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_pinning_power_scaling() {
    let mut low_p = HyperbolicCrystallizerParams::default();
    low_p.microwave_pinning_power_uw = 1.0;

    let mut high_p = HyperbolicCrystallizerParams::default();
    high_p.microwave_pinning_power_uw = 29.0;

    let low_m = HyperbolicCrystallizerSolver::new(low_p).evaluate_metrics();
    let high_m = HyperbolicCrystallizerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.crystallization_fidelity > low_m.crystallization_fidelity);
    assert!(high_m.crystallized_state_retention_fraction > low_m.crystallized_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synthetic_curvature_radius_scaling() {
    let mut low_p = HyperbolicCrystallizerParams::default();
    low_p.synthetic_curvature_radius_um = 0.3;

    let mut high_p = HyperbolicCrystallizerParams::default();
    high_p.synthetic_curvature_radius_um = 4.8;

    let low_m = HyperbolicCrystallizerSolver::new(low_p).evaluate_metrics();
    let high_m = HyperbolicCrystallizerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.crystallization_fidelity > low_m.crystallization_fidelity);
    assert!(high_m.crystallized_state_retention_fraction > low_m.crystallized_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_hyperbolic_tessellation_pitch_scaling() {
    let mut low_p = HyperbolicCrystallizerParams::default();
    low_p.hyperbolic_tessellation_pitch_um = 1.0;

    let mut high_p = HyperbolicCrystallizerParams::default();
    high_p.hyperbolic_tessellation_pitch_um = 19.0;

    let low_m = HyperbolicCrystallizerSolver::new(low_p).evaluate_metrics();
    let high_m = HyperbolicCrystallizerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.crystallization_fidelity > low_m.crystallization_fidelity);
    assert!(high_m.crystallized_state_retention_fraction > low_m.crystallized_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
