#![deny(unsafe_code)]

//! Interactive Topological Acoustic Moire Quasicrystal & Aubry-Andre-Harper (AAH) Mobility Edge Visualizer.
//!
//! Provides a 5-tab CAD simulation environment:
//! 1. Real-Space Wavefunction & IPR Canvas (spatial eigenmode intensity and site localization).
//! 2. Hofstadter Butterfly Fractal Spectrum Plot (energy vs incommensurate flux beta in [0, 1]).
//! 3. Energy vs IPR / NPR Mobility Edge Scatter Plot (demonstrating mobility edge coexistence).
//! 4. Phason Evolution & Boundary Edge Mode Flow (energy levels vs phason phase phi in [0, 2*pi]).
//! 5. Fractal Dimension & Scaling Curves (multifractal dimension D2 vs Delta / J).

use egui::{pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, Ui, Window};
use egui_plot::{Line, Plot, PlotPoints, Points, VLine};
use phonon_solver::aah_quasicrystal::{
    AahHamiltonian, AahLatticeEngine, AahModelKind, AahParams, ButterflyPoint,
    GOLDEN_RATIO_CONJUGATE,
};
use std::f64::consts::PI;

/// Active tab in the AAH Quasicrystal Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AahDialogTab {
    SpatialWavefunction,
    HofstadterButterfly,
    MobilityEdgeScatter,
    PhasonEvolution,
    FractalScaling,
}

/// Modal dialog for Topological Acoustic AAH Quasicrystal simulation.
pub struct AahQuasicrystalDialog {
    pub is_open: bool,
    pub active_tab: AahDialogTab,

    pub hopping_j: f64,
    pub modulation_delta: f64,
    pub off_diagonal_lambda: f64,
    pub incommensurate_beta: f64,
    pub phason_phi: f64,
    pub model_kind: AahModelKind,
    pub num_sites: usize,
    pub selected_mode_idx: usize,

    pub engine: AahLatticeEngine,
    pub butterfly_data: Vec<ButterflyPoint>,
    pub phason_data: Vec<(f64, Vec<f64>)>,
}

impl Default for AahQuasicrystalDialog {
    fn default() -> Self {
        let params = AahParams::mobility_edge_phase();
        let engine = AahLatticeEngine::new_fast(params, 50);
        let butterfly_data = vec![ButterflyPoint {
            beta: 0.5,
            energies: vec![0.0],
        }];
        let phason_data = vec![(0.0, vec![0.0])];

        Self {
            is_open: false,
            active_tab: AahDialogTab::SpatialWavefunction,
            hopping_j: params.hopping_j,
            modulation_delta: params.modulation_delta,
            off_diagonal_lambda: params.off_diagonal_lambda,
            incommensurate_beta: params.incommensurate_beta,
            phason_phi: params.phason_phi,
            model_kind: params.model_kind,
            num_sites: 50,
            selected_mode_idx: 25,
            engine,
            butterfly_data,
            phason_data,
        }
    }
}

impl AahQuasicrystalDialog {
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
        let params = AahParams {
            hopping_j: self.hopping_j,
            modulation_delta: self.modulation_delta,
            off_diagonal_lambda: self.off_diagonal_lambda,
            incommensurate_beta: self.incommensurate_beta,
            phason_phi: self.phason_phi,
            hopping_phase_theta: 0.0,
            model_kind: self.model_kind,
        };

        self.engine = AahLatticeEngine::new(params, self.num_sites);
        let hamiltonian = AahHamiltonian::new(params);
        self.butterfly_data = hamiltonian.generate_hofstadter_butterfly(25, 30);
        self.phason_data = hamiltonian.generate_phason_spectrum(20, 30);

