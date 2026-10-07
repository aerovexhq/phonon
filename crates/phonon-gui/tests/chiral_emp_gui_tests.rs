#![deny(unsafe_code)]

//! Automated GUI test suite for Phase 411: Phonon Studio Topological Chiral Acoustic
//! Edge-Magnetoplasmon Circulator & Non-Reciprocal Quantum Hall Router dialog.

use egui::Context;
use phonon_gui::widgets::chiral_emp_dialog::{ChiralEmpDialog, ChiralEmpDialogTab};
use std::time::Instant;

#[test]
fn test_chiral_emp_dialog_initialization() {
    let dialog = ChiralEmpDialog::default();

    assert!(!dialog.is_open, "Dialog must start closed by default");
    assert_eq!(
        dialog.active_tab,
        ChiralEmpDialogTab::DispersionSaw,
        "Default tab must be DispersionSaw"
    );
    assert!(!dialog.cached_dispersion.is_empty(), "Cached dispersion curve must be populated");
    assert!(!dialog.cached_spectrum.is_empty(), "Cached spectrum sweep must be populated");
    assert!(dialog.cached_transport.corner_power_transmission >= 0.95);
    assert_eq!(dialog.cached_audit.passed_count, 10, "Audit must achieve 10/10 PASS");
    assert!(dialog.cached_audit.all_passed, "Audit must be full pass");
}

#[test]
fn test_chiral_emp_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = ChiralEmpDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency must be strictly sub-5ms (measured: {:.3} ms)",
        elapsed.as_secs_f64() * 1000.0
    );
    assert_eq!(dialog.cached_audit.passed_count, 10);
}

#[test]
fn test_chiral_emp_dialog_tab_switching() {
    let mut dialog = ChiralEmpDialog::default();

    let tabs = [
        ChiralEmpDialogTab::DispersionSaw,
        ChiralEmpDialogTab::CirculatorSMatrix,
        ChiralEmpDialogTab::TopologicalDefectRouting,
        ChiralEmpDialogTab::SplitGateMultiplexer,
        ChiralEmpDialogTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_chiral_emp_dialog_preset_switching_and_recompute() {
    let mut dialog = ChiralEmpDialog::default();

    // Load High-Field nu=1 Preset
    dialog.load_preset_nu1();
    assert_eq!(dialog.filling_factor_nu, 1);
    assert!(dialog.magnetic_field_t > 8.0);
    assert!(dialog.defect_present);
    assert_eq!(dialog.cached_audit.passed_count, 10);

    // Load GaAs nu=2 Preset
    dialog.load_preset_nu2();
    assert_eq!(dialog.filling_factor_nu, 2);
    assert!((dialog.magnetic_field_t - 4.136).abs() < 0.01);
    assert!(!dialog.defect_present);
    assert_eq!(dialog.cached_audit.passed_count, 10);
}

#[test]
fn test_chiral_emp_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = ChiralEmpDialog::default();
    dialog.is_open = true;

    let tabs = [
        ChiralEmpDialogTab::DispersionSaw,
        ChiralEmpDialogTab::CirculatorSMatrix,
        ChiralEmpDialogTab::TopologicalDefectRouting,
        ChiralEmpDialogTab::SplitGateMultiplexer,
        ChiralEmpDialogTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
    }
}
