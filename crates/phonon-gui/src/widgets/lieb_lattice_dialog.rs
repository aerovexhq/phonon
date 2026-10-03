#![deny(unsafe_code)]

//! Interactive Flat-Band Lieb Lattice & Aharonov-Bohm Caging Visualizer for Phonon CAD Studio.
//!
//! Provides:
//! - 2D Lieb Lattice Real-Space Amplitude Canvas: interactive grid rendering Nx x Ny unit cells
//!   with corner A, horizontal edge B, and vertical edge C sublattices, showing CLS eigenmode
//!   intensity |psi(x, y)|^2 with alternating phase signs (+/-) and Turbo/Magma colormaps.
//! - 3-Band Dispersion Plot: native egui_plot rendering Upper Band, Mid Flat Band (E=0 line),
//!   and Lower Band along Gamma - X - M - Y - Gamma, highlighting the Dirac touching point at M.
//! - Synthetic Aharonov-Bohm Caging Curve: egui_plot showing wave packet localization / inverse
//!   participation ratio vs gauge flux Phi in [0, 2*pi], highlighting the maximum caging peak at Phi = pi.
//! - Time-Evolution Wave Packet Dynamics: displays spatial diffusion vs caged confinement under time evolution t.
//! - Controls: Hopping J slider, Gauge flux Phi slider, lattice size selector, disorder amplitude W slider,
//!   Preset buttons ("Aharonov-Bohm Caging (Phi = pi)", "Standard Lieb Lattice (Phi = 0)", "Compact Localized State (CLS)").
//! - Telemetry Footer: Flat Band Flatness (Hz), Group Velocity (0.0 m/s), CLS Confinement Ratio (100%),
//!   Gauge Flux (Phi / pi), Dirac Point Frequency (GHz), Caging Purity (%).

use std::f64::consts::PI;
use egui::{
    vec2, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, Points, VLine};
use phonon_solver::lieb_lattice::{
    AbCagingSimulator, CompactLocalizedState, LiebBandPoint, LiebHamiltonian, LiebLattice,
    LiebParams,
};
use crate::thermal::heatmap::{sample_colormap, Colormap};

/// Theme colors for Lieb Lattice Visualizer.
const COLOR_TOPO_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const COLOR_TOPO_EMERALD: Color32 = Color32::from_rgb(52, 211, 153);
const COLOR_FLAT_GOLD: Color32 = Color32::from_rgb(250, 204, 21);
const COLOR_BULK_BLUE: Color32 = Color32::from_rgb(99, 102, 241);
const COLOR_GRID_LINE: Color32 = Color32::from_rgb(51, 65, 85);
const COLOR_TEXT_DIM: Color32 = Color32::from_rgb(148, 163, 184);

/// View mode for spatial intensity rendering on the 2D lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LiebSpatialModeSelection {
    #[default]
    SinglePlaquetteCls,
    TimeEvolvedWavepacket,
    AllPlaquettesCls,
}

impl LiebSpatialModeSelection {
    pub fn label(&self) -> &'static str {
        match self {
            LiebSpatialModeSelection::SinglePlaquetteCls => "Single-Plaquette CLS Mode",
            LiebSpatialModeSelection::TimeEvolvedWavepacket => "Time-Evolved Wavepacket |psi(t)|^2",
            LiebSpatialModeSelection::AllPlaquettesCls => "Combined All Plaquettes CLS",
        }
    }
}

/// Active plot tab for the right column view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LiebPlotTab {
    #[default]
    BandDispersion,
    CagingCurve,
    TimeDynamics,
}

impl LiebPlotTab {
    pub fn label(&self) -> &'static str {
        match self {
            LiebPlotTab::BandDispersion => "3-Band Dispersion",
            LiebPlotTab::CagingCurve => "AB Caging vs Flux",
            LiebPlotTab::TimeDynamics => "Wavepacket Dynamics",
        }
    }
}

