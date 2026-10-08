#![deny(unsafe_code)]

//! Synthetic UI action scripting and scenario execution engine for Phonon UI DevTools.

use crate::app::{PhononApp, ToolMode};
use crate::widgets::confirmation_modal::{DemoCircuitKind, PendingAction};
use crate::widgets::preferences_dialog::PreferencesTab;
use crate::widgets::project_dialog::ProjectDialogMode;
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
    /// Switches active Preferences dialog tab.
    SwitchPreferencesTab(PreferencesTab),
    /// Closes the Preferences modal dialog.
    ClosePreferences,
    /// Triggers the Unsaved Changes confirmation modal popup.
    TriggerUnsavedChangesPopup,
    /// Closes the confirmation modal popup.
    CloseConfirmationModal,
    /// Opens the Project Dialog in the given mode.
    OpenProjectDialog(ProjectDialogMode),
    /// Closes the Project Dialog.
    CloseProjectDialog,
    /// Opens the Spotlight Command Palette modal.
    OpenCommandPalette,
    /// Closes the Command Palette.
    CloseCommandPalette,
    /// Toggles or sets the in-app DevTools panel visibility.
    SetDevtoolsPanelVisible(bool),
    /// Opens the Chiral Heat Transistor & Thermal Diode CAD dialog.
    OpenChiralHeatTransistorDialog,
    /// Closes the Chiral Heat Transistor & Thermal Diode CAD dialog.
    CloseChiralHeatTransistorDialog,
    /// Opens the Topological Corner Kerr Microcomb & Dissipative Soliton CAD dialog.
    OpenCornerKerrMicrocombDialog,
    /// Closes the Topological Corner Kerr Microcomb & Dissipative Soliton CAD dialog.
    CloseCornerKerrMicrocombDialog,
    /// Opens the Topological Acoustic Superconducting Nanowire Single-Phonon Detector (SNSPD) & Quantum Transceiver CAD dialog.
    OpenAcousticSnspdDialog,
    /// Closes the Topological Acoustic Superconducting Nanowire Single-Phonon Detector (SNSPD) & Quantum Transceiver CAD dialog.
    CloseAcousticSnspdDialog,
    /// Opens the Floquet Chiral Magnon-Phonon Entanglement Router & CV-QKD CAD dialog.
    OpenFloquetCvQkdDialog,
    /// Closes the Floquet Chiral Magnon-Phonon Entanglement Router & CV-QKD CAD dialog.
    CloseFloquetCvQkdDialog,
    /// Loads a built-in demo circuit.
    LoadDemo(DemoCircuitKind),
    /// Expands or collapses a category in the component palette drawer.
    ExpandCategory(crate::schematic::categories::ComponentCategory),
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

    /// Scenario 4: Open Preferences modal dialog across tabs.
    pub fn preferences_dialog_scenario() -> Self {
        Self::new(
            "Preferences Modal Dialog Inspection",
            "Opens the categorized Preferences dialog displaying General, Canvas, Themes, Keybinds, History, and Thermal settings",
            vec![
                UiAction::OpenPreferences,
                UiAction::SwitchPreferencesTab(PreferencesTab::General),
                UiAction::WaitFrames(3),
                UiAction::TakeScreenshot {
                    label: "Preferences Dialog - General Settings (Vertical Layout)".to_string(),
                    filename: "08_preferences_general_tab.png".to_string(),
                },
                UiAction::SwitchPreferencesTab(PreferencesTab::CanvasLayout),
                UiAction::WaitFrames(3),
                UiAction::TakeScreenshot {
                    label: "Preferences Dialog - Canvas & Layout Settings".to_string(),
                    filename: "09_preferences_canvas_tab.png".to_string(),
                },
                UiAction::SwitchPreferencesTab(PreferencesTab::ThemeColors),
                UiAction::WaitFrames(3),
                UiAction::TakeScreenshot {
                    label: "Preferences Dialog - Theme & Color Swatches".to_string(),
                    filename: "10_preferences_themes_tab.png".to_string(),
                },
                UiAction::SwitchPreferencesTab(PreferencesTab::Keybindings),
                UiAction::WaitFrames(3),
                UiAction::TakeScreenshot {
                    label: "Preferences Dialog - Keybindings Registry".to_string(),
                    filename: "11_preferences_keybinds_tab.png".to_string(),
                },
                UiAction::SwitchPreferencesTab(PreferencesTab::HistoryUndo),
                UiAction::WaitFrames(3),
                UiAction::TakeScreenshot {
                    label: "Preferences Dialog - History & Undo Limits".to_string(),
                    filename: "12_preferences_history_tab.png".to_string(),
                },
                UiAction::SwitchPreferencesTab(PreferencesTab::ThermalPhysics),
                UiAction::WaitFrames(3),
                UiAction::TakeScreenshot {
                    label: "Preferences Dialog - Thermal & Physics Settings".to_string(),
                    filename: "13_preferences_thermal_tab.png".to_string(),
                },
                UiAction::ClosePreferences,
                UiAction::WaitFrames(2),
            ],
        )
    }

    /// Scenario 5: Unsaved Changes confirmation modal popup.
    pub fn unsaved_changes_scenario() -> Self {
        Self::new(
            "Unsaved Changes Confirmation Modal Inspection",
            "Triggers the Unsaved Changes modal with backdrop positioned behind the window",
            vec![
                UiAction::TriggerUnsavedChangesPopup,
                UiAction::WaitFrames(6),
                UiAction::TakeScreenshot {
                    label: "Unsaved Changes Confirmation Modal - Backdrop Behind Window".to_string(),
                    filename: "14_unsaved_changes_popup.png".to_string(),
                },
                UiAction::CloseConfirmationModal,
                UiAction::WaitFrames(2),
            ],
        )
    }

    /// Scenario 6: Project Manager modal dialog.
    pub fn project_dialog_scenario() -> Self {
        Self::new(
            "Project Manager Modal Dialog Inspection",
            "Opens the Save As project manager dialog with live destination preview",
            vec![
                UiAction::OpenProjectDialog(ProjectDialogMode::SaveAs),
                UiAction::WaitFrames(6),
                UiAction::TakeScreenshot {
                    label: "Project Manager Save-As Modal Dialog".to_string(),
                    filename: "15_project_save_as_dialog.png".to_string(),
                },
                UiAction::CloseProjectDialog,
                UiAction::WaitFrames(2),
            ],
        )
    }

    /// Scenario 7: Spotlight Command Palette (Ctrl+K).
    pub fn command_palette_scenario() -> Self {
        Self::new(
            "Spotlight Command Palette Inspection",
            "Opens the Command Palette overlay with action registry search",
            vec![
                UiAction::OpenCommandPalette,
                UiAction::WaitFrames(6),
                UiAction::TakeScreenshot {
                    label: "Spotlight Command Palette Modal (Ctrl+K)".to_string(),
                    filename: "16_command_palette.png".to_string(),
                },
                UiAction::CloseCommandPalette,
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
        steps.extend(Self::unsaved_changes_scenario().steps);
        steps.extend(Self::project_dialog_scenario().steps);
        steps.extend(Self::command_palette_scenario().steps);
        steps.extend(Self::half_adder_and_logic_gates_scenario().steps);
        steps.extend(Self::quantum_metamaterial_and_heat_transistor_scenario().steps);
        steps.extend(Self::corner_kerr_microcomb_scenario().steps);
        steps.extend(Self::acoustic_snspd_scenario().steps);
        steps.extend(Self::floquet_cv_qkd_scenario().steps);
        steps.extend(vec![
            UiAction::SetDevtoolsPanelVisible(true),
            UiAction::WaitFrames(3),
            UiAction::TakeScreenshot {
                label: "Phonon UI DevTools In-App Inspector Panel Active".to_string(),
                filename: "17_devtools_inspector_panel.png".to_string(),
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

    /// Scenario 8: Basic Logic Gates and Half Adder circuit verification and interaction.
    pub fn half_adder_and_logic_gates_scenario() -> Self {
        Self::new(
            "Basic Logic Gates and Half Adder Verification",
            "Loads the 8-gate basic logic bench and the dual-tier Half Adder circuit, runs transient simulation, and exercises interactive dragging",
            vec![
                // Step 1: Load Basic Logic Gates Demo
                UiAction::LoadDemo(DemoCircuitKind::BasicGates),
                UiAction::WaitFrames(6),
                UiAction::TakeScreenshot {
                    label: "Basic Logic Gates Verification Bench (8 Gate Primitives)".to_string(),
                    filename: "18_basic_gates_and_half_adder.png".to_string(),
                },

                // Step 2: Load Half Adder Demo (Discrete XOR/AND + Macro HA block)
                UiAction::LoadDemo(DemoCircuitKind::HalfAdder),
                UiAction::WaitFrames(6),

                // Step 3: Run Transient simulation to drive waveforms into oscilloscope
                UiAction::RunTransient,
                UiAction::WaitFrames(8),
                UiAction::TakeScreenshot {
                    label: "Half Adder Digital Transient Oscilloscope Traces".to_string(),
                    filename: "19_half_adder_oscilloscope.png".to_string(),
                },

                // Step 4: Interact with circuit - select XOR gate, drag and reroute
                UiAction::SelectComponent(4), // UXOR1
                UiAction::WaitFrames(2),
                UiAction::StartDrag(4),
                UiAction::DragDelta(Vec2::new(40.0, 0.0)),
                UiAction::WaitFrames(2),
                UiAction::FinishDrag,
                UiAction::WaitFrames(4),
                UiAction::TakeScreenshot {
                    label: "Half Adder Component Dragged & Manhattan Wires Dynamically Rerouted".to_string(),
                    filename: "20_half_adder_interaction.png".to_string(),
                },

                // Step 5: Enriched Component Library inspection
                UiAction::ClearSelection,
                UiAction::ExpandCategory(crate::schematic::categories::ComponentCategory::LogicGates),
                UiAction::ExpandCategory(crate::schematic::categories::ComponentCategory::IntegratedCircuits),
                UiAction::WaitFrames(4),
                UiAction::TakeScreenshot {
                    label: "Enriched Multi-Tier Component Library (58 Components across 11 Categories)".to_string(),
                    filename: "21_enriched_component_library.png".to_string(),
                },
            ],
        )
    }

    /// Scenario 9: Topological Quantum Metamaterials Demo & Chiral Magnon-Phonon Heat Transistor Dialog.
    pub fn quantum_metamaterial_and_heat_transistor_scenario() -> Self {
        Self::new(
            "Topological Quantum Metamaterials & Chiral Heat Transistor Verification",
            "Loads the Topological Quantum Metamaterials circuit (SAW IDT, Parafermionic Cavity, Skyrmion Router, Majorana Junction), runs transient simulation, and opens the Chiral Heat Transistor CAD dialog",
            vec![
                // Step 1: Load Quantum Metamaterial Demo
                UiAction::LoadDemo(DemoCircuitKind::QuantumMetamaterial),
                UiAction::WaitFrames(6),
                UiAction::TakeScreenshot {
                    label: "Topological Quantum Metamaterials Schematic Canvas".to_string(),
                    filename: "22_quantum_metamaterial_schematic.png".to_string(),
                },

                // Step 2: Open Chiral Heat Transistor Dialog
                UiAction::OpenChiralHeatTransistorDialog,
                UiAction::WaitFrames(6),
                UiAction::TakeScreenshot {
                    label: "Chiral Magnon-Phonon Heat Transistor & Thermal Diode Dialog".to_string(),
                    filename: "23_chiral_heat_transistor_dialog.png".to_string(),
                },

                // Step 3: Close Dialog
                UiAction::CloseChiralHeatTransistorDialog,
                UiAction::WaitFrames(2),
            ],
        )
    }

    /// Scenario 10: Topological Corner Kerr Microcomb & Dissipative Soliton Dialog.
    pub fn corner_kerr_microcomb_scenario() -> Self {
        Self::new(
            "Topological Corner Kerr Microcomb & Soliton Verification",
            "Opens the Topological Corner Kerr Microcomb & Dissipative Soliton CAD dialog and verifies all 5 tabs and physics metrics",
            vec![
                UiAction::OpenCornerKerrMicrocombDialog,
                UiAction::WaitFrames(6),
                UiAction::TakeScreenshot {
                    label: "Topological Corner Kerr Microcomb & Dissipative Soliton Dialog".to_string(),
                    filename: "24_corner_kerr_microcomb_dialog.png".to_string(),
                },
                UiAction::CloseCornerKerrMicrocombDialog,
                UiAction::WaitFrames(2),
            ],
        )
    }

    /// Scenario 11: Topological Acoustic Superconducting Nanowire Single-Phonon Detector (SNSPD) & Quantum Transceiver Dialog.
    pub fn acoustic_snspd_scenario() -> Self {
        Self::new(
            "Topological Acoustic SNSPD & Quantum Transceiver Verification",
            "Opens the Topological Acoustic SNSPD & Quantum Transceiver CAD dialog and verifies all 5 tabs and physics metrics",
            vec![
                UiAction::OpenAcousticSnspdDialog,
                UiAction::WaitFrames(6),
                UiAction::TakeScreenshot {
                    label: "Topological Acoustic Superconducting Nanowire Single-Phonon Detector (SNSPD) Dialog".to_string(),
                    filename: "25_acoustic_snspd_dialog.png".to_string(),
                },
                UiAction::CloseAcousticSnspdDialog,
                UiAction::WaitFrames(2),
            ],
        )
    }

    /// Scenario 12: Topological Acoustic Floquet Chiral Magnon-Phonon Entanglement Router & CV-QKD Dialog.
    pub fn floquet_cv_qkd_scenario() -> Self {
        Self::new(
            "Topological Acoustic Floquet Chiral Polariton Router & CV-QKD Verification",
            "Opens the Floquet Chiral Polariton Router & CV-QKD CAD dialog and verifies all 5 tabs and physics metrics",
            vec![
                UiAction::OpenFloquetCvQkdDialog,
                UiAction::WaitFrames(6),
                UiAction::TakeScreenshot {
                    label: "Topological Acoustic Floquet Chiral Polariton Router & CV-QKD Dialog".to_string(),
                    filename: "26_floquet_cv_qkd_dialog.png".to_string(),
                },
                UiAction::CloseFloquetCvQkdDialog,
                UiAction::WaitFrames(2),
            ],
        )
    }

    /// Scenario 13: High-Level & Low-Level Contraptions Showcase for Documentation & README.
    pub fn contraptions_showcase_scenario() -> Self {
        Self::new(
            "High & Low Level Contraptions Showcase",
            "Loads the 4-Bit ALU Processor Slice, RF Microwave Transceiver, and Topological Quantum Acoustic Processor contraptions, taking high-res screenshots for documentation",
            vec![
                // 1. 4-Bit Arithmetic Logic Unit & Processor Slice
                UiAction::LoadDemo(DemoCircuitKind::Alu4Bit),
                UiAction::WaitFrames(8),
                UiAction::TakeScreenshot {
                    label: "4-Bit ALU & Digital Processor Slice (Ripple Adders, MUXes, Registers)".to_string(),
                    filename: "27_contraption_4bit_alu.png".to_string(),
                },

                // 2. RF Microwave Heterodyne Transceiver Front-End
                UiAction::LoadDemo(DemoCircuitKind::RfTransceiver),
                UiAction::WaitFrames(8),
                UiAction::TakeScreenshot {
                    label: "RF Microwave Heterodyne Transceiver (BJT LNA, Mixer, SAW Filter, Cauer Thermal Ladder)".to_string(),
                    filename: "28_contraption_rf_transceiver.png".to_string(),
                },

                // 3. Topological Quantum Acoustic Metamaterial Processor
                UiAction::LoadDemo(DemoCircuitKind::TopologicalQuantumProcessor),
                UiAction::WaitFrames(8),
                UiAction::TakeScreenshot {
                    label: "Topological Quantum Acoustic Processor (SAW IDT, Majorana Braiding, Skyrmion Router, SNSPD)".to_string(),
                    filename: "29_contraption_topological_quantum_processor.png".to_string(),
                },
            ],
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
            UiAction::SwitchPreferencesTab(tab) => {
                app.preferences_dialog.active_tab = tab;
                self.execution_log.push(format!("Switched Preferences tab to {:?}", tab));
            }
            UiAction::ClosePreferences => {
                app.preferences_dialog.close();
                self.execution_log.push("Closed Preferences modal dialog".to_string());
            }
            UiAction::TriggerUnsavedChangesPopup => {
                app.mark_dirty();
                app.request_action(PendingAction::NewProject);
                self.execution_log.push("Triggered Unsaved Changes confirmation popup".to_string());
            }
            UiAction::CloseConfirmationModal => {
                app.pending_confirmation_action = None;
                self.execution_log.push("Dismissed Unsaved Changes confirmation popup".to_string());
            }
            UiAction::OpenProjectDialog(mode) => {
                match mode {
                    ProjectDialogMode::Open => app.project_dialog.open_for_open(),
                    ProjectDialogMode::SaveAs => app.project_dialog.open_for_save_as(&app.project_title),
                    ProjectDialogMode::Manager => app.project_dialog.open_for_manager(),
                }
                self.execution_log.push(format!("Opened Project Dialog in mode {:?}", mode));
            }
            UiAction::CloseProjectDialog => {
                app.project_dialog.close();
                self.execution_log.push("Closed Project Dialog".to_string());
            }
            UiAction::OpenCommandPalette => {
                app.command_palette.open();
                self.execution_log.push("Opened Command Palette".to_string());
            }
            UiAction::CloseCommandPalette => {
                app.command_palette.close();
                self.execution_log.push("Closed Command Palette".to_string());
            }
            UiAction::SetDevtoolsPanelVisible(visible) => {
                app.devtools_state.visible = visible;
                self.execution_log.push(format!("Set DevTools panel visibility to {visible}"));
            }
            UiAction::OpenChiralHeatTransistorDialog => {
                app.chiral_heat_transistor_dialog.is_open = true;
                self.execution_log.push("Opened Chiral Heat Transistor modal dialog".to_string());
            }
            UiAction::CloseChiralHeatTransistorDialog => {
                app.chiral_heat_transistor_dialog.is_open = false;
                self.execution_log.push("Closed Chiral Heat Transistor modal dialog".to_string());
            }
            UiAction::OpenCornerKerrMicrocombDialog => {
                app.corner_kerr_microcomb_dialog.is_open = true;
                self.execution_log.push("Opened Corner Kerr Microcomb modal dialog".to_string());
            }
            UiAction::CloseCornerKerrMicrocombDialog => {
                app.corner_kerr_microcomb_dialog.is_open = false;
                self.execution_log.push("Closed Corner Kerr Microcomb modal dialog".to_string());
            }
            UiAction::OpenAcousticSnspdDialog => {
                app.acoustic_snspd_dialog.is_open = true;
                self.execution_log.push("Opened Acoustic SNSPD modal dialog".to_string());
            }
            UiAction::CloseAcousticSnspdDialog => {
                app.acoustic_snspd_dialog.is_open = false;
                self.execution_log.push("Closed Acoustic SNSPD modal dialog".to_string());
            }
            UiAction::OpenFloquetCvQkdDialog => {
                app.floquet_cv_qkd_dialog.is_open = true;
                self.execution_log.push("Opened Floquet CV-QKD modal dialog".to_string());
            }
            UiAction::CloseFloquetCvQkdDialog => {
                app.floquet_cv_qkd_dialog.is_open = false;
                self.execution_log.push("Closed Floquet CV-QKD modal dialog".to_string());
            }
            UiAction::LoadDemo(kind) => {
                match kind {
                    DemoCircuitKind::VoltageDivider => app.load_voltage_divider_demo(),
                    DemoCircuitKind::BjtAmplifier => app.load_bjt_amplifier_demo(),
                    DemoCircuitKind::CmosInverter => app.load_cmos_inverter_demo(),
                    DemoCircuitKind::DiodeClipper => app.load_diode_clipper_demo(),
                    DemoCircuitKind::NmosSwitch => app.load_nmos_switch_demo(),
                    DemoCircuitKind::HalfAdder => app.load_half_adder_demo(),
                    DemoCircuitKind::BasicGates => app.load_basic_gates_demo(),
                    DemoCircuitKind::QuantumMetamaterial => app.load_quantum_metamaterial_demo(),
                    DemoCircuitKind::Alu4Bit => app.load_4bit_alu_demo(),
                    DemoCircuitKind::RfTransceiver => app.load_rf_transceiver_demo(),
                    DemoCircuitKind::TopologicalQuantumProcessor => app.load_topological_quantum_processor_demo(),
                }
                self.execution_log.push(format!("Loaded demo circuit {kind:?}"));
            }
            UiAction::ExpandCategory(cat) => {
                app.palette.open_categories.insert(cat, true);
                self.execution_log.push(format!("Expanded category {:?}", cat));
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
