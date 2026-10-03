#![deny(unsafe_code)]

//! Interactive Non-Hermitian Chiral Exceptional Surface Studio Visualizer for Phonon CAD Studio.
//!
//! Provides:
//! - 3D/2D Riemann Eigenvalue Sheets: complex eigenvalue sheets Re(lambda) over parameter coordinates
//!   (gamma_1, gamma_2), rendering the continuous exceptional surface seam where branches merge.
//! - Directional Chiral Sensitivity Polar Plot: polar response diagram rendering sensitivity
//!   enhancement eta(theta) in dB as a function of incident acoustic angle theta in [0, 360 deg].
//! - Ultrasonic Transducer Sensor Array Canvas: 2D schematic diagram of the N-element phased
//!   sensor array, showing beam steering, alternating gain/loss weighting, and coupling waveguides.
//! - Fractional Perturbation Splitting Curve: log-log comparison between Non-Hermitian square-root
//!   sensitivity (Delta lambda ~ sqrt(epsilon)) and linear Hermitian response (Delta lambda ~ epsilon).
//! - Controls: Gain/Loss parameters gamma_1, gamma_2, coupling kappa, incident acoustic angle theta,
//!   perturbation amplitude epsilon, array element count N, "Optimal ES Operating Point" button.
//! - Telemetry Footer: ES Order (ES2/ES3), Sensitivity Enhancement (x), Directivity (dB),
//!   Petermann Factor K, SNR Net Gain, Coalescence Residual.

use egui::{
    vec2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use phonon_solver::exceptional_surface::{
    ChiralDirectionalMetrics, EsManifoldParams, ExceptionalSurfaceArray,
    ExceptionalSurfaceHamiltonian, RiemannSheetPoint, SnrAnalysis, SurfaceEigenvalues,
};

/// Palette colors for Exceptional Surface studio visualizer.
const COLOR_SURFACE_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const COLOR_SURFACE_PURPLE: Color32 = Color32::from_rgb(168, 85, 247);
const COLOR_SURFACE_EMERALD: Color32 = Color32::from_rgb(52, 211, 153);
const COLOR_GAIN_RED: Color32 = Color32::from_rgb(239, 68, 68);
const COLOR_LOSS_BLUE: Color32 = Color32::from_rgb(59, 130, 246);
const COLOR_POLAR_GOLD: Color32 = Color32::from_rgb(250, 204, 21);
const COLOR_HERM_GRAY: Color32 = Color32::from_rgb(148, 163, 184);

/// Modal dialog for the Phonon Non-Hermitian Chiral Exceptional Surface Sensor Studio.
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionalSurfaceDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Controls
    /// Bare acoustic resonance frequency omega_0 in MHz.
    pub omega_0_mhz: f64,
    /// Primary non-Hermitian gain/loss rate gamma_1 in MHz.
    pub gamma_1_mhz: f64,
    /// Secondary non-Hermitian manifold parameter gamma_2 in MHz.
    pub gamma_2_mhz: f64,
    /// Inter-cavity acoustic coupling strength kappa in MHz.
    pub kappa_mhz: f64,
    /// Incident acoustic wave angle theta in degrees [0, 360).
    pub incident_angle_deg: f64,
    /// Perturbation amplitude epsilon.
    pub perturbation_eps: f64,
    /// Number of ultrasonic transducer array elements.
    pub num_elements: usize,

    // Cached Simulation State
    pub array: ExceptionalSurfaceArray,
    pub directional_metrics: ChiralDirectionalMetrics,
    pub snr_analysis: SnrAnalysis,
    pub eigenvalues: SurfaceEigenvalues,
    pub riemann_points: Vec<RiemannSheetPoint>,

    // Cached Plot Curves
    pub riemann_branch_1: Vec<[f64; 2]>,
    pub riemann_branch_2: Vec<[f64; 2]>,
    pub riemann_branch_3: Vec<[f64; 2]>,
    pub polar_curve: Vec<[f64; 2]>,
    pub sensitivity_curve_nh: Vec<[f64; 2]>,
    pub sensitivity_curve_herm: Vec<[f64; 2]>,

    pub status_msg: String,
}

