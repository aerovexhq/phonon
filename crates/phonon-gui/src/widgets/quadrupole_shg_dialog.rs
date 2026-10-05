#![deny(unsafe_code)]

//! Interactive Topological Acoustic Quadrupole Second-Harmonic Generation (SHG) Metamaterial Dialog.
//!
//! Provides:
//! - 2D Lattice Spatial Intensity Canvas: dual-harmonic field concentration in zero-dimensional corner nanocavities.
//! - Dual-Harmonic Discrete Spectrum: stem plot showing fundamental (f1) and frequency-doubled (2*f1) emission lines in dBm.
//! - Pump Power Conversion Efficiency Curve: non-linear power scaling and depletion saturation plateau.
//! - Corner Mode Spatial Decay Cross-Section: exponential decay envelope along the lattice diagonal.
//! - Disorder Robustness Curve: topological bulk gap protection preserving >90% non-linear conversion under hopping disorder.
//! - Telemetry Footer: Bulk Quadrupole Moment q_xy, Fundamental Confinement %, SHG Confinement %, Conversion Efficiency %,
//!   SHG Power, SHG Frequency, and Disorder Retention Ratio.

use egui::{pos2, vec2, Color32, FontId, RichText, Sense, Stroke, Ui};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::quadrupole_shg::{
    QuadrupoleShgParams, ShgEmissionEngine, ShgEmissionParams,
};

/// Color palette for the quadrupole SHG metamaterial visualizer.
const COLOR_LATTICE_BG: Color32 = Color32::from_rgb(15, 23, 42); // Dark slate
const COLOR_FUNDAMENTAL: Color32 = Color32::from_rgb(56, 189, 248); // Sky blue
const COLOR_SHG_GLOW: Color32 = Color32::from_rgb(244, 63, 94); // Rose red / SHG
const COLOR_EFFICIENCY: Color32 = Color32::from_rgb(168, 85, 247); // Purple
const COLOR_DISORDER_TOPO: Color32 = Color32::from_rgb(34, 197, 94); // Emerald green
const COLOR_DISORDER_TRIVIAL: Color32 = Color32::from_rgb(234, 179, 8); // Amber yellow

/// Active visualization tab in the Quadrupole SHG dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadrupoleShgDialogTab {
    /// 2D Lattice Spatial Intensity Canvas.
    LatticeSpatialIntensity,
    /// Dual-Harmonic Spectrum Stem Plot.
    DualHarmonicSpectrum,
    /// Pump Power Conversion Efficiency Curve.
    EfficiencySaturationCurve,
    /// Corner Mode Spatial Decay Cross-Section.
    CornerDecayProfile,
    /// Topological Disorder Robustness Comparison.
    DisorderRobustness,
}

/// Interactive modal dialog for Topological Acoustic Quadrupole SHG Metamaterials.
#[derive(Debug, Clone)]
pub struct QuadrupoleShgDialog {
    /// Window open state.
    pub is_open: bool,
    /// Current selected view tab.
    pub active_tab: QuadrupoleShgDialogTab,

    /// Emission solver engine.
    pub engine: ShgEmissionEngine,

    // Controls buffers
    pub intracell_gamma_khz: f64,
    pub intercell_lambda_khz: f64,
    pub pump_power_w: f64,
    pub non_linear_chi2: f64,
    pub disorder_w: f64,
    pub fundamental_freq_hz: f64,
    pub grid_cells: usize,
}

impl Default for QuadrupoleShgDialog {
    fn default() -> Self {
        let params = ShgEmissionParams {
            lattice: QuadrupoleShgParams {
                intracell_gamma_khz: 2.0,
                intercell_lambda_khz: 10.0,
                fundamental_freq_hz: 2500.0,
                non_linear_chi2: 0.08,
                quality_factor_q1: 1200.0,
                quality_factor_q2: 1800.0,
                nx: 4,
                ny: 4,
                disorder_w: 0.0,
            },
            pump_power_w: 0.5,
        };

        let engine = ShgEmissionEngine::new(params);

        Self {
            is_open: false,
            active_tab: QuadrupoleShgDialogTab::LatticeSpatialIntensity,
            engine,
            intracell_gamma_khz: 2.0,
            intercell_lambda_khz: 10.0,
            pump_power_w: 0.5,
            non_linear_chi2: 0.08,
            disorder_w: 0.0,
            fundamental_freq_hz: 2500.0,
            grid_cells: 4,
        }
    }
}

