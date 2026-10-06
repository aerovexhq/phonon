#![deny(unsafe_code)]

use phonon_gui::widgets::pt_symmetric_dialog::{
    PtSymmetricDialog, PtSymmetricDialogTab,
};
use phonon_solver::pt_symmetric_acoustic::PtPhaseClassification;

#[test]
fn test_pt_symmetric_dialog_initialization() {
    let dialog = PtSymmetricDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        PtSymmetricDialogTab::PressureFieldProfile
    );
    assert_eq!(dialog.coupling_kappa_hz, 500.0);
    assert_eq!(dialog.gain_loss_gamma_hz, 500.0);
    assert_eq!(dialog.resonance_freq_hz, 3000.0);
    assert_eq!(dialog.background_loss_hz, 5.0);
    assert_eq!(dialog.metamaterial_length_mm, 50.0);

    // Initial exceptional point physics
    let phase = dialog.engine.solver.params.phase();
    assert_eq!(phase, PtPhaseClassification::ExceptionalPoint);

    let metrics = &dialog.engine.metrics;
    assert!(
        metrics.center_reflection_left_db <= -30.0,
        "Left reflection should be <= -30 dB at EP, got {:.2} dB",
        metrics.center_reflection_left_db
    );
    assert!(
        metrics.center_reflection_right_db >= -5.0,
        "Right reflection should be >= -5 dB at EP, got {:.2} dB",
        metrics.center_reflection_right_db
    );
    assert!(
        metrics.unidirectional_contrast_ratio * 100.0 >= 95.0,
        "Contrast should be >= 95%, got {:.2}%",
        metrics.unidirectional_contrast_ratio * 100.0
    );
    assert!(
        metrics.generalized_unitarity_residual < 1e-4,
        "Unitarity residual should be < 1e-4, got {:e}",
        metrics.generalized_unitarity_residual
    );
    assert!(
        dialog.engine.solver.metrics.petermann_factor >= 50.0,
        "Petermann factor at EP should be >= 50, got {:.2}",
        dialog.engine.solver.metrics.petermann_factor
    );
}

#[test]
fn test_pt_symmetric_dialog_presets() {
    let mut dialog = PtSymmetricDialog::default();

    // Preset 1: Unidirectional Invisibility EP (gamma = kappa = 500 Hz)
    dialog.coupling_kappa_hz = 500.0;
    dialog.gain_loss_gamma_hz = 500.0;
    dialog.background_loss_hz = 5.0;
    dialog.recompute();
    assert_eq!(
        dialog.engine.solver.params.phase(),
        PtPhaseClassification::ExceptionalPoint
    );
    assert!(dialog.engine.metrics.unidirectional_contrast_ratio * 100.0 >= 95.0);

    // Preset 2: Exact PT Phase (gamma/kappa = 0.6)
    dialog.coupling_kappa_hz = 500.0;
    dialog.gain_loss_gamma_hz = 300.0;
    dialog.recompute();
    assert_eq!(
        dialog.engine.solver.params.phase(),
        PtPhaseClassification::ExactPtSymmetric
    );
    assert!(dialog.engine.metrics.unidirectional_contrast_ratio * 100.0 < 95.0);

    // Preset 3: Broken PT Phase (gamma/kappa = 1.3)
    dialog.coupling_kappa_hz = 500.0;
    dialog.gain_loss_gamma_hz = 650.0;
    dialog.recompute();
    assert_eq!(
        dialog.engine.solver.params.phase(),
        PtPhaseClassification::BrokenPtSymmetric
    );

    // Preset 4: Passive Lossy Reference (gamma = 0)
    dialog.gain_loss_gamma_hz = 0.0;
    dialog.recompute();
    assert_eq!(
        dialog.engine.solver.params.phase(),
        PtPhaseClassification::ExactPtSymmetric
    );
    assert!(dialog.engine.metrics.unidirectional_contrast_ratio * 100.0 < 50.0);
}

#[test]
fn test_pt_symmetric_headless_render() {
    let mut dialog = PtSymmetricDialog::default();
    let ctx = egui::Context::default();

    // Tab 1: PressureFieldProfile
    dialog.active_tab = PtSymmetricDialogTab::PressureFieldProfile;
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out1.textures_delta.clear();

    // Tab 2: EigenvalueBifurcation
    dialog.active_tab = PtSymmetricDialogTab::EigenvalueBifurcation;
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out2.textures_delta.clear();

    // Tab 3: UnidirectionalSpectra
    dialog.active_tab = PtSymmetricDialogTab::UnidirectionalSpectra;
    let mut out3 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out3.textures_delta.clear();

    // Tab 4: InvisibilityContrast
    dialog.active_tab = PtSymmetricDialogTab::InvisibilityContrast;
    let mut out4 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out4.textures_delta.clear();

    // Tab 5: SMatrixCoalescence
    dialog.active_tab = PtSymmetricDialogTab::SMatrixCoalescence;
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
