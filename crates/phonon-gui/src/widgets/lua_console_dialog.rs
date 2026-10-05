#![deny(unsafe_code)]

//! Interactive Lua Testbench Console Dialog for Phonon Visual Studio.
//!
//! Provides a script editor with line numbers, preset testbench scripts,
//! execution controls, output console telemetry, colored assertions, and
//! interactive runtime permission prompts.

use egui::{Color32, RichText, ScrollArea, TextEdit, Ui, Vec2};

use crate::scripting::lua_engine::LuaEngine;
use crate::scripting::permissions::{PermissionKind, PermissionState};

/// Available script preset templates for automated testbenches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScriptPreset {
    #[default]
    DcVoltageDivider,
    TransientSineWave,
    MathematicalWaveform,
    AutomatedReport,
}

impl ScriptPreset {
    /// Returns human-readable preset name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::DcVoltageDivider => "DC Voltage Divider Operating Point Check",
            Self::TransientSineWave => "Transient Sine Wave Attenuation & Ripple Check",
            Self::MathematicalWaveform => "Mathematical Waveform Expression Graphing",
            Self::AutomatedReport => "Automated Report Export",
        }
    }

    /// Returns sample Lua script source for this preset.
    pub fn script_source(&self) -> &'static str {
        match self {
            Self::DcVoltageDivider => {
                "-- Preset 1: DC Voltage Divider Operating Point Check\n\
                 phonon.log(\"Running DC operating point verification...\")\n\
                 local res = phonon.run_dc()\n\
                 local vin = phonon.get_voltage(\"VIN\")\n\
                 local vout = phonon.get_voltage(\"VOUT\")\n\
                 print(\"Node VIN: \" .. tostring(vin) .. \" V\")\n\
                 print(\"Node VOUT: \" .. tostring(vout) .. \" V\")\n\
                 phonon.assert_range(vin, 4.9, 5.1, \"VIN within 5V supply rail\")\n\
                 phonon.assert_range(vout, 2.4, 2.6, \"VOUT half-rail divider check\")\n\
                 print(\"DC verification completed successfully.\")\n"
            }
            Self::TransientSineWave => {
                "-- Preset 2: Transient Sine Wave Attenuation & Ripple Check\n\
                 phonon.log(\"Running transient sine wave simulation...\")\n\
                 local sim = phonon.run_transient(0.003, 0.00001)\n\
                 local vin = phonon.get_voltage(\"VIN\")\n\
                 local vout = phonon.get_voltage(\"VOUT\")\n\
                 phonon.assert_range(vout, 0.0, 5.0, \"VOUT within dynamic range\")\n\
                 local atten = vout / (vin + 1e-6)\n\
                 phonon.assert_range(atten, 0.0, 1.0, \"Attenuation factor <= 1.0\")\n\
                 print(\"Transient attenuation checks passed.\")\n"
            }
            Self::MathematicalWaveform => {
                "-- Preset 3: Mathematical Waveform Expression Graphing\n\
                 phonon.log(\"Plotting synthetic testbench waveforms...\")\n\
                 phonon.plot_expression(\"Synthetic Carrier 1kHz\", \"2.5 + 2.5 * sin(2 * pi * 1000 * t)\")\n\
                 phonon.plot_expression(\"Modulated Waveform\", \"1.25 + 1.25 * sin(2 * pi * 2000 * t - 0.5)\")\n\
                 phonon.plot_expression(\"Harmonic Ripple\", \"0.5 * cos(2 * pi * 5000 * t)\")\n\
                 print(\"Registered 3 expression waveforms into virtual oscilloscope.\")\n"
            }
            Self::AutomatedReport => {
                "-- Preset 4: Automated Report Export\n\
                 phonon.log(\"Exporting automated testbench report...\")\n\
                 local vin = phonon.get_voltage(\"VIN\")\n\
                 local vout = phonon.get_voltage(\"VOUT\")\n\
                 local report = \"=== PHONON CAD AUTOMATED REPORT ===\\n\" ..\n\
                                \"VIN = \" .. tostring(vin) .. \" V\\n\" ..\n\
                                \"VOUT = \" .. tostring(vout) .. \" V\\n\" ..\n\
                                \"Status: PASSED\\n\"\n\
                 phonon.write_file(\"testbench_report.txt\", report)\n\
                 local verified = phonon.read_file(\"testbench_report.txt\")\n\
                 print(\"Report written and verified successfully.\")\n\
                 phonon.assert_eq(#verified > 0 and 1 or 0, 1, 0, \"Report file not empty\")\n"
            }
        }
    }
}

