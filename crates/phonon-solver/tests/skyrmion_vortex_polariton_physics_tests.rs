#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological skyrmion-vortex polariton networks and non-Clifford geometric braiding engines.

use phonon_models::skyrmion_vortex_polariton::SkyrmionVortexPolaritonParams;
use phonon_solver::skyrmion_vortex_polariton::SkyrmionVortexPolaritonSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SkyrmionVortexPolaritonParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        5.0,   // below 15.0 nm
        0.2,   // below 0.5 meV
    );
    assert_eq!(underflow.dzyaloshinskii_moriya_interaction_mev, 1.0);
    assert_eq!(underflow.superconducting_vortex_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.skyrmion_shuttling_velocity_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_drive_power_uw, 0.5);
    assert_eq!(underflow.polariton_core_radius_nm, 15.0);
    assert_eq!(underflow.magnetic_anisotropy_energy_mev, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = SkyrmionVortexPolaritonParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 40.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        250.0,  // above 160.0 nm
        35.0,   // above 25.0 meV
    );
    assert_eq!(overflow.dzyaloshinskii_moriya_interaction_mev, 35.0);
    assert_eq!(overflow.superconducting_vortex_gap_mev, 40.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.skyrmion_shuttling_velocity_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_drive_power_uw, 30.0);
    assert_eq!(overflow.polariton_core_radius_nm, 160.0);
    assert_eq!(overflow.magnetic_anisotropy_energy_mev, 25.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SkyrmionVortexPolaritonParams::default();
    assert_eq!(params.dzyaloshinskii_moriya_interaction_mev, 16.5);
    assert_eq!(params.superconducting_vortex_gap_mev, 20.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.6);
    assert_eq!(params.skyrmion_shuttling_velocity_m_per_s, 1300.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_drive_power_uw, 5.4);
    assert_eq!(params.polariton_core_radius_nm, 50.0);
    assert_eq!(params.magnetic_anisotropy_energy_mev, 8.5);

    let solver = SkyrmionVortexPolaritonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.gate_fidelity >= 0.9980,
        "Gate fidelity must be >= 0.9980, got {:.6}",
        metrics.gate_fidelity
    );
    assert!(
        metrics.polariton_state_retention_fraction >= 0.9970,
        "Polariton state retention fraction must be >= 0.9970, got {:.6}",
        metrics.polariton_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_polariton_crosstalk_isolation_db >= 54.0,
        "Inter-polariton crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.inter_polariton_crosstalk_isolation_db
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
fn test_dzyaloshinskii_moriya_interaction_scaling() {
    let mut low_dmi = SkyrmionVortexPolaritonParams::default();
    low_dmi.dzyaloshinskii_moriya_interaction_mev = 2.0;
    let mut high_dmi = SkyrmionVortexPolaritonParams::default();
    high_dmi.dzyaloshinskii_moriya_interaction_mev = 33.0;

    let low_solver = SkyrmionVortexPolaritonSolver::new(low_dmi);
    let high_solver = SkyrmionVortexPolaritonSolver::new(high_dmi);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Higher DMI must enhance gate fidelity"
    );
    assert!(
        high_solver.compute_polariton_state_retention_fraction()
            > low_solver.compute_polariton_state_retention_fraction(),
        "Higher DMI must boost polariton state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher DMI must widen topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher DMI must suppress dephasing rate"
    );
}

#[test]
fn test_superconducting_vortex_gap_scaling() {
    let mut low_gap = SkyrmionVortexPolaritonParams::default();
    low_gap.superconducting_vortex_gap_mev = 3.0;
    let mut high_gap = SkyrmionVortexPolaritonParams::default();
    high_gap.superconducting_vortex_gap_mev = 38.0;

    let low_solver = SkyrmionVortexPolaritonSolver::new(low_gap);
    let high_solver = SkyrmionVortexPolaritonSolver::new(high_gap);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Higher superconducting vortex gap must enhance gate fidelity"
    );
    assert!(
        high_solver.compute_polariton_state_retention_fraction()
            > low_solver.compute_polariton_state_retention_fraction(),
        "Higher superconducting vortex gap must boost polariton state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher superconducting vortex gap must widen topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher superconducting vortex gap must suppress dephasing rate"
    );
}

