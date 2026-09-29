#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for Floquet-Bloch synthetic
//! gauge acoustic fields and dynamically reconfigurable phononic quantum simulators.

use phonon_models::floquet_synthetic_gauge::FloquetSyntheticGaugeParams;
use phonon_solver::floquet_synthetic_gauge::FloquetSyntheticGaugeSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = FloquetSyntheticGaugeParams::new(
        0.5,   // below 1.0 GHz
        5.0,   // below 10.0 MHz
        0.01,  // below 0.05
        2,     // below 4
        0.10,  // below 0.2 rad
        2.0,   // below 5.0 MHz
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 kHz
    );
    assert!((underflow.acoustic_center_freq_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.floquet_drive_freq_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.parametric_modulation_depth - 0.05).abs() < 1e-9);
    assert_eq!(underflow.lattice_plaquette_count, 4);
    assert!((underflow.synthetic_phase_gradient_rad - 0.2).abs() < 1e-9);
    assert!((underflow.inter_site_coupling_mhz - 5.0).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.acoustic_damping_rate_khz - 0.5).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = FloquetSyntheticGaugeParams::new(
        15.0,    // above 12.0 GHz
        250.0,   // above 200.0 MHz
        0.80,    // above 0.60
        100,     // above 64
        4.0,     // above 3.14159 rad
        80.0,    // above 60.0 MHz
        70.0,    // above 50.0 mK
        80.0,    // above 50.0 kHz
    );
    assert!((overflow.acoustic_center_freq_ghz - 12.0).abs() < 1e-9);
    assert!((overflow.floquet_drive_freq_mhz - 200.0).abs() < 1e-9);
    assert!((overflow.parametric_modulation_depth - 0.60).abs() < 1e-9);
    assert_eq!(overflow.lattice_plaquette_count, 64);
    assert!((overflow.synthetic_phase_gradient_rad - 3.14159).abs() < 1e-9);
    assert!((overflow.inter_site_coupling_mhz - 60.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.acoustic_damping_rate_khz - 50.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FloquetSyntheticGaugeParams::default();
    let solver = FloquetSyntheticGaugeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.dynamical_state_fidelity >= 0.9950,
        "Default state fidelity must be >= 0.9950, got {:.6}",
        metrics.dynamical_state_fidelity
    );
    assert!(
        metrics.synthetic_magnetic_flux_ratio >= 0.500,
        "Default synthetic flux ratio must be >= 0.500, got {:.4}",
        metrics.synthetic_magnetic_flux_ratio
    );
    assert!(
        metrics.flux_quantization_error <= 0.010,
        "Default flux quantization error must be <= 0.010, got {:.6}",
        metrics.flux_quantization_error
    );
    assert!(
        metrics.chern_switching_time_ns <= 20.0,
        "Default Chern switching time must be <= 20.0 ns, got {:.4} ns",
        metrics.chern_switching_time_ns
    );
    assert!(
        metrics.topological_band_isolation_db >= 30.0,
        "Default topological band isolation must be >= 30.0 dB, got {:.4} dB",
        metrics.topological_band_isolation_db
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_modulation_depth_scaling() {
    let base = FloquetSyntheticGaugeParams::default();
    let solver_base = FloquetSyntheticGaugeSolver::new(base);

    let high_mod = FloquetSyntheticGaugeParams::new(
        base.acoustic_center_freq_ghz,
        base.floquet_drive_freq_mhz,
        0.35, // increased from 0.28
        base.lattice_plaquette_count,
        base.synthetic_phase_gradient_rad,
        base.inter_site_coupling_mhz,
        base.operating_temp_m_k,
        base.acoustic_damping_rate_khz,
    );
    let solver_high_mod = FloquetSyntheticGaugeSolver::new(high_mod);

    let m_base = solver_base.evaluate_metrics();
    let m_high_mod = solver_high_mod.evaluate_metrics();

    assert!(
        m_high_mod.dynamical_state_fidelity >= m_base.dynamical_state_fidelity,
        "Higher parametric modulation depth must enhance dynamical state fidelity"
    );
    assert!(
        m_high_mod.synthetic_magnetic_flux_ratio > m_base.synthetic_magnetic_flux_ratio,
        "Higher parametric modulation depth must increase synthetic magnetic flux ratio"
    );
    assert!(
        m_high_mod.topological_band_isolation_db > m_base.topological_band_isolation_db,
        "Higher parametric modulation depth must increase topological band isolation"
    );
}

#[test]
fn test_drive_frequency_scaling() {
    let base = FloquetSyntheticGaugeParams::default();
    let solver_base = FloquetSyntheticGaugeSolver::new(base);

    let fast_drive = FloquetSyntheticGaugeParams::new(
        base.acoustic_center_freq_ghz,
        120.0, // increased from 80.0 MHz
        base.parametric_modulation_depth,
        base.lattice_plaquette_count,
        base.synthetic_phase_gradient_rad,
        base.inter_site_coupling_mhz,
        base.operating_temp_m_k,
        base.acoustic_damping_rate_khz,
    );
    let solver_fast = FloquetSyntheticGaugeSolver::new(fast_drive);

    let m_base = solver_base.evaluate_metrics();
    let m_fast = solver_fast.evaluate_metrics();

    assert!(
        m_fast.chern_switching_time_ns < m_base.chern_switching_time_ns,
        "Higher Floquet drive frequency must accelerate Chern switching latency"
    );
}

#[test]
fn test_temperature_degradation() {
    let base = FloquetSyntheticGaugeParams::default();
    let solver_base = FloquetSyntheticGaugeSolver::new(base);

    let warm = FloquetSyntheticGaugeParams::new(
        base.acoustic_center_freq_ghz,
        base.floquet_drive_freq_mhz,
        base.parametric_modulation_depth,
        base.lattice_plaquette_count,
        base.synthetic_phase_gradient_rad,
        base.inter_site_coupling_mhz,
        25.0, // warmed from 15.0 mK
        base.acoustic_damping_rate_khz,
    );
    let solver_warm = FloquetSyntheticGaugeSolver::new(warm);

    let m_base = solver_base.evaluate_metrics();
    let m_warm = solver_warm.evaluate_metrics();

    assert!(
        m_warm.dynamical_state_fidelity < m_base.dynamical_state_fidelity,
        "Elevated temperature must degrade dynamical state fidelity"
    );
    assert!(
        m_warm.synthetic_magnetic_flux_ratio < m_base.synthetic_magnetic_flux_ratio,
        "Elevated temperature must reduce synthetic magnetic flux ratio"
    );
    assert!(
        m_warm.flux_quantization_error > m_base.flux_quantization_error,
        "Elevated temperature must increase flux quantization error"
    );
    assert!(
        m_warm.topological_band_isolation_db < m_base.topological_band_isolation_db,
        "Elevated temperature must reduce topological band isolation"
    );
}

#[test]
fn test_damping_rate_scaling() {
    let base = FloquetSyntheticGaugeParams::default();
    let solver_base = FloquetSyntheticGaugeSolver::new(base);

    let high_damping = FloquetSyntheticGaugeParams::new(
        base.acoustic_center_freq_ghz,
        base.floquet_drive_freq_mhz,
        base.parametric_modulation_depth,
        base.lattice_plaquette_count,
        base.synthetic_phase_gradient_rad,
        base.inter_site_coupling_mhz,
        base.operating_temp_m_k,
        10.0, // increased from 5.0 kHz
    );
    let solver_damped = FloquetSyntheticGaugeSolver::new(high_damping);

    let m_base = solver_base.evaluate_metrics();
    let m_damped = solver_damped.evaluate_metrics();

    assert!(
        m_damped.dynamical_state_fidelity < m_base.dynamical_state_fidelity,
        "Higher acoustic damping rate must reduce dynamical state fidelity"
    );
    assert!(
        m_damped.flux_quantization_error > m_base.flux_quantization_error,
        "Higher acoustic damping rate must increase flux quantization error"
    );
}

#[test]
fn test_plaquette_count_scaling() {
    let base = FloquetSyntheticGaugeParams::default();
    let solver_base = FloquetSyntheticGaugeSolver::new(base);

    let larger_lattice = FloquetSyntheticGaugeParams::new(
        base.acoustic_center_freq_ghz,
        base.floquet_drive_freq_mhz,
        base.parametric_modulation_depth,
        36, // increased from 16 plaquettes
        base.synthetic_phase_gradient_rad,
        base.inter_site_coupling_mhz,
        base.operating_temp_m_k,
        base.acoustic_damping_rate_khz,
    );
    let solver_larger = FloquetSyntheticGaugeSolver::new(larger_lattice);

    let m_base = solver_base.evaluate_metrics();
    let m_larger = solver_larger.evaluate_metrics();

    assert!(
        m_larger.chern_switching_time_ns > m_base.chern_switching_time_ns,
        "Larger plaquette count must increase Chern switching transit time"
    );
}

#[test]
fn test_physical_compliance_thresholds() {
    let params = FloquetSyntheticGaugeParams::default();
    let solver = FloquetSyntheticGaugeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
    assert!(metrics.dynamical_state_fidelity >= 0.9950);
    assert!(metrics.synthetic_magnetic_flux_ratio >= 0.500);
    assert!(metrics.flux_quantization_error <= 0.010);
    assert!(metrics.chern_switching_time_ns <= 20.0);
    assert!(metrics.topological_band_isolation_db >= 30.0);
}
