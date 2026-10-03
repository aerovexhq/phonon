#![deny(unsafe_code)]

//! Interactive Weyl and Dirac Semimetal Metamaterial Studio modal dialog.
//!
//! Provides:
//! - 3D Brillouin Zone Visualizer: interactive 3D momentum space view with Weyl nodes and Fermi arcs.
//! - 2D Surface Fermi Arc Contour View: open topological Fermi arcs connecting projected Weyl points.
//! - Chiral Anomaly Magnetoconductance Plot: quadratic magnetoconductance sigma(B) and negative longitudinal magnetoresistance (NLMR).
//! - Acoustic Beam Splitter Simulator: chiral valley wave packet routing into Port 1 and Port 2.
//! - Real-time topological invariant telemetry and Nielsen-Ninomiya conservation monitoring.

use egui::{
    pos2, vec2, Color32, FontId, Pos2, ProgressBar, Rect, RichText, Sense, Stroke, Ui,
};
use egui_plot::{Legend, Line, Plot, PlotPoints, Points};
use phonon_solver::weyl_semimetal::{
    ChiralAnomalyTransport, FermiArcSurface, WeylNodeType, WeylSemimetalModel,
};
use std::f64::consts::PI;

/// Operating symmetry mode of the semimetal metamaterial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemimetalMode {
    /// Time-reversal broken minimal 2-node Weyl semimetal.
    TrsBroken2Node,
    /// Inversion broken 4-node Weyl semimetal with preserved TRS.
    InversionBroken4Node,
    /// 4-band Dirac semimetal with degenerate pairs having zero net chirality.
    DiracSemimetal,
}

impl SemimetalMode {
    /// Label for UI dropdown.
    pub fn label(&self) -> &'static str {
        match self {
            Self::TrsBroken2Node => "TRS Broken (2 Weyl Nodes)",
            Self::InversionBroken4Node => "Inversion Broken (4 Weyl Nodes)",
            Self::DiracSemimetal => "Dirac Semimetal (Degenerate Nodes)",
        }
    }
}

/// Interactive modal dialog for topological Weyl and Dirac semimetal metamaterial design.
#[derive(Debug, Clone, PartialEq)]
pub struct WeylSemimetalDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Controls
    /// Symmetry breaking configuration.
    pub mode: SemimetalMode,
    /// Normalized tilt parameter t = |w| / v_F (t < 1.0 is Type-I, t > 1.0 is Type-II).
    pub tilt_parameter: f64,
    /// Normalized node separation Delta_k / (pi / a).
    pub node_separation_norm: f64,
    /// Applied magnetic field B in Tesla.
    pub magnetic_field_tesla: f64,
    /// Alignment angle theta between drive field E and magnetic field B in degrees.
    pub angle_eb_deg: f64,
    /// Fermi arc sagitta curvature factor.
    pub arc_curvature: f64,
    /// 3D BZ view rotation yaw angle in degrees.
    pub view_yaw_deg: f32,
    /// 3D BZ view rotation pitch angle in degrees.
    pub view_pitch_deg: f32,

    // Cached physics models
    /// Active bulk Weyl / Dirac model.
    pub model: WeylSemimetalModel,
    /// Active (001) surface Fermi arc model.
    pub fermi_arc_surface: Option<FermiArcSurface>,
    /// Active chiral anomaly transport kernel.
    pub transport: ChiralAnomalyTransport,

    // Cached telemetry metrics
    /// Sum of all chiral topological charges (Nielsen-Ninomiya: sum C = 0).
    pub total_monopole_charge: i32,
    /// Weyl point separation in units of pi/a.
    pub node_separation_pi_a: f64,
    /// Surface Fermi arc transmission efficiency percentage.
    pub fermi_arc_transmission_pct: f64,
    /// Chiral anomaly conductance enhancement ratio sigma(B) / sigma(0).
    pub chiral_anomaly_enhancement: f64,
    /// Acoustic beam splitter valley cross-talk isolation in dB.
    pub port_isolation_db: f64,
    /// Acoustic beam splitter Port 1 transmission percentage.
    pub port1_transmission_pct: f64,
    /// Acoustic beam splitter Port 2 transmission percentage.
    pub port2_transmission_pct: f64,
    /// Weyl node tilt classification.
    pub node_type: WeylNodeType,

    // Cached plot data
    /// Magnetoconductance curve points [B, sigma(B)].
    pub magnetoconductance_curve: Vec<[f64; 2]>,
    /// Magnetoresistance curve points [B, rho(B)].
    pub magnetoresistance_curve: Vec<[f64; 2]>,
    /// 2D surface Fermi arc trajectory points [kx, ky].
    pub fermi_arc_points: Vec<[f64; 2]>,

    /// Status message displayed in dialog footer.
    pub status_msg: String,
    /// Trigger flag for simulation updates.
    pub run_requested: bool,
}

