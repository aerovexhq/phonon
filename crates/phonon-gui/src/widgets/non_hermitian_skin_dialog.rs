#![deny(unsafe_code)]

//! Interactive Non-Hermitian Skin Effect Acoustic Sensor & Directional Funnel Studio Dialog.
//!
//! Provides:
//! - 1D Acoustic Chain & Skin Accumulation Canvas: interactive visual representation of the non-reciprocal
//!   acoustic resonator chain with asymmetric hopping arrows and glowing skin state localization at the funnel boundary.
//! - Complex Energy Spectrum & Point-Gap Loop Plot: native egui_plot rendering PBC closed loop enclosing E_B = 0,
//!   with OBC eigenvalues collapsed onto the real axis.
//! - Generalized Brillouin Zone (GBZ) Polar Plot: unit circle |z|=1 vs deformed GBZ circle |beta| = exp(-g).
//! - Ultrasensitive Sensor Response Curve: native egui_plot comparing EP_N non-Hermitian frequency shift
//!   vs linear Hermitian reference across analyte mass.
//! - Directional Funnel S-Parameter Spectrum: forward S21 vs backward S12 non-reciprocal transmission.
//! - Telemetry Footer: Point-Gap Winding Number W, GBZ Radius, Skin Depth, Funneling Efficiency,
//!   Non-Reciprocal Isolation, and Sensitivity Enhancement Factor.

use egui::{
    pos2, vec2, Color32, FontId, Rect, RichText, Sense, Stroke, Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::non_hermitian_skin::{
    AcousticFunnelParams, AcousticFunnelSolver, HatanoNelsonParams, NonHermitianSkinSolver,
};
use std::f64::consts::PI;

/// Palette colors for the non-Hermitian skin effect studio.
const COLOR_SITE_NORMAL: Color32 = Color32::from_rgb(71, 85, 105); // Slate
const COLOR_SKIN_GLOW: Color32 = Color32::from_rgb(239, 68, 68);    // Bright Red / Skin accumulation
const COLOR_HOPPING_RIGHT: Color32 = Color32::from_rgb(59, 130, 246); // Blue
const COLOR_HOPPING_LEFT: Color32 = Color32::from_rgb(148, 163, 184); // Muted Slate
const COLOR_PBC_LOOP: Color32 = Color32::from_rgb(168, 85, 247);    // Purple
const COLOR_OBC_EIGS: Color32 = Color32::from_rgb(52, 211, 153);    // Emerald
const COLOR_GBZ: Color32 = Color32::from_rgb(245, 158, 11);         // Amber
const COLOR_UNIT_CIRCLE: Color32 = Color32::from_rgb(100, 116, 139);// Slate
const COLOR_SENSOR_EP: Color32 = Color32::from_rgb(236, 72, 153);   // Pink / EP response
const COLOR_SENSOR_HERM: Color32 = Color32::from_rgb(156, 163, 175);// Gray / Linear ref

/// Active view tab in the Non-Hermitian Skin Effect dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonHermitianSkinDialogTab {
    /// 1D Acoustic Chain & Skin Accumulation Canvas.
    ChainAndSkinCanvas,
    /// Complex Energy Spectrum & Point-Gap Winding Loop.
    ComplexSpectrumAndWinding,
    /// Generalized Brillouin Zone (GBZ) Polar Deformation.
    GeneralizedBrillouinZone,
    /// Ultrasensitive EP_N Sensor Frequency Shift vs Analyte Mass.
    UltrasensitiveSensorCurve,
    /// Directional Funnel S-Parameter Non-Reciprocal Spectrum.
    DirectionalFunnelSParameters,
}

/// Interactive modal dialog for the Non-Hermitian Skin Effect Sensor & Funnel.
#[derive(Debug, Clone)]
pub struct NonHermitianSkinDialog {
    /// Window visibility toggle.
    pub is_open: bool,
    /// Current selected view tab.
    pub active_tab: NonHermitianSkinDialogTab,

