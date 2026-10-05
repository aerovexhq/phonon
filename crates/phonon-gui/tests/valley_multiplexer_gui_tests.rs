#![deny(unsafe_code)]

use phonon_gui::widgets::valley_multiplexer_dialog::{ValleyDialogTab, ValleyMultiplexerDialog};

#[test]
fn test_valley_multiplexer_dialog_initialization() {
    let dialog = ValleyMultiplexerDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        ValleyDialogTab::JunctionFieldCanvas
    );
    assert_eq!(dialog.valley_polarization, 1.0);
    assert_eq!(dialog.operating_frequency_khz, 4.8);
    assert_eq!(dialog.asymmetry_delta, 0.22);
    assert!(!dialog.has_corner_bend);

    // Initial Berry metrics
    assert_eq!(dialog.berry_metrics.delta_cv_interface, 1.0);
    assert_eq!(dialog.berry_metrics.c_k, 0.5);
    assert_eq!(dialog.berry_metrics.c_k_prime, -0.5);

    // Initial S-parameters (pure K valley routes to Port 2)
    assert!(
        dialog.mux_solver.s_parameters.s21_db > dialog.mux_solver.s_parameters.s31_db,
        "K-valley routing must prioritize Port 2 over Port 3"
    );
    assert!(
        dialog.mux_solver.s_parameters.valley_isolation_db >= 20.0,
        "Valley isolation must be >= 20.0 dB, got {}",
        dialog.mux_solver.s_parameters.valley_isolation_db
    );
}

#[test]
fn test_valley_multiplexer_preset_switching() {
    let mut dialog = ValleyMultiplexerDialog::default();

    // Preset: Valley K' -> Port 3
    dialog.valley_polarization = -1.0;
    dialog.splitting_bias = 0.0;
    dialog.has_corner_bend = false;
    dialog.recompute();

    assert!(
        dialog.mux_solver.s_parameters.s31_db > dialog.mux_solver.s_parameters.s21_db,
        "K'-valley routing must prioritize Port 3 over Port 2"
    );
    assert!(
        dialog.mux_solver.s_parameters.valley_isolation_db >= 20.0,
        "Valley isolation for K' must be >= 20.0 dB, got {}",
        dialog.mux_solver.s_parameters.valley_isolation_db
    );

    // Preset: 50:50 Balanced Splitter
    dialog.valley_polarization = 0.0;
    dialog.splitting_bias = 0.0;
    dialog.recompute();

    let diff = (dialog.mux_solver.s_parameters.s21_db - dialog.mux_solver.s_parameters.s31_db).abs();
    assert!(
        diff < 0.1,
        "Balanced splitting must have symmetric S21 and S31 within 0.1 dB, got diff = {}",
        diff
    );
    assert!(
        (dialog.mux_solver.s_parameters.splitting_ratio - 0.5).abs() < 0.02,
        "Splitting ratio must be approximately 0.5, got {}",
        dialog.mux_solver.s_parameters.splitting_ratio
    );

    // Preset: Sharp Corner Bend Immunity
    dialog.valley_polarization = 1.0;
    dialog.has_corner_bend = true;
    dialog.recompute();

    assert!(
        dialog.mux_solver.s_parameters.corner_immunity_ratio >= 0.95,
        "Corner immunity ratio must be >= 0.95, got {}",
        dialog.mux_solver.s_parameters.corner_immunity_ratio
    );
}

#[test]
fn test_valley_multiplexer_headless_render() {
    let mut dialog = ValleyMultiplexerDialog::default();
    let ctx = egui::Context::default();

    // Tab 1: JunctionFieldCanvas
    dialog.active_tab = ValleyDialogTab::JunctionFieldCanvas;
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out1.textures_delta.clear();

    // Tab 2: BerryCurvatureContour
    dialog.active_tab = ValleyDialogTab::BerryCurvatureContour;
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out2.textures_delta.clear();

    // Tab 3: KinkEdgeDispersion
    dialog.active_tab = ValleyDialogTab::KinkEdgeDispersion;
    let mut out3 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out3.textures_delta.clear();

    // Tab 4: SParameterSpectrum
    dialog.active_tab = ValleyDialogTab::SParameterSpectrum;
    let mut out4 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out4.textures_delta.clear();
}
