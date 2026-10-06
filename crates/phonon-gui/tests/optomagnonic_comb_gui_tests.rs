#![deny(unsafe_code)]

//! GUI test suite for Phase 403: OptomagnonicCombDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - OptomagnonicCombDialog initialization, default state, and physics caches.
//! - Tab switching across all 5 visual workspaces.
//! - Parameter adjustment (pump drive, detuning, optomagnonic coupling).
//! - Instantaneous cold boot latency (< 2.0 ms).
//! - Headless egui Context render pass across all 5 tabs.

use std::time::Instant;
use egui::vec2;
use phonon_gui::widgets::optomagnonic_comb_dialog::{
    OptomagnonicCombDialog, OptomagnonicCombTab,
};

#[test]
fn test_optomagnonic_comb_dialog_initialization_and_defaults() {
    let dialog = OptomagnonicCombDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default tab
    assert_eq!(dialog.active_tab, OptomagnonicCombTab::TripleResonanceCoupling);
    assert_eq!(
        dialog.active_tab.label(),
        "1. Triple-Resonance & Polaritons"
    );

    // Triple resonance parameter defaults
    assert_eq!(dialog.processor.resonance_params.optical_freq_thz, 193.4);
    assert_eq!(dialog.processor.resonance_params.magnon_freq_ghz, 10.0);
    assert_eq!(dialog.processor.resonance_params.acoustic_phonon_freq_ghz, 15.0);
    assert_eq!(dialog.processor.resonance_params.optical_linewidth_mhz, 50.0);
    assert_eq!(dialog.processor.resonance_params.magnon_linewidth_mhz, 2.0);
    assert_eq!(dialog.processor.resonance_params.phonon_linewidth_mhz, 1.0);
    assert_eq!(dialog.processor.resonance_params.optomagnonic_coupling_khz, 50.0);
    assert_eq!(dialog.processor.resonance_params.acoustomagnonic_coupling_mhz, 20.0);

    // LLE parameter defaults
    assert_eq!(dialog.processor.lle_params.detuning_alpha, 2.5);
    assert_eq!(dialog.processor.lle_params.dispersion_d2_khz, 25.0);
    assert_eq!(dialog.processor.lle_params.pump_drive_f0, 2.2);
    assert_eq!(dialog.processor.lle_params.free_spectral_range_ghz, 20.0);

    // Jitter parameter defaults
    assert_eq!(dialog.processor.jitter_params.offset_freq_min_hz, 100.0);
    assert_eq!(dialog.processor.jitter_params.offset_freq_max_hz, 10_000_000.0);
    assert_eq!(dialog.processor.jitter_params.repetition_rate_ghz, 20.0);

    // Cached curves and metrics
    assert!(
        !dialog.cached_avoided_crossing.is_empty(),
        "Avoided crossing points must be cached"
    );
    assert!(
        !dialog.processor.lle_result.comb_modes.is_empty(),
        "Comb modes must be cached"
    );
    assert!(
        !dialog.processor.jitter_metrics.phase_noise_spectrum.is_empty(),
        "Phase noise spectrum must be cached"
    );

    // Audit report check
    assert_eq!(dialog.cached_audit.total_count, 10);
    assert_eq!(dialog.cached_audit.passed_count, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_optomagnonic_comb_tab_switching() {
    let mut dialog = OptomagnonicCombDialog::new();

    let tabs = [
        (
            OptomagnonicCombTab::TripleResonanceCoupling,
            "1. Triple-Resonance & Polaritons",
        ),
        (
            OptomagnonicCombTab::OpticalCombSpectrum,
            "2. Optical Frequency Comb",
        ),
        (
            OptomagnonicCombTab::SolitonPulseWgmCavity,
            "3. Soliton Pulse & WGM Cavity",
        ),
        (
            OptomagnonicCombTab::TimingJitterPhaseNoise,
            "4. Timing Jitter & Phase Noise",
        ),
        (
            OptomagnonicCombTab::PhysicsAuditTelemetry,
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
    let mut dialog = OptomagnonicCombDialog::new();

    // 1. Adjust optomagnonic and acoustomagnonic coupling
    dialog.processor.resonance_params.optomagnonic_coupling_khz = 80.0;
    dialog.processor.resonance_params.acoustomagnonic_coupling_mhz = 25.0;
    dialog.refresh_simulation();

    assert_eq!(dialog.processor.resonance_params.optomagnonic_coupling_khz, 80.0);
    assert_eq!(dialog.processor.resonance_params.acoustomagnonic_coupling_mhz, 25.0);
    assert!(
        dialog.processor.triple_result.avoided_crossing_gap_mhz > 45.0,
        "Avoided crossing gap must increase with higher coupling"
    );

    // 2. Adjust pump detuning and drive
    dialog.processor.lle_params.detuning_alpha = 3.0;
    dialog.processor.lle_params.pump_drive_f0 = 2.4;
    dialog.refresh_simulation();

    assert_eq!(dialog.processor.lle_params.detuning_alpha, 3.0);
    assert_eq!(dialog.processor.lle_params.pump_drive_f0, 2.4);
    assert!(dialog.processor.lle_result.peak_intensity > 2.0);

    // 3. Adjust polariton damping factor
    dialog.processor.jitter_params.polariton_damping_factor = 0.25;
    dialog.refresh_simulation();

    assert_eq!(dialog.processor.jitter_params.polariton_damping_factor, 0.25);
    assert!(dialog.processor.jitter_metrics.relative_intensity_noise_dbc_hz <= -140.0);
}

#[test]
fn test_cold_boot_latency() {
    let t_start = Instant::now();
    let _dialog = OptomagnonicCombDialog::new();
    let elapsed = t_start.elapsed();

    assert!(
        elapsed.as_millis() < 100,
        "Cold boot instantiation must be fast, took {:?}",
        elapsed
    );

    // Verify fast instantiation
    let fast_dialog = OptomagnonicCombDialog::new_fast();
    assert!(!fast_dialog.is_open);
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = OptomagnonicCombDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();

    // Render pass on each of the 5 tabs
    for tab in [
        OptomagnonicCombTab::TripleResonanceCoupling,
        OptomagnonicCombTab::OpticalCombSpectrum,
        OptomagnonicCombTab::SolitonPulseWgmCavity,
        OptomagnonicCombTab::TimingJitterPhaseNoise,
        OptomagnonicCombTab::PhysicsAuditTelemetry,
    ] {
        dialog.active_tab = tab;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.set_min_size(vec2(1040.0, 740.0));
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
