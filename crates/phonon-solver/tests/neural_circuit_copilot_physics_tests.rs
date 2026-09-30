#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Reinforcement Learning Co-Pilot
//! & Neural Circuit Synthesizer.

use phonon_models::neural_circuit_copilot::NeuralCircuitCopilotParams;
use phonon_solver::neural_circuit_copilot::NeuralCircuitCopilotSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = NeuralCircuitCopilotParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.rl_policy_coupling_mev, 1.0);
    assert_eq!(underflow.topological_policy_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.neural_inference_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_critic_power_uw, 0.5);
    assert_eq!(underflow.synthetic_actor_depth_factor, 1.0);
    assert_eq!(underflow.synaptic_routing_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = NeuralCircuitCopilotParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.rl_policy_coupling_mev, 35.0);
    assert_eq!(overflow.topological_policy_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.neural_inference_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_critic_power_uw, 30.0);
    assert_eq!(overflow.synthetic_actor_depth_factor, 8.0);
    assert_eq!(overflow.synaptic_routing_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = NeuralCircuitCopilotParams::default();
    assert_eq!(params.rl_policy_coupling_mev, 18.0);
    assert_eq!(params.topological_policy_gap_mev, 24.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 6.5);
    assert_eq!(params.neural_inference_dispatch_speed_m_per_s, 1500.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_critic_power_uw, 6.5);
    assert_eq!(params.synthetic_actor_depth_factor, 4.0);
    assert_eq!(params.synaptic_routing_pitch_um, 5.5);

    let solver = NeuralCircuitCopilotSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.copilot_synthesis_fidelity >= 0.9980,
        "Co-pilot synthesis fidelity must be >= 0.9980, got {:.6}",
        metrics.copilot_synthesis_fidelity
    );
    assert!(
        metrics.neural_state_retention_fraction >= 0.9970,
        "Neural state retention fraction must be >= 0.9970, got {:.6}",
        metrics.neural_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_layer_crosstalk_isolation_db >= 55.0,
        "Inter-layer crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_layer_crosstalk_isolation_db
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
fn test_rl_policy_coupling_scaling() {
    let mut low_p = NeuralCircuitCopilotParams::default();
    low_p.rl_policy_coupling_mev = 2.0;
    let mut high_p = NeuralCircuitCopilotParams::default();
    high_p.rl_policy_coupling_mev = 30.0;

    let low_m = NeuralCircuitCopilotSolver::new(low_p).evaluate_metrics();
    let high_m = NeuralCircuitCopilotSolver::new(high_p).evaluate_metrics();

    assert!(high_m.copilot_synthesis_fidelity > low_m.copilot_synthesis_fidelity);
    assert!(high_m.neural_state_retention_fraction > low_m.neural_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_crosstalk_isolation_db > low_m.inter_layer_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_topological_policy_gap_scaling() {
    let mut low_p = NeuralCircuitCopilotParams::default();
    low_p.topological_policy_gap_mev = 3.0;
    let mut high_p = NeuralCircuitCopilotParams::default();
    high_p.topological_policy_gap_mev = 40.0;

    let low_m = NeuralCircuitCopilotSolver::new(low_p).evaluate_metrics();
    let high_m = NeuralCircuitCopilotSolver::new(high_p).evaluate_metrics();

    assert!(high_m.copilot_synthesis_fidelity > low_m.copilot_synthesis_fidelity);
    assert!(high_m.neural_state_retention_fraction > low_m.neural_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_crosstalk_isolation_db > low_m.inter_layer_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_p = NeuralCircuitCopilotParams::default();
    low_p.acoustic_drive_frequency_ghz = 2.0;
    let mut high_p = NeuralCircuitCopilotParams::default();
    high_p.acoustic_drive_frequency_ghz = 10.0;

    let low_m = NeuralCircuitCopilotSolver::new(low_p).evaluate_metrics();
    let high_m = NeuralCircuitCopilotSolver::new(high_p).evaluate_metrics();

    assert!(high_m.copilot_synthesis_fidelity > low_m.copilot_synthesis_fidelity);
    assert!(high_m.neural_state_retention_fraction > low_m.neural_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_crosstalk_isolation_db > low_m.inter_layer_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_neural_inference_dispatch_speed_scaling() {
    let mut low_p = NeuralCircuitCopilotParams::default();
    low_p.neural_inference_dispatch_speed_m_per_s = 300.0;
    let mut high_p = NeuralCircuitCopilotParams::default();
    high_p.neural_inference_dispatch_speed_m_per_s = 2800.0;

    let low_m = NeuralCircuitCopilotSolver::new(low_p).evaluate_metrics();
    let high_m = NeuralCircuitCopilotSolver::new(high_p).evaluate_metrics();

    assert!(high_m.copilot_synthesis_fidelity > low_m.copilot_synthesis_fidelity);
    assert!(high_m.neural_state_retention_fraction > low_m.neural_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_crosstalk_isolation_db > low_m.inter_layer_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold_p = NeuralCircuitCopilotParams::default();
    cold_p.cryogenic_temperature_mk = 2.0;
    let mut warm_p = NeuralCircuitCopilotParams::default();
    warm_p.cryogenic_temperature_mk = 45.0;

    let cold_m = NeuralCircuitCopilotSolver::new(cold_p).evaluate_metrics();
    let warm_m = NeuralCircuitCopilotSolver::new(warm_p).evaluate_metrics();

    assert!(cold_m.copilot_synthesis_fidelity > warm_m.copilot_synthesis_fidelity);
    assert!(cold_m.neural_state_retention_fraction > warm_m.neural_state_retention_fraction);
    assert!(cold_m.topological_protection_gap_mhz > warm_m.topological_protection_gap_mhz);
    assert!(cold_m.inter_layer_crosstalk_isolation_db > warm_m.inter_layer_crosstalk_isolation_db);
    assert!(cold_m.topological_mode_dephasing_rate_hz < warm_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_microwave_critic_power_scaling() {
    let mut low_p = NeuralCircuitCopilotParams::default();
    low_p.microwave_critic_power_uw = 1.0;
    let mut high_p = NeuralCircuitCopilotParams::default();
    high_p.microwave_critic_power_uw = 25.0;

    let low_m = NeuralCircuitCopilotSolver::new(low_p).evaluate_metrics();
    let high_m = NeuralCircuitCopilotSolver::new(high_p).evaluate_metrics();

    assert!(high_m.copilot_synthesis_fidelity > low_m.copilot_synthesis_fidelity);
    assert!(high_m.neural_state_retention_fraction > low_m.neural_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_crosstalk_isolation_db > low_m.inter_layer_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synthetic_actor_depth_factor_scaling() {
    let mut low_p = NeuralCircuitCopilotParams::default();
    low_p.synthetic_actor_depth_factor = 2.0;
    let mut high_p = NeuralCircuitCopilotParams::default();
    high_p.synthetic_actor_depth_factor = 7.0;

    let low_m = NeuralCircuitCopilotSolver::new(low_p).evaluate_metrics();
    let high_m = NeuralCircuitCopilotSolver::new(high_p).evaluate_metrics();

    assert!(high_m.copilot_synthesis_fidelity > low_m.copilot_synthesis_fidelity);
    assert!(high_m.neural_state_retention_fraction > low_m.neural_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_crosstalk_isolation_db > low_m.inter_layer_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_synaptic_routing_pitch_scaling() {
    let mut low_p = NeuralCircuitCopilotParams::default();
    low_p.synaptic_routing_pitch_um = 1.0;
    let mut high_p = NeuralCircuitCopilotParams::default();
    high_p.synaptic_routing_pitch_um = 18.0;

    let low_m = NeuralCircuitCopilotSolver::new(low_p).evaluate_metrics();
    let high_m = NeuralCircuitCopilotSolver::new(high_p).evaluate_metrics();

    assert!(high_m.copilot_synthesis_fidelity > low_m.copilot_synthesis_fidelity);
    assert!(high_m.neural_state_retention_fraction > low_m.neural_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_layer_crosstalk_isolation_db > low_m.inter_layer_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}