/// Interactive Lua Testbench Console modal dialog state.
pub struct LuaConsoleDialog {
    pub is_open: bool,
    pub script_text: String,
    pub selected_preset: ScriptPreset,
    pub engine: LuaEngine,
    pub show_line_numbers: bool,
    pub status_message: String,
    pub is_running: bool,
    pub run_requested: bool,
}

impl Default for LuaConsoleDialog {
    fn default() -> Self {
        let preset = ScriptPreset::DcVoltageDivider;
        let mut engine = LuaEngine::new();
        // Set standard divider circuit values for default checks
        engine.set_voltage("VIN", 5.0);
        engine.set_voltage("VOUT", 2.5);
        Self {
            is_open: false,
            script_text: preset.script_source().to_string(),
            selected_preset: preset,
            engine,
            show_line_numbers: true,
            status_message: "Ready. Select a testbench preset or write custom Lua scripts.".to_string(),
            is_running: false,
            run_requested: false,
        }
    }
}

impl LuaConsoleDialog {
    /// Creates a new closed LuaConsoleDialog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a specified script preset into the editor.
    pub fn load_preset(&mut self, preset: ScriptPreset) {
        self.selected_preset = preset;
        self.script_text = preset.script_source().to_string();
        self.status_message = format!("Loaded preset: {}", preset.display_name());
    }

    /// Executes the current script in the editor.
    pub fn run_script(&mut self) {
        self.is_running = true;
        match self.engine.run_script(&self.script_text) {
            Ok(_) => {
                let passed = self.engine.passed_assertions();
                let failed = self.engine.failed_assertions();
                let total = self.engine.total_assertions();
                self.status_message = format!(
                    "Execution finished in {:.2} ms. Total tests: {}, Passed: {}, Failed: {}",
                    self.engine.execution_time_ms, total, passed, failed
                );
            }
            Err(err) => {
                self.status_message = format!("Script execution stopped with error: {}", err);
            }
        }
        self.is_running = false;
    }

    /// Clears the console output and telemetry.
    pub fn clear_console(&mut self) {
        self.engine.clear_log();
        self.status_message = "Console output cleared.".to_string();
    }

    /// Checks if a permission prompt is pending.
    pub fn has_pending_permission(&self) -> bool {
        self.engine.permissions.pending_request.is_some()
    }

    /// Resolves pending permission decision.
    pub fn resolve_permission(&mut self, state: PermissionState) {
        if let Some(kind) = self.engine.permissions.pending_request.clone() {
            match state {
                PermissionState::AllowedSession => {
                    self.engine.permissions.allow_session(kind);
                    self.run_script();
                }
                PermissionState::AllowedAlways => {
                    self.engine.permissions.allow_permanent(kind);
                    self.run_script();
                }
                PermissionState::Denied => {
                    self.engine.permissions.deny(kind, false);
                    self.status_message = "Operation permission denied by user.".to_string();
                }
                PermissionState::PendingPrompt => {}
            }
        }
    }

