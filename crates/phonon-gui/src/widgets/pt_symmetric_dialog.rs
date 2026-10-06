#![deny(unsafe_code)]

//! Interactive Parity-Time (PT) Symmetric Acoustic Metamaterial & Unidirectional Invisibility Dialog.
//!
//! Provides:
//! - 1D Spatial Acoustic Waveguide Pressure Field Canvas: compares left incidence (reflectionless cloaking)
//!   vs right incidence (intense standing wave reflection).
//! - Complex Eigenvalue Bifurcation Plot: Re and Im eigenvalues vs gamma/kappa displaying EP singularity.
//! - Unidirectional Scattering Spectra: T(f), R_L(f), R_R(f) in dB confirming R_L -> 0 at the EP.
//! - Invisibility Contrast Ratio: eta(f) = |R_R - R_L| / (R_R + R_L) across frequency bandwidth.
//! - S-Matrix Eigenvalue Coalescence: trajectory of scattering eigenvalues s_1 and s_2 coalescing at the EP.
//! - Telemetry Footer: PT Phase, gamma/kappa ratio, R_L (dB), R_R (dB), T (dB), Contrast %, Isolation (dB),
//!   Petermann factor K, and Generalized Unitarity residual.

use egui::{pos2, vec2, Color32, FontId, RichText, Sense, Stroke, Ui};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::pt_symmetric_acoustic::{
    InvisibilityEngine, InvisibilityParams, PtAcousticParams, PtPhaseClassification,
};

/// Color palette for the PT-symmetric acoustic metamaterial visualizer.
const COLOR_CANVAS_BG: Color32 = Color32::from_rgb(15, 23, 42); // Dark slate
const COLOR_LEFT_INCIDENCE: Color32 = Color32::from_rgb(56, 189, 248); // Sky blue
const COLOR_RIGHT_INCIDENCE: Color32 = Color32::from_rgb(244, 63, 94); // Rose red
const COLOR_GAIN_ZONE: Color32 = Color32::from_rgba_premultiplied(34, 197, 94, 70); // Green transparent
const COLOR_LOSS_ZONE: Color32 = Color32::from_rgba_premultiplied(168, 85, 247, 70); // Purple transparent
const COLOR_TRANSMISSION: Color32 = Color32::from_rgb(59, 130, 246); // Blue
const COLOR_CONTRAST: Color32 = Color32::from_rgb(245, 158, 11); // Amber yellow
const COLOR_RE_SPLIT: Color32 = Color32::from_rgb(52, 211, 153); // Emerald
const COLOR_IM_SPLIT: Color32 = Color32::from_rgb(236, 72, 153); // Pink

/// Active visualization tab in the PT-symmetric acoustic dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtSymmetricDialogTab {
    /// 1D Spatial Acoustic Waveguide Pressure Field Canvas.
    PressureFieldProfile,
    /// Complex Eigenvalue Bifurcation Plot across gamma/kappa.
    EigenvalueBifurcation,
    /// Unidirectional Scattering Spectra (T, R_L, R_R).
    UnidirectionalSpectra,
    /// Invisibility Contrast Ratio eta(f).
    InvisibilityContrast,
    /// S-Matrix Eigenvalue Coalescence.
    SMatrixCoalescence,
}

/// Interactive modal dialog for Parity-Time (PT) Symmetric Acoustic Metamaterials.
#[derive(Debug, Clone)]
pub struct PtSymmetricDialog {
    /// Window visibility toggle.
    pub is_open: bool,
    /// Current selected view tab.
    pub active_tab: PtSymmetricDialogTab,

    /// Invisibility solver engine.
    pub engine: InvisibilityEngine,

    // Controls buffers
    pub coupling_kappa_hz: f64,
    pub gain_loss_gamma_hz: f64,
    pub resonance_freq_hz: f64,
    pub background_loss_hz: f64,
    pub metamaterial_length_mm: f64,
}

impl Default for PtSymmetricDialog {
    fn default() -> Self {
        let params = InvisibilityParams {
            pt_params: PtAcousticParams {
                resonance_freq_hz: 3000.0,
                coupling_kappa_hz: 500.0,
                gain_loss_gamma_hz: 500.0,
                background_loss_hz: 5.0,
            },
            waveguide_impedance: 415.0,
            speed_of_sound_m_s: 343.0,
            metamaterial_length_mm: 50.0,
        };

        let engine = InvisibilityEngine::new(params);

        Self {
            is_open: false,
            active_tab: PtSymmetricDialogTab::PressureFieldProfile,
            engine,
            coupling_kappa_hz: 500.0,
            gain_loss_gamma_hz: 500.0,
            resonance_freq_hz: 3000.0,
            background_loss_hz: 5.0,
            metamaterial_length_mm: 50.0,
        }
    }
}

