#![deny(unsafe_code)]

use phonon_solver::aah_quasicrystal::{
    solve_jacobi, solve_symmetric_tridiagonal, AahHamiltonian, AahLatticeEngine, AahParams,
    AahPhase, GOLDEN_RATIO_CONJUGATE,
};

#[test]
fn test_self_dual_transition_ipr_scaling() {
    // 1. Extended phase: Delta / J = 4.0 / 5.0 = 0.8 < 2.0
    let params_ext = AahParams::extended_phase();
    let engine_ext = AahLatticeEngine::new(params_ext, 60);

    assert_eq!(AahHamiltonian::new(params_ext).phase(), AahPhase::DelocalizedExtended);
    assert!(
        engine_ext.metrics.mean_ipr < 0.06,
        "Extended phase mean IPR must be small (< 0.06), got {:.4}",
        engine_ext.metrics.mean_ipr
    );
    assert!(
        engine_ext.metrics.fraction_extended_pct >= 50.0,
        "Extended state fraction must be >= 50%, got {:.1}%",
        engine_ext.metrics.fraction_extended_pct
    );

    // 2. Localized phase: Delta / J = 16.0 / 5.0 = 3.2 > 2.0
    let params_loc = AahParams::localized_phase();
    let engine_loc = AahLatticeEngine::new(params_loc, 60);

    assert_eq!(AahHamiltonian::new(params_loc).phase(), AahPhase::ExponentiallyLocalized);
    assert!(
        engine_loc.metrics.mean_ipr > 0.09,
        "Localized phase mean IPR must be large (> 0.09), got {:.4}",
        engine_loc.metrics.mean_ipr
    );
    assert!(
        engine_loc.metrics.fraction_localized_pct >= 50.0,
        "Localized state fraction must be >= 50%, got {:.1}%",
        engine_loc.metrics.fraction_localized_pct
    );

    // 3. Critical phase: Delta / J = 10.0 / 5.0 = 2.0 (Self-Dual point)
    let params_crit = AahParams::critical_phase();
    let engine_crit = AahLatticeEngine::new(params_crit, 60);

    assert_eq!(AahHamiltonian::new(params_crit).phase(), AahPhase::CriticalMultifractal);
    assert!(
        engine_crit.metrics.fractal_dimension_d2 > 0.25 && engine_crit.metrics.fractal_dimension_d2 < 0.85,
        "Critical fractal dimension D2 must be intermediate (~0.5), got {:.3}",
        engine_crit.metrics.fractal_dimension_d2
    );
}

#[test]
fn test_generalized_aah_mobility_edge() {
    let params = AahParams::mobility_edge_phase();
    let hamiltonian = AahHamiltonian::new(params);

    assert_eq!(hamiltonian.phase(), AahPhase::MobilityEdgeCoexistence);

    let analytical_ec = hamiltonian.mobility_edge_energy().expect("Must have analytical Ec");
    // Ec = 2 * (5.0 - 3.0) / 0.45 ~ 8.89 MHz
    let expected_ec = 2.0 * (5.0 - 3.0) / 0.45;
    assert!((analytical_ec - expected_ec).abs() < 1e-6);

    let engine = AahLatticeEngine::new(params, 80);
    assert!(
        engine.metrics.mobility_edge_detected,
        "Mobility edge coexistence must be detected in generalized AAH"
    );
    assert!(engine.metrics.fraction_localized_pct > 5.0);
    assert!(engine.metrics.fraction_extended_pct > 5.0);

    // Verify presence of both low-IPR (< 0.05) and high-IPR (> 0.10) states
    let has_extended = engine.eigenstates.iter().any(|s| s.ipr < 0.05);
    let has_localized = engine.eigenstates.iter().any(|s| s.ipr > 0.10);
    assert!(has_extended, "Must contain extended eigenstates");
    assert!(has_localized, "Must contain localized eigenstates");
}

#[test]
fn test_hofstadter_butterfly_spectrum() {
    let params = AahParams::default();
    let hamiltonian = AahHamiltonian::new(params);

    let butterfly = hamiltonian.generate_hofstadter_butterfly(30, 40);
    assert_eq!(butterfly.len(), 30);

    let max_bound = params.modulation_delta + 2.0 * params.hopping_j + 1.0;

    for pt in &butterfly {
        assert_eq!(pt.energies.len(), 40);
        // All energies must lie within tight physical spectral bounds
        for &e in &pt.energies {
            assert!(
                e.abs() <= max_bound,
                "Energy {:.3} exceeded theoretical spectral bound {:.3}",
                e, max_bound
            );
        }
        // Verify energies sorted ascending
        for i in 0..(pt.energies.len() - 1) {
            assert!(pt.energies[i] <= pt.energies[i + 1]);
        }
    }
}

#[test]
fn test_topological_phason_edge_states() {
    let params = AahParams {
        hopping_j: 5.0,
        modulation_delta: 7.0,
        incommensurate_beta: GOLDEN_RATIO_CONJUGATE,
        phason_phi: 0.0,
        ..Default::default()
    };
    let hamiltonian = AahHamiltonian::new(params);

    // Scan phason angle phi in [0, 2*pi]
    let phason_data = hamiltonian.generate_phason_spectrum(20, 50);
    assert_eq!(phason_data.len(), 20);

    // Check finite lattice engine detects boundary states
    let engine = AahLatticeEngine::new(params, 60);
    assert!(
        engine.metrics.boundary_edge_state_count > 0,
        "Incommensurate lattice must support boundary-localized states"
    );
}

#[test]
fn test_symmetric_tridiagonal_solver_accuracy() {
    let n = 25;
    let mut diag = Vec::with_capacity(n);
    let mut offdiag = Vec::with_capacity(n - 1);

    for i in 0..n {
        diag.push(4.0 * (i as f64 * 0.4).cos());
        if i + 1 < n {
            offdiag.push(2.5 + 0.5 * (i as f64 * 0.3).sin());
        }
    }

    // 1. Solve with tridiagonal QL
    let evals_tri = solve_symmetric_tridiagonal(&diag, &offdiag);

    // 2. Solve with full cyclic Jacobi
    let mut full_mat = vec![0.0; n * n];
    for i in 0..n {
        full_mat[i * n + i] = diag[i];
        if i + 1 < n {
            full_mat[i * n + (i + 1)] = offdiag[i];
            full_mat[(i + 1) * n + i] = offdiag[i];
        }
    }
    let (evals_jacobi, _) = solve_jacobi(&full_mat, n);

    assert_eq!(evals_tri.len(), n);
    assert_eq!(evals_jacobi.len(), n);

    for i in 0..n {
        let diff = (evals_tri[i] - evals_jacobi[i]).abs();
        assert!(
            diff < 1e-4,
            "Tridiagonal QL and Jacobi eigensolver diverged at mode {}: tri={:.5}, jacobi={:.5}, diff={:.2e}",
            i, evals_tri[i], evals_jacobi[i], diff
        );
    }
}
