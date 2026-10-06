#![deny(unsafe_code)]

use egui::Context;
use phonon_gui::widgets::octupole_insulator_dialog::{OctupoleDialogTab, OctupoleInsulatorDialog};
use phonon_solver::octupole_insulator::OctupolePhase;

#[test]
fn test_octupole_dialog_initialization() {
    let dialog = OctupoleInsulatorDialog::default();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, OctupoleDialogTab::SpatialDensity3D);
    assert_eq!(dialog.gamma, 2.0);
    assert_eq!(dialog.lambda, 10.0);
    assert_eq!(dialog.nx, 2);
    assert_eq!(dialog.ny, 2);
    assert_eq!(dialog.nz, 2);
    assert_eq!(dialog.disorder_w, 0.0);
    assert_eq!(dialog.selected_corner_idx, 0);

    // Topological state
    assert!(dialog.result.is_topological);
    assert_eq!(dialog.result.corner_states.len(), 8);
    assert!(dialog.result.energy_confinement_ratio >= 0.85);
    assert!(dialog.result.quality_factor >= 1.0e5);
    assert!(dialog.result.bulk_bandgap > 20.0);
    assert_eq!(dialog.hamiltonian.quantized_octupole_moment(), 0.5);
    assert_eq!(dialog.band_data.len(), 41);
}

#[test]
fn test_octupole_dialog_presets() {
    let mut dialog = OctupoleInsulatorDialog::default();

    // Preset 1: Topological Octupole (O_xyz = 1/2)
    dialog.gamma = 2.0;
    dialog.lambda = 10.0;
    dialog.disorder_w = 0.0;
    dialog.recompute();
    assert_eq!(dialog.hamiltonian.phase(), OctupolePhase::TopologicalOctupole);
    assert_eq!(dialog.hamiltonian.quantized_octupole_moment(), 0.5);
    assert_eq!(dialog.result.corner_states.len(), 8);
    assert!(dialog.result.energy_confinement_ratio >= 0.85);

    // Preset 2: Trivial Cubic Insulator (O_xyz = 0)
    dialog.gamma = 10.0;
    dialog.lambda = 2.0;
    dialog.disorder_w = 0.0;
    dialog.recompute();
    assert_eq!(dialog.hamiltonian.phase(), OctupolePhase::TrivialInsulator);
    assert_eq!(dialog.hamiltonian.quantized_octupole_moment(), 0.0);
    assert!(!dialog.result.is_topological);
    assert_eq!(dialog.result.corner_states.len(), 0);

    // Preset 3: High-Q Corner Nanocavity
    dialog.gamma = 1.0;
    dialog.lambda = 12.0;
    dialog.disorder_w = 0.0;
    dialog.recompute();
    assert!(dialog.result.is_topological);
    assert!(dialog.result.energy_confinement_ratio >= 0.90);
    assert!(dialog.result.quality_factor >= 1.0e6);

    // Preset 4: Disordered HOTI Cube
    dialog.gamma = 2.0;
    dialog.lambda = 10.0;
    dialog.disorder_w = 1.5;
    dialog.recompute();
    assert!(dialog.result.is_topological);
    assert_eq!(dialog.result.corner_states.len(), 8);
    assert!(dialog.result.energy_confinement_ratio >= 0.70);
}

#[test]
fn test_octupole_dialog_headless_render() {
    let mut dialog = OctupoleInsulatorDialog::default();
    let ctx = Context::default();

    // Tab 1: SpatialDensity3D
    dialog.active_tab = OctupoleDialogTab::SpatialDensity3D;
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out1.textures_delta.clear();

    // Tab 2: BulkDispersion
    dialog.active_tab = OctupoleDialogTab::BulkDispersion;
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out2.textures_delta.clear();

    // Tab 3: DiscreteSpectrum
    dialog.active_tab = OctupoleDialogTab::DiscreteSpectrum;
    let mut out3 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out3.textures_delta.clear();

    // Tab 4: GappingHierarchy
    dialog.active_tab = OctupoleDialogTab::GappingHierarchy;
    let mut out4 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out4.textures_delta.clear();

    // Tab 5: DefectRobustness
    dialog.active_tab = OctupoleDialogTab::DefectRobustness;
    let mut out5 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out5.textures_delta.clear();
}