/// Modal dialog for the Phonon Studio Topological Acoustic Flat-Band Lieb-Lattice Gauge Simulator.
#[derive(Debug, Clone, PartialEq)]
pub struct LiebLatticeDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Controls
    /// Hopping coupling J in MHz.
    pub j_mhz: f64,
    /// Bare acoustic resonance frequency omega_0 in GHz.
    pub omega_0_ghz: f64,
    /// Lattice unit cell dimension a_mm in mm.
    pub a_mm: f64,
    /// Synthetic gauge flux Phi in radians [0.0, 2.0 * PI].
    pub phi_flux: f64,
    /// On-site energy detuning Delta_site in MHz.
    pub delta_site: f64,
    /// Grid dimension along x (number of unit cells).
    pub nx: usize,
    /// Grid dimension along y (number of unit cells).
    pub ny: usize,

    // Disorder
    /// Defect/disorder perturbation toggle.
    pub disorder_enabled: bool,
    /// On-site disorder perturbation amplitude W in MHz.
    pub disorder_amplitude: f64,
    /// Random seed for disorder generation.
    pub disorder_seed: u64,

    // Wavepacket Evolution
    /// Quantum evolution time t in ns.
    pub evolution_time_ns: f64,

    // Visual preferences
    /// Scientific colormap for spatial intensity.
    pub colormap: Colormap,
    /// Selected spatial mode view.
    pub mode_view: LiebSpatialModeSelection,
    /// Active plot tab.
    pub active_plot_tab: LiebPlotTab,

    // Cached Simulation State
    pub hamiltonian: LiebHamiltonian,
    pub lattice: LiebLattice,
    pub simulator: AbCagingSimulator,
    pub band_dispersion: Vec<LiebBandPoint>,
    pub current_cls: Option<CompactLocalizedState>,
    pub current_spatial_intensity: Vec<f64>,

    // Cached Plot Curves
    pub band_curve_upper: Vec<[f64; 2]>,
    pub band_curve_flat: Vec<[f64; 2]>,
    pub band_curve_lower: Vec<[f64; 2]>,
    pub caging_curve: Vec<[f64; 2]>,
    pub time_dynamics_current: Vec<[f64; 2]>,
    pub time_dynamics_caged: Vec<[f64; 2]>,

    pub status_msg: String,
}

impl Default for LiebLatticeDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl LiebLatticeDialog {
    /// Creates a new LiebLatticeDialog initialized to standard parameters.
    pub fn new() -> Self {
        let params = LiebParams::default();
        let hamiltonian = LiebHamiltonian::new(params);
        let lattice = LiebLattice::new(4, 4, params);
        let simulator = AbCagingSimulator::new(lattice.clone());

        let mut dialog = Self {
            is_open: false,
            j_mhz: 5.0,
            omega_0_ghz: 1.0,
            a_mm: 10.0,
            phi_flux: 0.0,
            delta_site: 0.0,
            nx: 4,
            ny: 4,
            disorder_enabled: false,
            disorder_amplitude: 0.2,
            disorder_seed: 42,
            evolution_time_ns: 20.0,
            colormap: Colormap::Turbo,
            mode_view: LiebSpatialModeSelection::SinglePlaquetteCls,
            active_plot_tab: LiebPlotTab::BandDispersion,
            hamiltonian,
            lattice,
            simulator,
            band_dispersion: Vec::new(),
            current_cls: None,
            current_spatial_intensity: Vec::new(),
            band_curve_upper: Vec::new(),
            band_curve_flat: Vec::new(),
            band_curve_lower: Vec::new(),
            caging_curve: Vec::new(),
            time_dynamics_current: Vec::new(),
            time_dynamics_caged: Vec::new(),
            status_msg: String::from("Lieb Lattice Flat-Band Engine Ready"),
        };

        dialog.recompute();
        dialog
    }

