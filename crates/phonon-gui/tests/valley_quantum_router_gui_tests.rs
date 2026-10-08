#![deny(unsafe_code)]

//! GUI Integration test suite for Phase 444:
//! Topological Acoustic Boundary-Mode Valley-Hall Quantum Router & Entanglement Concentrator Dialog.

use egui::Context;
use phonon_gui::widgets::valley_quantum_router_dialog::{
    ValleyQuantumRouterDialog, ValleyRouterTab,
};
use phonon_solver::valley_quantum_router::RouterChannelTarget;

#[test]
fn test_valley_quantum_router_dialog_cold_boot_and_audit() {
    let start = std::time::Instant::now();
    let dialog = ValleyQuantumRouterDialog::new_fast();
    let boot_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        ValleyRouterTab::ValleyHallLatticeDispersion
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
fn test_valley_quantum_router_dialog_tab_switching() {
    let mut dialog = ValleyQuantumRouterDialog::new_fast();

    let tabs = [
        ValleyRouterTab::ValleyHallLatticeDispersion,
        ValleyRouterTab::ValleyPolarizationRouter,
        ValleyRouterTab::EntanglementConcentration,
        ValleyRouterTab::CryogenicDispersiveReadout,
        ValleyRouterTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_valley_quantum_router_dialog_interactive_recompute() {
    let mut dialog = ValleyQuantumRouterDialog::new_fast();

    // 1. Initial cached metrics check
    assert!(dialog.cached_lattice_metrics.valley_bulk_gap_mhz >= 3.0);
    assert_eq!(dialog.cached_lattice_metrics.valley_chern_difference, 1.0);
    assert!(dialog.cached_lattice_metrics.edge_group_velocity_ms >= 1200.0);
    assert!(dialog.cached_router_metrics.target_transmission_percent >= 90.0);
    assert!(dialog.cached_router_metrics.crosstalk_isolation_db >= 35.0);
    assert!(dialog.cached_concentrator_metrics.concentrated_concurrence >= 0.96);
    assert!(dialog.cached_concentrator_metrics.bell_state_fidelity >= 0.995);

    // 2. Adjust lattice parameters and recompute
    dialog.radius_a_um = 9.2;
    dialog.radius_b_um = 5.0;
    dialog.corner_bend_angle_deg = 120.0;
    dialog.has_boundary_defect = true;
    dialog.recompute();

    assert!(dialog.cached_lattice_metrics.valley_bulk_gap_mhz >= 3.0);
    assert!(dialog.cached_lattice_metrics.corner_transmission_ratio >= 0.95);
    assert!(dialog.cached_lattice_metrics.defect_immunity_ratio >= 0.95);

    // 3. Adjust router target channel and recompute
    dialog.target_channel = RouterChannelTarget::Port3Deflected120Kp;
    dialog.gate_voltage_v = 1.5;
    dialog.gate_rise_time_ns = 1.0;
    dialog.recompute();

    assert!(dialog.cached_router_metrics.target_transmission_percent >= 90.0);
    assert!(dialog.cached_router_metrics.switching_latency_ns <= 3.0);
    assert!(dialog.cached_router_metrics.crosstalk_isolation_db >= 35.0);

    // 4. Adjust concentrator parameters and recompute
    dialog.initial_state_alpha = 0.92;
    dialog.pump_power_mw = 25.0;
    dialog.interaction_length_um = 200.0;
    dialog.recompute();

    assert!(dialog.cached_concentrator_metrics.concentrated_concurrence >= 0.96);
    assert!(dialog.cached_concentrator_metrics.bell_state_fidelity >= 0.995);
    assert!(dialog.cached_concentrator_metrics.dispersive_readout_snr_db >= 18.0);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_valley_quantum_router_dialog_headless_render() {
    let mut dialog = ValleyQuantumRouterDialog::new_fast();
    dialog.is_open = true;

    let ctx = Context::default();

    // Test render pass for all 5 tabs
    let tabs = [
        ValleyRouterTab::ValleyHallLatticeDispersion,
        ValleyRouterTab::ValleyPolarizationRouter,
        ValleyRouterTab::EntanglementConcentration,
        ValleyRouterTab::CryogenicDispersiveReadout,
        ValleyRouterTab::AuditTelemetry,
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
