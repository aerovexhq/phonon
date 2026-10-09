#![deny(unsafe_code)]

use egui::Context;
use phonon_gui::widgets::moire_valley_qubit_dialog::{
    MoireValleyQubitDialog, MoireValleyQubitTab,
};
use std::time::Instant;

#[test]
fn test_moire_valley_qubit_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = MoireValleyQubitDialog::new_fast();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 5,
        "Fast boot latency must be < 5 ms, took {} ms",
        duration.as_millis()
    );

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        MoireValleyQubitTab::ValleyQubitDynamics
    );

    assert_eq!(dialog.cached_audit.passed_count, 10);
    assert_eq!(dialog.cached_audit.total_count, 10);
    assert!(dialog.cached_audit.is_all_pass());
    assert!(dialog.cached_qubit_metrics.gate_fidelity >= 0.999);
    assert!(dialog.cached_memory_metrics.storage_lifetime_ms >= 1.5);
    assert!(dialog.cached_bus_metrics.entanglement.concurrence >= 0.90);
}

#[test]
fn test_moire_valley_qubit_dialog_parameter_mutation_and_recompute() {
    let mut dialog = MoireValleyQubitDialog::new_fast();

    dialog.twist_angle_deg = 1.10;
    dialog.rabi_freq_mhz = 30.0;
    dialog.node_count = 10;
    dialog.recompute();

    assert!(!dialog.cached_rabi_trajectory.is_empty());
    assert!(!dialog.cached_dispersion.is_empty());
    assert!(!dialog.cached_storage_decay.is_empty());
    assert!(!dialog.cached_bus_spectrum.is_empty());

    assert!(dialog.cached_qubit_metrics.effective_rabi_freq_mhz >= 25.0);
    assert!(dialog.cached_bus_metrics.end_to_end_insertion_loss_db <= 0.40);
    assert!(dialog.cached_audit.passed_count >= 8);
}

#[test]
fn test_moire_valley_qubit_dialog_tab_switching() {
    let mut dialog = MoireValleyQubitDialog::new_fast();

    let tabs = [
        MoireValleyQubitTab::ValleyQubitDynamics,
        MoireValleyQubitTab::FlatBandMemoryCell,
        MoireValleyQubitTab::CryogenicValleyBus,
        MoireValleyQubitTab::EntanglementRouting,
        MoireValleyQubitTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!dialog.active_tab.label().is_empty());
    }
}

#[test]
fn test_moire_valley_qubit_dialog_headless_egui_render() {
    let mut dialog = MoireValleyQubitDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();

    for tab in [
        MoireValleyQubitTab::ValleyQubitDynamics,
        MoireValleyQubitTab::FlatBandMemoryCell,
        MoireValleyQubitTab::CryogenicValleyBus,
        MoireValleyQubitTab::EntanglementRouting,
        MoireValleyQubitTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        output.textures_delta.clear();
    }
}

