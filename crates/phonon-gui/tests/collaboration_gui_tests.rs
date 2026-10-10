#![deny(unsafe_code)]

//! GUI and headless integration tests for the P2P CAD Collaboration & CRDT Dialog.

use std::time::Instant;
use egui::Context;
use phonon_gui::widgets::{CollaborationDialog, CollaborationTab};

#[test]
fn test_collaboration_dialog_initialization_and_cold_boot() {
    let start = Instant::now();
    let dialog = CollaborationDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(!dialog.is_open, "Dialog starts closed");
    assert_eq!(dialog.active_tab, CollaborationTab::MeshNetwork);
    assert_eq!(dialog.engine.network.connected_peer_count(), 2);
    assert_eq!(dialog.audit_report.score, 10);
    assert!(
        elapsed.as_micros() < 2000,
        "CollaborationDialog::new_fast() must execute well under 2.0 ms (measured {} us)",
        elapsed.as_micros()
    );
}

#[test]
fn test_collaboration_dialog_tab_navigation() {
    let mut dialog = CollaborationDialog::new_fast();

    dialog.active_tab = CollaborationTab::CrdtDeltaSync;
    assert_eq!(dialog.active_tab, CollaborationTab::CrdtDeltaSync);

    dialog.active_tab = CollaborationTab::LivePresence;
    assert_eq!(dialog.active_tab, CollaborationTab::LivePresence);

    dialog.active_tab = CollaborationTab::SecurityCrypto;
    assert_eq!(dialog.active_tab, CollaborationTab::SecurityCrypto);

    dialog.active_tab = CollaborationTab::AuditTelemetry;
    assert_eq!(dialog.active_tab, CollaborationTab::AuditTelemetry);

    dialog.active_tab = CollaborationTab::MeshNetwork;
    assert_eq!(dialog.active_tab, CollaborationTab::MeshNetwork);
}

#[test]
fn test_collaboration_dialog_delta_broadcast() {
    let mut dialog = CollaborationDialog::new_fast();
    let initial_comps = dialog.engine.crdt.active_component_count();

    // Broadcast component addition
    dialog.engine.broadcast_component("cmp-r99", "R99", "Resistor", "1k", "0805", 300.0, 200.0);
    assert_eq!(dialog.engine.crdt.active_component_count(), initial_comps + 1);

    // Broadcast component deletion
    dialog.engine.broadcast_component_deletion("cmp-r99");
    assert_eq!(dialog.engine.crdt.active_component_count(), initial_comps);
}

#[test]
fn test_collaboration_dialog_headless_egui_render_pass() {
    let ctx = Context::default();
    let mut dialog = CollaborationDialog::new_fast();
    dialog.is_open = true;

    // Render across each tab to verify zero UI panics
    let tabs = [
        CollaborationTab::MeshNetwork,
        CollaborationTab::CrdtDeltaSync,
        CollaborationTab::LivePresence,
        CollaborationTab::SecurityCrypto,
        CollaborationTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render(ui.ctx());
        });
        out.textures_delta.clear();
    }

    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();

    assert!(dialog.is_open);
}
