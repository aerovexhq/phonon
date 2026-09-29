#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic
//! topological time crystals and Floquet-symmetry-enriched phononic memories.

use phonon_models::topological_time_crystal::TopologicalTimeCrystalParams;
use phonon_solver::topological_time_crystal::TopologicalTimeCrystalSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = TopologicalTimeCrystalParams::new(
        0.05,   // below 0.1 us
        0.0005, // below 0.001
        2.0,    // below 5.0 MHz
        5.0,    // below 10.0 MHz
        0.5,    // below 1.0 Hz
        0.5,    // below 1.0 mK
        4,      // below 8
        1,      // below 2
    );
    assert!((underflow.floquet_drive_period_us - 0.1).abs() < 1e-9);
    assert!((underflow.imperfect_pulse_rotation_error - 0.001).abs() < 1e-9);
    assert!((underflow.inter_resonator_interaction_mhz - 5.0).abs() < 1e-9);
    assert!((underflow.disorder_potential_strength_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.acoustic_loss_rate_hz - 1.0).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert_eq!(underflow.phononic_chain_length, 8);
    assert_eq!(underflow.subharmonic_period_multiplier, 2);

    // Test values above physical maximum bounds
    let overflow = TopologicalTimeCrystalParams::new(
        15.0,  // above 10.0 us
        0.20,  // above 0.10
        100.0, // above 80.0 MHz
        200.0, // above 150.0 MHz
        60.0,  // above 50.0 Hz
        70.0,  // above 50.0 mK
        100,   // above 64
        10,    // above 4
    );
    assert!((overflow.floquet_drive_period_us - 10.0).abs() < 1e-9);
    assert!((overflow.imperfect_pulse_rotation_error - 0.10).abs() < 1e-9);
    assert!((overflow.inter_resonator_interaction_mhz - 80.0).abs() < 1e-9);
    assert!((overflow.disorder_potential_strength_mhz - 150.0).abs() < 1e-9);
    assert!((overflow.acoustic_loss_rate_hz - 50.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert_eq!(overflow.phononic_chain_length, 64);
    assert_eq!(overflow.subharmonic_period_multiplier, 4);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TopologicalTimeCrystalParams::default();
    let solver = TopologicalTimeCrystalSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.time_crystalline_order_fidelity >= 0.9960,
        "Default time-crystalline order fidelity must be >= 0.9960, got {:.6}",
        metrics.time_crystalline_order_fidelity
    );
    assert!(
        metrics.subharmonic_locking_error <= 0.0020,
        "Default subharmonic locking error must be <= 0.0020, got {:.6}",
        metrics.subharmonic_locking_error
    );
    assert!(
        metrics.temporal_crystalline_lifetime_ms >= 100.0,
        "Default temporal crystalline lifetime must be >= 100.0 ms, got {:.4} ms",
        metrics.temporal_crystalline_lifetime_ms
    );
    assert!(
        metrics.memory_retention_isolation_db >= 45.0,
        "Default memory retention isolation must be >= 45.0 dB, got {:.4} dB",
        metrics.memory_retention_isolation_db
    );
    assert!(
        metrics.many_body_localization_ratio >= 0.920,
        "Default many-body localization ratio must be >= 0.920, got {:.6}",
        metrics.many_body_localization_ratio
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_disorder_scaling() {
    let base = TopologicalTimeCrystalParams::default();
    let solver_base = TopologicalTimeCrystalSolver::new(base);

    let high_disorder = TopologicalTimeCrystalParams::new(
        base.floquet_drive_period_us,
        base.imperfect_pulse_rotation_error,
        base.inter_resonator_interaction_mhz,
        95.0, // increased from 65.0 MHz
        base.acoustic_loss_rate_hz,
        base.operating_temp_m_k,
        base.phononic_chain_length,
        base.subharmonic_period_multiplier,
    );
    let solver_high_disorder = TopologicalTimeCrystalSolver::new(high_disorder);

    let m_base = solver_base.evaluate_metrics();
    let m_high = solver_high_disorder.evaluate_metrics();

    assert!(
        m_high.time_crystalline_order_fidelity >= m_base.time_crystalline_order_fidelity,
        "Higher disorder potential strength must enhance time-crystalline order fidelity"
    );
    assert!(
        m_high.temporal_crystalline_lifetime_ms > m_base.temporal_crystalline_lifetime_ms,
        "Higher disorder potential strength must prolong temporal crystalline lifetime"
    );
    assert!(
        m_high.memory_retention_isolation_db > m_base.memory_retention_isolation_db,
        "Higher disorder potential strength must increase memory retention isolation"
    );
    assert!(
        m_high.many_body_localization_ratio > m_base.many_body_localization_ratio,
        "Higher disorder potential strength must increase many-body localization ratio"
    );
}

#[test]
fn test_pulse_rotation_error_scaling() {
    let base = TopologicalTimeCrystalParams::default();
    let solver_base = TopologicalTimeCrystalSolver::new(base);

    let low_error = TopologicalTimeCrystalParams::new(
        base.floquet_drive_period_us,
        0.01, // lower than 0.02
        base.inter_resonator_interaction_mhz,
        base.disorder_potential_strength_mhz,
        base.acoustic_loss_rate_hz,
        base.operating_temp_m_k,
        base.phononic_chain_length,
        base.subharmonic_period_multiplier,
    );
    let solver_low_error = TopologicalTimeCrystalSolver::new(low_error);
    let m_low = solver_low_error.evaluate_metrics();

    let high_error = TopologicalTimeCrystalParams::new(
        base.floquet_drive_period_us,
        0.04, // increased from 0.02
        base.inter_resonator_interaction_mhz,
        base.disorder_potential_strength_mhz,
        base.acoustic_loss_rate_hz,
        base.operating_temp_m_k,
        base.phononic_chain_length,
        base.subharmonic_period_multiplier,
    );
    let solver_high_error = TopologicalTimeCrystalSolver::new(high_error);

    let m_base = solver_base.evaluate_metrics();
    let m_high = solver_high_error.evaluate_metrics();

    assert!(
        m_base.subharmonic_locking_error > m_low.subharmonic_locking_error,
        "Higher pulse rotation error must strictly increase subharmonic locking error from low-error base"
    );
    assert!(
        m_high.time_crystalline_order_fidelity < m_base.time_crystalline_order_fidelity,
        "Higher pulse rotation error must degrade time-crystalline order fidelity"
    );
    assert!(
        m_high.subharmonic_locking_error >= m_base.subharmonic_locking_error,
        "Higher pulse rotation error must increase or saturate subharmonic frequency locking error"
    );
}

#[test]
fn test_temperature_degradation() {
    let base = TopologicalTimeCrystalParams::default();
    let solver_base = TopologicalTimeCrystalSolver::new(base);

    let cold = TopologicalTimeCrystalParams::new(
        base.floquet_drive_period_us,
        base.imperfect_pulse_rotation_error,
        base.inter_resonator_interaction_mhz,
        base.disorder_potential_strength_mhz,
        base.acoustic_loss_rate_hz,
        5.0, // colder than 10.0 mK
        base.phononic_chain_length,
        base.subharmonic_period_multiplier,
    );
    let solver_cold = TopologicalTimeCrystalSolver::new(cold);
    let m_cold = solver_cold.evaluate_metrics();

    let warm = TopologicalTimeCrystalParams::new(
        base.floquet_drive_period_us,
        base.imperfect_pulse_rotation_error,
        base.inter_resonator_interaction_mhz,
        base.disorder_potential_strength_mhz,
        base.acoustic_loss_rate_hz,
        25.0, // warmed from 10.0 mK
        base.phononic_chain_length,
        base.subharmonic_period_multiplier,
    );
    let solver_warm = TopologicalTimeCrystalSolver::new(warm);

    let m_base = solver_base.evaluate_metrics();
    let m_warm = solver_warm.evaluate_metrics();

    assert!(
        m_base.subharmonic_locking_error > m_cold.subharmonic_locking_error,
        "Higher temperature must strictly increase subharmonic locking error from cryogenic cold base"
    );
    assert!(
        m_warm.time_crystalline_order_fidelity < m_base.time_crystalline_order_fidelity,
        "Elevated temperature must degrade time-crystalline order fidelity"
    );
    assert!(
        m_warm.subharmonic_locking_error >= m_base.subharmonic_locking_error,
        "Elevated temperature must increase or saturate subharmonic frequency locking error"
    );
    assert!(
        m_warm.temporal_crystalline_lifetime_ms < m_base.temporal_crystalline_lifetime_ms,
        "Elevated temperature must shorten temporal crystalline lifetime"
    );
    assert!(
        m_warm.memory_retention_isolation_db < m_base.memory_retention_isolation_db,
        "Elevated temperature must degrade memory retention isolation"
    );
    assert!(
        m_warm.many_body_localization_ratio < m_base.many_body_localization_ratio,
        "Elevated temperature must decrease many-body localization ratio"
    );
}

#[test]
fn test_acoustic_loss_scaling() {
    let base = TopologicalTimeCrystalParams::default();
    let solver_base = TopologicalTimeCrystalSolver::new(base);

    let high_loss = TopologicalTimeCrystalParams::new(
        base.floquet_drive_period_us,
        base.imperfect_pulse_rotation_error,
        base.inter_resonator_interaction_mhz,
        base.disorder_potential_strength_mhz,
        16.0, // increased from 8.0 Hz
        base.operating_temp_m_k,
        base.phononic_chain_length,
        base.subharmonic_period_multiplier,
    );
    let solver_high_loss = TopologicalTimeCrystalSolver::new(high_loss);

    let m_base = solver_base.evaluate_metrics();
    let m_high_loss = solver_high_loss.evaluate_metrics();

    assert!(
        m_high_loss.temporal_crystalline_lifetime_ms < m_base.temporal_crystalline_lifetime_ms,
        "Higher acoustic dissipation loss rate must shorten temporal crystalline lifetime"
    );
}

#[test]
fn test_physical_compliance_thresholds() {
    let params = TopologicalTimeCrystalParams::default();
    let solver = TopologicalTimeCrystalSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
    assert!(metrics.time_crystalline_order_fidelity >= 0.9960);
    assert!(metrics.subharmonic_locking_error <= 0.0020);
    assert!(metrics.temporal_crystalline_lifetime_ms >= 100.0);
    assert!(metrics.memory_retention_isolation_db >= 45.0);
    assert!(metrics.many_body_localization_ratio >= 0.920);
}
