#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for chiral acoustic axion electrodynamics
//! and dynamic magnetoelectric phonon circulators.

use phonon_models::chiral_axion_circulator::ChiralAxionCirculatorParams;
use phonon_solver::chiral_axion_circulator::ChiralAxionCirculatorSolver;
use std::f64::consts::PI;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = ChiralAxionCirculatorParams::new(
        0.5,    // below 1.0
        0.01,   // below 0.05
        0.5,    // below 1.0 GHz
        0.5,    // below 1.0 mK
        10.0,   // below 20.0 nm
        80.0,   // below 100.0 deg
        0.05,   // below 0.1 uW
        5.0e3,  // below 1.0e4
    );
    assert_eq!(underflow.axion_coupling_constant_theta, 1.0);
    assert_eq!(underflow.magnetoelectric_polarizability_alpha, 0.05);
    assert_eq!(underflow.acoustic_circulation_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.magnetic_heterostructure_thickness_nm, 20.0);
    assert_eq!(underflow.inter_port_angular_spacing_deg, 100.0);
    assert_eq!(underflow.acoustic_power_drive_uw, 0.1);
    assert_eq!(underflow.cavity_resonance_quality_factor, 1.0e4);

    // Test values strictly above physical maximum bounds
    let overflow = ChiralAxionCirculatorParams::new(
        5.0,    // above 3.5
        1.5,    // above 0.95
        25.0,   // above 15.0 GHz
        80.0,   // above 50.0 mK
        350.0,  // above 250.0 nm
        180.0,  // above 140.0 deg
        100.0,  // above 50.0 uW
        8.0e5,  // above 5.0e5
    );
    assert_eq!(overflow.axion_coupling_constant_theta, 3.5);
    assert_eq!(overflow.magnetoelectric_polarizability_alpha, 0.95);
    assert_eq!(overflow.acoustic_circulation_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.magnetic_heterostructure_thickness_nm, 250.0);
    assert_eq!(overflow.inter_port_angular_spacing_deg, 140.0);
    assert_eq!(overflow.acoustic_power_drive_uw, 50.0);
    assert_eq!(overflow.cavity_resonance_quality_factor, 5.0e5);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ChiralAxionCirculatorParams::default();
    assert!((params.axion_coupling_constant_theta - PI).abs() < 1.0e-12);
    assert_eq!(params.magnetoelectric_polarizability_alpha, 0.65);
    assert_eq!(params.acoustic_circulation_frequency_ghz, 4.6);
    assert_eq!(params.cryogenic_temperature_mk, 15.0);
    assert_eq!(params.magnetic_heterostructure_thickness_nm, 85.0);
    assert_eq!(params.inter_port_angular_spacing_deg, 120.0);
    assert_eq!(params.acoustic_power_drive_uw, 5.0);
    assert_eq!(params.cavity_resonance_quality_factor, 8.5e4);

    let solver = ChiralAxionCirculatorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.non_reciprocal_isolation_db >= 52.0,
        "Non-reciprocal isolation must be >= 52.0 dB, got {:.4} dB",
        metrics.non_reciprocal_isolation_db
    );
    assert!(
        metrics.axion_polariton_transmission_fidelity >= 0.9970,
        "Axion polariton transmission fidelity must be >= 0.9970, got {:.6}",
        metrics.axion_polariton_transmission_fidelity
    );
    assert!(
        metrics.circulator_insertion_loss_db <= 0.35,
        "Circulator insertion loss must be <= 0.35 dB, got {:.4} dB",
        metrics.circulator_insertion_loss_db
    );
    assert!(
        metrics.axionic_phase_stability_error_rad <= 0.0018,
        "Axionic phase stability error must be <= 0.0018 rad, got {:.6} rad",
        metrics.axionic_phase_stability_error_rad
    );
    assert!(
        metrics.harmonic_distortion_suppression_db >= 54.0,
        "Harmonic distortion suppression must be >= 54.0 dB, got {:.4} dB",
        metrics.harmonic_distortion_suppression_db
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_axion_coupling_constant_scaling() {
    let mut params_low = ChiralAxionCirculatorParams::default();
    params_low.axion_coupling_constant_theta = 1.2;
    let mut params_high = ChiralAxionCirculatorParams::default();
    params_high.axion_coupling_constant_theta = 3.4;

    let solver_low = ChiralAxionCirculatorSolver::new(params_low);
    let solver_high = ChiralAxionCirculatorSolver::new(params_high);

    // Higher axion coupling boosts isolation, transmission fidelity, and lowers insertion loss
    assert!(
        solver_high.compute_non_reciprocal_isolation_db()
            > solver_low.compute_non_reciprocal_isolation_db()
    );
    assert!(
        solver_high.compute_axion_polariton_transmission_fidelity()
            > solver_low.compute_axion_polariton_transmission_fidelity()
    );
    assert!(
        solver_high.compute_circulator_insertion_loss_db()
            < solver_low.compute_circulator_insertion_loss_db()
    );
}

#[test]
fn test_magnetoelectric_polarizability_scaling() {
    let mut params_low = ChiralAxionCirculatorParams::default();
    params_low.magnetoelectric_polarizability_alpha = 0.15;
    let mut params_high = ChiralAxionCirculatorParams::default();
    params_high.magnetoelectric_polarizability_alpha = 0.85;

    let solver_low = ChiralAxionCirculatorSolver::new(params_low);
    let solver_high = ChiralAxionCirculatorSolver::new(params_high);

    // Higher magnetoelectric polarizability boosts isolation and fidelity, reduces phase error
    assert!(
        solver_high.compute_non_reciprocal_isolation_db()
            > solver_low.compute_non_reciprocal_isolation_db()
    );
    assert!(
        solver_high.compute_axion_polariton_transmission_fidelity()
            > solver_low.compute_axion_polariton_transmission_fidelity()
    );
    assert!(
        solver_high.compute_axionic_phase_stability_error_rad()
            < solver_low.compute_axionic_phase_stability_error_rad()
    );
}

#[test]
fn test_acoustic_circulation_frequency_scaling() {
    let mut params_low = ChiralAxionCirculatorParams::default();
    params_low.acoustic_circulation_frequency_ghz = 2.0;
    let mut params_high = ChiralAxionCirculatorParams::default();
    params_high.acoustic_circulation_frequency_ghz = 12.0;

    let solver_low = ChiralAxionCirculatorSolver::new(params_low);
    let solver_high = ChiralAxionCirculatorSolver::new(params_high);

    // Higher circulation frequency boosts non-reciprocal isolation and harmonic distortion suppression
    assert!(
        solver_high.compute_non_reciprocal_isolation_db()
            > solver_low.compute_non_reciprocal_isolation_db()
    );
    assert!(
        solver_high.compute_harmonic_distortion_suppression_db()
            > solver_low.compute_harmonic_distortion_suppression_db()
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut params_low = ChiralAxionCirculatorParams::default();
    params_low.cryogenic_temperature_mk = 2.0;
    let mut params_high = ChiralAxionCirculatorParams::default();
    params_high.cryogenic_temperature_mk = 45.0;

    let solver_low = ChiralAxionCirculatorSolver::new(params_low);
    let solver_high = ChiralAxionCirculatorSolver::new(params_high);

    // Higher temperature degrades fidelity, increases insertion loss and phase stability error
    assert!(
        solver_high.compute_axion_polariton_transmission_fidelity()
            < solver_low.compute_axion_polariton_transmission_fidelity()
    );
    assert!(
        solver_high.compute_circulator_insertion_loss_db()
            > solver_low.compute_circulator_insertion_loss_db()
    );
    assert!(
        solver_high.compute_axionic_phase_stability_error_rad()
            > solver_low.compute_axionic_phase_stability_error_rad()
    );
}

#[test]
fn test_magnetic_heterostructure_thickness_scaling() {
    let mut params_thin = ChiralAxionCirculatorParams::default();
    params_thin.magnetic_heterostructure_thickness_nm = 30.0;
    let mut params_thick = ChiralAxionCirculatorParams::default();
    params_thick.magnetic_heterostructure_thickness_nm = 220.0;

    let solver_thin = ChiralAxionCirculatorSolver::new(params_thin);
    let solver_thick = ChiralAxionCirculatorSolver::new(params_thick);

    // Thicker heterostructure improves boundary mode confinement, increasing isolation and reducing phase error
    assert!(
        solver_thick.compute_non_reciprocal_isolation_db()
            > solver_thin.compute_non_reciprocal_isolation_db()
    );
    assert!(
        solver_thick.compute_axionic_phase_stability_error_rad()
            < solver_thin.compute_axionic_phase_stability_error_rad()
    );
}

#[test]
fn test_inter_port_angular_spacing_symmetry() {
    let mut params_sym = ChiralAxionCirculatorParams::default();
    params_sym.inter_port_angular_spacing_deg = 120.0; // exact 3-fold symmetry
    let mut params_asym = ChiralAxionCirculatorParams::default();
    params_asym.inter_port_angular_spacing_deg = 105.0; // asymmetric

    let solver_sym = ChiralAxionCirculatorSolver::new(params_sym);
    let solver_asym = ChiralAxionCirculatorSolver::new(params_asym);

    // Symmetric 120-degree spacing maximizes isolation and minimizes insertion loss
    assert!(
        solver_sym.compute_non_reciprocal_isolation_db()
            > solver_asym.compute_non_reciprocal_isolation_db()
    );
    assert!(
        solver_sym.compute_circulator_insertion_loss_db()
            < solver_asym.compute_circulator_insertion_loss_db()
    );
}

#[test]
fn test_acoustic_power_drive_scaling() {
    let mut params_low = ChiralAxionCirculatorParams::default();
    params_low.acoustic_power_drive_uw = 1.0;
    let mut params_high = ChiralAxionCirculatorParams::default();
    params_high.acoustic_power_drive_uw = 40.0;

    let solver_low = ChiralAxionCirculatorSolver::new(params_low);
    let solver_high = ChiralAxionCirculatorSolver::new(params_high);

    // Higher drive power improves dynamic isolation and harmonic distortion suppression within linear range
    assert!(
        solver_high.compute_non_reciprocal_isolation_db()
            > solver_low.compute_non_reciprocal_isolation_db()
    );
    assert!(
        solver_high.compute_harmonic_distortion_suppression_db()
            > solver_low.compute_harmonic_distortion_suppression_db()
    );
}

#[test]
fn test_cavity_quality_factor_scaling() {
    let mut params_low_q = ChiralAxionCirculatorParams::default();
    params_low_q.cavity_resonance_quality_factor = 2.0e4;
    let mut params_high_q = ChiralAxionCirculatorParams::default();
    params_high_q.cavity_resonance_quality_factor = 4.0e5;

    let solver_low_q = ChiralAxionCirculatorSolver::new(params_low_q);
    let solver_high_q = ChiralAxionCirculatorSolver::new(params_high_q);

    // Higher cavity Q factor suppresses insertion loss and enhances harmonic distortion suppression
    assert!(
        solver_high_q.compute_circulator_insertion_loss_db()
            < solver_low_q.compute_circulator_insertion_loss_db()
    );
    assert!(
        solver_high_q.compute_harmonic_distortion_suppression_db()
            > solver_low_q.compute_harmonic_distortion_suppression_db()
    );
    assert!(
        solver_high_q.compute_axionic_phase_stability_error_rad()
            < solver_low_q.compute_axionic_phase_stability_error_rad()
    );
}
