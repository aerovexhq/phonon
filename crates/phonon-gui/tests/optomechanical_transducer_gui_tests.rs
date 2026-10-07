#![deny(unsafe_code)]

//! Comprehensive Automated Test Suite for Phase 408 GUI CAD Dialog:
//! Cryogenic Quantum Optomechanical Transducer & Microwave-to-Acoustic Interconnect.

use egui::Context;
use phonon_gui::widgets::optomechanical_transducer_dialog::{
    OptomechanicalTransducerDialog, OptomechanicalTransducerTab,
};
use std::time::Instant;

#[test]
fn test_optomechanical_transducer_dialog_initialization() {
    let dialog = OptomechanicalTransducerDialog::default();

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        OptomechanicalTransducerTab::TripartiteArchitecture
    );
    assert_eq!(dialog.omega_m_ghz, 4.0);
    assert_eq!(dialog.gamma_m_khz, 1.8);
    assert!(dialog.n_e_pump > 0.0);
    assert!(dialog.n_o_pump > 0.0);
    assert!(dialog.cached_audit_report.is_fully_compliant);
    assert_eq!(dialog.cached_audit_report.passed_count, 10);
}

#[test]
fn test_optomechanical_transducer_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = OptomechanicalTransducerDialog::new_fast();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(
        elapsed_ms < 5.0,
        "Cold boot latency too high: {:.3} ms (target < 5.0 ms)",
        elapsed_ms
    );
    assert!(dialog.cached_audit_report.is_fully_compliant);
}

#[test]
fn test_optomechanical_transducer_dialog_tab_switching() {
    let mut dialog = OptomechanicalTransducerDialog::default();

    let tabs = [
        OptomechanicalTransducerTab::TripartiteArchitecture,
        OptomechanicalTransducerTab::ConversionSpectrumEfficiency,
        OptomechanicalTransducerTab::GroundStateCoolingNoise,
        OptomechanicalTransducerTab::TransmonCoherentInterconnect,
        OptomechanicalTransducerTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_optomechanical_transducer_dialog_parameter_adjustment() {
    let mut dialog = OptomechanicalTransducerDialog::default();

    // Adjust parameters and recompute
    dialog.n_e_pump = 2.0e6;
    dialog.n_o_pump = 3.0e6;
    dialog.bath_temp_mk = 15.0;
    dialog.coupling_g_q_mhz = 50.0;
    dialog.recompute();

    assert!(dialog.cached_spectrum.len() >= 100);
    assert!(dialog.cached_cooling_curve.len() >= 30);
    assert!(dialog.cached_swap_trajectory.len() >= 40);
    assert!(dialog.cached_audit_report.is_fully_compliant);

    let peak_eff = dialog.transducer.transduction.compute_peak_transduction_efficiency();
    assert!(peak_eff >= 0.70);
}

#[test]
fn test_optomechanical_transducer_dialog_headless_render() {
    let mut dialog = OptomechanicalTransducerDialog::default();
    dialog.is_open = true;

    let ctx = Context::default();

    let tabs = [
        OptomechanicalTransducerTab::TripartiteArchitecture,
        OptomechanicalTransducerTab::ConversionSpectrumEfficiency,
        OptomechanicalTransducerTab::GroundStateCoolingNoise,
        OptomechanicalTransducerTab::TransmonCoherentInterconnect,
        OptomechanicalTransducerTab::AuditTelemetry,
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
