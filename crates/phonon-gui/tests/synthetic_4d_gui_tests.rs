#![deny(unsafe_code)]

use phonon_gui::widgets::synthetic_4d_dialog::{
    Synthetic4dDialog, Synthetic4dDialogTab,
};

#[test]
fn test_synthetic_4d_dialog_initialization() {
    let dialog = Synthetic4dDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        Synthetic4dDialogTab::PhysicalToSyntheticProjection
    );
    assert_eq!(dialog.mass_m, 3.0);
    assert_eq!(dialog.hopping_t_khz, 2.5);
    assert_eq!(dialog.drive_field_ey, 1.0);
    assert_eq!(dialog.synthetic_field_bzw, 1.0);
    assert_eq!(dialog.disorder_w, 0.0);

    // Initial topological physics
    assert!(dialog.engine.lattice_solver.params.is_topological());
    assert_eq!(dialog.engine.lattice_solver.params.second_chern_number(), -1);
    assert!(dialog.engine.lattice_solver.calculated_bulk_gap_khz > 0.0);
    assert!(
        dialog.engine.lattice_solver.boundary_confinement_ratio >= 0.85,
        "Boundary localization should be >= 85%, got {:.3}",
        dialog.engine.lattice_solver.boundary_confinement_ratio
    );
    assert!(dialog.engine.metrics.non_linear_hall_current < 0.0);
    assert!(dialog.engine.metrics.chiral_directivity_db >= 20.0);
}

#[test]
fn test_synthetic_4d_dialog_presets() {
    let mut dialog = Synthetic4dDialog::default();

    // Preset 1: Topological 4D QHE
    dialog.mass_m = 3.0;
    dialog.recompute();
    assert_eq!(dialog.engine.lattice_solver.params.second_chern_number(), -1);
    assert!(dialog.engine.lattice_solver.params.is_topological());
    assert!(dialog.engine.metrics.non_linear_hall_conductance < 0.0);

    // Preset 2: High-Chern Phase (m = 1.0, C2 = +3)
    dialog.mass_m = 1.0;
    dialog.recompute();
    assert_eq!(dialog.engine.lattice_solver.params.second_chern_number(), 3);
    assert!(dialog.engine.lattice_solver.params.is_topological());
    assert!(dialog.engine.metrics.non_linear_hall_conductance > 0.0);

    // Preset 3: Trivial 4D Insulator (m = 5.0, C2 = 0)
    dialog.mass_m = 5.0;
    dialog.recompute();
    assert_eq!(dialog.engine.lattice_solver.params.second_chern_number(), 0);
    assert!(!dialog.engine.lattice_solver.params.is_topological());
    assert_eq!(dialog.engine.metrics.non_linear_hall_conductance, 0.0);

    // Preset 4: Disorder Robustness Test (W = 0.25)
    dialog.mass_m = 3.0;
    dialog.disorder_w = 0.25;
    dialog.recompute();
    assert!(
        dialog.engine.metrics.disorder_retention_ratio >= 0.90,
        "Disorder retention should be >= 90%, got {:.3}",
        dialog.engine.metrics.disorder_retention_ratio
    );
}

#[test]
fn test_synthetic_4d_headless_render() {
    let mut dialog = Synthetic4dDialog::default();
    let ctx = egui::Context::default();

    // Tab 1: PhysicalToSyntheticProjection
    dialog.active_tab = Synthetic4dDialogTab::PhysicalToSyntheticProjection;
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out1.textures_delta.clear();

    // Tab 2: BulkDispersion4D
    dialog.active_tab = Synthetic4dDialogTab::BulkDispersion4D;
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out2.textures_delta.clear();

    // Tab 3: BoundaryChiralDispersion
    dialog.active_tab = Synthetic4dDialogTab::BoundaryChiralDispersion;
    let mut out3 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out3.textures_delta.clear();

    // Tab 4: NonLinearHallResponse
    dialog.active_tab = Synthetic4dDialogTab::NonLinearHallResponse;
    let mut out4 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out4.textures_delta.clear();

    // Tab 5: SyntheticFrequencyLadder
    dialog.active_tab = Synthetic4dDialogTab::SyntheticFrequencyLadder;
    let mut out5 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out5.textures_delta.clear();
}
