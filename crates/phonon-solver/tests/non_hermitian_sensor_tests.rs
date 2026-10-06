#![deny(unsafe_code)]

//! Automated Verification Test Suite for Phase 398:
//! Phonon Studio Quantum Metamaterial Non-Hermitian Floquet Skin-Effect Sensor & Exceptional Point Magnetometer.

use phonon_solver::non_hermitian_sensor::{
    AcousticMagnonicMagnetometer, Complex, EpSensor, EpSensorParams, ExceptionalPointOrder,
    MagnetoacousticParams, NonHermitianLatticeParams, SkinEffectSolver,
};

#[test]
fn test_asymmetric_hopping_point_gap_winding_number() {
    // 1. Forward hopping dominant: t_R = 10.0 MHz, t_L = 2.0 MHz -> W = +1
    let params_forward = NonHermitianLatticeParams {
        t_R: 10.0,
        t_L: 2.0,
        V_0: 0.0,
        gamma_gain: 1.5,
        n_sites: 40,
        a_mm: 2.0,
    };
    let solver_forward = SkinEffectSolver::new(params_forward);
    assert_eq!(
        solver_forward.winding_number, 1,
        "Winding number must be +1 when t_R > t_L"
    );

    // 2. Backward hopping dominant: t_R = 2.0 MHz, t_L = 10.0 MHz -> W = -1
    let params_backward = NonHermitianLatticeParams {
        t_R: 2.0,
        t_L: 10.0,
        V_0: 0.0,
        gamma_gain: 1.5,
        n_sites: 40,
        a_mm: 2.0,
    };
    let solver_backward = SkinEffectSolver::new(params_backward);
    assert_eq!(
        solver_backward.winding_number, -1,
        "Winding number must be -1 when t_R < t_L"
    );

    // 3. Reciprocal Hermitian limit: t_R = t_L = 6.0 MHz -> W = 0
    let params_reciprocal = NonHermitianLatticeParams {
        t_R: 6.0,
        t_L: 6.0,
        V_0: 0.0,
        gamma_gain: 0.0,
        n_sites: 40,
        a_mm: 2.0,
    };
    let solver_reciprocal = SkinEffectSolver::new(params_reciprocal);
    assert_eq!(
        solver_reciprocal.winding_number, 0,
        "Winding number must be 0 for reciprocal hopping (t_R == t_L)"
    );

    // 4. Verify Generalized Brillouin Zone radius r_gbz = sqrt(|t_L / t_R|)
    let expected_gbz = (2.0_f64 / 10.0_f64).sqrt();
    assert!(
        (solver_forward.gbz_radius - expected_gbz).abs() < 1e-12,
        "GBZ radius must match sqrt(t_L / t_R) = {}, got {}",
        expected_gbz,
        solver_forward.gbz_radius
    );
}

#[test]
fn test_real_space_skin_effect_boundary_localization() {
    let params = NonHermitianLatticeParams {
        t_R: 10.0,
        t_L: 2.0,
        V_0: 0.0,
        gamma_gain: 1.5,
        n_sites: 40,
        a_mm: 2.0,
    };
    let solver = SkinEffectSolver::new(params.clone());

    // 1. Skin localization ratio: fraction of probability within first 10% of boundary sites >= 80%
    assert!(
        solver.skin_localization_ratio >= 0.80,
        "Boundary localization ratio must be >= 80% (got {:.2}%)",
        solver.skin_localization_ratio * 100.0
    );
    assert!(
        solver.skin_localization_ratio > 0.90,
        "Expected observed boundary localization >= 90%, got {:.2}%",
        solver.skin_localization_ratio * 100.0
    );

    // 2. Skin depth xi_skin = a / ln(t_R / t_L)
    let expected_skin_depth = 2.0 / (10.0_f64 / 2.0_f64).ln();
    assert!(
        (solver.skin_depth - expected_skin_depth).abs() < 1e-12,
        "Skin depth must match a / ln(t_R / t_L) = {:.4} mm, got {:.4} mm",
        expected_skin_depth,
        solver.skin_depth
    );

    // 3. Asymmetric spatial eigenmode profile: rightmost site intensity > leftmost site intensity by many orders of magnitude
    let profile = &solver.skin_intensity_profile;
    assert_eq!(profile.len(), 40);
    assert!(
        profile[39] > profile[0] * 1e6,
        "Right boundary intensity must vastly exceed left boundary intensity due to exponential skin accumulation"
    );

    // 4. OBC matrix assembly check
    let obc_mat = solver.assemble_obc_matrix();
    assert_eq!(obc_mat.len(), 40);
    assert_eq!(obc_mat[0].len(), 40);
    assert_eq!(obc_mat[1][0], Complex::new(10.0, 0.0)); // t_R
    assert_eq!(obc_mat[0][1], Complex::new(2.0, 0.0)); // t_L
}

