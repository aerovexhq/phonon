#![deny(unsafe_code)]

//! Interactive Acoustic Skyrmion Vortex Lattice & Chiral Domain Wall Router Studio Dialog.
//!
//! Provides:
//! - 2D Real-Space Vector Field Canvas: interactive rendering of n(x, y) with color-coded n_z
//!   (blue core n_z = -1, red background n_z = +1) and in-plane vector glyph arrows indicating chirality (Neel vs Bloch).
//! - Topological Charge Density q(x, y) Colormap: renders topological charge density peaks.
//! - Topological Hall Deflection Trajectory Plot: native egui_plot of skyrmion path (x(t), y(t)) displaying Hall angle theta_H.
//! - Chiral Domain Wall S-Parameter Spectrum: native egui_plot of S_21(f) and S_12(f) in dB showing chiral isolation band (>= 30 dB) and defect immunity comparison.
//! - Controls & Presets: Skyrmion radius slider, lattice pitch slider, helicity gamma selector (Neel vs Bloch), driving force slider, defect toggle.
//! - Telemetry Footer: Topological Charge Q, Hall Angle theta_H (deg), Insertion Loss (dB), Chiral Isolation (dB), Corner Transmission (%), Skyrmion Radius (um).

use egui::{
    pos2, vec2, Color32, FontId, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints};
use phonon_solver::acoustic_skyrmion_router::{
    ChiralDomainWallRouter, DefectTransmissionResult, DomainWallDefect, SkyrmionLatticeParams,
    SkyrmionLatticeType, ThieleDynamics, TopologicalChargeCalculator, TrajectoryPoint, Vector3Field,
};
use std::f64::consts::PI;

/// Palette colors for the acoustic skyrmion studio.
const COLOR_SKYRMION_CORE: Color32 = Color32::from_rgb(37, 99, 235); // Deep Blue (n_z = -1)
const COLOR_BACKGROUND_UP: Color32 = Color32::from_rgb(220, 38, 38); // Crimson Red (n_z = +1)
const COLOR_EQUATOR_INPLANE: Color32 = Color32::from_rgb(30, 41, 59); // Slate Dark
const COLOR_ARROW_GLYPH: Color32 = Color32::from_rgb(254, 240, 138); // Light Gold
const COLOR_TRANSMISSION: Color32 = Color32::from_rgb(52, 211, 153); // Emerald
const COLOR_ISOLATION: Color32 = Color32::from_rgb(244, 63, 94); // Rose
const COLOR_REFLECTION: Color32 = Color32::from_rgb(245, 158, 11); // Amber
const COLOR_TRAJECTORY: Color32 = Color32::from_rgb(56, 189, 248); // Cyan
const COLOR_DEFECT_OBSTACLE: Color32 = Color32::from_rgb(249, 115, 22); // Orange

/// Active view tab in the skyrmion router studio dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyrmionDialogTab {
    /// 2D real-space vector field pseudospin canvas.
    RealSpaceVectorField,
    /// 2D topological charge density q(x, y) peak colormap.
    TopologicalChargeDensity,
    /// Thiele center-of-mass Hall deflection trajectory plot.
    HallDeflectionTrajectory,
    /// Chiral domain wall S-parameter frequency spectrum plot.
    DomainWallSpectrum,
}

/// Interactive modal dialog for the Acoustic Skyrmion Vortex Lattice & Domain Wall Router Studio.
#[derive(Debug, Clone, PartialEq)]
pub struct SkyrmionRouterDialog {
    /// Window visibility toggle.
    pub is_open: bool,
    /// Current selected view tab.
    pub active_tab: SkyrmionDialogTab,

    // Controls
    /// Physical lattice and vector field parameters.
    pub params: SkyrmionLatticeParams,
    /// Thiele center of mass dynamics model.
    pub thiele: ThieleDynamics,
    /// Chiral domain wall acoustic router model.
    pub router: ChiralDomainWallRouter,
    /// Active domain wall defect obstacle kind.
    pub active_defect: DomainWallDefect,

    // Cached Simulation State
    /// Discrete 3D unit vector field n(x, y).
    pub vector_field: Vector3Field,
    /// Spatial topological charge density values q(x, y).
    pub charge_density: Vec<f64>,
    /// Continuous integrated topological charge Q.
    pub computed_charge: f64,
    /// Discrete solid angle lattice topological charge Q.
    pub discrete_charge: f64,
    /// Skyrmion center of mass drift trajectory points.
    pub trajectory: Vec<TrajectoryPoint>,
    /// Chiral domain wall defect immunity metrics.
    pub defect_result: DefectTransmissionResult,

