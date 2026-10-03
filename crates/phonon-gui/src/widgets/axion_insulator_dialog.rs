#![deny(unsafe_code)]

//! Interactive Higher-Order Axion Insulator Visualizer for Phonon CAD Studio.
//!
//! Provides:
//! - 3D Prism / Cross-Section Spatial Density Canvas: interactive rendering of Nx x Ny rod cross-section
//!   showing 3D perspective with 4 glowing chiral 1D hinge states at the corners, showing chiral propagation
//!   direction (+z vs -z arrows) and Turbo/Magma/Inferno colormaps.
//! - Chiral Hinge Dispersion Plot: native egui_plot rendering E(kz) across kz in [-pi, pi], displaying
//!   bulk band continuum, surface bandgap, and the gapless chiral linear hinge branches crossing zero energy.
//! - Surface Hall Conductance & Domain Wall Indicator: visual diagram displaying opposing surface Hall
//!   conductances (+1/2 and -1/2) meeting at the hinge domain walls.
//! - Non-Reciprocal Hinge S-Parameter Spectrum: plots forward transmission S21(f) vs backward transmission S12(f)
//!   showing unidirectional acoustic transmission along the hinge.
//! - Controls: Hopping t slider, Mass M_0 slider, Surface mass Delta_surf slider, Axion angle toggle (PI vs 0),
//!   Cross-section size selector, Disorder toggle.
//! - Telemetry Footer: Axion Phase (Topological Axion Insulator vs Trivial), Quantized Axion Angle (theta / pi),
//!   Bulk Gap (MHz), Surface Gap (MHz), Hinge Confinement (%), Chiral Velocity v_F (mm/us), Hinge Directivity (dB).

use std::f64::consts::PI;
use egui::{
    vec2, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, Points, VLine};
use phonon_solver::axion_insulator::{
    AxionBandPoint, AxionHamiltonian, AxionParams, AxionRodLattice, AxionRodResult,
    HingeId, HingeSParameters,
};
use crate::thermal::heatmap::{sample_colormap, Colormap};

/// Theme colors for Axion Insulator Studio.
const COLOR_TOPO_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const COLOR_TOPO_EMERALD: Color32 = Color32::from_rgb(52, 211, 153);
const COLOR_TRIVIAL_AMBER: Color32 = Color32::from_rgb(251, 191, 36);
const COLOR_HINGE_GOLD: Color32 = Color32::from_rgb(250, 204, 21);
const COLOR_FORWARD_PURPLE: Color32 = Color32::from_rgb(168, 85, 247);
const COLOR_BACKWARD_ROSE: Color32 = Color32::from_rgb(244, 63, 94);
const COLOR_SURFACE_POS: Color32 = Color32::from_rgb(34, 211, 238);
const COLOR_SURFACE_NEG: Color32 = Color32::from_rgb(248, 113, 113);
const COLOR_GRID_LINE: Color32 = Color32::from_rgb(51, 65, 85);
const COLOR_TEXT_DIM: Color32 = Color32::from_rgb(148, 163, 184);

/// View mode for spatial intensity rendering on the rod cross-section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AxionSpatialViewMode {
    #[default]
    AllHingesCombined,
    Hinge1BottomLeft,
    Hinge2BottomRight,
    Hinge3TopRight,
    Hinge4TopLeft,
}

impl AxionSpatialViewMode {
    pub fn label(&self) -> &'static str {
        match self {
            AxionSpatialViewMode::AllHingesCombined => "All 4 Hinges Combined",
            AxionSpatialViewMode::Hinge1BottomLeft => "Hinge 1 (BL: +z forward)",
            AxionSpatialViewMode::Hinge2BottomRight => "Hinge 2 (BR: -z backward)",
            AxionSpatialViewMode::Hinge3TopRight => "Hinge 3 (TR: +z forward)",
            AxionSpatialViewMode::Hinge4TopLeft => "Hinge 4 (TL: -z backward)",
        }
    }
}

/// Active plot tab for the right column view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AxionPlotTab {
    #[default]
    ChiralHingeDispersion,
    NonReciprocalSParameters,
    SurfaceHallDomainWalls,
    BulkBandStructure,
}

impl AxionPlotTab {
    pub fn label(&self) -> &'static str {
        match self {
            AxionPlotTab::ChiralHingeDispersion => "Chiral Hinge Dispersion E(kz)",
            AxionPlotTab::NonReciprocalSParameters => "Non-Reciprocal S-Parameters",
            AxionPlotTab::SurfaceHallDomainWalls => "Surface Hall Domain Walls",
            AxionPlotTab::BulkBandStructure => "3D Bulk Band Structure",
        }
    }
}

