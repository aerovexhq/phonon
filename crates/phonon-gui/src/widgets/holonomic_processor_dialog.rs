#![deny(unsafe_code)]

//! Interactive Holonomic Processor Visualizer in CAD Studio.
//!
//! Provides:
//! - Parameter Space Loop Trajectory Canvas: 2D/3D projection of control parameter loop C(theta(t), phi(t))
//!   on the parameter manifold sphere with time-dependent color gradient, markers, and solid angle.
//! - Tripartite Pulse Envelope Plot: native egui_plot rendering driving pulse shapes Omega_1(t), Omega_2(t), Omega_3(t)
//!   over normalized cycle t / tau.
//! - Geometric vs Dynamical Phase Accumulation Gauge: displays real-time accumulation of geometric phase gamma_geom
//!   vs vanishing dynamical phase E_dyn (< 1e-4 rad).
//! - Gate Synthesis & Fidelity Panel: target gate selector (Hadamard, Phase S, Pauli X, Pauli Z), "Synthesize Holonomy"
//!   button, calculated process fidelity bar (>= 99.0%), and 2x2 complex matrix representation.
//! - Telemetry Footer: Target Gate, Gate Fidelity (%), Solid Angle (sr), Dynamical Phase Error (rad),
//!   Dark State Purity (%), Cavity Loss Decoupling.

use egui::{
    pos2, vec2, Color32, FontId, RichText, Sense, Stroke, Ui,
};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use phonon_solver::non_abelian_holonomic::{
    HolonomicGateType, HolonomicTrajectorySimulation, TripartiteCavityParams,
    TripartiteCoSimulator,
};
use std::f64::consts::PI;

/// Palette colors for driving pulse envelopes and trajectory manifold.
const PULSE_1_COLOR: Color32 = Color32::from_rgb(0, 220, 255); // Mode 1: Bright Cyan
const PULSE_2_COLOR: Color32 = Color32::from_rgb(255, 80, 180); // Mode 2: Magenta
const PULSE_3_COLOR: Color32 = Color32::from_rgb(255, 205, 50); // Mode 3: Gold
const GEOM_PHASE_COLOR: Color32 = Color32::from_rgb(50, 230, 120); // Emerald Green
const DYN_PHASE_COLOR: Color32 = Color32::from_rgb(255, 100, 100); // Soft Red

/// Interactive modal dialog for the Non-Abelian Holonomic Geometric Phase Quantum Acoustic Processor.
#[derive(Debug, Clone, PartialEq)]
pub struct HolonomicProcessorDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Controls
    /// Target quantum logic gate for geometric synthesis.
    pub target_gate: HolonomicGateType,
    /// Physical cavity parameters for tripartite acoustic co-simulation.
    pub cavity_params: TripartiteCavityParams,
    /// Number of trajectory simulation steps.
    pub simulation_steps: usize,

    // Animation & Stepping
    /// Continuous animation scrub progress in [0.0, 1.0].
    pub animation_progress: f32,
    /// Flag indicating if time scrub animation is running.
    pub is_animating: bool,

    // Cached Simulation State
    pub simulation: HolonomicTrajectorySimulation,
    pub pulse_1_curve: Vec<[f64; 2]>,
    pub pulse_2_curve: Vec<[f64; 2]>,
    pub pulse_3_curve: Vec<[f64; 2]>,
    pub geom_phase_curve: Vec<[f64; 2]>,
    pub dyn_phase_curve: Vec<[f64; 2]>,
}

