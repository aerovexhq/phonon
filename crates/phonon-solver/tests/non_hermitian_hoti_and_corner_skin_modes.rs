//! Integration tests for Non-Hermitian Higher-Order Topological Insulators (HOTI),
//! quantized bulk quadrupole polarization, and 0D corner skin mode localization.

use phonon_models::non_hermitian_chiral_hoti::NonHermitianHotiParams;
use phonon_solver::non_hermitian_chiral_hoti::NonHermitianHotiSolver;

#[test]
fn test_default_hoti_corner_metrics() {
    let params = NonHermitianHotiParams::default();
    let solver = NonHermitianHotiSolver::new(params);
    let metrics = solver.solve();

    assert_eq!(metrics.topological_corner_mode_count, 4);
    assert_eq!(metrics.quantized_quadrupole_polarization, 0.5);
    assert!(
        metrics.corner_localization_contrast_db >= 30.0,
        "Contrast {} must be >= 30.0 dB",
        metrics.corner_localization_contrast_db
    );
    assert!(
        metrics.corner_skin_depth_cells <= 2.0,
        "Skin depth {} must be <= 2.0 cells",
        metrics.corner_skin_depth_cells
    );
    assert!(
        metrics.corner_skin_depth_cells >= 0.1,
        "Skin depth {} must be positive and physical",
        metrics.corner_skin_depth_cells
    );
}

#[test]
fn test_topological_phase_transition() {
    // Topologically trivial regime: gamma > lambda
    let trivial_params = NonHermitianHotiParams {
        intra_cell_hopping_mhz: 2.0,
        inter_cell_hopping_mhz: 0.5,
        ..Default::default()
    };
    let solver_trivial = NonHermitianHotiSolver::new(trivial_params);
    let metrics_trivial = solver_trivial.solve();

    assert_eq!(metrics_trivial.quantized_quadrupole_polarization, 0.0);
    assert_eq!(metrics_trivial.topological_corner_mode_count, 0);

    // Topologically non-trivial regime: gamma < lambda
    let topo_params = NonHermitianHotiParams {
        intra_cell_hopping_mhz: 0.3,
        inter_cell_hopping_mhz: 1.5,
        ..Default::default()
    };
    let solver_topo = NonHermitianHotiSolver::new(topo_params);
    let metrics_topo = solver_topo.solve();

    assert_eq!(metrics_topo.quantized_quadrupole_polarization, 0.5);
    assert_eq!(metrics_topo.topological_corner_mode_count, 4);
    assert!(metrics_topo.corner_localization_contrast_db >= 30.0);
}

#[test]
fn test_non_hermitian_asymmetry_skin_depth_scaling() {
    // Increasing non-reciprocal asymmetry eta should decrease the corner skin depth (sharper localization)
    let p_low_eta = NonHermitianHotiParams {
        non_reciprocal_asymmetry_eta: 0.25,
        ..Default::default()
    };
    let p_high_eta = NonHermitianHotiParams {
        non_reciprocal_asymmetry_eta: 0.75,
        ..Default::default()
    };

    let solver_low = NonHermitianHotiSolver::new(p_low_eta);
    let solver_high = NonHermitianHotiSolver::new(p_high_eta);

    let xi_low = solver_low.compute_corner_skin_depth_cells();
    let xi_high = solver_high.compute_corner_skin_depth_cells();

    assert!(
        xi_high < xi_low,
        "Higher non-reciprocal drift should shorten corner skin depth (xi_high={} vs xi_low={})",
        xi_high,
        xi_low
    );
}

#[test]
fn test_lattice_size_scaling_contrast() {
    let p_small = NonHermitianHotiParams {
        grid_dim_x: 12,
        grid_dim_y: 12,
        ..Default::default()
    };
    let p_large = NonHermitianHotiParams {
        grid_dim_x: 32,
        grid_dim_y: 32,
        ..Default::default()
    };

    let solver_small = NonHermitianHotiSolver::new(p_small);
    let solver_large = NonHermitianHotiSolver::new(p_large);

    let contrast_small = solver_small.compute_corner_localization_contrast_db();
    let contrast_large = solver_large.compute_corner_localization_contrast_db();

    assert!(
        contrast_large >= contrast_small,
        "Larger lattice should provide higher corner-to-bulk contrast (large={} vs small={})",
        contrast_large,
        contrast_small
    );
    assert!(contrast_small >= 30.0);
    assert!(contrast_large <= 75.0);
}