impl Default for WeylSemimetalDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl WeylSemimetalDialog {
    /// Creates a new dialog instance with physical default parameters.
    pub fn new() -> Self {
        let vf = [1000.0, 1000.0, 1000.0];
        let tilt = [300.0, 0.0, 0.0];
        let default_model = WeylSemimetalModel::new_trs_broken_pair(0.6 * PI, vf, tilt, 1.0);
        let default_transport = ChiralAnomalyTransport::default();

        let mut dialog = Self {
            is_open: false,
            mode: SemimetalMode::TrsBroken2Node,
            tilt_parameter: 0.30,
            node_separation_norm: 0.60,
            magnetic_field_tesla: 3.0,
            angle_eb_deg: 0.0,
            arc_curvature: 0.25,
            view_yaw_deg: 35.0,
            view_pitch_deg: 25.0,

            model: default_model,
            fermi_arc_surface: None,
            transport: default_transport,

            total_monopole_charge: 0,
            node_separation_pi_a: 0.60,
            fermi_arc_transmission_pct: 97.4,
            chiral_anomaly_enhancement: 2.35,
            port_isolation_db: 34.2,
            port1_transmission_pct: 97.2,
            port2_transmission_pct: 0.05,
            node_type: WeylNodeType::TypeI,

            magnetoconductance_curve: Vec::new(),
            magnetoresistance_curve: Vec::new(),
            fermi_arc_points: Vec::new(),

            status_msg: "Weyl Semimetal Studio initialized.".to_string(),
            run_requested: false,
        };

        dialog.recompute();
        dialog
    }

