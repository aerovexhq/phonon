#![deny(unsafe_code)]

//! Automated GUI test suite for Phase 436:
//! Phonon Studio Non-Hermitian Exceptional Surface Chiral Phonon Diode & Unidirectional Quantum Repeater.

use egui::{Context, RawInput};
use phonon_gui::widgets::exceptional_surface_diode_dialog::{
    ExceptionalSurfaceDiodeDialog, ExceptionalSurfaceDiodeTab,
};

#[test]
fn test_exceptional_surface_diode_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = ExceptionalSurfaceDiodeDialog::new_fast();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(
        elapsed_ms < 15.0,
        "Cold boot latency must be fast (< 15 ms), took {:.2} ms",
        elapsed_ms
    );
    assert!(
        dialog.cached_audit.all_passed,
        "Default baseline state must pass all 10 physics audit criteria"
    );
    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(
        dialog.cached_diode_metrics.peak_insertion_loss_db <= 0.50,
        "Peak insertion loss must be <= 0.50 dB"
    );
    assert!(
        dialog.cached_diode_metrics.peak_isolation_db >= 35.0,
        "Peak backward isolation must be >= 35.0 dB"
    );
    assert!(
        dialog.cached_repeater_metrics.bell_pair_fidelity >= 0.980,
        "Bell pair fidelity must be >= 98%"
    );
}

#[test]
fn test_exceptional_surface_diode_dialog_tab_switching() {
    let mut dialog = ExceptionalSurfaceDiodeDialog::new_fast();

    let tabs = [
        ExceptionalSurfaceDiodeTab::ExceptionalSurfaceTopology,
        ExceptionalSurfaceDiodeTab::ChiralPhononDiode,
        ExceptionalSurfaceDiodeTab::NonReciprocalSpectrum,
        ExceptionalSurfaceDiodeTab::QuantumRepeaterNode,
        ExceptionalSurfaceDiodeTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_exceptional_surface_diode_parameter_adjustment_and_recompute() {
    let mut dialog = ExceptionalSurfaceDiodeDialog::new_fast();

    dialog.center_freq_ghz = 5.2;
    dialog.target_isolation_db = 42.0;
    dialog.target_insertion_loss_db = 0.28;
    dialog.coherence_time_t2_us = 75.0;
    dialog.recompute();

    assert_eq!(dialog.cached_diode_metrics.center_freq_ghz, 5.2);
    assert_eq!(dialog.cached_diode_metrics.peak_isolation_db, 42.0);
    assert!(
        dialog.cached_diode_metrics.peak_insertion_loss_db <= 0.30,
        "Peak insertion loss should be updated"
    );
    assert!(
        dialog.cached_repeater_metrics.quantum_memory_t2_us >= 50.0,
        "Coherence time must remain high"
    );
    assert!(
        dialog.cached_audit.all_passed,
        "Audit must still pass with custom parameters"
    );
}

#[test]
fn test_exceptional_surface_diode_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = ExceptionalSurfaceDiodeDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        ExceptionalSurfaceDiodeTab::ExceptionalSurfaceTopology,
        ExceptionalSurfaceDiodeTab::ChiralPhononDiode,
        ExceptionalSurfaceDiodeTab::NonReciprocalSpectrum,
        ExceptionalSurfaceDiodeTab::QuantumRepeaterNode,
        ExceptionalSurfaceDiodeTab::AuditTelemetry,
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
