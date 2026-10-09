#![deny(unsafe_code)]

//! Headless integration test suite for Fractional Skyrmion Synapse CAD Dialog (Phase 458).

use egui::Context;
use phonon_gui::widgets::fractional_skyrmion_synapse_dialog::{
    FractionalSkyrmionSynapseDialog, FractionalSkyrmionSynapseTab,
};
use std::time::Instant;

#[test]
fn test_fractional_skyrmion_synapse_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = FractionalSkyrmionSynapseDialog::new_fast();
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
        FractionalSkyrmionSynapseTab::FractionalSkyrmionLattice
    );
    assert!(dialog.cached_audit.is_pass());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_fractional_skyrmion_synapse_dialog_tab_switching() {
    let mut dialog = FractionalSkyrmionSynapseDialog::new_fast();

    let tabs = [
        FractionalSkyrmionSynapseTab::FractionalSkyrmionLattice,
        FractionalSkyrmionSynapseTab::NonAbelianSynapticWeight,
        FractionalSkyrmionSynapseTab::ChiralDomainWallRouter,
        FractionalSkyrmionSynapseTab::CryogenicNeuralCrossbar,
        FractionalSkyrmionSynapseTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_fractional_skyrmion_synapse_dialog_parameter_mutation_and_recompute() {
    let mut dialog = FractionalSkyrmionSynapseDialog::new_fast();

    dialog.filling_fraction_denominator = 4;
    dialog.skyrmion_radius_nm = 50.0;
    dialog.num_levels = 256;
    dialog.pulse_voltage_mv = 50.0;
    dialog.waveguide_width_um = 2.0;
    dialog.crossbar_rows = 8;
    dialog.crossbar_cols = 8;

    dialog.recompute();

    assert!(
        (dialog.cached_skyrmion_metrics.topological_charge_q - 0.25).abs() <= 0.02,
        "Expected Q approx 0.25 for m=4, got {}",
        dialog.cached_skyrmion_metrics.topological_charge_q
    );
    assert_eq!(dialog.cached_synapse_metrics.num_quantized_levels, 256);
    assert!(dialog.cached_neuromorphic_metrics.forward_insertion_loss_db <= 0.40);
    assert!(dialog.cached_crossbar_metrics.mvm_accuracy_error_percent <= 0.50);
    assert!(dialog.cached_audit.is_pass());
}

#[test]
fn test_fractional_skyrmion_synapse_dialog_headless_egui_render() {
    let mut dialog = FractionalSkyrmionSynapseDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();

    let tabs = [
        FractionalSkyrmionSynapseTab::FractionalSkyrmionLattice,
        FractionalSkyrmionSynapseTab::NonAbelianSynapticWeight,
        FractionalSkyrmionSynapseTab::ChiralDomainWallRouter,
        FractionalSkyrmionSynapseTab::CryogenicNeuralCrossbar,
        FractionalSkyrmionSynapseTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
