#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Hermitian
//! higher-order topological phononic lasers and chiral quadrupole acoustical frequency synthesizers.

use phonon_models::non_hermitian_quadrupole_laser::NonHermitianQuadrupoleLaserParams;
use phonon_solver::non_hermitian_quadrupole_laser::NonHermitianQuadrupoleLaserSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = NonHermitianQuadrupoleLaserParams::new(
        2.0,    // below 10.0 kHz
        1.0,    // below 10.0 kHz
        1.0,    // below 5.0 MHz
        0.20,   // below 0.50
        0.5,    // below 1.0 GHz
        0.1,    // below 1.0 mK
        0.0001, // below 0.001
        1,      // below 4
    );
    assert!((underflow.pump_gain_rate_khz - 10.0).abs() < 1e-9);
    assert!((underflow.loss_dissipation_rate_khz - 10.0).abs() < 1e-9);
    assert!((underflow.quadrupole_coupling_mhz - 5.0).abs() < 1e-9);
    assert!((underflow.corner_confinement_factor - 0.50).abs() < 1e-9);
    assert!((underflow.acoustic_resonator_frequency_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.cryogenic_temp_mk - 1.0).abs() < 1e-9);
    assert!((underflow.non_linear_saturation_parameter - 0.001).abs() < 1e-9);
    assert_eq!(underflow.lattice_dimension, 4);

    // Test values above physical maximum bounds
    let overflow = NonHermitianQuadrupoleLaserParams::new(
        800.0, // above 500.0 kHz
        750.0, // above 500.0 kHz
        95.0,  // above 60.0 MHz
        1.25,  // above 0.99
        20.0,  // above 12.0 GHz
        120.0, // above 50.0 mK
        0.15,  // above 0.05
        64,    // above 32
    );
    assert!((overflow.pump_gain_rate_khz - 500.0).abs() < 1e-9);
    assert!((overflow.loss_dissipation_rate_khz - 500.0).abs() < 1e-9);
    assert!((overflow.quadrupole_coupling_mhz - 60.0).abs() < 1e-9);
    assert!((overflow.corner_confinement_factor - 0.99).abs() < 1e-9);
    assert!((overflow.acoustic_resonator_frequency_ghz - 12.0).abs() < 1e-9);
    assert!((overflow.cryogenic_temp_mk - 50.0).abs() < 1e-9);
    assert!((overflow.non_linear_saturation_parameter - 0.05).abs() < 1e-9);
    assert_eq!(overflow.lattice_dimension, 32);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = NonHermitianQuadrupoleLaserParams::default();
    let solver = NonHermitianQuadrupoleLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.corner_mode_lasing_fidelity >= 0.9970,
        "Default corner mode lasing fidelity must be >= 0.9970, got {:.6}",
        metrics.corner_mode_lasing_fidelity
    );
    assert!(
        metrics.fractional_frequency_instability <= 1.5e-12,
        "Default fractional frequency instability must be <= 1.5e-12, got {:.3e}",
        metrics.fractional_frequency_instability
    );
    assert!(
        metrics.side_mode_suppression_ratio_db >= 45.0,
        "Default side mode suppression ratio must be >= 45.0 dB, got {:.2} dB",
        metrics.side_mode_suppression_ratio_db
    );
    assert!(
        metrics.topological_corner_mode_lifetime_ms >= 80.0,
        "Default corner mode lifetime must be >= 80.0 ms, got {:.2} ms",
        metrics.topological_corner_mode_lifetime_ms
    );
    assert!(
        metrics.pt_symmetry_confinement_ratio >= 0.920,
        "Default PT symmetry confinement ratio must be >= 0.920, got {:.4}",
        metrics.pt_symmetry_confinement_ratio
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_quadrupole_coupling_scaling() {
    let base = NonHermitianQuadrupoleLaserParams::default();
    let solver_base = NonHermitianQuadrupoleLaserSolver::new(base);

    let high_coupling = NonHermitianQuadrupoleLaserParams::new(
        base.pump_gain_rate_khz,
        base.loss_dissipation_rate_khz,
        45.0, // increased from 28.0 MHz
        base.corner_confinement_factor,
        base.acoustic_resonator_frequency_ghz,
        base.cryogenic_temp_mk,
        base.non_linear_saturation_parameter,
        base.lattice_dimension,
    );
    let solver_high = NonHermitianQuadrupoleLaserSolver::new(high_coupling);

    let base_smsr = solver_base.compute_side_mode_suppression_ratio_db();
    let high_smsr = solver_high.compute_side_mode_suppression_ratio_db();
    assert!(
        high_smsr > base_smsr,
        "Stronger quadrupole coupling must increase side-mode suppression ratio ({} -> {})",
        base_smsr,
        high_smsr
    );

    let base_instability = solver_base.compute_fractional_frequency_instability();
    let high_instability = solver_high.compute_fractional_frequency_instability();
    assert!(
        high_instability < base_instability,
        "Stronger quadrupole coupling must reduce fractional frequency instability ({} -> {})",
        base_instability,
        high_instability
    );
}

#[test]
fn test_corner_confinement_scaling() {
    let base = NonHermitianQuadrupoleLaserParams::default();
    let solver_base = NonHermitianQuadrupoleLaserSolver::new(base);

    let high_conf = NonHermitianQuadrupoleLaserParams::new(
        base.pump_gain_rate_khz,
        base.loss_dissipation_rate_khz,
        base.quadrupole_coupling_mhz,
        0.95, // increased from 0.88
        base.acoustic_resonator_frequency_ghz,
        base.cryogenic_temp_mk,
        base.non_linear_saturation_parameter,
        base.lattice_dimension,
    );
    let solver_high = NonHermitianQuadrupoleLaserSolver::new(high_conf);

    let base_lifetime = solver_base.compute_topological_corner_mode_lifetime_ms();
    let high_lifetime = solver_high.compute_topological_corner_mode_lifetime_ms();
    assert!(
        high_lifetime > base_lifetime,
        "Higher corner confinement must increase corner mode lifetime ({} -> {})",
        base_lifetime,
        high_lifetime
    );

    let base_pt = solver_base.compute_pt_symmetry_confinement_ratio();
    let high_pt = solver_high.compute_pt_symmetry_confinement_ratio();
    assert!(
        high_pt > base_pt,
        "Higher corner confinement must enhance PT symmetry confinement ratio ({} -> {})",
        base_pt,
        high_pt
    );
}

#[test]
fn test_gain_loss_balance_scaling() {
    let base = NonHermitianQuadrupoleLaserParams::default();

    // Exact balance: pump_gain = loss_dissipation = 115.0 kHz
    let balanced = NonHermitianQuadrupoleLaserParams::new(
        115.0,
        115.0,
        base.quadrupole_coupling_mhz,
        base.corner_confinement_factor,
        base.acoustic_resonator_frequency_ghz,
        base.cryogenic_temp_mk,
        base.non_linear_saturation_parameter,
        base.lattice_dimension,
    );
    let solver_balanced = NonHermitianQuadrupoleLaserSolver::new(balanced);

    // Highly imbalanced: pump_gain = 250.0, loss = 50.0
    let imbalanced = NonHermitianQuadrupoleLaserParams::new(
        250.0,
        50.0,
        base.quadrupole_coupling_mhz,
        base.corner_confinement_factor,
        base.acoustic_resonator_frequency_ghz,
        base.cryogenic_temp_mk,
        base.non_linear_saturation_parameter,
        base.lattice_dimension,
    );
    let solver_imbalanced = NonHermitianQuadrupoleLaserSolver::new(imbalanced);

    let pt_balanced = solver_balanced.compute_pt_symmetry_confinement_ratio();
    let pt_imbalanced = solver_imbalanced.compute_pt_symmetry_confinement_ratio();
    assert!(
        pt_balanced > pt_imbalanced,
        "Exact PT gain-loss balance must yield higher PT confinement ratio than large imbalance ({} vs {})",
        pt_balanced,
        pt_imbalanced
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let base = NonHermitianQuadrupoleLaserParams::default();
    let solver_base = NonHermitianQuadrupoleLaserSolver::new(base);

    let warm = NonHermitianQuadrupoleLaserParams::new(
        base.pump_gain_rate_khz,
        base.loss_dissipation_rate_khz,
        base.quadrupole_coupling_mhz,
        base.corner_confinement_factor,
        base.acoustic_resonator_frequency_ghz,
        40.0, // elevated from 15.0 mK
        base.non_linear_saturation_parameter,
        base.lattice_dimension,
    );
    let solver_warm = NonHermitianQuadrupoleLaserSolver::new(warm);

    let base_instability = solver_base.compute_fractional_frequency_instability();
    let warm_instability = solver_warm.compute_fractional_frequency_instability();
    assert!(
        warm_instability > base_instability,
        "Elevated cryogenic temperature must increase fractional frequency instability ({} -> {})",
        base_instability,
        warm_instability
    );

    let base_lifetime = solver_base.compute_topological_corner_mode_lifetime_ms();
    let warm_lifetime = solver_warm.compute_topological_corner_mode_lifetime_ms();
    assert!(
        warm_lifetime < base_lifetime,
        "Elevated temperature must decrease corner mode coherence lifetime ({} -> {})",
        base_lifetime,
        warm_lifetime
    );
}

#[test]
fn test_lattice_dimension_scaling() {
    let base = NonHermitianQuadrupoleLaserParams::default();
    let solver_base = NonHermitianQuadrupoleLaserSolver::new(base);

    let large_lattice = NonHermitianQuadrupoleLaserParams::new(
        base.pump_gain_rate_khz,
        base.loss_dissipation_rate_khz,
        base.quadrupole_coupling_mhz,
        base.corner_confinement_factor,
        base.acoustic_resonator_frequency_ghz,
        base.cryogenic_temp_mk,
        base.non_linear_saturation_parameter,
        24, // increased from 12
    );
    let solver_large = NonHermitianQuadrupoleLaserSolver::new(large_lattice);

    let base_smsr = solver_base.compute_side_mode_suppression_ratio_db();
    let large_smsr = solver_large.compute_side_mode_suppression_ratio_db();
    assert!(
        large_smsr > base_smsr,
        "Larger lattice dimension must increase side-mode suppression ratio ({} -> {})",
        base_smsr,
        large_smsr
    );

    let base_lifetime = solver_base.compute_topological_corner_mode_lifetime_ms();
    let large_lifetime = solver_large.compute_topological_corner_mode_lifetime_ms();
    assert!(
        large_lifetime > base_lifetime,
        "Larger lattice dimension must enhance corner mode lifetime ({} -> {})",
        base_lifetime,
        large_lifetime
    );
}
