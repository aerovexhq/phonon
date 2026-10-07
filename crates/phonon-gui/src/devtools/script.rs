#![deny(unsafe_code)]

//! Synthetic UI action scripting and scenario execution engine for Phonon UI DevTools.

use crate::app::{PhononApp, ToolMode};
use crate::widgets::confirmation_modal::DemoCircuitKind;
use egui::{Pos2, Vec2};

/// Individual synthetic UI action step.
#[derive(Debug, Clone, PartialEq)]
pub enum UiAction {
    /// Selects a specific component by ID.
    SelectComponent(usize),
    /// Clears any active selection.
    ClearSelection,
    /// Initiates a drag operation on a component.
    StartDrag(usize),
    /// Drags the active selection to the specified world coordinates.
    DragTo(Pos2),
    /// Drags the active selection by the specified world delta.
    DragDelta(Vec2),
    /// Finishes the drag operation, committing changes to history.
    FinishDrag,
    /// Begins a rubberband marquee selection at world pos.
    StartMarquee(Pos2),
    /// Updates the marquee selection bounding area.
    UpdateMarquee(Pos2),
    /// Finalizes the marquee selection.
    FinishMarquee,
    /// Switches the active CAD tool.
    SetTool(ToolMode),
    /// Triggers the transient circuit simulation and updates oscilloscope traces.
    RunTransient,
    /// Triggers DC operating point simulation.
    RunDc,
    /// Opens the categorized Preferences modal dialog.
    OpenPreferences,
    /// Closes the Preferences modal dialog.
    ClosePreferences,
    /// Toggles or sets the in-app DevTools panel visibility.
    SetDevtoolsPanelVisible(bool),
    /// Loads a built-in demo circuit.
    LoadDemo(DemoCircuitKind),
    /// Requests a named screenshot capture.
    TakeScreenshot {
        label: String,
        filename: String,
    },
    /// Waits for a specified number of render frames.
    WaitFrames(usize),
}

/// A named sequence of UI actions representing an automated test scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct UiScript {
    pub name: String,
    pub description: String,
    pub steps: Vec<UiAction>,
}

impl UiScript {
    pub fn new(name: &str, description: &str, steps: Vec<UiAction>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            steps,
        }
    }

    /// Scenario 1: Select and drag component across canvas with live wire rubberbanding.
    pub fn component_drag_scenario() -> Self {
        Self::new(
            "Component Drag & Manhattan Reroute",
            "Selects resistor R1, drags it across the canvas, and verifies wire rubberbanding and Manhattan rerouting",
            vec![
                UiAction::TakeScreenshot {
                    label: "Initial Voltage Divider Schematic Canvas".to_string(),
                    filename: "01_initial_canvas.png".to_string(),
                },
                UiAction::SelectComponent(1), // Resistor R1
                UiAction::WaitFrames(2),
                UiAction::TakeScreenshot {
                    label: "Component Selected with Halo & Properties Inspector".to_string(),
                    filename: "02_component_selected.png".to_string(),
                },
                UiAction::StartDrag(1),
                UiAction::DragDelta(Vec2::new(80.0, 60.0)),
                UiAction::WaitFrames(2),
                UiAction::TakeScreenshot {
                    label: "Component Drag In Progress with Wire Rubberbanding".to_string(),
                    filename: "03_component_dragged.png".to_string(),
                },
                UiAction::FinishDrag,
                UiAction::WaitFrames(3),
                UiAction::TakeScreenshot {
                    label: "Drag Completed with Manhattan Wire Routing Updated".to_string(),
                    filename: "04_drag_completed.png".to_string(),
                },
            ],
        )
    }

    /// Scenario 2: Run transient simulation and inspect dual-domain virtual oscilloscope.
    pub fn transient_simulation_scenario() -> Self {
        Self::new(
            "Transient Simulation & Oscilloscope Traces",
            "Executes transient solver and displays multi-trace waveforms on the virtual oscilloscope",
            vec![
                UiAction::RunTransient,
                UiAction::WaitFrames(3),
                UiAction::TakeScreenshot {
                    label: "Transient Simulation Output on Dual-Domain Virtual Oscilloscope".to_string(),
                    filename: "05_transient_oscilloscope.png".to_string(),
                },
            ],
        )
    }

    /// Scenario 3: Marquee region drag selection across multiple components.
    pub fn marquee_selection_scenario() -> Self {
        Self::new(
            "Marquee Region Drag Selection",
            "Drags a marquee bounding box across components to perform multi-item selection",
            vec![
                UiAction::ClearSelection,
                UiAction::SetTool(ToolMode::Select),
                UiAction::StartMarquee(Pos2::new(100.0, 50.0)),
                UiAction::UpdateMarquee(Pos2::new(350.0, 320.0)),
                UiAction::WaitFrames(2),
                UiAction::TakeScreenshot {
                    label: "Marquee Selection Bounding Box Active".to_string(),
                    filename: "06_marquee_selection_active.png".to_string(),
                },
                UiAction::FinishMarquee,
                UiAction::WaitFrames(2),
                UiAction::TakeScreenshot {
                    label: "Multi-Selection Halos on Selected Components".to_string(),
                    filename: "07_marquee_multi_selected.png".to_string(),
                },
            ],
        )
    }

    /// Scenario 4: Open Preferences modal dialog.
    pub fn preferences_dialog_scenario() -> Self {
        Self::new(
            "Preferences Modal Dialog Inspection",
            "Opens the categorized Preferences dialog displaying Themes, Keybinds, and Canvas settings",
            vec![
                UiAction::OpenPreferences,
                UiAction::WaitFrames(3),
                UiAction::TakeScreenshot {
                    label: "Categorized Preferences Modal Dialog".to_string(),
                    filename: "08_preferences_dialog.png".to_string(),
                },
                UiAction::ClosePreferences,
                UiAction::WaitFrames(2),
            ],
        )
    }

    /// Full comprehensive test suite executing all visual UI scenarios.
    pub fn full_test_suite() -> Self {
        let mut steps = Vec::new();
        steps.extend(Self::component_drag_scenario().steps);
        steps.extend(Self::transient_simulation_scenario().steps);
        steps.extend(Self::marquee_selection_scenario().steps);
        steps.extend(Self::preferences_dialog_scenario().steps);
        steps.extend(vec![
            UiAction::SetDevtoolsPanelVisible(true),
            UiAction::WaitFrames(3),
            UiAction::TakeScreenshot {
                label: "Phonon UI DevTools In-App Inspector Panel Active".to_string(),
                filename: "09_devtools_inspector_panel.png".to_string(),
            },
            UiAction::SetDevtoolsPanelVisible(false),
            UiAction::WaitFrames(2),
        ]);

        Self::new(
            "Full Phonon UI DevTools Visual Suite",
            "Executes complete visual automated test suite capturing screenshots across all interaction types",
            steps,
        )
    }
}

