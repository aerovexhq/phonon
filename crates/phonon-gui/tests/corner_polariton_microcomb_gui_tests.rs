#![deny(unsafe_code)]

//! Automated GUI test suite for Phase 409: Topological Corner-Polariton Micro-Comb
//! Soliton & Dissipative Kerr Acoustic Frequency Synthesizer dialog.

use egui::Context;
use phonon_gui::widgets::corner_polariton_microcomb_dialog::{
    CornerPolaritonMicrocombDialog, CornerPolaritonMicrocombTab,
};
use std::time::Instant;

#[test]
fn test_corner_polariton_microcomb_dialog_initialization() {
    let dialog = CornerPolaritonMicrocombDialog::default();

    assert!(!dialog.is_open, "Dialog must start closed by default");
    assert_eq!(
        dialog.active_tab,
        CornerPolaritonMicrocombTab::CornerPolaritonCavity,
        "Default tab must be CornerPolaritonCavity"
    );
    assert!(!dialog.cached_modes.is_empty(), "Cached modes must be populated");
    assert!(!dialog.cached_spectrum.is_empty(), "Cached spectrum must be populated");
    assert!(!dialog.cached_temporal.is_empty(), "Cached temporal profile must be populated");
    assert_eq!(dialog.cached_audit_report.total_count, 10, "Audit must evaluate 10 criteria");
    assert_eq!(dialog.cached_audit_report.passed_count, 10, "Audit must achieve 10/10 PASS");
}

#[test]
fn test_corner_polariton_microcomb_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = CornerPolaritonMicrocombDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot latency must be strictly sub-5ms (measured: {:.3} ms)",
        elapsed.as_secs_f64() * 1000.0
    );
    assert_eq!(dialog.cached_audit_report.passed_count, 10);
}

#[test]
fn test_corner_polariton_microcomb_dialog_tab_switching() {
    let mut dialog = CornerPolaritonMicrocombDialog::default();

    let tabs = [
        CornerPolaritonMicrocombTab::CornerPolaritonCavity,
        CornerPolaritonMicrocombTab::DissipativeSolitonComb,
        CornerPolaritonMicrocombTab::KerrBistabilityMI,
        CornerPolaritonMicrocombTab::TimingSynthesizerF2F,
        CornerPolaritonMicrocombTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_corner_polariton_microcomb_dialog_parameter_adjustment() {
    let mut dialog = CornerPolaritonMicrocombDialog::default();

    dialog.pump_power_mw = 35.0;
    dialog.pump_detuning_mhz = 18.0;
    dialog.recompute();

    assert!(dialog.last_solve_time_us > 0.0);
    assert!(dialog.synthesizer.soliton.params.pump_power_mw == 35.0);
    assert!(dialog.synthesizer.soliton.params.pump_detuning_mhz == 18.0);
    assert!(!dialog.cached_spectrum.is_empty());
    assert!(!dialog.cached_temporal.is_empty());
}

#[test]
fn test_corner_polariton_microcomb_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = CornerPolaritonMicrocombDialog::default();
    dialog.is_open = true;

    let tabs = [
        CornerPolaritonMicrocombTab::CornerPolaritonCavity,
        CornerPolaritonMicrocombTab::DissipativeSolitonComb,
        CornerPolaritonMicrocombTab::KerrBistabilityMI,
        CornerPolaritonMicrocombTab::TimingSynthesizerF2F,
        CornerPolaritonMicrocombTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
    }

    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();
}