impl Default for ExceptionalSurfaceDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ExceptionalSurfaceDialog {
    /// Creates a new `ExceptionalSurfaceDialog` initialized on the exceptional surface manifold.
    pub fn new() -> Self {
        let params = EsManifoldParams::default();
        let omega_0_mhz = params.omega_0_mhz;
        let gamma_1_mhz = params.gamma_1_mhz;
        let gamma_2_mhz = params.gamma_2_mhz;
        let kappa_mhz = params.kappa_mhz;
        let incident_angle_deg = 0.0;
        let perturbation_eps = 1e-3;
        let num_elements = 8;

        let array = ExceptionalSurfaceArray::new(params, num_elements, 2.5);
        let directional_metrics = array.evaluate_directional_sensing(incident_angle_deg, perturbation_eps);
        let snr_analysis = array.evaluate_snr_advantage(perturbation_eps);
        let eigenvalues = array.hamiltonian.solve_eigenvalues();
        let riemann_points = array.hamiltonian.compute_riemann_surface(0.5, 3.5, 0.5, 2.5, 16, 16);

        let mut dialog = Self {
            is_open: false,
            omega_0_mhz,
            gamma_1_mhz,
            gamma_2_mhz,
            kappa_mhz,
            incident_angle_deg,
            perturbation_eps,
            num_elements,
            array,
            directional_metrics,
            snr_analysis,
            eigenvalues,
            riemann_points,
            riemann_branch_1: Vec::new(),
            riemann_branch_2: Vec::new(),
            riemann_branch_3: Vec::new(),
            polar_curve: Vec::new(),
            sensitivity_curve_nh: Vec::new(),
            sensitivity_curve_herm: Vec::new(),
            status_msg: "Exceptional Surface manifold mode locked. Coalesced eigenspace active.".to_string(),
        };

        dialog.update_cached_curves();
        dialog
    }

    /// Automatically tunes coupling kappa to lock exactly onto the Exceptional Surface manifold.
    pub fn apply_optimal_operating_point(&mut self) {
        let tuned = EsManifoldParams::on_surface(
            self.omega_0_mhz,
            self.gamma_1_mhz,
            self.gamma_2_mhz,
            self.array.hamiltonian.params.asymmetry_alpha,
        );
        self.kappa_mhz = tuned.kappa_mhz;
        self.recompute();
        self.status_msg = format!(
            "Locked to Exceptional Surface: gamma_1 = {:.2} MHz, gamma_2 = {:.2} MHz, kappa = {:.4} MHz.",
            self.gamma_1_mhz, self.gamma_2_mhz, self.kappa_mhz
        );
    }

    /// Recomputes simulation state with current slider parameter values.
    pub fn recompute(&mut self) {
        let mut params = self.array.hamiltonian.params;
        params.omega_0_mhz = self.omega_0_mhz;
        params.gamma_1_mhz = self.gamma_1_mhz;
        params.gamma_2_mhz = self.gamma_2_mhz;
        params.kappa_mhz = self.kappa_mhz;

        self.array = ExceptionalSurfaceArray::new(params, self.num_elements, 2.5);
        self.eigenvalues = self.array.hamiltonian.solve_eigenvalues();
        self.directional_metrics = self
            .array
            .evaluate_directional_sensing(self.incident_angle_deg, self.perturbation_eps);
        self.snr_analysis = self.array.evaluate_snr_advantage(self.perturbation_eps);

        self.update_cached_curves();
    }

