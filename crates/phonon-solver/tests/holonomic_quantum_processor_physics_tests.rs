#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum non-Abelian
//! holonomic acoustic gate processors and braided phonon circuit architectures.

use phonon_models::holonomic_quantum_processor::HolonomicQuantumProcessorParams;
use phonon_solver::holonomic_quantum_processor::HolonomicQuantumProcessorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = HolonomicQuantumProcessorParams::new(
        1.0,   // below 2.0 GHz
        10.0,  // below 20.0 MHz
        0.85,  // below 0.92
        5.0,   // below 10.0 MHz
        0.1,   // below 0.2 kHz
        0.5,   // below 1.0 mK
        0.5,   // below 1.0 ns
        1,     // below 2
    );
    assert!((underflow.qubit_acoustic_freq_ghz - 2.0).abs() < 1e-9);
    assert!((underflow.driving_field_amplitude_mhz - 20.0).abs() < 1e-9);
    assert!((underflow.dynamical_phase_cancellation_depth - 0.92).abs() < 1e-9);
    assert!((underflow.inter_qubit_coupling_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.acoustic_dephasing_rate_khz - 0.2).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.pulse_shaping_truncation_ns - 1.0).abs() < 1e-9);
    assert_eq!(underflow.qubit_register_size, 2);

    // Test values above physical maximum bounds
    let overflow = HolonomicQuantumProcessorParams::new(
        15.0,   // above 12.0 GHz
        250.0,  // above 200.0 MHz
        1.05,   // above 1.00
        120.0,  // above 100.0 MHz
        30.0,   // above 20.0 kHz
        70.0,   // above 50.0 mK
        15.0,   // above 10.0 ns
        50,     // above 32
    );
    assert!((overflow.qubit_acoustic_freq_ghz - 12.0).abs() < 1e-9);
    assert!((overflow.driving_field_amplitude_mhz - 200.0).abs() < 1e-9);
    assert!((overflow.dynamical_phase_cancellation_depth - 1.00).abs() < 1e-9);
    assert!((overflow.inter_qubit_coupling_mhz - 100.0).abs() < 1e-9);
    assert!((overflow.acoustic_dephasing_rate_khz - 20.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.pulse_shaping_truncation_ns - 10.0).abs() < 1e-9);
    assert_eq!(overflow.qubit_register_size, 32);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = HolonomicQuantumProcessorParams::default();
    let solver = HolonomicQuantumProcessorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.holonomic_gate_fidelity >= 0.9960,
        "Default holonomic gate fidelity must be >= 0.9960, got {:.6}",
        metrics.holonomic_gate_fidelity
    );
    assert!(
        metrics.two_qubit_gate_duration_ns <= 35.0,
        "Default two-qubit gate duration must be <= 35.0 ns, got {:.4} ns",
        metrics.two_qubit_gate_duration_ns
    );
    assert!(
        metrics.geometric_phase_error <= 0.0050,
        "Default geometric phase error must be <= 0.0050, got {:.6}",
        metrics.geometric_phase_error
    );
    assert!(
        metrics.fault_tolerant_logic_depth >= 100,
        "Default fault-tolerant logic depth must be >= 100, got {}",
        metrics.fault_tolerant_logic_depth
    );
    assert!(
        metrics.crosstalk_isolation_db >= 40.0,
        "Default crosstalk isolation must be >= 40.0 dB, got {:.4} dB",
        metrics.crosstalk_isolation_db
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_driving_amplitude_scaling() {
    let base = HolonomicQuantumProcessorParams::default();
    let solver_base = HolonomicQuantumProcessorSolver::new(base);

    let high_drive = HolonomicQuantumProcessorParams::new(
        base.qubit_acoustic_freq_ghz,
        140.0, // increased from 90.0 MHz
        base.dynamical_phase_cancellation_depth,
        base.inter_qubit_coupling_mhz,
        base.acoustic_dephasing_rate_khz,
        base.operating_temp_m_k,
        base.pulse_shaping_truncation_ns,
        base.qubit_register_size,
    );
    let solver_high_drive = HolonomicQuantumProcessorSolver::new(high_drive);

    let m_base = solver_base.evaluate_metrics();
    let m_high_drive = solver_high_drive.evaluate_metrics();

    assert!(
        m_high_drive.two_qubit_gate_duration_ns < m_base.two_qubit_gate_duration_ns,
        "Higher driving field amplitude must accelerate two-qubit entangling gate speed"
    );
    assert!(
        m_high_drive.crosstalk_isolation_db > m_base.crosstalk_isolation_db,
        "Higher driving field amplitude must improve crosstalk isolation"
    );
}

#[test]
fn test_cancellation_depth_scaling() {
    let base = HolonomicQuantumProcessorParams::default();
    let solver_base = HolonomicQuantumProcessorSolver::new(base);

    let high_cancel = HolonomicQuantumProcessorParams::new(
        base.qubit_acoustic_freq_ghz,
        base.driving_field_amplitude_mhz,
        0.999, // improved from 0.995
        base.inter_qubit_coupling_mhz,
        base.acoustic_dephasing_rate_khz,
        base.operating_temp_m_k,
        base.pulse_shaping_truncation_ns,
        base.qubit_register_size,
    );
    let solver_high_cancel = HolonomicQuantumProcessorSolver::new(high_cancel);

    let m_base = solver_base.evaluate_metrics();
    let m_high_cancel = solver_high_cancel.evaluate_metrics();

    assert!(
        m_high_cancel.holonomic_gate_fidelity > m_base.holonomic_gate_fidelity,
        "Higher dynamical phase cancellation depth must enhance holonomic gate fidelity"
    );
    assert!(
        m_high_cancel.geometric_phase_error < m_base.geometric_phase_error,
        "Higher dynamical phase cancellation depth must reduce geometric phase error"
    );
}

#[test]
fn test_temperature_degradation() {
    let base = HolonomicQuantumProcessorParams::default();
    let solver_base = HolonomicQuantumProcessorSolver::new(base);

    let warm = HolonomicQuantumProcessorParams::new(
        base.qubit_acoustic_freq_ghz,
        base.driving_field_amplitude_mhz,
        base.dynamical_phase_cancellation_depth,
        base.inter_qubit_coupling_mhz,
        base.acoustic_dephasing_rate_khz,
        25.0, // warmed from 12.0 mK
        base.pulse_shaping_truncation_ns,
        base.qubit_register_size,
    );
    let solver_warm = HolonomicQuantumProcessorSolver::new(warm);

    let m_base = solver_base.evaluate_metrics();
    let m_warm = solver_warm.evaluate_metrics();

    assert!(
        m_warm.holonomic_gate_fidelity < m_base.holonomic_gate_fidelity,
        "Elevated temperature must degrade holonomic gate fidelity"
    );
    assert!(
        m_warm.geometric_phase_error > m_base.geometric_phase_error,
        "Elevated temperature must increase geometric phase error"
    );
    assert!(
        m_warm.crosstalk_isolation_db < m_base.crosstalk_isolation_db,
        "Elevated temperature must reduce crosstalk isolation"
    );
}

#[test]
fn test_inter_qubit_coupling_scaling() {
    let base = HolonomicQuantumProcessorParams::default();
    let solver_base = HolonomicQuantumProcessorSolver::new(base);

    let strong_coupling = HolonomicQuantumProcessorParams::new(
        base.qubit_acoustic_freq_ghz,
        base.driving_field_amplitude_mhz,
        base.dynamical_phase_cancellation_depth,
        70.0, // increased from 45.0 MHz
        base.acoustic_dephasing_rate_khz,
        base.operating_temp_m_k,
        base.pulse_shaping_truncation_ns,
        base.qubit_register_size,
    );
    let solver_strong = HolonomicQuantumProcessorSolver::new(strong_coupling);

    let m_base = solver_base.evaluate_metrics();
    let m_strong = solver_strong.evaluate_metrics();

    assert!(
        m_strong.two_qubit_gate_duration_ns < m_base.two_qubit_gate_duration_ns,
        "Stronger inter-qubit acoustic coupling must accelerate entangling gate speed"
    );
}

#[test]
fn test_dephasing_rate_scaling() {
    let base = HolonomicQuantumProcessorParams::default();
    let solver_base = HolonomicQuantumProcessorSolver::new(base);

    let high_dephasing = HolonomicQuantumProcessorParams::new(
        base.qubit_acoustic_freq_ghz,
        base.driving_field_amplitude_mhz,
        base.dynamical_phase_cancellation_depth,
        base.inter_qubit_coupling_mhz,
        6.0, // increased from 2.0 kHz
        base.operating_temp_m_k,
        base.pulse_shaping_truncation_ns,
        base.qubit_register_size,
    );
    let solver_dephased = HolonomicQuantumProcessorSolver::new(high_dephasing);

    let m_base = solver_base.evaluate_metrics();
    let m_dephased = solver_dephased.evaluate_metrics();

    assert!(
        m_dephased.holonomic_gate_fidelity < m_base.holonomic_gate_fidelity,
        "Higher acoustic dephasing rate must degrade holonomic gate fidelity"
    );
}

#[test]
fn test_physical_compliance_thresholds() {
    let params = HolonomicQuantumProcessorParams::default();
    let solver = HolonomicQuantumProcessorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
    assert!(metrics.holonomic_gate_fidelity >= 0.9960);
    assert!(metrics.two_qubit_gate_duration_ns <= 35.0);
    assert!(metrics.geometric_phase_error <= 0.0050);
    assert!(metrics.fault_tolerant_logic_depth >= 100);
    assert!(metrics.crosstalk_isolation_db >= 40.0);
}
