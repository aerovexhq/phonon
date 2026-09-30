#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Molecular Spintronic Qubit Interface &
//! Diamond NV-Center Acoustic Transducer Engine.

use phonon_models::molecular_spintronics::MolecularSpintronicsParams;
use phonon_solver::molecular_spintronics::MolecularSpintronicsSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = MolecularSpintronicsParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.spintronic_coupling_mev, 1.0);
    assert_eq!(underflow.topological_spintronic_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.spin_acoustic_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_nv_cavity_factor, 1.0);
    assert_eq!(underflow.spintronic_cell_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = MolecularSpintronicsParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.spintronic_coupling_mev, 35.0);
    assert_eq!(overflow.topological_spintronic_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.spin_acoustic_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_nv_cavity_factor, 8.0);
    assert_eq!(overflow.spintronic_cell_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = MolecularSpintronicsParams::default();
    assert_eq!(params.spintronic_coupling_mev, 22.0);
    assert_eq!(params.topological_spintronic_gap_mev, 28.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 8.5);
    assert_eq!(params.spin_acoustic_dispatch_speed_m_per_s, 1900.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 8.5);
    assert_eq!(params.synthetic_nv_cavity_factor, 4.0);
    assert_eq!(params.spintronic_cell_pitch_um, 7.5);

    let solver = MolecularSpintronicsSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.transduction_fidelity >= 0.9980,
        "Transduction fidelity must be >= 0.9980, got {:.6}",
        metrics.transduction_fidelity
    );
    assert!(
        metrics.spin_state_retention_fraction >= 0.9970,
        "Spin state retention fraction must be >= 0.9970, got {:.6}",
        metrics.spin_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_qubit_crosstalk_isolation_db >= 55.0,
        "Inter-qubit crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_qubit_crosstalk_isolation_db
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
fn test_coupling_scaling() {
    let mut p_low = MolecularSpintronicsParams::default();
    p_low.spintronic_coupling_mev = 2.0;
    let mut p_high = MolecularSpintronicsParams::default();
    p_high.spintronic_coupling_mev = 34.0;

    let solver_low = MolecularSpintronicsSolver::new(p_low);
    let solver_high = MolecularSpintronicsSolver::new(p_high);

    assert!(
        solver_high.compute_transduction_fidelity() > solver_low.compute_transduction_fidelity(),
        "Higher spintronic coupling energy should enhance transduction fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher spintronic coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher spintronic coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_gap_scaling() {
    let mut p_low = MolecularSpintronicsParams::default();
    p_low.topological_spintronic_gap_mev = 3.0;
    let mut p_high = MolecularSpintronicsParams::default();
    p_high.topological_spintronic_gap_mev = 44.0;

    let solver_low = MolecularSpintronicsSolver::new(p_low);
    let solver_high = MolecularSpintronicsSolver::new(p_high);

    assert!(
        solver_high.compute_spin_state_retention_fraction()
            > solver_low.compute_spin_state_retention_fraction(),
        "Wider topological spintronic gap should improve spin state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological gap should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Wider topological gap should reduce dephasing rate"
    );
}

#[test]
fn test_acoustic_frequency_scaling() {
    let mut p_low = MolecularSpintronicsParams::default();
    p_low.acoustic_drive_frequency_ghz = 2.0;
    let mut p_high = MolecularSpintronicsParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.0;

    let solver_low = MolecularSpintronicsSolver::new(p_low);
    let solver_high = MolecularSpintronicsSolver::new(p_high);

    assert!(
        solver_high.compute_transduction_fidelity() > solver_low.compute_transduction_fidelity(),
        "Higher acoustic drive frequency should enhance transduction fidelity"
    );
    assert!(
        solver_high.compute_inter_qubit_crosstalk_isolation_db()
            > solver_low.compute_inter_qubit_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should increase isolation"
    );
}

#[test]
fn test_spin_speed_scaling() {
    let mut p_low = MolecularSpintronicsParams::default();
    p_low.spin_acoustic_dispatch_speed_m_per_s = 300.0;
    let mut p_high = MolecularSpintronicsParams::default();
    p_high.spin_acoustic_dispatch_speed_m_per_s = 2900.0;

    let solver_low = MolecularSpintronicsSolver::new(p_low);
    let solver_high = MolecularSpintronicsSolver::new(p_high);

    assert!(
        solver_high.compute_transduction_fidelity() > solver_low.compute_transduction_fidelity(),
        "Faster spin-acoustic dispatch speed should increase fidelity"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Faster spin-acoustic dispatch speed should reduce dephasing rate"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = MolecularSpintronicsParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = MolecularSpintronicsParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = MolecularSpintronicsSolver::new(p_cold);
    let solver_warm = MolecularSpintronicsSolver::new(p_warm);

    assert!(
        solver_cold.compute_transduction_fidelity() > solver_warm.compute_transduction_fidelity(),
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
    let mut p_low = MolecularSpintronicsParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = MolecularSpintronicsParams::default();
    p_high.microwave_probe_power_uw = 28.0;

    let solver_low = MolecularSpintronicsSolver::new(p_low);
    let solver_high = MolecularSpintronicsSolver::new(p_high);

    assert!(
        solver_high.compute_spin_state_retention_fraction()
            > solver_low.compute_spin_state_retention_fraction(),
        "Optimized microwave probe power should enhance spin state retention fraction"
    );
    assert!(
        solver_high.compute_inter_qubit_crosstalk_isolation_db()
            > solver_low.compute_inter_qubit_crosstalk_isolation_db(),
        "Higher probe power within linear regime should improve SNR and isolation"
    );
}

#[test]
fn test_synthetic_nv_cavity_scaling() {
    let mut p_low = MolecularSpintronicsParams::default();
    p_low.synthetic_nv_cavity_factor = 1.5;
    let mut p_high = MolecularSpintronicsParams::default();
    p_high.synthetic_nv_cavity_factor = 7.5;

    let solver_low = MolecularSpintronicsSolver::new(p_low);
    let solver_high = MolecularSpintronicsSolver::new(p_high);

    assert!(
        solver_high.compute_transduction_fidelity() > solver_low.compute_transduction_fidelity(),
        "Higher synthetic NV cavity factor should improve transduction fidelity"
    );
    assert!(
        solver_high.compute_inter_qubit_crosstalk_isolation_db()
            > solver_low.compute_inter_qubit_crosstalk_isolation_db(),
        "Higher synthetic NV cavity factor should boost crosstalk isolation"
    );
}

#[test]
fn test_spintronic_cell_pitch_scaling() {
    let mut p_tight = MolecularSpintronicsParams::default();
    p_tight.spintronic_cell_pitch_um = 1.0;
    let mut p_wide = MolecularSpintronicsParams::default();
    p_wide.spintronic_cell_pitch_um = 18.0;

    let solver_tight = MolecularSpintronicsSolver::new(p_tight);
    let solver_wide = MolecularSpintronicsSolver::new(p_wide);

    assert!(
        solver_wide.compute_inter_qubit_crosstalk_isolation_db()
            > solver_tight.compute_inter_qubit_crosstalk_isolation_db(),
        "Wider spintronic cell pitch must significantly increase crosstalk isolation"
    );
    assert!(
        solver_wide.compute_topological_protection_gap_mhz()
            > solver_tight.compute_topological_protection_gap_mhz(),
        "Wider spintronic cell pitch suppresses parasitic fringing and widens protection gap"
    );
}
