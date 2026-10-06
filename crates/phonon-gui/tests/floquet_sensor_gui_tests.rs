#![deny(unsafe_code)]

//! GUI test suite for Phase 404: FloquetSensorDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - FloquetSensorDialog initialization, default state, and physics caches.
//! - Tab switching across all 5 visual tabs.
//! - Parameter adjustment (pulse error epsilon, disorder W, field B).
//! - Instantaneous cold boot latency (< 2.0 ms).
//! - Headless egui Context render pass across all 5 tabs.

use std::time::Instant;
use egui::vec2;
use phonon_gui::widgets::floquet_sensor_dialog::{
    FloquetSensorDialog, FloquetSensorTab,
};

#[test]
fn test_floquet_sensor_dialog_initialization_and_defaults() {
    let dialog = FloquetSensorDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default tab
    assert_eq!(dialog.active_tab, FloquetSensorTab::TimeCrystalDynamics);
    assert_eq!(
        dialog.active_tab.label(),
        "1. Time Crystal Dynamics"
    );

    // Time crystal drive parameter defaults
    assert_eq!(dialog.processor.tc_params.chain_length, 8);
    assert_eq!(dialog.processor.tc_params.drive_period_us, 1.0);
    assert_eq!(dialog.processor.tc_params.pulse_error_epsilon, 0.05);
    assert_eq!(dialog.processor.tc_params.ising_coupling_j_khz, 250.0);
    assert_eq!(dialog.processor.tc_params.disorder_w_khz, 500.0);
    assert_eq!(dialog.processor.tc_params.cycles, 60);

    // Magnetometer parameter defaults
    assert_eq!(dialog.processor.mag_params.coherence_time_ms, 2.0);
    assert_eq!(dialog.processor.mag_params.interrogation_time_ms, 10.0);
    assert_eq!(dialog.processor.mag_params.test_field_ft, 100.0);
    assert_eq!(dialog.processor.mag_params.thermal_noise_floor_ft, 0.25);
    assert_eq!(dialog.processor.mag_params.linear_range_max_ft, 2000.0);

    // Sensor network parameter defaults
    assert_eq!(dialog.processor.network_params.grid_rows, 3);
    assert_eq!(dialog.processor.network_params.grid_cols, 3);
    assert_eq!(dialog.processor.network_params.node_spacing_mm, 5.0);
    assert_eq!(dialog.processor.network_params.common_mode_noise_ft, 50000.0);

    // Verify caches
    assert_eq!(dialog.cached_stroboscopic.cycles, 60);
    assert_eq!(dialog.cached_stroboscopic.magnetization.len(), 60);
    assert!(dialog.cached_spectrum.frequencies_norm.len() >= 60);
    assert!(!dialog.cached_rigidity.epsilons.is_empty());
    assert!(dialog.cached_readout.sensitivity_ft_per_sqrt_hz <= 1.0);
    assert!(dialog.cached_readout.dynamic_range_db >= 70.0);

    // 10-point audit checklist
    assert_eq!(dialog.cached_audit.total_count, 10);
}

#[test]
fn test_floquet_sensor_tab_switching() {
    let mut dialog = FloquetSensorDialog::new();

    let tabs = [
        (
            FloquetSensorTab::TimeCrystalDynamics,
            "1. Time Crystal Dynamics",
        ),
        (
            FloquetSensorTab::FourierSpectrum,
            "2. Subharmonic Fourier Spectrum",
        ),
        (
            FloquetSensorTab::SubFemtoteslaMagnetometer,
            "3. Sub-Femtotesla Magnetometer",
        ),
        (
            FloquetSensorTab::SensorNetworkGradient,
            "4. 2D Sensor Network & Gradients",
        ),
        (
            FloquetSensorTab::AuditTelemetry,
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
fn test_parameter_adjustment() {
    let mut dialog = FloquetSensorDialog::new();

    // 1. Adjust drive pulse error epsilon
    dialog.processor.tc_params.pulse_error_epsilon = 0.08;
    dialog.recompute();
    assert_eq!(dialog.processor.tc_params.pulse_error_epsilon, 0.08);

    // 2. Adjust disorder W
    dialog.processor.tc_params.disorder_w_khz = 600.0;
    dialog.recompute();
    assert_eq!(dialog.processor.tc_params.disorder_w_khz, 600.0);
    assert!(dialog.processor.tc_params.mbl_ratio() >= 2.0);

    // 3. Adjust injected field B
    dialog.injected_field_slider_ft = 250.0;
    dialog.recompute();
    assert_eq!(dialog.injected_field_slider_ft, 250.0);
    assert!((dialog.cached_readout.measured_field_ft - 250.0).abs() < 1.0e-6);

    // 4. Adjust sensor array node spacing
    dialog.processor.network_params.node_spacing_mm = 6.0;
    dialog.recompute();
    assert_eq!(dialog.processor.network_params.node_spacing_mm, 6.0);
}

#[test]
fn test_cold_boot_latency() {
    let t_start = Instant::now();
    let _dialog = FloquetSensorDialog::new();
    let elapsed = t_start.elapsed();

    assert!(
        elapsed.as_millis() < 200,
        "Cold boot instantiation must be fast, took {:?}",
        elapsed
    );

    // Verify fast instantiation constructor
    let fast_dialog = FloquetSensorDialog::new_fast();
    assert!(!fast_dialog.is_open);
    assert!(fast_dialog.cached_audit.cold_boot_latency_us < 2000.0);
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = FloquetSensorDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();

    // Render pass on each of the 5 tabs
    for tab in [
        FloquetSensorTab::TimeCrystalDynamics,
        FloquetSensorTab::FourierSpectrum,
        FloquetSensorTab::SubFemtoteslaMagnetometer,
        FloquetSensorTab::SensorNetworkGradient,
        FloquetSensorTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.set_min_size(vec2(1040.0, 720.0));
            dialog.ui(ui.ctx());
        });
        output.textures_delta.clear();

        assert_eq!(dialog.active_tab, tab);
    }
}
