#![deny(unsafe_code)]

//! Automated GUI test suite for Wafer-Scale Yield, DFM & Harvesting Economics Dialog.

use egui::Context;
use phonon_gui::widgets::wafer_yield_dialog::{
    WaferMapColorMode, WaferYieldDialog, WaferYieldTab,
};
use std::time::Instant;

#[test]
fn test_wafer_yield_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dlg = WaferYieldDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "WaferYieldDialog::new_fast() must boot in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dlg.is_open);
    assert_eq!(dlg.active_tab, WaferYieldTab::WaferMap);
    assert_eq!(dlg.color_mode, WaferMapColorMode::HarvestSku);
    assert!(dlg.cached_report.gross_dpw > 400);
    assert!(dlg.cached_report.gross_revenue_usd > 100000.0);
    assert!(!dlg.cached_curves.is_empty());
}

#[test]
fn test_wafer_yield_dialog_solve_and_telemetry() {
    let mut dlg = WaferYieldDialog::new_fast();

    // Execute full solve
    dlg.run_solve();
    assert!(dlg.cached_report.gross_dpw > 400);
    assert!(dlg.cached_report.total_good_dies > 0);
    assert!(dlg.cached_report.functional_yield_pct >= dlg.cached_report.unharvested_yield_pct);
    assert!(dlg.cached_report.gross_revenue_usd >= dlg.cached_report.unharvested_revenue_usd);
    assert!(dlg.cached_report.gross_margin_pct > 0.0);
    assert!(!dlg.sim.simulated_dies.is_empty());
}

#[test]
fn test_wafer_yield_dialog_tab_and_color_mode_navigation() {
    let mut dlg = WaferYieldDialog::new_fast();

    dlg.active_tab = WaferYieldTab::DefectYieldModels;
    assert_eq!(dlg.active_tab, WaferYieldTab::DefectYieldModels);

    dlg.active_tab = WaferYieldTab::RadialDefectProfile;
    assert_eq!(dlg.active_tab, WaferYieldTab::RadialDefectProfile);

    dlg.active_tab = WaferYieldTab::ChipletHarvesting;
    assert_eq!(dlg.active_tab, WaferYieldTab::ChipletHarvesting);

    dlg.active_tab = WaferYieldTab::WaferEconomics;
    assert_eq!(dlg.active_tab, WaferYieldTab::WaferEconomics);

    dlg.color_mode = WaferMapColorMode::ThresholdVoltage;
    assert_eq!(dlg.color_mode, WaferMapColorMode::ThresholdVoltage);

    dlg.color_mode = WaferMapColorMode::GateLength;
    assert_eq!(dlg.color_mode, WaferMapColorMode::GateLength);
}

#[test]
fn test_wafer_yield_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dlg = WaferYieldDialog::new_with_baseline();
    dlg.is_open = true;

    let tabs = [
        WaferYieldTab::WaferMap,
        WaferYieldTab::DefectYieldModels,
        WaferYieldTab::RadialDefectProfile,
        WaferYieldTab::ChipletHarvesting,
        WaferYieldTab::WaferEconomics,
    ];

    for tab in tabs {
        dlg.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dlg.ui(ui.ctx());
        });
        out.textures_delta.clear();
    }

    // Test show alias
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dlg.show(ui.ctx());
    });
    out_window.textures_delta.clear();
}
