#![deny(unsafe_code)]

//! Interactive Topological Acoustic Bound States in the Continuum (BIC) & Vortex Cavity Dialog.
//!
//! Provides:
//! - 2D Momentum-Space Far-Field Polarization Vortex Canvas: vector map showing velocity field
//!   winding around the BIC topological defect singularity.
//! - Diverging Q-Factor Momentum Profile: plot of Q(k) showing the singularity peak at k = k_BIC.
//! - Ultra-Sharp Fano Resonance Transmission Spectrum: T(f) in dB and cavity field enhancement.
//! - Quasi-BIC Asymmetry Scaling Curve: Q vs alpha demonstrating inverse-quadratic 1 / alpha^2 scaling.
//! - Near-Field Acoustic Radiation Vortex Canvas: donut pressure intensity and azimuthal phase winding.
//! - Telemetry Footer: BIC Mode, Topological Charge q, Peak Q, Radiative Linewidth,
//!   Field Enhancement Factor, Fano Asymmetry q_F, OAM Mode Purity %, and Transmission Dip (dB).

use egui::{pos2, vec2, Color32, FontId, RichText, Sense, Stroke, Ui};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::acoustic_bic::{
    BicKind, BicLatticeParams, CavityVortexEngine, CavityVortexParams,
};

/// Color palette for the acoustic BIC visualizer.
const COLOR_CANVAS_BG: Color32 = Color32::from_rgb(15, 23, 42); // Slate dark
const COLOR_VORTEX_CORE: Color32 = Color32::from_rgb(244, 63, 94); // Rose red
const COLOR_Q_FACTOR: Color32 = Color32::from_rgb(52, 211, 153); // Emerald
const COLOR_FANO_TRANSMISSION: Color32 = Color32::from_rgb(59, 130, 246); // Blue
const COLOR_ENHANCEMENT: Color32 = Color32::from_rgb(245, 158, 11); // Amber
const COLOR_DONUT_RING: Color32 = Color32::from_rgb(168, 85, 247); // Purple

/// Active visualization tab in the acoustic BIC dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcousticBicDialogTab {
    /// 2D Far-Field Polarization Vortex Vector Map in momentum space.
    PolarizationVortexMap,
    /// Diverging Loaded Quality Factor Q(k) across momentum.
    DivergingQFactor,
    /// Ultra-Sharp Fano Resonance Transmission Spectrum.
    FanoTransmissionSpectrum,
    /// Quasi-BIC Asymmetry Scaling Curve Q(alpha).
    QuasiBicScaling,
    /// Near-Field Acoustic Radiation Vortex Donut Profile.
    NearFieldVortexProfile,
}

/// Interactive modal dialog for Topological Acoustic BICs and Vortex Cavities.
#[derive(Debug, Clone)]
pub struct AcousticBicDialog {
    /// Window visibility toggle.
    pub is_open: bool,
    /// Current active tab.
    pub active_tab: AcousticBicDialogTab,

    /// Microcavity vortex engine.
    pub engine: CavityVortexEngine,

    // Controls buffers
    pub bic_kind: BicKind,
    pub asymmetry_parameter: f64,
    pub resonance_freq_hz: f64,
    pub intrinsic_loss_hz: f64,
    pub fano_asymmetry_q: f64,
    pub cavity_diameter_mm: f64,
}

impl Default for AcousticBicDialog {
    fn default() -> Self {
        let lattice_params = BicLatticeParams {
            resonance_freq_hz: 4000.0,
            speed_of_sound_m_s: 343.0,
            lattice_pitch_mm: 40.0,
            cavity_radius_mm: 12.0,
            bic_kind: BicKind::SymmetryProtectedGamma,
            asymmetry_parameter: 0.05,
            intrinsic_loss_hz: 0.5,
            radiative_coupling_coeff_hz: 250.0,
        };

        let params = CavityVortexParams {
            lattice_params,
            fano_asymmetry_q: -2.0,
            drive_amplitude_pa: 100.0,
            frequency_span_hz: 40.0,
            cavity_diameter_mm: 24.0,
        };

        let engine = CavityVortexEngine::new(params);

        Self {
            is_open: false,
            active_tab: AcousticBicDialogTab::PolarizationVortexMap,
            engine,
            bic_kind: BicKind::SymmetryProtectedGamma,
            asymmetry_parameter: 0.05,
            resonance_freq_hz: 4000.0,
            intrinsic_loss_hz: 0.5,
            fano_asymmetry_q: -2.0,
            cavity_diameter_mm: 24.0,
        }
    }
}