        if self.selected_mode_idx >= self.engine.eigenstates.len() {
            self.selected_mode_idx = self.engine.eigenstates.len().saturating_sub(1);
        }
    }

    /// Primary display method.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new(RichText::new("Topological Acoustic Moire Quasicrystal & AAH Mobility Edge Studio").strong())
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
                AahDialogTab::SpatialWavefunction,
                "Wavefunction & IPR Canvas",
            );
            ui.selectable_value(
                &mut self.active_tab,
                AahDialogTab::HofstadterButterfly,
                "Hofstadter Butterfly",
            );
            ui.selectable_value(
                &mut self.active_tab,
                AahDialogTab::MobilityEdgeScatter,
                "Mobility Edge Scatter",
            );
            ui.selectable_value(
                &mut self.active_tab,
                AahDialogTab::PhasonEvolution,
                "Phason Edge Flow",
            );
            ui.selectable_value(
                &mut self.active_tab,
                AahDialogTab::FractalScaling,
                "Fractal Dimension D2",
            );
        });

        ui.separator();

        match self.active_tab {
            AahDialogTab::SpatialWavefunction => {
                self.render_wavefunction_canvas(ui);
            }
            AahDialogTab::HofstadterButterfly => {
                self.render_butterfly_plot(ui);
            }
            AahDialogTab::MobilityEdgeScatter => {
                self.render_mobility_edge_plot(ui);
            }
            AahDialogTab::PhasonEvolution => {
                self.render_phason_plot(ui);
            }
            AahDialogTab::FractalScaling => {
                self.render_fractal_scaling_plot(ui);
            }
        }

        ui.separator();
        self.render_telemetry(ui);
    }

    /// Render presets and interactive controls.
    fn render_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Presets:").strong());

            if ui
                .button("Extended Bloch Phase (Delta/J < 2)")
                .on_hover_text("Delocalized Bloch-like states across the entire spectrum")
                .clicked()
            {
                let p = AahParams::extended_phase();
                self.hopping_j = p.hopping_j;
                self.modulation_delta = p.modulation_delta;
                self.off_diagonal_lambda = p.off_diagonal_lambda;
                self.model_kind = p.model_kind;
                self.recompute();
            }

            if ui
                .button("Critical Multifractal (Delta/J = 2)")
                .on_hover_text("Self-dual transition with scale-invariant multifractal wavefunctions")
                .clicked()
            {
                let p = AahParams::critical_phase();
                self.hopping_j = p.hopping_j;
                self.modulation_delta = p.modulation_delta;
                self.off_diagonal_lambda = p.off_diagonal_lambda;
                self.model_kind = p.model_kind;
                self.recompute();
            }

            if ui
                .button("Localized Anderson Phase (Delta/J > 2)")
                .on_hover_text("Exponentially localized acoustic states with vanishing transmission")
                .clicked()
            {
                let p = AahParams::localized_phase();
                self.hopping_j = p.hopping_j;
                self.modulation_delta = p.modulation_delta;
                self.off_diagonal_lambda = p.off_diagonal_lambda;
                self.model_kind = p.model_kind;
                self.recompute();
            }

            if ui
                .button("Mobility Edge Coexistence")
                .on_hover_text("Generalized AAH with energy-dependent mobility edge Ec separating extended and localized states")
                .clicked()
            {
                let p = AahParams::mobility_edge_phase();
                self.hopping_j = p.hopping_j;
                self.modulation_delta = p.modulation_delta;
                self.off_diagonal_lambda = p.off_diagonal_lambda;
                self.model_kind = p.model_kind;
                self.recompute();
            }

            if ui
                .button("Topological Phason Mode")
                .on_hover_text("Boundary-localized chiral edge states traversing the central bandgap")
                .clicked()
            {
                self.hopping_j = 5.0;
                self.modulation_delta = 7.0;
                self.off_diagonal_lambda = 0.0;
                self.phason_phi = PI * 0.5;
                self.model_kind = AahModelKind::StandardAah;
                self.recompute();
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;

            ui.label("Hopping J (MHz):");
            changed |= ui
                .add(egui::Slider::new(&mut self.hopping_j, 1.0..=10.0).text("J"))
                .changed();

            ui.separator();

            ui.label("Modulation Delta (MHz):");
            changed |= ui
                .add(egui::Slider::new(&mut self.modulation_delta, 0.0..=20.0).text("Delta"))
                .changed();

            ui.separator();

            ui.label("Off-Diagonal lambda:");
            changed |= ui
                .add(egui::Slider::new(&mut self.off_diagonal_lambda, 0.0..=0.8).text("lambda"))
                .changed();

            ui.separator();

            ui.label("Phason phi:");
            changed |= ui
                .add(egui::Slider::new(&mut self.phason_phi, 0.0..=2.0 * PI).text("phi"))
                .changed();

            if changed {
                self.recompute();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Model:");
            let mut model_changed = false;
            let current_kind = self.model_kind;
            egui::ComboBox::from_id_salt("aah_model_select")
                .selected_text(current_kind.label())
                .show_ui(ui, |ui| {
                    model_changed |= ui
                        .selectable_value(&mut self.model_kind, AahModelKind::StandardAah, AahModelKind::StandardAah.label())
                        .changed();
                    model_changed |= ui
                        .selectable_value(&mut self.model_kind, AahModelKind::GeneralizedAah, AahModelKind::GeneralizedAah.label())
                        .changed();
                    model_changed |= ui
                        .selectable_value(&mut self.model_kind, AahModelKind::MoireQuasicrystal2D, AahModelKind::MoireQuasicrystal2D.label())
                        .changed();
                });

            if model_changed {
                self.recompute();
            }

            ui.separator();
            ui.label("Chain Sites:");
            if ui.selectable_label(self.num_sites == 40, "40").clicked() {
                self.num_sites = 40;
                self.recompute();
            }
            if ui.selectable_label(self.num_sites == 60, "60").clicked() {
                self.num_sites = 60;
                self.recompute();
            }
            if ui.selectable_label(self.num_sites == 80, "80").clicked() {
                self.num_sites = 80;
                self.recompute();
            }

            ui.separator();
            ui.label("Mode Inspector:");
            let max_idx = self.engine.eigenstates.len().saturating_sub(1);
            ui.add(egui::Slider::new(&mut self.selected_mode_idx, 0..=max_idx).text("Mode"));
        });
    }

    /// Tab 1: Real-space wavefunction intensity and IPR canvas.
    fn render_wavefunction_canvas(&self, ui: &mut Ui) {
        ui.label(RichText::new("1D Quasiperiodic Acoustic Wavefunction Profile").strong());

        let mode = if self.selected_mode_idx < self.engine.eigenstates.len() {
            &self.engine.eigenstates[self.selected_mode_idx]
        } else {
            return;
        };

        ui.label(format!(
            "Mode #{}: Energy = {:.2} MHz | IPR = {:.4} | NPR = {:.4} | State: {}",
            mode.mode_index,
            mode.energy,
            mode.ipr,
            mode.npr,
            if mode.is_localized {
                "Exponentially Localized"
            } else if mode.is_extended {
                "Extended Bloch Wave"
            } else {
                "Critical Multifractal"
            }
        ));

        let available_size = ui.available_size();
        let (rect, _response) = ui.allocate_exact_size(
            vec2(available_size.x, available_size.y.max(260.0)),
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

        let n = mode.spatial_intensity.len();
        if n == 0 {
            return;
        }

        let margin_x = 30.0;
        let margin_y = 30.0;
        let plot_w = rect.width() - 2.0 * margin_x;
        let plot_h = rect.height() - 2.0 * margin_y;

        let max_val = mode
            .spatial_intensity
            .iter()
            .copied()
            .fold(0.0_f64, |acc, v| acc.max(v))
            .max(1e-6);

        let bar_width = (plot_w / n as f32).max(2.0);

        // Draw baseline
        let baseline_y = rect.max.y - margin_y;
        painter.line_segment(
            [pos2(rect.min.x + margin_x, baseline_y), pos2(rect.max.x - margin_x, baseline_y)],
            Stroke::new(1.0, Color32::from_rgb(80, 95, 120)),
        );

        // Draw intensity bars
        for (i, &p) in mode.spatial_intensity.iter().enumerate() {
            let x = rect.min.x + margin_x + (i as f32) * (plot_w / n as f32);
            let bar_h = (p / max_val) as f32 * plot_h;
            let top_y = baseline_y - bar_h;

            let bar_color = if mode.is_localized {
                Color32::from_rgb(255, 100, 100)
            } else if mode.is_extended {
                Color32::from_rgb(0, 230, 255)
            } else {
                Color32::from_rgb(255, 220, 60)
            };

            let bar_rect = Rect::from_min_max(
                pos2(x, top_y),
                pos2((x + bar_width - 1.0).max(x + 1.0), baseline_y),
            );
            painter.rect_filled(bar_rect, 1.0, bar_color);
        }

        // Center of mass marker
        let com_x = rect.min.x + margin_x + (mode.center_of_mass as f32) * (plot_w / n as f32);
        painter.line_segment(
            [pos2(com_x, rect.min.y + margin_y), pos2(com_x, baseline_y)],
            Stroke::new(1.5, Color32::from_rgb(255, 235, 120)),
        );
        painter.text(
            pos2(com_x, rect.min.y + margin_y - 6.0),
            egui::Align2::CENTER_BOTTOM,
            format!("COM: {:.1}", mode.center_of_mass),
            egui::FontId::monospace(10.0),
            Color32::from_rgb(255, 235, 120),
        );
    }

    /// Tab 2: Hofstadter butterfly fractal spectrum plot.
    fn render_butterfly_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Hofstadter Butterfly Energy Spectrum").strong());
        ui.label("Eigenenergy levels as a function of incommensurability ratio beta in [0, 1].");

        let mut pts = Vec::new();
        for b_pt in &self.butterfly_data {
            for &e in &b_pt.energies {
                pts.push([b_pt.beta, e]);
            }
        }

        let butterfly_points = Points::new("Eigenvalues", PlotPoints::new(pts))
            .color(Color32::from_rgb(0, 220, 255))
            .radius(1.5);

        Plot::new("hofstadter_butterfly_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label("Incommensurate Ratio beta (synthetic flux)")
            .y_axis_label("Energy (MHz)")
            .height(280.0)
            .show(ui, |plot_ui| {
                plot_ui.points(butterfly_points);
                // Current operating point beta
                plot_ui.vline(
                    VLine::new("Operating beta", self.incommensurate_beta)
                        .color(Color32::from_rgb(255, 180, 60)),
                );
            });
    }

    /// Tab 3: Energy vs IPR / NPR scatter plot showing mobility edge.
    fn render_mobility_edge_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Energy-Dependent Mobility Edge & IPR Distribution").strong());
        ui.label("Eigenstate IPR vs Energy. Delocalized states cluster at IPR ~ 0; localized states appear at IPR > 0.08.");

        let mut localized_pts = Vec::new();
        let mut extended_pts = Vec::new();
        let mut critical_pts = Vec::new();

        for s in &self.engine.eigenstates {
            if s.is_localized {
                localized_pts.push([s.energy, s.ipr]);
            } else if s.is_extended {
                extended_pts.push([s.energy, s.ipr]);
            } else {
                critical_pts.push([s.energy, s.ipr]);
            }
        }

        let loc_points = Points::new("Localized States", PlotPoints::new(localized_pts))
            .color(Color32::from_rgb(255, 100, 100))
            .radius(4.0);

        let ext_points = Points::new("Extended States", PlotPoints::new(extended_pts))
            .color(Color32::from_rgb(0, 230, 255))
            .radius(4.0);

        let crit_points = Points::new("Critical Multifractal", PlotPoints::new(critical_pts))
            .color(Color32::from_rgb(255, 220, 60))
            .radius(4.0);

        Plot::new("mobility_edge_scatter_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label("Energy (MHz)")
            .y_axis_label("Inverse Participation Ratio (IPR)")
            .height(280.0)
            .show(ui, |plot_ui| {
                plot_ui.points(ext_points);
                plot_ui.points(loc_points);
                plot_ui.points(crit_points);

                if let Some(ec) = self.engine.metrics.mobility_edge_energy_mhz {
                    plot_ui.vline(
                        VLine::new("Mobility Edge Ec", ec).color(Color32::from_rgb(255, 120, 255)),
                    );
                }
            });
    }

    /// Tab 4: Phason evolution plot showing topological boundary modes.
    fn render_phason_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Topological Phason Mode Energy Flow").strong());
        ui.label("Energy levels vs synthetic phason angle phi in [0, 2*pi]. In-gap branches represent boundary-localized chiral modes.");

        let mut pts = Vec::new();
        for (phi, evals) in &self.phason_data {
            for &e in evals {
                pts.push([*phi / PI, e]);
            }
        }

        let phason_points = Points::new("Phason Spectrum", PlotPoints::new(pts))
            .color(Color32::from_rgb(180, 210, 255))
            .radius(2.0);

        Plot::new("phason_evolution_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label("Phason Angle phi / pi")
            .y_axis_label("Energy (MHz)")
            .height(280.0)
            .show(ui, |plot_ui| {
                plot_ui.points(phason_points);
                plot_ui.vline(
                    VLine::new("Active phi", self.phason_phi / PI)
                        .color(Color32::from_rgb(255, 180, 60)),
                );
            });
    }

    /// Tab 5: Fractal scaling curve D2 vs Delta / J.
    fn render_fractal_scaling_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Multifractal Dimension D2 & Self-Dual Scaling").strong());
        ui.label("Scaling dimension D2 as potential ratio Delta/J is tuned through the self-dual critical point at Delta/J = 2.");

        let mut d2_pts = Vec::new();
        let mut ipr_pts = Vec::new();

        let ratios = [0.4, 0.8, 1.2, 1.6, 2.0, 2.4, 2.8, 3.2, 3.6, 4.0];
        for &r in &ratios {
            let p = AahParams {
                hopping_j: 5.0,
                modulation_delta: 5.0 * r,
                off_diagonal_lambda: 0.0,
                incommensurate_beta: GOLDEN_RATIO_CONJUGATE,
                phason_phi: 0.0,
                hopping_phase_theta: 0.0,
                model_kind: AahModelKind::StandardAah,
            };
            let eng = AahLatticeEngine::new(p, 40);
            d2_pts.push([r, eng.metrics.fractal_dimension_d2]);
            ipr_pts.push([r, eng.metrics.mean_ipr]);
        }

        let d2_line = Line::new("Fractal Dimension D2", PlotPoints::new(d2_pts))
            .color(Color32::from_rgb(0, 230, 255))
            .width(2.5);

        let ipr_line = Line::new("Mean IPR", PlotPoints::new(ipr_pts))
            .color(Color32::from_rgb(255, 120, 100))
            .width(2.0);

        Plot::new("aah_fractal_scaling_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label("Modulation Ratio Delta / J")
            .y_axis_label("Metric")
            .height(280.0)
            .show(ui, |plot_ui| {
                plot_ui.line(d2_line);
                plot_ui.line(ipr_line);
                plot_ui.vline(
                    VLine::new("Critical Point Delta/J = 2", 2.0)
                        .color(Color32::from_rgb(255, 235, 120)),
                );
            });
    }

    /// Telemetry footer.
    fn render_telemetry(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let phase = self.engine.params.delta_over_j();
            let phase_text = if self.engine.metrics.mobility_edge_detected {
                RichText::new("Mobility Edge Coexistence").color(Color32::from_rgb(255, 120, 255)).strong()
            } else if (phase - 2.0).abs() < 0.15 {
                RichText::new("Critical Multifractal").color(Color32::from_rgb(255, 220, 60)).strong()
            } else if phase < 2.0 {
                RichText::new("Delocalized Extended").color(Color32::from_rgb(0, 230, 255)).strong()
            } else {
                RichText::new("Exponentially Localized").color(Color32::from_rgb(255, 100, 100)).strong()
            };
            ui.label(phase_text);

            ui.separator();
            ui.label(format!("Delta/J: {:.2}", self.engine.params.delta_over_j()));

            ui.separator();
            ui.label(format!("Mean IPR: {:.4}", self.engine.metrics.mean_ipr));

            ui.separator();
            ui.label(format!("D2: {:.3}", self.engine.metrics.fractal_dimension_d2));

            if let Some(ec) = self.engine.metrics.mobility_edge_energy_mhz {
                ui.separator();
                ui.label(format!("Ec: {:.2} MHz", ec));
            }

            ui.separator();
            ui.label(format!("Loc: {:.0}% | Ext: {:.0}%", self.engine.metrics.fraction_localized_pct, self.engine.metrics.fraction_extended_pct));

            ui.separator();
            ui.label(format!("Edge Modes: {}", self.engine.metrics.boundary_edge_state_count));
        });
    }
}