    /// Updates cached plot curves for high-performance immediate-mode rendering.
    fn update_cached_curves(&mut self) {
        // Polar response curve: x = R * cos(theta), y = R * sin(theta) with R in dB
        self.polar_curve = self
            .directional_metrics
            .polar_response
            .iter()
            .map(|&(deg, db)| {
                let th = deg.to_radians();
                // Map dB [-40, 0] to positive radius [0.0, 1.0]
                let r = ((db + 40.0) / 40.0).max(0.02);
                [r * th.cos(), r * th.sin()]
            })
            .collect();

        // Sensitivity vs perturbation log-log curves
        let (nh_c, herm_c) = self
            .array
            .compute_sensitivity_comparison(1e-5, 1e-1, 40);
        self.sensitivity_curve_nh = nh_c;
        self.sensitivity_curve_herm = herm_c;

        // Riemann sheets 1D cross-section along gamma_1
        let g2 = self.gamma_2_mhz;
        let mut b1 = Vec::with_capacity(30);
        let mut b2 = Vec::with_capacity(30);
        let mut b3 = Vec::with_capacity(30);

        for i in 0..30 {
            let g1 = 0.5 + (i as f64) * (3.0 / 29.0);
            let mut test_params = self.array.hamiltonian.params;
            test_params.gamma_1_mhz = g1;
            test_params.gamma_2_mhz = g2;
            let h = ExceptionalSurfaceHamiltonian::new(test_params);
            let ev = h.solve_eigenvalues();
            b1.push([g1, ev.lambda_1.re]);
            b2.push([g1, ev.lambda_2.re]);
            b3.push([g1, ev.lambda_3.re]);
        }
        self.riemann_branch_1 = b1;
        self.riemann_branch_2 = b2;
        self.riemann_branch_3 = b3;
    }

