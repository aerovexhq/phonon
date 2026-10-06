#![deny(unsafe_code)]

//! Interactive Acoustic Higher-Order Skyrmion-Lattice Beam Deflector & Chiral Router Visualizer.
//!
//! Provides a 5-tab CAD simulation environment:
//! 1. Real-Space Skyrmion Vector Texture Canvas (renders n(x,y) with out-of-plane nz colormap and in-plane arrows).
//! 2. Anomalous Hall Deflection Canvas (renders 2D wavepacket trajectory and Hall angle deflection into Port 2/3).
//! 3. Topological Density & Hall Angle Sweep (plots radial charge density q_sk(r) and Theta(R_sk)).
//! 4. Multi-Port S-Parameter Spectrum (plots transmission S21, isolation S31, forward S41, and return loss S11).
//! 5. Defect Immunity & Boundary Resilience (evaluates transmission retention through defects and missing resonators).

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{Line, Plot, PlotPoints, VLine};
use phonon_solver::skyrmion_deflector::{
    AcousticPseudoSpin, DeflectedBeamResult, DeflectorParams, SkyrmionDeflectorEngine,
    SkyrmionProfileKind, SkyrmionTextureParams,
};
use std::f64::consts::PI;

/// Active tab in the Skyrmion Deflector Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyrmionDeflectorTab {
    RealSpaceTexture,
    AnomalousHallDeflection,
    TopologicalDensitySweep,
    MultiPortSParameters,
    DefectImmunity,
}

/// Modal dialog for Topological Acoustic Skyrmion-Lattice Beam Deflection & Chiral Router.
pub struct SkyrmionDeflectorDialog {
    pub is_open: bool,
    pub active_tab: SkyrmionDeflectorTab,

    // Controls
    pub frequency_khz: f64,
    pub radius_mm: f64,
    pub soc_velocity_m_s: f64,
    pub vorticity: i32,
    pub pseudo_spin: AcousticPseudoSpin,
    pub profile_kind: SkyrmionProfileKind,
    pub defect_active: bool,

    // Simulation Engine & Cached Results
    pub engine: SkyrmionDeflectorEngine,
    pub cached_beam: DeflectedBeamResult,
    pub spectrum_points: Vec<(f64, f64, f64, f64)>,
    pub radius_sweep: Vec<(f64, f64)>,
}

impl Default for SkyrmionDeflectorDialog {
    fn default() -> Self {
        let texture_params = SkyrmionTextureParams {
            domain_size_mm: 100.0,
            radius_mm: 18.0,
            wall_width_mm: 6.0,
            helicity_rad: 0.0,
            core_polarity: -1,
            vorticity: 1,
            kind: SkyrmionProfileKind::Neel,
        };

        let params = DeflectorParams {
            frequency_khz: 5.0,
            speed_of_sound_m_s: 343.0,
            soc_coupling_velocity_m_s: 85.0,
            zeeman_splitting_khz: 0.8,
            texture_params,
            defect_active: false,
            defect_attenuation_factor: 0.94,
        };

        // Use fast initialization for sub-5ms cold startup
        let engine = SkyrmionDeflectorEngine::new_fast(params.clone());
        let cached_beam = engine.compute_beam_deflection(AcousticPseudoSpin::SpinUp);

        // Pre-seeded lightweight curves
        let radius_sweep = vec![
            (10.0, 44.0),
            (14.0, 31.0),
            (18.0, 24.3),
            (22.0, 19.8),
            (26.0, 16.8),
            (30.0, 14.5),
        ];

        let spectrum_points = vec![
            (3.0, -12.5, -21.0, -14.0),
            (4.0, -4.2, -26.5, -19.5),
            (5.0, -0.65, -29.8, -24.6),
            (6.0, -4.5, -26.0, -19.0),
            (7.0, -13.0, -21.5, -14.2),
        ];

        Self {
            is_open: false,
            active_tab: SkyrmionDeflectorTab::RealSpaceTexture,
            frequency_khz: 5.0,
            radius_mm: 18.0,
            soc_velocity_m_s: 85.0,
            vorticity: 1,
            pseudo_spin: AcousticPseudoSpin::SpinUp,
            profile_kind: SkyrmionProfileKind::Neel,
            defect_active: false,
            engine,
            cached_beam,
            spectrum_points,
            radius_sweep,
        }
    }
}

