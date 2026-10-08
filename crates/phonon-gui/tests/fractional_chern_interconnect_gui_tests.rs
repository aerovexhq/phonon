#![deny(unsafe_code)]

//! GUI Integration test suite for Phase 440:
//! Quantum Metamaterial Fractional Chern Insulator & Non-Abelian Parafermion Braiding Interconnect Dialog.

use egui::Context;
use phonon_gui::widgets::fractional_chern_interconnect_dialog::{
    FractionalChernInterconnectDialog, FractionalChernTab,
};
use phonon_solver::fractional_chern_interconnect::QuditGateKind;

#[test]
fn test_fractional_chern_dialog_cold_boot_and_audit() {
    let start = std::time::Instant::now();
    let dialog = FractionalChernInterconnectDialog::new_fast();
    let boot_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, FractionalChernTab::FractionalChernLattice);
    assert!(
        boot_time_ms < 5.0,
        "Cold boot latency {:.2} ms exceeds 5.0 ms threshold",
        boot_time_ms
    );

    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_fractional_chern_dialog_tab_switching() {
    let mut dialog = FractionalChernInterconnectDialog::new_fast();

    let tabs = [
        FractionalChernTab::FractionalChernLattice,
        FractionalChernTab::ParafermionDomainWalls,
        FractionalChernTab::NonAbelianBraiding,
        FractionalChernTab::QuditGateSynthesis,
        FractionalChernTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_fractional_chern_dialog_interactive_recompute() {
    let mut dialog = FractionalChernInterconnectDialog::new_fast();

    // 1. Initial Fractional Chern metrics check
    assert_eq!(dialog.cached_lattice_metrics.quasiparticle_charge_e_star, 1.0 / 3.0);
    assert!(dialog.cached_braiding_metrics.compiled_gate_fidelity >= 0.999);

    // 2. Adjust lattice parameters and test recompute
    dialog.intracell_hopping_mhz = 3.0;
    dialog.intercell_hopping_mhz = 10.0;
    dialog.has_edge_obstacle = true;
    dialog.recompute();
    assert!(dialog.cached_lattice_metrics.defect_transmission_ratio >= 0.95);

    // 3. Switch target gate to CSUM entangler
    dialog.target_gate = QuditGateKind::CSumTwoQutrit;
    dialog.recompute();
    assert!(dialog.cached_braiding_metrics.compiled_gate_fidelity >= 0.999);

    // 4. Switch target gate to Phase S_3
    dialog.target_gate = QuditGateKind::PhaseS3;
    dialog.recompute();
    assert!(dialog.cached_braiding_metrics.compiled_gate_fidelity >= 0.999);
}

#[test]
fn test_fractional_chern_dialog_headless_render() {
    let mut dialog = FractionalChernInterconnectDialog::new_fast();
    dialog.is_open = true;

    let ctx = Context::default();

    // Test render pass for all 5 tabs
    let tabs = [
        FractionalChernTab::FractionalChernLattice,
        FractionalChernTab::ParafermionDomainWalls,
        FractionalChernTab::NonAbelianBraiding,
        FractionalChernTab::QuditGateSynthesis,
        FractionalChernTab::AuditTelemetry,
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