    // Cached Plot Curves
    /// Trajectory curve [x_um, y_um].
    pub trajectory_curve: Vec<[f64; 2]>,
    /// Forward transmission S_21 curve [freq_khz, s21_db].
    pub s21_curve: Vec<[f64; 2]>,
    /// Backward isolation S_12 curve [freq_khz, s12_db].
    pub s12_curve: Vec<[f64; 2]>,
    /// Reflection S_11 curve [freq_khz, s11_db].
    pub s11_curve: Vec<[f64; 2]>,

    // Telemetry Metrics
    /// Topological Acoustic Hall Angle in degrees.
    pub hall_angle_deg: f64,
    /// Steady-state drift velocity components [vx, vy] in m/s.
    pub drift_velocity_mps: [f64; 2],
    /// Forward transmission insertion loss in dB.
    pub insertion_loss_db: f64,
    /// Backward chiral isolation in dB.
    pub isolation_db: f64,
    /// Defect transmission ratio as percentage (T_defect / T_clean * 100).
    pub corner_transmission_percent: f64,
    /// Status message string.
    pub status_msg: String,
}

impl Default for SkyrmionRouterDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SkyrmionRouterDialog {
    /// Creates a new `SkyrmionRouterDialog` initialized with default physical parameters.
    pub fn new() -> Self {
        let mut dialog = Self {
            is_open: false,
            active_tab: SkyrmionDialogTab::RealSpaceVectorField,
            params: SkyrmionLatticeParams::default(),
            thiele: ThieleDynamics::default(),
            router: ChiralDomainWallRouter::default(),
            active_defect: DomainWallDefect::None,
            vector_field: Vector3Field::new(64, 64, 400.0),
            charge_density: Vec::new(),
            computed_charge: -1.0,
            discrete_charge: -1.0,
            trajectory: Vec::new(),
            defect_result: DefectTransmissionResult {
                defect: DomainWallDefect::None,
                clean_transmission: 0.965,
                defect_transmission: 0.965,
                transmission_ratio: 1.0,
                backscattering_suppression_db: 45.0,
            },
            trajectory_curve: Vec::new(),
            s21_curve: Vec::new(),
            s12_curve: Vec::new(),
            s11_curve: Vec::new(),
            hall_angle_deg: 0.0,
            drift_velocity_mps: [0.0, 0.0],
            insertion_loss_db: 0.35,
            isolation_db: 33.1,
            corner_transmission_percent: 100.0,
            status_msg: String::new(),
        };

        dialog.recompute();
        dialog
    }

    /// Recomputes all physical fields, topological charges, Thiele dynamics, and router spectra.
    pub fn recompute(&mut self) {
        // 1. Build Vector Field
        self.vector_field = Vector3Field::from_params(&self.params);

        // 2. Compute Topological Charge Density and Integral
        let calc = TopologicalChargeCalculator::new();
        let periodic = matches!(
            self.params.lattice_type,
            SkyrmionLatticeType::HexagonalVortexLattice | SkyrmionLatticeType::SquareLattice
        );
        self.charge_density = calc.compute_charge_density(&self.vector_field, periodic);
        self.computed_charge = calc.compute_total_charge(&self.vector_field, periodic);
        self.discrete_charge =
            calc.compute_lattice_solid_angle_charge(&self.vector_field, periodic);

        // 3. Thiele Dynamics Simulation
        self.thiele.q_topo = if self.computed_charge.abs() > 0.3 {
            self.computed_charge.signum()
        } else {
            -1.0
        };
        self.thiele.alpha = self.params.dissipation_alpha;
        self.drift_velocity_mps = self.thiele.compute_drift_velocity();
        self.hall_angle_deg = self.thiele.compute_hall_angle_deg();
        self.trajectory = self.thiele.compute_trajectory([0.0, 0.0], 1e-3, 60);
        self.trajectory_curve = self
            .trajectory
            .iter()
            .map(|p| [p.x_um, p.y_um])
            .collect();

        // 4. Chiral Domain Wall Waveguide Routing
        self.router = self.router.clone().with_defect(self.active_defect);
        let spectrum = self.router.compute_spectrum(15.0, 35.0, 60);
        self.defect_result = self.router.evaluate_defect_immunity(self.active_defect);

        self.s21_curve = spectrum.points.iter().map(|p| [p.freq_khz, p.s21_db]).collect();
        self.s12_curve = spectrum.points.iter().map(|p| [p.freq_khz, p.s12_db]).collect();
        self.s11_curve = spectrum.points.iter().map(|p| [p.freq_khz, p.s11_db]).collect();

        let center_pt = self
            .router
            .compute_s_parameters_at(self.router.center_freq_khz, self.active_defect);
        self.insertion_loss_db = center_pt.s21_db;
        self.isolation_db = center_pt.s12_db;
        self.corner_transmission_percent = self.defect_result.transmission_ratio * 100.0;

        self.status_msg = format!(
            "Topological Charge Q = {:.3}, Hall Angle theta_H = {:.2} deg, IL = {:.2} dB, Chiral Iso = {:.1} dB, Corner T = {:.1}%",
            self.computed_charge,
            self.hall_angle_deg,
            self.insertion_loss_db,
            self.isolation_db,
            self.corner_transmission_percent
        );
    }