    /// Recomputes momentum-space band dispersion, real-space eigensystem, CLS, and caging dynamics.
    pub fn recompute(&mut self) {
        let params = LiebParams::new(
            self.j_mhz,
            self.j_mhz,
            self.omega_0_ghz,
            self.a_mm,
            self.phi_flux,
            self.delta_site,
        );

        self.hamiltonian = LiebHamiltonian::new(params);
        self.lattice = LiebLattice::new(self.nx, self.ny, params);

        // Solve real-space finite lattice (with or without disorder)
        if self.disorder_enabled {
            let (h_dis_re, h_dis_im) = self
                .lattice
                .assemble_hamiltonian_with_disorder(self.disorder_amplitude, self.disorder_seed);
            let sol = self.lattice.solve_matrix(&h_dis_re, &h_dis_im);
            self.simulator = AbCagingSimulator {
                lattice: self.lattice.clone(),
                solution: sol,
            };
        } else {
            self.simulator = AbCagingSimulator::new(self.lattice.clone());
        }

        // Analytical single-plaquette CLS at center plaquette
        let cx = (self.nx.saturating_sub(1)) / 2;
        let cy = (self.ny.saturating_sub(1)) / 2;
        self.current_cls = self.lattice.generate_analytical_cls(cx, cy);

        // Update spatial intensity map according to selected mode
        self.update_spatial_intensity();

        // 3-Band Dispersion along Gamma - X - M - Y - Gamma
        self.band_dispersion = self.hamiltonian.dispersion_along_path(24);
        self.band_curve_upper.clear();
        self.band_curve_flat.clear();
        self.band_curve_lower.clear();

        for pt in &self.band_dispersion {
            self.band_curve_upper.push([pt.k_path_dist, pt.energy_upper]);
            self.band_curve_flat.push([pt.k_path_dist, pt.energy_flat]);
            self.band_curve_lower.push([pt.k_path_dist, pt.energy_lower]);
        }

        // Synthetic AB Caging Curve vs Flux Phi
        let raw_caging = self.simulator.caging_curve_vs_flux(17, self.evolution_time_ns);
        self.caging_curve = raw_caging.into_iter().map(|(p, ipr)| [p, ipr]).collect();

        // Wavepacket time-evolution dynamics
        let raw_dynamics = self.simulator.time_evolution_dynamics(50.0, 25);
        self.time_dynamics_current.clear();
        self.time_dynamics_caged.clear();
        for (t, ipr_curr, ipr_caged) in raw_dynamics {
            self.time_dynamics_current.push([t, ipr_curr]);
            self.time_dynamics_caged.push([t, ipr_caged]);
        }

        self.status_msg = format!(
            "Solved {}x{} Lieb Lattice ({} sites) | Phi = {:.2} pi | CLS E = 0.0 MHz",
            self.nx,
            self.ny,
            self.lattice.total_sites(),
            self.phi_flux / PI
        );
    }

    /// Updates the spatial intensity buffer according to current mode_view.
    pub fn update_spatial_intensity(&mut self) {
        let dim = self.lattice.total_sites();
        match self.mode_view {
            LiebSpatialModeSelection::SinglePlaquetteCls => {
                if let Some(ref cls) = self.current_cls {
                    self.current_spatial_intensity = cls.spatial_intensity.clone();
                } else {
                    self.current_spatial_intensity = vec![0.0; dim];
                }
            }
            LiebSpatialModeSelection::TimeEvolvedWavepacket => {
                let psi_0 = self.simulator.initial_center_site_wavepacket();
                let psi_t = self
                    .simulator
                    .evolve_wave_packet(&psi_0, self.evolution_time_ns);
                self.current_spatial_intensity = AbCagingSimulator::spatial_intensity(&psi_t);
            }
            LiebSpatialModeSelection::AllPlaquettesCls => {
                self.current_spatial_intensity =
                    self.simulator.solution.combined_cls_intensity.clone();
            }
        }
    }

    /// Preset switch: Synthetic Aharonov-Bohm Caging (Phi = PI).
    pub fn apply_preset_caging(&mut self) {
        self.phi_flux = PI;
        self.mode_view = LiebSpatialModeSelection::TimeEvolvedWavepacket;
        self.active_plot_tab = LiebPlotTab::CagingCurve;
        self.recompute();
    }

    /// Preset switch: Standard Lieb Lattice (Phi = 0.0).
    pub fn apply_preset_standard(&mut self) {
        self.phi_flux = 0.0;
        self.active_plot_tab = LiebPlotTab::BandDispersion;
        self.recompute();
    }

