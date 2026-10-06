#![deny(unsafe_code)]

//! GUI test suite for RHBD DRC, Fast SEL Quenching & Autonomous Self-Healing Dialog.

use egui::Context;
use phonon_gui::widgets::rhbd_self_healing_dialog::{RhbdSelfHealingDialog, RhbdTab};
use std::time::Instant;

#[test]
fn test_rhbd_self_healing_dialog_initialization_and_cold_boot() {
    let start = Instant::now();
    let dialog = RhbdSelfHealingDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "RhbdSelfHealingDialog::new_fast() must initialize in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, RhbdTab::LayoutDrcInspector);
    assert_eq!(dialog.substrate_sheet_res_ohm_sq, 25.0);
    assert_eq!(dialog.ion_strike_charge_pc, 2.5);
    assert_eq!(dialog.detection_delay_ns, 10.0);
    assert_eq!(dialog.cutoff_switch_delay_ns, 25.0);
    assert!(!dialog.cached_quenched_v_curve.is_empty());
    assert!(!dialog.cached_unquenched_v_curve.is_empty());
    assert!(!dialog.cached_quenched_temp_curve.is_empty());
    assert!(!dialog.cached_unquenched_temp_curve.is_empty());
}

#[test]
fn test_rhbd_self_healing_dialog_recompute_and_parameter_updates() {
    let mut dialog = RhbdSelfHealingDialog::new();

    dialog.substrate_sheet_res_ohm_sq = 30.0;
    dialog.ion_strike_charge_pc = 3.2;
    dialog.detection_delay_ns = 12.0;
    dialog.cutoff_switch_delay_ns = 22.0;

    dialog.recompute_sim();

    let report = dialog.sim.report();
    assert_eq!(report.sel_quenching_latency_ns, 34.0);
    assert!(report.sel_burnout_prevented);
    assert!(report.sel_quenched_peak_temp_c < 100.0);
    assert!(report.flight_continuity_certified);

    assert!(!dialog.cached_quenched_v_curve.is_empty());
    assert!(!dialog.cached_quenched_temp_curve.is_empty());
    assert!(!dialog.cached_unquenched_temp_curve.is_empty());
}

#[test]
fn test_rhbd_self_healing_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dialog = RhbdSelfHealingDialog::new();
    dialog.is_open = true;

    let tabs = [
        RhbdTab::LayoutDrcInspector,
        RhbdTab::SelCrowbarScope,
        RhbdTab::ParasiticThyristorTCAD,
        RhbdTab::AutonomousTaskMigration,
        RhbdTab::AirworthinessCompliance,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
    }

    // Test alias ui(&ctx)
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();
}
