#![deny(unsafe_code)]

//! GUI test suite for Phase 426: Chiral Magnon-Phonon Heat Transistor & Thermal Diode Dialog.

use egui::Context;
use phonon_gui::widgets::chiral_heat_transistor_dialog::{
    ChiralHeatTransistorDialog, HeatTransistorTab,
};

#[test]
fn test_chiral_heat_transistor_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = ChiralHeatTransistorDialog::new_fast();
    let duration = start.elapsed();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, HeatTransistorTab::ThermalRectification);
    assert!(
        duration.as_millis() < 5,
        "Cold-boot latency must be under 5 ms (got {:?})",
        duration
    );
    assert!(dialog.cached_diode_metrics.peak_rectification_ratio >= 25.0);
    assert!(dialog.cached_trans_metrics.max_differential_gain >= 5.0);
}

#[test]
fn test_chiral_heat_transistor_dialog_tab_switching() {
    let mut dialog = ChiralHeatTransistorDialog::default();

    dialog.active_tab = HeatTransistorTab::ThermalRectification;
    assert_eq!(dialog.active_tab.label(), "Thermal Rectification");

    dialog.active_tab = HeatTransistorTab::HeatTransistorAmplification;
    assert_eq!(dialog.active_tab.label(), "Heat Transistor");

    dialog.active_tab = HeatTransistorTab::FloquetMagnonPhononCoupling;
    assert_eq!(dialog.active_tab.label(), "Floquet & EP Coupling");

    dialog.active_tab = HeatTransistorTab::RealSpaceThermalCanvas;
    assert_eq!(dialog.active_tab.label(), "Thermal Canvas");

    dialog.active_tab = HeatTransistorTab::AuditTelemetry;
    assert_eq!(dialog.active_tab.label(), "Physics Audit & Telemetry");
}

#[test]
fn test_chiral_heat_transistor_parameter_adjustment_and_recompute() {
    let mut dialog = ChiralHeatTransistorDialog::default();

    dialog.drive_freq_ghz = 3.2;
    dialog.diode_coupling_g_mhz = 22.0;
    dialog.gate_bias = 0.75;
    dialog.recompute();

    assert_eq!(dialog.processor.diode.params.drive_freq_ghz, 3.2);
    assert_eq!(dialog.processor.diode.params.coupling_g_mhz, 22.0);
    assert_eq!(dialog.processor.transistor.params.gate_bias, 0.75);
    assert!(dialog.cached_audit.all_passed);
    assert_eq!(dialog.cached_audit.total_score, 10);
}

#[test]
fn test_chiral_heat_transistor_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = ChiralHeatTransistorDialog::default();
    dialog.is_open = true;

    // Test render pass for all tabs
    let tabs = [
        HeatTransistorTab::ThermalRectification,
        HeatTransistorTab::HeatTransistorAmplification,
        HeatTransistorTab::FloquetMagnonPhononCoupling,
        HeatTransistorTab::RealSpaceThermalCanvas,
        HeatTransistorTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
