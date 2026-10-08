#![deny(unsafe_code)]

//! GUI Integration test suite for Phase 442:
//! Topological Acoustic Synthetic Gauge Field & Non-Abelian Holonomic Quantum Gate Processor Dialog.

use egui::Context;
use phonon_gui::widgets::synthetic_gauge_holonomy_dialog::{
    SyntheticGaugeHolonomyDialog, SyntheticGaugeTab,
};
use phonon_solver::synthetic_gauge_holonomy::SyntheticHolonomicGateKind;

#[test]
fn test_synthetic_gauge_dialog_cold_boot_and_audit() {
    let start = std::time::Instant::now();
    let dialog = SyntheticGaugeHolonomyDialog::new_fast();
    let boot_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, SyntheticGaugeTab::SyntheticGaugeLattice);
    assert!(
        boot_time_ms < 5.0,
        "Cold boot latency {:.2} ms exceeds 5.0 ms threshold",
        boot_time_ms
    );

    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_synthetic_gauge_dialog_tab_switching() {
    let mut dialog = SyntheticGaugeHolonomyDialog::new_fast();

    let tabs = [
        SyntheticGaugeTab::SyntheticGaugeLattice,
        SyntheticGaugeTab::WilczekZeeHolonomy,
        SyntheticGaugeTab::HolonomicGateSynthesis,
        SyntheticGaugeTab::DegenerateManifold,
        SyntheticGaugeTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_synthetic_gauge_dialog_interactive_recompute() {
    let mut dialog = SyntheticGaugeHolonomyDialog::new_fast();

    // 1. Initial cached metrics check
    assert_eq!(dialog.cached_gauge_metrics.synthetic_chern_number, 1.0);
    assert!(dialog.cached_holonomy_metrics.commutator_norm >= 0.50);
    assert!(dialog.cached_gate_metrics.process_fidelity >= 0.999);

    // 2. Adjust synthetic gauge lattice parameters and recompute
    dialog.bare_coupling_mhz = 12.0;
    dialog.modulation_amplitude_mhz = 5.0;
    dialog.has_edge_defect = true;
    dialog.recompute();
    assert!(dialog.cached_gauge_metrics.defect_immunity_ratio >= 0.95);

    // 3. Switch target gate to ControlledPhaseCZ and recompute
    dialog.target_gate = SyntheticHolonomicGateKind::ControlledPhaseCZ;
    dialog.inter_qubit_coupling_mhz = 15.0;
    dialog.recompute();
    assert!(dialog.cached_gate_metrics.process_fidelity >= 0.999);
    assert!(dialog.cached_gate_metrics.entangling_concurrence >= 0.95);

    // 4. Switch target gate to PauliX and recompute
    dialog.target_gate = SyntheticHolonomicGateKind::PauliX;
    dialog.recompute();
    assert!(dialog.cached_gate_metrics.process_fidelity >= 0.999);
}

#[test]
fn test_synthetic_gauge_dialog_headless_render() {
    let mut dialog = SyntheticGaugeHolonomyDialog::new_fast();
    dialog.is_open = true;

    let ctx = Context::default();

    // Test render pass for all 5 tabs
    let tabs = [
        SyntheticGaugeTab::SyntheticGaugeLattice,
        SyntheticGaugeTab::WilczekZeeHolonomy,
        SyntheticGaugeTab::HolonomicGateSynthesis,
        SyntheticGaugeTab::DegenerateManifold,
        SyntheticGaugeTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_dialog_contents(ui);
        });
        output.textures_delta.clear();
        assert_eq!(dialog.active_tab, tab);
    }

    // Also verify standard window ui() call
    let mut window_output = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    window_output.textures_delta.clear();

    assert!(dialog.is_open);
}
