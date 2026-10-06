#![deny(unsafe_code)]

use phonon_solver::octupole_insulator::{
    CubicCornerId, OctupoleCubicLattice, OctupoleHamiltonian, OctupoleParams, OctupolePhase,
};
use std::f64::consts::PI;

#[test]
fn test_clifford_gamma_matrices_anticommutation() {
    let gammas = OctupoleHamiltonian::clifford_gamma_matrices();

    // Verify 6 matrices are non-zero
    assert_eq!(gammas.len(), 6);

    // Verify anti-commutation relations: {Gamma_i, Gamma_j} = 0 for i != j
    for i in 0..6 {
        for j in 0..6 {
            if i != j {
                // Compute matrix anticommutator: {Gi, Gj} = Gi * Gj + Gj * Gi
                let mut max_anticomm = 0.0;
                for r in 0..8 {
                    for c in 0..8 {
                        let mut term1 = 0.0;
                        let mut term2 = 0.0;
                        for k in 0..8 {
                            term1 += gammas[i][r][k] * gammas[j][k][c];
                            term2 += gammas[j][r][k] * gammas[i][k][c];
                        }
                        let anticomm = (term1 + term2).abs();
                        if anticomm > max_anticomm {
                            max_anticomm = anticomm;
                        }
                    }
                }
                assert!(
                    max_anticomm < 1e-12,
                    "Gamma matrices {} and {} failed to anticommute: max = {:.2e}",
                    i, j, max_anticomm
                );
            }
        }
    }
}

#[test]
fn test_octupole_bulk_dispersion_and_bandgap() {
    let params = OctupoleParams {
        gamma: 2.0,
        lambda: 10.0,
        omega_0: 1.0,
        a_mm: 5.0,
    };
    let solver = OctupoleHamiltonian::new(params);

    assert_eq!(solver.phase(), OctupolePhase::TopologicalOctupole);

    // Analytical bandgap at R(pi, pi, pi): Delta_bulk = 2 * sqrt(3) * |10.0 - 2.0| = 16 * sqrt(3) ~ 27.71 MHz
    let expected_gap = 2.0 * 3.0_f64.sqrt() * 8.0;
    let computed_gap = solver.bulk_bandgap();
    assert!(
        (computed_gap - expected_gap).abs() < 1e-10,
        "Bulk bandgap mismatch: got {:.4}, expected {:.4}",
        computed_gap, expected_gap
    );

    // Eigenvalues at Gamma(0, 0, 0): E = +/- sqrt(3) * (gamma + lambda) = +/- sqrt(3) * 12.0 ~ 20.78 MHz
    let evals_gamma = solver.bulk_eigenvalues_at(0.0, 0.0, 0.0);
    assert_eq!(evals_gamma.len(), 8);
    let expected_e_gamma = 3.0_f64.sqrt() * 12.0;
    assert!((evals_gamma[0].abs() - expected_e_gamma).abs() < 1e-10);
    assert!((evals_gamma[7].abs() - expected_e_gamma).abs() < 1e-10);

    // Eigenvalues at R(pi, pi, pi): E = +/- sqrt(3) * |lambda - gamma| = +/- sqrt(3) * 8.0 ~ 13.86 MHz
    let evals_r = solver.bulk_eigenvalues_at(PI, PI, PI);
    let expected_e_r = 3.0_f64.sqrt() * 8.0;
    assert!((evals_r[0].abs() - expected_e_r).abs() < 1e-10);
    assert!((evals_r[7].abs() - expected_e_r).abs() < 1e-10);

    // High symmetry path generation
    let bands = solver.band_structure(10);
    assert!(bands.len() >= 40);
    for pt in &bands {
        assert!(pt.energies[0] <= pt.energies[7]);
        assert!(pt.energies[7] > 0.0);
        assert!(pt.energies[0] < 0.0);
    }
}

