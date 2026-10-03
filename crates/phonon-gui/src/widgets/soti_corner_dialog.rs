#![deny(unsafe_code)]

//! Interactive Higher-Order Topological Corner State Acoustic Resonator Visualizer for Phonon CAD Studio.
//!
//! Provides:
//! - 2D Lattice Spatial Energy Density Canvas: interactive grid rendering Nx x Ny unit cells with
//!   4 sublattices per cell, displaying bright localized corner states at the 4 corners,
//!   with color intensity maps (Turbo/Magma).
//! - Quadrupole Band Structure Plot: native egui_plot rendering 4 bands along Gamma - X - M - Y - Gamma
//!   with highlighted bulk bandgap.
//! - Discrete Energy Spectrum Diagram: displays discrete eigenmode energy levels, clearly showing
//!   valence band, conduction band, and the 4 isolated zero-energy in-gap corner states at E approx 0.
//! - Defect / Disorder Toggle: checkbox to introduce random site/coupling disorder and display
//!   corner mode mid-gap pinning and spatial resilience.
//! - Controls: Intracell coupling gamma slider, intercell coupling lambda slider, lattice size selector,
//!   bare frequency omega_0, preset Topological SOTI vs Trivial Insulator buttons.
//! - Telemetry Footer: Phase (Topological SOTI vs Trivial), Quantized Quadrupole Moment q_xy,
//!   Bulk Gap Delta_bulk (MHz), Corner Confinement (%), Localization Length xi (mm),
//!   Resonator Quality Factor Q.

use egui::{
    vec2, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, Points, VLine};
use phonon_solver::soti_corner_resonator::{
    BandDispersionPoint, BbhHamiltonian, CornerId, QuadrupoleParams, SotiLattice,
    SotiLatticeResult, HIGH_SYMMETRY_PATH,
};
use crate::thermal::heatmap::{sample_colormap, Colormap};

/// Theme colors for SOTI Corner Resonator Studio.
const COLOR_TOPO_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const COLOR_TOPO_EMERALD: Color32 = Color32::from_rgb(52, 211, 153);
const COLOR_TRIVIAL_AMBER: Color32 = Color32::from_rgb(251, 191, 36);
const COLOR_CORNER_GOLD: Color32 = Color32::from_rgb(250, 204, 21);
const COLOR_BULK_BLUE: Color32 = Color32::from_rgb(99, 102, 241);
const COLOR_GRID_LINE: Color32 = Color32::from_rgb(51, 65, 85);
const COLOR_TEXT_DIM: Color32 = Color32::from_rgb(148, 163, 184);

/// View mode for spatial intensity rendering on the 2D lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpatialModeSelection {
    #[default]
    CombinedCorners,
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

impl SpatialModeSelection {
    pub fn label(&self) -> &'static str {
        match self {
            SpatialModeSelection::CombinedCorners => "All 4 Corners Combined",
            SpatialModeSelection::BottomLeft => "Mode 1: Bottom-Left Corner",
            SpatialModeSelection::BottomRight => "Mode 2: Bottom-Right Corner",
            SpatialModeSelection::TopLeft => "Mode 3: Top-Left Corner",
            SpatialModeSelection::TopRight => "Mode 4: Top-Right Corner",
        }
    }
}

/// Modal dialog for the Phonon Studio Topological Higher-Order Corner State Acoustic Resonator.
#[derive(Debug, Clone, PartialEq)]
pub struct SotiCornerDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Controls
    /// Intracell hopping coupling gamma in MHz.
    pub gamma_mhz: f64,
    /// Intercell hopping coupling lambda in MHz.
    pub lambda_mhz: f64,
    /// Bare acoustic resonance frequency omega_0 in GHz.
    pub omega_0_ghz: f64,
    /// Lattice unit cell dimension a_mm in mm.
    pub a_mm: f64,
    /// Lattice grid dimension along x (number of unit cells).
    pub nx: usize,
    /// Lattice grid dimension along y (number of unit cells).
    pub ny: usize,

    // Disorder
    /// Defect/disorder perturbation toggle.
    pub disorder_enabled: bool,
    /// Coupling disorder perturbation amplitude W in MHz.
    pub disorder_amplitude: f64,
    /// Random seed for disorder generation.
    pub disorder_seed: u64,

    // Visual preferences
    /// Scientific colormap for spatial intensity.
    pub colormap: Colormap,
    /// Selected spatial mode view.
    pub mode_view: SpatialModeSelection,

    // Cached Simulation State
    pub bbh: BbhHamiltonian,
    pub lattice: SotiLattice,
    pub lattice_result: SotiLatticeResult,
    pub band_dispersion: Vec<BandDispersionPoint>,

    // Cached Plot Curves
    pub band_curve_1: Vec<[f64; 2]>,
    pub band_curve_2: Vec<[f64; 2]>,
    pub band_curve_3: Vec<[f64; 2]>,
    pub band_curve_4: Vec<[f64; 2]>,
    pub discrete_spectrum_bulk: Vec<[f64; 2]>,
    pub discrete_spectrum_corner: Vec<[f64; 2]>,

    pub status_msg: String,
}

