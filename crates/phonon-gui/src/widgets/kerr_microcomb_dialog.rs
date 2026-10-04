#![deny(unsafe_code)]

//! Interactive Kerr Microcomb Studio Visualizer for Phonon CAD Studio.
//!
//! Provides:
//! - 2D Microresonator Ring Cavity Field Canvas: circulating acoustic/optical field intensity
//!   |psi(theta)|^2 around ring perimeter with circulating bright dissipative soliton pulse packet.
//! - Optical Spectrum Analyzer (OSA) Plot: native egui_plot bar chart rendering comb line powers
//!   S(mu) in dBm across relative mode numbers mu, highlighting pump line (mu=0) and bandwidth span.
//! - Detuning Hysteresis Scan Curve: plots intracavity power vs detuning alpha, displaying
//!   the characteristic soliton step plateau.
//! - Temporal Soliton Pulse Profile: plots |psi(theta)|^2 vs azimuthal angle theta [-pi, pi],
//!   displaying the sharp sech^2 bright soliton pulse and analytical fit.
//! - Controls: Pump power slider F_0^2, detuning alpha slider, dispersion D_2 slider,
//!   "Sweep Detuning" button, "Single Soliton" preset, "Turing Roll" preset.
//! - Telemetry Footer: Microcomb State, Comb Line Count, 3-dB Bandwidth (GHz),
//!   Pulse FWHM (ps), Repetition Rate f_rep (MHz), Intracavity Power (mW).