impl QuadrupoleShgDialog {
    /// Construct a new default dialog.
    pub fn new() -> Self {
        Self::default()
    }

    /// UI update pass for the dialog.
    pub fn ui(&mut self, ctx: &egui::Context) {
        self.show(ctx);
    }

    /// Recompute all solver fields and metrics from current buffer state.
    pub fn recompute(&mut self) {
        self.engine.params.lattice.intracell_gamma_khz = self.intracell_gamma_khz;
        self.engine.params.lattice.intercell_lambda_khz = self.intercell_lambda_khz;
        self.engine.params.lattice.fundamental_freq_hz = self.fundamental_freq_hz;
        self.engine.params.lattice.non_linear_chi2 = self.non_linear_chi2;
        self.engine.params.lattice.disorder_w = self.disorder_w;
        self.engine.params.lattice.nx = self.grid_cells;
        self.engine.params.lattice.ny = self.grid_cells;
        self.engine.params.pump_power_w = self.pump_power_w;

        self.engine.recompute();
    }

    /// Render the dialog window if open.
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Topological Quadrupole Acoustic SHG Studio")
            .open(&mut is_open)
            .default_width(860.0)
            .default_height(650.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Render the complete inner content of the dialog.
    pub fn render_content(&mut self, ui: &mut Ui) {
        self.render_controls(ui);
        ui.separator();

        // View Tabs
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.active_tab,
                QuadrupoleShgDialogTab::LatticeSpatialIntensity,
                "2D Lattice Spatial Intensity",
            );
            ui.selectable_value(
                &mut self.active_tab,
                QuadrupoleShgDialogTab::DualHarmonicSpectrum,
                "Dual-Harmonic Spectrum",
            );
            ui.selectable_value(
                &mut self.active_tab,
                QuadrupoleShgDialogTab::EfficiencySaturationCurve,
                "Efficiency vs Pump Power",
            );
            ui.selectable_value(
                &mut self.active_tab,
                QuadrupoleShgDialogTab::CornerDecayProfile,
                "Corner Spatial Decay Profile",
            );
            ui.selectable_value(
                &mut self.active_tab,
                QuadrupoleShgDialogTab::DisorderRobustness,
                "Disorder Immunity Comparison",
            );
        });

        ui.separator();

        match self.active_tab {
            QuadrupoleShgDialogTab::LatticeSpatialIntensity => {
                self.render_lattice_canvas(ui);
            }
            QuadrupoleShgDialogTab::DualHarmonicSpectrum => {
                self.render_harmonic_spectrum(ui);
            }
            QuadrupoleShgDialogTab::EfficiencySaturationCurve => {
                self.render_efficiency_curve(ui);
            }
            QuadrupoleShgDialogTab::CornerDecayProfile => {
                self.render_decay_profile(ui);
            }
            QuadrupoleShgDialogTab::DisorderRobustness => {
                self.render_disorder_robustness(ui);
            }
        }

        ui.separator();
        self.render_telemetry(ui);
    }

    /// Render interactive presets and sliders.
    fn render_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Presets:").strong());

            if ui
                .button("Optimal Topological SOTI (gamma/lambda = 0.2)")
                .on_hover_text("Second-order topological phase with maximal corner confinement and SHG efficiency")
                .clicked()
            {
                self.intracell_gamma_khz = 2.0;
                self.intercell_lambda_khz = 10.0;
                self.pump_power_w = 0.5;
                self.disorder_w = 0.0;
                self.recompute();
            }

            if ui
                .button("Trivial Phase Limit (gamma/lambda = 1.2)")
                .on_hover_text("Trivial insulating phase with zero corner states and quenched SHG conversion")
                .clicked()
            {
                self.intracell_gamma_khz = 12.0;
                self.intercell_lambda_khz = 10.0;
                self.recompute();
            }

            if ui
                .button("High-Power Saturated Emission (1.5 W)")
                .on_hover_text("High-pump regime displaying non-linear conversion depletion saturation")
                .clicked()
            {
                self.intracell_gamma_khz = 2.0;
                self.intercell_lambda_khz = 10.0;
                self.pump_power_w = 1.5;
                self.recompute();
            }

            if ui
                .button("Disorder Immunity Test (W = 0.2 lambda)")
                .on_hover_text("Introduce 20% hopping disorder to demonstrate topological defect immunity")
                .clicked()
            {
                self.intracell_gamma_khz = 2.0;
                self.intercell_lambda_khz = 10.0;
                self.disorder_w = 0.20;
                self.recompute();
            }
        });

        ui.add_space(4.0);

        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            ui.label("Intracell gamma (kHz):");
            if ui
                .add(egui::Slider::new(&mut self.intracell_gamma_khz, 0.5..=15.0).step_by(0.1))
                .changed()
            {
                changed = true;
            }

            ui.label("Intercell lambda (kHz):");
            if ui
                .add(egui::Slider::new(&mut self.intercell_lambda_khz, 5.0..=20.0).step_by(0.1))
                .changed()
            {
                changed = true;
            }

            ui.label("Pump Power (W):");
            if ui
                .add(egui::Slider::new(&mut self.pump_power_w, 0.05..=2.5).step_by(0.05))
                .changed()
            {
                changed = true;
            }
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("Disorder W (lambda):");
            if ui
                .add(egui::Slider::new(&mut self.disorder_w, 0.0..=0.40).step_by(0.01))
                .changed()
            {
                changed = true;
            }

            ui.label("Non-linear chi^(2):");
            if ui
                .add(egui::Slider::new(&mut self.non_linear_chi2, 0.01..=0.25).step_by(0.01))
                .changed()
            {
                changed = true;
            }

            ui.label("Fundamental f1 (Hz):");
            if ui
                .add(egui::Slider::new(&mut self.fundamental_freq_hz, 1000.0..=5000.0).step_by(100.0))
                .changed()
            {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }
    }

    /// Tab 1: 2D Spatial Lattice Canvas showing fundamental vs SHG corner localization.
    fn render_lattice_canvas(&self, ui: &mut Ui) {
        let desired_size = vec2(ui.available_width().max(400.0), 340.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, COLOR_LATTICE_BG);

        let grid = &self.engine.lattice_solver.shg_intensity_grid;
        if grid.is_empty() || grid[0].is_empty() {
            return;
        }

        let sy = grid.len();
        let sx = grid[0].len();

        let margin = 35.0;
        let canvas_w = rect.width() - 2.0 * margin;
        let canvas_h = rect.height() - 2.0 * margin;

        let step_x = canvas_w / (sx.max(2) - 1) as f32;
        let step_y = canvas_h / (sy.max(2) - 1) as f32;

        // Draw hopping couplings between sites
        for iy in 0..sy {
            for ix in 0..sx {
                let px = rect.min.x + margin + ix as f32 * step_x;
                let py = rect.min.y + margin + iy as f32 * step_y;

                // Horizontal hopping link
                if ix + 1 < sx {
                    let next_px = px + step_x;
                    let is_intercell = ix % 2 == 1;
                    let stroke_col = if is_intercell {
                        Color32::from_rgb(79, 70, 229) // Intercell lambda (indigo)
                    } else {
                        Color32::from_rgb(100, 116, 139) // Intracell gamma (slate)
                    };
                    painter.line_segment(
                        [pos2(px, py), pos2(next_px, py)],
                        Stroke::new(1.2, stroke_col),
                    );
                }

                // Vertical hopping link
                if iy + 1 < sy {
                    let next_py = py + step_y;
                    let is_intercell = iy % 2 == 1;
                    let stroke_col = if is_intercell {
                        Color32::from_rgb(79, 70, 229)
                    } else {
                        Color32::from_rgb(100, 116, 139)
                    };
                    painter.line_segment(
                        [pos2(px, py), pos2(px, next_py)],
                        Stroke::new(1.2, stroke_col),
                    );
                }
            }
        }

        // Draw acoustic sites with dual-color fundamental and SHG intensity
        for iy in 0..sy {
            for ix in 0..sx {
                let px = rect.min.x + margin + ix as f32 * step_x;
                let py = rect.min.y + margin + iy as f32 * step_y;

                let p1 = self.engine.lattice_solver.fundamental_intensity_grid[iy][ix];
                let p2 = self.engine.lattice_solver.shg_intensity_grid[iy][ix];

                // Outer halo for fundamental mode
                let r1 = (3.0 + p1 * 32.0).clamp(3.0, 16.0) as f32;
                let alpha1 = (p1 * 255.0 * 2.0).clamp(30.0, 220.0) as u8;
                let col1 = Color32::from_rgba_premultiplied(56, 189, 248, alpha1);
                painter.circle_filled(pos2(px, py), r1, col1);

                // Inner core for second-harmonic generation (SHG)
                let r2 = (2.0 + p2 * 24.0).clamp(2.0, 10.0) as f32;
                let alpha2 = (p2 * 255.0 * 2.5).clamp(40.0, 255.0) as u8;
                let col2 = Color32::from_rgba_premultiplied(244, 63, 94, alpha2);
                painter.circle_filled(pos2(px, py), r2, col2);

                // Site boundary ring
                painter.circle_stroke(
                    pos2(px, py),
                    r1,
                    Stroke::new(0.8, Color32::from_rgb(148, 163, 184)),
                );
            }
        }

        // Legend overlay
        let leg_x = rect.min.x + 12.0;
        let leg_y = rect.min.y + 12.0;
        painter.circle_filled(pos2(leg_x + 6.0, leg_y + 6.0), 5.0, COLOR_FUNDAMENTAL);
        painter.text(
            pos2(leg_x + 16.0, leg_y),
            egui::Align2::LEFT_TOP,
            "Fundamental Mode (omega)",
            FontId::proportional(11.0),
            Color32::from_rgb(226, 232, 240),
        );

        painter.circle_filled(pos2(leg_x + 6.0, leg_y + 22.0), 5.0, COLOR_SHG_GLOW);
        painter.text(
            pos2(leg_x + 16.0, leg_y + 16.0),
            egui::Align2::LEFT_TOP,
            "Generated Second-Harmonic (2*omega)",
            FontId::proportional(11.0),
            Color32::from_rgb(226, 232, 240),
        );
    }

    /// Tab 2: Dual-Harmonic Spectrum Stem Plot.
    fn render_harmonic_spectrum(&self, ui: &mut Ui) {
        let f1 = self.engine.lattice_solver.params.fundamental_freq_hz;
        let f2 = 2.0 * f1;

        let p1_dbm = if !self.engine.metrics.spectrum_lines.is_empty() {
            self.engine.metrics.spectrum_lines[0].power_dbm
        } else {
            20.0
        };

        let p2_dbm = if self.engine.metrics.spectrum_lines.len() > 1 {
            self.engine.metrics.spectrum_lines[1].power_dbm
        } else {
            10.0
        };

        let plot = Plot::new("harmonic_spectrum_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Frequency (kHz)")
            .y_axis_label("Power (dBm)");

        let stem_fund = PlotPoints::new(vec![[f1 / 1000.0, -30.0], [f1 / 1000.0, p1_dbm]]);
        let stem_shg = PlotPoints::new(vec![[f2 / 1000.0, -30.0], [f2 / 1000.0, p2_dbm]]);

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Fundamental Harmonic f1", stem_fund)
                    .color(COLOR_FUNDAMENTAL)
                    .width(4.5),
            );
            plot_ui.line(
                Line::new("Second Harmonic 2*f1", stem_shg)
                    .color(COLOR_SHG_GLOW)
                    .width(4.5),
            );

            // Reference baseline
            plot_ui.hline(HLine::new("Noise Floor (-30 dBm)", -30.0).color(Color32::from_rgb(100, 116, 139)));
        });
    }

    /// Tab 3: Pump Power Conversion Efficiency Curve.
    fn render_efficiency_curve(&self, ui: &mut Ui) {
        let curve_data = self.engine.efficiency_curve(50, 0.05, 2.5);
        let plot_points = PlotPoints::new(curve_data.iter().map(|&(p, eff)| [p, eff]).collect());

        let current_p = self.engine.params.pump_power_w;
        let current_eff = self.engine.metrics.conversion_efficiency_percent;
        let op_point = PlotPoints::new(vec![[current_p, current_eff]]);

        let plot = Plot::new("efficiency_curve_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Pump Acoustic Power (W)")
            .y_axis_label("Conversion Efficiency eta_SHG (%)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("SHG Conversion Efficiency", plot_points)
                    .color(COLOR_EFFICIENCY)
                    .width(2.5),
            );
            plot_ui.line(
                Line::new("Operating Point", op_point)
                    .color(COLOR_SHG_GLOW)
                    .width(6.0),
            );
            plot_ui.vline(
                VLine::new("Current Pump Power", current_p)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
        });
    }

    /// Tab 4: Corner Mode Spatial Decay Cross-Section.
    fn render_decay_profile(&self, ui: &mut Ui) {
        let xi = self.engine.lattice_solver.params.corner_decay_length();
        let num_pts = 60;
        let max_dist = 4.0; // Distance in unit cells

        let mut pts1 = Vec::with_capacity(num_pts);
        let mut pts2 = Vec::with_capacity(num_pts);

        for i in 0..num_pts {
            let d = max_dist * (i as f64) / (num_pts - 1) as f64;
            let val1 = (-2.0 * d / xi).exp();
            let val2 = (-2.0 * d / (xi * 0.75)).exp();
            pts1.push([d, val1]);
            pts2.push([d, val2]);
        }

        let plot = Plot::new("decay_profile_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Distance from Corner (Unit Cells)")
            .y_axis_label("Normalized Energy Density");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Fundamental Mode Envelope", PlotPoints::new(pts1))
                    .color(COLOR_FUNDAMENTAL)
                    .width(2.5),
            );
            plot_ui.line(
                Line::new("SHG Mode Envelope (Sub-Wavelength)", PlotPoints::new(pts2))
                    .color(COLOR_SHG_GLOW)
                    .width(2.5),
            );
            plot_ui.vline(
                VLine::new("Localization Length xi", xi)
                    .color(Color32::from_rgb(245, 158, 11)),
            );
        });
    }

    /// Tab 5: Topological Disorder Robustness Comparison.
    fn render_disorder_robustness(&self, ui: &mut Ui) {
        let num_pts = 40;
        let mut topo_pts = Vec::with_capacity(num_pts);
        let mut trivial_pts = Vec::with_capacity(num_pts);

        for i in 0..num_pts {
            let w = 0.40 * (i as f64) / (num_pts - 1) as f64;
            let r_topo = (1.0 - 0.25 * (w / 0.4).powi(2)).clamp(0.75, 1.0) * 100.0;
            let r_trivial = (1.0 - 0.85 * (w / 0.4)).clamp(0.1, 1.0) * 100.0;
            topo_pts.push([w, r_topo]);
            trivial_pts.push([w, r_trivial]);
        }

        let current_w = self.disorder_w;
        let current_retention = self.engine.metrics.disorder_immunity_ratio * 100.0;
        let op_point = PlotPoints::new(vec![[current_w, current_retention]]);

        let plot = Plot::new("disorder_robustness_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Coupling Disorder Amplitude W (lambda)")
            .y_axis_label("Efficiency Retention Ratio (%)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Topological SOTI Phase", PlotPoints::new(topo_pts))
                    .color(COLOR_DISORDER_TOPO)
                    .width(2.8),
            );
            plot_ui.line(
                Line::new("Trivial Insulator Phase", PlotPoints::new(trivial_pts))
                    .color(COLOR_DISORDER_TRIVIAL)
                    .width(2.0),
            );
            plot_ui.line(
                Line::new("Current Lattice Disorder", op_point)
                    .color(COLOR_SHG_GLOW)
                    .width(6.0),
            );
            plot_ui.hline(
                HLine::new("90% Retention Immunity Threshold", 90.0)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
        });
    }

    /// Render telemetry metrics footer.
    fn render_telemetry(&self, ui: &mut Ui) {
        let is_topo = self.engine.lattice_solver.params.is_topological_soti();
        let q_xy = self.engine.lattice_solver.params.bulk_quadrupole_moment();
        let conf1 = self.engine.lattice_solver.metrics.fundamental_corner_confinement * 100.0;
        let conf2 = self.engine.lattice_solver.metrics.shg_corner_confinement * 100.0;
        let eff = self.engine.metrics.conversion_efficiency_percent;
        let p2_mw = self.engine.metrics.second_harmonic_power_w * 1000.0;
        let f2_hz = self.engine.lattice_solver.metrics.second_harmonic_freq_hz;
        let retention = self.engine.metrics.disorder_immunity_ratio * 100.0;

        ui.horizontal_wrapped(|ui| {
            let phase_text = if is_topo {
                RichText::new("Topological SOTI").color(Color32::from_rgb(52, 211, 153)).strong()
            } else {
                RichText::new("Trivial Insulator").color(Color32::from_rgb(239, 68, 68)).strong()
            };
            ui.label(format!("Phase: "));
            ui.label(phase_text);
            ui.separator();

            ui.label(format!("Quadrupole Moment q_xy: {:.2}", q_xy));
            ui.separator();

            ui.label(format!("Fund. Confinement: {:.1}%", conf1));
            ui.separator();

            ui.label(format!("SHG Confinement: {:.1}%", conf2));
            ui.separator();

            ui.label(format!("Efficiency: {:.2}%", eff));
            ui.separator();

            ui.label(format!("SHG Power: {:.1} mW", p2_mw));
            ui.separator();

            ui.label(format!("SHG Freq: {:.0} Hz", f2_hz));
            ui.separator();

            ui.label(format!("Disorder Retention: {:.1}%", retention));
        });
    }
}
