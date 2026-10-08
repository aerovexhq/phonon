#![deny(unsafe_code)]

//! Automated GUI test suite for Phase 435:
//! Phonon Studio Topological Higher-Order Acoustic Superconducting Circuit QED Quantum Transducer & Multi-Qubit Crossbar.

use egui::{Context, RawInput};
use phonon_gui::widgets::circuit_qed_transducer_dialog::{
    CircuitQedTab, CircuitQedTransducerDialog,
};

#[test]
fn test_circuit_qed_transducer_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = CircuitQedTransducerDialog::new_fast();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(
        elapsed_ms < 10.0,
        "Cold boot latency must be < 10 ms, took {:.2} ms",
        elapsed_ms
    );
    assert!(
        dialog.cached_audit.all_passed,
        "Default baseline state must pass all 10 physics audit criteria"
    );
    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(
        dialog.cached_corner_metrics.confinement_ratio >= 0.85,
        "Corner confinement must be >= 85%"
    );
    assert!(
        dialog.cached_qed_metrics.state_transfer_fidelity >= 0.990,
        "State transfer fidelity must be >= 99%"
    );
}

#[test]
fn test_circuit_qed_transducer_dialog_tab_switching() {
    let mut dialog = CircuitQedTransducerDialog::new_fast();

    let tabs = [
        CircuitQedTab::TopologicalCornerState,
        CircuitQedTab::CircuitQedDynamics,
        CircuitQedTab::QuantumStateTransfer,
        CircuitQedTab::MultiQubitCrossbar,
        CircuitQedTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_circuit_qed_transducer_parameter_adjustment_and_recompute() {
    let mut dialog = CircuitQedTransducerDialog::new_fast();

    dialog.gamma_mhz = 1.0;
    dialog.lambda_mhz = 15.0;
    dialog.recompute();

    assert!(
        (dialog.cached_corner_metrics.bulk_gap_mhz - 28.0).abs() < 1e-3,
        "Bulk gap must be 2 * |15 - 1| = 28 MHz, got {:.2} MHz",
        dialog.cached_corner_metrics.bulk_gap_mhz
    );
    assert!(
        dialog.cached_corner_metrics.confinement_ratio >= 0.85,
        "Confinement must remain >= 85%"
    );
    assert!(
        dialog.cached_audit.all_passed,
        "Audit must still pass with enhanced topological gap"
    );
}

#[test]
fn test_circuit_qed_transducer_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = CircuitQedTransducerDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        CircuitQedTab::TopologicalCornerState,
        CircuitQedTab::CircuitQedDynamics,
        CircuitQedTab::QuantumStateTransfer,
        CircuitQedTab::MultiQubitCrossbar,
        CircuitQedTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                dialog.render_contents(ui);
            });
        });
        out.textures_delta.clear();
    }
}
