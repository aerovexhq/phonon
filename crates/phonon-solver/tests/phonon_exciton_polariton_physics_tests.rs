#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum
//! phonon-exciton polariton condensates and chiral optomechanical polariton transducers.

use phonon_models::phonon_exciton_polariton::PhononExcitonPolaritonParams;
use phonon_solver::phonon_exciton_polariton::PhononExcitonPolaritonSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = PhononExcitonPolaritonParams::new(
        300.0, // below 350.0 THz
        1.0,   // below 2.0 GHz
        3.0,   // below 5.0 meV
        1.0,   // below 2.0 meV
        5.0,   // below 10.0 MHz
        0.1,   // below 0.2 mW
        0.005, // below 0.01 K
        5.0e3, // below 1.0e4
    );
    assert!((underflow.optical_cavity_freq_thz - 350.0).abs() < 1e-9);
    assert!((underflow.acoustic_phonon_freq_ghz - 2.0).abs() < 1e-9);
    assert!((underflow.exciton_binding_energy_mev - 5.0).abs() < 1e-9);
    assert!((underflow.rabi_splitting_energy_mev - 2.0).abs() < 1e-9);
    assert!((underflow.piezo_deform_coupling_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.optical_pump_power_mw - 0.2).abs() < 1e-9);
    assert!((underflow.operating_temp_k - 0.01).abs() < 1e-9);
    assert!((underflow.cavity_quality_factor - 1.0e4).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = PhononExcitonPolaritonParams::new(
        500.0, // above 450.0 THz
        25.0,  // above 20.0 GHz
        70.0,  // above 60.0 meV
        40.0,  // above 30.0 meV
        150.0, // above 120.0 MHz
        15.0,  // above 10.0 mW
        5.0,   // above 4.0 K
        2.0e6, // above 1.0e6
    );
    assert!((overflow.optical_cavity_freq_thz - 450.0).abs() < 1e-9);
    assert!((overflow.acoustic_phonon_freq_ghz - 20.0).abs() < 1e-9);
    assert!((overflow.exciton_binding_energy_mev - 60.0).abs() < 1e-9);
    assert!((overflow.rabi_splitting_energy_mev - 30.0).abs() < 1e-9);
    assert!((overflow.piezo_deform_coupling_mhz - 120.0).abs() < 1e-9);
    assert!((overflow.optical_pump_power_mw - 10.0).abs() < 1e-9);
    assert!((overflow.operating_temp_k - 4.0).abs() < 1e-9);
    assert!((overflow.cavity_quality_factor - 1.0e6).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = PhononExcitonPolaritonParams::default();
    let solver = PhononExcitonPolaritonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.quantum_state_fidelity >= 0.9940,
        "Default state fidelity must be >= 0.9940, got {:.6}",
        metrics.quantum_state_fidelity
    );
    assert!(
        metrics.condensation_threshold_pump_mw <= 1.200,
        "Default condensation threshold must be <= 1.200 mW, got {:.4} mW",
        metrics.condensation_threshold_pump_mw
    );
    assert!(
        metrics.polariton_coherence_time_ps >= 25.0,
        "Default coherence time must be >= 25.0 ps, got {:.4} ps",
        metrics.polariton_coherence_time_ps
    );
    assert_eq!(
        metrics.vortex_topological_charge, 1,
        "Default vortex topological charge must be 1"
    );
    assert!(
        metrics.optomechanical_coupling_rate_mhz >= 40.0,
        "Default optomechanical coupling rate must be >= 40.0 MHz, got {:.4} MHz",
        metrics.optomechanical_coupling_rate_mhz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_piezo_deformation_coupling_scaling() {
    let base = PhononExcitonPolaritonParams::default();
    let solver_base = PhononExcitonPolaritonSolver::new(base);

    let high_coupling = PhononExcitonPolaritonParams::new(
        base.optical_cavity_freq_thz,
        base.acoustic_phonon_freq_ghz,
        base.exciton_binding_energy_mev,
        base.rabi_splitting_energy_mev,
        80.0, // increased from 55.0 MHz
        base.optical_pump_power_mw,
        base.operating_temp_k,
        base.cavity_quality_factor,
    );
    let solver_high = PhononExcitonPolaritonSolver::new(high_coupling);

    let m_base = solver_base.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.quantum_state_fidelity >= m_base.quantum_state_fidelity,
        "Stronger piezoelectric coupling must enhance state transfer fidelity"
    );
    assert!(
        m_high.optomechanical_coupling_rate_mhz > m_base.optomechanical_coupling_rate_mhz,
        "Stronger piezoelectric coupling must increase optomechanical coupling rate"
    );
}

#[test]
fn test_optical_pump_power_scaling() {
    let base = PhononExcitonPolaritonParams::default();
    let solver_base = PhononExcitonPolaritonSolver::new(base);

    let high_pump = PhononExcitonPolaritonParams::new(
        base.optical_cavity_freq_thz,
        base.acoustic_phonon_freq_ghz,
        base.exciton_binding_energy_mev,
        base.rabi_splitting_energy_mev,
        base.piezo_deform_coupling_mhz,
        5.0, // increased from 2.5 mW
        base.operating_temp_k,
        base.cavity_quality_factor,
    );
    let solver_high = PhononExcitonPolaritonSolver::new(high_pump);

    let m_base = solver_base.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.polariton_coherence_time_ps > m_base.polariton_coherence_time_ps,
        "Higher pump power must increase temporal polariton coherence time"
    );
}

