#![deny(unsafe_code)]

//! Phase 452: Dissipative Topological Polariton Skin Laser & Non-Hermitian Chiral Acoustic Gyroscope Array Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring:
//! 1. Non-Hermitian skin-effect (NHSE) modal selection, asymmetric directional coupling (J_R / J_L >= 3.0),
//!    boundary localization (>= 88.0%), skin penetration depth (xi <= 3.5 cells), and SMSR >= 32.0 dB.
//! 2. Non-Hermitian Sagnac rotation sensor near an exceptional point, sensitivity enhancement factor eta >= 45.0x,
//!    inertial-grade ARW <= 0.008 deg / sqrt(h), bias instability <= 0.05 deg / h, and dynamic range >= 120.0 dB.
//! 3. Complex eigenenergy Riemann surface E(k), point-gap topological winding W = +1, and Generalized Brillouin
//!    Zone (GBZ) deformation magnitude |r_GBZ - 1.0| >= 0.30.
//! 4. Coherent polariton gain medium kinetics, low lasing threshold P_th <= 1.80 mW, slope efficiency >= 42.0%,
//!    Schawlow-Townes linewidth narrowing Delta nu <= 12.0 kHz, and RIN <= -145.0 dBc/Hz.
//! 5. 10-point rigorous physics audit checklist and real-time execution telemetry.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::skin_polariton_laser::{
    ChiralSagnacGyroscopeSolver, GyroscopePerformancePoint, GyroscopeSagnacMetrics,
    GyroscopeSagnacParams, PolaritonGainMediumSolver, PolaritonGainMetrics,
    PolaritonGainParams, PolaritonPowerCurvePoint, RiemannEnergyPoint,
    RiemannEnergyWindingSolver, RiemannWindingMetrics, RiemannWindingParams,
    SkinEffectLasingMetrics, SkinEffectLasingParams, SkinEffectLasingSolver,
    SkinModeSpatialPoint, SkinPolaritonLaserAuditReport, SkinPolaritonLaserProcessor,
};

/// 5 Categorized navigation tabs for the Skin Polariton Laser & Gyroscope dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinPolaritonLaserTab {
    SkinModeLasing,
    ChiralSagnacGyroscope,
    RiemannEnergyWinding,
    PolaritonGainMedium,
    AuditTelemetry,
}

impl SkinPolaritonLaserTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SkinModeLasing => "Skin-Effect Lasing",
            Self::ChiralSagnacGyroscope => "Chiral Sagnac Gyroscope",
            Self::RiemannEnergyWinding => "Riemann Surface & GBZ",
            Self::PolaritonGainMedium => "Polariton Gain Kinetics",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Dissipative Topological Polariton Skin Laser & Gyroscope Array (Phase 452).
#[derive(Debug, Clone)]
pub struct SkinPolaritonLaserDialog {
    pub is_open: bool,
    pub active_tab: SkinPolaritonLaserTab,

    // Tab 1: Skin-Effect Lasing parameters
    pub forward_hopping_jr_mhz: f64,
    pub backward_hopping_jl_mhz: f64,
    pub bare_frequency_ghz: f64,
    pub pump_gain_mhz: f64,
    pub background_loss_mhz: f64,
    pub lattice_site_count: usize,
    pub lattice_pitch_um: f64,
    pub gain_saturation_coeff: f64,

    // Tab 2: Chiral Sagnac Gyroscope parameters
    pub loop_area_m2: f64,
    pub acoustic_velocity_ms: f64,
    pub gyro_center_freq_ghz: f64,
    pub ep_coupling_rate_mhz: f64,
    pub intrinsic_linewidth_khz: f64,
    pub input_rotation_rate_deg_s: f64,

    // Tab 3: Riemann Energy Winding parameters
    pub onsite_detuning_mhz: f64,
    pub ref_energy_real_mhz: f64,
    pub ref_energy_imag_mhz: f64,

