#![deny(unsafe_code)]

//! Verification test suite for Phonon Visual Studio Interactive Smith Chart,
//! RF S-Parameter Extraction, and Harmonic Balance Frequency-Domain Dialog.

use phonon_gui::widgets::smith_chart_dialog::SmithChartDialog;

#[test]
fn test_smith_chart_dialog_initialization_defaults() {
    let dialog = SmithChartDialog::new();

    // Dialog should be closed by default
    assert!(!dialog.is_open, "Dialog should be closed by default");

    // Frequency sweep configuration defaults
    assert_eq!(dialog.start_freq_hz, 100_000.0);
    assert_eq!(dialog.stop_freq_hz, 10_000_000_000.0);
    assert_eq!(dialog.sweep_points, 101);
    assert!(dialog.is_log_sweep);
    assert_eq!(dialog.z0_ref, 50.0);

    // Visual trajectory and overlay toggles
    assert!(dialog.show_s11);
    assert!(dialog.show_s22);
    assert!(dialog.show_stability_circles);

    // Verify baseline simulation outputs
    assert_eq!(dialog.s_parameters.len(), 101);
    assert!(dialog.hb_result.is_some());
    assert!(dialog.nl_metrics.is_some());
    assert!(!dialog.run_requested);
}

#[test]
fn test_frequency_sweep_execution_and_trajectories() {
    let mut dialog = SmithChartDialog::new();

    // Configure 51-point linear sweep for a matched transmission line DUT
    dialog.start_freq_hz = 1.0e9;
    dialog.stop_freq_hz = 5.0e9;
    dialog.sweep_points = 51;
    dialog.is_log_sweep = false;
    dialog.circuit_type_idx = 2; // Transmission Line
    dialog.tline_zc = 50.0;
    dialog.tline_delay_ps = 200.0; // 200 ps

    dialog.run_simulation();

    assert_eq!(dialog.s_parameters.len(), 51);
    assert!((dialog.s_parameters[0].freq_hz - 1.0e9).abs() < 1e-3);
    assert!((dialog.s_parameters[50].freq_hz - 5.0e9).abs() < 1e-3);

    for s in &dialog.s_parameters {
        assert!(s.s11_mag() < 1e-9);
        assert!((s.s21_mag() - 1.0).abs() < 1e-9);
        assert!((s.vswr() - 1.0).abs() < 1e-6);
        assert!(s.return_loss_db() > 100.0);
    }

    // Configure RLC tank DUT
    dialog.circuit_type_idx = 1; // Shunt RLC Tank
    dialog.run_simulation();
    assert_eq!(dialog.s_parameters.len(), 51);
    assert!(dialog.s_parameters[25].s11_mag() <= 1.0);
    assert!(dialog.s_parameters[25].s21_mag() <= 1.0);
}

#[test]
fn test_touchstone_s2p_generation_and_export() {
    let mut dialog = SmithChartDialog::new();
    dialog.start_freq_hz = 500.0e6;
    dialog.stop_freq_hz = 2.5e9;
    dialog.sweep_points = 21;
    dialog.circuit_type_idx = 3; // 6 dB symmetric T-pad attenuator
    dialog.run_simulation();

    let s2p_str = dialog.export_touchstone_s2p();

    // Verify Touchstone format headers
    assert!(s2p_str.contains("# HZ S RI R 50"));
    assert!(s2p_str.contains("! Touchstone 2-port S-parameters file"));

    // Verify row count (excluding comment lines and header)
    let data_rows: Vec<&str> = s2p_str
        .lines()
        .filter(|l| !l.starts_with('!') && !l.starts_with('#') && !l.trim().is_empty())
        .collect();
    assert_eq!(data_rows.len(), 21);

    // Verify first row contains frequency and 8 S-parameter reals/imaginaries
    let cols: Vec<&str> = data_rows[0].split_whitespace().collect();
    assert_eq!(cols.len(), 9);
    let f_start: f64 = cols[0].parse().unwrap();
    assert!((f_start - 500.0e6).abs() < 1e-2);

    // Verify insertion loss of 6 dB pad is approximately 6 dB (S21 ~ 0.5)
    let s21_re: f64 = cols[3].parse().unwrap();
    let s21_im: f64 = cols[4].parse().unwrap();
    let s21_mag = (s21_re * s21_re + s21_im * s21_im).sqrt();
    assert!((s21_mag - 0.501187).abs() < 0.05);
}

#[test]
fn test_harmonic_balance_spectrum_and_display() {
    let mut dialog = SmithChartDialog::new();
    dialog.hb_harmonics = 7;
    dialog.hb_pin_dbm = 2.0;
    dialog.hb_fund_hz = 2.4e9; // 2.4 GHz ISM band

    dialog.run_simulation();

    let hb = dialog.hb_result.as_ref().expect("HB result must be available");
    assert_eq!(hb.harmonics.len(), 8); // DC + 7 harmonics
    assert!(hb.converged);

    // Check fundamental output power
    let p_fund = hb.fundamental_power_dbm();
    assert!(p_fund > -25.0 && p_fund < 15.0);

    // Check third harmonic power
    let p_harm3 = hb.third_harmonic_power_dbm();
    assert!(p_fund > p_harm3);

    // Non-linear intercept metrics
    let nl = dialog.nl_metrics.as_ref().expect("Nonlinear metrics must be available");
    assert!(nl.linear_gain_db.is_finite());
    assert!(nl.p1db_in_dbm.is_finite());
    assert!(nl.p1db_out_dbm.is_finite());
    assert!(nl.ip3_in_dbm.is_finite());
    assert!(nl.ip3_out_dbm.is_finite());
}

#[test]
fn test_smith_chart_ui_headless_render() {
    let mut dialog = SmithChartDialog::new();
    dialog.is_open = true;

    // Headless egui Context execution test
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(!dialog.s_parameters.is_empty());
}
