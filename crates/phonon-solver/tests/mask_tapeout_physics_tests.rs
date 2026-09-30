#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Automated GDSII/OASIS Photolithography Mask
//! & Cryogenic Foundry Tapeout Synthesis Engine.

use phonon_models::mask_tapeout::MaskTapeoutParams;
use phonon_solver::mask_tapeout::MaskTapeoutSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = MaskTapeoutParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.mask_fracture_coupling_mev, 1.0);
    assert_eq!(underflow.topological_tapeout_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.polygon_raster_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_tapeout_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_opc_layer_factor, 1.0);
    assert_eq!(underflow.mask_feature_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = MaskTapeoutParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.mask_fracture_coupling_mev, 35.0);
    assert_eq!(overflow.topological_tapeout_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.polygon_raster_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_tapeout_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_opc_layer_factor, 8.0);
    assert_eq!(overflow.mask_feature_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = MaskTapeoutParams::default();
    assert_eq!(params.mask_fracture_coupling_mev, 19.5);
    assert_eq!(params.topological_tapeout_gap_mev, 25.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 7.2);
    assert_eq!(params.polygon_raster_dispatch_speed_m_per_s, 1650.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_tapeout_probe_power_uw, 7.2);
    assert_eq!(params.synthetic_opc_layer_factor, 4.0);
    assert_eq!(params.mask_feature_pitch_um, 6.2);

    let solver = MaskTapeoutSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.mask_synthesis_fidelity >= 0.9980,
        "Mask synthesis fidelity must be >= 0.9980, got {:.6}",
        metrics.mask_synthesis_fidelity
    );
    assert!(
        metrics.layout_state_retention_fraction >= 0.9970,
        "Layout state retention fraction must be >= 0.9970, got {:.6}",
        metrics.layout_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_layer_drc_isolation_db >= 55.0,
        "Inter-layer DRC isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_layer_drc_isolation_db
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
fn test_mask_fracture_coupling_scaling() {
    let mut low_p = MaskTapeoutParams::default();
    low_p.mask_fracture_coupling_mev = 2.0;
    let mut high_p = MaskTapeoutParams::default();
    high_p.mask_fracture_coupling_mev = 30.0;

    let low_m = MaskTapeoutSolver::new(low_p).evaluate_metrics();
    let high_m = MaskTapeoutSolver::new(high_p).evaluate_metrics();

    assert!(high_m.mask_synthesis_fidelity > low_m.mask_synthesis_fidelity);
    assert!(high_m.layout_state_retention_fraction > low_m.layout_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_drc_isolation_db > low_m.inter_layer_drc_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_tapeout_gap_scaling() {
    let mut low_p = MaskTapeoutParams::default();
    low_p.topological_tapeout_gap_mev = 3.0;
    let mut high_p = MaskTapeoutParams::default();
    high_p.topological_tapeout_gap_mev = 40.0;

    let low_m = MaskTapeoutSolver::new(low_p).evaluate_metrics();
    let high_m = MaskTapeoutSolver::new(high_p).evaluate_metrics();

    assert!(high_m.mask_synthesis_fidelity > low_m.mask_synthesis_fidelity);
    assert!(high_m.layout_state_retention_fraction > low_m.layout_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_drc_isolation_db > low_m.inter_layer_drc_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = MaskTapeoutParams::default();
    low_p.acoustic_drive_frequency_ghz = 2.0;
    let mut high_p = MaskTapeoutParams::default();
    high_p.acoustic_drive_frequency_ghz = 10.0;

    let low_m = MaskTapeoutSolver::new(low_p).evaluate_metrics();
    let high_m = MaskTapeoutSolver::new(high_p).evaluate_metrics();

    assert!(high_m.mask_synthesis_fidelity > low_m.mask_synthesis_fidelity);
    assert!(high_m.layout_state_retention_fraction > low_m.layout_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_drc_isolation_db > low_m.inter_layer_drc_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_polygon_raster_dispatch_speed_scaling() {
    let mut low_p = MaskTapeoutParams::default();
    low_p.polygon_raster_dispatch_speed_m_per_s = 300.0;
    let mut high_p = MaskTapeoutParams::default();
    high_p.polygon_raster_dispatch_speed_m_per_s = 2800.0;

    let low_m = MaskTapeoutSolver::new(low_p).evaluate_metrics();
    let high_m = MaskTapeoutSolver::new(high_p).evaluate_metrics();

    assert!(high_m.mask_synthesis_fidelity > low_m.mask_synthesis_fidelity);
    assert!(high_m.layout_state_retention_fraction > low_m.layout_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_drc_isolation_db > low_m.inter_layer_drc_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold_p = MaskTapeoutParams::default();
    cold_p.cryogenic_temperature_mk = 2.0;
    let mut warm_p = MaskTapeoutParams::default();
    warm_p.cryogenic_temperature_mk = 45.0;

    let cold_m = MaskTapeoutSolver::new(cold_p).evaluate_metrics();
    let warm_m = MaskTapeoutSolver::new(warm_p).evaluate_metrics();

    assert!(cold_m.mask_synthesis_fidelity > warm_m.mask_synthesis_fidelity);
    assert!(cold_m.layout_state_retention_fraction > warm_m.layout_state_retention_fraction);
    assert!(cold_m.topological_protection_gap_mhz > warm_m.topological_protection_gap_mhz);
    assert!(cold_m.inter_layer_drc_isolation_db > warm_m.inter_layer_drc_isolation_db);
    assert!(cold_m.topological_mode_dephasing_rate_hz < warm_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_tapeout_probe_power_scaling() {
    let mut low_p = MaskTapeoutParams::default();
    low_p.microwave_tapeout_probe_power_uw = 1.0;
    let mut high_p = MaskTapeoutParams::default();
    high_p.microwave_tapeout_probe_power_uw = 25.0;

    let low_m = MaskTapeoutSolver::new(low_p).evaluate_metrics();
    let high_m = MaskTapeoutSolver::new(high_p).evaluate_metrics();

    assert!(high_m.mask_synthesis_fidelity > low_m.mask_synthesis_fidelity);
    assert!(high_m.layout_state_retention_fraction > low_m.layout_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_drc_isolation_db > low_m.inter_layer_drc_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synthetic_opc_layer_factor_scaling() {
    let mut low_p = MaskTapeoutParams::default();
    low_p.synthetic_opc_layer_factor = 2.0;
    let mut high_p = MaskTapeoutParams::default();
    high_p.synthetic_opc_layer_factor = 7.0;

    let low_m = MaskTapeoutSolver::new(low_p).evaluate_metrics();
    let high_m = MaskTapeoutSolver::new(high_p).evaluate_metrics();

    assert!(high_m.mask_synthesis_fidelity > low_m.mask_synthesis_fidelity);
    assert!(high_m.layout_state_retention_fraction > low_m.layout_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_drc_isolation_db > low_m.inter_layer_drc_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_mask_feature_pitch_scaling() {
    let mut low_p = MaskTapeoutParams::default();
    low_p.mask_feature_pitch_um = 1.0;
    let mut high_p = MaskTapeoutParams::default();
    high_p.mask_feature_pitch_um = 18.0;

    let low_m = MaskTapeoutSolver::new(low_p).evaluate_metrics();
    let high_m = MaskTapeoutSolver::new(high_p).evaluate_metrics();

    assert!(high_m.mask_synthesis_fidelity > low_m.mask_synthesis_fidelity);
    assert!(high_m.layout_state_retention_fraction > low_m.layout_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_drc_isolation_db > low_m.inter_layer_drc_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