#[test]
fn test_temperature_degradation() {
    let base = PhononExcitonPolaritonParams::default();
    let solver_base = PhononExcitonPolaritonSolver::new(base);

    let warm = PhononExcitonPolaritonParams::new(
        base.optical_cavity_freq_thz,
        base.acoustic_phonon_freq_ghz,
        base.exciton_binding_energy_mev,
        base.rabi_splitting_energy_mev,
        base.piezo_deform_coupling_mhz,
        base.optical_pump_power_mw,
        1.5, // warmed from 0.30 K
        base.cavity_quality_factor,
    );
    let solver_warm = PhononExcitonPolaritonSolver::new(warm);

    let m_base = solver_base.evaluate_metrics();
    let m_warm = solver_warm.evaluate_metrics();

    assert!(
        m_warm.quantum_state_fidelity < m_base.quantum_state_fidelity,
        "Elevated temperature must degrade quantum state fidelity"
    );
    assert!(
        m_warm.condensation_threshold_pump_mw > m_base.condensation_threshold_pump_mw,
        "Elevated temperature must increase condensation threshold pump power"
    );
    assert!(
        m_warm.polariton_coherence_time_ps < m_base.polariton_coherence_time_ps,
        "Elevated temperature must shorten polariton coherence lifetime"
    );
}

#[test]
fn test_cavity_quality_factor_scaling() {
    let base = PhononExcitonPolaritonParams::default();
    let solver_base = PhononExcitonPolaritonSolver::new(base);

    let high_q = PhononExcitonPolaritonParams::new(
        base.optical_cavity_freq_thz,
        base.acoustic_phonon_freq_ghz,
        base.exciton_binding_energy_mev,
        base.rabi_splitting_energy_mev,
        base.piezo_deform_coupling_mhz,
        base.optical_pump_power_mw,
        base.operating_temp_k,
        5.0e5, // increased from 1.5e5
    );
    let solver_high_q = PhononExcitonPolaritonSolver::new(high_q);

    let m_base = solver_base.evaluate_metrics();
    let m_high_q = solver_high_q.evaluate_metrics();

    assert!(
        m_high_q.condensation_threshold_pump_mw < m_base.condensation_threshold_pump_mw,
        "Higher cavity Q-factor must reduce condensation threshold pump power"
    );
    assert!(
        m_high_q.polariton_coherence_time_ps > m_base.polariton_coherence_time_ps,
        "Higher cavity Q-factor must enhance polariton coherence lifetime"
    );
    assert!(
        m_high_q.quantum_state_fidelity >= m_base.quantum_state_fidelity,
        "Higher cavity Q-factor must improve quantum state transfer fidelity"
    );
}

#[test]
fn test_physical_compliance_thresholds() {
    let params = PhononExcitonPolaritonParams::default();
    let solver = PhononExcitonPolaritonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
    assert!(metrics.quantum_state_fidelity >= 0.9940);
    assert!(metrics.condensation_threshold_pump_mw <= 1.200);
    assert!(metrics.polariton_coherence_time_ps >= 25.0);
    assert_eq!(metrics.vortex_topological_charge, 1);
    assert!(metrics.optomechanical_coupling_rate_mhz >= 40.0);
}
