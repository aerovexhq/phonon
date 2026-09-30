#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological skyrmion-lattice quantum neural processors and synaptic
//! braiding synthesizers.

use phonon_models::skyrmion_neural_processor::SkyrmionNeuralProcessorParams;
use phonon_solver::skyrmion_neural_processor::SkyrmionNeuralProcessorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SkyrmionNeuralProcessorParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        20.0,  // below 30.0 nm
        2.0,   // below 4.0
    );
    assert_eq!(underflow.synaptic_weight_coupling_mev, 1.0);
    assert_eq!(underflow.topological_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_activation_frequency_ghz, 1.0);
    assert_eq!(underflow.synaptic_braiding_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_programming_power_uw, 0.5);
    assert_eq!(underflow.skyrmion_lattice_pitch_nm, 30.0);
    assert_eq!(underflow.synaptic_array_dimension, 4.0);

    // Test values strictly above physical maximum bounds
    let overflow = SkyrmionNeuralProcessorParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        300.0,  // above 250.0 nm
        80.0,   // above 64.0
    );
    assert_eq!(overflow.synaptic_weight_coupling_mev, 35.0);
    assert_eq!(overflow.topological_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_activation_frequency_ghz, 12.0);
    assert_eq!(overflow.synaptic_braiding_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_programming_power_uw, 30.0);
    assert_eq!(overflow.skyrmion_lattice_pitch_nm, 250.0);
    assert_eq!(overflow.synaptic_array_dimension, 64.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SkyrmionNeuralProcessorParams::default();
    assert_eq!(params.synaptic_weight_coupling_mev, 16.5);
    assert_eq!(params.topological_gap_mev, 22.0);
    assert_eq!(params.acoustic_activation_frequency_ghz, 5.7);
    assert_eq!(params.synaptic_braiding_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_programming_power_uw, 5.8);
    assert_eq!(params.skyrmion_lattice_pitch_nm, 85.0);
    assert_eq!(params.synaptic_array_dimension, 16.0);

    let solver = SkyrmionNeuralProcessorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.neuromorphic_inference_fidelity >= 0.9980,
        "Neuromorphic inference fidelity must be >= 0.9980, got {:.6}",
        metrics.neuromorphic_inference_fidelity
    );
    assert!(
        metrics.synaptic_state_retention_fraction >= 0.9970,
        "Synaptic state retention fraction must be >= 0.9970, got {:.6}",
        metrics.synaptic_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_synapse_crosstalk_isolation_db >= 54.0,
        "Inter-synapse crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.inter_synapse_crosstalk_isolation_db
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
fn test_synaptic_weight_coupling_scaling() {
    let mut low_p = SkyrmionNeuralProcessorParams::default();
    low_p.synaptic_weight_coupling_mev = 2.0;

    let mut high_p = SkyrmionNeuralProcessorParams::default();
    high_p.synaptic_weight_coupling_mev = 34.0;

    let low_m = SkyrmionNeuralProcessorSolver::new(low_p).evaluate_metrics();
    let high_m = SkyrmionNeuralProcessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.neuromorphic_inference_fidelity > low_m.neuromorphic_inference_fidelity);
    assert!(high_m.synaptic_state_retention_fraction > low_m.synaptic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_synapse_crosstalk_isolation_db > low_m.inter_synapse_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_gap_scaling() {
    let mut low_p = SkyrmionNeuralProcessorParams::default();
    low_p.topological_gap_mev = 3.0;

    let mut high_p = SkyrmionNeuralProcessorParams::default();
    high_p.topological_gap_mev = 44.0;

    let low_m = SkyrmionNeuralProcessorSolver::new(low_p).evaluate_metrics();
    let high_m = SkyrmionNeuralProcessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.neuromorphic_inference_fidelity > low_m.neuromorphic_inference_fidelity);
    assert!(high_m.synaptic_state_retention_fraction > low_m.synaptic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_synapse_crosstalk_isolation_db > low_m.inter_synapse_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_activation_frequency_scaling() {
    let mut low_p = SkyrmionNeuralProcessorParams::default();
    low_p.acoustic_activation_frequency_ghz = 1.5;

    let mut high_p = SkyrmionNeuralProcessorParams::default();
    high_p.acoustic_activation_frequency_ghz = 11.5;

    let low_m = SkyrmionNeuralProcessorSolver::new(low_p).evaluate_metrics();
    let high_m = SkyrmionNeuralProcessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.neuromorphic_inference_fidelity > low_m.neuromorphic_inference_fidelity);
    assert!(high_m.synaptic_state_retention_fraction > low_m.synaptic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_synapse_crosstalk_isolation_db > low_m.inter_synapse_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synaptic_braiding_speed_scaling() {
    let mut low_p = SkyrmionNeuralProcessorParams::default();
    low_p.synaptic_braiding_speed_m_per_s = 300.0;

    let mut high_p = SkyrmionNeuralProcessorParams::default();
    high_p.synaptic_braiding_speed_m_per_s = 2900.0;

    let low_m = SkyrmionNeuralProcessorSolver::new(low_p).evaluate_metrics();
    let high_m = SkyrmionNeuralProcessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.neuromorphic_inference_fidelity > low_m.neuromorphic_inference_fidelity);
    assert!(high_m.synaptic_state_retention_fraction > low_m.synaptic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_synapse_crosstalk_isolation_db > low_m.inter_synapse_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_p = SkyrmionNeuralProcessorParams::default();
    low_p.cryogenic_temperature_mk = 2.0;

    let mut high_p = SkyrmionNeuralProcessorParams::default();
    high_p.cryogenic_temperature_mk = 48.0;

    let low_m = SkyrmionNeuralProcessorSolver::new(low_p).evaluate_metrics();
    let high_m = SkyrmionNeuralProcessorSolver::new(high_p).evaluate_metrics();

    // Higher temperature degrades performance and increases dephasing
    assert!(low_m.neuromorphic_inference_fidelity > high_m.neuromorphic_inference_fidelity);
    assert!(low_m.synaptic_state_retention_fraction > high_m.synaptic_state_retention_fraction);
    assert!(low_m.topological_protection_gap_mhz > high_m.topological_protection_gap_mhz);
    assert!(low_m.inter_synapse_crosstalk_isolation_db > high_m.inter_synapse_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz > low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_programming_power_scaling() {
    let mut low_p = SkyrmionNeuralProcessorParams::default();
    low_p.microwave_programming_power_uw = 1.0;

    let mut high_p = SkyrmionNeuralProcessorParams::default();
    high_p.microwave_programming_power_uw = 29.0;

    let low_m = SkyrmionNeuralProcessorSolver::new(low_p).evaluate_metrics();
    let high_m = SkyrmionNeuralProcessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.neuromorphic_inference_fidelity > low_m.neuromorphic_inference_fidelity);
    assert!(high_m.synaptic_state_retention_fraction > low_m.synaptic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_synapse_crosstalk_isolation_db > low_m.inter_synapse_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_skyrmion_lattice_pitch_scaling() {
    let mut low_p = SkyrmionNeuralProcessorParams::default();
    low_p.skyrmion_lattice_pitch_nm = 35.0;

    let mut high_p = SkyrmionNeuralProcessorParams::default();
    high_p.skyrmion_lattice_pitch_nm = 240.0;

    let low_m = SkyrmionNeuralProcessorSolver::new(low_p).evaluate_metrics();
    let high_m = SkyrmionNeuralProcessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.neuromorphic_inference_fidelity > low_m.neuromorphic_inference_fidelity);
    assert!(high_m.synaptic_state_retention_fraction > low_m.synaptic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_synapse_crosstalk_isolation_db > low_m.inter_synapse_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synaptic_array_dimension_scaling() {
    let mut low_p = SkyrmionNeuralProcessorParams::default();
    low_p.synaptic_array_dimension = 6.0;

    let mut high_p = SkyrmionNeuralProcessorParams::default();
    high_p.synaptic_array_dimension = 62.0;

    let low_m = SkyrmionNeuralProcessorSolver::new(low_p).evaluate_metrics();
    let high_m = SkyrmionNeuralProcessorSolver::new(high_p).evaluate_metrics();

    assert!(high_m.neuromorphic_inference_fidelity > low_m.neuromorphic_inference_fidelity);
    assert!(high_m.synaptic_state_retention_fraction > low_m.synaptic_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_synapse_crosstalk_isolation_db > low_m.inter_synapse_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