    // Solver engines
    pub skin_solver: NonHermitianSkinSolver,
    pub funnel_solver: AcousticFunnelSolver,

    // Control slider buffers
    pub chain_length: usize,
    pub asymmetry_g: f64,
    pub analyte_mass_pg: f64,
    pub injection_site: usize,
    pub operating_frequency_khz: f64,
}

impl Default for NonHermitianSkinDialog {
    fn default() -> Self {
        let lattice_params = HatanoNelsonParams {
            chain_length: 30,
            base_hopping_t0: 800.0,
            asymmetry_g: 0.45,
            on_site_potential: 0.0,
            boundary_perturbation_eps: 1e-6,
            resonance_frequency_hz: 5000.0,
        };

        let funnel_params = AcousticFunnelParams {
            lattice: lattice_params.clone(),
            injection_site: 0,
            cavity_loss_gamma: 25.0,
            analyte_mass_pg: 1.0,
            speed_of_sound_m_s: 343.0,
        };

        let skin_solver = NonHermitianSkinSolver::new(lattice_params);
        let funnel_solver = AcousticFunnelSolver::new(funnel_params);

        Self {
            is_open: false,
            active_tab: NonHermitianSkinDialogTab::ChainAndSkinCanvas,
            skin_solver,
            funnel_solver,
            chain_length: 30,
            asymmetry_g: 0.45,
            analyte_mass_pg: 1.0,
            injection_site: 0,
            operating_frequency_khz: 5.0,
        }
    }
}

impl NonHermitianSkinDialog {
    /// Construct a new dialog with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Primary modal UI entry point.
    pub fn ui(&mut self, ctx: &egui::Context) {
        self.show(ctx);
    }

    /// Recompute all solver physics fields with current slider parameters.
    pub fn recompute(&mut self) {
        self.skin_solver.params.chain_length = self.chain_length;
        self.skin_solver.params.asymmetry_g = self.asymmetry_g;
        self.skin_solver.params.resonance_frequency_hz = self.operating_frequency_khz * 1000.0;
        self.skin_solver.params.boundary_perturbation_eps = self.analyte_mass_pg * 1e-6;
        self.skin_solver.recompute();

        self.funnel_solver.params.lattice = self.skin_solver.params.clone();
        self.funnel_solver.params.injection_site = self.injection_site;
        self.funnel_solver.params.analyte_mass_pg = self.analyte_mass_pg;
        self.funnel_solver.recompute();
    }

