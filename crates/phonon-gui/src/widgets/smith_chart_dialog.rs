#![deny(unsafe_code)]

//! Interactive Smith Chart & RF Microwave Visualizer modal dialog for Phonon Visual Studio.
//!
//! Provides vector Smith chart rendering (|Gamma| <= 1, constant-r circles, constant-x arcs),
//! S11 and S22 trajectory curves with cursor readout, stability circles and K/Delta metrics,
//! multi-harmonic spectrum bar chart, P1dB and IP3 extraction, and Touchstone S2P file export.

use egui::{
    pos2, vec2, Color32, FontId, Pos2, Rect, RichText, ScrollArea, Sense, Stroke, Ui, Vec2,
};
use phonon_solver::rf::{
    constant_reactance_arc, constant_resistance_circle, gamma_to_z, load_stability_circle,
    source_stability_circle, standard_reactance_values, standard_resistance_values,
    Complex64, FrequencySweep, HarmonicBalanceResult, HarmonicBalanceSolver, MultiPortSSolver,
    NonLinearMetrics, NonlinearDevice, SweepType, TwoPortSParameters,
};

/// Hover cursor marker metadata on the Smith chart.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoveredMarker {
    pub freq_hz: f64,
    pub is_s22: bool,
    pub gamma: Complex64,
    pub z_actual: Complex64,
}

/// Interactive modal dialog for RF S-parameters, Smith chart visualizer, and Harmonic Balance.
#[derive(Debug, Clone, PartialEq)]
pub struct SmithChartDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Frequency Sweep Configuration
    pub start_freq_hz: f64,
    pub stop_freq_hz: f64,
    pub sweep_points: usize,
    pub is_log_sweep: bool,
    pub z0_ref: f64,

    // RF Circuit Preset DUT Configuration
    pub circuit_type_idx: usize,
    pub r_val: f64,
    pub l_val: f64,
    pub c_val: f64,
    pub tline_zc: f64,
    pub tline_delay_ps: f64,
    pub amp_gain_db: f64,
    pub amp_rev_iso_db: f64,

    // Harmonic Balance & Non-Linear Configuration
    pub hb_harmonics: usize,
    pub hb_pin_dbm: f64,
    pub hb_fund_hz: f64,
    pub hb_a1: f64,
    pub hb_a2: f64,
    pub hb_a3: f64,

    // Simulation Outputs
    pub s_parameters: Vec<TwoPortSParameters>,
    pub hb_result: Option<HarmonicBalanceResult>,
    pub nl_metrics: Option<NonLinearMetrics>,

    // GUI Display and Interaction State
    pub show_s11: bool,
    pub show_s22: bool,
    pub show_stability_circles: bool,
    pub selected_freq_idx: usize,
    pub hovered_info: Option<HoveredMarker>,
    pub status_msg: String,
    pub export_status: Option<String>,
    pub run_requested: bool,
}

impl Default for SmithChartDialog {
    fn default() -> Self {
        let mut dialog = Self {
            is_open: false,
            start_freq_hz: 100_000.0,         // 100 kHz
            stop_freq_hz: 10_000_000_000.0,   // 10 GHz
            sweep_points: 101,
            is_log_sweep: true,
            z0_ref: 50.0,

            circuit_type_idx: 0, // 0: Series RLC Resonator
            r_val: 10.0,
            l_val: 2.53303e-9,   // ~3.16 GHz resonance with 1 pF
            c_val: 1.0e-12,
            tline_zc: 50.0,
            tline_delay_ps: 100.0,
            amp_gain_db: 14.0,
            amp_rev_iso_db: 28.0,

            hb_harmonics: 5,
            hb_pin_dbm: 0.0,
            hb_fund_hz: 1.0e9, // 1 GHz
            hb_a1: 0.02,
            hb_a2: 0.004,
            hb_a3: 0.015,

            s_parameters: Vec::new(),
            hb_result: None,
            nl_metrics: None,

            show_s11: true,
            show_s22: true,
            show_stability_circles: true,
            selected_freq_idx: 50,
            hovered_info: None,
            status_msg: "Ready. Click 'Run RF Sweep & HB Analysis' to extract S-parameters.".to_string(),
            export_status: None,
            run_requested: false,
        };
        // Populate baseline simulation data on startup
        dialog.run_simulation();
        dialog
    }
}