impl AcousticBicDialog {
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
        self.engine.params.lattice_params.bic_kind = self.bic_kind;
        self.engine.params.lattice_params.asymmetry_parameter = self.asymmetry_parameter;
        self.engine.params.lattice_params.resonance_freq_hz = self.resonance_freq_hz;
        self.engine.params.lattice_params.intrinsic_loss_hz = self.intrinsic_loss_hz;
        self.engine.params.fano_asymmetry_q = self.fano_asymmetry_q;
        self.engine.params.cavity_diameter_mm = self.cavity_diameter_mm;

        self.engine.recompute();
    }

    /// Render the dialog window if open.
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Topological Acoustic BIC & High-Q Vortex Cavity Studio")
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
                AcousticBicDialogTab::PolarizationVortexMap,
                "Polarization Vortex Map",
            );
            ui.selectable_value(
                &mut self.active_tab,
                AcousticBicDialogTab::DivergingQFactor,
                "Diverging Q(k) Profile",
            );
            ui.selectable_value(
                &mut self.active_tab,
                AcousticBicDialogTab::FanoTransmissionSpectrum,
                "Fano Transmission Spectrum",
            );
            ui.selectable_value(
                &mut self.active_tab,
                AcousticBicDialogTab::QuasiBicScaling,
                "Quasi-BIC Scaling Q(alpha)",
            );
            ui.selectable_value(
                &mut self.active_tab,
                AcousticBicDialogTab::NearFieldVortexProfile,
                "Near-Field Vortex Beam",
            );
        });

        ui.separator();

        match self.active_tab {
            AcousticBicDialogTab::PolarizationVortexMap => {
                self.render_vortex_canvas(ui);
            }
            AcousticBicDialogTab::DivergingQFactor => {
                self.render_q_factor_plot(ui);
            }
            AcousticBicDialogTab::FanoTransmissionSpectrum => {
                self.render_fano_spectrum_plot(ui);
            }
            AcousticBicDialogTab::QuasiBicScaling => {
                self.render_scaling_plot(ui);
            }
            AcousticBicDialogTab::NearFieldVortexProfile => {
                self.render_near_field_canvas(ui);
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
                .button("Symmetry-Protected Gamma-BIC (q = +1, alpha = 0)")
                .on_hover_text("Ideal BIC at Gamma point with vanishing radiation coupling and infinite Q_rad")
                .clicked()
            {
                self.bic_kind = BicKind::SymmetryProtectedGamma;
                self.asymmetry_parameter = 0.0;
                self.recompute();
            }

            if ui
                .button("Friedrich-Wintgen Off-Gamma BIC (q = -1)")
                .on_hover_text("Interference BIC at off-Gamma momentum formed by destructive channel interference")
                .clicked()
            {
                self.bic_kind = BicKind::FriedrichWintgenOffGamma;
                self.asymmetry_parameter = 0.0;
                self.recompute();
            }

            if ui
                .button("High-Q Quasi-BIC (alpha = 0.05)")
                .on_hover_text("Slightly broken symmetry opening a controllable radiation channel with Q > 1500")
                .clicked()
            {
                self.bic_kind = BicKind::SymmetryProtectedGamma;
                self.asymmetry_parameter = 0.05;
                self.recompute();
            }

            if ui
                .button("Radiative Vortex Emitter (alpha = 0.15)")
                .on_hover_text("Substantial asymmetry emitting radiated acoustic vortex beam with high OAM mode purity")
                .clicked()
            {
                self.bic_kind = BicKind::SymmetryProtectedGamma;
                self.asymmetry_parameter = 0.15;
                self.recompute();
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;

            ui.label("Asymmetry alpha:");
            if ui
                .add(egui::Slider::new(&mut self.asymmetry_parameter, 0.0..=0.30).step_by(0.01))
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Resonance f0:");
            if ui
                .add(egui::Slider::new(&mut self.resonance_freq_hz, 1000.0..=8000.0).suffix(" Hz"))
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Intrinsic Loss:");
            if ui
                .add(egui::Slider::new(&mut self.intrinsic_loss_hz, 0.05..=5.0).suffix(" Hz"))
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Fano q_F:");
            if ui
                .add(egui::Slider::new(&mut self.fano_asymmetry_q, -5.0..=5.0).step_by(0.1))
                .changed()
            {
                changed = true;
            }

            if changed {
                self.recompute();
            }
        });
    }

    /// Render 2D Momentum-Space Far-Field Polarization Vortex Canvas.
    fn render_vortex_canvas(&self, ui: &mut Ui) {
        let (rect, _resp) = ui.allocate_exact_size(vec2(ui.available_width(), 340.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 6.0, COLOR_CANVAS_BG);

        let center_x = rect.center().x;
        let center_y = rect.center().y;
        let scale = (rect.height() * 0.42).min(rect.width() * 0.35);

        // Draw momentum grid axes
        painter.line_segment(
            [pos2(center_x - scale, center_y), pos2(center_x + scale, center_y)],
            Stroke::new(1.0, Color32::from_rgb(71, 85, 105)),
        );
        painter.line_segment(
            [pos2(center_x, center_y - scale), pos2(center_x, center_y + scale)],
            Stroke::new(1.0, Color32::from_rgb(71, 85, 105)),
        );

        painter.text(
            pos2(center_x + scale - 10.0, center_y + 12.0),
            egui::Align2::RIGHT_TOP,
            "kx",
            FontId::proportional(12.0),
            Color32::from_rgb(148, 163, 184),
        );
        painter.text(
            pos2(center_x + 12.0, center_y - scale + 10.0),
            egui::Align2::LEFT_TOP,
            "ky",
            FontId::proportional(12.0),
            Color32::from_rgb(148, 163, 184),
        );

        // Draw polarization vectors
        let vec_len = 10.0;
        for vec_pt in &self.engine.solver.polarization_field {
            let px = center_x + (vec_pt.kx_norm as f32) * scale;
            let py = center_y - (vec_pt.ky_norm as f32) * scale;

            if px >= rect.min.x + 8.0 && px <= rect.max.x - 8.0 && py >= rect.min.y + 8.0 && py <= rect.max.y - 8.0 {
                let dx = (vec_pt.vx as f32) * vec_len * 0.5;
                let dy = -(vec_pt.vy as f32) * vec_len * 0.5;

                // Color by orientation angle
                let angle_norm = ((vec_pt.orientation_angle_rad + std::f64::consts::PI)
                    / (2.0 * std::f64::consts::PI))
                    .clamp(0.0, 1.0) as f32;
                let color = Color32::from_rgb(
                    (56.0 + 180.0 * angle_norm) as u8,
                    (189.0 - 100.0 * angle_norm) as u8,
                    (248.0 - 120.0 * angle_norm) as u8,
                );

                painter.line_segment(
                    [pos2(px - dx, py - dy), pos2(px + dx, py + dy)],
                    Stroke::new(1.8, color),
                );
            }
        }

        // Draw BIC singularity core marker
        let (k_bic_x, k_bic_y) = self.engine.solver.bic_singularity_pos;
        let bic_px = center_x + (k_bic_x as f32) * scale;
        let bic_py = center_y - (k_bic_y as f32) * scale;

        painter.circle_filled(pos2(bic_px, bic_py), 6.0, COLOR_VORTEX_CORE);
        painter.circle_stroke(pos2(bic_px, bic_py), 10.0, Stroke::new(1.5, Color32::WHITE));

        // Legend / Overlay text
        let charge_str = format!("Topological Vortex Charge q = {:+}", self.engine.solver.calculated_topological_charge);
        painter.text(
            pos2(rect.min.x + 16.0, rect.min.y + 16.0),
            egui::Align2::LEFT_TOP,
            charge_str,
            FontId::proportional(14.0),
            COLOR_VORTEX_CORE,
        );
        painter.text(
            pos2(rect.min.x + 16.0, rect.min.y + 36.0),
            egui::Align2::LEFT_TOP,
            "Polarization field swirling around singularity",
            FontId::proportional(11.0),
            Color32::from_rgb(148, 163, 184),
        );
    }

    /// Render Diverging Loaded Quality Factor Q(k) across momentum.
    fn render_q_factor_plot(&self, ui: &mut Ui) {
        let (k_bic_x, _) = self.engine.solver.bic_singularity_pos;

        // Extract 1D cross-section along ky = 0
        let mut q_points = Vec::new();
        for pt in &self.engine.solver.polarization_field {
            if pt.ky_norm.abs() < 0.05 {
                q_points.push([pt.kx_norm, pt.quality_factor.min(50000.0)]);
            }
        }
        q_points.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());

        let plot = Plot::new("bic_q_factor_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Normalized Wavevector kx / (pi / a0)")
            .y_axis_label("Quality Factor Q(kx)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Loaded Quality Factor Q(kx)", PlotPoints::new(q_points))
                    .color(COLOR_Q_FACTOR)
                    .width(2.5),
            );
            plot_ui.vline(
                VLine::new("BIC Singularity k_BIC", k_bic_x)
                    .color(COLOR_VORTEX_CORE)
                    .stroke(Stroke::new(1.5, COLOR_VORTEX_CORE)),
            );
        });
    }

    /// Render Ultra-Sharp Fano Resonance Transmission Spectrum.
    fn render_fano_spectrum_plot(&self, ui: &mut Ui) {
        let f0 = self.engine.params.lattice_params.resonance_freq_hz;
        let t_pts: Vec<[f64; 2]> = self
            .engine
            .spectrum
            .iter()
            .map(|pt| [pt.frequency_hz, pt.transmission_db])
            .collect();

        let enh_pts: Vec<[f64; 2]> = self
            .engine
            .spectrum
            .iter()
            .map(|pt| [pt.frequency_hz, 10.0 * (pt.cavity_enhancement_ratio.max(1.0)).log10()])
            .collect();

        let plot = Plot::new("bic_fano_spectrum_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Frequency (Hz)")
            .y_axis_label("Transmission T / Enhancement (dB)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Fano Transmission T(f) (dB)", PlotPoints::new(t_pts))
                    .color(COLOR_FANO_TRANSMISSION)
                    .width(2.5),
            );
            plot_ui.line(
                Line::new("Cavity Energy Enhancement |p_cav|^2 / |p_inc|^2 (dB)", PlotPoints::new(enh_pts))
                    .color(COLOR_ENHANCEMENT)
                    .width(2.0),
            );
            plot_ui.vline(
                VLine::new("Resonance Frequency f0", f0)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
            plot_ui.hline(
                HLine::new("0 dB Level", 0.0)
                    .color(Color32::from_rgb(71, 85, 105)),
            );
        });
    }

    /// Render Quasi-BIC Asymmetry Scaling Curve Q(alpha).
    fn render_scaling_plot(&self, ui: &mut Ui) {
        let f0 = self.engine.params.lattice_params.resonance_freq_hz;
        let c_rad = self.engine.params.lattice_params.radiative_coupling_coeff_hz;
        let gamma_nr = self.engine.params.lattice_params.intrinsic_loss_hz;

        let num_alpha_pts = 41;
        let mut scaling_pts = Vec::with_capacity(num_alpha_pts);

        for i in 1..=num_alpha_pts {
            let a = 0.005 + 0.25 * (i as f64) / (num_alpha_pts as f64);
            let gamma_tot = c_rad * a * a + gamma_nr;
            let q = f0 / (2.0 * gamma_tot);
            scaling_pts.push([a, q]);
        }

        let current_a = self.asymmetry_parameter;
        let current_q = self.engine.metrics.peak_q_factor;

        let plot = Plot::new("bic_scaling_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Asymmetry Parameter alpha")
            .y_axis_label("Loaded Quality Factor Q");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Q(alpha) ~ 1 / alpha^2 Scaling", PlotPoints::new(scaling_pts))
                    .color(COLOR_Q_FACTOR)
                    .width(2.5),
            );
            plot_ui.vline(
                VLine::new("Current Asymmetry alpha", current_a)
                    .color(COLOR_ENHANCEMENT),
            );
            plot_ui.hline(
                HLine::new("Current Q Level", current_q)
                    .color(COLOR_ENHANCEMENT),
            );
        });
    }

    /// Render Near-Field Acoustic Radiation Vortex Canvas.
    fn render_near_field_canvas(&self, ui: &mut Ui) {
        let (rect, _resp) = ui.allocate_exact_size(vec2(ui.available_width(), 340.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 6.0, COLOR_CANVAS_BG);

        let center_x = rect.center().x;
        let center_y = rect.center().y;
        let r_max_px = (rect.height() * 0.42).min(rect.width() * 0.35);

        // Draw cavity boundary circle
        painter.circle_stroke(
            pos2(center_x, center_y),
            r_max_px,
            Stroke::new(1.5, Color32::from_rgb(71, 85, 105)),
        );

        let cavity_r_mm = self.cavity_diameter_mm * 0.5;

        // Render pressure distribution and phase markers
        for pt in &self.engine.near_field_grid {
            let px = center_x + (pt.x_mm as f32 / cavity_r_mm as f32) * r_max_px;
            let py = center_y - (pt.y_mm as f32 / cavity_r_mm as f32) * r_max_px;

            if px >= rect.min.x + 4.0 && px <= rect.max.x - 4.0 && py >= rect.min.y + 4.0 && py <= rect.max.y - 4.0 {
                // Color by phase and intensity
                let phase_norm = ((pt.phase_rad + std::f64::consts::PI)
                    / (2.0 * std::f64::consts::PI))
                    .fract()
                    .abs() as f32;
                let intensity = (pt.pressure_amplitude as f32).clamp(0.0, 1.0);

                let color = Color32::from_rgba_premultiplied(
                    (168.0 * phase_norm * intensity) as u8,
                    (85.0 * (1.0 - phase_norm) * intensity) as u8,
                    (247.0 * intensity) as u8,
                    (200.0 * intensity) as u8,
                );

                let dot_radius = 2.0 + 3.0 * intensity;
                painter.circle_filled(pos2(px, py), dot_radius, color);
            }
        }

        // Draw central vortex core null
        painter.circle_filled(pos2(center_x, center_y), 5.0, COLOR_VORTEX_CORE);
        painter.circle_stroke(pos2(center_x, center_y), 7.0, Stroke::new(1.0, Color32::WHITE));

        // Legend / Overlay text
        painter.text(
            pos2(rect.min.x + 16.0, rect.min.y + 16.0),
            egui::Align2::LEFT_TOP,
            format!("Vortex Donut Profile (OAM Purity: {:.1}%)", self.engine.metrics.oam_mode_purity_pct),
            FontId::proportional(14.0),
            COLOR_DONUT_RING,
        );
        painter.text(
            pos2(rect.min.x + 16.0, rect.min.y + 36.0),
            egui::Align2::LEFT_TOP,
            "Center phase singularity with zero pressure amplitude at core",
            FontId::proportional(11.0),
            Color32::from_rgb(148, 163, 184),
        );
    }

    /// Render telemetry metrics footer.
    fn render_telemetry(&self, ui: &mut Ui) {
        let metrics = &self.engine.metrics;
        let bic_kind_str = match self.bic_kind {
            BicKind::SymmetryProtectedGamma => "Symmetry-Protected Gamma-BIC",
            BicKind::FriedrichWintgenOffGamma => "Friedrich-Wintgen Off-Gamma BIC",
            BicKind::HigherOrderVortex => "Higher-Order Vortex BIC",
        };

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(bic_kind_str).color(COLOR_Q_FACTOR).strong());
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Topological Charge q: {:+}", metrics.topological_vortex_charge)).strong());
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Loaded Q: {:.0}", metrics.peak_q_factor)).color(COLOR_Q_FACTOR));
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("gamma_rad: {:.2} Hz", metrics.radiative_linewidth_hz)));
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Field Enhancement: {:.0}x", metrics.peak_field_enhancement)).color(COLOR_ENHANCEMENT));
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Fano q_F: {:.1}", metrics.fano_asymmetry_q)));
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("OAM Purity: {:.1}%", metrics.oam_mode_purity_pct)).color(COLOR_DONUT_RING));
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Fano Dip: {:.1} dB", metrics.min_transmission_dip_db)));
        });
    }
}
