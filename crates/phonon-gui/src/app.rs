#![deny(unsafe_code)]

//! The central Phonon GUI application orchestrator, CAD layout, and interactive simulation.

use crate::oscilloscope::{OscilloscopePanel, WaveformTrace};
use crate::schematic::{
    compile_schematic, compute_junction_dots, CanvasCommand, CompiledCircuit, ComponentKind,
    HistoryStack, SchematicCanvas, SchematicComponent, SchematicWire,
};
use crate::thermal::{Colormap, ThermalOverlay};
use crate::widgets::{
    render_top_frame_with_app, ComponentPalette, DynamicsStatusBadge, TopFrameAction,
    TopFrameConfig,
};
use eframe::{App, Frame};
use egui::{
    CentralPanel, Color32, FontId, Key, Panel, PointerButton, Pos2, Sense, Stroke, Ui, Vec2,
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
            .finish()
    }
}

impl Default for PhononApp {
    fn default() -> Self {
        let mut app = Self {
            canvas: SchematicCanvas::new(),
            components: Vec::new(),
            wires: Vec::new(),
            next_comp_id: 1,
            next_wire_id: 1,
            selected_tool: ToolMode::Select,
            selected_component_id: None,
            selected_wire_id: None,
            active_wire_start: None,
            oscilloscope: OscilloscopePanel::new(),
            thermal: ThermalOverlay::new(),
            show_oscilloscope: true,
            show_thermal_overlay: true,
            show_netlist_window: false,
            sim_status: String::new(),
            dc_node_voltages: HashMap::new(),
            component_temperatures: HashMap::new(),
            compiled_circuit: None,
            spice_netlist_text: String::new(),
            dragging_component: false,
            drag_start_pos: None,
            history: HistoryStack::new(),
            editing_comp_value: None,
            placement_rotation: 0,
            top_frame_config: TopFrameConfig::default(),
            dynamics_backend: Box::new(AutoSelectingDynamicsBackend::new()),
            palette: ComponentPalette::new(),
        };

        // Initialize with default Voltage Divider demo
        app.load_voltage_divider_demo();
        app.history.clear();
        app
    }
}

impl PhononApp {
    /// Creates a default `PhononApp` with an auto-selecting dynamics backend.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// Creates a `PhononApp` instance with an injected custom physics dynamics backend.
    pub fn with_backend(
        _cc: &eframe::CreationContext<'_>,
        backend: Box<dyn PhysicsDynamicsBackend>,
    ) -> Self {
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
        }
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
        });
    }

    /// Renders the right inspector panel for selected components and simulation parameters.
    fn render_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector");

        if let Some(cid) = self.selected_component_id {
            let mut do_rotate = false;
            let mut do_delete = false;
            let mut record_cmd = None;
            if let Some(comp) = self.components.iter_mut().find(|c| c.id == cid) {
                ui.group(|ui| {
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

                    ui.separator();
                    ui.label("Pin Terminals:");
                    for (pin_name, p_world) in comp.all_pins() {
                        ui.monospace(format!(
                            "Pin {}: ({:.0}, {:.0})",
                            pin_name, p_world.x, p_world.y
                        ));
                    }
                });

                if ui.button("Delete Component").clicked() {
                    do_delete = true;
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
                ui.group(|ui| {
                    ui.label(format!("Wire ID: {}", wire.id));
                    ui.label(format!("Segments: {}", wire.segments.len()));
                    let total_len: f32 = wire.segments.iter().map(|s| s.length()).sum();
                    ui.label(format!("Total Length: {:.1} px", total_len));
                });
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
            ui.group(|ui| {
                ui.label(format!("Placing: {}", prefix));
                ui.label(format!("Rotation: {} deg", rot_deg));
                ui.horizontal(|ui| {
                    if ui.button("Rotate 90 deg (R)").clicked() {
                        do_rotate = true;
                    }
                });
            });
            if do_rotate {
                self.rotate_active();
            }
        } else {
            ui.label("No component or wire selected.");
            ui.separator();
            ui.label(format!("Total Components: {}", self.components.len()));
            ui.label(format!("Total Wires: {}", self.wires.len()));

            if !self.dc_node_voltages.is_empty() {
                ui.separator();
                ui.heading("DC Node Voltages");
                for (node, &volts) in &self.dc_node_voltages {
                    ui.monospace(format!("{}: {:.4} V", node, volts));
                }
            }

            if !self.component_temperatures.is_empty() {
                ui.separator();
                ui.heading("Component Temperatures");
                for (comp_name, &temp_c) in &self.component_temperatures {
                    ui.monospace(format!("{}: {:.1} °C", comp_name, temp_c));
                }
            }
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
            if action == TopFrameAction::Close {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
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
                    if let Ok(compiled) = compile_schematic(&self.components, &self.wires) {
                        self.spice_netlist_text = compiled.spice_netlist;
                    }
                    self.show_netlist_window = true;
                }
                if ui.button("Clear Canvas").clicked() {
                    self.clear_all();
                }

                ui.separator();
                DynamicsStatusBadge::new().ui(ui, self.dynamics_backend.as_ref());
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
        Panel::left("palette_panel")
            .resizable(true)
            .default_size(180.0)
            .show(ui, |ui| {
                self.render_palette(ui);
            });

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
            egui::Window::new("Exported SPICE Netlist")
                .open(&mut self.show_netlist_window)
                .resizable(true)
                .default_size([450.0, 320.0])
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
                });
        }
    }
}

impl App for PhononApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        self.update(ui, frame);
    }
}
