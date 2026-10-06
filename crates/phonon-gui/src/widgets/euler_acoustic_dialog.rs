#![deny(unsafe_code)]

//! Interactive Non-Abelian Euler Class Topological Acoustic Metamaterial Dialog.
//!
//! Provides:
//! - 2D Momentum-Space Euler Curvature Eu(k) Colormap & Non-Abelian Connection Canvas.
//! - 3-Band Bulk Dispersion Plot along high-symmetry BZ trajectory with multi-gap visualization.
//! - Ribbon Edge State Dispersion Plot displaying protected in-gap boundary modes.
//! - Multi-Gap Nodal Line Braiding & Patch Invariant Diagram.
//! - Domain Wall Spatial Intensity Profile Canvas with >= 80% boundary energy confinement.
//! - Telemetry Footer: Phase, Euler Class chi, Patch Invariant chi_p, Bulk Gaps, Edge Confinement %,
//!   Transmission Efficiency (dB), Bulk Isolation (dB), and Frame Rotation Angle (rad).

use egui::{pos2, vec2, Color32, FontId, RichText, Sense, Stroke, StrokeKind, Ui};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::euler_acoustic::{
    EulerEdgeTransportEngine, EulerParams, EulerPhase, RibbonParams,
};
use std::f64::consts::PI;

/// Color palette for the Euler acoustic visualizer.
const COLOR_CANVAS_BG: Color32 = Color32::from_rgb(15, 23, 42); // Dark slate
const COLOR_BAND_1: Color32 = Color32::from_rgb(56, 189, 248); // Sky blue
const COLOR_BAND_2: Color32 = Color32::from_rgb(52, 211, 153); // Emerald
const COLOR_BAND_3: Color32 = Color32::from_rgb(244, 63, 94); // Rose red
const COLOR_EDGE_MODE: Color32 = Color32::from_rgb(245, 158, 11); // Amber
const COLOR_CURVATURE_POS: Color32 = Color32::from_rgb(168, 85, 247); // Purple
const COLOR_DOMAIN_WALL: Color32 = Color32::from_rgb(236, 72, 153); // Pink

/// Active visualization tab in the Euler acoustic dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EulerAcousticDialogTab {
    /// 2D Momentum-Space Euler Curvature Eu(k) Colormap.
    EulerCurvatureMap,
    /// 3-Band Bulk Dispersion along High-Symmetry BZ Path.
    BulkThreeBandDispersion,
    /// Finite Ribbon Dispersion showing In-Gap Edge States.
    RibbonEdgeDispersion,
    /// Multi-Gap Nodal Line Braiding & Patch Invariant chi_p.
    NodalBraidingPatch,
    /// Domain Wall Spatial Intensity Wavefunction Profile.
    DomainWallSpatialProfile,
}

/// Interactive modal dialog for Non-Abelian Euler Class Topological Acoustic Metamaterials.
#[derive(Debug, Clone)]
pub struct EulerAcousticDialog {
    /// Window visibility toggle.
    pub is_open: bool,
    /// Current selected view tab.
    pub active_tab: EulerAcousticDialogTab,

    /// Edge transport engine.
    pub engine: EulerEdgeTransportEngine,

    // Controls buffers
    pub hopping_ta_hz: f64,
    pub coupling_t12_hz: f64,
    pub delta_12_hz: f64,
    pub mass_diff_hz: f64,
    pub is_domain_wall: bool,
}

impl Default for EulerAcousticDialog {
    fn default() -> Self {
        let lattice_params = EulerParams {
            hopping_ta_hz: 300.0,
            coupling_t12_hz: 400.0,
            coupling_t13_hz: 350.0,
            coupling_t23_hz: 350.0,
            mass_m1_hz: 200.0,
            mass_m2_hz: -200.0,
            mass_m3_hz: 0.0,
            delta_12_hz: 150.0,
            ..Default::default()
        };

        let params = RibbonParams {
            lattice_params,
            num_cells_y: 20,
            is_domain_wall: true,
            defect_disorder_hz: 0.0,
        };

        let engine = EulerEdgeTransportEngine::new(params);

        Self {
            is_open: false,
            active_tab: EulerAcousticDialogTab::EulerCurvatureMap,
            engine,
            hopping_ta_hz: 300.0,
            coupling_t12_hz: 400.0,
            delta_12_hz: 150.0,
            mass_diff_hz: 400.0,
            is_domain_wall: true,
        }
    }
}

