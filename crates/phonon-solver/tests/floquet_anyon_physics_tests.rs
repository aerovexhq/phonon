#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Floquet-Engineered Non-Abelian Anyon
//! Weaving Fabric & Fractional Quantum Hall Acoustic Engine.

use phonon_models::floquet_anyon::FloquetAnyonParams;
use phonon_solver::floquet_anyon::FloquetAnyonSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FloquetAnyonParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.floquet_coupling_mev, 1.0);
    assert_eq!(underflow.topological_braiding_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.weaving_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_flux_quantum_factor, 1.0);
    assert_eq!(underflow.anyon_lattice_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = FloquetAnyonParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.floquet_coupling_mev, 35.0);
    assert_eq!(overflow.topological_braiding_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.weaving_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_flux_quantum_factor, 8.0);
    assert_eq!(overflow.anyon_lattice_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FloquetAnyonParams::default();
    assert_eq!(params.floquet_coupling_mev, 24.0);
    assert_eq!(params.topological_braiding_gap_mev, 30.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 9.5);
    assert_eq!(params.weaving_dispatch_speed_m_per_s, 2100.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 9.5);
    assert_eq!(params.synthetic_flux_quantum_factor, 4.0);
    assert_eq!(params.anyon_lattice_pitch_um, 8.5);

    let solver = FloquetAnyonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.braiding_fidelity >= 0.9980,
        "Braiding fidelity must be >= 0.9980, got {:.6}",
        metrics.braiding_fidelity
    );
    assert!(
        metrics.anyonic_state_retention_fraction >= 0.9970,
        "Anyonic state retention fraction must be >= 0.9970, got {:.6}",
        metrics.anyonic_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_braid_crosstalk_isolation_db >= 55.0,
        "Inter-braid crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_braid_crosstalk_isolation_db
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
fn test_floquet_coupling_scaling() {
    let mut p_low = FloquetAnyonParams::default();
    p_low.floquet_coupling_mev = 2.0;
    let mut p_high = FloquetAnyonParams::default();
    p_high.floquet_coupling_mev = 34.0;

    let solver_low = FloquetAnyonSolver::new(p_low);
    let solver_high = FloquetAnyonSolver::new(p_high);

    assert!(
        solver_high.compute_braiding_fidelity() > solver_low.compute_braiding_fidelity(),
        "Higher Floquet coupling energy should enhance braiding fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher Floquet coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher Floquet coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_braiding_gap_scaling() {
    let mut p_low = FloquetAnyonParams::default();
    p_low.topological_braiding_gap_mev = 3.0;
    let mut p_high = FloquetAnyonParams::default();
    p_high.topological_braiding_gap_mev = 44.0;

    let solver_low = FloquetAnyonSolver::new(p_low);
    let solver_high = FloquetAnyonSolver::new(p_high);

    assert!(
        solver_high.compute_anyonic_state_retention_fraction()
            > solver_low.compute_anyonic_state_retention_fraction(),
        "Wider topological braiding gap should improve anyonic state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological braiding gap should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Wider topological braiding gap should reduce dephasing rate"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = FloquetAnyonParams::default();
    p_low.acoustic_drive_frequency_ghz = 2.0;
    let mut p_high = FloquetAnyonParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.0;

    let solver_low = FloquetAnyonSolver::new(p_low);
    let solver_high = FloquetAnyonSolver::new(p_high);

    assert!(
        solver_high.compute_braiding_fidelity() > solver_low.compute_braiding_fidelity(),
        "Higher acoustic drive frequency should enhance braiding fidelity"
    );
    assert!(
        solver_high.compute_inter_braid_crosstalk_isolation_db()
            > solver_low.compute_inter_braid_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should increase crosstalk isolation"
    );
}

#[test]
fn test_weaving_dispatch_speed_scaling() {
    let mut p_low = FloquetAnyonParams::default();
    p_low.weaving_dispatch_speed_m_per_s = 300.0;
    let mut p_high = FloquetAnyonParams::default();
    p_high.weaving_dispatch_speed_m_per_s = 2900.0;

    let solver_low = FloquetAnyonSolver::new(p_low);
    let solver_high = FloquetAnyonSolver::new(p_high);

    assert!(
        solver_high.compute_braiding_fidelity() > solver_low.compute_braiding_fidelity(),
        "Faster weaving dispatch speed should increase fidelity"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Faster weaving dispatch speed should reduce dephasing rate"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = FloquetAnyonParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = FloquetAnyonParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = FloquetAnyonSolver::new(p_cold);
    let solver_warm = FloquetAnyonSolver::new(p_warm);

    assert!(
        solver_cold.compute_braiding_fidelity() > solver_warm.compute_braiding_fidelity(),
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
    let mut p_low = FloquetAnyonParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = FloquetAnyonParams::default();
    p_high.microwave_probe_power_uw = 28.0;

    let solver_low = FloquetAnyonSolver::new(p_low);
    let solver_high = FloquetAnyonSolver::new(p_high);

    assert!(
        solver_high.compute_anyonic_state_retention_fraction()
            > solver_low.compute_anyonic_state_retention_fraction(),
        "Optimized microwave probe power should enhance anyonic state retention fraction"
    );
    assert!(
        solver_high.compute_inter_braid_crosstalk_isolation_db()
            > solver_low.compute_inter_braid_crosstalk_isolation_db(),
        "Higher probe power within linear regime should improve SNR and isolation"
    );
}

#[test]
fn test_synthetic_flux_quantum_factor_scaling() {
    let mut p_low = FloquetAnyonParams::default();
    p_low.synthetic_flux_quantum_factor = 1.5;
    let mut p_high = FloquetAnyonParams::default();
    p_high.synthetic_flux_quantum_factor = 7.5;

    let solver_low = FloquetAnyonSolver::new(p_low);
    let solver_high = FloquetAnyonSolver::new(p_high);

    assert!(
        solver_high.compute_braiding_fidelity() > solver_low.compute_braiding_fidelity(),
        "Higher synthetic flux quantum factor should improve braiding fidelity"
    );
    assert!(
        solver_high.compute_inter_braid_crosstalk_isolation_db()
            > solver_low.compute_inter_braid_crosstalk_isolation_db(),
        "Higher synthetic flux quantum factor should boost crosstalk isolation"
    );
}

#[test]
fn test_anyon_lattice_pitch_scaling() {
    let mut p_tight = FloquetAnyonParams::default();
    p_tight.anyon_lattice_pitch_um = 1.0;
    let mut p_wide = FloquetAnyonParams::default();
    p_wide.anyon_lattice_pitch_um = 18.0;

    let solver_tight = FloquetAnyonSolver::new(p_tight);
    let solver_wide = FloquetAnyonSolver::new(p_wide);

    assert!(
        solver_wide.compute_inter_braid_crosstalk_isolation_db()
            > solver_tight.compute_inter_braid_crosstalk_isolation_db(),
        "Wider anyon pitch must significantly increase crosstalk isolation"
    );
    assert!(
        solver_wide.compute_topological_protection_gap_mhz()
            > solver_tight.compute_topological_protection_gap_mhz(),
        "Wider anyon pitch suppresses parasitic fringing and widens protection gap"
    );
}
