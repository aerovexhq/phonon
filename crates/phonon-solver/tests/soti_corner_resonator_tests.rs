#![deny(unsafe_code)]

//! Test suite for Phase 347: SOTI Quadrupole Lattice BBH Hamiltonian & Corner Nanocavity Engine.

use phonon_solver::soti_corner_resonator::{
    BbhHamiltonian, CornerId, QuadrupoleParams, SotiLattice,
};
use std::f64::consts::PI;

#[test]
fn test_bbh_bulk_bandgap_and_band_structure() {
    let params = QuadrupoleParams::new(2.0, 10.0, 1.0, 5.0);
    let bbh = BbhHamiltonian::new(params);

    // Bulk bandgap Delta_bulk = 2 * |lambda - gamma| = 2 * |10.0 - 2.0| = 16.0 MHz
    let delta = bbh.bulk_bandgap();
    assert!((delta - 16.0).abs() < 1e-10, "Bulk bandgap should be 16.0 MHz, got {delta}");

    // At Gamma (0, 0): E = +/- sqrt( (gamma+lambda)^2 + (gamma+lambda)^2 ) = +/- sqrt( 2 * 12^2 ) = +/- 12*sqrt(2) approx +/- 16.97 MHz
    let ev_gamma = bbh.eigenvalues(0.0, 0.0);
    let expected_gamma_mag = (2.0 * 12.0 * 12.0_f64).sqrt();
    assert!((ev_gamma[0].abs() - expected_gamma_mag).abs() < 1e-6);
    assert!((ev_gamma[3].abs() - expected_gamma_mag).abs() < 1e-6);

    // At M (pi, pi): E = +/- sqrt( 2 * (lambda-gamma)^2 ) = +/- sqrt( 2 * 8^2 ) = +/- 8*sqrt(2) approx +/- 11.31 MHz
    let ev_m = bbh.eigenvalues(PI, PI);
    let expected_m_mag = (2.0 * 8.0 * 8.0_f64).sqrt();
    assert!((ev_m[0].abs() - expected_m_mag).abs() < 1e-6);
    assert!((ev_m[3].abs() - expected_m_mag).abs() < 1e-6);

    // Compute band dispersion along BZ path
    let dispersion = bbh.compute_band_dispersion(25);
    assert!(dispersion.len() > 80, "Should generate points along all 4 segments");

    // Check first and last points are Gamma
    assert_eq!(dispersion.first().unwrap().symmetry_label, Some("Gamma"));
    assert_eq!(dispersion.last().unwrap().symmetry_label, Some("Gamma"));

    // Check all eigenvalues along path satisfy bulk gap isolation: |E| >= delta_bulk / 2
    for pt in &dispersion {
        assert!(pt.eigenvalues[2] >= 0.0);
        assert!(pt.eigenvalues[1] <= 0.0);
        assert!(pt.eigenvalues[2] >= (params.lambda - params.gamma) - 1e-6);
    }
}

#[test]
fn test_quantized_quadrupole_moment() {
    // Topological phase: gamma < lambda
    let topo_params = QuadrupoleParams::new(2.0, 10.0, 1.0, 5.0);
    let topo_bbh = BbhHamiltonian::new(topo_params);
    assert!(topo_bbh.is_topological());
    assert!((topo_bbh.quadrupole_moment() - 0.5).abs() < 1e-12);
    let (px, py) = topo_bbh.edge_dipole_moments();
    assert!((px - 0.5).abs() < 1e-12);
    assert!((py - 0.5).abs() < 1e-12);

    // Trivial phase: gamma > lambda
    let triv_params = QuadrupoleParams::new(10.0, 2.0, 1.0, 5.0);
    let triv_bbh = BbhHamiltonian::new(triv_params);
    assert!(!triv_bbh.is_topological());
    assert!((triv_bbh.quadrupole_moment() - 0.0).abs() < 1e-12);
    let (px_triv, py_triv) = triv_bbh.edge_dipole_moments();
    assert!((px_triv - 0.0).abs() < 1e-12);
    assert!((py_triv - 0.0).abs() < 1e-12);
}

