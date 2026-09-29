#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic
//! tensor network simulators and continuous-variable fault-tolerant
//! magic state distillation.

use phonon_models::quantum_acoustic_tensor_distillation::QuantumAcousticTensorDistillationParams;
use phonon_solver::quantum_acoustic_tensor_distillation::QuantumAcousticTensorDistillationSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = QuantumAcousticTensorDistillationParams::new(
        1,       // below 4
        2,       // below 8
        0.5,     // below 1.0 GHz
        1.0e4,   // below 1.0e5
        0.1,     // below 0.5
        0.5,     // below 1.0 MHz
        0.2,     // below 1.0 mK
        0.20,    // below 0.50
    );
    assert_eq!(underflow.resonator_modes, 4);
    assert_eq!(underflow.bond_dimension, 8);
    assert!((underflow.acoustic_frequency_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.cavity_q_factor - 1.0e5).abs() < 1e-9);
    assert!((underflow.squeezing_param_r - 0.5).abs() < 1e-9);
    assert!((underflow.non_linear_coupling_mhz - 1.0).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.photon_subtraction_efficiency - 0.50).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = QuantumAcousticTensorDistillationParams::new(
        128,     // above 64
        256,     // above 128
        20.0,    // above 12.0 GHz
        1.0e9,   // above 1.0e8
        5.0,     // above 2.5
        100.0,   // above 50.0 MHz
        80.0,    // above 50.0 mK
        1.20,    // above 0.99
    );
    assert_eq!(overflow.resonator_modes, 64);
    assert_eq!(overflow.bond_dimension, 128);
    assert!((overflow.acoustic_frequency_ghz - 12.0).abs() < 1e-9);
    assert!((overflow.cavity_q_factor - 1.0e8).abs() < 1e-9);
    assert!((overflow.squeezing_param_r - 2.5).abs() < 1e-9);
    assert!((overflow.non_linear_coupling_mhz - 50.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.photon_subtraction_efficiency - 0.99).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = QuantumAcousticTensorDistillationParams::default();
    let solver = QuantumAcousticTensorDistillationSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical targets for default parameters
    assert!(
        metrics.magic_state_fidelity >= 0.990,
        "Default magic state fidelity must be >= 0.990, got {:.5}",
        metrics.magic_state_fidelity
    );
    assert!(
        metrics.photon_subtraction_prob >= 0.150,
        "Default photon subtraction prob must be >= 0.150, got {:.4}",
        metrics.photon_subtraction_prob
    );
    assert!(
        metrics.distillation_cycle_latency_us <= 5.0,
        "Default cycle latency must be <= 5.0 us, got {:.3} us",
        metrics.distillation_cycle_latency_us
    );
    assert!(
        metrics.non_gaussian_gate_fidelity >= 0.985,
        "Default non-Gaussian gate fidelity must be >= 0.985, got {:.5}",
        metrics.non_gaussian_gate_fidelity
    );
    assert!(
        metrics.acoustic_error_threshold >= 0.015,
        "Default acoustic error threshold must be >= 0.015, got {:.5}",
        metrics.acoustic_error_threshold
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_magic_state_fidelity_scaling() {
    let base = QuantumAcousticTensorDistillationParams::default();
    let solver_base = QuantumAcousticTensorDistillationSolver::new(base);
    let fid_base = solver_base.compute_magic_state_fidelity();

    // Higher squeezing r should improve magic state output fidelity
    let high_squeeze = QuantumAcousticTensorDistillationParams::new(
        base.resonator_modes,
        base.bond_dimension,
        base.acoustic_frequency_ghz,
        base.cavity_q_factor,
        1.80, // higher than 1.25
        base.non_linear_coupling_mhz,
        base.operating_temp_m_k,
        base.photon_subtraction_efficiency,
    );
    let solver_squeeze = QuantumAcousticTensorDistillationSolver::new(high_squeeze);
    assert!(
        solver_squeeze.compute_magic_state_fidelity() > fid_base,
        "Higher squeezing parameter r must enhance magic state fidelity"
    );

    // Higher photon subtraction efficiency should improve fidelity
    let high_eff = QuantumAcousticTensorDistillationParams::new(
        base.resonator_modes,
        base.bond_dimension,
        base.acoustic_frequency_ghz,
        base.cavity_q_factor,
        base.squeezing_param_r,
        base.non_linear_coupling_mhz,
        base.operating_temp_m_k,
        0.95, // higher than 0.88
    );
    let solver_eff = QuantumAcousticTensorDistillationSolver::new(high_eff);
    assert!(
        solver_eff.compute_magic_state_fidelity() > fid_base,
        "Higher photon subtraction efficiency must enhance magic state fidelity"
    );
}

#[test]
fn test_photon_subtraction_prob_scaling() {
    let base = QuantumAcousticTensorDistillationParams::default();
    let solver_base = QuantumAcousticTensorDistillationSolver::new(base);
    let prob_base = solver_base.compute_photon_subtraction_prob();

    // Higher squeezing r enhances photon subtraction probability
    let high_squeeze = QuantumAcousticTensorDistillationParams::new(
        base.resonator_modes,
        base.bond_dimension,
        base.acoustic_frequency_ghz,
        base.cavity_q_factor,
        1.75, // higher than 1.25
        base.non_linear_coupling_mhz,
        base.operating_temp_m_k,
        base.photon_subtraction_efficiency,
    );
    let solver_squeeze = QuantumAcousticTensorDistillationSolver::new(high_squeeze);
    assert!(
        solver_squeeze.compute_photon_subtraction_prob() > prob_base,
        "Higher squeezing parameter r must increase photon subtraction probability"
    );
}

#[test]
fn test_distillation_cycle_latency_scaling() {
    let base = QuantumAcousticTensorDistillationParams::default();
    let solver_base = QuantumAcousticTensorDistillationSolver::new(base);
    let lat_base = solver_base.compute_distillation_cycle_latency_us();

    // Larger bond dimension increases tensor contraction latency
    let larger_bond = QuantumAcousticTensorDistillationParams::new(
        base.resonator_modes,
        48, // higher than 32
        base.acoustic_frequency_ghz,
        base.cavity_q_factor,
        base.squeezing_param_r,
        base.non_linear_coupling_mhz,
        base.operating_temp_m_k,
        base.photon_subtraction_efficiency,
    );
    let solver_bond = QuantumAcousticTensorDistillationSolver::new(larger_bond);
    assert!(
        solver_bond.compute_distillation_cycle_latency_us() > lat_base,
        "Larger bond dimension chi must increase distillation cycle latency"
    );

    // More resonator modes increases tensor contraction latency
    let more_modes = QuantumAcousticTensorDistillationParams::new(
        24, // higher than 16
        base.bond_dimension,
        base.acoustic_frequency_ghz,
        base.cavity_q_factor,
        base.squeezing_param_r,
        base.non_linear_coupling_mhz,
        base.operating_temp_m_k,
        base.photon_subtraction_efficiency,
    );
    let solver_modes = QuantumAcousticTensorDistillationSolver::new(more_modes);
    assert!(
        solver_modes.compute_distillation_cycle_latency_us() > lat_base,
        "More resonator modes must increase distillation cycle latency"
    );
}

#[test]
fn test_non_gaussian_gate_fidelity_scaling() {
    let base = QuantumAcousticTensorDistillationParams::default();
    let solver_base = QuantumAcousticTensorDistillationSolver::new(base);
    let fid_base = solver_base.compute_non_gaussian_gate_fidelity();

    // Higher non-linear coupling improves gate fidelity
    let high_coupling = QuantumAcousticTensorDistillationParams::new(
        base.resonator_modes,
        base.bond_dimension,
        base.acoustic_frequency_ghz,
        base.cavity_q_factor,
        base.squeezing_param_r,
        28.0, // higher than 18.0 MHz
        base.operating_temp_m_k,
        base.photon_subtraction_efficiency,
    );
    let solver_coupling = QuantumAcousticTensorDistillationSolver::new(high_coupling);
    assert!(
        solver_coupling.compute_non_gaussian_gate_fidelity() > fid_base,
        "Higher non-linear coupling must enhance gate fidelity"
    );
}

#[test]
fn test_acoustic_error_threshold_scaling() {
    let base = QuantumAcousticTensorDistillationParams::default();
    let solver_base = QuantumAcousticTensorDistillationSolver::new(base);
    let th_base = solver_base.compute_acoustic_error_threshold();

    // Higher cavity Q-factor improves error threshold
    let high_q = QuantumAcousticTensorDistillationParams::new(
        base.resonator_modes,
        base.bond_dimension,
        base.acoustic_frequency_ghz,
        5.0e7, // higher than 2.0e7
        base.squeezing_param_r,
        base.non_linear_coupling_mhz,
        base.operating_temp_m_k,
        base.photon_subtraction_efficiency,
    );
    let solver_q = QuantumAcousticTensorDistillationSolver::new(high_q);
    assert!(
        solver_q.compute_acoustic_error_threshold() > th_base,
        "Higher cavity Q-factor must enhance physical error threshold"
    );
}

#[test]
fn test_temperature_scaling_and_degradation() {
    let base = QuantumAcousticTensorDistillationParams::default();
    let solver_base = QuantumAcousticTensorDistillationSolver::new(base);

    // Warm temperature (higher thermal noise) degrades all key quantum metrics
    let warm_params = QuantumAcousticTensorDistillationParams::new(
        base.resonator_modes,
        base.bond_dimension,
        base.acoustic_frequency_ghz,
        base.cavity_q_factor,
        base.squeezing_param_r,
        base.non_linear_coupling_mhz,
        25.0, // warmer than 15.0 mK
        base.photon_subtraction_efficiency,
    );
    let solver_warm = QuantumAcousticTensorDistillationSolver::new(warm_params);

    assert!(
        solver_warm.compute_magic_state_fidelity() < solver_base.compute_magic_state_fidelity(),
        "Higher operating temperature must decrease magic state fidelity"
    );
    assert!(
        solver_warm.compute_photon_subtraction_prob() < solver_base.compute_photon_subtraction_prob(),
        "Higher operating temperature must decrease photon subtraction probability"
    );
    assert!(
        solver_warm.compute_non_gaussian_gate_fidelity() < solver_base.compute_non_gaussian_gate_fidelity(),
        "Higher operating temperature must decrease non-Gaussian gate fidelity"
    );
    assert!(
        solver_warm.compute_acoustic_error_threshold() < solver_base.compute_acoustic_error_threshold(),
        "Higher operating temperature must decrease acoustic error threshold"
    );
}
