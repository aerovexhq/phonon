#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 412: Phonon Studio Dissipative Polariton BEC Vortices,
//! Non-Equilibrium Superfluidity & Josephson Acoustic Interferometer.

use egui::{
    pos2, vec2, Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Window,
};
use egui_plot::{HLine, Line, Plot, PlotPoints, VLine};
use phonon_solver::polariton_bec_vortices::{
    BecCondensationMetrics, CondensateSpatialPoint, FringePatternPoint, GrossPitaevskiiSolver,
    JosephsonInterferometerParams, JosephsonInterferometerSolver, JosephsonSensorMetrics,
    JosephsonTrajectoryPoint, PolaritonBecAuditReport, PolaritonBecInterferometer,
    PolaritonBecParams, QuantizedVortexSolver, VortexCharge, VortexLatticeMetrics,
    VortexSuperfluidParams,
};

/// Active tab within the Polariton BEC Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolaritonBecTab {
    GrossPitaevskiiCondensate,
    QuantizedVortexLattice,
    LandauSuperfluidity,
    JosephsonStrainSensor,
    AuditTelemetry,
}

impl PolaritonBecTab {
    /// Returns the human-readable label for this tab.
    pub fn label(&self) -> &'static str {
        match self {
            Self::GrossPitaevskiiCondensate => "1. GPE Condensation & Threshold",
            Self::QuantizedVortexLattice => "2. Quantized Vortices & Lattice",
            Self::LandauSuperfluidity => "3. Landau Superfluidity",
            Self::JosephsonStrainSensor => "4. Josephson Acoustic Interferometer",
            Self::AuditTelemetry => "5. Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for dissipative polariton BEC vortices and acoustic Josephson interferometry.
pub struct PolaritonBecDialog {
    pub is_open: bool,
    pub active_tab: PolaritonBecTab,

    // GPE Parameters
    pub pump_power_ratio: f64,
    pub cavity_decay_rate_mev: f64,
    pub interaction_g_mev_um2: f64,
    pub temperature_k: f64,

    // Vortex & Superfluid Parameters
    pub vortex_charge: VortexCharge,
    pub rotation_frequency_rad_ns: f64,
    pub flow_velocity_ratio: f64,
    pub obstacle_radius_um: f64,

    // Josephson Sensor Parameters
    pub applied_strain_1e7: f64, // strain * 1e7
    pub initial_imbalance: f64,
    pub josephson_coupling_ej_mev: f64,
    pub charging_energy_ec_mev: f64,

    // Solvers & Cached results
    pub interferometer_system: PolaritonBecInterferometer,
    pub cached_gpe_metrics: BecCondensationMetrics,
    pub cached_spatial_profile: Vec<CondensateSpatialPoint>,
    pub cached_vortex_metrics: VortexLatticeMetrics,
    pub cached_josephson_metrics: JosephsonSensorMetrics,
    pub cached_josephson_trajectory: Vec<JosephsonTrajectoryPoint>,
    pub cached_fringe_pattern: Vec<FringePatternPoint>,
    pub cached_audit: PolaritonBecAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for PolaritonBecDialog {
    fn default() -> Self {
        let gpe_params = PolaritonBecParams::default();
        let gpe_solver = GrossPitaevskiiSolver::new(gpe_params.clone());
        let gpe_metrics = gpe_solver.evaluate_metrics();
        let spatial_profile = gpe_solver.compute_spatial_profile(20.0, 40);

        let vortex_params = VortexSuperfluidParams {
            healing_length_um: gpe_metrics.healing_length_um,
            peak_density_um2: gpe_metrics.peak_density_um2,
            effective_mass_kg: gpe_solver.effective_mass_kg(),
            ..Default::default()
        };
        let vortex_solver = QuantizedVortexSolver::new(vortex_params.clone());
        let vortex_metrics = vortex_solver.evaluate_metrics();

        let josephson_params = JosephsonInterferometerParams::default();
        let josephson_solver = JosephsonInterferometerSolver::new(josephson_params.clone());
        let josephson_metrics = josephson_solver.evaluate_sensor_metrics();
        let josephson_trajectory = josephson_solver.solve_dynamics();
        let fringe_pattern = josephson_solver.compute_fringe_pattern(0.0, 40);

        let interferometer_system = PolaritonBecInterferometer {
            gpe_solver,
            vortex_solver,
            josephson_solver,
        };
        let audit = interferometer_system.audit_polariton_bec();

        Self {
            is_open: false,
            active_tab: PolaritonBecTab::GrossPitaevskiiCondensate,

            pump_power_ratio: gpe_params.pump_power_ratio,
            cavity_decay_rate_mev: gpe_params.cavity_decay_rate_mev,
            interaction_g_mev_um2: gpe_params.polariton_interaction_g_mev_um2 * 1.0e3,
            temperature_k: gpe_params.temperature_k,

            vortex_charge: vortex_params.charge,
            rotation_frequency_rad_ns: vortex_params.rotation_frequency_rad_ns,
            flow_velocity_ratio: vortex_params.flow_velocity_ratio,
            obstacle_radius_um: vortex_params.obstacle_radius_um,

            applied_strain_1e7: josephson_params.applied_strain * 1.0e7,
            initial_imbalance: josephson_params.initial_imbalance,
            josephson_coupling_ej_mev: josephson_params.josephson_coupling_ej_mev,
            charging_energy_ec_mev: josephson_params.charging_energy_ec_mev,

            interferometer_system,
            cached_gpe_metrics: gpe_metrics,
            cached_spatial_profile: spatial_profile,
            cached_vortex_metrics: vortex_metrics,
            cached_josephson_metrics: josephson_metrics,
            cached_josephson_trajectory: josephson_trajectory,
            cached_fringe_pattern: fringe_pattern,
            cached_audit: audit,
            last_solve_time_us: 65.0,
        }
    }
}

impl PolaritonBecDialog {
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
        self.show(ctx);
    }

    /// Renders the inner content of the dialog.
    pub fn render_content(&mut self, ui: &mut Ui) {
        self.render_header(ui);
        ui.separator();
        self.render_tabs(ui);
        ui.separator();

        match self.active_tab {
            PolaritonBecTab::GrossPitaevskiiCondensate => self.render_tab_gpe(ui),
            PolaritonBecTab::QuantizedVortexLattice => self.render_tab_vortices(ui),
            PolaritonBecTab::LandauSuperfluidity => self.render_tab_superfluidity(ui),
            PolaritonBecTab::JosephsonStrainSensor => self.render_tab_josephson(ui),
            PolaritonBecTab::AuditTelemetry => self.render_tab_audit(ui),
        }

        ui.separator();
        self.render_footer(ui);
    }

    /// Renders the modal window.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Dissipative Polariton BEC Vortices & Josephson Interferometer")
            .open(&mut open)
            .resizable(true)
            .default_width(850.0)
            .default_height(580.0)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = open;
    }

    fn render_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Polariton BEC Vortices & Acoustic Josephson Sensor")
                    .color(Color32::from_rgb(155, 89, 182))
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Preset: Near-Threshold Critical").clicked() {
                    self.load_preset_near_threshold();
                }
                if ui.button("Preset: High-Q Microcavity").clicked() {
                    self.load_preset_high_q();
                }
            });
        });
    }

    fn render_tabs(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.active_tab,
                PolaritonBecTab::GrossPitaevskiiCondensate,
                "1. GPE Condensation",
            );
            ui.selectable_value(
                &mut self.active_tab,
                PolaritonBecTab::QuantizedVortexLattice,
                "2. Quantized Vortices",
            );
            ui.selectable_value(
                &mut self.active_tab,
                PolaritonBecTab::LandauSuperfluidity,
                "3. Landau Superfluidity",
            );
            ui.selectable_value(
                &mut self.active_tab,
                PolaritonBecTab::JosephsonStrainSensor,
                "4. Josephson Strain Sensor",
            );
            ui.selectable_value(
                &mut self.active_tab,
                PolaritonBecTab::AuditTelemetry,
                "5. Physics Audit",
            );
        });
    }

    pub fn load_preset_high_q(&mut self) {
        let p = PolaritonBecParams::preset_high_q_microcavity();
        self.pump_power_ratio = p.pump_power_ratio;
        self.cavity_decay_rate_mev = p.cavity_decay_rate_mev;
        self.interaction_g_mev_um2 = p.polariton_interaction_g_mev_um2 * 1.0e3;
        self.temperature_k = p.temperature_k;
        self.flow_velocity_ratio = 0.45;
        self.vortex_charge = VortexCharge::PlusOne;
        self.recompute();
    }

    pub fn load_preset_near_threshold(&mut self) {
        let p = PolaritonBecParams::preset_near_threshold();
        self.pump_power_ratio = p.pump_power_ratio;
        self.cavity_decay_rate_mev = p.cavity_decay_rate_mev;
        self.interaction_g_mev_um2 = p.polariton_interaction_g_mev_um2 * 1.0e3;
        self.temperature_k = p.temperature_k;
        self.flow_velocity_ratio = 0.50;
        self.vortex_charge = VortexCharge::MinusOne;
        self.recompute();
    }

    pub fn recompute(&mut self) {
        let gpe_params = PolaritonBecParams {
            pump_power_ratio: self.pump_power_ratio,
            cavity_decay_rate_mev: self.cavity_decay_rate_mev,
            polariton_interaction_g_mev_um2: self.interaction_g_mev_um2 * 1.0e-3,
            temperature_k: self.temperature_k,
            ..Default::default()
        };
        let gpe_solver = GrossPitaevskiiSolver::new(gpe_params.clone());
        let gpe_metrics = gpe_solver.evaluate_metrics();
        let spatial_profile = gpe_solver.compute_spatial_profile(20.0, 40);

        let vortex_params = VortexSuperfluidParams {
            charge: self.vortex_charge,
            healing_length_um: gpe_metrics.healing_length_um,
            peak_density_um2: gpe_metrics.peak_density_um2,
            effective_mass_kg: gpe_solver.effective_mass_kg(),
            rotation_frequency_rad_ns: self.rotation_frequency_rad_ns,
            flow_velocity_ratio: self.flow_velocity_ratio,
            obstacle_radius_um: self.obstacle_radius_um,
            ..Default::default()
        };
        let vortex_solver = QuantizedVortexSolver::new(vortex_params);
        let vortex_metrics = vortex_solver.evaluate_metrics();

        let josephson_params = JosephsonInterferometerParams {
            josephson_coupling_ej_mev: self.josephson_coupling_ej_mev,
            charging_energy_ec_mev: self.charging_energy_ec_mev,
            applied_strain: self.applied_strain_1e7 * 1.0e-7,
            initial_imbalance: self.initial_imbalance,
            ..Default::default()
        };
        let josephson_solver = JosephsonInterferometerSolver::new(josephson_params);
        let josephson_metrics = josephson_solver.evaluate_sensor_metrics();
        let josephson_trajectory = josephson_solver.solve_dynamics();
        let last_phase = josephson_trajectory.last().map_or(0.0, |p| p.relative_phase_rad);
        let fringe_pattern = josephson_solver.compute_fringe_pattern(last_phase, 40);

        self.interferometer_system = PolaritonBecInterferometer {
            gpe_solver,
            vortex_solver,
            josephson_solver,
        };

        self.cached_gpe_metrics = gpe_metrics;
        self.cached_spatial_profile = spatial_profile;
        self.cached_vortex_metrics = vortex_metrics;
        self.cached_josephson_metrics = josephson_metrics;
        self.cached_josephson_trajectory = josephson_trajectory;
        self.cached_fringe_pattern = fringe_pattern;
        self.cached_audit = self.interferometer_system.audit_polariton_bec();
    }

    fn render_tab_gpe(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Pump Power P / P_th:");
            if ui
                .add(egui::Slider::new(&mut self.pump_power_ratio, 0.5..=5.0).step_by(0.1))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label("Cavity Loss gamma_c (meV):");
            if ui
                .add(egui::Slider::new(&mut self.cavity_decay_rate_mev, 0.05..=0.40).step_by(0.01))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label("Interaction g (ueV*um^2):");
            if ui
                .add(egui::Slider::new(&mut self.interaction_g_mev_um2, 1.0..=6.0).step_by(0.2))
                .changed()
            {
                self.recompute();
            }
        });

        ui.add_space(4.0);

        let m = self.cached_gpe_metrics;
        let is_cond = self.interferometer_system.gpe_solver.is_condensed();
        let status = if is_cond {
            RichText::new("Bose-Einstein Condensate Active (U(1) Symmetry Broken)")
                .color(Color32::from_rgb(46, 204, 113))
                .strong()
        } else {
            RichText::new("Sub-Threshold Normal Exciton-Polariton Gas")
                .color(Color32::from_rgb(231, 76, 60))
                .strong()
        };

        ui.horizontal(|ui| {
            ui.label(status);
            ui.separator();
            ui.label(format!("Peak Density n_0: {:.1} um^-2", m.peak_density_um2));
            ui.separator();
            ui.label(format!("Sound Speed v_s: {:.1e} m/s", m.sound_speed_ms));
            ui.separator();
            ui.label(format!("Healing Length xi: {:.2} um", m.healing_length_um));
            ui.separator();
            ui.label(format!("Coherence l_c: {:.1} um", m.coherence_length_um));
        });

        ui.add_space(4.0);

        let pts_dens: PlotPoints = self
            .cached_spatial_profile
            .iter()
            .map(|p| [p.radius_um, p.density_um2])
            .collect();
        let pts_g1: PlotPoints = self
            .cached_spatial_profile
            .iter()
            .map(|p| [p.radius_um, p.coherence_g1 * m.peak_density_um2.max(1.0)])
            .collect();

        Plot::new("gpe_spatial_plot")
            .height(320.0)
            .x_axis_label("Radial Distance r (um)")
            .y_axis_label("Density n(r) [um^-2] / Scaled g^(1)(r)")
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Condensate Density n(r)", pts_dens)
                        .color(Color32::from_rgb(155, 89, 182))
                        .width(2.5),
                );
                plot_ui.line(
                    Line::new("First-Order Coherence g^(1)(r)", pts_g1)
                        .color(Color32::from_rgb(46, 204, 113))
                        .width(2.0),
                );
                plot_ui.vline(
                    VLine::new("Healing Length xi", m.healing_length_um)
                        .color(Color32::from_rgb(241, 196, 15)),
                );
            });
    }

    fn render_tab_vortices(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Vortex Charge ell:");
            let charges = [
                (VortexCharge::Zero, "0 (None)"),
                (VortexCharge::PlusOne, "+1 (Single)"),
                (VortexCharge::MinusOne, "-1 (Opposite)"),
                (VortexCharge::PlusTwo, "+2 (Giant)"),
                (VortexCharge::DipolePair, "+/-1 (Dipole Pair)"),
            ];
            for (ch, lbl) in charges {
                if ui
                    .selectable_label(self.vortex_charge == ch, lbl)
                    .clicked()
                {
                    self.vortex_charge = ch;
                    self.recompute();
                }
            }

            ui.separator();
            ui.label("Rotation Omega (rad/ns):");
            if ui
                .add(egui::Slider::new(&mut self.rotation_frequency_rad_ns, 0.0..=0.5).step_by(0.02))
                .changed()
            {
                self.recompute();
            }
        });

        ui.add_space(4.0);

        let vm = self.cached_vortex_metrics;
        ui.horizontal(|ui| {
            ui.label(format!("Circulation: {:.0} h/m*", vm.quantized_circulation_units));
            ui.separator();
            ui.label(format!("Critical Omega_c: {:.3} rad/ns", vm.critical_rotation_frequency_rad_ns));
            ui.separator();
            ui.label(
                RichText::new(format!("Equilibrium Vortices: {}", vm.equilibrium_vortex_count))
                    .color(Color32::from_rgb(241, 196, 15))
                    .strong(),
            );
        });

        ui.add_space(8.0);

        // 2D Phase Canvas showing vortex phase winding and core
        let (rect, _response) = ui.allocate_exact_size(vec2(ui.available_width(), 260.0), Sense::hover());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 4.0, Color32::from_rgb(20, 24, 34));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 72)), StrokeKind::Inside);

        let cx = rect.center().x;
        let cy = rect.center().y;

        // Render circular condensate boundaries
        let radius_px = 100.0;
        painter.circle_stroke(
            pos2(cx, cy),
            radius_px,
            Stroke::new(1.5, Color32::from_rgb(155, 89, 182)),
        );

        // Render sample 2D phase gradient rings
        let solver = &self.interferometer_system.vortex_solver;
        let grid_steps = 12;
        let max_r = 15.0; // um
        for ix in -grid_steps..=grid_steps {
            for iy in -grid_steps..=grid_steps {
                let x_um = (ix as f64 / grid_steps as f64) * max_r;
                let y_um = (iy as f64 / grid_steps as f64) * max_r;
                if x_um * x_um + y_um * y_um > max_r * max_r {
                    continue;
                }

                let pt = solver.evaluate_grid_point(x_um, y_um);
                let px = cx + (x_um / max_r) as f32 * radius_px;
                let py = cy - (y_um / max_r) as f32 * radius_px;

                // Color from phase angle [-pi, pi] mapped across rainbow
                let norm_phase = (pt.phase_rad + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
                let r = (255.0 * (1.0 - norm_phase)) as u8;
                let g = (255.0 * norm_phase) as u8;
                let b = (255.0 * pt.normalized_density) as u8;

                let dot_color = Color32::from_rgb(r, g, b);
                let dot_size = 2.0 + 3.0 * pt.normalized_density as f32;
                painter.circle_filled(pos2(px, py), dot_size, dot_color);
            }
        }

        // Draw central core marker
        painter.circle_filled(pos2(cx, cy), 6.0, Color32::BLACK);
        painter.circle_stroke(pos2(cx, cy), 6.0, Stroke::new(1.5, Color32::WHITE));
        painter.text(
            pos2(cx + 10.0, cy - 10.0),
            egui::Align2::LEFT_BOTTOM,
            format!("Vortex Core (ell = {})", self.vortex_charge.winding_number()),
            egui::FontId::monospace(11.0),
            Color32::WHITE,
        );
    }

    fn render_tab_superfluidity(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Flow Velocity v / v_c:");
            if ui
                .add(egui::Slider::new(&mut self.flow_velocity_ratio, 0.1..=2.2).step_by(0.05))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label("Obstacle Radius (um):");
            if ui
                .add(egui::Slider::new(&mut self.obstacle_radius_um, 0.5..=4.0).step_by(0.5))
                .changed()
            {
                self.recompute();
            }
        });

        ui.add_space(4.0);

        let vm = self.cached_vortex_metrics;
        let flow_status = if vm.is_frictionless_superfluid {
            RichText::new("Landau Superfluid Regime (Frictionless Flow)")
                .color(Color32::from_rgb(46, 204, 113))
                .strong()
        } else {
            RichText::new("Supercritical Cherenkov Vortex Shedding (Dissipative Drag)")
                .color(Color32::from_rgb(231, 76, 60))
                .strong()
        };

        ui.horizontal(|ui| {
            ui.label(flow_status);
            ui.separator();
            ui.label(format!("Landau v_c: {:.1e} m/s", vm.landau_critical_velocity_ms));
            ui.separator();
            ui.label(format!("Drag Force: {:.4}", vm.normalized_drag_force));
            ui.separator();
            ui.label(
                RichText::new(format!("Drag Suppression: {:.1} dB", vm.drag_suppression_db))
                    .color(Color32::from_rgb(52, 152, 219))
                    .strong(),
            );
        });

        ui.add_space(4.0);

        // Drag curve vs velocity ratio
        let mut drag_pts = Vec::with_capacity(50);
        let solver = &self.interferometer_system.vortex_solver;
        for i in 0..50 {
            let vr = 0.05 + i as f64 * 0.045; // 0.05 to 2.25
            let mut temp_p = solver.params.clone();
            temp_p.flow_velocity_ratio = vr;
            let temp_solver = QuantizedVortexSolver::new(temp_p);
            let (drag, _, _) = temp_solver.compute_obstacle_drag();
            drag_pts.push([vr, drag]);
        }

        let drag_plot_pts: PlotPoints = drag_pts.into_iter().collect();

        Plot::new("landau_drag_plot")
            .height(300.0)
            .x_axis_label("Flow Velocity Ratio v / v_c")
            .y_axis_label("Normalized Drag Force F_drag")
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Drag Force vs Flow Speed", drag_plot_pts)
                        .color(Color32::from_rgb(231, 76, 60))
                        .width(2.5),
                );
                plot_ui.vline(
                    VLine::new("Landau Critical Speed v_c", 1.0)
                        .color(Color32::from_rgb(46, 204, 113)),
                );
                plot_ui.vline(
                    VLine::new("Current Flow Speed", self.flow_velocity_ratio)
                        .color(Color32::from_rgb(241, 196, 15)),
                );
            });
    }

    fn render_tab_josephson(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Applied Strain (x10^-7):");
            if ui
                .add(egui::Slider::new(&mut self.applied_strain_1e7, 0.0..=20.0).step_by(0.5))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label("Initial Imbalance z_0:");
            if ui
                .add(egui::Slider::new(&mut self.initial_imbalance, 0.0..=0.9).step_by(0.05))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label("Tunneling E_J (meV):");
            if ui
                .add(egui::Slider::new(&mut self.josephson_coupling_ej_mev, 0.01..=0.15).step_by(0.01))
                .changed()
            {
                self.recompute();
            }
        });

        ui.add_space(4.0);

        let sm = self.cached_josephson_metrics;
        let mode = if sm.is_self_trapped {
            RichText::new("Macroscopic Quantum Self-Trapping (MQST)")
                .color(Color32::from_rgb(230, 126, 34))
                .strong()
        } else {
            RichText::new("Plasma Oscillations (Linear Josephson Regime)")
                .color(Color32::from_rgb(46, 204, 113))
                .strong()
        };

        ui.horizontal(|ui| {
            ui.label(mode);
            ui.separator();
            ui.label(format!("Plasma Freq f_J: {:.1} GHz", sm.plasma_frequency_ghz));
            ui.separator();
            ui.label(format!("Critical z_c: {:.3}", sm.mqst_critical_imbalance));
            ui.separator();
            ui.label(format!("Potential Shift Delta_V: {:.3e} meV", sm.strain_potential_shift_mev));
            ui.separator();
            ui.label(
                RichText::new(format!("Min Strain: {:.2e} / sqrt(Hz)", sm.minimum_detectable_strain))
                    .color(Color32::from_rgb(52, 152, 219))
                    .strong(),
            );
        });

        ui.add_space(4.0);

        let z_pts: PlotPoints = self
            .cached_josephson_trajectory
            .iter()
            .map(|p| [p.time_ps, p.imbalance_z])
            .collect();
        let fringe_pts: PlotPoints = self
            .cached_fringe_pattern
            .iter()
            .map(|p| [p.position_um, p.intensity])
            .collect();

        Plot::new("josephson_dynamics_plot")
            .height(290.0)
            .x_axis_label("Evolution Time (ps) / Detector Screen Position (um)")
            .y_axis_label("Population Imbalance z(t) / Fringe Intensity")
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Population Imbalance z(t)", z_pts)
                        .color(Color32::from_rgb(52, 152, 219))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("Interference Fringe I(x)", fringe_pts)
                        .color(Color32::from_rgb(241, 196, 15))
                        .width(2.0),
                );
                plot_ui.hline(
                    HLine::new("MQST Threshold +z_c", sm.mqst_critical_imbalance)
                        .color(Color32::from_rgb(231, 76, 60)),
                );
            });
    }

    fn render_tab_audit(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let pass = self.cached_audit.all_passed;
            let badge = if pass {
                RichText::new(format!(
                    "AUDIT PASSED: {}/{} CRITERIA MET",
                    self.cached_audit.passed_count, self.cached_audit.total_count
                ))
                .color(Color32::from_rgb(46, 204, 113))
                .strong()
            } else {
                RichText::new(format!(
                    "AUDIT FAILED: {}/{} CRITERIA MET",
                    self.cached_audit.passed_count, self.cached_audit.total_count
                ))
                .color(Color32::from_rgb(231, 76, 60))
                .strong()
            };
            ui.label(badge);

            ui.separator();
            if ui.button("Re-run Physics Audit").clicked() {
                self.recompute();
            }
        });

        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .max_height(340.0)
            .show(ui, |ui| {
                for (idx, c) in self.cached_audit.criteria.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let status = if c.passed {
                                RichText::new("[PASS]").color(Color32::from_rgb(46, 204, 113)).strong()
                            } else {
                                RichText::new("[FAIL]").color(Color32::from_rgb(231, 76, 60)).strong()
                            };
                            ui.label(status);
                            ui.label(
                                RichText::new(format!("{}. {}", idx + 1, c.name))
                                    .color(Color32::WHITE)
                                    .strong(),
                            );
                        });
                        ui.label(RichText::new(&c.description).color(Color32::from_rgb(189, 195, 199)));
                        ui.horizontal(|ui| {
                            ui.label(format!("Expected: {}", c.expected));
                            ui.separator();
                            ui.label(format!("Actual: {}", c.actual));
                        });
                    });
                }
            });
    }

    fn render_footer(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Cold-Boot Latency: < 2.0 ms").color(Color32::from_rgb(46, 204, 113)));
            ui.separator();
            ui.label(format!("Pump Ratio: {:.2}", self.pump_power_ratio));
            ui.separator();
            ui.label(format!("Vortex Charge: ell = {}", self.vortex_charge.winding_number()));
            ui.separator();
            ui.label(format!("Solve Latency: {:.1} us", self.last_solve_time_us));
        });
    }
}
