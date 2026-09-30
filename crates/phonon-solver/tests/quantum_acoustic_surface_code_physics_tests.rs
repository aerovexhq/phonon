#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Abelian quantum acoustic
//! fault-tolerant surface codes and chiral Majorana stabilizer simulators.

use phonon_models::quantum_acoustic_surface_code::QuantumAcousticSurfaceCodeParams;
use phonon_solver::quantum_acoustic_surface_code::QuantumAcousticSurfaceCodeSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = QuantumAcousticSurfaceCodeParams::new(
        2.0,    // below 3.0
        5.0e-5, // below 1.0e-4
        5.0,    // below 10.0 ns
        5.0,    // below 10.0 MHz
        0.5,    // below 1.0 mK
        1.0,    // below 2.0 GHz
        0.5,    // below 1.0 um
        5.0,    // below 10.0
    );
    assert_eq!(underflow.code_distance, 3.0);
    assert_eq!(underflow.physical_error_rate, 1.0e-4);
    assert_eq!(underflow.syndrome_extraction_time_ns, 10.0);
    assert_eq!(underflow.majorana_coupling_gap_mhz, 10.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.acoustic_stabilizer_frequency_ghz, 2.0);
    assert_eq!(underflow.inter_stabilizer_pitch_um, 1.0);
    assert_eq!(underflow.decoder_maximum_weight_iterations, 10.0);

    // Test values strictly above physical maximum bounds
    let overflow = QuantumAcousticSurfaceCodeParams::new(
        20.0,   // above 15.0
        0.05,   // above 0.02
        400.0,  // above 300.0 ns
        100.0,  // above 80.0 MHz
        75.0,   // above 50.0 mK
        20.0,   // above 15.0 GHz
        25.0,   // above 15.0 um
        300.0,  // above 200.0
    );
    assert_eq!(overflow.code_distance, 15.0);
    assert_eq!(overflow.physical_error_rate, 0.02);
    assert_eq!(overflow.syndrome_extraction_time_ns, 300.0);
    assert_eq!(overflow.majorana_coupling_gap_mhz, 80.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.acoustic_stabilizer_frequency_ghz, 15.0);
    assert_eq!(overflow.inter_stabilizer_pitch_um, 15.0);
    assert_eq!(overflow.decoder_maximum_weight_iterations, 200.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = QuantumAcousticSurfaceCodeParams::default();
    assert_eq!(params.code_distance, 5.0);
    assert_eq!(params.physical_error_rate, 0.0025);
    assert_eq!(params.syndrome_extraction_time_ns, 65.0);
    assert_eq!(params.majorana_coupling_gap_mhz, 38.0);
    assert_eq!(params.cryogenic_temperature_mk, 12.0);
    assert_eq!(params.acoustic_stabilizer_frequency_ghz, 5.8);
    assert_eq!(params.inter_stabilizer_pitch_um, 4.2);
    assert_eq!(params.decoder_maximum_weight_iterations, 50.0);

    let solver = QuantumAcousticSurfaceCodeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.logical_state_fidelity >= 0.9980,
        "Logical state fidelity must be >= 0.9980, got {:.6}",
        metrics.logical_state_fidelity
    );
    assert!(
        metrics.fault_tolerant_threshold_error_rate <= 0.0075,
        "Fault-tolerant threshold error rate must be <= 0.0075, got {:.6}",
        metrics.fault_tolerant_threshold_error_rate
    );
    assert!(
        metrics.syndrome_decoding_latency_ns <= 120.0,
        "Syndrome decoding latency must be <= 120.0 ns, got {:.2} ns",
        metrics.syndrome_decoding_latency_ns
    );
    assert!(
        metrics.uncorrectable_logical_error_rate <= 1.0e-5,
        "Uncorrectable logical error rate must be <= 1.0e-5, got {:.3e}",
        metrics.uncorrectable_logical_error_rate
    );
    assert!(
        metrics.inter_stabilizer_crosstalk_isolation_db >= 52.0,
        "Inter-stabilizer crosstalk isolation must be >= 52.0 dB, got {:.2} dB",
        metrics.inter_stabilizer_crosstalk_isolation_db
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_code_distance_scaling() {
    let mut params_low = QuantumAcousticSurfaceCodeParams::default();
    params_low.code_distance = 3.0;
    let mut params_high = QuantumAcousticSurfaceCodeParams::default();
    params_high.code_distance = 15.0;

    let solver_low = QuantumAcousticSurfaceCodeSolver::new(params_low);
    let solver_high = QuantumAcousticSurfaceCodeSolver::new(params_high);

    // Higher code distance increases fidelity, lowers threshold defect rate and logical error rate
    assert!(
        solver_high.compute_logical_state_fidelity() > solver_low.compute_logical_state_fidelity()
    );
    assert!(
        solver_high.compute_fault_tolerant_threshold_error_rate()
            < solver_low.compute_fault_tolerant_threshold_error_rate()
    );
    assert!(
        solver_high.compute_uncorrectable_logical_error_rate()
            < solver_low.compute_uncorrectable_logical_error_rate()
    );
}

#[test]
fn test_physical_error_rate_scaling() {
    let mut params_low = QuantumAcousticSurfaceCodeParams::default();
    params_low.physical_error_rate = 1.0e-4;
    let mut params_high = QuantumAcousticSurfaceCodeParams::default();
    params_high.physical_error_rate = 0.02;

    let solver_low = QuantumAcousticSurfaceCodeSolver::new(params_low);
    let solver_high = QuantumAcousticSurfaceCodeSolver::new(params_high);

    // Higher physical error rate degrades fidelity and increases uncorrectable logical errors
    assert!(
        solver_high.compute_logical_state_fidelity() < solver_low.compute_logical_state_fidelity()
    );
    assert!(
        solver_high.compute_fault_tolerant_threshold_error_rate()
            > solver_low.compute_fault_tolerant_threshold_error_rate()
    );
    assert!(
        solver_high.compute_uncorrectable_logical_error_rate()
            > solver_low.compute_uncorrectable_logical_error_rate()
    );
}

#[test]
fn test_syndrome_extraction_time_scaling() {
    let mut params_fast = QuantumAcousticSurfaceCodeParams::default();
    params_fast.syndrome_extraction_time_ns = 15.0;
    let mut params_slow = QuantumAcousticSurfaceCodeParams::default();
    params_slow.syndrome_extraction_time_ns = 250.0;

    let solver_fast = QuantumAcousticSurfaceCodeSolver::new(params_fast);
    let solver_slow = QuantumAcousticSurfaceCodeSolver::new(params_slow);

    // Longer extraction time increases overall syndrome decoding latency
    assert!(
        solver_slow.compute_syndrome_decoding_latency_ns()
            > solver_fast.compute_syndrome_decoding_latency_ns()
    );
}

#[test]
fn test_majorana_coupling_gap_scaling() {
    let mut params_low = QuantumAcousticSurfaceCodeParams::default();
    params_low.majorana_coupling_gap_mhz = 15.0;
    let mut params_high = QuantumAcousticSurfaceCodeParams::default();
    params_high.majorana_coupling_gap_mhz = 75.0;

    let solver_low = QuantumAcousticSurfaceCodeSolver::new(params_low);
    let solver_high = QuantumAcousticSurfaceCodeSolver::new(params_high);

    // Higher Majorana coupling gap improves topological protection and isolation
    assert!(
        solver_high.compute_logical_state_fidelity() > solver_low.compute_logical_state_fidelity()
    );
    assert!(
        solver_high.compute_fault_tolerant_threshold_error_rate()
            < solver_low.compute_fault_tolerant_threshold_error_rate()
    );
    assert!(
        solver_high.compute_inter_stabilizer_crosstalk_isolation_db()
            > solver_low.compute_inter_stabilizer_crosstalk_isolation_db()
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut params_cold = QuantumAcousticSurfaceCodeParams::default();
    params_cold.cryogenic_temperature_mk = 2.0;
    let mut params_warm = QuantumAcousticSurfaceCodeParams::default();
    params_warm.cryogenic_temperature_mk = 45.0;

    let solver_cold = QuantumAcousticSurfaceCodeSolver::new(params_cold);
    let solver_warm = QuantumAcousticSurfaceCodeSolver::new(params_warm);

    // Warmer temperatures induce thermal phonon dephasing and increase error rates
    assert!(
        solver_warm.compute_logical_state_fidelity() < solver_cold.compute_logical_state_fidelity()
    );
    assert!(
        solver_warm.compute_fault_tolerant_threshold_error_rate()
            > solver_cold.compute_fault_tolerant_threshold_error_rate()
    );
    assert!(
        solver_warm.compute_uncorrectable_logical_error_rate()
            > solver_cold.compute_uncorrectable_logical_error_rate()
    );
}

#[test]
fn test_acoustic_stabilizer_frequency_scaling() {
    let mut params_low = QuantumAcousticSurfaceCodeParams::default();
    params_low.acoustic_stabilizer_frequency_ghz = 2.5;
    let mut params_high = QuantumAcousticSurfaceCodeParams::default();
    params_high.acoustic_stabilizer_frequency_ghz = 14.0;

    let solver_low = QuantumAcousticSurfaceCodeSolver::new(params_low);
    let solver_high = QuantumAcousticSurfaceCodeSolver::new(params_high);

    // Higher frequency reduces evanescent overlap, improving crosstalk isolation
    assert!(
        solver_high.compute_inter_stabilizer_crosstalk_isolation_db()
            > solver_low.compute_inter_stabilizer_crosstalk_isolation_db()
    );
}

#[test]
fn test_inter_stabilizer_pitch_scaling() {
    let mut params_narrow = QuantumAcousticSurfaceCodeParams::default();
    params_narrow.inter_stabilizer_pitch_um = 1.5;
    let mut params_wide = QuantumAcousticSurfaceCodeParams::default();
    params_wide.inter_stabilizer_pitch_um = 12.0;

    let solver_narrow = QuantumAcousticSurfaceCodeSolver::new(params_narrow);
    let solver_wide = QuantumAcousticSurfaceCodeSolver::new(params_wide);

    // Larger pitch spacing dramatically increases crosstalk isolation
    assert!(
        solver_wide.compute_inter_stabilizer_crosstalk_isolation_db()
            > solver_narrow.compute_inter_stabilizer_crosstalk_isolation_db()
    );
    assert!(
        solver_wide.compute_uncorrectable_logical_error_rate()
            < solver_narrow.compute_uncorrectable_logical_error_rate()
    );
}

#[test]
fn test_decoder_maximum_weight_iterations_scaling() {
    let mut params_low = QuantumAcousticSurfaceCodeParams::default();
    params_low.decoder_maximum_weight_iterations = 15.0;
    let mut params_high = QuantumAcousticSurfaceCodeParams::default();
    params_high.decoder_maximum_weight_iterations = 180.0;

    let solver_low = QuantumAcousticSurfaceCodeSolver::new(params_low);
    let solver_high = QuantumAcousticSurfaceCodeSolver::new(params_high);

    // Higher decoder iterations enable more accurate graph matching convergence
    assert!(
        solver_high.compute_logical_state_fidelity() > solver_low.compute_logical_state_fidelity()
    );
    assert!(
        solver_high.compute_fault_tolerant_threshold_error_rate()
            < solver_low.compute_fault_tolerant_threshold_error_rate()
    );
    assert!(
        solver_high.compute_uncorrectable_logical_error_rate()
            < solver_low.compute_uncorrectable_logical_error_rate()
    );
}
