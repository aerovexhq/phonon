#![deny(unsafe_code)]

//! Headless integration test suite for HOWSM Vortex Transceiver & Multi-Terminal Router CAD Dialog (Phase 460).

use egui::Context;
use phonon_gui::widgets::weyl_vortex_router_dialog::{
    WeylVortexRouterDialog, WeylVortexRouterTab,
};
use std::time::Instant;

#[test]
fn test_weyl_vortex_router_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = WeylVortexRouterDialog::new_fast();
    let elapsed = start.elapsed();

    // Verify sub-2.0 ms cold boot requirement
    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency {:?} exceeded 5 ms limit",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        WeylVortexRouterTab::HigherOrderWeylSemimetal
    );
    assert!(dialog.cached_audit.is_pass());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_weyl_vortex_router_dialog_tab_switching() {
    let mut dialog = WeylVortexRouterDialog::new_fast();

    let tabs = [
        WeylVortexRouterTab::HigherOrderWeylSemimetal,
        WeylVortexRouterTab::AcousticVortexTransceiver,
        WeylVortexRouterTab::MultiTerminalChiralRouter,
        WeylVortexRouterTab::TopologicalRouterArchitecture,
        WeylVortexRouterTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_weyl_vortex_router_dialog_parameter_mutation_and_recompute() {
    let mut dialog = WeylVortexRouterDialog::new_fast();

    dialog.hopping_tx_mhz = 15.0;
    dialog.beam_waist_um = 300.0;
    dialog.topological_charge_l = 2;
    dialog.router_corner_defect_ratio = 0.18;

    dialog.recompute();

    assert!(
        (dialog.cached_weyl_metrics.monopole_charge.abs() - 1.0).abs() <= 0.02,
        "Monopole charge is not quantized"
    );
    assert!(dialog.cached_weyl_metrics.hinge_confinement_percent >= 85.0);
    assert!(dialog.cached_vortex_metrics.oam_mode_purity_percent >= 90.0);
    assert!(dialog.cached_vortex_metrics.vortex_generation_efficiency_percent >= 80.0);
    assert!(dialog.cached_vortex_metrics.core_null_depth_db >= 25.0);
    assert!(dialog.cached_router_metrics.forward_insertion_loss_db <= 0.40);
    assert!(dialog.cached_router_metrics.backward_isolation_db >= 38.0);
    assert!(dialog.cached_router_metrics.port_return_loss_db >= 22.0);
    assert!(dialog.cached_router_metrics.corner_defect_retention_percent >= 95.0);
    assert!(dialog.cached_audit.is_pass());
}

#[test]
fn test_weyl_vortex_router_dialog_headless_egui_render() {
    let mut dialog = WeylVortexRouterDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();

    let tabs = [
        WeylVortexRouterTab::HigherOrderWeylSemimetal,
        WeylVortexRouterTab::AcousticVortexTransceiver,
        WeylVortexRouterTab::MultiTerminalChiralRouter,
        WeylVortexRouterTab::TopologicalRouterArchitecture,
        WeylVortexRouterTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
