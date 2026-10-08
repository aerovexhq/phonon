#![deny(unsafe_code)]

//! GUI and headless integration tests for Topological Josephson Memory & Phase-Slip Crossbar Dialog (Phase 449).

use egui::Context;
use phonon_gui::widgets::topological_josephson_memory_dialog::{
    TopologicalJosephsonMemoryDialog, TopologicalJosephsonTab,
};
use std::time::Instant;

#[test]
fn test_topological_josephson_dialog_cold_boot() {
    let start = Instant::now();
    let dialog = TopologicalJosephsonMemoryDialog::new_fast();
    let duration = start.elapsed();

    // Verify sub-5ms cold boot latency
    assert!(
        duration.as_millis() < 5,
        "Cold boot latency {:?} must be < 5ms",
        duration
    );

    // Verify initial state
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, TopologicalJosephsonTab::TopologicalJosephsonJunction);
    assert_eq!(dialog.cached_crossbar_cells.len(), 64);
    assert!(dialog.cached_audit.all_passed());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_topological_josephson_dialog_tab_switching() {
    let mut dialog = TopologicalJosephsonMemoryDialog::new_fast();

    let tabs = [
        TopologicalJosephsonTab::TopologicalJosephsonJunction,
        TopologicalJosephsonTab::ChiralSotSwitching,
        TopologicalJosephsonTab::QuantumPhaseSlipQubit,
        TopologicalJosephsonTab::CryogenicCrossbarMatrix,
        TopologicalJosephsonTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_topological_josephson_dialog_recompute_and_toggle() {
    let mut dialog = TopologicalJosephsonMemoryDialog::new_fast();

    // Test crossbar cell toggle
    let initial_bit = dialog.cached_crossbar_cells[0].bit_value;
    dialog.toggle_cell(0, 0);
    assert_ne!(dialog.cached_crossbar_cells[0].bit_value, initial_bit);

    // Test parameter modification and recompute
    dialog.critical_current_ua = 12.0;
    dialog.recompute();

    assert!(dialog.cached_cpr_metrics.conventional_critical_current_ua >= 12.0);
    assert!(dialog.cached_audit.all_passed());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_topological_josephson_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = TopologicalJosephsonMemoryDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        TopologicalJosephsonTab::TopologicalJosephsonJunction,
        TopologicalJosephsonTab::ChiralSotSwitching,
        TopologicalJosephsonTab::QuantumPhaseSlipQubit,
        TopologicalJosephsonTab::CryogenicCrossbarMatrix,
        TopologicalJosephsonTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
        assert_eq!(dialog.active_tab, tab);
    }

    assert!(dialog.is_open);
}
