#![deny(unsafe_code)]

//! Interactive Fractional Quantum Hall Anyon Braiding & Non-Abelian Topological Circuit Co-Simulator.
//!
//! Provides:
//! - Spacetime World-Line Braiding Trajectory Canvas: renders multi-strand anyon worldlines
//!   crossing over and under according to compiled braid words with clear crossing topology.
//! - Aharonov-Bohm Conductance Oscillations Plot: egui_plot rendering G(B) comparing even vs odd
//!   bulk anyon parity states and filling fractions (nu = 5/2 vs nu = 1/3).
//! - Shot Noise & Fractional Charge Gauge: displays measured Fano factor F = e*/e, quasiparticle
//!   charge, tunneling transmission probability, and Johnson-Nyquist thermal floor.
//! - Topological Quantum Gate Compiler Panel: compiles target gates (H, S, X, Z, T, CNOT, CZ)
//!   into braid words with fidelity >= 99.0% and interactive animation stepping.
//! - Telemetry Footer: Anyon Model, Topological Gap, Yang-Baxter residual norm, Gate Fidelity,
//!   Fano Factor, and bulk parity state.

use egui::{
    pos2, vec2, Color32, FontId, ProgressBar, RichText, Sense, Stroke, Ui,
};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use phonon_solver::fqh_braiding::{
    AnyonModelKind, BraidGenerator, FillingFraction, FqhEdgeInterferometer,
    InterferometerType, SynthesisResult, TargetGate, TopologicalGateSynthesizer,
};
use std::f64::consts::PI;

/// Palette colors for anyon worldline strands.
const STRAND_COLORS: [Color32; 8] = [
    Color32::from_rgb(0, 220, 255),   // Strand 1: Bright Cyan
    Color32::from_rgb(255, 80, 180),  // Strand 2: Magenta
    Color32::from_rgb(255, 205, 50),  // Strand 3: Gold
    Color32::from_rgb(50, 230, 120),  // Strand 4: Emerald Green
    Color32::from_rgb(180, 120, 255), // Strand 5: Purple
    Color32::from_rgb(255, 130, 90),  // Strand 6: Coral
    Color32::from_rgb(160, 240, 60),  // Strand 7: Lime
    Color32::from_rgb(90, 160, 255),  // Strand 8: Sky Blue
];

/// Interactive modal dialog for FQH Anyon Braiding & Topological Circuits.
#[derive(Debug, Clone, PartialEq)]
pub struct FqhBraidingDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Controls
    /// Active anyon model (Moore-Read Pfaffian vs Fibonacci).
    pub anyon_model: AnyonModelKind,
    /// Target quantum gate for braid compilation.
    pub target_gate: TargetGate,
    /// Filling fraction of the fractional quantum Hall fluid.
    pub filling_fraction: FillingFraction,
    /// Interferometer geometry (Fabry-Perot vs Mach-Zehnder).
    pub interferometer_type: InterferometerType,
    /// Applied perpendicular magnetic field B in Tesla.
    pub magnetic_field_tesla: f64,
    /// Quantum point contact gate voltage V_g in Volts.
    pub gate_voltage_v: f64,
    /// Cryogenic temperature T in mK.
    pub temperature_mk: f64,
    /// Enclosed magnetic flux loop area in um^2.
    pub area_um2: f64,
    /// Number of enclosed bulk quasiparticles.
    pub bulk_anyon_count: usize,
    /// Injected bias current in nA.
    pub bias_current_na: f64,

    // Animation & Stepping
    /// Current animated step index along the braid word.
    pub animation_step: usize,
    /// Continuous animation scrub progress in [0.0, 1.0].
    pub animation_progress: f32,

    // Cached simulation models and metrics
    pub interferometer: FqhEdgeInterferometer,
    pub synthesis_result: SynthesisResult,
    pub yb_residual_norm: f64,
    pub measured_fano_factor: f64,
    pub shot_noise_density: f64,
    pub tunneling_prob: f64,
    pub thermal_noise_floor: f64,
    pub ab_curve_active: Vec<[f64; 2]>,
    pub ab_curve_alt: Vec<[f64; 2]>,
}

