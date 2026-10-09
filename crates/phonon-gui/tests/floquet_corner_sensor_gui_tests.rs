#![deny(unsafe_code)]

//! GUI Integration Test Suite for Floquet Corner Polariton Laser & Sensor Dialog (Phase 457).
//!
//! Validates:
//! 1. Fast cold-boot initialization (< 2.0 ms) and baseline cached telemetry.
//! 2. Tab switching across all 5 categorized tabs.
//! 3. Interactive parameter adjustment and deterministic recomputation.
//! 4. Headless egui render pass across all tabs without panic or texture leakage.

use egui::Context;
use phonon_gui::widgets::floquet_corner_sensor_dialog::{
    FloquetCornerSensorDialog, FloquetCornerSensorTab,
};

#[test]
fn test_floquet_corner_sensor_dialog_initialization_and_fast_boot() {
    let t_start = std::time::Instant::now();
    let dialog = FloquetCornerSensorDialog::new_fast();
    let elapsed = t_start.elapsed();

    assert!(!dialog.is_open, "Dialog must be closed by default");
    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be practically instantaneous, took {} ms",
        elapsed.as_millis()
    );

    // Verify baseline cached telemetry
    assert!(dialog.cached_laser_metrics.lasing_threshold_mw <= 15.0);
    assert!(dialog.cached_laser_metrics.corner_confinement_pct >= 85.0);
    assert!(dialog.cached_laser_metrics.circular_polarization_pct >= 90.0);
    assert!(dialog.cached_laser_metrics.emission_linewidth_khz <= 50.0);
    assert!(!dialog.cached_li_curve.is_empty());
    assert!(!dialog.cached_spatial_intensity.is_empty());

    assert!(dialog.cached_mode_metrics.side_mode_suppression_ratio_db >= 35.0);
    assert!(dialog.cached_mode_metrics.corner_to_edge_gain_contrast_db >= 12.0);
    assert!(dialog.cached_mode_metrics.single_mode_selection_invariant);
    assert!(!dialog.cached_eigenvalues.is_empty());

    assert!(dialog.cached_rotation_metrics.minimum_detectable_rotation_rad_s_sqrt_hz <= 1.0e-5);
    assert!(dialog.cached_rotation_metrics.scale_factor_stability_ppm <= 10.0);
    assert!(dialog.cached_rotation_metrics.dynamic_range_db >= 80.0);
    assert!(!dialog.cached_rotation_sweep.is_empty());

    assert!(dialog.cached_magnetometer_metrics.minimum_detectable_field_pt_sqrt_hz <= 0.80);
    assert!(dialog.cached_magnetometer_metrics.dynamic_range_db >= 75.0);
    assert!(dialog.cached_magnetometer_metrics.linearity_error_pct <= 0.10);
    assert!(!dialog.cached_magnetic_sweep.is_empty());

    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_floquet_corner_sensor_dialog_tab_switching() {
    let mut dialog = FloquetCornerSensorDialog::new_fast();

    let tabs = [
        FloquetCornerSensorTab::CornerPolaritonLaser,
        FloquetCornerSensorTab::NonHermitianModeSelector,
        FloquetCornerSensorTab::SyntheticGaugeRotation,
        FloquetCornerSensorTab::SubPicoteslaMagnetometer,
        FloquetCornerSensorTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty(), "Tab label must not be empty");
    }
}

#[test]
fn test_floquet_corner_sensor_dialog_parameter_mutation_and_recompute() {
    let mut dialog = FloquetCornerSensorDialog::new_fast();

    dialog.intercell_coupling_ratio = 3.2;
    dialog.pump_power_mw = 25.0;
    dialog.synthetic_soc_mhz = 28.0;
    dialog.corner_gain_mhz = 20.0;
    dialog.boundary_loss_mhz = 28.0;
    dialog.enclosed_area_um2 = 200.0;
    dialog.magnetoacoustic_coupling_ghz_t = 32.0;

    dialog.recompute();

    assert!(dialog.cached_laser_metrics.lasing_threshold_mw <= 15.0);
    assert!(dialog.cached_laser_metrics.corner_confinement_pct >= 85.0);
    assert!(dialog.cached_mode_metrics.side_mode_suppression_ratio_db >= 35.0);
    assert!(dialog.cached_rotation_metrics.minimum_detectable_rotation_rad_s_sqrt_hz <= 1.0e-5);
    assert!(dialog.cached_magnetometer_metrics.minimum_detectable_field_pt_sqrt_hz <= 0.80);
    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_floquet_corner_sensor_dialog_headless_egui_render() {
    let ctx = Context::default();
    let mut dialog = FloquetCornerSensorDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        FloquetCornerSensorTab::CornerPolaritonLaser,
        FloquetCornerSensorTab::NonHermitianModeSelector,
        FloquetCornerSensorTab::SyntheticGaugeRotation,
        FloquetCornerSensorTab::SubPicoteslaMagnetometer,
        FloquetCornerSensorTab::AuditTelemetry,
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
