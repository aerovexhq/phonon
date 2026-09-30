#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological
//! Chiral Phonon-Magnon Polariton Frequency Comb Synthesizer & Quantum Soliton Transceiver Engine.

use phonon_models::phonon_magnon_polariton_comb::PhononMagnonPolaritonCombParams;
use phonon_solver::phonon_magnon_polariton_comb::PhononMagnonPolaritonCombSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = PhononMagnonPolaritonCombParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.comb_coupling_mev, 1.0);
    assert_eq!(underflow.topological_comb_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.soliton_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_comb_lines_factor, 1.0);
    assert_eq!(underflow.resonator_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = PhononMagnonPolaritonCombParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.comb_coupling_mev, 35.0);
    assert_eq!(overflow.topological_comb_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.soliton_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_comb_lines_factor, 8.0);
    assert_eq!(overflow.resonator_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = PhononMagnonPolaritonCombParams::default();
    assert_eq!(params.comb_coupling_mev, 35.0);
    assert_eq!(params.topological_comb_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.soliton_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 19.8);
    assert_eq!(params.synthetic_comb_lines_factor, 4.0);
    assert_eq!(params.resonator_pitch_um, 18.8);

    let solver = PhononMagnonPolaritonCombSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.frequency_comb_fidelity >= 0.9980,
        "Frequency comb fidelity must be >= 0.9980, got {:.6}",
        metrics.frequency_comb_fidelity
    );
    assert!(
        metrics.soliton_state_retention_fraction >= 0.9970,
        "Soliton state retention fraction must be >= 0.9970, got {:.6}",
        metrics.soliton_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_comb_line_crosstalk_isolation_db >= 55.0,
        "Inter-comb-line crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_comb_line_crosstalk_isolation_db
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
fn test_comb_coupling_monotonicity() {
    let p_low = PhononMagnonPolaritonCombParams {
        comb_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = PhononMagnonPolaritonCombParams {
        comb_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = PhononMagnonPolaritonCombSolver::new(p_low);
    let s_high = PhononMagnonPolaritonCombSolver::new(p_high);

    assert!(
        s_high.compute_frequency_comb_fidelity() > s_low.compute_frequency_comb_fidelity()
    );
    assert!(
        s_high.compute_soliton_state_retention_fraction()
            > s_low.compute_soliton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_comb_line_crosstalk_isolation_db()
            > s_low.compute_inter_comb_line_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_comb_gap_monotonicity() {
    let p_low = PhononMagnonPolaritonCombParams {
        topological_comb_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = PhononMagnonPolaritonCombParams {
        topological_comb_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = PhononMagnonPolaritonCombSolver::new(p_low);
    let s_high = PhononMagnonPolaritonCombSolver::new(p_high);

    assert!(
        s_high.compute_frequency_comb_fidelity() > s_low.compute_frequency_comb_fidelity()
    );
    assert!(
        s_high.compute_soliton_state_retention_fraction()
            > s_low.compute_soliton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_comb_line_crosstalk_isolation_db()
            > s_low.compute_inter_comb_line_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = PhononMagnonPolaritonCombParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = PhononMagnonPolaritonCombParams {
        acoustic_drive_frequency_ghz: 11.0,
        ..Default::default()
    };

    let s_low = PhononMagnonPolaritonCombSolver::new(p_low);
    let s_high = PhononMagnonPolaritonCombSolver::new(p_high);

    assert!(
        s_high.compute_frequency_comb_fidelity() > s_low.compute_frequency_comb_fidelity()
    );
    assert!(
        s_high.compute_soliton_state_retention_fraction()
            > s_low.compute_soliton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_comb_line_crosstalk_isolation_db()
            > s_low.compute_inter_comb_line_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_soliton_dispatch_speed_monotonicity() {
    let p_low = PhononMagnonPolaritonCombParams {
        soliton_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = PhononMagnonPolaritonCombParams {
        soliton_dispatch_speed_m_per_s: 2500.0,
        ..Default::default()
    };

    let s_low = PhononMagnonPolaritonCombSolver::new(p_low);
    let s_high = PhononMagnonPolaritonCombSolver::new(p_high);

    assert!(
        s_high.compute_frequency_comb_fidelity() > s_low.compute_frequency_comb_fidelity()
    );
    assert!(
        s_high.compute_soliton_state_retention_fraction()
            > s_low.compute_soliton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_comb_line_crosstalk_isolation_db()
            > s_low.compute_inter_comb_line_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = PhononMagnonPolaritonCombParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = PhononMagnonPolaritonCombParams {
        cryogenic_temperature_mk: 45.0,
        ..Default::default()
    };

    let s_cold = PhononMagnonPolaritonCombSolver::new(p_cold);
    let s_warm = PhononMagnonPolaritonCombSolver::new(p_warm);

    // Warm dilution temperature degrades metrics but increases dephasing
    assert!(
        s_cold.compute_frequency_comb_fidelity() > s_warm.compute_frequency_comb_fidelity()
    );
    assert!(
        s_cold.compute_soliton_state_retention_fraction()
            > s_warm.compute_soliton_state_retention_fraction()
    );
    assert!(
        s_cold.compute_topological_protection_gap_mhz()
            > s_warm.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_cold.compute_inter_comb_line_crosstalk_isolation_db()
            > s_warm.compute_inter_comb_line_crosstalk_isolation_db()
    );
    assert!(
        s_cold.compute_topological_mode_dephasing_rate_hz()
            < s_warm.compute_topological_mode_dephasing_rate_hz()
    );

    // Even at warm limits, parameters still maintain physical compliance
    assert!(s_warm.evaluate_metrics().is_physically_compliant);
}

#[test]
fn test_microwave_probe_power_scaling() {
    let p_low = PhononMagnonPolaritonCombParams {
        microwave_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = PhononMagnonPolaritonCombParams {
        microwave_probe_power_uw: 28.0,
        ..Default::default()
    };

    let s_low = PhononMagnonPolaritonCombSolver::new(p_low);
    let s_high = PhononMagnonPolaritonCombSolver::new(p_high);

    assert!(
        s_high.compute_frequency_comb_fidelity() > s_low.compute_frequency_comb_fidelity()
    );
    assert!(
        s_high.compute_soliton_state_retention_fraction()
            > s_low.compute_soliton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_comb_line_crosstalk_isolation_db()
            > s_low.compute_inter_comb_line_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_synthetic_comb_lines_scaling() {
    let p_few = PhononMagnonPolaritonCombParams {
        synthetic_comb_lines_factor: 2.0,
        ..Default::default()
    };
    let p_many = PhononMagnonPolaritonCombParams {
        synthetic_comb_lines_factor: 7.0,
        ..Default::default()
    };

    let s_few = PhononMagnonPolaritonCombSolver::new(p_few);
    let s_many = PhononMagnonPolaritonCombSolver::new(p_many);

    assert!(
        s_many.compute_frequency_comb_fidelity() > s_few.compute_frequency_comb_fidelity()
    );
    assert!(
        s_many.compute_soliton_state_retention_fraction()
            > s_few.compute_soliton_state_retention_fraction()
    );
    assert!(
        s_many.compute_topological_protection_gap_mhz()
            > s_few.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_many.compute_inter_comb_line_crosstalk_isolation_db()
            > s_few.compute_inter_comb_line_crosstalk_isolation_db()
    );
    assert!(
        s_many.compute_topological_mode_dephasing_rate_hz()
            < s_few.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_resonator_pitch_scaling() {
    let p_tight = PhononMagnonPolaritonCombParams {
        resonator_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = PhononMagnonPolaritonCombParams {
        resonator_pitch_um: 18.0,
        ..Default::default()
    };

    let s_tight = PhononMagnonPolaritonCombSolver::new(p_tight);
    let s_wide = PhononMagnonPolaritonCombSolver::new(p_wide);

    assert!(
        s_wide.compute_frequency_comb_fidelity() > s_tight.compute_frequency_comb_fidelity()
    );
    assert!(
        s_wide.compute_soliton_state_retention_fraction()
            > s_tight.compute_soliton_state_retention_fraction()
    );
    assert!(
        s_wide.compute_topological_protection_gap_mhz()
            > s_tight.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_wide.compute_inter_comb_line_crosstalk_isolation_db()
            > s_tight.compute_inter_comb_line_crosstalk_isolation_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_tight.compute_topological_mode_dephasing_rate_hz()
    );
}
