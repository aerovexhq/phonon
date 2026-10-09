#![deny(unsafe_code)]

//! Headless integration test suite for Floquet Chiral Magnon Crossbar & Entanglement Router CAD Dialog (Phase 464).

use egui::Context;
use phonon_gui::widgets::floquet_magnon_crossbar_dialog::{
    FloquetMagnonCrossbarDialog, FloquetMagnonCrossbarTab,
};
use std::time::Instant;

#[test]
fn test_floquet_crossbar_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = FloquetMagnonCrossbarDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency {:?} exceeded 5 ms limit",
        elapsed
    );
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        FloquetMagnonCrossbarTab::ChiralFloquetTransceiver
    );
    assert!(dialog.cached_audit.is_pass());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_floquet_crossbar_dialog_tab_switching() {
    let mut dialog = FloquetMagnonCrossbarDialog::new_fast();
    let tabs = [
        FloquetMagnonCrossbarTab::ChiralFloquetTransceiver,
        FloquetMagnonCrossbarTab::SyntheticCirculatorArray,
        FloquetMagnonCrossbarTab::ClusterEntanglementRouter,
        FloquetMagnonCrossbarTab::CryoCmosInterface,
        FloquetMagnonCrossbarTab::AuditTelemetry,
    ];
    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_floquet_crossbar_dialog_parameter_mutation_and_recompute() {
    let mut dialog = FloquetMagnonCrossbarDialog::new_fast();

    dialog.bare_acoustic_freq_ghz = 5.0;
    dialog.floquet_drive_freq_mhz = 550.0;
    dialog.floquet_drive_amplitude_oe = 14.0;
    dialog.magnetoelastic_coupling_mhz = 45.0;
    dialog.port_count = 8;
    dialog.cluster_node_count = 8;
    dialog.parametric_squeezing_r = 1.05;
    dialog.dilution_temp_mk = 12.0;

    dialog.recompute();

    assert!(dialog.cached_transceiver_metrics.insertion_loss_db <= 0.30);
    assert!(dialog.cached_transceiver_metrics.reverse_isolation_db >= 40.0);
    assert!(dialog.cached_transceiver_metrics.transduction_bandwidth_mhz >= 120.0);
    assert!(dialog.cached_circulator_metrics.directivity_db >= 38.0);
    assert!(dialog.cached_circulator_metrics.return_loss_db >= 22.0);
    assert!(dialog.cached_router_metrics.squeezing_depth_db >= 6.0);
    assert!(dialog.cached_router_metrics.duan_simon_nullifier <= 0.50);
    assert!(dialog.cached_router_metrics.entanglement_routing_fidelity_percent >= 99.2);
    assert!(dialog.cached_router_metrics.thermal_phonon_occupancy <= 1.0e-3);
    assert!(dialog.cached_audit.is_pass());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_floquet_crossbar_dialog_headless_egui_render() {
    let mut dialog = FloquetMagnonCrossbarDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();
    let tabs = [
        FloquetMagnonCrossbarTab::ChiralFloquetTransceiver,
        FloquetMagnonCrossbarTab::SyntheticCirculatorArray,
        FloquetMagnonCrossbarTab::ClusterEntanglementRouter,
        FloquetMagnonCrossbarTab::CryoCmosInterface,
        FloquetMagnonCrossbarTab::AuditTelemetry,
    ];
    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
