#![deny(unsafe_code)]

//! Test suite for Phase 339: FloquetMetasurfaceDialog in CAD Studio.
//!
//! Verifies:
//! - FloquetMetasurfaceDialog initialization and default parameters.
//! - Parameter adjustment (modulation frequency, phase gradient, incident angle, OAM charge).
//! - Floquet harmonic spectrum and beam steering calculation.
//! - Headless egui Context execution render pass and canvas rendering.

use phonon_gui::widgets::floquet_metasurface_dialog::FloquetMetasurfaceDialog;

#[test]
fn test_floquet_metasurface_dialog_initialization_and_defaults() {
    let dialog = FloquetMetasurfaceDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Physical parameter defaults
    assert!((dialog.carrier_freq_hz - 5000.0).abs() < 1e-9);
    assert!((dialog.incident_angle_deg - 15.0).abs() < 1e-9);
    assert!((dialog.modulation_freq_hz - 500.0).abs() < 1e-9);
    assert!((dialog.phase_gradient_rad_m - 150.0).abs() < 1e-9);
    assert!((dialog.modulation_depth - 0.6).abs() < 1e-9);
    assert_eq!(dialog.harmonic_order, 2);
    assert_eq!(dialog.oam_in, 0);
    assert_eq!(dialog.oam_delta, 1);
    assert!((dialog.speed_of_sound_m_s - 343.0).abs() < 1e-9);
    assert!((dialog.unit_cell_pitch_m - 0.01).abs() < 1e-9);

    // Initial simulation telemetry must be populated
    assert_eq!(dialog.sidebands.len(), 5, "Must compute sidebands n in -2..=2");
    assert!(
        dialog.isolation_db >= 30.0,
        "Default isolation must achieve >= 30.0 dB: got {:.2} dB",
        dialog.isolation_db
    );
    assert!(
        dialog.oam_purity_pct >= 95.0,
        "OAM mode purity must be >= 95.0%: got {:.2}%",
        dialog.oam_purity_pct
    );
    assert!(
        dialog.b_eff > 0.0,
        "Synthetic gauge field B_eff must be positive: got {:.4}",
        dialog.b_eff
    );
    assert!(
        dialog.dominant_efficiency_pct > 5.0,
        "Dominant sideband efficiency must exceed 5%: got {:.2}%",
        dialog.dominant_efficiency_pct
    );

    // Cached plot curves must be populated
    assert_eq!(dialog.s21_curve.len(), 81);
    assert_eq!(dialog.s12_curve.len(), 81);
    assert_eq!(dialog.polar_map.r_steps, 16);
    assert_eq!(dialog.polar_map.theta_steps, 32);
}

#[test]
fn test_parameter_adjustment_in_gui_state() {
    let mut dialog = FloquetMetasurfaceDialog::new();

    // 1. Adjust modulation frequency Omega_m
    dialog.modulation_freq_hz = 1000.0;
    dialog.recompute();
    assert!(
        (dialog.floquet_freq_shift_hz - 1000.0).abs() < 1e-9,
        "Floquet Doppler frequency shift must equal 1000 Hz: got {}",
        dialog.floquet_freq_shift_hz
    );

    // 2. Adjust phase gradient to propagating regime (g_x = 40.0 rad/m)
    dialog.phase_gradient_rad_m = 40.0;
    dialog.recompute();
    assert!(
        dialog.steering_angle_deg.is_some(),
        "Steering angle must be propagating with g_x = 40 rad/m"
    );
    let angle = dialog.steering_angle_deg.unwrap();
    assert!(
        angle > dialog.incident_angle_deg,
        "Steering angle {} deg must exceed incident angle {} deg",
        angle,
        dialog.incident_angle_deg
    );

    // 3. Adjust incident angle to 30.0 degrees
    dialog.incident_angle_deg = 30.0;
    dialog.recompute();
    assert!((dialog.incident_angle_deg - 30.0).abs() < 1e-9);

    // 4. Adjust OAM transfer charge Delta_l to 2
    dialog.oam_delta = 2;
    dialog.recompute();
    assert_eq!(dialog.oam.l_out, 2);
    assert!(
        dialog.oam_purity_pct >= 95.0,
        "OAM purity for Delta_l=2 must maintain >= 95%: got {:.2}%",
        dialog.oam_purity_pct
    );
}

#[test]
fn test_floquet_harmonic_spectrum_and_beam_steering() {
    let mut dialog = FloquetMetasurfaceDialog::new();
    dialog.phase_gradient_rad_m = 45.0; // Propagating steering regime
    dialog.recompute();

    // Verify all sidebands have valid frequencies and Doppler shifts
    for sb in &dialog.sidebands {
        let expected_f = dialog.carrier_freq_hz + (sb.n as f64) * dialog.modulation_freq_hz;
        assert!(
            (sb.f_n - expected_f).abs() < 1e-9,
            "Sideband n={} frequency {} does not match expected {}",
            sb.n,
            sb.f_n,
            expected_f
        );
        assert!(
            sb.power_refl >= 0.0 && sb.power_refl <= 1.0,
            "Power reflection must fall in [0, 1]"
        );
    }

    // Verify beam steering calculation for n = +1
    let sb_1 = dialog.sidebands.iter().find(|s| s.n == 1).unwrap();
    assert!(
        !sb_1.is_evanescent,
        "n=+1 sideband must be propagating for g_x = 45 rad/m"
    );
    let steer_deg = sb_1.theta_refl_deg.unwrap();
    assert!(
        steer_deg > dialog.incident_angle_deg,
        "Reflected beam angle {} deg must be steered beyond incident angle {} deg",
        steer_deg,
        dialog.incident_angle_deg
    );

    // Verify non-reciprocal isolation requirement >= 30 dB
    assert!(
        dialog.isolation_db >= 30.0,
        "Scattering isolation must be >= 30 dB: got {:.2} dB",
        dialog.isolation_db
    );
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = FloquetMetasurfaceDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(!dialog.sidebands.is_empty());
    assert!(!dialog.s21_curve.is_empty());
    assert!(!dialog.s12_curve.is_empty());
    assert!(!dialog.polar_map.grid.is_empty());
}