/// Runtime executor for a `UiScript` driving `PhononApp`.
#[derive(Debug, Clone)]
pub struct ScriptRunner {
    pub script: UiScript,
    pub step_index: usize,
    pub wait_counter: usize,
    pub is_finished: bool,
    pub pending_screenshot: Option<(String, String)>,
    pub execution_log: Vec<String>,
}

impl ScriptRunner {
    pub fn new(script: UiScript) -> Self {
        Self {
            script,
            step_index: 0,
            wait_counter: 0,
            is_finished: false,
            pending_screenshot: None,
            execution_log: Vec::new(),
        }
    }

    /// Advances the script by one render frame.
    /// Returns `Some((label, filename))` if a screenshot should be taken on this frame.
    pub fn step(&mut self, app: &mut PhononApp) -> Option<(String, String)> {
        if self.is_finished {
            return None;
        }

        if self.wait_counter > 0 {
            self.wait_counter -= 1;
            return None;
        }

        if self.step_index >= self.script.steps.len() {
            self.is_finished = true;
            self.execution_log.push(format!("Script '{}' completed successfully.", self.script.name));
            return None;
        }

        let action = self.script.steps[self.step_index].clone();
        self.step_index += 1;

        match action {
            UiAction::SelectComponent(id) => {
                app.select_component(id, false);
                self.execution_log.push(format!("Selected component ID {id}"));
            }
            UiAction::ClearSelection => {
                app.clear_selection();
                self.execution_log.push("Cleared canvas selection".to_string());
            }
            UiAction::StartDrag(id) => {
                app.devtools_start_drag(id);
                self.execution_log.push(format!("Started drag on component ID {id}"));
            }
            UiAction::DragTo(pos) => {
                app.devtools_drag_to(pos);
                self.execution_log.push(format!("Dragged selection to pos {pos:?}"));
            }
            UiAction::DragDelta(delta) => {
                app.devtools_drag_delta(delta);
                self.execution_log.push(format!("Dragged selection by delta {delta:?}"));
            }
            UiAction::FinishDrag => {
                app.devtools_finish_drag();
                self.execution_log.push("Finished drag operation and committed routing".to_string());
            }
            UiAction::StartMarquee(pos) => {
                app.devtools_start_marquee(pos);
                self.execution_log.push(format!("Started marquee selection at {pos:?}"));
            }
            UiAction::UpdateMarquee(pos) => {
                app.devtools_update_marquee(pos);
                self.execution_log.push(format!("Updated marquee selection to {pos:?}"));
            }
            UiAction::FinishMarquee => {
                app.devtools_finish_marquee();
                self.execution_log.push("Finished marquee selection".to_string());
            }
            UiAction::SetTool(tool) => {
                app.selected_tool = tool.clone();
                self.execution_log.push(format!("Switched tool to {tool:?}"));
            }
            UiAction::RunTransient => {
                app.run_transient_demo();
                self.execution_log.push("Ran transient demo simulation".to_string());
            }
            UiAction::RunDc => {
                app.run_dc_op();
                self.execution_log.push("Ran DC operating point simulation".to_string());
            }
            UiAction::OpenPreferences => {
                app.preferences_dialog.open(None);
                self.execution_log.push("Opened Preferences modal dialog".to_string());
            }
            UiAction::ClosePreferences => {
                app.preferences_dialog.close();
                self.execution_log.push("Closed Preferences modal dialog".to_string());
            }
            UiAction::SetDevtoolsPanelVisible(visible) => {
                app.devtools_state.visible = visible;
                self.execution_log.push(format!("Set DevTools panel visibility to {visible}"));
            }
            UiAction::LoadDemo(kind) => {
                match kind {
                    DemoCircuitKind::VoltageDivider => app.load_voltage_divider_demo(),
                    DemoCircuitKind::BjtAmplifier => app.load_bjt_amplifier_demo(),
                    DemoCircuitKind::CmosInverter => app.load_cmos_inverter_demo(),
                    DemoCircuitKind::DiodeClipper => app.load_diode_clipper_demo(),
                    DemoCircuitKind::NmosSwitch => app.load_nmos_switch_demo(),
                }
                self.execution_log.push(format!("Loaded demo circuit {kind:?}"));
            }
            UiAction::TakeScreenshot { label, filename } => {
                self.execution_log.push(format!("Requested screenshot: {filename} ({label})"));
                return Some((label, filename));
            }
            UiAction::WaitFrames(n) => {
                self.wait_counter = n;
            }
        }

        None
    }
}
