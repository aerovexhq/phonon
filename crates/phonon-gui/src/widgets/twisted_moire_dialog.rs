#![deny(unsafe_code)]

//! Interactive Twisted Bilayer Moiré Studio Visualizer in CAD Studio.
//!
//! Provides:
//! - Real-Space Moiré Interference Superlattice Canvas: 2D hexagonal superlattice rendering
//!   with highlighted AA cores, AB/BA domains, displacement field arrows, and L_M scale bar.
//! - Mini-Brillouin Zone Band Dispersion Plot: native egui_plot rendering polariton dispersion
//!   along Gamma - M - K - Gamma, with glowing flat-band highlight at magic angle.
//! - Density of States (DOS) Plot: plots D(E) displaying giant Van Hove singularity peak.
//! - Localized Soliton Pressure Profile: radial cross-section p(r) of trapped polariton soliton.
//! - Physical Controls: Twist angle slider theta, interlayer shear potentials w_AA, w_AB,
//!   relaxation toggle, soliton amplitude slider, and angle presets.
//! - Telemetry Footer: Twist Angle theta (deg), Moiré Period L_M (nm), Flat Bandwidth Delta E (meV),
//!   Group Velocity Quenching (v_g / v_0), DOS Peak Ratio, Soliton Confinement xi (nm).

use egui::{
    pos2, vec2, Color32, FontId, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::twisted_moire_superlattice::{
    AtomicRelaxationField, BilayerLatticeParams, DensityOfStates, LocalizedAcousticSoliton,
    MoireBandStructure,
};
use std::f64::consts::PI;

/// Palette colors for moiré superlattice visualizer.
const COLOR_AA_CORE: Color32 = Color32::from_rgb(250, 204, 21); // Amber / Gold
const COLOR_AB_DOMAIN: Color32 = Color32::from_rgb(30, 58, 95); // Deep Indigo
const COLOR_BA_DOMAIN: Color32 = Color32::from_rgb(20, 82, 105); // Deep Teal
const COLOR_DOMAIN_WALL: Color32 = Color32::from_rgb(99, 102, 241); // Indigo / Violet
const COLOR_FLAT_BAND: Color32 = Color32::from_rgb(250, 204, 21); // Gold highlight
const COLOR_DISPERSIVE: Color32 = Color32::from_rgb(56, 189, 248); // Cyan
const COLOR_DOS_PEAK: Color32 = Color32::from_rgb(244, 63, 94); // Rose
const COLOR_SOLITON: Color32 = Color32::from_rgb(52, 211, 153); // Emerald

/// Interactive modal dialog for the Twisted Bilayer Moiré Phonon Polariton Studio.
#[derive(Debug, Clone, PartialEq)]
pub struct TwistedMoireDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Controls
    /// Physical parameters for the twisted bilayer superlattice.
    pub params: BilayerLatticeParams,
    /// Atomic relaxation enabled toggle.
    pub relaxation_enabled: bool,
    /// Soliton peak acoustic pressure amplitude p_0 in MPa.
    pub soliton_amplitude_mpa: f64,
    /// Energy window span for DOS calculation in meV.
    pub e_span_mev: f64,

    // Cached Simulation State
    pub relaxation_field: AtomicRelaxationField,
    pub band_structure: MoireBandStructure,
    pub dos: DensityOfStates,
    pub soliton: LocalizedAcousticSoliton,

    // Cached Plot Curves
    pub flat_band_upper_curve: Vec<[f64; 2]>,
    pub flat_band_lower_curve: Vec<[f64; 2]>,
    pub dispersive_curves: Vec<Vec<[f64; 2]>>,
    pub dos_curve: Vec<[f64; 2]>,
    pub soliton_curve: Vec<[f64; 2]>,

    // Telemetry Metrics
    pub twist_angle_deg: f64,
    pub moire_period_nm: f64,
    pub flat_bandwidth_mev: f64,
    pub group_velocity_ratio: f64,
    pub dos_peak_ratio: f64,
    pub soliton_confinement_xi_nm: f64,
    pub soliton_confinement_ratio: f64,
    pub aa_domain_fraction: f64,
    pub isolation_gap_mev: f64,

    pub status_msg: String,
}

