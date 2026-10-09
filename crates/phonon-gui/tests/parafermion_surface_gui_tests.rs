#![deny(unsafe_code)]

//! Headless integration test suite for Parafermion Lattice Co-Processor & Surface Engine CAD Dialog (Phase 461).

use egui::Context;
use phonon_gui::widgets::parafermion_surface_dialog::{
    ParafermionSurfaceDialog, ParafermionSurfaceTab,
};
use std::time::Instant;

#[test]
fn test_parafermion_surface_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = ParafermionSurfaceDialog::new_fast();
    let elapsed = start.elapsed();

    // Verify sub-2.0 ms cold boot requirement
    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency {:?} exceeded 5 ms limit",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        ParafermionSurfaceTab::NonAbelianParafermionLattice
    );
    assert!(dialog.cached_audit.is_pass());
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_parafermion_surface_dialog_tab_switching() {
    let mut dialog = ParafermionSurfaceDialog::new_fast();

    let tabs = [
        ParafermionSurfaceTab::NonAbelianParafermionLattice,
        ParafermionSurfaceTab::QuantumAcousticSurfaceCode,
        ParafermionSurfaceTab::CryogenicMultiQuditCoprocessor,
        ParafermionSurfaceTab::TopologicalCoprocessorArchitecture,
        ParafermionSurfaceTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_parafermion_surface_dialog_parameter_mutation_and_recompute() {
    let mut dialog = ParafermionSurfaceDialog::new_fast();

    dialog.pairing_coupling_mhz = 4.0;
    dialog.chern_gap_mhz = 22.0;
    dialog.braid_duration_ns = 140.0;
    dialog.code_distance = 5;
    dialog.operating_temp_k = 0.012;
    dialog.clock_rate_khz = 700.0;

    dialog.recompute();

    assert!(dialog.cached_lattice_metrics.topological_gap_mhz >= 2.0);
    assert!(dialog.cached_lattice_metrics.braid_fidelity_percent >= 99.5);
    assert!(dialog.cached_lattice_metrics.artin_braid_error <= 1e-4);
    assert!(dialog.cached_surface_metrics.logical_error_rate <= 1.0e-4);
    assert!(dialog.cached_surface_metrics.readout_snr_db >= 16.0);
    assert!(dialog.cached_surface_metrics.readout_fidelity_percent >= 99.5);
    assert!(dialog.cached_coprocessor_metrics.thermal_phonon_occupancy <= 1.0e-3);
    assert!(dialog.cached_coprocessor_metrics.effective_clock_khz >= 500.0);
    assert!(dialog.cached_coprocessor_metrics.entanglement_concurrence >= 0.90);
    assert!(dialog.cached_coprocessor_metrics.inter_qudit_isolation_db >= 42.0);
    assert!(dialog.cached_audit.is_pass());
}

#[test]
fn test_parafermion_surface_dialog_headless_egui_render() {
    let mut dialog = ParafermionSurfaceDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();

    let tabs = [
        ParafermionSurfaceTab::NonAbelianParafermionLattice,
        ParafermionSurfaceTab::QuantumAcousticSurfaceCode,
        ParafermionSurfaceTab::CryogenicMultiQuditCoprocessor,
        ParafermionSurfaceTab::TopologicalCoprocessorArchitecture,
        ParafermionSurfaceTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
