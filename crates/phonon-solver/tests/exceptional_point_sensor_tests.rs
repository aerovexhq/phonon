#![deny(unsafe_code)]

//! Test suite for Phase 334: Non-Hermitian Exceptional Point Sensor & PT-Symmetric Circuit Simulator.
//!
//! Verifies:
//! - EP2, EP3, EP4 eigenvalue coalescence at zero perturbation.
//! - Jordan block structure, nilpotency, and Petermann excess noise factor divergence.
//! - Power-law fractional perturbation scaling: Delta lambda ~ epsilon^(1/2) for EP2,
//!   epsilon^(1/3) for EP3, and epsilon^(1/4) for EP4.
//! - PT-symmetric circuit phase transitions from Exact to Broken phases.
//! - Non-Hermitian Skin Effect (NHSE) directional boundary localization.

use phonon_solver::ep_sensor::{
    Complex, EpOrder, NhseLattice, NonHermitianHamiltonian, PtCircuitParams, PtPhase,
};

#[test]
fn test_ep_eigenvalue_coalescence_at_zero_perturbation() {
    // 1. Order-2 Exceptional Point (EP2)
    let lambda_0_ep2 = Complex::new(100.0, 0.0);
    let ep2 = NonHermitianHamiltonian::new(EpOrder::EP2, lambda_0_ep2);
    let evs2 = ep2.solve_eigenvalues(Complex::ZERO);
    assert_eq!(evs2.len(), 2);
    for ev in &evs2 {
        assert!(
            (*ev - lambda_0_ep2).norm() < 1e-12,
            "EP2 eigenvalue {:?} must coalesce to lambda_0 {:?}",
            ev,
            lambda_0_ep2
        );
    }
    assert!(
        ep2.eigenvalue_splitting(Complex::ZERO) < 1e-12,
        "EP2 splitting must be zero at unperturbed EP"
    );

    // 2. Order-3 Exceptional Point (EP3)
    let lambda_0_ep3 = Complex::new(50.0, 12.5);
    let ep3 = NonHermitianHamiltonian::new(EpOrder::EP3, lambda_0_ep3);
    let evs3 = ep3.solve_eigenvalues(Complex::ZERO);
    assert_eq!(evs3.len(), 3);
    for ev in &evs3 {
        assert!(
            (*ev - lambda_0_ep3).norm() < 1e-12,
            "EP3 eigenvalue {:?} must coalesce to lambda_0 {:?}",
            ev,
            lambda_0_ep3
        );
    }
    assert!(
        ep3.eigenvalue_splitting(Complex::ZERO) < 1e-12,
        "EP3 splitting must be zero at unperturbed EP"
    );

    // 3. Order-4 Exceptional Point (EP4)
    let lambda_0_ep4 = Complex::new(25.0, -8.0);
    let ep4 = NonHermitianHamiltonian::new(EpOrder::EP4, lambda_0_ep4);
    let evs4 = ep4.solve_eigenvalues(Complex::ZERO);
    assert_eq!(evs4.len(), 4);
    for ev in &evs4 {
        assert!(
            (*ev - lambda_0_ep4).norm() < 1e-12,
            "EP4 eigenvalue {:?} must coalesce to lambda_0 {:?}",
            ev,
            lambda_0_ep4
        );
    }
    assert!(
        ep4.eigenvalue_splitting(Complex::ZERO) < 1e-12,
        "EP4 splitting must be zero at unperturbed EP"
    );
}

