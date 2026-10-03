#![deny(unsafe_code)]

//! Interactive Floquet Engineered Spatio-Temporal Acoustic Metasurface Simulator dialog.
//!
//! Provides:
//! - 2D Spatio-Temporal Wavefront Canvas: interactive 2D spatial acoustic pressure field Re[p(x, y, t)]
//!   rendering incident plane wave propagating toward metasurface boundary, dynamic modulating phase elements
//!   along x with shifting colors, and deflected reflected beam steered at angle theta_refl.
//! - Floquet Harmonic Spectrum Bar Chart: plots reflection power in sidebands n in {-2, -1, 0, +1, +2} in dB,
//!   showing energy conversion into dominant n = +1 Doppler-shifted mode.
//! - Non-Reciprocal Transmission Isolation Plot: plots forward transmission S_21(f) vs backward transmission S_12(f)
//!   in dB, highlighting the non-reciprocal isolation band (>= 30 dB).
//! - OAM Vortex Beam Helical Phase Map: 2D polar colormap (r, theta) showing spiral phase winding exp(i * l * theta)
//!   with l arms.
//! - Telemetry Footer: Steering Angle theta_refl (deg), Floquet Frequency Shift Delta f (Hz),
//!   Non-Reciprocal Isolation (dB), OAM Mode Purity (%), Synthetic Gauge Field B_eff, Dominant Sideband Efficiency (%).

use egui::{
    pos2, vec2, Color32, FontId, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{Bar, BarChart, HLine, Legend, Line, Plot, PlotPoints};
use phonon_solver::floquet_metasurface::{
    FloquetMetasurfaceSolver, FloquetModulationParams, FloquetSidebandResult,
    MetasurfaceUnitCell, NonReciprocalScattering, OrbitalAngularMomentum, PolarPhaseMap,
    SyntheticGaugeField,
};
use std::f64::consts::PI;

/// Interactive modal dialog for Floquet Spatio-Temporal Metasurface simulation and CAD visualization.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetMetasurfaceDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Physical Controls
    /// Incident carrier frequency f_inc in Hz (default 5000.0 Hz).
    pub carrier_freq_hz: f64,
    /// Incident angle theta_inc in degrees (default 15.0 deg).
    pub incident_angle_deg: f64,
    /// Spatio-temporal modulation frequency Omega_m in Hz (default 500.0 Hz).
    pub modulation_freq_hz: f64,
    /// Spatial phase gradient g_x in rad/m (default 150.0 rad/m).
    pub phase_gradient_rad_m: f64,
    /// Parametric modulation depth M in [0.0, 1.0] (default 0.6).
    pub modulation_depth: f64,
    /// Max Floquet harmonic sideband order N_F (default 2).
    pub harmonic_order: i32,
    /// Incident OAM charge l_in (default 0).
    pub oam_in: i32,
    /// Transferred OAM charge Delta_l (default 1).
    pub oam_delta: i32,
    /// Acoustic medium speed of sound c_s in m/s (default 343.0 m/s).
    pub speed_of_sound_m_s: f64,
    /// Unit cell spatial pitch d_x in meters (default 0.01 m).
    pub unit_cell_pitch_m: f64,

    // Animation Controls
    /// Virtual acoustic time t in seconds for dynamic phase visualization.
    pub current_time_s: f64,
    /// Flag indicating whether virtual time animation is active.
    pub is_animating: bool,

    // Cached Simulation State
    /// Evaluated Floquet harmonic sidebands.
    pub sidebands: Vec<FloquetSidebandResult>,
    /// Non-reciprocal scattering parameters.
    pub scattering: NonReciprocalScattering,
    /// Synthetic gauge field evaluation.
    pub gauge_field: SyntheticGaugeField,
    /// Orbital Angular Momentum vortex configuration.
    pub oam: OrbitalAngularMomentum,
    /// 2D polar phase map for vortex visualization.
    pub polar_map: PolarPhaseMap,
    /// Forward transmission curve [f_hz, S21_db].
    pub s21_curve: Vec<[f64; 2]>,
    /// Backward transmission curve [f_hz, S12_db].
    pub s12_curve: Vec<[f64; 2]>,

    // Telemetry Metrics
    /// Steering angle for dominant / target sideband in degrees (if propagating).
    pub steering_angle_deg: Option<f64>,
    /// Doppler frequency shift Delta_f for dominant sideband in Hz.
    pub floquet_freq_shift_hz: f64,
    /// Non-reciprocal isolation S_21 - S_12 in dB.
    pub isolation_db: f64,
    /// OAM vortex modal purity in percent (e.g. 96.8%).
    pub oam_purity_pct: f64,
    /// Synthetic magnetic field B_eff in s/m^2.
    pub b_eff: f64,
    /// Dominant Doppler sideband power reflection efficiency in percent.
    pub dominant_efficiency_pct: f64,

    /// Execution rerun flag.
    pub run_requested: bool,
}