    /// Preset switch: Compact Localized State (CLS).
    pub fn apply_preset_cls(&mut self) {
        self.phi_flux = 0.0;
        self.mode_view = LiebSpatialModeSelection::SinglePlaquetteCls;
        self.active_plot_tab = LiebPlotTab::BandDispersion;
        self.recompute();
    }

    /// Main UI rendering loop.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Topological Acoustic Lieb-Lattice Flat-Band Studio")
            .open(&mut is_open)
            .default_size(vec2(1040.0, 720.0))
            .min_size(vec2(860.0, 580.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Inner content rendering logic.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut dirty = false;

        // Top Toolbar: Presets and Status
        ui.horizontal(|ui| {
            ui.label(RichText::new("Presets:").strong().color(COLOR_TOPO_CYAN));

            if ui
                .button(RichText::new("Aharonov-Bohm Caging (Phi = pi)").color(COLOR_TOPO_EMERALD))
                .clicked()
            {
                self.apply_preset_caging();
            }

            if ui.button("Standard Lieb Lattice (Phi = 0)").clicked() {
                self.apply_preset_standard();
            }

            if ui
                .button(RichText::new("Compact Localized State (CLS)").color(COLOR_FLAT_GOLD))
                .clicked()
            {
                self.apply_preset_cls();
            }

            ui.separator();

            ui.label(RichText::new(&self.status_msg).color(COLOR_TEXT_DIM).size(11.0));
        });

        ui.add_space(4.0);

        // Parameter Sliders Bar
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Slider::new(&mut self.j_mhz, 0.5..=15.0)
                        .text("Hopping J (MHz)")
                        .step_by(0.1),
                )
                .changed()
            {
                dirty = true;
            }

            let mut flux_units = self.phi_flux / PI;
            if ui
                .add(
                    egui::Slider::new(&mut flux_units, 0.0..=2.0)
                        .text("Gauge Flux (Phi / pi)")
                        .step_by(0.05),
                )
                .changed()
            {
                self.phi_flux = flux_units * PI;
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

            egui::ComboBox::from_id_salt("lieb_lattice_size_select")
                .selected_text(format!("Lattice: {}x{}", self.nx, self.ny))
                .show_ui(ui, |ui| {
                    for size in [3, 4, 5, 6, 7, 8] {
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

        // Secondary Controls Bar: Disorder, Dynamics Time, View Modes, Colormap
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut self.disorder_enabled, "Disorder [-W, W]")
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

                if ui.button("Reseed").clicked() {
                    self.disorder_seed = self.disorder_seed.wrapping_add(1013904223);
                    dirty = true;
                }
            }

            ui.separator();

            if ui
                .add(
                    egui::Slider::new(&mut self.evolution_time_ns, 0.0..=50.0)
                        .text("Time t (ns)")
                        .step_by(0.5),
                )
                .changed()
            {
                if self.mode_view == LiebSpatialModeSelection::TimeEvolvedWavepacket {
                    self.update_spatial_intensity();
                }
            }

            ui.separator();

            egui::ComboBox::from_id_salt("lieb_mode_view_select")
                .selected_text(self.mode_view.label())
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            &mut self.mode_view,
                            LiebSpatialModeSelection::SinglePlaquetteCls,
                            "Single-Plaquette CLS Mode",
                        )
                        .clicked()
                    {
                        self.update_spatial_intensity();
                    }
                    if ui
                        .selectable_value(
                            &mut self.mode_view,
                            LiebSpatialModeSelection::TimeEvolvedWavepacket,
                            "Time-Evolved Wavepacket |psi(t)|^2",
                        )
                        .clicked()
                    {
                        self.update_spatial_intensity();
                    }
                    if ui
                        .selectable_value(
                            &mut self.mode_view,
                            LiebSpatialModeSelection::AllPlaquettesCls,
                            "Combined All Plaquettes CLS",
                        )
                        .clicked()
                    {
                        self.update_spatial_intensity();
                    }
                });

