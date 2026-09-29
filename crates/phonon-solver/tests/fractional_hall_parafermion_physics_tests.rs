#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for fractional quantum Hall
//! acoustic metamaterials and non-Abelian parafermion interferometers.

use phonon_models::fractional_hall_parafermion::FractionalHallParafermionParams;
use phonon_solver::fractional_hall_parafermion::FractionalHallParafermionSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = FractionalHallParafermionParams::new(
        0.5, // below 1.0 GHz
        5.0, // below 10.0 MHz
        0.1, // below 0.20
        1,   // below 3
        5.0, // below 10.0 um
        0.1, // below 0.2 kHz
        0.5, // below 1.0 mK
        2.0, // below 5.0 MHz
    );
    assert!((underflow.acoustic_resonance_freq_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.synthetic_lorentz_coupling_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.fractional_filling_factor - 0.20).abs() < 1e-9);
    assert_eq!(underflow.parafermion_order_z_m, 3);
    assert!((underflow.interferometer_arm_length_um - 10.0).abs() < 1e-9);
    assert!((underflow.acoustic_damping_rate_khz - 0.2).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.quasiparticle_tunneling_mhz - 5.0).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = FractionalHallParafermionParams::new(
        15.0,  // above 12.0 GHz
        150.0, // above 120.0 MHz
        0.95,  // above 0.80
        10,    // above 6
        150.0, // above 120.0 um
        30.0,  // above 20.0 kHz
        70.0,  // above 50.0 mK
        80.0,  // above 50.0 MHz
    );
    assert!((overflow.acoustic_resonance_freq_ghz - 12.0).abs() < 1e-9);
    assert!((overflow.synthetic_lorentz_coupling_mhz - 120.0).abs() < 1e-9);
    assert!((overflow.fractional_filling_factor - 0.80).abs() < 1e-9);
    assert_eq!(overflow.parafermion_order_z_m, 6);
    assert!((overflow.interferometer_arm_length_um - 120.0).abs() < 1e-9);
    assert!((overflow.acoustic_damping_rate_khz - 20.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.quasiparticle_tunneling_mhz - 50.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FractionalHallParafermionParams::default();
    let solver = FractionalHallParafermionSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.braid_phase_fidelity >= 0.9970,
        "Default braid phase fidelity must be >= 0.9970, got {:.6}",
        metrics.braid_phase_fidelity
    );
    assert!(
        metrics.fractional_state_fidelity >= 0.9950,
        "Default fractional state fidelity must be >= 0.9950, got {:.6}",
        metrics.fractional_state_fidelity
    );
    assert!(
        metrics.fractional_quantization_error <= 0.0050,
        "Default fractional quantization error must be <= 0.0050, got {:.6}",
        metrics.fractional_quantization_error
    );
    assert!(
        metrics.topological_fractional_gap_mhz >= 15.0,
        "Default topological fractional gap must be >= 15.0 MHz, got {:.4} MHz",
        metrics.topological_fractional_gap_mhz
    );
    assert!(
        metrics.braiding_visibility >= 0.9600,
        "Default braiding visibility must be >= 0.9600, got {:.6}",
        metrics.braiding_visibility
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_lorentz_coupling_scaling() {
    let base = FractionalHallParafermionParams::default();
    let solver_base = FractionalHallParafermionSolver::new(base);

    let high_lorentz = FractionalHallParafermionParams::new(
        base.acoustic_resonance_freq_ghz,
        100.0, // increased from 65.0 MHz
        base.fractional_filling_factor,
        base.parafermion_order_z_m,
        base.interferometer_arm_length_um,
        base.acoustic_damping_rate_khz,
        base.operating_temp_m_k,
        base.quasiparticle_tunneling_mhz,
    );
    let solver_high_lorentz = FractionalHallParafermionSolver::new(high_lorentz);

    let m_base = solver_base.evaluate_metrics();
    let m_high = solver_high_lorentz.evaluate_metrics();

    assert!(
        m_high.braid_phase_fidelity >= m_base.braid_phase_fidelity,
        "Higher synthetic Lorentz coupling must enhance braid phase fidelity"
    );
    assert!(
        m_high.topological_fractional_gap_mhz > m_base.topological_fractional_gap_mhz,
        "Higher synthetic Lorentz coupling must expand the topological fractional gap"
    );
}

#[test]
fn test_arm_length_scaling() {
    let base = FractionalHallParafermionParams::default();
    let solver_base = FractionalHallParafermionSolver::new(base);

    let long_arm = FractionalHallParafermionParams::new(
        base.acoustic_resonance_freq_ghz,
        base.synthetic_lorentz_coupling_mhz,
        base.fractional_filling_factor,
        base.parafermion_order_z_m,
        90.0, // increased from 45.0 um
        base.acoustic_damping_rate_khz,
        base.operating_temp_m_k,
        base.quasiparticle_tunneling_mhz,
    );
    let solver_long_arm = FractionalHallParafermionSolver::new(long_arm);

    let m_base = solver_base.evaluate_metrics();
    let m_long = solver_long_arm.evaluate_metrics();

    assert!(
        m_long.fractional_state_fidelity < m_base.fractional_state_fidelity,
        "Longer interferometer arm length must decrease fractional state fidelity due to propagation loss"
    );
}

#[test]
fn test_temperature_degradation() {
    let base = FractionalHallParafermionParams::default();
    let solver_base = FractionalHallParafermionSolver::new(base);

    let warm = FractionalHallParafermionParams::new(
        base.acoustic_resonance_freq_ghz,
        base.synthetic_lorentz_coupling_mhz,
        base.fractional_filling_factor,
        base.parafermion_order_z_m,
        base.interferometer_arm_length_um,
        base.acoustic_damping_rate_khz,
        30.0, // warmed from 12.0 mK
        base.quasiparticle_tunneling_mhz,
    );
    let solver_warm = FractionalHallParafermionSolver::new(warm);

    let m_base = solver_base.evaluate_metrics();
    let m_warm = solver_warm.evaluate_metrics();

    let cold = FractionalHallParafermionParams::new(
        base.acoustic_resonance_freq_ghz,
        base.synthetic_lorentz_coupling_mhz,
        base.fractional_filling_factor,
        base.parafermion_order_z_m,
        base.interferometer_arm_length_um,
        base.acoustic_damping_rate_khz,
        6.0, // colder than 12.0 mK
        base.quasiparticle_tunneling_mhz,
    );
    let solver_cold = FractionalHallParafermionSolver::new(cold);
    let m_cold = solver_cold.evaluate_metrics();

    assert!(
        m_base.fractional_quantization_error > m_cold.fractional_quantization_error,
        "Higher temperature must strictly increase fractional quantization error from cryogenic base"
    );
    assert!(
        m_warm.braid_phase_fidelity < m_base.braid_phase_fidelity,
        "Elevated temperature must degrade braid phase fidelity"
    );
    assert!(
        m_warm.fractional_state_fidelity < m_base.fractional_state_fidelity,
        "Elevated temperature must degrade fractional state fidelity"
    );
    assert!(
        m_warm.fractional_quantization_error >= m_base.fractional_quantization_error,
        "Elevated temperature must increase or saturate fractional quantization error"
    );
    assert!(
        m_warm.topological_fractional_gap_mhz < m_base.topological_fractional_gap_mhz,
        "Elevated temperature must compress the topological fractional gap"
    );
    assert!(
        m_warm.braiding_visibility < m_base.braiding_visibility,
        "Elevated temperature must decrease braiding visibility"
    );
}

#[test]
fn test_damping_rate_scaling() {
    let base = FractionalHallParafermionParams::default();
    let solver_base = FractionalHallParafermionSolver::new(base);

    let low_damp = FractionalHallParafermionParams::new(
        base.acoustic_resonance_freq_ghz,
        base.synthetic_lorentz_coupling_mhz,
        base.fractional_filling_factor,
        base.parafermion_order_z_m,
        base.interferometer_arm_length_um,
        0.9, // lower than 1.8 kHz
        base.operating_temp_m_k,
        base.quasiparticle_tunneling_mhz,
    );
    let solver_low_damp = FractionalHallParafermionSolver::new(low_damp);
    let m_low = solver_low_damp.evaluate_metrics();

    let damped = FractionalHallParafermionParams::new(
        base.acoustic_resonance_freq_ghz,
        base.synthetic_lorentz_coupling_mhz,
        base.fractional_filling_factor,
        base.parafermion_order_z_m,
        base.interferometer_arm_length_um,
        5.0, // increased from 1.8 kHz
        base.operating_temp_m_k,
        base.quasiparticle_tunneling_mhz,
    );
    let solver_damped = FractionalHallParafermionSolver::new(damped);

    let m_base = solver_base.evaluate_metrics();
    let m_damped = solver_damped.evaluate_metrics();

    assert!(
        m_base.fractional_quantization_error > m_low.fractional_quantization_error,
        "Higher acoustic damping must strictly increase fractional quantization error from low-loss base"
    );
    assert!(
        m_damped.braid_phase_fidelity < m_base.braid_phase_fidelity,
        "Higher acoustic damping must degrade braid phase fidelity"
    );
    assert!(
        m_damped.fractional_quantization_error >= m_base.fractional_quantization_error,
        "Higher acoustic damping must increase or saturate fractional quantization error"
    );
    assert!(
        m_damped.braiding_visibility < m_base.braiding_visibility,
        "Higher acoustic damping must degrade braiding visibility"
    );
}

#[test]
fn test_physical_compliance_thresholds() {
    let params = FractionalHallParafermionParams::default();
    let solver = FractionalHallParafermionSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
    assert!(metrics.braid_phase_fidelity >= 0.9970);
    assert!(metrics.fractional_state_fidelity >= 0.9950);
    assert!(metrics.fractional_quantization_error <= 0.0050);
    assert!(metrics.topological_fractional_gap_mhz >= 15.0);
    assert!(metrics.braiding_visibility >= 0.9600);
}
