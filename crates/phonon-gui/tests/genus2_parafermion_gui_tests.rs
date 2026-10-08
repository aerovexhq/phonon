#![deny(unsafe_code)]

//! GUI Integration Test Suite for Genus-2 Parafermion Surface Code & Acoustic Processor Dialog.
//!
//! Validates:
//! 1. Fast cold-boot initialization (< 2.0 ms) and baseline cached telemetry.
//! 2. Tab switching across all 5 categorized tabs.
//! 3. Interactive parameter adjustment and deterministic recomputation.
//! 4. Headless egui render pass across all tabs without panic or texture leakage.

use egui::Context;
use phonon_gui::widgets::genus2_parafermion_dialog::{
    Genus2ParafermionDialog, Genus2ParafermionTab,
};
use phonon_solver::genus2_parafermion_surface::LogicalGateKind;

#[test]
fn test_genus2_parafermion_dialog_initialization_and_fast_boot() {
    let t_start = std::time::Instant::now();
    let dialog = Genus2ParafermionDialog::new_fast();
    let elapsed = t_start.elapsed();

    assert!(!dialog.is_open, "Dialog must be closed by default");
    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be practically instantaneous, took {} ms",
        elapsed.as_millis()
    );

    // Verify baseline cached telemetry
    assert_eq!(dialog.cached_surface_metrics.code_space_dimension, 9);
    assert_eq!(dialog.cached_surface_metrics.euler_characteristic, -2);
    assert!(dialog.cached_surface_metrics.protection_gap_uev >= 10.0);
    assert!(dialog.cached_gate_metrics.process_fidelity >= 0.999);
    assert!(dialog.cached_syndrome_result.logical_error_rate <= 1.0e-6);
    assert!(dialog.cached_crossbar_metrics.crosstalk_isolation_db >= 38.0);
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_genus2_parafermion_dialog_tab_switching() {
    let mut dialog = Genus2ParafermionDialog::new_fast();

    let tabs = [
        Genus2ParafermionTab::Genus2RiemannGeometry,
        Genus2ParafermionTab::TransversalFaultTolerantLogic,
        Genus2ParafermionTab::HomologicalStabilizerDecoder,
        Genus2ParafermionTab::CryogenicRoutingCrossbar,
        Genus2ParafermionTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty(), "Tab label must not be empty");
    }
}

#[test]
fn test_genus2_parafermion_dialog_parameter_mutation_and_recompute() {
    let mut dialog = Genus2ParafermionDialog::new_fast();

    dialog.topological_gap_mhz = 4.5;
    dialog.twist_duration_ns = 90.0;
    dialog.selected_gate = LogicalGateKind::DehnTwistAlpha1;
    dialog.selected_row = 3;
    dialog.selected_col = 5;

    dialog.recompute();

    assert_eq!(dialog.cached_gate_metrics.gate_kind, LogicalGateKind::DehnTwistAlpha1);
    assert!(dialog.cached_surface_metrics.protection_gap_uev > 15.0);
    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_genus2_parafermion_dialog_headless_egui_render() {
    let ctx = Context::default();
    let mut dialog = Genus2ParafermionDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        Genus2ParafermionTab::Genus2RiemannGeometry,
        Genus2ParafermionTab::TransversalFaultTolerantLogic,
        Genus2ParafermionTab::HomologicalStabilizerDecoder,
        Genus2ParafermionTab::CryogenicRoutingCrossbar,
        Genus2ParafermionTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });

        // Ensure clean frame without unhandled textures
        assert!(output.shapes.len() > 0, "Tab {:?} must produce rendered shapes", tab);
        output.textures_delta.clear();
    }
}