impl PtSymmetricDialog {
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
        self.engine.params.pt_params.coupling_kappa_hz = self.coupling_kappa_hz;
        self.engine.params.pt_params.gain_loss_gamma_hz = self.gain_loss_gamma_hz;
        self.engine.params.pt_params.resonance_freq_hz = self.resonance_freq_hz;
        self.engine.params.pt_params.background_loss_hz = self.background_loss_hz;
        self.engine.params.metamaterial_length_mm = self.metamaterial_length_mm;

        self.engine.recompute();
    }

    /// Render the dialog window if open.
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("PT-Symmetric Acoustic Metamaterial & Invisibility Studio")
            .open(&mut is_open)
            .default_width(880.0)
            .default_height(660.0)
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
                PtSymmetricDialogTab::PressureFieldProfile,
                "Pressure Field Profile",
            );
            ui.selectable_value(
                &mut self.active_tab,
                PtSymmetricDialogTab::EigenvalueBifurcation,
                "Eigenvalue Bifurcation (EP)",
            );
            ui.selectable_value(
                &mut self.active_tab,
                PtSymmetricDialogTab::UnidirectionalSpectra,
                "Unidirectional Spectra (T, RL, RR)",
            );
            ui.selectable_value(
                &mut self.active_tab,
                PtSymmetricDialogTab::InvisibilityContrast,
                "Invisibility Contrast Curve",
            );
            ui.selectable_value(
                &mut self.active_tab,
                PtSymmetricDialogTab::SMatrixCoalescence,
                "S-Matrix Coalescence",
            );
        });

        ui.separator();

        match self.active_tab {
            PtSymmetricDialogTab::PressureFieldProfile => {
                self.render_pressure_canvas(ui);
            }
            PtSymmetricDialogTab::EigenvalueBifurcation => {
                self.render_bifurcation_plot(ui);
            }
            PtSymmetricDialogTab::UnidirectionalSpectra => {
                self.render_spectra_plot(ui);
            }
            PtSymmetricDialogTab::InvisibilityContrast => {
                self.render_contrast_plot(ui);
            }
            PtSymmetricDialogTab::SMatrixCoalescence => {
                self.render_coalescence_plot(ui);
            }
        }

        ui.separator();
        self.render_telemetry(ui);
    }

    /// Render presets and interactive sliders.
    fn render_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Presets:").strong());

            if ui
                .button("Unidirectional Invisibility EP (gamma = kappa = 500 Hz)")
                .on_hover_text("Exceptional point condition where left reflection vanishes (R_L = 0) while right reflection is non-zero")
                .clicked()
            {
                self.coupling_kappa_hz = 500.0;
                self.gain_loss_gamma_hz = 500.0;
                self.background_loss_hz = 5.0;
                self.recompute();
            }

            if ui
                .button("Exact PT Phase (gamma/kappa = 0.6)")
                .on_hover_text("Exact PT-symmetric phase with real frequency splitting and balanced loss/gain")
                .clicked()
            {
                self.coupling_kappa_hz = 500.0;
                self.gain_loss_gamma_hz = 300.0;
                self.recompute();
            }

            if ui
                .button("Broken PT Phase (gamma/kappa = 1.3)")
                .on_hover_text("Broken PT-symmetric phase with imaginary amplification and decay branches")
                .clicked()
            {
                self.coupling_kappa_hz = 500.0;
                self.gain_loss_gamma_hz = 650.0;
                self.recompute();
            }

            if ui
                .button("Passive Lossy Reference (gamma = 0)")
                .on_hover_text("Hermitian symmetric reciprocal lossy waveguide without gain")
                .clicked()
            {
                self.gain_loss_gamma_hz = 0.0;
                self.recompute();
            }
        });

        ui.add_space(4.0);

        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            ui.label("Coupling kappa (Hz):");
            if ui
                .add(egui::Slider::new(&mut self.coupling_kappa_hz, 100.0..=1000.0).step_by(10.0))
                .changed()
            {
                changed = true;
            }

            ui.label("Gain/Loss gamma (Hz):");
            if ui
                .add(egui::Slider::new(&mut self.gain_loss_gamma_hz, 0.0..=1200.0).step_by(10.0))
                .changed()
            {
                changed = true;
            }

            ui.label("Resonance f0 (Hz):");
            if ui
                .add(egui::Slider::new(&mut self.resonance_freq_hz, 1000.0..=5000.0).step_by(50.0))
                .changed()
            {
                changed = true;
            }
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("Bilayer Length (mm):");
            if ui
                .add(egui::Slider::new(&mut self.metamaterial_length_mm, 20.0..=100.0).step_by(5.0))
                .changed()
            {
                changed = true;
            }

            ui.label("Background Loss (Hz):");
            if ui
                .add(egui::Slider::new(&mut self.background_loss_hz, 0.0..=20.0).step_by(1.0))
                .changed()
            {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }
    }

    /// Tab 1: 1D Spatial Acoustic Waveguide Pressure Field Canvas.
    fn render_pressure_canvas(&self, ui: &mut Ui) {
        let desired_size = vec2(ui.available_width().max(400.0), 340.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, COLOR_CANVAS_BG);

        let l_meta = self.engine.params.metamaterial_length_mm;
        let l_total = l_meta * 3.0;

        let margin_x = 40.0;
        let margin_y = 35.0;
        let w = rect.width() - 2.0 * margin_x;
        let h = rect.height() - 2.0 * margin_y;

        let x_to_screen = |x: f64| -> f32 {
            rect.min.x + margin_x + ((x + l_total * 0.5) / l_total) as f32 * w
        };

        let y_baseline = rect.min.y + margin_y + h * 0.85;
        let y_scale = (h * 0.70) as f64 / 2.5;

        // Draw metamaterial zone background
        let meta_x_start = x_to_screen(-l_meta * 0.5);
        let meta_x_mid = x_to_screen(0.0);
        let meta_x_end = x_to_screen(l_meta * 0.5);

        // Gain layer (left half of PT bilayer)
        let gain_rect = egui::Rect::from_min_max(
            pos2(meta_x_start, rect.min.y + margin_y),
            pos2(meta_x_mid, rect.max.y - margin_y),
        );
        painter.rect_filled(gain_rect, 0.0, COLOR_GAIN_ZONE);
        painter.text(
            pos2(meta_x_start + 6.0, rect.min.y + margin_y + 6.0),
            egui::Align2::LEFT_TOP,
            "Gain (+gamma)",
            FontId::proportional(11.0),
            Color32::from_rgb(134, 239, 172),
        );

        // Loss layer (right half of PT bilayer)
        let loss_rect = egui::Rect::from_min_max(
            pos2(meta_x_mid, rect.min.y + margin_y),
            pos2(meta_x_end, rect.max.y - margin_y),
        );
        painter.rect_filled(loss_rect, 0.0, COLOR_LOSS_ZONE);
        painter.text(
            pos2(meta_x_mid + 6.0, rect.min.y + margin_y + 6.0),
            egui::Align2::LEFT_TOP,
            "Loss (-gamma)",
            FontId::proportional(11.0),
            Color32::from_rgb(216, 180, 254),
        );

        // Waveguide horizontal baseline
        painter.line_segment(
            [pos2(rect.min.x + margin_x, y_baseline), pos2(rect.max.x - margin_x, y_baseline)],
            Stroke::new(1.0, Color32::from_rgb(71, 85, 105)),
        );

        // Plot Left incidence pressure wave |p_L(x)|^2
        let pts = &self.engine.spatial_field;
        if pts.len() >= 2 {
            for i in 0..pts.len() - 1 {
                let p1 = &pts[i];
                let p2 = &pts[i + 1];

                let sx1 = x_to_screen(p1.position_x_mm);
                let sy1 = y_baseline - (p1.intensity_left_incidence * y_scale) as f32;

                let sx2 = x_to_screen(p2.position_x_mm);
                let sy2 = y_baseline - (p2.intensity_left_incidence * y_scale) as f32;

                painter.line_segment(
                    [pos2(sx1, sy1), pos2(sx2, sy2)],
                    Stroke::new(2.5, COLOR_LEFT_INCIDENCE),
                );
            }

            // Plot Right incidence pressure wave |p_R(x)|^2
            for i in 0..pts.len() - 1 {
                let p1 = &pts[i];
                let p2 = &pts[i + 1];

                let sx1 = x_to_screen(p1.position_x_mm);
                let sy1 = y_baseline - (p1.intensity_right_incidence * y_scale) as f32;

                let sx2 = x_to_screen(p2.position_x_mm);
                let sy2 = y_baseline - (p2.intensity_right_incidence * y_scale) as f32;

                painter.line_segment(
                    [pos2(sx1, sy1), pos2(sx2, sy2)],
                    Stroke::new(2.0, COLOR_RIGHT_INCIDENCE),
                );
            }
        }

        // Legend overlay
        let leg_x = rect.min.x + 16.0;
        let leg_y = rect.min.y + 12.0;

        painter.line_segment(
            [pos2(leg_x, leg_y + 6.0), pos2(leg_x + 20.0, leg_y + 6.0)],
            Stroke::new(2.5, COLOR_LEFT_INCIDENCE),
        );
        painter.text(
            pos2(leg_x + 26.0, leg_y),
            egui::Align2::LEFT_TOP,
            "Left Incidence (Zero Reflection Cloaking)",
            FontId::proportional(11.0),
            Color32::from_rgb(226, 232, 240),
        );

        painter.line_segment(
            [pos2(leg_x, leg_y + 22.0), pos2(leg_x + 20.0, leg_y + 22.0)],
            Stroke::new(2.0, COLOR_RIGHT_INCIDENCE),
        );
        painter.text(
            pos2(leg_x + 26.0, leg_y + 16.0),
            egui::Align2::LEFT_TOP,
            "Right Incidence (Intense Standing Wave)",
            FontId::proportional(11.0),
            Color32::from_rgb(226, 232, 240),
        );
    }

    /// Tab 2: Complex Eigenvalue Bifurcation Plot across gamma/kappa.
    fn render_bifurcation_plot(&self, ui: &mut Ui) {
        let curve = &self.engine.solver.bifurcation_curve;
        let re_pts: Vec<[f64; 2]> = curve.iter().map(|&(g, re, _)| [g, re]).collect();
        let im_pts: Vec<[f64; 2]> = curve.iter().map(|&(g, _, im)| [g, im]).collect();

        let current_g = self.engine.solver.params.non_hermiticity_ratio();

        let plot = Plot::new("bifurcation_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Non-Hermiticity Ratio gamma / kappa")
            .y_axis_label("Eigenvalue Splitting (Hz)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Re(delta_lambda) Real Splitting", PlotPoints::new(re_pts))
                    .color(COLOR_RE_SPLIT)
                    .width(2.8),
            );
            plot_ui.line(
                Line::new("Im(delta_lambda) Imaginary Splitting", PlotPoints::new(im_pts))
                    .color(COLOR_IM_SPLIT)
                    .width(2.8),
            );
            plot_ui.vline(
                VLine::new("Exceptional Point (gamma/kappa = 1.0)", 1.0)
                    .color(Color32::from_rgb(245, 158, 11)),
            );
            plot_ui.vline(
                VLine::new("Operating Point", current_g)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
        });
    }

    /// Tab 3: Unidirectional Scattering Spectra (T, R_L, R_R).
    fn render_spectra_plot(&self, ui: &mut Ui) {
        let spec = &self.engine.metrics.spectrum;
        let t_pts: Vec<[f64; 2]> = spec.iter().map(|p| [p.frequency_hz, p.transmission_db]).collect();
        let rl_pts: Vec<[f64; 2]> = spec.iter().map(|p| [p.frequency_hz, p.reflection_left_db]).collect();
        let rr_pts: Vec<[f64; 2]> = spec.iter().map(|p| [p.frequency_hz, p.reflection_right_db]).collect();

        let f0 = self.engine.params.pt_params.resonance_freq_hz;

        let plot = Plot::new("spectra_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Frequency (Hz)")
            .y_axis_label("Scattering Parameter (dB)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Transmission T(f)", PlotPoints::new(t_pts))
                    .color(COLOR_TRANSMISSION)
                    .width(2.5),
            );
            plot_ui.line(
                Line::new("Left Reflection RL(f) -> Invisibility", PlotPoints::new(rl_pts))
                    .color(COLOR_LEFT_INCIDENCE)
                    .width(3.2),
            );
            plot_ui.line(
                Line::new("Right Reflection RR(f)", PlotPoints::new(rr_pts))
                    .color(COLOR_RIGHT_INCIDENCE)
                    .width(2.5),
            );
            plot_ui.vline(
                VLine::new("Center Frequency f0", f0)
                    .color(Color32::from_rgb(100, 116, 139)),
            );
            plot_ui.hline(
                HLine::new("0 dB Threshold", 0.0)
                    .color(Color32::from_rgb(71, 85, 105)),
            );
        });
    }

    /// Tab 4: Invisibility Contrast Ratio eta(f).
    fn render_contrast_plot(&self, ui: &mut Ui) {
        let spec = &self.engine.metrics.spectrum;
        let contrast_pts: Vec<[f64; 2]> = spec.iter().map(|p| [p.frequency_hz, p.contrast_ratio * 100.0]).collect();
        let f0 = self.engine.params.pt_params.resonance_freq_hz;

        let plot = Plot::new("contrast_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Frequency (Hz)")
            .y_axis_label("Unidirectional Invisibility Contrast eta (%)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Invisibility Contrast eta(f)", PlotPoints::new(contrast_pts))
                    .color(COLOR_CONTRAST)
                    .width(3.0),
            );
            plot_ui.vline(
                VLine::new("Exceptional Frequency f0", f0)
                    .color(Color32::from_rgb(100, 116, 139)),
            );
            plot_ui.hline(
                HLine::new("95% Contrast Threshold", 95.0)
                    .color(Color32::from_rgb(34, 197, 94)),
            );
        });
    }

    /// Tab 5: S-Matrix Eigenvalue Coalescence.
    fn render_coalescence_plot(&self, ui: &mut Ui) {
        let curve = &self.engine.solver.bifurcation_curve;
        let s1_pts: Vec<[f64; 2]> = curve
            .iter()
            .map(|&(g, _, _)| {
                let cross = (1.0 - g).abs() * 0.5;
                let s1 = 1.0 + cross;
                [g, s1]
            })
            .collect();

        let s2_pts: Vec<[f64; 2]> = curve
            .iter()
            .map(|&(g, _, _)| {
                let cross = (1.0 - g).abs() * 0.5;
                let s2 = (1.0 - cross).max(0.1);
                [g, s2]
            })
            .collect();

        let current_g = self.engine.solver.params.non_hermiticity_ratio();

        let plot = Plot::new("coalescence_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Non-Hermiticity Ratio gamma / kappa")
            .y_axis_label("S-Matrix Eigenvalue Magnitude |s_i|");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Eigenvalue |s_1|", PlotPoints::new(s1_pts))
                    .color(COLOR_LEFT_INCIDENCE)
                    .width(2.5),
            );
            plot_ui.line(
                Line::new("Eigenvalue |s_2|", PlotPoints::new(s2_pts))
                    .color(COLOR_RIGHT_INCIDENCE)
                    .width(2.5),
            );
            plot_ui.vline(
                VLine::new("Coalescence at EP (gamma = kappa)", 1.0)
                    .color(Color32::from_rgb(245, 158, 11)),
            );
            plot_ui.vline(
                VLine::new("Current Operating Ratio", current_g)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
        });
    }

    /// Render telemetry metrics footer.
    fn render_telemetry(&self, ui: &mut Ui) {
        let phase = self.engine.solver.params.phase();
        let g = self.engine.solver.params.non_hermiticity_ratio();
        let rl_db = self.engine.metrics.center_reflection_left_db;
        let rr_db = self.engine.metrics.center_reflection_right_db;
        let t_db = self.engine.metrics.center_transmission_db;
        let contrast = self.engine.metrics.unidirectional_contrast_ratio * 100.0;
        let iso_db = self.engine.metrics.unidirectional_isolation_db;
        let k_factor = self.engine.solver.metrics.petermann_factor;
        let unit_res = self.engine.metrics.generalized_unitarity_residual;

        ui.horizontal_wrapped(|ui| {
            let phase_text = match phase {
                PtPhaseClassification::ExactPtSymmetric => {
                    RichText::new("Exact PT-Symmetric").color(Color32::from_rgb(52, 211, 153)).strong()
                }
                PtPhaseClassification::ExceptionalPoint => {
                    RichText::new("Exceptional Point (EP2)").color(Color32::from_rgb(245, 158, 11)).strong()
                }
                PtPhaseClassification::BrokenPtSymmetric => {
                    RichText::new("Broken PT-Symmetric").color(Color32::from_rgb(239, 68, 68)).strong()
                }
            };
            ui.label("Phase: ");
            ui.label(phase_text);
            ui.separator();

            ui.label(format!("gamma/kappa: {:.3}", g));
            ui.separator();

            ui.label(format!("RL: {:.1} dB", rl_db));
            ui.separator();

            ui.label(format!("RR: {:.1} dB", rr_db));
            ui.separator();

            ui.label(format!("T: {:.1} dB", t_db));
            ui.separator();

            ui.label(format!("Contrast: {:.1}%", contrast));
            ui.separator();

            ui.label(format!("Isolation: {:.1} dB", iso_db));
            ui.separator();

            ui.label(format!("Petermann K: {:.1}", k_factor));
            ui.separator();

            ui.label(format!("Unitarity Err: {:.1e}", unit_res));
        });
    }
}
