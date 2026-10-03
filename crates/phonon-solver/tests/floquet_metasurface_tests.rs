#![deny(unsafe_code)]

//! Test suite for Phase 339: Floquet Engineered Spatio-Temporal Acoustic Metasurface Simulator.
//!
//! Verifies:
//! - Floquet sideband frequency shifting: f_n = f_inc + n * Omega_m.
//! - Generalized Snell's law angle deflection: k_x_n = k_x_inc + n * g_x.
//! - Non-reciprocal Doppler transmission isolation S_21 - S_12 >= 30.0 dB.
//! - Synthetic gauge field vector potential and Aharonov-Bohm phase.
//! - OAM vortex beam reflection mode purity >= 95% and unwanted sideband suppression >= 25.0 dB.

use phonon_solver::floquet_metasurface::{
    bessel_j, FloquetMetasurfaceSolver, FloquetModulationParams, MetasurfaceUnitCell,
    OrbitalAngularMomentum, SyntheticGaugeField,
};
use std::f64::consts::PI;

#[test]
fn test_floquet_sideband_frequency_shifting() {
    let cell = MetasurfaceUnitCell::default();
    let mod_params = FloquetModulationParams {
        f_inc: 5000.0,
        theta_inc_deg: 15.0,
        omega_m_hz: 500.0,
        g_x: 150.0,
        m: 0.6,
        n_f: 2,
    };
    let solver = FloquetMetasurfaceSolver::new(cell, mod_params);
    let sidebands = solver.compute_sidebands();

    assert_eq!(sidebands.len(), 5); // n in -2, -1, 0, 1, 2

    for sb in &sidebands {
        let expected_fn = mod_params.f_inc + (sb.n as f64) * mod_params.omega_m_hz;
        assert!(
            (sb.f_n - expected_fn).abs() < 1e-9,
            "Sideband n={} frequency {} Hz does not match expected {} Hz",
            sb.n,
            sb.f_n,
            expected_fn
        );

        let expected_delta_f = (sb.n as f64) * mod_params.omega_m_hz;
        assert!(
            (sb.delta_f - expected_delta_f).abs() < 1e-9,
            "Sideband n={} Doppler shift {} Hz does not match expected {} Hz",
            sb.n,
            sb.delta_f,
            expected_delta_f
        );
    }

    // Verify Bessel function expansion values
    let j0 = bessel_j(0, 0.6);
    let j1 = bessel_j(1, 0.6);
    let j2 = bessel_j(2, 0.6);
    assert!((j0 - 0.9120).abs() < 0.005);
    assert!((j1 - 0.2867).abs() < 0.005);
    assert!((j2 - 0.0437).abs() < 0.005);
}

