#![deny(unsafe_code)]

//! GUI test suite for Phase 428: Topological Acoustic Superconducting Nanowire Single-Phonon Detector (SNSPD) & Quantum Transceiver Dialog.

use egui::Context;
use phonon_gui::widgets::acoustic_snspd_dialog::{AcousticSnspdDialog, SnspdTab};

#[test]
fn test_acoustic_snspd_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = AcousticSnspdDialog::new_fast();
    let duration = start.elapsed();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, SnspdTab::NanowireHotspotDynamics);
    assert!(
        duration.as_millis() < 5,
        "Cold-boot latency must be under 5 ms (got {:?})",
        duration
    );
    assert!(dialog.cached_tele.bias_ratio > 0.85);
    assert!(dialog.cached_tele.hotspot_radius_nm > 10.0);
    assert!(dialog.cached_tele.peak_voltage_mv > 0.20);
    assert!(dialog.cached_tele.rise_time_ps < 60.0);
    assert!(dialog.cached_tele.timing_jitter_fwhm_ps < 6.0);
    assert!(dialog.cached_trans.max_count_rate_mcps >= 800.0);
    assert!(dialog.cached_trans.secret_key_rate_mbps >= 10.0);
    assert!(dialog.cached_trans.qber_percent <= 2.0);
    assert!(dialog.cached_audit.all_passed);
    assert_eq!(dialog.cached_audit.total_score, 10);
}

#[test]
fn test_acoustic_snspd_dialog_tab_switching() {
    let mut dialog = AcousticSnspdDialog::default();

    dialog.active_tab = SnspdTab::NanowireHotspotDynamics;
    assert_eq!(dialog.active_tab.label(), "Nanowire Hotspot Dynamics");

    dialog.active_tab = SnspdTab::SinglePhononCounting;
    assert_eq!(dialog.active_tab.label(), "Single-Phonon Counting");

    dialog.active_tab = SnspdTab::TimingJitterSpectrum;
    assert_eq!(dialog.active_tab.label(), "Timing Jitter Spectrum");

    dialog.active_tab = SnspdTab::QuantumTransceiver;
    assert_eq!(dialog.active_tab.label(), "Quantum Transceiver");

    dialog.active_tab = SnspdTab::AuditTelemetry;
    assert_eq!(dialog.active_tab.label(), "Physics Audit & Telemetry");
}

#[test]
fn test_acoustic_snspd_parameter_adjustment_and_recompute() {
    let mut dialog = AcousticSnspdDialog::default();

    dialog.bias_current_ib_ua = 21.0;
    dialog.critical_current_ic_ua = 24.0;
    dialog.kinetic_inductance_nh = 50.0;
    dialog.recompute();

    assert_eq!(dialog.processor.hotspot_solver.params.bias_current_ib_ua, 21.0);
    assert_eq!(dialog.processor.hotspot_solver.params.critical_current_ic_ua, 24.0);
    assert_eq!(dialog.processor.hotspot_solver.params.kinetic_inductance_nh, 50.0);
    assert!(dialog.cached_audit.all_passed);
    assert_eq!(dialog.cached_audit.total_score, 10);
}

#[test]
fn test_acoustic_snspd_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = AcousticSnspdDialog::default();
    dialog.is_open = true;

    // Test render pass for all 5 tabs
    let tabs = [
        SnspdTab::NanowireHotspotDynamics,
        SnspdTab::SinglePhononCounting,
        SnspdTab::TimingJitterSpectrum,
        SnspdTab::QuantumTransceiver,
        SnspdTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
