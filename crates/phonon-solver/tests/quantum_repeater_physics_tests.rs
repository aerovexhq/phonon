#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for hybrid superconducting
//! opto-acoustic quantum repeaters and entanglement distribution networks.

use phonon_models::opto_acoustic_quantum_repeater::OptoAcousticQuantumRepeaterParams;
use phonon_solver::opto_acoustic_quantum_repeater::OptoAcousticQuantumRepeaterSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = OptoAcousticQuantumRepeaterParams::new(
        1,      // below 2
        0.5,    // below 1.0 km
        0.30,   // below 0.50
        0.5,    // below 1.0 ms
        0.10,   // below 0.15 dB/km
        0,      // below 1
        0.2,    // below 1.0 mK
        0.2,    // below 0.5 MHz
    );
    assert_eq!(underflow.repeater_nodes_count, 2);
    assert!((underflow.channel_distance_km - 1.0).abs() < 1e-9);
    assert!((underflow.transducer_efficiency - 0.50).abs() < 1e-9);
    assert!((underflow.acoustic_memory_coherence_ms - 1.0).abs() < 1e-9);
    assert!((underflow.optical_fiber_attenuation_db_per_km - 0.15).abs() < 1e-9);
    assert_eq!(underflow.purification_rounds, 1);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.pump_repetition_freq_mhz - 0.5).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = OptoAcousticQuantumRepeaterParams::new(
        32,     // above 16
        150.0,  // above 100.0 km
        1.20,   // above 0.99
        80.0,   // above 50.0 ms
        0.50,   // above 0.35 dB/km
        8,      // above 5
        75.0,   // above 50.0 mK
        30.0,   // above 20.0 MHz
    );
    assert_eq!(overflow.repeater_nodes_count, 16);
    assert!((overflow.channel_distance_km - 100.0).abs() < 1e-9);
    assert!((overflow.transducer_efficiency - 0.99).abs() < 1e-9);
    assert!((overflow.acoustic_memory_coherence_ms - 50.0).abs() < 1e-9);
    assert!((overflow.optical_fiber_attenuation_db_per_km - 0.35).abs() < 1e-9);
    assert_eq!(overflow.purification_rounds, 5);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.pump_repetition_freq_mhz - 20.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = OptoAcousticQuantumRepeaterParams::default();
    let solver = OptoAcousticQuantumRepeaterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical targets for default parameters
    assert!(
        metrics.bell_state_fidelity >= 0.950,
        "Default Bell-state fidelity must be >= 0.950, got {:.5}",
        metrics.bell_state_fidelity
    );
    assert!(
        metrics.repetition_rate_khz >= 100.0,
        "Default repetition rate must be >= 100.0 kHz, got {:.3} kHz",
        metrics.repetition_rate_khz
    );
    assert!(
        metrics.distribution_latency_us <= 10.0,
        "Default latency must be <= 10.0 us, got {:.3} us",
        metrics.distribution_latency_us
    );
    assert!(
        metrics.memory_transduction_roundtrip_fidelity >= 0.980,
        "Default roundtrip fidelity must be >= 0.980, got {:.5}",
        metrics.memory_transduction_roundtrip_fidelity
    );
    assert!(
        metrics.purification_efficiency >= 0.850,
        "Default purification efficiency must be >= 0.850, got {:.5}",
        metrics.purification_efficiency
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_distance_scaling() {
    let base = OptoAcousticQuantumRepeaterParams::default();
    let solver_base = OptoAcousticQuantumRepeaterSolver::new(base);
    let fid_base = solver_base.compute_bell_state_fidelity();
    let lat_base = solver_base.compute_distribution_latency_us();
    let rate_base = solver_base.compute_repetition_rate_khz();

    // Longer channel distance should decrease fidelity, increase latency, and decrease repetition rate
    let long_dist = OptoAcousticQuantumRepeaterParams::new(
        base.repeater_nodes_count,
        base.channel_distance_km + 30.0,
        base.transducer_efficiency,
        base.acoustic_memory_coherence_ms,
        base.optical_fiber_attenuation_db_per_km,
        base.purification_rounds,
        base.operating_temp_m_k,
        base.pump_repetition_freq_mhz,
    );
    let solver_long = OptoAcousticQuantumRepeaterSolver::new(long_dist);
    let fid_long = solver_long.compute_bell_state_fidelity();
    let lat_long = solver_long.compute_distribution_latency_us();
    let rate_long = solver_long.compute_repetition_rate_khz();

    assert!(
        fid_long <= fid_base,
        "Longer channel distance must not increase Bell-state fidelity: long={:.5}, base={:.5}",
        fid_long, fid_base
    );
    assert!(
        lat_long >= lat_base,
        "Longer channel distance must increase distribution latency: long={:.3}, base={:.3}",
        lat_long, lat_base
    );
    assert!(
        rate_long <= rate_base,
        "Longer channel distance must decrease repetition rate: long={:.3}, base={:.3}",
        rate_long, rate_base
    );
}

#[test]
fn test_purification_scaling() {
    let base = OptoAcousticQuantumRepeaterParams::default();
    let solver_base = OptoAcousticQuantumRepeaterSolver::new(base);
    let fid_base = solver_base.compute_bell_state_fidelity();

    // Higher purification rounds should improve Bell-state fidelity
    let high_pur = OptoAcousticQuantumRepeaterParams::new(
        base.repeater_nodes_count,
        base.channel_distance_km,
        base.transducer_efficiency,
        base.acoustic_memory_coherence_ms,
        base.optical_fiber_attenuation_db_per_km,
        base.purification_rounds + 2,
        base.operating_temp_m_k,
        base.pump_repetition_freq_mhz,
    );
    let solver_high_pur = OptoAcousticQuantumRepeaterSolver::new(high_pur);
    let fid_high_pur = solver_high_pur.compute_bell_state_fidelity();

    assert!(
        fid_high_pur >= fid_base,
        "Additional purification rounds must increase Bell-state fidelity: high={:.5}, base={:.5}",
        fid_high_pur, fid_base
    );
}

#[test]
fn test_temperature_scaling_and_degradation() {
    let base = OptoAcousticQuantumRepeaterParams::default();
    let solver_base = OptoAcousticQuantumRepeaterSolver::new(base);
    let fid_base = solver_base.compute_bell_state_fidelity();
    let roundtrip_base = solver_base.compute_memory_transduction_roundtrip_fidelity();
    let pur_eff_base = solver_base.compute_purification_efficiency();

    // Elevated temperature should degrade fidelities and purification efficiency
    let hot = OptoAcousticQuantumRepeaterParams::new(
        base.repeater_nodes_count,
        base.channel_distance_km,
        base.transducer_efficiency,
        base.acoustic_memory_coherence_ms,
        base.optical_fiber_attenuation_db_per_km,
        base.purification_rounds,
        base.operating_temp_m_k + 15.0,
        base.pump_repetition_freq_mhz,
    );
    let solver_hot = OptoAcousticQuantumRepeaterSolver::new(hot);
    let fid_hot = solver_hot.compute_bell_state_fidelity();
    let roundtrip_hot = solver_hot.compute_memory_transduction_roundtrip_fidelity();
    let pur_eff_hot = solver_hot.compute_purification_efficiency();

    assert!(
        fid_hot <= fid_base,
        "Thermal noise must degrade Bell-state fidelity: hot={:.5}, base={:.5}",
        fid_hot, fid_base
    );
    assert!(
        roundtrip_hot <= roundtrip_base,
        "Thermal noise must degrade roundtrip fidelity: hot={:.5}, base={:.5}",
        roundtrip_hot, roundtrip_base
    );
    assert!(
        pur_eff_hot <= pur_eff_base,
        "Thermal noise must degrade purification efficiency: hot={:.5}, base={:.5}",
        pur_eff_hot, pur_eff_base
    );
}

#[test]
fn test_transducer_efficiency_scaling() {
    let base = OptoAcousticQuantumRepeaterParams::default();
    let solver_base = OptoAcousticQuantumRepeaterSolver::new(base);
    let fid_base = solver_base.compute_bell_state_fidelity();
    let rate_base = solver_base.compute_repetition_rate_khz();
    let roundtrip_base = solver_base.compute_memory_transduction_roundtrip_fidelity();

    // Higher transducer efficiency should improve fidelities and repetition rate
    let high_trans = OptoAcousticQuantumRepeaterParams::new(
        base.repeater_nodes_count,
        base.channel_distance_km,
        0.95,
        base.acoustic_memory_coherence_ms,
        base.optical_fiber_attenuation_db_per_km,
        base.purification_rounds,
        base.operating_temp_m_k,
        base.pump_repetition_freq_mhz,
    );
    let solver_high_trans = OptoAcousticQuantumRepeaterSolver::new(high_trans);
    let fid_high_trans = solver_high_trans.compute_bell_state_fidelity();
    let rate_high_trans = solver_high_trans.compute_repetition_rate_khz();
    let roundtrip_high_trans = solver_high_trans.compute_memory_transduction_roundtrip_fidelity();

    assert!(
        fid_high_trans >= fid_base,
        "Higher transducer efficiency must improve Bell-state fidelity: high={:.5}, base={:.5}",
        fid_high_trans, fid_base
    );
    assert!(
        rate_high_trans >= rate_base,
        "Higher transducer efficiency must improve repetition rate: high={:.3}, base={:.3}",
        rate_high_trans, rate_base
    );
    assert!(
        roundtrip_high_trans >= roundtrip_base,
        "Higher transducer efficiency must improve roundtrip fidelity: high={:.5}, base={:.5}",
        roundtrip_high_trans, roundtrip_base
    );
}

#[test]
fn test_pump_frequency_scaling() {
    let base = OptoAcousticQuantumRepeaterParams::default();
    let solver_base = OptoAcousticQuantumRepeaterSolver::new(base);
    let rate_base = solver_base.compute_repetition_rate_khz();

    // Higher pump repetition frequency should increase entanglement distribution rate
    let fast_pump = OptoAcousticQuantumRepeaterParams::new(
        base.repeater_nodes_count,
        base.channel_distance_km,
        base.transducer_efficiency,
        base.acoustic_memory_coherence_ms,
        base.optical_fiber_attenuation_db_per_km,
        base.purification_rounds,
        base.operating_temp_m_k,
        base.pump_repetition_freq_mhz + 5.0,
    );
    let solver_fast = OptoAcousticQuantumRepeaterSolver::new(fast_pump);
    let rate_fast = solver_fast.compute_repetition_rate_khz();

    assert!(
        rate_fast >= rate_base,
        "Higher pump frequency must increase repetition rate: fast={:.3}, base={:.3}",
        rate_fast, rate_base
    );
}
