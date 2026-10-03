#![deny(unsafe_code)]

//! Test suite for Phase 346: Non-Hermitian Chiral Exceptional Surface Acoustic Sensing Array.
//!
//! Verifies:
//! - Continuous 2D manifold condition F(gamma_1, gamma_2, kappa) = 0 yielding degenerate coalesced eigenvalues.
//! - Jordan block defectiveness and Petermann factor divergence along the surface.
//! - Fractional perturbation sensitivity scaling Delta lambda ~ sqrt(epsilon) and enhancement > 50x.
//! - Directional chiral acoustic sensing with forward-to-backward directivity >= 25 dB.
//! - Sub-threshold net SNR advantage in the presence of Petermann noise amplification.

use phonon_solver::exceptional_surface::{
    EsComplex, EsManifoldParams, ExceptionalSurfaceArray, ExceptionalSurfaceHamiltonian,
};

#[test]
fn test_exceptional_surface_manifold_condition_and_coalescence() {
    // 1. Point on the Exceptional Surface manifold with gamma_1 = 2.0, gamma_2 = 1.0
    let params_on = EsManifoldParams::on_surface(10.0, 2.0, 1.0, 0.35);
    let solver_on = ExceptionalSurfaceHamiltonian::new(params_on);

    let ev_on = solver_on.solve_eigenvalues();
    assert!(
        ev_on.min_splitting < 0.15,
        "Eigenvalues must coalesce on the exceptional surface: got splitting {} MHz",
        ev_on.min_splitting
    );
    assert!(
        solver_on.is_on_surface(0.15),
        "System must be classified as on the exceptional surface"
    );

    // 2. Point far from the Exceptional Surface (perturbed kappa)
    let mut params_off = params_on;
    params_off.kappa_mhz = 5.0; // Deliberate detuning off the surface
    let solver_off = ExceptionalSurfaceHamiltonian::new(params_off);

    let ev_off = solver_off.solve_eigenvalues();
    assert!(
        ev_off.min_splitting > 0.50,
        "Detuned point must have separated eigenvalues: got splitting {} MHz",
        ev_off.min_splitting
    );
    assert!(
        !solver_off.is_on_surface(0.15),
        "Detuned point must not be classified as on the exceptional surface"
    );
}

#[test]
fn test_jordan_defectiveness_and_petermann_divergence() {
    let params_on = EsManifoldParams::on_surface(10.0, 2.0, 1.0, 0.35);
    let solver_on = ExceptionalSurfaceHamiltonian::new(params_on);

    let mut params_off = params_on;
    params_off.kappa_mhz = 6.0;
    let solver_off = ExceptionalSurfaceHamiltonian::new(params_off);

    let k_on = solver_on.petermann_factor();
    let k_off = solver_off.petermann_factor();

    assert!(
        k_on > 100.0,
        "Petermann factor must diverge near exceptional surface: got {}",
        k_on
    );
    assert!(
        k_off < 50.0,
        "Petermann factor must remain modest far from exceptional surface: got {}",
        k_off
    );
    assert!(
        k_on > k_off * 5.0,
        "On-surface Petermann factor ({}) must substantially exceed off-surface ({})",
        k_on,
        k_off
    );
}

#[test]
fn test_fractional_perturbation_scaling_and_enhancement() {
    let params = EsManifoldParams::on_surface(10.0, 2.0, 1.0, 0.35);
    let solver = ExceptionalSurfaceHamiltonian::new(params);

    // Micro-perturbations: epsilon = 1e-4 and 1e-3
    let eps_small = EsComplex::from_real(1e-4);
    let eps_med = EsComplex::from_real(1e-3);

    let (_dl_s, split_small, eta_small) = solver.perturbation_splitting(eps_small);
    let (_dl_m, split_med, eta_med) = solver.perturbation_splitting(eps_med);

    // Verify significant enhancement factor: eta = |Delta lambda| / |epsilon| > 50x
    assert!(
        eta_small > 50.0,
        "Sensitivity enhancement for 1e-4 must be > 50x: got {}x",
        eta_small
    );
    assert!(
        eta_med > 10.0,
        "Sensitivity enhancement for 1e-3 must be > 10x: got {}x",
        eta_med
    );

    // Verify sub-linear fractional scaling:
    // When epsilon increases by 10x (from 1e-4 to 1e-3),
    // linear response would scale by 10x, but square-root response scales by ~sqrt(10) ~ 3.16x
    let splitting_ratio = split_med / split_small.max(1e-12);
    assert!(
        splitting_ratio < 7.0,
        "Fractional splitting ratio must be sub-linear (< 7.0 for 10x epsilon increase): got {}",
        splitting_ratio
    );
}

#[test]
fn test_directional_chiral_acoustic_sensing_and_directivity() {
    let params = EsManifoldParams::on_surface(10.0, 2.0, 1.0, 0.35);
    let array = ExceptionalSurfaceArray::new(params, 8, 2.5);

    let eps = 1e-3;
    let forward_metrics = array.evaluate_directional_sensing(0.0, eps);
    let backward_metrics = array.evaluate_directional_sensing(180.0, eps);

    assert!(
        forward_metrics.enhancement_factor > backward_metrics.enhancement_factor,
        "Forward enhancement ({}) must exceed backward ({})",
        forward_metrics.enhancement_factor,
        backward_metrics.enhancement_factor
    );

    assert!(
        forward_metrics.directivity_db >= 25.0,
        "Directional directivity must be >= 25.0 dB: got {} dB",
        forward_metrics.directivity_db
    );

    assert!(
        !forward_metrics.polar_response.is_empty(),
        "Polar response curve must be populated"
    );
}

#[test]
fn test_subthreshold_net_snr_advantage() {
    let params = EsManifoldParams::on_surface(10.0, 2.0, 1.0, 0.35);
    let array = ExceptionalSurfaceArray::new(params, 8, 2.5);

    // In the micro-perturbation regime (eps = 1e-4), signal amplification
    // exceeds the square root of the Petermann noise penalty
    let snr_analysis = array.evaluate_snr_advantage(1e-4);

    assert!(
        snr_analysis.signal_enhancement > 50.0,
        "Signal enhancement must be > 50x: got {}",
        snr_analysis.signal_enhancement
    );
    assert!(
        snr_analysis.net_snr_gain > 1.0,
        "Net SNR gain in sub-threshold regime must be > 1.0: got {}",
        snr_analysis.net_snr_gain
    );
    assert!(
        snr_analysis.is_subthreshold_advantage,
        "Operating point must achieve sub-threshold SNR advantage"
    );
}
