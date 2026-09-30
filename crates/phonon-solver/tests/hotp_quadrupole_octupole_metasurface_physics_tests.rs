#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic higher-order
//! topological quadrupole-octupole superlattices and non-Hermitian corner metasurfaces.

use phonon_models::hotp_quadrupole_octupole_metasurface::HotpQuadrupoleOctupoleParams;
use phonon_solver::hotp_quadrupole_octupole_metasurface::HotpQuadrupoleOctupoleSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = HotpQuadrupoleOctupoleParams::new(
        0.05, // below 0.10
        0.50, // below 0.80
        0.005, // below 0.01
        0.5,  // below 1.0 GHz
        0.5,  // below 1.0 mK
        1.5,  // below 2.0
        4.0,  // below 6.0 cells
        0.50, // below 0.80 pi
    );
    assert_eq!(underflow.intra_cell_hopping_gamma, 0.10);
    assert_eq!(underflow.inter_cell_hopping_lambda, 0.80);
    assert_eq!(underflow.non_hermitian_gain_loss_gamma, 0.01);
    assert_eq!(underflow.acoustic_corner_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.multipole_order, 2.0);
    assert_eq!(underflow.superlattice_dimension_cells, 6.0);
    assert_eq!(underflow.synthetic_gauge_flux_pi, 0.80);

    // Test values strictly above physical maximum bounds
    let overflow = HotpQuadrupoleOctupoleParams::new(
        1.50, // above 0.90
        3.50, // above 2.50
        0.80, // above 0.40
        25.0, // above 15.0 GHz
        80.0, // above 50.0 mK
        4.5,  // above 3.0
        45.0, // above 32.0 cells
        1.60, // above 1.20 pi
    );
    assert_eq!(overflow.intra_cell_hopping_gamma, 0.90);
    assert_eq!(overflow.inter_cell_hopping_lambda, 2.50);
    assert_eq!(overflow.non_hermitian_gain_loss_gamma, 0.40);
    assert_eq!(overflow.acoustic_corner_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.multipole_order, 3.0);
    assert_eq!(overflow.superlattice_dimension_cells, 32.0);
    assert_eq!(overflow.synthetic_gauge_flux_pi, 1.20);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = HotpQuadrupoleOctupoleParams::default();
    assert_eq!(params.intra_cell_hopping_gamma, 0.35);
    assert_eq!(params.inter_cell_hopping_lambda, 1.45);
    assert_eq!(params.non_hermitian_gain_loss_gamma, 0.12);
    assert_eq!(params.acoustic_corner_frequency_ghz, 5.2);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.multipole_order, 2.0);
    assert_eq!(params.superlattice_dimension_cells, 12.0);
    assert_eq!(params.synthetic_gauge_flux_pi, 1.0);

    let solver = HotpQuadrupoleOctupoleSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.corner_state_localization_fidelity >= 0.9980,
        "Corner state localization fidelity must be >= 0.9980, got {:.6}",
        metrics.corner_state_localization_fidelity
    );
    assert!(
        metrics.higher_order_topological_gap_mhz >= 45.0,
        "Higher-order topological gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.higher_order_topological_gap_mhz
    );
    assert!(
        metrics.multipole_topological_charge >= 0.990,
        "Multipole topological charge must be >= 0.990, got {:.6}",
        metrics.multipole_topological_charge
    );
    assert!(
        metrics.corner_to_bulk_crosstalk_isolation_db >= 54.0,
        "Corner-to-bulk crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.corner_to_bulk_crosstalk_isolation_db
    );
    assert!(
        metrics.topological_mode_dephasing_rate_hz <= 15.0,
        "Topological mode dephasing rate must be <= 15.0 Hz, got {:.4} Hz",
        metrics.topological_mode_dephasing_rate_hz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_intra_cell_hopping_scaling() {
    let mut low_gamma = HotpQuadrupoleOctupoleParams::default();
    low_gamma.intra_cell_hopping_gamma = 0.15;
    let solver_low = HotpQuadrupoleOctupoleSolver::new(low_gamma);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_gamma = HotpQuadrupoleOctupoleParams::default();
    high_gamma.intra_cell_hopping_gamma = 0.75;
    let solver_high = HotpQuadrupoleOctupoleSolver::new(high_gamma);
    let metrics_high = solver_high.evaluate_metrics();

    // Lower intra-cell hopping sharpens corner localization and widens higher-order gap
    assert!(
        metrics_low.corner_state_localization_fidelity
            > metrics_high.corner_state_localization_fidelity
    );
    assert!(
        metrics_low.higher_order_topological_gap_mhz
            > metrics_high.higher_order_topological_gap_mhz
    );
    assert!(
        metrics_low.corner_to_bulk_crosstalk_isolation_db
            > metrics_high.corner_to_bulk_crosstalk_isolation_db
    );
    assert!(
        metrics_low.topological_mode_dephasing_rate_hz
            < metrics_high.topological_mode_dephasing_rate_hz
    );
}

#[test]
fn test_inter_cell_hopping_scaling() {
    let mut low_lambda = HotpQuadrupoleOctupoleParams::default();
    low_lambda.inter_cell_hopping_lambda = 0.90;
    let solver_low = HotpQuadrupoleOctupoleSolver::new(low_lambda);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_lambda = HotpQuadrupoleOctupoleParams::default();
    high_lambda.inter_cell_hopping_lambda = 2.20;
    let solver_high = HotpQuadrupoleOctupoleSolver::new(high_lambda);
    let metrics_high = solver_high.evaluate_metrics();

    // Higher inter-cell hopping increases localization fidelity and topological gap
    assert!(
        high_lambda.inter_cell_hopping_lambda > low_lambda.inter_cell_hopping_lambda
    );
    assert!(
        metrics_high.corner_state_localization_fidelity
            > metrics_low.corner_state_localization_fidelity
    );
    assert!(
        metrics_high.higher_order_topological_gap_mhz
            > metrics_low.higher_order_topological_gap_mhz
    );
    assert!(
        metrics_high.corner_to_bulk_crosstalk_isolation_db
            > metrics_low.corner_to_bulk_crosstalk_isolation_db
    );
    assert!(
        metrics_high.topological_mode_dephasing_rate_hz
            < metrics_low.topological_mode_dephasing_rate_hz
    );
}

#[test]
fn test_non_hermitian_gain_loss_scaling() {
    let mut low_nh = HotpQuadrupoleOctupoleParams::default();
    low_nh.non_hermitian_gain_loss_gamma = 0.02;
    let solver_low = HotpQuadrupoleOctupoleSolver::new(low_nh);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_nh = HotpQuadrupoleOctupoleParams::default();
    high_nh.non_hermitian_gain_loss_gamma = 0.35;
    let solver_high = HotpQuadrupoleOctupoleSolver::new(high_nh);
    let metrics_high = solver_high.evaluate_metrics();

    // Non-Hermitian skin mode amplification enhances corner localization and isolation
    assert!(
        metrics_high.corner_state_localization_fidelity
            > metrics_low.corner_state_localization_fidelity
    );
    assert!(
        metrics_high.corner_to_bulk_crosstalk_isolation_db
            > metrics_low.corner_to_bulk_crosstalk_isolation_db
    );
    assert!(
        metrics_high.higher_order_topological_gap_mhz
            > metrics_low.higher_order_topological_gap_mhz
    );
}

#[test]
fn test_acoustic_corner_frequency_scaling() {
    let mut low_f = HotpQuadrupoleOctupoleParams::default();
    low_f.acoustic_corner_frequency_ghz = 2.0;
    let solver_low = HotpQuadrupoleOctupoleSolver::new(low_f);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_f = HotpQuadrupoleOctupoleParams::default();
    high_f.acoustic_corner_frequency_ghz = 12.0;
    let solver_high = HotpQuadrupoleOctupoleSolver::new(high_f);
    let metrics_high = solver_high.evaluate_metrics();

    // Higher frequency scales the topological gap and isolation
    assert!(
        metrics_high.higher_order_topological_gap_mhz
            > metrics_low.higher_order_topological_gap_mhz
    );
    assert!(
        metrics_high.corner_to_bulk_crosstalk_isolation_db
            > metrics_low.corner_to_bulk_crosstalk_isolation_db
    );
    assert!(
        metrics_high.topological_mode_dephasing_rate_hz
            < metrics_low.topological_mode_dephasing_rate_hz
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold = HotpQuadrupoleOctupoleParams::default();
    cold.cryogenic_temperature_mk = 2.0;
    let solver_cold = HotpQuadrupoleOctupoleSolver::new(cold);
    let metrics_cold = solver_cold.evaluate_metrics();

    let mut warm = HotpQuadrupoleOctupoleParams::default();
    warm.cryogenic_temperature_mk = 45.0;
    let solver_warm = HotpQuadrupoleOctupoleSolver::new(warm);
    let metrics_warm = solver_warm.evaluate_metrics();

    // Cooler temperatures enhance localization, increase gap, and minimize dephasing
    assert!(
        metrics_cold.corner_state_localization_fidelity
            > metrics_warm.corner_state_localization_fidelity
    );
    assert!(
        metrics_cold.higher_order_topological_gap_mhz
            > metrics_warm.higher_order_topological_gap_mhz
    );
    assert!(
        metrics_cold.multipole_topological_charge
            > metrics_warm.multipole_topological_charge
    );
    assert!(
        metrics_cold.topological_mode_dephasing_rate_hz
            < metrics_warm.topological_mode_dephasing_rate_hz
    );
}

#[test]
fn test_multipole_order_octupole_vs_quadrupole() {
    let mut quadrupole = HotpQuadrupoleOctupoleParams::default();
    quadrupole.multipole_order = 2.0;
    let solver_quad = HotpQuadrupoleOctupoleSolver::new(quadrupole);
    let metrics_quad = solver_quad.evaluate_metrics();

    let mut octupole = HotpQuadrupoleOctupoleParams::default();
    octupole.multipole_order = 3.0;
    let solver_oct = HotpQuadrupoleOctupoleSolver::new(octupole);
    let metrics_oct = solver_oct.evaluate_metrics();

    // Octupole higher codimension protection provides higher isolation and gap
    assert!(
        metrics_oct.corner_to_bulk_crosstalk_isolation_db
            > metrics_quad.corner_to_bulk_crosstalk_isolation_db
    );
    assert!(
        metrics_oct.higher_order_topological_gap_mhz
            > metrics_quad.higher_order_topological_gap_mhz
    );
    assert!(
        metrics_oct.topological_mode_dephasing_rate_hz
            < metrics_quad.topological_mode_dephasing_rate_hz
    );
}

#[test]
fn test_superlattice_dimension_scaling() {
    let mut small_lattice = HotpQuadrupoleOctupoleParams::default();
    small_lattice.superlattice_dimension_cells = 8.0;
    let solver_small = HotpQuadrupoleOctupoleSolver::new(small_lattice);
    let metrics_small = solver_small.evaluate_metrics();

    let mut large_lattice = HotpQuadrupoleOctupoleParams::default();
    large_lattice.superlattice_dimension_cells = 28.0;
    let solver_large = HotpQuadrupoleOctupoleSolver::new(large_lattice);
    let metrics_large = solver_large.evaluate_metrics();

    // Larger superlattice size exponentially increases corner isolation and fidelity
    assert!(
        metrics_large.corner_state_localization_fidelity
            > metrics_small.corner_state_localization_fidelity
    );
    assert!(
        metrics_large.corner_to_bulk_crosstalk_isolation_db
            > metrics_small.corner_to_bulk_crosstalk_isolation_db
    );
    assert!(
        metrics_large.topological_mode_dephasing_rate_hz
            < metrics_small.topological_mode_dephasing_rate_hz
    );
}

#[test]
fn test_synthetic_gauge_flux_pi_tuning() {
    let mut tuned_flux = HotpQuadrupoleOctupoleParams::default();
    tuned_flux.synthetic_gauge_flux_pi = 1.0; // exact pi-flux
    let solver_tuned = HotpQuadrupoleOctupoleSolver::new(tuned_flux);
    let metrics_tuned = solver_tuned.evaluate_metrics();

    let mut detuned_flux = HotpQuadrupoleOctupoleParams::default();
    detuned_flux.synthetic_gauge_flux_pi = 0.85; // detuned from pi
    let solver_detuned = HotpQuadrupoleOctupoleSolver::new(detuned_flux);
    let metrics_detuned = solver_detuned.evaluate_metrics();

    // Exact pi flux maximizes topological charge, topological gap, and reduces dephasing
    assert!(
        metrics_tuned.multipole_topological_charge
            > metrics_detuned.multipole_topological_charge
    );
    assert!(
        metrics_tuned.higher_order_topological_gap_mhz
            > metrics_detuned.higher_order_topological_gap_mhz
    );
    assert!(
        metrics_tuned.corner_state_localization_fidelity
            > metrics_detuned.corner_state_localization_fidelity
    );
    assert!(
        metrics_tuned.topological_mode_dephasing_rate_hz
            < metrics_detuned.topological_mode_dephasing_rate_hz
    );
}