impl Default for SotiCornerDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SotiCornerDialog {
    /// Creates a new `SotiCornerDialog` initialized in the topological SOTI phase.
    pub fn new() -> Self {
        let params = QuadrupoleParams::topological();
        let nx = 5;
        let ny = 5;

        let bbh = BbhHamiltonian::new(params);
        let lattice = SotiLattice::new(nx, ny, params);
        let lattice_result = lattice.solve();
        let band_dispersion = bbh.compute_band_dispersion(30);

        let mut dialog = Self {
            is_open: false,
            gamma_mhz: params.gamma,
            lambda_mhz: params.lambda,
            omega_0_ghz: params.omega_0,
            a_mm: params.a_mm,
            nx,
            ny,
            disorder_enabled: false,
            disorder_amplitude: 0.5,
            disorder_seed: 42,
            colormap: Colormap::Turbo,
            mode_view: SpatialModeSelection::CombinedCorners,
            bbh,
            lattice,
            lattice_result,
            band_dispersion,
            band_curve_1: Vec::new(),
            band_curve_2: Vec::new(),
            band_curve_3: Vec::new(),
            band_curve_4: Vec::new(),
            discrete_spectrum_bulk: Vec::new(),
            discrete_spectrum_corner: Vec::new(),
            status_msg: "Topological SOTI phase active: 4 isolated corner states formed".to_string(),
        };

        dialog.update_plot_curves();
        dialog
    }

    /// Recomputes tight-binding Hamiltonian, band dispersion, and finite lattice eigenstates.
    pub fn recompute(&mut self) {
        let params = QuadrupoleParams::new(
            self.gamma_mhz,
            self.lambda_mhz,
            self.omega_0_ghz,
            self.a_mm,
        );
        self.bbh = BbhHamiltonian::new(params);
        self.lattice = SotiLattice::new(self.nx, self.ny, params);

        if self.disorder_enabled {
            self.lattice_result = self
                .lattice
                .solve_with_disorder(self.disorder_amplitude, self.disorder_seed);
            self.status_msg = format!(
                "Disorder W = {:.2} MHz active: corner modes topologically pinned",
                self.disorder_amplitude
            );
        } else {
            self.lattice_result = self.lattice.solve();
            if self.lattice_result.is_topological {
                self.status_msg = format!(
                    "Topological SOTI phase (gamma/lambda = {:.3}): 4 corner states pinned",
                    self.gamma_mhz / self.lambda_mhz.max(1e-6)
                );
            } else {
                self.status_msg = format!(
                    "Trivial Insulator phase (gamma/lambda = {:.3}): no corner states",
                    self.gamma_mhz / self.lambda_mhz.max(1e-6)
                );
            }
        }

        self.band_dispersion = self.bbh.compute_band_dispersion(30);
        self.update_plot_curves();
    }

