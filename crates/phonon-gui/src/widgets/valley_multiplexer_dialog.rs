#![deny(unsafe_code)]

//! Interactive Valley-Polarized Topological Acoustic Multiplexer & Beam Splitter Studio Dialog.
//!
//! Provides:
//! - 2D Junction Field Canvas: interactive 2D spatial acoustic pressure distribution P(x, y)
//!   routing valley-locked acoustic wave packets into Port 2 (+60 deg) vs Port 3 (-60 deg).
//! - Berry Curvature Contour: native egui_plot rendering Berry curvature peaks (+0.5 at K, -0.5 at K').
//! - Gapless Kink Edge Dispersion: native egui_plot showing bulk band edges and gapless valley kink states.
//! - S-Parameter Transmission Spectrum: native egui_plot of S21(f), S31(f), and return loss S11(f) in dB.
//! - Controls & Presets: Valley polarization slider, inversion asymmetry Delta, tunable beam splitting bias,
//!   sharp 60-degree corner bend defect toggle, and operating frequency.
//! - Telemetry Footer: Valley Chern Number Cv, Topological Gap (Hz), S21 Transmission (dB),
//!   S31 Transmission (dB), Valley Contrast Ratio (%), Corner Immunity (%).

use egui::{
    pos2, vec2, Color32, FontId, Rect, RichText, Sense, Stroke, Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints};
use phonon_solver::valley_acoustic_multiplexer::{
    MultiplexerJunctionParams, MultiplexerSParameters, ValleyBerryCurvature, ValleyIndex,
    ValleyLatticeParams, ValleyLatticeSolver, ValleyMultiplexerSolver,
};
use std::f64::consts::PI;

/// Palette colors for the valley-polarized multiplexer studio.
const COLOR_VALLEY_K: Color32 = Color32::from_rgb(59, 130, 246); // Blue (+K)
const COLOR_VALLEY_KP: Color32 = Color32::from_rgb(239, 68, 68); // Red (-K')
const COLOR_PORT2_TRANS: Color32 = Color32::from_rgb(52, 211, 153); // Emerald
const COLOR_PORT3_TRANS: Color32 = Color32::from_rgb(244, 63, 94); // Rose
const COLOR_REFLECTION: Color32 = Color32::from_rgb(245, 158, 11); // Amber
const COLOR_PRESSURE_POS: Color32 = Color32::from_rgb(56, 189, 248); // Cyan
const COLOR_PRESSURE_NEG: Color32 = Color32::from_rgb(168, 85, 247); // Purple
const COLOR_CHANNEL_WALL: Color32 = Color32::from_rgb(203, 213, 225); // Slate Light

/// Active view tab in the valley multiplexer dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValleyDialogTab {
    /// 2D Junction Field & Channel Routing Canvas.
    JunctionFieldCanvas,
    /// Berry Curvature Omega_z around K and K' valleys.
    BerryCurvatureContour,
    /// Gapless Kink Edge State Dispersion across Valley Bandgap.
    KinkEdgeDispersion,
    /// S-Parameter Frequency Spectrum (S21, S31, S11).
    SParameterSpectrum,
}

/// Interactive modal dialog for the Valley-Polarized Acoustic Multiplexer & Beam Splitter.
#[derive(Debug, Clone)]
pub struct ValleyMultiplexerDialog {
    /// Window visibility toggle.
    pub is_open: bool,
    /// Current selected view tab.
    pub active_tab: ValleyDialogTab,

    // Solver engines
    pub lattice_solver: ValleyLatticeSolver,
    pub mux_solver: ValleyMultiplexerSolver,

    // Control slider buffers
    pub valley_polarization: f64,
    pub asymmetry_delta: f64,
    pub splitting_bias: f64,
    pub has_corner_bend: bool,
    pub operating_frequency_khz: f64,

    // Cached Berry metrics
    pub berry_metrics: ValleyBerryCurvature,
}

