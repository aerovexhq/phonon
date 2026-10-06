#![deny(unsafe_code)]

//! Automated GUI test suite for Silicon Lifecycle Management (SLM) & On-Die Telemetry Digital Twin Dialog.

use egui::Context;
use phonon_gui::widgets::silicon_lifecycle_dialog::{SiliconLifecycleDialog, SlmTab};
use phonon_solver::silicon_lifecycle::ProtocolType;
use std::time::Instant;

#[test]
fn test_slm_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dlg = SiliconLifecycleDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "SiliconLifecycleDialog::new_fast() must boot in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dlg.is_open);
    assert_eq!(dlg.active_tab, SlmTab::DieThermalMap);
    assert!(dlg.cached_report.total_sensors > 0);
    assert!(dlg.cached_report.silicon_health_score_pct > 80.0);
    assert!(!dlg.transient_history.is_empty());
}

#[test]
fn test_slm_dialog_tab_navigation_and_protocol_switch() {
    let mut dlg = SiliconLifecycleDialog::new_fast();

    dlg.active_tab = SlmTab::TelemetryStreams;
    assert_eq!(dlg.active_tab, SlmTab::TelemetryStreams);

    dlg.active_tab = SlmTab::PlacementAdvisor;
    assert_eq!(dlg.active_tab, SlmTab::PlacementAdvisor);

    dlg.active_tab = SlmTab::TransientsAndAlarms;
    assert_eq!(dlg.active_tab, SlmTab::TransientsAndAlarms);

    dlg.active_tab = SlmTab::DigitalTwinAnalytics;
    assert_eq!(dlg.active_tab, SlmTab::DigitalTwinAnalytics);

    dlg.active_tab = SlmTab::DieThermalMap;
    assert_eq!(dlg.active_tab, SlmTab::DieThermalMap);

    // Protocol switching
    dlg.sim.set_protocol(ProtocolType::PcieMctpPldm);
    assert_eq!(dlg.sim.telemetry.active_protocol, ProtocolType::PcieMctpPldm);
}

#[test]
fn test_slm_dialog_placement_optimization_trigger() {
    let mut dlg = SiliconLifecycleDialog::new_fast();
    dlg.placement_budget = 32;

    dlg.sim.optimize_sensor_placement(dlg.placement_budget);
    dlg.cached_studies = dlg.sim.placement_studies.clone();

    assert_eq!(dlg.cached_studies.len(), 3);
    let opt_study = &dlg.cached_studies[2];
    assert!(opt_study.strategy_name.contains("Gradient-Optimized"));
    assert!(opt_study.max_unobserved_delta_c > 0.0);
}

#[test]
fn test_slm_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dlg = SiliconLifecycleDialog::new_fast();
    dlg.is_open = true;

    let tabs = [
        SlmTab::DieThermalMap,
        SlmTab::TelemetryStreams,
        SlmTab::PlacementAdvisor,
        SlmTab::TransientsAndAlarms,
        SlmTab::DigitalTwinAnalytics,
    ];

    for tab in tabs {
        dlg.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dlg.ui(ui.ctx());
        });
        out.textures_delta.clear();
    }

    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dlg.show(ui.ctx());
    });
    out_window.textures_delta.clear();
}