impl Default for FqhBraidingDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl FqhBraidingDialog {
    /// Creates a new FqhBraidingDialog with default parameters.
    pub fn new() -> Self {
        let anyon_model = AnyonModelKind::MooreReadPfaffian;
        let target_gate = TargetGate::Hadamard;
        let filling_fraction = FillingFraction::Nu5_2;
        let interferometer_type = InterferometerType::FabryPerot;
        let magnetic_field_tesla = 4.0;
        let gate_voltage_v = -0.5;
        let temperature_mk = 20.0;
        let area_um2 = 2.0;
        let bulk_anyon_count = 0;
        let bias_current_na = 1.0;

        let interferometer = FqhEdgeInterferometer::new(
            filling_fraction,
            interferometer_type,
            area_um2,
            5.0,
            5.5,
            magnetic_field_tesla,
            gate_voltage_v,
            temperature_mk,
            bulk_anyon_count,
            bias_current_na,
        );

        let synthesis_result = TopologicalGateSynthesizer::compile(target_gate, anyon_model);
        let (_, yb_residual_norm) = anyon_model.verify_yang_baxter();
        let measured_fano_factor = interferometer.fano_factor();
        let shot_noise_density = interferometer.shot_noise();
        let tunneling_prob = interferometer.tunneling_probability(gate_voltage_v);
        let thermal_noise_floor = interferometer.thermal_noise_floor();

        let mut dialog = Self {
            is_open: false,
            anyon_model,
            target_gate,
            filling_fraction,
            interferometer_type,
            magnetic_field_tesla,
            gate_voltage_v,
            temperature_mk,
            area_um2,
            bulk_anyon_count,
            bias_current_na,
            animation_step: 0,
            animation_progress: 1.0,
            interferometer,
            synthesis_result,
            yb_residual_norm,
            measured_fano_factor,
            shot_noise_density,
            tunneling_prob,
            thermal_noise_floor,
            ab_curve_active: Vec::new(),
            ab_curve_alt: Vec::new(),
        };

        dialog.recompute();
        dialog
    }

    /// Recomputes physics sweeps, gate synthesis, and telemetry.
    pub fn recompute(&mut self) {
        // 1. Rebuild interferometer model
        self.interferometer = FqhEdgeInterferometer::new(
            self.filling_fraction,
            self.interferometer_type,
            self.area_um2,
            5.0,
            5.5,
            self.magnetic_field_tesla,
            self.gate_voltage_v,
            self.temperature_mk,
            self.bulk_anyon_count,
            self.bias_current_na,
        );

        // 2. Re-compile topological quantum gate
        self.synthesis_result =
            TopologicalGateSynthesizer::compile(self.target_gate, self.anyon_model);

        // 3. Verify Yang-Baxter relation
        let (_, yb_err) = self.anyon_model.verify_yang_baxter();
        self.yb_residual_norm = yb_err;

        // 4. Update transport & shot noise telemetry
        self.measured_fano_factor = self.interferometer.fano_factor();
        self.shot_noise_density = self.interferometer.shot_noise();
        self.tunneling_prob = self.interferometer.tunneling_probability(self.gate_voltage_v);
        self.thermal_noise_floor = self.interferometer.thermal_noise_floor();

        // 5. Generate Aharonov-Bohm conductance oscillation curves
        let delta_b = self.interferometer.oscillation_period_delta_b();
        let b_min = (self.magnetic_field_tesla - 1.5 * delta_b).max(0.0);
        let b_max = b_min + 3.0 * delta_b;
        let num_pts = 120;

        self.ab_curve_active =
            self.interferometer
                .simulate_b_field_sweep(b_min, b_max, num_pts);

        // Generate alternate comparison curve (odd parity if currently even, or even if odd)
        let mut alt_interferometer = self.interferometer.clone();
        alt_interferometer.bulk_anyon_count = if self.bulk_anyon_count % 2 == 0 { 1 } else { 0 };
        self.ab_curve_alt = alt_interferometer.simulate_b_field_sweep(b_min, b_max, num_pts);
    }