    /// Renders modal window.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        egui::Window::new("Lua Testbench Console")
            .open(&mut open)
            .default_size(Vec2::new(780.0, 560.0))
            .min_size(Vec2::new(550.0, 400.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = open;
    }

    /// Renders inner dialog controls and panels.
    pub fn render_content(&mut self, ui: &mut Ui) {
        // 1. Telemetry Header Bar
        ui.horizontal(|ui| {
            let total = self.engine.total_assertions();
            let passed = self.engine.passed_assertions();
            let failed = self.engine.failed_assertions();
            let runtime = self.engine.execution_time_ms;

            ui.label(
                RichText::new(format!("Total Tests: {}", total))
                    .strong()
                    .color(Color32::from_rgb(200, 210, 225)),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("Passed: {}", passed))
                    .strong()
                    .color(Color32::from_rgb(80, 220, 100)),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("Failed: {}", failed))
                    .strong()
                    .color(if failed > 0 {
                        Color32::from_rgb(255, 90, 90)
                    } else {
                        Color32::from_rgb(140, 150, 160)
                    }),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("Runtime: {:.2} ms", runtime))
                    .color(Color32::from_rgb(100, 200, 255)),
            );
        });

        ui.add_space(6.0);
        ui.separator();

        // 2. Permission Request Prompt Banner
        if let Some(req) = self.engine.permissions.pending_request.clone() {
            ui.add_space(4.0);
            egui::Frame::new()
                .fill(Color32::from_rgb(45, 38, 20))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(220, 160, 40)))
                .corner_radius(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let action = match req {
                            PermissionKind::FileRead(_) => "read file",
                            PermissionKind::FileWrite(_) => "write file",
                        };
                        ui.label(
                            RichText::new(format!(
                                "[SECURITY PROMPT] Script requests permission to {} at '{}'",
                                action,
                                req.path()
                            ))
                            .color(Color32::from_rgb(255, 210, 100))
                            .strong(),
                        );
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("Allow Once (Session)").clicked() {
                            self.resolve_permission(PermissionState::AllowedSession);
                        }
                        if ui.button("Always Allow (Permanent)").clicked() {
                            self.resolve_permission(PermissionState::AllowedAlways);
                        }
                        if ui.button("Deny").clicked() {
                            self.resolve_permission(PermissionState::Denied);
                        }
                    });
                });
            ui.add_space(4.0);
        }

        // 3. Script Controls & Preset Selection
        ui.horizontal(|ui| {
            if ui.button("Run Script").clicked() {
                self.run_script();
            }
            if ui.button("Clear Output").clicked() {
                self.clear_console();
            }
            if ui.button("Reset VM").clicked() {
                self.engine.reset();
                self.status_message = "Lua VM state reset.".to_string();
            }

            ui.separator();
            ui.label("Preset:");
            egui::ComboBox::from_id_salt("lua_preset_combo")
                .selected_text(self.selected_preset.display_name())
                .show_ui(ui, |ui| {
                    let presets = [
                        ScriptPreset::DcVoltageDivider,
                        ScriptPreset::TransientSineWave,
                        ScriptPreset::MathematicalWaveform,
                        ScriptPreset::AutomatedReport,
                    ];
                    for &p in &presets {
                        if ui
                            .selectable_value(&mut self.selected_preset, p, p.display_name())
                            .clicked()
                        {
                            self.load_preset(p);
                        }
                    }
                });

            ui.checkbox(&mut self.show_line_numbers, "Line Numbers");
        });

        ui.add_space(4.0);

        // 4. Editor & Output Console split
        let total_height = ui.available_height() - 28.0;
        let editor_height = (total_height * 0.55).max(120.0);
        let console_height = (total_height - editor_height - 12.0).max(80.0);

        // Script Editor Panel
        ui.group(|ui| {
            ui.label(RichText::new("Lua Script Editor").strong());
            ScrollArea::vertical()
                .id_salt("lua_editor_scroll")
                .max_height(editor_height)
                .show(ui, |ui| {
                    if self.show_line_numbers {
                        ui.horizontal(|ui| {
                            let line_count = self.script_text.lines().count().max(1);
                            let mut line_num_str = String::new();
                            for l in 1..=line_count {
                                line_num_str.push_str(&format!("{:3}\n", l));
                            }
                            ui.label(
                                RichText::new(line_num_str)
                                    .monospace()
                                    .color(Color32::from_rgb(110, 120, 140)),
                            );
                            ui.add(
                                TextEdit::multiline(&mut self.script_text)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(ui.available_width())
                                    .lock_focus(true),
                            );
                        });
                    } else {
                        ui.add(
                            TextEdit::multiline(&mut self.script_text)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .lock_focus(true),
                        );
                    }
                });
        });

        ui.add_space(4.0);

        // Output Console Panel
        ui.group(|ui| {
            ui.label(RichText::new("Testbench Output Console").strong());
            ScrollArea::vertical()
                .id_salt("lua_console_scroll")
                .max_height(console_height)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if self.engine.output_log.is_empty() && self.engine.error_trace.is_none() {
                        ui.label(
                            RichText::new("Output console empty. Run a script to view logs.")
                                .italics()
                                .color(Color32::from_rgb(120, 130, 145)),
                        );
                    } else {
                        for line in &self.engine.output_log {
                            let text = RichText::new(line).monospace();
                            if line.contains("[PASS]") {
                                ui.label(text.color(Color32::from_rgb(80, 220, 100)));
                            } else if line.contains("[FAIL]") {
                                ui.label(text.color(Color32::from_rgb(255, 90, 90)));
                            } else if line.contains("[PHONON]") {
                                ui.label(text.color(Color32::from_rgb(100, 200, 255)));
                            } else if line.contains("[ERROR]") {
                                ui.label(text.color(Color32::from_rgb(255, 120, 120)));
                            } else {
                                ui.label(text.color(Color32::from_rgb(215, 225, 235)));
                            }
                        }
                    }
                });
        });

        // 5. Status bar footer
        ui.add_space(4.0);
        ui.label(
            RichText::new(&self.status_message)
                .size(11.0)
                .color(Color32::from_rgb(150, 165, 185)),
        );
    }
}
