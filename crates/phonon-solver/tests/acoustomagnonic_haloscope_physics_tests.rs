#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Cavity Acoustomagnonic
//! Dark-Matter Axion Haloscope and Metrology Engine.

use phonon_models::acoustomagnonic_haloscope::AcoustomagnonicHaloscopeParams;
use phonon_solver::acoustomagnonic_haloscope::AcoustomagnonicHaloscopeSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AcoustomagnonicHaloscopeParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.haloscope_coupling_mev, 1.0);
    assert_eq!(underflow.topological_axion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.haloscope_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_cavity_modes_factor, 1.0);
    assert_eq!(underflow.cavity_resonator_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = AcoustomagnonicHaloscopeParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.haloscope_coupling_mev, 35.0);
    assert_eq!(overflow.topological_axion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.haloscope_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_cavity_modes_factor, 8.0);
    assert_eq!(overflow.cavity_resonator_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AcoustomagnonicHaloscopeParams::default();
    assert_eq!(params.haloscope_coupling_mev, 29.5);
    assert_eq!(params.topological_axion_gap_mev, 35.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.haloscope_dispatch_speed_m_per_s, 2650.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 12.2);
    assert_eq!(params.synthetic_cavity_modes_factor, 4.0);
    assert_eq!(params.cavity_resonator_pitch_um, 11.2);

    let solver = AcoustomagnonicHaloscopeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.axion_conversion_fidelity >= 0.9980,
        "Axion conversion fidelity must be >= 0.9980, got {:.6}",
        metrics.axion_conversion_fidelity
    );
    assert!(
        metrics.metrology_state_retention_fraction >= 0.9970,
        "Metrology state retention fraction must be >= 0.9970, got {:.6}",
        metrics.metrology_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_cavity_crosstalk_isolation_db >= 55.0,
        "Inter-cavity crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_cavity_crosstalk_isolation_db
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
fn test_haloscope_coupling_scaling() {
    let mut p_low = AcoustomagnonicHaloscopeParams::default();
    p_low.haloscope_coupling_mev = 2.0;
    let mut p_high = AcoustomagnonicHaloscopeParams::default();
    p_high.haloscope_coupling_mev = 34.0;

    let solver_low = AcoustomagnonicHaloscopeSolver::new(p_low);
    let solver_high = AcoustomagnonicHaloscopeSolver::new(p_high);

    assert!(
        solver_high.compute_axion_conversion_fidelity()
            > solver_low.compute_axion_conversion_fidelity(),
        "Higher haloscope coupling energy should enhance axion conversion fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher haloscope coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher haloscope coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_axion_gap_scaling() {
    let mut p_low = AcoustomagnonicHaloscopeParams::default();
    p_low.topological_axion_gap_mev = 3.0;
    let mut p_high = AcoustomagnonicHaloscopeParams::default();
    p_high.topological_axion_gap_mev = 44.0;

    let solver_low = AcoustomagnonicHaloscopeSolver::new(p_low);
    let solver_high = AcoustomagnonicHaloscopeSolver::new(p_high);

    assert!(
        solver_high.compute_metrology_state_retention_fraction()
            > solver_low.compute_metrology_state_retention_fraction(),
        "Wider topological axion gap should improve metrology state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological axion gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = AcoustomagnonicHaloscopeParams::default();
    p_low.acoustic_drive_frequency_ghz = 1.5;
    let mut p_high = AcoustomagnonicHaloscopeParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.8;

    let solver_low = AcoustomagnonicHaloscopeSolver::new(p_low);
    let solver_high = AcoustomagnonicHaloscopeSolver::new(p_high);

    assert!(
        solver_high.compute_axion_conversion_fidelity()
            > solver_low.compute_axion_conversion_fidelity(),
        "Higher acoustic drive frequency should enhance axion conversion fidelity"
    );
    assert!(
        solver_high.compute_inter_cavity_crosstalk_isolation_db()
            > solver_low.compute_inter_cavity_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_haloscope_dispatch_speed_scaling() {
    let mut p_low = AcoustomagnonicHaloscopeParams::default();
    p_low.haloscope_dispatch_speed_m_per_s = 300.0;
    let mut p_high = AcoustomagnonicHaloscopeParams::default();
    p_high.haloscope_dispatch_speed_m_per_s = 2950.0;

    let solver_low = AcoustomagnonicHaloscopeSolver::new(p_low);
    let solver_high = AcoustomagnonicHaloscopeSolver::new(p_high);

    assert!(
        solver_high.compute_metrology_state_retention_fraction()
            > solver_low.compute_metrology_state_retention_fraction(),
        "Higher haloscope dispatch speed should improve retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher haloscope dispatch speed should increase protection gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = AcoustomagnonicHaloscopeParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = AcoustomagnonicHaloscopeParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = AcoustomagnonicHaloscopeSolver::new(p_cold);
    let solver_warm = AcoustomagnonicHaloscopeSolver::new(p_warm);

    assert!(
        solver_cold.compute_axion_conversion_fidelity()
            > solver_warm.compute_axion_conversion_fidelity(),
        "Lower cryogenic temperature should improve axion conversion fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = AcoustomagnonicHaloscopeParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = AcoustomagnonicHaloscopeParams::default();
    p_high.microwave_probe_power_uw = 29.0;

    let solver_low = AcoustomagnonicHaloscopeSolver::new(p_low);
    let solver_high = AcoustomagnonicHaloscopeSolver::new(p_high);

    assert!(
        solver_high.compute_axion_conversion_fidelity()
            > solver_low.compute_axion_conversion_fidelity(),
        "Higher probe power should improve axion conversion fidelity"
    );
    assert!(
        solver_high.compute_inter_cavity_crosstalk_isolation_db()
            > solver_low.compute_inter_cavity_crosstalk_isolation_db(),
        "Higher probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_cavity_modes_scaling() {
    let mut p_low = AcoustomagnonicHaloscopeParams::default();
    p_low.synthetic_cavity_modes_factor = 1.5;
    let mut p_high = AcoustomagnonicHaloscopeParams::default();
    p_high.synthetic_cavity_modes_factor = 7.5;

    let solver_low = AcoustomagnonicHaloscopeSolver::new(p_low);
    let solver_high = AcoustomagnonicHaloscopeSolver::new(p_high);

    assert!(
        solver_high.compute_inter_cavity_crosstalk_isolation_db()
            > solver_low.compute_inter_cavity_crosstalk_isolation_db(),
        "Higher synthetic cavity modes factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher synthetic cavity modes factor should suppress dephasing rate"
    );
}

#[test]
fn test_cavity_resonator_pitch_scaling() {
    let mut p_small = AcoustomagnonicHaloscopeParams::default();
    p_small.cavity_resonator_pitch_um = 1.0;
    let mut p_large = AcoustomagnonicHaloscopeParams::default();
    p_large.cavity_resonator_pitch_um = 19.0;

    let solver_small = AcoustomagnonicHaloscopeSolver::new(p_small);
    let solver_large = AcoustomagnonicHaloscopeSolver::new(p_large);

    assert!(
        solver_large.compute_inter_cavity_crosstalk_isolation_db()
            > solver_small.compute_inter_cavity_crosstalk_isolation_db(),
        "Larger cavity resonator pitch should enhance inter-cavity crosstalk isolation"
    );
    assert!(
        solver_large.compute_topological_protection_gap_mhz()
            > solver_small.compute_topological_protection_gap_mhz(),
        "Larger cavity resonator pitch should increase protection gap"
    );
}