    /// Main entry point for rendering the modal dialog within an egui Context.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Non-Hermitian Chiral Exceptional Surface Studio")
            .open(&mut is_open)
            .default_size(vec2(1040.0, 680.0))
            .min_size(vec2(860.0, 560.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the entire internal contents of the Exceptional Surface Studio.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut dirty = false;

        // Top Control Toolbar
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Operating Parameters:")
                    .color(COLOR_SURFACE_CYAN)
                    .strong(),
            );

            ui.add(
                egui::Slider::new(&mut self.gamma_1_mhz, 0.5..=4.0)
                    .text("gamma_1 (MHz)")
                    .step_by(0.05),
            );
            if ui.add(
                egui::Slider::new(&mut self.gamma_2_mhz, 0.5..=3.0)
                    .text("gamma_2 (MHz)")
                    .step_by(0.05),
            ).changed() {
                dirty = true;
            }

            if ui.add(
                egui::Slider::new(&mut self.kappa_mhz, 0.1..=4.0)
                    .text("kappa (MHz)")
                    .step_by(0.01),
            ).changed() {
                dirty = true;
            }

            if ui
                .button(RichText::new("Lock to Exceptional Surface").strong())
                .clicked()
            {
                self.apply_optimal_operating_point();
            }
        });

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Acoustic Sensing Probe:")
                    .color(COLOR_POLAR_GOLD)
                    .strong(),
            );

            if ui.add(
                egui::Slider::new(&mut self.incident_angle_deg, 0.0..=360.0)
                    .text("theta (deg)")
                    .step_by(5.0),
            ).changed() {
                dirty = true;
            }

            if ui.add(
                egui::Slider::new(&mut self.perturbation_eps, 1e-4..=1e-2)
                    .text("epsilon")
                    .logarithmic(true),
            ).changed() {
                dirty = true;
            }

            if ui.add(
                egui::Slider::new(&mut self.num_elements, 4..=16)
                    .text("Array Elements")
                    .step_by(2.0),
            ).changed() {
                dirty = true;
            }
        });

        if dirty {
            self.recompute();
        }

        ui.separator();

        // Status banner
        ui.horizontal(|ui| {
            ui.label(RichText::new(&self.status_msg).color(Color32::LIGHT_GRAY).size(11.0));
        });

        ui.separator();

        // 2x2 Visual Studio Grid
        let avail_h = ui.available_height() - 36.0;
        let sub_h = (avail_h * 0.48).max(180.0);

        ui.columns(2, |cols| {
            // Top-Left: Ultrasonic Phased Sensor Array Canvas
            cols[0].group(|ui| {
                ui.label(
                    RichText::new("Phased Ultrasonic Transducer Array Layout")
                        .color(COLOR_SURFACE_EMERALD)
                        .strong(),
                );
                self.render_array_canvas(ui, sub_h - 28.0);
            });

            // Top-Right: Chiral Polar Directivity Response
            cols[1].group(|ui| {
                ui.label(
                    RichText::new("Directional Chiral Sensitivity Polar Pattern eta(theta)")
                        .color(COLOR_POLAR_GOLD)
                        .strong(),
                );
                self.render_polar_plot(ui, sub_h - 28.0);
            });
        });

        ui.columns(2, |cols| {
            // Bottom-Left: Complex Riemann Eigenvalue Sheets
            cols[0].group(|ui| {
                ui.label(
                    RichText::new("Eigenvalue Riemann Sheets Re(lambda) vs gamma_1")
                        .color(COLOR_SURFACE_CYAN)
                        .strong(),
                );
                self.render_riemann_plot(ui, sub_h - 28.0);
            });

            // Bottom-Right: Fractional Perturbation Splitting Curve
            cols[1].group(|ui| {
                ui.label(
                    RichText::new("Fractional Sensitivity Delta lambda vs Perturbation epsilon")
                        .color(COLOR_SURFACE_PURPLE)
                        .strong(),
                );
                self.render_sensitivity_plot(ui, sub_h - 28.0);
            });
        });

        ui.separator();

        // Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders the 2D schematic diagram of the ultrasonic sensor array.
    fn render_array_canvas(&self, ui: &mut Ui, h: f32) {
        let (response, painter) =
            ui.allocate_painter(vec2(ui.available_width(), h), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));

        let center = rect.center();
        let n = self.array.elements.len();
        if n == 0 {
            return;
        }

        let spacing = (rect.width() * 0.7) / (n as f32);
        let start_x = center.x - (n as f32 - 1.0) * spacing * 0.5;

        // Substrate waveguiding bar
        let bar_rect = Rect::from_center_size(
            Pos2::new(center.x, center.y + 10.0),
            vec2(spacing * (n as f32 + 0.5), 14.0),
        );
        painter.rect_filled(bar_rect, 2.0, Color32::from_rgb(30, 41, 59));
        painter.rect_stroke(
            bar_rect,
            2.0,
            Stroke::new(1.0, Color32::from_rgb(51, 65, 85)),
            StrokeKind::Inside,
        );

        // Render each transducer element
        for (i, elem) in self.array.elements.iter().enumerate() {
            let elem_x = start_x + (i as f32) * spacing;
            let elem_y = center.y - 10.0;

            let elem_color = if elem.gain_weight > 0.0 {
                COLOR_GAIN_RED
            } else {
                COLOR_LOSS_BLUE
            };

            let elem_box = Rect::from_center_size(Pos2::new(elem_x, elem_y), vec2(16.0, 24.0));
            painter.rect_filled(elem_box, 3.0, elem_color);
            painter.rect_stroke(
                elem_box,
                3.0,
                Stroke::new(1.0, Color32::WHITE),
                StrokeKind::Inside,
            );

            // Index label
            painter.text(
                Pos2::new(elem_x, elem_y - 18.0),
                egui::Align2::CENTER_CENTER,
                format!("#{}", elem.index + 1),
                FontId::proportional(10.0),
                Color32::LIGHT_GRAY,
            );
        }

        // Acoustic incidence wave vector arrow
        let theta_rad = self.incident_angle_deg.to_radians();
        let arrow_len = 35.0;
        let arrow_start = Pos2::new(
            center.x - arrow_len * (theta_rad.cos() as f32),
            center.y - 50.0 - arrow_len * (theta_rad.sin() as f32),
        );
        let arrow_end = Pos2::new(center.x, center.y - 50.0);

        painter.line_segment(
            [arrow_start, arrow_end],
            Stroke::new(2.5, COLOR_POLAR_GOLD),
        );
        painter.text(
            Pos2::new(arrow_start.x, arrow_start.y - 8.0),
            egui::Align2::CENTER_CENTER,
            format!("k_inc ({:.0} deg)", self.incident_angle_deg),
            FontId::proportional(11.0),
            COLOR_POLAR_GOLD,
        );
    }

    /// Renders the directional chiral sensitivity polar pattern plot.
    fn render_polar_plot(&self, ui: &mut Ui, h: f32) {
        let pts: PlotPoints = self.polar_curve.clone().into();
        let polar_line = Line::new("eta(theta)", pts)
            .color(COLOR_POLAR_GOLD)
            .width(2.5);

        Plot::new("polar_directivity_plot")
            .height(h)
            .data_aspect(1.0)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(polar_line);
            });
    }

    /// Renders the eigenvalue Riemann sheet cross-section plot.
    fn render_riemann_plot(&self, ui: &mut Ui, h: f32) {
        let b1_pts: PlotPoints = self.riemann_branch_1.clone().into();
        let b2_pts: PlotPoints = self.riemann_branch_2.clone().into();
        let b3_pts: PlotPoints = self.riemann_branch_3.clone().into();

        let line_1 = Line::new("Re(lambda_1)", b1_pts)
            .color(COLOR_SURFACE_CYAN)
            .width(2.0);
        let line_2 = Line::new("Re(lambda_2)", b2_pts)
            .color(COLOR_SURFACE_PURPLE)
            .width(2.0);
        let line_3 = Line::new("Re(lambda_3)", b3_pts)
            .color(COLOR_HERM_GRAY)
            .width(1.5);

        Plot::new("riemann_sheets_plot")
            .height(h)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(line_1);
                plot_ui.line(line_2);
                plot_ui.line(line_3);
            });
    }

    /// Renders the fractional perturbation splitting log-log plot.
    fn render_sensitivity_plot(&self, ui: &mut Ui, h: f32) {
        let nh_pts: PlotPoints = self.sensitivity_curve_nh.clone().into();
        let herm_pts: PlotPoints = self.sensitivity_curve_herm.clone().into();

        let nh_line = Line::new("Exceptional Surface (Delta lambda ~ sqrt(eps))", nh_pts)
            .color(COLOR_SURFACE_EMERALD)
            .width(2.5);
        let herm_line = Line::new("Hermitian Linear Baseline (Delta lambda ~ eps)", herm_pts)
            .color(COLOR_HERM_GRAY)
            .width(1.5);

        Plot::new("sensitivity_comparison_plot")
            .height(h)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(nh_line);
                plot_ui.line(herm_line);
            });
    }

    /// Renders the telemetry footer displaying physical metrics.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let is_on_surface = self.eigenvalues.min_splitting < 0.15;
            let (badge_bg, badge_txt, badge_label) = if is_on_surface {
                (COLOR_SURFACE_EMERALD, Color32::BLACK, "ON EXCEPTIONAL SURFACE")
            } else {
                (Color32::from_rgb(249, 115, 22), Color32::BLACK, "DETUNED OFF-SURFACE")
            };

            egui::Frame::new()
                .fill(badge_bg)
                .corner_radius(3.0)
                .inner_margin(egui::Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.label(RichText::new(badge_label).color(badge_txt).size(11.0).strong());
                });

            ui.separator();

            ui.label(
                RichText::new(format!(
                    "Sensitivity Enhancement: {:.1}x",
                    self.directional_metrics.enhancement_factor
                ))
                .color(COLOR_SURFACE_CYAN)
                .size(11.0),
            );

            ui.separator();

            ui.label(
                RichText::new(format!(
                    "Directivity: {:.1} dB",
                    self.directional_metrics.directivity_db
                ))
                .color(COLOR_POLAR_GOLD)
                .size(11.0),
            );

            ui.separator();

            ui.label(
                RichText::new(format!(
                    "Petermann Factor K: {:.1e}",
                    self.snr_analysis.petermann_factor
                ))
                .color(Color32::LIGHT_GRAY)
                .size(11.0),
            );

            ui.separator();

            let snr_color = if self.snr_analysis.is_subthreshold_advantage {
                COLOR_SURFACE_EMERALD
            } else {
                Color32::LIGHT_GRAY
            };
            ui.label(
                RichText::new(format!(
                    "Net SNR Gain: {:.2}x",
                    self.snr_analysis.net_snr_gain
                ))
                .color(snr_color)
                .size(11.0),
            );

            ui.separator();

            ui.label(
                RichText::new(format!(
                    "Coalescence Residual: {:.4} MHz",
                    self.eigenvalues.coalescence_residual
                ))
                .color(Color32::GRAY)
                .size(11.0),
            );
        });
    }
}
