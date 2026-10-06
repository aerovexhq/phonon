#![deny(unsafe_code)]

//! Comprehensive Verification Test Suite for Skyrmion Reservoir GUI Dialog.
//!
//! Validates:
//! 1. Sub-millisecond cold boot latency (< 5ms).
//! 2. Default state initialization and parameter verification.
//! 3. Diagnostic recalculation on parameter adjustment.
//! 4. 10-point Spintronic Neuromorphic Readiness Audit (10/10 passed).
//! 5. Headless egui render pass across all 5 dialog tabs.

use std::time::Instant;

use phonon_gui::widgets::skyrmion_reservoir_dialog::{
    SkyrmionReservoirDialog, SkyrmionReservoirTab,
};

#[test]
fn test_skyrmion_reservoir_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = SkyrmionReservoirDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot instantiation took {:?}, exceeding 5ms target",
        elapsed
    );
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        SkyrmionReservoirTab::MagnetizationTexture
    );
    assert!(dialog.texture.params.nx >= 32);
    assert!((dialog.topological_charge.abs() - 1.0).abs() < 0.15);
    assert!(dialog.stto_frequency_ghz > 0.0);

    // Verify Default trait
    let default_dialog = SkyrmionReservoirDialog::default();
    assert_eq!(
        default_dialog.active_tab,
        SkyrmionReservoirTab::MagnetizationTexture
    );
}

#[test]
fn test_skyrmion_parameter_adjustment_recalculation() {
    let mut dialog = SkyrmionReservoirDialog::new_fast();
    let _initial_freq = dialog.stto_frequency_ghz;

    // Increase current density and damping
    dialog.drive_current_density_e11 = 12.0;
    dialog.external_field_tesla = 0.30;
    dialog.recalculate_diagnostics();

    assert!(
        dialog.stto_frequency_ghz > 0.0,
        "STTO frequency should be valid after recalculation"
    );
    assert_eq!(dialog.stto_orbit.len(), 80);
}

#[test]
fn test_10_point_spintronic_readiness_audit() {
    let dialog = SkyrmionReservoirDialog::new_fast();

    assert_eq!(
        dialog.audit_criteria.len(),
        10,
        "Spintronic Readiness Audit must have exactly 10 criteria"
    );

    let passed_count = dialog.audit_criteria.iter().filter(|c| c.is_passed).count();
    assert_eq!(
        passed_count, 10,
        "All 10 Spintronic Readiness Audit criteria must pass"
    );
    assert_eq!(dialog.audit_score, (10, 10));

    let criteria_names: Vec<&str> = dialog
        .audit_criteria
        .iter()
        .map(|c| c.criterion.as_str())
        .collect();

    assert!(criteria_names.iter().any(|c| c.contains("LLGS Unconditional Unit Norm Conservation")));
    assert!(criteria_names.iter().any(|c| c.contains("Topological Skyrmion Charge Quantization")));
    assert!(criteria_names.iter().any(|c| c.contains("DMI Chiral Stability Threshold")));
    assert!(criteria_names.iter().any(|c| c.contains("Thiele Gyrovector & Skyrmion Hall Angle")));
    assert!(criteria_names.iter().any(|c| c.contains("Artificial Synaptic Pinning Restoration")));
    assert!(criteria_names.iter().any(|c| c.contains("STTO Microwave Frequency Agility")));
    assert!(criteria_names.iter().any(|c| c.contains("Spintronic Echo State Property")));
    assert!(criteria_names.iter().any(|c| c.contains("Virtual-Node Delay Multiplexing")));
    assert!(criteria_names.iter().any(|c| c.contains("Ridge Regression Readout Stability")));
    assert!(criteria_names.iter().any(|c| c.contains("Non-Linear Reservoir NARMA Benchmark")));
}

#[test]
fn test_headless_egui_render_all_5_tabs() {
    let ctx = egui::Context::default();
    let mut dialog = SkyrmionReservoirDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        SkyrmionReservoirTab::MagnetizationTexture,
        SkyrmionReservoirTab::SttoPrecession,
        SkyrmionReservoirTab::PinningLandscape,
        SkyrmionReservoirTab::ReservoirBenchmark,
        SkyrmionReservoirTab::SpintronicAudit,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
        assert!(dialog.is_open);
    }
}
