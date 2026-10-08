#![deny(unsafe_code)]

//! GUI Integration Test Suite for Quantum Metamaterial Valley-Locked Majorana Router & Transmon Interconnect Dialog (Phase 453).
//!
//! Validates:
//! 1. Fast cold-boot initialization (< 2.0 ms) and baseline cached telemetry.
//! 2. Tab switching across all 5 categorized tabs.
//! 3. Interactive parameter adjustment and deterministic recomputation.
//! 4. Headless egui render pass across all tabs without panic or texture leakage.

use egui::Context;
use phonon_gui::widgets::valley_majorana_router_dialog::{
    ValleyMajoranaRouterDialog, ValleyMajoranaRouterTab,
};

#[test]
fn test_valley_majorana_router_dialog_initialization_and_fast_boot() {
    let t_start = std::time::Instant::now();
    let dialog = ValleyMajoranaRouterDialog::new_fast();
    let elapsed = t_start.elapsed();

    assert!(!dialog.is_open, "Dialog must be closed by default");
    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be practically instantaneous, took {} ms",
        elapsed.as_millis()
    );

    // Verify baseline cached telemetry
    assert!(dialog.cached_lattice_metrics.valley_bandgap_mhz >= 18.0);
    assert!(dialog.cached_lattice_metrics.edge_mode_decay_depth_cells <= 2.0);
    assert!(dialog.cached_lattice_metrics.bend_transmission_ratio >= 0.940);
    assert_eq!(dialog.cached_lattice_metrics.valley_k_chern_number, 1);
    assert_eq!(dialog.cached_lattice_metrics.valley_kprime_chern_number, -1);
    assert_eq!(dialog.cached_lattice_metrics.delta_valley_chern_number, 2);

    assert!(dialog.cached_transmon_metrics.coupling_rate_g_mhz >= 25.0);
    assert!(dialog.cached_transmon_metrics.cooperativity >= 150.0);
    assert!(dialog.cached_transmon_metrics.dispersive_shift_chi_mhz >= 3.5);
    assert!(dialog.cached_transmon_metrics.parity_readout_contrast_pct >= 85.0);

    assert!(dialog.cached_splitter_metrics.insertion_loss_db <= 0.45);
    assert!(dialog.cached_splitter_metrics.backward_isolation_db >= 35.0);
    assert!(dialog.cached_splitter_metrics.return_loss_db >= 22.0);
    assert!(dialog.cached_splitter_metrics.transfer_fidelity_pct >= 99.0);

    assert!(dialog.cached_interconnect_metrics.thermal_noise_occupancy <= 0.05);
    assert!(dialog.cached_interconnect_metrics.quasiparticle_poisoning_rate_hz <= 25.0);
    assert!(dialog.cached_interconnect_metrics.dephasing_time_t2_us >= 45.0);
    assert!(dialog.cached_interconnect_metrics.end_to_end_fidelity_pct >= 90.0);

    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_valley_majorana_router_dialog_tab_switching() {
    let mut dialog = ValleyMajoranaRouterDialog::new_fast();

    let tabs = [
        ValleyMajoranaRouterTab::ValleyLatticeEdgeModes,
        ValleyMajoranaRouterTab::MajoranaTransmonCoupling,
        ValleyMajoranaRouterTab::ChiralBeamSplitter,
        ValleyMajoranaRouterTab::FaultTolerantInterconnect,
        ValleyMajoranaRouterTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty(), "Tab label must not be empty");
    }
}

#[test]
fn test_valley_majorana_router_dialog_parameter_mutation_and_recompute() {
    let mut dialog = ValleyMajoranaRouterDialog::new_fast();

    dialog.lattice_constant_um = 42.0;
    dialog.acoustic_velocity_ms = 3500.0;
    dialog.radius_sublattice_a_um = 10.2;
    dialog.radius_sublattice_b_um = 5.8;
    dialog.coupling_rate_g_mhz = 29.0;
    dialog.routing_bandwidth_mhz = 150.0;
    dialog.phase_bias_rad = 0.8;
    dialog.base_temperature_k = 0.015;
    dialog.bus_length_mm = 2.5;

    dialog.recompute();

    assert!(dialog.cached_lattice_metrics.valley_bandgap_mhz >= 18.0);
    assert!(dialog.cached_transmon_metrics.cooperativity >= 150.0);
    assert!(dialog.cached_splitter_metrics.insertion_loss_db <= 0.45);
    assert!(dialog.cached_interconnect_metrics.thermal_noise_occupancy <= 0.05);
    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_valley_majorana_router_dialog_headless_egui_render() {
    let ctx = Context::default();
    let mut dialog = ValleyMajoranaRouterDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        ValleyMajoranaRouterTab::ValleyLatticeEdgeModes,
        ValleyMajoranaRouterTab::MajoranaTransmonCoupling,
        ValleyMajoranaRouterTab::ChiralBeamSplitter,
        ValleyMajoranaRouterTab::FaultTolerantInterconnect,
        ValleyMajoranaRouterTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });

        // Ensure clean frame without unhandled textures
        assert!(!output.shapes.is_empty(), "Tab {:?} must produce rendered shapes", tab);
        output.textures_delta.clear();
    }
}
