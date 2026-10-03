#![deny(unsafe_code)]

//! Interactive Acoustic Chern Insulator Chiral Circulator & Non-Reciprocal Router Studio dialog.
//!
//! Provides:
//! - 2D Acoustic Pressure Distribution Canvas: renders 3-port honeycomb lattice with circulating
//!   acoustic pressure field, spinning fluid cylinder vortex markers with rotation arrows,
//!   and wave packet routing unidirectionally from active input port to output port.
//! - 3-Port S-Parameter Spectrum Plot: native egui_plot rendering S_21(f), S_31(f), and S_11(f)
//!   in dB over frequency [3.0 kHz, 5.0 kHz], clearly showing insertion loss band and isolation dip (>= 35 dB).
//! - Corner Defect Immunity Toggle: checkbox / button to insert sharp 90-degree corner or missing cylinder
//!   obstacle in real-time, displaying edge wave flowing smoothly around obstacle with zero reflection.
//! - Controls: Active Input Port selector (Port 1, Port 2, Port 3), fluid spin rate Omega slider,
//!   center frequency f_0, defect toggle.
//! - Telemetry Footer: Chern Number (C = +1), Insertion Loss (dB), Isolation (dB),
//!   Corner Transmission (%), Topological Gap (kHz), Fluid Spin Rate Omega (rad/s).

use egui::{
    vec2, Color32, FontId, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::acoustic_chern_circulator::{
    ChernLatticeParams, ChiralEdgeMode, CirculatorPort, DefectImmunityResult, ObstacleKind,
    SParameterSpectrum, ScatteringMatrix3x3, ThreePortCirculator,
};
use std::f64::consts::PI;

/// Palette colors for acoustic Chern circulator studio.
const COLOR_ACTIVE_INPUT: Color32 = Color32::from_rgb(52, 211, 153); // Emerald
const COLOR_ACTIVE_OUTPUT: Color32 = Color32::from_rgb(56, 189, 248); // Cyan
const COLOR_ISOLATED_PORT: Color32 = Color32::from_rgb(244, 63, 94); // Rose
const COLOR_VORTEX_CYLINDER: Color32 = Color32::from_rgb(14, 165, 233); // Sky Blue
const COLOR_SPIN_ARROW: Color32 = Color32::from_rgb(250, 204, 21); // Gold
const COLOR_OBSTACLE_BLOCK: Color32 = Color32::from_rgb(249, 115, 22); // Amber-Orange
const COLOR_RESONATOR_WALL: Color32 = Color32::from_rgb(51, 65, 85); // Slate 700
const COLOR_CAVITY_INTERIOR: Color32 = Color32::from_rgb(15, 23, 42); // Slate 900
const COLOR_EDGE_CHANNEL: Color32 = Color32::from_rgba_premultiplied(56, 189, 248, 120);

/// Interactive modal dialog for the Acoustic Chern Insulator Chiral Circulator Studio.
#[derive(Debug, Clone, PartialEq)]
pub struct ChernCirculatorDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Controls
    /// Physical parameters for the honeycomb Chern lattice.
    pub params: ChernLatticeParams,
    /// Active input port (Port 1, 2, or 3).
    pub active_port: CirculatorPort,
    /// Boundary defect obstacle kind.
    pub obstacle: ObstacleKind,
    /// Fast toggle for sharp 90-degree corner defect.
    pub corner_defect_enabled: bool,

    // Cached Simulation State
    pub circulator: ThreePortCirculator,
    pub scattering_matrix: ScatteringMatrix3x3,
    pub spectrum: SParameterSpectrum,
    pub defect_result: DefectImmunityResult,
    pub edge_mode: ChiralEdgeMode,

    // Cached Plot Curves
    pub s21_curve: Vec<[f64; 2]>,
    pub s31_curve: Vec<[f64; 2]>,
    pub s11_curve: Vec<[f64; 2]>,

    // Telemetry Metrics
    pub chern_number: i32,
    pub insertion_loss_db: f64,
    pub isolation_db: f64,
    pub return_loss_db: f64,
    pub corner_transmission_percent: f64,
    pub topological_gap_khz: f64,
    pub fluid_spin_rate_rad_s: f64,
    pub edge_velocity_mps: f64,

    pub status_msg: String,
}

