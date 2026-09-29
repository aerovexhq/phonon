//! Automated unit and physical validation tests for Floquet second-order topological
//! phononic corner states and quantum transduction.

use phonon_models::floquet_corner_transduction::CornerTransductionParams;
use phonon_solver::floquet_corner_transduction::FloquetCornerTransductionSolver;

#[test]
fn test_corner_mode_localization_purity() {
    let params = CornerTransductionParams::default();
    let solver = FloquetCornerTransductionSolver::new(params);
    let purity = solver.compute_corner_mode_localization_purity();

    // Target corner localization purity >= 96.0% (0.960)
    assert!(
        purity >= 0.960,
        "Corner mode localization purity must be >= 0.960, got {:.4}",
        purity
    );
    assert!(
        purity <= 1.0,
        "Purity cannot exceed unity, got {:.4}",
        purity
    );
}

#[test]
fn test_bidirectional_transduction_efficiency() {
    let params = CornerTransductionParams::default();
    let solver = FloquetCornerTransductionSolver::new(params);
    let efficiency = solver.compute_bidirectional_transduction_efficiency();

    // Target bidirectional transduction efficiency >= 45.0% (0.450)
    assert!(
        efficiency >= 0.450,
        "Bidirectional transduction efficiency must be >= 0.450, got {:.4}",
        efficiency
    );
    assert!(
        efficiency <= 1.0,
        "Transduction efficiency cannot exceed unity, got {:.4}",
        efficiency
    );
}

#[test]
fn test_added_noise_photons() {
    let params = CornerTransductionParams::default();
    let solver = FloquetCornerTransductionSolver::new(params);
    let n_add = solver.compute_added_noise_photons();

    // Target added noise photons <= 0.20
    assert!(
        n_add <= 0.20,
        "Added quantum noise photons must be <= 0.20, got {:.4}",
        n_add
    );
    assert!(
        n_add > 0.0,
        "Added noise photons must be strictly positive, got {:.4}",
        n_add
    );
}

#[test]
fn test_corner_acoustic_quality_factor() {
    let params = CornerTransductionParams::default();
    let solver = FloquetCornerTransductionSolver::new(params);
    let q_m = solver.compute_corner_acoustic_quality_factor();

    // Target acoustic quality factor >= 1.5e5
    assert!(
        q_m >= 1.5e5,
        "Corner acoustic quality factor must be >= 1.5e5, got {:.2e}",
        q_m
    );
}

#[test]
fn test_quadrupole_topological_invariant() {
    let params_topo = CornerTransductionParams::default();
    let solver_topo = FloquetCornerTransductionSolver::new(params_topo);
    let q_xy_topo = solver_topo.compute_quadrupole_topological_invariant();

    // In topological phase (inter_intra_hopping_ratio > 1.0), q_xy = 0.500
    assert!(
        (q_xy_topo - 0.500).abs() < 1.0e-6,
        "Topological invariant must be quantized to 0.500, got {:.4}",
        q_xy_topo
    );

    // Trivial phase check (ratio < 1.0)
    let mut params_triv = CornerTransductionParams::default();
    params_triv.inter_intra_hopping_ratio = 0.8;
    let solver_triv = FloquetCornerTransductionSolver::new(params_triv);
    let q_xy_triv = solver_triv.compute_quadrupole_topological_invariant();

    assert!(
        q_xy_triv.abs() < 1.0e-6,
        "Trivial phase invariant must be 0.000, got {:.4}",
        q_xy_triv
    );
}

#[test]
fn test_full_corner_transduction_physical_compliance() {
    let params = CornerTransductionParams::default();
    let solver = FloquetCornerTransductionSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default corner transduction configuration must be fully compliant"
    );
    assert!(
        metrics.corner_mode_localization_purity >= 0.960,
        "Localization purity violation"
    );
    assert!(
        metrics.bidirectional_transduction_efficiency >= 0.450,
        "Transduction efficiency violation"
    );
    assert!(
        metrics.added_noise_photons <= 0.20,
        "Added noise photon threshold violation"
    );
    assert!(
        metrics.corner_acoustic_quality_factor >= 1.5e5,
        "Acoustic quality factor violation"
    );
    assert!(
        (metrics.quadrupole_topological_invariant - 0.500).abs() < 1.0e-6,
        "Quadrupole topological invariant quantization violation"
    );
}