#[test]
fn test_midgap_corner_states_in_finite_lattice() {
    let params = QuadrupoleParams::new(2.0, 10.0, 1.0, 5.0);
    let lattice = SotiLattice::new(5, 5, params);
    assert_eq!(lattice.total_sites(), 4 * 5 * 5); // 100 sites

    let res = lattice.solve();
    assert!(res.is_topological);
    assert_eq!(res.corner_states.len(), 4, "Must identify exactly 4 mid-gap corner states");

    // Check all 4 corner states have energy |E| < 0.05 * Delta_bulk (16.0 * 0.05 = 0.8 MHz)
    let threshold = 0.05 * res.bulk_bandgap;
    for cs in &res.corner_states {
        assert!(
            cs.energy.abs() < threshold,
            "Corner mode energy {} must be below mid-gap threshold {}",
            cs.energy,
            threshold
        );
        // Corner mode energy should be tightly pinned near zero
        assert!(
            cs.energy.abs() < 0.01,
            "Corner mode energy {} should be pinned near zero",
            cs.energy
        );
    }

    // Verify all 4 distinct corners are present
    let mut corners_found = std::collections::HashSet::new();
    for cs in &res.corner_states {
        corners_found.insert(cs.corner_id);
    }
    assert!(corners_found.contains(&CornerId::BottomLeft));
    assert!(corners_found.contains(&CornerId::BottomRight));
    assert!(corners_found.contains(&CornerId::TopLeft));
    assert!(corners_found.contains(&CornerId::TopRight));
}

#[test]
fn test_corner_spatial_energy_confinement() {
    // Topological lattice: 5x5 cells
    let topo_params = QuadrupoleParams::new(2.0, 10.0, 1.0, 5.0);
    let topo_lattice = SotiLattice::new(5, 5, topo_params);
    let topo_res = topo_lattice.solve();

    assert!(
        topo_res.energy_confinement_ratio >= 0.80,
        "Energy confinement ratio in 4 corner unit cells must be >= 80%, got {:.2}%",
        topo_res.energy_confinement_ratio * 100.0
    );

    // Each individual corner state should have > 80% localization in its corner
    for cs in &topo_res.corner_states {
        assert!(
            cs.localization_ratio >= 0.80,
            "Individual corner state {:?} localization ratio must be >= 80%, got {:.2}%",
            cs.corner_id,
            cs.localization_ratio * 100.0
        );
    }

    // Localization decay length xi = a / ln(lambda/gamma) = 5.0 / ln(5.0) approx 3.106 mm
    assert!((topo_res.localization_length_mm - (5.0 / 5.0_f64.ln())).abs() < 1e-4);

    // Quality factor estimation Q >= 1e4 up to 1e6
    assert!(
        topo_res.quality_factor >= 1.0e4 && topo_res.quality_factor <= 1.0e6,
        "Quality factor must be between 1e4 and 1e6, got {}",
        topo_res.quality_factor
    );

    // Trivial lattice: gamma > lambda
    let triv_params = QuadrupoleParams::new(10.0, 2.0, 1.0, 5.0);
    let triv_lattice = SotiLattice::new(5, 5, triv_params);
    let triv_res = triv_lattice.solve();

    assert!(!triv_res.is_topological);
    // In trivial phase, there are no isolated in-gap corner states
    assert_eq!(triv_res.corner_states.len(), 0);
    assert_eq!(triv_res.energy_confinement_ratio, 0.0);
}

#[test]
fn test_topological_defect_immunity_against_disorder() {
    let params = QuadrupoleParams::new(2.0, 10.0, 1.0, 5.0);
    let lattice = SotiLattice::new(5, 5, params);

    // Add random disorder W = 0.5 MHz to all hopping couplings
    let disorder_w = 0.5;
    let seed = 42;
    let res_disordered = lattice.solve_with_disorder(disorder_w, seed);

    assert_eq!(res_disordered.corner_states.len(), 4);

    // Confirm that corner states remain pinned near mid-gap (|E_corner| << Delta_bulk)
    let threshold = 0.05 * res_disordered.bulk_bandgap;
    for cs in &res_disordered.corner_states {
        assert!(
            cs.energy.abs() < threshold,
            "Disordered corner state energy {} must remain well below mid-gap threshold {}",
            cs.energy,
            threshold
        );
        // Confinement should still be robustly high (> 75%)
        assert!(
            cs.all_corners_confinement > 0.75,
            "Disordered corner state must preserve high confinement: {}",
            cs.all_corners_confinement
        );
    }
}
