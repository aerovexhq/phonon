#![deny(unsafe_code)]

//! Verification test suite for Phase 355: Interactive Lua Testbench Console GUI Dialog.
//!
//! Tests dialog initialization, preset switching, script execution from UI state,
//! telemetry header badges, and headless egui context rendering.

use phonon_gui::widgets::lua_console_dialog::{LuaConsoleDialog, ScriptPreset};

#[test]
fn test_lua_console_dialog_initialization_defaults() {
    let dialog = LuaConsoleDialog::new();

    assert!(!dialog.is_open, "Dialog should be closed by default");
    assert_eq!(dialog.selected_preset, ScriptPreset::DcVoltageDivider);
    assert!(!dialog.script_text.is_empty(), "Default script must be pre-populated");
    assert!(dialog.show_line_numbers);
    assert!(!dialog.is_running);
    assert_eq!(dialog.engine.total_assertions(), 0);
}

#[test]
fn test_default_preset_execution_and_telemetry() {
    let mut dialog = LuaConsoleDialog::new();

    // Default preset checks VIN and VOUT operating point (pre-configured to 5.0 and 2.5)
    dialog.run_script();

    assert_eq!(dialog.engine.total_assertions(), 2);
    assert_eq!(dialog.engine.passed_assertions(), 2);
    assert_eq!(dialog.engine.failed_assertions(), 0);
    assert!(dialog.status_message.contains("Passed: 2"));
    assert!(!dialog.engine.output_log.is_empty());
}

#[test]
fn test_preset_switching_and_loading() {
    let mut dialog = LuaConsoleDialog::new();

    // 1. Switch to Transient Sine Wave
    dialog.load_preset(ScriptPreset::TransientSineWave);
    assert_eq!(dialog.selected_preset, ScriptPreset::TransientSineWave);
    assert!(dialog.script_text.contains("run_transient"));
    dialog.run_script();
    assert!(dialog.engine.passed_assertions() >= 1);

    // 2. Switch to Mathematical Waveform Expression Graphing
    dialog.load_preset(ScriptPreset::MathematicalWaveform);
    assert_eq!(dialog.selected_preset, ScriptPreset::MathematicalWaveform);
    assert!(dialog.script_text.contains("plot_expression"));
    dialog.run_script();
    assert_eq!(dialog.engine.generated_traces.len(), 3);

    // 3. Switch to Automated Report Export
    dialog.load_preset(ScriptPreset::AutomatedReport);
    assert_eq!(dialog.selected_preset, ScriptPreset::AutomatedReport);
    assert!(dialog.script_text.contains("write_file"));

    // Running report export will request permission
    dialog.run_script();
    assert!(dialog.has_pending_permission());
}

#[test]
fn test_console_clear_and_reset() {
    let mut dialog = LuaConsoleDialog::new();
    dialog.run_script();
    assert!(!dialog.engine.output_log.is_empty());

    dialog.clear_console();
    assert!(dialog.engine.output_log.is_empty());
    assert_eq!(dialog.engine.total_assertions(), 0);
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = LuaConsoleDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
}
