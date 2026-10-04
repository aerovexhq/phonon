#![deny(unsafe_code)]

//! Interactive CAD Studio Thermal Floorplan & Heatmap Visualizer.
//!
//! Provides 2D semiconductor layout canvas rendering, colored thermography overlay
//! (Turbo, Magma, Inferno colormaps), marching squares isothermal contour line overlays,
//! peak hotspot telemetry, time-step playback scrubber, and floorplan component auto-placement.

use crate::thermal::heatmap::{sample_colormap, Colormap};
use egui::{
    pos2, vec2, Align2, Color32, FontId, Pos2, Rect, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, Ui, Vec2,
};
use phonon_core::{CircuitGraph, ComponentRecord};
use phonon_models::diode::DiodeModel;
use phonon_solver::mna::ModelContext;
use phonon_thermal::floorplan::{DieProperties, DynamicFloorplanMesh, FloorplanComponent};
use phonon_thermal::transient_co_sim::{
    ElectroThermalCoSimulator, ElectroThermalTransientTrajectory,
};

/// Interactive modal dialog for thermal floorplan visualization and electro-thermal co-simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct ThermalFloorplanDialog {
    /// Visibility flag of the modal dialog window.
    pub is_open: bool,
    /// 2D finite difference thermal mesh.
    pub mesh: DynamicFloorplanMesh,
    /// Recorded transient electro-thermal simulation trajectory.
    pub trajectory: Option<ElectroThermalTransientTrajectory>,
    /// Selected playback step index in the recorded trajectory.
    pub selected_step_idx: usize,
    /// Configured simulation integration time step Delta t in seconds.
    pub dt_s: f64,
    /// Configured simulation stop time t_stop in seconds.
    pub t_stop_s: f64,
    /// Selected scientific colormap.
    pub colormap: Colormap,
    /// Overlay toggle for thermography heatmap.
    pub show_heatmap: bool,
    /// Overlay toggle for isothermal contour lines.
    pub show_contours: bool,
    /// Overlay toggle for component layout outlines and labels.
    pub show_components: bool,
    /// Overlay toggle for peak hotspot reticle target.
    pub show_hotspot_reticle: bool,
    /// Number of isothermal contour intervals to extract.
    pub contour_count: usize,
    /// Thermal runaway safety threshold in Kelvin.
    pub runaway_threshold_k: f64,
    /// Status notification text.
    pub status_msg: String,
    /// Flag indicating whether thermal runaway was detected.
    pub runaway_detected: bool,
    /// Component triggering runaway if breached.
    pub runaway_component: Option<String>,
    /// Component experiencing highest thermal stress.
    pub highest_stress_component: Option<String>,
    /// Flag signaling app orchestrator to execute co-simulation.
    pub run_requested: bool,
}

impl Default for ThermalFloorplanDialog {
    fn default() -> Self {
        let die = DieProperties {
            width: 0.005,       // 5.0 mm
            height: 0.005,      // 5.0 mm
            thickness: 0.0003,  // 300 um
            ambient_temperature: 300.0,
            h_conv: 15.0,
            ..Default::default()
        };

        let default_comps = vec![
            FloorplanComponent::new("D1", 0.001, 0.0015, 0.0008, 0.0008),
            FloorplanComponent::new("R1", 0.003, 0.0015, 0.0010, 0.0008),
            FloorplanComponent::new("M1", 0.002, 0.0032, 0.0012, 0.0010),
        ];

        let mesh = DynamicFloorplanMesh::new(die, 16, 16, default_comps);

        let mut dialog = Self {
            is_open: false,
            mesh,
            trajectory: None,
            selected_step_idx: 0,
            dt_s: 1.0e-4,    // 100 us
            t_stop_s: 0.01,  // 10 ms
            colormap: Colormap::Turbo,
            show_heatmap: true,
            show_contours: true,
            show_components: true,
            show_hotspot_reticle: true,
            contour_count: 5,
            runaway_threshold_k: 500.0,
            status_msg: "Ready. Click 'Run Co-Simulation' to begin electro-thermal solving.".to_string(),
            runaway_detected: false,
            runaway_component: None,
            highest_stress_component: None,
            run_requested: false,
        };

        // Populate baseline pre-seeded trajectory so visualizer opens with active thermal data without running heavy PDE solves during cold boot
        let mut traj = ElectroThermalTransientTrajectory::new();
        let mut comp_powers = std::collections::HashMap::new();
        comp_powers.insert("D1".to_string(), 0.0);
        comp_powers.insert("R1".to_string(), 0.0);
        comp_powers.insert("M1".to_string(), 0.0);
        let mut comp_temps = std::collections::HashMap::new();
        comp_temps.insert("D1".to_string(), 300.0);
        comp_temps.insert("R1".to_string(), 300.0);
        comp_temps.insert("M1".to_string(), 300.0);
        traj.timestamps.push(0.0);
        traj.records.push(phonon_thermal::transient_co_sim::TransientStepRecord {
            time_s: 0.0,
            node_voltages: vec![12.0, 0.7, 0.0],
            component_powers: comp_powers,
            component_temperatures: comp_temps,
            grid_temperatures: dialog.mesh.temperatures.clone(),
            peak_temp_k: 300.0,
            peak_pos: (0.0025, 0.0025),
            is_runaway: false,
        });
        dialog.trajectory = Some(traj);
        dialog
    }
}

