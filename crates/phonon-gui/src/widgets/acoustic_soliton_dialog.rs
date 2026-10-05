#![deny(unsafe_code)]

//! Interactive Acoustic Domain Wall Soliton & Waveguide Studio Dialog.
//!
//! Provides:
//! - 1D Solitary Wave Profile & Energy Density Plot: native egui_plot of phase field phi(x)
//!   and Hamiltonian energy density H(x), displaying localized domain walls (kinks, antikinks, breathers).
//! - 2D Waveguide Acoustic Pressure Canvas: interactive 2D spatial canvas displaying the domain wall channel
//!   with self-trapped acoustic wave packets, S-bend curvature, and obstacle diffraction.
//! - S-Parameter Transmission Spectrum: native egui_plot of S21(f) and S11(f) in dB.
//! - Controls & Presets: Velocity ratio slider, characteristic resonance omega_0, damping gamma,
//!   waveguide S-bend offset, and defect obstacle toggle.
//! - Telemetry Footer: Topological Charge Q, Energy Drift dE/E0, Transverse Confinement Gamma (%),
//!   Forward Transmission S21 (dB), Defect Immunity Ratio (%).

use egui::{
    pos2, vec2, Color32, FontId, Rect, RichText, Sense, Stroke, Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints};
use phonon_solver::acoustic_domain_wall_soliton::{
    DomainWallWaveguideParams, DomainWallWaveguideRouter, SineGordonParams, SineGordonSolver,
    SolitonKind,
};
use std::f64::consts::PI;

/// Palette colors for the acoustic domain wall soliton studio.
const COLOR_PHASE_PHI: Color32 = Color32::from_rgb(56, 189, 248); // Sky Blue
const COLOR_ENERGY_DENSITY: Color32 = Color32::from_rgb(251, 146, 60); // Amber Orange
const COLOR_WAVEGUIDE_WALL: Color32 = Color32::from_rgb(168, 85, 247); // Violet
const COLOR_PRESSURE_POS: Color32 = Color32::from_rgb(239, 68, 68); // Red (+P)
const COLOR_PRESSURE_NEG: Color32 = Color32::from_rgb(59, 130, 246); // Blue (-P)
const COLOR_DEFECT_OBSTACLE: Color32 = Color32::from_rgb(249, 115, 22); // Orange
const COLOR_CONSERVATIVE: Color32 = Color32::from_rgb(34, 197, 94); // Emerald Green
const COLOR_TRANSMISSION: Color32 = Color32::from_rgb(52, 211, 153); // Emerald
const COLOR_REFLECTION: Color32 = Color32::from_rgb(244, 63, 94); // Rose

/// Active view tab in the domain wall soliton dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolitonDialogTab {
    /// 1D Phase Field phi(x) and Hamiltonian Energy Density H(x).
    PhaseAndEnergyProfile,
    /// 2D Trapped Waveguide Acoustic Pressure Field Canvas.
    Waveguide2DField,
    /// Scattering Parameters (S21, S11, Isolation).
    SParameters,
}

/// Interactive modal dialog for the Non-Linear Acoustic Domain Wall Kink & Soliton Waveguide Studio.
#[derive(Debug, Clone)]
pub struct AcousticSolitonDialog {
    /// Window visibility toggle.
    pub is_open: bool,
    /// Current selected view tab.
    pub active_tab: SolitonDialogTab,

    // Solver engines
    /// 1D Sine-Gordon solver parameters.
    pub sg_params: SineGordonParams,
    /// 1D Sine-Gordon numerical solver.
    pub sg_solver: SineGordonSolver,

    /// 2D Domain wall waveguide router parameters.
    pub wg_params: DomainWallWaveguideParams,
    /// 2D Domain wall waveguide router engine.
    pub wg_router: DomainWallWaveguideRouter,

    // Control slider buffers
    pub velocity_ratio: f64,
    pub omega_0_khz: f64,
    pub speed_of_sound: f64,
    pub damping_gamma: f64,
    pub bend_offset_mm: f64,
    pub has_defect: bool,

    /// Stepping count accumulator.
    pub total_steps_executed: usize,
}

