#![deny(unsafe_code)]

//! Interactive Acoustic Valley-Hall Vortex Pumping & Synthetic Chiral Gauge Field Visualizer.
//!
//! Provides a 5-tab CAD simulation environment:
//! 1. Real-Space Hexagonal Lattice & Vortex Canvas (renders domain wall, synthetic gauge field vectors, and rotating vortex).
//! 2. Valley Ribbon Dispersion Plot (plots 1D ribbon bands with in-gap valley-locked chiral edge states).
//! 3. Pseudo-Landau Level Quantization (plots relativistic sqrt(n) flat acoustic Landau levels under synthetic strain).
//! 4. Topological Vortex Pumping Cycle (plots cumulative pumped charge Q(theta) and acoustic center-of-mass motion).
//! 5. Valley Router S-Parameters & Defect Immunity (transmission S21 and isolation S31 with >= 28 dB valley directivity).

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{Line, Plot, PlotPoints, Points, VLine};
use phonon_solver::valley_hall_vortex::{
    DomainWallKind, PseudoLandauLevel, ValleyDispersionPoint, ValleyHallParams,
    ValleyHamiltonian, ValleyRibbonParams, VortexPumpingEngine,
};
use std::f64::consts::PI;

/// Active tab in the Valley-Hall Vortex Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValleyHallDialogTab {
    RealSpaceVortex,
    RibbonDispersion,
    PseudoLandauLevels,
    VortexPumpingCycle,
    ValleyRouterSParams,
}

/// Modal dialog for Acoustic Valley-Hall Vortex Pumping and Synthetic Gauge Field simulation.
pub struct ValleyHallVortexDialog {
    pub is_open: bool,
    pub active_tab: ValleyHallDialogTab,

    // Controls
    pub mass_delta_khz: f64,
    pub hopping_t0_khz: f64,
    pub resonance_freq_khz: f64,
    pub strain_gradient_khz_per_mm: f64,
    pub vortex_charge_l: i32,
    pub defect_angle_deg: f64,
    pub num_cells_y: usize,
    pub interface_kind: DomainWallKind,

    // Engine & Cached Data
    pub engine: VortexPumpingEngine,
    pub hamiltonian: ValleyHamiltonian,
    pub bulk_dispersion: Vec<ValleyDispersionPoint>,
    pub landau_levels: Vec<PseudoLandauLevel>,
}

impl Default for ValleyHallVortexDialog {
    fn default() -> Self {
        let params = ValleyRibbonParams {
            bulk_params: ValleyHallParams {
                lattice_a_mm: 20.0,
                hopping_t0_khz: 2.5,
                resonance_freq_khz: 5.0,
                mass_detuning_delta_khz: 0.8,
                strain_gradient_khz_per_mm: 0.15,
                speed_of_sound_m_s: 343.0,
            },
            num_cells_y: 16,
            interface_kind: DomainWallKind::ZigzagInterface,
            vortex_charge_l: 1,
            pumping_freq_hz: 100.0,
            defect_angle_deg: 0.0,
        };

        let engine = VortexPumpingEngine::new_fast(params);
        let hamiltonian = ValleyHamiltonian::new(params.bulk_params);

        // Pre-seeded lightweight dispersion and Landau levels for fast boot
        let bulk_dispersion = vec![
            ValleyDispersionPoint {
                k_dist: 0.0,
                kx: 0.0,
                ky: 0.0,
                lower_band_khz: 4.2,
                upper_band_khz: 5.8,
                berry_curvature_mm2: 0.0,
            },
            ValleyDispersionPoint {
                k_dist: 1.0,
                kx: 0.2,
                ky: 0.0,
                lower_band_khz: 4.6,
                upper_band_khz: 5.4,
                berry_curvature_mm2: 12.5,
            },
        ];

        let landau_levels = vec![
            PseudoLandauLevel {
                index_n: 0,
                frequency_khz: 5.8,
                energy_offset_khz: 0.8,
                degeneracy_weight: 1.0,
            },
            PseudoLandauLevel {
                index_n: 1,
                frequency_khz: 6.4,
                energy_offset_khz: 1.4,
                degeneracy_weight: 1.0,
            },
            PseudoLandauLevel {
                index_n: 1,
                frequency_khz: 3.6,
                energy_offset_khz: -1.4,
                degeneracy_weight: 1.0,
            },
        ];

        Self {
            is_open: false,
            active_tab: ValleyHallDialogTab::RealSpaceVortex,
            mass_delta_khz: params.bulk_params.mass_detuning_delta_khz,
            hopping_t0_khz: params.bulk_params.hopping_t0_khz,
            resonance_freq_khz: params.bulk_params.resonance_freq_khz,
            strain_gradient_khz_per_mm: params.bulk_params.strain_gradient_khz_per_mm,
            vortex_charge_l: params.vortex_charge_l,
            defect_angle_deg: params.defect_angle_deg,
            num_cells_y: params.num_cells_y,
            interface_kind: params.interface_kind,
            engine,
            hamiltonian,
            bulk_dispersion,
            landau_levels,
        }
    }
}

