#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Superconducting Quatrit
//! State Synthesizer & Multi-Valued Quantum Logic Engine.

use phonon_models::superconducting_quatrit::SuperconductingQuatritParams;
use phonon_solver::superconducting_quatrit::SuperconductingQuatritSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SuperconductingQuatritParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.quatrit_coupling_mev, 1.0);
    assert_eq!(underflow.topological_quatrit_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.quatrit_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_quatrit_levels_factor, 1.0);
    assert_eq!(underflow.quatrit_cell_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = SuperconductingQuatritParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.quatrit_coupling_mev, 35.0);
    assert_eq!(overflow.topological_quatrit_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.quatrit_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_quatrit_levels_factor, 8.0);
    assert_eq!(overflow.quatrit_cell_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SuperconductingQuatritParams::default();
    assert_eq!(params.quatrit_coupling_mev, 35.0);
    assert_eq!(params.topological_quatrit_gap_mev, 41.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.quatrit_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 15.2);
    assert_eq!(params.synthetic_quatrit_levels_factor, 4.0);
    assert_eq!(params.quatrit_cell_pitch_um, 14.2);

    let solver = SuperconductingQuatritSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.quatrit_synthesis_fidelity >= 0.9980,
        "Quatrit synthesis fidelity must be >= 0.9980, got {:.6}",
        metrics.quatrit_synthesis_fidelity
    );
    assert!(
        metrics.quatrit_state_retention_fraction >= 0.9970,
        "Quatrit state retention fraction must be >= 0.9970, got {:.6}",
        metrics.quatrit_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_level_crosstalk_isolation_db >= 55.0,
        "Inter-level crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_level_crosstalk_isolation_db
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
fn test_quatrit_coupling_scaling() {
    let mut p_low = SuperconductingQuatritParams::default();
    p_low.quatrit_coupling_mev = 2.0;
    let mut p_high = SuperconductingQuatritParams::default();
    p_high.quatrit_coupling_mev = 34.0;

    let solver_low = SuperconductingQuatritSolver::new(p_low);
    let solver_high = SuperconductingQuatritSolver::new(p_high);

    assert!(
        solver_high.compute_quatrit_synthesis_fidelity()
            > solver_low.compute_quatrit_synthesis_fidelity(),
        "Higher quatrit coupling energy should enhance quatrit synthesis fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher quatrit coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher quatrit coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_quatrit_gap_scaling() {
    let mut p_low = SuperconductingQuatritParams::default();
    p_low.topological_quatrit_gap_mev = 3.0;
    let mut p_high = SuperconductingQuatritParams::default();
    p_high.topological_quatrit_gap_mev = 44.0;

    let solver_low = SuperconductingQuatritSolver::new(p_low);
    let solver_high = SuperconductingQuatritSolver::new(p_high);

    assert!(
        solver_high.compute_quatrit_state_retention_fraction()
            > solver_low.compute_quatrit_state_retention_fraction(),
        "Wider topological quatrit gap should improve state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological quatrit gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = SuperconductingQuatritParams::default();
    p_low.acoustic_drive_frequency_ghz = 1.5;
    let mut p_high = SuperconductingQuatritParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.8;

    let solver_low = SuperconductingQuatritSolver::new(p_low);
    let solver_high = SuperconductingQuatritSolver::new(p_high);

    assert!(
        solver_high.compute_quatrit_synthesis_fidelity()
            > solver_low.compute_quatrit_synthesis_fidelity(),
        "Higher acoustic drive frequency should enhance quatrit synthesis fidelity"
    );
    assert!(
        solver_high.compute_inter_level_crosstalk_isolation_db()
            > solver_low.compute_inter_level_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_quatrit_dispatch_speed_scaling() {
    let mut p_low = SuperconductingQuatritParams::default();
    p_low.quatrit_dispatch_speed_m_per_s = 300.0;
    let mut p_high = SuperconductingQuatritParams::default();
    p_high.quatrit_dispatch_speed_m_per_s = 2950.0;

    let solver_low = SuperconductingQuatritSolver::new(p_low);
    let solver_high = SuperconductingQuatritSolver::new(p_high);

    assert!(
        solver_high.compute_quatrit_state_retention_fraction()
            > solver_low.compute_quatrit_state_retention_fraction(),
        "Higher quatrit dispatch speed should improve state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher quatrit dispatch speed should increase protection gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = SuperconductingQuatritParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = SuperconductingQuatritParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = SuperconductingQuatritSolver::new(p_cold);
    let solver_warm = SuperconductingQuatritSolver::new(p_warm);

    assert!(
        solver_cold.compute_quatrit_synthesis_fidelity()
            > solver_warm.compute_quatrit_synthesis_fidelity(),
        "Lower cryogenic temperature should improve quatrit synthesis fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = SuperconductingQuatritParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = SuperconductingQuatritParams::default();
    p_high.microwave_probe_power_uw = 29.0;

    let solver_low = SuperconductingQuatritSolver::new(p_low);
    let solver_high = SuperconductingQuatritSolver::new(p_high);

    assert!(
        solver_high.compute_quatrit_synthesis_fidelity()
            > solver_low.compute_quatrit_synthesis_fidelity(),
        "Higher probe power should improve quatrit synthesis fidelity"
    );
    assert!(
        solver_high.compute_inter_level_crosstalk_isolation_db()
            > solver_low.compute_inter_level_crosstalk_isolation_db(),
        "Higher probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_quatrit_levels_scaling() {
    let mut p_low = SuperconductingQuatritParams::default();
    p_low.synthetic_quatrit_levels_factor = 1.5;
    let mut p_high = SuperconductingQuatritParams::default();
    p_high.synthetic_quatrit_levels_factor = 7.5;

    let solver_low = SuperconductingQuatritSolver::new(p_low);
    let solver_high = SuperconductingQuatritSolver::new(p_high);

    assert!(
        solver_high.compute_inter_level_crosstalk_isolation_db()
            > solver_low.compute_inter_level_crosstalk_isolation_db(),
        "Higher synthetic quatrit levels factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher synthetic quatrit levels factor should suppress dephasing rate"
    );
}

#[test]
fn test_quatrit_cell_pitch_scaling() {
    let mut p_small = SuperconductingQuatritParams::default();
    p_small.quatrit_cell_pitch_um = 1.0;
    let mut p_large = SuperconductingQuatritParams::default();
    p_large.quatrit_cell_pitch_um = 19.0;

    let solver_small = SuperconductingQuatritSolver::new(p_small);
    let solver_large = SuperconductingQuatritSolver::new(p_large);

    assert!(
        solver_large.compute_inter_level_crosstalk_isolation_db()
            > solver_small.compute_inter_level_crosstalk_isolation_db(),
        "Larger quatrit cell pitch should enhance inter-level crosstalk isolation"
    );
    assert!(
        solver_large.compute_topological_protection_gap_mhz()
            > solver_small.compute_topological_protection_gap_mhz(),
        "Larger quatrit cell pitch should increase protection gap"
    );
}