#[test]
fn test_jordan_block_structure_and_petermann_factor_divergence() {
    let lambda_0 = Complex::new(10.0, 0.0);
    let coupling = 2.0;
    let ep = NonHermitianHamiltonian::new_with_coupling(EpOrder::EP3, lambda_0, coupling);

    // Verify Jordan block structure: diagonal = lambda_0, superdiagonal = coupling
    let mat = ep.matrix_ep();
    assert_eq!(mat.len(), 3);
    assert_eq!(mat[0][0], lambda_0);
    assert_eq!(mat[1][1], lambda_0);
    assert_eq!(mat[2][2], lambda_0);
    assert_eq!(mat[0][1], Complex::from_real(coupling));
    assert_eq!(mat[1][2], Complex::from_real(coupling));
    assert_eq!(mat[1][0], Complex::ZERO);
    assert_eq!(mat[2][0], Complex::ZERO);
    assert_eq!(mat[2][1], Complex::ZERO);

    // Verify nilpotency: J = H_EP - lambda_0 * I
    let mut j = vec![vec![Complex::ZERO; 3]; 3];
    for i in 0..3 {
        for k in 0..3 {
            j[i][k] = mat[i][k];
        }
        j[i][i] = j[i][i] - lambda_0;
    }

    // J^2 is non-zero
    let mut j2 = vec![vec![Complex::ZERO; 3]; 3];
    for i in 0..3 {
        for k in 0..3 {
            for l in 0..3 {
                j2[i][k] = j2[i][k] + (j[i][l] * j[l][k]);
            }
        }
    }
    assert_eq!(j2[0][2], Complex::from_real(coupling * coupling));

    // J^3 is strictly zero
    let mut j3 = vec![vec![Complex::ZERO; 3]; 3];
    for i in 0..3 {
        for k in 0..3 {
            for l in 0..3 {
                j3[i][k] = j3[i][k] + (j2[i][l] * j[l][k]);
            }
        }
    }
    for i in 0..3 {
        for k in 0..3 {
            assert!(
                j3[i][k].norm() < 1e-12,
                "J^3 must be identically zero for EP3"
            );
        }
    }

    // Petermann factor divergence:
    // At epsilon = 0, Petermann factor is infinite
    let k_zero = ep.petermann_factor(Complex::ZERO);
    assert!(
        k_zero.is_infinite(),
        "Petermann factor at exact EP must diverge to infinity"
    );

    // For small perturbations, K is very large and diverges as epsilon -> 0
    let eps_small = Complex::new(1e-4, 0.0);
    let eps_large = Complex::new(1e-2, 0.0);
    let k_small = ep.petermann_factor(eps_small);
    let k_large = ep.petermann_factor(eps_large);

    assert!(
        k_small > 100.0,
        "Near-EP Petermann factor {} must be large",
        k_small
    );
    assert!(
        k_small > k_large,
        "Petermann factor must increase as perturbation vanishes: k_small={}, k_large={}",
        k_small,
        k_large
    );
}