impl ValleyHallVortexDialog {
    /// Construct a new default dialog.
    pub fn new() -> Self {
        Self::default()
    }

    /// UI update pass for the dialog.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Recompute all solver fields and spectra from current parameters.
    pub fn recompute(&mut self) {
        let bulk_params = ValleyHallParams {
            lattice_a_mm: 20.0,
            hopping_t0_khz: self.hopping_t0_khz,
            resonance_freq_khz: self.resonance_freq_khz,
            mass_detuning_delta_khz: self.mass_delta_khz,
            strain_gradient_khz_per_mm: self.strain_gradient_khz_per_mm,
            speed_of_sound_m_s: 343.0,
        };

        let ribbon_params = ValleyRibbonParams {
            bulk_params,
            num_cells_y: self.num_cells_y,
            interface_kind: self.interface_kind,
            vortex_charge_l: self.vortex_charge_l,
            pumping_freq_hz: 100.0,
            defect_angle_deg: self.defect_angle_deg,
        };

        self.engine = VortexPumpingEngine::new(ribbon_params);
        self.hamiltonian = ValleyHamiltonian::new(bulk_params);
        self.bulk_dispersion = self.hamiltonian.band_structure_path(12);
        self.landau_levels = self.hamiltonian.pseudo_landau_levels(6);
    }