/// Modal dialog for the Phonon Studio Quantum Metamaterial Higher-Order Axion Insulator Simulator.
#[derive(Debug, Clone, PartialEq)]
pub struct AxionInsulatorDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Controls
    /// Inter-cavity acoustic hopping amplitude t_hop in MHz.
    pub t_hop_mhz: f64,
    /// 3D Dirac mass parameter M_0 in MHz.
    pub m_0_mhz: f64,
    /// Dirac acoustic velocity v_dirac in MHz*mm.
    pub v_dirac_mhz_mm: f64,
    /// Axion angle theta in radians (PI or 0.0).
    pub theta_rad: f64,
    /// Surface time-reversal symmetry breaking mass Delta_surf in MHz.
    pub delta_surf_mhz: f64,
    /// Metamaterial unit cell lattice constant a_mm in mm.
    pub a_mm: f64,
    /// Bare acoustic resonance frequency omega_0 in GHz.
    pub omega_0_ghz: f64,
    /// Cross-section grid size along x.
    pub nx: usize,
    /// Cross-section grid size along y.
    pub ny: usize,

    // Disorder
    /// Defect/disorder perturbation toggle.
    pub disorder_enabled: bool,
    /// Disorder amplitude W in MHz.
    pub disorder_amplitude: f64,
    /// Random seed for disorder generation.
    pub disorder_seed: u64,

    // Visual preferences
    /// Scientific colormap for spatial intensity.
    pub colormap: Colormap,
    /// Selected spatial mode view.
    pub spatial_view_mode: AxionSpatialViewMode,
    /// Active plot tab.
    pub active_plot_tab: AxionPlotTab,

    // Cached Simulation State
    pub params: AxionParams,
    pub hamiltonian: AxionHamiltonian,
    pub lattice: AxionRodLattice,
    pub rod_result: AxionRodResult,
    pub s_parameters: HingeSParameters,
    pub bulk_dispersion: Vec<AxionBandPoint>,

    // Cached Plot Curves
    pub dispersion_forward_branch: Vec<[f64; 2]>,
    pub dispersion_backward_branch: Vec<[f64; 2]>,
    pub dispersion_bulk_bands: Vec<Vec<[f64; 2]>>,
    pub s21_curve: Vec<[f64; 2]>,
    pub s12_curve: Vec<[f64; 2]>,
    pub bulk_band_curves: [Vec<[f64; 2]>; 4],

    pub status_msg: String,
}

impl Default for AxionInsulatorDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl AxionInsulatorDialog {
    /// Creates a new default instance initialized to the 3D Topological Axion Insulator phase.
    pub fn new() -> Self {
        let params = AxionParams::topological();
        let hamiltonian = AxionHamiltonian::new(params);
        let lattice = AxionRodLattice::new(5, 5, params);
        let rod_result = lattice.solve();
        let s_parameters = lattice.compute_s_parameters(40);
        let bulk_dispersion = hamiltonian.compute_bulk_dispersion(15);

        let mut dialog = Self {
            is_open: false,
            t_hop_mhz: params.t_hop,
            m_0_mhz: params.m_0,
            v_dirac_mhz_mm: params.v_dirac,
            theta_rad: params.theta,
            delta_surf_mhz: params.delta_surf,
            a_mm: params.a_mm,
            omega_0_ghz: params.omega_0,
            nx: 5,
            ny: 5,
            disorder_enabled: false,
            disorder_amplitude: 0.5,
            disorder_seed: 42,
            colormap: Colormap::Turbo,
            spatial_view_mode: AxionSpatialViewMode::AllHingesCombined,
            active_plot_tab: AxionPlotTab::ChiralHingeDispersion,
            params,
            hamiltonian,
            lattice,
            rod_result,
            s_parameters,
            bulk_dispersion,
            dispersion_forward_branch: Vec::new(),
            dispersion_backward_branch: Vec::new(),
            dispersion_bulk_bands: Vec::new(),
            s21_curve: Vec::new(),
            s12_curve: Vec::new(),
            bulk_band_curves: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            status_msg: "Topological Axion Insulator active: theta = pi, 4 chiral hinge modes formed".to_string(),
        };

        dialog.update_plot_curves();
        dialog
    }

    /// Recomputes tight-binding Hamiltonian, band dispersion, and rod lattice eigenstates.
    pub fn recompute(&mut self) {
        self.params = AxionParams::new(
            self.t_hop_mhz,
            self.m_0_mhz,
            self.v_dirac_mhz_mm,
            self.theta_rad,
            self.delta_surf_mhz,
            self.a_mm,
            self.omega_0_ghz,
        );
        self.hamiltonian = AxionHamiltonian::new(self.params);
        self.lattice = AxionRodLattice::new(self.nx, self.ny, self.params);

        if self.disorder_enabled {
            let disorder_res = self
                .lattice
                .evaluate_disorder_robustness(self.disorder_amplitude, 3, self.disorder_seed);
            self.rod_result = self.lattice.solve();
            self.status_msg = format!(
                "Disorder W = {:.2} MHz active: hinge confinement {:.1}%, robust = {}",
                self.disorder_amplitude,
                disorder_res.mean_confinement_ratio * 100.0,
                disorder_res.is_robust
            );
        } else {
            self.rod_result = self.lattice.solve();
            if self.rod_result.is_topological {
                self.status_msg = format!(
                    "Topological Axion Insulator (theta = pi): 4 chiral hinge modes (confinement {:.1}%)",
                    self.rod_result.mean_hinge_confinement * 100.0
                );
            } else {
                self.status_msg = "Trivial Insulator phase (theta = 0): gapless hinge states absent".to_string();
            }
        }

        self.s_parameters = self.lattice.compute_s_parameters(40);
        self.bulk_dispersion = self.hamiltonian.compute_bulk_dispersion(15);
        self.update_plot_curves();
    }

