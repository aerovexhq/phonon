#![deny(unsafe_code)]

use phonon_gui::widgets::acoustic_bic_dialog::{
    AcousticBicDialog, AcousticBicDialogTab,
};
use phonon_solver::acoustic_bic::BicKind;

#[test]
fn test_acoustic_bic_dialog_initialization() {
    let dialog = AcousticBicDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        AcousticBicDialogTab::PolarizationVortexMap
    );
    assert_eq!(dialog.bic_kind, BicKind::SymmetryProtectedGamma);
    assert_eq!(dialog.asymmetry_parameter, 0.05);
    assert_eq!(dialog.resonance_freq_hz, 4000.0);
    assert_eq!(dialog.intrinsic_loss_hz, 0.5);
    assert_eq!(dialog.fano_asymmetry_q, -2.0);
    assert_eq!(dialog.cavity_diameter_mm, 24.0);

    // Initial metrics verification
    let metrics = &dialog.engine.metrics;
    assert_eq!(metrics.topological_vortex_charge, 1);
    assert!(
        metrics.peak_q_factor >= 1500.0,
        "Loaded Q must be >= 1500, got {:.1}",
        metrics.peak_q_factor
    );
    assert!(
        metrics.peak_field_enhancement >= 1000.0,
        "Cavity enhancement must be >= 1000x, got {:.1}x",
        metrics.peak_field_enhancement
    );
    assert!(
        metrics.oam_mode_purity_pct >= 95.0,
        "OAM purity must be >= 95%, got {:.2}%",
        metrics.oam_mode_purity_pct
    );
    assert!(
        metrics.min_transmission_dip_db <= -10.0,
        "Fano dip must be <= -10 dB, got {:.2} dB",
        metrics.min_transmission_dip_db
    );
}

#[test]
fn test_acoustic_bic_dialog_presets() {
    let mut dialog = AcousticBicDialog::default();

    // Preset 1: Symmetry-Protected Gamma-BIC (q = +1, alpha = 0.0)
    dialog.bic_kind = BicKind::SymmetryProtectedGamma;
    dialog.asymmetry_parameter = 0.0;
    dialog.recompute();
    assert_eq!(dialog.engine.metrics.topological_vortex_charge, 1);
    assert_eq!(dialog.engine.metrics.radiative_linewidth_hz, 0.0);
    assert!(dialog.engine.metrics.peak_q_factor >= 3900.0);

    // Preset 2: Friedrich-Wintgen Off-Gamma BIC (q = -1)
    dialog.bic_kind = BicKind::FriedrichWintgenOffGamma;
    dialog.asymmetry_parameter = 0.0;
    dialog.recompute();
    assert_eq!(dialog.engine.metrics.topological_vortex_charge, -1);
    assert_eq!(dialog.engine.solver.bic_singularity_pos, (0.35, 0.0));

    // Preset 3: High-Q Quasi-BIC (alpha = 0.05)
    dialog.bic_kind = BicKind::SymmetryProtectedGamma;
    dialog.asymmetry_parameter = 0.05;
    dialog.recompute();
    assert_eq!(dialog.engine.metrics.topological_vortex_charge, 1);
    assert!(dialog.engine.metrics.peak_q_factor >= 1500.0);

    // Preset 4: Radiative Vortex Emitter (alpha = 0.15)
    dialog.bic_kind = BicKind::SymmetryProtectedGamma;
    dialog.asymmetry_parameter = 0.15;
    dialog.recompute();
    assert!(dialog.engine.metrics.radiative_linewidth_hz > 5.0);
    assert!(dialog.engine.metrics.oam_mode_purity_pct >= 95.0);
}

#[test]
fn test_acoustic_bic_headless_render() {
    let mut dialog = AcousticBicDialog::default();
    let ctx = egui::Context::default();

    // Tab 1: PolarizationVortexMap
    dialog.active_tab = AcousticBicDialogTab::PolarizationVortexMap;
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out1.textures_delta.clear();

    // Tab 2: DivergingQFactor
    dialog.active_tab = AcousticBicDialogTab::DivergingQFactor;
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out2.textures_delta.clear();

    // Tab 3: FanoTransmissionSpectrum
    dialog.active_tab = AcousticBicDialogTab::FanoTransmissionSpectrum;
    let mut out3 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out3.textures_delta.clear();

    // Tab 4: QuasiBicScaling
    dialog.active_tab = AcousticBicDialogTab::QuasiBicScaling;
    let mut out4 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out4.textures_delta.clear();

    // Tab 5: NearFieldVortexProfile
    dialog.active_tab = AcousticBicDialogTab::NearFieldVortexProfile;
    let mut out5 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out5.textures_delta.clear();

    // Window show pass when open
    dialog.is_open = true;
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();
}
