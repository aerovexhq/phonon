#![deny(unsafe_code)]

//! Interactive Directional Cosmic Heavy Ion Radiation Track & 3D Anisotropic Shielding Dialog.
//!
//! Provides a 5-tab CAD simulation environment:
//! 1. 3D Incident Ion Track & Chiplet Stack Canvas (renders oblique ray piercing 3D chiplet stack).
//! 2. Bragg Peak LET Depth Curve (plots LET(z) in MeV*cm^2/mg with die boundary markers).
//! 3. Spacecraft Anisotropic Shielding Polar Map (plots effective areal mass and attenuation over 360 deg).
//! 4. Oblique Multi-Die MBU & SEL Crossbar (displays struck dies, deposited charge, and SEL status).
//! 5. Mission Shielding Trade-Off & TID vs Thickness (plots 5-year TID vs thickness for Al vs Graded-Z).

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{Line, Plot, PlotPoints, VLine};
use phonon_solver::directional_radiation::{
    DirectionalRadiationCoSimulator, HeavyIonSpecies, IncidentTrajectory,
    MultiDieMbuEngine, RadiationTelemetryReport, ShieldingMaterial, SpacecraftShieldingModel,
};
use std::f64::consts::PI;

/// Active tab in the Directional Radiation Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionalRadiationTab {
    Track3dCanvas,
    BraggPeakCurve,
    AnisotropicPolarMap,
    MultiDieMbuMatrix,
    TidTradeoffPlot,
}

/// Modal dialog for Directional Heavy Ion Radiation & 3D Anisotropic Shielding Simulation.
pub struct DirectionalRadiationDialog {
    pub is_open: bool,
    pub active_tab: DirectionalRadiationTab,

    // Trajectory Controls
    pub ion_species: HeavyIonSpecies,
    pub theta_deg: f64,
    pub phi_deg: f64,
    pub energy_mev_per_nuc: f64,

    // Vehicle Shielding Controls
    pub hull_thickness_mm: f64,
    pub mission_duration_years: f64,
    pub primary_material: ShieldingMaterial,

    // Simulation Engine & Cached Results
    pub sim: DirectionalRadiationCoSimulator,
    pub cached_let_curve: Vec<[f64; 2]>,
    pub cached_polar_sweep: Vec<(f64, f64, f64)>,
    pub cached_al_tradeoff: Vec<[f64; 2]>,
    pub cached_gz_tradeoff: Vec<[f64; 2]>,
}

impl Default for DirectionalRadiationDialog {
    fn default() -> Self {
        let sim = DirectionalRadiationCoSimulator::new_fast();

        // Pre-seeded lightweight curves for instant rendering
        let cached_let_curve = vec![
            [0.0, 35.2],
            [20.0, 38.5],
            [40.0, 42.1],
            [60.0, 47.8],
            [80.0, 56.2],
            [100.0, 69.4],
            [120.0, 78.4], // Bragg Peak
            [125.0, 65.0],
            [130.0, 15.0],
            [135.0, 0.0],
        ];

        let cached_polar_sweep = vec![
            (0.0, 2.5, 32.0),
            (45.0, 3.2, 45.0),
            (90.0, 4.8, 62.0),
            (135.0, 7.5, 78.0),
            (180.0, 12.0, 88.0),
            (225.0, 8.2, 82.0),
            (270.0, 4.0, 55.0),
            (315.0, 2.8, 38.0),
            (360.0, 2.5, 32.0),
        ];

        let cached_al_tradeoff = vec![
            [0.0, 42.5],
            [2.5, 31.2],
            [5.0, 22.8],
            [7.5, 16.5],
            [10.0, 12.0],
            [15.0, 6.4],
            [20.0, 3.4],
        ];

        let cached_gz_tradeoff = vec![
            [0.0, 42.5],
            [2.5, 26.5],
            [5.0, 16.2],
            [7.5, 9.8],
            [10.0, 6.0],
            [15.0, 2.2],
            [20.0, 0.8],
        ];

        Self {
            is_open: false,
            active_tab: DirectionalRadiationTab::Track3dCanvas,
            ion_species: HeavyIonSpecies::IronFe56,
            theta_deg: 30.0,
            phi_deg: 45.0,
            energy_mev_per_nuc: 150.0,
            hull_thickness_mm: 3.5,
            mission_duration_years: 5.0,
            primary_material: ShieldingMaterial::Aluminum,
            sim,
            cached_let_curve,
            cached_polar_sweep,
            cached_al_tradeoff,
            cached_gz_tradeoff,
        }
    }
}

