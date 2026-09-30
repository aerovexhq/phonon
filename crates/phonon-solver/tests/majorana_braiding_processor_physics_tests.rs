#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Topological Majorana Zero-Mode
//! Braiding Processor and Parity Qubit Synthesizer.

use phonon_models::majorana_braiding_processor::MajoranaBraidingProcessorParams;
use phonon_solver::majorana_braiding_processor::MajoranaBraidingProcessorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = MajoranaBraidingProcessorParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.braiding_coupling_mev, 1.0);
    assert_eq!(underflow.topological_majorana_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.braiding_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_majorana_modes_factor, 1.0);
    assert_eq!(underflow.braiding_junction_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = MajoranaBraidingProcessorParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.braiding_coupling_mev, 35.0);
    assert_eq!(overflow.topological_majorana_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.braiding_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_majorana_modes_factor, 8.0);
    assert_eq!(overflow.braiding_junction_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = MajoranaBraidingProcessorParams::default();
    assert_eq!(params.braiding_coupling_mev, 29.0);
    assert_eq!(params.topological_majorana_gap_mev, 35.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.braiding_dispatch_speed_m_per_s, 2600.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 12.0);
    assert_eq!(params.synthetic_majorana_modes_factor, 4.0);
    assert_eq!(params.braiding_junction_pitch_um, 11.0);

    let solver = MajoranaBraidingProcessorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.braiding_fidelity >= 0.9980,
        "Braiding gate fidelity must be >= 0.9980, got {:.6}",
        metrics.braiding_fidelity
    );
    assert!(
        metrics.parity_state_retention_fraction >= 0.9970,
        "Parity state retention fraction must be >= 0.9970, got {:.6}",
        metrics.parity_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_junction_crosstalk_isolation_db >= 55.0,
        "Inter-junction crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_junction_crosstalk_isolation_db
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
fn test_braiding_coupling_scaling() {
    let mut p_low = MajoranaBraidingProcessorParams::default();
    p_low.braiding_coupling_mev = 2.0;
    let mut p_high = MajoranaBraidingProcessorParams::default();
    p_high.braiding_coupling_mev = 34.0;

    let solver_low = MajoranaBraidingProcessorSolver::new(p_low);
    let solver_high = MajoranaBraidingProcessorSolver::new(p_high);

    assert!(
        solver_high.compute_braiding_fidelity()
            > solver_low.compute_braiding_fidelity(),
        "Higher braiding coupling energy should enhance braiding gate fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher braiding coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher braiding coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_majorana_gap_scaling() {
    let mut p_low = MajoranaBraidingProcessorParams::default();
    p_low.topological_majorana_gap_mev = 3.0;
    let mut p_high = MajoranaBraidingProcessorParams::default();
    p_high.topological_majorana_gap_mev = 44.0;

    let solver_low = MajoranaBraidingProcessorSolver::new(p_low);
    let solver_high = MajoranaBraidingProcessorSolver::new(p_high);

    assert!(
        solver_high.compute_parity_state_retention_fraction()
            > solver_low.compute_parity_state_retention_fraction(),
        "Wider topological Majorana gap should improve parity state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological Majorana gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = MajoranaBraidingProcessorParams::default();
    p_low.acoustic_drive_frequency_ghz = 1.5;
    let mut p_high = MajoranaBraidingProcessorParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.8;

    let solver_low = MajoranaBraidingProcessorSolver::new(p_low);
    let solver_high = MajoranaBraidingProcessorSolver::new(p_high);

    assert!(
        solver_high.compute_braiding_fidelity()
            > solver_low.compute_braiding_fidelity(),
        "Higher acoustic drive frequency should enhance braiding gate fidelity"
    );
    assert!(
        solver_high.compute_inter_junction_crosstalk_isolation_db()
            > solver_low.compute_inter_junction_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_braiding_dispatch_speed_scaling() {
    let mut p_low = MajoranaBraidingProcessorParams::default();
    p_low.braiding_dispatch_speed_m_per_s = 300.0;
    let mut p_high = MajoranaBraidingProcessorParams::default();
    p_high.braiding_dispatch_speed_m_per_s = 2950.0;

    let solver_low = MajoranaBraidingProcessorSolver::new(p_low);
    let solver_high = MajoranaBraidingProcessorSolver::new(p_high);

    assert!(
        solver_high.compute_parity_state_retention_fraction()
            > solver_low.compute_parity_state_retention_fraction(),
        "Higher braiding dispatch speed should improve parity state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher braiding dispatch speed should increase protection gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = MajoranaBraidingProcessorParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = MajoranaBraidingProcessorParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = MajoranaBraidingProcessorSolver::new(p_cold);
    let solver_warm = MajoranaBraidingProcessorSolver::new(p_warm);

    assert!(
        solver_cold.compute_braiding_fidelity()
            > solver_warm.compute_braiding_fidelity(),
        "Lower cryogenic temperature should improve braiding gate fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = MajoranaBraidingProcessorParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = MajoranaBraidingProcessorParams::default();
    p_high.microwave_probe_power_uw = 29.0;

    let solver_low = MajoranaBraidingProcessorSolver::new(p_low);
    let solver_high = MajoranaBraidingProcessorSolver::new(p_high);

    assert!(
        solver_high.compute_braiding_fidelity()
            > solver_low.compute_braiding_fidelity(),
        "Higher probe power should improve braiding gate fidelity"
    );
    assert!(
        solver_high.compute_inter_junction_crosstalk_isolation_db()
            > solver_low.compute_inter_junction_crosstalk_isolation_db(),
        "Higher probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_majorana_modes_scaling() {
    let mut p_low = MajoranaBraidingProcessorParams::default();
    p_low.synthetic_majorana_modes_factor = 1.5;
    let mut p_high = MajoranaBraidingProcessorParams::default();
    p_high.synthetic_majorana_modes_factor = 7.5;

    let solver_low = MajoranaBraidingProcessorSolver::new(p_low);
    let solver_high = MajoranaBraidingProcessorSolver::new(p_high);

    assert!(
        solver_high.compute_inter_junction_crosstalk_isolation_db()
            > solver_low.compute_inter_junction_crosstalk_isolation_db(),
        "Higher synthetic Majorana modes factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher synthetic Majorana modes factor should suppress dephasing rate"
    );
}

#[test]
fn test_braiding_junction_pitch_scaling() {
    let mut p_small = MajoranaBraidingProcessorParams::default();
    p_small.braiding_junction_pitch_um = 1.0;
    let mut p_large = MajoranaBraidingProcessorParams::default();
    p_large.braiding_junction_pitch_um = 19.0;

    let solver_small = MajoranaBraidingProcessorSolver::new(p_small);
    let solver_large = MajoranaBraidingProcessorSolver::new(p_large);

    assert!(
        solver_large.compute_inter_junction_crosstalk_isolation_db()
            > solver_small.compute_inter_junction_crosstalk_isolation_db(),
        "Larger junction pitch should enhance inter-junction crosstalk isolation"
    );
    assert!(
        solver_large.compute_topological_protection_gap_mhz()
            > solver_small.compute_topological_protection_gap_mhz(),
        "Larger junction pitch should increase protection gap"
    );
}