impl Default for FloquetMetasurfaceDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl FloquetMetasurfaceDialog {
    /// Creates a new Floquet Metasurface dialog initialized with nominal parameters and solved state.
    pub fn new() -> Self {
        let mut dialog = Self {
            is_open: false,
            carrier_freq_hz: 5000.0,
            incident_angle_deg: 15.0,
            modulation_freq_hz: 500.0,
            phase_gradient_rad_m: 150.0,
            modulation_depth: 0.6,
            harmonic_order: 2,
            oam_in: 0,
            oam_delta: 1,
            speed_of_sound_m_s: 343.0,
            unit_cell_pitch_m: 0.01,
            current_time_s: 0.0,
            is_animating: true,

            sidebands: Vec::new(),
            scattering: NonReciprocalScattering {
                s21_linear: 0.8,
                s21_db: -0.97,
                s12_linear: 0.0008,
                s12_db: -31.0,
                isolation_db: 32.5,
            },
            gauge_field: SyntheticGaugeField::default(),
            oam: OrbitalAngularMomentum::default(),
            polar_map: PolarPhaseMap {
                r_steps: 16,
                theta_steps: 32,
                grid: Vec::new(),
            },
            s21_curve: Vec::new(),
            s12_curve: Vec::new(),

            steering_angle_deg: None,
            floquet_freq_shift_hz: 500.0,
            isolation_db: 32.5,
            oam_purity_pct: 96.8,
            b_eff: 4.77,
            dominant_efficiency_pct: 8.2,

            run_requested: false,
        };

        dialog.recompute();
        dialog
    }

    /// Recomputes all physical dispersion relations, sideband harmonics, non-reciprocity, and phase maps.
    pub fn recompute(&mut self) {
        let cell = MetasurfaceUnitCell::new(
            self.carrier_freq_hz,
            self.unit_cell_pitch_m,
            self.speed_of_sound_m_s,
            1.225,
        );
        let modulation = FloquetModulationParams::new(
            self.carrier_freq_hz,
            self.incident_angle_deg,
            self.modulation_freq_hz,
            self.phase_gradient_rad_m,
            self.modulation_depth,
            self.harmonic_order,
        );
        let solver = FloquetMetasurfaceSolver::new(cell, modulation);

        self.sidebands = solver.compute_sidebands();
        self.scattering = solver.compute_scattering();
        self.isolation_db = self.scattering.isolation_db;

        // Synthetic gauge field
        self.gauge_field = SyntheticGaugeField::new(
            self.phase_gradient_rad_m,
            self.modulation_freq_hz,
            self.unit_cell_pitch_m,
        );
        self.b_eff = self.gauge_field.b_eff_z;

        // OAM configuration
        self.oam = OrbitalAngularMomentum::new(
            self.oam_in,
            self.oam_delta,
            self.modulation_freq_hz,
        );
        self.oam_purity_pct = self.oam.mode_purity * 100.0;
        self.polar_map = self.oam.generate_polar_phase_map(16, 32, self.current_time_s);

        // Dominant sideband telemetry
        if let Some(dom) = self.sidebands.iter().find(|s| s.n == 1) {
            self.steering_angle_deg = dom.theta_refl_deg;
            self.floquet_freq_shift_hz = dom.delta_f;
            self.dominant_efficiency_pct = dom.power_refl * 100.0;
        } else {
            self.steering_angle_deg = None;
            self.floquet_freq_shift_hz = 0.0;
            self.dominant_efficiency_pct = 0.0;
        }

        // Transmission spectrum across carrier passband
        let span = 1000.0;
        let (s21, s12) = solver.compute_transmission_spectrum(
            self.carrier_freq_hz - span,
            self.carrier_freq_hz + span,
            81,
        );
        self.s21_curve = s21;
        self.s12_curve = s12;
    }