#[test]
fn test_magnetic_anisotropy_energy_scaling() {
    let mut low_ani = SkyrmionVortexPolaritonParams::default();
    low_ani.magnetic_anisotropy_energy_mev = 1.0;
    let mut high_ani = SkyrmionVortexPolaritonParams::default();
    high_ani.magnetic_anisotropy_energy_mev = 24.0;

    let low_solver = SkyrmionVortexPolaritonSolver::new(low_ani);
    let high_solver = SkyrmionVortexPolaritonSolver::new(high_ani);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher magnetic anisotropy must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_polariton_crosstalk_isolation_db()
            > low_solver.compute_inter_polariton_crosstalk_isolation_db(),
        "Higher magnetic anisotropy must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Higher magnetic anisotropy must increase gate fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher magnetic anisotropy must decrease dephasing rate"
    );
}

#[test]
fn test_polariton_core_radius_scaling() {
    let mut low_core = SkyrmionVortexPolaritonParams::default();
    low_core.polariton_core_radius_nm = 20.0;
    let mut high_core = SkyrmionVortexPolaritonParams::default();
    high_core.polariton_core_radius_nm = 150.0;

    let low_solver = SkyrmionVortexPolaritonSolver::new(low_core);
    let high_solver = SkyrmionVortexPolaritonSolver::new(high_core);

    assert!(
        high_solver.compute_inter_polariton_crosstalk_isolation_db()
            > low_solver.compute_inter_polariton_crosstalk_isolation_db(),
        "Larger polariton core radius must boost crosstalk isolation"
    );
    assert!(
        high_solver.compute_polariton_state_retention_fraction()
            > low_solver.compute_polariton_state_retention_fraction(),
        "Larger polariton core radius must improve retention fraction"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger polariton core radius must reduce dephasing rate"
    );
}

#[test]
fn test_skyrmion_shuttling_velocity_scaling() {
    let mut low_v = SkyrmionVortexPolaritonParams::default();
    low_v.skyrmion_shuttling_velocity_m_per_s = 300.0;
    let mut high_v = SkyrmionVortexPolaritonParams::default();
    high_v.skyrmion_shuttling_velocity_m_per_s = 2800.0;

    let low_solver = SkyrmionVortexPolaritonSolver::new(low_v);
    let high_solver = SkyrmionVortexPolaritonSolver::new(high_v);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Higher shuttling velocity must increase gate fidelity"
    );
    assert!(
        high_solver.compute_polariton_state_retention_fraction()
            > low_solver.compute_polariton_state_retention_fraction(),
        "Higher shuttling velocity must improve state retention fraction"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher shuttling velocity must suppress dephasing by shortening transit exposure"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_f = SkyrmionVortexPolaritonParams::default();
    low_f.acoustic_drive_frequency_ghz = 1.5;
    let mut high_f = SkyrmionVortexPolaritonParams::default();
    high_f.acoustic_drive_frequency_ghz = 11.0;

    let low_solver = SkyrmionVortexPolaritonSolver::new(low_f);
    let high_solver = SkyrmionVortexPolaritonSolver::new(high_f);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Higher acoustic drive frequency must increase gate fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher acoustic drive frequency must enhance topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic drive frequency must reduce dephasing"
    );
}

#[test]
fn test_microwave_drive_power_scaling() {
    let mut low_p = SkyrmionVortexPolaritonParams::default();
    low_p.microwave_drive_power_uw = 1.0;
    let mut high_p = SkyrmionVortexPolaritonParams::default();
    high_p.microwave_drive_power_uw = 28.0;

    let low_solver = SkyrmionVortexPolaritonSolver::new(low_p);
    let high_solver = SkyrmionVortexPolaritonSolver::new(high_p);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Higher microwave drive power must increase gate fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave drive power must improve topological gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave drive power must reduce dephasing"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_t = SkyrmionVortexPolaritonParams::default();
    low_t.cryogenic_temperature_mk = 2.0;
    let mut high_t = SkyrmionVortexPolaritonParams::default();
    high_t.cryogenic_temperature_mk = 48.0;

    let low_solver = SkyrmionVortexPolaritonSolver::new(low_t);
    let high_solver = SkyrmionVortexPolaritonSolver::new(high_t);

    assert!(
        low_solver.compute_gate_fidelity() > high_solver.compute_gate_fidelity(),
        "Lower temperature must enhance gate fidelity due to reduced thermal noise"
    );
    assert!(
        low_solver.compute_polariton_state_retention_fraction()
            > high_solver.compute_polariton_state_retention_fraction(),
        "Lower temperature must protect polariton state retention"
    );
    assert!(
        low_solver.compute_topological_protection_gap_mhz()
            > high_solver.compute_topological_protection_gap_mhz(),
        "Lower temperature must preserve a wider topological protection gap"
    );
    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower temperature must suppress thermal dephasing rate"
    );
}