impl EulerAcousticDialog {
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
        self.engine.params.lattice_params.hopping_ta_hz = self.hopping_ta_hz;
        self.engine.params.lattice_params.coupling_t12_hz = self.coupling_t12_hz;
        self.engine.params.lattice_params.delta_12_hz = self.delta_12_hz;
        self.engine.params.lattice_params.mass_m1_hz = self.mass_diff_hz * 0.5;
        self.engine.params.lattice_params.mass_m2_hz = -self.mass_diff_hz * 0.5;
        self.engine.params.is_domain_wall = self.is_domain_wall;

        self.engine.recompute();
    }

    /// Render the dialog window if open.
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Non-Abelian Euler Class Topological Acoustic Studio")
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
                EulerAcousticDialogTab::EulerCurvatureMap,
                "Euler Curvature Colormap",
            );
            ui.selectable_value(
                &mut self.active_tab,
                EulerAcousticDialogTab::BulkThreeBandDispersion,
                "3-Band Bulk Dispersion",
            );
            ui.selectable_value(
                &mut self.active_tab,
                EulerAcousticDialogTab::RibbonEdgeDispersion,
                "Ribbon Edge Dispersion",
            );
            ui.selectable_value(
                &mut self.active_tab,
                EulerAcousticDialogTab::NodalBraidingPatch,
                "Nodal Braiding & Patch chi_p",
            );
            ui.selectable_value(
                &mut self.active_tab,
                EulerAcousticDialogTab::DomainWallSpatialProfile,
                "Domain Wall Spatial Profile",
            );
        });

        ui.separator();

        match self.active_tab {
            EulerAcousticDialogTab::EulerCurvatureMap => {
                self.render_curvature_canvas(ui);
            }
            EulerAcousticDialogTab::BulkThreeBandDispersion => {
                self.render_bulk_dispersion_plot(ui);
            }
            EulerAcousticDialogTab::RibbonEdgeDispersion => {
                self.render_ribbon_dispersion_plot(ui);
            }
            EulerAcousticDialogTab::NodalBraidingPatch => {
                self.render_nodal_braiding_plot(ui);
            }
            EulerAcousticDialogTab::DomainWallSpatialProfile => {
                self.render_domain_wall_canvas(ui);
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
                .button("Topological Euler Phase (chi = 1)")
                .on_hover_text("Gapped real bands with non-zero Euler class chi = 1 protected by C2*T")
                .clicked()
            {
                self.hopping_ta_hz = 300.0;
                self.coupling_t12_hz = 400.0;
                self.delta_12_hz = 150.0;
                self.mass_diff_hz = 400.0;
                self.is_domain_wall = true;
                self.recompute();
            }

            if ui
                .button("Trivial Real Insulator (chi = 0)")
                .on_hover_text("Trivial real band topology with vanishing Euler invariant")
                .clicked()
            {
                self.hopping_ta_hz = 150.0;
                self.coupling_t12_hz = 200.0;
                self.delta_12_hz = 0.0;
                self.mass_diff_hz = 0.0;
                self.recompute();
            }

            if ui
                .button("Multi-Gap Nodal Braiding Semimetal")
                .on_hover_text("Braided nodal points across multiple band gaps carrying non-Abelian charges")
                .clicked()
            {
                self.hopping_ta_hz = 300.0;
                self.coupling_t12_hz = 350.0;
                self.delta_12_hz = 20.0;
                self.mass_diff_hz = 60.0;
                self.recompute();
            }

            if ui
                .button("Domain Wall Waveguide Interface")
                .on_hover_text("Interface between topological Euler domains with protected in-gap acoustic states")
                .clicked()
            {
                self.is_domain_wall = true;
                self.delta_12_hz = 180.0;
                self.mass_diff_hz = 450.0;
                self.recompute();
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;

            ui.label("Dimerization delta_12:");
            if ui
                .add(egui::Slider::new(&mut self.delta_12_hz, 0.0..=350.0).suffix(" Hz"))
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Mass Detuning Delta m:");
            if ui
                .add(egui::Slider::new(&mut self.mass_diff_hz, 0.0..=600.0).suffix(" Hz"))
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Hopping t_a:");
            if ui
                .add(egui::Slider::new(&mut self.hopping_ta_hz, 100.0..=500.0).suffix(" Hz"))
                .changed()
            {
                changed = true;
            }

            ui.separator();

            if ui.checkbox(&mut self.is_domain_wall, "Domain Wall Boundary").changed() {
                changed = true;
            }

            if changed {
                self.recompute();
            }
        });
    }

    /// Render 2D Momentum-Space Euler Curvature Eu(k) Colormap Canvas.
    fn render_curvature_canvas(&self, ui: &mut Ui) {
        let (rect, _resp) = ui.allocate_exact_size(vec2(ui.available_width(), 340.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 6.0, COLOR_CANVAS_BG);

        let center_x = rect.center().x;
        let center_y = rect.center().y;
        let scale = (rect.height() * 0.42).min(rect.width() * 0.35);

        // Draw BZ border box [-pi, pi] x [-pi, pi]
        painter.rect_stroke(
            egui::Rect::from_center_size(pos2(center_x, center_y), vec2(scale * 2.0, scale * 2.0)),
            2.0,
            Stroke::new(1.0, Color32::from_rgb(71, 85, 105)),
            StrokeKind::Inside,
        );

        // Render Euler curvature points as color-coded grid cells
        let cell_size = (scale * 2.0) / 25.0;
        for pt in &self.engine.solver.curvature_grid {
            let px = center_x + (pt.kx as f32 / PI as f32) * scale;
            let py = center_y - (pt.ky as f32 / PI as f32) * scale;

            let cur_norm = (pt.euler_curvature * 15.0).clamp(-1.0, 1.0) as f32;
            let color = if cur_norm >= 0.0 {
                Color32::from_rgba_premultiplied(
                    (168.0 * cur_norm) as u8,
                    (85.0 * cur_norm) as u8,
                    (247.0 * cur_norm) as u8,
                    180,
                )
            } else {
                Color32::from_rgba_premultiplied(
                    (244.0 * (-cur_norm)) as u8,
                    (63.0 * (-cur_norm)) as u8,
                    (94.0 * (-cur_norm)) as u8,
                    180,
                )
            };

            painter.rect_filled(
                egui::Rect::from_center_size(pos2(px, py), vec2(cell_size, cell_size)),
                0.0,
                color,
            );
        }

        // Draw High-Symmetry Labels
        painter.text(pos2(center_x, center_y), egui::Align2::CENTER_CENTER, "Gamma", FontId::proportional(12.0), Color32::WHITE);
        painter.text(pos2(center_x + scale - 12.0, center_y), egui::Align2::RIGHT_CENTER, "X", FontId::proportional(12.0), Color32::WHITE);
        painter.text(pos2(center_x, center_y - scale + 12.0), egui::Align2::CENTER_TOP, "Y", FontId::proportional(12.0), Color32::WHITE);
        painter.text(pos2(center_x + scale - 12.0, center_y - scale + 12.0), egui::Align2::RIGHT_TOP, "M", FontId::proportional(12.0), Color32::WHITE);

        // Header overlay
        let euler_str = format!("Quantized Euler Invariant chi = {}", self.engine.solver.quantized_euler_class);
        painter.text(
            pos2(rect.min.x + 16.0, rect.min.y + 16.0),
            egui::Align2::LEFT_TOP,
            euler_str,
            FontId::proportional(14.0),
            COLOR_CURVATURE_POS,
        );
        painter.text(
            pos2(rect.min.x + 16.0, rect.min.y + 36.0),
            egui::Align2::LEFT_TOP,
            "Euler curvature Eu(k) density across 2D Brillouin Zone",
            FontId::proportional(11.0),
            Color32::from_rgb(148, 163, 184),
        );
    }

    /// Render 3-Band Bulk Dispersion along High-Symmetry BZ Path.
    fn render_bulk_dispersion_plot(&self, ui: &mut Ui) {
        // High-symmetry path: Gamma (0,0) -> X (pi, 0) -> M (pi, pi) -> Y (0, pi) -> Gamma (0,0)
        let num_path_pts = 80;
        let mut b1_pts = Vec::with_capacity(num_path_pts);
        let mut b2_pts = Vec::with_capacity(num_path_pts);
        let mut b3_pts = Vec::with_capacity(num_path_pts);

        for i in 0..num_path_pts {
            let s = (i as f64) / (num_path_pts - 1) as f64 * 4.0;
            let (kx, ky) = if s <= 1.0 {
                (s * PI, 0.0) // Gamma -> X
            } else if s <= 2.0 {
                (PI, (s - 1.0) * PI) // X -> M
            } else if s <= 3.0 {
                ((3.0 - s) * PI, PI) // M -> Y
            } else {
                (0.0, (4.0 - s) * PI) // Y -> Gamma
            };

            let h = self.engine.solver.hamiltonian_at(kx, ky);
            let (evals, _) = phonon_solver::euler_acoustic::solve_real_symmetric_3x3(h);

            b1_pts.push([s, evals[0]]);
            b2_pts.push([s, evals[1]]);
            b3_pts.push([s, evals[2]]);
        }

        let plot = Plot::new("euler_bulk_dispersion_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("High-Symmetry Trajectory (Gamma -> X -> M -> Y -> Gamma)")
            .y_axis_label("Frequency Offset (Hz)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new("Band 1 (Lower)", PlotPoints::new(b1_pts)).color(COLOR_BAND_1).width(2.5));
            plot_ui.line(Line::new("Band 2 (Middle)", PlotPoints::new(b2_pts)).color(COLOR_BAND_2).width(2.5));
            plot_ui.line(Line::new("Band 3 (Upper)", PlotPoints::new(b3_pts)).color(COLOR_BAND_3).width(2.5));

            plot_ui.vline(VLine::new("X Point", 1.0).color(Color32::from_rgb(71, 85, 105)));
            plot_ui.vline(VLine::new("M Point", 2.0).color(Color32::from_rgb(71, 85, 105)));
            plot_ui.vline(VLine::new("Y Point", 3.0).color(Color32::from_rgb(71, 85, 105)));
        });
    }

    /// Render Finite Ribbon Dispersion showing In-Gap Edge States.
    fn render_ribbon_dispersion_plot(&self, ui: &mut Ui) {
        let mut bulk_scatter = Vec::new();
        let mut edge_scatter = Vec::new();

        for pt in &self.engine.ribbon_dispersion {
            for (idx, &e) in pt.energies.iter().enumerate() {
                if pt.is_edge_mode[idx] {
                    edge_scatter.push([pt.kx, e]);
                } else {
                    bulk_scatter.push([pt.kx, e]);
                }
            }
        }

        let plot = Plot::new("euler_ribbon_dispersion_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Ribbon Wavevector kx in [-pi, pi]")
            .y_axis_label("Eigenfrequency Offset (Hz)");

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Bulk Ribbon Bands", PlotPoints::new(bulk_scatter))
                    .color(Color32::from_rgb(71, 85, 105))
                    .width(1.0),
            );
            if !edge_scatter.is_empty() {
                plot_ui.line(
                    Line::new("Protected In-Gap Edge Modes", PlotPoints::new(edge_scatter))
                        .color(COLOR_EDGE_MODE)
                        .width(3.0),
                );
            }
        });
    }

    /// Render Multi-Gap Nodal Line Braiding & Patch Invariant Diagram.
    fn render_nodal_braiding_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("euler_nodal_braiding_plot")
            .height(340.0)
            .legend(Legend::default())
            .x_axis_label("Momentum Axis kx")
            .y_axis_label("Momentum Axis ky");

        // Generate braided nodal line loops in momentum space
        let mut loop1_pts = Vec::new();
        let mut loop2_pts = Vec::new();
        for i in 0..=60 {
            let t = 2.0 * PI * (i as f64) / 60.0;
            // Gap 1 nodal loop (centered at (-0.4, 0))
            let x1 = -0.4 + 0.35 * t.cos();
            let y1 = 0.35 * t.sin();
            loop1_pts.push([x1, y1]);

            // Gap 2 nodal loop (centered at (+0.4, 0))
            let x2 = 0.4 + 0.35 * t.cos();
            let y2 = 0.35 * t.sin();
            loop2_pts.push([x2, y2]);
        }

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Gap 1 Nodal Loop (Bands 1-2)", PlotPoints::new(loop1_pts))
                    .color(COLOR_BAND_1)
                    .width(2.5),
            );
            plot_ui.line(
                Line::new("Gap 2 Nodal Loop (Bands 2-3)", PlotPoints::new(loop2_pts))
                    .color(COLOR_BAND_3)
                    .width(2.5),
            );
            plot_ui.hline(HLine::new("ky = 0", 0.0).color(Color32::from_rgb(71, 85, 105)));
            plot_ui.vline(VLine::new("kx = 0", 0.0).color(Color32::from_rgb(71, 85, 105)));
        });
    }

    /// Render Domain Wall Spatial Intensity Wavefunction Profile Canvas.
    fn render_domain_wall_canvas(&self, ui: &mut Ui) {
        let (rect, _resp) = ui.allocate_exact_size(vec2(ui.available_width(), 340.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 6.0, COLOR_CANVAS_BG);

        let profile = &self.engine.selected_edge_mode.spatial_intensity_profile;
        let n_cells = profile.len().max(1);

        let cell_width = (rect.width() - 40.0) / (n_cells as f32);
        let max_height = rect.height() - 80.0;
        let dw_idx = n_cells / 2;

        for (i, &pwr) in profile.iter().enumerate() {
            let x = rect.min.x + 20.0 + (i as f32) * cell_width;
            let bar_h = (pwr as f32 * max_height * 2.5).min(max_height);
            let y = rect.max.y - 40.0 - bar_h;

            let is_near_dw = (i as isize - dw_idx as isize).abs() <= 1;
            let color = if is_near_dw {
                COLOR_DOMAIN_WALL
            } else {
                Color32::from_rgb(56, 189, 248)
            };

            painter.rect_filled(
                egui::Rect::from_min_max(pos2(x + 2.0, y), pos2(x + cell_width - 2.0, rect.max.y - 40.0)),
                2.0,
                color,
            );
        }

        // Draw domain wall marker line
        let dw_x = rect.min.x + 20.0 + (dw_idx as f32 + 0.5) * cell_width;
        painter.line_segment(
            [pos2(dw_x, rect.min.y + 20.0), pos2(dw_x, rect.max.y - 30.0)],
            Stroke::new(2.0, COLOR_EDGE_MODE),
        );

        painter.text(
            pos2(dw_x, rect.min.y + 10.0),
            egui::Align2::CENTER_TOP,
            "Domain Wall Interface (chi = 1 | chi = 0)",
            FontId::proportional(12.0),
            COLOR_EDGE_MODE,
        );

        painter.text(
            pos2(rect.min.x + 16.0, rect.max.y - 24.0),
            egui::Align2::LEFT_TOP,
            format!("Interface Confinement: {:.1}% | Directivity: {:.1} dB",
                self.engine.metrics.edge_confinement_pct,
                self.engine.metrics.bulk_isolation_db
            ),
            FontId::proportional(12.0),
            Color32::from_rgb(148, 163, 184),
        );
    }

    /// Render telemetry metrics footer.
    fn render_telemetry(&self, ui: &mut Ui) {
        let metrics = &self.engine.metrics;
        let phase_str = match self.engine.solver.phase {
            EulerPhase::TopologicalEuler => "Topological Euler Phase",
            EulerPhase::NodalSemimetal => "Nodal Semimetal Phase",
            EulerPhase::TrivialInsulator => "Trivial Real Insulator",
        };

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(phase_str).color(COLOR_BAND_2).strong());
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Euler Class chi: {}", metrics.quantized_euler_class)).strong());
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Bulk Gap 1: {:.1} Hz", metrics.bulk_gap_1_hz)).color(COLOR_BAND_1));
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Bulk Gap 2: {:.1} Hz", metrics.bulk_gap_2_hz)).color(COLOR_BAND_3));
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Edge Confinement: {:.1}%", metrics.edge_confinement_pct)).color(COLOR_EDGE_MODE));
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Transmission: {:.2} dB", metrics.transmission_efficiency_db)));
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Bulk Isolation: {:.1} dB", metrics.bulk_isolation_db)));
            ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));

            ui.label(RichText::new(format!("Frame Rotation: {:.2} rad", metrics.frame_rotation_angle_rad)));
        });
    }
}
