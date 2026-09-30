#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological monopole-harmonic entanglement teleporters and compactified
//! quantum transceivers.

use phonon_models::monopole_harmonic_teleporter::MonopoleHarmonicTeleporterParams;
use phonon_solver::monopole_harmonic_teleporter::MonopoleHarmonicTeleporterSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = MonopoleHarmonicTeleporterParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        5.0,   // below 10.0 nm
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.monopole_berry_curvature_strength, 1.0);
    assert_eq!(underflow.topological_superconducting_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_harmonic_frequency_ghz, 1.0);
    assert_eq!(underflow.teleportation_drift_velocity_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_transceiver_power_uw, 0.5);
    assert_eq!(underflow.compactification_radius_nm, 10.0);
    assert_eq!(underflow.channel_separation_distance_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = MonopoleHarmonicTeleporterParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        250.0,  // above 200.0 nm
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.monopole_berry_curvature_strength, 35.0);
    assert_eq!(overflow.topological_superconducting_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_harmonic_frequency_ghz, 12.0);
    assert_eq!(overflow.teleportation_drift_velocity_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_transceiver_power_uw, 30.0);
    assert_eq!(overflow.compactification_radius_nm, 200.0);
    assert_eq!(overflow.channel_separation_distance_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = MonopoleHarmonicTeleporterParams::default();
    assert_eq!(params.monopole_berry_curvature_strength, 16.5);
    assert_eq!(params.topological_superconducting_gap_mev, 22.0);
    assert_eq!(params.acoustic_harmonic_frequency_ghz, 5.7);
    assert_eq!(params.teleportation_drift_velocity_m_per_s, 1420.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_transceiver_power_uw, 5.7);
    assert_eq!(params.compactification_radius_nm, 65.0);
    assert_eq!(params.channel_separation_distance_um, 5.2);

    let solver = MonopoleHarmonicTeleporterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.teleportation_fidelity >= 0.9980,
        "Teleportation fidelity must be >= 0.9980, got {:.6}",
        metrics.teleportation_fidelity
    );
    assert!(
        metrics.anyon_state_retention_fraction >= 0.9970,
        "Anyon state retention fraction must be >= 0.9970, got {:.6}",
        metrics.anyon_state_retention_fraction
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
fn test_berry_curvature_scaling() {
    let mut low_p = MonopoleHarmonicTeleporterParams::default();
    low_p.monopole_berry_curvature_strength = 2.0;

    let mut high_p = MonopoleHarmonicTeleporterParams::default();
    high_p.monopole_berry_curvature_strength = 34.0;

    let low_m = MonopoleHarmonicTeleporterSolver::new(low_p).evaluate_metrics();
    let high_m = MonopoleHarmonicTeleporterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.teleportation_fidelity > low_m.teleportation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_superconducting_gap_scaling() {
    let mut low_p = MonopoleHarmonicTeleporterParams::default();
    low_p.topological_superconducting_gap_mev = 3.0;

    let mut high_p = MonopoleHarmonicTeleporterParams::default();
    high_p.topological_superconducting_gap_mev = 44.0;

    let low_m = MonopoleHarmonicTeleporterSolver::new(low_p).evaluate_metrics();
    let high_m = MonopoleHarmonicTeleporterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.teleportation_fidelity > low_m.teleportation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_harmonic_frequency_scaling() {
    let mut low_p = MonopoleHarmonicTeleporterParams::default();
    low_p.acoustic_harmonic_frequency_ghz = 1.5;

    let mut high_p = MonopoleHarmonicTeleporterParams::default();
    high_p.acoustic_harmonic_frequency_ghz = 11.5;

    let low_m = MonopoleHarmonicTeleporterSolver::new(low_p).evaluate_metrics();
    let high_m = MonopoleHarmonicTeleporterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.teleportation_fidelity > low_m.teleportation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_teleportation_drift_velocity_scaling() {
    let mut low_p = MonopoleHarmonicTeleporterParams::default();
    low_p.teleportation_drift_velocity_m_per_s = 300.0;

    let mut high_p = MonopoleHarmonicTeleporterParams::default();
    high_p.teleportation_drift_velocity_m_per_s = 2900.0;

    let low_m = MonopoleHarmonicTeleporterSolver::new(low_p).evaluate_metrics();
    let high_m = MonopoleHarmonicTeleporterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.teleportation_fidelity > low_m.teleportation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_p = MonopoleHarmonicTeleporterParams::default();
    low_p.cryogenic_temperature_mk = 2.0;

    let mut high_p = MonopoleHarmonicTeleporterParams::default();
    high_p.cryogenic_temperature_mk = 48.0;

    let low_m = MonopoleHarmonicTeleporterSolver::new(low_p).evaluate_metrics();
    let high_m = MonopoleHarmonicTeleporterSolver::new(high_p).evaluate_metrics();

    // Higher temperature degrades performance and increases dephasing
    assert!(low_m.teleportation_fidelity > high_m.teleportation_fidelity);
    assert!(low_m.anyon_state_retention_fraction > high_m.anyon_state_retention_fraction);
    assert!(low_m.topological_protection_gap_mhz > high_m.topological_protection_gap_mhz);
    assert!(low_m.inter_channel_crosstalk_isolation_db > high_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz > low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_transceiver_power_scaling() {
    let mut low_p = MonopoleHarmonicTeleporterParams::default();
    low_p.microwave_transceiver_power_uw = 1.0;

    let mut high_p = MonopoleHarmonicTeleporterParams::default();
    high_p.microwave_transceiver_power_uw = 29.0;

    let low_m = MonopoleHarmonicTeleporterSolver::new(low_p).evaluate_metrics();
    let high_m = MonopoleHarmonicTeleporterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.teleportation_fidelity > low_m.teleportation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_compactification_radius_scaling() {
    let mut low_p = MonopoleHarmonicTeleporterParams::default();
    low_p.compactification_radius_nm = 15.0;

    let mut high_p = MonopoleHarmonicTeleporterParams::default();
    high_p.compactification_radius_nm = 195.0;

    let low_m = MonopoleHarmonicTeleporterSolver::new(low_p).evaluate_metrics();
    let high_m = MonopoleHarmonicTeleporterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.teleportation_fidelity > low_m.teleportation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_channel_separation_distance_scaling() {
    let mut low_p = MonopoleHarmonicTeleporterParams::default();
    low_p.channel_separation_distance_um = 1.0;

    let mut high_p = MonopoleHarmonicTeleporterParams::default();
    high_p.channel_separation_distance_um = 19.0;

    let low_m = MonopoleHarmonicTeleporterSolver::new(low_p).evaluate_metrics();
    let high_m = MonopoleHarmonicTeleporterSolver::new(high_p).evaluate_metrics();

    assert!(high_m.teleportation_fidelity > low_m.teleportation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
