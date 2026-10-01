#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Skyrmion-Polariton Quantum Transceiver & Metamaterial Crossbar Switch Engine.

use phonon_models::skyrmion_polariton_transceiver::SkyrmionPolaritonTransceiverParams;
use phonon_solver::skyrmion_polariton_transceiver::SkyrmionPolaritonTransceiverSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SkyrmionPolaritonTransceiverParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.transceiver_coupling_mev, 1.0);
    assert_eq!(underflow.topological_polariton_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.transceiver_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.optical_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_crossbar_nodes_factor, 1.0);
    assert_eq!(underflow.crossbar_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = SkyrmionPolaritonTransceiverParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        35.0,   // above 25.0 um
    );
    assert_eq!(overflow.transceiver_coupling_mev, 35.0);
    assert_eq!(overflow.topological_polariton_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.transceiver_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.optical_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_crossbar_nodes_factor, 8.0);
    assert_eq!(overflow.crossbar_pitch_um, 25.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SkyrmionPolaritonTransceiverParams::default();
    assert_eq!(params.transceiver_coupling_mev, 35.0);
    assert_eq!(params.topological_polariton_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.transceiver_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.optical_probe_power_uw, 25.0);
    assert_eq!(params.synthetic_crossbar_nodes_factor, 4.0);
    assert_eq!(params.crossbar_pitch_um, 24.0);

    let solver = SkyrmionPolaritonTransceiverSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.transceiver_fidelity >= 0.9980,
        "Transceiver fidelity must be >= 0.9980, got {:.6}",
        metrics.transceiver_fidelity
    );
    assert!(
        metrics.skyrmion_polariton_state_retention_fraction >= 0.9970,
        "Skyrmion-polariton state retention fraction must be >= 0.9970, got {:.6}",
        metrics.skyrmion_polariton_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_node_crosstalk_isolation_db >= 55.0,
        "Inter-node crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_node_crosstalk_isolation_db
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
fn test_transceiver_coupling_monotonicity() {
    let p_low = SkyrmionPolaritonTransceiverParams {
        transceiver_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = SkyrmionPolaritonTransceiverParams {
        transceiver_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = SkyrmionPolaritonTransceiverSolver::new(p_low);
    let s_high = SkyrmionPolaritonTransceiverSolver::new(p_high);

    assert!(
        s_high.compute_transceiver_fidelity()
            > s_low.compute_transceiver_fidelity()
    );
    assert!(
        s_high.compute_skyrmion_polariton_state_retention_fraction()
            > s_low.compute_skyrmion_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_polariton_gap_monotonicity() {
    let p_low = SkyrmionPolaritonTransceiverParams {
        topological_polariton_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = SkyrmionPolaritonTransceiverParams {
        topological_polariton_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = SkyrmionPolaritonTransceiverSolver::new(p_low);
    let s_high = SkyrmionPolaritonTransceiverSolver::new(p_high);

    assert!(
        s_high.compute_transceiver_fidelity()
            > s_low.compute_transceiver_fidelity()
    );
    assert!(
        s_high.compute_skyrmion_polariton_state_retention_fraction()
            > s_low.compute_skyrmion_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = SkyrmionPolaritonTransceiverParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = SkyrmionPolaritonTransceiverParams {
        acoustic_drive_frequency_ghz: 10.0,
        ..Default::default()
    };

    let s_low = SkyrmionPolaritonTransceiverSolver::new(p_low);
    let s_high = SkyrmionPolaritonTransceiverSolver::new(p_high);

    assert!(
        s_high.compute_transceiver_fidelity()
            > s_low.compute_transceiver_fidelity()
    );
    assert!(
        s_high.compute_skyrmion_polariton_state_retention_fraction()
            > s_low.compute_skyrmion_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_transceiver_dispatch_speed_monotonicity() {
    let p_low = SkyrmionPolaritonTransceiverParams {
        transceiver_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = SkyrmionPolaritonTransceiverParams {
        transceiver_dispatch_speed_m_per_s: 2500.0,
        ..Default::default()
    };

    let s_low = SkyrmionPolaritonTransceiverSolver::new(p_low);
    let s_high = SkyrmionPolaritonTransceiverSolver::new(p_high);

    assert!(
        s_high.compute_transceiver_fidelity()
            > s_low.compute_transceiver_fidelity()
    );
    assert!(
        s_high.compute_skyrmion_polariton_state_retention_fraction()
            > s_low.compute_skyrmion_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_optical_probe_power_monotonicity() {
    let p_low = SkyrmionPolaritonTransceiverParams {
        optical_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = SkyrmionPolaritonTransceiverParams {
        optical_probe_power_uw: 25.0,
        ..Default::default()
    };

    let s_low = SkyrmionPolaritonTransceiverSolver::new(p_low);
    let s_high = SkyrmionPolaritonTransceiverSolver::new(p_high);

    assert!(
        s_high.compute_transceiver_fidelity()
            > s_low.compute_transceiver_fidelity()
    );
    assert!(
        s_high.compute_skyrmion_polariton_state_retention_fraction()
            > s_low.compute_skyrmion_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_synthetic_crossbar_nodes_monotonicity() {
    let p_low = SkyrmionPolaritonTransceiverParams {
        synthetic_crossbar_nodes_factor: 2.0,
        ..Default::default()
    };
    let p_high = SkyrmionPolaritonTransceiverParams {
        synthetic_crossbar_nodes_factor: 6.0,
        ..Default::default()
    };

    let s_low = SkyrmionPolaritonTransceiverSolver::new(p_low);
    let s_high = SkyrmionPolaritonTransceiverSolver::new(p_high);

    assert!(
        s_high.compute_transceiver_fidelity()
            > s_low.compute_transceiver_fidelity()
    );
    assert!(
        s_high.compute_skyrmion_polariton_state_retention_fraction()
            > s_low.compute_skyrmion_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = SkyrmionPolaritonTransceiverParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = SkyrmionPolaritonTransceiverParams {
        cryogenic_temperature_mk: 40.0,
        ..Default::default()
    };

    let s_cold = SkyrmionPolaritonTransceiverSolver::new(p_cold);
    let s_warm = SkyrmionPolaritonTransceiverSolver::new(p_warm);

    assert!(
        s_cold.compute_transceiver_fidelity()
            > s_warm.compute_transceiver_fidelity()
    );
    assert!(
        s_cold.compute_skyrmion_polariton_state_retention_fraction()
            > s_warm.compute_skyrmion_polariton_state_retention_fraction()
    );
    assert!(
        s_cold.compute_topological_protection_gap_mhz()
            > s_warm.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_cold.compute_inter_node_crosstalk_isolation_db()
            > s_warm.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_cold.compute_topological_mode_dephasing_rate_hz()
            < s_warm.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_crossbar_pitch_scaling() {
    let p_narrow = SkyrmionPolaritonTransceiverParams {
        crossbar_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = SkyrmionPolaritonTransceiverParams {
        crossbar_pitch_um: 22.0,
        ..Default::default()
    };

    let s_narrow = SkyrmionPolaritonTransceiverSolver::new(p_narrow);
    let s_wide = SkyrmionPolaritonTransceiverSolver::new(p_wide);

    assert!(
        s_wide.compute_transceiver_fidelity()
            > s_narrow.compute_transceiver_fidelity()
    );
    assert!(
        s_wide.compute_skyrmion_polariton_state_retention_fraction()
            > s_narrow.compute_skyrmion_polariton_state_retention_fraction()
    );
    assert!(
        s_wide.compute_topological_protection_gap_mhz()
            > s_narrow.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_wide.compute_inter_node_crosstalk_isolation_db()
            > s_narrow.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_narrow.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_extreme_physical_limits_compliance() {
    let min_params = SkyrmionPolaritonTransceiverParams::new(
        1.0, 2.0, 1.0, 200.0, 50.0, 0.5, 1.0, 0.5,
    );
    let solver_min = SkyrmionPolaritonTransceiverSolver::new(min_params);
    let metrics_min = solver_min.evaluate_metrics();

    assert!(metrics_min.transceiver_fidelity >= 0.9980);
    assert!(metrics_min.skyrmion_polariton_state_retention_fraction >= 0.9970);
    assert!(metrics_min.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_min.inter_node_crosstalk_isolation_db >= 55.0);
    assert!(metrics_min.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_min.is_physically_compliant);

    let max_params = SkyrmionPolaritonTransceiverParams::new(
        35.0, 45.0, 12.0, 3000.0, 1.0, 30.0, 8.0, 25.0,
    );
    let solver_max = SkyrmionPolaritonTransceiverSolver::new(max_params);
    let metrics_max = solver_max.evaluate_metrics();

    assert!(metrics_max.transceiver_fidelity >= 0.9980);
    assert!(metrics_max.skyrmion_polariton_state_retention_fraction >= 0.9970);
    assert!(metrics_max.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_max.inter_node_crosstalk_isolation_db >= 55.0);
    assert!(metrics_max.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_max.is_physically_compliant);
}
