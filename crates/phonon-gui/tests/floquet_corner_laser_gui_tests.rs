#![deny(unsafe_code)]

//! GUI Integration and Headless Render Test Suite for Phase 421:
//! Phonon Studio Topological Acoustic Floquet Higher-Order Corner-State Laser
//! & Non-Hermitian Vortex Amplifier.

use egui::Context;
use phonon_gui::widgets::floquet_corner_laser_dialog::{
    FloquetCornerLaserDialog, FloquetCornerLaserTab,
};
use phonon_solver::floquet_corner_laser::VortexOamCharge;

#[test]
fn test_floquet_corner_laser_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = FloquetCornerLaserDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, FloquetCornerLaserTab::CornerModeLasing);
    assert_eq!(dialog.cached_corner_modes.len(), 4);
    assert_eq!(dialog.cached_emission_spectrum.len(), 51);
    assert_eq!(dialog.cached_gain_spectrum.len(), 51);
    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency {:?} must be strictly under 5 ms",
        elapsed
    );
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_floquet_corner_laser_dialog_tab_switching() {
    let mut dialog = FloquetCornerLaserDialog::new_fast();

    let tabs = [
        FloquetCornerLaserTab::CornerModeLasing,
        FloquetCornerLaserTab::FloquetModulation,
        FloquetCornerLaserTab::VortexAmplifier,
        FloquetCornerLaserTab::RealSpaceMetamaterial,
        FloquetCornerLaserTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_floquet_corner_laser_dialog_recompute_and_parameter_updates() {
    let mut dialog = FloquetCornerLaserDialog::new_fast();

    dialog.pump_power_mw = 35.0;
    dialog.intracell_gamma_mhz = 1.8;
    dialog.intercell_lambda_mhz = 9.0;
    dialog.selected_charge = VortexOamCharge::PlusTwo;
    dialog.interaction_length_mm = 28.0;

    dialog.recompute();

    assert_eq!(dialog.cached_corner_modes.len(), 4);
    assert_eq!(dialog.cached_emission_spectrum.len(), 51);
    assert_eq!(dialog.cached_gain_spectrum.len(), 51);
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_floquet_corner_laser_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = FloquetCornerLaserDialog::new_fast();
    dialog.is_open = true;

    for tab in &[
        FloquetCornerLaserTab::CornerModeLasing,
        FloquetCornerLaserTab::FloquetModulation,
        FloquetCornerLaserTab::VortexAmplifier,
        FloquetCornerLaserTab::RealSpaceMetamaterial,
        FloquetCornerLaserTab::AuditTelemetry,
    ] {
        dialog.active_tab = *tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