impl DirectionalRadiationDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast constructor for cold boot latency optimization (< 0.1ms).
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Recompute simulation engine with current dialog parameters.
    pub fn recompute(&mut self) {
        let traj = IncidentTrajectory {
            theta_rad: self.theta_deg * PI / 180.0,
            phi_rad: self.phi_deg * PI / 180.0,
            energy_mev_per_nuc: self.energy_mev_per_nuc,
            species: self.ion_species,
        };

        let mut shielding = SpacecraftShieldingModel::default();
        shielding.baseline_hull_thickness_mm = self.hull_thickness_mm;
        shielding.primary_material = self.primary_material;
        shielding.mission_duration_years = self.mission_duration_years;

        let mbu_engine = MultiDieMbuEngine::default();
        self.sim = DirectionalRadiationCoSimulator::new(traj.clone(), shielding.clone(), mbu_engine);

        // Update Bragg LET curve
        let mut let_pts = Vec::with_capacity(32);
        for s in 0..32 {
            let z = (s as f64) * 5.0; // 0 to 155 um depth
            let let_val = self.sim.track_profile.let_at_depth_um(&traj, z);
            let_pts.push([z, let_val]);
        }
        self.cached_let_curve = let_pts;

        // Update polar azimuth sweep
        self.cached_polar_sweep = shielding.compute_polar_azimuth_sweep(traj.theta_rad, 24);

        // Update tradeoff curves
        let (al, gz) = shielding.compute_tid_thickness_tradeoff(25.0, 20);
        self.cached_al_tradeoff = al;
        self.cached_gz_tradeoff = gz;
    }

    /// Main render method for the dialog window.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Directional Cosmic Heavy Ion Radiation Track & 3D Anisotropic Shielding Studio")
            .open(&mut is_open)
            .default_size(vec2(880.0, 700.0))
            .min_width(740.0)
            .min_height(560.0)
            .show(ctx, |ui| {
                self.render_content(ui);
            });

        self.is_open = is_open;
    }

    /// Internal content rendering for the dialog.
    pub fn render_content(&mut self, ui: &mut Ui) {
        self.render_header(ui);
        ui.separator();
        self.render_tab_bar(ui);
        ui.separator();

        match self.active_tab {
            DirectionalRadiationTab::Track3dCanvas => self.render_track_3d_canvas(ui),
            DirectionalRadiationTab::BraggPeakCurve => self.render_bragg_peak_plot(ui),
            DirectionalRadiationTab::AnisotropicPolarMap => self.render_polar_shielding_map(ui),
            DirectionalRadiationTab::MultiDieMbuMatrix => self.render_mbu_crossbar(ui),
            DirectionalRadiationTab::TidTradeoffPlot => self.render_tid_tradeoff_plot(ui),
        }

        ui.separator();
        self.render_controls_and_presets(ui);
        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Cosmic Heavy Ion Radiation Track & 3D Shielding Co-Simulator")
                    .color(Color32::from_rgb(255, 120, 80))
                    .strong(),
            );
            ui.label(
                RichText::new("Vector-Incident Track k_rad | Bragg Peak LET | 3D Anisotropic Ray-Tracing | Multi-Die MBU/SEL")
                    .weak(),
            );
        });
    }

    fn render_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    self.active_tab == DirectionalRadiationTab::Track3dCanvas,
                    "3D Track & Chiplet Canvas",
                )
                .clicked()
            {
                self.active_tab = DirectionalRadiationTab::Track3dCanvas;
            }
            if ui
                .selectable_label(
                    self.active_tab == DirectionalRadiationTab::BraggPeakCurve,
                    "Bragg Peak LET Curve",
                )
                .clicked()
            {
                self.active_tab = DirectionalRadiationTab::BraggPeakCurve;
            }
            if ui
                .selectable_label(
                    self.active_tab == DirectionalRadiationTab::AnisotropicPolarMap,
                    "Anisotropic Shielding Map",
                )
                .clicked()
            {
                self.active_tab = DirectionalRadiationTab::AnisotropicPolarMap;
            }
            if ui
                .selectable_label(
                    self.active_tab == DirectionalRadiationTab::MultiDieMbuMatrix,
                    "Multi-Die MBU Crossbar",
                )
                .clicked()
            {
                self.active_tab = DirectionalRadiationTab::MultiDieMbuMatrix;
            }
            if ui
                .selectable_label(
                    self.active_tab == DirectionalRadiationTab::TidTradeoffPlot,
                    "Mission TID Trade-Off",
                )
                .clicked()
            {
                self.active_tab = DirectionalRadiationTab::TidTradeoffPlot;
            }
        });
    }

    /// Tab 1: 3D Incident Ion Track & Chiplet Stack Canvas.
    fn render_track_3d_canvas(&self, ui: &mut Ui) {
        ui.label(RichText::new("3D Oblique Ion Track Ray-Tracing through Heterogeneous Multi-Die Chiplet Package:").italics());

        let (rect, _resp) = ui.allocate_exact_size(vec2(ui.available_width(), 320.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Dark background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 18, 25));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 75)), StrokeKind::Outside);

        let cx = rect.center().x;
        let cy = rect.center().y;

        // Draw package layers (isometric 3D perspective projection)
        // Layer 0: Active Silicon Interposer (Bottom)
        let interposer_rect = egui::Rect::from_center_size(pos2(cx, cy + 80.0), vec2(460.0, 40.0));
        painter.rect_filled(interposer_rect, 2.0, Color32::from_rgb(30, 45, 65));
        painter.rect_stroke(interposer_rect, 2.0, Stroke::new(1.5, Color32::from_rgb(60, 95, 140)), StrokeKind::Outside);
        painter.text(
            pos2(interposer_rect.left() + 10.0, interposer_rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Die 0: Active Silicon Interposer (z = 10 um)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(150, 190, 240),
        );

        // Layer 1: Host Compute SoC (Middle Left)
        let compute_rect = egui::Rect::from_min_size(pos2(cx - 200.0, cy - 10.0), vec2(180.0, 50.0));
        painter.rect_filled(compute_rect, 2.0, Color32::from_rgb(35, 55, 80));
        painter.rect_stroke(compute_rect, 2.0, Stroke::new(1.5, Color32::from_rgb(80, 130, 190)), StrokeKind::Outside);
        painter.text(
            pos2(compute_rect.center().x, compute_rect.center().y),
            egui::Align2::CENTER_CENTER,
            "Die 1: Compute SoC (z = 60 um)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(180, 220, 255),
        );

        // Layer 2: Stacked HBM/SRAM Memory (Top Right)
        let memory_rect = egui::Rect::from_min_size(pos2(cx + 20.0, cy - 80.0), vec2(180.0, 55.0));
        painter.rect_filled(memory_rect, 2.0, Color32::from_rgb(50, 40, 75));
        painter.rect_stroke(memory_rect, 2.0, Stroke::new(1.5, Color32::from_rgb(130, 90, 190)), StrokeKind::Outside);
        painter.text(
            pos2(memory_rect.center().x, memory_rect.center().y),
            egui::Align2::CENTER_CENTER,
            "Die 2: HBM3 Memory (z = 120 um)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(220, 180, 255),
        );

        // Directional oblique heavy ion track ray
        let theta_rad = self.theta_deg * PI / 180.0;
        let phi_rad = self.phi_deg * PI / 180.0;

        let ray_dx = theta_rad.sin() * phi_rad.cos() * 220.0;
        let ray_dy = theta_rad.cos() * 260.0;

        let ray_start = pos2(cx + 100.0 - (ray_dx as f32) * 0.5, rect.top() + 20.0);
        let ray_end = pos2(ray_start.x + (ray_dx as f32), ray_start.y + (ray_dy as f32));

        // Draw ionization halo around heavy ion track
        painter.line_segment([ray_start, ray_end], Stroke::new(10.0, Color32::from_rgba_unmultiplied(255, 80, 40, 50)));
        painter.line_segment([ray_start, ray_end], Stroke::new(4.0, Color32::from_rgba_unmultiplied(255, 160, 40, 160)));
        painter.line_segment([ray_start, ray_end], Stroke::new(1.5, Color32::from_rgb(255, 240, 200)));

        // Arrow head at ray end
        painter.circle_filled(ray_end, 4.0, Color32::from_rgb(255, 80, 40));

        // Highlight hit cell locations
        for hit in &self.sim.latest_hits {
            let hit_color = if hit.sel_triggered {
                Color32::from_rgb(255, 50, 50)
            } else {
                Color32::from_rgb(255, 200, 40)
            };

            let hit_y = match hit.die_id {
                0 => interposer_rect.center().y,
                1 => compute_rect.center().y,
                2 => memory_rect.center().y,
                _ => cy,
            };

            let hit_pos = pos2(cx + (hit.entry_point_um[0] as f32) * 0.04, hit_y);
            painter.circle_filled(hit_pos, 7.0, hit_color);
            painter.circle_stroke(hit_pos, 12.0, Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 220, 100, 120)));
            painter.text(
                pos2(hit_pos.x + 15.0, hit_pos.y),
                egui::Align2::LEFT_CENTER,
                format!("Hit: {} cells ({:.1} fC)", hit.upset_cell_count, hit.deposited_charge_fc),
                egui::FontId::proportional(11.0),
                hit_color,
            );
        }

        // Annotations
        painter.text(
            pos2(rect.left() + 15.0, rect.bottom() - 15.0),
            egui::Align2::LEFT_BOTTOM,
            format!("Ion Species: {} | Theta = {:.1} deg, Phi = {:.1} deg", self.ion_species.name(), self.theta_deg, self.phi_deg),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(180, 200, 230),
        );

        let sel_str = if self.sim.latest_telemetry.sel_triggered_any {
            "CRITICAL: Parasitic Thyristor SEL Triggered!"
        } else {
            "SEL Quenched (Safe Latchup Margin)"
        };
        painter.text(
            pos2(rect.right() - 15.0, rect.bottom() - 15.0),
            egui::Align2::RIGHT_BOTTOM,
            sel_str,
            egui::FontId::proportional(11.0),
            if self.sim.latest_telemetry.sel_triggered_any {
                Color32::from_rgb(255, 60, 60)
            } else {
                Color32::from_rgb(80, 240, 120)
            },
        );
    }

    /// Tab 2: Bragg Peak LET Depth Curve.
    fn render_bragg_peak_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Stopping Power Linear Energy Transfer LET(z) in Silicon vs Penetration Depth:").italics());

        let points: Vec<[f64; 2]> = self.cached_let_curve.clone();

        Plot::new("bragg_peak_plot")
            .height(300.0)
            .x_axis_label("Silicon Depth z (um)")
            .y_axis_label("Linear Energy Transfer LET (MeV*cm^2/mg)")
            .show(ui, |plot_ui| {
                // Die boundaries: Interposer (z=10), Compute (z=60), Memory (z=120)
                plot_ui.vline(VLine::new("Die 0: Interposer", 10.0).color(Color32::from_rgb(100, 160, 240)));
                plot_ui.vline(VLine::new("Die 1: Compute SoC", 60.0).color(Color32::from_rgb(80, 200, 160)));
                plot_ui.vline(VLine::new("Die 2: HBM3 Memory", 120.0).color(Color32::from_rgb(220, 140, 255)));

                plot_ui.line(
                    Line::new("Bragg LET Curve", PlotPoints::new(points))
                        .color(Color32::from_rgb(255, 120, 40))
                        .width(2.5),
                );
            });
    }

    /// Tab 3: Spacecraft Anisotropic Shielding Polar Map.
    fn render_polar_shielding_map(&self, ui: &mut Ui) {
        ui.label(RichText::new("360-Degree Azimuthal Shielding Areal Mass (g/cm^2) & Directional Attenuation %:").italics());

        let sweep_mass: Vec<[f64; 2]> = self
            .cached_polar_sweep
            .iter()
            .map(|&(phi, mass, _)| [phi, mass])
            .collect();

        let sweep_atten: Vec<[f64; 2]> = self
            .cached_polar_sweep
            .iter()
            .map(|&(phi, _, atten)| [phi, atten])
            .collect();

        Plot::new("polar_shielding_plot")
            .height(300.0)
            .x_axis_label("Azimuth Angle Phi (deg)")
            .y_axis_label("Areal Mass (g/cm^2) / Attenuation (%)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Areal Mass (g/cm^2)", PlotPoints::new(sweep_mass))
                        .color(Color32::from_rgb(100, 180, 255))
                        .width(2.0),
                );

                plot_ui.line(
                    Line::new("Flux Attenuation (%)", PlotPoints::new(sweep_atten))
                        .color(Color32::from_rgb(255, 190, 40))
                        .width(2.0),
                );
            });
    }

    /// Tab 4: Oblique Multi-Die MBU & SEL Crossbar.
    fn render_mbu_crossbar(&self, ui: &mut Ui) {
        ui.label(RichText::new("Multi-Die Piercing Analysis: Struck Chiplet Layers, Deposited Charge & Latchup Status:").italics());

        egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
            egui::Grid::new("mbu_crossbar_grid")
                .striped(true)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Die Layer").strong());
                    ui.label(RichText::new("Track Length").strong());
                    ui.label(RichText::new("Deposited Charge").strong());
                    ui.label(RichText::new("MBU Upset Cells").strong());
                    ui.label(RichText::new("SET Duration").strong());
                    ui.label(RichText::new("SEL Risk").strong());
                    ui.end_row();

                    for hit in &self.sim.latest_hits {
                        ui.label(&hit.die_name);
                        ui.label(format!("{:.1} um", hit.track_length_um));
                        ui.label(format!("{:.1} fC", hit.deposited_charge_fc));
                        ui.label(format!("{} cells", hit.upset_cell_count));
                        ui.label(format!("{:.1} ps", hit.set_pulse_duration_ps));

                        if hit.sel_triggered {
                            ui.label(RichText::new("LATCHUP TRIGGERED").color(Color32::from_rgb(255, 60, 60)).strong());
                        } else {
                            ui.label(RichText::new("Normal / Safe").color(Color32::from_rgb(80, 240, 120)));
                        }
                        ui.end_row();
                    }

                    if self.sim.latest_hits.is_empty() {
                        ui.label(RichText::new("No active die hits in current trajectory window.").italics());
                        ui.end_row();
                    }
                });
        });
    }

    /// Tab 5: Mission Shielding Trade-Off & TID vs Thickness.
    fn render_tid_tradeoff_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("5-Year Mission Total Ionizing Dose (TID in krad(Si)) vs Shielding Thickness:").italics());

        let al_points: Vec<[f64; 2]> = self.cached_al_tradeoff.clone();
        let gz_points: Vec<[f64; 2]> = self.cached_gz_tradeoff.clone();

        Plot::new("tid_tradeoff_plot")
            .height(300.0)
            .x_axis_label("Shielding Thickness (mm)")
            .y_axis_label("5-Year Mission TID (krad(Si))")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Aluminum (Al 6061-T6)", PlotPoints::new(al_points))
                        .color(Color32::from_rgb(100, 180, 255))
                        .width(2.0),
                );

                plot_ui.line(
                    Line::new("Graded-Z Composite", PlotPoints::new(gz_points))
                        .color(Color32::from_rgb(255, 120, 80))
                        .width(2.5),
                );
            });
    }

    fn render_controls_and_presets(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Radiation Controls:").strong());

            ui.label("Ion Species:");
            let mut changed = false;
            egui::ComboBox::from_id_salt("ion_species_combo")
                .selected_text(self.ion_species.name())
                .show_ui(ui, |ui| {
                    changed |= ui.selectable_value(&mut self.ion_species, HeavyIonSpecies::Proton, "Proton (1H)").changed();
                    changed |= ui.selectable_value(&mut self.ion_species, HeavyIonSpecies::Alpha, "Alpha (4He)").changed();
                    changed |= ui.selectable_value(&mut self.ion_species, HeavyIonSpecies::Carbon, "Carbon (12C)").changed();
                    changed |= ui.selectable_value(&mut self.ion_species, HeavyIonSpecies::Silicon, "Silicon (28Si)").changed();
                    changed |= ui.selectable_value(&mut self.ion_species, HeavyIonSpecies::IronFe56, "Iron (56Fe)").changed();
                });

            ui.label("Zenith Theta:");
            changed |= ui
                .add(egui::Slider::new(&mut self.theta_deg, 0.0..=80.0).suffix(" deg"))
                .changed();

            ui.label("Azimuth Phi:");
            changed |= ui
                .add(egui::Slider::new(&mut self.phi_deg, 0.0..=360.0).suffix(" deg"))
                .changed();

            ui.label("Energy:");
            changed |= ui
                .add(egui::Slider::new(&mut self.energy_mev_per_nuc, 10.0..=1000.0).suffix(" MeV/nuc"))
                .changed();

            if changed {
                self.recompute();
            }
        });

        ui.add_space(3.0);
        ui.horizontal(|ui| {
            let mut changed = false;
            ui.label("Hull Thickness:");
            changed |= ui
                .add(egui::Slider::new(&mut self.hull_thickness_mm, 1.0..=25.0).suffix(" mm Al"))
                .changed();

            ui.label("Mission Duration:");
            changed |= ui
                .add(egui::Slider::new(&mut self.mission_duration_years, 1.0..=15.0).suffix(" yrs"))
                .changed();

            ui.separator();
            ui.label(RichText::new("Presets:").strong());

            if ui.button("Solar Proton Event (SPE)").clicked() {
                self.ion_species = HeavyIonSpecies::Proton;
                self.energy_mev_per_nuc = 80.0;
                self.theta_deg = 15.0;
                self.phi_deg = 0.0;
                self.recompute();
            }

            if ui.button("GCR Heavy Ion (Fe-56 Strike)").clicked() {
                self.ion_species = HeavyIonSpecies::IronFe56;
                self.energy_mev_per_nuc = 150.0;
                self.theta_deg = 35.0;
                self.phi_deg = 45.0;
                self.recompute();
            }

            if ui.button("LEO Van Allen Proton").clicked() {
                self.ion_species = HeavyIonSpecies::Proton;
                self.energy_mev_per_nuc = 120.0;
                self.theta_deg = 45.0;
                self.phi_deg = 90.0;
                self.recompute();
            }

            if ui.button("High-Z Spot Shielded Die").clicked() {
                self.ion_species = HeavyIonSpecies::IronFe56;
                self.hull_thickness_mm = 8.0;
                self.primary_material = ShieldingMaterial::Tungsten;
                self.recompute();
            }

            if changed {
                self.recompute();
            }
        });
    }

    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let t: &RadiationTelemetryReport = &self.sim.latest_telemetry;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Telemetry:").strong());

            ui.label(format!("Peak LET: {:.1} MeV*cm^2/mg", t.peak_let_mev_cm2_mg));
            ui.separator();

            ui.label(format!("Max Q_dep: {:.1} fC", t.max_deposited_charge_fc));
            ui.separator();

            ui.label(format!("MBU Cells: {}", t.total_mbu_flipped_cells));
            ui.separator();

            ui.label(format!("Pierced Dies: {}", t.pierced_die_count));
            ui.separator();

            let sel_text = if t.sel_triggered_any { "TRIGGERED" } else { "None" };
            ui.label(format!("SEL Risk: {}", sel_text));
            ui.separator();

            ui.label(format!("Mission TID: {:.1} krad", t.localized_mission_tid_krad));
            ui.separator();

            ui.label(format!("Shielding Attenuation: {:.1}%", t.shielding_attenuation_percent));
        });
    }
}