impl SmithChartDialog {
    /// Constructs a new SmithChartDialog in closed state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Executes linear S-parameter AC frequency sweep and non-linear Harmonic Balance.
    pub fn run_simulation(&mut self) {
        let sweep_type = if self.is_log_sweep {
            SweepType::Logarithmic
        } else {
            SweepType::Linear
        };
        let sweep = FrequencySweep::new(
            self.start_freq_hz,
            self.stop_freq_hz,
            self.sweep_points,
            sweep_type,
        );

        let solver = MultiPortSSolver::new(self.z0_ref, sweep);

        // 1. Solve Linear S-Parameters based on selected DUT
        let sweep_res = match self.circuit_type_idx {
            0 => solver.solve_series_rlc(self.r_val, self.l_val, self.c_val),
            1 => solver.solve_shunt_rlc(self.r_val, self.l_val, self.c_val),
            2 => solver.solve_transmission_line(self.tline_zc, self.tline_delay_ps * 1e-12),
            3 => solver.solve_tee_attenuator(16.6667, 66.6667), // 6 dB symmetric T-pad
            4 => solver.solve_amplifier(
                self.amp_gain_db,
                self.amp_rev_iso_db,
                0.25,
                -0.45,
                0.15,
                -0.30,
                5.0e9,
            ),
            _ => solver.solve_series_rlc(self.r_val, self.l_val, self.c_val),
        };

        self.s_parameters = sweep_res.s_parameters;
        if self.selected_freq_idx >= self.s_parameters.len() {
            self.selected_freq_idx = self.s_parameters.len().saturating_sub(1);
        }

        // 2. Solve Non-Linear Harmonic Balance
        let device = NonlinearDevice::Polynomial {
            a1: self.hb_a1,
            a2: self.hb_a2,
            a3: self.hb_a3,
        };
        let hb_solver = HarmonicBalanceSolver::new(self.hb_fund_hz, self.hb_harmonics, device)
            .with_z0(self.z0_ref)
            .with_max_iterations(50)
            .with_tolerance(1e-8);

        self.hb_result = hb_solver.solve_single_tone(self.hb_pin_dbm).ok();

        // 3. Extract Non-Linear Metrics (P1dB and IP3)
        self.nl_metrics = hb_solver
            .compute_compression_and_intercept(-25.0, 15.0, 12)
            .ok();

        self.status_msg = format!(
            "Sweep completed: {} frequency points computed ({:.2} MHz - {:.2} GHz).",
            self.s_parameters.len(),
            self.start_freq_hz / 1e6,
            self.stop_freq_hz / 1e9,
        );
    }

    /// Formats the current S-parameter dataset into standard Touchstone S2P format.
    pub fn export_touchstone_s2p(&self) -> String {
        phonon_solver::rf::format_touchstone_s2p(&self.s_parameters, self.z0_ref)
    }