    /// Updates cached plot curves for egui_plot rendering.
    fn update_plot_curves(&mut self) {
        // 1. Dispersion curves
        let mut fwd = Vec::new();
        let mut bwd = Vec::new();

        let v_f = self.rod_result.fermi_velocity_mhz;
        if self.rod_result.is_topological {
            for step in 0..41 {
                let kz = -PI + (step as f64) * 2.0 * PI / 40.0;
                let ef = (v_f * kz.sin()).clamp(-self.rod_result.surface_bandgap_mhz * 1.5, self.rod_result.surface_bandgap_mhz * 1.5);
                let eb = (-v_f * kz.sin()).clamp(-self.rod_result.surface_bandgap_mhz * 1.5, self.rod_result.surface_bandgap_mhz * 1.5);
                fwd.push([kz, ef]);
                bwd.push([kz, eb]);
            }
        }
        self.dispersion_forward_branch = fwd;
        self.dispersion_backward_branch = bwd;

        // Collect all band lines across kz
        let mut band_lines = Vec::new();
        if !self.rod_result.kz_dispersion.is_empty() {
            let num_evals = self.rod_result.kz_dispersion[0].1.len();
            for band_idx in 0..num_evals {
                let mut line = Vec::with_capacity(self.rod_result.kz_dispersion.len());
                for (kz, evals) in &self.rod_result.kz_dispersion {
                    if band_idx < evals.len() {
                        line.push([*kz, evals[band_idx]]);
                    }
                }
                band_lines.push(line);
            }
        }
        self.dispersion_bulk_bands = band_lines;

        // 2. S-parameters
        let mut s21 = Vec::with_capacity(self.s_parameters.freq_ghz.len());
        let mut s12 = Vec::with_capacity(self.s_parameters.freq_ghz.len());
        for i in 0..self.s_parameters.freq_ghz.len() {
            s21.push([self.s_parameters.freq_ghz[i], self.s_parameters.s21_db[i]]);
            s12.push([self.s_parameters.freq_ghz[i], self.s_parameters.s12_db[i]]);
        }
        self.s21_curve = s21;
        self.s12_curve = s12;

        // 3. Bulk bands
        let mut b0 = Vec::with_capacity(self.bulk_dispersion.len());
        let mut b1 = Vec::with_capacity(self.bulk_dispersion.len());
        let mut b2 = Vec::with_capacity(self.bulk_dispersion.len());
        let mut b3 = Vec::with_capacity(self.bulk_dispersion.len());
        for pt in &self.bulk_dispersion {
            let x = pt.path_distance;
            b0.push([x, pt.eigenvalues[0]]);
            b1.push([x, pt.eigenvalues[1]]);
            b2.push([x, pt.eigenvalues[2]]);
            b3.push([x, pt.eigenvalues[3]]);
        }
        self.bulk_band_curves = [b0, b1, b2, b3];
    }

    /// Sets the Topological Axion Insulator preset (theta = PI).
    pub fn set_topological_preset(&mut self) {
        self.t_hop_mhz = 5.0;
        self.m_0_mhz = 10.0;
        self.v_dirac_mhz_mm = 15.0;
        self.theta_rad = PI;
        self.delta_surf_mhz = 2.0;
        self.recompute();
    }

    /// Sets the Trivial Insulator preset (theta = 0.0).
    pub fn set_trivial_preset(&mut self) {
        self.t_hop_mhz = 5.0;
        self.m_0_mhz = 35.0;
        self.v_dirac_mhz_mm = 15.0;
        self.theta_rad = 0.0;
        self.delta_surf_mhz = 0.0;
        self.recompute();
    }

