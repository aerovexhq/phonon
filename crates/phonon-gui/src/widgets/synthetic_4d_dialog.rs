#![deny(unsafe_code)]

//! Interactive Topological Acoustic Synthetic Dimension & 4D Quantum Hall Effect Metamaterial Dialog.
//!
//! Provides:
//! - 2D Physical to Synthetic Frequency Projection Canvas: physical resonator array coupled to synthetic modal rings.
//! - 4D Bulk Band Dispersion Plot: 4-band Dirac spectrum along high-symmetry BZ trajectory (Gamma - X - M - R - W - Gamma).
//! - 3D Boundary Chiral Hyper-Surface Dispersion: gapless chiral linear modes traversing the bulk gap.
//! - Non-Linear 4D Hall Response Curve: quantized Hall current j_x vs E_y * B_zw with slope proportional to C_2.
//! - Synthetic Frequency Ladder Modal Spectrum: harmonic sidebands n * Omega_mod in dB.
//! - Telemetry Footer: Second Chern Number C_2, Bulk Gap (kHz), 4D Hall Conductance, Chiral Directivity (dB),
//!   Boundary Localization %, and Disorder Retention Ratio.

use egui::{pos2, vec2, Color32, FontId, RichText, Sense, Stroke, Ui};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::synthetic_4d_qhe::{
    Synthetic4dParams, SyntheticHallEngine, SyntheticHallParams,
};

/// Color palette for the 4D QHE synthetic dimension visualizer.
const COLOR_CANVAS_BG: Color32 = Color32::from_rgb(15, 23, 42); // Dark slate
const COLOR_PHYSICAL_SITE: Color32 = Color32::from_rgb(51, 65, 85); // Slate
const COLOR_BOUNDARY_GLOW: Color32 = Color32::from_rgb(244, 63, 94); // Rose red / boundary
const COLOR_SYNTHETIC_RING: Color32 = Color32::from_rgb(56, 189, 248); // Sky blue
const COLOR_BULK_UPPER: Color32 = Color32::from_rgb(168, 85, 247); // Purple
const COLOR_BULK_LOWER: Color32 = Color32::from_rgb(59, 130, 246); // Blue
const COLOR_CHIRAL_BRANCH: Color32 = Color32::from_rgb(239, 68, 68); // Red
const COLOR_HALL_RESPONSE: Color32 = Color32::from_rgb(34, 197, 94); // Emerald green
const COLOR_HARMONICS: Color32 = Color32::from_rgb(245, 158, 11); // Amber yellow

/// Active visualization tab in the 4D QHE synthetic dimension dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Synthetic4dDialogTab {
    /// 2D Physical to Synthetic Frequency Projection Canvas.
    PhysicalToSyntheticProjection,
    /// 4D Bulk Band Dispersion along high-symmetry BZ path.
    BulkDispersion4D,
    /// 3D Boundary Chiral Hyper-Surface Dispersion.
    BoundaryChiralDispersion,
    /// Quantized Non-Linear 4D Hall Response.
    NonLinearHallResponse,
    /// Synthetic Frequency Ladder Modal Spectrum.
    SyntheticFrequencyLadder,
}

/// Interactive modal dialog for Topological Acoustic Synthetic Dimension & 4D QHE Metamaterials.
#[derive(Debug, Clone)]
pub struct Synthetic4dDialog {
    /// Window visibility toggle.
    pub is_open: bool,
    /// Current selected view tab.
    pub active_tab: Synthetic4dDialogTab,

    /// Solver engine.
    pub engine: SyntheticHallEngine,

    // Controls buffers
    pub mass_m: f64,
    pub hopping_t_khz: f64,
    pub drive_field_ey: f64,
    pub synthetic_field_bzw: f64,
    pub synthetic_coupling: f64,
    pub disorder_w: f64,
    pub resonance_freq_hz: f64,
}

