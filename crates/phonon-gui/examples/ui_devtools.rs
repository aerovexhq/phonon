#![deny(unsafe_code)]

//! Phonon UI DevTools: Interactive runner, synthetic action driver, and screenshot generator.
//!
//! Run with:
//! `cargo run -p phonon-gui --example ui_devtools --features devtools -- [options]`
//!
//! Options:
//!   --scenario <name>   Scenario to execute: 'drag', 'sim', 'marquee', 'prefs', 'all' (default: 'all')
//!   --interactive       Launch interactive window with DevTools panel open
//!   --out-dir <path>    Output directory for screenshots (default: 'artifacts/screenshots')

use phonon_gui::devtools::{capture_window_screenshot, find_phonon_window_id, ScreenshotInfo, UiScript};
use phonon_gui::PhononApp;
use std::path::PathBuf;

struct DevtoolsRunnerApp {
    app: PhononApp,
    runner: phonon_gui::devtools::ScriptRunner,
    out_dir: PathBuf,
    captured: Vec<ScreenshotInfo>,
    interactive: bool,
    frame_count: u64,
    exit_delay_frames: usize,
}

impl DevtoolsRunnerApp {
    fn new(script: UiScript, out_dir: PathBuf, interactive: bool) -> Self {
        let runner = phonon_gui::devtools::ScriptRunner::new(script);
        let mut app = PhononApp::default();
        app.load_voltage_divider_demo();
        app.devtools_state.visible = interactive;
        app.devtools_state.screenshots_dir = out_dir.clone();

        Self {
            app,
            runner,
            out_dir,
            captured: Vec::new(),
            interactive,
            frame_count: 0,
            exit_delay_frames: 10,
        }
    }
}

impl eframe::App for DevtoolsRunnerApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.frame_count += 1;

        // Run PhononApp update logic
        self.app.update(ui, frame);

        let ctx = ui.ctx();

        // Execute synthetic UI script steps
        if !self.interactive && !self.runner.is_finished {
            // Allow initial layout stabilization for first 5 frames
            if self.frame_count >= 5 {
                if let Some((label, filename)) = self.runner.step(&mut self.app) {
                    let path = self.out_dir.join(&filename);
                    let win_id = find_phonon_window_id();
                    match capture_window_screenshot(win_id.as_deref(), &path, &label) {
                        Ok(info) => {
                            println!(
                                "[Screenshot Captured] {} -> {} ({}x{}, {} bytes)",
                                label,
                                info.file_path.display(),
                                info.width,
                                info.height,
                                info.file_size_bytes
                            );
                            self.captured.push(info);
                        }
                        Err(err) => {
                            eprintln!("[Screenshot Error] Failed to capture {filename}: {err}");
                        }
                    }
                }
            }
            ctx.request_repaint();
        } else if !self.interactive && self.runner.is_finished {
            if self.exit_delay_frames > 0 {
                self.exit_delay_frames -= 1;
                ctx.request_repaint();
            } else {
                println!("\nDevTools execution finished successfully.");
                println!("Total screenshots captured: {}", self.captured.len());
                for info in &self.captured {
                    println!("  - [{}] {}", info.label, info.file_path.display());
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut scenario_name = "all".to_string();
    let mut interactive = false;
    let mut out_dir = PathBuf::from("artifacts/screenshots");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--scenario" => {
                if i + 1 < args.len() {
                    scenario_name = args[i + 1].clone();
                    i += 1;
                }
            }
            "--interactive" => {
                interactive = true;
            }
            "--out-dir" => {
                if i + 1 < args.len() {
                    out_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let _ = std::fs::create_dir_all(&out_dir);

    let script = match scenario_name.to_lowercase().as_str() {
        "drag" => UiScript::component_drag_scenario(),
        "sim" | "transient" => UiScript::transient_simulation_scenario(),
        "marquee" => UiScript::marquee_selection_scenario(),
        "prefs" | "preferences" => UiScript::preferences_dialog_scenario(),
        _ => UiScript::full_test_suite(),
    };

    println!("=======================================================");
    println!("Starting Phonon UI DevTools");
    println!("Scenario: {}", script.name);
    println!("Description: {}", script.description);
    println!("Steps count: {}", script.steps.len());
    println!("Output directory: {}", out_dir.display());
    println!("Interactive mode: {}", interactive);
    println!("=======================================================\n");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Phonon Studio - DevTools")
            .with_inner_size([1280.0, 850.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Phonon Studio - DevTools",
        native_options,
        Box::new(move |_cc| Ok(Box::new(DevtoolsRunnerApp::new(script, out_dir, interactive)))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