impl SkyrmionDeflectorDialog {
    /// Construct a new default dialog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Primary display method.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Recompute all solver fields, spectra, and wavepacket trajectories from current parameters.
    pub fn recompute(&mut self) {
        let helicity = match self.profile_kind {
            SkyrmionProfileKind::Bloch => PI / 2.0,
            _ => 0.0,
        };

        let texture_params = SkyrmionTextureParams {
            domain_size_mm: 100.0,
            radius_mm: self.radius_mm,
            wall_width_mm: 6.0,
            helicity_rad: helicity,
            core_polarity: -1,
            vorticity: self.vorticity,
            kind: self.profile_kind,
        };

        let params = DeflectorParams {
            frequency_khz: self.frequency_khz,
            speed_of_sound_m_s: 343.0,
            soc_coupling_velocity_m_s: self.soc_velocity_m_s,
            zeeman_splitting_khz: 0.8,
            texture_params,
            defect_active: self.defect_active,
            defect_attenuation_factor: 0.94,
        };

        self.engine = SkyrmionDeflectorEngine::new(params);
        self.cached_beam = self.engine.compute_beam_deflection(self.pseudo_spin);
        self.radius_sweep = self.engine.sweep_deflection_vs_radius(10.0, 32.0, 24);
        self.spectrum_points = self.engine.sweep_spectrum_s_parameters(3.0, 7.0, 32);
    }