impl Default for TwistedMoireDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl TwistedMoireDialog {
    /// Creates a new TwistedMoireDialog initialized with magic-angle graphene parameters.
    pub fn new() -> Self {
        let params = BilayerLatticeParams::default(); // theta = 1.08 deg
        let relaxation_enabled = true;
        let soliton_amplitude_mpa = 5.0;
        let e_span_mev = 25.0;

        let relaxation_field = AtomicRelaxationField::new(params);
        let band_structure = MoireBandStructure::compute(params, relaxation_enabled, 100);
        let dos = DensityOfStates::compute(&band_structure, e_span_mev, 120);
        let soliton = LocalizedAcousticSoliton::solve(&params, soliton_amplitude_mpa, 80);

        let mut dialog = Self {
            is_open: false,
            params,
            relaxation_enabled,
            soliton_amplitude_mpa,
            e_span_mev,

            relaxation_field,
            band_structure,
            dos,
            soliton,

            flat_band_upper_curve: Vec::new(),
            flat_band_lower_curve: Vec::new(),
            dispersive_curves: Vec::new(),
            dos_curve: Vec::new(),
            soliton_curve: Vec::new(),

            twist_angle_deg: 1.08,
            moire_period_nm: 13.05,
            flat_bandwidth_mev: 0.42,
            group_velocity_ratio: 0.02,
            dos_peak_ratio: 24.0,
            soliton_confinement_xi_nm: 3.2,
            soliton_confinement_ratio: 0.24,
            aa_domain_fraction: 0.165,
            isolation_gap_mev: 14.0,

            status_msg: "Magic-angle moiré flat polariton band simulator ready.".to_string(),
        };

        dialog.recompute();
        dialog
    }

    /// Recomputes all continuum elasticity, band structure, DOS, and soliton profiles.
    pub fn recompute(&mut self) {
        self.relaxation_field = AtomicRelaxationField::new(self.params);
        self.band_structure = MoireBandStructure::compute(self.params, self.relaxation_enabled, 100);
        self.dos = DensityOfStates::compute(&self.band_structure, self.e_span_mev, 120);
        self.soliton = LocalizedAcousticSoliton::solve(&self.params, self.soliton_amplitude_mpa, 80);

        // Update telemetry metrics
        self.twist_angle_deg = self.params.theta_deg;
        self.moire_period_nm = self.params.moire_period();
        self.flat_bandwidth_mev = self.band_structure.flat_bandwidth_mev;
        self.group_velocity_ratio = self.band_structure.normalized_group_velocity;
        self.dos_peak_ratio = self.dos.peak_ratio;
        self.soliton_confinement_xi_nm = self.soliton.xi_nm;
        self.soliton_confinement_ratio = self.soliton.confinement_ratio;
        self.aa_domain_fraction = self.relaxation_field.aa_domain_fraction(self.relaxation_enabled);
        self.isolation_gap_mev = self.band_structure.isolation_gap_mev;

        // Populate plot curves
        // 1. Central flat bands (indices 2 and 3)
        self.flat_band_lower_curve = self
            .band_structure
            .points
            .iter()
            .map(|pt| [pt.k_dist, pt.energies[2]])
            .collect();

        self.flat_band_upper_curve = self
            .band_structure
            .points
            .iter()
            .map(|pt| [pt.k_dist, pt.energies[3]])
            .collect();

        // 2. Dispersive and continuum branches (indices 0, 1, 4, 5)
        self.dispersive_curves.clear();
        for band_idx in [0, 1, 4, 5] {
            let curve: Vec<[f64; 2]> = self
                .band_structure
                .points
                .iter()
                .map(|pt| [pt.k_dist, pt.energies[band_idx]])
                .collect();
            self.dispersive_curves.push(curve);
        }

        // 3. Density of States curve
        self.dos_curve = self
            .dos
            .points
            .iter()
            .map(|pt| [pt.energy_mev, pt.dos])
            .collect();

        // 4. Soliton cross section curve
        self.soliton_curve = self.soliton.cross_section.clone();

        self.status_msg = if (self.params.theta_deg - 1.08).abs() < 0.04 {
            format!(
                "Magic angle active (theta = {:.2} deg, Delta_E = {:.2} meV, v_g/v_0 = {:.3})",
                self.params.theta_deg, self.flat_bandwidth_mev, self.group_velocity_ratio
            )
        } else {
            format!(
                "Dispersive regime (theta = {:.2} deg, Delta_E = {:.2} meV, L_M = {:.1} nm)",
                self.params.theta_deg, self.flat_bandwidth_mev, self.moire_period_nm
            )
        };
    }

