#![deny(unsafe_code)]

//! GUI test suite for Physics-Based Silicon Aging, Reliability & Electromigration (EM) CAD Dialog.

use egui::Context;
use phonon_gui::widgets::silicon_aging_dialog::{AgingTab, SiliconAgingDialog};
use std::time::Instant;

#[test]
fn test_silicon_aging_dialog_initialization_and_cold_boot() {
    let start = Instant::now();
    let dialog = SiliconAgingDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "SiliconAgingDialog::new_fast() must initialize in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, AgingTab::BtiDrift);
    assert!(!dialog.cached_curves.time_years.is_empty());
    assert!(!dialog.cached_curves.nbti_shifts_mv.is_empty());
    assert!(!dialog.cached_curves.pbti_shifts_mv.is_empty());
    assert!(!dialog.cached_curves.hci_shifts_mv.is_empty());
    assert!(!dialog.cached_curves.total_shifts_mv.is_empty());
    assert!(!dialog.cached_curves.freq_ghz.is_empty());
    assert!(!dialog.cached_curves.tddb_failure_prob.is_empty());
    assert!(!dialog.cached_curves.required_guardband_mv.is_empty());

    let tel = &dialog.cached_telemetry;
    assert_eq!(tel.v_dd_nominal_v, 0.85);
    assert!(tel.ten_year_total_shift_mv > 0.0);
    assert!(tel.ten_year_freq_penalty_pct > 0.0);
    assert!(tel.tddb_10yr_failure_prob > 0.0);
    assert!(tel.recommended_guardband_mv > 0.0);
    assert!(tel.is_10yr_qualified);
}

#[test]
fn test_silicon_aging_dialog_recompute_and_parameter_updates() {
    let mut dialog = SiliconAgingDialog::new_fast();

    // Modify operating voltage and temperature
    dialog.sim.v_dd_nominal_v = 0.90;
    dialog.sim.temp_c = 95.0;
    dialog.sim.duty_cycle = 0.60;

    dialog.recompute_all();

    assert!(!dialog.cached_curves.time_years.is_empty());
    assert!(!dialog.cached_em_results.is_empty());

    let tel = &dialog.cached_telemetry;
    assert_eq!(tel.v_dd_nominal_v, 0.90);
    assert_eq!(tel.operating_temp_c, 95.0);
    assert_eq!(tel.duty_cycle, 0.60);
    assert!(tel.ten_year_total_shift_mv > 20.0);
    assert!(tel.recommended_guardband_mv > 0.0);
}

#[test]
fn test_silicon_aging_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dialog = SiliconAgingDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        AgingTab::BtiDrift,
        AgingTab::HciDegradation,
        AgingTab::TddbReliability,
        AgingTab::InterconnectEm,
        AgingTab::DatacenterDerating,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
    }

    // Test show method alias
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.show(ui.ctx());
    });
    out_window.textures_delta.clear();
}