impl Default for ChernCirculatorDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ChernCirculatorDialog {
    /// Creates a new `ChernCirculatorDialog` initialized with default physical parameters.
    pub fn new() -> Self {
        let params = ChernLatticeParams::default();
        let active_port = CirculatorPort::Port1;
        let obstacle = ObstacleKind::None;
        let corner_defect_enabled = false;

        let circulator = ThreePortCirculator::new(params)
            .with_active_port(active_port)
            .with_obstacle(obstacle);

        let scattering_matrix = circulator.compute_scattering_matrix();
        let spectrum = circulator.compute_spectrum(3.0, 5.0, 80);
        let defect_result = circulator.evaluate_defect_immunity(obstacle);
        let edge_mode = params.compute_chiral_edge_mode();

        let s21_curve: Vec<[f64; 2]> = spectrum
            .points
            .iter()
            .map(|p| [p.freq_khz, p.s21_db])
            .collect();
        let s31_curve: Vec<[f64; 2]> = spectrum
            .points
            .iter()
            .map(|p| [p.freq_khz, p.s31_db])
            .collect();
        let s11_curve: Vec<[f64; 2]> = spectrum
            .points
            .iter()
            .map(|p| [p.freq_khz, p.s11_db])
            .collect();

        let out_port = circulator.output_port_for(active_port);
        let iso_port = circulator.isolated_port_for(active_port);
        let il_db = scattering_matrix.insertion_loss_db(active_port, out_port);
        let iso_db = scattering_matrix.isolation_db(active_port, iso_port);
        let rl_db = scattering_matrix.return_loss_db(active_port);

        let chern = params.compute_chern_number();
        let gap_khz = params.compute_topological_gap_khz();

        Self {
            is_open: false,
            params,
            active_port,
            obstacle,
            corner_defect_enabled,
            circulator,
            scattering_matrix,
            spectrum,
            defect_result,
            edge_mode: edge_mode.clone(),
            s21_curve,
            s31_curve,
            s11_curve,
            chern_number: chern,
            insertion_loss_db: il_db,
            isolation_db: iso_db,
            return_loss_db: rl_db,
            corner_transmission_percent: defect_result.transmission_percent,
            topological_gap_khz: gap_khz,
            fluid_spin_rate_rad_s: params.omega_rad_s,
            edge_velocity_mps: edge_mode.group_velocity_mps,
            status_msg: format!(
                "Chern Circulator online: C = +{}, Gap = {:.2} kHz, Isolation = {:.1} dB",
                chern, gap_khz, iso_db
            ),
        }
    }