impl Default for AcousticSolitonDialog {
    fn default() -> Self {
        let sg_params = SineGordonParams {
            grid_points: 256,
            length_m: 0.06,
            speed_of_sound: 1500.0,
            omega_0: 2.0 * PI * 40_000.0,
            damping_gamma: 0.0,
            cfl_factor: 0.35,
        };

        let initial_kind = SolitonKind::MovingKink {
            x0: -0.01,
            velocity_ratio: 0.25,
        };

        let sg_solver = SineGordonSolver::new(sg_params.clone(), initial_kind);

        let wg_params = DomainWallWaveguideParams {
            length_m: 0.04,
            width_m: 0.02,
            nx: 70,
            ny: 45,
            speed_of_sound: 1500.0,
            carrier_frequency_hz: 120_000.0,
            wall_thickness_m: 0.0015,
            bend_offset_m: 0.003,
            has_defect_obstacle: false,
            defect_radius_m: 0.0012,
        };

        let wg_router = DomainWallWaveguideRouter::new(wg_params.clone());

        Self {
            is_open: false,
            active_tab: SolitonDialogTab::PhaseAndEnergyProfile,
            sg_params,
            sg_solver,
            wg_params,
            wg_router,
            velocity_ratio: 0.25,
            omega_0_khz: 40.0,
            speed_of_sound: 1500.0,
            damping_gamma: 0.0,
            bend_offset_mm: 3.0,
            has_defect: false,
            total_steps_executed: 0,
        }
    }
}

impl AcousticSolitonDialog {
    /// Construct a new dialog with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Primary modal UI entry point.
    pub fn ui(&mut self, ctx: &egui::Context) {
        self.show(ctx);
    }

    /// Re-initialize 1D Sine-Gordon solver with current control values.
    pub fn reinit_soliton(&mut self, kind: SolitonKind) {
        self.sg_params.speed_of_sound = self.speed_of_sound;
        self.sg_params.omega_0 = 2.0 * PI * self.omega_0_khz * 1000.0;
        self.sg_params.damping_gamma = self.damping_gamma;
        self.sg_solver = SineGordonSolver::new(self.sg_params.clone(), kind);
        self.total_steps_executed = 0;
    }

    /// Re-solve 2D domain wall waveguide router with current parameters.
    pub fn update_waveguide(&mut self) {
        self.wg_params.speed_of_sound = self.speed_of_sound;
        self.wg_params.bend_offset_m = self.bend_offset_mm * 1e-3;
        self.wg_params.has_defect_obstacle = self.has_defect;
        self.wg_router = DomainWallWaveguideRouter::new(self.wg_params.clone());
    }

    /// Advance 1D Sine-Gordon simulation by N internal steps.
    pub fn step_simulation(&mut self, steps: usize) {
        self.sg_solver.step_n(steps);
        self.total_steps_executed += steps;
    }