#[test]
fn test_generalized_snells_law_angle_deflection() {
    let cell = MetasurfaceUnitCell {
        f_0: 5000.0,
        d_x: 0.01,
        c_s: 343.0,
        rho_0: 1.225,
        z_0: 1.225 * 343.0,
    };
    let theta_inc_deg = 15.0;
    let mod_params = FloquetModulationParams {
        f_inc: 5000.0,
        theta_inc_deg,
        omega_m_hz: 500.0,
        g_x: 150.0,
        m: 0.6,
        n_f: 2,
    };
    let solver = FloquetMetasurfaceSolver::new(cell, mod_params);
    let sidebands = solver.compute_sidebands();

    let k_0 = 2.0 * PI * 5000.0 / 343.0;
    let k_x_inc = k_0 * theta_inc_deg.to_radians().sin();

    for sb in &sidebands {
        let expected_k_x_n = k_x_inc + (sb.n as f64) * mod_params.g_x;
        assert!(
            (sb.k_x_n - expected_k_x_n).abs() < 1e-9,
            "Sideband n={} k_x_n {} does not match expected {}",
            sb.n,
            sb.k_x_n,
            expected_k_x_n
        );

        let k_n = 2.0 * PI * sb.f_n / cell.c_s;
        let ratio = sb.k_x_n / k_n;
        if ratio.abs() <= 1.0 {
            assert!(
                !sb.is_evanescent,
                "Mode with |k_x/k| <= 1.0 must be propagating"
            );
            let expected_angle = ratio.asin().to_degrees();
            let actual_angle = sb.theta_refl_deg.expect("Propagating mode must have angle");
            assert!(
                (actual_angle - expected_angle).abs() < 1e-6,
                "Reflected angle {} deg does not match expected {} deg",
                actual_angle,
                expected_angle
            );
        } else {
            assert!(
                sb.is_evanescent,
                "Mode with |k_x/k| > 1.0 must be evanescent"
            );
            assert!(sb.theta_refl_deg.is_none());
        }
    }

    // Specifically verify n = 0 specular reflection equals incident angle
    let sb_0 = sidebands.iter().find(|s| s.n == 0).unwrap();
    let angle_0 = sb_0.theta_refl_deg.unwrap();
    assert!(
        (angle_0 - theta_inc_deg).abs() < 1e-6,
        "n=0 specular reflection must match theta_inc=15 deg: got {}",
        angle_0
    );

    // With g_x = 150 rad/m, n = 1 has k_x_1 = 173.7 rad/m > k_1 (100.8 rad/m), so it is evanescent
    let sb_1 = sidebands.iter().find(|s| s.n == 1).unwrap();
    assert!(
        sb_1.is_evanescent,
        "With g_x = 150 rad/m, n = 1 must be evanescent: |k_x/k| > 1.0"
    );
    assert!(sb_1.theta_refl_deg.is_none());

    // Now test with propagating phase gradient g_x = 40.0 rad/m
    let mod_params_prop = FloquetModulationParams {
        f_inc: 5000.0,
        theta_inc_deg,
        omega_m_hz: 500.0,
        g_x: 40.0,
        m: 0.6,
        n_f: 2,
    };
    let solver_prop = FloquetMetasurfaceSolver::new(cell, mod_params_prop);
    let sbs_prop = solver_prop.compute_sidebands();
    let sb_1_prop = sbs_prop.iter().find(|s| s.n == 1).unwrap();
    assert!(
        !sb_1_prop.is_evanescent,
        "With g_x = 40 rad/m, n = 1 must be propagating"
    );
    let angle_1 = sb_1_prop.theta_refl_deg.unwrap();
    assert!(
        angle_1 > theta_inc_deg,
        "n=+1 deflected angle {} deg must exceed incident angle {} deg",
        angle_1,
        theta_inc_deg
    );
}

#[test]
fn test_non_reciprocal_doppler_transmission_isolation() {
    let cell = MetasurfaceUnitCell::default();
    let mod_params = FloquetModulationParams {
        f_inc: 5000.0,
        theta_inc_deg: 15.0,
        omega_m_hz: 500.0,
        g_x: 150.0,
        m: 0.6,
        n_f: 2,
    };
    let solver = FloquetMetasurfaceSolver::new(cell, mod_params);
    let scat = solver.compute_scattering();

    // Verify non-reciprocal isolation requirement S_21 - S_12 >= 30.0 dB
    let isolation_diff = scat.s21_db - scat.s12_db;
    assert!(
        isolation_diff >= 30.0,
        "Transmission isolation S_21 - S_12 must be >= 30.0 dB: got {:.2} dB",
        isolation_diff
    );
    assert!(
        scat.isolation_db >= 30.0,
        "Isolation metric must be >= 30.0 dB: got {:.2} dB",
        scat.isolation_db
    );
    assert!(
        (isolation_diff - scat.isolation_db).abs() < 1e-9,
        "Isolation metric and difference must be identical"
    );

    // Verify forward transmission is non-zero
    assert!(
        scat.s21_linear > 0.05,
        "Forward transmission must be positive: got {}",
        scat.s21_linear
    );

    // Verify backward transmission is heavily attenuated (< -30 dB)
    assert!(
        scat.s12_db <= -30.0,
        "Backward transmission must be suppressed <= -30 dB: got {:.2} dB",
        scat.s12_db
    );

    // Spectrum calculation check
    let (s21_curve, s12_curve) = solver.compute_transmission_spectrum(4500.0, 5500.0, 21);
    assert_eq!(s21_curve.len(), 21);
    assert_eq!(s12_curve.len(), 21);
    // Center point at 5000 Hz must satisfy isolation >= 30 dB
    let center_idx = 10;
    assert!((s21_curve[center_idx][0] - 5000.0).abs() < 1e-6);
    let center_iso = s21_curve[center_idx][1] - s12_curve[center_idx][1];
    assert!(
        center_iso >= 30.0,
        "Center frequency isolation must be >= 30.0 dB: got {:.2} dB",
        center_iso
    );
}