impl Default for ValleyMultiplexerDialog {
    fn default() -> Self {
        let lattice_params = ValleyLatticeParams::default();
        let lattice_solver = ValleyLatticeSolver::new(lattice_params.clone());
        let berry_metrics = lattice_solver.compute_berry_metrics();

        let mux_params = MultiplexerJunctionParams {
            lattice: lattice_params,
            valley_polarization: 1.0,
            splitting_bias: 0.0,
            has_corner_bend: false,
            corner_bend_angle_deg: 60.0,
            operating_frequency_hz: 4800.0,
            nx: 80,
            ny: 60,
            domain_width_m: 0.06,
            domain_height_m: 0.045,
        };

        let mux_solver = ValleyMultiplexerSolver::new(mux_params);

        Self {
            is_open: false,
            active_tab: ValleyDialogTab::JunctionFieldCanvas,
            lattice_solver,
            mux_solver,
            valley_polarization: 1.0,
            asymmetry_delta: 0.22,
            splitting_bias: 0.0,
            has_corner_bend: false,
            operating_frequency_khz: 4.8,
            berry_metrics,
        }
    }
}

impl ValleyMultiplexerDialog {
    /// Construct a new dialog with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast cold-boot constructor that defers heavy 2D pressure field synthesis
    /// until the dialog is opened.
    pub fn new_fast() -> Self {
        let lattice_params = ValleyLatticeParams::default();
        let lattice_solver = ValleyLatticeSolver::new(lattice_params.clone());
        let berry_metrics = lattice_solver.compute_berry_metrics();

        let mux_params = MultiplexerJunctionParams {
            lattice: lattice_params,
            valley_polarization: 1.0,
            splitting_bias: 0.0,
            has_corner_bend: false,
            corner_bend_angle_deg: 60.0,
            operating_frequency_hz: 4800.0,
            nx: 80,
            ny: 60,
            domain_width_m: 0.06,
            domain_height_m: 0.045,
        };

        let mux_solver = ValleyMultiplexerSolver {
            params: mux_params,
            pressure_field: Vec::new(),
            intensity_field: Vec::new(),
            s_parameters: MultiplexerSParameters {
                s21_db: -0.35,
                s31_db: -35.0,
                s11_db: -28.0,
                valley_isolation_db: 34.65,
                valley_contrast_ratio: 0.98,
                corner_immunity_ratio: 0.96,
                splitting_ratio: 0.98,
            },
        };

        Self {
            is_open: false,
            active_tab: ValleyDialogTab::JunctionFieldCanvas,
            lattice_solver,
            mux_solver,
            valley_polarization: 1.0,
            asymmetry_delta: 0.22,
            splitting_bias: 0.0,
            has_corner_bend: false,
            operating_frequency_khz: 4.8,
            berry_metrics,
        }
    }

    /// Primary modal UI entry point.
    pub fn ui(&mut self, ctx: &egui::Context) {
        self.show(ctx);
    }

    /// Recompute all solver fields with current control parameters.
    pub fn recompute(&mut self) {
        self.lattice_solver.params.asymmetry_delta = self.asymmetry_delta;
        self.lattice_solver.params.dirac_frequency_hz = self.operating_frequency_khz * 1000.0;
        self.berry_metrics = self.lattice_solver.compute_berry_metrics();

        self.mux_solver.params.lattice = self.lattice_solver.params.clone();
        self.mux_solver.params.valley_polarization = self.valley_polarization;
        self.mux_solver.params.splitting_bias = self.splitting_bias;
        self.mux_solver.params.has_corner_bend = self.has_corner_bend;
        self.mux_solver.params.operating_frequency_hz = self.operating_frequency_khz * 1000.0;
        self.mux_solver.solve();
    }

    /// Main rendering entry point for the modal window.
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        if self.mux_solver.pressure_field.is_empty() {
            self.recompute();
        }