#[test]
fn test_ep2_coalescence_and_square_root_splitting() {
    let mut ep_params = EpSensorParams {
        order: ExceptionalPointOrder::EP2,
        kappa_0: 5.0,
        gamma_ep: 5.0,
        epsilon: 0.0,
        omega_0: 1000.0,
    };
    let sensor = EpSensor::new(ep_params.clone());

    // 1. Eigenvalue coalescence at exact EP condition (epsilon = 0)
    let eigs_at_ep = sensor.eigenvalues();
    assert_eq!(eigs_at_ep.len(), 2);
    let split_zero = sensor.eigenvalue_splitting(0.0);
    assert!(
        split_zero < 1e-9,
        "Eigenvalues must coalesce at exact EP condition (splitting = 0), got {}",
        split_zero
    );
    assert!(
        (eigs_at_ep[0].re - 1000.0).abs() < 1e-9 && (eigs_at_ep[1].re - 1000.0).abs() < 1e-9,
        "Real parts of coalesced eigenvalues must match omega_0 = 1000.0 MHz"
    );

    // 2. Power-law verification: Delta_omega proportional to epsilon^(1/2)
    let eps1 = 1e-4;
    let eps2 = 1e-6;
    let exponent = sensor.verify_power_law(eps1, eps2);
    assert!(
        (exponent - 0.50).abs() < 0.02,
        "EP2 frequency splitting must exhibit square-root scaling (p ~ 0.50), got {:.4}",
        exponent
    );

    // 3. Magnitude check: Delta_omega(eps) ~ 2 * sqrt(gamma) * sqrt(eps)
    let eps_test = 1e-4;
    let split_obs = sensor.eigenvalue_splitting(eps_test);
    let split_theo = 2.0 * (5.0_f64 * eps_test).sqrt();
    assert!(
        (split_obs - split_theo).abs() / split_theo < 0.01,
        "Observed splitting {:.6} MHz must match theoretical 2*sqrt(gamma*eps) = {:.6} MHz",
        split_obs,
        split_theo
    );

    // 4. Update perturbation parameter and verify struct consistency
    ep_params.epsilon = eps_test;
    let sensor_pert = EpSensor::new(ep_params);
    let eigs_pert = sensor_pert.eigenvalues();
    assert!(
        (eigs_pert[0] - eigs_pert[1]).norm() > 0.01,
        "Perturbed eigenvalues must split symmetrically away from coalescence"
    );
}