impl ThermalFloorplanDialog {
    /// Constructs a new ThermalFloorplanDialog in closed state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Automatically places schematic components onto the semiconductor die.
    pub fn auto_place_components(&mut self, graph: &CircuitGraph) {
        let mut candidate_names = Vec::new();

        for comp in graph.components() {
            match comp {
                ComponentRecord::Diode { name, .. }
                | ComponentRecord::Resistor { name, .. }
                | ComponentRecord::Mosfet { name, .. }
                | ComponentRecord::Bjt { name, .. } => {
                    candidate_names.push(name.clone());
                }
                _ => {}
            }
        }

        if candidate_names.is_empty() {
            candidate_names = vec!["D1".to_string(), "R1".to_string(), "M1".to_string()];
        }

        let n = candidate_names.len();
        let cols = (n as f64).sqrt().ceil() as usize;
        let rows = ((n as f64) / (cols as f64)).ceil() as usize;

        let w_die = self.mesh.die.width;
        let h_die = self.mesh.die.height;

        let cell_w = w_die / ((cols + 1) as f64);
        let cell_h = h_die / ((rows + 1) as f64);

        let comp_w = (cell_w * 0.7).min(0.0015);
        let comp_h = (cell_h * 0.7).min(0.0015);

        let mut placed = Vec::with_capacity(n);
        for (idx, name) in candidate_names.into_iter().enumerate() {
            let col = idx % cols;
            let row = idx / cols;

            let cx = (col as f64 + 0.65) * cell_w;
            let cy = (row as f64 + 0.65) * cell_h;

            placed.push(FloorplanComponent::new(name, cx, cy, comp_w, comp_h));
        }

        self.mesh.components = placed;
        self.mesh.temperatures.fill(self.mesh.die.ambient_temperature);
        self.status_msg = format!("Auto-placed {} components across the semiconductor die.", n);
    }

    /// Runs a default demo electro-thermal simulation to initialize the canvas.
    pub fn run_demo_simulation(&mut self) {
        let mut graph = CircuitGraph::new();
        let _ = graph.add_voltage_source("V1", "n_in", "0", 12.0);
        let _ = graph.add_resistor("R1", "n_in", "n_mid", 40.0);
        let _ = graph.add_diode("D1", "n_mid", "0");

        let mut ctx = ModelContext::new();
        ctx.temperature_kelvin = 300.0;
        ctx.set_diode_model("D1", DiodeModel::default());

        self.run_simulation(&graph, &ctx);
    }

