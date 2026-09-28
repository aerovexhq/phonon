//! Tests for Grad-Shafranov equilibrium magnetics, Solov'ev analytical solutions,
//! safety factor profiles, and shear Alfvén wave dispersion.

use phonon_models::plasma::{
    AlfvenWaveProperties, SafetyFactorProfile, SolovevEquilibrium, TokamakBeta, TokamakGeometry,
    ToroidalAlfvenEigenmode, VACUUM_PERMEABILITY,
};
use phonon_solver::plasma::GradShafranovSolver;

#[test]
fn test_tokamak_geometry_and_solovev_analytical() {
    let iter = TokamakGeometry::iter_baseline();
    assert!((iter.aspect_ratio() - 3.1).abs() < 1e-6);
    assert!((iter.inverse_aspect_ratio() - (2.0 / 6.2)).abs() < 1e-6);
    assert!(iter.plasma_volume() > 800.0); // ITER plasma volume ~ 840 m^3

    let solovev = SolovevEquilibrium::from_geometry_and_current(iter);
    let (r0, _z0) = (iter.major_radius_r0, 0.0);

    // Magnetic axis should have psi = 0
    let psi_axis = solovev.flux_at(r0, 0.0);
    assert_eq!(psi_axis, 0.0);

    // Magnetic field at magnetic axis: B_R = 0, B_Z = 0, B_phi = B0
    let b_axis = solovev.magnetic_field_at(r0, 0.0);
    assert!(b_axis[0].abs() < 1e-9);
    assert!(b_axis[2].abs() < 1e-9);
    assert!((b_axis[1] - iter.toroidal_b0).abs() < 1e-6);

    // Solov'ev exact operator residual: Delta* psi = 2 * (1 + kappa^2) * (psi_0 / R0^4) * R^2
    let op_val = solovev.grad_shafranov_operator_at(r0, 0.0);
    assert!(op_val > 0.0);
}

#[test]
fn test_safety_factor_and_shear() {
    let q_prof = SafetyFactorProfile::new(1.05, 3.8);

    // Axis and edge safety factor
    assert!((q_prof.q_at_rho(0.0) - 1.05).abs() < 1e-6);
    assert!((q_prof.q_at_rho(1.0) - 3.8).abs() < 1e-6);

    // Magnetic shear: on axis s(0) = 0, at edge s(1) > 0
    assert!((q_prof.shear_at_rho(0.0)).abs() < 1e-6);
    assert!(q_prof.shear_at_rho(1.0) > 1.0);

    // Resonant surface for q = 2/1 mode
    let rho_2_1 = q_prof
        .resonant_surface(2, 1)
        .expect("q=2 surface should exist");
    assert!(rho_2_1 > 0.0 && rho_2_1 < 1.0);
    let q_res = q_prof.q_at_rho(rho_2_1);
    assert!((q_res - 2.0).abs() < 1e-6);

    // Mercier stability: magnetic well stabilization with positive shear
    let d_i = q_prof.mercier_criterion(0.5, 1.0, -1e4, 5.0);
    assert!(d_i > 0.0, "Mercier criterion should indicate MHD stability");
}

#[test]
fn test_grad_shafranov_2d_solver_convergence() {
    let sparc = TokamakGeometry::sparc_baseline();
    let solver = GradShafranovSolver::new(500, 1e-5, 1.70);

    let solution = solver.solve(&sparc, 25, 25);
    assert!(
        solution.converged,
        "2D Grad-Shafranov solver should converge"
    );
    assert!(solution.iterations < 500);
    assert!(solution.max_residual < 1e-5);

    // Check that magnetic axis is close to R0
    let (r_axis, z_axis) = solution.magnetic_axis;
    assert!((r_axis - sparc.major_radius_r0).abs() < sparc.minor_radius_a * 0.3);
    assert!(z_axis.abs() < 0.2);

    // Extracted safety factor profile
    let q_prof = solution.extract_safety_factor_profile(sparc.major_radius_r0 * sparc.toroidal_b0);
    assert!(q_prof.q_0 >= 0.8 && q_prof.q_0 <= 1.5);
    assert!(q_prof.q_a > q_prof.q_0);
}

#[test]
fn test_alfven_wave_dispersion_and_tae_eigenmodes() {
    let b0 = 5.0; // 5 Tesla
    let rho_m = 1.0e-7; // kg/m^3 (approx 3e19 m^-3 deuterium)
    let v_a = AlfvenWaveProperties::alfven_speed(b0, rho_m);

    // Expected v_A = B / sqrt(mu_0 * rho) approx 5 / sqrt(1.256e-6 * 1e-7) approx 1.41e7 m/s
    let expected_va = b0 / (VACUUM_PERMEABILITY * rho_m).sqrt();
    assert!((v_a - expected_va).abs() < 1.0);

    let p_plasma = 5.0e5; // 0.5 MPa
    let c_s = AlfvenWaveProperties::sound_speed(p_plasma, rho_m);
    assert!(c_s > 0.0);

    // Fast and slow magnetosonic speeds
    let v_fast = AlfvenWaveProperties::fast_magnetosonic_speed(v_a, c_s, 0.0);
    let v_slow = AlfvenWaveProperties::slow_magnetosonic_speed(v_a, c_s, 0.0);
    assert!(v_fast >= v_a);
    assert!(v_slow <= c_s);

    // Toroidal Alfven Eigenmode (TAE)
    let tae = ToroidalAlfvenEigenmode::new(1, 1);
    assert!((tae.resonant_q() - 1.5).abs() < 1e-6); // q_TAE = (1 + 0.5)/1 = 1.5

    let r0 = 6.2;
    let omega_tae = tae.eigenfrequency(v_a, r0);
    assert!(omega_tae > 0.0);

    let gap = tae.gap_width(0.3, v_a, r0);
    assert!((gap - 0.3 * omega_tae).abs() < 1e-6);

    let beta = TokamakBeta::calculate(p_plasma, &TokamakGeometry::iter_baseline());
    assert!(beta.beta_t > 0.0 && beta.beta_n > 0.0);
}