    /// Main rendering entry point for the modal window.
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new(
            RichText::new("Non-Hermitian Skin Effect Acoustic Sensor & Directional Funnel")
                .strong()
                .size(15.0),
        )
        .open(&mut is_open)
        .resizable(true)
        .default_width(860.0)
        .default_height(620.0)
        .show(ctx, |ui| {
            self.render_content(ui);
        });
        self.is_open = is_open;
    }

    /// Render inner dialog content.
    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.active_tab,
                NonHermitianSkinDialogTab::ChainAndSkinCanvas,
                RichText::new("1D Acoustic Chain & Skin States").strong(),
            );
            ui.selectable_value(
                &mut self.active_tab,
                NonHermitianSkinDialogTab::ComplexSpectrumAndWinding,
                RichText::new("Point-Gap Winding Loop").strong(),
            );
            ui.selectable_value(
                &mut self.active_tab,
                NonHermitianSkinDialogTab::GeneralizedBrillouinZone,
                RichText::new("Generalized BZ (GBZ)").strong(),
            );
            ui.selectable_value(
                &mut self.active_tab,
                NonHermitianSkinDialogTab::UltrasensitiveSensorCurve,
                RichText::new("EP_N Mass Sensor Response").strong(),
            );
            ui.selectable_value(
                &mut self.active_tab,
                NonHermitianSkinDialogTab::DirectionalFunnelSParameters,
                RichText::new("Directional S-Parameters").strong(),
            );
        });

        ui.separator();

        // Control Toolbar
        self.render_controls(ui);

        ui.separator();

        // Active Tab Visualizations
        match self.active_tab {
            NonHermitianSkinDialogTab::ChainAndSkinCanvas => self.render_chain_canvas(ui),
            NonHermitianSkinDialogTab::ComplexSpectrumAndWinding => {
                self.render_complex_spectrum_plot(ui)
            }
            NonHermitianSkinDialogTab::GeneralizedBrillouinZone => self.render_gbz_plot(ui),
            NonHermitianSkinDialogTab::UltrasensitiveSensorCurve => {
                self.render_sensor_curve_plot(ui)
            }
            NonHermitianSkinDialogTab::DirectionalFunnelSParameters => {
                self.render_s_parameters_plot(ui)
            }
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
                .button("Maximal Funnel (g=0.6)")
                .on_hover_text("Strong non-reciprocal asymmetry funneling acoustic wave packets rightward")
                .clicked()
            {
                self.asymmetry_g = 0.60;
                self.analyte_mass_pg = 1.0;
                self.chain_length = 30;
                self.recompute();
            }

            if ui
                .button("Reciprocal Limit (g=0)")
                .on_hover_text("Hermitian symmetric reciprocal chain with zero skin effect")
                .clicked()
            {
                self.asymmetry_g = 0.0;
                self.recompute();
            }

            if ui
                .button("Ultrasensitive (1 pg)")
                .on_hover_text("Non-Hermitian boundary sensing with 1 picogram analyte mass")
                .clicked()
            {
                self.asymmetry_g = 0.50;
                self.analyte_mass_pg = 1.0;
                self.recompute();
            }

            if ui
                .button("Sub-Picogram (0.01 pg)")
                .on_hover_text("Demonstrate extreme sensitivity for 10 femtogram analyte")
                .clicked()
            {
                self.asymmetry_g = 0.50;
                self.analyte_mass_pg = 0.01;
                self.recompute();
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;

            ui.label("Asymmetry g:");
            if ui
                .add(egui::Slider::new(&mut self.asymmetry_g, 0.0..=0.8).step_by(0.02))
                .changed()
            {
                changed = true;
            }

            ui.separator();
            ui.label("Chain Length N:");
            if ui
                .add(egui::Slider::new(&mut self.chain_length, 10..=60))
                .changed()
            {
                changed = true;
            }

            ui.separator();
            ui.label("Analyte Mass (pg):");
            if ui
                .add(egui::Slider::new(&mut self.analyte_mass_pg, 0.001..=10.0).logarithmic(true))
                .changed()
            {
                changed = true;
            }

            ui.separator();
            ui.label("Frequency (kHz):");
            if ui
                .add(egui::Slider::new(&mut self.operating_frequency_khz, 1.0..=12.0).step_by(0.1))
                .changed()
            {
                changed = true;
            }

            if changed {
                self.recompute();
            }
        });
    }

    /// Tab 1: Render 1D Acoustic Chain & Skin Accumulation Canvas.
    fn render_chain_canvas(&self, ui: &mut Ui) {
        let desired_size = vec2(ui.available_width(), 320.0);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
        let rect = response.rect;

        // Dark background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));

        let n = self.chain_length.max(2);
        let padding = 40.0;
        let chain_width = rect.width() - 2.0 * padding;
        let dx = chain_width / (n - 1) as f32;
        let y_center = rect.center().y;

        let profile = &self.skin_solver.skin_intensity_profile;
        let p_profile = &self.funnel_solver.pressure_profile;

        // Draw inter-site coupling arrows
        let t_r = self.skin_solver.params.hopping_right();
        let _t_l = self.skin_solver.params.hopping_left();
        let stroke_right = Stroke::new(
            (2.0 + (t_r / 1000.0) as f32).clamp(1.5, 5.0),
            COLOR_HOPPING_RIGHT,
        );
        let stroke_left = Stroke::new(1.0, COLOR_HOPPING_LEFT);

        for i in 0..(n - 1) {
            let x0 = rect.left() + padding + (i as f32) * dx;
            let x1 = rect.left() + padding + ((i + 1) as f32) * dx;

            // Forward hopping arrow (top arc)
            let arc_y_fwd = y_center - 14.0;
            painter.line_segment([pos2(x0 + 8.0, arc_y_fwd), pos2(x1 - 8.0, arc_y_fwd)], stroke_right);
            painter.circle_filled(pos2(x1 - 9.0, arc_y_fwd), 3.0, COLOR_HOPPING_RIGHT);

            // Backward hopping arrow (bottom arc)
            let arc_y_bwd = y_center + 14.0;
            painter.line_segment([pos2(x0 + 8.0, arc_y_bwd), pos2(x1 - 8.0, arc_y_bwd)], stroke_left);
        }

        // Draw cavity sites with glowing skin state accumulation
        for i in 0..n {
            let x = rect.left() + padding + (i as f32) * dx;
            let intensity = profile.get(i).copied().unwrap_or(0.0);
            let pressure = p_profile.get(i).copied().unwrap_or(0.0);

            // Base radius + intensity expansion
            let r_node = (7.0 + 12.0 * intensity as f32).clamp(5.0, 20.0);

            // Color interpolation: blue to glowing red
            let red_factor = (intensity * 4.0).clamp(0.0, 1.0) as f32;
            let node_color = Color32::from_rgb(
                (COLOR_SITE_NORMAL.r() as f32 * (1.0 - red_factor) + COLOR_SKIN_GLOW.r() as f32 * red_factor) as u8,
                (COLOR_SITE_NORMAL.g() as f32 * (1.0 - red_factor) + COLOR_SKIN_GLOW.g() as f32 * red_factor) as u8,
                (COLOR_SITE_NORMAL.b() as f32 * (1.0 - red_factor) + COLOR_SKIN_GLOW.b() as f32 * red_factor) as u8,
            );

            // Outer glow if intense
            if intensity > 0.08 {
                painter.circle_filled(
                    pos2(x, y_center),
                    r_node + 6.0,
                    Color32::from_rgba_unmultiplied(239, 68, 68, (intensity * 200.0).min(180.0) as u8),
                );
            }

            painter.circle_filled(pos2(x, y_center), r_node, node_color);
            painter.circle_stroke(pos2(x, y_center), r_node, Stroke::new(1.5, Color32::WHITE));

            // Pressure amplitude bar below node
            let bar_h = (pressure as f32 * 45.0).clamp(0.0, 50.0);
            let bar_rect = Rect::from_min_size(pos2(x - 3.0, y_center + 26.0), vec2(6.0, bar_h));
            painter.rect_filled(bar_rect, 1.0, COLOR_SKIN_GLOW);
        }

        // Legend / labels
        painter.text(
            pos2(rect.left() + padding, y_center - 45.0),
            egui::Align2::LEFT_CENTER,
            "Source Injection Port (Port 1)",
            FontId::proportional(12.0),
            Color32::LIGHT_GRAY,
        );
        painter.text(
            pos2(rect.right() - padding, y_center - 45.0),
            egui::Align2::RIGHT_CENTER,
            "Directional Funnel Sink (Port 2)",
            FontId::proportional(12.0),
            COLOR_SKIN_GLOW,
        );
    }

    /// Tab 2: Complex Energy Spectrum & Point-Gap Winding Loop.
    fn render_complex_spectrum_plot(&self, ui: &mut Ui) {
        let pbc_points: PlotPoints = self
            .skin_solver
            .topology
            .pbc_spectrum
            .iter()
            .map(|&(re, im)| [re, im])
            .collect();

        let obc_points: PlotPoints = self
            .skin_solver
            .topology
            .obc_eigenvalues
            .iter()
            .map(|&(re, im)| [re, im])
            .collect();

        let plot = Plot::new("complex_spectrum_plot")
            .height(320.0)
            .legend(Legend::default())
            .x_axis_label("Re(E) [Hz]")
            .y_axis_label("Im(E) [Hz]");

        plot.show(ui, |plot_ui| {
            // Reference base point E_B = 0
            plot_ui.hline(HLine::new("Base Re", 0.0).color(Color32::from_rgb(60, 70, 85)));
            plot_ui.vline(VLine::new("Base Im", 0.0).color(Color32::from_rgb(60, 70, 85)));

            // PBC complex trajectory forming point-gap winding loop
            plot_ui.line(
                Line::new("PBC Spectrum (Winding Loop)", pbc_points)
                    .color(COLOR_PBC_LOOP)
                    .width(2.5),
            );

            // OBC collapsed eigenvalues inside loop
            plot_ui.line(
                Line::new("OBC Eigenvalues (Skin Collapsed)", obc_points)
                    .color(COLOR_OBC_EIGS)
                    .width(4.0),
            );
        });
    }

    /// Tab 3: Generalized Brillouin Zone (GBZ) Polar Deformation.
    fn render_gbz_plot(&self, ui: &mut Ui) {
        let num_pts = 120;
        let mut unit_circle_pts = Vec::with_capacity(num_pts + 1);
        let mut gbz_pts = Vec::with_capacity(num_pts + 1);
        let r_gbz = self.skin_solver.topology.gbz_radius;

        for i in 0..=num_pts {
            let theta = 2.0 * PI * (i as f64) / (num_pts as f64);
            unit_circle_pts.push([theta.cos(), theta.sin()]);
            gbz_pts.push([r_gbz * theta.cos(), r_gbz * theta.sin()]);
        }

        let plot = Plot::new("gbz_plot")
            .height(320.0)
            .legend(Legend::default())
            .data_aspect(1.0)
            .x_axis_label("Re(beta)")
            .y_axis_label("Im(beta)");

        let unit_pts = PlotPoints::new(unit_circle_pts);
        let gbz_plot_pts = PlotPoints::new(gbz_pts);

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Conventional BZ (|z| = 1.0)", unit_pts)
                    .color(COLOR_UNIT_CIRCLE)
                    .width(1.5),
            );
            plot_ui.line(
                Line::new(format!("Generalized BZ (|beta| = {:.3})", r_gbz), gbz_plot_pts)
                    .color(COLOR_GBZ)
                    .width(2.5),
            );
        });
    }

    /// Tab 4: Ultrasensitive EP_N Sensor Frequency Shift vs Analyte Mass.
    fn render_sensor_curve_plot(&self, ui: &mut Ui) {
        let n = self.chain_length.max(4);
        let g = self.asymmetry_g;

        let num_samples = 80;
        let mut ep_curve = Vec::with_capacity(num_samples);
        let mut herm_curve = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            // Logarithmic mass from 1e-4 pg to 10 pg
            let log_m = -4.0 + 5.0 * (i as f64) / (num_samples as f64);
            let mass_pg = 10.0_f64.powf(log_m);
            let eps = mass_pg * 1e-6;

            let delta_f_herm = 120.0 * eps;
            let delta_f_ep = 120.0 * eps.powf(1.0 / ((n as f64) * 0.35).max(2.0)) * (g.abs() * 2.5).exp();

            ep_curve.push([mass_pg, delta_f_ep]);
            herm_curve.push([mass_pg, delta_f_herm]);
        }

        let plot = Plot::new("sensor_curve_plot")
            .height(320.0)
            .legend(Legend::default())
            .x_axis_label("Analyte Mass [pg]")
            .y_axis_label("Frequency Shift Delta f [Hz]");

        let ep_pts = PlotPoints::new(ep_curve);
        let herm_pts = PlotPoints::new(herm_curve);

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Non-Hermitian EP_N Sensor Response", ep_pts)
                    .color(COLOR_SENSOR_EP)
                    .width(3.0),
            );
            plot_ui.line(
                Line::new("Hermitian Reference Sensor (Linear)", herm_pts)
                    .color(COLOR_SENSOR_HERM)
                    .width(1.5),
            );
        });
    }

    /// Tab 5: Directional Funnel S-Parameter Non-Reciprocal Spectrum.
    fn render_s_parameters_plot(&self, ui: &mut Ui) {
        let f0 = self.operating_frequency_khz * 1000.0;
        let s21_base = self.funnel_solver.s_parameters.s21_forward_db;
        let s12_base = self.funnel_solver.s_parameters.s12_backward_db;
        let s11_base = self.funnel_solver.s_parameters.s11_return_loss_db;

        let num_pts = 100;
        let span = 1500.0; // +/- 1.5 kHz
        let mut s21_pts = Vec::with_capacity(num_pts);
        let mut s12_pts = Vec::with_capacity(num_pts);
        let mut s11_pts = Vec::with_capacity(num_pts);

        for i in 0..num_pts {
            let f = (f0 - span) + 2.0 * span * (i as f64) / (num_pts as f64);
            let delta_f = (f - f0) / 400.0;
            let roll = -10.0 * (delta_f * delta_f) / (1.0 + delta_f * delta_f);

            s21_pts.push([f / 1000.0, s21_base + roll]);
            s12_pts.push([f / 1000.0, s12_base + roll]);
            s11_pts.push([f / 1000.0, s11_base - roll]);
        }

        let plot = Plot::new("funnel_s_params_plot")
            .height(320.0)
            .legend(Legend::default())
            .x_axis_label("Frequency [kHz]")
            .y_axis_label("S-Parameters [dB]");

        let s21_plot_pts = PlotPoints::new(s21_pts);
        let s12_plot_pts = PlotPoints::new(s12_pts);
        let s11_plot_pts = PlotPoints::new(s11_pts);

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Forward Transmission S21 (Port 1 -> 2)", s21_plot_pts)
                    .color(COLOR_HOPPING_RIGHT)
                    .width(2.5),
            );
            plot_ui.line(
                Line::new("Backward Transmission S12 (Port 2 -> 1)", s12_plot_pts)
                    .color(COLOR_SKIN_GLOW)
                    .width(2.0),
            );
            plot_ui.line(
                Line::new("Return Loss S11 (Port 1)", s11_plot_pts)
                    .color(COLOR_GBZ)
                    .width(1.5),
            );
        });
    }

    /// Render telemetry status bar.
    fn render_telemetry(&self, ui: &mut Ui) {
        let top = &self.skin_solver.topology;
        let s_params = &self.funnel_solver.s_parameters;

        ui.horizontal_wrapped(|ui| {
            ui.monospace(
                RichText::new(format!("Winding W: {}", top.winding_number))
                    .color(COLOR_PBC_LOOP)
                    .strong(),
            );
            ui.separator();
            ui.monospace(format!("GBZ Radius: {:.3}", top.gbz_radius));
            ui.separator();
            ui.monospace(format!("Skin Depth: {:.1} sites", top.skin_depth_sites));
            ui.separator();
            ui.monospace(
                RichText::new(format!(
                    "Funnel Eff: {:.1}%",
                    s_params.funnel_accumulation_efficiency * 100.0
                ))
                .color(COLOR_SKIN_GLOW)
                .strong(),
            );
            ui.separator();
            ui.monospace(
                RichText::new(format!(
                    "Isolation: {:.1} dB",
                    s_params.non_reciprocal_isolation_db
                ))
                .color(COLOR_HOPPING_RIGHT)
                .strong(),
            );
            ui.separator();
            ui.monospace(
                RichText::new(format!(
                    "Sensitivity: {:.0}x",
                    s_params.sensitivity_enhancement_factor
                ))
                .color(COLOR_SENSOR_EP)
                .strong(),
            );
            ui.separator();
            ui.monospace(format!("Shift: {:.2} Hz", s_params.sensor_frequency_shift_hz));
        });
    }
}
