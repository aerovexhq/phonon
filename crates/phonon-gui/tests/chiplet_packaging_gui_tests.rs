#![deny(unsafe_code)]

//! GUI test suite for 2.5D/3D Multi-Die & Chiplet Packaging CAD Studio Dialog.

use egui::Context;
use phonon_gui::widgets::chiplet_packaging_dialog::{ChipletPackagingDialog, PackagingTab};
use phonon_solver::chiplet_packaging::{PackagingArchitecture, UcieDataRateGbps};
use std::time::Instant;

#[test]
fn test_chiplet_packaging_dialog_initialization_and_cold_boot() {
    let start = Instant::now();
    let dialog = ChipletPackagingDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "ChipletPackagingDialog::new_fast() must initialize in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, PackagingTab::ArchitectureStack3D);
    assert!(!dialog.cached_eye_upper_inner.is_empty());
    assert!(!dialog.cached_eye_lower_inner.is_empty());
    assert!(!dialog.cached_s21_curve.is_empty());
    assert!(!dialog.cached_s11_curve.is_empty());
    assert!(!dialog.cached_warpage_temp_curve.is_empty());

    let rep = dialog.sim.report();
    assert!(rep.eye_height_mv > 0.0);
    assert!(rep.eye_width_ui > 0.0);
    assert!(rep.tsv_inductance_ph > 0.0);
    assert!(rep.is_fully_qualified);
}

#[test]
fn test_chiplet_packaging_dialog_recompute_and_parameter_updates() {
    let mut dialog = ChipletPackagingDialog::new();

    // Switch architecture to Intel EMIB
    dialog.sim.apply_architecture_preset(PackagingArchitecture::Intel_EMIB);
    dialog.recompute_sim();
    assert_eq!(dialog.sim.architecture, PackagingArchitecture::Intel_EMIB);

    // Switch data rate to 32 Gbps
    dialog.sim.ucie_params.data_rate = UcieDataRateGbps::Rate32Gbps;
    dialog.sim.ucie_params.trace_length_mm = 1.0;
    dialog.recompute_sim();

    let rep = dialog.sim.report();
    assert_eq!(rep.architecture, PackagingArchitecture::Intel_EMIB);
    assert!(!dialog.cached_eye_upper_inner.is_empty());
    assert!(!dialog.cached_s21_curve.is_empty());
    assert!(!dialog.cached_warpage_temp_curve.is_empty());

    // Switch to Organic 2.5D
    dialog.sim.apply_architecture_preset(PackagingArchitecture::Organic_2_5D);
    dialog.recompute_sim();
    assert_eq!(dialog.sim.architecture, PackagingArchitecture::Organic_2_5D);
}

#[test]
fn test_chiplet_packaging_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dialog = ChipletPackagingDialog::new();
    dialog.is_open = true;

    let tabs = [
        PackagingTab::ArchitectureStack3D,
        PackagingTab::UcieEyeDiagram,
        PackagingTab::TsvRdlParasitics,
        PackagingTab::ThermoMechanicalWarpage,
        PackagingTab::AssemblyReliability,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
    }

    // Test alias ui(&ctx) and window show
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();
}
