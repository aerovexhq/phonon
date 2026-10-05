#![deny(unsafe_code)]

use phonon_gui::widgets::non_hermitian_skin_dialog::{
    NonHermitianSkinDialog, NonHermitianSkinDialogTab,
};

#[test]
fn test_non_hermitian_skin_dialog_initialization() {
    let dialog = NonHermitianSkinDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        NonHermitianSkinDialogTab::ChainAndSkinCanvas
    );
    assert_eq!(dialog.chain_length, 30);
    assert_eq!(dialog.asymmetry_g, 0.45);
    assert_eq!(dialog.analyte_mass_pg, 1.0);
    assert_eq!(dialog.operating_frequency_khz, 5.0);

    // Initial topology and S-parameters
    assert_eq!(dialog.skin_solver.topology.winding_number, 1);
    assert!(
        dialog.funnel_solver.s_parameters.non_reciprocal_isolation_db >= 35.0,
        "Isolation must be >= 35.0 dB, got {}",
        dialog.funnel_solver.s_parameters.non_reciprocal_isolation_db
    );
    assert!(
        dialog.funnel_solver.s_parameters.sensitivity_enhancement_factor >= 1000.0,
        "Sensitivity enhancement must be >= 1000x, got {}",
        dialog.funnel_solver.s_parameters.sensitivity_enhancement_factor
    );
}

#[test]
fn test_non_hermitian_skin_dialog_presets() {
    let mut dialog = NonHermitianSkinDialog::default();

    // Preset: Maximal Funnel (g = 0.60)
    dialog.asymmetry_g = 0.60;
    dialog.analyte_mass_pg = 1.0;
    dialog.chain_length = 30;
    dialog.recompute();

    assert!(
        dialog.funnel_solver.s_parameters.non_reciprocal_isolation_db >= 40.0,
        "Maximal funnel isolation must be >= 40.0 dB, got {}",
        dialog.funnel_solver.s_parameters.non_reciprocal_isolation_db
    );
    assert!(
        dialog.skin_solver.boundary_localization_fraction() >= 0.85,
        "Boundary localization must be >= 85%, got {}",
        dialog.skin_solver.boundary_localization_fraction()
    );

    // Preset: Reciprocal Limit (g = 0.0)
    dialog.asymmetry_g = 0.0;
    dialog.recompute();

    assert_eq!(dialog.skin_solver.topology.winding_number, 0);
    assert!(
        dialog.funnel_solver.s_parameters.non_reciprocal_isolation_db < 1.0,
        "Reciprocal chain must have near-zero isolation, got {}",
        dialog.funnel_solver.s_parameters.non_reciprocal_isolation_db
    );

    // Preset: Sub-Picogram Sensor
    dialog.asymmetry_g = 0.50;
    dialog.analyte_mass_pg = 0.01;
    dialog.recompute();

    assert!(
        dialog.funnel_solver.s_parameters.sensor_frequency_shift_hz > 0.05,
        "Sub-picogram analyte must yield detectable frequency shift, got {}",
        dialog.funnel_solver.s_parameters.sensor_frequency_shift_hz
    );
    assert!(
        dialog.funnel_solver.s_parameters.sensitivity_enhancement_factor >= 1000.0,
        "Sub-picogram sensitivity enhancement must be >= 1000x, got {}",
        dialog.funnel_solver.s_parameters.sensitivity_enhancement_factor
    );
}

#[test]
fn test_non_hermitian_skin_headless_render() {
    let mut dialog = NonHermitianSkinDialog::default();
    let ctx = egui::Context::default();

    // Tab 1: ChainAndSkinCanvas
    dialog.active_tab = NonHermitianSkinDialogTab::ChainAndSkinCanvas;
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out1.textures_delta.clear();

    // Tab 2: ComplexSpectrumAndWinding
    dialog.active_tab = NonHermitianSkinDialogTab::ComplexSpectrumAndWinding;
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out2.textures_delta.clear();

    // Tab 3: GeneralizedBrillouinZone
    dialog.active_tab = NonHermitianSkinDialogTab::GeneralizedBrillouinZone;
    let mut out3 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out3.textures_delta.clear();

    // Tab 4: UltrasensitiveSensorCurve
    dialog.active_tab = NonHermitianSkinDialogTab::UltrasensitiveSensorCurve;
    let mut out4 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out4.textures_delta.clear();

    // Tab 5: DirectionalFunnelSParameters
    dialog.active_tab = NonHermitianSkinDialogTab::DirectionalFunnelSParameters;
    let mut out5 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out5.textures_delta.clear();
}
