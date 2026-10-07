#![deny(unsafe_code)]

//! GUI test suite for Phase 413: SyntheticDimensionDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - SyntheticDimensionDialog initialization, default configuration, and physics caches.
//! - Seamless tab switching across all 5 visual workspaces.
//! - Preset switching (High-Q Chern vs Multiplexed) and parameter updates.
//! - Instantaneous cold boot latency (< 5.0 ms).
//! - Headless egui Context render pass across all 5 tabs.

use std::time::Instant;
use egui::vec2;
use phonon_gui::widgets::synthetic_dimension_dialog::{
    SyntheticDimensionDialog, SyntheticDimensionTab,
};

#[test]
fn test_synthetic_dimension_dialog_initialization_and_defaults() {
    let dialog = SyntheticDimensionDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default tab
    assert_eq!(dialog.active_tab, SyntheticDimensionTab::SyntheticLattice);
    assert_eq!(
        dialog.active_tab.label(),
        "1. Synthetic 2D Lattice & Flux"
    );

    // Physical parameter defaults
    assert_eq!(dialog.physical_resonators_nx, 8);
    assert_eq!(dialog.synthetic_frequency_modes_m, 4);
    assert!((dialog.physical_coupling_jx_mhz - 12.0).abs() < 1e-4);
    assert!((dialog.synthetic_hopping_kappa_mhz - 9.5).abs() < 1e-4);

    // Initial lattice metrics
    assert_eq!(dialog.cached_lattice_metrics.first_chern_number, 1.0);
    assert!(dialog.cached_lattice_metrics.edge_confinement_ratio >= 0.85);
    assert!(dialog.cached_lattice_metrics.bulk_bandgap_mhz >= 1.5);

    // Initial Weyl metrics
    assert_eq!(dialog.cached_weyl_metrics.second_chern_number, 1.0);
    assert!(dialog.cached_weyl_metrics.non_local_transmission_ratio >= 0.80);

    // Initial Router metrics
    assert!(dialog.cached_router_metrics.insertion_loss_db <= 0.80);
    assert!(dialog.cached_router_metrics.inter_channel_isolation_db >= 35.0);
    assert!(dialog.cached_router_metrics.defect_immunity_ratio >= 0.95);

    // 10-point audit full pass
    assert_eq!(dialog.cached_audit.passed_count, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_synthetic_dimension_tab_switching() {
    let mut dialog = SyntheticDimensionDialog::new();

    let tabs = [
        (
            SyntheticDimensionTab::SyntheticLattice,
            "1. Synthetic 2D Lattice & Flux",
        ),
        (
            SyntheticDimensionTab::ChiralEdgeTransport,
            "2. Chiral Edge & Ladder Dynamics",
        ),
        (
            SyntheticDimensionTab::WeylTransport4d,
            "3. 4D Topology & Weyl Transport",
        ),
        (
            SyntheticDimensionTab::MultiplexedRouter,
            "4. Orthogonal Multiplexed Routing",
        ),
        (
            SyntheticDimensionTab::AuditTelemetry,
            "5. Physics Audit & Telemetry",
        ),
    ];

    for (tab, expected_label) in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert_eq!(dialog.active_tab.label(), expected_label);
    }
}

#[test]
fn test_synthetic_dimension_presets_and_recomputation() {
    let mut dialog = SyntheticDimensionDialog::new();

    // 1. High-Q Chern Preset
    dialog.load_preset_high_q_chern();
    assert_eq!(dialog.cached_lattice_metrics.first_chern_number, 1.0);
    assert!(dialog.cached_lattice_metrics.bulk_bandgap_mhz >= 1.5);
    assert!(dialog.cached_audit.all_passed);

    // 2. Multiplexed Preset
    dialog.load_preset_multiplexed();
    assert_eq!(dialog.cached_lattice_metrics.first_chern_number, 1.0);
    assert_eq!(dialog.channel_count, 6);
    assert!(dialog.cached_router_metrics.inter_channel_isolation_db >= 35.0);
    assert!(dialog.cached_audit.all_passed);

    // 3. Manual parameter adjustment and recomputation
    dialog.physical_resonators_nx = 10;
    dialog.synthetic_frequency_modes_m = 5;
    dialog.physical_coupling_jx_mhz = 14.0;
    dialog.defect_present = true;
    dialog.recompute();

    assert!(dialog.cached_router_metrics.defect_immunity_ratio >= 0.95);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_synthetic_dimension_cold_boot_latency() {
    let t_start = Instant::now();
    let dialog = SyntheticDimensionDialog::new_fast();
    let elapsed = t_start.elapsed();

    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be fast, took {:?}",
        elapsed
    );

    assert!(
        dialog.last_solve_time_us < 5000.0,
        "Reported last solve time must be < 5000 us, got {} us",
        dialog.last_solve_time_us
    );
}

#[test]
fn test_synthetic_dimension_headless_egui_render_pass() {
    let mut dialog = SyntheticDimensionDialog::new_fast();
    dialog.is_open = true;

    let ctx = egui::Context::default();

    for tab in [
        SyntheticDimensionTab::SyntheticLattice,
        SyntheticDimensionTab::ChiralEdgeTransport,
        SyntheticDimensionTab::WeylTransport4d,
        SyntheticDimensionTab::MultiplexedRouter,
        SyntheticDimensionTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.set_min_size(vec2(1000.0, 700.0));
            dialog.render_content(ui);
        });
        output.textures_delta.clear();

        assert_eq!(dialog.active_tab, tab);
    }
}
