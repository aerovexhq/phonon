#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Real-Time Holographic Telemetry Engine
//! & Immersive Spatial CAD Fabric.

use phonon_models::holographic_telemetry::HolographicTelemetryParams;
use phonon_solver::holographic_telemetry::HolographicTelemetrySolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = HolographicTelemetryParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.holographic_coupling_mev, 1.0);
    assert_eq!(underflow.topological_telemetry_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.spatial_voxel_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_field_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_lightfield_depth_factor, 1.0);
    assert_eq!(underflow.voxel_grid_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = HolographicTelemetryParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.holographic_coupling_mev, 35.0);
    assert_eq!(overflow.topological_telemetry_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.spatial_voxel_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_field_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_lightfield_depth_factor, 8.0);
    assert_eq!(overflow.voxel_grid_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = HolographicTelemetryParams::default();
    assert_eq!(params.holographic_coupling_mev, 18.5);
    assert_eq!(params.topological_telemetry_gap_mev, 24.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 6.8);
    assert_eq!(params.spatial_voxel_dispatch_speed_m_per_s, 1550.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_field_probe_power_uw, 6.8);
    assert_eq!(params.synthetic_lightfield_depth_factor, 4.0);
    assert_eq!(params.voxel_grid_pitch_um, 5.8);

    let solver = HolographicTelemetrySolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.holographic_render_fidelity >= 0.9980,
        "Holographic render fidelity must be >= 0.9980, got {:.6}",
        metrics.holographic_render_fidelity
    );
    assert!(
        metrics.spatial_state_retention_fraction >= 0.9970,
        "Spatial state retention fraction must be >= 0.9970, got {:.6}",
        metrics.spatial_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_view_crosstalk_isolation_db >= 55.0,
        "Inter-view crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_view_crosstalk_isolation_db
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
fn test_holographic_coupling_scaling() {
    let mut low_p = HolographicTelemetryParams::default();
    low_p.holographic_coupling_mev = 2.0;
    let mut high_p = HolographicTelemetryParams::default();
    high_p.holographic_coupling_mev = 30.0;

    let low_m = HolographicTelemetrySolver::new(low_p).evaluate_metrics();
    let high_m = HolographicTelemetrySolver::new(high_p).evaluate_metrics();

    assert!(high_m.holographic_render_fidelity > low_m.holographic_render_fidelity);
    assert!(high_m.spatial_state_retention_fraction > low_m.spatial_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_view_crosstalk_isolation_db > low_m.inter_view_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_telemetry_gap_scaling() {
    let mut low_p = HolographicTelemetryParams::default();
    low_p.topological_telemetry_gap_mev = 3.0;
    let mut high_p = HolographicTelemetryParams::default();
    high_p.topological_telemetry_gap_mev = 40.0;

    let low_m = HolographicTelemetrySolver::new(low_p).evaluate_metrics();
    let high_m = HolographicTelemetrySolver::new(high_p).evaluate_metrics();

    assert!(high_m.holographic_render_fidelity > low_m.holographic_render_fidelity);
    assert!(high_m.spatial_state_retention_fraction > low_m.spatial_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_view_crosstalk_isolation_db > low_m.inter_view_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = HolographicTelemetryParams::default();
    low_p.acoustic_drive_frequency_ghz = 2.0;
    let mut high_p = HolographicTelemetryParams::default();
    high_p.acoustic_drive_frequency_ghz = 10.0;

    let low_m = HolographicTelemetrySolver::new(low_p).evaluate_metrics();
    let high_m = HolographicTelemetrySolver::new(high_p).evaluate_metrics();

    assert!(high_m.holographic_render_fidelity > low_m.holographic_render_fidelity);
    assert!(high_m.spatial_state_retention_fraction > low_m.spatial_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_view_crosstalk_isolation_db > low_m.inter_view_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_spatial_voxel_dispatch_speed_scaling() {
    let mut low_p = HolographicTelemetryParams::default();
    low_p.spatial_voxel_dispatch_speed_m_per_s = 300.0;
    let mut high_p = HolographicTelemetryParams::default();
    high_p.spatial_voxel_dispatch_speed_m_per_s = 2800.0;

    let low_m = HolographicTelemetrySolver::new(low_p).evaluate_metrics();
    let high_m = HolographicTelemetrySolver::new(high_p).evaluate_metrics();

    assert!(high_m.holographic_render_fidelity > low_m.holographic_render_fidelity);
    assert!(high_m.spatial_state_retention_fraction > low_m.spatial_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_view_crosstalk_isolation_db > low_m.inter_view_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold_p = HolographicTelemetryParams::default();
    cold_p.cryogenic_temperature_mk = 2.0;
    let mut warm_p = HolographicTelemetryParams::default();
    warm_p.cryogenic_temperature_mk = 45.0;

    let cold_m = HolographicTelemetrySolver::new(cold_p).evaluate_metrics();
    let warm_m = HolographicTelemetrySolver::new(warm_p).evaluate_metrics();

    assert!(cold_m.holographic_render_fidelity > warm_m.holographic_render_fidelity);
    assert!(cold_m.spatial_state_retention_fraction > warm_m.spatial_state_retention_fraction);
    assert!(cold_m.topological_protection_gap_mhz > warm_m.topological_protection_gap_mhz);
    assert!(cold_m.inter_view_crosstalk_isolation_db > warm_m.inter_view_crosstalk_isolation_db);
    assert!(cold_m.topological_mode_dephasing_rate_hz < warm_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_field_probe_power_scaling() {
    let mut low_p = HolographicTelemetryParams::default();
    low_p.microwave_field_probe_power_uw = 1.0;
    let mut high_p = HolographicTelemetryParams::default();
    high_p.microwave_field_probe_power_uw = 25.0;

    let low_m = HolographicTelemetrySolver::new(low_p).evaluate_metrics();
    let high_m = HolographicTelemetrySolver::new(high_p).evaluate_metrics();

    assert!(high_m.holographic_render_fidelity > low_m.holographic_render_fidelity);
    assert!(high_m.spatial_state_retention_fraction > low_m.spatial_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_view_crosstalk_isolation_db > low_m.inter_view_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synthetic_lightfield_depth_factor_scaling() {
    let mut low_p = HolographicTelemetryParams::default();
    low_p.synthetic_lightfield_depth_factor = 2.0;
    let mut high_p = HolographicTelemetryParams::default();
    high_p.synthetic_lightfield_depth_factor = 7.0;

    let low_m = HolographicTelemetrySolver::new(low_p).evaluate_metrics();
    let high_m = HolographicTelemetrySolver::new(high_p).evaluate_metrics();

    assert!(high_m.holographic_render_fidelity > low_m.holographic_render_fidelity);
    assert!(high_m.spatial_state_retention_fraction > low_m.spatial_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_view_crosstalk_isolation_db > low_m.inter_view_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_voxel_grid_pitch_scaling() {
    let mut low_p = HolographicTelemetryParams::default();
    low_p.voxel_grid_pitch_um = 1.0;
    let mut high_p = HolographicTelemetryParams::default();
    high_p.voxel_grid_pitch_um = 18.0;

    let low_m = HolographicTelemetrySolver::new(low_p).evaluate_metrics();
    let high_m = HolographicTelemetrySolver::new(high_p).evaluate_metrics();

    assert!(high_m.holographic_render_fidelity > low_m.holographic_render_fidelity);
    assert!(high_m.spatial_state_retention_fraction > low_m.spatial_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_view_crosstalk_isolation_db > low_m.inter_view_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
