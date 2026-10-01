#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Floquet-Chern Parafermion Quantum Repeater & Teleportation Router Engine (Phase 292).

use phonon_models::floquet_parafermion_repeater::FloquetParafermionRepeaterParams;
use phonon_solver::floquet_parafermion_repeater::FloquetParafermionRepeaterSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FloquetParafermionRepeaterParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.repeater_coupling_mev, 1.0);
    assert_eq!(underflow.topological_parafermion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.teleportation_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_repeater_nodes_factor, 1.0);
    assert_eq!(underflow.repeater_junction_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = FloquetParafermionRepeaterParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        35.0,   // above 25.0 um
    );
    assert_eq!(overflow.repeater_coupling_mev, 35.0);
    assert_eq!(overflow.topological_parafermion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.teleportation_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_repeater_nodes_factor, 8.0);
    assert_eq!(overflow.repeater_junction_pitch_um, 25.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FloquetParafermionRepeaterParams::default();
    assert_eq!(params.repeater_coupling_mev, 35.0);
    assert_eq!(params.topological_parafermion_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.teleportation_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 29.8);
    assert_eq!(params.synthetic_repeater_nodes_factor, 4.0);
    assert_eq!(params.repeater_junction_pitch_um, 25.0);

    let solver = FloquetParafermionRepeaterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.repeater_fidelity >= 0.9980,
        "Repeater fidelity must be >= 0.9980, got {:.6}",
        metrics.repeater_fidelity
    );
    assert!(
        metrics.topological_state_retention_fraction >= 0.9970,
        "Topological state retention fraction must be >= 0.9970, got {:.6}",
        metrics.topological_state_retention_fraction
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
fn test_repeater_coupling_monotonicity() {
    let p_low = FloquetParafermionRepeaterParams {
        repeater_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionRepeaterParams {
        repeater_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionRepeaterSolver::new(p_low);
    let s_high = FloquetParafermionRepeaterSolver::new(p_high);

    assert!(
        s_high.compute_repeater_fidelity()
            > s_low.compute_repeater_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
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
fn test_topological_parafermion_gap_monotonicity() {
    let p_low = FloquetParafermionRepeaterParams {
        topological_parafermion_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionRepeaterParams {
        topological_parafermion_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionRepeaterSolver::new(p_low);
    let s_high = FloquetParafermionRepeaterSolver::new(p_high);

    assert!(
        s_high.compute_repeater_fidelity()
            > s_low.compute_repeater_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
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
    let p_low = FloquetParafermionRepeaterParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionRepeaterParams {
        acoustic_drive_frequency_ghz: 11.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionRepeaterSolver::new(p_low);
    let s_high = FloquetParafermionRepeaterSolver::new(p_high);

    assert!(
        s_high.compute_repeater_fidelity()
            > s_low.compute_repeater_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
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
fn test_teleportation_dispatch_speed_monotonicity() {
    let p_low = FloquetParafermionRepeaterParams {
        teleportation_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionRepeaterParams {
        teleportation_dispatch_speed_m_per_s: 2800.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionRepeaterSolver::new(p_low);
    let s_high = FloquetParafermionRepeaterSolver::new(p_high);

    assert!(
        s_high.compute_repeater_fidelity()
            > s_low.compute_repeater_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
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
fn test_microwave_probe_power_monotonicity() {
    let p_low = FloquetParafermionRepeaterParams {
        microwave_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionRepeaterParams {
        microwave_probe_power_uw: 28.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionRepeaterSolver::new(p_low);
    let s_high = FloquetParafermionRepeaterSolver::new(p_high);

    assert!(
        s_high.compute_repeater_fidelity()
            > s_low.compute_repeater_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
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
fn test_synthetic_repeater_nodes_monotonicity() {
    let p_low = FloquetParafermionRepeaterParams {
        synthetic_repeater_nodes_factor: 2.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionRepeaterParams {
        synthetic_repeater_nodes_factor: 7.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionRepeaterSolver::new(p_low);
    let s_high = FloquetParafermionRepeaterSolver::new(p_high);

    assert!(
        s_high.compute_repeater_fidelity()
            > s_low.compute_repeater_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
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
    let p_cold = FloquetParafermionRepeaterParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = FloquetParafermionRepeaterParams {
        cryogenic_temperature_mk: 40.0,
        ..Default::default()
    };

    let s_cold = FloquetParafermionRepeaterSolver::new(p_cold);
    let s_warm = FloquetParafermionRepeaterSolver::new(p_warm);

    assert!(
        s_cold.compute_repeater_fidelity()
            > s_warm.compute_repeater_fidelity()
    );
    assert!(
        s_cold.compute_topological_state_retention_fraction()
            > s_warm.compute_topological_state_retention_fraction()
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
fn test_repeater_junction_pitch_scaling() {
    let p_narrow = FloquetParafermionRepeaterParams {
        repeater_junction_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = FloquetParafermionRepeaterParams {
        repeater_junction_pitch_um: 22.0,
        ..Default::default()
    };

    let s_narrow = FloquetParafermionRepeaterSolver::new(p_narrow);
    let s_wide = FloquetParafermionRepeaterSolver::new(p_wide);

    assert!(
        s_wide.compute_repeater_fidelity()
            > s_narrow.compute_repeater_fidelity()
    );
    assert!(
        s_wide.compute_topological_state_retention_fraction()
            > s_narrow.compute_topological_state_retention_fraction()
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
    let min_params = FloquetParafermionRepeaterParams::new(
        1.0, 2.0, 1.0, 200.0, 50.0, 0.5, 1.0, 0.5,
    );
    let solver_min = FloquetParafermionRepeaterSolver::new(min_params);
    let metrics_min = solver_min.evaluate_metrics();

    assert!(metrics_min.repeater_fidelity >= 0.9980);
    assert!(metrics_min.topological_state_retention_fraction >= 0.9970);
    assert!(metrics_min.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_min.inter_node_crosstalk_isolation_db >= 55.0);
    assert!(metrics_min.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_min.is_physically_compliant);

    let max_params = FloquetParafermionRepeaterParams::new(
        35.0, 45.0, 12.0, 3000.0, 1.0, 30.0, 8.0, 25.0,
    );
    let solver_max = FloquetParafermionRepeaterSolver::new(max_params);
    let metrics_max = solver_max.evaluate_metrics();

    assert!(metrics_max.repeater_fidelity >= 0.9980);
    assert!(metrics_max.topological_state_retention_fraction >= 0.9970);
    assert!(metrics_max.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_max.inter_node_crosstalk_isolation_db >= 55.0);
    assert!(metrics_max.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_max.is_physically_compliant);
}
