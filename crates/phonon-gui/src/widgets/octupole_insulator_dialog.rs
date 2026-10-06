#![deny(unsafe_code)]

//! Interactive Higher-Order Topological Acoustic Octupole Insulator & 3D Corner State Nanocavity Visualizer.
//!
//! Provides a 5-tab CAD simulation environment:
//! 1. 3D Cubic Nanocavity Spatial Density Canvas (isometric wireframe projection of 8 corner modes).
//! 2. 8-Band Bulk 3D Dispersion Plot (Gamma - X - M - R - Gamma path with bulk bandgap).
//! 3. Discrete Eigenmode Energy Spectrum (valence band, conduction band, and isolated zero modes).
//! 4. Surface & Hinge Gapping Hierarchy (dimensional cascade of topological multipole moments).
//! 5. Defect Immunity & Nanocavity Q-Factor (robustness against random acoustic disorder).

use egui::{pos2, vec2, Color32, Context, Pos2, Rect, RichText, Sense, Stroke, Ui, Window};
use egui_plot::{Line, Plot, PlotPoints, Points};
use phonon_solver::octupole_insulator::{
    BandPoint3D, CubicCornerId, DefectRobustnessPoint, OctupoleCubicLattice, OctupoleHamiltonian,
    OctupoleLatticeResult, OctupoleParams,
};
use std::f64::consts::PI;

/// Active tab in the 3D Octupole Insulator Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OctupoleDialogTab {
    SpatialDensity3D,
    BulkDispersion,
    DiscreteSpectrum,
    GappingHierarchy,
    DefectRobustness,
}

/// Modal dialog for Higher-Order Topological Acoustic Octupole Insulator simulation.
pub struct OctupoleInsulatorDialog {
    pub is_open: bool,
    pub active_tab: OctupoleDialogTab,
    pub gamma: f64,
    pub lambda: f64,
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub disorder_w: f64,
    pub selected_corner_idx: usize,

    pub hamiltonian: OctupoleHamiltonian,
    pub lattice: OctupoleCubicLattice,
    pub result: OctupoleLatticeResult,
    pub band_data: Vec<BandPoint3D>,
    pub robustness_data: Vec<DefectRobustnessPoint>,
}

impl Default for OctupoleInsulatorDialog {
    fn default() -> Self {
        let params = OctupoleParams {
            gamma: 2.0,
            lambda: 10.0,
            omega_0: 1.0,
            a_mm: 5.0,
        };

        let hamiltonian = OctupoleHamiltonian::new(params);
        let band_data = hamiltonian.band_structure(10);
        let lattice = OctupoleCubicLattice::new(params, 2, 2, 2);
        let result = lattice.solve();
        let disorder_levels = [0.0, 0.5, 1.0, 1.5, 2.0];
        let robustness_data = lattice.evaluate_defect_robustness(&disorder_levels, 42);

        Self {
            is_open: false,
            active_tab: OctupoleDialogTab::SpatialDensity3D,
            gamma: 2.0,
            lambda: 10.0,
            nx: 2,
            ny: 2,
            nz: 2,
            disorder_w: 0.0,
            selected_corner_idx: 0,
            hamiltonian,
            lattice,
            result,
            band_data,
            robustness_data,
        }
    }
}

impl OctupoleInsulatorDialog {
    /// Construct a new default dialog.
    pub fn new() -> Self {
        Self::default()
    }

    /// UI update pass for the dialog.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Recompute all solver fields and metrics from current buffer state.
    pub fn recompute(&mut self) {
        let params = OctupoleParams {
            gamma: self.gamma,
            lambda: self.lambda,
            omega_0: 1.0,
            a_mm: 5.0,
        };

        self.hamiltonian = OctupoleHamiltonian::new(params);
        self.band_data = self.hamiltonian.band_structure(10);

        self.lattice = OctupoleCubicLattice::new(params, self.nx, self.ny, self.nz);
        if self.disorder_w > 1e-6 {
            self.result = self.lattice.solve_with_disorder(self.disorder_w, 42);
        } else {
            self.result = self.lattice.solve();
        }

        let disorder_levels = [0.0, 0.5, 1.0, 1.5, 2.0];
        self.robustness_data = self.lattice.evaluate_defect_robustness(&disorder_levels, 42);
    }

