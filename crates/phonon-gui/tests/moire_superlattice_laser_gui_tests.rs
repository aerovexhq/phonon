#![deny(unsafe_code)]

//! Headless integration test suite for the Moire Superlattice Polariton Laser & Valley Sensor CAD Dialog (Phase 462).

use egui::Context;
use phonon_gui::widgets::moire_superlattice_laser_dialog::{
    MoireSuperlatticeLaserDialog, MoireSuperlatticeTab,
};
use std::time::Instant;

#[test]
fn test_moire_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = MoireSuperlatticeLaserDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency {:?} exceeded 5 ms limit",
        elapsed
    );
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        MoireSuperlatticeTab::MoireSuperlatticeFlatBands
    );
    assert!(dialog.cached_audit.is_pass());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_moire_dialog_tab_switching() {
    let mut dialog = MoireSuperlatticeLaserDialog::new_fast();
    let tabs = [
        MoireSuperlatticeTab::MoireSuperlatticeFlatBands,
        MoireSuperlatticeTab::PolaritonLaserCondensation,
        MoireSuperlatticeTab::DistributedValleySensor,
        MoireSuperlatticeTab::TopologicalMoireArchitecture,
        MoireSuperlatticeTab::AuditTelemetry,
    ];
    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_moire_dialog_parameter_mutation_and_recompute() {
    let mut dialog = MoireSuperlatticeLaserDialog::new_fast();
    dialog.pump_power_mw = 3.0;
    dialog.sensor_node_count = 6;
    dialog.recompute();

    assert!(dialog.cached_moire_metrics.flat_band_bandwidth_mhz <= 1.5);
    assert!(dialog.cached_moire_metrics.aa_spatial_confinement_percent >= 80.0);
    assert!(dialog.cached_audit.is_pass());
}

#[test]
fn test_moire_dialog_headless_egui_render() {
    let mut dialog = MoireSuperlatticeLaserDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();
    let tabs = [
        MoireSuperlatticeTab::MoireSuperlatticeFlatBands,
        MoireSuperlatticeTab::PolaritonLaserCondensation,
        MoireSuperlatticeTab::DistributedValleySensor,
        MoireSuperlatticeTab::TopologicalMoireArchitecture,
        MoireSuperlatticeTab::AuditTelemetry,
    ];
    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
