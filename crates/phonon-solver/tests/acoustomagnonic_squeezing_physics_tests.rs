#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Cavity Acoustomagnonic Squeezing &
//! Quantum Entangled Spin-Phonon Comb Engine.

use phonon_models::acoustomagnonic_squeezing::AcoustomagnonicSqueezingParams;
use phonon_solver::acoustomagnonic_squeezing::AcoustomagnonicSqueezingSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AcoustomagnonicSqueezingParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.squeezing_coupling_mev, 1.0);
    assert_eq!(underflow.topological_magnon_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.entanglement_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_squeezing_modes_factor, 1.0);
    assert_eq!(underflow.cavity_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = AcoustomagnonicSqueezingParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.squeezing_coupling_mev, 35.0);
    assert_eq!(overflow.topological_magnon_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.entanglement_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_squeezing_modes_factor, 8.0);
    assert_eq!(overflow.cavity_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AcoustomagnonicSqueezingParams::default();
    assert_eq!(params.squeezing_coupling_mev, 26.5);
    assert_eq!(params.topological_magnon_gap_mev, 32.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 10.8);
    assert_eq!(params.entanglement_dispatch_speed_m_per_s, 2350.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 10.8);
    assert_eq!(params.synthetic_squeezing_modes_factor, 4.0);
    assert_eq!(params.cavity_pitch_um, 9.8);

    let solver = AcoustomagnonicSqueezingSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.squeezing_fidelity >= 0.9980,
        "Squeezing fidelity must be >= 0.9980, got {:.6}",
        metrics.squeezing_fidelity
    );
    assert!(
        metrics.entanglement_state_retention_fraction >= 0.9970,
        "Entanglement state retention fraction must be >= 0.9970, got {:.6}",
        metrics.entanglement_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_mode_crosstalk_isolation_db >= 55.0,
        "Inter-mode crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_mode_crosstalk_isolation_db
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
fn test_squeezing_coupling_scaling() {
    let mut p_low = AcoustomagnonicSqueezingParams::default();
    p_low.squeezing_coupling_mev = 2.0;
    let mut p_high = AcoustomagnonicSqueezingParams::default();
    p_high.squeezing_coupling_mev = 34.0;

    let solver_low = AcoustomagnonicSqueezingSolver::new(p_low);
    let solver_high = AcoustomagnonicSqueezingSolver::new(p_high);

    assert!(
        solver_high.compute_squeezing_fidelity()
            > solver_low.compute_squeezing_fidelity(),
        "Higher squeezing coupling energy should enhance squeezing fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher squeezing coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher squeezing coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_magnon_gap_scaling() {
    let mut p_low = AcoustomagnonicSqueezingParams::default();
    p_low.topological_magnon_gap_mev = 3.0;
    let mut p_high = AcoustomagnonicSqueezingParams::default();
    p_high.topological_magnon_gap_mev = 44.0;

    let solver_low = AcoustomagnonicSqueezingSolver::new(p_low);
    let solver_high = AcoustomagnonicSqueezingSolver::new(p_high);

    assert!(
        solver_high.compute_entanglement_state_retention_fraction()
            > solver_low.compute_entanglement_state_retention_fraction(),
        "Wider topological magnon gap should improve retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological magnon gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = AcoustomagnonicSqueezingParams::default();
    p_low.acoustic_drive_frequency_ghz = 1.5;
    let mut p_high = AcoustomagnonicSqueezingParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.5;

    let solver_low = AcoustomagnonicSqueezingSolver::new(p_low);
    let solver_high = AcoustomagnonicSqueezingSolver::new(p_high);

    assert!(
        solver_high.compute_squeezing_fidelity()
            > solver_low.compute_squeezing_fidelity(),
        "Higher acoustic drive frequency should enhance squeezing fidelity"
    );
    assert!(
        solver_high.compute_inter_mode_crosstalk_isolation_db()
            > solver_low.compute_inter_mode_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_entanglement_dispatch_speed_scaling() {
    let mut p_low = AcoustomagnonicSqueezingParams::default();
    p_low.entanglement_dispatch_speed_m_per_s = 300.0;
    let mut p_high = AcoustomagnonicSqueezingParams::default();
    p_high.entanglement_dispatch_speed_m_per_s = 2900.0;

    let solver_low = AcoustomagnonicSqueezingSolver::new(p_low);
    let solver_high = AcoustomagnonicSqueezingSolver::new(p_high);

    assert!(
        solver_high.compute_squeezing_fidelity()
            > solver_low.compute_squeezing_fidelity(),
        "Higher dispatch speed should improve squeezing fidelity"
    );
    assert!(
        solver_high.compute_entanglement_state_retention_fraction()
            > solver_low.compute_entanglement_state_retention_fraction(),
        "Higher dispatch speed should improve retention fraction"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = AcoustomagnonicSqueezingParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = AcoustomagnonicSqueezingParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = AcoustomagnonicSqueezingSolver::new(p_cold);
    let solver_warm = AcoustomagnonicSqueezingSolver::new(p_warm);

    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature must strictly suppress dephasing rate"
    );
    assert!(
        solver_cold.compute_squeezing_fidelity()
            > solver_warm.compute_squeezing_fidelity(),
        "Lower cryogenic temperature should improve squeezing fidelity"
    );
    assert!(
        solver_cold.compute_entanglement_state_retention_fraction()
            > solver_warm.compute_entanglement_state_retention_fraction(),
        "Lower cryogenic temperature should improve retention fraction"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = AcoustomagnonicSqueezingParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = AcoustomagnonicSqueezingParams::default();
    p_high.microwave_probe_power_uw = 29.0;

    let solver_low = AcoustomagnonicSqueezingSolver::new(p_low);
    let solver_high = AcoustomagnonicSqueezingSolver::new(p_high);

    assert!(
        solver_high.compute_squeezing_fidelity()
            > solver_low.compute_squeezing_fidelity(),
        "Higher microwave probe power should improve squeezing fidelity"
    );
    assert!(
        solver_high.compute_inter_mode_crosstalk_isolation_db()
            > solver_low.compute_inter_mode_crosstalk_isolation_db(),
        "Higher microwave probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_squeezing_modes_scaling() {
    let mut p_low = AcoustomagnonicSqueezingParams::default();
    p_low.synthetic_squeezing_modes_factor = 1.5;
    let mut p_high = AcoustomagnonicSqueezingParams::default();
    p_high.synthetic_squeezing_modes_factor = 7.5;

    let solver_low = AcoustomagnonicSqueezingSolver::new(p_low);
    let solver_high = AcoustomagnonicSqueezingSolver::new(p_high);

    assert!(
        solver_high.compute_inter_mode_crosstalk_isolation_db()
            > solver_low.compute_inter_mode_crosstalk_isolation_db(),
        "Higher synthetic squeezing modes factor should improve crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher synthetic squeezing modes factor should increase protection gap"
    );
}

#[test]
fn test_cavity_pitch_scaling() {
    let mut p_narrow = AcoustomagnonicSqueezingParams::default();
    p_narrow.cavity_pitch_um = 1.0;
    let mut p_wide = AcoustomagnonicSqueezingParams::default();
    p_wide.cavity_pitch_um = 19.0;

    let solver_narrow = AcoustomagnonicSqueezingSolver::new(p_narrow);
    let solver_wide = AcoustomagnonicSqueezingSolver::new(p_wide);

    assert!(
        solver_wide.compute_inter_mode_crosstalk_isolation_db()
            > solver_narrow.compute_inter_mode_crosstalk_isolation_db(),
        "Larger cavity pitch should enhance inter-mode crosstalk isolation"
    );
    assert!(
        solver_wide.compute_topological_mode_dephasing_rate_hz()
            < solver_narrow.compute_topological_mode_dephasing_rate_hz(),
        "Larger cavity pitch should suppress dephasing rate"
    );
}