#[test]
fn test_synthetic_gauge_field_and_aharonov_bohm_phase() {
    let g_x = 150.0;
    let omega_m_hz = 500.0;
    let pitch_m = 0.01; // 10 mm
    let gauge = SyntheticGaugeField::new(g_x, omega_m_hz, pitch_m);

    // A_eff_x = g_x / (2 * pi * Omega_m)
    let expected_a = g_x / (2.0 * PI * omega_m_hz);
    assert!(
        (gauge.a_eff_x - expected_a).abs() < 1e-9,
        "A_eff_x {} does not match expected {}",
        gauge.a_eff_x,
        expected_a
    );

    // B_eff_z = A_eff_x / pitch_m
    let expected_b = expected_a / pitch_m;
    assert!(
        (gauge.b_eff_z - expected_b).abs() < 1e-9,
        "B_eff_z {} does not match expected {}",
        gauge.b_eff_z,
        expected_b
    );

    // Aharonov-Bohm phase for a loop of area 0.05 m x 0.05 m = 0.0025 m^2
    let loop_area = 0.0025;
    let phi_ab = gauge.aharonov_bohm_phase(loop_area);
    let expected_phi = expected_b * loop_area;
    assert!(
        (phi_ab - expected_phi).abs() < 1e-9,
        "Aharonov-Bohm phase {} does not match expected {}",
        phi_ab,
        expected_phi
    );
    assert!(
        phi_ab > 0.0,
        "Synthetic Aharonov-Bohm phase must be strictly positive"
    );
}

#[test]
fn test_oam_vortex_beam_reflection_mode_purity() {
    let l_in = 0;
    let delta_l = 1;
    let omega_m_hz = 500.0;
    let oam = OrbitalAngularMomentum::new(l_in, delta_l, omega_m_hz);

    assert_eq!(oam.l_out, 1, "Target OAM charge must be l_in + Delta_l = 1");

    // Modal purity target >= 95% (0.95)
    assert!(
        oam.mode_purity >= 0.95,
        "OAM mode purity must be >= 0.95: got {:.4}",
        oam.mode_purity
    );

    // Unwanted sideband suppression target >= 25.0 dB
    assert!(
        oam.sideband_suppression_db >= 25.0,
        "Unwanted sideband suppression must be >= 25.0 dB: got {:.2} dB",
        oam.sideband_suppression_db
    );

    // Phase evaluation
    let phase_0 = oam.phase_at(1.0, 0.0, 0.0);
    assert!(phase_0.abs() < 1e-9);

    let phase_pi = oam.phase_at(1.0, PI, 0.0);
    assert!((phase_pi - PI).abs() < 1e-9);

    // Polar phase map generation
    let polar_map = oam.generate_polar_phase_map(16, 32, 0.0);
    assert_eq!(polar_map.r_steps, 16);
    assert_eq!(polar_map.theta_steps, 32);
    assert_eq!(polar_map.grid.len(), 16);
    assert_eq!(polar_map.grid[0].len(), 32);

    for row in &polar_map.grid {
        for &val in row {
            assert!(
                (0.0..=2.0 * PI + 1e-6).contains(&val),
                "Phase value {} must fall in [0, 2*pi]",
                val
            );
        }
    }
}
