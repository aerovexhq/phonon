#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Levitated
//! Nanoparticle Metrology and Quantum Force Sensor Engine.

use phonon_models::acoustically_levitated_nanoparticle::AcousticallyLevitatedNanoparticleParams;
use phonon_solver::acoustically_levitated_nanoparticle::AcousticallyLevitatedNanoparticleSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AcousticallyLevitatedNanoparticleParams::new(
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
    assert_eq!(underflow.topological_force_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.levitation_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_trap_nodes_factor, 1.0);
    assert_eq!(underflow.levitation_trap_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = AcousticallyLevitatedNanoparticleParams::new(
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
    assert_eq!(overflow.topological_force_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.levitation_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_trap_nodes_factor, 8.0);
    assert_eq!(overflow.levitation_trap_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AcousticallyLevitatedNanoparticleParams::default();
    assert_eq!(params.levitation_coupling_mev, 31.0);
    assert_eq!(params.topological_force_gap_mev, 37.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.levitation_dispatch_speed_m_per_s, 2800.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 13.0);
    assert_eq!(params.synthetic_trap_nodes_factor, 4.0);
    assert_eq!(params.levitation_trap_pitch_um, 12.0);

    let solver = AcousticallyLevitatedNanoparticleSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.force_sensitivity_fidelity >= 0.9980,
        "Force sensitivity fidelity must be >= 0.9980, got {:.6}",
        metrics.force_sensitivity_fidelity
    );
    assert!(
        metrics.coherent_state_retention_fraction >= 0.9970,
        "Coherent state retention fraction must be >= 0.9970, got {:.6}",
        metrics.coherent_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_trap_crosstalk_isolation_db >= 55.0,
        "Inter-trap crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_trap_crosstalk_isolation_db
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
    let mut p_low = AcousticallyLevitatedNanoparticleParams::default();
    p_low.levitation_coupling_mev = 2.0;
    let mut p_high = AcousticallyLevitatedNanoparticleParams::default();
    p_high.levitation_coupling_mev = 34.0;

    let solver_low = AcousticallyLevitatedNanoparticleSolver::new(p_low);
    let solver_high = AcousticallyLevitatedNanoparticleSolver::new(p_high);

    assert!(
        solver_high.compute_force_sensitivity_fidelity()
            > solver_low.compute_force_sensitivity_fidelity(),
        "Higher levitation coupling energy should enhance force sensitivity fidelity"
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
fn test_topological_force_gap_scaling() {
    let mut p_low = AcousticallyLevitatedNanoparticleParams::default();
    p_low.topological_force_gap_mev = 3.0;
    let mut p_high = AcousticallyLevitatedNanoparticleParams::default();
    p_high.topological_force_gap_mev = 44.0;

    let solver_low = AcousticallyLevitatedNanoparticleSolver::new(p_low);
    let solver_high = AcousticallyLevitatedNanoparticleSolver::new(p_high);

    assert!(
        solver_high.compute_coherent_state_retention_fraction()
            > solver_low.compute_coherent_state_retention_fraction(),
        "Wider topological force gap should improve coherent state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological force gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = AcousticallyLevitatedNanoparticleParams::default();
    p_low.acoustic_drive_frequency_ghz = 1.5;
    let mut p_high = AcousticallyLevitatedNanoparticleParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.8;

    let solver_low = AcousticallyLevitatedNanoparticleSolver::new(p_low);
    let solver_high = AcousticallyLevitatedNanoparticleSolver::new(p_high);

    assert!(
        solver_high.compute_force_sensitivity_fidelity()
            > solver_low.compute_force_sensitivity_fidelity(),
        "Higher acoustic drive frequency should enhance force sensitivity fidelity"
    );
    assert!(
        solver_high.compute_inter_trap_crosstalk_isolation_db()
            > solver_low.compute_inter_trap_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_levitation_dispatch_speed_scaling() {
    let mut p_low = AcousticallyLevitatedNanoparticleParams::default();
    p_low.levitation_dispatch_speed_m_per_s = 300.0;
    let mut p_high = AcousticallyLevitatedNanoparticleParams::default();
    p_high.levitation_dispatch_speed_m_per_s = 2950.0;

    let solver_low = AcousticallyLevitatedNanoparticleSolver::new(p_low);
    let solver_high = AcousticallyLevitatedNanoparticleSolver::new(p_high);

    assert!(
        solver_high.compute_coherent_state_retention_fraction()
            > solver_low.compute_coherent_state_retention_fraction(),
        "Higher levitation dispatch speed should improve retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher levitation dispatch speed should increase protection gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = AcousticallyLevitatedNanoparticleParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = AcousticallyLevitatedNanoparticleParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = AcousticallyLevitatedNanoparticleSolver::new(p_cold);
    let solver_warm = AcousticallyLevitatedNanoparticleSolver::new(p_warm);

    assert!(
        solver_cold.compute_force_sensitivity_fidelity()
            > solver_warm.compute_force_sensitivity_fidelity(),
        "Lower cryogenic temperature should improve force sensitivity fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = AcousticallyLevitatedNanoparticleParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = AcousticallyLevitatedNanoparticleParams::default();
    p_high.microwave_probe_power_uw = 29.0;

    let solver_low = AcousticallyLevitatedNanoparticleSolver::new(p_low);
    let solver_high = AcousticallyLevitatedNanoparticleSolver::new(p_high);

    assert!(
        solver_high.compute_force_sensitivity_fidelity()
            > solver_low.compute_force_sensitivity_fidelity(),
        "Higher probe power should improve force sensitivity fidelity"
    );
    assert!(
        solver_high.compute_inter_trap_crosstalk_isolation_db()
            > solver_low.compute_inter_trap_crosstalk_isolation_db(),
        "Higher probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_trap_nodes_scaling() {
    let mut p_low = AcousticallyLevitatedNanoparticleParams::default();
    p_low.synthetic_trap_nodes_factor = 1.5;
    let mut p_high = AcousticallyLevitatedNanoparticleParams::default();
    p_high.synthetic_trap_nodes_factor = 7.5;

    let solver_low = AcousticallyLevitatedNanoparticleSolver::new(p_low);
    let solver_high = AcousticallyLevitatedNanoparticleSolver::new(p_high);

    assert!(
        solver_high.compute_inter_trap_crosstalk_isolation_db()
            > solver_low.compute_inter_trap_crosstalk_isolation_db(),
        "Higher synthetic trap nodes factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher synthetic trap nodes factor should suppress dephasing rate"
    );
}

#[test]
fn test_levitation_trap_pitch_scaling() {
    let mut p_small = AcousticallyLevitatedNanoparticleParams::default();
    p_small.levitation_trap_pitch_um = 1.0;
    let mut p_large = AcousticallyLevitatedNanoparticleParams::default();
    p_large.levitation_trap_pitch_um = 19.0;

    let solver_small = AcousticallyLevitatedNanoparticleSolver::new(p_small);
    let solver_large = AcousticallyLevitatedNanoparticleSolver::new(p_large);

    assert!(
        solver_large.compute_inter_trap_crosstalk_isolation_db()
            > solver_small.compute_inter_trap_crosstalk_isolation_db(),
        "Larger levitation trap pitch should enhance inter-trap crosstalk isolation"
    );
    assert!(
        solver_large.compute_topological_protection_gap_mhz()
            > solver_small.compute_topological_protection_gap_mhz(),
        "Larger levitation trap pitch should increase protection gap"
    );
}
