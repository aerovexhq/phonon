#![deny(unsafe_code)]

//! GUI test suite for Phase 412: PolaritonBecDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - PolaritonBecDialog initialization, default configuration, and physics caches.
//! - Seamless tab switching across all 5 visual workspaces.
//! - Preset switching (High-Q cavity vs Near-Threshold) and parameter updates.
//! - Instantaneous cold boot latency (< 5.0 ms).
//! - Headless egui Context render pass across all 5 tabs.

use std::time::Instant;
use egui::vec2;
use phonon_gui::widgets::polariton_bec_dialog::{PolaritonBecDialog, PolaritonBecTab};
use phonon_solver::polariton_bec_vortices::VortexCharge;

#[test]
fn test_polariton_bec_dialog_initialization_and_defaults() {
    let dialog = PolaritonBecDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default tab
    assert_eq!(dialog.active_tab, PolaritonBecTab::GrossPitaevskiiCondensate);
    assert_eq!(
        dialog.active_tab.label(),
        "1. GPE Condensation & Threshold"
    );

    // Physical parameter defaults
    assert!((dialog.pump_power_ratio - 2.2).abs() < 1e-4);
    assert!((dialog.flow_velocity_ratio - 0.55).abs() < 1e-4);
    assert!((dialog.josephson_coupling_ej_mev - 0.045).abs() < 1e-4);

    // Initial condensation metrics
    assert!(dialog.cached_gpe_metrics.condensate_fraction > 0.50);
    assert!(dialog.cached_gpe_metrics.peak_density_um2 > 0.0);
    assert!(dialog.cached_gpe_metrics.sound_speed_ms > 0.0);
    assert!(dialog.cached_gpe_metrics.healing_length_um > 0.0);

    // Initial vortex metrics
    assert_eq!(dialog.vortex_charge, VortexCharge::PlusOne);
    assert!(dialog.cached_vortex_metrics.drag_suppression_db >= 30.0);
    assert!(dialog.cached_vortex_metrics.is_frictionless_superfluid);

    // Initial Josephson interferometer metrics
    assert!(dialog.cached_josephson_metrics.fringe_visibility >= 0.90);
    assert!(dialog.cached_josephson_metrics.minimum_detectable_strain < 1.0e-9);

    // 10-point audit full pass
    assert_eq!(dialog.cached_audit.passed_count, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_polariton_bec_tab_switching() {
    let mut dialog = PolaritonBecDialog::new();

    let tabs = [
        (
            PolaritonBecTab::GrossPitaevskiiCondensate,
            "1. GPE Condensation & Threshold",
        ),
        (
            PolaritonBecTab::QuantizedVortexLattice,
            "2. Quantized Vortices & Lattice",
        ),
        (
            PolaritonBecTab::LandauSuperfluidity,
            "3. Landau Superfluidity",
        ),
        (
            PolaritonBecTab::JosephsonStrainSensor,
            "4. Josephson Acoustic Interferometer",
        ),
        (
            PolaritonBecTab::AuditTelemetry,
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
fn test_polariton_bec_presets_and_recomputation() {
    let mut dialog = PolaritonBecDialog::new();

    // 1. High-Q Microcavity preset
    dialog.load_preset_high_q();
    assert!(dialog.cached_gpe_metrics.condensate_fraction > 0.50);
    assert!(dialog.pump_power_ratio > 1.0);
    assert!(dialog.cached_josephson_metrics.fringe_visibility >= 0.90);
    assert!(dialog.cached_audit.all_passed);

    // 2. Near-Threshold Criticality preset
    dialog.load_preset_near_threshold();
    assert!(dialog.cached_gpe_metrics.condensate_fraction > 0.0);
    if !dialog.cached_audit.all_passed {
        for c in &dialog.cached_audit.criteria {
            if !c.passed {
                eprintln!("NEAR-THRESHOLD FAILED CRITERION: {} - expected: {}, actual: {}", c.name, c.expected, c.actual);
            }
        }
    }
    assert!(dialog.cached_audit.all_passed);

    // 3. Manual parameter adjustment and recomputation
    dialog.pump_power_ratio = 2.0;
    dialog.flow_velocity_ratio = 0.35;
    dialog.josephson_coupling_ej_mev = 0.20;
    dialog.recompute();

    assert!(dialog.cached_gpe_metrics.peak_density_um2 > 0.0);
    assert!(dialog.cached_vortex_metrics.drag_suppression_db >= 30.0);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_polariton_bec_cold_boot_latency() {
    let t_start = Instant::now();
    let dialog = PolaritonBecDialog::new_fast();
    let elapsed = t_start.elapsed();

    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be fast, took {:?}",
        elapsed
    );

    assert!(
        dialog.last_solve_time_us < 5000.0,
        "Reported last solve time must be < 5000 us, got {} us",
        dialog.last_solve_time_us
    );
}

#[test]
fn test_polariton_bec_headless_egui_render_pass() {
    let mut dialog = PolaritonBecDialog::new_fast();
    dialog.is_open = true;

    let ctx = egui::Context::default();

    for tab in [
        PolaritonBecTab::GrossPitaevskiiCondensate,
        PolaritonBecTab::QuantizedVortexLattice,
        PolaritonBecTab::LandauSuperfluidity,
        PolaritonBecTab::JosephsonStrainSensor,
        PolaritonBecTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.set_min_size(vec2(1000.0, 700.0));
            dialog.render_content(ui);
        });
        output.textures_delta.clear();

        assert_eq!(dialog.active_tab, tab);
    }
}