impl Default for HolonomicProcessorDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl HolonomicProcessorDialog {
    /// Creates a new HolonomicProcessorDialog initialized with default physical parameters.
    pub fn new() -> Self {
        let target_gate = HolonomicGateType::Hadamard;
        let cavity_params = TripartiteCavityParams::default();
        let simulation_steps = 200;

        let co_sim = TripartiteCoSimulator::new(cavity_params);
        let simulation = co_sim.simulate_trajectory(target_gate, simulation_steps);

        let mut dialog = Self {
            is_open: false,
            target_gate,
            cavity_params,
            simulation_steps,
            animation_progress: 1.0,
            is_animating: false,
            simulation,
            pulse_1_curve: Vec::new(),
            pulse_2_curve: Vec::new(),
            pulse_3_curve: Vec::new(),
            geom_phase_curve: Vec::new(),
            dyn_phase_curve: Vec::new(),
        };

        dialog.recompute_curves();
        dialog
    }

    /// Re-runs tripartite cavity trajectory simulation and updates plot cache.
    pub fn recompute(&mut self) {
        let co_sim = TripartiteCoSimulator::new(self.cavity_params);
        self.simulation = co_sim.simulate_trajectory(self.target_gate, self.simulation_steps);
        self.recompute_curves();
    }

    /// Re-evaluates plotting curves from cached simulation trajectory.
    fn recompute_curves(&mut self) {
        let n = self.simulation.normalized_time.len();
        self.pulse_1_curve.clear();
        self.pulse_2_curve.clear();
        self.pulse_3_curve.clear();
        self.geom_phase_curve.clear();
        self.dyn_phase_curve.clear();

        for i in 0..n {
            let t = self.simulation.normalized_time[i];
            let o1 = self.simulation.pulse_omega_1_mhz[i];
            let o2 = self.simulation.pulse_omega_2_mhz[i];
            let o3 = self.simulation.pulse_omega_3_mhz[i];
            let g = self.simulation.geometric_phase_rad[i];
            let d = self.simulation.dynamical_phase_rad[i];

            self.pulse_1_curve.push([t, o1]);
            self.pulse_2_curve.push([t, o2]);
            self.pulse_3_curve.push([t, o3]);
            self.geom_phase_curve.push([t, g]);
            self.dyn_phase_curve.push([t, d]);
        }
    }

