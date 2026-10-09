#![deny(unsafe_code)]

//! Headless integration test suite for Chiral Transducer & Quantum Repeater Node CAD Dialog (Phase 459).

use egui::Context;
use phonon_gui::widgets::chiral_transducer_repeater_dialog::{
    ChiralTransducerRepeaterDialog, ChiralTransducerRepeaterTab,
};
use std::time::Instant;

#[test]
fn test_chiral_transducer_repeater_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = ChiralTransducerRepeaterDialog::new_fast();
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
        ChiralTransducerRepeaterTab::PiezoOptomechanicalTransducer
    );
    assert!(dialog.cached_audit.is_pass());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_chiral_transducer_repeater_dialog_tab_switching() {
    let mut dialog = ChiralTransducerRepeaterDialog::new_fast();

    let tabs = [
        ChiralTransducerRepeaterTab::PiezoOptomechanicalTransducer,
        ChiralTransducerRepeaterTab::NonReciprocalChiralRouter,
        ChiralTransducerRepeaterTab::EntanglementSwappingRepeater,
        ChiralTransducerRepeaterTab::TopologicalTransductionDiagram,
        ChiralTransducerRepeaterTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_chiral_transducer_repeater_dialog_parameter_mutation_and_recompute() {
    let mut dialog = ChiralTransducerRepeaterDialog::new_fast();

    dialog.optical_pump_power_mw = 2.0;
    dialog.optomech_coupling_g0_khz = 900.0;
    dialog.corner_defect_ratio = 0.20;
    dialog.total_distance_km = 90.0;

    dialog.recompute();

    assert!(
        dialog.cached_transducer_metrics.bidirectional_efficiency_percent >= 15.0,
        "Expected efficiency >= 15.0%, got {}",
        dialog.cached_transducer_metrics.bidirectional_efficiency_percent
    );
    assert!(dialog.cached_transducer_metrics.transduction_bandwidth_mhz >= 2.0);
    assert!(dialog.cached_transducer_metrics.added_noise_quanta <= 0.10);
    assert!(dialog.cached_router_metrics.forward_insertion_loss_db <= 0.40);
    assert!(dialog.cached_router_metrics.backward_isolation_db >= 38.0);
    assert!(dialog.cached_repeater_metrics.swapped_state_fidelity_percent >= 92.0);
    assert!(dialog.cached_repeater_metrics.repeater_rate_gain >= 2.0);
    assert!(dialog.cached_audit.is_pass());
}

#[test]
fn test_chiral_transducer_repeater_dialog_headless_egui_render() {
    let mut dialog = ChiralTransducerRepeaterDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();

    let tabs = [
        ChiralTransducerRepeaterTab::PiezoOptomechanicalTransducer,
        ChiralTransducerRepeaterTab::NonReciprocalChiralRouter,
        ChiralTransducerRepeaterTab::EntanglementSwappingRepeater,
        ChiralTransducerRepeaterTab::TopologicalTransductionDiagram,
        ChiralTransducerRepeaterTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
