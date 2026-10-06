#![deny(unsafe_code)]

//! GUI test suite for Phase 401: ChiralCirculatorDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - ChiralCirculatorDialog initialization, default state, and physics caches.
//! - Tab switching across all 5 visual workspaces.
//! - Parameter adjustment (drive frequency, drive amplitude, active port switching).
//! - Instantaneous cold boot latency (< 2.0 ms).
//! - Headless egui Context render pass across all 5 tabs.

use std::time::Instant;
use egui::vec2;
use phonon_gui::widgets::chiral_circulator_dialog::{
    ChiralCirculatorDialog, ChiralCirculatorTab,
};

#[test]
fn test_chiral_circulator_dialog_initialization_and_defaults() {
    let dialog = ChiralCirculatorDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default tab
    assert_eq!(dialog.active_tab, ChiralCirculatorTab::FloquetDispersion);
    assert_eq!(
        dialog.active_tab.label(),
        "1. Floquet Chiral Dispersion"
    );

    // Engine parameter defaults
    assert_eq!(dialog.engine.polariton_params.bare_phonon_freq_ghz, 5.0);
    assert_eq!(dialog.engine.polariton_params.bare_magnon_freq_ghz, 5.0);
    assert_eq!(dialog.engine.polariton_params.magnetoelastic_coupling_mhz, 40.0);
    assert_eq!(dialog.engine.polariton_params.floquet_drive_freq_ghz, 0.5);
    assert_eq!(dialog.engine.polariton_params.floquet_drive_amplitude_oe, 15.0);

    // Circulator parameter defaults
    assert_eq!(dialog.engine.circulator_params.center_freq_ghz, 5.0);
    assert_eq!(dialog.engine.circulator_params.bandwidth_3db_mhz, 120.0);
    assert_eq!(dialog.engine.circulator_params.port_impedance_ohms, 50.0);
    assert_eq!(dialog.engine.circulator_params.active_port, 1);

    // Cryogenic parameter defaults
    assert_eq!(dialog.engine.isolator_params.operating_temp_k, 0.020);

    // Cached curves and metrics
    assert!(
        !dialog.cached_dispersion.is_empty(),
        "Dispersion points must be cached"
    );
    assert!(
        !dialog.cached_s21_curve.is_empty(),
        "S21 curve points must be cached"
    );
    assert!(
        !dialog.cached_s12_curve.is_empty(),
        "S12 curve points must be cached"
    );
    assert!(
        !dialog.cached_s11_curve.is_empty(),
        "S11 curve points must be cached"
    );

    // Audit report check
    assert_eq!(dialog.cached_audit.total_count, 10);
    assert_eq!(dialog.cached_audit.passed_count, 10);
    assert!(dialog.cached_audit.overall_pass);
}

#[test]
fn test_chiral_circulator_tab_switching() {
    let mut dialog = ChiralCirculatorDialog::new();

    let tabs = [
        (
            ChiralCirculatorTab::FloquetDispersion,
            "1. Floquet Chiral Dispersion",
        ),
        (
            ChiralCirculatorTab::SParameters,
            "2. 3-Port Circulator S-Parameters",
        ),
        (
            ChiralCirculatorTab::ResonatorPressureField,
            "3. Resonator & Pressure Field",
        ),
        (
            ChiralCirculatorTab::CryogenicIsolationNoise,
            "4. Cryogenic Isolation & Noise",
        ),
        (
            ChiralCirculatorTab::AuditTelemetry,
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
fn test_parameter_adjustment_and_port_switching() {
    let mut dialog = ChiralCirculatorDialog::new();

    // 1. Adjust Floquet drive frequency and amplitude
    dialog.engine.polariton_params.floquet_drive_freq_ghz = 0.8;
    dialog.engine.polariton_params.floquet_drive_amplitude_oe = 25.0;
    dialog.refresh_simulation();

    let (_, _, delta_k) = dialog.engine.dispersion.forward_backward_wavenumbers(5.0);
    assert!(
        delta_k > 0.0,
        "Dispersion asymmetry must be positive after parameter adjustment"
    );

    // 2. Active port switching
    dialog.engine.circulator_params.active_port = 2;
    dialog.refresh_simulation();
    assert_eq!(dialog.engine.circulator_params.active_port, 2);

    let s = dialog.engine.center_s_matrix();
    let trans = dialog.engine.circulator.active_transmission(&s);
    assert!(
        trans.abs() >= 0.944,
        "Transmission at Port 2 must maintain insertion loss <= 0.5 dB: |S_32| = {}",
        trans.abs()
    );

    dialog.engine.circulator_params.active_port = 3;
    dialog.refresh_simulation();
    assert_eq!(dialog.engine.circulator_params.active_port, 3);
    let trans3 = dialog.engine.circulator.active_transmission(&s);
    assert!(
        trans3.abs() >= 0.944,
        "Transmission at Port 3 must maintain insertion loss <= 0.5 dB: |S_13| = {}",
        trans3.abs()
    );

    // 3. Temperature adjustment
    dialog.engine.isolator_params.operating_temp_k = 0.100; // 100 mK
    dialog.refresh_simulation();
    assert_eq!(dialog.engine.isolator_params.operating_temp_k, 0.100);
    assert!(dialog.cached_metrics.noise_temperature_k > 0.0);
}

#[test]
fn test_cold_boot_latency() {
    let t_start = Instant::now();
    let dialog = ChiralCirculatorDialog::new();
    let elapsed = t_start.elapsed();

    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be fast, took {:?}",
        elapsed
    );

    assert!(
        dialog.cached_audit.cold_boot_latency_us < 2000.0,
        "Cold boot latency {} us must be < 2000 us (2.0 ms)",
        dialog.cached_audit.cold_boot_latency_us
    );
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = ChiralCirculatorDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();

    // Render pass on each of the 5 tabs
    for tab in [
        ChiralCirculatorTab::FloquetDispersion,
        ChiralCirculatorTab::SParameters,
        ChiralCirculatorTab::ResonatorPressureField,
        ChiralCirculatorTab::CryogenicIsolationNoise,
        ChiralCirculatorTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.set_min_size(vec2(1000.0, 700.0));
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
}