#[test]
fn test_power_law_fractional_perturbation_scaling() {
    // 1. EP2: Delta lambda ~ epsilon^(1/2)
    let ep2 = NonHermitianHamiltonian::new(EpOrder::EP2, Complex::ZERO);
    let eps1_ep2 = Complex::new(1e-4, 0.0);
    let eps2_ep2 = Complex::new(1e-2, 0.0);
    let split1_ep2 = ep2.eigenvalue_splitting(eps1_ep2);
    let split2_ep2 = ep2.eigenvalue_splitting(eps2_ep2);

    let exponent_ep2 = (split2_ep2 / split1_ep2).ln() / (eps2_ep2.norm() / eps1_ep2.norm()).ln();
    assert!(
        (exponent_ep2 - 0.50).abs() < 1e-3,
        "EP2 splitting exponent {} must be 0.50 (square root)",
        exponent_ep2
    );

    // Sensitivity enhancement over Hermitian baseline:
    // Baseline Hermitian sensitivity is 1.0 (linear Delta lambda = epsilon)
    // EP2 enhancement = split / epsilon = (2 * sqrt(1e-4)) / 1e-4 = 0.02 / 1e-4 = 200x
    let enh_ep2 = ep2.sensitivity_enhancement(eps1_ep2);
    assert!(
        enh_ep2 >= 100.0,
        "EP2 sensitivity enhancement {} must be >= 100x for 1e-4 perturbation",
        enh_ep2
    );

    // 2. EP3: Delta lambda ~ epsilon^(1/3)
    let ep3 = NonHermitianHamiltonian::new(EpOrder::EP3, Complex::ZERO);
    let eps1_ep3 = Complex::new(1e-6, 0.0);
    let eps2_ep3 = Complex::new(1e-3, 0.0);
    let split1_ep3 = ep3.eigenvalue_splitting(eps1_ep3);
    let split2_ep3 = ep3.eigenvalue_splitting(eps2_ep3);

    let exponent_ep3 = (split2_ep3 / split1_ep3).ln() / (eps2_ep3.norm() / eps1_ep3.norm()).ln();
    assert!(
        (exponent_ep3 - (1.0 / 3.0)).abs() < 1e-3,
        "EP3 splitting exponent {} must be 0.3333 (cube root)",
        exponent_ep3
    );

    // EP3 enhancement = split / epsilon = (sqrt(3) * (1e-6)^(1/3)) / 1e-6 = 1.732e-2 / 1e-6 = 17,320x
    let enh_ep3 = ep3.sensitivity_enhancement(eps1_ep3);
    assert!(
        enh_ep3 > 1000.0,
        "EP3 sensitivity enhancement {} must be > 1000x for 1e-6 perturbation",
        enh_ep3
    );

    // 3. EP4: Delta lambda ~ epsilon^(1/4)
    let ep4 = NonHermitianHamiltonian::new(EpOrder::EP4, Complex::ZERO);
    let eps1_ep4 = Complex::new(1e-8, 0.0);
    let eps2_ep4 = Complex::new(1e-4, 0.0);
    let split1_ep4 = ep4.eigenvalue_splitting(eps1_ep4);
    let split2_ep4 = ep4.eigenvalue_splitting(eps2_ep4);

    let exponent_ep4 = (split2_ep4 / split1_ep4).ln() / (eps2_ep4.norm() / eps1_ep4.norm()).ln();
    assert!(
        (exponent_ep4 - 0.25).abs() < 1e-3,
        "EP4 splitting exponent {} must be 0.25 (quartic root)",
        exponent_ep4
    );
}

#[test]
fn test_pt_circuit_phase_transition_from_exact_to_broken() {
    let w0 = 1.0e8; // 100 Mrad/s bare resonance
    let kappa = 5.0e6; // 5 Mrad/s coupling rate

    // 1. Exact PT-symmetric Phase: gamma < kappa
    let gamma_exact = 2.0e6;
    let circuit_exact = PtCircuitParams::with_rates(w0, gamma_exact, kappa);
    assert_eq!(circuit_exact.pt_phase(), PtPhase::Exact);

    let (w1_ex, w2_ex) = circuit_exact.eigenfrequencies();
    assert_eq!(w1_ex.im, 0.0, "Exact phase eigenfrequency must be real");
    assert_eq!(w2_ex.im, 0.0, "Exact phase eigenfrequency must be real");
    assert!(
        w1_ex.re > w2_ex.re,
        "Eigenmodes must be split by 2 * sqrt(kappa^2 - gamma^2)"
    );

    // Transient simulation shows bounded envelope
    let traj_exact = circuit_exact.simulate_transient(1.0, 0.0, 1.0e-6, 1000);
    let max_env = traj_exact.envelope.iter().cloned().fold(0.0f64, f64::max);
    assert!(
        max_env < 20.0,
        "Exact phase envelope {} must remain bounded",
        max_env
    );

    // 2. Exceptional Point: gamma == kappa
    let circuit_ep = PtCircuitParams::with_rates(w0, kappa, kappa);
    assert_eq!(circuit_ep.pt_phase(), PtPhase::ExceptionalPoint);

    let (w1_ep, w2_ep) = circuit_ep.eigenfrequencies();
    assert!(
        (w1_ep.re - w0).abs() < 1e-3,
        "EP frequency must coalesce to w0"
    );
    assert!(
        (w2_ep.re - w0).abs() < 1e-3,
        "EP frequency must coalesce to w0"
    );
    assert_eq!(w1_ep.im, 0.0);
    assert_eq!(w2_ep.im, 0.0);

    // 3. Broken PT-symmetric Phase: gamma > kappa
    let gamma_broken = 9.0e6;
    let circuit_broken = PtCircuitParams::with_rates(w0, gamma_broken, kappa);
    assert_eq!(circuit_broken.pt_phase(), PtPhase::Broken);

    let (w1_br, w2_br) = circuit_broken.eigenfrequencies();
    assert_eq!(
        w1_br.re, w0,
        "Real frequencies in broken phase lock to bare w0"
    );
    assert_eq!(
        w2_br.re, w0,
        "Real frequencies in broken phase lock to bare w0"
    );
    assert!(
        w1_br.im > 0.0,
        "Broken phase mode 1 has net positive gain rate"
    );
    assert!(
        w2_br.im < 0.0,
        "Broken phase mode 2 has net positive loss rate"
    );
    assert!(
        (w1_br.im + w2_br.im).abs() < 1e-6,
        "Im parts must be conjugate opposites"
    );

    // Transient simulation exhibits exponential amplification
    let traj_broken = circuit_broken.simulate_transient(1.0, 0.0, 1.0e-6, 1000);
    let final_env = *traj_broken.envelope.last().unwrap();
    assert!(
        final_env > 2.0,
        "Broken phase must undergo exponential amplitude growth: got {}",
        final_env
    );
}