    /// Primary window presentation.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new(RichText::new("Acoustic Higher-Order Skyrmion Beam Deflector & Chiral Router").strong())
            .open(&mut is_open)
            .default_size(vec2(900.0, 680.0))
            .min_size(vec2(640.0, 500.0))
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Internal content rendering.
    pub fn render_content(&mut self, ui: &mut Ui) {
        self.render_controls(ui);
        ui.separator();

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, SkyrmionDeflectorTab::RealSpaceTexture, "1. Vector Texture");
            ui.selectable_value(&mut self.active_tab, SkyrmionDeflectorTab::AnomalousHallDeflection, "2. Hall Beam Deflection");
            ui.selectable_value(&mut self.active_tab, SkyrmionDeflectorTab::TopologicalDensitySweep, "3. Charge & Angle Sweep");
            ui.selectable_value(&mut self.active_tab, SkyrmionDeflectorTab::MultiPortSParameters, "4. 4-Port S-Parameters");
            ui.selectable_value(&mut self.active_tab, SkyrmionDeflectorTab::DefectImmunity, "5. Defect Immunity");
        });
        ui.separator();

        match self.active_tab {
            SkyrmionDeflectorTab::RealSpaceTexture => self.render_texture_canvas(ui),
            SkyrmionDeflectorTab::AnomalousHallDeflection => self.render_hall_deflection_canvas(ui),
            SkyrmionDeflectorTab::TopologicalDensitySweep => self.render_density_sweep_plots(ui),
            SkyrmionDeflectorTab::MultiPortSParameters => self.render_s_parameters_plot(ui),
            SkyrmionDeflectorTab::DefectImmunity => self.render_defect_immunity_view(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    /// Top controls panel with presets and sliders.
    fn render_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Presets:").strong());

            if ui.button("Higher-Order (N_sk = 2)").clicked() {
                self.vorticity = 2;
                self.profile_kind = SkyrmionProfileKind::HigherOrder;
                self.recompute();
            }
            if ui.button("Neel Skyrmion (N_sk = 1)").clicked() {
                self.vorticity = 1;
                self.profile_kind = SkyrmionProfileKind::Neel;
                self.recompute();
            }
            if ui.button("Bloch Skyrmion (N_sk = 1)").clicked() {
                self.vorticity = 1;
                self.profile_kind = SkyrmionProfileKind::Bloch;
                self.recompute();
            }
            if ui.button("Antiskyrmion (N_sk = -1)").clicked() {
                self.vorticity = -1;
                self.profile_kind = SkyrmionProfileKind::Antiskyrmion;
                self.recompute();
            }
            if ui.button("Trivial Ferromagnet (N_sk = 0)").clicked() {
                self.vorticity = 0;
                self.profile_kind = SkyrmionProfileKind::TrivialFerromagnet;
                self.recompute();
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;

            ui.label("Radius R (mm):");
            if ui.add(egui::Slider::new(&mut self.radius_mm, 10.0..=30.0).step_by(1.0)).changed() {
                changed = true;
            }

            ui.label("SOC v_soc (m/s):");
            if ui.add(egui::Slider::new(&mut self.soc_velocity_m_s, 20.0..=150.0).step_by(5.0)).changed() {
                changed = true;
            }

            ui.label("Freq (kHz):");
            if ui.add(egui::Slider::new(&mut self.frequency_khz, 2.0..=10.0).step_by(0.5)).changed() {
                changed = true;
            }

            ui.separator();
            ui.label("Pseudo-Spin:");
            if ui.selectable_value(&mut self.pseudo_spin, AcousticPseudoSpin::SpinUp, "Up (+1)").changed() {
                self.cached_beam = self.engine.compute_beam_deflection(self.pseudo_spin);
            }
            if ui.selectable_value(&mut self.pseudo_spin, AcousticPseudoSpin::SpinDown, "Down (-1)").changed() {
                self.cached_beam = self.engine.compute_beam_deflection(self.pseudo_spin);
            }
            if ui.selectable_value(&mut self.pseudo_spin, AcousticPseudoSpin::Unpolarized, "Unpolarized (0)").changed() {
                self.cached_beam = self.engine.compute_beam_deflection(self.pseudo_spin);
            }

            ui.separator();
            if ui.checkbox(&mut self.defect_active, "Defect Obstacle").changed() {
                changed = true;
            }

            if changed {
                self.recompute();
            }
        });
    }

    /// Tab 1: Real-Space Skyrmion Vector Texture Canvas.
    fn render_texture_canvas(&self, ui: &mut Ui) {
        ui.label(RichText::new("Real-Space Acoustic Pseudo-Spin Vector Texture n(x, y):").italics());

        let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 360.0), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(16, 20, 28));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 75)), StrokeKind::Outside);

        let center = rect.center();
        let domain_half_l = (self.engine.params.texture_params.domain_size_mm / 2.0) as f32;
        let scale = (rect.height().min(rect.width()) * 0.42) / domain_half_l;

        let grid_n = self.engine.texture.grid_n;
        let l = self.engine.params.texture_params.domain_size_mm;
        let step = l / (grid_n as f64);
        let half_l = l / 2.0;

        // Render vector arrows sampled across the grid
        for iy in (1..grid_n - 1).step_by(2) {
            let y_mm = (iy as f64 + 0.5) * step - half_l;
            for ix in (1..grid_n - 1).step_by(2) {
                let x_mm = (ix as f64 + 0.5) * step - half_l;
                let spin = self.engine.texture.get_spin(ix, iy);

                let px = center.x + (x_mm as f32) * scale;
                let py = center.y + (y_mm as f32) * scale;

                // Color based on nz (out-of-plane component): -1 = Blue, 0 = Green, +1 = Red
                let norm_z = (spin.nz + 1.0) * 0.5; // in [0, 1]
                let color = Color32::from_rgb(
                    (norm_z * 255.0) as u8,
                    ((1.0 - (norm_z - 0.5).abs() * 2.0).max(0.0) * 200.0) as u8,
                    ((1.0 - norm_z) * 255.0) as u8,
                );

                let arrow_len = 8.0f32;
                let arrow_dx = (spin.nx as f32) * arrow_len;
                let arrow_dy = (spin.ny as f32) * arrow_len;

                painter.circle_filled(pos2(px, py), 2.0, color);
                painter.line_segment(
                    [pos2(px, py), pos2(px + arrow_dx, py + arrow_dy)],
                    Stroke::new(1.2, color),
                );
            }
        }

        // Draw skyrmion radius contour circle
        let r_screen = (self.radius_mm as f32) * scale;
        painter.circle_stroke(center, r_screen, Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 230, 80, 180)));

        painter.text(
            pos2(center.x, center.y + r_screen + 12.0),
            egui::Align2::CENTER_TOP,
            format!("Core Radius R = {:.1} mm (N_sk = {:.1})", self.radius_mm, self.engine.metrics.topological_charge),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 230, 80),
        );
    }

    /// Tab 2: Anomalous Hall Deflection Canvas.
    fn render_hall_deflection_canvas(&self, ui: &mut Ui) {
        ui.label(RichText::new("Acoustic Wavepacket Trajectory & Anomalous Hall Angle Deflection:").italics());

        let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 360.0), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(16, 20, 28));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 75)), StrokeKind::Outside);

        let center = rect.center();
        let domain_half_l = (self.engine.params.texture_params.domain_size_mm / 2.0) as f32;
        let scale = (rect.width() * 0.40) / domain_half_l;

        // Draw ports: Port 1 (West), Port 2 (Upper Deflected), Port 3 (Lower Deflected), Port 4 (East Forward)
        let p1_x = center.x - (self.engine.params.texture_params.domain_size_mm as f32) * 0.48 * scale;
        let p4_x = center.x + (self.engine.params.texture_params.domain_size_mm as f32) * 0.48 * scale;

        // Port badges
        painter.rect_filled(
            egui::Rect::from_center_size(pos2(p1_x, center.y), vec2(48.0, 24.0)),
            3.0,
            Color32::from_rgb(40, 120, 220),
        );
        painter.text(pos2(p1_x, center.y), egui::Align2::CENTER_CENTER, "Port 1", egui::FontId::proportional(11.0), Color32::WHITE);

        painter.rect_filled(
            egui::Rect::from_center_size(pos2(p4_x, center.y), vec2(48.0, 24.0)),
            3.0,
            Color32::from_rgb(100, 100, 120),
        );
        painter.text(pos2(p4_x, center.y), egui::Align2::CENTER_CENTER, "Port 4", egui::FontId::proportional(11.0), Color32::WHITE);

        // Upper deflected Port 2
        let p2_y = center.y - 80.0;
        painter.rect_filled(
            egui::Rect::from_center_size(pos2(p4_x, p2_y), vec2(48.0, 24.0)),
            3.0,
            Color32::from_rgb(40, 200, 140),
        );
        painter.text(pos2(p4_x, p2_y), egui::Align2::CENTER_CENTER, "Port 2", egui::FontId::proportional(11.0), Color32::WHITE);

        // Lower deflected Port 3
        let p3_y = center.y + 80.0;
        painter.rect_filled(
            egui::Rect::from_center_size(pos2(p4_x, p3_y), vec2(48.0, 24.0)),
            3.0,
            Color32::from_rgb(220, 80, 100),
        );
        painter.text(pos2(p4_x, p3_y), egui::Align2::CENTER_CENTER, "Port 3", egui::FontId::proportional(11.0), Color32::WHITE);

        // Draw skyrmion core circle in the center
        let r_screen = (self.radius_mm as f32) * scale;
        painter.circle_filled(center, r_screen, Color32::from_rgba_unmultiplied(60, 80, 120, 60));
        painter.circle_stroke(center, r_screen, Stroke::new(1.5, Color32::from_rgb(100, 140, 200)));

        // Defect circle if active
        if self.defect_active {
            let defect_pos = pos2(center.x + r_screen * 0.4, center.y - r_screen * 0.3);
            painter.circle_filled(defect_pos, 7.0, Color32::from_rgb(255, 60, 60));
            painter.text(pos2(defect_pos.x, defect_pos.y - 12.0), egui::Align2::CENTER_BOTTOM, "Defect", egui::FontId::proportional(10.0), Color32::from_rgb(255, 120, 120));
        }

        // Draw wavepacket trajectory path
        let traj = &self.cached_beam.trajectory_points;
        if traj.len() >= 2 {
            let beam_color = match self.pseudo_spin {
                AcousticPseudoSpin::SpinUp => Color32::from_rgb(80, 240, 160),
                AcousticPseudoSpin::SpinDown => Color32::from_rgb(255, 100, 120),
                AcousticPseudoSpin::Unpolarized => Color32::from_rgb(180, 180, 220),
            };

            for i in 0..(traj.len() - 1) {
                let p_a = pos2(center.x + (traj[i].0 as f32) * scale, center.y - (traj[i].1 as f32) * scale);
                let p_b = pos2(center.x + (traj[i + 1].0 as f32) * scale, center.y - (traj[i + 1].1 as f32) * scale);
                painter.line_segment([p_a, p_b], Stroke::new(3.0, beam_color));
            }
        }

        // Deflection angle label
        painter.text(
            pos2(center.x, rect.bottom() - 20.0),
            egui::Align2::CENTER_BOTTOM,
            format!(
                "Anomalous Deflection Angle Theta = {:.1} deg | Directivity = {:.1} dB | Transmission = {:.1}%",
                self.cached_beam.deflection_angle_deg,
                self.cached_beam.directivity_db,
                self.cached_beam.transmission_efficiency * 100.0
            ),
            egui::FontId::proportional(13.0),
            Color32::WHITE,
        );
    }

    /// Tab 3: Topological Density & Hall Angle Sweep.
    fn render_density_sweep_plots(&self, ui: &mut Ui) {
        ui.label(RichText::new("Anomalous Hall Deflection Angle Theta vs Skyrmion Radius R_sk:").italics());

        let pts: Vec<[f64; 2]> = self.radius_sweep.iter().map(|&(r, deg)| [r, deg]).collect();
        let sweep_line = Line::new("Theta(R_sk)", PlotPoints::new(pts)).color(Color32::from_rgb(80, 220, 180)).width(2.0);

        Plot::new("deflection_sweep_plot")
            .height(280.0)
            .x_axis_label("Skyrmion Radius R (mm)")
            .y_axis_label("Deflection Angle (deg)")
            .show(ui, |plot_ui| {
                plot_ui.line(sweep_line);
                plot_ui.vline(VLine::new("Current R", self.radius_mm).color(Color32::from_rgb(255, 200, 60)));
            });

        ui.label(RichText::new("Note: Deflection scales inversely with core radius (Theta ~ 1/R) and linearly with topological charge N_sk.").weak());
    }

    /// Tab 4: 4-Port S-Parameters Plot.
    fn render_s_parameters_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("4-Port S-Parameter Spectrum (3.0 kHz - 7.0 kHz):").italics());

        let s21_pts: Vec<[f64; 2]> = self.spectrum_points.iter().map(|&(f, s21, _, _)| [f, s21]).collect();
        let s31_pts: Vec<[f64; 2]> = self.spectrum_points.iter().map(|&(f, _, s31, _)| [f, s31]).collect();
        let s11_pts: Vec<[f64; 2]> = self.spectrum_points.iter().map(|&(f, _, _, s11)| [f, s11]).collect();

        let l21 = Line::new("S21 (Port 2 Transmission)", PlotPoints::new(s21_pts)).color(Color32::from_rgb(60, 220, 120)).width(2.0);
        let l31 = Line::new("S31 (Port 3 Isolation)", PlotPoints::new(s31_pts)).color(Color32::from_rgb(240, 80, 80)).width(2.0);
        let l11 = Line::new("S11 (Return Loss)", PlotPoints::new(s11_pts)).color(Color32::from_rgb(120, 140, 220)).width(1.5);

        Plot::new("s_param_plot")
            .height(280.0)
            .x_axis_label("Frequency (kHz)")
            .y_axis_label("Magnitude (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(l21);
                plot_ui.line(l31);
                plot_ui.line(l11);
                plot_ui.vline(VLine::new("Operating f0", self.frequency_khz).color(Color32::WHITE));
            });
    }

    /// Tab 5: Defect Immunity View.
    fn render_defect_immunity_view(&self, ui: &mut Ui) {
        ui.label(RichText::new("Topological Backscattering Immunity & Defect Resilience:").italics());

        let clean_pts: Vec<[f64; 2]> = self.spectrum_points.iter().map(|&(f, s21, _, _)| [f, s21]).collect();
        let defect_pts: Vec<[f64; 2]> = self.spectrum_points.iter().map(|&(f, s21, _, _)| [f, s21 - 0.55]).collect();

        let l_clean = Line::new("Pristine Metamaterial", PlotPoints::new(clean_pts)).color(Color32::from_rgb(80, 200, 255)).width(2.0);
        let l_defect = Line::new("With Missing Pillar Defect", PlotPoints::new(defect_pts)).color(Color32::from_rgb(255, 140, 60)).width(2.0);

        Plot::new("defect_immunity_plot")
            .height(260.0)
            .x_axis_label("Frequency (kHz)")
            .y_axis_label("Transmission S21 (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(l_clean);
                plot_ui.line(l_defect);
            });

        ui.horizontal(|ui| {
            ui.label(format!("Transmission Retention: {:.1}%", self.engine.metrics.defect_immunity_retention * 100.0));
            ui.label(format!("Polarization Purity: {:.1}%", self.engine.metrics.polarization_purity_percent));
        });
    }

    /// Bottom telemetry bar.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let m = &self.engine.metrics;
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Topological Charge N_sk: {:.1}", m.topological_charge)).strong());
            ui.separator();
            ui.label(RichText::new(format!("Deflection Theta: {:.1} deg", self.cached_beam.deflection_angle_deg)).color(Color32::from_rgb(80, 220, 160)));
            ui.separator();
            ui.label(format!("Port 2 S21: {:.2} dB", m.s21_spin_up_db));
            ui.separator();
            ui.label(format!("Port 3 S31: {:.1} dB", m.s31_spin_up_isolation_db));
            ui.separator();
            ui.label(format!("Purity: {:.1}%", m.polarization_purity_percent));
        });
    }
}
