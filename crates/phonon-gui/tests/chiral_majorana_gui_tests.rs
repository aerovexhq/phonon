#![deny(unsafe_code)]

use egui::Context;
use phonon_gui::widgets::chiral_majorana_dialog::{ChiralMajoranaDialog, ChiralMajoranaTab};
use phonon_solver::chiral_majorana_braiding::ChiralCliffordGateKind;

#[test]
fn test_chiral_majorana_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = ChiralMajoranaDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, ChiralMajoranaTab::BraidingLattice);
    assert_eq!(dialog.qubit_count, 2);
    assert_eq!(dialog.code_distance, 3);
    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency {:?} must be strictly under 5 ms",
        elapsed
    );
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_chiral_majorana_dialog_tab_switching() {
    let mut dialog = ChiralMajoranaDialog::new_fast();

    let tabs = [
        ChiralMajoranaTab::BraidingLattice,
        ChiralMajoranaTab::SurfaceCodeDecoder,
        ChiralMajoranaTab::GateSynthesis,
        ChiralMajoranaTab::ParityReadout,
        ChiralMajoranaTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_chiral_majorana_dialog_recompute_and_parameter_updates() {
    let mut dialog = ChiralMajoranaDialog::new_fast();

    dialog.qubit_count = 3;
    dialog.topological_gap_mhz = 5.0;
    dialog.code_distance = 5;
    dialog.physical_error_rate = 0.002;
    dialog.selected_gate = ChiralCliffordGateKind::Cnot;
    dialog.dispersive_shift_mhz = 5.5;

    dialog.recompute();

    assert_eq!(dialog.processor.network.modes.len(), 12); // 3 qubits * 4 MZMs
    assert_eq!(dialog.cached_compiled_gate.gate_kind, ChiralCliffordGateKind::Cnot);
    assert!(dialog.cached_compiled_gate.process_fidelity >= 0.999);
    assert!(dialog.cached_decoding_result.logical_error_rate < 0.002);
    assert_eq!(dialog.cached_spectrum.len(), 61);
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
}

#[test]
fn test_chiral_majorana_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = ChiralMajoranaDialog::new_fast();
    dialog.is_open = true;

    for tab in &[
        ChiralMajoranaTab::BraidingLattice,
        ChiralMajoranaTab::SurfaceCodeDecoder,
        ChiralMajoranaTab::GateSynthesis,
        ChiralMajoranaTab::ParityReadout,
        ChiralMajoranaTab::AuditTelemetry,
    ] {
        dialog.active_tab = *tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