#[test]
fn test_non_hermitian_skin_effect_directional_localization() {
    let num_sites = 20;

    // 1. Rightward asymmetric coupling: t_R = 2.5, t_L = 1.0
    let lattice_right = NhseLattice::new(num_sites, 1.0, 2.5);
    let res_right = lattice_right.solve_eigenmodes();

    assert!(res_right.is_localized_right);
    assert!(!res_right.is_localized_left);
    assert!(
        res_right.boundary_localization_ratio > 100.0,
        "Rightward NHSE localization ratio {} must be > 100",
        res_right.boundary_localization_ratio
    );

    // Verify all eigenmodes concentrate on rightmost site (num_sites - 1)
    for (mode_idx, profile) in res_right.eigenmode_profiles.iter().enumerate() {
        let p_left = profile[0];
        let p_right = profile[num_sites - 1];
        assert!(
            p_right > p_left * 50.0,
            "Mode {} right boundary {} must dominate left boundary {}",
            mode_idx,
            p_right,
            p_left
        );
    }

    // 2. Leftward asymmetric coupling: t_R = 1.0, t_L = 2.5
    let lattice_left = NhseLattice::new(num_sites, 2.5, 1.0);
    let res_left = lattice_left.solve_eigenmodes();

    assert!(res_left.is_localized_left);
    assert!(!res_left.is_localized_right);
    assert!(
        res_left.boundary_localization_ratio > 100.0,
        "Leftward NHSE localization ratio {} must be > 100",
        res_left.boundary_localization_ratio
    );

    // Verify all eigenmodes concentrate on leftmost site (0)
    for (mode_idx, profile) in res_left.eigenmode_profiles.iter().enumerate() {
        let p_left = profile[0];
        let p_right = profile[num_sites - 1];
        assert!(
            p_left > p_right * 50.0,
            "Mode {} left boundary {} must dominate right boundary {}",
            mode_idx,
            p_left,
            p_right
        );
    }

    // 3. Reciprocal Hermitian lattice: t_R = t_L = 1.5
    let lattice_sym = NhseLattice::new(num_sites, 1.5, 1.5);
    let res_sym = lattice_sym.solve_eigenmodes();
    assert!(!res_sym.is_localized_right);
    assert!(!res_sym.is_localized_left);
    assert!(
        res_sym.skin_depth.is_infinite(),
        "Hermitian reciprocal lattice must have infinite skin depth"
    );
}
