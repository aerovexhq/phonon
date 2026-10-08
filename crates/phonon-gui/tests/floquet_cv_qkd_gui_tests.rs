#![deny(unsafe_code)]

//! GUI test suite for Phase 429: Topological Acoustic Floquet Chiral Magnon-Phonon
//! Entanglement Router & Continuous-Variable Quantum Key Distribution (CV-QKD) Dialog.

use egui::Context;
use phonon_gui::widgets::floquet_cv_qkd_dialog::{CvQkdTab, FloquetCvQkdDialog};

#[test]
fn test_floquet_cv_qkd_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = FloquetCvQkdDialog::new_fast();
    let duration = start.elapsed();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, CvQkdTab::ChiralPolaritonRouting);
    assert!(
        duration.as_millis() < 5,
        "Cold-boot latency must be under 5 ms (got {:?})",
        duration
    );
    assert!(dialog.cached_router_tele.chiral_isolation_db >= 30.0);
    assert!(dialog.cached_router_tele.forward_transmittance >= 0.90);
    assert!(dialog.cached_cov.squeezing_db >= 8.0);
    assert!(dialog.cached_cov.symplectic_eigenvalue < 1.0);
    assert!(dialog.cached_qkd_tele.secret_key_rate_mbps >= 5.0);
    assert!(dialog.cached_audit.all_passed);
    assert_eq!(dialog.cached_audit.total_score, 10);
}

#[test]
fn test_floquet_cv_qkd_dialog_tab_switching() {
    let mut dialog = FloquetCvQkdDialog::default();

    dialog.active_tab = CvQkdTab::ChiralPolaritonRouting;
    assert_eq!(dialog.active_tab.label(), "Chiral Polariton Routing");

    dialog.active_tab = CvQkdTab::EprEntanglementCovariance;
    assert_eq!(dialog.active_tab.label(), "EPR Entanglement & Covariance");

    dialog.active_tab = CvQkdTab::CvQkdSecretKeyRate;
    assert_eq!(dialog.active_tab.label(), "CV-QKD Secret Key Rate");

    dialog.active_tab = CvQkdTab::AvionicsQuantumBus;
    assert_eq!(dialog.active_tab.label(), "Avionics Quantum Bus");

    dialog.active_tab = CvQkdTab::AuditTelemetry;
    assert_eq!(dialog.active_tab.label(), "Physics Audit & Telemetry");
}

#[test]
fn test_floquet_cv_qkd_parameter_adjustment_and_recompute() {
    let mut dialog = FloquetCvQkdDialog::default();

    dialog.squeezing_r = 1.45;
    dialog.distance_m = 15.0;
    dialog.drive_amplitude_mhz = 60.0;
    dialog.recompute();

    assert_eq!(dialog.processor.qkd_engine.params.squeezing_r, 1.45);
    assert_eq!(dialog.processor.qkd_engine.params.distance_m, 15.0);
    assert_eq!(dialog.processor.router.params.drive_amplitude_mhz, 60.0);
    assert!(dialog.cached_audit.all_passed);
    assert_eq!(dialog.cached_audit.total_score, 10);
}

#[test]
fn test_floquet_cv_qkd_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = FloquetCvQkdDialog::default();
    dialog.is_open = true;

    // Test render pass for all 5 tabs
    let tabs = [
        CvQkdTab::ChiralPolaritonRouting,
        CvQkdTab::EprEntanglementCovariance,
        CvQkdTab::CvQkdSecretKeyRate,
        CvQkdTab::AvionicsQuantumBus,
        CvQkdTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