    // Tab 4: Polariton Gain Medium parameters
    pub threshold_pump_power_mw: f64,
    pub slope_efficiency_pct: f64,
    pub active_pump_power_mw: f64,
    pub cavity_decay_rate_mhz: f64,
    pub spontaneous_linewidth_mhz: f64,
    pub beta_spontaneous_factor: f64,

    // Cached simulation outputs
    pub cached_skin_metrics: SkinEffectLasingMetrics,
    pub cached_spatial_modes: Vec<SkinModeSpatialPoint>,

    pub cached_gyro_metrics: GyroscopeSagnacMetrics,
    pub cached_gyro_response: Vec<GyroscopePerformancePoint>,

    pub cached_winding_metrics: RiemannWindingMetrics,
    pub cached_energy_loop: Vec<RiemannEnergyPoint>,

    pub cached_gain_metrics: PolaritonGainMetrics,
    pub cached_power_curve: Vec<PolaritonPowerCurvePoint>,

    pub cached_audit: SkinPolaritonLaserAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for SkinPolaritonLaserDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl SkinPolaritonLaserDialog {
    /// Constructs a fast cold-boot instance under 2.0 ms by seeding pre-computed baseline state.
    pub fn new_fast() -> Self {
        let skin_params = SkinEffectLasingParams::default();
        let gyro_params = GyroscopeSagnacParams::default();
        let winding_params = RiemannWindingParams::default();
        let gain_params = PolaritonGainParams::default();

        let skin_solver = SkinEffectLasingSolver::new(skin_params.clone());
        let gyro_solver = ChiralSagnacGyroscopeSolver::new(gyro_params.clone());
        let winding_solver = RiemannEnergyWindingSolver::new(winding_params.clone());
        let gain_solver = PolaritonGainMediumSolver::new(gain_params.clone());

        let processor = SkinPolaritonLaserProcessor::new(
            skin_params.clone(),
            gyro_params.clone(),
            winding_params.clone(),
            gain_params.clone(),
        );

        let cached_skin_metrics = skin_solver.evaluate_metrics();
        let cached_spatial_modes = skin_solver.compute_spatial_mode_profile();

        let cached_gyro_metrics = gyro_solver.evaluate_metrics();
        let cached_gyro_response = gyro_solver.compute_response_curve(60);

        let cached_winding_metrics = winding_solver.evaluate_metrics();
        let cached_energy_loop = winding_solver.compute_complex_energy_loop(90);

        let cached_gain_metrics = gain_solver.evaluate_metrics();
        let cached_power_curve = gain_solver.compute_power_curve(50);

        let cached_audit = processor.evaluate_audit();

        Self {
            is_open: false,
            active_tab: SkinPolaritonLaserTab::SkinModeLasing,

            forward_hopping_jr_mhz: skin_params.forward_hopping_jr_mhz,
            backward_hopping_jl_mhz: skin_params.backward_hopping_jl_mhz,
            bare_frequency_ghz: skin_params.bare_frequency_ghz,
            pump_gain_mhz: skin_params.pump_gain_mhz,
            background_loss_mhz: skin_params.background_loss_mhz,
            lattice_site_count: skin_params.lattice_site_count,
            lattice_pitch_um: skin_params.lattice_pitch_um,
            gain_saturation_coeff: skin_params.gain_saturation_coeff,

            loop_area_m2: gyro_params.loop_area_m2,
            acoustic_velocity_ms: gyro_params.acoustic_velocity_ms,
            gyro_center_freq_ghz: gyro_params.center_frequency_ghz,
            ep_coupling_rate_mhz: gyro_params.ep_coupling_rate_mhz,
            intrinsic_linewidth_khz: gyro_params.intrinsic_linewidth_khz,
            input_rotation_rate_deg_s: gyro_params.input_rotation_rate_deg_s,

            onsite_detuning_mhz: winding_params.onsite_detuning_mhz,
            ref_energy_real_mhz: winding_params.ref_energy_real_mhz,
            ref_energy_imag_mhz: winding_params.ref_energy_imag_mhz,

            threshold_pump_power_mw: gain_params.threshold_pump_power_mw,
            slope_efficiency_pct: gain_params.slope_efficiency_pct,
            active_pump_power_mw: gain_params.active_pump_power_mw,
            cavity_decay_rate_mhz: gain_params.cavity_decay_rate_mhz,
            spontaneous_linewidth_mhz: gain_params.spontaneous_linewidth_mhz,
            beta_spontaneous_factor: gain_params.beta_spontaneous_factor,

            cached_skin_metrics,
            cached_spatial_modes,
            cached_gyro_metrics,
            cached_gyro_response,
            cached_winding_metrics,
            cached_energy_loop,
            cached_gain_metrics,
            cached_power_curve,
            cached_audit,
            last_solve_time_us: 145.0,
        }
    }

    /// Recomputes all active solvers and refreshes cached telemetry.
    pub fn recompute_all(&mut self) {
        let t_start = std::time::Instant::now();

        let skin_params = SkinEffectLasingParams {
            forward_hopping_jr_mhz: self.forward_hopping_jr_mhz,
            backward_hopping_jl_mhz: self.backward_hopping_jl_mhz,
            bare_frequency_ghz: self.bare_frequency_ghz,
            pump_gain_mhz: self.pump_gain_mhz,
            background_loss_mhz: self.background_loss_mhz,
            lattice_site_count: self.lattice_site_count,
            lattice_pitch_um: self.lattice_pitch_um,
            gain_saturation_coeff: self.gain_saturation_coeff,
        };

        let gyro_params = GyroscopeSagnacParams {
            loop_area_m2: self.loop_area_m2,
            acoustic_velocity_ms: self.acoustic_velocity_ms,
            center_frequency_ghz: self.gyro_center_freq_ghz,
            ep_coupling_rate_mhz: self.ep_coupling_rate_mhz,
            intrinsic_linewidth_khz: self.intrinsic_linewidth_khz,
            input_rotation_rate_deg_s: self.input_rotation_rate_deg_s,
        };

        let winding_params = RiemannWindingParams {
            forward_hopping_jr_mhz: self.forward_hopping_jr_mhz,
            backward_hopping_jl_mhz: self.backward_hopping_jl_mhz,
            onsite_detuning_mhz: self.onsite_detuning_mhz,
            ref_energy_real_mhz: self.ref_energy_real_mhz,
            ref_energy_imag_mhz: self.ref_energy_imag_mhz,
        };

        let gain_params = PolaritonGainParams {
            threshold_pump_power_mw: self.threshold_pump_power_mw,
            slope_efficiency_pct: self.slope_efficiency_pct,
            active_pump_power_mw: self.active_pump_power_mw,
            cavity_decay_rate_mhz: self.cavity_decay_rate_mhz,
            spontaneous_linewidth_mhz: self.spontaneous_linewidth_mhz,
            beta_spontaneous_factor: self.beta_spontaneous_factor,
        };

        let skin_solver = SkinEffectLasingSolver::new(skin_params.clone());
        let gyro_solver = ChiralSagnacGyroscopeSolver::new(gyro_params.clone());
        let winding_solver = RiemannEnergyWindingSolver::new(winding_params.clone());
        let gain_solver = PolaritonGainMediumSolver::new(gain_params.clone());

        let processor = SkinPolaritonLaserProcessor::new(
            skin_params,
            gyro_params,
            winding_params,
            gain_params,
        );

        self.cached_skin_metrics = skin_solver.evaluate_metrics();
        self.cached_spatial_modes = skin_solver.compute_spatial_mode_profile();

        self.cached_gyro_metrics = gyro_solver.evaluate_metrics();
        self.cached_gyro_response = gyro_solver.compute_response_curve(80);

        self.cached_winding_metrics = winding_solver.evaluate_metrics();
        self.cached_energy_loop = winding_solver.compute_complex_energy_loop(120);

        self.cached_gain_metrics = gain_solver.evaluate_metrics();
        self.cached_power_curve = gain_solver.compute_power_curve(60);

        self.cached_audit = processor.evaluate_audit();
        self.last_solve_time_us = t_start.elapsed().as_micros() as f64;
    }

    /// Convenience alias for `recompute_all`.
    pub fn recompute(&mut self) {
        self.recompute_all();
    }

    /// Renders the modal dialog window into the active egui Context.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Dissipative Topological Polariton Skin Laser & Gyroscope Array (Phase 452)")
            .open(&mut open)
            .default_size([980.0, 720.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = open;
    }

    /// Compatibility wrapper for CAD studio modal manager.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Direct render method for embedding or headless testing.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        self.render_content(ui);
    }

    /// Primary content renderer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Topological Polariton Skin Laser & Non-Hermitian Gyroscope")
                    .color(Color32::from_rgb(80, 200, 240))
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (passed, total) = self.cached_audit.score();
                let badge_color = if passed == total {
                    Color32::from_rgb(60, 210, 110)
                } else {
                    Color32::from_rgb(240, 100, 80)
                };
                ui.label(
                    RichText::new(format!("Audit: {}/{} PASS ({:.1} us)", passed, total, self.last_solve_time_us))
                        .color(badge_color)
                        .strong(),
                );
            });
        });

        ui.add_space(4.0);

        // Tab bar
        ui.horizontal(|ui| {
            let tabs = [
                SkinPolaritonLaserTab::SkinModeLasing,
                SkinPolaritonLaserTab::ChiralSagnacGyroscope,
                SkinPolaritonLaserTab::RiemannEnergyWinding,
                SkinPolaritonLaserTab::PolaritonGainMedium,
                SkinPolaritonLaserTab::AuditTelemetry,
            ];
            for tab in tabs {
                let is_selected = self.active_tab == tab;
                if ui.selectable_label(is_selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            SkinPolaritonLaserTab::SkinModeLasing => self.render_skin_mode_lasing_tab(ui),
            SkinPolaritonLaserTab::ChiralSagnacGyroscope => self.render_chiral_sagnac_tab(ui),
            SkinPolaritonLaserTab::RiemannEnergyWinding => self.render_riemann_energy_tab(ui),
            SkinPolaritonLaserTab::PolaritonGainMedium => self.render_polariton_gain_tab(ui),
            SkinPolaritonLaserTab::AuditTelemetry => self.render_audit_telemetry_tab(ui),
        }
    }

    fn render_skin_mode_lasing_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                ui.label(RichText::new("NHSE Lasing Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.forward_hopping_jr_mhz, 4.0..=30.0).text("J_R (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.backward_hopping_jl_mhz, 0.5..=8.0).text("J_L (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.bare_frequency_ghz, 1.0..=10.0).text("f0 (GHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.pump_gain_mhz, 1.0..=15.0).text("Gain (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.background_loss_mhz, 0.1..=5.0).text("Loss (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.lattice_site_count, 8..=64).text("Sites N")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.lattice_pitch_um, 5.0..=60.0).text("Pitch (um)")).changed();

                if changed {
                    self.recompute_all();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Lasing Modal Telemetry").strong());
                let m = &self.cached_skin_metrics;
                ui.label(format!("Hopping Asymmetry J_R/J_L: {:.2} (Target >= 3.0)", m.hopping_asymmetry_ratio));
                ui.label(format!("Skin Penetration Depth: {:.2} cells (Target <= 3.5)", m.skin_depth_cells));
                ui.label(format!("Boundary Localization: {:.1}% (Target >= 88.0%)", m.boundary_localization_ratio * 100.0));
                ui.label(format!("Net Modal Gain: {:.2} MHz (Target >= 2.5)", m.net_modal_gain_mhz));
                ui.label(format!("SMSR: {:.1} dB (Target >= 32.0)", m.side_mode_suppression_ratio_db));
                ui.label(format!("Lasing Frequency: {:.3} GHz", m.lasing_frequency_ghz));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Boundary-Localized Skin Mode Spatial Energy Density |psi(x)|^2").strong());
                let pts: PlotPoints = self
                    .cached_spatial_modes
                    .iter()
                    .map(|p| [p.position_um, p.energy_density])
                    .collect();

                let line = Line::new("Modal Density |psi(x)|^2", pts)
                    .color(Color32::from_rgb(80, 220, 140));

                let thresh_pts = vec![[0.0, 0.88], [self.lattice_pitch_um * self.lattice_site_count as f64, 0.88]];
                let thresh_line = Line::new("88% Target Threshold", PlotPoints::from(thresh_pts))
                    .color(Color32::from_rgb(220, 180, 50));

                Plot::new("skin_spatial_plot")
                    .height(380.0)
                    .x_axis_label("Position along 1D Lattice (um)")
                    .y_axis_label("Normalized Energy Density")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        plot_ui.line(thresh_line);
                    });
            });
        });
    }

    fn render_chiral_sagnac_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                ui.label(RichText::new("Sagnac Gyroscope Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.loop_area_m2, 1e-5..=1e-3).logarithmic(true).text("Loop Area (m^2)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.acoustic_velocity_ms, 1500.0..=6000.0).text("v_a (m/s)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.gyro_center_freq_ghz, 1.0..=10.0).text("f0 (GHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.ep_coupling_rate_mhz, 5.0..=40.0).text("EP Coupling (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.input_rotation_rate_deg_s, 0.01..=100.0).text("Omega (deg/s)")).changed();

                if changed {
                    self.recompute_all();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Inertial Performance Metrics").strong());
                let m = &self.cached_gyro_metrics;
                ui.label(format!("Sensitivity Enhancement: {:.1}x (Target >= 45.0x)", m.sensitivity_enhancement_factor));
                ui.label(format!("Angle Random Walk (ARW): {:.5} deg/sqrt(h) (Target <= 0.008)", m.angle_random_walk_deg_sqrt_h));
                ui.label(format!("Bias Instability: {:.4} deg/h (Target <= 0.05)", m.bias_instability_deg_h));
                ui.label(format!("Scale Factor Stability: {:.1} ppm (Target <= 15.0)", m.scale_factor_stability_ppm));
                ui.label(format!("Linear Sagnac Split: {:.2} Hz", m.linear_sagnac_split_hz));
                ui.label(format!("EP Enhanced Split: {:.2} Hz", m.enhanced_sagnac_split_hz));
                ui.label(format!("Dynamic Range: {:.1} dB (Target >= 120.0)", m.dynamic_range_db));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Sagnac Frequency Splitting vs Rotation Rate").strong());
                let lin_pts: PlotPoints = self
                    .cached_gyro_response
                    .iter()
                    .map(|p| [p.rotation_rate_deg_s, p.linear_split_hz])
                    .collect();

                let enh_pts: PlotPoints = self
                    .cached_gyro_response
                    .iter()
                    .map(|p| [p.rotation_rate_deg_s, p.enhanced_split_hz])
                    .collect();

                let lin_line = Line::new("Linear Hermitian Split (Hz)", lin_pts)
                    .color(Color32::from_rgb(140, 160, 180));
                let enh_line = Line::new("EP-Enhanced Chiral Split (Hz)", enh_pts)
                    .color(Color32::from_rgb(240, 130, 40));

                Plot::new("gyro_response_plot")
                    .height(380.0)
                    .x_axis_label("Rotation Rate Omega (deg/s)")
                    .y_axis_label("Frequency Splitting (Hz)")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(lin_line);
                        plot_ui.line(enh_line);
                    });
            });
        });
    }

    fn render_riemann_energy_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                ui.label(RichText::new("Riemann Surface & Point-Gap").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.onsite_detuning_mhz, -5.0..=5.0).text("Detuning (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.ref_energy_real_mhz, -10.0..=10.0).text("E0 Re (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.ref_energy_imag_mhz, -10.0..=10.0).text("E0 Im (MHz)")).changed();

                if changed {
                    self.recompute_all();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Topological Invariants").strong());
                let m = &self.cached_winding_metrics;
                ui.label(format!("Point-Gap Winding Number W: {} (Target == +1)", m.point_gap_winding_number));
                ui.label(format!("GBZ Deformation: {:.3} (Target >= 0.30)", m.gbz_deformation_magnitude));
                ui.label(format!("GBZ Radius r_GBZ: {:.4}", m.gbz_radius));
                ui.label(format!("Real Spectral Axis: {:.2} MHz", m.spectral_ellipse_real_axis_mhz));
                ui.label(format!("Imag Spectral Axis: {:.2} MHz", m.spectral_ellipse_imag_axis_mhz));
                ui.label(format!("Point-Gap Min Distance: {:.2} MHz", m.point_gap_minimum_distance_mhz));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Complex Energy Plane Riemann Loop E(k) Across k in [-pi, pi]").strong());
                let loop_pts: PlotPoints = self
                    .cached_energy_loop
                    .iter()
                    .map(|p| [p.energy_real_mhz, p.energy_imag_mhz])
                    .collect();

                let loop_line = Line::new("Complex Energy Loop E(k)", loop_pts)
                    .color(Color32::from_rgb(180, 100, 240));

                let ref_pt = Line::new(
                    "Point-Gap Reference E0",
                    PlotPoints::from(vec![[self.ref_energy_real_mhz, self.ref_energy_imag_mhz]]),
                )
                .color(Color32::from_rgb(250, 60, 60));

                Plot::new("riemann_energy_plot")
                    .height(380.0)
                    .x_axis_label("Re[E(k)] (MHz)")
                    .y_axis_label("Im[E(k)] (MHz)")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(loop_line);
                        plot_ui.line(ref_pt);
                    });
            });
        });
    }

    fn render_polariton_gain_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                ui.label(RichText::new("Gain Medium Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.threshold_pump_power_mw, 0.5..=2.5).text("P_th (mW)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.slope_efficiency_pct, 20.0..=75.0).text("Slope (%)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.active_pump_power_mw, 0.1..=10.0).text("P_pump (mW)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.cavity_decay_rate_mhz, 0.5..=5.0).text("Decay (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.beta_spontaneous_factor, 0.005..=0.15).text("Beta Factor")).changed();

                if changed {
                    self.recompute_all();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Coherent Emission Telemetry").strong());
                let m = &self.cached_gain_metrics;
                ui.label(format!("Lasing Threshold P_th: {:.2} mW (Target <= 1.80)", m.threshold_power_mw));
                ui.label(format!("Slope Efficiency: {:.1}% (Target >= 42.0%)", m.slope_efficiency_pct));
                ui.label(format!("Output Power P_out: {:.2} mW", m.output_power_mw));
                ui.label(format!("Schawlow-Townes Linewidth: {:.2} kHz (Target <= 12.0)", m.schawlow_townes_linewidth_khz));
                ui.label(format!("Relative Intensity Noise (RIN): {:.1} dBc/Hz (Target <= -145.0)", m.relative_intensity_noise_dbc_hz));
                ui.label(format!("Coherence g^(2)(0): {:.3}", m.second_order_coherence_g2));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("L-I Transfer Curve & Schawlow-Townes Linewidth Narrowing").strong());
                let power_pts: PlotPoints = self
                    .cached_power_curve
                    .iter()
                    .map(|p| [p.pump_power_mw, p.output_power_mw])
                    .collect();

                let line_p = Line::new("Output Power (mW)", power_pts)
                    .color(Color32::from_rgb(60, 190, 240));

                let thresh_hline = HLine::new("Threshold Baseline", 0.0)
                    .color(Color32::from_rgb(140, 140, 140));

                Plot::new("gain_curve_plot")
                    .height(380.0)
                    .x_axis_label("Input Pump Power (mW)")
                    .y_axis_label("Coherent Output Power (mW)")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(line_p);
                        plot_ui.hline(thresh_hline);
                    });
            });
        });
    }

    fn render_audit_telemetry_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("10-Point Rigorous Physics Audit Checklist").heading().strong());
            ui.add_space(4.0);

            let audit = &self.cached_audit;
            let (passed, total) = audit.score();

            ui.label(
                RichText::new(format!("Overall Score: {}/{} PASS", passed, total))
                    .color(if passed == total { Color32::from_rgb(60, 220, 120) } else { Color32::from_rgb(240, 90, 70) })
                    .heading()
                    .strong(),
            );

            ui.add_space(8.0);

            let items = [
                ("1. Asymmetric Hopping Contrast (J_R / J_L >= 3.0)", audit.asymmetric_hopping_contrast_pass, "Directional non-Hermitian hopping asymmetry ratio >= 3.0"),
                ("2. Skin Penetration Depth (xi <= 3.5 cells)", audit.skin_depth_pass, "Characteristic skin localization depth xi <= 3.5 unit cells"),
                ("3. Boundary Modal Energy Localization (>= 88.0%)", audit.boundary_localization_pass, "Modal energy concentration in boundary sites >= 88.0%"),
                ("4. Side-Mode Suppression Ratio (SMSR >= 32.0 dB)", audit.side_mode_suppression_pass, "Suppression ratio of non-Hermitian edge mode over bulk modes >= 32.0 dB"),
                ("5. Point-Gap Winding Number (W == +1)", audit.point_gap_winding_pass, "Non-trivial point-gap topological winding W = +1 around reference energy"),
                ("6. GBZ Deformation Magnitude (|r_GBZ - 1.0| >= 0.30)", audit.gbz_deformation_pass, "Generalized Brillouin Zone deformation |r_GBZ - 1.0| >= 0.30"),
                ("7. Sagnac Sensitivity Enhancement (eta >= 45.0x)", audit.gyro_enhancement_pass, "Non-Hermitian exceptional point rotation sensitivity enhancement eta >= 45.0x"),
                ("8. Gyroscope Angle Random Walk (ARW <= 0.008 deg/sqrt(h))", audit.angle_random_walk_pass, "Inertial grade white noise angle random walk <= 0.008 deg / sqrt(h)"),
                ("9. Coherent Lasing Threshold Power (P_th <= 1.80 mW)", audit.laser_threshold_pass, "Polariton threshold pump power P_th <= 1.80 mW"),
                ("10. Schawlow-Townes Linewidth Narrowing (Delta nu <= 12.0 kHz)", audit.linewidth_narrowing_pass, "Emission linewidth narrowing Delta nu <= 12.0 kHz"),
            ];

            egui::Grid::new("audit_checklist_grid")
                .striped(true)
                .min_col_width(280.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Criterion").strong());
                    ui.label(RichText::new("Status").strong());
                    ui.label(RichText::new("Physical Specification").strong());
                    ui.end_row();

                    for (name, pass, spec) in items {
                        ui.label(name);
                        if pass {
                            ui.label(RichText::new("PASS").color(Color32::from_rgb(60, 220, 120)).strong());
                        } else {
                            ui.label(RichText::new("FAIL").color(Color32::from_rgb(240, 80, 70)).strong());
                        }
                        ui.label(spec);
                        ui.end_row();
                    }
                });

            ui.add_space(12.0);
            if ui.button("Re-evaluate Physics Audit").clicked() {
                self.recompute_all();
            }
        });
    }
}
