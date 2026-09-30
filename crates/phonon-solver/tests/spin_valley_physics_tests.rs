#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Mediated Spin-Valley Polariton Multiplexer &
//! 2D Valleytronics Engine.

use phonon_models::spin_valley::SpinValleyParams;
use phonon_solver::spin_valley::SpinValleySolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SpinValleyParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.valley_coupling_mev, 1.0);
    assert_eq!(underflow.topological_valley_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.polariton_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_valley_layers_factor, 1.0);
    assert_eq!(underflow.multiplexer_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = SpinValleyParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.valley_coupling_mev, 35.0);
    assert_eq!(overflow.topological_valley_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.polariton_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_valley_layers_factor, 8.0);
    assert_eq!(overflow.multiplexer_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SpinValleyParams::default();
    assert_eq!(params.valley_coupling_mev, 25.5);
    assert_eq!(params.topological_valley_gap_mev, 31.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 10.2);
    assert_eq!(params.polariton_dispatch_speed_m_per_s, 2250.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 10.2);
    assert_eq!(params.synthetic_valley_layers_factor, 4.0);
    assert_eq!(params.multiplexer_pitch_um, 9.2);

    let solver = SpinValleySolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.multiplexing_fidelity >= 0.9980,
        "Multiplexing fidelity must be >= 0.9980, got {:.6}",
        metrics.multiplexing_fidelity
    );
    assert!(
        metrics.valley_polarization_retention_fraction >= 0.9970,
        "Valley polarization retention fraction must be >= 0.9970, got {:.6}",
        metrics.valley_polarization_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_valley_crosstalk_isolation_db >= 55.0,
        "Inter-valley crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_valley_crosstalk_isolation_db
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
fn test_valley_coupling_scaling() {
    let mut p_low = SpinValleyParams::default();
    p_low.valley_coupling_mev = 2.0;
    let mut p_high = SpinValleyParams::default();
    p_high.valley_coupling_mev = 34.0;

    let solver_low = SpinValleySolver::new(p_low);
    let solver_high = SpinValleySolver::new(p_high);

    assert!(
        solver_high.compute_multiplexing_fidelity()
            > solver_low.compute_multiplexing_fidelity(),
        "Higher valley coupling energy should enhance multiplexing fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher valley coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher valley coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_valley_gap_scaling() {
    let mut p_low = SpinValleyParams::default();
    p_low.topological_valley_gap_mev = 3.0;
    let mut p_high = SpinValleyParams::default();
    p_high.topological_valley_gap_mev = 44.0;

    let solver_low = SpinValleySolver::new(p_low);
    let solver_high = SpinValleySolver::new(p_high);

    assert!(
        solver_high.compute_valley_polarization_retention_fraction()
            > solver_low.compute_valley_polarization_retention_fraction(),
        "Wider topological valley gap should improve retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological valley gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = SpinValleyParams::default();
    p_low.acoustic_drive_frequency_ghz = 1.5;
    let mut p_high = SpinValleyParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.5;

    let solver_low = SpinValleySolver::new(p_low);
    let solver_high = SpinValleySolver::new(p_high);

    assert!(
        solver_high.compute_multiplexing_fidelity()
            > solver_low.compute_multiplexing_fidelity(),
        "Higher acoustic drive frequency should enhance multiplexing fidelity"
    );
    assert!(
        solver_high.compute_inter_valley_crosstalk_isolation_db()
            > solver_low.compute_inter_valley_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_polariton_dispatch_speed_scaling() {
    let mut p_low = SpinValleyParams::default();
    p_low.polariton_dispatch_speed_m_per_s = 300.0;
    let mut p_high = SpinValleyParams::default();
    p_high.polariton_dispatch_speed_m_per_s = 2900.0;

    let solver_low = SpinValleySolver::new(p_low);
    let solver_high = SpinValleySolver::new(p_high);

    assert!(
        solver_high.compute_multiplexing_fidelity()
            > solver_low.compute_multiplexing_fidelity(),
        "Higher polariton dispatch speed should improve multiplexing fidelity"
    );
    assert!(
        solver_high.compute_valley_polarization_retention_fraction()
            > solver_low.compute_valley_polarization_retention_fraction(),
        "Higher polariton dispatch speed should improve retention fraction"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = SpinValleyParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = SpinValleyParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = SpinValleySolver::new(p_cold);
    let solver_warm = SpinValleySolver::new(p_warm);

    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature must strictly suppress dephasing rate"
    );
    assert!(
        solver_cold.compute_multiplexing_fidelity()
            > solver_warm.compute_multiplexing_fidelity(),
        "Lower cryogenic temperature should improve multiplexing fidelity"
    );
    assert!(
        solver_cold.compute_valley_polarization_retention_fraction()
            > solver_warm.compute_valley_polarization_retention_fraction(),
        "Lower cryogenic temperature should improve retention fraction"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = SpinValleyParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = SpinValleyParams::default();
    p_high.microwave_probe_power_uw = 29.0;

    let solver_low = SpinValleySolver::new(p_low);
    let solver_high = SpinValleySolver::new(p_high);

    assert!(
        solver_high.compute_multiplexing_fidelity()
            > solver_low.compute_multiplexing_fidelity(),
        "Higher microwave probe power should improve multiplexing fidelity"
    );
    assert!(
        solver_high.compute_inter_valley_crosstalk_isolation_db()
            > solver_low.compute_inter_valley_crosstalk_isolation_db(),
        "Higher microwave probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_valley_layers_scaling() {
    let mut p_low = SpinValleyParams::default();
    p_low.synthetic_valley_layers_factor = 1.5;
    let mut p_high = SpinValleyParams::default();
    p_high.synthetic_valley_layers_factor = 7.5;

    let solver_low = SpinValleySolver::new(p_low);
    let solver_high = SpinValleySolver::new(p_high);

    assert!(
        solver_high.compute_inter_valley_crosstalk_isolation_db()
            > solver_low.compute_inter_valley_crosstalk_isolation_db(),
        "Higher synthetic valley layers factor should improve crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher synthetic valley layers factor should increase protection gap"
    );
}

#[test]
fn test_multiplexer_pitch_scaling() {
    let mut p_narrow = SpinValleyParams::default();
    p_narrow.multiplexer_pitch_um = 1.0;
    let mut p_wide = SpinValleyParams::default();
    p_wide.multiplexer_pitch_um = 19.0;

    let solver_narrow = SpinValleySolver::new(p_narrow);
    let solver_wide = SpinValleySolver::new(p_wide);

    assert!(
        solver_wide.compute_inter_valley_crosstalk_isolation_db()
            > solver_narrow.compute_inter_valley_crosstalk_isolation_db(),
        "Larger multiplexer pitch should enhance inter-valley crosstalk isolation"
    );
    assert!(
        solver_wide.compute_topological_mode_dephasing_rate_hz()
            < solver_narrow.compute_topological_mode_dephasing_rate_hz(),
        "Larger multiplexer pitch should suppress dephasing rate"
    );
}
