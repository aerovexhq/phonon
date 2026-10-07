#![deny(unsafe_code)]

//! Automated GUI test suite for Phase 410: Phonon Studio Quantum Acoustic Giant Atom
//! Waveguide QED & Non-Markovian Multi-Point Entanglement Processor dialog.

use egui::Context;
use phonon_gui::widgets::giant_atom_dialog::{GiantAtomDialog, GiantAtomDialogTab};
use phonon_solver::giant_atom_qed::GiantAtomTopology;
use std::time::Instant;

#[test]
fn test_giant_atom_dialog_initialization() {
    let dialog = GiantAtomDialog::default();

    assert!(!dialog.is_open, "Dialog must start closed by default");
    assert_eq!(
        dialog.active_tab,
        GiantAtomDialogTab::ArchitectureCanvas,
        "Default tab must be ArchitectureCanvas"
    );
    assert!(!dialog.cached_dynamics.trajectory.is_empty(), "Cached dynamics trajectory must be populated");
    assert!(!dialog.cached_spectrum.points.is_empty(), "Cached scattering spectrum must be populated");
    assert!(!dialog.cached_entanglement.trajectory.is_empty(), "Cached entanglement trajectory must be populated");
    assert_eq!(dialog.cached_audit.total_pass_count, 10, "Audit must achieve 10/10 PASS");
    assert!(dialog.cached_audit.is_full_pass, "Audit must be full pass");
}

#[test]
fn test_giant_atom_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = GiantAtomDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency must be strictly sub-5ms (measured: {:.3} ms)",
        elapsed.as_secs_f64() * 1000.0
    );
    assert_eq!(dialog.cached_audit.total_pass_count, 10);
}

#[test]
fn test_giant_atom_dialog_tab_switching() {
    let mut dialog = GiantAtomDialog::default();

    let tabs = [
        GiantAtomDialogTab::ArchitectureCanvas,
        GiantAtomDialogTab::NonMarkovianDynamics,
        GiantAtomDialogTab::WaveguideSMatrix,
        GiantAtomDialogTab::DecoherenceFreeEntanglement,
        GiantAtomDialogTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_giant_atom_dialog_preset_switching_and_recompute() {
    let mut dialog = GiantAtomDialog::default();

    // Switch to Braided
    dialog.topology = GiantAtomTopology::Braided;
    dialog.recompute();

    assert_eq!(dialog.topology, GiantAtomTopology::Braided);
    assert!(dialog.cached_coupling_matrix.exchange_coupling_mhz >= 1.0);
    assert!(dialog.cached_entanglement.peak_fidelity >= 0.98);
    assert!(dialog.last_solve_time_us > 0.0);

    // Switch to Superradiant
    dialog.topology = GiantAtomTopology::Separate;
    dialog.atom_freq_ghz = 4.5;
    dialog.acoustic_velocity_ms = 3480.0;
    dialog.coupling_spacing_um = 3480.0 / 4.5;
    dialog.recompute();

    assert_eq!(dialog.topology, GiantAtomTopology::Separate);
    assert!(!dialog.cached_dynamics.trajectory.is_empty());
}

#[test]
fn test_giant_atom_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = GiantAtomDialog::default();
    dialog.is_open = true;

    let tabs = [
        GiantAtomDialogTab::ArchitectureCanvas,
        GiantAtomDialogTab::NonMarkovianDynamics,
        GiantAtomDialogTab::WaveguideSMatrix,
        GiantAtomDialogTab::DecoherenceFreeEntanglement,
        GiantAtomDialogTab::AuditTelemetry,
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
