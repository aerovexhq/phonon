#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Silicon-to-Cloud Deployment
//! Gateway & Production Digital Twin Cloud Fabric.

use phonon_models::cloud_deployment::CloudDeploymentParams;
use phonon_solver::cloud_deployment::CloudDeploymentSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = CloudDeploymentParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.deployment_coupling_mev, 1.0);
    assert_eq!(underflow.topological_deployment_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.stream_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_cloud_nodes_factor, 1.0);
    assert_eq!(underflow.gateway_interconnect_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = CloudDeploymentParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.deployment_coupling_mev, 35.0);
    assert_eq!(overflow.topological_deployment_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.stream_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_cloud_nodes_factor, 8.0);
    assert_eq!(overflow.gateway_interconnect_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = CloudDeploymentParams::default();
    assert_eq!(params.deployment_coupling_mev, 21.0);
    assert_eq!(params.topological_deployment_gap_mev, 27.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 8.0);
    assert_eq!(params.stream_dispatch_speed_m_per_s, 1800.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 8.0);
    assert_eq!(params.synthetic_cloud_nodes_factor, 4.0);
    assert_eq!(params.gateway_interconnect_pitch_um, 7.0);

    let solver = CloudDeploymentSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.deployment_fidelity >= 0.9980,
        "Deployment fidelity must be >= 0.9980, got {:.6}",
        metrics.deployment_fidelity
    );
    assert!(
        metrics.cloud_state_retention_fraction >= 0.9970,
        "Cloud state retention fraction must be >= 0.9970, got {:.6}",
        metrics.cloud_state_retention_fraction
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
fn test_coupling_scaling() {
    let mut p_low = CloudDeploymentParams::default();
    p_low.deployment_coupling_mev = 2.0;
    let mut p_high = CloudDeploymentParams::default();
    p_high.deployment_coupling_mev = 34.0;

    let solver_low = CloudDeploymentSolver::new(p_low);
    let solver_high = CloudDeploymentSolver::new(p_high);

    assert!(
        solver_high.compute_deployment_fidelity() > solver_low.compute_deployment_fidelity(),
        "Higher coupling energy should enhance deployment fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_gap_scaling() {
    let mut p_low = CloudDeploymentParams::default();
    p_low.topological_deployment_gap_mev = 3.0;
    let mut p_high = CloudDeploymentParams::default();
    p_high.topological_deployment_gap_mev = 44.0;

    let solver_low = CloudDeploymentSolver::new(p_low);
    let solver_high = CloudDeploymentSolver::new(p_high);

    assert!(
        solver_high.compute_cloud_state_retention_fraction()
            > solver_low.compute_cloud_state_retention_fraction(),
        "Wider topological deployment gap should improve retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological gap should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Wider topological gap should reduce dephasing rate"
    );
}

#[test]
fn test_acoustic_frequency_scaling() {
    let mut p_low = CloudDeploymentParams::default();
    p_low.acoustic_drive_frequency_ghz = 2.0;
    let mut p_high = CloudDeploymentParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.0;

    let solver_low = CloudDeploymentSolver::new(p_low);
    let solver_high = CloudDeploymentSolver::new(p_high);

    assert!(
        solver_high.compute_deployment_fidelity() > solver_low.compute_deployment_fidelity(),
        "Higher acoustic drive frequency should enhance deployment fidelity"
    );
    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should increase isolation"
    );
}

#[test]
fn test_stream_dispatch_speed_scaling() {
    let mut p_low = CloudDeploymentParams::default();
    p_low.stream_dispatch_speed_m_per_s = 300.0;
    let mut p_high = CloudDeploymentParams::default();
    p_high.stream_dispatch_speed_m_per_s = 2900.0;

    let solver_low = CloudDeploymentSolver::new(p_low);
    let solver_high = CloudDeploymentSolver::new(p_high);

    assert!(
        solver_high.compute_deployment_fidelity() > solver_low.compute_deployment_fidelity(),
        "Faster stream telemetry dispatch speed should increase fidelity"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Faster stream telemetry dispatch speed should reduce dephasing rate"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = CloudDeploymentParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = CloudDeploymentParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = CloudDeploymentSolver::new(p_cold);
    let solver_warm = CloudDeploymentSolver::new(p_warm);

    assert!(
        solver_cold.compute_deployment_fidelity() > solver_warm.compute_deployment_fidelity(),
        "Lower temperature should preserve higher fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower temperature should exhibit lower thermal dephasing"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = CloudDeploymentParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = CloudDeploymentParams::default();
    p_high.microwave_probe_power_uw = 28.0;

    let solver_low = CloudDeploymentSolver::new(p_low);
    let solver_high = CloudDeploymentSolver::new(p_high);

    assert!(
        solver_high.compute_cloud_state_retention_fraction()
            > solver_low.compute_cloud_state_retention_fraction(),
        "Optimized microwave probe power should enhance cloud state retention"
    );
    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher probe power within linear regime should improve SNR and isolation"
    );
}

#[test]
fn test_synthetic_cloud_nodes_scaling() {
    let mut p_low = CloudDeploymentParams::default();
    p_low.synthetic_cloud_nodes_factor = 1.5;
    let mut p_high = CloudDeploymentParams::default();
    p_high.synthetic_cloud_nodes_factor = 7.5;

    let solver_low = CloudDeploymentSolver::new(p_low);
    let solver_high = CloudDeploymentSolver::new(p_high);

    assert!(
        solver_high.compute_deployment_fidelity() > solver_low.compute_deployment_fidelity(),
        "Higher synthetic cloud nodes factor should improve deployment fidelity"
    );
    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher synthetic cloud nodes factor should boost crosstalk isolation"
    );
}

#[test]
fn test_gateway_interconnect_pitch_scaling() {
    let mut p_tight = CloudDeploymentParams::default();
    p_tight.gateway_interconnect_pitch_um = 1.0;
    let mut p_wide = CloudDeploymentParams::default();
    p_wide.gateway_interconnect_pitch_um = 18.0;

    let solver_tight = CloudDeploymentSolver::new(p_tight);
    let solver_wide = CloudDeploymentSolver::new(p_wide);

    assert!(
        solver_wide.compute_inter_node_crosstalk_isolation_db()
            > solver_tight.compute_inter_node_crosstalk_isolation_db(),
        "Wider gateway interconnect pitch must significantly increase crosstalk isolation"
    );
    assert!(
        solver_wide.compute_topological_protection_gap_mhz()
            > solver_tight.compute_topological_protection_gap_mhz(),
        "Wider interconnect pitch suppresses parasitic fringing and widens protection gap"
    );
}
