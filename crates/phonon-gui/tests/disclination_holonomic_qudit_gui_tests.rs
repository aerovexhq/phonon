#![deny(unsafe_code)]

//! Headless integration test suite for Disclination Cavity & Holonomic Qudit Processor CAD Dialog (Phase 463).

use egui::Context;
use phonon_gui::widgets::disclination_holonomic_qudit_dialog::{
    DisclinationHolonomicQuditDialog, DisclinationHolonomicTab,
};
use phonon_solver::disclination_holonomic_qudit::{
    FrankAngleKind, QuditDimension, QuditHolonomicGateKind,
};
use std::time::Instant;

#[test]
fn test_disclination_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = DisclinationHolonomicQuditDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency {:?} exceeded 5 ms limit",
        elapsed
    );
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        DisclinationHolonomicTab::DisclinationCavityFlatBands
    );
    assert!(dialog.cached_audit.is_pass());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_disclination_dialog_tab_switching() {
    let mut dialog = DisclinationHolonomicQuditDialog::new_fast();
    let tabs = [
        DisclinationHolonomicTab::DisclinationCavityFlatBands,
        DisclinationHolonomicTab::HolonomicQuditGates,
        DisclinationHolonomicTab::MultiQuditProcessor,
        DisclinationHolonomicTab::CryogenicReadout,
        DisclinationHolonomicTab::AuditTelemetry,
    ];
    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_disclination_dialog_parameter_mutation_and_recompute() {
    let mut dialog = DisclinationHolonomicQuditDialog::new_fast();

    dialog.bare_frequency_mhz = 160.0;
    dialog.frank_angle = FrankAngleKind::C4Plus90Deg;
    dialog.intracell_hopping_gamma_mhz = 3.2;
    dialog.intercell_hopping_lambda_mhz = 13.0;
    dialog.qudit_dimension = QuditDimension::QutritD3;
    dialog.selected_gate = QuditHolonomicGateKind::ShiftX;
    dialog.cavity_count = 5;
    dialog.dilution_temp_mk = 12.0;

    dialog.recompute();

    assert!(dialog.cached_cavity_metrics.bulk_bandgap_mhz >= 4.0);
    assert!(dialog.cached_cavity_metrics.fractional_charge_error <= 0.05);
    assert!(dialog.cached_cavity_metrics.core_energy_confinement_percent >= 82.0);
    assert!(dialog.cached_cavity_metrics.cavity_quality_factor >= 25_000.0);
    assert!(dialog.cached_qudit_metrics.gate_fidelity_percent >= 99.5);
    assert!(dialog.cached_qudit_metrics.diabatic_leakage_rate <= 1.0e-4);
    assert!(dialog.cached_qudit_metrics.non_abelian_commutator_norm >= 0.50);
    assert!(dialog.cached_processor_metrics.entangling_concurrence >= 0.90);
    assert!(dialog.cached_processor_metrics.thermal_phonon_occupancy <= 1.0e-3);
    assert!(dialog.cached_processor_metrics.readout_snr_db >= 16.0);
    assert!(dialog.cached_processor_metrics.readout_fidelity_percent >= 99.5);
    assert!(dialog.cached_audit.is_pass());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_disclination_dialog_headless_egui_render() {
    let mut dialog = DisclinationHolonomicQuditDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();
    let tabs = [
        DisclinationHolonomicTab::DisclinationCavityFlatBands,
        DisclinationHolonomicTab::HolonomicQuditGates,
        DisclinationHolonomicTab::MultiQuditProcessor,
        DisclinationHolonomicTab::CryogenicReadout,
        DisclinationHolonomicTab::AuditTelemetry,
    ];
    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
