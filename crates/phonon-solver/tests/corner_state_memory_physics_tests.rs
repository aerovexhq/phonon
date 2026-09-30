#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological higher-order corner state quantum memory arrays and holonomic storage registers.

use phonon_models::corner_state_memory::CornerStateMemoryParams;
use phonon_solver::corner_state_memory::CornerStateMemorySolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = CornerStateMemoryParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.05,  // below 0.1 e
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.quadrupole_coupling_energy_mev, 1.0);
    assert_eq!(underflow.topological_corner_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.holonomic_drift_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_readout_power_uw, 0.5);
    assert_eq!(underflow.synthetic_octupole_charge_e, 0.1);
    assert_eq!(underflow.corner_cell_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = CornerStateMemoryParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        8.0,    // above 5.0 e
        35.0,   // above 20.0 um
    );
    assert_eq!(overflow.quadrupole_coupling_energy_mev, 35.0);
    assert_eq!(overflow.topological_corner_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.holonomic_drift_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_readout_power_uw, 30.0);
    assert_eq!(overflow.synthetic_octupole_charge_e, 5.0);
    assert_eq!(overflow.corner_cell_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = CornerStateMemoryParams::default();
    assert_eq!(params.quadrupole_coupling_energy_mev, 16.5);
    assert_eq!(params.topological_corner_gap_mev, 22.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.8);
    assert_eq!(params.holonomic_drift_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_readout_power_uw, 5.8);
    assert_eq!(params.synthetic_octupole_charge_e, 1.6);
    assert_eq!(params.corner_cell_pitch_um, 4.8);

    let solver = CornerStateMemorySolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.memory_fidelity >= 0.9980,
        "Memory fidelity must be >= 0.9980, got {:.6}",
        metrics.memory_fidelity
    );
    assert!(
        metrics.state_retention_fraction >= 0.9970,
        "State retention fraction must be >= 0.9970, got {:.6}",
        metrics.state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_cell_crosstalk_isolation_db >= 55.0,
        "Inter-cell crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_cell_crosstalk_isolation_db
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
fn test_quadrupole_coupling_energy_scaling() {
    let mut low_p = CornerStateMemoryParams::default();
    low_p.quadrupole_coupling_energy_mev = 2.0;

    let mut high_p = CornerStateMemoryParams::default();
    high_p.quadrupole_coupling_energy_mev = 34.0;

    let low_m = CornerStateMemorySolver::new(low_p).evaluate_metrics();
    let high_m = CornerStateMemorySolver::new(high_p).evaluate_metrics();

    assert!(high_m.memory_fidelity > low_m.memory_fidelity);
    assert!(high_m.state_retention_fraction > low_m.state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_cell_crosstalk_isolation_db > low_m.inter_cell_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_corner_gap_scaling() {
    let mut low_p = CornerStateMemoryParams::default();
    low_p.topological_corner_gap_mev = 3.0;

    let mut high_p = CornerStateMemoryParams::default();
    high_p.topological_corner_gap_mev = 43.0;

    let low_m = CornerStateMemorySolver::new(low_p).evaluate_metrics();
    let high_m = CornerStateMemorySolver::new(high_p).evaluate_metrics();

    assert!(high_m.memory_fidelity > low_m.memory_fidelity);
    assert!(high_m.state_retention_fraction > low_m.state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_cell_crosstalk_isolation_db > low_m.inter_cell_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = CornerStateMemoryParams::default();
    low_p.acoustic_drive_frequency_ghz = 1.5;

    let mut high_p = CornerStateMemoryParams::default();
    high_p.acoustic_drive_frequency_ghz = 11.5;

    let low_m = CornerStateMemorySolver::new(low_p).evaluate_metrics();
    let high_m = CornerStateMemorySolver::new(high_p).evaluate_metrics();

    assert!(high_m.memory_fidelity > low_m.memory_fidelity);
    assert!(high_m.state_retention_fraction > low_m.state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_cell_crosstalk_isolation_db > low_m.inter_cell_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_holonomic_drift_speed_scaling() {
    let mut low_p = CornerStateMemoryParams::default();
    low_p.holonomic_drift_speed_m_per_s = 300.0;

    let mut high_p = CornerStateMemoryParams::default();
    high_p.holonomic_drift_speed_m_per_s = 2900.0;

    let low_m = CornerStateMemorySolver::new(low_p).evaluate_metrics();
    let high_m = CornerStateMemorySolver::new(high_p).evaluate_metrics();

    assert!(high_m.memory_fidelity > low_m.memory_fidelity);
    assert!(high_m.state_retention_fraction > low_m.state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_cell_crosstalk_isolation_db > low_m.inter_cell_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_p = CornerStateMemoryParams::default();
    low_p.cryogenic_temperature_mk = 2.0;

    let mut high_p = CornerStateMemoryParams::default();
    high_p.cryogenic_temperature_mk = 48.0;

    let low_m = CornerStateMemorySolver::new(low_p).evaluate_metrics();
    let high_m = CornerStateMemorySolver::new(high_p).evaluate_metrics();

    // Higher temperature degrades performance and increases dephasing
    assert!(low_m.memory_fidelity > high_m.memory_fidelity);
    assert!(low_m.state_retention_fraction > high_m.state_retention_fraction);
    assert!(low_m.topological_protection_gap_mhz > high_m.topological_protection_gap_mhz);
    assert!(low_m.inter_cell_crosstalk_isolation_db > high_m.inter_cell_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz > low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_readout_power_scaling() {
    let mut low_p = CornerStateMemoryParams::default();
    low_p.microwave_readout_power_uw = 1.0;

    let mut high_p = CornerStateMemoryParams::default();
    high_p.microwave_readout_power_uw = 29.0;

    let low_m = CornerStateMemorySolver::new(low_p).evaluate_metrics();
    let high_m = CornerStateMemorySolver::new(high_p).evaluate_metrics();

    assert!(high_m.memory_fidelity > low_m.memory_fidelity);
    assert!(high_m.state_retention_fraction > low_m.state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_cell_crosstalk_isolation_db > low_m.inter_cell_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synthetic_octupole_charge_scaling() {
    let mut low_p = CornerStateMemoryParams::default();
    low_p.synthetic_octupole_charge_e = 0.3;

    let mut high_p = CornerStateMemoryParams::default();
    high_p.synthetic_octupole_charge_e = 4.8;

    let low_m = CornerStateMemorySolver::new(low_p).evaluate_metrics();
    let high_m = CornerStateMemorySolver::new(high_p).evaluate_metrics();

    assert!(high_m.memory_fidelity > low_m.memory_fidelity);
    assert!(high_m.state_retention_fraction > low_m.state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_cell_crosstalk_isolation_db > low_m.inter_cell_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_corner_cell_pitch_scaling() {
    let mut low_p = CornerStateMemoryParams::default();
    low_p.corner_cell_pitch_um = 1.0;

    let mut high_p = CornerStateMemoryParams::default();
    high_p.corner_cell_pitch_um = 19.0;

    let low_m = CornerStateMemorySolver::new(low_p).evaluate_metrics();
    let high_m = CornerStateMemorySolver::new(high_p).evaluate_metrics();

    assert!(high_m.memory_fidelity > low_m.memory_fidelity);
    assert!(high_m.state_retention_fraction > low_m.state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_cell_crosstalk_isolation_db > low_m.inter_cell_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
