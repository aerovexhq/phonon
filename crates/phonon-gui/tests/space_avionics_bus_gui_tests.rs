#![deny(unsafe_code)]

//! GUI test suite for Phase 379: SpaceWire/SpaceFibre & Avionics AFDX Bus Contention Co-Simulator Dialog.

use egui::Context;
use phonon_gui::widgets::space_avionics_bus_dialog::{SpaceAvionicsBusDialog, SpaceAvionicsBusTab};
use std::time::Instant;

#[test]
fn test_space_avionics_bus_dialog_initialization_and_cold_boot() {
    let start = Instant::now();
    let dialog = SpaceAvionicsBusDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "SpaceAvionicsBusDialog::new_fast() must initialize in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, SpaceAvionicsBusTab::NetworkTopology);
    assert_eq!(dialog.spw_bit_rate_mbps, 200.0);
    assert_eq!(dialog.spfi_lane_count, 2);
    assert_eq!(dialog.spfi_scheduling_choice, 0); // WRR
    assert!(!dialog.cached_spfi_burst_curve.is_empty());
    assert!(!dialog.cached_latency_sweep.is_empty());
    assert!(!dialog.cached_noc_temps.is_empty());
}

#[test]
fn test_space_avionics_bus_dialog_recompute_and_parameter_updates() {
    let mut dialog = SpaceAvionicsBusDialog::new();

    // Reconfigure to SAR radar streaming payload
    dialog.spw_bit_rate_mbps = 400.0;
    dialog.spfi_lane_count = 4;
    dialog.spfi_lane_rate_gbps = 6.25;
    dialog.spfi_scheduling_choice = 1; // Strict priority
    dialog.burst_rate_gbps = 8.0;
    dialog.afdx_bag_choice = 1; // 2 ms
    dialog.afdx_frame_size_bytes = 1518;
    dialog.afdx_switch_hops = 4;
    dialog.noc_policy_choice = 1; // Thermal deflection
    dialog.noc_thermal_threshold_c = 80.0;

    dialog.recompute_sim();

    let report = dialog.sim.report();
    assert_eq!(report.spacewire_rate_mbps, 400.0);
    // 4 lanes * 6.25 Gbps * 0.80 = 20.0 Gbps
    assert!((report.spacefibre_aggregate_gbps - 20.0).abs() < 1e-2);
    assert!(report.afdx_allocated_bw_mbps > 0.0);
    assert!(report.afdx_end_to_end_latency_bound_us > 0.0);

    // Verify refreshed plot cache lengths
    assert_eq!(dialog.cached_spfi_burst_curve.len(), 50);
    assert_eq!(dialog.cached_latency_sweep.len(), 6);
    assert_eq!(dialog.cached_noc_temps.len(), 16);
}

#[test]
fn test_space_avionics_bus_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dialog = SpaceAvionicsBusDialog::new();
    dialog.is_open = true;

    let tabs = [
        SpaceAvionicsBusTab::NetworkTopology,
        SpaceAvionicsBusTab::SpaceFibreQoS,
        SpaceAvionicsBusTab::Arinc664AfdxLinks,
        SpaceAvionicsBusTab::NoCThermalMesh,
        SpaceAvionicsBusTab::JitterContentionScope,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
    }

    // Also test alias ui(&ctx)
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();
}