    /// Primary display method.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new(RichText::new("Higher-Order Topological Acoustic Octupole Insulator").strong())
            .open(&mut is_open)
            .default_size(vec2(860.0, 680.0))
            .min_size(vec2(600.0, 500.0))
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
                OctupoleDialogTab::SpatialDensity3D,
                "3D Cubic Nanocavity Density",
            );
            ui.selectable_value(
                &mut self.active_tab,
                OctupoleDialogTab::BulkDispersion,
                "8-Band Bulk 3D Dispersion",
            );
            ui.selectable_value(
                &mut self.active_tab,
                OctupoleDialogTab::DiscreteSpectrum,
                "Discrete Eigenmode Spectrum",
            );
            ui.selectable_value(
                &mut self.active_tab,
                OctupoleDialogTab::GappingHierarchy,
                "Surface & Hinge Gapping Hierarchy",
            );
            ui.selectable_value(
                &mut self.active_tab,
                OctupoleDialogTab::DefectRobustness,
                "Defect Immunity & Q-Factor",
            );
        });

        ui.separator();

        match self.active_tab {
            OctupoleDialogTab::SpatialDensity3D => {
                self.render_spatial_density_canvas(ui);
            }
            OctupoleDialogTab::BulkDispersion => {
                self.render_bulk_dispersion_plot(ui);
            }
            OctupoleDialogTab::DiscreteSpectrum => {
                self.render_discrete_spectrum_plot(ui);
            }
            OctupoleDialogTab::GappingHierarchy => {
                self.render_gapping_hierarchy_panel(ui);
            }
            OctupoleDialogTab::DefectRobustness => {
                self.render_defect_robustness_plot(ui);
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
                .button("Topological Octupole (O_xyz = 1/2)")
                .on_hover_text("Quantized bulk octupole moment O_xyz = 1/2 with 8 localized corner states")
                .clicked()
            {
                self.gamma = 2.0;
                self.lambda = 10.0;
                self.disorder_w = 0.0;
                self.recompute();
            }

            if ui
                .button("Trivial Cubic Insulator (O_xyz = 0)")
                .on_hover_text("Trivial cubic phase (gamma > lambda) with no in-gap corner states")
                .clicked()
            {
                self.gamma = 10.0;
                self.lambda = 2.0;
                self.disorder_w = 0.0;
                self.recompute();
            }

            if ui
                .button("High-Q Corner Nanocavity")
                .on_hover_text("Deep topological confinement ratio >= 90% and quality factor Q > 10^6")
                .clicked()
            {
                self.gamma = 1.0;
                self.lambda = 12.0;
                self.disorder_w = 0.0;
                self.recompute();
            }

            if ui
                .button("Disordered HOTI Cube")
                .on_hover_text("Random acoustic coupling disorder W = 1.5 MHz demonstrating corner mode mid-gap pinning")
                .clicked()
            {
                self.gamma = 2.0;
                self.lambda = 10.0;
                self.disorder_w = 1.5;
                self.recompute();
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;

            ui.label("Intracell coupling gamma (MHz):");
            changed |= ui
                .add(egui::Slider::new(&mut self.gamma, 0.5..=15.0).text("gamma"))
                .changed();

            ui.separator();

            ui.label("Intercell coupling lambda (MHz):");
            changed |= ui
                .add(egui::Slider::new(&mut self.lambda, 0.5..=15.0).text("lambda"))
                .changed();

            ui.separator();

            ui.label("Disorder W (MHz):");
            changed |= ui
                .add(egui::Slider::new(&mut self.disorder_w, 0.0..=3.0).text("W"))
                .changed();

            if changed {
                self.recompute();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Lattice Size:");
            let is_2x2 = self.nx == 2 && self.ny == 2 && self.nz == 2;
            let is_3x3 = self.nx == 3 && self.ny == 3 && self.nz == 3;

            if ui.selectable_label(is_2x2, "2x2x2 (64 sites)").clicked() {
                self.nx = 2;
                self.ny = 2;
                self.nz = 2;
                self.recompute();
            }
            if ui.selectable_label(is_3x3, "3x3x3 (216 sites)").clicked() {
                self.nx = 3;
                self.ny = 3;
                self.nz = 3;
                self.recompute();
            }

            ui.separator();
            ui.label("Corner Inspector:");
            let corner_labels = [
                "All 8 Corners (Combined)",
                "Corner 1 [BSW (0,0,0)]",
                "Corner 2 [BSE (1,0,0)]",
                "Corner 3 [BNW (0,1,0)]",
                "Corner 4 [BNE (1,1,0)]",
                "Corner 5 [TSW (0,0,1)]",
                "Corner 6 [TSE (1,0,1)]",
                "Corner 7 [TNW (0,1,1)]",
                "Corner 8 [TNE (1,1,1)]",
            ];
            egui::ComboBox::from_id_salt("corner_inspector_select")
                .selected_text(corner_labels[self.selected_corner_idx])
                .show_ui(ui, |ui| {
                    for (i, &lbl) in corner_labels.iter().enumerate() {
                        ui.selectable_value(&mut self.selected_corner_idx, i, lbl);
                    }
                });
        });
    }

    /// Tab 1: Isometric 3D wireframe canvas of the cubic nanocavity.
    fn render_spatial_density_canvas(&self, ui: &mut Ui) {
        ui.label(RichText::new("3D Real-Space Acoustic Nanocavity Wavefunction Density").strong());
        ui.label("Isometric projection of the cubic lattice. 8 mid-gap zero modes are localized at the 8 cube vertices.");

        let available_size = ui.available_size();
        let (rect, _response) = ui.allocate_exact_size(
            vec2(available_size.x, available_size.y.max(280.0)),
            Sense::hover(),
        );

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 14, 20));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(45, 55, 75)),
            egui::StrokeKind::Inside,
        );

        let center_x = rect.center().x;
        let center_y = rect.center().y + 20.0;
        let scale = 65.0;

        // Isometric projection: (x, y, z) -> (2D screen x, 2D screen y)
        // x goes bottom-right (+cos 30 deg, +sin 30 deg)
        // y goes bottom-left (-cos 30 deg, +sin 30 deg)
        // z goes straight up (0, -1)
        let cos30 = (PI / 6.0).cos();
        let sin30 = (PI / 6.0).sin();

        let to_screen = |gx: f64, gy: f64, gz: f64| -> Pos2 {
            let offset_x = (gx - gy) * cos30 * scale;
            let offset_y = (gx + gy) * sin30 * scale - gz * scale;
            pos2(center_x + offset_x as f32, center_y + offset_y as f32)
        };

        // Draw bounding cube wireframe for the finite lattice
        let nx_f = (self.nx - 1) as f64;
        let ny_f = (self.ny - 1) as f64;
        let nz_f = (self.nz - 1) as f64;

        // Unit cell center coordinates for grid lines
        for cz in 0..self.nz {
            for cy in 0..self.ny {
                for cx in 0..self.nx {
                    let p0 = to_screen(cx as f64 - nx_f * 0.5, cy as f64 - ny_f * 0.5, cz as f64 - nz_f * 0.5);

                    if cx + 1 < self.nx {
                        let p1 = to_screen((cx + 1) as f64 - nx_f * 0.5, cy as f64 - ny_f * 0.5, cz as f64 - nz_f * 0.5);
                        painter.line_segment([p0, p1], Stroke::new(1.0, Color32::from_rgba_unmultiplied(60, 80, 110, 80)));
                    }
                    if cy + 1 < self.ny {
                        let p1 = to_screen(cx as f64 - nx_f * 0.5, (cy + 1) as f64 - ny_f * 0.5, cz as f64 - nz_f * 0.5);
                        painter.line_segment([p0, p1], Stroke::new(1.0, Color32::from_rgba_unmultiplied(60, 80, 110, 80)));
                    }
                    if cz + 1 < self.nz {
                        let p1 = to_screen(cx as f64 - nx_f * 0.5, cy as f64 - ny_f * 0.5, (cz + 1) as f64 - nz_f * 0.5);
                        painter.line_segment([p0, p1], Stroke::new(1.0, Color32::from_rgba_unmultiplied(60, 80, 110, 80)));
                    }
                }
            }
        }

        // Active intensity profile
        let intensity_slice: &[f64] = if self.selected_corner_idx == 0 {
            &self.result.combined_corner_intensity
        } else if self.selected_corner_idx - 1 < self.result.corner_states.len() {
            &self.result.corner_states[self.selected_corner_idx - 1].spatial_intensity
        } else {
            &self.result.combined_corner_intensity
        };

        // Render nodes with color-coded intensity
        let max_intensity = intensity_slice
            .iter()
            .copied()
            .fold(0.0_f64, |acc, v| acc.max(v))
            .max(1e-6);

        // Render the 8 corners with prominent badges and glowing spheres
        let corners = [
            (CubicCornerId::Corner000, 0.0, 0.0, 0.0, "BSW"),
            (CubicCornerId::Corner100, nx_f, 0.0, 0.0, "BSE"),
            (CubicCornerId::Corner010, 0.0, ny_f, 0.0, "BNW"),
            (CubicCornerId::Corner110, nx_f, ny_f, 0.0, "BNE"),
            (CubicCornerId::Corner001, 0.0, 0.0, nz_f, "TSW"),
            (CubicCornerId::Corner101, nx_f, 0.0, nz_f, "TSE"),
            (CubicCornerId::Corner011, 0.0, ny_f, nz_f, "TNW"),
            (CubicCornerId::Corner111, nx_f, ny_f, nz_f, "TNE"),
        ];

        for &(_cid, cx, cy, cz, label) in &corners {
            let p = to_screen(cx - nx_f * 0.5, cy - ny_f * 0.5, cz - nz_f * 0.5);

            // Compute total corner cell weight
            let mut corner_w = 0.0;
            let (cx_u, cy_u, cz_u) = (cx as usize, cy as usize, cz as usize);
            for sz in 0..2 {
                for sy in 0..2 {
                    for sx in 0..2 {
                        let idx = self.lattice.site_index(cx_u, cy_u, cz_u, sx, sy, sz);
                        if idx < intensity_slice.len() {
                            corner_w += intensity_slice[idx];
                        }
                    }
                }
            }

            let norm_w = (corner_w / max_intensity).clamp(0.0, 1.0);
            let radius = 6.0 + 10.0 * norm_w as f32;

            if self.result.is_topological {
                // Pulsating cyan/gold glow for topological corner states
                painter.circle_filled(p, radius + 4.0, Color32::from_rgba_unmultiplied(40, 200, 220, 60));
                painter.circle_filled(p, radius, Color32::from_rgb(0, 230, 255));
                painter.circle_stroke(p, radius, Stroke::new(1.5, Color32::WHITE));
            } else {
                painter.circle_filled(p, 5.0, Color32::from_rgb(100, 120, 140));
            }

            // Label text badge
            let text_color = if self.result.is_topological {
                Color32::from_rgb(255, 235, 120)
            } else {
                Color32::from_rgb(160, 175, 190)
            };
            painter.text(
                pos2(p.x, p.y - radius - 8.0),
                egui::Align2::CENTER_CENTER,
                format!("{} ({:.1}%)", label, corner_w * 100.0),
                egui::FontId::monospace(10.0),
                text_color,
            );
        }

        // Corner indicator legend
        let legend_rect = Rect::from_min_size(
            pos2(rect.min.x + 12.0, rect.min.y + 12.0),
            vec2(220.0, 70.0),
        );
        painter.rect_filled(legend_rect, 4.0, Color32::from_rgba_unmultiplied(20, 25, 35, 220));
        painter.rect_stroke(
            legend_rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(60, 75, 100)),
            egui::StrokeKind::Inside,
        );

        let phase_str = if self.result.is_topological {
            "Topological Octupole (O_xyz = 1/2)"
        } else {
            "Trivial Insulator (O_xyz = 0)"
        };
        painter.text(
            pos2(legend_rect.min.x + 8.0, legend_rect.min.y + 10.0),
            egui::Align2::LEFT_TOP,
            phase_str,
            egui::FontId::proportional(11.0),
            if self.result.is_topological {
                Color32::from_rgb(0, 255, 180)
            } else {
                Color32::from_rgb(255, 100, 100)
            },
        );
        painter.text(
            pos2(legend_rect.min.x + 8.0, legend_rect.min.y + 28.0),
            egui::Align2::LEFT_TOP,
            format!("8-Corner Confinement: {:.1}%", self.result.energy_confinement_ratio * 100.0),
            egui::FontId::monospace(10.5),
            Color32::from_rgb(220, 230, 240),
        );
        painter.text(
            pos2(legend_rect.min.x + 8.0, legend_rect.min.y + 46.0),
            egui::Align2::LEFT_TOP,
            format!("Nanocavity Q-Factor: {:.1e}", self.result.quality_factor),
            egui::FontId::monospace(10.5),
            Color32::from_rgb(220, 230, 240),
        );
    }

    /// Tab 2: 8-band bulk 3D dispersion plot.
    fn render_bulk_dispersion_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("3D Simple Cubic Brillouin Zone Dispersion").strong());
        ui.label("8 bulk bands along Gamma(0,0,0) -> X(pi,0,0) -> M(pi,pi,0) -> R(pi,pi,pi) -> Gamma(0,0,0).");

        let mut upper_pts = Vec::new();
        let mut lower_pts = Vec::new();

        for pt in &self.band_data {
            lower_pts.push([pt.k_dist, pt.energies[0]]);
            upper_pts.push([pt.k_dist, pt.energies[7]]);
        }

        let upper_line = Line::new("Conduction Bands (E > 0)", PlotPoints::new(upper_pts))
            .color(Color32::from_rgb(255, 120, 100))
            .width(2.0);

        let lower_line = Line::new("Valence Bands (E < 0)", PlotPoints::new(lower_pts))
            .color(Color32::from_rgb(100, 180, 255))
            .width(2.0);

        Plot::new("octupole_bulk_dispersion_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label("BZ Path (Gamma -> X -> M -> R -> Gamma)")
            .y_axis_label("Energy Detuning (MHz)")
            .height(280.0)
            .show(ui, |plot_ui| {
                plot_ui.line(upper_line);
                plot_ui.line(lower_line);
            });
    }

    /// Tab 3: Discrete eigenmode spectrum diagram.
    fn render_discrete_spectrum_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Finite Lattice Discrete Energy Spectrum").strong());
        ui.label("Ascending eigenmode energies. Isolated zero modes at E ~ 0 correspond to the 8 corner states.");

        let mut pts = Vec::new();
        let mut corner_pts = Vec::new();

        let mid_gap_limit = 0.1 * self.result.bulk_bandgap;

        for (idx, &e) in self.result.all_eigenvalues.iter().enumerate() {
            if e.abs() < mid_gap_limit && self.result.is_topological {
                corner_pts.push([idx as f64, e]);
            } else {
                pts.push([idx as f64, e]);
            }
        }

        let bulk_points = Points::new("Bulk & Surface Modes", PlotPoints::new(pts))
            .color(Color32::from_rgb(140, 160, 190))
            .radius(2.5);

        let corner_points = Points::new("8 Pinned Corner States (E ~ 0)", PlotPoints::new(corner_pts))
            .color(Color32::from_rgb(0, 240, 255))
            .radius(5.0);

        Plot::new("octupole_discrete_spectrum_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label("Eigenmode Index")
            .y_axis_label("Energy (MHz)")
            .height(280.0)
            .show(ui, |plot_ui| {
                plot_ui.points(bulk_points);
                plot_ui.points(corner_points);
            });
    }

    /// Tab 4: Surface and hinge gapping hierarchy.
    fn render_gapping_hierarchy_panel(&self, ui: &mut Ui) {
        ui.label(RichText::new("Dimensional Boundary Hierarchy of 3D Octupole Insulator").strong());
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            // Step 1: 3D Bulk
            ui.group(|ui| {
                ui.set_width(170.0);
                ui.label(RichText::new("3D Bulk").strong());
                ui.separator();
                ui.label(format!("Gap: {:.1} MHz", self.result.bulk_bandgap));
                ui.label(format!("Octupole O_xyz: {:.1}", self.hamiltonian.quantized_octupole_moment()));
                ui.label("Full 3D bandgap");
            });

            ui.label("->");

            // Step 2: 2D Surfaces
            ui.group(|ui| {
                ui.set_width(170.0);
                ui.label(RichText::new("2D Surfaces").strong());
                ui.separator();
                ui.label(format!("Quadrupole q_ij: {:.1}", self.hamiltonian.surface_quadrupole_moment()));
                ui.label("Gapped surfaces");
                ui.label("6 cubic faces");
            });

            ui.label("->");

            // Step 3: 1D Hinges
            ui.group(|ui| {
                ui.set_width(170.0);
                ui.label(RichText::new("1D Hinges").strong());
                ui.separator();
                ui.label(format!("Dipole p_i: {:.1}", self.hamiltonian.hinge_dipole_moment()));
                ui.label("Gapped hinges");
                ui.label("12 cubic edges");
            });

            ui.label("->");

            // Step 4: 0D Corners
            ui.group(|ui| {
                ui.set_width(180.0);
                ui.label(RichText::new("0D Corners").strong().color(Color32::from_rgb(0, 230, 255)));
                ui.separator();
                ui.label("Corner Charge: 1/2");
                ui.label(format!("8 Zero Modes: E ~ 0"));
                ui.label(format!("Confinement: {:.1}%", self.result.energy_confinement_ratio * 100.0));
            });
        });

        ui.add_space(16.0);
        ui.label(RichText::new("Topological Mechanism:").strong());
        ui.label("The bulk exhibits quantized pi-flux on every cubic plaquette, which systematically opens topological mass terms on the 2D surface Dirac cones and 1D hinge channels. This dimensional gapping cascade funnels acoustic energy exclusively into the 8 zero-dimensional corner nanocavities.");
    }

    /// Tab 5: Defect immunity and Q-factor scaling.
    fn render_defect_robustness_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Topological Corner Mode Defect Immunity").strong());
        ui.label("Corner energy pinning |E| and spatial energy confinement ratio vs random coupling disorder W.");

        let mut energy_pts = Vec::new();
        let mut conf_pts = Vec::new();

        for pt in &self.robustness_data {
            energy_pts.push([pt.disorder_w, pt.mean_corner_energy_mhz]);
            conf_pts.push([pt.disorder_w, pt.confinement_pct]);
        }

        let energy_line = Line::new("Mean Corner Energy (MHz)", PlotPoints::new(energy_pts))
            .color(Color32::from_rgb(255, 100, 100))
            .width(2.0);

        let conf_line = Line::new("Confinement Ratio (%)", PlotPoints::new(conf_pts))
            .color(Color32::from_rgb(0, 230, 255))
            .width(2.0);

        Plot::new("octupole_robustness_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label("Disorder Amplitude W (MHz)")
            .y_axis_label("Value")
            .height(280.0)
            .show(ui, |plot_ui| {
                plot_ui.line(energy_line);
                plot_ui.line(conf_line);
            });
    }

    /// Telemetry footer.
    fn render_telemetry(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let phase_text = if self.result.is_topological {
                RichText::new("Topological Octupole").color(Color32::from_rgb(0, 255, 180)).strong()
            } else {
                RichText::new("Trivial Insulator").color(Color32::from_rgb(255, 100, 100)).strong()
            };
            ui.label(phase_text);

            ui.separator();
            ui.label(format!("O_xyz: {:.1}", self.hamiltonian.quantized_octupole_moment()));

            ui.separator();
            ui.label(format!("Bulk Gap: {:.1} MHz", self.result.bulk_bandgap));

            ui.separator();
            ui.label(format!("Confinement: {:.1}%", self.result.energy_confinement_ratio * 100.0));

            ui.separator();
            ui.label(format!("Q: {:.1e}", self.result.quality_factor));

            ui.separator();
            ui.label(format!("xi: {:.2} mm", self.result.localization_length_mm));

            ui.separator();
            ui.label(format!("Corners: {}", self.result.corner_states.len()));
        });
    }
}
