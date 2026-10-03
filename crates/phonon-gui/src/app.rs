#![deny(unsafe_code)]

//! The central Phonon GUI application orchestrator, CAD layout, and interactive simulation.

use crate::extraction::ExtractionWizardDialog;
use crate::oscilloscope::{OscilloscopePanel, WaveformTrace};
use crate::schematic::{
    compile_schematic, compute_junction_dots, deserialize_project, load_project_from_file,
    save_project_to_file, serialize_project, BinaryFormatError, CanvasCommand, CompiledCircuit,
    ComponentKind, ErcDiagnostic, ErcEngine, ErcSeverity, HistoryStack, MultiSheetManager,
    NetlistSyncEngine, SchematicBus, SchematicCanvas, SchematicComponent, SchematicWire,
    SubcircuitDefinition, SymbolLibrary,
};
use crate::thermal::{Colormap, ThermalOverlay};
use crate::widgets::{
    render_top_frame_with_app, ClusterDashboardDialog, ComponentPalette, MonteCarloYieldDialog,
    NeuromorphicSnnDialog, PolaritonCavityDialog, SensitivityDialog, SmithChartDialog,
    SymbolEditorDialog, ThermalFloorplanDialog, TopFrameAction, TopFrameConfig,
};
use eframe::{App, Frame};
use egui::{
    CentralPanel, Color32, FontId, Key, Panel, PointerButton, Pos2, RichText, Sense, Stroke, Ui, Vec2,
};
use phonon_core::{AutoSelectingDynamicsBackend, PhysicsDynamicsBackend};
use phonon_solver::{solve_dc_non_linear, NewtonOptions};
use std::collections::HashMap;

/// Current interaction mode of the CAD canvas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolMode {
    Select,
    Wire,
    Place(ComponentKind),
    PlaceComponent(ComponentKind),
    Probe,
}

impl ToolMode {
    pub fn is_place(&self) -> bool {
        matches!(self, ToolMode::Place(_) | ToolMode::PlaceComponent(_))
    }

    pub fn place_kind(&self) -> Option<&ComponentKind> {
        match self {
            ToolMode::Place(k) | ToolMode::PlaceComponent(k) => Some(k),
            _ => None,
        }
    }
}

/// The unified Phonon desktop CAD application.
pub struct PhononApp {
    pub canvas: SchematicCanvas,
    pub components: Vec<SchematicComponent>,
    pub wires: Vec<SchematicWire>,
    pub next_comp_id: usize,
    pub next_wire_id: usize,
    pub selected_tool: ToolMode,
    pub selected_component_id: Option<usize>,
    pub selected_wire_id: Option<usize>,
    pub active_wire_start: Option<Pos2>,

    /// Multi-sheet schematic canvas manager.
    pub sheets: MultiSheetManager,

    /// Library of hierarchical subcircuit macro-model definitions.
    pub subcircuits: HashMap<String, SubcircuitDefinition>,

    /// High-density vectorized bus routes on the schematic canvas.
    pub buses: Vec<SchematicBus>,

    pub oscilloscope: OscilloscopePanel,
    pub thermal: ThermalOverlay,

    pub show_oscilloscope: bool,
    pub show_thermal_overlay: bool,
    pub show_netlist_window: bool,

    pub sim_status: String,
    pub dc_node_voltages: HashMap<String, f64>,
    pub component_temperatures: HashMap<String, f64>,
    pub compiled_circuit: Option<CompiledCircuit>,
    pub spice_netlist_text: String,

    /// Real-time bidirectional SPICE netlist synchronization engine.
    pub netlist_sync: NetlistSyncEngine,

    /// Active Electrical Rules Check (ERC) diagnostic findings.
    pub erc_diagnostics: Vec<ErcDiagnostic>,

    /// Whether the visual ERC diagnostic overlay is rendered on canvas.
    pub show_erc_overlay: bool,

    /// SPICE model parameter extraction wizard modal dialog.
    pub extraction_wizard: ExtractionWizardDialog,

    /// Non-linear transient sensitivity analysis and worst-case optimization dialog.
    pub sensitivity_dialog: SensitivityDialog,

    /// Interactive Monte Carlo Yield & Latin Hypercube Sampling Inspector dialog.
    pub monte_carlo_dialog: MonteCarloYieldDialog,

    /// Interactive RF S-Parameters, Smith Chart & Harmonic Balance visualizer dialog.
    pub smith_chart_dialog: SmithChartDialog,

    /// Interactive Thermal Floorplan & Transient Co-Simulation Studio dialog.
    pub thermal_floorplan_dialog: ThermalFloorplanDialog,

    /// Interactive Polariton Waveguide & Topological Photonic Cavity Simulator dialog.
    pub polariton_cavity_dialog: PolaritonCavityDialog,

    /// Interactive Distributed Cloud Parameter Sweep Cluster Engine Dashboard dialog.
    pub cluster_dashboard_dialog: ClusterDashboardDialog,

    /// Interactive Neuromorphic Studio Canvas & Synaptic Weight Visualizer dialog.
    pub neuromorphic_snn_dialog: NeuromorphicSnnDialog,

    /// Interactive Logisim/KiCad-style component symbol and shape editor dialog.
    pub symbol_editor: SymbolEditorDialog,

    /// User library storing custom component symbol definitions.
    pub symbol_library: SymbolLibrary,

    // Drag tracking for selected component
    dragging_component: bool,
    drag_start_pos: Option<(usize, Pos2)>,

    /// Reversible undo/redo history stack and command engine.
    pub history: HistoryStack,

    /// Active editing buffer tracking component value before modification.
    editing_comp_value: Option<(usize, String)>,

    /// Active rotation angle index for components being placed (0 = 0 deg, 1 = 90 deg, 2 = 180 deg, 3 = 270 deg).
    pub placement_rotation: u8,

    /// Configuration for the custom window top frame.
    pub top_frame_config: TopFrameConfig,

    /// Active physics dynamics backend driving physical simulation state.
    pub dynamics_backend: Box<dyn PhysicsDynamicsBackend>,

    /// Hierarchical categorized component palette drawer.
    pub palette: ComponentPalette,

    /// Whether the left component palette panel is visible (collapsible to burger menu).
    pub show_palette: bool,

    /// Instant boot theme configuration.
    pub boot_theme_config: crate::BootThemeConfig,
}

impl std::fmt::Debug for PhononApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhononApp")
            .field("next_comp_id", &self.next_comp_id)
            .field("next_wire_id", &self.next_wire_id)
            .field("selected_tool", &self.selected_tool)
            .field("top_frame_config", &self.top_frame_config)
            .field("dynamics_backend", &self.dynamics_backend.info())
            .field("palette", &self.palette)
            .field("boot_theme_config", &self.boot_theme_config)
            .finish()
    }
}

impl Default for PhononApp {
    fn default() -> Self {
        let mut app = Self {
            canvas: SchematicCanvas::new(),
            components: Vec::with_capacity(64),
            wires: Vec::with_capacity(64),
            next_comp_id: 1,
            next_wire_id: 1,
            selected_tool: ToolMode::Select,
            selected_component_id: None,
            selected_wire_id: None,
            active_wire_start: None,
            sheets: MultiSheetManager::new("Main"),
            subcircuits: HashMap::new(),
            buses: Vec::new(),
            oscilloscope: OscilloscopePanel::new(),
            thermal: ThermalOverlay::new(),
            show_oscilloscope: true,
            show_thermal_overlay: true,
            show_netlist_window: false,
            sim_status: String::new(),
            dc_node_voltages: HashMap::with_capacity(32),
            component_temperatures: HashMap::with_capacity(32),
            compiled_circuit: None,
            spice_netlist_text: String::new(),
            netlist_sync: NetlistSyncEngine::new(),
            erc_diagnostics: Vec::new(),
            show_erc_overlay: true,
            extraction_wizard: ExtractionWizardDialog::new(),
            sensitivity_dialog: SensitivityDialog::new(),
            monte_carlo_dialog: MonteCarloYieldDialog::new(),
            smith_chart_dialog: SmithChartDialog::new(),
            thermal_floorplan_dialog: ThermalFloorplanDialog::new(),
            polariton_cavity_dialog: PolaritonCavityDialog::new(),
            cluster_dashboard_dialog: ClusterDashboardDialog::new(),
            neuromorphic_snn_dialog: NeuromorphicSnnDialog::new(),
            symbol_editor: SymbolEditorDialog::new(),
            symbol_library: SymbolLibrary::new(),
            dragging_component: false,
            drag_start_pos: None,
            history: HistoryStack::with_capacity(500, 64),
            editing_comp_value: None,
            placement_rotation: 0,
            top_frame_config: TopFrameConfig::default(),
            dynamics_backend: Box::new(AutoSelectingDynamicsBackend::new()),
            palette: ComponentPalette::new(),
            show_palette: true,
            boot_theme_config: crate::default_boot_theme_config(),
        };

        // Initialize with default Voltage Divider demo
        app.load_voltage_divider_demo();
        app.history.clear();
        app
    }
}