#[test]
fn test_quantized_octupole_and_boundary_moments() {
    let topo_params = OctupoleParams::topological();
    let topo_solver = OctupoleHamiltonian::new(topo_params);
    assert_eq!(topo_solver.quantized_octupole_moment(), 0.5);
    assert_eq!(topo_solver.surface_quadrupole_moment(), 0.5);
    assert_eq!(topo_solver.hinge_dipole_moment(), 0.5);

    let trivial_params = OctupoleParams::trivial();
    let trivial_solver = OctupoleHamiltonian::new(trivial_params);
    assert_eq!(trivial_solver.phase(), OctupolePhase::TrivialInsulator);
    assert_eq!(trivial_solver.quantized_octupole_moment(), 0.0);
    assert_eq!(trivial_solver.surface_quadrupole_moment(), 0.0);
    assert_eq!(trivial_solver.hinge_dipole_moment(), 0.0);
}

#[test]
fn test_finite_cubic_lattice_eight_corner_states() {
    // 2x2x2 unit cells = 8 cells = 64 sites
    let params = OctupoleParams {
        gamma: 1.5,
        lambda: 10.0,
        omega_0: 1.0,
        a_mm: 5.0,
    };
    let lattice = OctupoleCubicLattice::new(params, 2, 2, 2);
    assert_eq!(lattice.total_sites(), 64);

    let result = lattice.solve();
    assert!(result.is_topological);
    assert_eq!(result.total_sites, 64);

    // Exactly 8 localized corner states must be detected at mid-gap
    assert_eq!(
        result.corner_states.len(), 8,
        "Expected 8 mid-gap corner modes, found {}",
        result.corner_states.len()
    );

    // Verify all 8 distinct corners are represented
    let expected_corners = [
        CubicCornerId::Corner000,
        CubicCornerId::Corner100,
        CubicCornerId::Corner010,
        CubicCornerId::Corner110,
        CubicCornerId::Corner001,
        CubicCornerId::Corner101,
        CubicCornerId::Corner011,
        CubicCornerId::Corner111,
    ];
    for (idx, &expected) in expected_corners.iter().enumerate() {
        assert_eq!(result.corner_states[idx].corner_id, expected);
        // Mid-gap energy must be close to zero
        assert!(
            result.corner_states[idx].energy.abs() < 0.1 * result.bulk_bandgap,
            "Corner mode {} energy {:.3} MHz exceeds mid-gap limit",
            idx, result.corner_states[idx].energy
        );
    }

    // Energy confinement across all 8 corners must exceed 85%
    assert!(
        result.energy_confinement_ratio >= 0.85,
        "Corner energy confinement must be >= 85%, got {:.2}%",
        result.energy_confinement_ratio * 100.0
    );

    // Nanocavity quality factor must be >= 1e5
    assert!(
        result.quality_factor >= 1.0e5,
        "Nanocavity Q-factor must be >= 1e5, got {:.1e}",
        result.quality_factor
    );

    // In contrast, in the trivial phase, zero mid-gap corner modes should be detected
    let trivial_lattice = OctupoleCubicLattice::new(OctupoleParams::trivial(), 2, 2, 2);
    let trivial_result = trivial_lattice.solve();
    assert!(!trivial_result.is_topological);
    assert_eq!(trivial_result.corner_states.len(), 0);
}

#[test]
fn test_defect_immunity_against_coupling_disorder() {
    let params = OctupoleParams {
        gamma: 1.5,
        lambda: 10.0,
        omega_0: 1.0,
        a_mm: 5.0,
    };
    let lattice = OctupoleCubicLattice::new(params, 2, 2, 2);

    let disorder_levels = [0.0, 0.5, 1.0];
    let robustness = lattice.evaluate_defect_robustness(&disorder_levels, 42);

    assert_eq!(robustness.len(), 3);
    for pt in &robustness {
        // Corner state energies remain pinned near zero even with disorder
        assert!(
            pt.mean_corner_energy_mhz < 3.0,
            "Mean corner energy at W={:.1} was {:.3} MHz, expected < 3.0 MHz",
            pt.disorder_w, pt.mean_corner_energy_mhz
        );
        // Confinement remains high
        assert!(
            pt.confinement_pct >= 75.0,
            "Confinement at W={:.1} dropped to {:.2}%, expected >= 75%",
            pt.disorder_w, pt.confinement_pct
        );
    }
}
