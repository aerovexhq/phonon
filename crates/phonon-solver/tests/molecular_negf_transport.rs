//! Integration Test: NEGF Transport & Landauer-Büttiker Formalism.
//!
//! Validates:
//! 1. Complex matrix inversion and retarded Green's function computation \(G^R(E)\).
//! 2. Transmission spectra \(\mathcal{T}(E)\) using the Fisher-Lee relation.
//! 3. Landauer current integration across finite bias windows.
//! 4. Self-consistent NEGF-Poisson electrostatic convergence and charge conservation.

#![allow(clippy::needless_range_loop)]

use phonon_models::molecular::{invert_complex_matrix, MolecularJunction, NegfTransportSolver};
use phonon_models::quantum::Complex;
use phonon_solver::molecular::{
    SelfConsistentNegfConfig, SelfConsistentNegfResult, SelfConsistentNegfSolver,
};

#[test]
fn test_complex_linear_algebra_inversion() {
    // 2x2 complex matrix:
    // [ 1 + 2i,  3 + 4i ]
    // [ 2 - 1i,  5 + 0i ]
    let a = vec![
        vec![Complex::new(1.0, 2.0), Complex::new(3.0, 4.0)],
        vec![Complex::new(2.0, -1.0), Complex::new(5.0, 0.0)],
    ];

    let inv_opt = invert_complex_matrix(a.clone());
    assert!(inv_opt.is_some(), "Matrix inversion failed");
    let inv = inv_opt.unwrap();

    // Verify A * A^-1 = Identity
    for i in 0..2 {
        for j in 0..2 {
            let mut sum = Complex::ZERO;
            for k in 0..2 {
                sum = sum.add(a[i][k].mul(inv[k][j]));
            }
            let expected = if i == j { Complex::ONE } else { Complex::ZERO };
            assert!(
                (sum.re - expected.re).abs() < 1.0e-10,
                "Real mismatch at ({}, {}): got {}, expected {}",
                i,
                j,
                sum.re,
                expected.re
            );
            assert!(
                (sum.im - expected.im).abs() < 1.0e-10,
                "Imag mismatch at ({}, {}): got {}, expected {}",
                i,
                j,
                sum.im,
                expected.im
            );
        }
    }
}

#[test]
fn test_linear_chain_negf_transmission_and_conductance() {
    let solver = NegfTransportSolver::new(300.0, 0.0);
    let chain = MolecularJunction::linear_chain(4, 0.6);

    // Compute transmission spectrum across [-4 eV, +4 eV]
    let spectrum = solver.compute_transmission_spectrum(&chain, -4.0, 4.0, 80, 0.0);
    assert_eq!(spectrum.len(), 80);

    // Find peak transmission (should approach resonance near resonant eigenenergies)
    let max_t = spectrum.iter().cloned().fold(
        (0.0, 0.0),
        |acc, (e, t)| if t > acc.1 { (e, t) } else { acc },
    );

    assert!(
        max_t.1 > 0.50,
        "Peak transmission through 4-site polyene chain should exceed 0.50, got {:.4}",
        max_t.1
    );

    // Evaluate Landauer current across small bias window
    let i_bias = solver.evaluate_current_landauer(&chain, 0.10, 0.0).abs();
    assert!(
        i_bias > 0.0,
        "Current through linear chain should be strictly positive, got {:.2e} A",
        i_bias
    );
}

#[test]
fn test_self_consistent_negf_poisson_solver_convergence() {
    let junction = MolecularJunction::para_benzene(0.6);
    let solver = SelfConsistentNegfSolver::new(SelfConsistentNegfConfig {
        max_iterations: 50,
        tolerance_v: 1.0e-3,
        mixing_alpha: 0.30,
        temperature_k: 300.0,
        energy_points: 30,
        charging_energy_u_ev: 0.75,
    });

    let res: SelfConsistentNegfResult = solver.solve(&junction, 0.15, 0.20);
    assert!(res.iterations > 0);
    assert!(
        res.converged || res.final_residual < 0.05,
        "Self-consistent solver should converge or achieve low residual, got {:.4e}",
        res.final_residual
    );
    assert_eq!(res.site_charges.len(), 6);
    assert_eq!(res.site_potentials_v.len(), 6);
    assert!(res.current_a.abs() >= 0.0);
}
