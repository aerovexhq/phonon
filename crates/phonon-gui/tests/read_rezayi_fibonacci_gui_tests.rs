#![deny(unsafe_code)]

//! GUI Integration test suite for Phase 447:
//! Fractional Quantum Hall Non-Abelian Read-Rezayi Fibonacci Anyon Acoustic Interferometer & Universal Topological Quantum Bus Dialog.

use egui::Context;
use phonon_gui::widgets::read_rezayi_fibonacci_dialog::{
    ReadRezayiDialogTab, ReadRezayiFibonacciDialog,
};
use phonon_solver::read_rezayi_fibonacci::{
    EnclosedTopologicalCharge, FibonacciTargetGate, ReadRezayiFilling,
};

#[test]
fn test_read_rezayi_dialog_cold_boot_and_audit() {
    let start = std::time::Instant::now();
    let dialog = ReadRezayiFibonacciDialog::new_fast();
    let boot_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, ReadRezayiDialogTab::ReadRezayiTopology);
    assert!(
        boot_time_ms < 5.0,
        "Cold boot latency {:.2} ms exceeds 5.0 ms threshold",
        boot_time_ms
    );

    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_read_rezayi_dialog_tab_switching() {
    let mut dialog = ReadRezayiFibonacciDialog::new_fast();

    let tabs = [
        ReadRezayiDialogTab::ReadRezayiTopology,
        ReadRezayiDialogTab::FibonacciBraidingLattice,
        ReadRezayiDialogTab::SawAcousticInterferometer,
        ReadRezayiDialogTab::QuantumAcousticBus,
        ReadRezayiDialogTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_read_rezayi_dialog_interactive_recompute() {
    let mut dialog = ReadRezayiFibonacciDialog::new_fast();

    assert!(dialog.cached_state_metrics.fibonacci_quantum_dimension > 1.61);
    assert!(dialog.cached_braiding_metrics.compiled_gate_fidelity >= 0.999);
    assert!(dialog.cached_interferometer_metrics.vacuum_visibility_percent >= 80.0);
    assert!(dialog.cached_bus_metrics.state_transfer_fidelity >= 0.990);

    // Switch parameters and trigger recompute
    dialog.filling_factor = ReadRezayiFilling::Nu2Plus2Over3;
    dialog.target_gate = FibonacciTargetGate::PiOver8T;
    dialog.enclosed_charge = EnclosedTopologicalCharge::Vacuum1;
    dialog.bus_length_um = 180.0;
    dialog.recompute();

    assert_eq!(dialog.cached_state_metrics.filling_factor_val, 8.0 / 3.0);
    assert_eq!(dialog.cached_braiding_metrics.braid_word_length, 12);
    assert_eq!(dialog.cached_interferometer_metrics.visibility_suppression_ratio, 1.0);
    assert!(dialog.last_solve_time_us < 5000.0);
}

#[test]
fn test_read_rezayi_dialog_egui_headless_render() {
    let ctx = Context::default();
    let mut dialog = ReadRezayiFibonacciDialog::new_fast();
    dialog.is_open = true;

    for tab in [
        ReadRezayiDialogTab::ReadRezayiTopology,
        ReadRezayiDialogTab::FibonacciBraidingLattice,
        ReadRezayiDialogTab::SawAcousticInterferometer,
        ReadRezayiDialogTab::QuantumAcousticBus,
        ReadRezayiDialogTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();

        assert_eq!(dialog.active_tab, tab);
    }

    assert!(dialog.is_open);
}