    /// Recomputes bulk dispersion, surface Fermi arcs, chiral transport, and telemetry.
    pub fn recompute(&mut self) {
        let vf = [1000.0, 1000.0, 1000.0];
        let w_mag = self.tilt_parameter * vf[0];
        let tilt = [w_mag, 0.0, 0.0];
        let delta_k = self.node_separation_norm * PI;
        let lattice_a = 1.0;

        // 1. Bulk Model Construction
        self.model = match self.mode {
            SemimetalMode::TrsBroken2Node => {
                WeylSemimetalModel::new_trs_broken_pair(delta_k, vf, tilt, lattice_a)
            }
            SemimetalMode::InversionBroken4Node => {
                WeylSemimetalModel::new_inversion_broken_quad(delta_k, vf, tilt, lattice_a)
            }
            SemimetalMode::DiracSemimetal => {
                WeylSemimetalModel::new_dirac_semimetal(delta_k, vf, lattice_a)
            }
        };

        self.node_type = if self.tilt_parameter > 1.0 {
            WeylNodeType::TypeII
        } else {
            WeylNodeType::TypeI
        };
        self.total_monopole_charge = self.model.total_chirality();
        self.node_separation_pi_a = self.node_separation_norm;

        // 2. Surface Fermi Arc Construction
        let xi_0 = 1.2;
        let surface = match self.mode {
            SemimetalMode::TrsBroken2Node | SemimetalMode::InversionBroken4Node => {
                let half_sep = delta_k * 0.5;
                Some(FermiArcSurface::new(
                    [half_sep, 0.0],
                    [-half_sep, 0.0],
                    self.arc_curvature,
                    xi_0,
                    lattice_a,
                ))
            }
            SemimetalMode::DiracSemimetal => {
                // In Dirac semimetals, Fermi arcs merge into closed or trivial contours
                let half_sep = delta_k * 0.5;
                Some(FermiArcSurface::new(
                    [half_sep, 0.0],
                    [-half_sep, 0.0],
                    0.0,
                    xi_0,
                    lattice_a,
                ))
            }
        };

        if let Some(ref s) = surface {
            let trajectory = s.generate_arc_trajectory(80);
            self.fermi_arc_points = trajectory.iter().map(|p| [p.kx, p.ky]).collect();
            let base_t = 0.985 - 0.02 * (self.tilt_parameter - 0.3).max(0.0);
            self.fermi_arc_transmission_pct = (base_t * 100.0).clamp(94.0, 99.8);
        } else {
            self.fermi_arc_points.clear();
            self.fermi_arc_transmission_pct = 95.0;
        }
        self.fermi_arc_surface = surface;

        // 3. Chiral Anomaly Magnetotransport Sweeps
        let angle_rad = self.angle_eb_deg.to_radians();
        self.chiral_anomaly_enhancement = self
            .transport
            .conductivity_enhancement_factor(self.magnetic_field_tesla);

        let num_plot_pts = 100;
        let mut cond_pts = Vec::with_capacity(num_plot_pts);
        let mut res_pts = Vec::with_capacity(num_plot_pts);

        for i in 0..num_plot_pts {
            let b = 10.0 * (i as f64) / (num_plot_pts - 1) as f64;
            let sigma = self.transport.magnetoconductance(b, angle_rad);
            let rho = self.transport.magnetoresistance(b, angle_rad);
            cond_pts.push([b, sigma]);
            res_pts.push([b, rho]);
        }
        self.magnetoconductance_curve = cond_pts;
        self.magnetoresistance_curve = res_pts;

        // 4. Acoustic Beam Splitter Simulation
        let chirality_input = match self.mode {
            SemimetalMode::TrsBroken2Node | SemimetalMode::InversionBroken4Node => 1,
            SemimetalMode::DiracSemimetal => 0,
        };
        let beam = self
            .transport
            .route_chiral_beam(chirality_input, self.magnetic_field_tesla);
        self.port1_transmission_pct = beam.port1_transmission * 100.0;
        self.port2_transmission_pct = beam.port2_transmission * 100.0;
        self.port_isolation_db = beam.isolation_db;

        self.status_msg = format!(
            "Model: {}, Type: {}, Delta_k={:.2} pi/a, B={:.1} T, Iso={:.1} dB",
            self.mode.label(),
            self.node_type.as_str(),
            self.node_separation_norm,
            self.magnetic_field_tesla,
            self.port_isolation_db
        );
    }

    /// Primary render entry point for modal window.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Phonon Studio Universal Topological Dirac & Weyl Semimetal Metamaterial Simulator")
            .open(&mut is_open)
            .default_size([1040.0, 720.0])
            .min_size([820.0, 580.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders controls, plots, 3D visualizer, and telemetry panels.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut recompute = false;

        // 1. Top Controls Bar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 4.0);

            // Symmetry Mode Selector
            ui.label(RichText::new("Symmetry Mode:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            egui::ComboBox::from_id_salt("weyl_mode_combo")
                .selected_text(self.mode.label())
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.mode, SemimetalMode::TrsBroken2Node, SemimetalMode::TrsBroken2Node.label()).clicked() {
                        recompute = true;
                    }
                    if ui.selectable_value(&mut self.mode, SemimetalMode::InversionBroken4Node, SemimetalMode::InversionBroken4Node.label()).clicked() {
                        recompute = true;
                    }
                    if ui.selectable_value(&mut self.mode, SemimetalMode::DiracSemimetal, SemimetalMode::DiracSemimetal.label()).clicked() {
                        recompute = true;
                    }
                });

            ui.separator();

            // Tilt Slider
            ui.label(RichText::new("Tilt Parameter t:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.tilt_parameter, 0.0..=2.5)
                        .step_by(0.05)
                        .text(match self.node_type {
                            WeylNodeType::TypeI => "Type-I",
                            WeylNodeType::TypeII => "Type-II",
                        }),
                )
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            // Node Separation Slider
            ui.label(RichText::new("Separation Delta_k:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.node_separation_norm, 0.2..=1.4)
                        .step_by(0.05)
                        .suffix(" pi/a"),
                )
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            // Magnetic Field Slider
            ui.label(RichText::new("Magnetic Field B:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.magnetic_field_tesla, 0.0..=10.0)
                        .step_by(0.1)
                        .suffix(" T"),
                )
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            // Angle Slider
            ui.label(RichText::new("Angle theta(E, B):").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.angle_eb_deg, 0.0..=90.0)
                        .step_by(1.0)
                        .suffix(" deg"),
                )
                .changed()
            {
                recompute = true;
            }

            // Reset Button
            if ui.button("Reset Defaults").clicked() {
                self.mode = SemimetalMode::TrsBroken2Node;
                self.tilt_parameter = 0.30;
                self.node_separation_norm = 0.60;
                self.magnetic_field_tesla = 3.0;
                self.angle_eb_deg = 0.0;
                self.arc_curvature = 0.25;
                recompute = true;
            }
        });

        if recompute || self.run_requested {
            self.run_requested = false;
            self.recompute();
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 2. Main 2x2 Grid Visualization Canvas
        let total_avail = ui.available_size();
        let quad_w = ((total_avail.x - 12.0) * 0.5).max(380.0);
        let quad_h = ((total_avail.y - 120.0) * 0.5).max(220.0);

        ui.horizontal(|ui| {
            // Panel 1: 3D Brillouin Zone Visualizer
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("3D Brillouin Zone Momentum Space (Drag to Rotate)")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(120, 200, 255)),
                );
                self.render_3d_brillouin_zone(ui, quad_w, quad_h - 22.0);
            });