    /// Renders modal window.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Phonon Studio Fractional Quantum Hall Anyon Braiding & Non-Abelian Topological Circuit Co-Simulator")
            .open(&mut is_open)
            .default_size([1080.0, 740.0])
            .min_size([860.0, 600.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders dialog contents: controls, 4 dashboard quadrants, and telemetry footer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut needs_recompute = false;

        // 1. Top Controls Bar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 4.0);

            // Anyon Model Selector
            ui.label(RichText::new("Anyon Model:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            egui::ComboBox::from_id_salt("fqh_model_combo")
                .selected_text(self.anyon_model.name())
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.anyon_model, AnyonModelKind::MooreReadPfaffian, AnyonModelKind::MooreReadPfaffian.name()).clicked() {
                        needs_recompute = true;
                    }
                    if ui.selectable_value(&mut self.anyon_model, AnyonModelKind::Fibonacci, AnyonModelKind::Fibonacci.name()).clicked() {
                        needs_recompute = true;
                    }
                });

            ui.separator();

            // Target Gate Selector
            ui.label(RichText::new("Target Gate:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            egui::ComboBox::from_id_salt("fqh_gate_combo")
                .selected_text(self.target_gate.label())
                .show_ui(ui, |ui| {
                    let gates = [
                        TargetGate::Hadamard,
                        TargetGate::PhaseS,
                        TargetGate::PauliX,
                        TargetGate::PauliZ,
                        TargetGate::TGate,
                        TargetGate::Cnot,
                        TargetGate::Cz,
                    ];
                    for g in gates {
                        if ui.selectable_value(&mut self.target_gate, g, g.label()).clicked() {
                            needs_recompute = true;
                        }
                    }
                });

            ui.separator();

            // Filling Fraction Selector
            ui.label(RichText::new("Filling Fraction:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            egui::ComboBox::from_id_salt("fqh_filling_combo")
                .selected_text(self.filling_fraction.label())
                .show_ui(ui, |ui| {
                    let fillings = [
                        FillingFraction::Nu5_2,
                        FillingFraction::Nu1_3,
                        FillingFraction::Nu2_3,
                        FillingFraction::Nu2_5,
                    ];
                    for f in fillings {
                        if ui.selectable_value(&mut self.filling_fraction, f, f.label()).clicked() {
                            needs_recompute = true;
                        }
                    }
                });

            ui.separator();

            // Interferometer Geometry
            ui.label(RichText::new("Geometry:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            egui::ComboBox::from_id_salt("fqh_ifm_combo")
                .selected_text(self.interferometer_type.label())
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.interferometer_type, InterferometerType::FabryPerot, InterferometerType::FabryPerot.label()).clicked() {
                        needs_recompute = true;
                    }
                    if ui.selectable_value(&mut self.interferometer_type, InterferometerType::MachZehnder, InterferometerType::MachZehnder.label()).clicked() {
                        needs_recompute = true;
                    }
                });

            ui.separator();

            // Compile Braid Button
            if ui.button(RichText::new("Compile Braid").color(Color32::from_rgb(100, 240, 210)).strong()).clicked() {
                needs_recompute = true;
                self.animation_step = 0;
                self.animation_progress = 1.0;
            }

            // Step Braid Animation Button
            let num_ops = self.synthesis_result.braid_sequence.generators.len().max(1);
            if ui.button(RichText::new("Step Braid Animation").color(Color32::from_rgb(255, 205, 50))).clicked() {
                self.animation_step = (self.animation_step + 1) % (num_ops + 1);
                self.animation_progress = (self.animation_step as f32) / (num_ops as f32);
            }
        });

        ui.add_space(3.0);

        // Parameter Sliders Bar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 4.0);

            // B field slider
            ui.label(RichText::new("B Field:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui.add(egui::Slider::new(&mut self.magnetic_field_tesla, 0.0..=10.0).step_by(0.1).suffix(" T")).changed() {
                needs_recompute = true;
            }

            ui.separator();

            // Area slider
            ui.label(RichText::new("Area A:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui.add(egui::Slider::new(&mut self.area_um2, 0.5..=10.0).step_by(0.1).suffix(" um^2")).changed() {
                needs_recompute = true;
            }

            ui.separator();

            // Gate Voltage slider
            ui.label(RichText::new("Gate V_g:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui.add(egui::Slider::new(&mut self.gate_voltage_v, -1.0..=0.0).step_by(0.02).suffix(" V")).changed() {
                needs_recompute = true;
            }

            ui.separator();

            // Temperature slider
            ui.label(RichText::new("Temp T:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui.add(egui::Slider::new(&mut self.temperature_mk, 5.0..=100.0).step_by(1.0).suffix(" mK")).changed() {
                needs_recompute = true;
            }

            ui.separator();

            // Bulk Anyon Count slider
            ui.label(RichText::new("Bulk Anyons n_bulk:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui.add(egui::Slider::new(&mut self.bulk_anyon_count, 0..=8)).changed() {
                needs_recompute = true;
            }
        });

        if needs_recompute {
            self.recompute();
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 2. Central 2x2 Quadrant Grid
        let avail_size = ui.available_size();
        let footer_reserve = 42.0;
        let content_h = (avail_size.y - footer_reserve).max(340.0);
        let quad_w = ((avail_size.x - 16.0) * 0.5).max(380.0);
        let quad_h = ((content_h - 16.0) * 0.5).max(180.0);

        // Top Row: Quadrant 1 (World-Line Canvas) & Quadrant 2 (Aharonov-Bohm Plot)
        ui.horizontal(|ui| {
            // Quadrant 1: World-Line Braiding Trajectory Canvas
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Spacetime World-Line Braiding Trajectory (Over/Under Crossings)")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(0, 220, 255)),
                );
                self.render_braiding_worldline_canvas(ui, quad_w, quad_h - 22.0);
            });

            ui.separator();

            // Quadrant 2: Aharonov-Bohm Conductance Oscillations Plot
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Aharonov-Bohm Conductance Oscillations G(B) [Even vs Odd Parity]")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(100, 240, 210)),
                );
                self.render_aharonov_bohm_plot(ui, quad_w, quad_h - 22.0);
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Bottom Row: Quadrant 3 (Topological Gate Compiler) & Quadrant 4 (Shot Noise Gauge)
        ui.horizontal(|ui| {
            // Quadrant 3: Topological Gate Compiler Panel
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Topological Quantum Gate Compiler & Fidelity Monitor")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(255, 205, 50)),
                );
                self.render_gate_compiler_panel(ui, quad_w, quad_h - 22.0);
            });

            ui.separator();

            // Quadrant 4: Shot Noise & Fractional Charge Gauge
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Fractional Shot Noise S_I & Quasiparticle Charge Gauge")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(255, 130, 90)),
                );
                self.render_shot_noise_gauge(ui, quad_w, quad_h - 22.0);
            });
        });

        ui.add_space(6.0);
        ui.separator();

        // 3. Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders 2D spacetime worldline braiding trajectory canvas with topological over/under crossings.
    fn render_braiding_worldline_canvas(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let (response, painter) = ui.allocate_painter(vec2(width, height), Sense::click_and_drag());
        let rect = response.rect;

        // Dark background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 16, 24));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(35, 45, 60)),
            egui::StrokeKind::Inside,
        );

        let num_strands = self.synthesis_result.num_strands.max(3);
        let generators = &self.synthesis_result.braid_sequence.generators;
        let num_ops = generators.len().max(1);

        let margin_x = 35.0;
        let margin_y = 28.0;
        let plot_w = rect.width() - 2.0 * margin_x;
        let plot_h = rect.height() - 2.0 * margin_y;

        let strand_spacing = plot_w / (num_strands as f32 - 1.0).max(1.0);
        let time_step_h = plot_h / (num_ops as f32);

        // Track strand horizontal positions over discrete time steps
        // strand_pos[op_idx][strand_idx]
        let mut strand_pos: Vec<Vec<f32>> = Vec::with_capacity(num_ops + 1);
        let mut initial_positions = Vec::with_capacity(num_strands);
        for i in 0..num_strands {
            initial_positions.push(rect.left() + margin_x + (i as f32) * strand_spacing);
        }
        strand_pos.push(initial_positions.clone());

        let mut current_positions = initial_positions;
        for g in generators {
            let mut next_positions = current_positions.clone();
            let s_idx = g.strand_index.saturating_sub(1);
            if s_idx + 1 < num_strands {
                // Exchange strand positions
                next_positions.swap(s_idx, s_idx + 1);
            }
            strand_pos.push(next_positions.clone());
            current_positions = next_positions;
        }

        // Draw time gridlines
        for step in 0..=num_ops {
            let y = rect.top() + margin_y + (step as f32) * time_step_h;
            painter.line_segment(
                [pos2(rect.left() + 8.0, y), pos2(rect.right() - 8.0, y)],
                Stroke::new(0.5, Color32::from_rgb(25, 32, 45)),
            );
        }

        // Draw strands segment by segment with topological over/under distinction
        let num_sub_samples = 24;
        for step in 0..num_ops {
            let y_start = rect.top() + margin_y + (step as f32) * time_step_h;
            let y_end = y_start + time_step_h;
            let g = generators.get(step).copied().unwrap_or(BraidGenerator::sigma(1));
            let s_idx = g.strand_index.saturating_sub(1);

            // Determine which strands cross in this step
            let is_crossing = s_idx + 1 < num_strands;
            let over_strand = if g.direction >= 0 { s_idx } else { s_idx + 1 };
            let under_strand = if g.direction >= 0 { s_idx + 1 } else { s_idx };

            // Render non-crossing straight strands first
            for s in 0..num_strands {
                if is_crossing && (s == s_idx || s == s_idx + 1) {
                    continue; // Rendered below with crossing logic
                }
                let color = STRAND_COLORS[s % STRAND_COLORS.len()];
                let x = strand_pos[step][s];
                let x_next = strand_pos[step + 1][s];
                painter.line_segment(
                    [pos2(x, y_start), pos2(x_next, y_end)],
                    Stroke::new(2.5, color),
                );
            }

            if is_crossing {
                let x_under_start = strand_pos[step][under_strand];
                let x_under_end = strand_pos[step + 1][under_strand];
                let x_over_start = strand_pos[step][over_strand];
                let x_over_end = strand_pos[step + 1][over_strand];
                let color_under = STRAND_COLORS[under_strand % STRAND_COLORS.len()];
                let color_over = STRAND_COLORS[over_strand % STRAND_COLORS.len()];

                // 1. Draw UNDER strand with a small gap around the crossing point (tau in [0.42, 0.58])
                let mut under_pts_1 = Vec::new();
                let mut under_pts_2 = Vec::new();

                for sub in 0..=num_sub_samples {
                    let tau = (sub as f32) / (num_sub_samples as f32);
                    let smooth_tau = 0.5 * (1.0 - (PI as f32 * tau).cos());
                    let x = x_under_start + (x_under_end - x_under_start) * smooth_tau;
                    let y = y_start + time_step_h * tau;

                    if tau < 0.42 {
                        under_pts_1.push(pos2(x, y));
                    } else if tau > 0.58 {
                        under_pts_2.push(pos2(x, y));
                    }
                }

                if under_pts_1.len() >= 2 {
                    for i in 0..(under_pts_1.len() - 1) {
                        painter.line_segment(
                            [under_pts_1[i], under_pts_1[i + 1]],
                            Stroke::new(2.5, color_under),
                        );
                    }
                }
                if under_pts_2.len() >= 2 {
                    for i in 0..(under_pts_2.len() - 1) {
                        painter.line_segment(
                            [under_pts_2[i], under_pts_2[i + 1]],
                            Stroke::new(2.5, color_under),
                        );
                    }
                }

                // 2. Draw OVER strand continuous with full prominence and slight highlight
                let mut over_pts = Vec::new();
                for sub in 0..=num_sub_samples {
                    let tau = (sub as f32) / (num_sub_samples as f32);
                    let smooth_tau = 0.5 * (1.0 - (PI as f32 * tau).cos());
                    let x = x_over_start + (x_over_end - x_over_start) * smooth_tau;
                    let y = y_start + time_step_h * tau;
                    over_pts.push(pos2(x, y));
                }

                // Shadow under the over-strand at crossing center
                let mid_pt = over_pts[num_sub_samples / 2];
                painter.circle_filled(mid_pt, 4.0, Color32::from_rgba_premultiplied(12, 16, 24, 220));

                for i in 0..(over_pts.len() - 1) {
                    painter.line_segment(
                        [over_pts[i], over_pts[i + 1]],
                        Stroke::new(3.0, color_over),
                    );
                }
            }
        }

        // Draw top (t = 0) and bottom (t = T) strand endpoint circles and labels
        for s in 0..num_strands {
            let x_top = strand_pos[0][s];
            let y_top = rect.top() + margin_y;
            let color = STRAND_COLORS[s % STRAND_COLORS.len()];

            painter.circle_filled(pos2(x_top, y_top), 5.0, color);
            painter.text(
                pos2(x_top, y_top - 12.0),
                egui::Align2::CENTER_CENTER,
                format!("{}", s + 1),
                FontId::proportional(11.0),
                color,
            );

            let x_bot = strand_pos[num_ops][s];
            let y_bot = rect.top() + margin_y + (num_ops as f32) * time_step_h;
            painter.circle_filled(pos2(x_bot, y_bot), 5.0, color);
            painter.text(
                pos2(x_bot, y_bot + 12.0),
                egui::Align2::CENTER_CENTER,
                format!("{}", s + 1),
                FontId::proportional(11.0),
                color,
            );
        }

        // Animated playhead marker line showing current animation step
        if self.animation_step <= num_ops {
            let playhead_y = rect.top() + margin_y + (self.animation_step as f32) * time_step_h;
            painter.line_segment(
                [pos2(rect.left() + 10.0, playhead_y), pos2(rect.right() - 10.0, playhead_y)],
                Stroke::new(1.5, Color32::from_rgb(255, 220, 60)),
            );
            painter.text(
                pos2(rect.left() + 20.0, playhead_y - 8.0),
                egui::Align2::LEFT_CENTER,
                format!("Step {}/{}", self.animation_step, num_ops),
                FontId::monospace(10.0),
                Color32::from_rgb(255, 220, 60),
            );
        }
    }

    /// Renders Aharonov-Bohm conductance oscillation plot comparing even vs odd bulk parity.
    fn render_aharonov_bohm_plot(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let delta_b = self.interferometer.oscillation_period_delta_b();
        let pts_active: PlotPoints = self.ab_curve_active.iter().copied().collect();
        let pts_alt: PlotPoints = self.ab_curve_alt.iter().copied().collect();

        let active_label = if self.bulk_anyon_count % 2 == 0 {
            format!("G(B) Even Parity (n_bulk = {})", self.bulk_anyon_count)
        } else {
            format!("G(B) Odd Parity (n_bulk = {})", self.bulk_anyon_count)
        };

        let alt_label = if self.bulk_anyon_count % 2 == 0 {
            "G(B) Odd Parity (Extinguished Visibility)".to_string()
        } else {
            "G(B) Even Parity (Unsuppressed Contrast)".to_string()
        };

        let active_name = format!("{} (Period Delta B = {:.4} T)", active_label, delta_b);
        let plot = Plot::new("fqh_ab_conductance_plot")
            .width(width)
            .height(height)
            .legend(Legend::default())
            .x_axis_label("Magnetic Field B [Tesla]")
            .y_axis_label("Conductance G [e^2 / h]")
            .show_grid(true);

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(active_name, pts_active)
                    .color(Color32::from_rgb(0, 220, 255))
                    .width(2.2),
            );
            plot_ui.line(
                Line::new(alt_label, pts_alt)
                    .color(Color32::from_rgb(255, 120, 100))
                    .width(1.5),
            );
        });
    }

    /// Renders topological quantum gate compiler panel with braid word and fidelity progress.
    fn render_gate_compiler_panel(&mut self, ui: &mut Ui, width: f32, height: f32) {
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_rgb(14, 18, 28))
            .stroke(Stroke::new(1.0, Color32::from_rgb(40, 50, 70)))
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.set_width(width);
                ui.set_height(height);
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

                // Target Gate header
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Target Unitary:").size(12.0).color(Color32::from_rgb(180, 200, 230)));
                    ui.label(
                        RichText::new(self.target_gate.label())
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(255, 215, 60)),
                    );
                    ui.label(RichText::new(format!("({} anyon strands)", self.synthesis_result.num_strands)).size(11.0).color(Color32::GRAY));
                });

                // Compiled Braid Word
                ui.label(RichText::new("Compiled Braid Word rho(sigma):").size(11.0).color(Color32::from_rgb(160, 180, 210)));
                egui::Frame::canvas(ui.style())
                    .fill(Color32::from_rgb(8, 12, 18))
                    .corner_radius(3.0)
                    .show(ui, |ui| {
                        ui.set_width(width - 16.0);
                        ui.label(
                            RichText::new(self.synthesis_result.braid_sequence.format_word())
                                .monospace()
                                .size(12.0)
                                .color(Color32::from_rgb(100, 240, 210)),
                        );
                    });

                // Process Fidelity
                let fid_pct = (self.synthesis_result.fidelity * 100.0).clamp(0.0, 100.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Process Fidelity F:").size(12.0).color(Color32::from_rgb(180, 200, 230)));
                    let fid_color = if fid_pct >= 99.0 {
                        Color32::from_rgb(50, 230, 120)
                    } else {
                        Color32::from_rgb(255, 180, 50)
                    };
                    ui.label(RichText::new(format!("{:.4}%", fid_pct)).size(13.0).strong().color(fid_color));
                    if fid_pct >= 99.0 {
                        ui.label(RichText::new("[PASS >= 99.0%]").size(10.0).color(Color32::from_rgb(50, 230, 120)));
                    }
                });

                ui.add(
                    ProgressBar::new(self.synthesis_result.fidelity as f32)
                        .show_percentage()
                        .animate(false)
                        .fill(if fid_pct >= 99.0 {
                            Color32::from_rgb(50, 230, 120)
                        } else {
                            Color32::from_rgb(255, 180, 50)
                        }),
                );

                // Animation Stepping Controls
                let num_ops = self.synthesis_result.braid_sequence.generators.len().max(1);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Spacetime Step:").size(11.0).color(Color32::from_rgb(180, 200, 230)));
                    ui.label(
                        RichText::new(format!("{}/{}", self.animation_step, num_ops))
                            .monospace()
                            .strong()
                            .color(Color32::from_rgb(255, 205, 50)),
                    );

                    if ui.button("Prev").clicked() {
                        if self.animation_step > 0 {
                            self.animation_step -= 1;
                        } else {
                            self.animation_step = num_ops;
                        }
                    }
                    if ui.button("Next").clicked() {
                        self.animation_step = (self.animation_step + 1) % (num_ops + 1);
                    }
                    if ui.button("Reset").clicked() {
                        self.animation_step = 0;
                    }
                });
            });
    }

    /// Renders shot noise, Fano factor, and fractional charge metrics card.
    fn render_shot_noise_gauge(&mut self, ui: &mut Ui, width: f32, height: f32) {
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_rgb(14, 18, 28))
            .stroke(Stroke::new(1.0, Color32::from_rgb(40, 50, 70)))
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.set_width(width);
                ui.set_height(height);
                ui.spacing_mut().item_spacing = vec2(6.0, 5.0);

                // Quasiparticle Fractional Charge & Fano factor
                let q_charge_ratio = self.filling_fraction.quasiparticle_charge_e();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Quasiparticle Charge e*:").size(12.0).color(Color32::from_rgb(180, 200, 230)));
                    ui.label(
                        RichText::new(format!("{:.3} e", q_charge_ratio))
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(255, 130, 90)),
                    );
                    ui.label(
                        RichText::new(match self.filling_fraction {
                            FillingFraction::Nu5_2 => "(e / 4, Non-Abelian Majorana)",
                            FillingFraction::Nu1_3 => "(e / 3, Laughlin Anyon)",
                            FillingFraction::Nu2_3 => "(e / 3, Particle-Hole Anyon)",
                            FillingFraction::Nu2_5 => "(e / 5, Jain Sequence)",
                        })
                        .size(11.0)
                        .color(Color32::GRAY),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Measured Fano Factor F:").size(12.0).color(Color32::from_rgb(180, 200, 230)));
                    ui.label(
                        RichText::new(format!("{:.4}", self.measured_fano_factor))
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(0, 220, 255)),
                    );
                    ui.label(RichText::new("(S_I / 2eI P(1-P))").size(10.0).color(Color32::GRAY));
                });

                // Quasiparticle Tunneling Probability
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Tunneling Transmission P_tun:").size(11.0).color(Color32::from_rgb(180, 200, 230)));
                    ui.label(
                        RichText::new(format!("{:.3}", self.tunneling_prob))
                            .monospace()
                            .strong()
                            .color(Color32::from_rgb(255, 205, 50)),
                    );
                });
                ui.add(
                    ProgressBar::new(self.tunneling_prob as f32)
                        .show_percentage()
                        .animate(false)
                        .fill(Color32::from_rgb(255, 190, 60)),
                );

                // Shot noise & Johnson-Nyquist thermal floor
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Shot Noise S_I:").size(11.0).color(Color32::from_rgb(180, 200, 230)));
                    ui.label(
                        RichText::new(format!("{:.3e} A^2/Hz", self.shot_noise_density))
                            .monospace()
                            .color(Color32::from_rgb(230, 160, 255)),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Thermal Noise Floor S_th:").size(11.0).color(Color32::from_rgb(180, 200, 230)));
                    ui.label(
                        RichText::new(format!("{:.3e} A^2/Hz", self.thermal_noise_floor))
                            .monospace()
                            .color(Color32::from_rgb(150, 200, 255)),
                    );
                });

                // Parity Visibility Indicator
                let visibility = self.interferometer.bulk_parity_visibility();
                let parity_str = if self.bulk_anyon_count % 2 == 0 {
                    "EVEN PARITY (Full Interference Contrast V = 1.0)"
                } else if self.filling_fraction.is_non_abelian() {
                    "ODD PARITY (Majorana Zero Mode Extinction V = 0.0)"
                } else {
                    "ODD PARITY (Abelian State Preserved V = 1.0)"
                };
                let parity_color = if visibility > 0.5 {
                    Color32::from_rgb(50, 230, 120)
                } else {
                    Color32::from_rgb(255, 90, 80)
                };

                ui.label(
                    RichText::new(parity_str)
                        .size(11.0)
                        .strong()
                        .color(parity_color),
                );
            });
    }

    /// Renders bottom telemetry status bar.
    fn render_telemetry_footer(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(12.0, 2.0);

            // Anyon Model
            ui.label(RichText::new("Model:").size(10.0).color(Color32::GRAY));
            ui.label(
                RichText::new(self.anyon_model.name())
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(0, 220, 255)),
            );

            ui.separator();

            // Topological Gap
            ui.label(RichText::new("Gap Delta:").size(10.0).color(Color32::GRAY));
            ui.label(
                RichText::new(format!("{:.2} meV", self.anyon_model.topological_gap_mev()))
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(100, 240, 210)),
            );

            ui.separator();

            // Yang-Baxter Residual Norm
            ui.label(RichText::new("Yang-Baxter Norm:").size(10.0).color(Color32::GRAY));
            let yb_color = if self.yb_residual_norm < 1e-10 {
                Color32::from_rgb(50, 230, 120)
            } else {
                Color32::from_rgb(255, 90, 80)
            };
            ui.label(
                RichText::new(format!("{:.2e}", self.yb_residual_norm))
                    .size(11.0)
                    .strong()
                    .color(yb_color),
            );

            ui.separator();

            // Gate Fidelity
            ui.label(RichText::new("Gate Fidelity:").size(10.0).color(Color32::GRAY));
            let fid_pct = (self.synthesis_result.fidelity * 100.0).clamp(0.0, 100.0);
            ui.label(
                RichText::new(format!("{:.2}%", fid_pct))
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(255, 215, 60)),
            );

            ui.separator();

            // Fano Factor
            ui.label(RichText::new("Fano Factor:").size(10.0).color(Color32::GRAY));
            ui.label(
                RichText::new(format!("{:.4}", self.measured_fano_factor))
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(255, 130, 90)),
            );

            ui.separator();

            // Parity State
            ui.label(RichText::new("Parity:").size(10.0).color(Color32::GRAY));
            let parity_text = if self.bulk_anyon_count % 2 == 0 { "Even" } else { "Odd" };
            let parity_color = if self.bulk_anyon_count % 2 == 0 {
                Color32::from_rgb(50, 230, 120)
            } else {
                Color32::from_rgb(255, 120, 100)
            };
            ui.label(
                RichText::new(format!("{} (n = {})", parity_text, self.bulk_anyon_count))
                    .size(11.0)
                    .strong()
                    .color(parity_color),
            );
        });
    }
}
