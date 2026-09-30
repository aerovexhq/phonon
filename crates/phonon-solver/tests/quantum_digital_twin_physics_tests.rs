#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Quantum Digital Twin Micro-Architecture
//! Simulator & Sub-System Co-Emulation Fabric.

use phonon_models::quantum_digital_twin::QuantumDigitalTwinParams;
use phonon_solver::quantum_digital_twin::QuantumDigitalTwinSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = QuantumDigitalTwinParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.coemulation_coupling_mev, 1.0);
    assert_eq!(underflow.topological_emulation_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.instruction_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_coemulation_cores_factor, 1.0);
    assert_eq!(underflow.bus_interconnect_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = QuantumDigitalTwinParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.coemulation_coupling_mev, 35.0);
    assert_eq!(overflow.topological_emulation_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.instruction_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_coemulation_cores_factor, 8.0);
    assert_eq!(overflow.bus_interconnect_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = QuantumDigitalTwinParams::default();
    assert_eq!(params.coemulation_coupling_mev, 20.5);
    assert_eq!(params.topological_emulation_gap_mev, 26.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 7.8);
    assert_eq!(params.instruction_dispatch_speed_m_per_s, 1750.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 7.8);
    assert_eq!(params.synthetic_coemulation_cores_factor, 4.0);
    assert_eq!(params.bus_interconnect_pitch_um, 6.8);

    let solver = QuantumDigitalTwinSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.coemulation_fidelity >= 0.9980,
        "Co-emulation fidelity must be >= 0.9980, got {:.6}",
        metrics.coemulation_fidelity
    );
    assert!(
        metrics.quantum_bus_state_retention_fraction >= 0.9970,
        "Quantum bus state retention fraction must be >= 0.9970, got {:.6}",
        metrics.quantum_bus_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_core_crosstalk_isolation_db >= 55.0,
        "Inter-core crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_core_crosstalk_isolation_db
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
fn test_coemulation_coupling_scaling() {
    let mut low_p = QuantumDigitalTwinParams::default();
    low_p.coemulation_coupling_mev = 2.0;
    let mut high_p = QuantumDigitalTwinParams::default();
    high_p.coemulation_coupling_mev = 30.0;

    let low_m = QuantumDigitalTwinSolver::new(low_p).evaluate_metrics();
    let high_m = QuantumDigitalTwinSolver::new(high_p).evaluate_metrics();

    assert!(high_m.coemulation_fidelity > low_m.coemulation_fidelity);
    assert!(high_m.quantum_bus_state_retention_fraction > low_m.quantum_bus_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_core_crosstalk_isolation_db > low_m.inter_core_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_emulation_gap_scaling() {
    let mut low_p = QuantumDigitalTwinParams::default();
    low_p.topological_emulation_gap_mev = 3.0;
    let mut high_p = QuantumDigitalTwinParams::default();
    high_p.topological_emulation_gap_mev = 40.0;

    let low_m = QuantumDigitalTwinSolver::new(low_p).evaluate_metrics();
    let high_m = QuantumDigitalTwinSolver::new(high_p).evaluate_metrics();

    assert!(high_m.coemulation_fidelity > low_m.coemulation_fidelity);
    assert!(high_m.quantum_bus_state_retention_fraction > low_m.quantum_bus_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_core_crosstalk_isolation_db > low_m.inter_core_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = QuantumDigitalTwinParams::default();
    low_p.acoustic_drive_frequency_ghz = 2.0;
    let mut high_p = QuantumDigitalTwinParams::default();
    high_p.acoustic_drive_frequency_ghz = 10.0;

    let low_m = QuantumDigitalTwinSolver::new(low_p).evaluate_metrics();
    let high_m = QuantumDigitalTwinSolver::new(high_p).evaluate_metrics();

    assert!(high_m.coemulation_fidelity > low_m.coemulation_fidelity);
    assert!(high_m.quantum_bus_state_retention_fraction > low_m.quantum_bus_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_core_crosstalk_isolation_db > low_m.inter_core_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_instruction_dispatch_speed_scaling() {
    let mut low_p = QuantumDigitalTwinParams::default();
    low_p.instruction_dispatch_speed_m_per_s = 300.0;
    let mut high_p = QuantumDigitalTwinParams::default();
    high_p.instruction_dispatch_speed_m_per_s = 2800.0;

    let low_m = QuantumDigitalTwinSolver::new(low_p).evaluate_metrics();
    let high_m = QuantumDigitalTwinSolver::new(high_p).evaluate_metrics();

    assert!(high_m.coemulation_fidelity > low_m.coemulation_fidelity);
    assert!(high_m.quantum_bus_state_retention_fraction > low_m.quantum_bus_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_core_crosstalk_isolation_db > low_m.inter_core_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold_p = QuantumDigitalTwinParams::default();
    cold_p.cryogenic_temperature_mk = 2.0;
    let mut warm_p = QuantumDigitalTwinParams::default();
    warm_p.cryogenic_temperature_mk = 45.0;

    let cold_m = QuantumDigitalTwinSolver::new(cold_p).evaluate_metrics();
    let warm_m = QuantumDigitalTwinSolver::new(warm_p).evaluate_metrics();

    assert!(cold_m.coemulation_fidelity > warm_m.coemulation_fidelity);
    assert!(cold_m.quantum_bus_state_retention_fraction > warm_m.quantum_bus_state_retention_fraction);
    assert!(cold_m.topological_protection_gap_mhz > warm_m.topological_protection_gap_mhz);
    assert!(cold_m.inter_core_crosstalk_isolation_db > warm_m.inter_core_crosstalk_isolation_db);
    assert!(cold_m.topological_mode_dephasing_rate_hz < warm_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut low_p = QuantumDigitalTwinParams::default();
    low_p.microwave_probe_power_uw = 1.0;
    let mut high_p = QuantumDigitalTwinParams::default();
    high_p.microwave_probe_power_uw = 25.0;

    let low_m = QuantumDigitalTwinSolver::new(low_p).evaluate_metrics();
    let high_m = QuantumDigitalTwinSolver::new(high_p).evaluate_metrics();

    assert!(high_m.coemulation_fidelity > low_m.coemulation_fidelity);
    assert!(high_m.quantum_bus_state_retention_fraction > low_m.quantum_bus_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_core_crosstalk_isolation_db > low_m.inter_core_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synthetic_coemulation_cores_factor_scaling() {
    let mut low_p = QuantumDigitalTwinParams::default();
    low_p.synthetic_coemulation_cores_factor = 2.0;
    let mut high_p = QuantumDigitalTwinParams::default();
    high_p.synthetic_coemulation_cores_factor = 7.0;

    let low_m = QuantumDigitalTwinSolver::new(low_p).evaluate_metrics();
    let high_m = QuantumDigitalTwinSolver::new(high_p).evaluate_metrics();

    assert!(high_m.coemulation_fidelity > low_m.coemulation_fidelity);
    assert!(high_m.quantum_bus_state_retention_fraction > low_m.quantum_bus_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_core_crosstalk_isolation_db > low_m.inter_core_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_bus_interconnect_pitch_scaling() {
    let mut low_p = QuantumDigitalTwinParams::default();
    low_p.bus_interconnect_pitch_um = 1.0;
    let mut high_p = QuantumDigitalTwinParams::default();
    high_p.bus_interconnect_pitch_um = 18.0;

    let low_m = QuantumDigitalTwinSolver::new(low_p).evaluate_metrics();
    let high_m = QuantumDigitalTwinSolver::new(high_p).evaluate_metrics();

    assert!(high_m.coemulation_fidelity > low_m.coemulation_fidelity);
    assert!(high_m.quantum_bus_state_retention_fraction > low_m.quantum_bus_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_core_crosstalk_isolation_db > low_m.inter_core_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