            ui.separator();

            // Panel 2: 2D Surface Fermi Arc Contour View
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("2D Surface Brillouin Zone (001) Open Fermi Arc")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(100, 240, 210)),
                );
                self.render_fermi_arc_plot(ui, quad_w, quad_h - 22.0);
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            // Panel 3: Chiral Anomaly Magnetoconductance Plot
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Chiral Anomaly Quadratic Magnetoconductance sigma(B)")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(255, 190, 80)),
                );
                self.render_chiral_anomaly_plot(ui, quad_w, quad_h - 22.0);
            });

            ui.separator();

            // Panel 4: Acoustic Beam Splitter Simulator
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Acoustic Metamaterial Chiral Valley Beam Splitter")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(230, 140, 255)),
                );
                self.render_beam_splitter_simulator(ui, quad_w, quad_h - 22.0);
            });
        });

        ui.add_space(6.0);
        ui.separator();

        // 3. Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders 3D Brillouin zone momentum space with interactive rotation.
    fn render_3d_brillouin_zone(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let (response, painter) = ui.allocate_painter(vec2(width, height), Sense::drag());

        if response.dragged() {
            let delta = response.drag_delta();
            self.view_yaw_deg += delta.x * 0.5;
            self.view_pitch_deg = (self.view_pitch_deg - delta.y * 0.5).clamp(-85.0, 85.0);
        }

        let rect = response.rect;
        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 16, 24));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(35, 45, 60)), egui::StrokeKind::Inside);

        let center = rect.center();
        let scale = width.min(height) * 0.32;

        let yaw_rad = (self.view_yaw_deg as f64).to_radians();
        let pitch_rad = (self.view_pitch_deg as f64).to_radians();
        let (cos_y, sin_y) = (yaw_rad.cos(), yaw_rad.sin());
        let (cos_p, sin_p) = (pitch_rad.cos(), pitch_rad.sin());

        // 3D projection transformation
        let project = |pt: [f64; 3]| -> Pos2 {
            let (x, y, z) = (pt[0], pt[1], pt[2]);
            // Rotate around Y (yaw)
            let x1 = cos_y * x + sin_y * z;
            let y1 = y;
            let z1 = -sin_y * x + cos_y * z;
            // Rotate around X (pitch)
            let x2 = x1;
            let y2 = cos_p * y1 - sin_p * z1;
            pos2(
                center.x + (x2 as f32) * scale,
                center.y - (y2 as f32) * scale,
            )
        };

        // Wireframe Brillouin zone boundary cube [-1, 1]^3
        let b = 0.85;
        let corners = [
            [-b, -b, -b], [b, -b, -b], [b, b, -b], [-b, b, -b],
            [-b, -b, b],  [b, -b, b],  [b, b, b],  [-b, b, b],
        ];
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0),
            (4, 5), (5, 6), (6, 7), (7, 4),
            (0, 4), (1, 5), (2, 6), (3, 7),
        ];

        let edge_stroke = Stroke::new(1.0, Color32::from_rgb(45, 60, 80));
        for &(i, j) in &edges {
            painter.line_segment([project(corners[i]), project(corners[j])], edge_stroke);
        }

        // Coordinate axes
        let origin = project([0.0, 0.0, 0.0]);
        let ax_len = 1.1;
        painter.line_segment([origin, project([ax_len, 0.0, 0.0])], Stroke::new(1.5, Color32::from_rgb(220, 80, 80)));
        painter.line_segment([origin, project([0.0, ax_len, 0.0])], Stroke::new(1.5, Color32::from_rgb(80, 200, 80)));
        painter.line_segment([origin, project([0.0, 0.0, ax_len])], Stroke::new(1.5, Color32::from_rgb(80, 120, 240)));

        painter.text(project([ax_len * 1.05, 0.0, 0.0]), egui::Align2::LEFT_CENTER, "kx", FontId::proportional(11.0), Color32::from_rgb(220, 100, 100));
        painter.text(project([0.0, ax_len * 1.05, 0.0]), egui::Align2::LEFT_CENTER, "ky", FontId::proportional(11.0), Color32::from_rgb(100, 220, 100));
        painter.text(project([0.0, 0.0, ax_len * 1.05]), egui::Align2::LEFT_CENTER, "kz", FontId::proportional(11.0), Color32::from_rgb(120, 160, 255));

        // Render Surface Fermi Arc on top boundary (kz = b)
        if !self.fermi_arc_points.is_empty() {
            let arc_stroke = Stroke::new(2.5, Color32::from_rgb(30, 235, 210));
            for w in 0..(self.fermi_arc_points.len() - 1) {
                let p1 = project([self.fermi_arc_points[w][0] / PI * b, self.fermi_arc_points[w][1] / PI * b, b]);
                let p2 = project([self.fermi_arc_points[w + 1][0] / PI * b, self.fermi_arc_points[w + 1][1] / PI * b, b]);
                painter.line_segment([p1, p2], arc_stroke);
            }
        }

        // Render Bulk Weyl Nodes
        for (idx, node) in self.model.nodes.iter().enumerate() {
            let norm_k = [
                node.k0[0] / PI * b,
                node.k0[1] / PI * b,
                node.k0[2] / PI * b,
            ];
            let screen_pos = project(norm_k);
            let (node_color, label) = if node.chirality == 1 {
                (Color32::from_rgb(255, 60, 60), format!("W+ ({})", idx + 1))
            } else {
                (Color32::from_rgb(60, 130, 255), format!("W- ({})", idx + 1))
            };

            // Glow and node circle
            painter.circle_filled(screen_pos, 7.0, Color32::from_rgba_unmultiplied(node_color.r(), node_color.g(), node_color.b(), 70));
            painter.circle_filled(screen_pos, 4.0, node_color);
            painter.text(
                pos2(screen_pos.x + 8.0, screen_pos.y - 8.0),
                egui::Align2::LEFT_BOTTOM,
                label,
                FontId::proportional(11.0),
                node_color,
            );
        }
    }

    /// Renders 2D surface Fermi arc plot connecting projected Weyl nodes.
    fn render_fermi_arc_plot(&self, ui: &mut Ui, width: f32, height: f32) {
        Plot::new("fermi_arc_surface_plot")
            .width(width)
            .height(height)
            .data_aspect(1.0)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                // Surface BZ boundary box
                let bz_bound = vec![
                    [-PI, -PI], [PI, -PI], [PI, PI], [-PI, PI], [-PI, -PI],
                ];
                plot_ui.line(
                    Line::new("Surface BZ Boundary", PlotPoints::new(bz_bound))
                        .color(Color32::from_rgb(60, 75, 95))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 75, 95))),
                );

                // Open Fermi arc trajectory
                if !self.fermi_arc_points.is_empty() {
                    plot_ui.line(
                        Line::new("Open Fermi Arc Contour", PlotPoints::new(self.fermi_arc_points.clone()))
                            .color(Color32::from_rgb(20, 240, 210))
                            .stroke(Stroke::new(3.0, Color32::from_rgb(20, 240, 210))),
                    );
                }

                // Projected bulk Weyl nodes
                if let Some(ref s) = self.fermi_arc_surface {
                    plot_ui.points(
                        Points::new("W+ Monopole (C = +1)", PlotPoints::new(vec![s.projected_w_plus]))
                            .color(Color32::from_rgb(255, 60, 60))
                            .radius(6.0),
                    );
                    plot_ui.points(
                        Points::new("W- Monopole (C = -1)", PlotPoints::new(vec![s.projected_w_minus]))
                            .color(Color32::from_rgb(60, 130, 255))
                            .radius(6.0),
                    );
                }
            });
    }

    /// Renders chiral anomaly quadratic magnetoconductance and NLMR plot.
    fn render_chiral_anomaly_plot(&self, ui: &mut Ui, width: f32, height: f32) {
        Plot::new("chiral_anomaly_magneto_plot")
            .width(width)
            .height(height)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Conductance sigma(B) ~ B^2", PlotPoints::new(self.magnetoconductance_curve.clone()))
                        .color(Color32::from_rgb(255, 180, 50))
                        .stroke(Stroke::new(2.5, Color32::from_rgb(255, 180, 50))),
                );
                plot_ui.line(
                    Line::new("Resistivity rho(B) [NLMR]", PlotPoints::new(self.magnetoresistance_curve.clone()))
                        .color(Color32::from_rgb(80, 200, 255))
                        .stroke(Stroke::new(2.0, Color32::from_rgb(80, 200, 255))),
                );

                // Current operating point marker
                let curr_cond = self.transport.magnetoconductance(
                    self.magnetic_field_tesla,
                    self.angle_eb_deg.to_radians(),
                );
                plot_ui.points(
                    Points::new("Operating Field B", PlotPoints::new(vec![[self.magnetic_field_tesla, curr_cond]]))
                        .color(Color32::WHITE)
                        .radius(5.0),
                );
            });
    }

    /// Renders acoustic beam splitter diagram and channel efficiency gauges.
    fn render_beam_splitter_simulator(&self, ui: &mut Ui, width: f32, height: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_height(height);

            // Channel gauges
            ui.horizontal(|ui| {
                ui.label(RichText::new("Port 1 (+1 Valley):").size(11.0).color(Color32::from_rgb(255, 120, 120)));
                ui.add(
                    ProgressBar::new((self.port1_transmission_pct / 100.0) as f32)
                        .text(format!("{:.1}%", self.port1_transmission_pct))
                        .desired_width(180.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Port 2 (-1 Valley):").size(11.0).color(Color32::from_rgb(120, 160, 255)));
                ui.add(
                    ProgressBar::new((self.port2_transmission_pct / 100.0) as f32)
                        .text(format!("{:.2}%", self.port2_transmission_pct))
                        .desired_width(180.0),
                );
            });

            // Metamaterial Router Schematic Canvas
            let canvas_h = (height - 65.0).max(120.0);
            let (response, painter) = ui.allocate_painter(vec2(width, canvas_h), Sense::hover());
            let r = response.rect;
            painter.rect_filled(r, 4.0, Color32::from_rgb(14, 18, 26));
            painter.rect_stroke(r, 4.0, Stroke::new(1.0, Color32::from_rgb(35, 45, 60)), egui::StrokeKind::Inside);

            let mid_y = r.center().y;
            let left_x = r.left() + 20.0;
            let junction_x = r.center().x - 20.0;
            let right_x = r.right() - 25.0;

            // Waveguide paths
            let stroke_in = Stroke::new(4.0, Color32::from_rgb(80, 180, 240));
            let stroke_p1 = Stroke::new(4.0, Color32::from_rgb(255, 90, 90));
            let stroke_p2 = Stroke::new(4.0, Color32::from_rgb(90, 140, 255));

            // Input waveguide
            painter.line_segment([pos2(left_x, mid_y), pos2(junction_x, mid_y)], stroke_in);
            painter.text(pos2(left_x, mid_y - 12.0), egui::Align2::LEFT_BOTTOM, "Input Waveguide", FontId::proportional(10.0), Color32::from_rgb(150, 190, 220));

            // Junction box
            let junc_rect = Rect::from_center_size(pos2(junction_x, mid_y), vec2(36.0, 36.0));
            painter.rect_filled(junc_rect, 3.0, Color32::from_rgb(30, 40, 55));
            painter.rect_stroke(junc_rect, 3.0, Stroke::new(1.5, Color32::from_rgb(80, 220, 200)), egui::StrokeKind::Inside);
            painter.text(junc_rect.center(), egui::Align2::CENTER_CENTER, "W", FontId::proportional(12.0), Color32::from_rgb(80, 220, 200));

            // Port 1 output (bends up)
            let p1_exit_y = r.top() + 25.0;
            painter.line_segment([pos2(junction_x + 18.0, mid_y - 8.0), pos2(right_x, p1_exit_y)], stroke_p1);
            painter.text(pos2(right_x, p1_exit_y - 8.0), egui::Align2::RIGHT_BOTTOM, format!("Port 1: {:.1}%", self.port1_transmission_pct), FontId::proportional(10.0), Color32::from_rgb(255, 120, 120));

            // Port 2 output (bends down)
            let p2_exit_y = r.bottom() - 25.0;
            painter.line_segment([pos2(junction_x + 18.0, mid_y + 8.0), pos2(right_x, p2_exit_y)], stroke_p2);
            painter.text(pos2(right_x, p2_exit_y + 8.0), egui::Align2::RIGHT_TOP, format!("Port 2: {:.2}%", self.port2_transmission_pct), FontId::proportional(10.0), Color32::from_rgb(120, 160, 255));
        });
    }

    /// Renders bottom telemetry cards and status summary.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(16.0, 0.0);

            // Monopole charge card
            ui.vertical(|ui| {
                ui.label(RichText::new("Total Monopole Charge").size(10.0).color(Color32::from_rgb(150, 170, 190)));
                let (chg_str, color) = if self.total_monopole_charge == 0 {
                    ("0 (Neutral - Nielsen-Ninomiya OK)", Color32::from_rgb(80, 230, 120))
                } else {
                    ("Violation != 0", Color32::from_rgb(255, 80, 80))
                };
                ui.label(RichText::new(chg_str).size(12.0).strong().color(color));
            });

            ui.separator();

            // Node separation card
            ui.vertical(|ui| {
                ui.label(RichText::new("Node Separation").size(10.0).color(Color32::from_rgb(150, 170, 190)));
                ui.label(RichText::new(format!("{:.2} pi/a", self.node_separation_pi_a)).size(12.0).strong().color(Color32::from_rgb(100, 200, 255)));
            });

            ui.separator();

            // Fermi Arc transmission card
            ui.vertical(|ui| {
                ui.label(RichText::new("Fermi Arc Transmission").size(10.0).color(Color32::from_rgb(150, 170, 190)));
                ui.label(RichText::new(format!("{:.1}%", self.fermi_arc_transmission_pct)).size(12.0).strong().color(Color32::from_rgb(60, 240, 200)));
            });

            ui.separator();

            // Chiral Anomaly enhancement card
            ui.vertical(|ui| {
                ui.label(RichText::new("Chiral Anomaly Gain").size(10.0).color(Color32::from_rgb(150, 170, 190)));
                ui.label(RichText::new(format!("{:.2}x", self.chiral_anomaly_enhancement)).size(12.0).strong().color(Color32::from_rgb(255, 200, 60)));
            });

            ui.separator();

            // Port isolation card
            ui.vertical(|ui| {
                ui.label(RichText::new("Valley Isolation").size(10.0).color(Color32::from_rgb(150, 170, 190)));
                let iso_color = if self.port_isolation_db >= 30.0 {
                    Color32::from_rgb(120, 240, 120)
                } else {
                    Color32::from_rgb(255, 140, 60)
                };
                ui.label(RichText::new(format!("{:.1} dB", self.port_isolation_db)).size(12.0).strong().color(iso_color));
            });

            ui.separator();

            // Classification badge
            ui.vertical(|ui| {
                ui.label(RichText::new("Weyl Node Type").size(10.0).color(Color32::from_rgb(150, 170, 190)));
                let (badge, bcolor) = match self.node_type {
                    WeylNodeType::TypeI => ("Type-I (Point)", Color32::from_rgb(100, 180, 255)),
                    WeylNodeType::TypeII => ("Type-II (Overtilted)", Color32::from_rgb(255, 140, 60)),
                };
                ui.label(RichText::new(badge).size(12.0).strong().color(bcolor));
            });
        });

        ui.add_space(2.0);
        ui.label(RichText::new(&self.status_msg).size(10.0).color(Color32::from_rgb(130, 150, 170)));
    }
}
