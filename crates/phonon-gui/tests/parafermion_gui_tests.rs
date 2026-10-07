#![deny(unsafe_code)]

//! GUI Integration and Headless Render Test Suite for Phase 422:
//! Phonon Studio Quantum Acoustic Non-Abelian Parafermion Braiding
//! & Fractional Chern Number Interconnect.

use egui::Context;
use phonon_gui::widgets::parafermion_dialog::{ParafermionDialog, ParafermionTab};
use phonon_solver::parafermion_braiding::{ParafermionGateKind, ParafermionOrder};

#[test]
fn test_parafermion_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = ParafermionDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, ParafermionTab::DomainWallLattice);
    assert_eq!(dialog.cached_modes.len(), 6);
    assert_eq!(dialog.cached_spectrum.len(), 41);
    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency {:?} must be strictly under 5 ms",
        elapsed
    );
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_parafermion_dialog_tab_switching() {
    let mut dialog = ParafermionDialog::new_fast();

    let tabs = [
        ParafermionTab::DomainWallLattice,
        ParafermionTab::NonAbelianBraiding,
        ParafermionTab::UniversalGateSynthesis,
        ParafermionTab::FractionalChargeReadout,
        ParafermionTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_parafermion_dialog_recompute_and_parameter_updates() {
    let mut dialog = ParafermionDialog::new_fast();

    dialog.selected_order = ParafermionOrder::Z4;
    dialog.topological_gap_mhz = 4.5;
    dialog.domain_coupling_mhz = 1.5;
    dialog.selected_gate = ParafermionGateKind::Hadamard;
    dialog.step_duration_ns = 220.0;

    dialog.recompute();

    assert_eq!(dialog.cached_modes.len(), 6);
    assert_eq!(dialog.cached_compiled_gate.gate_kind, ParafermionGateKind::Hadamard);
    assert!(dialog.cached_compiled_gate.process_fidelity >= 0.999);
    assert_eq!(dialog.cached_spectrum.len(), 41);
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_parafermion_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = ParafermionDialog::new_fast();
    dialog.is_open = true;

    for tab in &[
        ParafermionTab::DomainWallLattice,
        ParafermionTab::NonAbelianBraiding,
        ParafermionTab::UniversalGateSynthesis,
        ParafermionTab::FractionalChargeReadout,
        ParafermionTab::AuditTelemetry,
    ] {
        dialog.active_tab = *tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
