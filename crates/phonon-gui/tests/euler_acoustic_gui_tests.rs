#![deny(unsafe_code)]

use phonon_gui::widgets::euler_acoustic_dialog::{
    EulerAcousticDialog, EulerAcousticDialogTab,
};
use phonon_solver::euler_acoustic::EulerPhase;
use std::f64::consts::PI;

#[test]
fn test_euler_acoustic_dialog_initialization() {
    let dialog = EulerAcousticDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        EulerAcousticDialogTab::EulerCurvatureMap
    );
    assert_eq!(dialog.hopping_ta_hz, 300.0);
    assert_eq!(dialog.coupling_t12_hz, 400.0);
    assert_eq!(dialog.delta_12_hz, 150.0);
    assert_eq!(dialog.mass_diff_hz, 400.0);
    assert!(dialog.is_domain_wall);

    // Initial topological metrics
    let metrics = &dialog.engine.metrics;
    assert_eq!(metrics.quantized_euler_class, 1);
    assert!(
        metrics.bulk_gap_1_hz > 50.0,
        "Bulk gap 1 must be > 50 Hz, got {:.1}",
        metrics.bulk_gap_1_hz
    );
    assert!(
        metrics.edge_confinement_pct >= 80.0,
        "Domain wall edge confinement must be >= 80%, got {:.1}%",
        metrics.edge_confinement_pct
    );
    assert!(
        metrics.transmission_efficiency_db >= -0.5,
        "Transmission efficiency must be >= -0.5 dB, got {:.2} dB",
        metrics.transmission_efficiency_db
    );
    assert!(
        (metrics.frame_rotation_angle_rad - PI).abs() < 1e-4,
        "Frame rotation must be pi rad, got {:.4}",
        metrics.frame_rotation_angle_rad
    );
}

#[test]
fn test_euler_acoustic_dialog_presets() {
    let mut dialog = EulerAcousticDialog::default();

    // Preset 1: Topological Euler Phase (chi = 1)
    dialog.hopping_ta_hz = 300.0;
    dialog.coupling_t12_hz = 400.0;
    dialog.delta_12_hz = 150.0;
    dialog.mass_diff_hz = 400.0;
    dialog.is_domain_wall = true;
    dialog.recompute();
    assert_eq!(dialog.engine.metrics.quantized_euler_class, 1);
    assert!(dialog.engine.metrics.edge_confinement_pct >= 80.0);

    // Preset 2: Trivial Real Insulator (chi = 0)
    dialog.hopping_ta_hz = 150.0;
    dialog.coupling_t12_hz = 200.0;
    dialog.delta_12_hz = 0.0;
    dialog.mass_diff_hz = 0.0;
    dialog.recompute();
    assert_eq!(dialog.engine.metrics.quantized_euler_class, 0);
    assert_ne!(dialog.engine.solver.phase, EulerPhase::TopologicalEuler);

    // Preset 3: Multi-Gap Nodal Braiding Semimetal
    dialog.hopping_ta_hz = 300.0;
    dialog.coupling_t12_hz = 350.0;
    dialog.delta_12_hz = 20.0;
    dialog.mass_diff_hz = 60.0;
    dialog.recompute();
    assert_eq!(dialog.engine.solver.patch_euler_invariant, 1);

    // Preset 4: Domain Wall Waveguide
    dialog.is_domain_wall = true;
    dialog.delta_12_hz = 180.0;
    dialog.mass_diff_hz = 450.0;
    dialog.recompute();
    assert!(dialog.engine.metrics.edge_confinement_pct >= 80.0);
    assert!(dialog.engine.metrics.bulk_isolation_db >= 25.0);
}

#[test]
fn test_euler_acoustic_headless_render() {
    let mut dialog = EulerAcousticDialog::default();
    let ctx = egui::Context::default();

    // Tab 1: EulerCurvatureMap
    dialog.active_tab = EulerAcousticDialogTab::EulerCurvatureMap;
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out1.textures_delta.clear();

    // Tab 2: BulkThreeBandDispersion
    dialog.active_tab = EulerAcousticDialogTab::BulkThreeBandDispersion;
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out2.textures_delta.clear();

    // Tab 3: RibbonEdgeDispersion
    dialog.active_tab = EulerAcousticDialogTab::RibbonEdgeDispersion;
    let mut out3 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out3.textures_delta.clear();

    // Tab 4: NodalBraidingPatch
    dialog.active_tab = EulerAcousticDialogTab::NodalBraidingPatch;
    let mut out4 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out4.textures_delta.clear();

    // Tab 5: DomainWallSpatialProfile
    dialog.active_tab = EulerAcousticDialogTab::DomainWallSpatialProfile;
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