    /// Main rendering entry point for the modal dialog.
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new(
            RichText::new("Acoustic Domain Wall Soliton & Waveguide Studio")
                .strong()
                .size(15.0),
        )
        .open(&mut is_open)
        .resizable(true)
        .default_width(820.0)
        .default_height(600.0)
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
                SolitonDialogTab::PhaseAndEnergyProfile,
                RichText::new("1D Soliton Dynamics").strong(),
            );
            ui.selectable_value(
                &mut self.active_tab,
                SolitonDialogTab::Waveguide2DField,
                RichText::new("2D Waveguide Field").strong(),
            );
            ui.selectable_value(
                &mut self.active_tab,
                SolitonDialogTab::SParameters,
                RichText::new("S-Parameters & Transmission").strong(),
            );
        });

        ui.separator();

        // Control Toolbar
        self.render_controls(ui);

        ui.separator();

        // Active Tab Visualization
        match self.active_tab {
            SolitonDialogTab::PhaseAndEnergyProfile => self.render_phase_and_energy_plot(ui),
            SolitonDialogTab::Waveguide2DField => self.render_waveguide_canvas(ui),
            SolitonDialogTab::SParameters => self.render_s_parameters_plot(ui),
        }

        ui.separator();

        // Telemetry Footer
        self.render_telemetry(ui);
    }

    /// Render preset selection and parameter sliders.
    fn render_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Presets:").strong());

            if ui
                .button("Moving Kink (Q = +1)")
                .on_hover_text("Initialize single moving topological kink")
                .clicked()
            {
                let kind = SolitonKind::MovingKink {
                    x0: -0.015,
                    velocity_ratio: self.velocity_ratio,
                };
                self.reinit_soliton(kind);
            }

            if ui
                .button("Moving Antikink (Q = -1)")
                .on_hover_text("Initialize single moving topological antikink")
                .clicked()
            {
                let kind = SolitonKind::MovingAntikink {
                    x0: 0.015,
                    velocity_ratio: -self.velocity_ratio,
                };
                self.reinit_soliton(kind);
            }

            if ui
                .button("Kink-Antikink Collision (Q = 0)")
                .on_hover_text("Initialize two colliding solitons with net zero charge")
                .clicked()
            {
                let kind = SolitonKind::KinkAntikinkCollision {
                    x_left: -0.015,
                    v_left_ratio: self.velocity_ratio,
                    x_right: 0.015,
                    v_right_ratio: -self.velocity_ratio,
                };
                self.reinit_soliton(kind);
            }

            if ui
                .button("Breather Soliton (Q = 0)")
                .on_hover_text("Initialize bound pulsating breather solitary wave")
                .clicked()
            {
                let kind = SolitonKind::Breather {
                    x0: 0.0,
                    frequency_ratio: 0.6,
                };
                self.reinit_soliton(kind);
            }

            ui.separator();

            if ui
                .button(RichText::new("Step 10 Cycles").strong())
                .on_hover_text("Advance simulation by 10 stable time steps")
                .clicked()
            {
                self.step_simulation(10);
            }

            if ui
                .button(RichText::new("Run Transient (50)").strong())
                .on_hover_text("Advance simulation by 50 stable time steps")
                .clicked()
            {
                self.step_simulation(50);
            }

            if ui.button("Reset").clicked() {
                let kind = self.sg_solver.state.kind.clone();
                self.reinit_soliton(kind);
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;

            ui.label("Velocity v/c0:");
            if ui
                .add(
                    egui::Slider::new(&mut self.velocity_ratio, 0.05..=0.85)
                        .step_by(0.05)
                        .text("u"),
                )
                .changed()
            {
                changed = true;
            }

            ui.label("Resonance w0:");
            if ui
                .add(
                    egui::Slider::new(&mut self.omega_0_khz, 10.0..=100.0)
                        .step_by(5.0)
                        .text("kHz"),
                )
                .changed()
            {
                changed = true;
            }

            ui.label("Damping gamma:");
            if ui
                .add(
                    egui::Slider::new(&mut self.damping_gamma, 0.0..=2000.0)
                        .step_by(100.0)
                        .text("1/s"),
                )
                .changed()
            {
                changed = true;
            }

            if changed {
                let kind = self.sg_solver.state.kind.clone();
                self.reinit_soliton(kind);
            }
        });

        ui.horizontal(|ui| {
            let mut wg_changed = false;

            ui.label("Waveguide S-Bend:");
            if ui
                .add(
                    egui::Slider::new(&mut self.bend_offset_mm, 0.0..=8.0)
                        .step_by(0.5)
                        .text("mm"),
                )
                .changed()
            {
                wg_changed = true;
            }

            if ui
                .checkbox(&mut self.has_defect, "Defect Obstacle")
                .on_hover_text("Insert localized rigid scattering defect obstacle in channel")
                .changed()
            {
                wg_changed = true;
            }

            if wg_changed {
                self.update_waveguide();
            }
        });
    }

    /// Render 1D phase field phi(x) and Hamiltonian energy density plot.
    fn render_phase_and_energy_plot(&self, ui: &mut Ui) {
        let n = self.sg_solver.params.grid_points;
        let mut phi_points = Vec::with_capacity(n);
        let mut energy_points = Vec::with_capacity(n);

        let max_energy = self
            .sg_solver
            .state
            .energy_density
            .iter()
            .copied()
            .fold(1e-12_f64, f64::max);

        for i in 0..n {
            let x_mm = self.sg_solver.state.x[i] * 1000.0;
            let phi_val = self.sg_solver.state.phi[i];
            let norm_energy = (self.sg_solver.state.energy_density[i] / max_energy) * (2.0 * PI);

            phi_points.push([x_mm, phi_val]);
            energy_points.push([x_mm, norm_energy]);
        }

        let phi_line = Line::new("Phase Field phi(x) [rad]", PlotPoints::new(phi_points))
            .color(COLOR_PHASE_PHI)
            .width(2.5);

        let energy_line = Line::new("Energy Density H(x) [scaled]", PlotPoints::new(energy_points))
            .color(COLOR_ENERGY_DENSITY)
            .width(2.0);

        Plot::new("soliton_phase_energy_plot")
            .legend(Legend::default())
            .height(300.0)
            .x_axis_label("Position x (mm)")
            .y_axis_label("Phase / Energy Amplitude")
            .show(ui, |plot_ui| {
                plot_ui.hline(
                    HLine::new("2*pi Bound", 2.0 * PI)
                        .color(Color32::from_gray(100)),
                );
                plot_ui.hline(
                    HLine::new("Zero", 0.0)
                        .color(Color32::from_gray(80)),
                );
                plot_ui.line(phi_line);
                plot_ui.line(energy_line);
            });
    }

    /// Render 2D Waveguide Acoustic Pressure Field Canvas.
    fn render_waveguide_canvas(&self, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(
            vec2(ui.available_width().max(300.0), 280.0),
            Sense::hover(),
        );

        let rect = response.rect;
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42)); // Slate background

        let nx = self.wg_router.params.nx;
        let ny = self.wg_router.params.ny;

        if nx < 2 || ny < 2 {
            return;
        }

        let cell_w = rect.width() / nx as f32;
        let cell_h = rect.height() / ny as f32;

        // Render pressure heatmap
        for iy in 0..ny {
            for ix in 0..nx {
                let p = self.wg_router.pressure_field[iy][ix];
                let color = if p >= 0.0 {
                    let alpha = (p.clamp(0.0, 1.0) * 220.0) as u8;
                    Color32::from_rgba_unmultiplied(
                        COLOR_PRESSURE_POS.r(),
                        COLOR_PRESSURE_POS.g(),
                        COLOR_PRESSURE_POS.b(),
                        alpha,
                    )
                } else {
                    let alpha = ((-p).clamp(0.0, 1.0) * 220.0) as u8;
                    Color32::from_rgba_unmultiplied(
                        COLOR_PRESSURE_NEG.r(),
                        COLOR_PRESSURE_NEG.g(),
                        COLOR_PRESSURE_NEG.b(),
                        alpha,
                    )
                };

                let px = rect.min.x + ix as f32 * cell_w as f32;
                let py = rect.min.y + iy as f32 * cell_h as f32;
                let cell_rect = Rect::from_min_size(
                    pos2(px, py),
                    vec2(cell_w as f32 + 0.5, cell_h as f32 + 0.5),
                );
                painter.rect_filled(cell_rect, 0.0, color);
            }
        }

        // Draw domain wall centerline
        let mut wall_points = Vec::with_capacity(nx);
        for ix in 0..nx {
            let px = rect.min.x + (ix as f32 + 0.5) * cell_w as f32;
            let y_dw = self.wg_router.wall_center_y[ix];
            let y_norm = y_dw / (self.wg_router.params.width_m * 0.5); // [-1, 1]
            let py = rect.center().y - (y_norm as f32) * (rect.height() * 0.45);
            wall_points.push(pos2(px, py));
        }

        if wall_points.len() >= 2 {
            for i in 0..wall_points.len() - 1 {
                painter.line_segment(
                    [wall_points[i], wall_points[i + 1]],
                    Stroke::new(2.0, COLOR_WAVEGUIDE_WALL),
                );
            }
        }

        // Draw defect obstacle if present
        if self.wg_router.params.has_defect_obstacle {
            let center_x = rect.center().x;
            let center_y = rect.center().y;
            let r_px = (self.wg_router.params.defect_radius_m
                / self.wg_router.params.width_m
                * rect.height() as f64) as f32;
            painter.circle_filled(pos2(center_x, center_y), r_px.max(4.0), COLOR_DEFECT_OBSTACLE);
            painter.circle_stroke(
                pos2(center_x, center_y),
                r_px.max(4.0),
                Stroke::new(1.5, Color32::WHITE),
            );
        }

        // Scale bar & legend overlay
        painter.text(
            pos2(rect.min.x + 12.0, rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            "Domain Wall Acoustic Waveguide Channel (Trapped Bound Mode)",
            FontId::proportional(12.0),
            Color32::from_gray(220),
        );
    }

    /// Render S-parameter transmission and reflection plot.
    fn render_s_parameters_plot(&self, ui: &mut Ui) {
        let f0 = self.wg_router.params.carrier_frequency_hz / 1000.0;
        let s21_val = self.wg_router.s_parameters.s21_db;
        let s11_val = self.wg_router.s_parameters.s11_db;
        let iso_val = self.wg_router.s_parameters.isolation_db;

        let num_points = 50;
        let mut s21_points = Vec::with_capacity(num_points);
        let mut s11_points = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let f = f0 - 25.0 + (i as f64 / (num_points - 1) as f64) * 50.0;
            let df = ((f - f0) / 15.0).abs();
            // Dispersive dip around detuned band
            let s21_f = s21_val - df * df * 0.8;
            let s11_f = (s11_val + df * 4.0).min(-10.0);

            s21_points.push([f, s21_f]);
            s11_points.push([f, s11_f]);
        }

        let s21_line = Line::new("S21 Transmission (dB)", PlotPoints::new(s21_points))
            .color(COLOR_TRANSMISSION)
            .width(2.5);

        let s11_line = Line::new("S11 Return Loss (dB)", PlotPoints::new(s11_points))
            .color(COLOR_REFLECTION)
            .width(2.0);

        Plot::new("soliton_s_params_plot")
            .legend(Legend::default())
            .height(280.0)
            .x_axis_label("Frequency (kHz)")
            .y_axis_label("Scattering Parameter (dB)")
            .show(ui, |plot_ui| {
                plot_ui.hline(
                    HLine::new("0 dB Reference", 0.0)
                        .color(Color32::from_gray(80)),
                );
                plot_ui.line(s21_line);
                plot_ui.line(s11_line);
            });

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "Bulk Isolation: {:.1} dB | Confinement Gamma: {:.1}% | Defect Immunity: {:.1}%",
                    iso_val,
                    self.wg_router.s_parameters.confinement_factor * 100.0,
                    self.wg_router.s_parameters.defect_immunity_ratio * 100.0
                ))
                .size(11.0)
                .color(Color32::from_gray(180)),
            );
        });
    }

    /// Render real-time telemetry footer.
    fn render_telemetry(&self, ui: &mut Ui) {
        let q = self.sg_solver.state.topological_charge;
        let drift_pct = self.sg_solver.relative_energy_drift() * 100.0;
        let total_e = self.sg_solver.state.total_energy;
        let s21_db = self.wg_router.s_parameters.s21_db;
        let confinement_pct = self.wg_router.s_parameters.confinement_factor * 100.0;

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Topological Charge Q:").strong());
            let q_color = if q.abs() < 0.1 {
                Color32::from_rgb(148, 163, 184)
            } else if q > 0.5 {
                COLOR_CONSERVATIVE
            } else {
                COLOR_REFLECTION
            };
            ui.colored_label(q_color, RichText::new(format!("{:.3}", q)).strong());

            ui.separator();

            ui.label(RichText::new("Energy Drift:").strong());
            let drift_color = if drift_pct < 0.5 {
                COLOR_CONSERVATIVE
            } else if drift_pct < 2.0 {
                Color32::from_rgb(250, 204, 21)
            } else {
                COLOR_REFLECTION
            };
            ui.colored_label(
                drift_color,
                RichText::new(format!("{:.3}%", drift_pct)).strong(),
            );

            ui.separator();

            ui.label(RichText::new("Total Energy:").strong());
            ui.label(format!("{:.2e} J", total_e));

            ui.separator();

            ui.label(RichText::new("Waveguide Confinement:").strong());
            ui.colored_label(
                COLOR_CONSERVATIVE,
                RichText::new(format!("{:.1}%", confinement_pct)).strong(),
            );

            ui.separator();

            ui.label(RichText::new("S21 Transmission:").strong());
            ui.colored_label(
                COLOR_TRANSMISSION,
                RichText::new(format!("{:.2} dB", s21_db)).strong(),
            );

            ui.separator();

            ui.label(RichText::new("Steps:").strong());
            ui.label(format!("{}", self.total_steps_executed));
        });
    }
}
