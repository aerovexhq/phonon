#![deny(unsafe_code)]

use egui::Context;
use phonon_gui::widgets::aah_quasicrystal_dialog::{AahDialogTab, AahQuasicrystalDialog};
use phonon_solver::aah_quasicrystal::{AahHamiltonian, AahModelKind, AahParams, AahPhase};

#[test]
fn test_aah_dialog_initialization() {
    let dialog = AahQuasicrystalDialog::default();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, AahDialogTab::SpatialWavefunction);
    assert_eq!(dialog.hopping_j, 5.0);
    assert_eq!(dialog.modulation_delta, 6.0);
    assert_eq!(dialog.off_diagonal_lambda, 0.45);
    assert_eq!(dialog.num_sites, 50);
    assert_eq!(dialog.model_kind, AahModelKind::GeneralizedAah);

    // Initial calculation state
    assert_eq!(dialog.engine.num_sites, 50);
    assert!(dialog.butterfly_data.len() > 0);
    assert!(dialog.phason_data.len() > 0);
    assert!(dialog.engine.metrics.mobility_edge_detected);
}

#[test]
fn test_aah_dialog_presets() {
    let mut dialog = AahQuasicrystalDialog::default();

    // Preset 1: Coexistent Mobility Edge (Generalized AAH)
    dialog.hopping_j = 1.0;
    dialog.modulation_delta = 1.5;
    dialog.off_diagonal_lambda = 0.5;
    dialog.model_kind = AahModelKind::GeneralizedAah;
    dialog.recompute();
    assert!(dialog.engine.metrics.mobility_edge_detected);
    let me = dialog.engine.metrics.mobility_edge_energy_mhz;
    assert!(me.is_some());
    assert!((me.unwrap() - 1.0).abs() < 1e-6);

    let h1 = AahHamiltonian::new(AahParams::generalized(1.0, 1.5, 0.5));
    assert_eq!(h1.phase(), AahPhase::MobilityEdgeCoexistence);

    // Preset 2: Standard AAH Extended Phase (Delta / 2J < 1)
    dialog.hopping_j = 5.0;
    dialog.modulation_delta = 4.0;
    dialog.off_diagonal_lambda = 0.0;
    dialog.model_kind = AahModelKind::StandardAah;
    dialog.recompute();
    assert!(!dialog.engine.metrics.mobility_edge_detected);
    let h2 = AahHamiltonian::new(AahParams::standard(5.0, 4.0));
    assert_eq!(h2.phase(), AahPhase::DelocalizedExtended);

    // Preset 3: Standard AAH Localized Phase (Delta / 2J > 1)
    dialog.hopping_j = 5.0;
    dialog.modulation_delta = 16.0;
    dialog.off_diagonal_lambda = 0.0;
    dialog.model_kind = AahModelKind::StandardAah;
    dialog.recompute();
    assert!(!dialog.engine.metrics.mobility_edge_detected);
    let h3 = AahHamiltonian::new(AahParams::standard(5.0, 16.0));
    assert_eq!(h3.phase(), AahPhase::ExponentiallyLocalized);

    // Preset 4: Critical Self-Dual Point (Delta / 2J == 1)
    dialog.hopping_j = 5.0;
    dialog.modulation_delta = 10.0;
    dialog.off_diagonal_lambda = 0.0;
    dialog.model_kind = AahModelKind::StandardAah;
    dialog.recompute();
    let h4 = AahHamiltonian::new(AahParams::standard(5.0, 10.0));
    assert_eq!(h4.phase(), AahPhase::CriticalMultifractal);
}

#[test]
fn test_aah_dialog_headless_render() {
    let mut dialog = AahQuasicrystalDialog::default();
    let ctx = Context::default();

    // Tab 1: SpatialWavefunction
    dialog.active_tab = AahDialogTab::SpatialWavefunction;
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out1.textures_delta.clear();

    // Tab 2: HofstadterButterfly
    dialog.active_tab = AahDialogTab::HofstadterButterfly;
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out2.textures_delta.clear();

    // Tab 3: MobilityEdgeScatter
    dialog.active_tab = AahDialogTab::MobilityEdgeScatter;
    let mut out3 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out3.textures_delta.clear();

    // Tab 4: PhasonEvolution
    dialog.active_tab = AahDialogTab::PhasonEvolution;
    let mut out4 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out4.textures_delta.clear();

    // Tab 5: FractalScaling
    dialog.active_tab = AahDialogTab::FractalScaling;
    let mut out5 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out5.textures_delta.clear();
}
