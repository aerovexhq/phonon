#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological anyon braiding circuit compilers and topological QASM synthesizers.

use phonon_models::braiding_circuit_compiler::BraidingCircuitCompilerParams;
use phonon_solver::braiding_circuit_compiler::BraidingCircuitCompilerSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = BraidingCircuitCompilerParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 order
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.compiler_coupling_energy_mev, 1.0);
    assert_eq!(underflow.topological_braiding_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.braiding_execution_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_synthesis_power_uw, 0.5);
    assert_eq!(underflow.synthetic_braid_depth_order, 1.0);
    assert_eq!(underflow.braiding_channel_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = BraidingCircuitCompilerParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 order
        35.0,   // above 20.0 um
    );
    assert_eq!(overflow.compiler_coupling_energy_mev, 35.0);
    assert_eq!(overflow.topological_braiding_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.braiding_execution_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_synthesis_power_uw, 30.0);
    assert_eq!(overflow.synthetic_braid_depth_order, 8.0);
    assert_eq!(overflow.braiding_channel_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = BraidingCircuitCompilerParams::default();
    assert_eq!(params.compiler_coupling_energy_mev, 16.5);
    assert_eq!(params.topological_braiding_gap_mev, 22.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.8);
    assert_eq!(params.braiding_execution_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_synthesis_power_uw, 5.8);
    assert_eq!(params.synthetic_braid_depth_order, 4.0);
    assert_eq!(params.braiding_channel_pitch_um, 4.8);

    let solver = BraidingCircuitCompilerSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.compiling_fidelity >= 0.9980,
        "Compiling fidelity must be >= 0.9980, got {:.6}",
        metrics.compiling_fidelity
    );
    assert!(
        metrics.braiding_state_retention_fraction >= 0.9970,
        "Braiding state retention fraction must be >= 0.9970, got {:.6}",
        metrics.braiding_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_channel_crosstalk_isolation_db >= 55.0,
        "Inter-channel crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_channel_crosstalk_isolation_db
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
fn test_compiler_coupling_energy_scaling() {
    let mut low_p = BraidingCircuitCompilerParams::default();
    low_p.compiler_coupling_energy_mev = 2.0;
    let mut high_p = BraidingCircuitCompilerParams::default();
    high_p.compiler_coupling_energy_mev = 30.0;

    let low_m = BraidingCircuitCompilerSolver::new(low_p).evaluate_metrics();
    let high_m = BraidingCircuitCompilerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.compiling_fidelity > low_m.compiling_fidelity);
    assert!(high_m.braiding_state_retention_fraction > low_m.braiding_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_braiding_gap_scaling() {
    let mut low_p = BraidingCircuitCompilerParams::default();
    low_p.topological_braiding_gap_mev = 5.0;
    let mut high_p = BraidingCircuitCompilerParams::default();
    high_p.topological_braiding_gap_mev = 40.0;

    let low_m = BraidingCircuitCompilerSolver::new(low_p).evaluate_metrics();
    let high_m = BraidingCircuitCompilerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.compiling_fidelity > low_m.compiling_fidelity);
    assert!(high_m.braiding_state_retention_fraction > low_m.braiding_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = BraidingCircuitCompilerParams::default();
    low_p.acoustic_drive_frequency_ghz = 2.0;
    let mut high_p = BraidingCircuitCompilerParams::default();
    high_p.acoustic_drive_frequency_ghz = 10.0;

    let low_m = BraidingCircuitCompilerSolver::new(low_p).evaluate_metrics();
    let high_m = BraidingCircuitCompilerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.compiling_fidelity > low_m.compiling_fidelity);
    assert!(high_m.braiding_state_retention_fraction > low_m.braiding_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_braiding_execution_speed_scaling() {
    let mut low_p = BraidingCircuitCompilerParams::default();
    low_p.braiding_execution_speed_m_per_s = 400.0;
    let mut high_p = BraidingCircuitCompilerParams::default();
    high_p.braiding_execution_speed_m_per_s = 2600.0;

    let low_m = BraidingCircuitCompilerSolver::new(low_p).evaluate_metrics();
    let high_m = BraidingCircuitCompilerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.compiling_fidelity > low_m.compiling_fidelity);
    assert!(high_m.braiding_state_retention_fraction > low_m.braiding_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold_p = BraidingCircuitCompilerParams::default();
    cold_p.cryogenic_temperature_mk = 2.0;
    let mut warm_p = BraidingCircuitCompilerParams::default();
    warm_p.cryogenic_temperature_mk = 45.0;

    let cold_m = BraidingCircuitCompilerSolver::new(cold_p).evaluate_metrics();
    let warm_m = BraidingCircuitCompilerSolver::new(warm_p).evaluate_metrics();

    assert!(cold_m.compiling_fidelity > warm_m.compiling_fidelity);
    assert!(cold_m.braiding_state_retention_fraction > warm_m.braiding_state_retention_fraction);
    assert!(cold_m.topological_protection_gap_mhz > warm_m.topological_protection_gap_mhz);
    assert!(cold_m.inter_channel_crosstalk_isolation_db > warm_m.inter_channel_crosstalk_isolation_db);
    assert!(cold_m.topological_mode_dephasing_rate_hz < warm_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_synthesis_power_scaling() {
    let mut low_p = BraidingCircuitCompilerParams::default();
    low_p.microwave_synthesis_power_uw = 1.0;
    let mut high_p = BraidingCircuitCompilerParams::default();
    high_p.microwave_synthesis_power_uw = 25.0;

    let low_m = BraidingCircuitCompilerSolver::new(low_p).evaluate_metrics();
    let high_m = BraidingCircuitCompilerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.compiling_fidelity > low_m.compiling_fidelity);
    assert!(high_m.braiding_state_retention_fraction > low_m.braiding_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synthetic_braid_depth_order_scaling() {
    let mut low_p = BraidingCircuitCompilerParams::default();
    low_p.synthetic_braid_depth_order = 1.5;
    let mut high_p = BraidingCircuitCompilerParams::default();
    high_p.synthetic_braid_depth_order = 7.5;

    let low_m = BraidingCircuitCompilerSolver::new(low_p).evaluate_metrics();
    let high_m = BraidingCircuitCompilerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.compiling_fidelity > low_m.compiling_fidelity);
    assert!(high_m.braiding_state_retention_fraction > low_m.braiding_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_braiding_channel_pitch_scaling() {
    let mut low_p = BraidingCircuitCompilerParams::default();
    low_p.braiding_channel_pitch_um = 1.0;
    let mut high_p = BraidingCircuitCompilerParams::default();
    high_p.braiding_channel_pitch_um = 18.0;

    let low_m = BraidingCircuitCompilerSolver::new(low_p).evaluate_metrics();
    let high_m = BraidingCircuitCompilerSolver::new(high_p).evaluate_metrics();

    assert!(high_m.compiling_fidelity > low_m.compiling_fidelity);
    assert!(high_m.braiding_state_retention_fraction > low_m.braiding_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_channel_crosstalk_isolation_db > low_m.inter_channel_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
