#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Superconducting Quoctit State Synthesizer & Multi-Valued Topological Logic Engine.

use phonon_models::superconducting_quoctit::SuperconductingQuoctitParams;
use phonon_solver::superconducting_quoctit::SuperconductingQuoctitSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SuperconductingQuoctitParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.quoctit_coupling_mev, 1.0);
    assert_eq!(underflow.topological_quoctit_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.quoctit_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_quoctit_levels_factor, 1.0);
    assert_eq!(underflow.quoctit_cell_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = SuperconductingQuoctitParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        35.0,   // above 25.0 um
    );
    assert_eq!(overflow.quoctit_coupling_mev, 35.0);
    assert_eq!(overflow.topological_quoctit_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.quoctit_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_quoctit_levels_factor, 8.0);
    assert_eq!(overflow.quoctit_cell_pitch_um, 25.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SuperconductingQuoctitParams::default();
    assert_eq!(params.quoctit_coupling_mev, 35.0);
    assert_eq!(params.topological_quoctit_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.quoctit_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 22.5);
    assert_eq!(params.synthetic_quoctit_levels_factor, 8.0);
    assert_eq!(params.quoctit_cell_pitch_um, 21.5);

    let solver = SuperconductingQuoctitSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.quoctit_synthesis_fidelity >= 0.9980,
        "Quoctit synthesis fidelity must be >= 0.9980, got {:.6}",
        metrics.quoctit_synthesis_fidelity
    );
    assert!(
        metrics.quoctit_state_retention_fraction >= 0.9970,
        "Quoctit state retention fraction must be >= 0.9970, got {:.6}",
        metrics.quoctit_state_retention_fraction
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
fn test_quoctit_coupling_scaling() {
    let p_low = SuperconductingQuoctitParams {
        quoctit_coupling_mev: 2.0,
        ..Default::default()
    };
    let p_high = SuperconductingQuoctitParams {
        quoctit_coupling_mev: 34.0,
        ..Default::default()
    };

    let solver_low = SuperconductingQuoctitSolver::new(p_low);
    let solver_high = SuperconductingQuoctitSolver::new(p_high);

    assert!(
        solver_high.compute_quoctit_synthesis_fidelity()
            > solver_low.compute_quoctit_synthesis_fidelity(),
        "Higher quoctit coupling energy should enhance synthesis fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher quoctit coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher quoctit coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_quoctit_gap_scaling() {
    let p_low = SuperconductingQuoctitParams {
        topological_quoctit_gap_mev: 3.0,
        ..Default::default()
    };
    let p_high = SuperconductingQuoctitParams {
        topological_quoctit_gap_mev: 44.5,
        ..Default::default()
    };

    let solver_low = SuperconductingQuoctitSolver::new(p_low);
    let solver_high = SuperconductingQuoctitSolver::new(p_high);

    assert!(
        solver_high.compute_quoctit_state_retention_fraction()
            > solver_low.compute_quoctit_state_retention_fraction(),
        "Wider topological quoctit gap should improve state retention"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological quoctit gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let p_low = SuperconductingQuoctitParams {
        acoustic_drive_frequency_ghz: 1.5,
        ..Default::default()
    };
    let p_high = SuperconductingQuoctitParams {
        acoustic_drive_frequency_ghz: 11.8,
        ..Default::default()
    };

    let solver_low = SuperconductingQuoctitSolver::new(p_low);
    let solver_high = SuperconductingQuoctitSolver::new(p_high);

    assert!(
        solver_high.compute_quoctit_synthesis_fidelity()
            > solver_low.compute_quoctit_synthesis_fidelity(),
        "Higher acoustic drive frequency should enhance synthesis fidelity"
    );
    assert!(
        solver_high.compute_inter_level_crosstalk_isolation_db()
            > solver_low.compute_inter_level_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_quoctit_dispatch_speed_scaling() {
    let p_low = SuperconductingQuoctitParams {
        quoctit_dispatch_speed_m_per_s: 300.0,
        ..Default::default()
    };
    let p_high = SuperconductingQuoctitParams {
        quoctit_dispatch_speed_m_per_s: 2950.0,
        ..Default::default()
    };

    let solver_low = SuperconductingQuoctitSolver::new(p_low);
    let solver_high = SuperconductingQuoctitSolver::new(p_high);

    assert!(
        solver_high.compute_quoctit_state_retention_fraction()
            > solver_low.compute_quoctit_state_retention_fraction(),
        "Higher quoctit dispatch speed should improve state retention"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher quoctit dispatch speed should increase protection gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let p_cold = SuperconductingQuoctitParams {
        cryogenic_temperature_mk: 2.0,
        ..Default::default()
    };
    let p_warm = SuperconductingQuoctitParams {
        cryogenic_temperature_mk: 48.0,
        ..Default::default()
    };

    let solver_cold = SuperconductingQuoctitSolver::new(p_cold);
    let solver_warm = SuperconductingQuoctitSolver::new(p_warm);

    assert!(
        solver_cold.compute_quoctit_synthesis_fidelity()
            > solver_warm.compute_quoctit_synthesis_fidelity(),
        "Lower cryogenic temperature should improve quoctit synthesis fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let p_low = SuperconductingQuoctitParams {
        microwave_probe_power_uw: 1.0,
        ..Default::default()
    };
    let p_high = SuperconductingQuoctitParams {
        microwave_probe_power_uw: 29.0,
        ..Default::default()
    };

    let solver_low = SuperconductingQuoctitSolver::new(p_low);
    let solver_high = SuperconductingQuoctitSolver::new(p_high);

    assert!(
        solver_high.compute_quoctit_synthesis_fidelity()
            > solver_low.compute_quoctit_synthesis_fidelity(),
        "Higher probe power should improve quoctit synthesis fidelity"
    );
    assert!(
        solver_high.compute_inter_level_crosstalk_isolation_db()
            > solver_low.compute_inter_level_crosstalk_isolation_db(),
        "Higher probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_quoctit_levels_scaling() {
    let p_low = SuperconductingQuoctitParams {
        synthetic_quoctit_levels_factor: 1.5,
        ..Default::default()
    };
    let p_high = SuperconductingQuoctitParams {
        synthetic_quoctit_levels_factor: 7.5,
        ..Default::default()
    };

    let solver_low = SuperconductingQuoctitSolver::new(p_low);
    let solver_high = SuperconductingQuoctitSolver::new(p_high);

    assert!(
        solver_high.compute_inter_level_crosstalk_isolation_db()
            > solver_low.compute_inter_level_crosstalk_isolation_db(),
        "Higher synthetic quoctit levels factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher synthetic quoctit levels factor should suppress dephasing rate"
    );
}

#[test]
fn test_quoctit_cell_pitch_scaling() {
    let p_small = SuperconductingQuoctitParams {
        quoctit_cell_pitch_um: 1.0,
        ..Default::default()
    };
    let p_large = SuperconductingQuoctitParams {
        quoctit_cell_pitch_um: 24.0,
        ..Default::default()
    };

    let solver_small = SuperconductingQuoctitSolver::new(p_small);
    let solver_large = SuperconductingQuoctitSolver::new(p_large);

    assert!(
        solver_large.compute_inter_level_crosstalk_isolation_db()
            > solver_small.compute_inter_level_crosstalk_isolation_db(),
        "Larger quoctit cell pitch should enhance crosstalk isolation"
    );
    assert!(
        solver_large.compute_topological_protection_gap_mhz()
            > solver_small.compute_topological_protection_gap_mhz(),
        "Larger quoctit cell pitch should increase protection gap"
    );
}

#[test]
fn test_extreme_physical_limits_compliance() {
    let min_params = SuperconductingQuoctitParams::new(
        1.0, 2.0, 1.0, 200.0, 50.0, 0.5, 1.0, 0.5,
    );
    let solver_min = SuperconductingQuoctitSolver::new(min_params);
    let metrics_min = solver_min.evaluate_metrics();

    assert!(metrics_min.quoctit_synthesis_fidelity >= 0.9980);
    assert!(metrics_min.quoctit_state_retention_fraction >= 0.9970);
    assert!(metrics_min.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_min.inter_level_crosstalk_isolation_db >= 55.0);
    assert!(metrics_min.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_min.is_physically_compliant);

    let max_params = SuperconductingQuoctitParams::new(
        35.0, 45.0, 12.0, 3000.0, 1.0, 30.0, 8.0, 25.0,
    );
    let solver_max = SuperconductingQuoctitSolver::new(max_params);
    let metrics_max = solver_max.evaluate_metrics();

    assert!(metrics_max.quoctit_synthesis_fidelity >= 0.9980);
    assert!(metrics_max.quoctit_state_retention_fraction >= 0.9970);
    assert!(metrics_max.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_max.inter_level_crosstalk_isolation_db >= 55.0);
    assert!(metrics_max.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_max.is_physically_compliant);
}