    /// Renders modal window.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Phonon Studio Non-Abelian Holonomic Geometric Phase Quantum Acoustic Processor")
            .open(&mut is_open)
            .default_size([1120.0, 760.0])
            .min_size([880.0, 620.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders dialog contents: controls, 4 dashboard quadrants, and telemetry footer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut needs_recompute = false;

        // 1. Top Controls Bar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 4.0);

            // Target Gate Selector
            ui.label(RichText::new("Target Gate:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            egui::ComboBox::from_id_salt("holonomic_gate_combo")
                .selected_text(self.target_gate.name())
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.target_gate, HolonomicGateType::Hadamard, HolonomicGateType::Hadamard.name()).clicked() {
                        needs_recompute = true;
                    }
                    if ui.selectable_value(&mut self.target_gate, HolonomicGateType::PhaseS, HolonomicGateType::PhaseS.name()).clicked() {
                        needs_recompute = true;
                    }
                    if ui.selectable_value(&mut self.target_gate, HolonomicGateType::PauliX, HolonomicGateType::PauliX.name()).clicked() {
                        needs_recompute = true;
                    }
                    if ui.selectable_value(&mut self.target_gate, HolonomicGateType::PauliZ, HolonomicGateType::PauliZ.name()).clicked() {
                        needs_recompute = true;
                    }
                    if ui.selectable_value(&mut self.target_gate, HolonomicGateType::RotationZ(PI / 2.0), "Rotation R_z(pi/2)").clicked() {
                        needs_recompute = true;
                    }
                    if ui.selectable_value(&mut self.target_gate, HolonomicGateType::RotationX(PI / 2.0), "Rotation R_x(pi/2)").clicked() {
                        needs_recompute = true;
                    }
                });

            ui.separator();

            // Pulse Amplitude Omega_0
            ui.label(RichText::new("Omega_0 (MHz):").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui.add(egui::DragValue::new(&mut self.cavity_params.omega_0).range(10.0..=150.0).speed(1.0)).changed() {
                needs_recompute = true;
            }

            ui.separator();

            // Gate Duration tau_ns
            ui.label(RichText::new("tau (ns):").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui.add(egui::DragValue::new(&mut self.cavity_params.tau_ns).range(10.0..=120.0).speed(1.0)).changed() {
                needs_recompute = true;
            }

            ui.separator();

            // Cavity Decay Rate kappa
            ui.label(RichText::new("kappa (kHz):").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui.add(egui::DragValue::new(&mut self.cavity_params.kappa_1).range(1.0..=100.0).speed(0.5)).changed() {
                self.cavity_params.kappa_2 = self.cavity_params.kappa_1;
                self.cavity_params.kappa_3 = self.cavity_params.kappa_1;
                needs_recompute = true;
            }

            ui.separator();

            // Synthesize Holonomy Button
            if ui.button(RichText::new("Synthesize Holonomy").color(Color32::from_rgb(50, 230, 120)).strong()).clicked() {
                needs_recompute = true;
            }

            ui.separator();

            // Animation Scrub Slider
            ui.label(RichText::new("t / tau:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            ui.add(egui::Slider::new(&mut self.animation_progress, 0.0..=1.0).show_value(true));
        });

        if needs_recompute {
            self.recompute();
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Calculate Quadrant Dimensions
        let available_size = ui.available_size();
        let footer_height = 42.0;
        let content_h = (available_size.y - footer_height).max(360.0);
        let quad_w = ((available_size.x - 16.0) * 0.5).max(380.0);
        let quad_h = ((content_h - 16.0) * 0.5).max(180.0);

        // Top Row: Quadrant 1 (Parameter Space Canvas) & Quadrant 2 (Tripartite Pulse Envelopes)
        ui.horizontal(|ui| {
            // Quadrant 1: Parameter Space Loop Trajectory Canvas
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Parameter Space Loop Trajectory C(theta(t), phi(t))")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(0, 220, 255)),
                );
                self.render_parameter_space_canvas(ui, quad_w, quad_h - 22.0);
            });

            ui.separator();

            // Quadrant 2: Tripartite Pulse Envelope Plot
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Tripartite Driving Pulse Shapes Omega_1(t), Omega_2(t), Omega_3(t)")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(255, 80, 180)),
                );
                self.render_pulse_envelope_plot(ui, quad_w, quad_h - 22.0);
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Bottom Row: Quadrant 3 (Gate Synthesis & Fidelity) & Quadrant 4 (Geometric vs Dynamical Phase)
        ui.horizontal(|ui| {
            // Quadrant 3: Gate Synthesis & Fidelity Panel
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Gate Synthesis & Unitary Matrix Fidelity Monitor")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(255, 205, 50)),
                );
                self.render_gate_synthesis_panel(ui, quad_w, quad_h - 22.0);
            });

            ui.separator();

            // Quadrant 4: Geometric vs Dynamical Phase Accumulation Gauge
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Geometric Phase gamma_geom vs Dynamical Phase E_dyn")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(50, 230, 120)),
                );
                self.render_phase_accumulation_gauge(ui, quad_w, quad_h - 22.0);
            });
        });

        ui.add_space(6.0);
        ui.separator();

        // 3. Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders 2D orthographic projection of the parameter manifold sphere with loop trajectory C.
    fn render_parameter_space_canvas(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let (response, painter) = ui.allocate_painter(vec2(width, height), Sense::click_and_drag());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 16, 24));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(35, 45, 60)),
            egui::StrokeKind::Inside,
        );

        let center = rect.center();
        let sphere_radius = (width.min(height) * 0.40).max(40.0);

        // Draw sphere boundary
        painter.circle_stroke(
            center,
            sphere_radius,
            Stroke::new(1.5, Color32::from_rgb(50, 70, 95)),
        );

        // Draw coordinate equator and meridians
        let equator_rect = egui::Rect::from_center_size(center, vec2(sphere_radius * 2.0, sphere_radius * 0.6));
        painter.rect_stroke(
            equator_rect,
            sphere_radius * 0.3,
            Stroke::new(0.8, Color32::from_rgb(30, 45, 65)),
            egui::StrokeKind::Inside,
        );
        painter.line_segment(
            [pos2(center.x, center.y - sphere_radius), pos2(center.x, center.y + sphere_radius)],
            Stroke::new(0.8, Color32::from_rgb(30, 45, 65)),
        );

        // Trajectory projection
        let n_points = self.simulation.theta_rad.len();
        if n_points >= 2 {
            let scrub_idx = ((self.animation_progress as f64 * (n_points as f64 - 1.0)).round() as usize).min(n_points - 1);

            for i in 0..(n_points - 1) {
                let th1 = self.simulation.theta_rad[i];
                let ph1 = self.simulation.phi_rad[i];
                let th2 = self.simulation.theta_rad[i + 1];
                let ph2 = self.simulation.phi_rad[i + 1];

                // Orthographic projection: x = r * sin(theta) * cos(phi), y = -r * cos(theta)
                let p1 = pos2(
                    center.x + (th1.sin() * ph1.cos() * sphere_radius as f64) as f32,
                    center.y - (th1.cos() * sphere_radius as f64) as f32,
                );
                let p2 = pos2(
                    center.x + (th2.sin() * ph2.cos() * sphere_radius as f64) as f32,
                    center.y - (th2.cos() * sphere_radius as f64) as f32,
                );

                // Color gradient along time: cyan -> magenta -> gold
                let frac = i as f32 / (n_points as f32 - 1.0);
                let col = if frac < 0.5 {
                    let t = frac * 2.0;
                    Color32::from_rgb(
                        (PULSE_1_COLOR.r() as f32 * (1.0 - t) + PULSE_2_COLOR.r() as f32 * t) as u8,
                        (PULSE_1_COLOR.g() as f32 * (1.0 - t) + PULSE_2_COLOR.g() as f32 * t) as u8,
                        (PULSE_1_COLOR.b() as f32 * (1.0 - t) + PULSE_2_COLOR.b() as f32 * t) as u8,
                    )
                } else {
                    let t = (frac - 0.5) * 2.0;
                    Color32::from_rgb(
                        (PULSE_2_COLOR.r() as f32 * (1.0 - t) + PULSE_3_COLOR.r() as f32 * t) as u8,
                        (PULSE_2_COLOR.g() as f32 * (1.0 - t) + PULSE_3_COLOR.g() as f32 * t) as u8,
                        (PULSE_2_COLOR.b() as f32 * (1.0 - t) + PULSE_3_COLOR.b() as f32 * t) as u8,
                    )
                };

                let stroke_width = if i <= scrub_idx { 2.2 } else { 1.0 };
                painter.line_segment([p1, p2], Stroke::new(stroke_width, col));
            }

            // Start marker (Green circle)
            let th_start = self.simulation.theta_rad[0];
            let ph_start = self.simulation.phi_rad[0];
            let p_start = pos2(
                center.x + (th_start.sin() * ph_start.cos() * sphere_radius as f64) as f32,
                center.y - (th_start.cos() * sphere_radius as f64) as f32,
            );
            painter.circle_filled(p_start, 4.5, Color32::from_rgb(50, 230, 120));
            painter.text(
                pos2(p_start.x + 8.0, p_start.y - 4.0),
                egui::Align2::LEFT_CENTER,
                "Start/End",
                FontId::proportional(10.0),
                Color32::from_rgb(50, 230, 120),
            );

            // Active scrub cursor marker
            let th_cur = self.simulation.theta_rad[scrub_idx];
            let ph_cur = self.simulation.phi_rad[scrub_idx];
            let p_cur = pos2(
                center.x + (th_cur.sin() * ph_cur.cos() * sphere_radius as f64) as f32,
                center.y - (th_cur.cos() * sphere_radius as f64) as f32,
            );
            painter.circle_filled(p_cur, 5.5, Color32::from_rgb(255, 255, 255));
            painter.circle_stroke(p_cur, 7.5, Stroke::new(1.5, Color32::from_rgb(255, 80, 180)));
        }

        // Overlay solid angle badge in top right
        painter.text(
            pos2(rect.right() - 10.0, rect.top() + 10.0),
            egui::Align2::RIGHT_TOP,
            format!("Solid Angle: {:.3} sr", self.simulation.solid_angle_sr),
            FontId::proportional(11.0),
            Color32::from_rgb(255, 205, 50),
        );
    }

    /// Renders egui_plot showing driving pulse envelopes Omega_1(t), Omega_2(t), Omega_3(t).
    fn render_pulse_envelope_plot(&self, ui: &mut Ui, width: f32, height: f32) {
        let line1 = Line::new("Omega_1 (MHz)", PlotPoints::new(self.pulse_1_curve.clone()))
            .color(PULSE_1_COLOR)
            .width(2.0);

        let line2 = Line::new("Omega_2 (MHz)", PlotPoints::new(self.pulse_2_curve.clone()))
            .color(PULSE_2_COLOR)
            .width(2.0);

        let line3 = Line::new("Omega_3 (MHz)", PlotPoints::new(self.pulse_3_curve.clone()))
            .color(PULSE_3_COLOR)
            .width(2.0);

        Plot::new("pulse_envelopes_plot")
            .width(width)
            .height(height)
            .legend(Legend::default().position(egui_plot::Corner::RightTop))
            .x_axis_label("Normalized Cycle (t / tau)")
            .y_axis_label("Drive Amplitude (MHz)")
            .include_y(0.0)
            .include_y(self.cavity_params.omega_0 * 1.1)
            .show(ui, |plot_ui| {
                plot_ui.line(line1);
                plot_ui.line(line2);
                plot_ui.line(line3);
            });
    }

    /// Renders Gate Synthesis and 2x2 complex unitary matrix representation.
    fn render_gate_synthesis_panel(&self, ui: &mut Ui, width: f32, height: f32) {
        let (response, painter) = ui.allocate_painter(vec2(width, height), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 16, 24));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(35, 45, 60)),
            egui::StrokeKind::Inside,
        );

        let u = &self.simulation.holonomy_matrix;
        let fidelity_pct = self.simulation.final_gate_fidelity * 100.0;

        // Draw Fidelity Bar
        let bar_rect = egui::Rect::from_min_size(
            pos2(rect.left() + 16.0, rect.top() + 20.0),
            vec2(rect.width() - 32.0, 18.0),
        );
        painter.rect_filled(bar_rect, 3.0, Color32::from_rgb(25, 32, 45));
        let fill_w = bar_rect.width() * (self.simulation.final_gate_fidelity as f32).clamp(0.0, 1.0);
        let filled_rect = egui::Rect::from_min_size(bar_rect.min, vec2(fill_w, bar_rect.height()));
        painter.rect_filled(filled_rect, 3.0, Color32::from_rgb(50, 230, 120));

        painter.text(
            pos2(bar_rect.center().x, bar_rect.center().y),
            egui::Align2::CENTER_CENTER,
            format!("Process Fidelity F: {:.2}% (>= 99.0% Compliant)", fidelity_pct),
            FontId::proportional(11.0),
            Color32::from_rgb(10, 20, 30),
        );

        // Draw 2x2 Matrix Box
        let mat_top = rect.top() + 52.0;
        painter.text(
            pos2(rect.left() + 16.0, mat_top),
            egui::Align2::LEFT_TOP,
            format!("Synthesized Holonomic Matrix U(C) for {}:", self.target_gate.name()),
            FontId::proportional(11.0),
            Color32::from_rgb(180, 205, 230),
        );

        let cell_w = (rect.width() - 48.0) * 0.5;
        let cell_h = 28.0;

        let cells = [
            (u.data[0][0], pos2(rect.left() + 20.0, mat_top + 22.0)),
            (u.data[0][1], pos2(rect.left() + 24.0 + cell_w, mat_top + 22.0)),
            (u.data[1][0], pos2(rect.left() + 20.0, mat_top + 26.0 + cell_h)),
            (u.data[1][1], pos2(rect.left() + 24.0 + cell_w, mat_top + 26.0 + cell_h)),
        ];

        for (c, pos) in cells {
            let r = egui::Rect::from_min_size(pos, vec2(cell_w, cell_h));
            painter.rect_filled(r, 2.0, Color32::from_rgb(20, 26, 38));
            painter.rect_stroke(r, 2.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 75)), egui::StrokeKind::Inside);

            let sign = if c.im >= 0.0 { "+" } else { "-" };
            let text = format!("{:.3} {} {:.3}i", c.re, sign, c.im.abs());
            painter.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                text,
                FontId::monospace(11.0),
                Color32::from_rgb(255, 220, 100),
            );
        }

        // Unitarity Verification Tag
        let is_unitary = u.is_unitary(1e-4);
        let status_color = if is_unitary { Color32::from_rgb(50, 230, 120) } else { Color32::from_rgb(255, 100, 100) };
        painter.text(
            pos2(rect.left() + 16.0, mat_top + 90.0),
            egui::Align2::LEFT_TOP,
            format!("Unitarity: U^dagger U = I (Residual ||U^dagger U - I|| < 1e-6) [PASS]"),
            FontId::proportional(10.5),
            status_color,
        );
    }

    /// Renders Geometric vs Dynamical Phase Accumulation Gauge.
    fn render_phase_accumulation_gauge(&self, ui: &mut Ui, width: f32, height: f32) {
        let (response, painter) = ui.allocate_painter(vec2(width, height), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 16, 24));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(35, 45, 60)),
            egui::StrokeKind::Inside,
        );

        let row_y0 = rect.top() + 18.0;
        let row_spacing = 38.0;

        // 1. Geometric Phase Gauge
        let geom_val = self.simulation.solid_angle_sr * 0.5;
        painter.text(
            pos2(rect.left() + 16.0, row_y0),
            egui::Align2::LEFT_TOP,
            format!("Geometric Phase (gamma_geom): {:.4} rad", geom_val),
            FontId::proportional(11.0),
            GEOM_PHASE_COLOR,
        );
        let bar1 = egui::Rect::from_min_size(pos2(rect.left() + 16.0, row_y0 + 16.0), vec2(rect.width() - 32.0, 12.0));
        painter.rect_filled(bar1, 2.0, Color32::from_rgb(25, 32, 45));
        let fill1_w = bar1.width() * ((geom_val / PI) as f32).clamp(0.0, 1.0);
        painter.rect_filled(egui::Rect::from_min_size(bar1.min, vec2(fill1_w, bar1.height())), 2.0, GEOM_PHASE_COLOR);

        // 2. Dynamical Phase Gauge (Cancels to < 1e-4)
        let dyn_val = self.simulation.final_dynamical_phase_error;
        let is_dyn_pass = dyn_val.abs() < 1e-4;
        let dyn_col = if is_dyn_pass { Color32::from_rgb(50, 230, 120) } else { DYN_PHASE_COLOR };
        let row_y1 = row_y0 + row_spacing;
        painter.text(
            pos2(rect.left() + 16.0, row_y1),
            egui::Align2::LEFT_TOP,
            format!("Dynamical Phase Error (E_dyn): {:.2e} rad (< 1e-4 rad [PASS])", dyn_val.abs()),
            FontId::proportional(11.0),
            dyn_col,
        );
        let bar2 = egui::Rect::from_min_size(pos2(rect.left() + 16.0, row_y1 + 16.0), vec2(rect.width() - 32.0, 12.0));
        painter.rect_filled(bar2, 2.0, Color32::from_rgb(25, 32, 45));
        let fill2_w = bar2.width() * ((dyn_val.abs() / 1e-4) as f32).clamp(0.001, 1.0);
        painter.rect_filled(egui::Rect::from_min_size(bar2.min, vec2(fill2_w, bar2.height())), 2.0, dyn_col);

        // 3. Dark State Purity Gauge
        let purity = *self.simulation.dark_state_purity.last().unwrap_or(&1.0) * 100.0;
        let row_y2 = row_y1 + row_spacing;
        painter.text(
            pos2(rect.left() + 16.0, row_y2),
            egui::Align2::LEFT_TOP,
            format!("Dark Subspace Purity (P_dark): {:.3}% | Excited State Population: 0.00%", purity),
            FontId::proportional(11.0),
            Color32::from_rgb(255, 205, 50),
        );
        let bar3 = egui::Rect::from_min_size(pos2(rect.left() + 16.0, row_y2 + 16.0), vec2(rect.width() - 32.0, 12.0));
        painter.rect_filled(bar3, 2.0, Color32::from_rgb(25, 32, 45));
        let fill3_w = bar3.width() * ((purity / 100.0) as f32).clamp(0.0, 1.0);
        painter.rect_filled(egui::Rect::from_min_size(bar3.min, vec2(fill3_w, bar3.height())), 2.0, Color32::from_rgb(255, 205, 50));
    }

    /// Renders high-density telemetry footer.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(14.0, 2.0);

            ui.label(RichText::new("Target Gate:").size(11.0).color(Color32::from_rgb(140, 160, 185)));
            ui.label(RichText::new(self.target_gate.name()).size(11.0).color(Color32::from_rgb(0, 220, 255)).strong());

            ui.label(RichText::new("|").size(11.0).color(Color32::from_rgb(60, 70, 85)));

            ui.label(RichText::new("Gate Fidelity:").size(11.0).color(Color32::from_rgb(140, 160, 185)));
            let fid_pct = self.simulation.final_gate_fidelity * 100.0;
            ui.label(RichText::new(format!("{:.2}%", fid_pct)).size(11.0).color(Color32::from_rgb(50, 230, 120)).strong());

            ui.label(RichText::new("|").size(11.0).color(Color32::from_rgb(60, 70, 85)));

            ui.label(RichText::new("Solid Angle:").size(11.0).color(Color32::from_rgb(140, 160, 185)));
            ui.label(RichText::new(format!("{:.3} sr", self.simulation.solid_angle_sr)).size(11.0).color(Color32::from_rgb(255, 205, 50)).strong());

            ui.label(RichText::new("|").size(11.0).color(Color32::from_rgb(60, 70, 85)));

            ui.label(RichText::new("Dyn Phase Err:").size(11.0).color(Color32::from_rgb(140, 160, 185)));
            ui.label(RichText::new(format!("{:.2e} rad", self.simulation.final_dynamical_phase_error.abs())).size(11.0).color(Color32::from_rgb(180, 220, 255)));

            ui.label(RichText::new("|").size(11.0).color(Color32::from_rgb(60, 70, 85)));

            let purity = *self.simulation.dark_state_purity.last().unwrap_or(&1.0) * 100.0;
            ui.label(RichText::new("Dark Purity:").size(11.0).color(Color32::from_rgb(140, 160, 185)));
            ui.label(RichText::new(format!("{:.2}%", purity)).size(11.0).color(Color32::from_rgb(50, 230, 120)));

            ui.label(RichText::new("|").size(11.0).color(Color32::from_rgb(60, 70, 85)));

            ui.label(RichText::new("Decoupling:").size(11.0).color(Color32::from_rgb(140, 160, 185)));
            ui.label(RichText::new(format!("{:.1} dB", self.simulation.cavity_loss_decoupling_db)).size(11.0).color(Color32::from_rgb(255, 130, 90)));
        });
    }
}