impl Default for Synthetic4dDialog {
    fn default() -> Self {
        let params = SyntheticHallParams {
            lattice: Synthetic4dParams {
                mass_m: 3.0,
                hopping_t_khz: 2.5,
                resonance_freq_hz: 4000.0,
                synthetic_coupling: 0.60,
                nx_layers: 10,
                disorder_w: 0.0,
            },
            drive_field_ey: 1.0,
            synthetic_field_bzw: 1.0,
            modulation_freq_hz: 250.0,
            harmonic_span: 3,
        };

        let engine = SyntheticHallEngine::new(params);

        Self {
            is_open: false,
            active_tab: Synthetic4dDialogTab::PhysicalToSyntheticProjection,
            engine,
            mass_m: 3.0,
            hopping_t_khz: 2.5,
            drive_field_ey: 1.0,
            synthetic_field_bzw: 1.0,
            synthetic_coupling: 0.60,
            disorder_w: 0.0,
            resonance_freq_hz: 4000.0,
        }
    }
}

impl Synthetic4dDialog {
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
        self.engine.params.lattice.mass_m = self.mass_m;
        self.engine.params.lattice.hopping_t_khz = self.hopping_t_khz;
        self.engine.params.lattice.synthetic_coupling = self.synthetic_coupling;
        self.engine.params.lattice.disorder_w = self.disorder_w;
        self.engine.params.lattice.resonance_freq_hz = self.resonance_freq_hz;
        self.engine.params.drive_field_ey = self.drive_field_ey;
        self.engine.params.synthetic_field_bzw = self.synthetic_field_bzw;

