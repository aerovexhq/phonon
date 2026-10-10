#![deny(unsafe_code)]

//! GUI and headless integration tests for the HIL Protocol Bridge & Logic Analyzer Dialog.

use egui::Context;
use phonon_gui::widgets::{HilBridgeDialog, HilBridgeTab, ProtocolDecoderTab};
use std::time::Instant;

#[test]
fn test_hil_bridge_dialog_initialization_and_cold_boot() {
    let start = Instant::now();
    let dialog = HilBridgeDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(!dialog.is_open, "Dialog starts closed");
    assert_eq!(dialog.active_tab, HilBridgeTab::OscilloscopeStream);
    assert_eq!(dialog.decoder_tab, ProtocolDecoderTab::Spi);
    assert_eq!(dialog.audit_report.passed_count, 10);
    assert_eq!(dialog.audit_report.total_count, 10);
    assert!(dialog.audit_report.overall_pass);
    assert!(
        elapsed.as_micros() < 2000,
        "HilBridgeDialog::new_fast() must execute well under 2.0 ms (measured {} us)",
        elapsed.as_micros()
    );
}

#[test]
fn test_hil_bridge_dialog_tab_navigation() {
    let mut dialog = HilBridgeDialog::new_fast();

    dialog.active_tab = HilBridgeTab::LogicAnalyzer;
    assert_eq!(dialog.active_tab, HilBridgeTab::LogicAnalyzer);

    dialog.active_tab = HilBridgeTab::ScpiConsole;
    assert_eq!(dialog.active_tab, HilBridgeTab::ScpiConsole);

    dialog.active_tab = HilBridgeTab::CoSimulation;
    assert_eq!(dialog.active_tab, HilBridgeTab::CoSimulation);

    dialog.active_tab = HilBridgeTab::AuditTelemetry;
    assert_eq!(dialog.active_tab, HilBridgeTab::AuditTelemetry);

    dialog.active_tab = HilBridgeTab::OscilloscopeStream;
    assert_eq!(dialog.active_tab, HilBridgeTab::OscilloscopeStream);
}

#[test]
fn test_hil_bridge_protocol_decoder_tabs() {
    let mut dialog = HilBridgeDialog::new_fast();

    dialog.decoder_tab = ProtocolDecoderTab::I2c;
    assert_eq!(dialog.decoder_tab, ProtocolDecoderTab::I2c);

    dialog.decoder_tab = ProtocolDecoderTab::Uart;
    assert_eq!(dialog.decoder_tab, ProtocolDecoderTab::Uart);

    dialog.decoder_tab = ProtocolDecoderTab::Can;
    assert_eq!(dialog.decoder_tab, ProtocolDecoderTab::Can);

    dialog.decoder_tab = ProtocolDecoderTab::Spi;
    assert_eq!(dialog.decoder_tab, ProtocolDecoderTab::Spi);
}

#[test]
fn test_hil_bridge_scpi_command_execution() {
    let mut dialog = HilBridgeDialog::new_fast();
    let initial_history_len = dialog.command_history.len();

    let resp = dialog.engine.execute_scpi("*IDN?");
    dialog.command_history.push(("*IDN?".to_string(), resp, 135.0));

    assert_eq!(dialog.command_history.len(), initial_history_len + 1);
    let last = dialog.command_history.last().unwrap();
    assert_eq!(last.0, "*IDN?");
    assert!(last.1.contains("PHONON") || last.1.contains("VIRTUAL"));
}

#[test]
fn test_hil_bridge_co_sim_step() {
    let mut dialog = HilBridgeDialog::new_fast();
    let initial_steps = dialog.engine.synchronizer.metrics.co_sim_steps_completed;

    let metrics = dialog.engine.step_co_sim(10.0e-6, 1.65, 0x0001);
    assert_eq!(metrics.co_sim_steps_completed, initial_steps + 1);
    assert!(metrics.rms_jitter_us < 1.0);
}

#[test]
fn test_hil_bridge_dialog_headless_egui_render_pass() {
    let ctx = Context::default();
    let mut dialog = HilBridgeDialog::new_fast();
    dialog.is_open = true;

    // Render across each tab to verify zero UI panics
    let tabs = [
        HilBridgeTab::OscilloscopeStream,
        HilBridgeTab::LogicAnalyzer,
        HilBridgeTab::ScpiConsole,
        HilBridgeTab::CoSimulation,
        HilBridgeTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render(ui.ctx());
        });
        out.textures_delta.clear();
    }

    // Render logic analyzer with each decoder tab
    dialog.active_tab = HilBridgeTab::LogicAnalyzer;
    let decoder_tabs = [
        ProtocolDecoderTab::Spi,
        ProtocolDecoderTab::I2c,
        ProtocolDecoderTab::Uart,
        ProtocolDecoderTab::Can,
    ];
    for d_tab in decoder_tabs {
        dialog.decoder_tab = d_tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render(ui.ctx());
        });
        out.textures_delta.clear();
    }

    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();

    assert!(dialog.is_open);
}
