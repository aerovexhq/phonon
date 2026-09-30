#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Non-Abelian Topological Quantum State
//! Teleportation Network Engine.

use phonon_models::teleportation_network::TeleportationNetworkParams;
use phonon_solver::teleportation_network::TeleportationNetworkSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = TeleportationNetworkParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.teleportation_coupling_mev, 1.0);
    assert_eq!(underflow.topological_teleportation_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.teleportation_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_network_nodes_factor, 1.0);
    assert_eq!(underflow.network_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = TeleportationNetworkParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.teleportation_coupling_mev, 35.0);
    assert_eq!(overflow.topological_teleportation_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.teleportation_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_network_nodes_factor, 8.0);
    assert_eq!(overflow.network_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TeleportationNetworkParams::default();
    assert_eq!(params.teleportation_coupling_mev, 28.0);
    assert_eq!(params.topological_teleportation_gap_mev, 34.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 11.5);
    assert_eq!(params.teleportation_dispatch_speed_m_per_s, 2500.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 11.5);
    assert_eq!(params.synthetic_network_nodes_factor, 4.0);
    assert_eq!(params.network_pitch_um, 10.5);

    let solver = TeleportationNetworkSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.teleportation_fidelity >= 0.9980,
        "Teleportation fidelity must be >= 0.9980, got {:.6}",
        metrics.teleportation_fidelity
    );
    assert!(
        metrics.network_state_retention_fraction >= 0.9970,
        "Network state retention fraction must be >= 0.9970, got {:.6}",
        metrics.network_state_retention_fraction
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
fn test_teleportation_coupling_scaling() {
    let mut p_low = TeleportationNetworkParams::default();
    p_low.teleportation_coupling_mev = 2.0;
    let mut p_high = TeleportationNetworkParams::default();
    p_high.teleportation_coupling_mev = 34.0;

    let solver_low = TeleportationNetworkSolver::new(p_low);
    let solver_high = TeleportationNetworkSolver::new(p_high);

    assert!(
        solver_high.compute_teleportation_fidelity()
            > solver_low.compute_teleportation_fidelity(),
        "Higher teleportation coupling energy should enhance teleportation fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher teleportation coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher teleportation coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_teleportation_gap_scaling() {
    let mut p_low = TeleportationNetworkParams::default();
    p_low.topological_teleportation_gap_mev = 3.0;
    let mut p_high = TeleportationNetworkParams::default();
    p_high.topological_teleportation_gap_mev = 44.0;

    let solver_low = TeleportationNetworkSolver::new(p_low);
    let solver_high = TeleportationNetworkSolver::new(p_high);

    assert!(
        solver_high.compute_network_state_retention_fraction()
            > solver_low.compute_network_state_retention_fraction(),
        "Wider topological teleportation gap should improve network state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological teleportation gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = TeleportationNetworkParams::default();
    p_low.acoustic_drive_frequency_ghz = 1.5;
    let mut p_high = TeleportationNetworkParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.5;

    let solver_low = TeleportationNetworkSolver::new(p_low);
    let solver_high = TeleportationNetworkSolver::new(p_high);

    assert!(
        solver_high.compute_teleportation_fidelity()
            > solver_low.compute_teleportation_fidelity(),
        "Higher acoustic drive frequency should enhance teleportation fidelity"
    );
    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_teleportation_dispatch_speed_scaling() {
    let mut p_low = TeleportationNetworkParams::default();
    p_low.teleportation_dispatch_speed_m_per_s = 300.0;
    let mut p_high = TeleportationNetworkParams::default();
    p_high.teleportation_dispatch_speed_m_per_s = 2900.0;

    let solver_low = TeleportationNetworkSolver::new(p_low);
    let solver_high = TeleportationNetworkSolver::new(p_high);

    assert!(
        solver_high.compute_network_state_retention_fraction()
            > solver_low.compute_network_state_retention_fraction(),
        "Higher teleportation dispatch speed should improve retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher teleportation dispatch speed should increase protection gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = TeleportationNetworkParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = TeleportationNetworkParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = TeleportationNetworkSolver::new(p_cold);
    let solver_warm = TeleportationNetworkSolver::new(p_warm);

    assert!(
        solver_cold.compute_teleportation_fidelity()
            > solver_warm.compute_teleportation_fidelity(),
        "Lower cryogenic temperature should improve teleportation fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = TeleportationNetworkParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = TeleportationNetworkParams::default();
    p_high.microwave_probe_power_uw = 29.0;

    let solver_low = TeleportationNetworkSolver::new(p_low);
    let solver_high = TeleportationNetworkSolver::new(p_high);

    assert!(
        solver_high.compute_teleportation_fidelity()
            > solver_low.compute_teleportation_fidelity(),
        "Higher probe power should improve teleportation fidelity"
    );
    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_network_nodes_scaling() {
    let mut p_low = TeleportationNetworkParams::default();
    p_low.synthetic_network_nodes_factor = 1.5;
    let mut p_high = TeleportationNetworkParams::default();
    p_high.synthetic_network_nodes_factor = 7.5;

    let solver_low = TeleportationNetworkSolver::new(p_low);
    let solver_high = TeleportationNetworkSolver::new(p_high);

    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher synthetic network nodes factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher synthetic network nodes factor should suppress dephasing rate"
    );
}

#[test]
fn test_network_pitch_scaling() {
    let mut p_small = TeleportationNetworkParams::default();
    p_small.network_pitch_um = 1.0;
    let mut p_large = TeleportationNetworkParams::default();
    p_large.network_pitch_um = 19.0;

    let solver_small = TeleportationNetworkSolver::new(p_small);
    let solver_large = TeleportationNetworkSolver::new(p_large);

    assert!(
        solver_large.compute_inter_node_crosstalk_isolation_db()
            > solver_small.compute_inter_node_crosstalk_isolation_db(),
        "Larger network pitch should enhance inter-node crosstalk isolation"
    );
    assert!(
        solver_large.compute_topological_protection_gap_mhz()
            > solver_small.compute_topological_protection_gap_mhz(),
        "Larger network pitch should increase protection gap"
    );
}