    /// Executes transient electro-thermal co-simulation with the provided circuit and context.
    pub fn run_simulation(&mut self, graph: &CircuitGraph, ctx: &ModelContext) {
        let mut co_sim = ElectroThermalCoSimulator::new(graph.clone(), self.mesh.clone())
            .with_runaway_threshold(self.runaway_threshold_k);

        match co_sim.simulate(self.t_stop_s, self.dt_s, ctx) {
            Ok(traj) => {
                let steps = traj.len();
                let peak_k = traj.peak_die_temp_k;
                self.runaway_detected = false;
                self.runaway_component = None;

                // Identify highest thermal stress component
                if let Some(last) = traj.last_record() {
                    let max_comp = last
                        .component_temperatures
                        .iter()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(k, _)| k.clone());
                    self.highest_stress_component = max_comp;
                }

                self.mesh = co_sim.mesh;
                self.selected_step_idx = steps.saturating_sub(1);
                self.status_msg = format!(
                    "Co-simulation converged: {} steps, peak temperature {:.1} K ({:.1} °C).",
                    steps,
                    peak_k,
                    peak_k - 273.15
                );
                self.trajectory = Some(traj);
            }
            Err(err) => {
                self.runaway_detected = true;
                self.runaway_component = co_sim.trajectory.runaway_component.clone();
                let steps = co_sim.trajectory.len();
                self.mesh = co_sim.mesh;
                self.selected_step_idx = steps.saturating_sub(1);
                self.status_msg = format!("Warning: {}", err);
                self.trajectory = Some(co_sim.trajectory);
            }
        }
    }

    /// Renders the modal window UI.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Thermal Floorplan & Co-Simulation Studio")
            .open(&mut is_open)
            .default_size([920.0, 640.0])
            .min_size([700.0, 500.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the complete dialog contents.
    pub fn render_content(&mut self, ui: &mut Ui) {
        // 1. Top Telemetry & Status Notification Bar
        ui.horizontal(|ui| {
            if self.runaway_detected {
                let badge = format!(
                    "[THERMAL RUNAWAY DETECTED: {}]",
                    self.runaway_component
                        .as_deref()
                        .unwrap_or("Critical Junction")
                );
                ui.label(
                    RichText::new(badge)
                        .color(Color32::from_rgb(255, 60, 60))
                        .strong()
                        .size(12.0),
                );
            } else {
                ui.label(
                    RichText::new("[THERMALLY STABLE]")
                        .color(Color32::from_rgb(80, 220, 120))
                        .strong()
                        .size(12.0),
                );
            }

            ui.label(
                RichText::new(&self.status_msg)
                    .color(Color32::from_rgb(180, 210, 240))
                    .size(11.5),
            );
        });
        ui.separator();

        // 2. Control Toolbar
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(
                    RichText::new("Run Co-Simulation")
                        .color(Color32::from_rgb(100, 220, 255))
                        .strong(),
                )
                .clicked()
            {
                self.run_requested = true;
            }

            if ui.button("Auto-Place Components").clicked() {
                let empty_graph = CircuitGraph::new();
                self.auto_place_components(&empty_graph);
            }

            ui.label("|");

            ui.label("dt:");
            ui.add(
                egui::DragValue::new(&mut self.dt_s)
                    .range(1.0e-6..=0.01)
                    .speed(1.0e-5)
                    .custom_formatter(|val, _| {
                        if val >= 1.0e-3 {
                            format!("{:.2} ms", val * 1.0e3)
                        } else {
                            format!("{:.0} us", val * 1.0e6)
                        }
                    }),
            );

            ui.label("t_stop:");
            ui.add(
                egui::DragValue::new(&mut self.t_stop_s)
                    .range(1.0e-4..=1.0)
                    .speed(1.0e-3)
                    .custom_formatter(|val, _| format!("{:.1} ms", val * 1.0e3)),
            );

            ui.label("|");

            egui::ComboBox::from_label("Colormap")
                .selected_text(match self.colormap {
                    Colormap::Turbo => "Turbo",
                    Colormap::Magma => "Magma",
                    Colormap::Inferno => "Inferno",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.colormap, Colormap::Turbo, "Turbo");
                    ui.selectable_value(&mut self.colormap, Colormap::Magma, "Magma");
                    ui.selectable_value(&mut self.colormap, Colormap::Inferno, "Inferno");
                });

            ui.checkbox(&mut self.show_heatmap, "Heatmap");
            ui.checkbox(&mut self.show_contours, "Contours");
            ui.checkbox(&mut self.show_components, "Components");
            ui.checkbox(&mut self.show_hotspot_reticle, "Hotspot");
        });
        ui.separator();

        // 3. Main Workspace: Canvas on Left, Telemetry & Inspector on Right
        ui.columns(2, |columns| {
            // Left Column: 2D Die Floorplan Canvas
            columns[0].vertical(|ui| {
                ui.heading(
                    RichText::new("2D Silicon Die Thermography")
                        .color(Color32::from_rgb(220, 235, 255))
                        .size(13.0),
                );
                self.render_floorplan_canvas(ui);

                // Playback scrubber below canvas
                if let Some(ref traj) = self.trajectory {
                    let count = traj.len();
                    if count > 1 {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Time Scrubber:")
                                    .color(Color32::from_rgb(180, 200, 220))
                                    .size(11.0),
                            );
                            let max_step = count.saturating_sub(1);
                            ui.add(egui::Slider::new(&mut self.selected_step_idx, 0..=max_step));
                            if let Some(rec) = traj.records.get(self.selected_step_idx) {
                                ui.label(
                                    RichText::new(format!("t = {:.3} ms", rec.time_s * 1000.0))
                                        .color(Color32::from_rgb(100, 220, 255))
                                        .size(11.0),
                                );
                            }
                        });
                    }
                }
            });

            // Right Column: Hotspot Telemetry & Component Thermal Stress Table
            columns[1].vertical(|ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.render_telemetry_panel(ui);
                });
            });
        });
    }

    /// Renders the interactive 2D semiconductor layout canvas.
    fn render_floorplan_canvas(&mut self, ui: &mut Ui) {
        let canvas_dim = ui.available_width().clamp(260.0, 420.0);
        let (response, painter) = ui.allocate_painter(Vec2::splat(canvas_dim), Sense::hover());
        let rect = response.rect;

        let margin = 20.0f32;
        let draw_w = (rect.width() - 2.0 * margin) as f64;
        let draw_h = (rect.height() - 2.0 * margin) as f64;

        let die_w = self.mesh.die.width;
        let die_h = self.mesh.die.height;

        let scale = (draw_w / die_w).min(draw_h / die_h);
        let ox = rect.min.x + margin + ((draw_w - die_w * scale) * 0.5) as f32;
        let oy = rect.min.y + margin + ((draw_h - die_h * scale) * 0.5) as f32;

        // Coordinate transformation: physical (x, y) meters -> screen coordinates Pos2
        let to_screen = |x: f64, y: f64| -> Pos2 {
            let sx = ox + (x * scale) as f32;
            let sy = oy + ((die_h - y) * scale) as f32; // Invert Y for screen space
            pos2(sx, sy)
        };

        // 1. Die Substrate Border and Background
        let die_rect = Rect::from_min_max(to_screen(0.0, die_h), to_screen(die_w, 0.0));
        painter.rect_filled(die_rect, 4.0, Color32::from_rgb(16, 22, 32));
        painter.rect_stroke(
            die_rect,
            4.0,
            Stroke::new(2.0, Color32::from_rgb(80, 100, 130)),
            StrokeKind::Outside,
        );

        // Fetch temperature field for currently selected playback step
        let temps: Vec<f64> = if let Some(ref traj) = self.trajectory {
            if let Some(rec) = traj.records.get(self.selected_step_idx) {
                rec.grid_temperatures.clone()
            } else {
                self.mesh.temperatures.clone()
            }
        } else {
            self.mesh.temperatures.clone()
        };

        let min_t = temps.iter().copied().fold(f64::INFINITY, f64::min);
        let max_t = temps.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let span_t = (max_t - min_t).max(1.0);

        let nx = self.mesh.nx;
        let ny = self.mesh.ny;
        let dx = self.mesh.dx();
        let dy = self.mesh.dy();

        // 2. Colormap Thermography Heatmap Overlay
        if self.show_heatmap {
            for j in 0..ny {
                for i in 0..nx {
                    let idx = j * nx + i;
                    let t_val = temps[idx];
                    let u = ((t_val - min_t) / span_t).clamp(0.0, 1.0) as f32;
                    let color = sample_colormap(u, self.colormap);

                    let c0 = to_screen((i as f64) * dx, ((j + 1) as f64) * dy);
                    let c1 = to_screen(((i + 1) as f64) * dx, (j as f64) * dy);
                    let cell_rect = Rect::from_min_max(c0, c1);

                    painter.rect_filled(cell_rect, 0.0, color);
                }
            }
        }

        // 3. Marching Squares Isothermal Contour Overlays
        if self.show_contours && span_t > 0.5 {
            let mut contour_temps = Vec::with_capacity(self.contour_count);
            for k in 1..=self.contour_count {
                let frac = (k as f64) / ((self.contour_count + 1) as f64);
                contour_temps.push(min_t + frac * span_t);
            }

            // Extract contours from current playback step mesh
            let mut temp_mesh = self.mesh.clone();
            temp_mesh.temperatures = temps.clone();
            let contours = temp_mesh.compute_isothermal_contours(&contour_temps);

            for c in contours {
                let stroke_color = Color32::from_rgba_unmultiplied(255, 255, 255, 140);
                for (p1, p2) in c.segments {
                    painter.line_segment(
                        [to_screen(p1.0, p1.1), to_screen(p2.0, p2.1)],
                        Stroke::new(1.2, stroke_color),
                    );
                }
            }
        }

        // 4. Component Layout Outlines and Badges
        if self.show_components {
            for comp in &self.mesh.components {
                let p0 = to_screen(comp.x, comp.y + comp.height);
                let p1 = to_screen(comp.x + comp.width, comp.y);
                let comp_rect = Rect::from_min_max(p0, p1);

                // Semi-transparent component package background
                painter.rect_filled(
                    comp_rect,
                    2.0,
                    Color32::from_rgba_unmultiplied(20, 30, 45, 180),
                );
                painter.rect_stroke(
                    comp_rect,
                    2.0,
                    Stroke::new(1.5, Color32::from_rgb(0, 210, 255)),
                    StrokeKind::Inside,
                );

                // Component Identifier Tag
                painter.text(
                    comp_rect.center(),
                    Align2::CENTER_CENTER,
                    &comp.name,
                    FontId::monospace(10.0),
                    Color32::WHITE,
                );
            }
        }

        // 5. Peak Hotspot Reticle Target
        if self.show_hotspot_reticle {
            let (peak_t, peak_x, peak_y) = if let Some(ref traj) = self.trajectory {
                if let Some(rec) = traj.records.get(self.selected_step_idx) {
                    (rec.peak_temp_k, rec.peak_pos.0, rec.peak_pos.1)
                } else {
                    self.mesh.peak_temperature()
                }
            } else {
                self.mesh.peak_temperature()
            };

            let peak_screen = to_screen(peak_x, peak_y);
            let reticle_color = if peak_t >= self.runaway_threshold_k {
                Color32::from_rgb(255, 40, 40)
            } else {
                Color32::from_rgb(255, 180, 20)
            };

            painter.circle_stroke(peak_screen, 9.0, Stroke::new(1.8, reticle_color));
            painter.line_segment(
                [peak_screen - vec2(13.0, 0.0), peak_screen + vec2(13.0, 0.0)],
                Stroke::new(1.2, reticle_color),
            );
            painter.line_segment(
                [peak_screen - vec2(0.0, 13.0), peak_screen + vec2(0.0, 13.0)],
                Stroke::new(1.2, reticle_color),
            );
        }
    }

    /// Renders the hotspot telemetry and component thermal stress inspector panel.
    fn render_telemetry_panel(&mut self, ui: &mut Ui) {
        let (peak_t, peak_x, peak_y) = if let Some(ref traj) = self.trajectory {
            if let Some(rec) = traj.records.get(self.selected_step_idx) {
                (rec.peak_temp_k, rec.peak_pos.0, rec.peak_pos.1)
            } else {
                self.mesh.peak_temperature()
            }
        } else {
            self.mesh.peak_temperature()
        };

        ui.heading(
            RichText::new("Thermal Hotspot Telemetry")
                .color(Color32::from_rgb(220, 235, 255))
                .size(13.0),
        );

        egui::Grid::new("telemetry_grid")
            .striped(true)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.label("Peak Die Temperature:");
                let temp_color = if peak_t >= self.runaway_threshold_k {
                    Color32::from_rgb(255, 60, 60)
                } else if peak_t >= 373.15 {
                    Color32::from_rgb(255, 180, 40)
                } else {
                    Color32::from_rgb(100, 220, 255)
                };
                ui.label(
                    RichText::new(format!(
                        "{:.1} K ({:.1} °C)",
                        peak_t,
                        peak_t - 273.15
                    ))
                    .color(temp_color)
                    .strong(),
                );
                ui.end_row();

                ui.label("Hotspot Coordinates (x, y):");
                ui.label(format!("{:.2} mm, {:.2} mm", peak_x * 1000.0, peak_y * 1000.0));
                ui.end_row();

                ui.label("Highest Thermal Stress Device:");
                ui.label(
                    RichText::new(
                        self.highest_stress_component
                            .as_deref()
                            .unwrap_or("None"),
                    )
                    .color(Color32::from_rgb(255, 200, 100)),
                );
                ui.end_row();

                ui.label("Thermal Runaway Margin:");
                let margin_k = self.runaway_threshold_k - peak_t;
                let margin_color = if margin_k <= 0.0 {
                    Color32::from_rgb(255, 50, 50)
                } else if margin_k < 50.0 {
                    Color32::from_rgb(255, 160, 30)
                } else {
                    Color32::from_rgb(80, 220, 120)
                };
                ui.label(
                    RichText::new(format!("{:.1} K to threshold", margin_k))
                        .color(margin_color),
                );
                ui.end_row();
            });

        ui.separator();

        // Component Thermal Stress Table
        ui.heading(
            RichText::new("Placed Components & Junction Thermal Status")
                .color(Color32::from_rgb(220, 235, 255))
                .size(13.0),
        );

        egui::Grid::new("comp_table_grid")
            .striped(true)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Name").strong());
                ui.label(RichText::new("Footprint").strong());
                ui.label(RichText::new("Dissipation").strong());
                ui.label(RichText::new("Junction Temp").strong());
                ui.label(RichText::new("Status").strong());
                ui.end_row();

                for comp in &self.mesh.components {
                    ui.label(&comp.name);
                    ui.label(format!(
                        "{:.1} x {:.1} mm",
                        comp.width * 1000.0,
                        comp.height * 1000.0
                    ));

                    // Get power and temperature from current playback step
                    let (p_w, t_k) = if let Some(ref traj) = self.trajectory {
                        if let Some(rec) = traj.records.get(self.selected_step_idx) {
                            let p = rec.component_powers.get(&comp.name).copied().unwrap_or(0.0);
                            let t = rec
                                .component_temperatures
                                .get(&comp.name)
                                .copied()
                                .unwrap_or(300.0);
                            (p, t)
                        } else {
                            (0.0, 300.0)
                        }
                    } else {
                        (0.0, 300.0)
                    };

                    ui.label(format!("{:.2} W", p_w));
                    ui.label(format!("{:.1} K ({:.1} °C)", t_k, t_k - 273.15));

                    if t_k >= self.runaway_threshold_k {
                        ui.label(RichText::new("CRITICAL").color(Color32::from_rgb(255, 40, 40)));
                    } else if t_k >= 373.15 {
                        ui.label(RichText::new("WARM").color(Color32::from_rgb(255, 180, 40)));
                    } else {
                        ui.label(RichText::new("NOMINAL").color(Color32::from_rgb(80, 220, 120)));
                    }
                    ui.end_row();
                }
            });

        ui.separator();

        // Physical Die Transport Specifications
        ui.collapsing("Die Physics & Boundary Conditions", |ui| {
            let die = &self.mesh.die;
            ui.label(format!(
                "Die Footprint: {:.2} mm x {:.2} mm x {:.0} um",
                die.width * 1000.0,
                die.height * 1000.0,
                die.thickness * 1.0e6
            ));
            ui.label(format!("Substrate Material: Silicon (k = {:.1} W/(m*K))", die.k));
            ui.label(format!(
                "Volumetric Heat Capacity: {:.2e} J/(m^3*K)",
                die.volumetric_heat_capacity
            ));
            ui.label(format!(
                "Top Convective Coefficient: {:.1} W/(m^2*K)",
                die.h_conv
            ));
            ui.label(format!("Ambient Temperature: {:.1} K", die.ambient_temperature));
        });
    }
}
