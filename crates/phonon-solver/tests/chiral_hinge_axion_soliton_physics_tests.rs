#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for 3D topological acoustic
//! higher-order axion insulators and chiral hinge soliton networks.

use phonon_models::chiral_hinge_axion_soliton::ChiralHingeAxionSolitonParams;
use phonon_solver::chiral_hinge_axion_soliton::ChiralHingeAxionSolitonSolver;
use std::f64::consts::PI;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = ChiralHingeAxionSolitonParams::new(
        -1.0, // below 0.0 rad
        0.05, // below 0.10
        5.0,  // below 10.0 MHz
        0.10, // below 0.20 ns
        0.0001, // below 0.001
        0.5,  // below 1.0 GHz
        0.5,  // below 1.0 mK
        4,    // below 6
    );
    assert_eq!(underflow.axion_angle_rad, 0.0);
    assert_eq!(underflow.magnetoelectric_coupling_alpha, 0.10);
    assert_eq!(underflow.topological_bulk_gap_mhz, 10.0);
    assert_eq!(underflow.hinge_soliton_pulse_width_ns, 0.20);
    assert_eq!(underflow.acoustic_non_linearity_parameter, 0.001);
    assert_eq!(underflow.operating_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.lattice_dimension_3d, 6);

    // Test values strictly above physical maximum bounds
    let overflow = ChiralHingeAxionSolitonParams::new(
        10.0, // above 2*PI rad
        8.0,  // above 5.0
        150.0, // above 120.0 MHz
        15.0, // above 10.0 ns
        0.15, // above 0.08
        15.0, // above 12.0 GHz
        80.0, // above 50.0 mK
        50,   // above 32
    );
    assert_eq!(overflow.axion_angle_rad, 2.0 * PI);
    assert_eq!(overflow.magnetoelectric_coupling_alpha, 5.0);
    assert_eq!(overflow.topological_bulk_gap_mhz, 120.0);
    assert_eq!(overflow.hinge_soliton_pulse_width_ns, 10.0);
    assert_eq!(overflow.acoustic_non_linearity_parameter, 0.08);
    assert_eq!(overflow.operating_frequency_ghz, 12.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.lattice_dimension_3d, 32);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ChiralHingeAxionSolitonParams::default();
    let solver = ChiralHingeAxionSolitonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.hinge_state_transmission_fidelity >= 0.9970,
        "Default transmission fidelity must be >= 0.9970, got {:.6}",
        metrics.hinge_state_transmission_fidelity
    );
    assert!(
        metrics.topological_axion_gap_mhz >= 25.0,
        "Default topological axion gap must be >= 25.0 MHz, got {:.2} MHz",
        metrics.topological_axion_gap_mhz
    );
    assert!(
        metrics.non_linear_harmonic_distortion_db <= -48.0,
        "Default harmonic distortion must be <= -48.0 dB, got {:.2} dB",
        metrics.non_linear_harmonic_distortion_db
    );
    assert!(
        metrics.inter_hinge_crosstalk_isolation_db >= 46.0,
        "Default inter-hinge isolation must be >= 46.0 dB, got {:.2} dB",
        metrics.inter_hinge_crosstalk_isolation_db
    );
    assert!(
        metrics.hinge_soliton_group_velocity_mps >= 2200.0,
        "Default hinge group velocity must be >= 2200.0 m/s, got {:.2} m/s",
        metrics.hinge_soliton_group_velocity_mps
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_axion_angle_scaling() {
    let base = ChiralHingeAxionSolitonParams::default(); // theta = PI
    let solver_base = ChiralHingeAxionSolitonSolver::new(base);

    let mut detuned_angle = base;
    detuned_angle.axion_angle_rad = 0.5 * PI; // detuned away from PI
    let solver_detuned = ChiralHingeAxionSolitonSolver::new(detuned_angle);

    let fidelity_base = solver_base.compute_hinge_state_transmission_fidelity();
    let fidelity_detuned = solver_detuned.compute_hinge_state_transmission_fidelity();
    assert!(
        fidelity_base > fidelity_detuned,
        "Transmission fidelity at theta = PI ({:.6}) must exceed detuned angle ({:.6})",
        fidelity_base,
        fidelity_detuned
    );

    let gap_base = solver_base.compute_topological_axion_gap_mhz();
    let gap_detuned = solver_detuned.compute_topological_axion_gap_mhz();
    assert!(
        gap_base > gap_detuned,
        "Axion gap at theta = PI ({:.2} MHz) must exceed detuned angle ({:.2} MHz)",
        gap_base,
        gap_detuned
    );
}

#[test]
fn test_magnetoelectric_coupling_scaling() {
    let base = ChiralHingeAxionSolitonParams::default();

    let mut high_coupling = base;
    high_coupling.magnetoelectric_coupling_alpha = 3.5;
    let solver_high = ChiralHingeAxionSolitonSolver::new(high_coupling);

    let mut low_coupling = base;
    low_coupling.magnetoelectric_coupling_alpha = 0.8;
    let solver_low = ChiralHingeAxionSolitonSolver::new(low_coupling);

    let gap_high = solver_high.compute_topological_axion_gap_mhz();
    let gap_low = solver_low.compute_topological_axion_gap_mhz();
    assert!(
        gap_high > gap_low,
        "Higher magnetoelectric coupling must produce larger axion gap: {:.2} vs {:.2} MHz",
        gap_high,
        gap_low
    );

    let iso_high = solver_high.compute_inter_hinge_crosstalk_isolation_db();
    let iso_low = solver_low.compute_inter_hinge_crosstalk_isolation_db();
    assert!(
        iso_high > iso_low,
        "Higher magnetoelectric coupling must enhance inter-hinge isolation: {:.2} vs {:.2} dB",
        iso_high,
        iso_low
    );
}

#[test]
fn test_topological_bulk_gap_scaling() {
    let base = ChiralHingeAxionSolitonParams::default();

    let mut large_gap = base;
    large_gap.topological_bulk_gap_mhz = 80.0;
    let solver_large = ChiralHingeAxionSolitonSolver::new(large_gap);

    let mut small_gap = base;
    small_gap.topological_bulk_gap_mhz = 20.0;
    let solver_small = ChiralHingeAxionSolitonSolver::new(small_gap);

    let axion_gap_large = solver_large.compute_topological_axion_gap_mhz();
    let axion_gap_small = solver_small.compute_topological_axion_gap_mhz();
    assert!(
        axion_gap_large > axion_gap_small,
        "Larger bulk gap must produce larger axion gap: {:.2} vs {:.2} MHz",
        axion_gap_large,
        axion_gap_small
    );

    let iso_large = solver_large.compute_inter_hinge_crosstalk_isolation_db();
    let iso_small = solver_small.compute_inter_hinge_crosstalk_isolation_db();
    assert!(
        iso_large > iso_small,
        "Larger bulk gap must increase inter-hinge crosstalk isolation: {:.2} vs {:.2} dB",
        iso_large,
        iso_small
    );
}

#[test]
fn test_soliton_nonlinearity_scaling() {
    let base = ChiralHingeAxionSolitonParams::default();

    let mut high_beta = base;
    high_beta.acoustic_non_linearity_parameter = 0.050;
    let solver_high = ChiralHingeAxionSolitonSolver::new(high_beta);

    let mut low_beta = base;
    low_beta.acoustic_non_linearity_parameter = 0.005;
    let solver_low = ChiralHingeAxionSolitonSolver::new(low_beta);

    let nlhd_high = solver_high.compute_non_linear_harmonic_distortion_db();
    let nlhd_low = solver_low.compute_non_linear_harmonic_distortion_db();
    assert!(
        nlhd_high > nlhd_low,
        "Higher non-linearity must increase distortion (less negative dB): {:.2} vs {:.2} dB",
        nlhd_high,
        nlhd_low
    );
}

#[test]
fn test_cryogenic_temperature_degradation() {
    let base = ChiralHingeAxionSolitonParams::default();

    let mut cold = base;
    cold.cryogenic_temperature_mk = 5.0;
    let solver_cold = ChiralHingeAxionSolitonSolver::new(cold);

    let mut warm = base;
    warm.cryogenic_temperature_mk = 40.0;
    let solver_warm = ChiralHingeAxionSolitonSolver::new(warm);

    let fidelity_cold = solver_cold.compute_hinge_state_transmission_fidelity();
    let fidelity_warm = solver_warm.compute_hinge_state_transmission_fidelity();
    assert!(
        fidelity_cold > fidelity_warm,
        "Colder temperature must provide higher transmission fidelity: {:.6} vs {:.6}",
        fidelity_cold,
        fidelity_warm
    );

    let iso_cold = solver_cold.compute_inter_hinge_crosstalk_isolation_db();
    let iso_warm = solver_warm.compute_inter_hinge_crosstalk_isolation_db();
    assert!(
        iso_cold > iso_warm,
        "Colder temperature must provide higher inter-hinge isolation: {:.2} vs {:.2} dB",
        iso_cold,
        iso_warm
    );
}

#[test]
fn test_lattice_dimension_crosstalk_scaling() {
    let base = ChiralHingeAxionSolitonParams::default();

    let mut large_lattice = base;
    large_lattice.lattice_dimension_3d = 26;
    let solver_large = ChiralHingeAxionSolitonSolver::new(large_lattice);

    let mut small_lattice = base;
    small_lattice.lattice_dimension_3d = 8;
    let solver_small = ChiralHingeAxionSolitonSolver::new(small_lattice);

    let iso_large = solver_large.compute_inter_hinge_crosstalk_isolation_db();
    let iso_small = solver_small.compute_inter_hinge_crosstalk_isolation_db();
    assert!(
        iso_large > iso_small,
        "Larger lattice dimension must provide higher crosstalk isolation: {:.2} vs {:.2} dB",
        iso_large,
        iso_small
    );
}

#[test]
fn test_operating_frequency_group_velocity_scaling() {
    let base = ChiralHingeAxionSolitonParams::default();

    let mut high_freq = base;
    high_freq.operating_frequency_ghz = 9.0;
    let solver_high = ChiralHingeAxionSolitonSolver::new(high_freq);

    let mut low_freq = base;
    low_freq.operating_frequency_ghz = 2.0;
    let solver_low = ChiralHingeAxionSolitonSolver::new(low_freq);

    let vg_high = solver_high.compute_hinge_soliton_group_velocity_mps();
    let vg_low = solver_low.compute_hinge_soliton_group_velocity_mps();
    assert!(
        vg_high > vg_low,
        "Higher operating frequency must increase hinge group velocity: {:.2} vs {:.2} m/s",
        vg_high,
        vg_low
    );
}
