#![deny(unsafe_code)]

//! Integration & Cold-Boot Performance Tests for Non-Hermitian Skin-Effect Sensor & EP Magnetometer GUI.

use std::time::Instant;
use egui::Context;
use phonon_gui::{NonHermitianSensorDialog, NonHermitianSensorTab};
use phonon_solver::non_hermitian_sensor::ExceptionalPointOrder;

#[test]
fn test_nh_sensor_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = NonHermitianSensorDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot constructor must complete in < 5ms for instant responsiveness, took {:?}",
        elapsed
    );
    assert!(!dialog.is_open, "Dialog must be closed by default on cold boot");
    assert_eq!(
        dialog.active_tab,
        NonHermitianSensorTab::PointGapGbz,
        "Dialog must default to Point-Gap & GBZ Spectrum tab"
    );
}

#[test]
fn test_nh_sensor_dialog_recalculation_and_audit() {
    let dialog = NonHermitianSensorDialog::new();

    // 1. Audit verification: 10/10 criteria must pass
    for crit in &dialog.audit_criteria {
        assert!(
            crit.is_passed,
            "Criterion '{}' failed: spec='{}', obs='{}', notes='{}'",
            crit.criterion,
            crit.specification,
            crit.observed_state,
            crit.technical_notes
        );
    }
    assert_eq!(
        dialog.audit_score,
        (10, 10),
        "Non-Hermitian sensor audit must achieve perfect 10/10 pass score, got {:?}",
        dialog.audit_score
    );

    // 2. Physical readouts
    assert_eq!(dialog.skin_solver.winding_number, 1);
    assert!(dialog.skin_solver.gbz_radius < 1.0);
    assert!(dialog.skin_solver.skin_localization_ratio >= 0.80);
    assert!(dialog.telemetry.noise_floor_pt_per_rthz < 1.0);
    assert!(dialog.telemetry.dynamic_range_db >= 60.0);
}

#[test]
fn test_nh_sensor_dialog_parameter_adjustments() {
    let mut dialog = NonHermitianSensorDialog::new();

    // 1. Invert asymmetric hopping: t_R = 2.0 MHz, t_L = 10.0 MHz
    dialog.t_R = 2.0;
    dialog.t_L = 10.0;
    dialog.recalculate();
    assert_eq!(
        dialog.skin_solver.winding_number, -1,
        "Winding number must flip to -1 when backward hopping dominates"
    );
    assert!(
        dialog.skin_solver.gbz_radius > 1.0,
        "GBZ radius must invert when t_L > t_R"
    );

    // 2. Switch EP order to EP3
    dialog.ep_order = ExceptionalPointOrder::EP3;
    dialog.kappa_0 = 4.0;
    dialog.gamma_ep = 4.0 * std::f64::consts::SQRT_2;
    dialog.recalculate();
    assert_eq!(dialog.ep_order, ExceptionalPointOrder::EP3);
    let exponent3 = dialog.ep_sensor.verify_power_law(1e-4, 1e-6);
    assert!(
        (exponent3 - (1.0 / 3.0)).abs() < 0.05,
        "EP3 power-law exponent must be close to 0.333, got {:.4}",
        exponent3
    );

    // 3. Magnetometer external field adjustment
    let initial_snr = dialog.telemetry.snr_db;
    dialog.delta_b_pt = 20.0;
    dialog.recalculate();
    assert!(
        dialog.telemetry.snr_db > initial_snr,
        "Higher applied magnetic field must increase SNR"
    );
    assert_eq!(dialog.telemetry.measured_field_pt, 20.0);
}

#[test]
fn test_nh_sensor_dialog_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = NonHermitianSensorDialog::new();
    dialog.is_open = true;

    let tabs = [
        NonHermitianSensorTab::PointGapGbz,
        NonHermitianSensorTab::NonHermitianSkinEffect,
        NonHermitianSensorTab::ExceptionalPointSensitivity,
        NonHermitianSensorTab::SubPicoteslaMagnetometer,
        NonHermitianSensorTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
        assert!(dialog.is_open, "Dialog must remain open across tab switching");
    }
}