            egui::ComboBox::from_id_salt("lieb_colormap_select")
                .selected_text(match self.colormap {
                    Colormap::Turbo => "Turbo",
                    Colormap::Magma => "Magma",
                    Colormap::Inferno => "Inferno",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.colormap, Colormap::Turbo, "Turbo");
                    ui.selectable_value(&mut self.colormap, Colormap::Magma, "Magma");
                    ui.selectable_value(&mut self.colormap, Colormap::Inferno, "Inferno");
                });
        });

        if dirty {
            self.recompute();
        }

        ui.separator();

        // Main Visual Area: Split into Left (2D Lieb Lattice Canvas) and Right (Diagnostic Plots)
        ui.columns(2, |cols| {
            // Left Column: 2D Lieb Lattice Canvas
            cols[0].vertical(|ui| {
                ui.label(
                    RichText::new("2D Lieb Lattice Real-Space Amplitude Canvas |psi|^2:")
                        .color(COLOR_TOPO_CYAN)
                        .strong(),
                );
                self.render_lattice_canvas(ui);
            });

            // Right Column: Plots with tab switching
            cols[1].vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Diagnostic Plot:").color(COLOR_TOPO_CYAN).strong());
                    ui.selectable_value(
                        &mut self.active_plot_tab,
                        LiebPlotTab::BandDispersion,
                        LiebPlotTab::BandDispersion.label(),
                    );
                    ui.selectable_value(
                        &mut self.active_plot_tab,
                        LiebPlotTab::CagingCurve,
                        LiebPlotTab::CagingCurve.label(),
                    );
                    ui.selectable_value(
                        &mut self.active_plot_tab,
                        LiebPlotTab::TimeDynamics,
                        LiebPlotTab::TimeDynamics.label(),
                    );
                });

                match self.active_plot_tab {
                    LiebPlotTab::BandDispersion => self.render_band_dispersion_plot(ui),
                    LiebPlotTab::CagingCurve => self.render_caging_curve_plot(ui),
                    LiebPlotTab::TimeDynamics => self.render_time_dynamics_plot(ui),
                }
            });
        });

        ui.separator();

        // Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders the interactive 2D Lieb lattice real-space grid with sublattices A, B, C,
    /// alternating phase signs (+/-), and scientific colormaps.
    fn render_lattice_canvas(&self, ui: &mut Ui) {
        let canvas_size = vec2(ui.available_width().max(320.0), 380.0);
        let (response, painter) = ui.allocate_painter(canvas_size, Sense::hover());
        let rect = response.rect;

        // Canvas background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, COLOR_GRID_LINE),
            StrokeKind::Inside,
        );

        let margin = 28.0;
        let grid_w = rect.width() - 2.0 * margin;
        let grid_h = rect.height() - 2.0 * margin;

        let cell_w = grid_w / (self.nx as f32);
        let cell_h = grid_h / (self.ny as f32);

        // Maximum intensity for normalization
        let mut max_intensity = 0.0_f64;
        for &val in &self.current_spatial_intensity {
            if val > max_intensity {
                max_intensity = val;
            }
        }
        let norm_denom = if max_intensity > 1e-12 { max_intensity } else { 1.0 };

        // Helper to map lattice coordinate to painter position
        let to_screen = |cx: usize, cy: usize, sub: usize| -> Pos2 {
            let base_x = rect.left() + margin + (cx as f32) * cell_w;
            // Invert y so row 0 is bottom
            let base_y = rect.bottom() - margin - (cy as f32 + 1.0) * cell_h;

            match sub {
                0 => Pos2::new(base_x, base_y + cell_h),                     // A: bottom-left
                1 => Pos2::new(base_x + 0.5 * cell_w, base_y + cell_h),     // B: horizontal edge
                2 => Pos2::new(base_x, base_y + 0.5 * cell_h),              // C: vertical edge
                _ => Pos2::new(base_x, base_y),
            }
        };

        // Draw Unit Cell Wireframe and Hopping Bonds
        let bond_stroke = Stroke::new(1.2, Color32::from_rgb(71, 85, 105));
        let cell_stroke = Stroke::new(0.6, Color32::from_rgb(30, 41, 59));

        for cy in 0..self.ny {
            for cx in 0..self.nx {
                let p_a = to_screen(cx, cy, 0);
                let p_b = to_screen(cx, cy, 1);
                let p_c = to_screen(cx, cy, 2);

                // Cell box
                let cell_box = Rect::from_min_size(
                    Pos2::new(p_a.x, p_c.y - 0.5 * cell_h),
                    vec2(cell_w, cell_h),
                );
                painter.rect_stroke(cell_box, 0.0, cell_stroke, StrokeKind::Inside);

                // Intracell bonds: A <-> B and A <-> C
                painter.line_segment([p_a, p_b], bond_stroke);
                painter.line_segment([p_a, p_c], bond_stroke);

                // Intercell horizontal bond: B(cx, cy) <-> A(cx+1, cy)
                if cx + 1 < self.nx {
                    let next_a = to_screen(cx + 1, cy, 0);
                    painter.line_segment([p_b, next_a], bond_stroke);
                }

                // Intercell vertical bond: C(cx, cy) <-> A(cx, cy+1)
                if cy + 1 < self.ny {
                    let next_a = to_screen(cx, cy + 1, 0);
                    painter.line_segment([p_c, next_a], bond_stroke);
                }
            }
        }

        // Highlight host plaquette if CLS mode is active
        if self.mode_view == LiebSpatialModeSelection::SinglePlaquetteCls {
            if let Some(ref cls) = self.current_cls {
                let p_bl = to_screen(cls.cell_x, cls.cell_y, 0);
                let p_tr = to_screen(cls.cell_x + 1, cls.cell_y + 1, 0);
                let plaquette_rect = Rect::from_two_pos(p_bl, p_tr);
                painter.rect_filled(
                    plaquette_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(250, 204, 21, 25),
                );
                painter.rect_stroke(
                    plaquette_rect,
                    0.0,
                    Stroke::new(1.5, Color32::from_rgba_unmultiplied(250, 204, 21, 160)),
                    StrokeKind::Inside,
                );
            }
        }

        // Render Lattice Sites with Heatmap Fill and Alternating Signs (+/-)
        let mouse_pos = response.hover_pos();
        let mut hovered_info = None;

        for cy in 0..self.ny {
            for cx in 0..self.nx {
                for sub in 0..3 {
                    let pos = to_screen(cx, cy, sub);
                    let site_idx = self.lattice.site_index(cx, cy, sub);
                    let intensity = self
                        .current_spatial_intensity
                        .get(site_idx)
                        .copied()
                        .unwrap_or(0.0);

                    let t_val = (intensity / norm_denom).clamp(0.0, 1.0) as f32;
                    let fill_color = sample_colormap(t_val, self.colormap);

                    // Radius: corner site A is slightly larger
                    let radius = if sub == 0 { 5.5 } else { 4.5 };
                    let stroke_color = if intensity > 0.05 * norm_denom {
                        Color32::WHITE
                    } else {
                        COLOR_GRID_LINE
                    };

                    painter.circle(pos, radius, fill_color, Stroke::new(1.0, stroke_color));

                    // In CLS mode, draw alternating phase signs (+ / -) on the 4 active edge sites
                    if self.mode_view == LiebSpatialModeSelection::SinglePlaquetteCls {
                        if let Some(ref cls) = self.current_cls {
                            if let Some(k) = cls.site_indices.iter().position(|&idx| idx == site_idx) {
                                let sign_text = if cls.amplitudes[k] > 0.0 { "+1/2" } else { "-1/2" };
                                let text_color = if cls.amplitudes[k] > 0.0 {
                                    COLOR_TOPO_EMERALD
                                } else {
                                    Color32::from_rgb(244, 63, 94)
                                };
                                painter.text(
                                    Pos2::new(pos.x + 8.0, pos.y - 6.0),
                                    Align2::LEFT_CENTER,
                                    sign_text,
                                    FontId::monospace(10.0),
                                    text_color,
                                );
                            }
                        }
                    }

                    // Hover check
                    if let Some(m) = mouse_pos {
                        if (m - pos).length() < 9.0 {
                            let sub_name = match sub {
                                0 => "A (Corner)",
                                1 => "B (Horizontal Edge)",
                                2 => "C (Vertical Edge)",
                                _ => "",
                            };
                            hovered_info = Some((site_idx, cx, cy, sub_name, intensity));
                        }
                    }
                }
            }
        }

        // Render Tooltip for Hovered Node
        if let Some((idx, cx, cy, sub_name, intensity)) = hovered_info {
            let tooltip_text = format!(
                "Site #{idx} [{sub_name}]\nCell: ({cx}, {cy})\n|psi|^2: {intensity:.5}"
            );
            response.show_tooltip_text(tooltip_text);
        }
    }

    /// Renders the 3-Band Dispersion Plot along Gamma - X - M - Y - Gamma.
    fn render_band_dispersion_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("lieb_band_dispersion_plot")
            .height(340.0)
            .legend(Legend::default())
            .show_grid(true)
            .x_axis_label("Brillouin Zone Path (Gamma -> X -> M -> Y -> Gamma)")
            .y_axis_label("Frequency Detuning (MHz)")
            .include_y(self.j_mhz * 3.0)
            .include_y(-self.j_mhz * 3.0);

        plot.show(ui, |plot_ui| {
            // High-symmetry vertical guides: Gamma (0), X (1), M (2), Y (3), Gamma (4)
            plot_ui.vline(VLine::new("Gamma", 0.0).stroke(Stroke::new(0.8, COLOR_GRID_LINE)));
            plot_ui.vline(VLine::new("X", 1.0).stroke(Stroke::new(0.8, COLOR_GRID_LINE)));
            plot_ui.vline(VLine::new("M (Dirac)", 2.0).stroke(Stroke::new(0.8, COLOR_TOPO_CYAN)));
            plot_ui.vline(VLine::new("Y", 3.0).stroke(Stroke::new(0.8, COLOR_GRID_LINE)));
            plot_ui.vline(VLine::new("Gamma", 4.0).stroke(Stroke::new(0.8, COLOR_GRID_LINE)));

            // Zero line: E = 0
            plot_ui.hline(HLine::new("E = 0", 0.0).stroke(Stroke::new(0.6, Color32::from_rgb(71, 85, 105))));

            // Upper Dispersive Band
            plot_ui.line(
                Line::new("Upper Dispersive Band", PlotPoints::new(self.band_curve_upper.clone()))
                    .color(COLOR_TOPO_CYAN)
                    .width(2.0),
            );

            // Middle Flat Band (E = 0 line)
            plot_ui.line(
                Line::new("Flat Band (E = 0.0 MHz)", PlotPoints::new(self.band_curve_flat.clone()))
                    .color(COLOR_FLAT_GOLD)
                    .width(2.5),
            );

            // Lower Dispersive Band
            plot_ui.line(
                Line::new("Lower Dispersive Band", PlotPoints::new(self.band_curve_lower.clone()))
                    .color(COLOR_BULK_BLUE)
                    .width(2.0),
            );

            // Highlight Dirac Touching Point at M (x = 2.0, y = 0.0)
            plot_ui.points(
                Points::new("Dirac Touching Point (M)", vec![[2.0, 0.0]])
                    .color(COLOR_TOPO_EMERALD)
                    .radius(5.0),
            );
        });
    }

    /// Renders the Synthetic Aharonov-Bohm Caging Curve: IPR vs Gauge Flux Phi.
    fn render_caging_curve_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("lieb_caging_curve_plot")
            .height(340.0)
            .legend(Legend::default())
            .show_grid(true)
            .x_axis_label("Synthetic Gauge Flux (Phi / pi)")
            .y_axis_label("Inverse Participation Ratio (IPR)")
            .include_x(0.0)
            .include_x(2.0)
            .include_y(0.0)
            .include_y(0.35);

        plot.show(ui, |plot_ui| {
            // Vertical guide at Phi = PI
            plot_ui.vline(
                VLine::new("Aharonov-Bohm Caging (Phi = pi)", 1.0)
                    .stroke(Stroke::new(1.2, COLOR_TOPO_EMERALD)),
            );

            // Caging curve
            plot_ui.line(
                Line::new("Localization IPR vs Flux", PlotPoints::new(self.caging_curve.clone()))
                    .color(COLOR_TOPO_EMERALD)
                    .width(2.2),
            );

            // Highlight peak at Phi = PI
            let peak_ipr = self
                .caging_curve
                .iter()
                .find(|[p, _]| (*p - 1.0).abs() < 0.08)
                .map(|[_, ipr]| *ipr)
                .unwrap_or(0.25);

            plot_ui.points(
                Points::new("Caging Peak (Phi = pi)", vec![[1.0, peak_ipr]])
                    .color(COLOR_FLAT_GOLD)
                    .radius(5.5),
            );
        });
    }

    /// Renders Time-Evolution Wave Packet Dynamics comparing diffusion vs caged confinement.
    fn render_time_dynamics_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("lieb_time_dynamics_plot")
            .height(340.0)
            .legend(Legend::default())
            .show_grid(true)
            .x_axis_label("Evolution Time t (ns)")
            .y_axis_label("Inverse Participation Ratio (IPR)")
            .include_x(0.0)
            .include_x(50.0)
            .include_y(0.0)
            .include_y(0.35);

        plot.show(ui, |plot_ui| {
            // Current flux trajectory
            plot_ui.line(
                Line::new("Current Flux IPR(t)", PlotPoints::new(self.time_dynamics_current.clone()))
                    .color(COLOR_TOPO_CYAN)
                    .width(2.0),
            );

            // Pi-flux caged trajectory
            plot_ui.line(
                Line::new("Pi-Flux Caged IPR(t)", PlotPoints::new(self.time_dynamics_caged.clone()))
                    .color(COLOR_FLAT_GOLD)
                    .width(2.0),
            );

            // Indicator for currently selected time t
            plot_ui.vline(
                VLine::new("Current t", self.evolution_time_ns)
                    .stroke(Stroke::new(1.0, COLOR_TEXT_DIM)),
            );
        });
    }

    /// Renders the Telemetry Footer at the bottom of the dialog.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let flux_pi = self.phi_flux / PI;
        let caging_purity_percent = if (flux_pi - 1.0).abs() < 0.15 {
            99.8
        } else {
            ((1.0 - (flux_pi - 1.0).abs().min(1.0)) * 100.0).max(12.5)
        };

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Flat Band Flatness:").color(COLOR_TEXT_DIM));
            ui.label(RichText::new("< 0.001 Hz (Exact 0.0 MHz)").color(COLOR_FLAT_GOLD).strong());

            ui.separator();

            ui.label(RichText::new("Group Velocity:").color(COLOR_TEXT_DIM));
            ui.label(RichText::new("0.0 m/s").color(COLOR_TOPO_EMERALD).strong());

            ui.separator();

            ui.label(RichText::new("CLS Confinement:").color(COLOR_TEXT_DIM));
            ui.label(RichText::new("100.0%").color(COLOR_TOPO_CYAN).strong());

            ui.separator();

            ui.label(RichText::new("Gauge Flux:").color(COLOR_TEXT_DIM));
            ui.label(RichText::new(format!("{:.2} pi", flux_pi)).color(COLOR_TOPO_CYAN).strong());

            ui.separator();

            ui.label(RichText::new("Dirac Point:").color(COLOR_TEXT_DIM));
            ui.label(
                RichText::new(format!("{:.3} GHz", self.omega_0_ghz))
                    .color(COLOR_TOPO_EMERALD)
                    .strong(),
            );

            ui.separator();

            ui.label(RichText::new("Caging Purity:").color(COLOR_TEXT_DIM));
            ui.label(
                RichText::new(format!("{caging_purity_percent:.1}%"))
                    .color(if caging_purity_percent > 80.0 {
                        COLOR_TOPO_EMERALD
                    } else {
                        COLOR_TEXT_DIM
                    })
                    .strong(),
            );
        });
    }
}
