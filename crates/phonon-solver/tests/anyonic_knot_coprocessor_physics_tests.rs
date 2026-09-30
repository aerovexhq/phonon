#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological anyonic knot invariant quantum co-processors and Chern-Simons
//! calculators.

use phonon_models::anyonic_knot_coprocessor::AnyonicKnotCoprocessorParams;
use phonon_solver::anyonic_knot_coprocessor::AnyonicKnotCoprocessorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AnyonicKnotCoprocessorParams::new(
        0.5,   // below 1.0 meV
        0.5,   // below 1.0
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        10.0,  // below 20.0 nm
        1.0,   // below 3.0
    );
    assert_eq!(underflow.braid_crossing_coupling_mev, 1.0);
    assert_eq!(underflow.chern_simons_level_k, 1.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.knot_braiding_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_interferometer_power_uw, 0.5);
    assert_eq!(underflow.anyon_link_closure_radius_nm, 20.0);
    assert_eq!(underflow.knot_complexity_crossings, 3.0);

    // Test values strictly above physical maximum bounds
    let overflow = AnyonicKnotCoprocessorParams::new(
        50.0,   // above 35.0 meV
        20.0,   // above 12.0
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        250.0,  // above 200.0 nm
        35.0,   // above 24.0
    );
    assert_eq!(overflow.braid_crossing_coupling_mev, 35.0);
    assert_eq!(overflow.chern_simons_level_k, 12.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.knot_braiding_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_interferometer_power_uw, 30.0);
    assert_eq!(overflow.anyon_link_closure_radius_nm, 200.0);
    assert_eq!(overflow.knot_complexity_crossings, 24.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AnyonicKnotCoprocessorParams::default();
    assert_eq!(params.braid_crossing_coupling_mev, 16.5);
    assert_eq!(params.chern_simons_level_k, 4.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.8);
    assert_eq!(params.knot_braiding_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_interferometer_power_uw, 5.8);
    assert_eq!(params.anyon_link_closure_radius_nm, 75.0);
    assert_eq!(params.knot_complexity_crossings, 8.0);

    let solver = AnyonicKnotCoprocessorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.knot_calculation_fidelity >= 0.9980,
        "Knot calculation fidelity must be >= 0.9980, got {:.6}",
        metrics.knot_calculation_fidelity
    );
    assert!(
        metrics.anyon_state_retention_fraction >= 0.9970,
        "Anyon state retention fraction must be >= 0.9970, got {:.6}",
        metrics.anyon_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_knot_crosstalk_isolation_db >= 54.0,
        "Inter-knot crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.inter_knot_crosstalk_isolation_db
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
fn test_braid_crossing_coupling_scaling() {
    let mut low_p = AnyonicKnotCoprocessorParams::default();
    low_p.braid_crossing_coupling_mev = 2.0;

    let mut high_p = AnyonicKnotCoprocessorParams::default();
    high_p.braid_crossing_coupling_mev = 34.0;

    let low_m = AnyonicKnotCoprocessorSolver::new(low_p).evaluate_metrics();
    let high_m = AnyonicKnotCoprocessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.knot_calculation_fidelity > low_m.knot_calculation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_knot_crosstalk_isolation_db > low_m.inter_knot_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_chern_simons_level_k_scaling() {
    let mut low_p = AnyonicKnotCoprocessorParams::default();
    low_p.chern_simons_level_k = 1.5;

    let mut high_p = AnyonicKnotCoprocessorParams::default();
    high_p.chern_simons_level_k = 11.5;

    let low_m = AnyonicKnotCoprocessorSolver::new(low_p).evaluate_metrics();
    let high_m = AnyonicKnotCoprocessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.knot_calculation_fidelity > low_m.knot_calculation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_knot_crosstalk_isolation_db > low_m.inter_knot_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = AnyonicKnotCoprocessorParams::default();
    low_p.acoustic_drive_frequency_ghz = 1.5;

    let mut high_p = AnyonicKnotCoprocessorParams::default();
    high_p.acoustic_drive_frequency_ghz = 11.5;

    let low_m = AnyonicKnotCoprocessorSolver::new(low_p).evaluate_metrics();
    let high_m = AnyonicKnotCoprocessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.knot_calculation_fidelity > low_m.knot_calculation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_knot_crosstalk_isolation_db > low_m.inter_knot_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_knot_braiding_speed_scaling() {
    let mut low_p = AnyonicKnotCoprocessorParams::default();
    low_p.knot_braiding_speed_m_per_s = 300.0;

    let mut high_p = AnyonicKnotCoprocessorParams::default();
    high_p.knot_braiding_speed_m_per_s = 2900.0;

    let low_m = AnyonicKnotCoprocessorSolver::new(low_p).evaluate_metrics();
    let high_m = AnyonicKnotCoprocessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.knot_calculation_fidelity > low_m.knot_calculation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_knot_crosstalk_isolation_db > low_m.inter_knot_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_p = AnyonicKnotCoprocessorParams::default();
    low_p.cryogenic_temperature_mk = 2.0;

    let mut high_p = AnyonicKnotCoprocessorParams::default();
    high_p.cryogenic_temperature_mk = 48.0;

    let low_m = AnyonicKnotCoprocessorSolver::new(low_p).evaluate_metrics();
    let high_m = AnyonicKnotCoprocessorSolver::new(high_p).evaluate_metrics();

    // Higher temperature degrades performance and increases dephasing
    assert!(low_m.knot_calculation_fidelity > high_m.knot_calculation_fidelity);
    assert!(low_m.anyon_state_retention_fraction > high_m.anyon_state_retention_fraction);
    assert!(low_m.topological_protection_gap_mhz > high_m.topological_protection_gap_mhz);
    assert!(low_m.inter_knot_crosstalk_isolation_db > high_m.inter_knot_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz > low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_interferometer_power_scaling() {
    let mut low_p = AnyonicKnotCoprocessorParams::default();
    low_p.microwave_interferometer_power_uw = 1.0;

    let mut high_p = AnyonicKnotCoprocessorParams::default();
    high_p.microwave_interferometer_power_uw = 29.0;

    let low_m = AnyonicKnotCoprocessorSolver::new(low_p).evaluate_metrics();
    let high_m = AnyonicKnotCoprocessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.knot_calculation_fidelity > low_m.knot_calculation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_knot_crosstalk_isolation_db > low_m.inter_knot_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_anyon_link_closure_radius_scaling() {
    let mut low_p = AnyonicKnotCoprocessorParams::default();
    low_p.anyon_link_closure_radius_nm = 25.0;

    let mut high_p = AnyonicKnotCoprocessorParams::default();
    high_p.anyon_link_closure_radius_nm = 195.0;

    let low_m = AnyonicKnotCoprocessorSolver::new(low_p).evaluate_metrics();
    let high_m = AnyonicKnotCoprocessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.knot_calculation_fidelity > low_m.knot_calculation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_knot_crosstalk_isolation_db > low_m.inter_knot_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_knot_complexity_crossings_scaling() {
    let mut low_p = AnyonicKnotCoprocessorParams::default();
    low_p.knot_complexity_crossings = 4.0;

    let mut high_p = AnyonicKnotCoprocessorParams::default();
    high_p.knot_complexity_crossings = 23.0;

    let low_m = AnyonicKnotCoprocessorSolver::new(low_p).evaluate_metrics();
    let high_m = AnyonicKnotCoprocessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.knot_calculation_fidelity > low_m.knot_calculation_fidelity);
    assert!(high_m.anyon_state_retention_fraction > low_m.anyon_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_knot_crosstalk_isolation_db > low_m.inter_knot_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
