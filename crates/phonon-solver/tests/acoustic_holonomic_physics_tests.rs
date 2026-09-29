#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Abelian quantum acoustic
//! holonomic gates and geometric phase processors in phononic resonator networks.

use phonon_models::acoustic_holonomic_processor::AcousticHolonomicProcessorParams;
use phonon_solver::acoustic_holonomic_processor::AcousticHolonomicProcessorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = AcousticHolonomicProcessorParams::new(
        1.0,    // below 2.0 GHz
        5.0,    // below 10.0 MHz
        0.05,   // below 0.1 rad
        0.80,   // below 0.90
        0.1,    // below 0.5 kHz
        0.5,    // below 1.0 mK
        2.0,    // below 5.0 MHz
        0.5,    // below 1.0 ns
    );
    assert!((underflow.acoustic_resonance_ghz - 2.0).abs() < 1e-9);
    assert!((underflow.piezoelectric_drive_amplitude_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.wilczek_zee_phase_rad - 0.1).abs() < 1e-9);
    assert!((underflow.dynamical_phase_cancellation_ratio - 0.90).abs() < 1e-9);
    assert!((underflow.acoustic_damping_rate_khz - 0.5).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.qubit_coupling_rate_mhz - 5.0).abs() < 1e-9);
    assert!((underflow.pulse_rise_time_ns - 1.0).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = AcousticHolonomicProcessorParams::new(
        20.0,   // above 15.0 GHz
        150.0,  // above 100.0 MHz
        4.0,    // above 3.14159 rad
        1.20,   // above 1.00
        80.0,   // above 50.0 kHz
        70.0,   // above 50.0 mK
        80.0,   // above 50.0 MHz
        30.0,   // above 20.0 ns
    );
    assert!((overflow.acoustic_resonance_ghz - 15.0).abs() < 1e-9);
    assert!((overflow.piezoelectric_drive_amplitude_mhz - 100.0).abs() < 1e-9);
    assert!((overflow.wilczek_zee_phase_rad - 3.14159).abs() < 1e-9);
    assert!((overflow.dynamical_phase_cancellation_ratio - 1.00).abs() < 1e-9);
    assert!((overflow.acoustic_damping_rate_khz - 50.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.qubit_coupling_rate_mhz - 50.0).abs() < 1e-9);
    assert!((overflow.pulse_rise_time_ns - 20.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AcousticHolonomicProcessorParams::default();
    let solver = AcousticHolonomicProcessorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical targets for default parameters
    assert!(
        metrics.holonomic_gate_fidelity >= 0.9950,
        "Default holonomic gate fidelity must be >= 0.9950, got {:.5}",
        metrics.holonomic_gate_fidelity
    );
    assert!(
        metrics.gate_operation_time_ns <= 200.0,
        "Default gate operation time must be <= 200.0 ns, got {:.3} ns",
        metrics.gate_operation_time_ns
    );
    assert!(
        metrics.gate_error_rate <= 1.0e-3,
        "Default gate error rate must be <= 1.0e-3, got {:.5e}",
        metrics.gate_error_rate
    );
    assert!(
        metrics.two_qubit_entangling_fidelity >= 0.9920,
        "Default two-qubit entangling fidelity must be >= 0.9920, got {:.5}",
        metrics.two_qubit_entangling_fidelity
    );
    assert!(
        metrics.geometric_purity >= 0.9900,
        "Default geometric purity must be >= 0.9900, got {:.5}",
        metrics.geometric_purity
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_drive_amplitude_scaling() {
    let base = AcousticHolonomicProcessorParams::default();
    let solver_base = AcousticHolonomicProcessorSolver::new(base);
    let tau_base = solver_base.compute_gate_operation_time_ns();
    let fid_base = solver_base.compute_holonomic_gate_fidelity();

    let high_drive = AcousticHolonomicProcessorParams::new(
        base.acoustic_resonance_ghz,
        60.0, // increased from 35.0 MHz
        base.wilczek_zee_phase_rad,
        base.dynamical_phase_cancellation_ratio,
        base.acoustic_damping_rate_khz,
        base.operating_temp_m_k,
        base.qubit_coupling_rate_mhz,
        base.pulse_rise_time_ns,
    );
    let solver_high = AcousticHolonomicProcessorSolver::new(high_drive);
    let tau_high = solver_high.compute_gate_operation_time_ns();
    let fid_high = solver_high.compute_holonomic_gate_fidelity();

    assert!(
        tau_high < tau_base,
        "Higher drive amplitude must reduce gate operation time: {} < {}",
        tau_high,
        tau_base
    );
    assert!(
        fid_high >= fid_base,
        "Higher drive amplitude must improve gate fidelity: {} >= {}",
        fid_high,
        fid_base
    );
}

#[test]
fn test_cancellation_ratio_scaling() {
    let base = AcousticHolonomicProcessorParams::default();
    let solver_base = AcousticHolonomicProcessorSolver::new(base);
    let fid_base = solver_base.compute_holonomic_gate_fidelity();
    let purity_base = solver_base.compute_geometric_purity();
    let err_base = solver_base.compute_gate_error_rate();

    let ideal_cancellation = AcousticHolonomicProcessorParams::new(
        base.acoustic_resonance_ghz,
        base.piezoelectric_drive_amplitude_mhz,
        base.wilczek_zee_phase_rad,
        1.00, // increased from 0.99
        base.acoustic_damping_rate_khz,
        base.operating_temp_m_k,
        base.qubit_coupling_rate_mhz,
        base.pulse_rise_time_ns,
    );
    let solver_ideal = AcousticHolonomicProcessorSolver::new(ideal_cancellation);
    let fid_ideal = solver_ideal.compute_holonomic_gate_fidelity();
    let purity_ideal = solver_ideal.compute_geometric_purity();
    let err_ideal = solver_ideal.compute_gate_error_rate();

    assert!(
        fid_ideal > fid_base,
        "Higher cancellation ratio must increase fidelity: {} > {}",
        fid_ideal,
        fid_base
    );
    assert!(
        purity_ideal > purity_base,
        "Higher cancellation ratio must increase geometric purity: {} > {}",
        purity_ideal,
        purity_base
    );
    assert!(
        err_ideal < err_base,
        "Higher cancellation ratio must reduce gate error rate: {} < {}",
        err_ideal,
        err_base
    );
}

#[test]
fn test_temperature_degradation() {
    let cold = AcousticHolonomicProcessorParams::new(
        6.0, 35.0, 1.5708, 0.99, 5.0, 10.0, 20.0, 5.0, // 10 mK
    );
    let warm = AcousticHolonomicProcessorParams::new(
        6.0, 35.0, 1.5708, 0.99, 5.0, 25.0, 20.0, 5.0, // 25 mK
    );

    let solver_cold = AcousticHolonomicProcessorSolver::new(cold);
    let solver_warm = AcousticHolonomicProcessorSolver::new(warm);

    let metrics_cold = solver_cold.evaluate_metrics();
    let metrics_warm = solver_warm.evaluate_metrics();

    assert!(
        metrics_cold.holonomic_gate_fidelity > metrics_warm.holonomic_gate_fidelity,
        "Warmer temperature must decrease holonomic gate fidelity: {} > {}",
        metrics_cold.holonomic_gate_fidelity,
        metrics_warm.holonomic_gate_fidelity
    );
    assert!(
        metrics_cold.two_qubit_entangling_fidelity > metrics_warm.two_qubit_entangling_fidelity,
        "Warmer temperature must decrease two-qubit entangling fidelity: {} > {}",
        metrics_cold.two_qubit_entangling_fidelity,
        metrics_warm.two_qubit_entangling_fidelity
    );
    assert!(
        metrics_cold.gate_error_rate < metrics_warm.gate_error_rate,
        "Warmer temperature must increase gate error rate: {} < {}",
        metrics_cold.gate_error_rate,
        metrics_warm.gate_error_rate
    );
}

#[test]
fn test_damping_scaling() {
    let base = AcousticHolonomicProcessorParams::default();
    let solver_base = AcousticHolonomicProcessorSolver::new(base);
    let fid_base = solver_base.compute_holonomic_gate_fidelity();
    let err_base = solver_base.compute_gate_error_rate();

    let high_damping = AcousticHolonomicProcessorParams::new(
        base.acoustic_resonance_ghz,
        base.piezoelectric_drive_amplitude_mhz,
        base.wilczek_zee_phase_rad,
        base.dynamical_phase_cancellation_ratio,
        15.0, // increased from 5.0 kHz
        base.operating_temp_m_k,
        base.qubit_coupling_rate_mhz,
        base.pulse_rise_time_ns,
    );
    let solver_high_damping = AcousticHolonomicProcessorSolver::new(high_damping);
    let fid_high = solver_high_damping.compute_holonomic_gate_fidelity();
    let err_high = solver_high_damping.compute_gate_error_rate();

    assert!(
        fid_high < fid_base,
        "Higher damping rate must decrease fidelity: {} < {}",
        fid_high,
        fid_base
    );
    assert!(
        err_high > err_base,
        "Higher damping rate must increase error rate: {} > {}",
        err_high,
        err_base
    );
}

#[test]
fn test_physical_compliance() {
    let nominal = AcousticHolonomicProcessorParams::default();
    let solver_nominal = AcousticHolonomicProcessorSolver::new(nominal);
    let metrics_nominal = solver_nominal.evaluate_metrics();
    assert!(
        metrics_nominal.is_physically_compliant,
        "Nominal parameters must be physically compliant"
    );
    assert!(metrics_nominal.holonomic_gate_fidelity >= 0.9950);
    assert!(metrics_nominal.gate_operation_time_ns <= 200.0);
    assert!(metrics_nominal.gate_error_rate <= 1.0e-3);
    assert!(metrics_nominal.two_qubit_entangling_fidelity >= 0.9920);
    assert!(metrics_nominal.geometric_purity >= 0.9900);

    // Test across alternative high-performance design regimes
    let high_perf = AcousticHolonomicProcessorParams::new(
        8.0, 50.0, 1.5708, 0.995, 3.0, 12.0, 25.0, 4.0,
    );
    let solver_high_perf = AcousticHolonomicProcessorSolver::new(high_perf);
    let metrics_high_perf = solver_high_perf.evaluate_metrics();
    assert!(
        metrics_high_perf.is_physically_compliant,
        "High-performance parameters must be physically compliant"
    );
    assert!(metrics_high_perf.holonomic_gate_fidelity >= 0.9950);
    assert!(metrics_high_perf.gate_operation_time_ns <= 200.0);
    assert!(metrics_high_perf.gate_error_rate <= 1.0e-3);
    assert!(metrics_high_perf.two_qubit_entangling_fidelity >= 0.9920);
    assert!(metrics_high_perf.geometric_purity >= 0.9900);
}
