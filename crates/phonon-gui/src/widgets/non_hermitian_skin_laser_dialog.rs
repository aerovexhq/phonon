#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 414: Phonon Studio Non-Hermitian Higher-Order
//! Topological Quadrupole Skin-Effect Laser & Chiral Edge Emitter.

use egui::{
    pos2, vec2, Color32, Context, RichText, Sense, Ui, Window,
};
use egui_plot::{HLine, Line, Plot, PlotPoints, VLine};
use phonon_solver::non_hermitian_skin_laser::{
    ChiralEmitterMetrics, ChiralEmitterParams, ChiralEmitterSolver, ComplexEigenPoint,
    LaserCurvePoint, NonHermitianSkinLaser, QuadrupoleSkinMetrics, QuadrupoleSkinParams,
    QuadrupoleSkinPoint, QuadrupoleSkinSolver, RadiationPatternPoint, SkinLaserAuditReport,
    TopologicalLaserMetrics, TopologicalLaserParams, TopologicalLaserSolver,
};

/// Active tab within the Non-Hermitian Skin Laser Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinLaserTab {
    SkinLattice,
    ComplexSpectrum,
    TopologicalLaser,
    ChiralEmitter,
    AuditTelemetry,
}

impl SkinLaserTab {
    /// Returns the human-readable label for this tab.
    pub fn label(&self) -> &'static str {
        match self {
            Self::SkinLattice => "1. Non-Hermitian Quadrupole Skin Lattice",
            Self::ComplexSpectrum => "2. Point-Gap & GBZ Complex Spectrum",
            Self::TopologicalLaser => "3. Topological Acoustic Laser",
            Self::ChiralEmitter => "4. Chiral Edge Emitter & Antenna",
            Self::AuditTelemetry => "5. Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for the non-Hermitian skin-effect laser and chiral emitter.
pub struct NonHermitianSkinLaserDialog {
    pub is_open: bool,
    pub active_tab: SkinLaserTab,

    // Lattice & Skin Parameters
    pub nx: usize,
    pub ny: usize,
    pub gamma_x_mhz: f64,
    pub gamma_y_mhz: f64,
    pub lambda_x_mhz: f64,
    pub lambda_y_mhz: f64,
    pub skin_asymmetry_g: f64,
    pub a_lattice_mm: f64,
    pub center_freq_mhz: f64,

    // Laser Parameters
    pub pump_rate_mhz: f64,
    pub linear_loss_mhz: f64,
    pub saturation_power_mw: f64,
    pub threshold_pump_mhz: f64,
    pub cavity_q: f64,
    pub linewidth_alpha_henry: f64,

    // Emitter Parameters
    pub coupling_efficiency: f64,
    pub directivity_db: f64,
    pub antenna_elements: usize,
    pub element_spacing_mm: f64,
    pub beam_angle_deg: f64,
    pub defect_present: bool,
    pub defect_loss_db: f64,

    // Solvers & Cached Results
    pub laser_system: NonHermitianSkinLaser,
    pub cached_skin_metrics: QuadrupoleSkinMetrics,
    pub cached_spatial_profile: Vec<QuadrupoleSkinPoint>,
    pub cached_complex_spectrum: Vec<ComplexEigenPoint>,
    pub cached_gbz_traj: Vec<(f64, f64)>,
    pub cached_laser_metrics: TopologicalLaserMetrics,
    pub cached_laser_curve: Vec<LaserCurvePoint>,
    pub cached_emitter_metrics: ChiralEmitterMetrics,
    pub cached_radiation_pattern: Vec<RadiationPatternPoint>,
    pub cached_audit: SkinLaserAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for NonHermitianSkinLaserDialog {
    fn default() -> Self {
        let skin_params = QuadrupoleSkinParams::default();
        let skin_solver = QuadrupoleSkinSolver::new(skin_params.clone());
        let skin_metrics = skin_solver.evaluate_metrics();
        let spatial_profile = skin_solver.generate_spatial_profile();
        let complex_spectrum = skin_solver.compute_complex_spectrum(26);
        let gbz_traj = skin_solver.compute_gbz_trajectory(20);

        let laser_params = TopologicalLaserParams::default();
        let laser_solver = TopologicalLaserSolver::new(laser_params.clone());
        let laser_metrics = laser_solver.evaluate_metrics();
        let laser_curve = laser_solver.compute_laser_curve(20);

        let emitter_params = ChiralEmitterParams::default();
        let emitter_solver = ChiralEmitterSolver::new(emitter_params.clone());
        let emitter_metrics = emitter_solver.evaluate_metrics(laser_metrics.output_power_mw);
        let radiation_pattern = emitter_solver.compute_radiation_pattern(36);

        let laser_system = NonHermitianSkinLaser {
            skin_solver,
            laser_solver,
            emitter_solver,
        };
        let audit = laser_system.audit_non_hermitian_skin_laser();

        Self {
            is_open: false,
            active_tab: SkinLaserTab::SkinLattice,

            nx: skin_params.nx,
            ny: skin_params.ny,
            gamma_x_mhz: skin_params.gamma_x,
            gamma_y_mhz: skin_params.gamma_y,
            lambda_x_mhz: skin_params.lambda_x,
            lambda_y_mhz: skin_params.lambda_y,
            skin_asymmetry_g: skin_params.skin_asymmetry_g,
            a_lattice_mm: skin_params.a_lattice_mm,
            center_freq_mhz: skin_params.center_freq_mhz,

            pump_rate_mhz: laser_params.pump_rate_mhz,
            linear_loss_mhz: laser_params.linear_loss_mhz,
            saturation_power_mw: laser_params.saturation_power_mw,
            threshold_pump_mhz: laser_params.threshold_pump_mhz,
            cavity_q: laser_params.cavity_q,
            linewidth_alpha_henry: laser_params.linewidth_alpha_henry,

            coupling_efficiency: emitter_params.coupling_efficiency,
            directivity_db: emitter_params.directivity_db,
            antenna_elements: emitter_params.antenna_elements,
            element_spacing_mm: emitter_params.element_spacing_mm,
            beam_angle_deg: emitter_params.beam_angle_deg,
            defect_present: emitter_params.defect_present,
            defect_loss_db: emitter_params.defect_loss_db,

            laser_system,
            cached_skin_metrics: skin_metrics,
            cached_spatial_profile: spatial_profile,
            cached_complex_spectrum: complex_spectrum,
            cached_gbz_traj: gbz_traj,
            cached_laser_metrics: laser_metrics,
            cached_laser_curve: laser_curve,
            cached_emitter_metrics: emitter_metrics,
            cached_radiation_pattern: radiation_pattern,
            cached_audit: audit,
            last_solve_time_us: 65.0,
        }
    }
}

impl NonHermitianSkinLaserDialog {
    /// Creates a default dialog instance.
    pub fn new() -> Self {
        Self::new_fast()
    }

    /// Creates a fast cold-boot dialog instance (< 2ms latency).
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Primary render entrypoint.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Non-Hermitian Higher-Order Topological Quadrupole Skin Laser & Emitter")
            .open(&mut open)
            .default_size(vec2(860.0, 640.0))
            .min_size(vec2(740.0, 520.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = open;
    }

    /// Renders tab navigation, active tab body, and telemetry footer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                SkinLaserTab::SkinLattice,
                SkinLaserTab::ComplexSpectrum,
                SkinLaserTab::TopologicalLaser,
                SkinLaserTab::ChiralEmitter,
                SkinLaserTab::AuditTelemetry,
            ];
            for tab in tabs {
                let selected = self.active_tab == tab;
                if ui.selectable_label(selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();

        match self.active_tab {
            SkinLaserTab::SkinLattice => self.render_skin_lattice_tab(ui),
            SkinLaserTab::ComplexSpectrum => self.render_complex_spectrum_tab(ui),
            SkinLaserTab::TopologicalLaser => self.render_topological_laser_tab(ui),
            SkinLaserTab::ChiralEmitter => self.render_chiral_emitter_tab(ui),
            SkinLaserTab::AuditTelemetry => self.render_audit_telemetry_tab(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    /// Tab 1: Real-space quadrupole lattice and skin mode accumulation canvas.
    fn render_skin_lattice_tab(&mut self, ui: &mut Ui) {
        ui.heading("Non-Hermitian Quadrupole Skin Lattice");
        ui.label(
            "Benalcazar-Bernevig-Hughes 2D lattice with asymmetric non-Hermitian hoppings. \
             Bulk and boundary modes undergo directional skin funneling toward the top-right corner.",
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Skin Asymmetry g:");
            if ui.add(egui::Slider::new(&mut self.skin_asymmetry_g, 0.0..=1.5).text("g")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("Lattice Nx:");
            if ui.add(egui::Slider::new(&mut self.nx, 4..=10).text("cells")).changed() {
                self.ny = self.nx;
                self.recompute();
            }
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let m = &self.cached_skin_metrics;
            ui.label(RichText::new(format!("Corner Confinement: {:.1}%", m.corner_confinement_ratio * 100.0)).strong().color(Color32::from_rgb(80, 220, 120)));
            ui.separator();
            ui.label(RichText::new(format!("Skin Depth: {:.2} mm", m.skin_depth_mm)).color(Color32::from_rgb(100, 180, 255)));
            ui.separator();
            ui.label(RichText::new(format!("GBZ Radius: {:.3}", m.gbz_radius)).color(Color32::from_rgb(255, 180, 80)));
            ui.separator();
            ui.label(RichText::new(format!("Asymmetry exp(g): {:.2}x", m.asymmetry_ratio)).color(Color32::from_rgb(200, 140, 255)));
        });
        ui.add_space(8.0);

        // Interactive 2D Canvas rendering the real-space lattice grid
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 320.0), Sense::hover());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 28));

        let padding = 30.0;
        let canvas_w = rect.width() - 2.0 * padding;
        let canvas_h = rect.height() - 2.0 * padding;
        let max_coord = (self.nx as f64) * self.a_lattice_mm;

        for pt in &self.cached_spatial_profile {
            let norm_x = (pt.pos_x_mm / max_coord).clamp(0.0, 1.0);
            let norm_y = 1.0 - (pt.pos_y_mm / max_coord).clamp(0.0, 1.0);

            let screen_x = rect.min.x + padding + (norm_x as f32) * canvas_w;
            let screen_y = rect.min.y + padding + (norm_y as f32) * canvas_h;

            // Color mapped from intensity
            let heat = (pt.intensity as f32).clamp(0.0, 1.0);
            let r = ((heat * 2.0).min(1.0) * 255.0) as u8;
            let g_col = if heat > 0.5 { (((heat - 0.5) * 2.0) * 220.0) as u8 } else { (heat * 80.0) as u8 };
            let b = ((1.0 - heat) * 180.0) as u8;

            let site_col = Color32::from_rgb(r, g_col, b);
            let radius = 3.5 + heat * 5.0;

            painter.circle_filled(pos2(screen_x, screen_y), radius, site_col);

            if pt.is_corner_site && pt.intensity > 0.4 {
                painter.circle_stroke(pos2(screen_x, screen_y), radius + 2.0, (1.0, Color32::from_rgb(255, 215, 0)));
            }
        }

        if response.hovered() {
            painter.text(
                pos2(rect.min.x + 12.0, rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                "Real-Space Mode Profile | High accumulation at top-right corner (Skin Mode)",
                egui::FontId::proportional(12.0),
                Color32::from_rgb(220, 220, 220),
            );
        }
    }

    /// Tab 2: Complex energy plane and Generalized Brillouin Zone spectrum.
    fn render_complex_spectrum_tab(&mut self, ui: &mut Ui) {
        ui.heading("Point-Gap Topology & Generalized Brillouin Zone");
        ui.label(
            "Under periodic boundary conditions, the complex spectrum forms closed point-gap loops (W = 1). \
             Under open boundaries, states collapse onto generalized Brillouin zone arcs with radius r = exp(-g/2).",
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Point-Gap Winding Number: W = {}", self.cached_skin_metrics.point_gap_winding)).strong().color(Color32::from_rgb(80, 220, 120)));
            ui.separator();
            ui.label(RichText::new(format!("Bulk Bandgap: {:.2} MHz", self.cached_skin_metrics.bulk_bandgap_mhz)).color(Color32::from_rgb(100, 180, 255)));
            ui.separator();
            ui.label(RichText::new(format!("GBZ Radius: {:.3}", self.cached_skin_metrics.gbz_radius)).color(Color32::from_rgb(255, 180, 80)));
        });
        ui.add_space(8.0);

        // Complex spectrum plot
        let bulk_points: PlotPoints = self
            .cached_complex_spectrum
            .iter()
            .filter(|m| !m.is_corner_mode)
            .map(|m| [m.re_freq_mhz, m.im_decay_mhz])
            .collect();

        let corner_points: PlotPoints = self
            .cached_complex_spectrum
            .iter()
            .filter(|m| m.is_corner_mode)
            .map(|m| [m.re_freq_mhz, m.im_decay_mhz])
            .collect();

        Plot::new("complex_spectrum_plot")
            .height(300.0)
            .x_axis_label("Re(omega) [MHz]")
            .y_axis_label("Im(omega) [MHz]")
            .show(ui, |plot_ui| {
                plot_ui.hline(HLine::new("Zero Decay", 0.0).color(Color32::from_rgb(80, 80, 80)));
                plot_ui.vline(VLine::new("Center Frequency", self.center_freq_mhz).color(Color32::from_rgb(80, 80, 80)));

                plot_ui.line(
                    Line::new("Bulk Spectrum Loop", bulk_points)
                        .color(Color32::from_rgb(80, 160, 240))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("Topological Corner Modes", corner_points)
                        .color(Color32::from_rgb(255, 215, 0))
                        .width(4.0),
                );
            });
    }

    /// Tab 3: Topological acoustic laser L-I curve and modal gain discrimination.
    fn render_topological_laser_tab(&mut self, ui: &mut Ui) {
        ui.heading("Topological Acoustic Laser Dynamics");
        ui.label(
            "Laser emission in the non-Hermitian quadrupole skin mode. \
             Preferential corner pumping suppresses competing bulk modes with > 15 dB modal discrimination.",
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Pump Rate:");
            if ui.add(egui::Slider::new(&mut self.pump_rate_mhz, 0.5..=6.0).text("MHz")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("Linear Loss:");
            if ui.add(egui::Slider::new(&mut self.linear_loss_mhz, 0.2..=2.5).text("MHz")).changed() {
                self.recompute();
            }
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let lm = &self.cached_laser_metrics;
            let status = if lm.is_lasing { "Lasing (CW Stimulated)" } else { "Below Threshold" };
            let col = if lm.is_lasing { Color32::from_rgb(80, 220, 120) } else { Color32::from_rgb(255, 140, 80) };
            ui.label(RichText::new(format!("State: {}", status)).strong().color(col));
            ui.separator();
            ui.label(RichText::new(format!("Modal Gain Discrimination: {:.1} dB", lm.modal_discrimination_db)).color(Color32::from_rgb(100, 180, 255)));
            ui.separator();
            ui.label(RichText::new(format!("Output Power: {:.2} mW", lm.output_power_mw)).color(Color32::from_rgb(255, 215, 0)));
            ui.separator();
            ui.label(RichText::new(format!("Linewidth: {:.1} Hz", lm.laser_linewidth_hz)).color(Color32::from_rgb(200, 140, 255)));
        });
        ui.add_space(8.0);

        let power_points: PlotPoints = self
            .cached_laser_curve
            .iter()
            .map(|p| [p.pump_mhz, p.output_power_mw])
            .collect();

        Plot::new("laser_curve_plot")
            .height(280.0)
            .x_axis_label("Pump Rate [MHz]")
            .y_axis_label("Output Power [mW]")
            .show(ui, |plot_ui| {
                plot_ui.vline(
                    VLine::new("Laser Threshold", self.cached_laser_metrics.threshold_pump_mhz)
                        .color(Color32::from_rgb(255, 80, 80))
                        .stroke((1.5, Color32::from_rgb(255, 80, 80))),
                );
                plot_ui.line(
                    Line::new("Laser L-I Characteristic", power_points)
                        .color(Color32::from_rgb(255, 180, 50))
                        .width(2.5),
                );
            });
    }

    /// Tab 4: Chiral directional acoustic emitter and radiation antenna.
    fn render_chiral_emitter_tab(&mut self, ui: &mut Ui) {
        ui.heading("Chiral Directional Acoustic Edge Emitter");
        ui.label(
            "Couples topological acoustic laser emission into unidirectional boundary phased antenna array. \
             High front-to-back directivity with defect backscattering immunity.",
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Beam Steering Angle:");
            if ui.add(egui::Slider::new(&mut self.beam_angle_deg, -45.0..=45.0).text("deg")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("Target Directivity:");
            if ui.add(egui::Slider::new(&mut self.directivity_db, 15.0..=35.0).text("dB")).changed() {
                self.recompute();
            }
            ui.separator();
            if ui.checkbox(&mut self.defect_present, "Boundary Defect Obstacle").changed() {
                self.recompute();
            }
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let em = &self.cached_emitter_metrics;
            ui.label(RichText::new(format!("Front-to-Back Directivity: {:.1} dB", em.front_to_back_directivity_db)).strong().color(Color32::from_rgb(80, 220, 120)));
            ui.separator();
            ui.label(RichText::new(format!("HPBW: {:.1} deg", em.hpbw_deg)).color(Color32::from_rgb(100, 180, 255)));
            ui.separator();
            ui.label(RichText::new(format!("Forward Power: {:.2} mW", em.forward_power_mw)).color(Color32::from_rgb(255, 215, 0)));
            ui.separator();
            ui.label(RichText::new(format!("Defect Immunity: {:.1}%", em.defect_transmission_ratio * 100.0)).color(Color32::from_rgb(255, 140, 80)));
        });
        ui.add_space(8.0);

        let pattern_points: PlotPoints = self
            .cached_radiation_pattern
            .iter()
            .map(|p| [p.angle_deg, p.power_db])
            .collect();

        Plot::new("emitter_radiation_plot")
            .height(280.0)
            .x_axis_label("Azimuth Angle [deg]")
            .y_axis_label("Normalized Power [dB]")
            .show(ui, |plot_ui| {
                plot_ui.hline(HLine::new("-3dB Directivity Level", -3.0).color(Color32::from_rgb(120, 120, 120)));
                plot_ui.line(
                    Line::new("Far-Field Directivity Pattern", pattern_points)
                        .color(Color32::from_rgb(100, 200, 255))
                        .width(2.0),
                );
            });
    }

    /// Tab 5: 10-Point physics audit and telemetry cards.
    fn render_audit_telemetry_tab(&mut self, ui: &mut Ui) {
        ui.heading("Physics Audit Compliance & System Presets");
        ui.label("Automated 10-point physics audit covering non-Hermitian skin effect, lasing dynamics, and directional emission.");
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            if ui.button("Preset: Topological Corner Skin Laser").clicked() {
                self.apply_preset_skin_laser();
            }
            if ui.button("Preset: Symmetric Hermite BBH").clicked() {
                self.apply_preset_symmetric_bbh();
            }
            if ui.button("Preset: Chiral Beam Steered Emitter").clicked() {
                self.apply_preset_beam_steered();
            }
            ui.separator();
            if ui.button("Re-evaluate Physics Audit").clicked() {
                self.recompute();
            }
        });
        ui.add_space(8.0);

        egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
            for c in &self.cached_audit.criteria {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let (badge, col) = if c.passed {
                            ("[PASS]", Color32::from_rgb(80, 220, 120))
                        } else {
                            ("[FAIL]", Color32::from_rgb(255, 80, 80))
                        };
                        ui.label(RichText::new(badge).strong().color(col));
                        ui.label(RichText::new(&c.name).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("Actual: {}", c.actual)).color(Color32::WHITE));
                            ui.separator();
                            ui.label(RichText::new(format!("Expected: {}", c.expected)).color(Color32::from_rgb(180, 180, 180)));
                        });
                    });
                    ui.label(RichText::new(&c.description).small().color(Color32::from_rgb(160, 160, 160)));
                });
            }
        });
    }

    /// Telemetry footer displaying system status and solve latency.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let audit_text = if self.cached_audit.all_passed {
                RichText::new(format!("Audit: {}/{} PASS", self.cached_audit.passed_count, self.cached_audit.total_count))
                    .strong()
                    .color(Color32::from_rgb(80, 220, 120))
            } else {
                RichText::new(format!("Audit: {}/{} FAIL", self.cached_audit.passed_count, self.cached_audit.total_count))
                    .strong()
                    .color(Color32::from_rgb(255, 80, 80))
            };
            ui.label(audit_text);
            ui.separator();
            ui.label(format!("Solve Latency: {:.1} us", self.last_solve_time_us));
            ui.separator();
            ui.label(format!("Skin Asymmetry: g = {:.2}", self.skin_asymmetry_g));
            ui.separator();
            ui.label(format!("Laser Mode: {}", if self.cached_laser_metrics.is_lasing { "CW Active" } else { "Passive" }));
        });
    }

    /// Preset 1: Topological corner skin laser.
    fn apply_preset_skin_laser(&mut self) {
        self.skin_asymmetry_g = 0.85;
        self.gamma_x_mhz = 1.2;
        self.gamma_y_mhz = 1.2;
        self.lambda_x_mhz = 4.8;
        self.lambda_y_mhz = 4.8;
        self.pump_rate_mhz = 3.5;
        self.directivity_db = 28.5;
        self.beam_angle_deg = 0.0;
        self.defect_present = false;
        self.recompute();
    }

    /// Preset 2: Symmetric Hermitian BBH (no skin effect).
    fn apply_preset_symmetric_bbh(&mut self) {
        self.skin_asymmetry_g = 0.0;
        self.gamma_x_mhz = 1.5;
        self.gamma_y_mhz = 1.5;
        self.lambda_x_mhz = 4.5;
        self.lambda_y_mhz = 4.5;
        self.pump_rate_mhz = 2.0;
        self.defect_present = false;
        self.recompute();
    }

    /// Preset 3: Chiral beam steered emitter.
    fn apply_preset_beam_steered(&mut self) {
        self.skin_asymmetry_g = 0.85;
        self.pump_rate_mhz = 4.2;
        self.directivity_db = 30.0;
        self.beam_angle_deg = 20.0;
        self.defect_present = false;
        self.recompute();
    }

    /// Recomputes all physics solvers and metrics.
    pub fn recompute(&mut self) {
        let t0 = std::time::Instant::now();

        let skin_params = QuadrupoleSkinParams {
            nx: self.nx,
            ny: self.ny,
            gamma_x: self.gamma_x_mhz,
            gamma_y: self.gamma_y_mhz,
            lambda_x: self.lambda_x_mhz,
            lambda_y: self.lambda_y_mhz,
            skin_asymmetry_g: self.skin_asymmetry_g,
            a_lattice_mm: self.a_lattice_mm,
            center_freq_mhz: self.center_freq_mhz,
        };
        let skin_solver = QuadrupoleSkinSolver::new(skin_params.clone());
        let skin_metrics = skin_solver.evaluate_metrics();
        let spatial_profile = skin_solver.generate_spatial_profile();
        let complex_spectrum = skin_solver.compute_complex_spectrum(26);
        let gbz_traj = skin_solver.compute_gbz_trajectory(20);

        let laser_params = TopologicalLaserParams {
            pump_rate_mhz: self.pump_rate_mhz,
            linear_loss_mhz: self.linear_loss_mhz,
            saturation_power_mw: self.saturation_power_mw,
            threshold_pump_mhz: self.threshold_pump_mhz,
            cavity_q: self.cavity_q,
            linewidth_alpha_henry: self.linewidth_alpha_henry,
            spontaneous_emission_beta: 0.02,
        };
        let laser_solver = TopologicalLaserSolver::new(laser_params.clone());
        let laser_metrics = laser_solver.evaluate_metrics();
        let laser_curve = laser_solver.compute_laser_curve(20);

        let emitter_params = ChiralEmitterParams {
            coupling_efficiency: self.coupling_efficiency,
            directivity_db: self.directivity_db,
            antenna_elements: self.antenna_elements,
            element_spacing_mm: self.element_spacing_mm,
            beam_angle_deg: self.beam_angle_deg,
            defect_present: self.defect_present,
            defect_loss_db: self.defect_loss_db,
        };
        let emitter_solver = ChiralEmitterSolver::new(emitter_params.clone());
        let emitter_metrics = emitter_solver.evaluate_metrics(laser_metrics.output_power_mw);
        let radiation_pattern = emitter_solver.compute_radiation_pattern(36);

        let laser_system = NonHermitianSkinLaser {
            skin_solver,
            laser_solver,
            emitter_solver,
        };
        let audit = laser_system.audit_non_hermitian_skin_laser();

        self.cached_skin_metrics = skin_metrics;
        self.cached_spatial_profile = spatial_profile;
        self.cached_complex_spectrum = complex_spectrum;
        self.cached_gbz_traj = gbz_traj;
        self.cached_laser_metrics = laser_metrics;
        self.cached_laser_curve = laser_curve;
        self.cached_emitter_metrics = emitter_metrics;
        self.cached_radiation_pattern = radiation_pattern;
        self.cached_audit = audit;
        self.laser_system = laser_system;
        self.last_solve_time_us = t0.elapsed().as_micros() as f64;
    }
}