    /// Updates cached plot points for egui_plot render passes.
    fn update_plot_curves(&mut self) {
        let mut b1 = Vec::with_capacity(self.band_dispersion.len());
        let mut b2 = Vec::with_capacity(self.band_dispersion.len());
        let mut b3 = Vec::with_capacity(self.band_dispersion.len());
        let mut b4 = Vec::with_capacity(self.band_dispersion.len());

        for pt in &self.band_dispersion {
            let x = pt.path_distance;
            b1.push([x, pt.eigenvalues[0]]);
            b2.push([x, pt.eigenvalues[1]]);
            b3.push([x, pt.eigenvalues[2]]);
            b4.push([x, pt.eigenvalues[3]]);
        }

        self.band_curve_1 = b1;
        self.band_curve_2 = b2;
        self.band_curve_3 = b3;
        self.band_curve_4 = b4;

        // Discrete spectrum
        let mut bulk_pts = Vec::new();
        let mut corner_pts = Vec::new();

        let corner_indices: std::collections::HashSet<usize> = self
            .lattice_result
            .corner_states
            .iter()
            .map(|cs| cs.mode_index)
            .collect();

        for (idx, &e) in self.lattice_result.all_eigenvalues.iter().enumerate() {
            let p = [idx as f64, e];
            if corner_indices.contains(&idx) {
                corner_pts.push(p);
            } else {
                bulk_pts.push(p);
            }
        }

        self.discrete_spectrum_bulk = bulk_pts;
        self.discrete_spectrum_corner = corner_pts;
    }

    /// Sets the Topological SOTI preset (gamma = 2.0, lambda = 10.0 MHz).
    pub fn set_topological_preset(&mut self) {
        self.gamma_mhz = 2.0;
        self.lambda_mhz = 10.0;
        self.recompute();
    }

    /// Sets the Trivial Insulator preset (gamma = 10.0, lambda = 2.0 MHz).
    pub fn set_trivial_preset(&mut self) {
        self.gamma_mhz = 10.0;
        self.lambda_mhz = 2.0;
        self.recompute();
    }