        let mut is_open = self.is_open;
        egui::Window::new(
            RichText::new("Valley-Polarized Topological Acoustic Multiplexer & Beam Splitter")
                .strong()
                .size(15.0),
        )
        .open(&mut is_open)
        .resizable(true)
        .default_width(840.0)
        .default_height(600.0)
        .show(ctx, |ui| {
            self.render_content(ui);
        });
        self.is_open = is_open;
    }

    /// Render inner dialog content.
    pub fn render_content(&mut self, ui: &mut Ui) {
        if self.mux_solver.pressure_field.is_empty() {
            self.recompute();
        }

        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.active_tab,
                ValleyDialogTab::JunctionFieldCanvas,
                RichText::new("2D Junction Field").strong(),
            );
            ui.selectable_value(
                &mut self.active_tab,
                ValleyDialogTab::BerryCurvatureContour,
                RichText::new("Berry Curvature Peaks").strong(),
            );
            ui.selectable_value(
                &mut self.active_tab,
                ValleyDialogTab::KinkEdgeDispersion,
                RichText::new("Kink Edge Dispersion").strong(),
            );
            ui.selectable_value(
                &mut self.active_tab,
                ValleyDialogTab::SParameterSpectrum,
                RichText::new("S-Parameter Spectrum").strong(),
            );
        });

        ui.separator();

        // Control Toolbar
        self.render_controls(ui);

        ui.separator();

        // Active Tab Visualizations
        match self.active_tab {
            ValleyDialogTab::JunctionFieldCanvas => self.render_junction_canvas(ui),
            ValleyDialogTab::BerryCurvatureContour => self.render_berry_curvature_plot(ui),
            ValleyDialogTab::KinkEdgeDispersion => self.render_kink_dispersion_plot(ui),
            ValleyDialogTab::SParameterSpectrum => self.render_s_parameters_plot(ui),
        }

        ui.separator();

        // Telemetry Footer
        self.render_telemetry(ui);
    }

    /// Render presets and interactive sliders.
    fn render_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Presets:").strong());

            if ui
                .button("Valley K -> Port 2")
                .on_hover_text("Inject pure K-valley acoustic waves directed to upper output")
                .clicked()
            {
                self.valley_polarization = 1.0;
                self.splitting_bias = 0.0;
                self.has_corner_bend = false;
                self.recompute();
            }

            if ui
                .button("Valley K' -> Port 3")
                .on_hover_text("Inject pure K'-valley acoustic waves directed to lower output")
                .clicked()
            {
                self.valley_polarization = -1.0;
                self.splitting_bias = 0.0;
                self.has_corner_bend = false;
                self.recompute();
            }

            if ui
                .button("50:50 Balanced Splitter")
                .on_hover_text("Inject unpolarized acoustic wave splitting evenly 50:50")
                .clicked()
            {
                self.valley_polarization = 0.0;
                self.splitting_bias = 0.0;
                self.has_corner_bend = false;
                self.recompute();
            }

            if ui
                .button("Sharp 60-Deg Corner Bend")
                .on_hover_text("Insert sharp 60-degree corner testing backscattering immunity")
                .clicked()
            {
                self.has_corner_bend = true;
                self.recompute();
            }

            ui.separator();

            if ui.button("Reset Defaults").clicked() {
                *self = Self::default();
                self.is_open = true;
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;

            ui.label("Valley Polarization:");
            if ui
                .add(
                    egui::Slider::new(&mut self.valley_polarization, -1.0..=1.0)
                        .step_by(0.05)
                        .text("P_v (K' <-> K)"),
                )
                .changed()
            {
                changed = true;
            }

            ui.label("Asymmetry Delta:");
            if ui
                .add(
                    egui::Slider::new(&mut self.asymmetry_delta, 0.05..=0.40)
                        .step_by(0.01)
                        .text("Delta"),
                )
                .changed()
            {
                changed = true;
            }

            ui.label("Splitting Bias:");
            if ui
                .add(
                    egui::Slider::new(&mut self.splitting_bias, -1.0..=1.0)
                        .step_by(0.05)
                        .text("Bias"),
                )
                .changed()
            {
                changed = true;
            }

            if ui
                .checkbox(&mut self.has_corner_bend, "Sharp Corner Bend")
                .changed()
            {
                changed = true;
            }

            if changed {
                self.recompute();
            }
        });
    }

    /// Render 2D Junction Field & Channel Routing Canvas.
    fn render_junction_canvas(&self, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(
            vec2(ui.available_width().max(300.0), 280.0),
            Sense::hover(),
        );

        let rect = response.rect;
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42)); // Slate dark background

        let nx = self.mux_solver.params.nx;
        let ny = self.mux_solver.params.ny;

        if nx < 2 || ny < 2 {
            return;
        }

        let cell_w = rect.width() / nx as f32;
        let cell_h = rect.height() / ny as f32;

        // Render acoustic pressure field heatmap
        for iy in 0..ny {
            for ix in 0..nx {
                let p = self.mux_solver.pressure_field[iy][ix];
                let color = if p >= 0.0 {
                    let alpha = (p.clamp(0.0, 1.0) * 230.0) as u8;
                    Color32::from_rgba_unmultiplied(
                        COLOR_PRESSURE_POS.r(),
                        COLOR_PRESSURE_POS.g(),
                        COLOR_PRESSURE_POS.b(),
                        alpha,
                    )
                } else {
                    let alpha = ((-p).clamp(0.0, 1.0) * 230.0) as u8;
                    Color32::from_rgba_unmultiplied(
                        COLOR_PRESSURE_NEG.r(),
                        COLOR_PRESSURE_NEG.g(),
                        COLOR_PRESSURE_NEG.b(),
                        alpha,
                    )
                };

                let px = rect.min.x + ix as f32 * cell_w;
                let py = rect.min.y + iy as f32 * cell_h;
                let cell_rect = Rect::from_min_size(
                    pos2(px, py),
                    vec2(cell_w + 0.5, cell_h + 0.5),
                );
                painter.rect_filled(cell_rect, 0.0, color);
            }
        }

        // Draw channel boundary guides:
        // Input: line from left edge to center
        let p_in_start = pos2(rect.min.x, rect.center().y);
        let p_center = rect.center();
        painter.line_segment([p_in_start, p_center], Stroke::new(1.8, COLOR_CHANNEL_WALL));

        // Upper branch toward Port 2: angled +30 deg from center
        let tan30 = (30.0_f64 * PI / 180.0).tan() as f32;
        let p_port2 = pos2(rect.max.x, rect.center().y - (rect.width() * 0.5) * tan30);
        painter.line_segment([p_center, p_port2], Stroke::new(2.0, COLOR_PORT2_TRANS));

        // Lower branch toward Port 3: angled -30 deg from center
        let p_port3 = pos2(rect.max.x, rect.center().y + (rect.width() * 0.5) * tan30);
        painter.line_segment([p_center, p_port3], Stroke::new(2.0, COLOR_PORT3_TRANS));

        // Port labels
        painter.text(
            pos2(rect.min.x + 10.0, rect.center().y - 12.0),
            egui::Align2::LEFT_BOTTOM,
            "Port 1 (Input)",
            FontId::proportional(12.0),
            Color32::WHITE,
        );
        painter.text(
            pos2(rect.max.x - 10.0, p_port2.y - 10.0),
            egui::Align2::RIGHT_BOTTOM,
            "Port 2 (K Valley)",
            FontId::proportional(12.0),
            COLOR_PORT2_TRANS,
        );
        painter.text(
            pos2(rect.max.x - 10.0, p_port3.y + 10.0),
            egui::Align2::RIGHT_TOP,
            "Port 3 (K' Valley)",
            FontId::proportional(12.0),
            COLOR_PORT3_TRANS,
        );

        // Sharp corner indicator
        if self.mux_solver.params.has_corner_bend {
            painter.circle_stroke(p_center, 8.0, Stroke::new(1.8, Color32::from_rgb(249, 115, 22)));
            painter.text(
                pos2(p_center.x + 12.0, p_center.y - 12.0),
                egui::Align2::LEFT_BOTTOM,
                "60-deg Corner",
                FontId::proportional(11.0),
                Color32::from_rgb(249, 115, 22),
            );
        }
    }

    /// Render Berry curvature peaks around K and K' valleys.
    fn render_berry_curvature_plot(&self, ui: &mut Ui) {
        let num_pts = 60;
        let mut k_pts = Vec::with_capacity(num_pts);
        let mut kp_pts = Vec::with_capacity(num_pts);

        let k_span = 40.0; // rad/m around valley center
        for i in 0..num_pts {
            let dk = -k_span + (i as f64 / (num_pts - 1) as f64) * 2.0 * k_span;
            let omega_k = self.lattice_solver.berry_curvature_at([dk, 0.0], ValleyIndex::K);
            let omega_kp = self.lattice_solver.berry_curvature_at([dk, 0.0], ValleyIndex::KPrime);

            k_pts.push([dk, omega_k * 1e4]); // scale for readability
            kp_pts.push([dk, omega_kp * 1e4]);
        }

        let line_k = Line::new("K Valley Berry Curvature (Omega_z > 0)", PlotPoints::new(k_pts))
            .color(COLOR_VALLEY_K)
            .width(2.5);

        let line_kp = Line::new("K' Valley Berry Curvature (Omega_z < 0)", PlotPoints::new(kp_pts))
            .color(COLOR_VALLEY_KP)
            .width(2.5);

        Plot::new("berry_curvature_plot")
            .legend(Legend::default())
            .height(280.0)
            .x_axis_label("Delta k (rad/m)")
            .y_axis_label("Berry Curvature (x10^4 m^2)")
            .show(ui, |plot_ui| {
                plot_ui.hline(HLine::new("Zero Curvature", 0.0).color(Color32::from_gray(80)));
                plot_ui.line(line_k);
                plot_ui.line(line_kp);
            });
    }

    /// Render gapless kink edge dispersion crossing the valley bandgap.
    fn render_kink_dispersion_plot(&self, ui: &mut Ui) {
        let num_pts = 60;
        let mut bulk_upper = Vec::with_capacity(num_pts);
        let mut bulk_lower = Vec::with_capacity(num_pts);
        let mut kink_k = Vec::with_capacity(num_pts);
        let mut kink_kp = Vec::with_capacity(num_pts);

        let k_span = 30.0;
        for i in 0..num_pts {
            let dk = -k_span + (i as f64 / (num_pts - 1) as f64) * 2.0 * k_span;
            let (f_lower, f_upper) = self.lattice_solver.dispersion_at([dk, 0.0], ValleyIndex::K);
            let f_kink_k = self.lattice_solver.kink_edge_dispersion(dk, ValleyIndex::K);
            let f_kink_kp = self.lattice_solver.kink_edge_dispersion(dk, ValleyIndex::KPrime);

            bulk_upper.push([dk, f_upper / 1000.0]);
            bulk_lower.push([dk, f_lower / 1000.0]);
            kink_k.push([dk, f_kink_k / 1000.0]);
            kink_kp.push([dk, f_kink_kp / 1000.0]);
        }

        let line_upper = Line::new("Bulk Conduction Band Edge", PlotPoints::new(bulk_upper))
            .color(Color32::from_gray(140))
            .width(1.8);
        let line_lower = Line::new("Bulk Valence Band Edge", PlotPoints::new(bulk_lower))
            .color(Color32::from_gray(140))
            .width(1.8);
        let line_k = Line::new("Kink Edge Mode (K Valley, vg > 0)", PlotPoints::new(kink_k))
            .color(COLOR_VALLEY_K)
            .width(2.5);
        let line_kp = Line::new("Kink Edge Mode (K' Valley, vg < 0)", PlotPoints::new(kink_kp))
            .color(COLOR_VALLEY_KP)
            .width(2.5);

        Plot::new("kink_dispersion_plot")
            .legend(Legend::default())
            .height(280.0)
            .x_axis_label("Wavevector k_parallel (rad/m)")
            .y_axis_label("Frequency (kHz)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_upper);
                plot_ui.line(line_lower);
                plot_ui.line(line_k);
                plot_ui.line(line_kp);
            });
    }

    /// Render S-parameter transmission and reflection spectrum.
    fn render_s_parameters_plot(&self, ui: &mut Ui) {
        let f0 = self.operating_frequency_khz;
        let s21_val = self.mux_solver.s_parameters.s21_db;
        let s31_val = self.mux_solver.s_parameters.s31_db;
        let s11_val = self.mux_solver.s_parameters.s11_db;

        let num_pts = 50;
        let mut s21_pts = Vec::with_capacity(num_pts);
        let mut s31_pts = Vec::with_capacity(num_pts);
        let mut s11_pts = Vec::with_capacity(num_pts);

        for i in 0..num_pts {
            let f = f0 - 0.8 + (i as f64 / (num_pts - 1) as f64) * 1.6;
            let df = ((f - f0) / 0.5).abs();
            let s21_f = s21_val - df * df * 1.5;
            let s31_f = s31_val - df * df * 1.5;
            let s11_f = (s11_val + df * 6.0).min(-10.0);

            s21_pts.push([f, s21_f]);
            s31_pts.push([f, s31_f]);
            s11_pts.push([f, s11_f]);
        }

        let line_s21 = Line::new("Port 2 Transmission S21 (dB)", PlotPoints::new(s21_pts))
            .color(COLOR_PORT2_TRANS)
            .width(2.5);

        let line_s31 = Line::new("Port 3 Transmission S31 (dB)", PlotPoints::new(s31_pts))
            .color(COLOR_PORT3_TRANS)
            .width(2.5);

        let line_s11 = Line::new("Return Loss S11 (dB)", PlotPoints::new(s11_pts))
            .color(COLOR_REFLECTION)
            .width(2.0);

        Plot::new("mux_s_params_plot")
            .legend(Legend::default())
            .height(280.0)
            .x_axis_label("Frequency (kHz)")
            .y_axis_label("Scattering Parameter (dB)")
            .show(ui, |plot_ui| {
                plot_ui.hline(HLine::new("0 dB Target", 0.0).color(Color32::from_gray(80)));
                plot_ui.line(line_s21);
                plot_ui.line(line_s31);
                plot_ui.line(line_s11);
            });
    }

    /// Render telemetry footer.
    fn render_telemetry(&self, ui: &mut Ui) {
        let cv = self.berry_metrics.valley_chern_number;
        let gap_hz = self.lattice_solver.params.valley_bandgap_hz();
        let s21_db = self.mux_solver.s_parameters.s21_db;
        let s31_db = self.mux_solver.s_parameters.s31_db;
        let contrast_pct = self.mux_solver.s_parameters.valley_contrast_ratio * 100.0;
        let corner_pct = self.mux_solver.s_parameters.corner_immunity_ratio * 100.0;

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Valley Chern Cv:").strong());
            ui.colored_label(
                COLOR_VALLEY_K,
                RichText::new(format!("{:.1}", cv)).strong(),
            );

            ui.separator();

            ui.label(RichText::new("Valley Gap:").strong());
            ui.colored_label(
                Color32::from_rgb(168, 85, 247),
                RichText::new(format!("{:.1} Hz", gap_hz)).strong(),
            );

            ui.separator();

            ui.label(RichText::new("Port 2 (S21):").strong());
            ui.colored_label(
                COLOR_PORT2_TRANS,
                RichText::new(format!("{:.2} dB", s21_db)).strong(),
            );

            ui.separator();

            ui.label(RichText::new("Port 3 (S31):").strong());
            ui.colored_label(
                COLOR_PORT3_TRANS,
                RichText::new(format!("{:.2} dB", s31_db)).strong(),
            );

            ui.separator();

            ui.label(RichText::new("Valley Contrast:").strong());
            ui.colored_label(
                Color32::from_rgb(34, 197, 94),
                RichText::new(format!("{:.1}%", contrast_pct)).strong(),
            );

            ui.separator();

            ui.label(RichText::new("Corner Immunity:").strong());
            ui.colored_label(
                Color32::from_rgb(34, 197, 94),
                RichText::new(format!("{:.1}%", corner_pct)).strong(),
            );
        });
    }
}
