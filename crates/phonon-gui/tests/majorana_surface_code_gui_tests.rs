#![deny(unsafe_code)]

//! Comprehensive Automated Test Suite for Phase 407 GUI CAD Dialog:
//! Quantum Metamaterial Non-Abelian Majorana Braid Interconnect & Fault-Tolerant Surface Code Co-Processor.

use egui::Context;
use phonon_gui::widgets::majorana_surface_code_dialog::{
    MajoranaSurfaceCodeDialog, MajoranaSurfaceCodeTab,
};
use phonon_solver::majorana_surface_code::{CodeDistance, TargetCliffordGate};
use std::time::Instant;

#[test]
fn test_majorana_surface_code_dialog_initialization() {
    let dialog = MajoranaSurfaceCodeDialog::default();

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        MajoranaSurfaceCodeTab::BraidingCrossbarLattice
    );
    assert_eq!(dialog.code_distance, CodeDistance::Distance3);
    assert_eq!(dialog.selected_gate, TargetCliffordGate::Hadamard);
    assert!(dialog.topological_gap_mhz > 0.0);
    assert!(dialog.braid_duration_ns > 0.0);
    assert!(dialog.cached_audit_report.overall_pass);
    assert_eq!(dialog.cached_audit_report.passed_count, 10);
}

#[test]
fn test_majorana_surface_code_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = MajoranaSurfaceCodeDialog::new_fast();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(
        elapsed_ms < 5.0,
        "Cold boot latency too high: {:.3} ms (target < 5.0 ms)",
        elapsed_ms
    );
    assert!(dialog.cached_audit_report.overall_pass);
}

#[test]
fn test_majorana_surface_code_dialog_tab_switching() {
    let mut dialog = MajoranaSurfaceCodeDialog::default();

    let tabs = [
        MajoranaSurfaceCodeTab::BraidingCrossbarLattice,
        MajoranaSurfaceCodeTab::SurfaceCodeMatrix,
        MajoranaSurfaceCodeTab::FaultTolerantDecoders,
        MajoranaSurfaceCodeTab::MagicDistillationReadout,
        MajoranaSurfaceCodeTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_majorana_surface_code_dialog_parameter_adjustment() {
    let mut dialog = MajoranaSurfaceCodeDialog::default();

    // Change target gate to PhaseS and recompute
    dialog.selected_gate = TargetCliffordGate::PhaseS;
    dialog.topological_gap_mhz = 6.0;
    dialog.braid_duration_ns = 150.0;
    dialog.code_distance = CodeDistance::Distance5;
    dialog.recompute();

    assert_eq!(dialog.cached_compiled_gate.target, TargetCliffordGate::PhaseS);
    assert_eq!(dialog.cached_compiled_gate.braid_sequence, vec![1]);
    assert!(dialog.cached_compiled_gate.process_fidelity >= 0.9990);
    assert_eq!(dialog.coprocessor.surface_patch.checks.len(), 24);
    assert!(dialog.cached_audit_report.overall_pass);
}

#[test]
fn test_majorana_surface_code_dialog_headless_render() {
    let mut dialog = MajoranaSurfaceCodeDialog::default();
    dialog.is_open = true;

    let ctx = Context::default();

    let tabs = [
        MajoranaSurfaceCodeTab::BraidingCrossbarLattice,
        MajoranaSurfaceCodeTab::SurfaceCodeMatrix,
        MajoranaSurfaceCodeTab::FaultTolerantDecoders,
        MajoranaSurfaceCodeTab::MagicDistillationReadout,
        MajoranaSurfaceCodeTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;

        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });

        out.textures_delta.clear();
    }

    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();
}
