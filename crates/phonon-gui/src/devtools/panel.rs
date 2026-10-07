#![deny(unsafe_code)]

//! Interactive in-app DevTools overlay panel and scenario runner for Phonon Studio.

use super::capture::{capture_window_screenshot, find_phonon_window_id, ScreenshotInfo};
use super::script::{ScriptRunner, UiScript};
use crate::app::PhononApp;
use egui::{Color32, Context, RichText, ScrollArea, Vec2, Window};
use std::path::PathBuf;

/// Interactive runtime state for the DevTools overlay and automation engine.
#[derive(Debug, Clone)]
pub struct DevtoolsState {
    pub enabled: bool,
    pub visible: bool,
    pub runner: Option<ScriptRunner>,
    pub selected_scenario_idx: usize,
    pub auto_capture_screenshots: bool,
    pub screenshots_dir: PathBuf,
    pub captured_screenshots: Vec<ScreenshotInfo>,
    pub frame_counter: u64,
    pub status_message: String,
    pub log_messages: Vec<String>,
}

impl Default for DevtoolsState {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            runner: None,
            selected_scenario_idx: 0,
            auto_capture_screenshots: true,
            screenshots_dir: PathBuf::from("artifacts/screenshots"),
            captured_screenshots: Vec::new(),
            frame_counter: 0,
            status_message: "DevTools initialized. Ready to execute UI tests.".to_string(),
            log_messages: Vec::new(),
        }
    }
}

impl DevtoolsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggles visibility of the floating DevTools window.
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Starts execution of a named UI scenario.
    pub fn start_scenario(&mut self, script: UiScript) {
        let name = script.name.clone();
        self.runner = Some(ScriptRunner::new(script));
        self.status_message = format!("Running scenario: {name}");
        self.log_messages.push(format!("[DevTools] Started scenario: {name}"));
    }

    /// Renders the floating DevTools overlay and steps the active script if running.
    pub fn render(&mut self, ctx: &Context, app: &mut PhononApp) {
        self.frame_counter += 1;

        // Step active script runner if present
        if let Some(runner) = &mut self.runner {
            if let Some((label, filename)) = runner.step(app) {
                if self.auto_capture_screenshots {
                    let path = self.screenshots_dir.join(&filename);
                    let win_id = find_phonon_window_id();
                    match capture_window_screenshot(win_id.as_deref(), &path, &label) {
                        Ok(info) => {
                            self.log_messages.push(format!(
                                "[Screenshot] Saved '{}' to {}",
                                label,
                                info.file_path.display()
                            ));
                            self.captured_screenshots.push(info);
                        }
                        Err(e) => {
                            self.log_messages.push(format!("[Screenshot Error] {e}"));
                        }
                    }
                }
            }

            if runner.is_finished {
                self.status_message = format!("Scenario '{}' completed.", runner.script.name);
                self.log_messages.push(format!(
                    "[DevTools] Completed scenario '{}'. Total screenshots: {}",
                    runner.script.name,
                    self.captured_screenshots.len()
                ));
                self.runner = None;
            }
        }

        if !self.visible {
            return;
        }

        let mut is_open = self.visible;
        Window::new(RichText::new("Phonon UI DevTools").strong().color(Color32::from_rgb(100, 220, 255)))
            .open(&mut is_open)
            .default_size(Vec2::new(380.0, 500.0))
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("UI Inspection & Automation");
                ui.separator();

                // 1. Live Telemetry
                ui.label(RichText::new("Live State Telemetry:").strong());
                ui.horizontal(|ui| {
                    ui.label(format!("Frame: {}", self.frame_counter));
                    ui.separator();
                    ui.label(format!("Active Tool: {:?}", app.selected_tool));
                });
                ui.horizontal(|ui| {
                    ui.label(format!("Components: {}", app.components.len()));
                    ui.label(format!("Wires: {}", app.wires.len()));
                    ui.label(format!("Selected: {}", app.canvas.selected_component_ids.len()));
                });
                ui.horizontal(|ui| {
                    ui.label(format!("Canvas Zoom: {:.2}x", app.canvas.zoom));
                    ui.label(format!("Pan: ({:.1}, {:.1})", app.canvas.pan.x, app.canvas.pan.y));
                });

                ui.separator();

                // 2. Automated Test Scenarios
                ui.label(RichText::new("Automated Test Scenarios:").strong());

                let scenarios = [
                    ("Component Drag & Manhattan Reroute", UiScript::component_drag_scenario()),
                    ("Transient Simulation & Oscilloscope", UiScript::transient_simulation_scenario()),
                    ("Marquee Region Drag Selection", UiScript::marquee_selection_scenario()),
                    ("Preferences Modal Dialog", UiScript::preferences_dialog_scenario()),
                    ("Full Visual Test Suite", UiScript::full_test_suite()),
                ];

                ui.horizontal(|ui| {
                    for (i, (name, _)) in scenarios.iter().enumerate() {
                        if ui.selectable_label(self.selected_scenario_idx == i, *name).clicked() {
                            self.selected_scenario_idx = i;
                        }
                    }
                });

                let (_, active_scenario) = &scenarios[self.selected_scenario_idx];
                ui.label(RichText::new(&active_scenario.description).italics());

                ui.horizontal(|ui| {
                    let is_running = self.runner.is_some();
                    if ui.button(if is_running { "Running..." } else { "Run Selected Scenario" }).clicked() && !is_running {
                        self.start_scenario(active_scenario.clone());
                    }

                    if ui.button("Take Screenshot Now").clicked() {
                        let filename = format!("manual_screenshot_frame_{}.png", self.frame_counter);
                        let path = self.screenshots_dir.join(&filename);
                        let win_id = find_phonon_window_id();
                        match capture_window_screenshot(win_id.as_deref(), &path, "Manual Capture") {
                            Ok(info) => {
                                self.log_messages.push(format!(
                                    "[Manual Screenshot] Saved to {}",
                                    info.file_path.display()
                                ));
                                self.captured_screenshots.push(info);
                            }
                            Err(e) => {
                                self.log_messages.push(format!("[Screenshot Error] {e}"));
                            }
                        }
                    }
                });

                ui.checkbox(&mut self.auto_capture_screenshots, "Auto-capture screenshots during scenario steps");

                ui.separator();

                // 3. Execution Status & Log
                ui.label(RichText::new("Status:").strong());
                ui.label(RichText::new(&self.status_message).color(Color32::from_rgb(140, 240, 140)));

                ui.separator();
                ui.label(RichText::new(format!("Execution Log ({} entries):", self.log_messages.len())).strong());
                ScrollArea::vertical().id_salt("devtools_execution_log").max_height(140.0).show(ui, |ui| {
                    for log in self.log_messages.iter().rev() {
                        ui.label(RichText::new(log).monospace().size(11.0));
                    }
                });

                ui.separator();

                // 4. Captured Screenshots Gallery
                ui.label(RichText::new(format!("Captured Screenshots ({})", self.captured_screenshots.len())).strong());
                ScrollArea::vertical().id_salt("devtools_captured_gallery").max_height(100.0).show(ui, |ui| {
                    for info in &self.captured_screenshots {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&info.label).strong());
                            ui.label(format!("({}x{}, {} KB)", info.width, info.height, info.file_size_bytes / 1024));
                        });
                    }
                });
            });

        self.visible = is_open;
    }
}
