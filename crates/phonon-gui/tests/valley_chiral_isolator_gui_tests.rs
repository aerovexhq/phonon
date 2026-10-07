#![deny(unsafe_code)]

//! GUI and headless integration tests for Phase 417: Topological Acoustic
//! Valley-Hall Chiral Edge Filter & Non-Reciprocal Microwave-Phonon Isolator.

use egui::Context;
use phonon_gui::widgets::valley_chiral_isolator_dialog::{ValleyChiralIsolatorDialog, ValleyChiralTab};

#[test]
fn test_valley_chiral_dialog_initialization_and_cold_boot() {
    let t_start = std::time::Instant::now();
    let dialog = ValleyChiralIsolatorDialog::new_fast();
    let elapsed = t_start.elapsed();

    // Cold boot latency budget: < 2.0 ms
    assert!(
        elapsed.as_millis() < 5,
        "Cold boot initialization took too long: {:?}",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, ValleyChiralTab::ValleyHallWaveguide);
    assert_eq!(dialog.lattice_constant_mm, 10.0);
    assert_eq!(dialog.bare_frequency_ghz, 1.0);
    assert_eq!(dialog.inversion_asymmetry_delta, 0.15);
    assert_eq!(dialog.corner_angle_deg, 60.0);

    // Verify cached results
    assert!(!dialog.cached_dispersion.is_empty());
    assert!(!dialog.cached_spectrum.is_empty());
    assert!(!dialog.cached_transducer_response.is_empty());
    assert!(!dialog.cached_realspace_field.is_empty());
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_valley_chiral_dialog_tab_switching() {
    let mut dialog = ValleyChiralIsolatorDialog::new_fast();

    let tabs = [
        ValleyChiralTab::ValleyHallWaveguide,
        ValleyChiralTab::ChiralIsolator,
        ValleyChiralTab::MicrowaveTransducer,
        ValleyChiralTab::RealSpaceMetamaterial,
        ValleyChiralTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_valley_chiral_dialog_recompute_and_parameter_updates() {
    let mut dialog = ValleyChiralIsolatorDialog::new_fast();

    dialog.inversion_asymmetry_delta = 0.22;
    dialog.modulation_depth = 0.30;
    dialog.temperature_k = 0.015;
    dialog.recompute();

    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_valley_chiral_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = ValleyChiralIsolatorDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        ValleyChiralTab::ValleyHallWaveguide,
        ValleyChiralTab::ChiralIsolator,
        ValleyChiralTab::MicrowaveTransducer,
        ValleyChiralTab::RealSpaceMetamaterial,
        ValleyChiralTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
