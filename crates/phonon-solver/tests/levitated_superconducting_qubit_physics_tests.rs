#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Levitated Topological
//! Superconducting Qubit Resonator & Quantum Metrology Engine.

use phonon_models::levitated_superconducting_qubit::LevitatedSuperconductingQubitParams;
use phonon_solver::levitated_superconducting_qubit::LevitatedSuperconductingQubitSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = LevitatedSuperconductingQubitParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.levitation_coupling_mev, 1.0);
    assert_eq!(underflow.topological_qubit_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.metrology_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_resonator_nodes_factor, 1.0);
    assert_eq!(underflow.qubit_resonator_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = LevitatedSuperconductingQubitParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.levitation_coupling_mev, 35.0);
    assert_eq!(overflow.topological_qubit_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.metrology_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_resonator_nodes_factor, 8.0);
    assert_eq!(overflow.qubit_resonator_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = LevitatedSuperconductingQubitParams::default();
    assert_eq!(params.levitation_coupling_mev, 35.0);
    assert_eq!(params.topological_qubit_gap_mev, 44.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.metrology_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 16.8);
    assert_eq!(params.synthetic_resonator_nodes_factor, 4.0);
    assert_eq!(params.qubit_resonator_pitch_um, 15.8);

    let solver = LevitatedSuperconductingQubitSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.qubit_resonance_fidelity >= 0.9980,
        "Qubit resonance fidelity must be >= 0.9980, got {:.6}",
        metrics.qubit_resonance_fidelity
    );
    assert!(
        metrics.superconducting_state_retention_fraction >= 0.9970,
        "Superconducting state retention fraction must be >= 0.9970, got {:.6}",
        metrics.superconducting_state_retention_fraction
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
fn test_levitation_coupling_scaling() {
    let p_low = LevitatedSuperconductingQubitParams {
        levitation_coupling_mev: 2.0,
        ..Default::default()
    };
    let p_high = LevitatedSuperconductingQubitParams {
        levitation_coupling_mev: 34.0,
        ..Default::default()
    };

    let solver_low = LevitatedSuperconductingQubitSolver::new(p_low);
    let solver_high = LevitatedSuperconductingQubitSolver::new(p_high);

    assert!(
        solver_high.compute_qubit_resonance_fidelity()
            > solver_low.compute_qubit_resonance_fidelity(),
        "Higher levitation coupling energy should enhance qubit resonance fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher levitation coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher levitation coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_qubit_gap_scaling() {
    let p_low = LevitatedSuperconductingQubitParams {
        topological_qubit_gap_mev: 3.0,
        ..Default::default()
    };
    let p_high = LevitatedSuperconductingQubitParams {
        topological_qubit_gap_mev: 44.5,
        ..Default::default()
    };

    let solver_low = LevitatedSuperconductingQubitSolver::new(p_low);
    let solver_high = LevitatedSuperconductingQubitSolver::new(p_high);

    assert!(
        solver_high.compute_superconducting_state_retention_fraction()
            > solver_low.compute_superconducting_state_retention_fraction(),
        "Wider topological qubit gap should improve state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological qubit gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let p_low = LevitatedSuperconductingQubitParams {
        acoustic_drive_frequency_ghz: 1.5,
        ..Default::default()
    };
    let p_high = LevitatedSuperconductingQubitParams {
        acoustic_drive_frequency_ghz: 11.8,
        ..Default::default()
    };

    let solver_low = LevitatedSuperconductingQubitSolver::new(p_low);
    let solver_high = LevitatedSuperconductingQubitSolver::new(p_high);

    assert!(
        solver_high.compute_qubit_resonance_fidelity()
            > solver_low.compute_qubit_resonance_fidelity(),
        "Higher acoustic drive frequency should enhance resonance fidelity"
    );
    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_metrology_dispatch_speed_scaling() {
    let p_low = LevitatedSuperconductingQubitParams {
        metrology_dispatch_speed_m_per_s: 300.0,
        ..Default::default()
    };
    let p_high = LevitatedSuperconductingQubitParams {
        metrology_dispatch_speed_m_per_s: 2950.0,
        ..Default::default()
    };

    let solver_low = LevitatedSuperconductingQubitSolver::new(p_low);
    let solver_high = LevitatedSuperconductingQubitSolver::new(p_high);

    assert!(
        solver_high.compute_superconducting_state_retention_fraction()
            > solver_low.compute_superconducting_state_retention_fraction(),
        "Higher dispatch speed should improve state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher dispatch speed should increase protection gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let p_cold = LevitatedSuperconductingQubitParams {
        cryogenic_temperature_mk: 2.0,
        ..Default::default()
    };
    let p_warm = LevitatedSuperconductingQubitParams {
        cryogenic_temperature_mk: 48.0,
        ..Default::default()
    };

    let solver_cold = LevitatedSuperconductingQubitSolver::new(p_cold);
    let solver_warm = LevitatedSuperconductingQubitSolver::new(p_warm);

    assert!(
        solver_cold.compute_qubit_resonance_fidelity()
            > solver_warm.compute_qubit_resonance_fidelity(),
        "Lower cryogenic temperature should improve resonance fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let p_low = LevitatedSuperconductingQubitParams {
        microwave_probe_power_uw: 1.0,
        ..Default::default()
    };
    let p_high = LevitatedSuperconductingQubitParams {
        microwave_probe_power_uw: 29.0,
        ..Default::default()
    };

    let solver_low = LevitatedSuperconductingQubitSolver::new(p_low);
    let solver_high = LevitatedSuperconductingQubitSolver::new(p_high);

    assert!(
        solver_high.compute_qubit_resonance_fidelity()
            > solver_low.compute_qubit_resonance_fidelity(),
        "Higher probe power should improve resonance fidelity"
    );
    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_resonator_nodes_scaling() {
    let p_low = LevitatedSuperconductingQubitParams {
        synthetic_resonator_nodes_factor: 1.5,
        ..Default::default()
    };
    let p_high = LevitatedSuperconductingQubitParams {
        synthetic_resonator_nodes_factor: 7.5,
        ..Default::default()
    };

    let solver_low = LevitatedSuperconductingQubitSolver::new(p_low);
    let solver_high = LevitatedSuperconductingQubitSolver::new(p_high);

    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher synthetic nodes factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher synthetic nodes factor should suppress dephasing rate"
    );
}

#[test]
fn test_qubit_resonator_pitch_scaling() {
    let p_small = LevitatedSuperconductingQubitParams {
        qubit_resonator_pitch_um: 1.0,
        ..Default::default()
    };
    let p_large = LevitatedSuperconductingQubitParams {
        qubit_resonator_pitch_um: 19.0,
        ..Default::default()
    };

    let solver_small = LevitatedSuperconductingQubitSolver::new(p_small);
    let solver_large = LevitatedSuperconductingQubitSolver::new(p_large);

    assert!(
        solver_large.compute_inter_node_crosstalk_isolation_db()
            > solver_small.compute_inter_node_crosstalk_isolation_db(),
        "Larger qubit resonator pitch should enhance inter-node crosstalk isolation"
    );
    assert!(
        solver_large.compute_topological_protection_gap_mhz()
            > solver_small.compute_topological_protection_gap_mhz(),
        "Larger qubit resonator pitch should increase protection gap"
    );
}
