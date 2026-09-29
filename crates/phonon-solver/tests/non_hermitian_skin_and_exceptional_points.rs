//! Integration tests for non-Hermitian skin effect, non-Bloch band theory,
//! boundary skin mode accumulation, and higher-order exceptional points.

use phonon_models::non_hermitian_topo::NonHermitianSkinParams;
use phonon_solver::non_hermitian_topo::NonBlochSkinSolver;

#[test]
fn test_non_hermitian_skin_mode_localization() {
    let params = NonHermitianSkinParams {
        lattice_sites_count: 30,
        forward_hopping_mhz: 2.5,
        backward_hopping_mhz: 0.45,
        onsite_gain_loss_mhz: 1.0,
        perturbation_epsilon: 1.0e-3,
        pump_power_mw: 6.0,
    };
    let solver = NonBlochSkinSolver::new(params);
    let metrics = solver.solve_skin_metrics();

    // Skin depth localization length must be <= 3.0 unit cells
    assert!(
        metrics.skin_depth_unit_cells <= 3.0,
        "Skin depth {:.2} cells must be <= 3.0 cells",
        metrics.skin_depth_unit_cells
    );

    // Boundary localization contrast must be >= 25.0 dB
    assert!(
        metrics.skin_localization_contrast_db >= 25.0,
        "Localization contrast {:.2} dB must be >= 25.0 dB",
        metrics.skin_localization_contrast_db
    );

    // GBZ radius must be strictly less than 1.0
    assert!(
        metrics.gbz_radius < 1.0,
        "GBZ radius {:.4} must be < 1.0",
        metrics.gbz_radius
    );

    // Point-gap spectral winding number must be non-zero
    assert_ne!(metrics.spectral_winding_number, 0);

    // Verify spatial wavefunction distribution
    let wf = solver.solve_skin_wavefunction();
    assert_eq!(wf.len(), 30);

    // Sum of probability densities must be 1.0
    let total_prob: f64 = wf.iter().map(|(_, p)| *p).sum();
    assert!(
        (total_prob - 1.0).abs() < 1e-6,
        "Total probability must be 1.0"
    );

    // Right boundary (j = 29) must have exponentially higher probability than left boundary (j = 0)
    let left_prob = wf.first().unwrap().1;
    let right_prob = wf.last().unwrap().1;
    assert!(
        right_prob > left_prob * 1000.0,
        "Right boundary probability {:.4e} must exceed left {:.4e} by > 1000x",
        right_prob,
        left_prob
    );

    // Verify GBZ contour points
    let gbz = solver.solve_gbz_contour(32);
    assert_eq!(gbz.len(), 32);
    for (re, im) in &gbz {
        let radius = (re * re + im * im).sqrt();
        assert!(
            (radius - metrics.gbz_radius).abs() < 1e-6,
            "Contour radius should match gbz_radius"
        );
    }
}

#[test]
fn test_exceptional_point_sensitivity_enhancement() {
    let params = NonHermitianSkinParams {
        lattice_sites_count: 25,
        forward_hopping_mhz: 2.8,
        backward_hopping_mhz: 0.50,
        onsite_gain_loss_mhz: 1.5,
        perturbation_epsilon: 5.0e-4,
        pump_power_mw: 8.0,
    };
    let solver = NonBlochSkinSolver::new(params);
    let metrics = solver.solve_skin_metrics();

    // Sensitivity enhancement factor must be >= 10.0x
    assert!(
        metrics.sensitivity_enhancement_factor >= 10.0,
        "Sensitivity enhancement {:.1}x must be >= 10.0x",
        metrics.sensitivity_enhancement_factor
    );

    // EP order >= 2
    assert!(
        metrics.exceptional_point_order >= 2,
        "EP order {} must be >= 2",
        metrics.exceptional_point_order
    );

    // Verify sub-linear energy splitting: delta_E(eps) >> eps
    let split = solver.solve_exceptional_point_splitting(1.0e-4);
    assert!(
        split > 1.0e-3,
        "Energy splitting {:.4e} must be significantly larger than perturbation 1e-4",
        split
    );
}