    /// Renders the modal window.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Twisted Bilayer Moiré Phonon Polariton Studio")
            .open(&mut is_open)
            .default_size([1120.0, 760.0])
            .min_size([880.0, 620.0])
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

            // Twist angle slider
            ui.label(RichText::new("Twist Angle theta:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            let theta_changed = ui
                .add(
                    egui::Slider::new(&mut self.params.theta_deg, 0.8..=2.0)
                        .step_by(0.01)
                        .suffix(" deg"),
                )
                .changed();
            if theta_changed {
                needs_recompute = true;
            }

            // Presets
            if ui.button("Magic (1.08 deg)").clicked() {
                self.params.theta_deg = 1.08;
                needs_recompute = true;
            }
            if ui.button("Large (1.80 deg)").clicked() {
                self.params.theta_deg = 1.80;
                needs_recompute = true;
            }
            if ui.button("Small (0.85 deg)").clicked() {
                self.params.theta_deg = 0.85;
                needs_recompute = true;
            }

            ui.separator();

            // Interlayer coupling sliders
            ui.label(RichText::new("w_AA:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.params.w_aa, 50.0..=120.0)
                        .step_by(1.0)
                        .suffix(" meV"),
                )
                .changed()
            {
                needs_recompute = true;
            }

            ui.label(RichText::new("w_AB:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.params.w_ab, 70.0..=160.0)
                        .step_by(1.0)
                        .suffix(" meV"),
                )
                .changed()
            {
                needs_recompute = true;
            }

            ui.separator();

            // Atomic relaxation toggle
            if ui
                .checkbox(&mut self.relaxation_enabled, "Atomic Relaxation")
                .changed()
            {
                needs_recompute = true;
            }

            // Soliton amplitude
            ui.label(RichText::new("Soliton p_0:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.soliton_amplitude_mpa, 0.5..=10.0)
                        .step_by(0.1)
                        .suffix(" MPa"),
                )
                .changed()
            {
                needs_recompute = true;
            }
        });

        if needs_recompute {
            self.recompute();
        }

        ui.add_space(4.0);

        // Status banner
        ui.horizontal(|ui| {
            let is_magic = (self.params.theta_deg - 1.08).abs() < 0.04;
            let dot_color = if is_magic {
                COLOR_AA_CORE
            } else {
                Color32::from_rgb(140, 160, 180)
            };
            ui.label(RichText::new("o").color(dot_color).size(12.0));
            ui.label(
                RichText::new(&self.status_msg)
                    .size(11.0)
                    .color(Color32::from_rgb(200, 215, 230)),
            );
        });

        ui.separator();

        // 2. 2x2 Dashboard Layout:
        // Left Column: Real-Space Superlattice Canvas & Soliton Pressure Profile
        // Right Column: Mini-BZ Band Dispersion Plot & Density of States Plot
        let total_avail_h = (ui.available_height() - 75.0).max(400.0);
        let quadrant_h = (total_avail_h * 0.5).max(180.0);

        ui.columns(2, |cols| {
            // Left Column
            // Top: Real-Space Moiré Canvas
            cols[0].group(|ui| {
                ui.set_height(quadrant_h);
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("Real-Space Moiré Superlattice Canvas")
                            .size(11.0)
                            .color(Color32::from_rgb(180, 210, 240)),
                    );
                    let relax_str = if self.relaxation_enabled {
                        format!("Relaxed (AA: {:.1}%)", self.aa_domain_fraction * 100.0)
                    } else {
                        "Unrelaxed (AA: 33.3%)".to_string()
                    };
                    ui.label(
                        RichText::new(format!("[{}]", relax_str))
                            .size(10.0)
                            .color(COLOR_AA_CORE),
                    );
                });
                self.render_real_space_canvas(ui, quadrant_h - 28.0);
            });

            // Bottom: Localized Soliton Pressure Profile
            cols[0].group(|ui| {
                ui.set_height(quadrant_h);
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("Trapped Acoustic Soliton Pressure Profile p(r)")
                            .size(11.0)
                            .color(Color32::from_rgb(180, 210, 240)),
                    );
                    ui.label(
                        RichText::new(format!(
                            "[xi = {:.2} nm ({:.1}% L_M)]",
                            self.soliton_confinement_xi_nm, self.soliton_confinement_ratio * 100.0
                        ))
                        .size(10.0)
                        .color(COLOR_SOLITON),
                    );
                });
                self.render_soliton_plot(ui, quadrant_h - 28.0);
            });

            // Right Column
            // Top: Mini-BZ Dispersion Plot
            cols[1].group(|ui| {
                ui.set_height(quadrant_h);
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("Mini-Brillouin Zone Band Dispersion (Gamma-M-K-Gamma)")
                            .size(11.0)
                            .color(Color32::from_rgb(180, 210, 240)),
                    );
                    let flat_str = format!("Delta_E = {:.2} meV", self.flat_bandwidth_mev);
                    let is_magic = self.flat_bandwidth_mev < 1.0;
                    let text_color = if is_magic {
                        COLOR_AA_CORE
                    } else {
                        COLOR_DISPERSIVE
                    };
                    ui.label(RichText::new(format!("[{}]", flat_str)).size(10.0).color(text_color));
                });
                self.render_dispersion_plot(ui, quadrant_h - 28.0);
            });

            // Bottom: Density of States Plot
            cols[1].group(|ui| {
                ui.set_height(quadrant_h);
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new("Density of States D(E) & Van Hove Singularity")
                            .size(11.0)
                            .color(Color32::from_rgb(180, 210, 240)),
                    );
                    ui.label(
                        RichText::new(format!("[Peak Ratio: {:.1}x]", self.dos_peak_ratio))
                            .size(10.0)
                            .color(COLOR_DOS_PEAK),
                    );
                });
                self.render_dos_plot(ui, quadrant_h - 28.0);
            });
        });

        ui.add_space(4.0);
        ui.separator();

        // 3. Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders the 2D Real-Space Moiré Canvas.
    fn render_real_space_canvas(&self, ui: &mut Ui, h: f32) {
        let w = ui.available_width().max(100.0);
        let (response, painter) = ui.allocate_painter(vec2(w, h), Sense::hover());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(10, 16, 26));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(30, 48, 72)),
            StrokeKind::Inside,
        );

        let center = rect.center();
        let lm = self.moire_period_nm;

        // Visual scale: map 2.5 * L_M to canvas width
        let px_per_nm = (w * 0.35) / lm as f32;

        // Draw hexagonal superlattice nodes and domains
        // Hexagonal basis vectors in pixels:
        let a1_px = vec2(lm as f32 * px_per_nm, 0.0);
        let a2_px = vec2(
            0.5 * lm as f32 * px_per_nm,
            3.0_f32.sqrt() * 0.5 * lm as f32 * px_per_nm,
        );

        // Draw AB/BA domain background cells
        let r_aa_nm = if self.relaxation_enabled {
            lm * (self.aa_domain_fraction / PI).sqrt()
        } else {
            lm * (0.3333 / PI).sqrt()
        };
        let r_aa_px = (r_aa_nm as f32 * px_per_nm).max(4.0);

        // Grid of nodes around center
        let range = -2..=2;
        for i in range.clone() {
            for j in range.clone() {
                let node_pos = center + a1_px * (i as f32) + a2_px * (j as f32);
                if !rect.contains(node_pos) {
                    continue;
                }

                // AB and BA domain halos
                let r_domain_px = lm as f32 * 0.45 * px_per_nm;
                painter.circle_filled(
                    node_pos + a1_px * 0.33 + a2_px * 0.33,
                    r_domain_px * 0.7,
                    COLOR_AB_DOMAIN,
                );
                painter.circle_filled(
                    node_pos - a1_px * 0.33 - a2_px * 0.33,
                    r_domain_px * 0.7,
                    COLOR_BA_DOMAIN,
                );

                // Domain wall boundary stroke
                painter.circle_stroke(
                    node_pos,
                    r_aa_px * 1.8,
                    Stroke::new(1.0, COLOR_DOMAIN_WALL),
                );

                // AA site core circle
                painter.circle_filled(node_pos, r_aa_px, COLOR_AA_CORE);
                painter.circle_stroke(
                    node_pos,
                    r_aa_px,
                    Stroke::new(1.5, Color32::from_rgb(254, 240, 138)),
                );

                // Small center dot
                painter.circle_filled(node_pos, 2.0, Color32::from_rgb(180, 83, 9));
            }
        }

        // Draw displacement field vectors u(r) around central AA site
        let num_arrows = 8;
        let arrow_radius_nm = lm * 0.35;
        for k in 0..num_arrows {
            let phi = (k as f64) * (2.0 * PI / num_arrows as f64);
            let rx = arrow_radius_nm * phi.cos();
            let ry = arrow_radius_nm * phi.sin();
            let u = self.relaxation_field.displacement_at(rx, ry);

            let pt_start = center + vec2((rx as f32) * px_per_nm, (ry as f32) * px_per_nm);
            // Amplify displacement for visualization
            let amp = 250.0;
            let pt_end = pt_start + vec2((u[0] as f32) * amp, (u[1] as f32) * amp);

            painter.line_segment([pt_start, pt_end], Stroke::new(1.2, Color32::from_rgb(236, 72, 153)));
            painter.circle_filled(pt_start, 1.5, Color32::from_rgb(244, 114, 182));
        }

        // Scale bar in bottom-left
        let scale_nm = if lm > 12.0 { 10.0 } else { 5.0 };
        let bar_len_px = (scale_nm as f32) * px_per_nm;
        let bar_start = pos2(rect.min.x + 12.0, rect.max.y - 16.0);
        let bar_end = pos2(bar_start.x + bar_len_px, bar_start.y);

        painter.line_segment([bar_start, bar_end], Stroke::new(2.5, Color32::WHITE));
        painter.line_segment(
            [pos2(bar_start.x, bar_start.y - 3.0), pos2(bar_start.x, bar_start.y + 3.0)],
            Stroke::new(2.0, Color32::WHITE),
        );
        painter.line_segment(
            [pos2(bar_end.x, bar_end.y - 3.0), pos2(bar_end.x, bar_end.y + 3.0)],
            Stroke::new(2.0, Color32::WHITE),
        );
        painter.text(
            pos2(bar_start.x, bar_start.y - 12.0),
            egui::Align2::LEFT_CENTER,
            format!("Scale: {:.0} nm (L_M = {:.2} nm)", scale_nm, lm),
            FontId::monospace(10.0),
            Color32::WHITE,
        );

        // Stacking Legend in top-right
        let leg_x = rect.max.x - 110.0;
        let mut leg_y = rect.min.y + 12.0;

        painter.circle_filled(pos2(leg_x, leg_y), 4.0, COLOR_AA_CORE);
        painter.text(
            pos2(leg_x + 8.0, leg_y),
            egui::Align2::LEFT_CENTER,
            "AA site",
            FontId::proportional(10.0),
            COLOR_AA_CORE,
        );

        leg_y += 14.0;
        painter.circle_filled(pos2(leg_x, leg_y), 4.0, COLOR_AB_DOMAIN);
        painter.text(
            pos2(leg_x + 8.0, leg_y),
            egui::Align2::LEFT_CENTER,
            "AB domain",
            FontId::proportional(10.0),
            Color32::from_rgb(147, 197, 253),
        );

        leg_y += 14.0;
        painter.circle_filled(pos2(leg_x, leg_y), 4.0, COLOR_BA_DOMAIN);
        painter.text(
            pos2(leg_x + 8.0, leg_y),
            egui::Align2::LEFT_CENTER,
            "BA domain",
            FontId::proportional(10.0),
            Color32::from_rgb(110, 231, 183),
        );
    }

    /// Renders the Mini-Brillouin Zone Band Dispersion Plot.
    fn render_dispersion_plot(&self, ui: &mut Ui, h: f32) {
        let plot = Plot::new("moire_dispersion_plot")
            .legend(Legend::default())
            .x_axis_label("Wavevector Path (Gamma - M - K - Gamma)")
            .y_axis_label("Energy (meV)")
            .height(h)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            // Dispersive background branches
            for (idx, curve) in self.dispersive_curves.iter().enumerate() {
                let name = if idx == 0 {
                    "Dispersive Polariton Bands"
                } else {
                    ""
                };
                plot_ui.line(
                    Line::new(name, PlotPoints::new(curve.clone()))
                        .color(Color32::from_rgb(70, 110, 150))
                        .width(1.2),
                );
            }

            // Glowing flat bands at magic angle
            let is_magic = self.flat_bandwidth_mev < 1.0;
            let flat_color = if is_magic {
                COLOR_FLAT_BAND
            } else {
                COLOR_DISPERSIVE
            };
            let flat_width = if is_magic { 2.8 } else { 1.8 };

            plot_ui.line(
                Line::new("Flat Band (Upper)", PlotPoints::new(self.flat_band_upper_curve.clone()))
                    .color(flat_color)
                    .width(flat_width),
            );

            plot_ui.line(
                Line::new("Flat Band (Lower)", PlotPoints::new(self.flat_band_lower_curve.clone()))
                    .color(flat_color)
                    .width(flat_width),
            );

            // Zero energy line
            plot_ui.hline(HLine::new("E = 0", 0.0).color(Color32::from_rgb(60, 75, 95)));
        });
    }

    /// Renders the Density of States Plot.
    fn render_dos_plot(&self, ui: &mut Ui, h: f32) {
        let plot = Plot::new("moire_dos_plot")
            .legend(Legend::default())
            .x_axis_label("Energy (meV)")
            .y_axis_label("D(E) states/(meV*cell)")
            .height(h)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("D(E)", PlotPoints::new(self.dos_curve.clone()))
                    .color(COLOR_DOS_PEAK)
                    .width(2.0),
            );

            // Peak annotation line
            plot_ui.vline(
                VLine::new("Van Hove Singularity", self.dos.peak_energy_mev)
                    .color(Color32::from_rgb(251, 191, 36)),
            );

            // Background level reference
            plot_ui.hline(
                HLine::new("Background DOS", self.dos.background_dos)
                    .color(Color32::from_rgb(75, 85, 99)),
            );
        });
    }

    /// Renders the Localized Soliton Pressure Profile.
    fn render_soliton_plot(&self, ui: &mut Ui, h: f32) {
        let plot = Plot::new("moire_soliton_plot")
            .legend(Legend::default())
            .x_axis_label("Radial Coordinate r (nm)")
            .y_axis_label("Pressure p(r) (MPa)")
            .height(h)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("p(r) = p_0 * sech(r / xi)", PlotPoints::new(self.soliton_curve.clone()))
                    .color(COLOR_SOLITON)
                    .width(2.2),
            );

            // Confinement boundaries at +/- xi
            plot_ui.vline(
                VLine::new("+xi (Localization Length)", self.soliton.xi_nm)
                    .color(Color32::from_rgb(245, 158, 11)),
            );
            plot_ui.vline(
                VLine::new("-xi (Localization Length)", -self.soliton.xi_nm)
                    .color(Color32::from_rgb(245, 158, 11)),
            );

            // Half-power / 1/e reference line
            let p_half = self.soliton_amplitude_mpa / 1.0_f64.cosh();
            plot_ui.hline(HLine::new("p(xi) = p_0 * sech(1)", p_half).color(Color32::from_rgb(75, 85, 99)));
        });
    }

    /// Renders the Telemetry Footer with all 6 roadmap physical metrics.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 4.0);

            // 1. Twist Angle theta
            ui.group(|ui| {
                ui.label(RichText::new("Twist Angle:").size(10.0).color(Color32::from_rgb(150, 175, 200)));
                ui.label(
                    RichText::new(format!("{:.2} deg", self.twist_angle_deg))
                        .size(11.0)
                        .color(Color32::WHITE)
                        .strong(),
                );
            });

            // 2. Moiré Period L_M
            ui.group(|ui| {
                ui.label(RichText::new("Moiré Period L_M:").size(10.0).color(Color32::from_rgb(150, 175, 200)));
                ui.label(
                    RichText::new(format!("{:.2} nm", self.moire_period_nm))
                        .size(11.0)
                        .color(Color32::from_rgb(147, 197, 253))
                        .strong(),
                );
            });

            // 3. Flat Bandwidth Delta E
            ui.group(|ui| {
                ui.label(RichText::new("Bandwidth Delta E:").size(10.0).color(Color32::from_rgb(150, 175, 200)));
                let is_flat = self.flat_bandwidth_mev < 1.0;
                let color = if is_flat {
                    COLOR_AA_CORE
                } else {
                    Color32::from_rgb(248, 113, 113)
                };
                ui.label(
                    RichText::new(format!("{:.2} meV", self.flat_bandwidth_mev))
                        .size(11.0)
                        .color(color)
                        .strong(),
                );
            });

            // 4. Group Velocity Quenching v_g / v_0
            ui.group(|ui| {
                ui.label(RichText::new("Velocity Quenching:").size(10.0).color(Color32::from_rgb(150, 175, 200)));
                let is_quenched = self.group_velocity_ratio <= 0.05;
                let color = if is_quenched {
                    COLOR_SOLITON
                } else {
                    Color32::from_rgb(248, 113, 113)
                };
                ui.label(
                    RichText::new(format!("{:.3} (v_g/v_0)", self.group_velocity_ratio))
                        .size(11.0)
                        .color(color)
                        .strong(),
                );
            });

            // 5. DOS Peak Ratio
            ui.group(|ui| {
                ui.label(RichText::new("DOS Peak Ratio:").size(10.0).color(Color32::from_rgb(150, 175, 200)));
                ui.label(
                    RichText::new(format!("{:.1}x", self.dos_peak_ratio))
                        .size(11.0)
                        .color(COLOR_DOS_PEAK)
                        .strong(),
                );
            });

            // 6. Soliton Confinement xi
            ui.group(|ui| {
                ui.label(RichText::new("Soliton xi:").size(10.0).color(Color32::from_rgb(150, 175, 200)));
                ui.label(
                    RichText::new(format!(
                        "{:.2} nm ({:.1}% L_M)",
                        self.soliton_confinement_xi_nm, self.soliton_confinement_ratio * 100.0
                    ))
                    .size(11.0)
                    .color(COLOR_SOLITON)
                    .strong(),
                );
            });
        });
    }
}
