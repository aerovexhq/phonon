#![deny(unsafe_code)]

//! GUI Integration test suite for Phase 439:
//! Topological Acoustic Boundary-Mode Soliton Logic Gate & Majority Voter Dialog.

use egui::Context;
use phonon_gui::widgets::topological_soliton_dialog::{
    TopologicalSolitonDialog, TopologicalSolitonTab,
};
use phonon_solver::topological_soliton_logic::SolitonGateMode;

#[test]
fn test_topological_soliton_dialog_cold_boot_and_audit() {
    let start = std::time::Instant::now();
    let dialog = TopologicalSolitonDialog::new_fast();
    let boot_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, TopologicalSolitonTab::BoundarySoliton);
    assert!(
        boot_time_ms < 5.0,
        "Cold boot latency {:.2} ms exceeds 5.0 ms threshold",
        boot_time_ms
    );

    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_topological_soliton_dialog_tab_switching() {
    let mut dialog = TopologicalSolitonDialog::new_fast();

    let tabs = [
        TopologicalSolitonTab::BoundarySoliton,
        TopologicalSolitonTab::CollisionalDynamics,
        TopologicalSolitonTab::MajorityVoter,
        TopologicalSolitonTab::TopologicalLattice,
        TopologicalSolitonTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_topological_soliton_dialog_interactive_recompute() {
    let mut dialog = TopologicalSolitonDialog::new_fast();

    // 1. Initial Majority mode with A=1, B=1, C=0 -> Output 1
    dialog.input_a = true;
    dialog.input_b = true;
    dialog.input_c = false;
    dialog.gate_mode = SolitonGateMode::Majority;
    dialog.recompute();
    assert!(dialog.cached_gate_metrics.logic_output);

    // 2. Set A=0, B=1, C=0 in Majority mode -> Output 0
    dialog.input_a = false;
    dialog.input_b = true;
    dialog.input_c = false;
    dialog.recompute();
    assert!(!dialog.cached_gate_metrics.logic_output);

    // 3. Switch to OR gate with A=0, B=1 -> Output 1
    dialog.gate_mode = SolitonGateMode::OrGate;
    dialog.recompute();
    assert!(dialog.cached_gate_metrics.logic_output);

    // 4. Switch to AND gate with A=0, B=1 -> Output 0
    dialog.gate_mode = SolitonGateMode::AndGate;
    dialog.recompute();
    assert!(!dialog.cached_gate_metrics.logic_output);
}

#[test]
fn test_topological_soliton_dialog_headless_render() {
    let mut dialog = TopologicalSolitonDialog::new_fast();
    dialog.is_open = true;

    let ctx = Context::default();

    // Test render pass for all 5 tabs
    let tabs = [
        TopologicalSolitonTab::BoundarySoliton,
        TopologicalSolitonTab::CollisionalDynamics,
        TopologicalSolitonTab::MajorityVoter,
        TopologicalSolitonTab::TopologicalLattice,
        TopologicalSolitonTab::AuditTelemetry,
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