#[test]
fn test_ep3_cube_root_splitting() {
    let ep3_params = EpSensorParams {
        order: ExceptionalPointOrder::EP3,
        kappa_0: 5.0,
        gamma_ep: 5.0 * std::f64::consts::SQRT_2,
        epsilon: 0.0,
        omega_0: 1000.0,
    };
    assert!(ep3_params.is_at_ep());

    let sensor3 = EpSensor::new(ep3_params.clone());

    // 1. Coalescence at epsilon = 0
    let split_zero = sensor3.eigenvalue_splitting(0.0);
    assert!(
        split_zero < 1e-9,
        "EP3 eigenvalues must coalesce at epsilon = 0, got {}",
        split_zero
    );

    // 2. Power-law scaling: Delta_omega proportional to epsilon^(1/3)
    let eps1 = 1e-4;
    let eps2 = 1e-7;
    let exponent3 = sensor3.verify_power_law(eps1, eps2);
    assert!(
        (exponent3 - (1.0 / 3.0)).abs() < 0.02,
        "EP3 frequency splitting must exhibit cube-root scaling (p ~ 0.333), got {:.4}",
        exponent3
    );

    // 3. 3 coalesced eigenvalues
    let eigs3 = sensor3.eigenvalues();
    assert_eq!(eigs3.len(), 3);
}

#[test]
fn test_sub_picotesla_magnetic_sensitivity_and_noise_floor() {
    let mag_params = MagnetoacousticParams {
        B_me: 5.0,
        M_s: 140.0,
        f_0: 1.0,
        Q_ac: 10_000.0,
        T_kelvin: 4.2,
        acoustic_velocity_m_s: 3800.0,
        c44_gpa: 76.4,
        probe_power_uw: 1.0,
    };
    let ep_params = EpSensorParams {
        order: ExceptionalPointOrder::EP2,
        kappa_0: 5.0,
        gamma_ep: 5.0,
        epsilon: 1e-5,
        omega_0: 1000.0,
    };

    let magnetometer = AcousticMagnonicMagnetometer::new(mag_params, ep_params);

    // 1. Minimum detectable field B_min < 1.0 pT / sqrt(Hz)
    let b_min = magnetometer.minimum_detectable_field();
    assert!(
        b_min < 1.0,
        "Minimum detectable magnetic flux density must be < 1.0 pT / sqrt(Hz), got {:.4} pT / sqrt(Hz)",
        b_min
    );
    assert!(
        b_min > 0.05,
        "Noise floor must be physically non-zero, got {:.4}",
        b_min
    );

    // 2. Dynamic range DR >= 60.0 dB
    let dr = magnetometer.dynamic_range_db();
    assert!(
        dr >= 60.0,
        "Dynamic range must be >= 60.0 dB, got {:.2} dB",
        dr
    );

    // 3. Telemetry evaluation for a 5.0 pT input signal
    let telemetry = magnetometer.measure(5.0, 1.0);
    assert_eq!(telemetry.true_field_pt, 5.0);
    assert!(
        telemetry.snr_db > 10.0,
        "SNR for 5.0 pT signal in 1 Hz bandwidth must be positive and > 10 dB, got {:.2} dB",
        telemetry.snr_db
    );
    assert!(
        telemetry.mechanical_strain > 0.0,
        "Induced magnetoelastic strain must be strictly positive"
    );
    assert!(
        telemetry.frequency_splitting_khz > 0.0,
        "Frequency splitting readout must be strictly positive"
    );
}

#[test]
fn test_responsivity_enhancement_over_linear_sensor() {
    let ep_params = EpSensorParams {
        order: ExceptionalPointOrder::EP2,
        kappa_0: 5.0,
        gamma_ep: 5.0,
        epsilon: 1e-5,
        omega_0: 1000.0,
    };
    let sensor = EpSensor::new(ep_params);

    // At small perturbation epsilon = 1e-5:
    // S_EP = d(Delta_omega)/d(epsilon) ~ sqrt(gamma) / sqrt(epsilon) ~ sqrt(5) / sqrt(1e-5) ~ 707
    let enhancement = sensor.enhancement_factor(1e-5);
    assert!(
        enhancement > 100.0,
        "EP sensor responsivity enhancement over linear sensor must exceed 100x at small epsilon (got {:.2}x)",
        enhancement
    );

    // Divergent scaling: as epsilon decreases, enhancement increases further
    let enh_smaller = sensor.enhancement_factor(1e-6);
    assert!(
        enh_smaller > enhancement,
        "Enhancement factor must grow monotonically as epsilon approaches 0"
    );
}