        self.engine.recompute();
    }

    /// Render the dialog window if open.
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Topological Acoustic Synthetic Dimension & 4D QHE Studio")
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
                Synthetic4dDialogTab::PhysicalToSyntheticProjection,
                "Physical-Synthetic Projection",
            );
            ui.selectable_value(
                &mut self.active_tab,
                Synthetic4dDialogTab::BulkDispersion4D,
                "4D Bulk Band Dispersion",
            );
            ui.selectable_value(
                &mut self.active_tab,
                Synthetic4dDialogTab::BoundaryChiralDispersion,
                "3D Boundary Chiral Hyper-Surface",
            );
            ui.selectable_value(
                &mut self.active_tab,
                Synthetic4dDialogTab::NonLinearHallResponse,
                "Quantized 4D Hall Response",
            );
            ui.selectable_value(
                &mut self.active_tab,
                Synthetic4dDialogTab::SyntheticFrequencyLadder,
                "Synthetic Frequency Ladder",
            );
        });

        ui.separator();

        match self.active_tab {
            Synthetic4dDialogTab::PhysicalToSyntheticProjection => {
                self.render_projection_canvas(ui);
            }
            Synthetic4dDialogTab::BulkDispersion4D => {
                self.render_bulk_dispersion(ui);
            }
            Synthetic4dDialogTab::BoundaryChiralDispersion => {
                self.render_boundary_dispersion(ui);
            }
            Synthetic4dDialogTab::NonLinearHallResponse => {
                self.render_hall_response(ui);
            }
            Synthetic4dDialogTab::SyntheticFrequencyLadder => {
                self.render_frequency_ladder(ui);
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
                .button("Topological 4D QHE (m = 3.0, C2 = -1)")
                .on_hover_text("Standard 4D quantum Hall phase with second Chern number C2 = -1 and chiral boundary states")
                .clicked()
            {
                self.mass_m = 3.0;
                self.hopping_t_khz = 2.5;
                self.disorder_w = 0.0;
                self.recompute();
            }

            if ui
                .button("High-Chern Phase (m = 1.0, C2 = +3)")
                .on_hover_text("High-Chern topological phase with second Chern number C2 = +3")
                .clicked()
            {
                self.mass_m = 1.0;
                self.hopping_t_khz = 2.5;
                self.disorder_w = 0.0;
                self.recompute();
            }

            if ui
                .button("Trivial 4D Insulator (m = 5.0, C2 = 0)")
                .on_hover_text("Trivial 4D insulating phase with vanishing second Chern number and zero Hall response")
                .clicked()
            {
                self.mass_m = 5.0;
                self.recompute();
            }

            if ui
                .button("Disorder Robustness Test (W = 0.25)")
                .on_hover_text("Introduce 25% coupling disorder to verify topological protection of 4D Hall conductance")
                .clicked()
            {
                self.mass_m = 3.0;
                self.disorder_w = 0.25;
                self.recompute();
            }
        });

        ui.add_space(4.0);

        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            ui.label("Mass Parameter m:");
            if ui
                .add(egui::Slider::new(&mut self.mass_m, 0.0..=6.0).step_by(0.1))
                .changed()
            {
                changed = true;
            }

            ui.label("Hopping t (kHz):");
            if ui
                .add(egui::Slider::new(&mut self.hopping_t_khz, 0.5..=5.0).step_by(0.1))
                .changed()
            {
                changed = true;
            }

            ui.label("Drive Ey (norm):");
            if ui
                .add(egui::Slider::new(&mut self.drive_field_ey, 0.1..=3.0).step_by(0.1))
                .changed()
            {
                changed = true;
            }
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("Synthetic Field Bzw:");
            if ui
                .add(egui::Slider::new(&mut self.synthetic_field_bzw, 0.1..=3.0).step_by(0.1))
                .changed()
            {
                changed = true;
            }

            ui.label("Synthetic Coupling:");
            if ui
                .add(egui::Slider::new(&mut self.synthetic_coupling, 0.1..=0.9).step_by(0.05))
                .changed()
            {
                changed = true;
            }

            ui.label("Disorder W:");
            if ui
                .add(egui::Slider::new(&mut self.disorder_w, 0.0..=0.50).step_by(0.01))
                .changed()
            {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }
    }

    /// Tab 1: 2D Physical to Synthetic Frequency Projection Canvas.
    fn render_projection_canvas(&self, ui: &mut Ui) {
        let desired_size = vec2(ui.available_width().max(400.0), 340.0);
        let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, COLOR_CANVAS_BG);

        let num_physical_x = 8;
        let num_physical_y = 5;

        let margin_x = 50.0;
        let margin_y = 40.0;
        let w = rect.width() - 2.0 * margin_x;
        let h = rect.height() - 2.0 * margin_y;

        let dx = w / (num_physical_x - 1) as f32;
        let dy = h / (num_physical_y - 1) as f32;

        let is_topo = self.engine.lattice_solver.params.is_topological();

        // Draw physical 2D grid links
        for iy in 0..num_physical_y {
            for ix in 0..num_physical_x {
                let px = rect.min.x + margin_x + ix as f32 * dx;
                let py = rect.min.y + margin_y + iy as f32 * dy;

                if ix + 1 < num_physical_x {
                    painter.line_segment(
                        [pos2(px, py), pos2(px + dx, py)],
                        Stroke::new(1.0, Color32::from_rgb(71, 85, 105)),
                    );
                }
                if iy + 1 < num_physical_y {
                    painter.line_segment(
                        [pos2(px, py), pos2(px, py + dy)],
                        Stroke::new(1.0, Color32::from_rgb(71, 85, 105)),
                    );
                }
            }
        }

        // Draw synthetic frequency ring orbits around each physical node
        for iy in 0..num_physical_y {
            for ix in 0..num_physical_x {
                let px = rect.min.x + margin_x + ix as f32 * dx;
                let py = rect.min.y + margin_y + iy as f32 * dy;

                let is_boundary = ix == 0;
                let (r, site_col) = if is_boundary && is_topo {
                    (8.0, COLOR_BOUNDARY_GLOW)
                } else {
                    (5.0, COLOR_PHYSICAL_SITE)
                };

                // Synthetic dimension orbital ring
                painter.circle_stroke(
                    pos2(px, py),
                    r + 6.0,
                    Stroke::new(1.0, COLOR_SYNTHETIC_RING),
                );

                // Central resonator site
                painter.circle_filled(pos2(px, py), r, site_col);

                // Boundary chiral propagation arrow indicators on layer x=0
                if is_boundary && is_topo && iy + 1 < num_physical_y {
                    let arrow_y = py + dy * 0.5;
                    painter.line_segment(
                        [pos2(px, py + 8.0), pos2(px, py + dy - 8.0)],
                        Stroke::new(2.5, COLOR_BOUNDARY_GLOW),
                    );
                    painter.circle_filled(pos2(px, arrow_y), 3.0, Color32::from_rgb(254, 205, 211));
                }
            }
        }

        // Legend overlay
        let leg_x = rect.min.x + 12.0;
        let leg_y = rect.min.y + 12.0;
        painter.circle_filled(pos2(leg_x + 6.0, leg_y + 6.0), 5.0, COLOR_BOUNDARY_GLOW);
        painter.text(
            pos2(leg_x + 16.0, leg_y),
            egui::Align2::LEFT_TOP,
            "Chiral 3D Boundary State (x = 0)",
            FontId::proportional(11.0),
            Color32::from_rgb(226, 232, 240),
        );

        painter.circle_stroke(
            pos2(leg_x + 6.0, leg_y + 22.0),
            5.0,
            Stroke::new(1.2, COLOR_SYNTHETIC_RING),
        );
        painter.text(
            pos2(leg_x + 16.0, leg_y + 16.0),
            egui::Align2::LEFT_TOP,
            "Synthetic Frequency Orbit (kz, kw)",
            FontId::proportional(11.0),
            Color32::from_rgb(226, 232, 240),
        );
    }

    /// Tab 2: 4D Bulk Band Dispersion along high-symmetry BZ path.
    fn render_bulk_dispersion(&self, ui: &mut Ui) {
        let disp = &self.engine.lattice_solver.bulk_dispersion;
        let lower_pts: Vec<[f64; 2]> = disp.iter().map(|p| [p.path_coordinate, p.eigenvalues_khz[0]]).collect();
        let upper_pts: Vec<[f64; 2]> = disp.iter().map(|p| [p.path_coordinate, p.eigenvalues_khz[2]]).collect();

        let plot = Plot::new("bulk_dispersion_4d_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("4D Brillouin Zone Path (Gamma - X - M - R - W - Gamma)")
            .y_axis_label("Energy (kHz)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Conduction Dirac Bands (E > 0)", PlotPoints::new(upper_pts))
                    .color(COLOR_BULK_UPPER)
                    .width(2.5),
            );
            plot_ui.line(
                Line::new("Valence Dirac Bands (E < 0)", PlotPoints::new(lower_pts))
                    .color(COLOR_BULK_LOWER)
                    .width(2.5),
            );

            // High-symmetry vertex indicators
            for pt in disp {
                if let Some(lbl) = pt.label {
                    plot_ui.vline(
                        VLine::new(lbl, pt.path_coordinate)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );
                }
            }
        });
    }

    /// Tab 3: 3D Boundary Chiral Hyper-Surface Dispersion.
    fn render_boundary_dispersion(&self, ui: &mut Ui) {
        let modes = &self.engine.lattice_solver.boundary_modes;
        let chiral_pts: Vec<[f64; 2]> = modes.iter().map(|m| [m.k_parallel, m.energy_khz]).collect();

        let plot = Plot::new("boundary_chiral_dispersion_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Boundary Momentum ky (rad)")
            .y_axis_label("Energy (kHz)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Chiral Boundary Hyper-Surface Mode", PlotPoints::new(chiral_pts))
                    .color(COLOR_CHIRAL_BRANCH)
                    .width(3.2),
            );
            plot_ui.hline(
                HLine::new("Zero Energy Mid-Gap Crossing", 0.0)
                    .color(Color32::from_rgb(100, 116, 139)),
            );
        });
    }

    /// Tab 4: Quantized Non-Linear 4D Hall Response.
    fn render_hall_response(&self, ui: &mut Ui) {
        let curve_data = self.engine.hall_response_curve(50, 4.0);
        let plot_points = PlotPoints::new(curve_data.iter().map(|&(f, j)| [f, j]).collect());

        let current_field = self.drive_field_ey * self.synthetic_field_bzw;
        let current_j = self.engine.metrics.non_linear_hall_current;
        let op_point = PlotPoints::new(vec![[current_field, current_j]]);

        let plot = Plot::new("hall_response_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Applied Field Product Ey * Bzw")
            .y_axis_label("Non-Linear 4D Hall Current jx (norm)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Quantized 4D Hall Characteristic", plot_points)
                    .color(COLOR_HALL_RESPONSE)
                    .width(2.5),
            );
            plot_ui.line(
                Line::new("Current Operating Point", op_point)
                    .color(COLOR_CHIRAL_BRANCH)
                    .width(6.0),
            );
            plot_ui.vline(
                VLine::new("Drive Field Ey * Bzw", current_field)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
        });
    }

    /// Tab 5: Synthetic Frequency Ladder Modal Spectrum.
    fn render_frequency_ladder(&self, ui: &mut Ui) {
        let harmonics = &self.engine.metrics.synthetic_harmonics;

        let plot = Plot::new("synthetic_frequency_ladder_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Harmonic Order Index n (Sidebands)")
            .y_axis_label("Modal Power (dB)");

        plot.show(ui, |plot_ui| {
            for h in harmonics {
                let stem = PlotPoints::new(vec![[h.harmonic_index as f64, -50.0], [h.harmonic_index as f64, h.power_db]]);
                let label = format!("Sideband n = {}", h.harmonic_index);
                plot_ui.line(
                    Line::new(label, stem)
                        .color(COLOR_HARMONICS)
                        .width(4.5),
                );
            }

            plot_ui.hline(
                HLine::new("Baseline Noise Floor (-50 dB)", -50.0)
                    .color(Color32::from_rgb(100, 116, 139)),
            );
        });
    }

    /// Render telemetry metrics footer.
    fn render_telemetry(&self, ui: &mut Ui) {
        let is_topo = self.engine.lattice_solver.params.is_topological();
        let c2 = self.engine.lattice_solver.params.second_chern_number();
        let gap_khz = self.engine.lattice_solver.calculated_bulk_gap_khz;
        let sigma = self.engine.metrics.non_linear_hall_conductance;
        let j_x = self.engine.metrics.non_linear_hall_current;
        let directivity = self.engine.metrics.chiral_directivity_db;
        let loc = self.engine.lattice_solver.boundary_confinement_ratio * 100.0;
        let retention = self.engine.metrics.disorder_retention_ratio * 100.0;

        ui.horizontal_wrapped(|ui| {
            let phase_text = if is_topo {
                RichText::new("Topological 4D QHE").color(Color32::from_rgb(52, 211, 153)).strong()
            } else {
                RichText::new("Trivial 4D Insulator").color(Color32::from_rgb(239, 68, 68)).strong()
            };
            ui.label(format!("Phase: "));
            ui.label(phase_text);
            ui.separator();

            ui.label(format!("Second Chern C2: {}", c2));
            ui.separator();

            ui.label(format!("Bulk Gap: {:.2} kHz", gap_khz));
            ui.separator();

            ui.label(format!("sigma_4D: {:.3}", sigma));
            ui.separator();

            ui.label(format!("Hall Current jx: {:.3}", j_x));
            ui.separator();

            ui.label(format!("Boundary Confinement: {:.1}%", loc));
            ui.separator();

            ui.label(format!("Directivity: {:.1} dB", directivity));
            ui.separator();

            ui.label(format!("Disorder Retention: {:.1}%", retention));
        });
    }
}