    /// Main entry point for rendering the modal dialog within an egui Context.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Quantum Metamaterial Higher-Order Axion Insulator Simulator")
            .open(&mut is_open)
            .default_size(vec2(1100.0, 720.0))
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
                RichText::new("Axion Metamaterial Configuration:")
                    .color(COLOR_TOPO_CYAN)
                    .strong(),
            );

            if ui
                .button(RichText::new("Preset: Topological Axion Insulator").color(COLOR_TOPO_EMERALD))
                .on_hover_text("Set topological phase: theta = pi, M_0 = 10.0 MHz, t_hop = 5.0 MHz")
                .clicked()
            {
                self.set_topological_preset();
            }

            if ui
                .button(RichText::new("Preset: Trivial Insulator").color(COLOR_TRIVIAL_AMBER))
                .on_hover_text("Set trivial phase: theta = 0, M_0 = 35.0 MHz, Delta_surf = 0.0 MHz")
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
            egui::ComboBox::from_id_salt("axion_cmap_select")
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
                    egui::Slider::new(&mut self.t_hop_mhz, 1.0..=15.0)
                        .text("t_hop (MHz)")
                        .step_by(0.1),
                )
                .changed()
            {
                dirty = true;
            }

            if ui
                .add(
                    egui::Slider::new(&mut self.m_0_mhz, 0.0..=40.0)
                        .text("M_0 (MHz)")
                        .step_by(0.5),
                )
                .changed()
            {
                dirty = true;
            }

            if ui
                .add(
                    egui::Slider::new(&mut self.delta_surf_mhz, 0.0..=10.0)
                        .text("Delta_surf (MHz)")
                        .step_by(0.1),
                )
                .changed()
            {
                dirty = true;
            }

            ui.separator();

            // Axion Angle Toggle
            ui.label(RichText::new("Axion Angle theta:").color(COLOR_TEXT_DIM));
            let is_pi = (self.theta_rad - PI).abs() < 0.2;
            if ui
                .selectable_label(is_pi, "PI (Topological)")
                .on_hover_text("Quantized axion angle theta = pi (P_3 = 0.5)")
                .clicked()
                && !is_pi
            {
                self.theta_rad = PI;
                dirty = true;
            }
            if ui
                .selectable_label(!is_pi, "0.0 (Trivial)")
                .on_hover_text("Trivial axion angle theta = 0.0 (P_3 = 0.0)")
                .clicked()
                && is_pi
            {
                self.theta_rad = 0.0;
                dirty = true;
            }
        });

        ui.add_space(4.0);

        // Second Control Bar: Grid Size, Dirac Velocity & Disorder
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Slider::new(&mut self.v_dirac_mhz_mm, 5.0..=30.0)
                        .text("v_dirac (MHz*mm)")
                        .step_by(1.0),
                )
                .changed()
            {
                dirty = true;
            }

            ui.separator();

            ui.label(RichText::new("Rod Cross-Section:").color(COLOR_TEXT_DIM));
            for &(n, lbl) in &[(4, "4x4"), (5, "5x5"), (6, "6x6")] {
                if ui.selectable_label(self.nx == n, lbl).clicked() && self.nx != n {
                    self.nx = n;
                    self.ny = n;
                    dirty = true;
                }
            }

            ui.separator();

            if ui
                .checkbox(&mut self.disorder_enabled, "Disorder [-W, W]")
                .changed()
            {
                dirty = true;
            }

            if self.disorder_enabled
                && ui
                    .add(
                        egui::Slider::new(&mut self.disorder_amplitude, 0.1..=4.0)
                            .text("W (MHz)")
                            .step_by(0.1),
                    )
                    .changed()
            {
                dirty = true;
            }
        });

        if dirty {
            self.recompute();
        }

        ui.separator();

        // Main Two-Column Layout
        ui.columns(2, |columns| {
            // Left Column: 3D Prism / Cross-Section Spatial Density Canvas
            columns[0].vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("3D Prism Rod Cross-Section |psi(x, y)|^2")
                            .color(COLOR_TOPO_CYAN)
                            .strong(),
                    );

                    egui::ComboBox::from_id_salt("spatial_mode_view")
                        .selected_text(self.spatial_view_mode.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.spatial_view_mode,
                                AxionSpatialViewMode::AllHingesCombined,
                                AxionSpatialViewMode::AllHingesCombined.label(),
                            );
                            ui.selectable_value(
                                &mut self.spatial_view_mode,
                                AxionSpatialViewMode::Hinge1BottomLeft,
                                AxionSpatialViewMode::Hinge1BottomLeft.label(),
                            );
                            ui.selectable_value(
                                &mut self.spatial_view_mode,
                                AxionSpatialViewMode::Hinge2BottomRight,
                                AxionSpatialViewMode::Hinge2BottomRight.label(),
                            );
                            ui.selectable_value(
                                &mut self.spatial_view_mode,
                                AxionSpatialViewMode::Hinge3TopRight,
                                AxionSpatialViewMode::Hinge3TopRight.label(),
                            );
                            ui.selectable_value(
                                &mut self.spatial_view_mode,
                                AxionSpatialViewMode::Hinge4TopLeft,
                                AxionSpatialViewMode::Hinge4TopLeft.label(),
                            );
                        });
                });

                self.render_spatial_canvas(ui);
            });

            // Right Column: Plot Tabs & Graphs
            columns[1].vertical(|ui| {
                ui.horizontal(|ui| {
                    for &tab in &[
                        AxionPlotTab::ChiralHingeDispersion,
                        AxionPlotTab::NonReciprocalSParameters,
                        AxionPlotTab::SurfaceHallDomainWalls,
                        AxionPlotTab::BulkBandStructure,
                    ] {
                        let selected = self.active_plot_tab == tab;
                        if ui.selectable_label(selected, tab.label()).clicked() {
                            self.active_plot_tab = tab;
                        }
                    }
                });

                ui.separator();

                match self.active_plot_tab {
                    AxionPlotTab::ChiralHingeDispersion => self.render_dispersion_plot(ui),
                    AxionPlotTab::NonReciprocalSParameters => self.render_s_parameters_plot(ui),
                    AxionPlotTab::SurfaceHallDomainWalls => self.render_surface_hall_plot(ui),
                    AxionPlotTab::BulkBandStructure => self.render_bulk_dispersion_plot(ui),
                }
            });
        });

        ui.separator();

        // Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders the 3D prism wireframe cross-section canvas showing glowing chiral hinge states and direction arrows.
    fn render_spatial_canvas(&self, ui: &mut Ui) {
        let desired_size = vec2(ui.available_width().max(380.0), 380.0);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
        let rect = response.rect;

        // Background canvas card
        painter.rect_filled(rect, 6.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(rect, 6.0, Stroke::new(1.0, COLOR_GRID_LINE), StrokeKind::Inside);

        let margin = 48.0;
        let w_canvas = rect.width() - margin * 2.0;
        let h_canvas = rect.height() - margin * 2.0;

        let cell_w = w_canvas / (self.nx as f32);
        let cell_h = h_canvas / (self.ny as f32);

        // 3D Prism Extrusion Vector (dx, dy)
        let ext_dx = 38.0_f32;
        let ext_dy = -28.0_f32;

        // Draw 3D prism back-face wireframe
        let back_color = Color32::from_rgba_unmultiplied(71, 85, 105, 70);
        let back_rect = Rect::from_min_max(
            Pos2::new(rect.min.x + margin + ext_dx, rect.min.y + margin + ext_dy),
            Pos2::new(rect.max.x - margin + ext_dx, rect.max.y - margin + ext_dy),
        );
        painter.rect_stroke(back_rect, 4.0, Stroke::new(1.0, back_color), StrokeKind::Inside);

        // Connecting prism edges along the 4 corners
        let front_corners = [
            Pos2::new(rect.min.x + margin, rect.max.y - margin), // BL (Hinge 1)
            Pos2::new(rect.max.x - margin, rect.max.y - margin), // BR (Hinge 2)
            Pos2::new(rect.max.x - margin, rect.min.y + margin), // TR (Hinge 3)
            Pos2::new(rect.min.x + margin, rect.min.y + margin), // TL (Hinge 4)
        ];

        let back_corners = [
            Pos2::new(front_corners[0].x + ext_dx, front_corners[0].y + ext_dy),
            Pos2::new(front_corners[1].x + ext_dx, front_corners[1].y + ext_dy),
            Pos2::new(front_corners[2].x + ext_dx, front_corners[2].y + ext_dy),
            Pos2::new(front_corners[3].x + ext_dx, front_corners[3].y + ext_dy),
        ];

        // Draw the 4 hinge edge lines connecting front to back
        for i in 0..4 {
            let edge_color = if self.rod_result.is_topological {
                Color32::from_rgba_unmultiplied(250, 204, 21, 140)
            } else {
                Color32::from_rgba_unmultiplied(71, 85, 105, 80)
            };
            painter.line_segment([front_corners[i], back_corners[i]], Stroke::new(1.5, edge_color));
        }

        // Get spatial probability slice based on view mode
        let mut prob_slice = vec![0.0; self.lattice.total_sites()];
        if self.rod_result.is_topological && !self.rod_result.hinge_modes.is_empty() {
            match self.spatial_view_mode {
                AxionSpatialViewMode::AllHingesCombined => {
                    for m in &self.rod_result.hinge_modes {
                        for (idx, &val) in m.spatial_probability.iter().enumerate() {
                            if idx < prob_slice.len() {
                                prob_slice[idx] += val * 0.25;
                            }
                        }
                    }
                }
                AxionSpatialViewMode::Hinge1BottomLeft => {
                    if let Some(m) = self.rod_result.hinge_modes.iter().find(|h| h.hinge_id == HingeId::Hinge1) {
                        prob_slice.clone_from(&m.spatial_probability);
                    }
                }
                AxionSpatialViewMode::Hinge2BottomRight => {
                    if let Some(m) = self.rod_result.hinge_modes.iter().find(|h| h.hinge_id == HingeId::Hinge2) {
                        prob_slice.clone_from(&m.spatial_probability);
                    }
                }
                AxionSpatialViewMode::Hinge3TopRight => {
                    if let Some(m) = self.rod_result.hinge_modes.iter().find(|h| h.hinge_id == HingeId::Hinge3) {
                        prob_slice.clone_from(&m.spatial_probability);
                    }
                }
                AxionSpatialViewMode::Hinge4TopLeft => {
                    if let Some(m) = self.rod_result.hinge_modes.iter().find(|h| h.hinge_id == HingeId::Hinge4) {
                        prob_slice.clone_from(&m.spatial_probability);
                    }
                }
            }
        }

        let max_prob = prob_slice.iter().copied().fold(0.0_f64, |a, b| a.max(b)).max(1e-12);

        // Helper to convert (x, y) to screen position
        let get_pos = |x: usize, y: usize| -> Pos2 {
            let px = rect.min.x + margin + (x as f32 + 0.5) * cell_w;
            let py = rect.max.y - margin - (y as f32 + 0.5) * cell_h;
            Pos2::new(px, py)
        };

        // Draw internal coupling grid lines on front face
        for y in 0..self.ny {
            for x in 0..self.nx {
                let p = get_pos(x, y);
                if x + 1 < self.nx {
                    let px_next = get_pos(x + 1, y);
                    painter.line_segment([p, px_next], Stroke::new(1.0, Color32::from_rgb(30, 41, 59)));
                }
                if y + 1 < self.ny {
                    let py_next = get_pos(x, y + 1);
                    painter.line_segment([p, py_next], Stroke::new(1.0, Color32::from_rgb(30, 41, 59)));
                }
            }
        }

        // Draw cross-section cavity nodes
        let node_radius = (cell_w.min(cell_h) * 0.30).clamp(6.0, 16.0);

        for y in 0..self.ny {
            for x in 0..self.nx {
                let idx = self.lattice.site_index(x, y);
                let p = get_pos(x, y);
                let intensity = (prob_slice.get(idx).copied().unwrap_or(0.0) / max_prob).clamp(0.0, 1.0) as f32;

                let is_corner = (x == 0 || x == self.nx - 1) && (y == 0 || y == self.ny - 1);

                // Halo glow at hinges
                if is_corner && self.rod_result.is_topological {
                    let glow_color = Color32::from_rgba_unmultiplied(250, 204, 21, 60);
                    painter.circle_filled(p, node_radius * 1.7, glow_color);
                }

                let color = sample_colormap(intensity, self.colormap);
                painter.circle_filled(p, node_radius, color);

                let stroke_color = if is_corner && self.rod_result.is_topological {
                    COLOR_HINGE_GOLD
                } else {
                    Color32::from_rgb(71, 85, 105)
                };
                painter.circle_stroke(p, node_radius, Stroke::new(1.5, stroke_color));
            }
        }

        // Draw chiral propagation symbols at the 4 hinges
        if self.rod_result.is_topological {
            let hinge_locs = [
                (0, 0, HingeId::Hinge1),
                (self.nx - 1, 0, HingeId::Hinge2),
                (self.nx - 1, self.ny - 1, HingeId::Hinge3),
                (0, self.ny - 1, HingeId::Hinge4),
            ];

            for &(cx, cy, h_id) in &hinge_locs {
                let p = get_pos(cx, cy);
                let v_dir = h_id.chiral_velocity_direction();

                if v_dir == 1 {
                    // Out-of-page (+z forward propagation): Circle-dot symbol
                    painter.circle_stroke(p, node_radius * 0.7, Stroke::new(1.5, COLOR_FORWARD_PURPLE));
                    painter.circle_filled(p, 2.5, COLOR_FORWARD_PURPLE);
                    painter.text(
                        Pos2::new(p.x, p.y - node_radius - 6.0),
                        Align2::CENTER_CENTER,
                        "+z",
                        FontId::monospace(10.0),
                        COLOR_FORWARD_PURPLE,
                    );
                } else {
                    // Into-page (-z backward propagation): Circle-cross symbol
                    painter.circle_stroke(p, node_radius * 0.7, Stroke::new(1.5, COLOR_BACKWARD_ROSE));
                    let d = node_radius * 0.45;
                    painter.line_segment([Pos2::new(p.x - d, p.y - d), Pos2::new(p.x + d, p.y + d)], Stroke::new(1.5, COLOR_BACKWARD_ROSE));
                    painter.line_segment([Pos2::new(p.x - d, p.y + d), Pos2::new(p.x + d, p.y - d)], Stroke::new(1.5, COLOR_BACKWARD_ROSE));
                    painter.text(
                        Pos2::new(p.x, p.y - node_radius - 6.0),
                        Align2::CENTER_CENTER,
                        "-z",
                        FontId::monospace(10.0),
                        COLOR_BACKWARD_ROSE,
                    );
                }
            }
        }

        // Draw Surface Hall Conductance Badges on the 4 outer surfaces
        if self.rod_result.is_topological {
            // Left surface: sigma_xy = +1/2
            let p_left = Pos2::new(rect.min.x + margin * 0.45, rect.min.y + margin + h_canvas * 0.5);
            painter.text(p_left, Align2::CENTER_CENTER, "sigma=+1/2", FontId::proportional(10.0), COLOR_SURFACE_POS);

            // Right surface: sigma_xy = +1/2
            let p_right = Pos2::new(rect.max.x - margin * 0.45, rect.min.y + margin + h_canvas * 0.5);
            painter.text(p_right, Align2::CENTER_CENTER, "sigma=+1/2", FontId::proportional(10.0), COLOR_SURFACE_POS);

            // Bottom surface: sigma_xy = -1/2
            let p_bot = Pos2::new(rect.min.x + margin + w_canvas * 0.5, rect.max.y - margin * 0.35);
            painter.text(p_bot, Align2::CENTER_CENTER, "sigma=-1/2", FontId::proportional(10.0), COLOR_SURFACE_NEG);

            // Top surface: sigma_xy = -1/2
            let p_top = Pos2::new(rect.min.x + margin + w_canvas * 0.5, rect.min.y + margin * 0.45);
            painter.text(p_top, Align2::CENTER_CENTER, "sigma=-1/2", FontId::proportional(10.0), COLOR_SURFACE_NEG);
        } else {
            let p_center = Pos2::new(rect.center().x, rect.center().y);
            painter.text(
                p_center,
                Align2::CENTER_CENTER,
                "Trivial Phase: Axion Angle theta = 0 (sigma = 0)",
                FontId::proportional(12.0),
                COLOR_TRIVIAL_AMBER,
            );
        }
    }

    /// Renders native egui_plot of chiral hinge dispersion E(kz).
    fn render_dispersion_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("axion_hinge_dispersion_plot")
            .legend(Legend::default())
            .x_axis_label("Axial Momentum kz (rad)")
            .y_axis_label("Energy E(kz) (MHz)")
            .height(380.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            // Bulk bands in gray
            for (idx, line) in self.dispersion_bulk_bands.iter().enumerate() {
                if idx % 2 == 0 {
                    plot_ui.line(
                        Line::new(format!("Bulk Continuum {idx}"), PlotPoints::new(line.clone()))
                            .color(Color32::from_rgb(71, 85, 105))
                            .width(1.0),
                    );
                }
            }

            // Surface bandgap horizontal guides
            let surf_gap = self.rod_result.surface_bandgap_mhz;
            if surf_gap > 0.0 {
                plot_ui.hline(
                    HLine::new("+Delta_surf/2", surf_gap * 0.5)
                        .color(Color32::from_rgba_unmultiplied(56, 189, 248, 120))
                        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(56, 189, 248, 120))),
                );
                plot_ui.hline(
                    HLine::new("-Delta_surf/2", -surf_gap * 0.5)
                        .color(Color32::from_rgba_unmultiplied(56, 189, 248, 120))
                        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(56, 189, 248, 120))),
                );
            }

            // Zero energy line
            plot_ui.hline(HLine::new("E = 0", 0.0).color(Color32::from_rgba_unmultiplied(148, 163, 184, 80)));
            plot_ui.vline(VLine::new("kz = 0", 0.0).color(Color32::from_rgba_unmultiplied(148, 163, 184, 80)));

            // Forward Chiral Hinge Branch (+z, +v_F)
            if !self.dispersion_forward_branch.is_empty() {
                plot_ui.line(
                    Line::new("Chiral Hinge (+z: H1 & H3)", PlotPoints::new(self.dispersion_forward_branch.clone()))
                        .color(COLOR_TOPO_CYAN)
                        .width(2.5),
                );
            }

            // Backward Chiral Hinge Branch (-z, -v_F)
            if !self.dispersion_backward_branch.is_empty() {
                plot_ui.line(
                    Line::new("Chiral Hinge (-z: H2 & H4)", PlotPoints::new(self.dispersion_backward_branch.clone()))
                        .color(COLOR_BACKWARD_ROSE)
                        .width(2.5),
                );
            }

            // Zero crossing point marker
            if self.rod_result.is_topological {
                plot_ui.points(
                    Points::new("Zero Energy Crossing", PlotPoints::new(vec![[0.0, 0.0]]))
                        .color(COLOR_HINGE_GOLD)
                        .radius(5.0),
                );
            }
        });
    }

    /// Renders native egui_plot of non-reciprocal S-parameters S21(f) vs S12(f).
    fn render_s_parameters_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("axion_s_parameters_plot")
            .legend(Legend::default())
            .x_axis_label("Frequency f (GHz)")
            .y_axis_label("Transmission (dB)")
            .height(380.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            // Forward transmission S21
            plot_ui.line(
                Line::new("Forward S21(f)", PlotPoints::new(self.s21_curve.clone()))
                    .color(COLOR_TOPO_EMERALD)
                    .width(2.5),
            );

            // Backward transmission S12
            plot_ui.line(
                Line::new("Backward S12(f)", PlotPoints::new(self.s12_curve.clone()))
                    .color(COLOR_BACKWARD_ROSE)
                    .width(2.0),
            );

            // Center frequency guide
            plot_ui.vline(
                VLine::new("Center f0", self.s_parameters.f0_ghz)
                    .color(COLOR_TEXT_DIM)
                    .stroke(Stroke::new(1.0, COLOR_TEXT_DIM)),
            );
        });
    }

    /// Renders the visual Surface Hall Effect & Domain Wall diagram.
    fn render_surface_hall_plot(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(
                RichText::new("Axion Surface Hall Effect & Hinge Domain Wall Topology")
                    .color(COLOR_TOPO_CYAN)
                    .strong(),
            );
            ui.add_space(8.0);

            let (p3, sigma_val) = if self.rod_result.is_topological {
                ("1/2 (Quantized topological)", "pm 1/2 (Half-quantized Hall conductance)")
            } else {
                ("0 (Trivial)", "0 (Reciprocal surface)")
            };

            ui.label(format!("Magnetoelectric Response P_3: {p3}"));
            ui.label(format!("Surface Hall Conductance: {sigma_val}"));
            ui.label("Domain Wall Condition: Opposing surface Hall conductances (+1/2 and -1/2) meet at corners.");
            ui.label("Jackiw-Rebbi Mechanism: Delta sigma_xy = (+1/2) - (-1/2) = 1 traps a gapless 1D chiral hinge state.");

            ui.add_space(12.0);

            // Visual diagram card
            let desired_size = vec2(ui.available_width().max(380.0), 220.0);
            let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
            let rect = response.rect;

            painter.rect_filled(rect, 6.0, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(rect, 6.0, Stroke::new(1.0, COLOR_GRID_LINE), StrokeKind::Inside);

            let box_w = 200.0_f32;
            let box_h = 140.0_f32;
            let c = rect.center();
            let domain_rect = Rect::from_center_size(c, vec2(box_w, box_h));

            // Domain box
            painter.rect_filled(domain_rect, 4.0, Color32::from_rgb(30, 41, 59));
            painter.rect_stroke(domain_rect, 4.0, Stroke::new(2.0, COLOR_TOPO_CYAN), StrokeKind::Inside);

            // Face labels
            painter.text(Pos2::new(domain_rect.min.x - 12.0, c.y), Align2::RIGHT_CENTER, "Left: +1/2", FontId::proportional(11.0), COLOR_SURFACE_POS);
            painter.text(Pos2::new(domain_rect.max.x + 12.0, c.y), Align2::LEFT_CENTER, "Right: +1/2", FontId::proportional(11.0), COLOR_SURFACE_POS);
            painter.text(Pos2::new(c.x, domain_rect.max.y + 12.0), Align2::CENTER_TOP, "Bottom: -1/2", FontId::proportional(11.0), COLOR_SURFACE_NEG);
            painter.text(Pos2::new(c.x, domain_rect.min.y - 12.0), Align2::CENTER_BOTTOM, "Top: -1/2", FontId::proportional(11.0), COLOR_SURFACE_NEG);

            // Glowing hinges at the 4 corners
            let corners = [
                domain_rect.min,
                Pos2::new(domain_rect.max.x, domain_rect.min.y),
                domain_rect.max,
                Pos2::new(domain_rect.min.x, domain_rect.max.y),
            ];

            for (i, &cp) in corners.iter().enumerate() {
                painter.circle_filled(cp, 8.0, COLOR_HINGE_GOLD);
                let lbl = format!("Hinge {}", i + 1);
                painter.text(cp, Align2::CENTER_CENTER, &lbl[6..7], FontId::monospace(9.0), Color32::BLACK);
            }
        });
    }

    /// Renders native egui_plot of 3D bulk band structure along high-symmetry path.
    fn render_bulk_dispersion_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("axion_bulk_band_plot")
            .legend(Legend::default())
            .x_axis_label("Momentum Path (Gamma - X - M - Gamma - Z - R)")
            .y_axis_label("Energy (MHz)")
            .height(380.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Valence Band 1", PlotPoints::new(self.bulk_band_curves[0].clone()))
                    .color(Color32::from_rgb(99, 102, 241))
                    .width(2.0),
            );
            plot_ui.line(
                Line::new("Valence Band 2", PlotPoints::new(self.bulk_band_curves[1].clone()))
                    .color(Color32::from_rgb(59, 130, 246))
                    .width(2.0),
            );
            plot_ui.line(
                Line::new("Conduction Band 1", PlotPoints::new(self.bulk_band_curves[2].clone()))
                    .color(COLOR_TOPO_EMERALD)
                    .width(2.0),
            );
            plot_ui.line(
                Line::new("Conduction Band 2", PlotPoints::new(self.bulk_band_curves[3].clone()))
                    .color(COLOR_TOPO_CYAN)
                    .width(2.0),
            );
        });
    }

    /// Renders the real-time telemetry metrics in the footer.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            // Status msg
            ui.label(RichText::new(&self.status_msg).color(COLOR_TEXT_DIM).size(11.0));

            ui.separator();

            // Phase indicator
            let (phase_txt, phase_col) = if self.rod_result.is_topological {
                ("Topological Axion Insulator", COLOR_TOPO_EMERALD)
            } else {
                ("Trivial Insulator", COLOR_TRIVIAL_AMBER)
            };
            ui.label(RichText::new(format!("Phase: {phase_txt}")).color(phase_col).strong().size(11.0));

            ui.separator();

            // Quantized Axion Angle
            let theta_ratio = self.theta_rad / PI;
            ui.label(
                RichText::new(format!("theta/pi: {theta_ratio:.2} (P_3 = {:.1})", self.params.polarizability_p3()))
                    .color(COLOR_TOPO_CYAN)
                    .size(11.0),
            );

            ui.separator();

            // Bulk Gap
            ui.label(
                RichText::new(format!("Bulk Gap: {:.1} MHz", self.rod_result.bulk_bandgap_mhz))
                    .color(COLOR_TEXT_DIM)
                    .size(11.0),
            );

            ui.separator();

            // Surface Gap
            ui.label(
                RichText::new(format!("Surface Gap: {:.1} MHz", self.rod_result.surface_bandgap_mhz))
                    .color(COLOR_TEXT_DIM)
                    .size(11.0),
            );

            ui.separator();

            // Confinement
            let conf_pct = self.rod_result.mean_hinge_confinement * 100.0;
            let conf_col = if conf_pct >= 80.0 { COLOR_TOPO_EMERALD } else { COLOR_TRIVIAL_AMBER };
            ui.label(
                RichText::new(format!("Hinge Confinement: {conf_pct:.1}%"))
                    .color(conf_col)
                    .strong()
                    .size(11.0),
            );

            ui.separator();

            // Chiral velocity
            ui.label(
                RichText::new(format!("v_F: {:.2} mm/us", self.rod_result.fermi_velocity_mm_per_us))
                    .color(COLOR_FORWARD_PURPLE)
                    .size(11.0),
            );

            ui.separator();

            // Directivity
            let dir_col = if self.rod_result.directivity_db >= 25.0 { COLOR_TOPO_EMERALD } else { COLOR_TRIVIAL_AMBER };
            ui.label(
                RichText::new(format!("Directivity: {:.1} dB", self.rod_result.directivity_db))
                    .color(dir_col)
                    .strong()
                    .size(11.0),
            );
        });
    }
}
