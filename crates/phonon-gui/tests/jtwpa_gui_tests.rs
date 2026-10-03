#![deny(unsafe_code)]

//! Test suite for Phase 338: JtwpaDialog in CAD Studio.
//!
//! Verifies:
//! - JtwpaDialog initialization and default parameters.
//! - Parameter adjustment (pump power, junction count, RPM toggle).
//! - Gain and squeezing spectrum calculation in GUI state.
//! - Headless egui Context execution render pass and plot generation.

use phonon_gui::widgets::jtwpa_dialog::JtwpaDialog;

#[test]
fn test_jtwpa_dialog_initialization_and_defaults() {
    let dialog = JtwpaDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Physical parameter defaults
    assert!((dialog.pump_freq_ghz - 6.0).abs() < 1.0e-9);
    assert!((dialog.pump_power_dbm - (-50.0)).abs() < 1.0e-9);
    assert_eq!(dialog.num_cells, 1000);
    assert!((dialog.l_j0_ph - 80.0).abs() < 1.0e-9);
    assert!((dialog.i_c_ua - 5.0).abs() < 1.0e-9);
    assert!((dialog.c_g_ff - 50.0).abs() < 1.0e-9);
    assert!((dialog.c_j_ff - 10.0).abs() < 1.0e-9);
    assert!((dialog.a_um - 10.0).abs() < 1.0e-9);
    assert!(dialog.rpm_enabled, "RPM must be enabled by default");
    assert_eq!(dialog.m_cells, 16);
    assert!((dialog.f_rpm_ghz - 12.0).abs() < 1.0e-9);
    assert!((dialog.attenuation_db - 34.0).abs() < 1.0e-9);

    // Initial simulation telemetry must be populated
    assert!(
        dialog.peak_gain_db >= 20.0,
        "Default peak gain must achieve >= 20.0 dB: got {:.2} dB",
        dialog.peak_gain_db
    );
    assert!(
        dialog.bandwidth_3db_ghz >= 3.8,
        "3-dB bandwidth must cover octave bandwidth: got {:.2} GHz",
        dialog.bandwidth_3db_ghz
    );
    assert!(
        dialog.squeezing_db > 6.0,
        "Squeezing must exceed 6 dB below SQL: got {:.2} dB",
        dialog.squeezing_db
    );
    assert!(
        dialog.n_add <= 0.55,
        "Added noise quanta must approach quantum limit <= 0.55: got {:.4}",
        dialog.n_add
    );
    assert!(
        dialog.delta_k_rad_per_m.abs() < 25.0,
        "Phase mismatch with RPM must be near zero (< 25 rad/m): got {:.2} rad/m",
        dialog.delta_k_rad_per_m
    );

    // Cached plot curves must be populated
    assert!(!dialog.spectrum.is_empty());
    assert_eq!(dialog.gain_curve.len(), 100);
    assert_eq!(dialog.idler_gain_curve.len(), 100);
    assert_eq!(dialog.dispersion_bare_curve.len(), 100);
    assert_eq!(dialog.dispersion_rpm_curve.len(), 100);
}

#[test]
fn test_parameter_adjustment_in_gui_state() {
    let mut dialog = JtwpaDialog::new();

    // 1. Adjust pump power
    dialog.pump_power_dbm = -55.0;
    dialog.recompute();
    assert!((dialog.on_chip_power_dbm - (-89.0)).abs() < 1.0e-9);
    assert!(
        dialog.peak_gain_db < 20.0,
        "Lower pump power must reduce parametric gain below 20 dB: got {:.2} dB",
        dialog.peak_gain_db
    );

    // Restore nominal pump power and increase junction count
    dialog.pump_power_dbm = -50.0;
    dialog.num_cells = 1200;
    dialog.recompute();
    assert!(
        dialog.peak_gain_db >= 20.0,
        "Higher cell count at nominal power must maintain high gain: got {:.2} dB",
        dialog.peak_gain_db
    );

    // 2. Disable RPM engineering
    dialog.rpm_enabled = false;
    dialog.recompute();
    assert!(
        dialog.delta_k_rad_per_m.abs() > 400.0,
        "Disabling RPM must result in severe phase mismatch (> 400 rad/m): got {:.2} rad/m",
        dialog.delta_k_rad_per_m
    );
    assert!(
        dialog.peak_gain_db < 15.0,
        "Disabling RPM must suppress parametric gain due to phase mismatch: got {:.2} dB",
        dialog.peak_gain_db
    );

    // 3. Re-enable RPM engineering
    dialog.rpm_enabled = true;
    dialog.num_cells = 1000;
    dialog.recompute();
    assert!(
        dialog.delta_k_rad_per_m.abs() < 25.0,
        "Re-enabling RPM must restore phase matching: got {:.2} rad/m",
        dialog.delta_k_rad_per_m
    );
    assert!(
        dialog.peak_gain_db >= 20.0,
        "Re-enabling RPM must restore >= 20 dB gain: got {:.2} dB",
        dialog.peak_gain_db
    );
}

#[test]
fn test_gain_and_squeezing_spectrum_calculation() {
    let mut dialog = JtwpaDialog::new();
    dialog.signal_start_ghz = 4.0;
    dialog.signal_stop_ghz = 8.0;
    dialog.recompute();

    // Verify gain curve bounds
    for pt in &dialog.gain_curve {
        let f_ghz = pt[0];
        let g_db = pt[1];
        assert!(
            (4.0..=8.0).contains(&f_ghz),
            "Frequency {} GHz must fall within [4.0, 8.0] GHz",
            f_ghz
        );
        assert!(
            g_db >= 20.0,
            "Gain at {} GHz must achieve >= 20.0 dB: got {:.2} dB",
            f_ghz,
            g_db
        );
    }

    // Verify squeezing state
    assert!(
        dialog.squeezing.s_xx < 0.25,
        "Squeezed variance must be below SQL (0.25): got {}",
        dialog.squeezing.s_xx
    );
    assert!(
        dialog.squeezing.s_yy > 1.0,
        "Anti-squeezed variance must be above 1.0: got {}",
        dialog.squeezing.s_yy
    );
    let unc = dialog.squeezing.s_xx * dialog.squeezing.s_yy;
    assert!(
        (unc - 0.0625).abs() < 1.0e-6,
        "Heisenberg product S_xx * S_yy must equal 0.0625: got {}",
        unc
    );
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = JtwpaDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(!dialog.gain_curve.is_empty());
    assert!(!dialog.idler_gain_curve.is_empty());
    assert!(!dialog.dispersion_bare_curve.is_empty());
    assert!(!dialog.dispersion_rpm_curve.is_empty());
}