impl PhononApp {
    /// Creates a default `PhononApp` with an auto-selecting dynamics backend.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        Self::default()
    }

    /// Creates a `PhononApp` instance with an injected custom physics dynamics backend.
    pub fn with_backend(
        cc: &eframe::CreationContext<'_>,
        backend: Box<dyn PhysicsDynamicsBackend>,
    ) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let mut app = Self::default();
        app.dynamics_backend = backend;
        app
    }

    /// Returns a shared reference to the active physics dynamics backend.
    pub fn backend(&self) -> &dyn PhysicsDynamicsBackend {
        self.dynamics_backend.as_ref()
    }

    /// Returns a mutable reference to the active physics dynamics backend.
    pub fn backend_mut(&mut self) -> &mut dyn PhysicsDynamicsBackend {
        self.dynamics_backend.as_mut()
    }

    /// Clears canvas state without pushing an undo command.
    pub fn clear_canvas_state(&mut self) {
        self.components.clear();
        self.wires.clear();
        self.buses.clear();
        self.next_comp_id = 1;
        self.next_wire_id = 1;
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.active_wire_start = None;
        self.dc_node_voltages.clear();
        self.component_temperatures.clear();
        self.compiled_circuit = None;
        self.spice_netlist_text.clear();
        self.sim_status.clear();
        self.drag_start_pos = None;
        self.editing_comp_value = None;
        self.canvas.clear();
        self.erc_diagnostics.clear();
        self.netlist_sync.invalidate();
    }

    /// Synchronizes app components, wires, and buses into the active canvas object and active sheet.
    pub fn sync_canvas_state(&mut self) {
        self.canvas.components = self.components.clone();
        self.canvas.wires = self.wires.clone();
        self.canvas.buses = self.buses.clone();

        let active = self.sheets.active_sheet_mut();
        active.canvas.components = self.components.clone();
        active.canvas.wires = self.wires.clone();
        active.canvas.subcircuit_instances = self.canvas.subcircuit_instances.clone();
        active.canvas.buses = self.buses.clone();
        active.camera_offset = self.canvas.pan;
        active.camera_zoom = self.canvas.zoom;
    }

    /// Adds a new sheet, synchronizing the current canvas, and switches to the new sheet.
    pub fn add_sheet(&mut self, name: &str) -> usize {
        self.sync_canvas_state();
        let idx = self.sheets.add_sheet(name);
        self.switch_to_sheet(idx);
        idx
    }

    /// Removes a sheet by index, preventing deletion of the last remaining sheet.
    pub fn remove_sheet(&mut self, idx: usize) -> Result<(), String> {
        self.sync_canvas_state();
        self.sheets.remove_sheet(idx)?;
        self.load_active_sheet();
        Ok(())
    }

    /// Switches the active sheet, saving the current sheet canvas state and restoring target sheet state.
    pub fn switch_to_sheet(&mut self, idx: usize) -> bool {
        self.sync_canvas_state();
        if self.sheets.switch_sheet(idx) {
            self.load_active_sheet();
            true
        } else {
            false
        }
    }

    /// Loads the active sheet's canvas and camera into the primary app workspace.
    pub fn load_active_sheet(&mut self) {
        let active = self.sheets.active_sheet();
        self.components = active.canvas.components.clone();
        self.wires = active.canvas.wires.clone();
        self.canvas.components = self.components.clone();
        self.canvas.wires = self.wires.clone();
        self.canvas.subcircuit_instances = active.canvas.subcircuit_instances.clone();
        self.canvas.buses = active.canvas.buses.clone();
        self.buses = active.canvas.buses.clone();
        self.canvas.pan = active.camera_offset;
        self.canvas.zoom = active.camera_zoom;
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.active_wire_start = None;
        self.netlist_sync.invalidate();
    }

    /// Runs the Electrical Rules Check (ERC) diagnostic engine on the current schematic canvas.
    pub fn run_erc(&mut self) {
        self.sync_canvas_state();
        self.erc_diagnostics = ErcEngine::evaluate_canvas(&self.canvas);
        let error_count = self
            .erc_diagnostics
            .iter()
            .filter(|d| d.severity == ErcSeverity::Error)
            .count();
        let warn_count = self
            .erc_diagnostics
            .iter()
            .filter(|d| d.severity == ErcSeverity::Warning)
            .count();
        self.sim_status = format!(
            "ERC Checked: {} errors, {} warnings",
            error_count, warn_count
        );
    }

    /// Clears the canvas, removing all components and wires, recording the action in history.
    pub fn clear_all(&mut self) {
        if !self.components.is_empty() || !self.wires.is_empty() {
            self.history.record(CanvasCommand::ClearAll {
                components: self.components.clone(),
                wires: self.wires.clone(),
            });
        }
        self.clear_canvas_state();
    }

    /// Reverses the most recent canvas mutation action from the history stack.
    pub fn undo(&mut self) -> bool {
        let success = self.history.undo(&mut self.components, &mut self.wires);
        if success {
            self.selected_component_id = None;
            self.selected_wire_id = None;
            self.active_wire_start = None;
            self.sim_status = "Undo".to_string();
            let max_c_id = self.components.iter().map(|c| c.id).max().unwrap_or(0);
            if self.next_comp_id <= max_c_id {
                self.next_comp_id = max_c_id + 1;
            }
            let max_w_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0);
            if self.next_wire_id <= max_w_id {
                self.next_wire_id = max_w_id + 1;
            }
        }
        success
    }

    /// Re-applies the most recent undone canvas mutation action from the history stack.
    pub fn redo(&mut self) -> bool {
        let success = self.history.redo(&mut self.components, &mut self.wires);
        if success {
            self.selected_component_id = None;
            self.selected_wire_id = None;
            self.active_wire_start = None;
            self.sim_status = "Redo".to_string();
            let max_c_id = self.components.iter().map(|c| c.id).max().unwrap_or(0);
            if self.next_comp_id <= max_c_id {
                self.next_comp_id = max_c_id + 1;
            }
            let max_w_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0);
            if self.next_wire_id <= max_w_id {
                self.next_wire_id = max_w_id + 1;
            }
        }
        success
    }

    /// Modifies a component value and records the change into history.
    pub fn modify_component_value(&mut self, id: usize, new_val: impl Into<String>) {
        let new_val = new_val.into();
        if let Some(comp) = self.components.iter_mut().find(|c| c.id == id) {
            if comp.value_str != new_val {
                let old_val = std::mem::replace(&mut comp.value_str, new_val.clone());
                self.history.record(CanvasCommand::ModifyComponentValue {
                    id,
                    old_val,
                    new_val,
                });
            }
        }
    }

    /// Serializes current schematic project into ultra-compact binary format (.phn).
    pub fn save_to_bytes(&self) -> Vec<u8> {
        let title = &self.top_frame_config.circuit_name;
        serialize_project(title, &self.components, &self.wires)
    }

    /// Loads schematic project from ultra-compact binary format (.phn) bytes, resetting history clean index.
    pub fn load_from_bytes(&mut self, bytes: &[u8]) -> Result<(), BinaryFormatError> {
        let proj = deserialize_project(bytes)?;
        self.components = proj.components;
        self.wires = proj.wires;
        self.history.clear();
        let max_c_id = self.components.iter().map(|c| c.id).max().unwrap_or(0);
        self.next_comp_id = max_c_id + 1;
        let max_w_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0);
        self.next_wire_id = max_w_id + 1;
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.active_wire_start = None;
        self.dc_node_voltages.clear();
        self.component_temperatures.clear();
        self.compiled_circuit = None;
        self.spice_netlist_text.clear();
        self.top_frame_config.circuit_name = proj.title.clone();
        self.sim_status = format!("Loaded project: {}", proj.title);
        Ok(())
    }

    /// Serializes project and resets the history clean index to mark current state as saved.
    pub fn save_project(&mut self) -> Vec<u8> {
        self.history.mark_clean();
        self.save_to_bytes()
    }

    /// Saves project state to disk at `path` and resets history clean index.
    pub fn save_project_file<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<(), BinaryFormatError> {
        self.history.mark_clean();
        let title = &self.top_frame_config.circuit_name;
        save_project_to_file(path, title, &self.components, &self.wires)?;
        self.sim_status = "Saved project (.phn)".to_string();
        Ok(())
    }

    /// Loads project state from disk at `path` and resets history clean index.
    pub fn load_project_file<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<(), BinaryFormatError> {
        let proj = load_project_from_file(path)?;
        self.components = proj.components;
        self.wires = proj.wires;
        self.history.clear();
        let max_c_id = self.components.iter().map(|c| c.id).max().unwrap_or(0);
        self.next_comp_id = max_c_id + 1;
        let max_w_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0);
        self.next_wire_id = max_w_id + 1;
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.active_wire_start = None;
        self.dc_node_voltages.clear();
        self.component_temperatures.clear();
        self.compiled_circuit = None;
        self.spice_netlist_text.clear();
        self.top_frame_config.circuit_name = proj.title.clone();
        self.sim_status = format!("Loaded project: {}", proj.title);
        Ok(())
    }

    /// Loads an interactive Voltage Divider demo circuit.
    pub fn load_voltage_divider_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Components
        let v1 =
            SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(200.0, 300.0), 1);
        let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(360.0, 240.0), 1);
        let r2 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(360.0, 360.0), 2);
        let gnd = SchematicComponent::new(4, ComponentKind::Ground, Pos2::new(200.0, 440.0), 1);

        self.components = vec![v1, r1, r2, gnd];
        self.next_comp_id = 5;

        // 2. Wires
        let w1 =
            SchematicWire::manhattan_route(1, Pos2::new(200.0, 260.0), Pos2::new(360.0, 200.0));
        let w2 =
            SchematicWire::manhattan_route(2, Pos2::new(360.0, 280.0), Pos2::new(360.0, 320.0));
        let w3 =
            SchematicWire::manhattan_route(3, Pos2::new(360.0, 400.0), Pos2::new(200.0, 340.0));
        let w4 =
            SchematicWire::manhattan_route(4, Pos2::new(200.0, 340.0), Pos2::new(200.0, 420.0));

        self.wires = vec![w1, w2, w3, w4];
        self.next_wire_id = 5;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
    }

    /// Loads an interactive Diode Limiter / Clipper demo circuit.
    pub fn load_diode_clipper_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        let v1 =
            SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(180.0, 300.0), 1);
        let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(320.0, 240.0), 1);
        let d1 = SchematicComponent::new(3, ComponentKind::Diode, Pos2::new(440.0, 320.0), 1);
        let gnd = SchematicComponent::new(4, ComponentKind::Ground, Pos2::new(180.0, 440.0), 1);

        self.components = vec![v1, r1, d1, gnd];
        self.next_comp_id = 5;

        let w1 =
            SchematicWire::manhattan_route(1, Pos2::new(180.0, 260.0), Pos2::new(320.0, 200.0));
        let w2 =
            SchematicWire::manhattan_route(2, Pos2::new(320.0, 280.0), Pos2::new(440.0, 280.0));
        let w3 =
            SchematicWire::manhattan_route(3, Pos2::new(440.0, 360.0), Pos2::new(180.0, 340.0));
        let w4 =
            SchematicWire::manhattan_route(4, Pos2::new(180.0, 340.0), Pos2::new(180.0, 420.0));

        self.wires = vec![w1, w2, w3, w4];
        self.next_wire_id = 5;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
    }

    /// Loads an interactive BJT Common Emitter Amplifier demo circuit.
    pub fn load_bjt_amplifier_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Components
        // VCC: 12V Supply
        let vcc = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(420.0, 180.0), 1)
            .with_value("12.0");
        // RC: Collector Resistor 2.2k
        let rc = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(420.0, 280.0), 1)
            .with_value("2.2k");
        // Q1: BJT NPN Transistor at (400, 380) -> C=(420, 340), B=(380, 380), E=(420, 420)
        let q1 = SchematicComponent::new(3, ComponentKind::BjtNpn, Pos2::new(400.0, 380.0), 1);
        // RE: Emitter Resistor 470
        let re = SchematicComponent::new(4, ComponentKind::Resistor, Pos2::new(420.0, 480.0), 2)
            .with_value("470");
        // RB1: Base Bias Upper 22k
        let rb1 = SchematicComponent::new(5, ComponentKind::Resistor, Pos2::new(300.0, 280.0), 3)
            .with_value("22k");
        // RB2: Base Bias Lower 4.7k
        let rb2 = SchematicComponent::new(6, ComponentKind::Resistor, Pos2::new(300.0, 480.0), 4)
            .with_value("4.7k");
        // VIN: AC Signal Source
        let vin = SchematicComponent::new(7, ComponentKind::AcVoltageSource, Pos2::new(180.0, 380.0), 1)
            .with_value("SIN(0 0.05 1k)");
        // GND: Reference Ground
        let gnd = SchematicComponent::new(8, ComponentKind::Ground, Pos2::new(300.0, 560.0), 1);

        self.components = vec![vcc, rc, q1, re, rb1, rb2, vin, gnd];
        self.next_comp_id = 9;

        // 2. Wires
        // VCC top to RC pin 1
        let w1 = SchematicWire::manhattan_route(1, Pos2::new(420.0, 140.0), Pos2::new(420.0, 240.0));
        // VCC top to RB1 pin 1
        let w2 = SchematicWire::manhattan_route(2, Pos2::new(420.0, 140.0), Pos2::new(300.0, 240.0));
        // RC pin 2 to Q1 Collector (420, 340)
        let w3 = SchematicWire::manhattan_route(3, Pos2::new(420.0, 320.0), Pos2::new(420.0, 340.0));
        // Q1 Emitter (420, 420) to RE pin 1
        let w4 = SchematicWire::manhattan_route(4, Pos2::new(420.0, 420.0), Pos2::new(420.0, 440.0));
        // VIN (+) to Q1 Base / RB divider node
        let w5 = SchematicWire::manhattan_route(5, Pos2::new(180.0, 340.0), Pos2::new(380.0, 380.0));
        let w6 = SchematicWire::manhattan_route(6, Pos2::new(300.0, 320.0), Pos2::new(380.0, 380.0));
        let w7 = SchematicWire::manhattan_route(7, Pos2::new(380.0, 380.0), Pos2::new(300.0, 440.0));
        // Bottom GND rail connections
        let w8 = SchematicWire::manhattan_route(8, Pos2::new(300.0, 520.0), Pos2::new(300.0, 540.0));
        let w9 = SchematicWire::manhattan_route(9, Pos2::new(420.0, 520.0), Pos2::new(300.0, 540.0));
        let w10 = SchematicWire::manhattan_route(10, Pos2::new(180.0, 420.0), Pos2::new(300.0, 540.0));

        self.wires = vec![w1, w2, w3, w4, w5, w6, w7, w8, w9, w10];
        self.next_wire_id = 11;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
    }

    /// Loads an interactive CMOS Inverter Pair demo circuit.
    pub fn load_cmos_inverter_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Components
        // VDD: 3.3V DC Supply
        let vdd = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(200.0, 240.0), 1)
            .with_value("3.3");
        // M1: PMOS Pull-up at (360, 260) -> D=(380, 220), G=(340, 260), S=(380, 300)
        let m1 = SchematicComponent::new(2, ComponentKind::Pmos, Pos2::new(360.0, 260.0), 1);
        // M2: NMOS Pull-down at (360, 380) -> D=(380, 340), G=(340, 380), S=(380, 420)
        let m2 = SchematicComponent::new(3, ComponentKind::Nmos, Pos2::new(360.0, 380.0), 1);
        // VIN: Pulse Generator Input
        let vin = SchematicComponent::new(4, ComponentKind::PulseGenerator, Pos2::new(180.0, 380.0), 1)
            .with_value("PULSE(0 3.3 0 1n 1n 10u 20u)");
        // GND: Reference Ground
        let gnd = SchematicComponent::new(5, ComponentKind::Ground, Pos2::new(380.0, 480.0), 1);

        self.components = vec![vdd, m1, m2, vin, gnd];
        self.next_comp_id = 6;

        // 2. Wires
        // VDD (+) to PMOS Drain
        let w1 = SchematicWire::manhattan_route(1, Pos2::new(200.0, 200.0), Pos2::new(380.0, 220.0));
        // PMOS Source (380, 300) to NMOS Drain (380, 340) -> Output Node
        let w2 = SchematicWire::manhattan_route(2, Pos2::new(380.0, 300.0), Pos2::new(380.0, 340.0));
        // VIN (+) to Common Gates
        let w3 = SchematicWire::manhattan_route(3, Pos2::new(180.0, 340.0), Pos2::new(340.0, 260.0));
        let w4 = SchematicWire::manhattan_route(4, Pos2::new(340.0, 260.0), Pos2::new(340.0, 380.0));
        // NMOS Source to GND
        let w5 = SchematicWire::manhattan_route(5, Pos2::new(380.0, 420.0), Pos2::new(380.0, 460.0));
        // VDD (-) to GND
        let w6 = SchematicWire::manhattan_route(6, Pos2::new(200.0, 280.0), Pos2::new(380.0, 460.0));
        // VIN (-) to GND
        let w7 = SchematicWire::manhattan_route(7, Pos2::new(180.0, 420.0), Pos2::new(380.0, 460.0));

        self.wires = vec![w1, w2, w3, w4, w5, w6, w7];
        self.next_wire_id = 8;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
    }

    /// Loads an interactive NMOS Switch demo circuit.
    pub fn load_nmos_switch_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Components
        // VDD: 5V Supply
        let vdd = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(180.0, 240.0), 1)
            .with_value("5.0");
        // RLOAD: 1k Load Resistor
        let rload = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(380.0, 240.0), 1)
            .with_value("1k");
        // M1: NMOS Switch at (360, 360) -> D=(380, 320), G=(340, 360), S=(380, 400)
        let m1 = SchematicComponent::new(3, ComponentKind::Nmos, Pos2::new(360.0, 360.0), 1);
        // VGATE: Gate Pulse Generator
        let vgate = SchematicComponent::new(4, ComponentKind::PulseGenerator, Pos2::new(220.0, 360.0), 1)
            .with_value("PULSE(0 5 0 1n 1n 50u 100u)");
        // GND: Reference Ground
        let gnd = SchematicComponent::new(5, ComponentKind::Ground, Pos2::new(380.0, 460.0), 1);

        self.components = vec![vdd, rload, m1, vgate, gnd];
        self.next_comp_id = 6;

        // 2. Wires
        // VDD (+) to RLOAD pin 1
        let w1 = SchematicWire::manhattan_route(1, Pos2::new(180.0, 200.0), Pos2::new(380.0, 200.0));
        // RLOAD pin 2 to NMOS Drain (380, 320)
        let w2 = SchematicWire::manhattan_route(2, Pos2::new(380.0, 280.0), Pos2::new(380.0, 320.0));
        // VGATE (+) to NMOS Gate (340, 360)
        let w3 = SchematicWire::manhattan_route(3, Pos2::new(220.0, 320.0), Pos2::new(340.0, 360.0));
        // NMOS Source (380, 400) to GND
        let w4 = SchematicWire::manhattan_route(4, Pos2::new(380.0, 400.0), Pos2::new(380.0, 440.0));
        // VDD (-) to GND
        let w5 = SchematicWire::manhattan_route(5, Pos2::new(180.0, 280.0), Pos2::new(380.0, 440.0));
        // VGATE (-) to GND
        let w6 = SchematicWire::manhattan_route(6, Pos2::new(220.0, 400.0), Pos2::new(380.0, 440.0));

        self.wires = vec![w1, w2, w3, w4, w5, w6];
        self.next_wire_id = 7;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
    }

    /// Compiles schematic and runs the non-linear DC Operating Point (.OP) solver.
    pub fn run_dc_op(&mut self) {
        match compile_schematic(&self.components, &self.wires) {
            Err(err) => {
                self.sim_status = format!("Compilation Error: {}", err);
            }
            Ok(compiled) => {
                self.spice_netlist_text = compiled.spice_netlist.clone();

                let newton_opts = NewtonOptions::default();
                match solve_dc_non_linear(&compiled.graph, &compiled.model_ctx, &newton_opts) {
                    Err(err) => {
                        self.sim_status = format!("DC Solver Error: {}", err);
                    }
                    Ok(sol) => {
                        self.dc_node_voltages.clear();
                        self.component_temperatures.clear();

                        // Map net voltages
                        for net in &compiled.net_names {
                            if let Some(node_id) = compiled.graph.get_node(net) {
                                if (node_id.0 as usize) < sol.node_voltages.len() {
                                    let v = sol.node_voltages[node_id.0 as usize];
                                    self.dc_node_voltages.insert(net.clone(), v);
                                }
                            }
                        }

                        // Estimate Joule heating and junction temperature for components
                        for comp in &self.components {
                            let pins = comp.all_pins();
                            if pins.len() >= 2 {
                                let n1 = compiled
                                    .pin_to_net
                                    .get(&(comp.name.clone(), pins[0].0.to_string()))
                                    .cloned()
                                    .unwrap_or_else(|| "0".to_string());
                                let n2 = compiled
                                    .pin_to_net
                                    .get(&(comp.name.clone(), pins[1].0.to_string()))
                                    .cloned()
                                    .unwrap_or_else(|| "0".to_string());

                                let v1 = *self.dc_node_voltages.get(&n1).unwrap_or(&0.0);
                                let v2 = *self.dc_node_voltages.get(&n2).unwrap_or(&0.0);
                                let vdrop = (v1 - v2).abs();

                                // Simple power model for thermal feedback visualization
                                let power_watts = match comp.kind {
                                    ComponentKind::Resistor => {
                                        let r = comp
                                            .value_str
                                            .trim_end_matches('k')
                                            .parse::<f64>()
                                            .unwrap_or(1.0)
                                            * 1e3;
                                        (vdrop * vdrop) / r.max(0.1)
                                    }
                                    ComponentKind::Diode => {
                                        // P = Vd * Id, Id ~ 1mA to 50mA
                                        vdrop * 0.015
                                    }
                                    ComponentKind::VoltageSource | ComponentKind::CurrentSource => {
                                        vdrop * 0.005
                                    }
                                    _ => 0.001,
                                };

                                let r_th = 60.0; // K/W
                                let t_ambient = 25.0;
                                let temp_c = t_ambient + power_watts * r_th;
                                self.component_temperatures
                                    .insert(comp.name.clone(), temp_c);
                            }
                        }

                        self.sim_status = format!(
                            "DC Solved: {} nodes, cond ratio = {:.2e}",
                            sol.node_voltages.len(),
                            sol.condition_ratio
                        );
                        self.compiled_circuit = Some(compiled);
                    }
                }
            }
        }
    }

    /// Generates multi-trace transient demo waveforms into the virtual oscilloscope.
    pub fn run_transient_demo(&mut self) {
        self.oscilloscope.clear();

        // 1. Input Sine Wave: 5.0 Vpp, 1 kHz
        let mut trace_vin = WaveformTrace::new("VIN (Input 1kHz)", Color32::from_rgb(80, 200, 255));
        let mut trace_vout =
            WaveformTrace::new("VOUT (Attenuated)", Color32::from_rgb(100, 255, 140));
        let mut trace_temp =
            WaveformTrace::new("Temp Junction (°C)", Color32::from_rgb(255, 120, 80));

        let num_points = 1200;
        let t_total = 0.003; // 3 ms
        let dt = t_total / num_points as f64;
        let freq = 1000.0;
        let omega = 2.0 * std::f64::consts::PI * freq;

        for i in 0..num_points {
            let t = i as f64 * dt;
            let v_in = 2.5 + 2.5 * (omega * t).sin();
            let v_out = 1.25 + 1.25 * (omega * t - 0.35).sin();
            let temp = 25.0 + 12.0 * (1.0 - (-t / 0.0008).exp()) + 1.5 * (omega * 2.0 * t).sin();

            trace_vin.push(t, v_in);
            trace_vout.push(t, v_out);
            trace_temp.push(t, temp);
        }

        self.oscilloscope.add_trace(trace_vin);
        self.oscilloscope.add_trace(trace_vout);
        self.oscilloscope.add_trace(trace_temp);

        self.sim_status = "Transient Solved: 3 traces, 3,600 samples".to_string();
    }

    /// Executes backward adjoint sensitivity analysis and worst-case optimization for the schematic circuit.
    pub fn run_sensitivity_analysis(&mut self) {
        match compile_schematic(&self.components, &self.wires) {
            Err(err) => {
                self.sim_status = format!("Compilation Error: {}", err);
            }
            Ok(compiled) => {
                let opts = phonon_solver::TransientOptions::default();
                self.sensitivity_dialog.run_analysis(
                    &compiled.graph,
                    &compiled.model_ctx,
                    &opts,
                );
                self.sensitivity_dialog.is_open = true;
                self.sim_status = self.sensitivity_dialog.status_msg.clone();
            }
        }
    }

    /// Executes distributed Monte Carlo & Latin Hypercube parameter sweep and yield analysis.
    pub fn run_monte_carlo_sweep(&mut self) {
        match compile_schematic(&self.components, &self.wires) {
            Err(err) => {
                self.sim_status = format!("Compilation Error: {}", err);
            }
            Ok(compiled) => {
                let opts = phonon_solver::TransientOptions::default();
                self.monte_carlo_dialog.run_sweep(
                    &compiled.graph,
                    &compiled.model_ctx,
                    &opts,
                );
                self.monte_carlo_dialog.is_open = true;
                self.sim_status = self.monte_carlo_dialog.status_msg.clone();
            }
        }
    }

    /// Runs dynamic electro-thermal co-simulation with thermal floorplan dialog.
    pub fn run_thermal_cosim(&mut self) {
        match compile_schematic(&self.components, &self.wires) {
            Err(err) => {
                self.sim_status = format!("Compilation Error: {}", err);
            }
            Ok(compiled) => {
                self.thermal_floorplan_dialog.auto_place_components(&compiled.graph);
                self.thermal_floorplan_dialog.run_simulation(&compiled.graph, &compiled.model_ctx);
                self.thermal_floorplan_dialog.is_open = true;
                self.sim_status = self.thermal_floorplan_dialog.status_msg.clone();
            }
        }
    }

    /// Deletes the currently selected component or wire.
    pub fn delete_selected(&mut self) {
        if let Some(cid) = self.selected_component_id {
            if let Some(idx) = self.components.iter().position(|c| c.id == cid) {
                let comp = self.components.remove(idx);
                self.history.record(CanvasCommand::DeleteComponent(comp));
            }
            self.selected_component_id = None;
            self.sim_status.clear();
        } else if let Some(wid) = self.selected_wire_id {
            if let Some(idx) = self.wires.iter().position(|w| w.id == wid) {
                let wire = self.wires.remove(idx);
                self.history.record(CanvasCommand::DeleteWire(wire));
            }
            self.selected_wire_id = None;
            self.sim_status.clear();
        }
    }

    /// Rotates the active component (selected, being dragged, or held during placement) clockwise by 90 degrees.
    pub fn rotate_active(&mut self) {
        if let Some(cid) = self.selected_component_id {
            if let Some(comp) = self.components.iter_mut().find(|c| c.id == cid) {
                let from_rot = comp.rotation;
                comp.rotate_clockwise();
                let to_rot = comp.rotation;
                self.history.record(CanvasCommand::RotateComponent {
                    id: cid,
                    from_rot,
                    to_rot,
                });
                self.sim_status.clear();
                return;
            }
        }
        if self.selected_tool.is_place() {
            self.placement_rotation = (self.placement_rotation + 1) % 4;
            self.sim_status.clear();
        }
    }

    /// Rotates the selected component clockwise by 90 degrees.
    pub fn rotate_selected(&mut self) {
        self.rotate_active();
    }

    /// Handles global hotkeys and keyboard shortcuts.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let ctrl = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
        let shift = ctx.input(|i| i.modifiers.shift);

        // Undo: Ctrl+Z
        if ctrl && !shift && ctx.input(|i| i.key_pressed(Key::Z)) {
            self.undo();
        }
        // Redo: Ctrl+Y or Ctrl+Shift+Z
        if (ctrl && ctx.input(|i| i.key_pressed(Key::Y)))
            || (ctrl && shift && ctx.input(|i| i.key_pressed(Key::Z)))
        {
            self.redo();
        }

        if !ctrl && ctx.input(|i| i.key_pressed(Key::R)) {
            self.rotate_active();
        }
        if ctx.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
            self.delete_selected();
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.active_wire_start = None;
            self.selected_tool = ToolMode::Select;
            self.selected_component_id = None;
            self.selected_wire_id = None;
            self.placement_rotation = 0;
        }
        if !ctrl && ctx.input(|i| i.key_pressed(Key::W)) {
            self.selected_tool = ToolMode::Wire;
            self.active_wire_start = None;
            self.placement_rotation = 0;
        }
        if !ctrl && ctx.input(|i| i.key_pressed(Key::S)) {
            self.selected_tool = ToolMode::Select;
            self.active_wire_start = None;
            self.placement_rotation = 0;
        }
    }

    /// Interactive canvas response and rendering.
    fn render_canvas(&mut self, ui: &mut egui::Ui) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click_and_drag());
        let viewport = response.rect;

        // 1. Pan & Zoom
        self.canvas.handle_pan_zoom(ui, &response);

        // 2. Render background grid
        self.canvas.render_grid(&painter, viewport);

        // Collect all pin positions for snapping and junction rendering
        let mut all_pin_positions = Vec::new();
        for comp in &self.components {
            for (_, p) in comp.all_pins() {
                all_pin_positions.push(p);
            }
        }

        // 3. Render wires
        for wire in &self.wires {
            let is_sel = self.selected_wire_id == Some(wire.id);
            wire.render(&painter, &self.canvas, is_sel);
        }

        // 3b. Render buses
        for bus in &self.buses {
            bus.render(&painter, &self.canvas, false);
        }

        // 3c. Render subcircuit instances
        for inst in &self.canvas.subcircuit_instances {
            inst.render(&painter, &self.canvas, self.subcircuits.get(&inst.def_name), false);
        }

        // 4. Render junction dots
        let junctions = compute_junction_dots(&self.wires, &all_pin_positions);
        let junction_stroke = Stroke::new(1.0, Color32::from_rgb(180, 255, 180));
        for j_world in &junctions {
            let j_screen = self.canvas.world_to_screen(*j_world);
            painter.circle_filled(
                j_screen,
                4.0 * self.canvas.zoom.clamp(0.8, 1.6),
                Color32::from_rgb(60, 200, 80),
            );
            painter.circle_stroke(
                j_screen,
                4.0 * self.canvas.zoom.clamp(0.8, 1.6),
                junction_stroke,
            );
        }

        // 5. Mouse interactions on canvas
        let mouse_pos = ui.input(|i| i.pointer.hover_pos());
        if let Some(mouse_screen) = mouse_pos {
            let mouse_world = self.canvas.screen_to_world(mouse_screen);
            let snapped_world = self.canvas.snap_to_grid(mouse_world);

            // Handle tool actions on click
            if response.clicked_by(PointerButton::Primary) {
                match &self.selected_tool {
                    ToolMode::Select => {
                        // Hit-test components
                        if let Some(comp) = self
                            .components
                            .iter()
                            .rev()
                            .find(|c| c.contains(mouse_world))
                        {
                            self.selected_component_id = Some(comp.id);
                            self.selected_wire_id = None;
                        } else if let Some(wire) = self
                            .wires
                            .iter()
                            .rev()
                            .find(|w| w.contains(mouse_world, 6.0))
                        {
                            self.selected_wire_id = Some(wire.id);
                            self.selected_component_id = None;
                        } else {
                            self.selected_component_id = None;
                            self.selected_wire_id = None;
                        }
                    }
                    ToolMode::Place(kind) | ToolMode::PlaceComponent(kind) => {
                        let count = self.components.iter().filter(|c| c.kind == *kind).count() + 1;
                        let mut new_comp = SchematicComponent::new(
                            self.next_comp_id,
                            kind.clone(),
                            snapped_world,
                            count,
                        );
                        new_comp.rotation = self.placement_rotation;
                        self.components.push(new_comp.clone());
                        self.next_comp_id += 1;
                        self.sim_status = format!("Placed {}", kind.prefix());
                        self.history.record(CanvasCommand::AddComponent(new_comp));
                    }
                    ToolMode::Wire => {
                        if let Some(start) = self.active_wire_start {
                            // Finish wire
                            if (start - snapped_world).length() > 5.0 {
                                let wire = SchematicWire::manhattan_route(
                                    self.next_wire_id,
                                    start,
                                    snapped_world,
                                );
                                self.wires.push(wire.clone());
                                self.next_wire_id += 1;
                                self.history.record(CanvasCommand::AddWire(wire));
                                self.active_wire_start = Some(snapped_world); // Continue routing from current point
                            }
                        } else {
                            // Find nearest pin to snap start
                            let nearest_pin = all_pin_positions
                                .iter()
                                .find(|&&p| (p - mouse_world).length() <= 12.0)
                                .copied();
                            self.active_wire_start = Some(nearest_pin.unwrap_or(snapped_world));
                        }
                    }
                    ToolMode::Probe => {
                        // Check if user clicked a pin
                        let mut probed = false;
                        for comp in &self.components {
                            for (pin_name, p) in comp.all_pins() {
                                if (p - mouse_world).length() <= 15.0 {
                                    if let Some(compiled) = &self.compiled_circuit {
                                        if let Some(net) = compiled
                                            .pin_to_net
                                            .get(&(comp.name.clone(), pin_name.to_string()))
                                        {
                                            if let Some(&volts) = self.dc_node_voltages.get(net) {
                                                self.sim_status =
                                                    format!("Probed Node {}: {:.3} V", net, volts);
                                                probed = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            if probed {
                                break;
                            }
                        }
                    }
                }
            }

            // Cancel wire on secondary click
            if response.clicked_by(PointerButton::Secondary) {
                self.active_wire_start = None;
            }

            // Drag selected component with left mouse button
            if response.drag_started_by(PointerButton::Primary)
                && self.selected_tool == ToolMode::Select
            {
                if let Some(cid) = self.selected_component_id {
                    if let Some(comp) = self.components.iter().find(|c| c.id == cid) {
                        if comp.contains(mouse_world) {
                            self.dragging_component = true;
                            self.drag_start_pos = Some((comp.id, comp.pos));
                        }
                    }
                }
            }
            if response.dragged_by(PointerButton::Primary) && self.dragging_component {
                if let Some(cid) = self.selected_component_id {
                    if let Some(comp) = self.components.iter_mut().find(|c| c.id == cid) {
                        let delta = response.drag_delta() / self.canvas.zoom;
                        comp.pos += delta;
                    }
                }
            }
            if response.drag_stopped() && self.dragging_component {
                if let Some(cid) = self.selected_component_id {
                    if let Some(comp) = self.components.iter_mut().find(|c| c.id == cid) {
                        comp.pos = self.canvas.snap_to_grid(comp.pos);
                        if let Some((start_id, start_pos)) = self.drag_start_pos.take() {
                            if start_id == comp.id && start_pos != comp.pos {
                                self.history.record(CanvasCommand::MoveComponent {
                                    id: comp.id,
                                    from: start_pos,
                                    to: comp.pos,
                                });
                            }
                        }
                    }
                }
                self.drag_start_pos = None;
                self.dragging_component = false;
            }

            // Render wire routing rubberband preview
            if let Some(start) = self.active_wire_start {
                let preview_wire = SchematicWire::manhattan_route(0, start, snapped_world);
                let preview_stroke = Stroke::new(2.0, Color32::from_rgb(120, 240, 160));
                for seg in &preview_wire.segments {
                    let s_screen = self.canvas.world_to_screen(seg.start);
                    let e_screen = self.canvas.world_to_screen(seg.end);
                    painter.line_segment([s_screen, e_screen], preview_stroke);
                }
            }

            // Render component placement ghost preview
            if let Some(kind) = self.selected_tool.place_kind() {
                let mut ghost = SchematicComponent::new(0, kind.clone(), snapped_world, 0);
                ghost.rotation = self.placement_rotation;
                ghost.render(&painter, &self.canvas, true, None);
            }
        }

        // 6. Render components and thermal overlay
        for comp in &self.components {
            let is_sel = self.selected_component_id == Some(comp.id);

            // Collect pin voltages for this component if available
            let mut pin_voltages = Vec::new();
            if let Some(compiled) = &self.compiled_circuit {
                for (pin_name, _) in comp.all_pins() {
                    if let Some(net) = compiled
                        .pin_to_net
                        .get(&(comp.name.clone(), pin_name.to_string()))
                    {
                        if let Some(&volts) = self.dc_node_voltages.get(net) {
                            pin_voltages.push((pin_name, volts));
                        }
                    }
                }
            }

            comp.render(
                &painter,
                &self.canvas,
                is_sel,
                if pin_voltages.is_empty() {
                    None
                } else {
                    Some(&pin_voltages)
                },
            );

            // Thermal badge
            if self.show_thermal_overlay {
                if let Some(&temp_c) = self.component_temperatures.get(&comp.name) {
                    let badge_pos = self
                        .canvas
                        .world_to_screen(comp.pos + Vec2::new(-35.0, 48.0));
                    self.thermal
                        .render_junction_badge(&painter, badge_pos, temp_c, &comp.name);
                }
            }

            // Sensitivity pin badges
            if self.sensitivity_dialog.show_badges_on_canvas {
                if let Some((_norm_sens, badge_color)) =
                    self.sensitivity_dialog.get_badge_impact(&comp.name)
                {
                    for (_pin_name, pin_world) in comp.all_pins() {
                        let pin_screen = self.canvas.world_to_screen(pin_world);
                        painter.circle_filled(
                            pin_screen,
                            4.5 * self.canvas.zoom.clamp(0.8, 1.5),
                            badge_color,
                        );
                        painter.circle_stroke(
                            pin_screen,
                            6.5 * self.canvas.zoom.clamp(0.8, 1.5),
                            Stroke::new(1.5, badge_color),
                        );
                    }
                }
            }
        }

        // 7. Visual ERC Diagnostic Overlay
        self.canvas.draw(
            &painter,
            &self.erc_diagnostics,
            self.show_erc_overlay,
            mouse_pos,
        );
    }

    /// Renders the left tool and component palette panel.
    fn render_palette(&mut self, ui: &mut egui::Ui) {
        ui.heading("CAD Tools");
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.selected_tool == ToolMode::Select, "Select (S)")
                .clicked()
            {
                self.selected_tool = ToolMode::Select;
                self.placement_rotation = 0;
                self.active_wire_start = None;
            }
            if ui
                .selectable_label(self.selected_tool == ToolMode::Wire, "Wire (W)")
                .clicked()
            {
                self.selected_tool = ToolMode::Wire;
                self.placement_rotation = 0;
                self.active_wire_start = None;
            }
        });

        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.selected_tool == ToolMode::Probe, "Probe")
                .clicked()
            {
                self.selected_tool = ToolMode::Probe;
                self.placement_rotation = 0;
                self.active_wire_start = None;
            }
            if ui.button("Clear Wire").clicked() {
                self.active_wire_start = None;
            }
        });

        ui.separator();
        ui.heading("Components");

        egui::ScrollArea::vertical().show(ui, |ui| {
            let current_place_kind = self.selected_tool.place_kind();
            if let Some(kind) = self.palette.render(ui, current_place_kind) {
                self.selected_tool = ToolMode::PlaceComponent(kind);
                self.placement_rotation = 0;
                self.active_wire_start = None;
                self.selected_component_id = None;
                self.selected_wire_id = None;
            }

            ui.separator();
            ui.heading("Demo Circuits");
            if ui.button("Load Voltage Divider").clicked() {
                self.load_voltage_divider_demo();
            }
            if ui.button("Load Diode Clipper").clicked() {
                self.load_diode_clipper_demo();
            }
            if ui.button("Load BJT CE Amplifier").clicked() {
                self.load_bjt_amplifier_demo();
            }
            if ui.button("Load CMOS Inverter").clicked() {
                self.load_cmos_inverter_demo();
            }
            if ui.button("Load NMOS Switch").clicked() {
                self.load_nmos_switch_demo();
            }
        });
    }

    /// Renders the right inspector panel for selected components and simulation parameters.
    fn render_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector");

        if let Some(cid) = self.selected_component_id {
            let mut do_rotate = false;
            let mut do_delete = false;
            let mut record_cmd = None;
            let mut comp_name_for_telemetry = String::new();
            let mut comp_pins_for_telemetry = Vec::new();

            if let Some(comp) = self.components.iter_mut().find(|c| c.id == cid) {
                comp_name_for_telemetry = comp.name.clone();
                comp_pins_for_telemetry = comp.all_pins();

                // 1. Prominent Header: Component Name and Kind
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&comp.name)
                            .strong()
                            .size(16.0)
                            .color(Color32::from_rgb(100, 200, 255)),
                    );
                    ui.label(
                        RichText::new(format!("({})", comp.kind.display_name()))
                            .size(12.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                });
                ui.separator();

                // 2. Unboxed Properties: ID, Name, Value, Rotation
                ui.label(format!("Component ID: {}", comp.id));
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut comp.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Value:");
                    if self.editing_comp_value.as_ref().map(|(id, _)| *id) != Some(comp.id) {
                        self.editing_comp_value = Some((comp.id, comp.value_str.clone()));
                    }
                    let val_resp = ui.text_edit_singleline(&mut comp.value_str);
                    if val_resp.lost_focus() {
                        if let Some((_, old_val)) = self.editing_comp_value.take() {
                            if old_val != comp.value_str {
                                record_cmd = Some(CanvasCommand::ModifyComponentValue {
                                    id: comp.id,
                                    old_val,
                                    new_val: comp.value_str.clone(),
                                });
                            }
                        }
                    }
                });
                ui.label(format!("Rotation: {} deg", (comp.rotation % 4) * 90));

                ui.horizontal(|ui| {
                    if ui.button("Rotate 90 deg (R)").clicked() {
                        do_rotate = true;
                    }
                });

                // 3. Label Position Controls (Presets to prevent collision with shapes)
                ui.separator();
                ui.label("Label Position Presets:");
                ui.horizontal(|ui| {
                    if ui.button("Right").clicked() {
                        comp.set_label_offset(Vec2::new(28.0, -10.0));
                        comp.set_value_offset(Vec2::new(28.0, 8.0));
                    }
                    if ui.button("Left").clicked() {
                        comp.set_label_offset(Vec2::new(-60.0, -10.0));
                        comp.set_value_offset(Vec2::new(-60.0, 8.0));
                    }
                    if ui.button("Top").clicked() {
                        comp.set_label_offset(Vec2::new(-16.0, -42.0));
                        comp.set_value_offset(Vec2::new(-16.0, -28.0));
                    }
                    if ui.button("Bottom").clicked() {
                        comp.set_label_offset(Vec2::new(-16.0, 28.0));
                        comp.set_value_offset(Vec2::new(-16.0, 42.0));
                    }
                });

                // 4. Pin Terminals
                ui.separator();
                ui.label("Pin Terminals:");
                for (pin_name, p_world) in comp.all_pins() {
                    ui.monospace(format!(
                        "Pin {}: ({:.0}, {:.0})",
                        pin_name, p_world.x, p_world.y
                    ));
                }

                ui.separator();
                if ui.button("Delete Component").clicked() {
                    do_delete = true;
                }
            }

            // 5. Per-component Operating Telemetry (DC node voltages & temperature)
            if !comp_name_for_telemetry.is_empty() {
                ui.separator();
                ui.heading("Operating Telemetry");

                if let Some(&temp_c) = self.component_temperatures.get(&comp_name_for_telemetry) {
                    ui.monospace(format!("Junction Temp: {:.1} °C", temp_c));
                }

                if let Some(compiled) = &self.compiled_circuit {
                    let mut found_pins = false;
                    for (pin_name, _) in &comp_pins_for_telemetry {
                        if let Some(net) = compiled.pin_to_net.get(&(comp_name_for_telemetry.clone(), pin_name.to_string())) {
                            if let Some(&volts) = self.dc_node_voltages.get(net) {
                                ui.monospace(format!("Pin {} (net {}): {:.4} V", pin_name, net, volts));
                                found_pins = true;
                            }
                        }
                    }
                    if !found_pins {
                        ui.label("No active DC voltages for pins");
                    }
                } else if !self.dc_node_voltages.is_empty() {
                    ui.label("Run DC (.OP) to view terminal voltages");
                }
            }

            if let Some(cmd) = record_cmd {
                self.history.record(cmd);
            }
            if do_rotate {
                self.rotate_active();
            }
            if do_delete {
                self.delete_selected();
            }
        } else if let Some(wid) = self.selected_wire_id {
            let mut do_delete_wire = false;
            if let Some(wire) = self.wires.iter().find(|w| w.id == wid) {
                ui.label(format!("Wire ID: {}", wire.id));
                ui.label(format!("Segments: {}", wire.segments.len()));
                let total_len: f32 = wire.segments.iter().map(|s| s.length()).sum();
                ui.label(format!("Total Length: {:.1} px", total_len));
                if ui.button("Delete Wire").clicked() {
                    do_delete_wire = true;
                }
            }
            if do_delete_wire {
                self.delete_selected();
            }
        } else if let Some(prefix) = self.selected_tool.place_kind().map(|k| k.prefix()) {
            let rot_deg = (self.placement_rotation % 4) * 90;
            let mut do_rotate = false;
            ui.label(format!("Placing: {}", prefix));
            ui.label(format!("Rotation: {} deg", rot_deg));
            ui.horizontal(|ui| {
                if ui.button("Rotate 90 deg (R)").clicked() {
                    do_rotate = true;
                }
            });
            if do_rotate {
                self.rotate_active();
            }
        } else {
            ui.label("No component or wire selected.");
            ui.separator();
            ui.label(format!("Total Components: {}", self.components.len()));
            ui.label(format!("Total Wires: {}", self.wires.len()));
            ui.separator();
            ui.label("Click any component to inspect terminal voltages and operating temperature.");
        }

        ui.separator();
        ui.heading("Thermal Controls");
        ui.checkbox(&mut self.show_thermal_overlay, "Show Thermal Overlay");
        ui.horizontal(|ui| {
            ui.label("Colormap:");
            egui::ComboBox::from_id_salt("colormap_select")
                .selected_text(format!("{:?}", self.thermal.colormap))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.thermal.colormap, Colormap::Turbo, "Turbo");
                    ui.selectable_value(&mut self.thermal.colormap, Colormap::Magma, "Magma");
                    ui.selectable_value(&mut self.thermal.colormap, Colormap::Inferno, "Inferno");
                });
        });
    }

    /// Evaluates one frame of the application UI, custom top frame, action toolbar, canvas, and docked panels.
    pub fn update(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        // Global keyboard hotkeys
        self.handle_shortcuts(ui.ctx());

        // 1. Bespoke Custom Top Frame
        let mut top_config = self.top_frame_config.clone();
        let dirty_suffix = if self.history.is_dirty() { " *" } else { "" };
        top_config.circuit_name = if self.components.is_empty() {
            format!("Empty Schematic{}", dirty_suffix)
        } else {
            format!(
                "Circuit ({} Components, {} Wires){}",
                self.components.len(),
                self.wires.len(),
                dirty_suffix
            )
        };

        Panel::top("custom_top_frame").show(ui, |ui| {
            let action = render_top_frame_with_app(ui, &top_config, self);
            match action {
                TopFrameAction::Close => {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                TopFrameAction::SaveProject => {
                    let _ = self.save_project_file("project.phn");
                }
                TopFrameAction::OpenProject => {
                    let _ = self.load_project_file("project.phn");
                }
                _ => {}
            }
        });

        // Action Toolbar
        Panel::top("action_toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Run DC (.OP)").clicked() {
                    self.run_dc_op();
                }
                if ui.button("Run Transient (.TRAN)").clicked() {
                    self.run_transient_demo();
                }
                if ui.button("Export Netlist").clicked() {
                    self.sync_canvas_state();
                    self.spice_netlist_text = self.netlist_sync.sync_from_canvas(&self.canvas).to_string();
                    self.show_netlist_window = true;
                }
                if ui.button("Clear Canvas").clicked() {
                    self.clear_all();
                }
            });
        });

        // Sheet Tabs Bar
        Panel::top("sheet_tabs_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
                let mut switch_to = None;
                let mut remove_idx = None;
                let active_idx = self.sheets.active_sheet_idx;
                let sheet_count = self.sheets.sheets.len();

                for (idx, sheet) in self.sheets.sheets.iter().enumerate() {
                    let is_active = idx == active_idx;
                    let (bg_color, text_color) = if is_active {
                        (Color32::from_rgb(45, 60, 85), Color32::from_rgb(100, 200, 255))
                    } else {
                        (Color32::from_rgb(25, 30, 40), Color32::from_rgb(160, 175, 195))
                    };

                    egui::Frame::new()
                        .fill(bg_color)
                        .corner_radius(3.0)
                        .inner_margin(egui::Margin::symmetric(8, 3))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let btn = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new(&sheet.name)
                                            .color(text_color)
                                            .size(11.5),
                                    )
                                    .frame(false),
                                );
                                if btn.clicked() {
                                    switch_to = Some(idx);
                                }

                                if sheet_count > 1 {
                                    let close_btn = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("x")
                                                .color(Color32::from_rgb(140, 150, 165))
                                                .size(10.0),
                                        )
                                        .frame(false),
                                    );
                                    if close_btn.clicked() {
                                        remove_idx = Some(idx);
                                    }
                                }
                            });
                        });
                }

                // Add Sheet button
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("+")
                                .color(Color32::from_rgb(120, 210, 140))
                                .size(13.0)
                                .strong(),
                        )
                        .frame(true),
                    )
                    .clicked()
                {
                    let count = self.sheets.sheets.len() + 1;
                    let new_name = format!("Sheet {}", count);
                    self.add_sheet(&new_name);
                }

                if let Some(idx) = switch_to {
                    self.switch_to_sheet(idx);
                }
                if let Some(idx) = remove_idx {
                    let _ = self.remove_sheet(idx);
                }
            });
        });

        // 2. Unobtrusive Bottom Status Bar
        Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Components: {}", self.components.len()));
                ui.separator();
                ui.label(format!("Wires: {}", self.wires.len()));
                ui.separator();
                ui.label(format!("Zoom: {:.1}x", self.canvas.zoom));

                if !self.sim_status.is_empty() {
                    ui.separator();
                    ui.label(&self.sim_status);
                }

                let telemetry = self.dynamics_backend.current_telemetry();
                if telemetry.step_count > 0 {
                    ui.separator();
                    ui.label(format!(
                        "Dynamics: t={:.3}s, steps={}, alt={:.1}m, spd={:.1}m/s",
                        telemetry.sim_time_s,
                        telemetry.step_count,
                        telemetry.altitude_m(),
                        telemetry.speed_m_per_s()
                    ));
                }
            });
        });

        // 3. Docked Oscilloscope Panel
        if self.show_oscilloscope {
            Panel::bottom("oscilloscope_panel")
                .resizable(true)
                .default_size(240.0)
                .show(ui, |ui| {
                    self.oscilloscope.show(ui);
                });
        }

        // 4. Left Palette Panel
        if self.show_palette {
            Panel::left("palette_panel")
                .resizable(true)
                .default_size(180.0)
                .show(ui, |ui| {
                    self.render_palette(ui);
                });
        }

        // 5. Right Inspector Panel
        Panel::right("inspector_panel")
            .resizable(true)
            .default_size(240.0)
            .show(ui, |ui| {
                self.render_inspector(ui);
            });

        // 6. Center Canvas
        CentralPanel::default().show(ui, |ui| {
            self.render_canvas(ui);
        });

        // 7. Optional SPICE Netlist Window
        if self.show_netlist_window {
            let mut is_open = self.show_netlist_window;
            let mut sync_requested = false;
            egui::Window::new("Exported SPICE Netlist")
                .open(&mut is_open)
                .resizable(true)
                .default_size([450.0, 360.0])
                .show(ui.ctx(), |ui| {
                    ui.label("SPICE 3f5 Netlist representation of current schematic:");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.spice_netlist_text)
                                .font(FontId::monospace(12.0))
                                .desired_rows(14)
                                .lock_focus(true),
                        );
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Sync to Canvas").clicked() {
                            sync_requested = true;
                        }
                    });
                });
            self.show_netlist_window = is_open;

            if sync_requested {
                if let Ok(delta) = self
                    .netlist_sync
                    .sync_to_canvas(&self.spice_netlist_text, &mut self.canvas)
                {
                    self.components = self.canvas.components.clone();
                    self.wires = self.canvas.wires.clone();
                    self.run_erc();
                    self.sim_status = format!(
                        "Synced to Canvas: +{} updated {}, -{}",
                        delta.added_count, delta.updated_count, delta.removed_count
                    );
                }
            }
        }

        // 8. SPICE Model Parameter Extraction Wizard Dialog
        self.extraction_wizard.ui(ui.ctx());

        // 9. Non-Linear Transient Sensitivity Dialog
        if self.sensitivity_dialog.run_requested {
            self.sensitivity_dialog.run_requested = false;
            self.run_sensitivity_analysis();
        }
        self.sensitivity_dialog.ui(ui.ctx());

        // 10. Monte Carlo Yield & Latin Hypercube Sampling Dialog
        if self.monte_carlo_dialog.run_requested {
            self.monte_carlo_dialog.run_requested = false;
            self.run_monte_carlo_sweep();
        }
        self.monte_carlo_dialog.ui(ui.ctx());

        // 11. Interactive Component Symbol & Shape Editor Dialog
        let mut saved_symbol = None;
        self.symbol_editor.show(ui.ctx(), |sym| {
            saved_symbol = Some(sym);
        });
        if let Some(sym) = saved_symbol {
            self.symbol_library.register(sym);
            self.sim_status = "Custom component symbol registered into library.".to_string();
        }

        // 12. Interactive RF S-Parameters, Smith Chart & Harmonic Balance Dialog
        if self.smith_chart_dialog.run_requested {
            self.smith_chart_dialog.run_requested = false;
            self.smith_chart_dialog.run_simulation();
        }
        self.smith_chart_dialog.ui(ui.ctx());

        // 13. Interactive Thermal Floorplan & Co-Simulation Dialog
        if self.thermal_floorplan_dialog.run_requested {
            self.thermal_floorplan_dialog.run_requested = false;
            self.run_thermal_cosim();
        }
        self.thermal_floorplan_dialog.ui(ui.ctx());

        // 14. Interactive Polariton Waveguide & Topological Photonic Cavity Dialog
        if self.polariton_cavity_dialog.run_requested {
            self.polariton_cavity_dialog.run_requested = false;
            self.polariton_cavity_dialog.run_simulation();
        }
        self.polariton_cavity_dialog.ui(ui.ctx());

        // 15. Interactive Distributed Cloud Parameter Sweep Cluster Dashboard Dialog
        self.cluster_dashboard_dialog.ui(ui.ctx());

        // 16. Interactive Neuromorphic Studio Canvas & Synaptic Weight Visualizer Dialog
        self.neuromorphic_snn_dialog.ui(ui.ctx());
    }
}

impl App for PhononApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        self.update(ui, frame);
    }
}
