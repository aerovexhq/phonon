#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Skyrmionic-Phononic Memory Lattice &
//! Chiral Domain Wall Track Engine.

use phonon_models::skyrmionic_memory::SkyrmionicMemoryParams;
use phonon_solver::skyrmionic_memory::SkyrmionicMemorySolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SkyrmionicMemoryParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.skyrmion_coupling_mev, 1.0);
    assert_eq!(underflow.topological_skyrmion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.racetrack_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_tracks_factor, 1.0);
    assert_eq!(underflow.track_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = SkyrmionicMemoryParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.skyrmion_coupling_mev, 35.0);
    assert_eq!(overflow.topological_skyrmion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.racetrack_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_tracks_factor, 8.0);
    assert_eq!(overflow.track_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SkyrmionicMemoryParams::default();
    assert_eq!(params.skyrmion_coupling_mev, 24.5);
    assert_eq!(params.topological_skyrmion_gap_mev, 30.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 9.8);
    assert_eq!(params.racetrack_dispatch_speed_m_per_s, 2150.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 9.8);
    assert_eq!(params.synthetic_tracks_factor, 4.0);
    assert_eq!(params.track_pitch_um, 8.8);

    let solver = SkyrmionicMemorySolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.nucleation_fidelity >= 0.9980,
        "Nucleation fidelity must be >= 0.9980, got {:.6}",
        metrics.nucleation_fidelity
    );
    assert!(
        metrics.skyrmion_state_retention_fraction >= 0.9970,
        "Skyrmion state retention fraction must be >= 0.9970, got {:.6}",
        metrics.skyrmion_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_track_crosstalk_isolation_db >= 55.0,
        "Inter-track crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_track_crosstalk_isolation_db
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
fn test_skyrmion_coupling_scaling() {
    let mut p_low = SkyrmionicMemoryParams::default();
    p_low.skyrmion_coupling_mev = 2.0;
    let mut p_high = SkyrmionicMemoryParams::default();
    p_high.skyrmion_coupling_mev = 34.0;

    let solver_low = SkyrmionicMemorySolver::new(p_low);
    let solver_high = SkyrmionicMemorySolver::new(p_high);

    assert!(
        solver_high.compute_nucleation_fidelity() > solver_low.compute_nucleation_fidelity(),
        "Higher skyrmion coupling energy should enhance nucleation fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher skyrmion coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher skyrmion coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_skyrmion_gap_scaling() {
    let mut p_low = SkyrmionicMemoryParams::default();
    p_low.topological_skyrmion_gap_mev = 3.0;
    let mut p_high = SkyrmionicMemoryParams::default();
    p_high.topological_skyrmion_gap_mev = 44.0;

    let solver_low = SkyrmionicMemorySolver::new(p_low);
    let solver_high = SkyrmionicMemorySolver::new(p_high);

    assert!(
        solver_high.compute_skyrmion_state_retention_fraction()
            > solver_low.compute_skyrmion_state_retention_fraction(),
        "Wider topological skyrmion gap should improve retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological skyrmion gap should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Wider topological skyrmion gap should reduce dephasing rate"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = SkyrmionicMemoryParams::default();
    p_low.acoustic_drive_frequency_ghz = 2.0;
    let mut p_high = SkyrmionicMemoryParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.0;

    let solver_low = SkyrmionicMemorySolver::new(p_low);
    let solver_high = SkyrmionicMemorySolver::new(p_high);

    assert!(
        solver_high.compute_nucleation_fidelity() > solver_low.compute_nucleation_fidelity(),
        "Higher acoustic drive frequency should enhance nucleation fidelity"
    );
    assert!(
        solver_high.compute_inter_track_crosstalk_isolation_db()
            > solver_low.compute_inter_track_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should enhance crosstalk isolation"
    );
}

#[test]
fn test_racetrack_dispatch_speed_scaling() {
    let mut p_low = SkyrmionicMemoryParams::default();
    p_low.racetrack_dispatch_speed_m_per_s = 300.0;
    let mut p_high = SkyrmionicMemoryParams::default();
    p_high.racetrack_dispatch_speed_m_per_s = 2900.0;

    let solver_low = SkyrmionicMemorySolver::new(p_low);
    let solver_high = SkyrmionicMemorySolver::new(p_high);

    assert!(
        solver_high.compute_nucleation_fidelity() > solver_low.compute_nucleation_fidelity(),
        "Higher racetrack dispatch speed should increase nucleation fidelity"
    );
    assert!(
        solver_high.compute_skyrmion_state_retention_fraction()
            > solver_low.compute_skyrmion_state_retention_fraction(),
        "Higher racetrack dispatch speed should increase state retention fraction"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = SkyrmionicMemoryParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = SkyrmionicMemoryParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = SkyrmionicMemorySolver::new(p_cold);
    let solver_warm = SkyrmionicMemorySolver::new(p_warm);

    assert!(
        solver_cold.compute_nucleation_fidelity() > solver_warm.compute_nucleation_fidelity(),
        "Lower cryogenic temperature should improve nucleation fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = SkyrmionicMemoryParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = SkyrmionicMemoryParams::default();
    p_high.microwave_probe_power_uw = 28.0;

    let solver_low = SkyrmionicMemorySolver::new(p_low);
    let solver_high = SkyrmionicMemorySolver::new(p_high);

    assert!(
        solver_high.compute_nucleation_fidelity() > solver_low.compute_nucleation_fidelity(),
        "Higher microwave probe power should enhance nucleation fidelity"
    );
    assert!(
        solver_high.compute_skyrmion_state_retention_fraction()
            > solver_low.compute_skyrmion_state_retention_fraction(),
        "Higher microwave probe power should enhance state retention fraction"
    );
}

#[test]
fn test_synthetic_tracks_scaling() {
    let mut p_low = SkyrmionicMemoryParams::default();
    p_low.synthetic_tracks_factor = 1.5;
    let mut p_high = SkyrmionicMemoryParams::default();
    p_high.synthetic_tracks_factor = 7.5;

    let solver_low = SkyrmionicMemorySolver::new(p_low);
    let solver_high = SkyrmionicMemorySolver::new(p_high);

    assert!(
        solver_high.compute_inter_track_crosstalk_isolation_db()
            > solver_low.compute_inter_track_crosstalk_isolation_db(),
        "Higher synthetic tracks factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher synthetic tracks factor should increase protection gap"
    );
}

#[test]
fn test_track_pitch_scaling() {
    let mut p_narrow = SkyrmionicMemoryParams::default();
    p_narrow.track_pitch_um = 1.0;
    let mut p_wide = SkyrmionicMemoryParams::default();
    p_wide.track_pitch_um = 18.0;

    let solver_narrow = SkyrmionicMemorySolver::new(p_narrow);
    let solver_wide = SkyrmionicMemorySolver::new(p_wide);

    assert!(
        solver_wide.compute_inter_track_crosstalk_isolation_db()
            > solver_narrow.compute_inter_track_crosstalk_isolation_db(),
        "Larger track pitch should improve inter-track crosstalk isolation"
    );
    assert!(
        solver_wide.compute_topological_mode_dephasing_rate_hz()
            < solver_narrow.compute_topological_mode_dephasing_rate_hz(),
        "Larger track pitch should reduce dephasing rate"
    );
}