    /// Renders the modal window for the Smith Chart dialog.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("RF & Microwave S-Parameter Extraction & Smith Chart Visualizer")
            .open(&mut is_open)
            .resizable(true)
            .default_size(Vec2::new(940.0, 680.0))
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the inner content of the dialog.
    pub fn render_content(&mut self, ui: &mut Ui) {
        // Status & Notification Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&self.status_msg)
                    .color(Color32::from_rgb(180, 210, 240))
                    .size(11.5),
            );
            if let Some(ref exp_msg) = self.export_status {
                ui.label(
                    RichText::new(format!("| {}", exp_msg))
                        .color(Color32::from_rgb(80, 220, 120))
                        .size(11.5),
                );
            }
        });
        ui.separator();

        // Controls Bar
        ui.horizontal_wrapped(|ui| {
            ui.label("Start:");
            ui.add(
                egui::DragValue::new(&mut self.start_freq_hz)
                    .range(1.0e3..=50.0e9)
                    .speed(1.0e6)
                    .custom_formatter(|val, _| {
                        if val >= 1.0e9 {
                            format!("{:.2} GHz", val / 1.0e9)
                        } else if val >= 1.0e6 {
                            format!("{:.2} MHz", val / 1.0e6)
                        } else {
                            format!("{:.1} kHz", val / 1.0e3)
                        }
                    }),
            );

            ui.label("Stop:");
            ui.add(
                egui::DragValue::new(&mut self.stop_freq_hz)
                    .range(1.0e6..=100.0e9)
                    .speed(1.0e8)
                    .custom_formatter(|val, _| {
                        if val >= 1.0e9 {
                            format!("{:.2} GHz", val / 1.0e9)
                        } else if val >= 1.0e6 {
                            format!("{:.2} MHz", val / 1.0e6)
                        } else {
                            format!("{:.1} kHz", val / 1.0e3)
                        }
                    }),
            );

            ui.label("Points:");
            ui.add(egui::DragValue::new(&mut self.sweep_points).range(11..=401));

            ui.checkbox(&mut self.is_log_sweep, "Log Sweep");

            ui.label("Z0:");
            ui.add(egui::DragValue::new(&mut self.z0_ref).range(1.0..=500.0).speed(1.0).suffix(" Ohm"));

            ui.label("DUT:");
            egui::ComboBox::from_id_salt("circuit_preset_selector")
                .selected_text(match self.circuit_type_idx {
                    0 => "Series RLC Resonator",
                    1 => "Shunt RLC Tank",
                    2 => "Transmission Line",
                    3 => "T-Pad Attenuator (6 dB)",
                    4 => "RF Transistor Amplifier",
                    _ => "Custom",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.circuit_type_idx, 0, "Series RLC Resonator");
                    ui.selectable_value(&mut self.circuit_type_idx, 1, "Shunt RLC Tank");
                    ui.selectable_value(&mut self.circuit_type_idx, 2, "Transmission Line");
                    ui.selectable_value(&mut self.circuit_type_idx, 3, "T-Pad Attenuator (6 dB)");
                    ui.selectable_value(&mut self.circuit_type_idx, 4, "RF Transistor Amplifier");
                });

            if ui.button(RichText::new("Run Sweep & HB").strong()).clicked() {
                self.run_simulation();
            }

            if ui.button("Export Touchstone S2P").clicked() {
                let s2p_content = self.export_touchstone_s2p();
                ui.ctx().copy_text(s2p_content);
                self.export_status = Some(format!(
                    "Exported {} points in Touchstone .s2p format to clipboard.",
                    self.s_parameters.len()
                ));
            }
        });

        ui.separator();

        // Main Layout: 2 Columns (Left: Vector Smith Chart, Right: Metrics and Spectrum)
        ui.columns(2, |columns| {
            // Left Column: Vector Smith Chart
            columns[0].vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.show_s11, RichText::new("S11").color(Color32::from_rgb(0, 220, 255)));
                    ui.checkbox(&mut self.show_s22, RichText::new("S22").color(Color32::from_rgb(255, 140, 20)));
                    ui.checkbox(&mut self.show_stability_circles, RichText::new("Stability Circles").color(Color32::from_rgb(230, 60, 60)));
                });

                self.render_smith_chart_canvas(ui);

                // Cursor Readout
                if let Some(hover) = self.hovered_info {
                    ui.horizontal(|ui| {
                        let tag = if hover.is_s22 { "S22" } else { "S11" };
                        let f_str = if hover.freq_hz >= 1e9 {
                            format!("{:.3} GHz", hover.freq_hz / 1e9)
                        } else {
                            format!("{:.2} MHz", hover.freq_hz / 1e6)
                        };
                        let gamma_mag = hover.gamma.abs();
                        let gamma_ang = hover.gamma.arg().to_degrees();
                        let z_re = hover.z_actual.re;
                        let z_im = hover.z_actual.im;
                        let sign = if z_im >= 0.0 { "+" } else { "-" };

                        ui.label(
                            RichText::new(format!(
                                "[{}] f: {} | |Gamma|: {:.3} /_ {:.1} deg | Z: {:.1} {} j{:.1} Ohm",
                                tag, f_str, gamma_mag, gamma_ang, z_re, sign, z_im.abs()
                            ))
                            .color(Color32::from_rgb(220, 235, 255))
                            .size(11.0),
                        );
                    });
                } else {
                    ui.label(
                        RichText::new("Hover mouse over Smith chart to inspect frequency, Gamma, and impedance.")
                            .color(Color32::from_rgb(120, 140, 160))
                            .size(11.0),
                    );
                }
            });

            // Right Column: Stability, Gain, and Harmonic Balance Spectrum
            columns[1].vertical(|ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.render_metrics_and_spectrum(ui);
                });
            });
        });
    }

    /// Renders the interactive vector Smith chart canvas.
    fn render_smith_chart_canvas(&mut self, ui: &mut Ui) {
        let chart_dim = (ui.available_width().min(380.0)).max(240.0);
        let (response, painter) = ui.allocate_painter(Vec2::splat(chart_dim), Sense::hover());
        let rect = response.rect;
        let center = rect.center();
        let radius = (rect.width().min(rect.height()) as f64) * 0.46;

        let to_screen = |g: Complex64| -> Pos2 {
            center + vec2((g.re * radius) as f32, (-g.im * radius) as f32)
        };

        // 1. Background disk
        painter.circle_filled(center, radius as f32, Color32::from_rgb(14, 20, 30));
        painter.circle_stroke(center, radius as f32, Stroke::new(1.5, Color32::from_rgb(90, 110, 145)));

        // 2. Real axis line (x = 0)
        painter.line_segment(
            [to_screen(Complex64::new(-1.0, 0.0)), to_screen(Complex64::new(1.0, 0.0))],
            Stroke::new(1.0, Color32::from_rgb(70, 90, 120)),
        );

        // 3. Constant resistance circles: r in [0.2, 0.5, 1.0, 2.0, 5.0]
        for &r in &standard_resistance_values() {
            if r <= 0.0 {
                continue;
            }
            let circ = constant_resistance_circle(r);
            let c_screen = to_screen(circ.center);
            let r_screen = (circ.radius * radius) as f32;
            painter.circle_stroke(
                c_screen,
                r_screen,
                Stroke::new(0.8, Color32::from_rgb(40, 55, 80)),
            );

            // Label on real axis
            let label_pos = to_screen(Complex64::new((r - 1.0) / (r + 1.0), 0.0));
            painter.text(
                label_pos + vec2(0.0, 7.0),
                egui::Align2::CENTER_CENTER,
                format!("{:.1}", r),
                FontId::proportional(8.5),
                Color32::from_rgb(80, 100, 130),
            );
        }

        // 4. Constant reactance arcs: x in [0.2, 0.5, 1.0, 2.0, 5.0, -0.2, -0.5, ...]
        for &x in &standard_reactance_values() {
            let pts = constant_reactance_arc(x, 32);
            let screen_pts: Vec<Pos2> = pts.into_iter().map(to_screen).collect();
            for window in screen_pts.windows(2) {
                painter.line_segment([window[0], window[1]], Stroke::new(0.8, Color32::from_rgb(35, 48, 70)));
            }
        }

        // Center matched point (Z = Z0)
        painter.circle_filled(center, 2.5, Color32::from_rgb(130, 160, 200));

        // 5. Stability circles overlay for selected frequency point
        if self.show_stability_circles && !self.s_parameters.is_empty() {
            let idx = self.selected_freq_idx.min(self.s_parameters.len() - 1);
            let s_sel = &self.s_parameters[idx];

            let load_circ = load_stability_circle(s_sel);
            let src_circ = source_stability_circle(s_sel);

            let load_center_screen = to_screen(load_circ.center);
            let load_radius_screen = (load_circ.radius * radius) as f32;
            painter.circle_stroke(
                load_center_screen,
                load_radius_screen,
                Stroke::new(1.2, Color32::from_rgb(220, 60, 60)),
            );

            let src_center_screen = to_screen(src_circ.center);
            let src_radius_screen = (src_circ.radius * radius) as f32;
            painter.circle_stroke(
                src_center_screen,
                src_radius_screen,
                Stroke::new(1.2, Color32::from_rgb(240, 160, 40)),
            );
        }

        // 6. S11 Trajectory Curve (Cyan)
        if self.show_s11 && !self.s_parameters.is_empty() {
            let s11_pts: Vec<Pos2> = self.s_parameters.iter().map(|s| to_screen(s.s11)).collect();
            for window in s11_pts.windows(2) {
                painter.line_segment([window[0], window[1]], Stroke::new(2.0, Color32::from_rgb(0, 220, 255)));
            }

            // Mark selected point
            let idx = self.selected_freq_idx.min(self.s_parameters.len() - 1);
            let pt = to_screen(self.s_parameters[idx].s11);
            painter.circle_filled(pt, 4.0, Color32::from_rgb(0, 240, 255));
            painter.circle_stroke(pt, 6.0, Stroke::new(1.0, Color32::WHITE));
        }

        // 7. S22 Trajectory Curve (Orange)
        if self.show_s22 && !self.s_parameters.is_empty() {
            let s22_pts: Vec<Pos2> = self.s_parameters.iter().map(|s| to_screen(s.s22)).collect();
            for window in s22_pts.windows(2) {
                painter.line_segment([window[0], window[1]], Stroke::new(2.0, Color32::from_rgb(255, 140, 20)));
            }

            // Mark selected point
            let idx = self.selected_freq_idx.min(self.s_parameters.len() - 1);
            let pt = to_screen(self.s_parameters[idx].s22);
            painter.circle_filled(pt, 4.0, Color32::from_rgb(255, 160, 30));
        }

        // 8. Hover cursor detection and readout
        self.hovered_info = None;
        if let Some(hover_pos) = response.hover_pos() {
            // Find closest frequency point on S11 or S22
            let mut min_dist_sq = f64::INFINITY;
            let mut best_marker = None;

            for s in &self.s_parameters {
                let p11 = to_screen(s.s11);
                let d11 = (p11.x - hover_pos.x).hypot(p11.y - hover_pos.y) as f64;
                if d11 < min_dist_sq && self.show_s11 {
                    min_dist_sq = d11;
                    best_marker = Some(HoveredMarker {
                        freq_hz: s.freq_hz,
                        is_s22: false,
                        gamma: s.s11,
                        z_actual: gamma_to_z(s.s11, self.z0_ref),
                    });
                }

                let p22 = to_screen(s.s22);
                let d22 = (p22.x - hover_pos.x).hypot(p22.y - hover_pos.y) as f64;
                if d22 < min_dist_sq && self.show_s22 {
                    min_dist_sq = d22;
                    best_marker = Some(HoveredMarker {
                        freq_hz: s.freq_hz,
                        is_s22: true,
                        gamma: s.s22,
                        z_actual: gamma_to_z(s.s22, self.z0_ref),
                    });
                }
            }

            if min_dist_sq < 30.0 {
                self.hovered_info = best_marker;
                if let Some(m) = best_marker {
                    let screen_pt = to_screen(m.gamma);
                    painter.circle_stroke(
                        screen_pt,
                        8.0,
                        Stroke::new(1.5, Color32::from_rgb(255, 255, 100)),
                    );
                }
            } else {
                // Freeform cursor inside unit disk
                let dx = ((hover_pos.x - center.x) as f64) / radius;
                let dy = -(((hover_pos.y - center.y) as f64) / radius);
                let gamma = Complex64::new(dx, dy);
                if gamma.abs() <= 1.02 {
                    self.hovered_info = Some(HoveredMarker {
                        freq_hz: if !self.s_parameters.is_empty() {
                            self.s_parameters[self.selected_freq_idx.min(self.s_parameters.len() - 1)].freq_hz
                        } else {
                            1.0e9
                        },
                        is_s22: false,
                        gamma,
                        z_actual: gamma_to_z(gamma, self.z0_ref),
                    });
                }
            }
        }
    }

    /// Renders RF stability metrics, gain metrics, and Harmonic Balance spectrum bar chart.
    fn render_metrics_and_spectrum(&mut self, ui: &mut Ui) {
        // Frequency point selection
        if !self.s_parameters.is_empty() {
            ui.heading("Selected Operating Point");
            let max_idx = self.s_parameters.len() - 1;
            ui.horizontal(|ui| {
                ui.label("Frequency Index:");
                ui.add(egui::Slider::new(&mut self.selected_freq_idx, 0..=max_idx));
                let cur_s = &self.s_parameters[self.selected_freq_idx.min(max_idx)];
                let f_str = if cur_s.freq_hz >= 1e9 {
                    format!("{:.3} GHz", cur_s.freq_hz / 1e9)
                } else {
                    format!("{:.2} MHz", cur_s.freq_hz / 1e6)
                };
                ui.label(RichText::new(f_str).strong().color(Color32::from_rgb(0, 220, 255)));
            });

            let cur_s = &self.s_parameters[self.selected_freq_idx.min(max_idx)];
            let k = cur_s.stability_factor_k();
            let delta = cur_s.delta();
            let mu1 = cur_s.mu1();
            let is_stable = cur_s.is_unconditionally_stable();

            // Stability Indicator Card
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if is_stable {
                        ui.label(
                            RichText::new("UNCONDITIONALLY STABLE")
                                .color(Color32::from_rgb(60, 220, 100))
                                .strong(),
                        );
                    } else {
                        ui.label(
                            RichText::new("CONDITIONALLY STABLE")
                                .color(Color32::from_rgb(240, 160, 40))
                                .strong(),
                        );
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(format!("Rollett K: {:.3}", k));
                    ui.label(format!("|Delta|: {:.3}", delta.abs()));
                    ui.label(format!("mu1: {:.3}", mu1));
                });

                ui.horizontal(|ui| {
                    let msg_db = cur_s.msg_db();
                    ui.label(format!("MSG: {:.2} dB", msg_db));
                    if let Some(mag_db) = cur_s.mag_db() {
                        ui.label(format!("MAG: {:.2} dB", mag_db));
                    } else {
                        ui.label("MAG: N/A (K <= 1)");
                    }
                    ui.label(format!("VSWR: {:.2}", cur_s.vswr()));
                    ui.label(format!("RL: {:.1} dB", cur_s.return_loss_db()));
                });
            });
        }

        ui.add_space(8.0);

        // Harmonic Balance Spectrum Bar Chart & Non-Linear Metrics
        ui.heading("Harmonic Balance Spectrum & Non-Linearities");
        if let Some(ref hb) = self.hb_result {
            ui.horizontal(|ui| {
                ui.label(format!("Fund: {:.2} GHz", hb.fundamental_hz / 1e9));
                ui.label(format!("Drive Pin: {:.1} dBm", self.hb_pin_dbm));
                ui.label(format!("THD: {:.2}%", hb.thd_pct()));
                if hb.converged {
                    ui.label(RichText::new("Converged").color(Color32::from_rgb(60, 220, 100)));
                } else {
                    ui.label(RichText::new("Iterating").color(Color32::from_rgb(240, 160, 40)));
                }
            });

            // Spectrum Bar Chart Canvas
            self.render_spectrum_bar_chart(ui, hb);
        }

        // P1dB and IP3 Readouts
        if let Some(ref nl) = self.nl_metrics {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Non-Linear Intercepts:").strong());
                    ui.label(format!("Gain G0: {:+.2} dB", nl.linear_gain_db));
                });
                ui.horizontal(|ui| {
                    ui.label(format!("Pin(1dB): {:+.2} dBm", nl.p1db_in_dbm));
                    ui.label(format!("Pout(1dB): {:+.2} dBm", nl.p1db_out_dbm));
                });
                ui.horizontal(|ui| {
                    ui.label(format!("IIP3: {:+.2} dBm", nl.ip3_in_dbm));
                    ui.label(format!("OIP3: {:+.2} dBm", nl.ip3_out_dbm));
                });
            });
        }
    }

    /// Renders custom vector bar chart for harmonic components in dBm.
    fn render_spectrum_bar_chart(&self, ui: &mut Ui, hb: &HarmonicBalanceResult) {
        let chart_height = 120.0;
        let chart_width = ui.available_width().max(200.0);
        let (response, painter) = ui.allocate_painter(Vec2::new(chart_width, chart_height), Sense::hover());
        let rect = response.rect;

        // Chart bounds: -60 dBm to +20 dBm
        let min_dbm = -60.0_f64;
        let max_dbm = 20.0_f64;
        let span_dbm = max_dbm - min_dbm;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 20, 30));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 60, 85)), egui::StrokeKind::Inside);

        // Draw 0 dBm reference grid line
        let y_zero = rect.bottom() - ((0.0 - min_dbm) / span_dbm) as f32 * rect.height();
        painter.line_segment(
            [pos2(rect.left(), y_zero), pos2(rect.right(), y_zero)],
            Stroke::new(0.8, Color32::from_rgb(70, 85, 110)),
        );
        painter.text(
            pos2(rect.left() + 4.0, y_zero - 6.0),
            egui::Align2::LEFT_CENTER,
            "0 dBm",
            FontId::proportional(8.5),
            Color32::from_rgb(100, 120, 150),
        );

        let n_bars = hb.harmonics.len();
        if n_bars == 0 {
            return;
        }

        let bar_width = (rect.width() / (n_bars as f32 + 1.0)).min(32.0).max(12.0);
        let spacing = (rect.width() - (bar_width * n_bars as f32)) / (n_bars as f32 + 1.0);

        for (i, h) in hb.harmonics.iter().enumerate() {
            let p_clamped = h.power_dbm.clamp(min_dbm, max_dbm);
            let frac = (p_clamped - min_dbm) / span_dbm;
            let bar_h = (frac as f32 * (rect.height() - 16.0)).max(2.0);

            let x_left = rect.left() + spacing + (i as f32) * (bar_width + spacing);
            let x_right = x_left + bar_width;
            let y_top = rect.bottom() - 16.0 - bar_h;
            let y_bot = rect.bottom() - 16.0;

            let bar_rect = Rect::from_min_max(pos2(x_left, y_top), pos2(x_right, y_bot));

            let bar_color = match h.harmonic_index {
                0 => Color32::from_rgb(100, 150, 200), // DC
                1 => Color32::from_rgb(40, 220, 120),  // Fundamental
                2 => Color32::from_rgb(240, 160, 30),  // 2nd Harmonic
                3 => Color32::from_rgb(230, 70, 70),   // 3rd Harmonic
                _ => Color32::from_rgb(180, 70, 200),  // Higher Harmonics
            };

            painter.rect_filled(bar_rect, 2.0, bar_color);

            // Label harmonic index below bar
            let lbl = if h.harmonic_index == 0 {
                "DC".to_string()
            } else {
                format!("{}f0", h.harmonic_index)
            };
            painter.text(
                pos2((x_left + x_right) * 0.5, rect.bottom() - 8.0),
                egui::Align2::CENTER_CENTER,
                lbl,
                FontId::proportional(9.0),
                Color32::from_rgb(180, 200, 220),
            );

            // Power readout above bar
            if bar_h > 12.0 {
                painter.text(
                    pos2((x_left + x_right) * 0.5, y_top - 5.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{:.1}", h.power_dbm),
                    FontId::proportional(8.0),
                    Color32::from_rgb(210, 225, 245),
                );
            }
        }
    }
}
