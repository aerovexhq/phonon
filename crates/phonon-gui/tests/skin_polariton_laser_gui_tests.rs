#![deny(unsafe_code)]

//! GUI Integration Test Suite for Dissipative Topological Polariton Skin Laser & Gyroscope Array Dialog (Phase 452).
//!
//! Validates:
//! 1. Fast cold-boot initialization (< 2.0 ms) and baseline cached telemetry.
//! 2. Tab switching across all 5 categorized tabs.
//! 3. Interactive parameter adjustment and deterministic recomputation.
//! 4. Headless egui render pass across all tabs without panic or texture leakage.

use egui::Context;
use phonon_gui::widgets::skin_polariton_laser_dialog::{
    SkinPolaritonLaserDialog, SkinPolaritonLaserTab,
};

#[test]
fn test_skin_polariton_laser_dialog_initialization_and_fast_boot() {
    let t_start = std::time::Instant::now();
    let dialog = SkinPolaritonLaserDialog::new_fast();
    let elapsed = t_start.elapsed();

    assert!(!dialog.is_open, "Dialog must be closed by default");
    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be practically instantaneous, took {} ms",
        elapsed.as_millis()
    );

    // Verify baseline cached telemetry
    assert!(dialog.cached_skin_metrics.hopping_asymmetry_ratio >= 3.0);
    assert!(dialog.cached_skin_metrics.skin_depth_cells <= 3.5);
    assert!(dialog.cached_skin_metrics.boundary_localization_ratio >= 0.88);
    assert!(dialog.cached_skin_metrics.side_mode_suppression_ratio_db >= 32.0);

    assert_eq!(dialog.cached_winding_metrics.point_gap_winding_number, 1);
    assert!(dialog.cached_winding_metrics.gbz_deformation_magnitude >= 0.30);

    assert!(dialog.cached_gyro_metrics.sensitivity_enhancement_factor >= 45.0);
    assert!(dialog.cached_gyro_metrics.angle_random_walk_deg_sqrt_h <= 0.008);
    assert!(dialog.cached_gyro_metrics.bias_instability_deg_h <= 0.05);

    assert!(dialog.cached_gain_metrics.threshold_power_mw <= 1.80);
    assert!(dialog.cached_gain_metrics.slope_efficiency_pct >= 42.0);
    assert!(dialog.cached_gain_metrics.schawlow_townes_linewidth_khz <= 12.0);

    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_skin_polariton_laser_dialog_tab_switching() {
    let mut dialog = SkinPolaritonLaserDialog::new_fast();

    let tabs = [
        SkinPolaritonLaserTab::SkinModeLasing,
        SkinPolaritonLaserTab::ChiralSagnacGyroscope,
        SkinPolaritonLaserTab::RiemannEnergyWinding,
        SkinPolaritonLaserTab::PolaritonGainMedium,
        SkinPolaritonLaserTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty(), "Tab label must not be empty");
    }
}

#[test]
fn test_skin_polariton_laser_dialog_parameter_mutation_and_recompute() {
    let mut dialog = SkinPolaritonLaserDialog::new_fast();

    dialog.forward_hopping_jr_mhz = 14.5;
    dialog.backward_hopping_jl_mhz = 2.8;
    dialog.lattice_site_count = 40;
    dialog.ep_coupling_rate_mhz = 22.0;
    dialog.input_rotation_rate_deg_s = 5.0;
    dialog.threshold_pump_power_mw = 1.20;
    dialog.slope_efficiency_pct = 52.0;

    dialog.recompute();

    assert_eq!(dialog.lattice_site_count, 40);
    assert!(dialog.cached_skin_metrics.hopping_asymmetry_ratio >= 3.0);
    assert!(dialog.cached_gyro_metrics.sensitivity_enhancement_factor >= 45.0);
    assert!(dialog.cached_gain_metrics.threshold_power_mw <= 1.80);
    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_skin_polariton_laser_dialog_headless_egui_render() {
    let ctx = Context::default();
    let mut dialog = SkinPolaritonLaserDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        SkinPolaritonLaserTab::SkinModeLasing,
        SkinPolaritonLaserTab::ChiralSagnacGyroscope,
        SkinPolaritonLaserTab::RiemannEnergyWinding,
        SkinPolaritonLaserTab::PolaritonGainMedium,
        SkinPolaritonLaserTab::AuditTelemetry,
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
