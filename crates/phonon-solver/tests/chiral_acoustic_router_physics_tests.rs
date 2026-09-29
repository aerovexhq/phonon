#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for chiral quantum acoustic
//! metamaterial circulators and multi-terminal non-reciprocal router networks.

use phonon_models::chiral_acoustic_router::ChiralAcousticRouterParams;
use phonon_solver::chiral_acoustic_router::ChiralAcousticRouterSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = ChiralAcousticRouterParams::new(
        0.5,    // below 1.0 GHz
        5.0,    // below 10.0 MHz
        2,      // below 3 ports
        0.005,  // below 0.01
        5.0e4,  // below 1.0e5
        2.0,    // below 5.0 MHz
        0.5,    // below 1.0 mK
        -0.05,  // below 0.0
    );
    assert!((underflow.center_frequency_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.synthetic_angular_momentum_mhz - 10.0).abs() < 1e-9);
    assert_eq!(underflow.ports_count, 3);
    assert!((underflow.odd_viscosity_coefficient - 0.01).abs() < 1e-9);
    assert!((underflow.resonator_q_factor - 1.0e5).abs() < 1e-9);
    assert!((underflow.waveguide_coupling_rate_mhz - 5.0).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.fabrication_disorder_fraction - 0.0).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = ChiralAcousticRouterParams::new(
        20.0,   // above 15.0 GHz
        250.0,  // above 200.0 MHz
        12,     // above 8 ports
        0.80,   // above 0.50
        2.0e8,  // above 1.0e8
        75.0,   // above 50.0 MHz
        75.0,   // above 50.0 mK
        0.25,   // above 0.10
    );
    assert!((overflow.center_frequency_ghz - 15.0).abs() < 1e-9);
    assert!((overflow.synthetic_angular_momentum_mhz - 200.0).abs() < 1e-9);
    assert_eq!(overflow.ports_count, 8);
    assert!((overflow.odd_viscosity_coefficient - 0.50).abs() < 1e-9);
    assert!((overflow.resonator_q_factor - 1.0e8).abs() < 1e-9);
    assert!((overflow.waveguide_coupling_rate_mhz - 50.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.fabrication_disorder_fraction - 0.10).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ChiralAcousticRouterParams::default();
    let solver = ChiralAcousticRouterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.non_reciprocal_isolation_db >= 35.0,
        "Default isolation must be >= 35.0 dB, got {:.3} dB",
        metrics.non_reciprocal_isolation_db
    );
    assert!(
        metrics.insertion_loss_db <= 0.40,
        "Default insertion loss must be <= 0.40 dB, got {:.4} dB",
        metrics.insertion_loss_db
    );
    assert!(
        metrics.phase_coherence_fidelity >= 0.9920,
        "Default phase coherence fidelity must be >= 0.9920, got {:.5}",
        metrics.phase_coherence_fidelity
    );
    assert!(
        metrics.cross_talk_rejection_db >= 30.0,
        "Default cross talk rejection must be >= 30.0 dB, got {:.3} dB",
        metrics.cross_talk_rejection_db
    );
    assert!(
        metrics.operating_bandwidth_mhz >= 12.0,
        "Default operating bandwidth must be >= 12.0 MHz, got {:.3} MHz",
        metrics.operating_bandwidth_mhz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_angular_momentum_scaling() {
    let base = ChiralAcousticRouterParams::default();
    let solver_base = ChiralAcousticRouterSolver::new(base);
    let is_base = solver_base.compute_non_reciprocal_isolation_db();
    let cr_base = solver_base.compute_cross_talk_rejection_db();
    let bw_base = solver_base.compute_operating_bandwidth_mhz();

    // Increased synthetic angular momentum bias modulation
    let high_momentum = ChiralAcousticRouterParams::new(
        base.center_frequency_ghz,
        120.0, // increased from 80.0 MHz
        base.ports_count,
        base.odd_viscosity_coefficient,
        base.resonator_q_factor,
        base.waveguide_coupling_rate_mhz,
        base.operating_temp_m_k,
        base.fabrication_disorder_fraction,
    );
    let solver_high = ChiralAcousticRouterSolver::new(high_momentum);
    let is_high = solver_high.compute_non_reciprocal_isolation_db();
    let cr_high = solver_high.compute_cross_talk_rejection_db();
    let bw_high = solver_high.compute_operating_bandwidth_mhz();

    assert!(
        is_high >= is_base,
        "Higher angular momentum modulation must increase isolation (base: {:.3}, high: {:.3})",
        is_base,
        is_high
    );
    assert!(
        cr_high >= cr_base,
        "Higher angular momentum modulation must increase cross-talk rejection (base: {:.3}, high: {:.3})",
        cr_base,
        cr_high
    );
    assert!(
        bw_high >= bw_base,
        "Higher angular momentum modulation must broaden circulation bandwidth (base: {:.3}, high: {:.3})",
        bw_base,
        bw_high
    );
}

#[test]
fn test_temperature_degradation() {
    let base = ChiralAcousticRouterParams::default();
    let solver_base = ChiralAcousticRouterSolver::new(base);
    let is_base = solver_base.compute_non_reciprocal_isolation_db();
    let il_base = solver_base.compute_insertion_loss_db();
    let fid_base = solver_base.compute_phase_coherence_fidelity();
    let cr_base = solver_base.compute_cross_talk_rejection_db();

    // Warmer cryogenic operating temperature (30.0 mK vs 15.0 mK)
    let warmer = ChiralAcousticRouterParams::new(
        base.center_frequency_ghz,
        base.synthetic_angular_momentum_mhz,
        base.ports_count,
        base.odd_viscosity_coefficient,
        base.resonator_q_factor,
        base.waveguide_coupling_rate_mhz,
        30.0, // elevated temperature
        base.fabrication_disorder_fraction,
    );
    let solver_warmer = ChiralAcousticRouterSolver::new(warmer);
    let is_warm = solver_warmer.compute_non_reciprocal_isolation_db();
    let il_warm = solver_warmer.compute_insertion_loss_db();
    let fid_warm = solver_warmer.compute_phase_coherence_fidelity();
    let cr_warm = solver_warmer.compute_cross_talk_rejection_db();

    assert!(
        is_warm <= is_base,
        "Warmer temperature must decrease isolation (base: {:.3}, warmer: {:.3})",
        is_base,
        is_warm
    );
    assert!(
        il_warm >= il_base,
        "Warmer temperature must increase insertion loss (base: {:.4}, warmer: {:.4})",
        il_base,
        il_warm
    );
    assert!(
        fid_warm <= fid_base,
        "Warmer temperature must degrade phase coherence fidelity (base: {:.5}, warmer: {:.5})",
        fid_base,
        fid_warm
    );
    assert!(
        cr_warm <= cr_base,
        "Warmer temperature must degrade cross-talk rejection (base: {:.3}, warmer: {:.3})",
        cr_base,
        cr_warm
    );
}

#[test]
fn test_disorder_scaling() {
    let base = ChiralAcousticRouterParams::default();
    let solver_base = ChiralAcousticRouterSolver::new(base);
    let is_base = solver_base.compute_non_reciprocal_isolation_db();
    let il_base = solver_base.compute_insertion_loss_db();
    let fid_base = solver_base.compute_phase_coherence_fidelity();
    let cr_base = solver_base.compute_cross_talk_rejection_db();

    // Elevated fabrication disorder fraction (0.06 vs 0.02)
    let disordered = ChiralAcousticRouterParams::new(
        base.center_frequency_ghz,
        base.synthetic_angular_momentum_mhz,
        base.ports_count,
        base.odd_viscosity_coefficient,
        base.resonator_q_factor,
        base.waveguide_coupling_rate_mhz,
        base.operating_temp_m_k,
        0.06, // increased disorder
    );
    let solver_disordered = ChiralAcousticRouterSolver::new(disordered);
    let is_disorder = solver_disordered.compute_non_reciprocal_isolation_db();
    let il_disorder = solver_disordered.compute_insertion_loss_db();
    let fid_disorder = solver_disordered.compute_phase_coherence_fidelity();
    let cr_disorder = solver_disordered.compute_cross_talk_rejection_db();

    assert!(
        is_disorder <= is_base,
        "Higher disorder must decrease isolation (base: {:.3}, disordered: {:.3})",
        is_base,
        is_disorder
    );
    assert!(
        il_disorder >= il_base,
        "Higher disorder must increase insertion loss (base: {:.4}, disordered: {:.4})",
        il_base,
        il_disorder
    );
    assert!(
        fid_disorder <= fid_base,
        "Higher disorder must degrade phase coherence fidelity (base: {:.5}, disordered: {:.5})",
        fid_base,
        fid_disorder
    );
    assert!(
        cr_disorder <= cr_base,
        "Higher disorder must degrade cross-talk rejection (base: {:.3}, disordered: {:.3})",
        cr_base,
        cr_disorder
    );
}

#[test]
fn test_physical_compliance() {
    let params = ChiralAcousticRouterParams::default();
    let solver = ChiralAcousticRouterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
    assert!(metrics.non_reciprocal_isolation_db >= 35.0);
    assert!(metrics.insertion_loss_db <= 0.40);
    assert!(metrics.phase_coherence_fidelity >= 0.9920);
    assert!(metrics.cross_talk_rejection_db >= 30.0);
    assert!(metrics.operating_bandwidth_mhz >= 12.0);
}