    /// Advances the virtual acoustic time for wave animations.
    pub fn advance_time(&mut self, dt_s: f64) {
        if self.is_animating {
            self.current_time_s += dt_s;
            // Keep time within periodic bounds to prevent float overflow
            let period = if self.modulation_freq_hz > 1e-6 {
                1.0 / self.modulation_freq_hz
            } else {
                1.0
            };
            if self.current_time_s > 10.0 * period {
                self.current_time_s = self.current_time_s.rem_euclid(period);
            }
            self.polar_map = self.oam.generate_polar_phase_map(16, 32, self.current_time_s);
        }
    }

    /// Renders the modal dialog window if visible.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let dt = ctx.input(|i| i.stable_dt).min(0.05) as f64;
        self.advance_time(dt);

        let mut is_open = self.is_open;
        egui::Window::new(
            RichText::new("Floquet Engineered Spatio-Temporal Acoustic Metasurface Studio")
                .strong()
                .size(14.0)
                .color(Color32::from_rgb(100, 220, 255)),
        )
        .open(&mut is_open)
        .default_width(980.0)
        .default_height(680.0)
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            self.render_content(ui);
        });
        self.is_open = is_open;
    }

    /// Renders inner dialog contents: Controls, 2x2 Grid Visualization Canvas, and Telemetry Footer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut recompute = false;

        // 1. Top Controls Bar
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Carrier f_inc:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.carrier_freq_hz, 1000.0..=15000.0)
                        .step_by(100.0)
                        .suffix(" Hz"),
                )
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            ui.label(RichText::new("Incident Angle:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.incident_angle_deg, -60.0..=60.0)
                        .step_by(1.0)
                        .suffix(" deg"),
                )
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            ui.label(RichText::new("Modulation Omega_m:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.modulation_freq_hz, 50.0..=2000.0)
                        .step_by(25.0)
                        .suffix(" Hz"),
                )
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            ui.label(RichText::new("Gradient g_x:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.phase_gradient_rad_m, 0.0..=300.0)
                        .step_by(10.0)
                        .suffix(" rad/m"),
                )
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            ui.label(RichText::new("Depth M:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(egui::Slider::new(&mut self.modulation_depth, 0.0..=1.0).step_by(0.05))
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            ui.label(RichText::new("OAM Delta_l:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(egui::Slider::new(&mut self.oam_delta, -3..=3).step_by(1.0))
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            let anim_label = if self.is_animating { "Pause" } else { "Play" };
            if ui.button(anim_label).clicked() {
                self.is_animating = !self.is_animating;
            }

            if ui.button("Reset Defaults").clicked() {
                self.carrier_freq_hz = 5000.0;
                self.incident_angle_deg = 15.0;
                self.modulation_freq_hz = 500.0;
                self.phase_gradient_rad_m = 150.0;
                self.modulation_depth = 0.6;
                self.harmonic_order = 2;
                self.oam_in = 0;
                self.oam_delta = 1;
                self.speed_of_sound_m_s = 343.0;
                self.unit_cell_pitch_m = 0.01;
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
        let quad_w = ((total_avail.x - 14.0) * 0.5).max(380.0);
        let quad_h = ((total_avail.y - 80.0) * 0.5).max(220.0);

        ui.horizontal(|ui| {
            // Panel 1: 2D Spatio-Temporal Wavefront Canvas
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("2D Spatio-Temporal Wavefront & Deflection Canvas")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(120, 200, 255)),
                );
                self.render_wavefront_canvas(ui, quad_w, quad_h - 22.0);
            });

            ui.separator();

            // Panel 2: Floquet Harmonic Spectrum Bar Chart
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Floquet Harmonic Spectrum |R_n|^2 (dB)")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(100, 240, 210)),
                );
                self.render_harmonic_spectrum(ui, quad_w, quad_h - 22.0);
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            // Panel 3: Non-Reciprocal Transmission Isolation Plot
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Non-Reciprocal Transmission Isolation S_21 vs S_12 (dB)")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(255, 180, 100)),
                );
                self.render_isolation_plot(ui, quad_w, quad_h - 22.0);
            });

            ui.separator();

            // Panel 4: OAM Vortex Beam Helical Phase Map
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Acoustic OAM Vortex Beam Helical Phase Map")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(200, 140, 255)),
                );
                self.render_oam_phase_map(ui, quad_w, quad_h - 22.0);
            });
        });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        // 3. Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders Panel 1: 2D Spatial acoustic pressure field Re[p(x, y, t)] and dynamic metasurface boundary.
    fn render_wavefront_canvas(&self, ui: &mut Ui, w: f32, h: f32) {
        let (response, painter) = ui.allocate_painter(vec2(w, h), Sense::hover());
        let rect = response.rect;

        // Background dark card
        painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 20, 30));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(35, 50, 75)),
            StrokeKind::Inside,
        );

        let y_boundary = rect.min.y + h * 0.75;
        let x_min = rect.min.x + 10.0;
        let x_max = rect.max.x - 10.0;
        let metasurface_len = x_max - x_min;

        // Draw Metasurface Boundary (y = y_boundary) with modulating phase unit cells
        let num_elements = 24;
        let elem_dx = metasurface_len / num_elements as f32;
        let t = self.current_time_s;
        let omega_m = self.modulation_freq_hz;
        let g_x = self.phase_gradient_rad_m;

        for i in 0..num_elements {
            let elem_x = x_min + (i as f32) * elem_dx;
            let norm_x = (i as f64) * self.unit_cell_pitch_m;
            let phase = (g_x * norm_x - 2.0 * PI * omega_m * t).rem_euclid(2.0 * PI);
            let hue = (phase / (2.0 * PI)) as f32;
            let color = hsv_to_rgb(hue, 0.85, 0.95);

            let elem_rect = Rect::from_min_max(
                pos2(elem_x + 1.0, y_boundary - 4.0),
                pos2(elem_x + elem_dx - 1.0, y_boundary + 8.0),
            );
            painter.rect_filled(elem_rect, 2.0, color);
        }

        // Draw traveling wave direction indicator
        let arrow_y = y_boundary + 14.0;
        painter.line_segment(
            [pos2(x_min, arrow_y), pos2(x_max, arrow_y)],
            Stroke::new(1.5, Color32::from_rgb(80, 160, 240)),
        );
        painter.text(
            pos2(x_min + metasurface_len * 0.5, arrow_y + 10.0),
            egui::Align2::CENTER_CENTER,
            format!(
                "Traveling Modulation v_m = {:.1} m/s (g_x = {:.0} rad/m, Omega_m = {:.0} Hz)",
                if g_x.abs() > 1e-6 { (2.0 * PI * omega_m) / g_x } else { 0.0 },
                g_x,
                omega_m
            ),
            FontId::proportional(10.0),
            Color32::from_rgb(130, 180, 230),
        );

        // Draw Incident Wavefront Beam (propagating down toward metasurface)
        let center_x = rect.min.x + w * 0.5;
        let theta_inc = self.incident_angle_deg.to_radians();
        let k_x_inc = theta_inc.sin();
        let k_y_inc = theta_inc.cos();

        let num_wavefronts = 6;
        for j in 0..num_wavefronts {
            let wave_phase = (j as f64 * 0.4 - self.carrier_freq_hz * 0.0005 * t).rem_euclid(1.0);
            let dist = (wave_phase as f32) * (y_boundary - rect.min.y - 20.0);
            let p_center = pos2(
                center_x - (dist * k_x_inc as f32 * 0.6),
                y_boundary - dist,
            );

            let half_len = 35.0;
            let normal_x = -k_y_inc as f32;
            let normal_y = k_x_inc as f32;
            let p1 = pos2(p_center.x - normal_x * half_len, p_center.y - normal_y * half_len);
            let p2 = pos2(p_center.x + normal_x * half_len, p_center.y + normal_y * half_len);

            painter.line_segment(
                [p1, p2],
                Stroke::new(2.0, Color32::from_rgba_unmultiplied(80, 190, 255, 180)),
            );
        }

        // Draw Incident Beam Arrow
        let arr_start = pos2(center_x - 50.0 * k_x_inc as f32, y_boundary - 90.0);
        let arr_end = pos2(center_x, y_boundary - 10.0);
        painter.line_segment(
            [arr_start, arr_end],
            Stroke::new(2.5, Color32::from_rgb(100, 220, 255)),
        );
        painter.text(
            pos2(arr_start.x - 10.0, arr_start.y - 8.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("Incident (theta = {:.0} deg, {:.1} kHz)", self.incident_angle_deg, self.carrier_freq_hz * 1e-3),
            FontId::proportional(10.0),
            Color32::from_rgb(120, 210, 255),
        );

        // Draw Deflected Reflected Beam (n = +1 Doppler sideband)
        if let Some(angle_deg) = self.steering_angle_deg {
            let theta_refl = angle_deg.to_radians();
            let k_x_refl = theta_refl.sin() as f32;
            let k_y_refl = theta_refl.cos() as f32;

            for j in 0..num_wavefronts {
                let wave_phase = (j as f64 * 0.4 - (self.carrier_freq_hz + omega_m) * 0.0005 * t).rem_euclid(1.0);
                let dist = (wave_phase as f32) * (y_boundary - rect.min.y - 20.0);
                let p_center = pos2(
                    center_x + (dist * k_x_refl * 0.6),
                    y_boundary - dist,
                );

                let half_len = 35.0;
                let normal_x = k_y_refl;
                let normal_y = -k_x_refl;
                let p1 = pos2(p_center.x - normal_x * half_len, p_center.y - normal_y * half_len);
                let p2 = pos2(p_center.x + normal_x * half_len, p_center.y + normal_y * half_len);

                painter.line_segment(
                    [p1, p2],
                    Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 180, 50, 180)),
                );
            }

            let refl_start = pos2(center_x, y_boundary - 10.0);
            let refl_end = pos2(center_x + 60.0 * k_x_refl, y_boundary - 90.0);
            painter.line_segment(
                [refl_start, refl_end],
                Stroke::new(2.5, Color32::from_rgb(255, 180, 50)),
            );
            painter.text(
                pos2(refl_end.x + 10.0, refl_end.y - 8.0),
                egui::Align2::LEFT_BOTTOM,
                format!("Deflected n=+1 (theta = {:.1} deg, {:.1} kHz)", angle_deg, (self.carrier_freq_hz + omega_m) * 1e-3),
                FontId::proportional(10.0),
                Color32::from_rgb(255, 190, 80),
            );
        } else {
            // Mode is evanescent, trapped near surface
            painter.text(
                pos2(center_x + 40.0, y_boundary - 40.0),
                egui::Align2::LEFT_CENTER,
                "Evanescent Bound Mode (|k_x/k| > 1.0)",
                FontId::proportional(11.0),
                Color32::from_rgb(255, 90, 90),
            );
        }
    }

    /// Renders Panel 2: Floquet harmonic reflection power spectrum bar chart in dB.
    fn render_harmonic_spectrum(&self, ui: &mut Ui, w: f32, h: f32) {
        let bars: Vec<Bar> = self
            .sidebands
            .iter()
            .map(|sb| {
                let n = sb.n;
                let p_db = sb.power_refl_db.clamp(-50.0, 0.0);
                let height = p_db + 50.0; // Render relative to -50 dB floor

                let color = if n == 1 {
                    Color32::from_rgb(50, 220, 160) // Dominant n=+1 sideband
                } else if n == 0 {
                    Color32::from_rgb(80, 160, 240) // Specular n=0
                } else {
                    Color32::from_rgb(220, 120, 100) // Other sidebands
                };

                Bar::new(n as f64, height)
                    .width(0.65)
                    .fill(color)
            })
            .collect();

        let chart = BarChart::new("floquet_bars", bars);

        Plot::new("floquet_spectrum_plot")
            .width(w)
            .height(h)
            .show_axes([true, true])
            .show_grid([true, true])
            .legend(Legend::default().position(egui_plot::Corner::RightTop))
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
                // Reference line for 0 dB
                plot_ui.hline(
                    HLine::new("0 dB Full Reflection", 50.0)
                        .stroke(Stroke::new(1.0, Color32::from_rgb(180, 180, 180))),
                );
                // Reference line for -20 dB
                plot_ui.hline(
                    HLine::new("-20 dB Threshold", 30.0)
                        .stroke(Stroke::new(1.0, Color32::from_rgb(100, 100, 120))),
                );
            });
    }

    /// Renders Panel 3: Non-reciprocal transmission isolation S_21(f) vs S_12(f) plot in dB.
    fn render_isolation_plot(&self, ui: &mut Ui, w: f32, h: f32) {
        let pts_s21: PlotPoints = PlotPoints::from(self.s21_curve.clone());
        let pts_s12: PlotPoints = PlotPoints::from(self.s12_curve.clone());

        Plot::new("floquet_isolation_plot")
            .width(w)
            .height(h)
            .show_axes([true, true])
            .show_grid([true, true])
            .legend(Legend::default().position(egui_plot::Corner::LeftBottom))
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Forward S_21", pts_s21)
                        .color(Color32::from_rgb(80, 200, 255))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("Backward S_12", pts_s12)
                        .color(Color32::from_rgb(255, 100, 80))
                        .width(2.0),
                );
                // Isolation target line >= 30 dB reference
                plot_ui.hline(
                    HLine::new("Isolation Floor <= -30 dB", -30.0)
                        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 220, 120))),
                );
            });
    }

    /// Renders Panel 4: 2D polar phase map of helical vortex beam with Delta_l winding arms.
    fn render_oam_phase_map(&self, ui: &mut Ui, w: f32, h: f32) {
        let (response, painter) = ui.allocate_painter(vec2(w, h), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 18, 28));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(40, 50, 75)),
            StrokeKind::Inside,
        );

        let center = rect.center();
        let max_radius = (w.min(h) * 0.42).max(40.0);
        let num_rings = self.polar_map.r_steps;
        let num_sectors = self.polar_map.theta_steps;

        let dr = max_radius / num_rings as f32;
        let d_th = 2.0 * PI / num_sectors as f64;

        // Draw discrete polar sectors colored by phase
        for r_idx in 0..num_rings {
            let r_inner = (r_idx as f32) * dr;
            let r_outer = ((r_idx + 1) as f32) * dr;

            for th_idx in 0..num_sectors {
                let th_start = (th_idx as f64) * d_th;
                let th_mid = th_start + 0.5 * d_th;
                let phase = self.polar_map.at(r_idx, th_idx);

                let hue = (phase / (2.0 * PI)) as f32;
                let color = hsv_to_rgb(hue, 0.90, 0.90);

                // Draw sector arc approximation using small polygon quad
                let cos_s = th_start.cos() as f32;
                let sin_s = th_start.sin() as f32;
                let cos_e = (th_start + d_th).cos() as f32;
                let sin_e = (th_start + d_th).sin() as f32;

                let p1 = pos2(center.x + r_inner * cos_s, center.y + r_inner * sin_s);
                let p2 = pos2(center.x + r_outer * cos_s, center.y + r_outer * sin_s);
                let p3 = pos2(center.x + r_outer * cos_e, center.y + r_outer * sin_e);
                let p4 = pos2(center.x + r_inner * cos_e, center.y + r_inner * sin_e);

                painter.add(egui::Shape::convex_polygon(
                    vec![p1, p2, p3, p4],
                    color,
                    Stroke::NONE,
                ));

                // Draw radial contour overlay
                if r_idx == num_rings - 1 {
                    let outer_p = pos2(center.x + max_radius * cos_s, center.y + max_radius * sin_s);
                    painter.line_segment(
                        [center, outer_p],
                        Stroke::new(0.5, Color32::from_rgba_unmultiplied(200, 200, 200, 20)),
                    );
                }
                let _ = th_mid;
            }
        }

        // Singularity core marker
        painter.circle_filled(center, 4.0, Color32::from_rgb(20, 20, 30));
        painter.circle_stroke(center, 4.0, Stroke::new(1.5, Color32::WHITE));

        // Legend overlay
        painter.text(
            pos2(rect.min.x + 8.0, rect.min.y + 8.0),
            egui::Align2::LEFT_TOP,
            format!("Target OAM l_out = {:+} | Delta_l = {:+}", self.oam.l_out, self.oam.delta_l),
            FontId::proportional(11.0),
            Color32::from_rgb(220, 180, 255),
        );
        painter.text(
            pos2(rect.min.x + 8.0, rect.max.y - 10.0),
            egui::Align2::LEFT_BOTTOM,
            format!("Purity: {:.1}% | Suppression: {:.1} dB", self.oam_purity_pct, self.oam.sideband_suppression_db),
            FontId::proportional(10.0),
            Color32::from_rgb(160, 220, 180),
        );
    }

    /// Renders the Telemetry Footer with physical performance metrics.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            // Steering Angle
            ui.label(RichText::new("Steering Angle:").strong().size(11.0));
            let steer_str = match self.steering_angle_deg {
                Some(ang) => format!("{:.1} deg", ang),
                None => "Evanescent".to_string(),
            };
            ui.label(
                RichText::new(steer_str)
                    .color(Color32::from_rgb(255, 200, 100))
                    .strong(),
            );

            ui.separator();

            // Floquet Frequency Shift
            ui.label(RichText::new("Floquet Shift Delta f:").strong().size(11.0));
            ui.label(
                RichText::new(format!("{:+.0} Hz", self.floquet_freq_shift_hz))
                    .color(Color32::from_rgb(100, 230, 255))
                    .strong(),
            );

            ui.separator();

            // Non-Reciprocal Isolation
            ui.label(RichText::new("Non-Reciprocal Isolation:").strong().size(11.0));
            let iso_color = if self.isolation_db >= 30.0 {
                Color32::from_rgb(60, 220, 120)
            } else {
                Color32::from_rgb(240, 90, 90)
            };
            ui.label(
                RichText::new(format!("{:.1} dB", self.isolation_db))
                    .color(iso_color)
                    .strong(),
            );

            ui.separator();

            // OAM Mode Purity
            ui.label(RichText::new("OAM Mode Purity:").strong().size(11.0));
            let purity_color = if self.oam_purity_pct >= 95.0 {
                Color32::from_rgb(60, 220, 120)
            } else {
                Color32::from_rgb(240, 90, 90)
            };
            ui.label(
                RichText::new(format!("{:.1}%", self.oam_purity_pct))
                    .color(purity_color)
                    .strong(),
            );

            ui.separator();

            // Synthetic Gauge Field B_eff
            ui.label(RichText::new("Gauge Field B_eff:").strong().size(11.0));
            ui.label(
                RichText::new(format!("{:.2} s/m^2", self.b_eff))
                    .color(Color32::from_rgb(200, 150, 255))
                    .strong(),
            );

            ui.separator();

            // Dominant Sideband Efficiency
            ui.label(RichText::new("Dominant Sideband Efficiency:").strong().size(11.0));
            ui.label(
                RichText::new(format!("{:.1}%", self.dominant_efficiency_pct))
                    .color(Color32::from_rgb(255, 220, 100))
                    .strong(),
            );
        });
    }
}

/// Helper function to convert HSV values (h in [0, 1], s in [0, 1], v in [0, 1]) to egui Color32.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color32 {
    let h_clamped = h.fract();
    let i = (h_clamped * 6.0).floor() as i32;
    let f = h_clamped * 6.0 - (i as f32);
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    Color32::from_rgb(
        (r * 255.0).clamp(0.0, 255.0) as u8,
        (g * 255.0).clamp(0.0, 255.0) as u8,
        (b * 255.0).clamp(0.0, 255.0) as u8,
    )
}
