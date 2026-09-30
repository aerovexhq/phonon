#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Optomechanical Superradiance Lattice &
//! Chiral Phonon Laser Array Engine.

use phonon_models::superradiance_laser::SuperradianceLaserParams;
use phonon_solver::superradiance_laser::SuperradianceLaserSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SuperradianceLaserParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.superradiance_coupling_mev, 1.0);
    assert_eq!(underflow.topological_laser_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.stimulated_emission_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_laser_emitters_factor, 1.0);
    assert_eq!(underflow.laser_array_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = SuperradianceLaserParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.superradiance_coupling_mev, 35.0);
    assert_eq!(overflow.topological_laser_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.stimulated_emission_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_laser_emitters_factor, 8.0);
    assert_eq!(overflow.laser_array_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SuperradianceLaserParams::default();
    assert_eq!(params.superradiance_coupling_mev, 22.5);
    assert_eq!(params.topological_laser_gap_mev, 28.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 8.8);
    assert_eq!(params.stimulated_emission_dispatch_speed_m_per_s, 1950.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 8.8);
    assert_eq!(params.synthetic_laser_emitters_factor, 4.0);
    assert_eq!(params.laser_array_pitch_um, 7.8);

    let solver = SuperradianceLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.lasing_emission_fidelity >= 0.9980,
        "Lasing emission fidelity must be >= 0.9980, got {:.6}",
        metrics.lasing_emission_fidelity
    );
    assert!(
        metrics.phonon_state_retention_fraction >= 0.9970,
        "Phonon state retention fraction must be >= 0.9970, got {:.6}",
        metrics.phonon_state_retention_fraction
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
fn test_superradiance_coupling_scaling() {
    let mut p_low = SuperradianceLaserParams::default();
    p_low.superradiance_coupling_mev = 2.0;
    let mut p_high = SuperradianceLaserParams::default();
    p_high.superradiance_coupling_mev = 34.0;

    let solver_low = SuperradianceLaserSolver::new(p_low);
    let solver_high = SuperradianceLaserSolver::new(p_high);

    assert!(
        solver_high.compute_lasing_emission_fidelity() > solver_low.compute_lasing_emission_fidelity(),
        "Higher superradiance coupling energy should enhance lasing fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher superradiance coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher superradiance coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_laser_gap_scaling() {
    let mut p_low = SuperradianceLaserParams::default();
    p_low.topological_laser_gap_mev = 3.0;
    let mut p_high = SuperradianceLaserParams::default();
    p_high.topological_laser_gap_mev = 44.0;

    let solver_low = SuperradianceLaserSolver::new(p_low);
    let solver_high = SuperradianceLaserSolver::new(p_high);

    assert!(
        solver_high.compute_phonon_state_retention_fraction()
            > solver_low.compute_phonon_state_retention_fraction(),
        "Wider topological laser gap should improve phonon state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological laser gap should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Wider topological laser gap should reduce dephasing rate"
    );
}

#[test]
fn test_acoustic_frequency_scaling() {
    let mut p_low = SuperradianceLaserParams::default();
    p_low.acoustic_drive_frequency_ghz = 2.0;
    let mut p_high = SuperradianceLaserParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.0;

    let solver_low = SuperradianceLaserSolver::new(p_low);
    let solver_high = SuperradianceLaserSolver::new(p_high);

    assert!(
        solver_high.compute_lasing_emission_fidelity() > solver_low.compute_lasing_emission_fidelity(),
        "Higher acoustic drive frequency should enhance lasing fidelity"
    );
    assert!(
        solver_high.compute_inter_mode_crosstalk_isolation_db()
            > solver_low.compute_inter_mode_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should increase isolation"
    );
}

#[test]
fn test_emission_speed_scaling() {
    let mut p_low = SuperradianceLaserParams::default();
    p_low.stimulated_emission_dispatch_speed_m_per_s = 300.0;
    let mut p_high = SuperradianceLaserParams::default();
    p_high.stimulated_emission_dispatch_speed_m_per_s = 2900.0;

    let solver_low = SuperradianceLaserSolver::new(p_low);
    let solver_high = SuperradianceLaserSolver::new(p_high);

    assert!(
        solver_high.compute_lasing_emission_fidelity() > solver_low.compute_lasing_emission_fidelity(),
        "Faster stimulated emission dispatch speed should increase fidelity"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Faster stimulated emission dispatch speed should reduce dephasing rate"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = SuperradianceLaserParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = SuperradianceLaserParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = SuperradianceLaserSolver::new(p_cold);
    let solver_warm = SuperradianceLaserSolver::new(p_warm);

    assert!(
        solver_cold.compute_lasing_emission_fidelity() > solver_warm.compute_lasing_emission_fidelity(),
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
    let mut p_low = SuperradianceLaserParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = SuperradianceLaserParams::default();
    p_high.microwave_probe_power_uw = 28.0;

    let solver_low = SuperradianceLaserSolver::new(p_low);
    let solver_high = SuperradianceLaserSolver::new(p_high);

    assert!(
        solver_high.compute_phonon_state_retention_fraction()
            > solver_low.compute_phonon_state_retention_fraction(),
        "Optimized microwave probe power should enhance phonon state retention fraction"
    );
    assert!(
        solver_high.compute_inter_mode_crosstalk_isolation_db()
            > solver_low.compute_inter_mode_crosstalk_isolation_db(),
        "Higher probe power within linear regime should improve SNR and isolation"
    );
}

#[test]
fn test_synthetic_laser_emitters_scaling() {
    let mut p_low = SuperradianceLaserParams::default();
    p_low.synthetic_laser_emitters_factor = 1.5;
    let mut p_high = SuperradianceLaserParams::default();
    p_high.synthetic_laser_emitters_factor = 7.5;

    let solver_low = SuperradianceLaserSolver::new(p_low);
    let solver_high = SuperradianceLaserSolver::new(p_high);

    assert!(
        solver_high.compute_lasing_emission_fidelity() > solver_low.compute_lasing_emission_fidelity(),
        "Higher synthetic laser emitters factor should improve lasing emission fidelity"
    );
    assert!(
        solver_high.compute_inter_mode_crosstalk_isolation_db()
            > solver_low.compute_inter_mode_crosstalk_isolation_db(),
        "Higher synthetic laser emitters factor should boost crosstalk isolation"
    );
}

#[test]
fn test_laser_array_pitch_scaling() {
    let mut p_tight = SuperradianceLaserParams::default();
    p_tight.laser_array_pitch_um = 1.0;
    let mut p_wide = SuperradianceLaserParams::default();
    p_wide.laser_array_pitch_um = 18.0;

    let solver_tight = SuperradianceLaserSolver::new(p_tight);
    let solver_wide = SuperradianceLaserSolver::new(p_wide);

    assert!(
        solver_wide.compute_inter_mode_crosstalk_isolation_db()
            > solver_tight.compute_inter_mode_crosstalk_isolation_db(),
        "Wider laser array pitch must significantly increase crosstalk isolation"
    );
    assert!(
        solver_wide.compute_topological_protection_gap_mhz()
            > solver_tight.compute_topological_protection_gap_mhz(),
        "Wider laser array pitch suppresses parasitic fringing and widens protection gap"
    );
}