    /// Recomputes simulation state, S-parameters, defect immunity, and plot curves.
    pub fn recompute(&mut self) {
        if self.corner_defect_enabled && self.obstacle == ObstacleKind::None {
            self.obstacle = ObstacleKind::SharpCorner90;
        } else if !self.corner_defect_enabled && self.obstacle == ObstacleKind::SharpCorner90 {
            self.obstacle = ObstacleKind::None;
        }

        self.circulator = ThreePortCirculator::new(self.params)
            .with_active_port(self.active_port)
            .with_obstacle(self.obstacle);

        self.scattering_matrix = self.circulator.compute_scattering_matrix();
        self.spectrum = self.circulator.compute_spectrum(3.0, 5.0, 80);
        self.defect_result = self.circulator.evaluate_defect_immunity(self.obstacle);
        self.edge_mode = self.params.compute_chiral_edge_mode();

        self.s21_curve = self
            .spectrum
            .points
            .iter()
            .map(|p| [p.freq_khz, p.s21_db])
            .collect();
        self.s31_curve = self
            .spectrum
            .points
            .iter()
            .map(|p| [p.freq_khz, p.s31_db])
            .collect();
        self.s11_curve = self
            .spectrum
            .points
            .iter()
            .map(|p| [p.freq_khz, p.s11_db])
            .collect();

        let out_port = self.circulator.output_port_for(self.active_port);
        let iso_port = self.circulator.isolated_port_for(self.active_port);
        self.insertion_loss_db = self
            .scattering_matrix
            .insertion_loss_db(self.active_port, out_port);
        self.isolation_db = self
            .scattering_matrix
            .isolation_db(self.active_port, iso_port);
        self.return_loss_db = self.scattering_matrix.return_loss_db(self.active_port);

        self.chern_number = self.params.compute_chern_number();
        self.topological_gap_khz = self.params.compute_topological_gap_khz();
        self.fluid_spin_rate_rad_s = self.params.omega_rad_s;
        self.edge_velocity_mps = self.edge_mode.group_velocity_mps;
        self.corner_transmission_percent = self.defect_result.transmission_percent;

        let tr_str = if self.params.is_time_reversal_broken() {
            "TRS Broken"
        } else {
            "TRS Preserved"
        };
        self.status_msg = format!(
            "Chern C = {}, Gap = {:.2} kHz ({}), IL = {:.2} dB, Iso = {:.1} dB, Corner T = {:.1}%",
            self.chern_number,
            self.topological_gap_khz,
            tr_str,
            self.insertion_loss_db,
            self.isolation_db,
            self.corner_transmission_percent
        );
    }