use egui::{
    vec2, Color32, FontId, Pos2, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{Bar, BarChart, HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::kerr_microcomb::{
    CombSpectrum, DetuningScanResult, LleSplitStepSolver, MicrocombRegime, MicroresonatorParams,
};
use std::f64::consts::PI;

/// Palette colors for Kerr microcomb studio.
const COLOR_SOLITON_EMERALD: Color32 = Color32::from_rgb(52, 211, 153);
const COLOR_COMB_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const COLOR_PUMP_GOLD: Color32 = Color32::from_rgb(250, 204, 21);
const COLOR_CHAOS_AMBER: Color32 = Color32::from_rgb(249, 115, 22);
const COLOR_CW_SLATE: Color32 = Color32::from_rgb(148, 163, 184);
const COLOR_CAVITY_BG: Color32 = Color32::from_rgb(15, 23, 42);
const COLOR_RING_BORDER: Color32 = Color32::from_rgb(51, 65, 85);
const COLOR_BUS_WAVEGUIDE: Color32 = Color32::from_rgb(71, 85, 105);

/// Modal dialog for the Phonon Non-Linear Soliton Kerr Microcomb Studio.
#[derive(Debug, Clone, PartialEq)]
pub struct KerrMicrocombDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    /// LLE split-step Fourier numerical solver.
    pub solver: LleSplitStepSolver,
    /// Cached comb spectrum and pulse metrics.
    pub spectrum: CombSpectrum,
    /// Cached detuning hysteresis scan result.
    pub scan_result: Option<DetuningScanResult>,

    // Controls
    /// Pump detuning parameter alpha.
    pub detuning_alpha: f64,
    /// Pump drive power F_0^2.
    pub pump_power_f2: f64,
    /// Second-order chromatic dispersion D_2 in kHz.
    pub dispersion_d2_khz: f64,

    // Cached Plot Curves
    pub temporal_curve: Vec<[f64; 2]>,
    pub sech2_curve: Vec<[f64; 2]>,
    pub osa_bars: Vec<(i32, f64)>,
    pub scan_curve: Vec<[f64; 2]>,

    // Animation phase for circulating wave packet
    pub anim_phase: f32,
    pub status_msg: String,
}

impl Default for KerrMicrocombDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl KerrMicrocombDialog {
    /// Creates a new `KerrMicrocombDialog` initialized with single soliton parameters.
    pub fn new() -> Self {
        let params = MicroresonatorParams::default_bright_soliton();
        let detuning_alpha = params.alpha;
        let pump_power_f2 = params.f_squared();
        let dispersion_d2_khz = params.d2 / (2.0 * PI * 1000.0);

        let mut solver = LleSplitStepSolver::new(params);
        solver.init_soliton(0.0);

        let spectrum = CombSpectrum::from_state(&solver.state, &solver.params, 1.0);

        // Pre-seeded baseline detuning scan curve for fast sub-millisecond cold boot
        let alphas: Vec<f64> = (-5..=35).map(|i| -1.0 + (i as f64) * 0.2).collect();
        let powers: Vec<f64> = alphas.iter().map(|&a| if (2.0..=5.0).contains(&a) { 1.2 } else { 0.2 }).collect();
        let peak_powers: Vec<f64> = alphas.iter().map(|&a| if (2.0..=5.0).contains(&a) { 4.5 } else { 0.8 }).collect();
        let regimes: Vec<MicrocombRegime> = alphas.iter().map(|&a| if (2.0..=5.0).contains(&a) { MicrocombRegime::DissipativeSoliton } else { MicrocombRegime::LowPowerCw }).collect();
        let scan = DetuningScanResult {
            alphas,
            powers,
            peak_powers,
            regimes,
            soliton_step_range: Some((2.0, 5.0)),
            turing_roll_range: Some((1.0, 2.0)),
            chaos_range: Some((5.0, 6.0)),
            best_soliton_state: None,
        };

        let temporal_curve: Vec<[f64; 2]> = spectrum
            .theta
            .iter()
            .zip(spectrum.intensity.iter())
            .map(|(&th, &i)| [th, i])
            .collect();

        let sech2_curve: Vec<[f64; 2]> = spectrum
            .theta
            .iter()
            .zip(spectrum.sech2_fit.iter())
            .map(|(&th, &i)| [th, i])
            .collect();

        let osa_bars: Vec<(i32, f64)> = spectrum
            .mu
            .iter()
            .zip(spectrum.powers_dbm.iter())
            .map(|(&m, &p)| (m, p))
            .collect();

        let scan_curve: Vec<[f64; 2]> = scan
            .alphas
            .iter()
            .zip(scan.powers.iter())
            .map(|(&a, &p)| [a, p * 10.0])
            .collect();

        Self {
            is_open: false,
            solver,
            spectrum,
            scan_result: Some(scan),
            detuning_alpha,
            pump_power_f2,
            dispersion_d2_khz,
            temporal_curve,
            sech2_curve,
            osa_bars,
            scan_curve,
            anim_phase: 0.0,
            status_msg: "Dissipative Kerr single soliton mode locked. Sech^2 pulse established.".to_string(),
        }
    }

    /// Applies the single dissipative Kerr soliton preset.
    pub fn apply_single_soliton_preset(&mut self) {
        self.detuning_alpha = 3.5;
        self.pump_power_f2 = 4.0;
        self.dispersion_d2_khz = 10.0;

        self.solver.params = MicroresonatorParams::default_bright_soliton();
        self.solver.params.alpha = self.detuning_alpha;
        self.solver.params.f_drive = self.pump_power_f2.sqrt();
        self.solver.params.d2 = 2.0 * PI * self.dispersion_d2_khz * 1000.0;

        self.solver.init_soliton(0.0);
        self.solver.run_to_steady_state(80);
        self.update_cached_curves();
        self.status_msg = "Preset: Dissipative Single Soliton loaded (alpha = 3.5, F^2 = 4.0, D_2 = 10 kHz).".to_string();
    }

    /// Applies the periodic Turing pattern rolls preset.
    pub fn apply_turing_roll_preset(&mut self) {
        self.detuning_alpha = 1.8;
        self.pump_power_f2 = 2.56;
        self.dispersion_d2_khz = 10.0;

        self.solver.params = MicroresonatorParams::default_turing_roll();
        self.solver.params.alpha = self.detuning_alpha;
        self.solver.params.f_drive = self.pump_power_f2.sqrt();
        self.solver.params.d2 = 2.0 * PI * self.dispersion_d2_khz * 1000.0;

        self.solver.init_turing(8);
        self.solver.run_to_steady_state(80);
        self.update_cached_curves();
        self.status_msg = "Preset: Turing Pattern Rolls loaded (alpha = 1.8, F^2 = 2.56, m = 8 rolls).".to_string();
    }

    /// Performs a full detuning sweep scan across alpha values.
    pub fn sweep_detuning(&mut self) {
        let mut scan_solver = self.solver.clone();
        let scan = scan_solver.sweep_detuning(-1.0, 7.0, 70);

        self.scan_curve = scan
            .alphas
            .iter()
            .zip(scan.powers.iter())
            .map(|(&a, &p)| [a, p * 10.0])
            .collect();

        if let Some(best) = &scan.best_soliton_state {
            self.solver.state = best.clone();
            self.detuning_alpha = best.alpha;
        }

        self.scan_result = Some(scan);
        self.update_cached_curves();
        self.status_msg = "Detuning sweep complete: soliton step plateau mapped.".to_string();
    }

    /// Recomputes solver steady state with current slider parameter values.
    pub fn recompute(&mut self) {
        self.solver.params.alpha = self.detuning_alpha;
        self.solver.params.f_drive = self.pump_power_f2.max(0.1).sqrt();
        self.solver.params.d2 = 2.0 * PI * self.dispersion_d2_khz * 1000.0;

        self.solver.run_to_steady_state(30);
        self.update_cached_curves();
    }

    /// Updates cached plot curves and spectrum metrics.
    fn update_cached_curves(&mut self) {
        self.spectrum = CombSpectrum::from_state(&self.solver.state, &self.solver.params, 1.0);

        self.temporal_curve = self
            .spectrum
            .theta
            .iter()
            .zip(self.spectrum.intensity.iter())
            .map(|(&th, &i)| [th, i])
            .collect();

        self.sech2_curve = self
            .spectrum
            .theta
            .iter()
            .zip(self.spectrum.sech2_fit.iter())
            .map(|(&th, &i)| [th, i])
            .collect();

        self.osa_bars = self
            .spectrum
            .mu
            .iter()
            .zip(self.spectrum.powers_dbm.iter())
            .map(|(&m, &p)| (m, p))
            .collect();
    }

    /// Renders the modal window for the Kerr Microcomb Studio.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Kerr Microcomb Studio")
            .open(&mut is_open)
            .default_width(1020.0)
            .default_height(680.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the entire interactive studio workspace.
    pub fn render_content(&mut self, ui: &mut Ui) {
        // Increment animation phase
        self.anim_phase = (self.anim_phase + 0.04) % (2.0 * PI as f32);

        let mut needs_recompute = false;

        // 1. Controls Header
        ui.horizontal_wrapped(|ui| {
            // Presets
            if ui.button("Single Soliton").clicked() {
                self.apply_single_soliton_preset();
            }
            if ui.button("Turing Rolls").clicked() {
                self.apply_turing_roll_preset();
            }
            if ui.button("Sweep Detuning").clicked() {
                self.sweep_detuning();
            }

            ui.separator();

            // Pump Power F_0^2 Slider
            ui.label(RichText::new("Pump F^2:").size(11.0).color(COLOR_PUMP_GOLD));
            let p_changed = ui
                .add(
                    egui::Slider::new(&mut self.pump_power_f2, 1.0..=8.0)
                        .step_by(0.1)
                        .suffix(""),
                )
                .changed();
            if p_changed {
                needs_recompute = true;
            }

            // Detuning alpha Slider
            ui.label(RichText::new("Detuning alpha:").size(11.0).color(COLOR_COMB_CYAN));
            let a_changed = ui
                .add(
                    egui::Slider::new(&mut self.detuning_alpha, -2.0..=10.0)
                        .step_by(0.1)
                        .suffix(" rad"),
                )
                .changed();
            if a_changed {
                needs_recompute = true;
            }

            // Dispersion D_2 Slider
            ui.label(RichText::new("Dispersion D_2:").size(11.0).color(COLOR_SOLITON_EMERALD));
            let d_changed = ui
                .add(
                    egui::Slider::new(&mut self.dispersion_d2_khz, 1.0..=30.0)
                        .step_by(0.5)
                        .suffix(" kHz"),
                )
                .changed();
            if d_changed {
                needs_recompute = true;
            }

            ui.separator();

            if ui.button("Step Forward").clicked() {
                self.solver.step(0.02);
                self.update_cached_curves();
            }
        });

        if needs_recompute {
            self.recompute();
        }

        ui.add_space(2.0);

        // Status banner
        ui.horizontal(|ui| {
            let reg_color = match self.solver.state.regime {
                MicrocombRegime::DissipativeSoliton => COLOR_SOLITON_EMERALD,
                MicrocombRegime::TuringRolls => COLOR_COMB_CYAN,
                MicrocombRegime::ModulationInstabilityChaos | MicrocombRegime::BreatherSoliton => {
                    COLOR_CHAOS_AMBER
                }
                MicrocombRegime::LowPowerCw => COLOR_CW_SLATE,
            };
            ui.label(RichText::new("State:").size(11.0).color(Color32::from_rgb(160, 180, 200)));
            ui.label(
                RichText::new(self.solver.state.regime.label())
                    .size(11.0)
                    .color(reg_color),
            );
            ui.label(RichText::new("|").size(11.0).color(Color32::from_rgb(80, 100, 120)));
            ui.label(
                RichText::new(&self.status_msg)
                    .size(11.0)
                    .color(Color32::from_rgb(200, 215, 230)),
            );
        });

        ui.separator();

        // 2. 2x2 Workspace Grid
        let total_avail_h = (ui.available_height() - 55.0).max(380.0);
        let row_h = total_avail_h * 0.48;

        // Top Row: Ring Cavity Canvas (Left) + Temporal Soliton Pulse Profile (Right)
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.set_height(row_h);
                ui.heading(
                    RichText::new("Microresonator Ring Cavity Field Intensity |psi(theta)|^2")
                        .size(12.0)
                        .color(Color32::from_rgb(180, 210, 240)),
                );
                self.render_cavity_canvas(ui, row_h - 28.0);
            });

            cols[1].group(|ui| {
                ui.set_height(row_h);
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("Temporal Soliton Pulse Profile & Sech^2 Fit")
                            .size(12.0)
                            .color(Color32::from_rgb(180, 210, 240)),
                    );
                    ui.label(
                        RichText::new(format!("tau_FWHM: {:.1} ps", self.spectrum.tau_fwhm_ps))
                            .size(10.0)
                            .color(COLOR_SOLITON_EMERALD),
                    );
                });
                self.render_temporal_plot(ui, row_h - 28.0);
            });
        });

        ui.add_space(4.0);

        // Bottom Row: OSA Comb Spectrum (Left) + Detuning Hysteresis Scan (Right)
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.set_height(row_h);
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("Optical Spectrum Analyzer (OSA) Comb Lines S(mu) [dBm]")
                            .size(12.0)
                            .color(Color32::from_rgb(180, 210, 240)),
                    );
                    ui.label(
                        RichText::new(format!(
                            "3-dB BW: {:.2} GHz",
                            self.spectrum.bandwidth_3db_ghz()
                        ))
                        .size(10.0)
                        .color(COLOR_COMB_CYAN),
                    );
                });
                self.render_osa_plot(ui, row_h - 28.0);
            });

            cols[1].group(|ui| {
                ui.set_height(row_h);
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("Detuning Hysteresis Scan & Soliton Step Plateau")
                            .size(12.0)
                            .color(Color32::from_rgb(180, 210, 240)),
                    );
                    ui.label(
                        RichText::new(format!("Current alpha: {:.2}", self.detuning_alpha))
                            .size(10.0)
                            .color(COLOR_PUMP_GOLD),
                    );
                });
                self.render_scan_plot(ui, row_h - 28.0);
            });
        });

        ui.separator();

        // 3. Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders the 2D Microresonator Ring Cavity Field Intensity Canvas.
    fn render_cavity_canvas(&self, ui: &mut Ui, h: f32) {
        let w = ui.available_width().max(100.0);
        let (response, painter) = ui.allocate_painter(vec2(w, h), Sense::hover());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 4.0, COLOR_CAVITY_BG);
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, COLOR_RING_BORDER), StrokeKind::Inside);

        let center = rect.center() + vec2(0.0, -10.0);
        let outer_r = (w.min(h) * 0.38).clamp(50.0, 110.0);
        let ring_thick = outer_r * 0.22;
        let inner_r = outer_r - ring_thick;
        let mid_r = (outer_r + inner_r) * 0.5;

        // Ring inner and outer circles
        painter.circle_filled(center, outer_r, Color32::from_rgb(20, 30, 48));
        painter.circle_filled(center, inner_r, COLOR_CAVITY_BG);
        painter.circle_stroke(center, outer_r, Stroke::new(1.5, COLOR_RING_BORDER));
        painter.circle_stroke(center, inner_r, Stroke::new(1.5, COLOR_RING_BORDER));

        // Draw intensity map around ring circumference
        let n = self.solver.state.psi.len();
        let max_p = self.solver.state.peak_power.max(1e-4);

        if n > 0 {
            let seg_count = 64.min(n);
            let d_step = n / seg_count;

            for s in 0..seg_count {
                let idx = s * d_step;
                let ang1 = -PI + (idx as f64) * (2.0 * PI / n as f64);
                let ang2 = -PI + ((idx + d_step) as f64) * (2.0 * PI / n as f64);
                let mid_ang = (ang1 + ang2) * 0.5;

                let power = self.solver.state.psi[idx].norm_sq();
                let norm_int = (power / max_p).clamp(0.0, 1.0);

                // Colormap: Deep navy -> Cyan -> Emerald -> Radiant Gold
                let col = if norm_int > 0.6 {
                    let f = ((norm_int - 0.6) / 0.4) as f32;
                    Color32::from_rgb(
                        (52.0 + f * 203.0) as u8,
                        (211.0 + f * 44.0) as u8,
                        (153.0 * (1.0 - f)) as u8,
                    )
                } else if norm_int > 0.2 {
                    let f = ((norm_int - 0.2) / 0.4) as f32;
                    Color32::from_rgb(
                        (30.0 + f * 22.0) as u8,
                        (140.0 + f * 71.0) as u8,
                        (220.0 - f * 67.0) as u8,
                    )
                } else {
                    let f = (norm_int / 0.2) as f32;
                    Color32::from_rgb(
                        (15.0 + f * 15.0) as u8,
                        (30.0 + f * 110.0) as u8,
                        (60.0 + f * 160.0) as u8,
                    )
                };

                let pos = center + vec2((mid_ang.cos() as f32) * mid_r, (mid_ang.sin() as f32) * mid_r);
                painter.circle_filled(pos, ring_thick * 0.42, col);
            }
        }

        // Circulating bright soliton localized pulse packet marker
        let pulse_center = self.spectrum.pulse_center_theta as f32;
        let pulse_ang = pulse_center + self.anim_phase;
        let pulse_pos = center + vec2(pulse_ang.cos() * mid_r, pulse_ang.sin() * mid_r);

        // Radiant halo for soliton packet
        painter.circle_filled(
            pulse_pos,
            ring_thick * 0.65,
            Color32::from_rgba_unmultiplied(250, 204, 21, 140),
        );
        painter.circle_filled(
            pulse_pos,
            ring_thick * 0.40,
            Color32::from_rgb(255, 255, 220),
        );

        // Propagation rotation arrow along ring
        let arrow_ang = pulse_ang + 0.35;
        let arrow_pos = center + vec2(arrow_ang.cos() * mid_r, arrow_ang.sin() * mid_r);
        let tangent = vec2(-arrow_ang.sin(), arrow_ang.cos());
        painter.line_segment(
            [arrow_pos - tangent * 6.0, arrow_pos + tangent * 6.0],
            Stroke::new(2.0, COLOR_SOLITON_EMERALD),
        );

        // Bus waveguide tangent to ring at bottom
        let bus_y = center.y + outer_r + 8.0;
        let bus_x1 = center.x - outer_r * 1.3;
        let bus_x2 = center.x + outer_r * 1.3;

        painter.line_segment(
            [Pos2::new(bus_x1, bus_y), Pos2::new(bus_x2, bus_y)],
            Stroke::new(3.0, COLOR_BUS_WAVEGUIDE),
        );

        // Evanescent coupling dashed indicators
        painter.line_segment(
            [Pos2::new(center.x - 10.0, bus_y - 4.0), Pos2::new(center.x + 10.0, bus_y - 4.0)],
            Stroke::new(1.0, COLOR_COMB_CYAN),
        );

        // Waveguide port labels
        painter.text(
            Pos2::new(bus_x1, bus_y + 10.0),
            egui::Align2::LEFT_TOP,
            "Pump In (F_0)",
            FontId::proportional(10.0),
            COLOR_PUMP_GOLD,
        );
        painter.text(
            Pos2::new(bus_x2, bus_y + 10.0),
            egui::Align2::RIGHT_TOP,
            "Comb Out",
            FontId::proportional(10.0),
            COLOR_SOLITON_EMERALD,
        );

        // Center ring telemetry text
        painter.text(
            center + vec2(0.0, -8.0),
            egui::Align2::CENTER_CENTER,
            "Microresonator",
            FontId::proportional(10.5),
            Color32::from_rgb(200, 220, 240),
        );
        painter.text(
            center + vec2(0.0, 8.0),
            egui::Align2::CENTER_CENTER,
            "Q = 1.5M | FSR = 100M",
            FontId::proportional(9.5),
            COLOR_COMB_CYAN,
        );
    }

    /// Renders the Temporal Soliton Pulse Profile and sech^2 analytical fit plot.
    fn render_temporal_plot(&self, ui: &mut Ui, h: f32) {
        let temp_points = PlotPoints::new(self.temporal_curve.clone());
        let sech_points = PlotPoints::new(self.sech2_curve.clone());

        Plot::new("kerr_temporal_soliton_pulse")
            .height(h)
            .legend(Legend::default())
            .x_axis_label("Azimuthal Angle theta (rad)")
            .y_axis_label("Field Intensity |psi|^2")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Field |psi(theta)|^2", temp_points)
                        .color(COLOR_COMB_CYAN)
                        .width(2.2),
                );
                plot_ui.line(
                    Line::new("Sech^2 Fit", sech_points)
                        .color(COLOR_PUMP_GOLD)
                        .width(1.5),
                );

                // Horizontal line at background CW level
                plot_ui.hline(
                    HLine::new("CW Background", self.spectrum.background_intensity)
                        .stroke(Stroke::new(1.0, COLOR_CW_SLATE)),
                );

                // Vertical line at pulse center theta_0
                plot_ui.vline(
                    VLine::new("Soliton Center", self.spectrum.pulse_center_theta)
                        .stroke(Stroke::new(1.2, COLOR_SOLITON_EMERALD)),
                );
            });
    }

    /// Renders the Optical Spectrum Analyzer (OSA) Comb Lines bar chart in dBm.
    fn render_osa_plot(&self, ui: &mut Ui, h: f32) {
        let max_p = self.spectrum.powers_dbm.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let thresh_3db = max_p - 3.0;
        let thresh_10db = max_p - 10.0;
        let thresh_20db = max_p - 20.0;

        // Render relative modes [-32, 32]
        let bars: Vec<Bar> = self
            .osa_bars
            .iter()
            .filter(|&&(m, _)| m >= -32 && m <= 32)
            .map(|&(m, p_dbm)| {
                let color = if m == 0 {
                    COLOR_PUMP_GOLD // Pump line
                } else if p_dbm >= thresh_3db {
                    COLOR_SOLITON_EMERALD // 3-dB bandwidth span
                } else if p_dbm >= thresh_10db {
                    COLOR_COMB_CYAN // 10-dB bandwidth span
                } else if p_dbm >= thresh_20db {
                    Color32::from_rgb(120, 160, 210)
                } else {
                    Color32::from_rgb(70, 90, 120)
                };

                Bar::new(m as f64, p_dbm)
                    .width(0.65)
                    .fill(color)
            })
            .collect();

        let chart = BarChart::new("kerr_osa_comb_bars", bars);

        Plot::new("kerr_osa_spectrum_plot")
            .height(h)
            .legend(Legend::default())
            .x_axis_label("Relative Mode Index mu (0 = Pump)")
            .y_axis_label("Power S(mu) (dBm)")
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);

                // Bandwidth threshold reference lines
                plot_ui.hline(
                    HLine::new("-3 dB Threshold", thresh_3db)
                        .stroke(Stroke::new(1.2, COLOR_SOLITON_EMERALD)),
                );
                plot_ui.hline(
                    HLine::new("-10 dB Threshold", thresh_10db)
                        .stroke(Stroke::new(1.0, COLOR_COMB_CYAN)),
                );
                plot_ui.hline(
                    HLine::new("-20 dB Threshold", thresh_20db)
                        .stroke(Stroke::new(0.8, Color32::from_rgb(100, 120, 150))),
                );
            });
    }

    /// Renders the Detuning Hysteresis Scan Curve with the soliton step plateau.
    fn render_scan_plot(&self, ui: &mut Ui, h: f32) {
        let scan_points = PlotPoints::new(self.scan_curve.clone());

        Plot::new("kerr_detuning_scan_plot")
            .height(h)
            .legend(Legend::default())
            .x_axis_label("Detuning alpha (rad)")
            .y_axis_label("Intracavity Power (mW)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Intracavity Power vs alpha", scan_points)
                        .color(COLOR_COMB_CYAN)
                        .width(2.0),
                );

                // Operating point vertical line
                plot_ui.vline(
                    VLine::new("Current alpha", self.detuning_alpha)
                        .stroke(Stroke::new(1.5, COLOR_PUMP_GOLD)),
                );

                // Soliton existence step markers if identified
                if let Some(scan) = &self.scan_result {
                    if let Some((s_start, s_end)) = scan.soliton_step_range {
                        plot_ui.vline(
                            VLine::new("Soliton Step Start", s_start)
                                .stroke(Stroke::new(1.0, COLOR_SOLITON_EMERALD)),
                        );
                        plot_ui.vline(
                            VLine::new("Soliton Step End", s_end)
                                .stroke(Stroke::new(1.0, COLOR_SOLITON_EMERALD)),
                        );
                    }
                }
            });
    }

    /// Renders the bottom Telemetry Footer with all required physics metrics.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let p_intra_mw = self.solver.state.mean_power * 10.0;

        ui.horizontal_wrapped(|ui| {
            // Regime Badge
            let (badge_bg, badge_txt) = match self.solver.state.regime {
                MicrocombRegime::DissipativeSoliton => (COLOR_SOLITON_EMERALD, Color32::BLACK),
                MicrocombRegime::TuringRolls => (COLOR_COMB_CYAN, Color32::BLACK),
                MicrocombRegime::ModulationInstabilityChaos | MicrocombRegime::BreatherSoliton => {
                    (COLOR_CHAOS_AMBER, Color32::BLACK)
                }
                MicrocombRegime::LowPowerCw => (COLOR_CW_SLATE, Color32::BLACK),
            };

            egui::Frame::new()
                .fill(badge_bg)
                .corner_radius(3.0)
                .inner_margin(egui::Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(self.solver.state.regime.label())
                            .color(badge_txt)
                            .size(11.0)
                            .strong(),
                    );
                });

            ui.separator();

            // Comb Mode Counts
            ui.label(
                RichText::new(format!(
                    "Lines (3/10/20 dB): {} / {} / {}",
                    self.spectrum.mode_count_3db,
                    self.spectrum.mode_count_10db,
                    self.spectrum.mode_count_20db
                ))
                .size(11.0)
                .color(Color32::from_rgb(180, 205, 230)),
            );

            ui.separator();

            // 3-dB Bandwidth
            ui.label(
                RichText::new(format!(
                    "3-dB BW: {:.2} GHz",
                    self.spectrum.bandwidth_3db_ghz()
                ))
                .size(11.0)
                .color(COLOR_COMB_CYAN),
            );

            ui.separator();

            // Pulse FWHM Duration in ps
            ui.label(
                RichText::new(format!("tau_FWHM: {:.1} ps", self.spectrum.tau_fwhm_ps))
                    .size(11.0)
                    .color(COLOR_SOLITON_EMERALD),
            );

            ui.separator();

            // Repetition Rate
            ui.label(
                RichText::new(format!(
                    "f_rep: {:.1} MHz",
                    self.spectrum.repetition_rate_mhz()
                ))
                .size(11.0)
                .color(Color32::from_rgb(200, 215, 235)),
            );

            ui.separator();

            // Intracavity Power in mW
            ui.label(
                RichText::new(format!("P_intra: {:.2} mW", p_intra_mw))
                    .size(11.0)
                    .color(COLOR_PUMP_GOLD),
            );

            ui.separator();

            // Sech^2 Fit Quality R^2
            ui.label(
                RichText::new(format!("Sech^2 R^2: {:.3}", self.spectrum.fit_r_squared))
                    .size(11.0)
                    .color(Color32::from_rgb(160, 220, 180)),
            );
        });
    }
}
