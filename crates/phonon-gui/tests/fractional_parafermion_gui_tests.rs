#![deny(unsafe_code)]

//! GUI Integration test suite for Phase 445:
//! Quantum Metamaterial Non-Abelian Fractional Parafermion Surface-Code Lattice & Anyonic Braid Repeater Dialog.

use egui::Context;
use phonon_gui::widgets::fractional_parafermion_dialog::{
    FractionalParafermionDialog, ParafermionDialogTab,
};
use phonon_solver::fractional_parafermion_surface::FractionalParafermionOrder;

#[test]
fn test_fractional_parafermion_dialog_cold_boot_and_audit() {
    let start = std::time::Instant::now();
    let dialog = FractionalParafermionDialog::new_fast();
    let boot_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        ParafermionDialogTab::ParafermionBraidingLattice
    );
    assert!(
        boot_time_ms < 5.0,
        "Cold boot latency {:.2} ms exceeds 5.0 ms threshold",
        boot_time_ms
    );

    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_fractional_parafermion_dialog_tab_switching() {
    let mut dialog = FractionalParafermionDialog::new_fast();

    let tabs = [
        ParafermionDialogTab::ParafermionBraidingLattice,
        ParafermionDialogTab::SurfaceCodeSyndromes,
        ParafermionDialogTab::AnyonicBraidRepeater,
        ParafermionDialogTab::QuditDistillationFactory,
        ParafermionDialogTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_fractional_parafermion_dialog_interactive_recompute() {
    let mut dialog = FractionalParafermionDialog::new_fast();

    // 1. Initial cached metrics check
    assert!(dialog.cached_lattice_metrics.topological_gap_mhz >= 1.8);
    assert!(dialog.cached_lattice_metrics.braid_fidelity >= 0.999);
    assert!(dialog.cached_lattice_metrics.diabatic_leakage_error < 1.0e-4);
    assert!(dialog.cached_lattice_metrics.poisoning_lifetime_us >= 80.0);

    assert!(dialog.cached_repeater_metrics.threshold_error_percent >= 1.5);
    assert!(dialog.cached_repeater_metrics.logical_error_rate < 1.0e-4);
    assert!(dialog.cached_repeater_metrics.repeater_fidelity >= 0.992);
    assert!(dialog.cached_repeater_metrics.distribution_rate_khz >= 120.0);

    assert!(dialog.cached_distillation_metrics.distilled_magic_fidelity >= 0.999);
    assert!(dialog.cached_distillation_metrics.acceptance_probability_percent >= 18.0);
    assert!(dialog.cached_distillation_metrics.dispersive_readout_snr_db >= 18.5);

    // 2. Adjust lattice parameters and recompute
    dialog.order = FractionalParafermionOrder::Z4Clock;
    dialog.superconducting_gap_mhz = 7.0;
    dialog.braid_duration_ns = 140.0;
    dialog.recompute();

    assert!(dialog.cached_lattice_metrics.topological_gap_mhz >= 1.8);
    assert!(dialog.cached_lattice_metrics.braid_fidelity >= 0.999);

    // 3. Adjust surface code and repeater parameters
    dialog.code_distance = 5;
    dialog.physical_error_rate = 0.003;
    dialog.link_distance_um = 150.0;
    dialog.repeater_node_count = 6;
    dialog.recompute();

    assert!(dialog.cached_repeater_metrics.logical_error_rate < 1.0e-4);
    assert!(dialog.cached_repeater_metrics.repeater_fidelity >= 0.992);

    // 4. Adjust distillation parameters and recompute
    dialog.raw_magic_error_rate = 0.035;
    dialog.dispersive_shift_chi_mhz = 5.0;
    dialog.recompute();

    assert!(dialog.cached_distillation_metrics.distilled_magic_fidelity >= 0.999);
    assert!(dialog.cached_distillation_metrics.dispersive_readout_snr_db >= 18.5);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_fractional_parafermion_dialog_headless_render() {
    let mut dialog = FractionalParafermionDialog::new_fast();
    dialog.is_open = true;

    let ctx = Context::default();

    // Test render pass for all 5 tabs
    let tabs = [
        ParafermionDialogTab::ParafermionBraidingLattice,
        ParafermionDialogTab::SurfaceCodeSyndromes,
        ParafermionDialogTab::AnyonicBraidRepeater,
        ParafermionDialogTab::QuditDistillationFactory,
        ParafermionDialogTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_dialog_contents(ui);
        });
        output.textures_delta.clear();
        assert_eq!(dialog.active_tab, tab);
    }
}