    /// Renders the modal dialog window into the current `egui::Context`.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Phonon Topological Acoustic Skyrmion Router Studio")
            .open(&mut is_open)
            .default_size([1120.0, 740.0])
            .min_size([900.0, 600.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the full dialog content: top control panel, view tabs, main canvas/plot, and footer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut needs_recompute = false;

        // 1. Preset & Parameter Controls Bar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 4.0);

            // Presets
            ui.label(RichText::new("Presets:").strong().size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui.button("Single Skyrmion (Q = -1)").clicked() {
                self.params.lattice_type = SkyrmionLatticeType::SingleSkyrmion;
                self.params.skyrmion_radius_um = 50.0;
                self.params.vorticity_m = 1;
                self.params.helicity_gamma_rad = 0.0; // Neel
                self.thiele.force_un = [10.0, 0.0];
                self.active_defect = DomainWallDefect::None;
                self.active_tab = SkyrmionDialogTab::RealSpaceVectorField;
                needs_recompute = true;
            }
            if ui.button("Hexagonal Vortex Lattice").clicked() {
                self.params.lattice_type = SkyrmionLatticeType::HexagonalVortexLattice;
                self.params.lattice_pitch_um = 160.0;
                self.params.vorticity_m = 1;
                self.params.helicity_gamma_rad = PI / 2.0; // Bloch
                self.thiele.force_un = [10.0, 0.0];
                self.active_defect = DomainWallDefect::None;
                self.active_tab = SkyrmionDialogTab::RealSpaceVectorField;
                needs_recompute = true;
            }
            if ui.button("Chiral Domain Wall Waveguide").clicked() {
                self.params.lattice_type = SkyrmionLatticeType::DomainWallInterface;
                self.params.skyrmion_radius_um = 50.0;
                self.params.helicity_gamma_rad = 0.0;
                self.active_defect = DomainWallDefect::CornerBend90;
                self.active_tab = SkyrmionDialogTab::DomainWallSpectrum;
                needs_recompute = true;
            }
            if ui.button("Topological Hall Deflection").clicked() {
                self.params.lattice_type = SkyrmionLatticeType::SingleSkyrmion;
                self.params.skyrmion_radius_um = 60.0;
                self.params.vorticity_m = 1;
                self.thiele.force_un = [20.0, 0.0];
                self.active_tab = SkyrmionDialogTab::HallDeflectionTrajectory;
                needs_recompute = true;
            }
        });

        ui.add_space(4.0);

        // Parameter Sliders Bar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 4.0);

            // Radius slider
            ui.label(RichText::new("Radius:").size(11.0));
            if ui
                .add(
                    egui::Slider::new(&mut self.params.skyrmion_radius_um, 10.0..=150.0)
                        .suffix(" um")
                        .logarithmic(false),
                )
                .changed()
            {
                needs_recompute = true;
            }

            // Lattice Pitch slider
            ui.label(RichText::new("Pitch:").size(11.0));
            if ui
                .add(
                    egui::Slider::new(&mut self.params.lattice_pitch_um, 50.0..=300.0)
                        .suffix(" um"),
                )
                .changed()
            {
                needs_recompute = true;
            }

            // Helicity Gamma selector
            ui.label(RichText::new("Helicity:").size(11.0));
            let is_neel = self.params.helicity_gamma_rad.abs() < 0.1;
            if ui.selectable_label(is_neel, "Neel (0 deg)").clicked() && !is_neel {
                self.params.helicity_gamma_rad = 0.0;
                needs_recompute = true;
            }
            let is_bloch = (self.params.helicity_gamma_rad - PI / 2.0).abs() < 0.1;
            if ui.selectable_label(is_bloch, "Bloch (90 deg)").clicked() && !is_bloch {
                self.params.helicity_gamma_rad = PI / 2.0;
                needs_recompute = true;
            }

            // Driving Force slider
            ui.label(RichText::new("Drive F_x:").size(11.0));
            if ui
                .add(
                    egui::Slider::new(&mut self.thiele.force_un[0], 0.0..=50.0)
                        .suffix(" uN"),
                )
                .changed()
            {
                needs_recompute = true;
            }

            // Defect selector
            ui.label(RichText::new("Defect:").size(11.0));
            egui::ComboBox::from_id_salt("skyrmion_defect_sel")
                .selected_text(match self.active_defect {
                    DomainWallDefect::None => "None (Clean)",
                    DomainWallDefect::CornerBend90 => "90-Deg Corner Bend",
                    DomainWallDefect::CornerBend120 => "120-Deg Corner Bend",
                    DomainWallDefect::MissingResonator => "Missing Resonator Vacancy",
                    DomainWallDefect::BoundaryDisorder => "Boundary Disorder",
                })
                .show_ui(ui, |ui| {
                    if ui.selectable_label(self.active_defect == DomainWallDefect::None, "None (Clean)").clicked() {
                        self.active_defect = DomainWallDefect::None;
                        needs_recompute = true;
                    }
                    if ui.selectable_label(self.active_defect == DomainWallDefect::CornerBend90, "90-Deg Corner Bend").clicked() {
                        self.active_defect = DomainWallDefect::CornerBend90;
                        needs_recompute = true;
                    }
                    if ui.selectable_label(self.active_defect == DomainWallDefect::CornerBend120, "120-Deg Corner Bend").clicked() {
                        self.active_defect = DomainWallDefect::CornerBend120;
                        needs_recompute = true;
                    }
                    if ui.selectable_label(self.active_defect == DomainWallDefect::MissingResonator, "Missing Resonator Vacancy").clicked() {
                        self.active_defect = DomainWallDefect::MissingResonator;
                        needs_recompute = true;
                    }
                });
        });

        ui.add_space(4.0);

        // Action Buttons Bar
        ui.horizontal(|ui| {
            if ui.button("Calculate Topological Charge").clicked() {
                needs_recompute = true;
            }
            if ui.button("Simulate Thiele Drift").clicked() {
                self.active_tab = SkyrmionDialogTab::HallDeflectionTrajectory;
                needs_recompute = true;
            }
            if ui.button("Reset Field").clicked() {
                self.params = SkyrmionLatticeParams::default();
                self.thiele = ThieleDynamics::default();
                self.active_defect = DomainWallDefect::None;
                needs_recompute = true;
            }

            ui.separator();

            // Status message label
            ui.label(
                RichText::new(&self.status_msg)
                    .size(11.0)
                    .color(Color32::from_rgb(180, 220, 250)),
            );
        });

        if needs_recompute {
            self.recompute();
        }

        ui.separator();

        // 2. View Tabs Selection
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
            if ui
                .selectable_label(
                    self.active_tab == SkyrmionDialogTab::RealSpaceVectorField,
                    "2D Pseudospin Vector Field Canvas",
                )
                .clicked()
            {
                self.active_tab = SkyrmionDialogTab::RealSpaceVectorField;
            }
            if ui
                .selectable_label(
                    self.active_tab == SkyrmionDialogTab::TopologicalChargeDensity,
                    "Topological Charge Density q(x, y)",
                )
                .clicked()
            {
                self.active_tab = SkyrmionDialogTab::TopologicalChargeDensity;
            }
            if ui
                .selectable_label(
                    self.active_tab == SkyrmionDialogTab::HallDeflectionTrajectory,
                    "Thiele Hall Deflection Plot",
                )
                .clicked()
            {
                self.active_tab = SkyrmionDialogTab::HallDeflectionTrajectory;
            }
            if ui
                .selectable_label(
                    self.active_tab == SkyrmionDialogTab::DomainWallSpectrum,
                    "Chiral Domain Wall S-Parameter Spectrum",
                )
                .clicked()
            {
                self.active_tab = SkyrmionDialogTab::DomainWallSpectrum;
            }
        });

        ui.separator();

        // 3. Main Display Workspace
        let avail_h = (ui.available_height() - 55.0).max(360.0);
        match self.active_tab {
            SkyrmionDialogTab::RealSpaceVectorField => {
                self.render_vector_field_canvas(ui, avail_h);
            }
            SkyrmionDialogTab::TopologicalChargeDensity => {
                self.render_charge_density_canvas(ui, avail_h);
            }
            SkyrmionDialogTab::HallDeflectionTrajectory => {
                self.render_hall_trajectory_plot(ui, avail_h);
            }
            SkyrmionDialogTab::DomainWallSpectrum => {
                self.render_domain_wall_spectrum_plot(ui, avail_h);
            }
        }

        ui.separator();

        // 4. Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders the 2D Real-Space Vector Field Canvas showing n(x, y) with color-coded n_z
    /// and in-plane arrow glyphs.
    fn render_vector_field_canvas(&self, ui: &mut Ui, h: f32) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Real-Space Pseudo-Spin Field n(x, y): Blue core (n_z = -1), Red background (n_z = +1)")
                        .size(11.0)
                        .color(Color32::from_rgb(180, 205, 230)),
                );
            });

            let (canvas_rect, _resp) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
            let painter = ui.painter_at(canvas_rect);

            painter.rect_filled(canvas_rect, 4.0, COLOR_EQUATOR_INPLANE);
            painter.rect_stroke(canvas_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), StrokeKind::Inside);

            let side = canvas_rect.width().min(canvas_rect.height()) - 20.0;
            let center = canvas_rect.center();
            let view_rect = Rect::from_center_size(center, vec2(side, side));

            painter.rect_filled(view_rect, 2.0, Color32::BLACK);

            // Subsampled resolution for arrow glyphs and background tiles
            let target_grid = 24usize;
            let nx = self.vector_field.nx;
            let ny = self.vector_field.ny;
            let step_x = (nx / target_grid).max(1);
            let step_y = (ny / target_grid).max(1);

            let sub_nx = nx / step_x;
            let sub_ny = ny / step_y;
            let cell_w = side / (sub_nx as f32);
            let cell_h = side / (sub_ny as f32);

            for sj in 0..sub_ny {
                let j = sj * step_y;
                for si in 0..sub_nx {
                    let i = si * step_x;
                    let n = self.vector_field.get(i, j);

                    let c_min_x = view_rect.min.x + (si as f32) * cell_w;
                    let c_min_y = view_rect.min.y + (sj as f32) * cell_h;
                    let cell_box = Rect::from_min_size(pos2(c_min_x, c_min_y), vec2(cell_w, cell_h));

                    // Interpolate color based on n_z: -1 (Blue) -> +1 (Red)
                    let t = ((n[2] + 1.0) * 0.5).clamp(0.0, 1.0) as f32;
                    let r = ((1.0 - t) * (COLOR_SKYRMION_CORE.r() as f32) + t * (COLOR_BACKGROUND_UP.r() as f32)) as u8;
                    let g = ((1.0 - t) * (COLOR_SKYRMION_CORE.g() as f32) + t * (COLOR_BACKGROUND_UP.g() as f32)) as u8;
                    let b = ((1.0 - t) * (COLOR_SKYRMION_CORE.b() as f32) + t * (COLOR_BACKGROUND_UP.b() as f32)) as u8;
                    let cell_col = Color32::from_rgb(r, g, b);

                    painter.rect_filled(cell_box, 0.0, cell_col);

                    // Draw in-plane vector arrow glyph
                    let in_plane_mag = (n[0] * n[0] + n[1] * n[1]).sqrt();
                    if in_plane_mag > 0.08 {
                        let c_center = cell_box.center();
                        let arrow_len = (cell_w.min(cell_h) * 0.42 * (in_plane_mag as f32)).max(2.0);
                        let dir = vec2(n[0] as f32, -(n[1] as f32)); // screen y is inverted
                        let tip = c_center + dir * arrow_len;
                        let base = c_center - dir * (arrow_len * 0.4);

                        painter.line_segment([base, tip], Stroke::new(1.4, COLOR_ARROW_GLYPH));

                        // Arrow head wings
                        let perp = vec2(-dir.y, dir.x);
                        let wing_len = arrow_len * 0.35;
                        let w1 = tip - dir * wing_len + perp * (wing_len * 0.5);
                        let w2 = tip - dir * wing_len - perp * (wing_len * 0.5);
                        painter.line_segment([tip, w1], Stroke::new(1.2, COLOR_ARROW_GLYPH));
                        painter.line_segment([tip, w2], Stroke::new(1.2, COLOR_ARROW_GLYPH));
                    }
                }
            }

            // Legend badge in top-left
            painter.text(
                view_rect.min + vec2(10.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "Field: {:?} | Helicity: {:.1} rad | Vorticity m = {}",
                    self.params.lattice_type, self.params.helicity_gamma_rad, self.params.vorticity_m
                ),
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        });
    }

    /// Renders the Topological Charge Density Colormap q(x, y).
    fn render_charge_density_canvas(&self, ui: &mut Ui, h: f32) {
        ui.vertical(|ui| {
            ui.label(
                RichText::new("Topological Charge Density q(x, y) = (1 / 4pi) n . (dn/dx x dn/dy)")
                    .size(11.0)
                    .color(Color32::from_rgb(180, 205, 230)),
            );

            let (canvas_rect, _resp) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
            let painter = ui.painter_at(canvas_rect);

            painter.rect_filled(canvas_rect, 4.0, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(canvas_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), StrokeKind::Inside);

            let side = canvas_rect.width().min(canvas_rect.height()) - 20.0;
            let view_rect = Rect::from_center_size(canvas_rect.center(), vec2(side, side));

            let nx = self.vector_field.nx;
            let ny = self.vector_field.ny;

            // Find min/max density for normalization
            let mut min_q = 0.0f64;
            let mut max_q = 0.0f64;
            for &q in &self.charge_density {
                if q < min_q { min_q = q; }
                if q > max_q { max_q = q; }
            }
            let span = (max_q - min_q).max(1e-12);

            let cell_w = side / (nx as f32);
            let cell_h = side / (ny as f32);

            for j in 0..ny {
                for i in 0..nx {
                    let q = self.charge_density[j * nx + i];
                    let frac = ((q - min_q) / span).clamp(0.0, 1.0) as f32;

                    // Colormap: Peak negative density is bright yellow/orange, zero is dark slate
                    let r = (255.0 * (1.0 - frac) + 20.0 * frac) as u8;
                    let g = (200.0 * (1.0 - frac) + 30.0 * frac) as u8;
                    let b = (50.0 * (1.0 - frac) + 80.0 * frac) as u8;

                    let c_min_x = view_rect.min.x + (i as f32) * cell_w;
                    let c_min_y = view_rect.min.y + (j as f32) * cell_h;
                    let cell_box = Rect::from_min_size(pos2(c_min_x, c_min_y), vec2(cell_w, cell_h));

                    painter.rect_filled(cell_box, 0.0, Color32::from_rgb(r, g, b));
                }
            }

            // Legend badge
            painter.text(
                view_rect.min + vec2(10.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "Integrated Charge Q = {:.4} | Peak Density q_min = {:.2e} um^-2",
                    self.computed_charge, min_q
                ),
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        });
    }

    /// Renders the Topological Hall Deflection Trajectory Plot using native `egui_plot`.
    fn render_hall_trajectory_plot(&self, ui: &mut Ui, h: f32) {
        let pts = PlotPoints::new(self.trajectory_curve.clone());

        Plot::new("skyrmion_hall_plot")
            .height(h)
            .legend(Legend::default())
            .x_axis_label("X Position (um)")
            .y_axis_label("Transverse Y Position (um)")
            .include_y(15.0)
            .include_y(-15.0)
            .show(ui, |plot_ui| {
                // Undeflected forward reference ray (Y = 0)
                plot_ui.hline(
                    HLine::new("Undeflected Axis (F_y = 0)", 0.0)
                        .color(Color32::from_rgba_unmultiplied(148, 163, 184, 160))
                        .style(egui_plot::LineStyle::Dashed { length: 4.0 })
                        .width(1.0),
                );

                // Hall trajectory path
                plot_ui.line(
                    Line::new("Skyrmion Drift Trajectory (x(t), y(t))", pts)
                        .color(COLOR_TRAJECTORY)
                        .width(2.8),
                );
            });
    }

    /// Renders the Chiral Domain Wall S-Parameter Spectrum Plot using native `egui_plot`.
    fn render_domain_wall_spectrum_plot(&self, ui: &mut Ui, h: f32) {
        let s21_pts = PlotPoints::new(self.s21_curve.clone());
        let s12_pts = PlotPoints::new(self.s12_curve.clone());
        let s11_pts = PlotPoints::new(self.s11_curve.clone());

        Plot::new("domain_wall_s_param_plot")
            .height(h)
            .legend(Legend::default())
            .x_axis_label("Frequency (kHz)")
            .y_axis_label("Transmission / Isolation (dB)")
            .include_y(2.0)
            .include_y(-45.0)
            .show(ui, |plot_ui| {
                // S21 Forward Transmission
                plot_ui.line(
                    Line::new("Forward S_21 (Transmission)", s21_pts)
                        .color(COLOR_TRANSMISSION)
                        .width(2.5),
                );

                // S12 Reverse Isolation
                plot_ui.line(
                    Line::new("Backward S_12 (Chiral Isolation)", s12_pts)
                        .color(COLOR_ISOLATION)
                        .width(2.0),
                );

                // S11 Reflection
                plot_ui.line(
                    Line::new("Input Reflection S_11", s11_pts)
                        .color(COLOR_REFLECTION)
                        .width(1.5),
                );

                // 30 dB Isolation Target threshold
                plot_ui.hline(
                    HLine::new("Chiral Isolation Target (-30 dB)", -30.0)
                        .color(Color32::from_rgba_unmultiplied(244, 63, 94, 160))
                        .style(egui_plot::LineStyle::Dashed { length: 5.0 })
                        .width(1.2),
                );

                // 0.5 dB Insertion Loss threshold
                plot_ui.hline(
                    HLine::new("Max Insertion Loss Target (-0.5 dB)", -0.5)
                        .color(Color32::from_rgba_unmultiplied(52, 211, 153, 160))
                        .style(egui_plot::LineStyle::Dashed { length: 5.0 })
                        .width(1.2),
                );
            });
    }

    /// Renders the Telemetry Footer with essential physical and topological metrics.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(12.0, 0.0);

            // Topological Charge Q badge
            ui.label(
                RichText::new(format!("Topological Charge Q: {:.3}", self.computed_charge))
                    .strong()
                    .size(11.0)
                    .color(if (self.computed_charge.abs() - 1.0).abs() < 0.05 {
                        COLOR_TRANSMISSION
                    } else {
                        COLOR_TRAJECTORY
                    }),
            );

            // Hall Angle theta_H badge
            ui.label(
                RichText::new(format!("Hall Angle theta_H: {:.2} deg", self.hall_angle_deg))
                    .size(11.0)
                    .color(COLOR_TRAJECTORY),
            );

            // Insertion Loss badge
            ui.label(
                RichText::new(format!("Insertion Loss: {:.2} dB", self.insertion_loss_db))
                    .size(11.0)
                    .color(if self.insertion_loss_db <= 0.53 {
                        COLOR_TRANSMISSION
                    } else {
                        COLOR_REFLECTION
                    }),
            );

            // Chiral Isolation badge
            ui.label(
                RichText::new(format!("Chiral Isolation: {:.1} dB", self.isolation_db))
                    .size(11.0)
                    .color(if self.isolation_db >= 30.0 {
                        COLOR_TRANSMISSION
                    } else {
                        COLOR_ISOLATION
                    }),
            );

            // Corner Transmission badge
            ui.label(
                RichText::new(format!("Corner Transmission: {:.1}%", self.corner_transmission_percent))
                    .size(11.0)
                    .color(if self.corner_transmission_percent >= 95.0 {
                        COLOR_TRANSMISSION
                    } else {
                        COLOR_DEFECT_OBSTACLE
                    }),
            );

            // Skyrmion Radius badge
            ui.label(
                RichText::new(format!("Radius: {:.1} um", self.params.skyrmion_radius_um))
                    .size(11.0)
                    .color(Color32::from_rgb(180, 205, 230)),
            );
        });
    }
}