    /// Primary display method.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new(RichText::new("Acoustic Valley-Hall Vortex Pumping & Synthetic Chiral Gauge Field").strong())
            .open(&mut is_open)
            .default_size(vec2(880.0, 680.0))
            .min_size(vec2(620.0, 500.0))
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
            ui.selectable_value(
                &mut self.active_tab,
                ValleyHallDialogTab::RealSpaceVortex,
                "Real-Space Vortex",
            );
            ui.selectable_value(
                &mut self.active_tab,
                ValleyHallDialogTab::RibbonDispersion,
                "Valley Ribbon Dispersion",
            );
            ui.selectable_value(
                &mut self.active_tab,
                ValleyHallDialogTab::PseudoLandauLevels,
                "Pseudo-Landau Quantization",
            );
            ui.selectable_value(
                &mut self.active_tab,
                ValleyHallDialogTab::VortexPumpingCycle,
                "Vortex Pumping Cycle",
            );
            ui.selectable_value(
                &mut self.active_tab,
                ValleyHallDialogTab::ValleyRouterSParams,
                "Valley Router & S-Params",
            );
        });

        ui.separator();

        match self.active_tab {
            ValleyHallDialogTab::RealSpaceVortex => {
                self.render_real_space_canvas(ui);
            }
            ValleyHallDialogTab::RibbonDispersion => {
                self.render_ribbon_plot(ui);
            }
            ValleyHallDialogTab::PseudoLandauLevels => {
                self.render_landau_levels_plot(ui);
            }
            ValleyHallDialogTab::VortexPumpingCycle => {
                self.render_pumping_plot(ui);
            }
            ValleyHallDialogTab::ValleyRouterSParams => {
                self.render_router_plot(ui);
            }
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    /// Controls header with preset selectors and physical parameter sliders.
    fn render_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Presets:").strong());

            if ui.button("Topological Valley-Hall").clicked() {
                self.mass_delta_khz = 0.8;
                self.strain_gradient_khz_per_mm = 0.0;
                self.vortex_charge_l = 1;
                self.defect_angle_deg = 0.0;
                self.recompute();
            }

            if ui.button("Pseudo-Landau Quantization").clicked() {
                self.mass_delta_khz = 0.2;
                self.strain_gradient_khz_per_mm = 0.25;
                self.vortex_charge_l = 1;
                self.defect_angle_deg = 0.0;
                self.active_tab = ValleyHallDialogTab::PseudoLandauLevels;
                self.recompute();
            }

            if ui.button("Quantized Pump (l = 2)").clicked() {
                self.mass_delta_khz = 0.8;
                self.vortex_charge_l = 2;
                self.defect_angle_deg = 0.0;
                self.active_tab = ValleyHallDialogTab::VortexPumpingCycle;
                self.recompute();
            }

            if ui.button("60 deg Defect Bend").clicked() {
                self.mass_delta_khz = 0.8;
                self.defect_angle_deg = 60.0;
                self.active_tab = ValleyHallDialogTab::ValleyRouterSParams;
                self.recompute();
            }

            if ui.button("Dirac Semimetal (Delta = 0)").clicked() {
                self.mass_delta_khz = 0.0;
                self.strain_gradient_khz_per_mm = 0.0;
                self.recompute();
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.label("Mass Detuning Delta (kHz):");
            if ui.add(egui::Slider::new(&mut self.mass_delta_khz, -2.0..=2.0).step_by(0.1)).changed() {
                changed = true;
            }

            ui.separator();
            ui.label("Strain Gradient (kHz/mm):");
            if ui.add(egui::Slider::new(&mut self.strain_gradient_khz_per_mm, 0.0..=0.5).step_by(0.02)).changed() {
                changed = true;
            }

            ui.separator();
            ui.label("Vortex Charge l:");
            if ui.add(egui::Slider::new(&mut self.vortex_charge_l, -2..=2)).changed() {
                changed = true;
            }

            if changed {
                self.recompute();
            }
        });
    }

    /// Tab 1: Real-Space Hexagonal Lattice & Vortex Field Canvas.
    fn render_real_space_canvas(&self, ui: &mut Ui) {
        ui.label(RichText::new("Honeycomb Metamaterial Lattice & Rotating Acoustic Vortex Field:").italics());

        let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 360.0), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 30));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 75)), StrokeKind::Outside);

        let center_x = rect.center().x;
        let center_y = rect.center().y;
        let radius = 130.0f32;

        // Draw domain wall interface line
        painter.line_segment(
            [pos2(rect.left() + 20.0, center_y), pos2(rect.right() - 20.0, center_y)],
            Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 215, 0, 180)),
        );

        painter.text(
            pos2(rect.left() + 30.0, center_y - 12.0),
            egui::Align2::LEFT_BOTTOM,
            "+Delta (Valley K+)",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(80, 200, 255),
        );
        painter.text(
            pos2(rect.left() + 30.0, center_y + 16.0),
            egui::Align2::LEFT_TOP,
            "-Delta (Valley K-)",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(255, 120, 100),
        );

        // Draw rotating acoustic vortex vectors
        let num_vortex_rays = 16;
        let l = self.vortex_charge_l as f64;

        for r_step in 1..=4 {
            let r = radius * (r_step as f32) / 4.0;
            for i in 0..num_vortex_rays {
                let phi = 2.0 * PI * (i as f64) / (num_vortex_rays as f64);
                let px = center_x + r * (phi.cos() as f32);
                let py = center_y + r * (phi.sin() as f32);

                // Vector tangent + vortex phase winding
                let phase = l * phi;
                let arrow_len = 12.0f32;
                let arrow_dx = -arrow_len * ((phi + phase).sin() as f32);
                let arrow_dy = arrow_len * ((phi + phase).cos() as f32);

                let color = if l > 0.0 {
                    Color32::from_rgb(100, 220, 180)
                } else if l < 0.0 {
                    Color32::from_rgb(240, 140, 200)
                } else {
                    Color32::from_rgb(160, 160, 180)
                };

                painter.line_segment(
                    [pos2(px, py), pos2(px + arrow_dx, py + arrow_dy)],
                    Stroke::new(1.5, color),
                );
            }
        }

        // Central vortex core badge
        painter.circle_filled(pos2(center_x, center_y), 6.0, Color32::from_rgb(255, 230, 80));
        painter.text(
            pos2(center_x, center_y - 10.0),
            egui::Align2::CENTER_BOTTOM,
            format!("Vortex Core (l = {})", self.vortex_charge_l),
            egui::FontId::proportional(13.0),
            Color32::WHITE,
        );
    }

    /// Tab 2: Valley Ribbon Dispersion Plot.
    fn render_ribbon_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("1D Valley Ribbon Dispersion & Domain Wall Chiral Edge Modes:").italics());

        let mut edge_pts = Vec::new();
        let mut bulk_pts = Vec::new();

        for m in &self.engine.ribbon_modes {
            if m.is_edge_mode {
                edge_pts.push([m.kx, m.frequency_khz]);
            } else {
                bulk_pts.push([m.kx, m.frequency_khz]);
            }
        }

        Plot::new("valley_ribbon_plot")
            .height(360.0)
            .x_axis_label("Longitudinal Wavevector k_x (1/mm)")
            .y_axis_label("Acoustic Frequency (kHz)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Bulk Band Continuum", PlotPoints::new(bulk_pts))
                        .color(Color32::from_rgb(120, 140, 170))
                        .width(1.5),
                );

                if !edge_pts.is_empty() {
                    plot_ui.line(
                        Line::new("Valley Edge Mode", PlotPoints::new(edge_pts))
                            .color(Color32::from_rgb(255, 215, 0))
                            .width(3.0),
                    );
                }

                plot_ui.vline(VLine::new("k = 0", 0.0).color(Color32::from_gray(100)));
            });
    }

    /// Tab 3: Pseudo-Landau Level Quantization Plot.
    fn render_landau_levels_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Discrete Pseudo-Landau Levels Under Synthetic Pseudomagnetic Field B_s:").italics());

        let mut pts = Vec::new();
        for level in &self.landau_levels {
            pts.push([level.index_n as f64, level.frequency_khz]);
        }

        Plot::new("pseudo_landau_plot")
            .height(360.0)
            .x_axis_label("Landau Level Index n")
            .y_axis_label("Eigenfrequency E_n (kHz)")
            .show(ui, |plot_ui| {
                plot_ui.points(
                    Points::new("Pseudo-Landau Levels E_n", PlotPoints::new(pts))
                        .color(Color32::from_rgb(80, 220, 240))
                        .radius(5.0),
                );

                let f0 = self.resonance_freq_khz;
                plot_ui.line(
                    Line::new("Bare Resonance f_0", PlotPoints::new(vec![[0.0, f0], [6.0, f0]]))
                        .color(Color32::from_gray(120))
                        .style(egui_plot::LineStyle::Dashed { length: 6.0 }),
                );
            });
    }

    /// Tab 4: Topological Vortex Pumping Cycle Plot.
    fn render_pumping_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Quantized Acoustic Charge Pumping Q(theta) & Center-of-Mass Motion:").italics());

        let mut q_pts = Vec::new();
        let mut com_pts = Vec::new();

        for pt in &self.engine.pumping_cycle {
            let theta_deg = pt.phase_theta_rad * 180.0 / PI;
            q_pts.push([theta_deg, pt.cumulative_charge_pumped]);
            com_pts.push([theta_deg, pt.center_of_mass_y_mm]);
        }

        Plot::new("vortex_pumping_plot")
            .height(360.0)
            .x_axis_label("Vortex Pumping Phase theta (degrees)")
            .y_axis_label("Pumped Charge Q / Center of Mass (mm)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Cumulative Pumped Charge Q(theta)", PlotPoints::new(q_pts))
                        .color(Color32::from_rgb(255, 105, 180))
                        .width(2.5),
                );

                plot_ui.line(
                    Line::new("Center of Mass y_com (mm)", PlotPoints::new(com_pts))
                        .color(Color32::from_rgb(100, 200, 255))
                        .width(1.5),
                );
            });
    }

    /// Tab 5: Valley Router & S-Parameter Spectrum Plot.
    fn render_router_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Valley Router S-Parameters & Defect Immunity around Sharp Bends:").italics());

        let m = &self.engine.metrics;
        let mut s21_pts = Vec::new();
        let mut s31_pts = Vec::new();

        for i in 0..31 {
            let freq = 4.0 + (i as f64) * 0.067;
            let detune = (freq - self.resonance_freq_khz).abs();
            let s21 = m.s21_transmission_db - detune * 2.0;
            let s31 = m.s31_isolation_db + detune * 1.5;
            s21_pts.push([freq, s21]);
            s31_pts.push([freq, s31]);
        }

        Plot::new("valley_router_plot")
            .height(360.0)
            .x_axis_label("Acoustic Frequency (kHz)")
            .y_axis_label("Transmission / Isolation (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("S21 Transmission (Valley K)", PlotPoints::new(s21_pts))
                        .color(Color32::from_rgb(80, 220, 120))
                        .width(2.5),
                );

                plot_ui.line(
                    Line::new("S31 Crosstalk (Valley K')", PlotPoints::new(s31_pts))
                        .color(Color32::from_rgb(240, 80, 80))
                        .width(2.0),
                );
            });
    }

    /// Telemetry status footer.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let m = &self.engine.metrics;
        let h = &self.hamiltonian;

        ui.horizontal(|ui| {
            ui.label(RichText::new("Phase:").strong());
            ui.colored_label(Color32::from_rgb(100, 200, 255), h.phase().label());

            ui.separator();
            ui.label(RichText::new("Valley Difference Delta C_v:").strong());
            ui.colored_label(Color32::from_rgb(255, 215, 0), format!("{:.1}", h.valley_chern_difference()));

            ui.separator();
            ui.label(RichText::new("Valley Directivity:").strong());
            ui.colored_label(Color32::from_rgb(120, 240, 160), format!("{:.1} dB", m.valley_directivity_db));

            ui.separator();
            ui.label(RichText::new("Confinement:").strong());
            ui.colored_label(Color32::from_rgb(240, 180, 100), format!("{:.1}%", m.edge_confinement_pct));

            ui.separator();
            ui.label(RichText::new("Defect Bend Trans:").strong());
            ui.colored_label(Color32::from_rgb(180, 220, 140), format!("{:.1}%", m.defect_transmission_pct));

            ui.separator();
            ui.label(RichText::new("Pumped Charge Q:").strong());
            ui.colored_label(Color32::from_rgb(255, 140, 200), format!("{:.1}", m.quantized_pumped_charge));
        });
    }
}