    /// Renders the modal window.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Topological Acoustic Chern Circulator Studio")
            .open(&mut is_open)
            .default_size([1080.0, 720.0])
            .min_size([880.0, 580.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the complete dialog contents: control bar, 2-column workspace, and telemetry footer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut needs_recompute = false;

        // 1. Controls Bar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 4.0);

            // Active Input Port Selector
            ui.label(RichText::new("Input Port:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            let port_btn_p1 = ui.selectable_label(
                self.active_port == CirculatorPort::Port1,
                RichText::new("Port 1 (0 deg)").size(11.0),
            );
            if port_btn_p1.clicked() && self.active_port != CirculatorPort::Port1 {
                self.active_port = CirculatorPort::Port1;
                needs_recompute = true;
            }

            let port_btn_p2 = ui.selectable_label(
                self.active_port == CirculatorPort::Port2,
                RichText::new("Port 2 (120 deg)").size(11.0),
            );
            if port_btn_p2.clicked() && self.active_port != CirculatorPort::Port2 {
                self.active_port = CirculatorPort::Port2;
                needs_recompute = true;
            }

            let port_btn_p3 = ui.selectable_label(
                self.active_port == CirculatorPort::Port3,
                RichText::new("Port 3 (240 deg)").size(11.0),
            );
            if port_btn_p3.clicked() && self.active_port != CirculatorPort::Port3 {
                self.active_port = CirculatorPort::Port3;
                needs_recompute = true;
            }

            ui.separator();

            // Fluid spin rate Omega slider
            ui.label(RichText::new("Fluid Spin Omega:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            let omega_changed = ui
                .add(
                    egui::Slider::new(&mut self.params.omega_rad_s, -2400.0..=2400.0)
                        .step_by(50.0)
                        .suffix(" rad/s"),
                )
                .changed();
            if omega_changed {
                needs_recompute = true;
            }

            // Center frequency f_0 slider
            ui.label(RichText::new("Frequency f_0:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            let f0_changed = ui
                .add(
                    egui::Slider::new(&mut self.params.f_0_khz, 3.0..=5.0)
                        .step_by(0.05)
                        .suffix(" kHz"),
                )
                .changed();
            if f0_changed {
                needs_recompute = true;
            }

            ui.separator();

            // Corner Defect Immunity Toggle
            let defect_chk = ui.checkbox(
                &mut self.corner_defect_enabled,
                RichText::new("90-Deg Corner Defect").size(11.0).color(COLOR_OBSTACLE_BLOCK),
            );
            if defect_chk.changed() {
                needs_recompute = true;
            }

            // Obstacle Kind ComboBox
            egui::ComboBox::from_id_salt("obstacle_selector")
                .selected_text(self.obstacle.label())
                .show_ui(ui, |ui| {
                    if ui.selectable_label(self.obstacle == ObstacleKind::None, ObstacleKind::None.label()).clicked() {
                        self.obstacle = ObstacleKind::None;
                        self.corner_defect_enabled = false;
                        needs_recompute = true;
                    }
                    if ui.selectable_label(self.obstacle == ObstacleKind::SharpCorner90, ObstacleKind::SharpCorner90.label()).clicked() {
                        self.obstacle = ObstacleKind::SharpCorner90;
                        self.corner_defect_enabled = true;
                        needs_recompute = true;
                    }
                    if ui.selectable_label(self.obstacle == ObstacleKind::SharpBend120, ObstacleKind::SharpBend120.label()).clicked() {
                        self.obstacle = ObstacleKind::SharpBend120;
                        self.corner_defect_enabled = false;
                        needs_recompute = true;
                    }
                    if ui.selectable_label(self.obstacle == ObstacleKind::MissingSiteVacancy, ObstacleKind::MissingSiteVacancy.label()).clicked() {
                        self.obstacle = ObstacleKind::MissingSiteVacancy;
                        self.corner_defect_enabled = false;
                        needs_recompute = true;
                    }
                });
        });

        if needs_recompute {
            self.recompute();
        }

        ui.add_space(4.0);

        // Status banner
        ui.horizontal(|ui| {
            let dot_color = if self.chern_number != 0 {
                COLOR_ACTIVE_INPUT
            } else {
                Color32::from_rgb(140, 160, 180)
            };
            ui.label(RichText::new("o").color(dot_color).size(12.0));
            ui.label(
                RichText::new(&self.status_msg)
                    .size(11.0)
                    .color(Color32::from_rgb(200, 215, 230)),
            );
        });

        ui.separator();

        // 2. Main Workspace Layout:
        // Left Column: 2D Acoustic Pressure Distribution Canvas
        // Right Column: 3-Port S-Parameter Spectrum Plot
        let total_avail_h = (ui.available_height() - 75.0).max(360.0);

        ui.columns(2, |cols| {
            // Left Column: 2D Canvas
            cols[0].group(|ui| {
                ui.set_height(total_avail_h);
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("2D Acoustic Pressure Distribution & Chiral Routing")
                            .size(12.0)
                            .color(Color32::from_rgb(180, 210, 240)),
                    );
                    let obs_str = self.obstacle.label();
                    ui.label(
                        RichText::new(format!("[{}]", obs_str))
                            .size(10.0)
                            .color(if self.obstacle != ObstacleKind::None {
                                COLOR_OBSTACLE_BLOCK
                            } else {
                                COLOR_ACTIVE_OUTPUT
                            }),
                    );
                });
                self.render_acoustic_canvas(ui, total_avail_h - 32.0);
            });

            // Right Column: S-Parameter Spectrum Plot
            cols[1].group(|ui| {
                ui.set_height(total_avail_h);
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("3-Port S-Parameter Spectrum (dB)")
                            .size(12.0)
                            .color(Color32::from_rgb(180, 210, 240)),
                    );
                    ui.label(
                        RichText::new(format!("Bandgap: {:.2} kHz", self.topological_gap_khz))
                            .size(10.0)
                            .color(COLOR_ACTIVE_INPUT),
                    );
                });
                self.render_spectrum_plot(ui, total_avail_h - 32.0);
            });
        });

        ui.separator();

        // 3. Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders the 2D Acoustic Pressure Distribution Canvas with honeycomb lattice and vortex cylinders.
    fn render_acoustic_canvas(&self, ui: &mut Ui, h: f32) {
        let w = ui.available_width().max(100.0);
        let (response, painter) = ui.allocate_painter(vec2(w, h), Sense::hover());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 4.0, COLOR_CAVITY_INTERIOR);
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, COLOR_RESONATOR_WALL),
            StrokeKind::Inside,
        );

        let center = rect.center();
        let radius = (w.min(h) * 0.38).clamp(80.0, 220.0);

        // Circulator perimeter boundary
        painter.circle_filled(
            center,
            radius,
            Color32::from_rgba_unmultiplied(20, 30, 48, 180),
        );
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(2.0, COLOR_RESONATOR_WALL),
        );

        // Draw honeycomb resonator cavities inside the circulator disk
        let hex_spacing = radius * 0.35;
        let resonator_r = hex_spacing * 0.30;
        let cylinder_r = resonator_r * (self.params.r_cyl_mm / 10.0).clamp(0.25, 0.75) as f32;

        let out_port = self.circulator.output_port_for(self.active_port);
        let _iso_port = self.circulator.isolated_port_for(self.active_port);

        // Honeycomb grid coordinates
        let range = -2..=2;
        for ix in range.clone() {
            for iy in range.clone() {
                // Hexagonal basis
                let px = (ix as f32) * hex_spacing + (iy as f32) * hex_spacing * 0.5;
                let py = (iy as f32) * hex_spacing * 0.866;
                let site_pos = center + vec2(px, py);

                let dist = (site_pos - center).length();
                if dist > radius - resonator_r * 0.8 {
                    continue;
                }

                // If obstacle is missing vacancy and site is near boundary, skip rendering to represent vacancy
                if self.obstacle == ObstacleKind::MissingSiteVacancy
                    && dist > radius * 0.65
                    && py < -radius * 0.3
                {
                    // Draw vacancy marker
                    painter.circle_stroke(
                        site_pos,
                        resonator_r,
                        Stroke::new(1.5, COLOR_OBSTACLE_BLOCK),
                    );
                    painter.line_segment(
                        [
                            site_pos + vec2(-resonator_r * 0.6, -resonator_r * 0.6),
                            site_pos + vec2(resonator_r * 0.6, resonator_r * 0.6),
                        ],
                        Stroke::new(1.5, COLOR_OBSTACLE_BLOCK),
                    );
                    painter.line_segment(
                        [
                            site_pos + vec2(-resonator_r * 0.6, resonator_r * 0.6),
                            site_pos + vec2(resonator_r * 0.6, -resonator_r * 0.6),
                        ],
                        Stroke::new(1.5, COLOR_OBSTACLE_BLOCK),
                    );
                    continue;
                }

                // Resonator cavity outer circle
                painter.circle_filled(
                    site_pos,
                    resonator_r,
                    Color32::from_rgb(20, 32, 50),
                );
                painter.circle_stroke(
                    site_pos,
                    resonator_r,
                    Stroke::new(1.0, COLOR_RESONATOR_WALL),
                );

                // Spinning fluid cylinder vortex core
                painter.circle_filled(site_pos, cylinder_r, COLOR_VORTEX_CYLINDER);
                painter.circle_stroke(
                    site_pos,
                    cylinder_r,
                    Stroke::new(1.0, Color32::from_rgb(56, 189, 248)),
                );

                // Fluid rotation arrows
                if self.params.omega_rad_s.abs() > 1.0e-3 {
                    let spin_dir = self.params.omega_rad_s.signum() as f32;
                    let arrow_r = cylinder_r * 0.65;
                    let num_arc_pts = 6;
                    let mut arc_pts = Vec::with_capacity(num_arc_pts);
                    for a_idx in 0..num_arc_pts {
                        let ang = (a_idx as f32) * (PI as f32 * 0.3) * spin_dir;
                        arc_pts.push(site_pos + vec2(ang.cos() * arrow_r, ang.sin() * arrow_r));
                    }
                    for w in arc_pts.windows(2) {
                        painter.line_segment([w[0], w[1]], Stroke::new(1.2, COLOR_SPIN_ARROW));
                    }
                }
            }
        }

        // Draw 3 Ports around perimeter
        let port_length = radius * 0.28;
        let port_width = resonator_r * 1.8;

        for port in [CirculatorPort::Port1, CirculatorPort::Port2, CirculatorPort::Port3] {
            let angle = port.angle_rad() as f32;
            let dir = vec2(angle.cos(), -angle.sin()); // Screen y is inverted
            let perp = vec2(-dir.y, dir.x);

            let base_pt = center + dir * radius;
            let tip_pt = center + dir * (radius + port_length);

            let p1 = base_pt - perp * (port_width * 0.5);
            let p2 = base_pt + perp * (port_width * 0.5);
            let p3 = tip_pt + perp * (port_width * 0.5);
            let p4 = tip_pt - perp * (port_width * 0.5);

            // Port color coding
            let (fill_col, stroke_col, label_str) = if port == self.active_port {
                (
                    Color32::from_rgba_unmultiplied(52, 211, 153, 100),
                    COLOR_ACTIVE_INPUT,
                    "IN",
                )
            } else if port == out_port {
                (
                    Color32::from_rgba_unmultiplied(56, 189, 248, 100),
                    COLOR_ACTIVE_OUTPUT,
                    "OUT",
                )
            } else {
                (
                    Color32::from_rgba_unmultiplied(244, 63, 94, 60),
                    COLOR_ISOLATED_PORT,
                    "ISO",
                )
            };

            // Draw port waveguide stub
            painter.add(egui::Shape::convex_polygon(
                vec![p1, p2, p3, p4],
                fill_col,
                Stroke::new(1.5, stroke_col),
            ));

            // Port badge text
            let text_pos = tip_pt + dir * 14.0;
            let port_txt = format!("{}: {}", port.short_name(), label_str);
            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                port_txt,
                FontId::proportional(11.0),
                stroke_col,
            );
        }

        // Draw chiral acoustic edge wave routing from active input port to output port
        let in_angle = self.active_port.angle_rad() as f32;
        let out_angle = out_port.angle_rad() as f32;

        let is_clockwise = self.chern_number > 0;
        let arc_steps = 30;

        // Angle sweep between active port and output port
        let mut wave_pts = Vec::with_capacity(arc_steps);
        for s in 0..=arc_steps {
            let frac = (s as f32) / (arc_steps as f32);
            let ang = if is_clockwise {
                // Forward angle progression in screen coordinates
                in_angle + (out_angle - in_angle + 2.0 * PI as f32).rem_euclid(2.0 * PI as f32) * frac
            } else {
                in_angle - (in_angle - out_angle + 2.0 * PI as f32).rem_euclid(2.0 * PI as f32) * frac
            };
            let dir = vec2(ang.cos(), -ang.sin());
            let r_edge = radius * 0.90;
            wave_pts.push(center + dir * r_edge);
        }

        // Draw smooth boundary edge wave channel
        for w in wave_pts.windows(2) {
            painter.line_segment([w[0], w[1]], Stroke::new(4.5, COLOR_EDGE_CHANNEL));
            painter.line_segment([w[0], w[1]], Stroke::new(2.0, COLOR_ACTIVE_OUTPUT));
        }

        // Render defect obstacle if active
        if self.obstacle == ObstacleKind::SharpCorner90 {
            // Place 90-degree corner obstacle along edge wave path
            let mid_idx = wave_pts.len() / 2;
            let obs_pos = wave_pts[mid_idx];
            let obs_size = resonator_r * 1.5;

            let obs_rect = Rect::from_center_size(obs_pos, vec2(obs_size, obs_size));
            painter.rect_filled(obs_rect, 2.0, COLOR_OBSTACLE_BLOCK);
            painter.rect_stroke(
                obs_rect,
                2.0,
                Stroke::new(2.0, Color32::WHITE),
                StrokeKind::Inside,
            );

            // Label obstacle
            painter.text(
                obs_pos + vec2(0.0, -obs_size * 0.8),
                egui::Align2::CENTER_CENTER,
                "90-Deg Corner",
                FontId::proportional(10.0),
                COLOR_OBSTACLE_BLOCK,
            );

            // Draw edge wave detour bending sharply around 90-degree corner
            let detour_p1 = obs_pos + vec2(-obs_size * 0.8, obs_size * 0.8);
            let detour_p2 = obs_pos + vec2(-obs_size * 0.8, -obs_size * 0.8);
            let detour_p3 = obs_pos + vec2(obs_size * 0.8, -obs_size * 0.8);

            painter.line_segment([detour_p1, detour_p2], Stroke::new(2.5, COLOR_ACTIVE_INPUT));
            painter.line_segment([detour_p2, detour_p3], Stroke::new(2.5, COLOR_ACTIVE_INPUT));
        } else if self.obstacle == ObstacleKind::SharpBend120 {
            let mid_idx = wave_pts.len() / 2;
            let obs_pos = wave_pts[mid_idx];
            let obs_r = resonator_r * 1.1;

            painter.circle_filled(obs_pos, obs_r, COLOR_OBSTACLE_BLOCK);
            painter.circle_stroke(
                obs_pos,
                obs_r,
                Stroke::new(2.0, Color32::WHITE),
            );
            painter.text(
                obs_pos + vec2(0.0, -obs_r * 1.3),
                egui::Align2::CENTER_CENTER,
                "120-Deg Bend",
                FontId::proportional(10.0),
                COLOR_OBSTACLE_BLOCK,
            );
        }

        // Wave packet markers flowing along edge
        for i in 0..5 {
            let idx = (wave_pts.len() * (i + 1)) / 7;
            if idx < wave_pts.len() {
                painter.circle_filled(
                    wave_pts[idx],
                    3.5,
                    COLOR_ACTIVE_INPUT,
                );
            }
        }
    }

    /// Renders the 3-port S-parameter spectrum plot over [3.0 kHz, 5.0 kHz].
    fn render_spectrum_plot(&self, ui: &mut Ui, h: f32) {
        let f0 = self.params.f_0_khz;
        let half_gap = self.topological_gap_khz / 2.0;

        let s21_points = PlotPoints::new(self.s21_curve.clone());
        let s31_points = PlotPoints::new(self.s31_curve.clone());
        let s11_points = PlotPoints::new(self.s11_curve.clone());

        Plot::new("chern_s_parameter_spectrum")
            .height(h)
            .legend(Legend::default())
            .x_axis_label("Frequency (kHz)")
            .y_axis_label("S-Parameter (dB)")
            .include_y(2.0)
            .include_y(-45.0)
            .show(ui, |plot_ui| {
                // Forward Transmission S_21 (IL <= 0.5 dB)
                plot_ui.line(
                    Line::new("S_21 Transmission (dB)", s21_points)
                        .color(COLOR_ACTIVE_INPUT)
                        .width(2.5),
                );

                // Reverse Isolation S_31 (Dip >= 35 dB)
                plot_ui.line(
                    Line::new("S_31 Isolation (dB)", s31_points)
                        .color(COLOR_ISOLATED_PORT)
                        .width(2.0),
                );

                // Return Loss S_11 (<= -25 dB)
                plot_ui.line(
                    Line::new("S_11 Return Loss (dB)", s11_points)
                        .color(COLOR_SPIN_ARROW)
                        .width(1.5),
                );

                // Isolation target threshold line (-35 dB)
                plot_ui.hline(
                    HLine::new("Isolation Target (-35 dB)", -35.0)
                        .color(Color32::from_rgba_unmultiplied(244, 63, 94, 160))
                        .style(egui_plot::LineStyle::Dashed { length: 5.0 })
                        .width(1.2),
                );

                // Insertion loss threshold line (-0.5 dB)
                plot_ui.hline(
                    HLine::new("Max Insertion Loss (-0.5 dB)", -0.5)
                        .color(Color32::from_rgba_unmultiplied(52, 211, 153, 160))
                        .style(egui_plot::LineStyle::Dashed { length: 5.0 })
                        .width(1.2),
                );

                // Topological bandgap vertical marker boundaries
                if half_gap > 0.05 {
                    plot_ui.vline(
                        VLine::new("Bandgap Lower Bound", f0 - half_gap)
                            .color(Color32::from_rgba_unmultiplied(56, 189, 248, 140))
                            .style(egui_plot::LineStyle::Dotted { spacing: 4.0 })
                            .width(1.0),
                    );
                    plot_ui.vline(
                        VLine::new("Bandgap Upper Bound", f0 + half_gap)
                            .color(Color32::from_rgba_unmultiplied(56, 189, 248, 140))
                            .style(egui_plot::LineStyle::Dotted { spacing: 4.0 })
                            .width(1.0),
                    );
                }
            });
    }

    /// Renders the Telemetry Footer with all key physical invariants and transport metrics.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(16.0, 4.0);

            // Chern Number C
            ui.horizontal(|ui| {
                ui.label(RichText::new("Chern Number:").size(11.0).color(Color32::from_rgb(160, 175, 195)));
                let c_color = if self.chern_number > 0 {
                    COLOR_ACTIVE_INPUT
                } else if self.chern_number < 0 {
                    COLOR_ISOLATED_PORT
                } else {
                    Color32::GRAY
                };
                ui.label(RichText::new(format!("C = {:+}", self.chern_number)).size(11.0).strong().color(c_color));
            });

            // Insertion Loss (dB)
            ui.horizontal(|ui| {
                ui.label(RichText::new("Insertion Loss:").size(11.0).color(Color32::from_rgb(160, 175, 195)));
                let il_col = if self.insertion_loss_db <= 0.50 {
                    COLOR_ACTIVE_INPUT
                } else {
                    COLOR_SPIN_ARROW
                };
                ui.label(RichText::new(format!("{:.2} dB", self.insertion_loss_db)).size(11.0).strong().color(il_col));
            });

            // Isolation (dB)
            ui.horizontal(|ui| {
                ui.label(RichText::new("Isolation:").size(11.0).color(Color32::from_rgb(160, 175, 195)));
                let iso_col = if self.isolation_db >= 35.0 {
                    COLOR_ACTIVE_INPUT
                } else {
                    COLOR_ISOLATED_PORT
                };
                ui.label(RichText::new(format!("{:.1} dB", self.isolation_db)).size(11.0).strong().color(iso_col));
            });

            // Corner Transmission (%)
            ui.horizontal(|ui| {
                ui.label(RichText::new("Corner Transmission:").size(11.0).color(Color32::from_rgb(160, 175, 195)));
                let t_col = if self.corner_transmission_percent >= 95.0 {
                    COLOR_ACTIVE_INPUT
                } else {
                    COLOR_OBSTACLE_BLOCK
                };
                ui.label(RichText::new(format!("{:.1}%", self.corner_transmission_percent)).size(11.0).strong().color(t_col));
            });

            // Topological Gap (kHz)
            ui.horizontal(|ui| {
                ui.label(RichText::new("Topological Gap:").size(11.0).color(Color32::from_rgb(160, 175, 195)));
                ui.label(RichText::new(format!("{:.2} kHz", self.topological_gap_khz)).size(11.0).strong().color(COLOR_ACTIVE_OUTPUT));
            });

            // Fluid Spin Rate Omega (rad/s)
            ui.horizontal(|ui| {
                ui.label(RichText::new("Fluid Spin Omega:").size(11.0).color(Color32::from_rgb(160, 175, 195)));
                ui.label(RichText::new(format!("{:.0} rad/s", self.fluid_spin_rate_rad_s)).size(11.0).strong().color(COLOR_SPIN_ARROW));
            });
        });
    }
}
