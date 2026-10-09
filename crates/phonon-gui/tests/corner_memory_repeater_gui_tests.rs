#![deny(unsafe_code)]

use egui::Context;
use phonon_gui::widgets::corner_memory_repeater_dialog::{
    CornerMemoryRepeaterDialog, CornerMemoryRepeaterTab,
};
use std::time::Instant;

#[test]
fn test_corner_memory_repeater_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = CornerMemoryRepeaterDialog::new_fast();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 5,
        "Fast boot latency must be < 5 ms, took {} ms",
        duration.as_millis()
    );

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        CornerMemoryRepeaterTab::CornerStateMemory
    );

    assert_eq!(dialog.cached_audit.passed_count, 10);
    assert_eq!(dialog.cached_audit.total_count, 10);
    assert!(dialog.cached_audit.is_all_pass());
    assert!(dialog.cached_memory_metrics.corner_confinement_ratio >= 0.90);
    assert!(dialog.cached_memory_metrics.storage_lifetime_ms >= 2.0);
    assert!(dialog.cached_transduction_metrics.reverse_chiral_isolation_db >= 40.0);
    assert!(dialog.cached_transduction_metrics.state_transfer_fidelity >= 0.995);
    assert!(dialog.cached_repeater_metrics.squeezing_depth_db >= 6.5);
    assert!(dialog.cached_repeater_metrics.duan_simon_nullifier <= 0.40);
}

#[test]
fn test_corner_memory_repeater_dialog_parameter_mutation_and_recompute() {
    let mut dialog = CornerMemoryRepeaterDialog::new_fast();

    dialog.intracell_gamma_mhz = 2.0;
    dialog.intercell_lambda_mhz = 12.0;
    dialog.chiral_isolation_db = 48.0;
    dialog.repeater_nodes = 6;
    dialog.recompute();

    assert!(!dialog.cached_spatial_points.is_empty());
    assert!(!dialog.cached_transmission_spectrum.is_empty());
    assert!(!dialog.cached_repeater_nodes.is_empty());
    assert!(!dialog.cached_squeezing_profile.is_empty());

    assert_eq!(dialog.cached_repeater_nodes.len(), 6);
    assert!(dialog.cached_memory_metrics.bulk_bandgap_mhz >= 18.0);
    assert!(dialog.cached_transduction_metrics.reverse_chiral_isolation_db >= 45.0);
    assert!(dialog.cached_audit.passed_count >= 9);
}

#[test]
fn test_corner_memory_repeater_dialog_tab_switching() {
    let mut dialog = CornerMemoryRepeaterDialog::new_fast();

    let tabs = [
        CornerMemoryRepeaterTab::CornerStateMemory,
        CornerMemoryRepeaterTab::SyntheticGaugeTransduction,
        CornerMemoryRepeaterTab::CvQuantumRepeater,
        CornerMemoryRepeaterTab::SpatialLatticeCanvas,
        CornerMemoryRepeaterTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!dialog.active_tab.label().is_empty());
    }
}

#[test]
fn test_corner_memory_repeater_dialog_headless_egui_render() {
    let mut dialog = CornerMemoryRepeaterDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();

    for tab in [
        CornerMemoryRepeaterTab::CornerStateMemory,
        CornerMemoryRepeaterTab::SyntheticGaugeTransduction,
        CornerMemoryRepeaterTab::CvQuantumRepeater,
        CornerMemoryRepeaterTab::SpatialLatticeCanvas,
        CornerMemoryRepeaterTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        output.textures_delta.clear();
    }
}
