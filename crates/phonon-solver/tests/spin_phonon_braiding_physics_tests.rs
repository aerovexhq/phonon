#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological quantum error-mitigating spin-phonon braiding engines.

use phonon_models::spin_phonon_braiding::SpinPhononBraidingParams;
use phonon_solver::spin_phonon_braiding::SpinPhononBraidingSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SpinPhononBraidingParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.spin_phonon_coupling_mev, 1.0);
    assert_eq!(underflow.topological_pairing_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.braiding_drift_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_decoupling_power_uw, 0.5);
    assert_eq!(underflow.error_mitigation_order, 1.0);
    assert_eq!(underflow.spin_defect_separation_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = SpinPhononBraidingParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        10.0,   // above 8.0
        35.0,   // above 20.0 um
    );
    assert_eq!(overflow.spin_phonon_coupling_mev, 35.0);
    assert_eq!(overflow.topological_pairing_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.braiding_drift_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_decoupling_power_uw, 30.0);
    assert_eq!(overflow.error_mitigation_order, 8.0);
    assert_eq!(overflow.spin_defect_separation_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SpinPhononBraidingParams::default();
    assert_eq!(params.spin_phonon_coupling_mev, 16.5);
    assert_eq!(params.topological_pairing_gap_mev, 22.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.8);
    assert_eq!(params.braiding_drift_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_decoupling_power_uw, 5.8);
    assert_eq!(params.error_mitigation_order, 4.0);
    assert_eq!(params.spin_defect_separation_um, 4.8);

    let solver = SpinPhononBraidingSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.gate_fidelity >= 0.9980,
        "Gate fidelity must be >= 0.9980, got {:.6}",
        metrics.gate_fidelity
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
        metrics.inter_qubit_crosstalk_isolation_db >= 54.0,
        "Inter-qubit crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
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
fn test_spin_phonon_coupling_scaling() {
    let mut low_p = SpinPhononBraidingParams::default();
    low_p.spin_phonon_coupling_mev = 2.0;

    let mut high_p = SpinPhononBraidingParams::default();
    high_p.spin_phonon_coupling_mev = 34.0;

    let low_m = SpinPhononBraidingSolver::new(low_p).evaluate_metrics();
    let high_m = SpinPhononBraidingSolver::new(high_p).evaluate_metrics();

    assert!(high_m.gate_fidelity > low_m.gate_fidelity);
    assert!(high_m.anyonic_state_retention_fraction > low_m.anyonic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_qubit_crosstalk_isolation_db > low_m.inter_qubit_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_pairing_gap_scaling() {
    let mut low_p = SpinPhononBraidingParams::default();
    low_p.topological_pairing_gap_mev = 3.0;

    let mut high_p = SpinPhononBraidingParams::default();
    high_p.topological_pairing_gap_mev = 43.0;

    let low_m = SpinPhononBraidingSolver::new(low_p).evaluate_metrics();
    let high_m = SpinPhononBraidingSolver::new(high_p).evaluate_metrics();

    assert!(high_m.gate_fidelity > low_m.gate_fidelity);
    assert!(high_m.anyonic_state_retention_fraction > low_m.anyonic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_qubit_crosstalk_isolation_db > low_m.inter_qubit_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = SpinPhononBraidingParams::default();
    low_p.acoustic_drive_frequency_ghz = 1.5;

    let mut high_p = SpinPhononBraidingParams::default();
    high_p.acoustic_drive_frequency_ghz = 11.5;

    let low_m = SpinPhononBraidingSolver::new(low_p).evaluate_metrics();
    let high_m = SpinPhononBraidingSolver::new(high_p).evaluate_metrics();

    assert!(high_m.gate_fidelity > low_m.gate_fidelity);
    assert!(high_m.anyonic_state_retention_fraction > low_m.anyonic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_qubit_crosstalk_isolation_db > low_m.inter_qubit_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_braiding_drift_speed_scaling() {
    let mut low_p = SpinPhononBraidingParams::default();
    low_p.braiding_drift_speed_m_per_s = 300.0;

    let mut high_p = SpinPhononBraidingParams::default();
    high_p.braiding_drift_speed_m_per_s = 2900.0;

    let low_m = SpinPhononBraidingSolver::new(low_p).evaluate_metrics();
    let high_m = SpinPhononBraidingSolver::new(high_p).evaluate_metrics();

    assert!(high_m.gate_fidelity > low_m.gate_fidelity);
    assert!(high_m.anyonic_state_retention_fraction > low_m.anyonic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_qubit_crosstalk_isolation_db > low_m.inter_qubit_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_p = SpinPhononBraidingParams::default();
    low_p.cryogenic_temperature_mk = 2.0;

    let mut high_p = SpinPhononBraidingParams::default();
    high_p.cryogenic_temperature_mk = 48.0;

    let low_m = SpinPhononBraidingSolver::new(low_p).evaluate_metrics();
    let high_m = SpinPhononBraidingSolver::new(high_p).evaluate_metrics();

    // Higher temperature degrades performance and increases dephasing
    assert!(low_m.gate_fidelity > high_m.gate_fidelity);
    assert!(low_m.anyonic_state_retention_fraction > high_m.anyonic_state_retention_fraction);
    assert!(low_m.topological_protection_gap_mhz > high_m.topological_protection_gap_mhz);
    assert!(low_m.inter_qubit_crosstalk_isolation_db > high_m.inter_qubit_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz > low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_decoupling_power_scaling() {
    let mut low_p = SpinPhononBraidingParams::default();
    low_p.microwave_decoupling_power_uw = 1.0;

    let mut high_p = SpinPhononBraidingParams::default();
    high_p.microwave_decoupling_power_uw = 29.0;

    let low_m = SpinPhononBraidingSolver::new(low_p).evaluate_metrics();
    let high_m = SpinPhononBraidingSolver::new(high_p).evaluate_metrics();

    assert!(high_m.gate_fidelity > low_m.gate_fidelity);
    assert!(high_m.anyonic_state_retention_fraction > low_m.anyonic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_qubit_crosstalk_isolation_db > low_m.inter_qubit_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_error_mitigation_order_scaling() {
    let mut low_p = SpinPhononBraidingParams::default();
    low_p.error_mitigation_order = 1.5;

    let mut high_p = SpinPhononBraidingParams::default();
    high_p.error_mitigation_order = 7.5;

    let low_m = SpinPhononBraidingSolver::new(low_p).evaluate_metrics();
    let high_m = SpinPhononBraidingSolver::new(high_p).evaluate_metrics();

    assert!(high_m.gate_fidelity > low_m.gate_fidelity);
    assert!(high_m.anyonic_state_retention_fraction > low_m.anyonic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_qubit_crosstalk_isolation_db > low_m.inter_qubit_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_spin_defect_separation_scaling() {
    let mut low_p = SpinPhononBraidingParams::default();
    low_p.spin_defect_separation_um = 1.0;

    let mut high_p = SpinPhononBraidingParams::default();
    high_p.spin_defect_separation_um = 19.0;

    let low_m = SpinPhononBraidingSolver::new(low_p).evaluate_metrics();
    let high_m = SpinPhononBraidingSolver::new(high_p).evaluate_metrics();

    assert!(high_m.gate_fidelity > low_m.gate_fidelity);
    assert!(high_m.anyonic_state_retention_fraction > low_m.anyonic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_qubit_crosstalk_isolation_db > low_m.inter_qubit_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
