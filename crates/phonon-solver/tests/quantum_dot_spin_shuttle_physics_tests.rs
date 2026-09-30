#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Quantum Dot Spin Qubit Shuttle & Spin-Orbit Logic Engine.

use phonon_models::quantum_dot_spin_shuttle::QuantumDotSpinShuttleParams;
use phonon_solver::quantum_dot_spin_shuttle::QuantumDotSpinShuttleSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = QuantumDotSpinShuttleParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.shuttle_coupling_mev, 1.0);
    assert_eq!(underflow.topological_shuttle_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.shuttle_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_shuttle_channels_factor, 1.0);
    assert_eq!(underflow.shuttle_channel_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = QuantumDotSpinShuttleParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.shuttle_coupling_mev, 35.0);
    assert_eq!(overflow.topological_shuttle_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.shuttle_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_shuttle_channels_factor, 8.0);
    assert_eq!(overflow.shuttle_channel_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = QuantumDotSpinShuttleParams::default();
    assert_eq!(params.shuttle_coupling_mev, 31.5);
    assert_eq!(params.topological_shuttle_gap_mev, 37.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.shuttle_dispatch_speed_m_per_s, 2850.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 13.2);
    assert_eq!(params.synthetic_shuttle_channels_factor, 4.0);
    assert_eq!(params.shuttle_channel_pitch_um, 12.2);

    let solver = QuantumDotSpinShuttleSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.shuttle_fidelity >= 0.9980,
        "Shuttle fidelity must be >= 0.9980, got {:.6}",
        metrics.shuttle_fidelity
    );
    assert!(
        metrics.coherence_retention_fraction >= 0.9970,
        "Coherence retention fraction must be >= 0.9970, got {:.6}",
        metrics.coherence_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_channel_crosstalk_isolation_db >= 55.0,
        "Inter-channel crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_channel_crosstalk_isolation_db
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
fn test_shuttle_coupling_scaling() {
    let mut p_low = QuantumDotSpinShuttleParams::default();
    p_low.shuttle_coupling_mev = 2.0;
    let mut p_high = QuantumDotSpinShuttleParams::default();
    p_high.shuttle_coupling_mev = 34.0;

    let solver_low = QuantumDotSpinShuttleSolver::new(p_low);
    let solver_high = QuantumDotSpinShuttleSolver::new(p_high);

    assert!(
        solver_high.compute_shuttle_fidelity() > solver_low.compute_shuttle_fidelity(),
        "Higher shuttle coupling energy should enhance shuttle fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher shuttle coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher shuttle coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_shuttle_gap_scaling() {
    let mut p_low = QuantumDotSpinShuttleParams::default();
    p_low.topological_shuttle_gap_mev = 3.0;
    let mut p_high = QuantumDotSpinShuttleParams::default();
    p_high.topological_shuttle_gap_mev = 44.0;

    let solver_low = QuantumDotSpinShuttleSolver::new(p_low);
    let solver_high = QuantumDotSpinShuttleSolver::new(p_high);

    assert!(
        solver_high.compute_coherence_retention_fraction()
            > solver_low.compute_coherence_retention_fraction(),
        "Wider topological shuttle gap should improve coherence retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological shuttle gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = QuantumDotSpinShuttleParams::default();
    p_low.acoustic_drive_frequency_ghz = 1.5;
    let mut p_high = QuantumDotSpinShuttleParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.8;

    let solver_low = QuantumDotSpinShuttleSolver::new(p_low);
    let solver_high = QuantumDotSpinShuttleSolver::new(p_high);

    assert!(
        solver_high.compute_shuttle_fidelity() > solver_low.compute_shuttle_fidelity(),
        "Higher acoustic drive frequency should enhance shuttle fidelity"
    );
    assert!(
        solver_high.compute_inter_channel_crosstalk_isolation_db()
            > solver_low.compute_inter_channel_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_shuttle_dispatch_speed_scaling() {
    let mut p_low = QuantumDotSpinShuttleParams::default();
    p_low.shuttle_dispatch_speed_m_per_s = 300.0;
    let mut p_high = QuantumDotSpinShuttleParams::default();
    p_high.shuttle_dispatch_speed_m_per_s = 2950.0;

    let solver_low = QuantumDotSpinShuttleSolver::new(p_low);
    let solver_high = QuantumDotSpinShuttleSolver::new(p_high);

    assert!(
        solver_high.compute_coherence_retention_fraction()
            > solver_low.compute_coherence_retention_fraction(),
        "Higher shuttle dispatch speed should improve coherence retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher shuttle dispatch speed should increase protection gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = QuantumDotSpinShuttleParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = QuantumDotSpinShuttleParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = QuantumDotSpinShuttleSolver::new(p_cold);
    let solver_warm = QuantumDotSpinShuttleSolver::new(p_warm);

    assert!(
        solver_cold.compute_shuttle_fidelity() > solver_warm.compute_shuttle_fidelity(),
        "Lower cryogenic temperature should improve shuttle fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = QuantumDotSpinShuttleParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = QuantumDotSpinShuttleParams::default();
    p_high.microwave_probe_power_uw = 29.0;

    let solver_low = QuantumDotSpinShuttleSolver::new(p_low);
    let solver_high = QuantumDotSpinShuttleSolver::new(p_high);

    assert!(
        solver_high.compute_shuttle_fidelity() > solver_low.compute_shuttle_fidelity(),
        "Higher probe power should improve shuttle fidelity"
    );
    assert!(
        solver_high.compute_inter_channel_crosstalk_isolation_db()
            > solver_low.compute_inter_channel_crosstalk_isolation_db(),
        "Higher probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_shuttle_channels_scaling() {
    let mut p_low = QuantumDotSpinShuttleParams::default();
    p_low.synthetic_shuttle_channels_factor = 1.5;
    let mut p_high = QuantumDotSpinShuttleParams::default();
    p_high.synthetic_shuttle_channels_factor = 7.5;

    let solver_low = QuantumDotSpinShuttleSolver::new(p_low);
    let solver_high = QuantumDotSpinShuttleSolver::new(p_high);

    assert!(
        solver_high.compute_inter_channel_crosstalk_isolation_db()
            > solver_low.compute_inter_channel_crosstalk_isolation_db(),
        "Higher synthetic shuttle channels factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher synthetic shuttle channels factor should suppress dephasing rate"
    );
}

#[test]
fn test_shuttle_channel_pitch_scaling() {
    let mut p_small = QuantumDotSpinShuttleParams::default();
    p_small.shuttle_channel_pitch_um = 1.0;
    let mut p_large = QuantumDotSpinShuttleParams::default();
    p_large.shuttle_channel_pitch_um = 19.0;

    let solver_small = QuantumDotSpinShuttleSolver::new(p_small);
    let solver_large = QuantumDotSpinShuttleSolver::new(p_large);

    assert!(
        solver_large.compute_inter_channel_crosstalk_isolation_db()
            > solver_small.compute_inter_channel_crosstalk_isolation_db(),
        "Larger shuttle channel pitch should enhance inter-channel crosstalk isolation"
    );
    assert!(
        solver_large.compute_topological_protection_gap_mhz()
            > solver_small.compute_topological_protection_gap_mhz(),
        "Larger shuttle channel pitch should increase protection gap"
    );
}
