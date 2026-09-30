#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological Axion
//! Waveguide & Chiral Anomaly Synthesizer.

use phonon_models::topological_axion::TopologicalAxionParams;
use phonon_solver::topological_axion::TopologicalAxionSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = TopologicalAxionParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.axion_coupling_mev, 1.0);
    assert_eq!(underflow.topological_axion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.chiral_anomaly_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_axion_layers_factor, 1.0);
    assert_eq!(underflow.waveguide_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = TopologicalAxionParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.axion_coupling_mev, 35.0);
    assert_eq!(overflow.topological_axion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.chiral_anomaly_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_axion_layers_factor, 8.0);
    assert_eq!(overflow.waveguide_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TopologicalAxionParams::default();
    assert_eq!(params.axion_coupling_mev, 23.0);
    assert_eq!(params.topological_axion_gap_mev, 29.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 9.0);
    assert_eq!(params.chiral_anomaly_dispatch_speed_m_per_s, 2000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 9.0);
    assert_eq!(params.synthetic_axion_layers_factor, 4.0);
    assert_eq!(params.waveguide_pitch_um, 8.0);

    let solver = TopologicalAxionSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.chiral_transport_fidelity >= 0.9980,
        "Chiral transport fidelity must be >= 0.9980, got {:.6}",
        metrics.chiral_transport_fidelity
    );
    assert!(
        metrics.axion_polariton_retention_fraction >= 0.9970,
        "Axion-polariton retention fraction must be >= 0.9970, got {:.6}",
        metrics.axion_polariton_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.non_reciprocal_isolation_db >= 55.0,
        "Non-reciprocal isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.non_reciprocal_isolation_db
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
fn test_axion_coupling_scaling() {
    let mut p_low = TopologicalAxionParams::default();
    p_low.axion_coupling_mev = 2.0;
    let mut p_high = TopologicalAxionParams::default();
    p_high.axion_coupling_mev = 34.0;

    let solver_low = TopologicalAxionSolver::new(p_low);
    let solver_high = TopologicalAxionSolver::new(p_high);

    assert!(
        solver_high.compute_chiral_transport_fidelity() > solver_low.compute_chiral_transport_fidelity(),
        "Higher axion coupling energy should enhance chiral transport fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher axion coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher axion coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_axion_gap_scaling() {
    let mut p_low = TopologicalAxionParams::default();
    p_low.topological_axion_gap_mev = 3.0;
    let mut p_high = TopologicalAxionParams::default();
    p_high.topological_axion_gap_mev = 44.0;

    let solver_low = TopologicalAxionSolver::new(p_low);
    let solver_high = TopologicalAxionSolver::new(p_high);

    assert!(
        solver_high.compute_axion_polariton_retention_fraction()
            > solver_low.compute_axion_polariton_retention_fraction(),
        "Wider topological axion gap should improve axion-polariton retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological axion gap should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Wider topological axion gap should reduce dephasing rate"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = TopologicalAxionParams::default();
    p_low.acoustic_drive_frequency_ghz = 2.0;
    let mut p_high = TopologicalAxionParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.0;

    let solver_low = TopologicalAxionSolver::new(p_low);
    let solver_high = TopologicalAxionSolver::new(p_high);

    assert!(
        solver_high.compute_chiral_transport_fidelity() > solver_low.compute_chiral_transport_fidelity(),
        "Higher acoustic drive frequency should enhance chiral transport fidelity"
    );
    assert!(
        solver_high.compute_non_reciprocal_isolation_db()
            > solver_low.compute_non_reciprocal_isolation_db(),
        "Higher acoustic drive frequency should increase non-reciprocal isolation"
    );
}

#[test]
fn test_chiral_anomaly_dispatch_speed_scaling() {
    let mut p_low = TopologicalAxionParams::default();
    p_low.chiral_anomaly_dispatch_speed_m_per_s = 300.0;
    let mut p_high = TopologicalAxionParams::default();
    p_high.chiral_anomaly_dispatch_speed_m_per_s = 2900.0;

    let solver_low = TopologicalAxionSolver::new(p_low);
    let solver_high = TopologicalAxionSolver::new(p_high);

    assert!(
        solver_high.compute_chiral_transport_fidelity() > solver_low.compute_chiral_transport_fidelity(),
        "Faster chiral anomaly dispatch speed should increase fidelity"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Faster chiral anomaly dispatch speed should reduce dephasing rate"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = TopologicalAxionParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = TopologicalAxionParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = TopologicalAxionSolver::new(p_cold);
    let solver_warm = TopologicalAxionSolver::new(p_warm);

    assert!(
        solver_cold.compute_chiral_transport_fidelity() > solver_warm.compute_chiral_transport_fidelity(),
        "Lower temperature should preserve higher fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower temperature should exhibit lower thermal dephasing"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = TopologicalAxionParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = TopologicalAxionParams::default();
    p_high.microwave_probe_power_uw = 28.0;

    let solver_low = TopologicalAxionSolver::new(p_low);
    let solver_high = TopologicalAxionSolver::new(p_high);

    assert!(
        solver_high.compute_axion_polariton_retention_fraction()
            > solver_low.compute_axion_polariton_retention_fraction(),
        "Optimized microwave probe power should enhance axion-polariton retention fraction"
    );
    assert!(
        solver_high.compute_non_reciprocal_isolation_db()
            > solver_low.compute_non_reciprocal_isolation_db(),
        "Higher probe power within linear regime should improve SNR and isolation"
    );
}

#[test]
fn test_synthetic_axion_layers_scaling() {
    let mut p_low = TopologicalAxionParams::default();
    p_low.synthetic_axion_layers_factor = 1.5;
    let mut p_high = TopologicalAxionParams::default();
    p_high.synthetic_axion_layers_factor = 7.5;

    let solver_low = TopologicalAxionSolver::new(p_low);
    let solver_high = TopologicalAxionSolver::new(p_high);

    assert!(
        solver_high.compute_chiral_transport_fidelity() > solver_low.compute_chiral_transport_fidelity(),
        "Higher synthetic axion layers factor should improve chiral transport fidelity"
    );
    assert!(
        solver_high.compute_non_reciprocal_isolation_db()
            > solver_low.compute_non_reciprocal_isolation_db(),
        "Higher synthetic axion layers factor should boost non-reciprocal isolation"
    );
}

#[test]
fn test_waveguide_pitch_scaling() {
    let mut p_tight = TopologicalAxionParams::default();
    p_tight.waveguide_pitch_um = 1.0;
    let mut p_wide = TopologicalAxionParams::default();
    p_wide.waveguide_pitch_um = 18.0;

    let solver_tight = TopologicalAxionSolver::new(p_tight);
    let solver_wide = TopologicalAxionSolver::new(p_wide);

    assert!(
        solver_wide.compute_non_reciprocal_isolation_db()
            > solver_tight.compute_non_reciprocal_isolation_db(),
        "Wider waveguide pitch must significantly increase non-reciprocal isolation"
    );
    assert!(
        solver_wide.compute_topological_protection_gap_mhz()
            > solver_tight.compute_topological_protection_gap_mhz(),
        "Wider waveguide pitch suppresses parasitic fringing and widens protection gap"
    );
}
