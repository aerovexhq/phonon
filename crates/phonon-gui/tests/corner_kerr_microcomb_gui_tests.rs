#![deny(unsafe_code)]

//! GUI test suite for Phase 427: Topological Corner Kerr Microcomb & Dissipative Soliton Dialog.

use egui::Context;
use phonon_gui::widgets::corner_kerr_microcomb_dialog::{
    CornerCombTab, CornerKerrMicrocombDialog,
};

#[test]
fn test_corner_kerr_microcomb_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = CornerKerrMicrocombDialog::new_fast();
    let duration = start.elapsed();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, CornerCombTab::TopologicalCornerMode);
    assert!(
        duration.as_millis() < 5,
        "Cold-boot latency must be under 5 ms (got {:?})",
        duration
    );
    assert!(dialog.cached_topo.corner_confinement_ratio >= 90.0);
    assert!(dialog.cached_soliton.contrast_ratio_db >= 15.0);
    assert!(dialog.cached_comb.total_comb_lines >= 25);
    assert!(dialog.cached_trans.transduction_efficiency_percent >= 40.0);
}

#[test]
fn test_corner_kerr_microcomb_dialog_tab_switching() {
    let mut dialog = CornerKerrMicrocombDialog::default();

    dialog.active_tab = CornerCombTab::TopologicalCornerMode;
    assert_eq!(dialog.active_tab.label(), "Topological Corner Mode");

    dialog.active_tab = CornerCombTab::DissipativeKerrSoliton;
    assert_eq!(dialog.active_tab.label(), "Dissipative Kerr Soliton");

    dialog.active_tab = CornerCombTab::FrequencyCombSpectrum;
    assert_eq!(dialog.active_tab.label(), "Microcomb Spectrum");

    dialog.active_tab = CornerCombTab::QuantumTransduction;
    assert_eq!(dialog.active_tab.label(), "Quantum Transduction");

    dialog.active_tab = CornerCombTab::AuditTelemetry;
    assert_eq!(dialog.active_tab.label(), "Physics Audit & Telemetry");
}

#[test]
fn test_corner_kerr_microcomb_parameter_adjustment_and_recompute() {
    let mut dialog = CornerKerrMicrocombDialog::default();

    dialog.corner_resonance_freq_ghz = 2.4;
    dialog.pump_power_mw = 1.2;
    dialog.pump_detuning_delta_khz = 55.0;
    dialog.recompute();

    assert_eq!(dialog.processor.soliton_solver.params.corner_resonance_freq_ghz, 2.4);
    assert_eq!(dialog.processor.soliton_solver.params.pump_power_mw, 1.2);
    assert_eq!(dialog.processor.soliton_solver.params.pump_detuning_delta_khz, 55.0);
    assert!(dialog.cached_audit.all_passed);
    assert_eq!(dialog.cached_audit.total_score, 10);
}

#[test]
fn test_corner_kerr_microcomb_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = CornerKerrMicrocombDialog::default();
    dialog.is_open = true;

    // Test render pass for all tabs
    let tabs = [
        CornerCombTab::TopologicalCornerMode,
        CornerCombTab::DissipativeKerrSoliton,
        CornerCombTab::FrequencyCombSpectrum,
        CornerCombTab::QuantumTransduction,
        CornerCombTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