    /// Main entry point for rendering the modal dialog within an egui Context.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Topological SOTI Corner Resonator Studio")
            .open(&mut is_open)
            .default_size(vec2(1080.0, 720.0))
            .min_size(vec2(900.0, 600.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the complete dialog contents.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut dirty = false;

        // Top Control Bar
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Lattice Configuration:")
                    .color(COLOR_TOPO_CYAN)
                    .strong(),
            );

            if ui
                .button(RichText::new("Preset: Topological SOTI").color(COLOR_TOPO_EMERALD))
                .on_hover_text("Set topological phase: gamma = 2.0 MHz, lambda = 10.0 MHz")
                .clicked()
            {
                self.set_topological_preset();
            }

            if ui
                .button(RichText::new("Preset: Trivial Insulator").color(COLOR_TRIVIAL_AMBER))
                .on_hover_text("Set trivial phase: gamma = 10.0 MHz, lambda = 2.0 MHz")
                .clicked()
            {
                self.set_trivial_preset();
            }

            ui.separator();

            let cmap_name = match self.colormap {
                Colormap::Turbo => "Turbo",
                Colormap::Magma => "Magma",
                Colormap::Inferno => "Inferno",
            };
            egui::ComboBox::from_id_salt("soti_cmap_select")
                .selected_text(format!("Colormap: {cmap_name}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.colormap, Colormap::Turbo, "Turbo");
                    ui.selectable_value(&mut self.colormap, Colormap::Magma, "Magma");
                    ui.selectable_value(&mut self.colormap, Colormap::Inferno, "Inferno");
                });
        });

        ui.add_space(4.0);

        // Parameter Sliders Bar
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Slider::new(&mut self.gamma_mhz, 0.5..=15.0)
                        .text("gamma (MHz)")
                        .step_by(0.1),
                )
                .changed()
            {
                dirty = true;
            }

            if ui
                .add(
                    egui::Slider::new(&mut self.lambda_mhz, 0.5..=15.0)
                        .text("lambda (MHz)")
                        .step_by(0.1),
                )
                .changed()
            {
                dirty = true;
            }

            if ui
                .add(
                    egui::Slider::new(&mut self.omega_0_ghz, 0.1..=5.0)
                        .text("omega_0 (GHz)")
                        .step_by(0.1),
                )
                .changed()
            {
                dirty = true;
            }

            egui::ComboBox::from_id_salt("lattice_size_select")
                .selected_text(format!("Lattice: {}x{}", self.nx, self.ny))
                .show_ui(ui, |ui| {
                    for size in [4, 5, 6, 7, 8] {
                        if ui
                            .selectable_value(&mut self.nx, size, format!("{size}x{size}"))
                            .clicked()
                        {
                            self.ny = size;
                            dirty = true;
                        }
                    }
                });
        });

        ui.add_space(4.0);

        // Disorder / Defect Toggle Bar
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut self.disorder_enabled, "Defect / Disorder [-W, W]")
                .changed()
            {
                dirty = true;
            }

            if self.disorder_enabled {
                if ui
                    .add(
                        egui::Slider::new(&mut self.disorder_amplitude, 0.05..=2.0)
                            .text("W (MHz)")
                            .step_by(0.05),
                    )
                    .changed()
                {
                    dirty = true;
                }

                if ui.button("Reseed Disorder").clicked() {
                    self.disorder_seed = self.disorder_seed.wrapping_add(1013904223);
                    dirty = true;
                }
            }

            ui.separator();

            egui::ComboBox::from_id_salt("spatial_mode_view_select")
                .selected_text(self.mode_view.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.mode_view,
                        SpatialModeSelection::CombinedCorners,
                        "All 4 Corners Combined",
                    );
                    ui.selectable_value(
                        &mut self.mode_view,
                        SpatialModeSelection::BottomLeft,
                        "Mode 1: Bottom-Left Corner",
                    );
                    ui.selectable_value(
                        &mut self.mode_view,
                        SpatialModeSelection::BottomRight,
                        "Mode 2: Bottom-Right Corner",
                    );
                    ui.selectable_value(
                        &mut self.mode_view,
                        SpatialModeSelection::TopLeft,
                        "Mode 3: Top-Left Corner",
                    );
                    ui.selectable_value(
                        &mut self.mode_view,
                        SpatialModeSelection::TopRight,
                        "Mode 4: Top-Right Corner",
                    );
                });
        });

        if dirty {
            self.recompute();
        }

        ui.separator();

        // Main Visual Area: Split into Left (2D Lattice Canvas) and Right (Band Structure & Discrete Spectrum)
        ui.columns(2, |cols| {
            // Left Column: 2D Lattice Spatial Energy Density Canvas
            cols[0].vertical(|ui| {
                ui.label(
                    RichText::new("2D Lattice Spatial Energy Density Map |psi|^2:")
                        .color(COLOR_TOPO_CYAN)
                        .strong(),
                );
                self.render_lattice_canvas(ui);
            });

            // Right Column: Plots (Band Structure + Discrete Spectrum)
            cols[1].vertical(|ui| {
                ui.label(
                    RichText::new("Quadrupole Band Dispersion (Gamma - X - M - Y - Gamma):")
                        .color(COLOR_TOPO_CYAN)
                        .strong(),
                );
                self.render_band_structure_plot(ui);

                ui.add_space(6.0);

                ui.label(
                    RichText::new("Discrete Energy Spectrum (Valence / In-Gap / Conduction):")
                        .color(COLOR_TOPO_CYAN)
                        .strong(),
                );
                self.render_discrete_spectrum_plot(ui);
            });
        });

        ui.separator();

        // Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders the interactive 2D lattice grid with Turbo/Magma spatial intensity map.
    fn render_lattice_canvas(&self, ui: &mut Ui) {
        let canvas_size = vec2(ui.available_width().max(320.0), 380.0);
        let (response, painter) = ui.allocate_painter(canvas_size, Sense::hover());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, COLOR_GRID_LINE),
            StrokeKind::Inside,
        );

        let margin = 26.0;
        let grid_w = rect.width() - 2.0 * margin;
        let grid_h = rect.height() - 2.0 * margin;

        let cell_w = grid_w / (self.nx as f32);
        let cell_h = grid_h / (self.ny as f32);

        // Select spatial intensity array based on mode_view
        let intensity_slice: &[f64] = match self.mode_view {
            SpatialModeSelection::CombinedCorners => &self.lattice_result.combined_corner_intensity,
            SpatialModeSelection::BottomLeft => self
                .lattice_result
                .corner_states
                .iter()
                .find(|cs| cs.corner_id == CornerId::BottomLeft)
                .map(|cs| cs.spatial_intensity.as_slice())
                .unwrap_or(&self.lattice_result.combined_corner_intensity),
            SpatialModeSelection::BottomRight => self
                .lattice_result
                .corner_states
                .iter()
                .find(|cs| cs.corner_id == CornerId::BottomRight)
                .map(|cs| cs.spatial_intensity.as_slice())
                .unwrap_or(&self.lattice_result.combined_corner_intensity),
            SpatialModeSelection::TopLeft => self
                .lattice_result
                .corner_states
                .iter()
                .find(|cs| cs.corner_id == CornerId::TopLeft)
                .map(|cs| cs.spatial_intensity.as_slice())
                .unwrap_or(&self.lattice_result.combined_corner_intensity),
            SpatialModeSelection::TopRight => self
                .lattice_result
                .corner_states
                .iter()
                .find(|cs| cs.corner_id == CornerId::TopRight)
                .map(|cs| cs.spatial_intensity.as_slice())
                .unwrap_or(&self.lattice_result.combined_corner_intensity),
        };

        let max_intensity = intensity_slice
            .iter()
            .copied()
            .fold(0.0_f64, |a, b| a.max(b))
            .max(1e-12);

        // Helper to get site screen position (y inverted for intuitive display: y=0 at bottom)
        let get_site_pos = |x: usize, y: usize, s: usize| -> Pos2 {
            let cx = rect.min.x + margin + (x as f32) * cell_w;
            // Screen y increases downwards, so y=0 is at bottom
            let cy = rect.max.y - margin - ((y + 1) as f32) * cell_h;

            let (ox, oy) = match s {
                0 => (0.28 * cell_w, 0.72 * cell_h), // BL
                1 => (0.72 * cell_w, 0.72 * cell_h), // BR
                2 => (0.28 * cell_w, 0.28 * cell_h), // TL
                3 => (0.72 * cell_w, 0.28 * cell_h), // TR
                _ => (0.50 * cell_w, 0.50 * cell_h),
            };

            Pos2::new(cx + ox, cy + oy)
        };

        // Draw unit cell boundary frames and coupling lines
        for y in 0..self.ny {
            for x in 0..self.nx {
                let cell_min = Pos2::new(
                    rect.min.x + margin + (x as f32) * cell_w,
                    rect.max.y - margin - ((y + 1) as f32) * cell_h,
                );
                let cell_box = Rect::from_min_size(cell_min, vec2(cell_w, cell_h));

                // Cell boundary
                let is_corner = self.lattice.is_corner_cell(x, y);
                let cell_border_color = if is_corner && self.lattice_result.is_topological {
                    Color32::from_rgba_unmultiplied(250, 204, 21, 60)
                } else {
                    Color32::from_rgba_unmultiplied(51, 65, 85, 80)
                };
                painter.rect_stroke(
                    cell_box,
                    2.0,
                    Stroke::new(1.0, cell_border_color),
                    StrokeKind::Inside,
                );

                // Intracell coupling lines
                let p0 = get_site_pos(x, y, 0);
                let p1 = get_site_pos(x, y, 1);
                let p2 = get_site_pos(x, y, 2);
                let p3 = get_site_pos(x, y, 3);

                let intracell_stroke = Stroke::new(1.0, Color32::from_rgb(71, 85, 105));
                painter.line_segment([p0, p1], intracell_stroke);
                painter.line_segment([p2, p3], intracell_stroke);
                painter.line_segment([p0, p2], intracell_stroke);
                painter.line_segment([p1, p3], intracell_stroke);

                // Intercell couplings along x
                if x + 1 < self.nx {
                    let next_p0 = get_site_pos(x + 1, y, 0);
                    let next_p2 = get_site_pos(x + 1, y, 2);
                    let intercell_stroke =
                        Stroke::new(1.5, Color32::from_rgba_unmultiplied(56, 189, 248, 140));
                    painter.line_segment([p1, next_p0], intercell_stroke);
                    painter.line_segment([p3, next_p2], intercell_stroke);
                }

                // Intercell couplings along y
                if y + 1 < self.ny {
                    let next_p0 = get_site_pos(x, y + 1, 0);
                    let next_p1 = get_site_pos(x, y + 1, 1);
                    let intercell_stroke =
                        Stroke::new(1.5, Color32::from_rgba_unmultiplied(56, 189, 248, 140));
                    painter.line_segment([p2, next_p0], intercell_stroke);
                    painter.line_segment([p3, next_p1], intercell_stroke);
                }
            }
        }

        // Draw resonator sites with sampled scientific colormap
        for y in 0..self.ny {
            for x in 0..self.nx {
                for s in 0..4 {
                    let p = get_site_pos(x, y, s);
                    let site_idx = self.lattice.site_index(x, y, s);
                    let raw_val = intensity_slice.get(site_idx).copied().unwrap_or(0.0);
                    let norm_u = (raw_val / max_intensity).clamp(0.0, 1.0) as f32;

                    let color = sample_colormap(norm_u, self.colormap);
                    let base_radius = 3.5;
                    let radius = base_radius + 5.5 * norm_u.sqrt();

                    // Glow ring for high localized intensity
                    if norm_u > 0.4 {
                        painter.circle_filled(
                            p,
                            radius + 4.0,
                            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 60),
                        );
                    }

                    painter.circle_filled(p, radius, color);
                    painter.circle_stroke(
                        p,
                        radius,
                        Stroke::new(0.8, Color32::from_rgba_unmultiplied(255, 255, 255, 180)),
                    );
                }
            }
        }

        // Corner labels
        let font_lbl = FontId::proportional(11.0);
        let bl_pos = Pos2::new(rect.min.x + 8.0, rect.max.y - 14.0);
        let br_pos = Pos2::new(rect.max.x - 8.0, rect.max.y - 14.0);
        let tl_pos = Pos2::new(rect.min.x + 8.0, rect.min.y + 14.0);
        let tr_pos = Pos2::new(rect.max.x - 8.0, rect.min.y + 14.0);

        painter.text(bl_pos, Align2::LEFT_BOTTOM, "BL Corner", font_lbl.clone(), COLOR_CORNER_GOLD);
        painter.text(br_pos, Align2::RIGHT_BOTTOM, "BR Corner", font_lbl.clone(), COLOR_CORNER_GOLD);
        painter.text(tl_pos, Align2::LEFT_TOP, "TL Corner", font_lbl.clone(), COLOR_CORNER_GOLD);
        painter.text(tr_pos, Align2::RIGHT_TOP, "TR Corner", font_lbl, COLOR_CORNER_GOLD);
    }

    /// Renders the 4-band dispersion relation along the BZ path Gamma -> X -> M -> Y -> Gamma.
    fn render_band_structure_plot(&self, ui: &mut Ui) {
        let plot_height = 180.0;
        let delta_bulk = self.lattice_result.bulk_bandgap;

        let plot = Plot::new("bbh_band_plot")
            .height(plot_height)
            .legend(Legend::default().position(egui_plot::Corner::RightTop))
            .show_x(false)
            .show_y(true)
            .y_axis_label("Energy (MHz)")
            .allow_zoom(false)
            .allow_drag(false);

        plot.show(ui, |plot_ui| {
            // Bulk bandgap boundary lines
            let half_gap = delta_bulk / 2.0;
            plot_ui.hline(
                HLine::new("+Delta/2", half_gap)
                    .color(COLOR_TOPO_EMERALD)
                    .stroke(Stroke::new(1.0, COLOR_TOPO_EMERALD)),
            );
            plot_ui.hline(
                HLine::new("-Delta/2", -half_gap)
                    .color(COLOR_TOPO_EMERALD)
                    .stroke(Stroke::new(1.0, COLOR_TOPO_EMERALD)),
            );

            // Vertical lines for high-symmetry points
            for (idx, vertex) in HIGH_SYMMETRY_PATH.iter().enumerate() {
                let x = (idx as f64) * std::f64::consts::PI;
                plot_ui.vline(
                    VLine::new(vertex.label, x)
                        .color(COLOR_GRID_LINE)
                        .stroke(Stroke::new(0.8, COLOR_GRID_LINE)),
                );
            }

            // 4 dispersion bands
            plot_ui.line(
                Line::new("Valence Band 1", PlotPoints::new(self.band_curve_1.clone()))
                    .color(COLOR_BULK_BLUE)
                    .width(2.0),
            );
            plot_ui.line(
                Line::new("Valence Band 2", PlotPoints::new(self.band_curve_2.clone()))
                    .color(Color32::from_rgb(129, 140, 248))
                    .width(1.5),
            );
            plot_ui.line(
                Line::new("Conduction Band 1", PlotPoints::new(self.band_curve_3.clone()))
                    .color(Color32::from_rgb(244, 114, 182))
                    .width(1.5),
            );
            plot_ui.line(
                Line::new("Conduction Band 2", PlotPoints::new(self.band_curve_4.clone()))
                    .color(Color32::from_rgb(236, 72, 153))
                    .width(2.0),
            );
        });
    }

    /// Renders the discrete energy spectrum showing valence, in-gap corner states, and conduction bands.
    fn render_discrete_spectrum_plot(&self, ui: &mut Ui) {
        let plot_height = 160.0;
        let delta_bulk = self.lattice_result.bulk_bandgap;

        let plot = Plot::new("discrete_spectrum_plot")
            .height(plot_height)
            .legend(Legend::default().position(egui_plot::Corner::LeftTop))
            .show_x(true)
            .show_y(true)
            .x_axis_label("Mode Index")
            .y_axis_label("Energy (MHz)")
            .allow_zoom(false)
            .allow_drag(false);

        plot.show(ui, |plot_ui| {
            // Zero energy line
            plot_ui.hline(
                HLine::new("Zero Energy (Mid-Gap)", 0.0)
                    .color(Color32::from_rgba_unmultiplied(250, 204, 21, 100))
                    .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(250, 204, 21, 100))),
            );

            // Bulk gap indicators
            plot_ui.hline(
                HLine::new("+Delta/2", delta_bulk / 2.0)
                    .color(COLOR_TOPO_EMERALD)
                    .stroke(Stroke::new(0.8, COLOR_TOPO_EMERALD)),
            );
            plot_ui.hline(
                HLine::new("-Delta/2", -delta_bulk / 2.0)
                    .color(COLOR_TOPO_EMERALD)
                    .stroke(Stroke::new(0.8, COLOR_TOPO_EMERALD)),
            );

            // Bulk modes
            plot_ui.points(
                Points::new("Bulk / Edge Modes", PlotPoints::new(self.discrete_spectrum_bulk.clone()))
                    .radius(2.5)
                    .color(COLOR_BULK_BLUE),
            );

            // In-gap corner states
            plot_ui.points(
                Points::new("Isolated Corner States", PlotPoints::new(self.discrete_spectrum_corner.clone()))
                    .radius(5.0)
                    .color(COLOR_CORNER_GOLD),
            );
        });
    }

    /// Renders the physical telemetry footer panel.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(14.0, 4.0);

            // Phase indicator badge
            if self.lattice_result.is_topological {
                ui.label(
                    RichText::new("PHASE: TOPOLOGICAL SOTI")
                        .color(COLOR_TOPO_EMERALD)
                        .strong(),
                );
            } else {
                ui.label(
                    RichText::new("PHASE: TRIVIAL INSULATOR")
                        .color(COLOR_TRIVIAL_AMBER)
                        .strong(),
                );
            }

            ui.label(RichText::new("|").color(COLOR_GRID_LINE));

            // Quantized Quadrupole Moment
            let q_xy = self.bbh.quadrupole_moment();
            ui.label(format!("Quantized Quadrupole q_xy: {q_xy:.2}"));

            ui.label(RichText::new("|").color(COLOR_GRID_LINE));

            // Bulk Gap Delta_bulk
            ui.label(format!(
                "Bulk Gap Delta: {:.2} MHz",
                self.lattice_result.bulk_bandgap
            ));

            ui.label(RichText::new("|").color(COLOR_GRID_LINE));

            // Corner Confinement
            let conf_pct = self.lattice_result.energy_confinement_ratio * 100.0;
            ui.label(format!("Corner Confinement: {conf_pct:.1}%"));

            ui.label(RichText::new("|").color(COLOR_GRID_LINE));

            // Localization Length xi
            if self.lattice_result.localization_length_mm.is_finite() {
                ui.label(format!(
                    "xi: {:.2} mm",
                    self.lattice_result.localization_length_mm
                ));
            } else {
                ui.label("xi: Infinity (Trivial)");
            }

            ui.label(RichText::new("|").color(COLOR_GRID_LINE));

            // Resonator Quality Factor Q
            ui.label(format!(
                "Resonator Q: {:.2e}",
                self.lattice_result.quality_factor
            ));

            ui.label(RichText::new("|").color(COLOR_GRID_LINE));

            // Identified midgap states
            let n_corner = self.lattice_result.corner_states.len();
            ui.label(format!("Corner Modes: {n_corner} / 4"));
        });

        ui.add_space(2.0);
        ui.label(RichText::new(&self.status_msg).size(11.0).color(COLOR_TEXT_DIM));
    }
}
