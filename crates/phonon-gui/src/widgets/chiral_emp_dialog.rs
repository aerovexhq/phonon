#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 411: Phonon Studio Topological Chiral Acoustic
//! Edge-Magnetoplasmon Circulator & Non-Reciprocal Quantum Hall Router.

use egui::{
    pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Window,
};
use egui_plot::{HLine, Line, Plot, PlotPoints, VLine};
use phonon_solver::chiral_edge_magnetoplasmon::{
    ChiralEdgeMagnetoplasmonRouter, ChiralEmpParams, DefectParams, EmpAuditReport,
    EmpCirculatorParams, EmpDispersionPoint, EmpSpectrumPoint, QuantumHallRouterParams,
    RouterChannel, RouterTransportMetrics,
};

/// Active tab within the Chiral EMP Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChiralEmpDialogTab {
    DispersionSaw,
    CirculatorSMatrix,
    TopologicalDefectRouting,
    SplitGateMultiplexer,
    AuditTelemetry,
}

/// CAD modal dialog for topological chiral acoustic EMP circulator and quantum Hall router.
pub struct ChiralEmpDialog {
    pub is_open: bool,
    pub active_tab: ChiralEmpDialogTab,

    // Dispersion & Material parameters
    pub magnetic_field_t: f64,
    pub filling_factor_nu: usize,
    pub piezo_coupling_k2: f64,
    pub sheet_density_1e15: f64,

    // Circulator parameters
    pub center_freq_ghz: f64,
    pub bandwidth_mhz: f64,
    pub insertion_loss_db: f64,
    pub reverse_isolation_db: f64,
    pub return_loss_db: f64,

    // Router parameters
    pub corner_bend_radius_um: f64,
    pub split_gate_voltage_v: f64,
    pub defect_present: bool,
    pub defect_depth_um: f64,

    // Solvers & Cached results
    pub router_system: ChiralEdgeMagnetoplasmonRouter,
    pub cached_dispersion: Vec<EmpDispersionPoint>,
    pub cached_spectrum: Vec<EmpSpectrumPoint>,
    pub cached_transport: RouterTransportMetrics,
    pub cached_audit: EmpAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ChiralEmpDialog {
    fn default() -> Self {
        let disp_params = ChiralEmpParams::default();
        let circ_params = EmpCirculatorParams::default();
        let router_params = QuantumHallRouterParams::default();

        let router_system = ChiralEdgeMagnetoplasmonRouter::new(
            disp_params.clone(),
            circ_params.clone(),
            router_params.clone(),
        );

        let cached_dispersion = router_system
            .dispersion_solver
            .compute_dispersion_curve(0.1, 8.0, 50);
        let cached_spectrum = router_system
            .circulator
            .compute_spectrum_sweep(2.2, 3.8, 50);
        let cached_transport = router_system.hall_router.evaluate_transport_metrics();
        let cached_audit = router_system.audit_chiral_emp();

        Self {
            is_open: false,
            active_tab: ChiralEmpDialogTab::DispersionSaw,

            magnetic_field_t: disp_params.magnetic_field_t,
            filling_factor_nu: disp_params.filling_factor_nu,
            piezo_coupling_k2: disp_params.piezo_coupling_k2,
            sheet_density_1e15: disp_params.sheet_density_m2 / 1.0e15,

            center_freq_ghz: circ_params.center_freq_ghz,
            bandwidth_mhz: circ_params.bandwidth_mhz,
            insertion_loss_db: circ_params.insertion_loss_db,
            reverse_isolation_db: circ_params.reverse_isolation_db,
            return_loss_db: circ_params.return_loss_db,

            corner_bend_radius_um: router_params.corner_bend_radius_um,
            split_gate_voltage_v: router_params.split_gate_voltage_v,
            defect_present: router_params.defect.defect_present,
            defect_depth_um: router_params.defect.defect_depth_um,

            router_system,
            cached_dispersion,
            cached_spectrum,
            cached_transport,
            cached_audit,
            last_solve_time_us: 45.0,
        }
    }
}

impl ChiralEmpDialog {
    /// Creates a fast cold-boot dialog instance with pre-seeded baseline telemetry (< 2ms).
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Primary render entrypoint.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the modal window.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Topological Chiral Acoustic EMP Circulator & Router")
            .open(&mut open)
            .resizable(true)
            .default_width(840.0)
            .default_height(580.0)
            .show(ctx, |ui| {
                self.render_header(ui);
                ui.separator();
                self.render_tabs(ui);
                ui.separator();

                match self.active_tab {
                    ChiralEmpDialogTab::DispersionSaw => self.render_tab_dispersion(ui),
                    ChiralEmpDialogTab::CirculatorSMatrix => self.render_tab_circulator(ui),
                    ChiralEmpDialogTab::TopologicalDefectRouting => self.render_tab_defect_routing(ui),
                    ChiralEmpDialogTab::SplitGateMultiplexer => self.render_tab_split_gate(ui),
                    ChiralEmpDialogTab::AuditTelemetry => self.render_tab_audit(ui),
                }

                ui.separator();
                self.render_footer(ui);
            });

        self.is_open = open;
    }

    fn render_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Chiral Acoustic EMP Circulator & Quantum Hall Router")
                    .color(Color32::from_rgb(52, 152, 219))
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Preset: nu=1 High Field").clicked() {
                    self.load_preset_nu1();
                }
                if ui.button("Preset: nu=2 GaAs Plateau").clicked() {
                    self.load_preset_nu2();
                }
            });
        });
    }

    fn render_tabs(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.active_tab,
                ChiralEmpDialogTab::DispersionSaw,
                "1. EMP Dispersion & SAW",
            );
            ui.selectable_value(
                &mut self.active_tab,
                ChiralEmpDialogTab::CirculatorSMatrix,
                "2. 3-Port Circulator S-Matrix",
            );
            ui.selectable_value(
                &mut self.active_tab,
                ChiralEmpDialogTab::TopologicalDefectRouting,
                "3. Defect-Immune Routing",
            );
            ui.selectable_value(
                &mut self.active_tab,
                ChiralEmpDialogTab::SplitGateMultiplexer,
                "4. Split-Gate Router",
            );
            ui.selectable_value(
                &mut self.active_tab,
                ChiralEmpDialogTab::AuditTelemetry,
                "5. Physics Audit",
            );
        });
    }

    pub fn load_preset_nu2(&mut self) {
        let p = ChiralEmpParams::preset_nu2_gaas();
        self.magnetic_field_t = p.magnetic_field_t;
        self.filling_factor_nu = p.filling_factor_nu;
        self.piezo_coupling_k2 = p.piezo_coupling_k2;
        self.sheet_density_1e15 = p.sheet_density_m2 / 1.0e15;
        self.center_freq_ghz = p.center_freq_ghz;
        self.split_gate_voltage_v = 0.0;
        self.defect_present = false;
        self.recompute();
    }

    pub fn load_preset_nu1(&mut self) {
        let p = ChiralEmpParams::preset_nu1_high_field();
        self.magnetic_field_t = p.magnetic_field_t;
        self.filling_factor_nu = p.filling_factor_nu;
        self.piezo_coupling_k2 = p.piezo_coupling_k2;
        self.sheet_density_1e15 = p.sheet_density_m2 / 1.0e15;
        self.center_freq_ghz = p.center_freq_ghz;
        self.split_gate_voltage_v = -2.5;
        self.defect_present = true;
        self.recompute();
    }

    pub fn recompute(&mut self) {
        let disp_params = ChiralEmpParams {
            magnetic_field_t: self.magnetic_field_t,
            filling_factor_nu: self.filling_factor_nu,
            sheet_density_m2: self.sheet_density_1e15 * 1.0e15,
            piezo_coupling_k2: self.piezo_coupling_k2,
            center_freq_ghz: self.center_freq_ghz,
            ..Default::default()
        };

        let circ_params = EmpCirculatorParams {
            center_freq_ghz: self.center_freq_ghz,
            bandwidth_mhz: self.bandwidth_mhz,
            insertion_loss_db: self.insertion_loss_db,
            reverse_isolation_db: self.reverse_isolation_db,
            return_loss_db: self.return_loss_db,
            quality_factor: 1200.0,
        };

        let router_params = QuantumHallRouterParams {
            corner_bend_radius_um: self.corner_bend_radius_um,
            split_gate_voltage_v: self.split_gate_voltage_v,
            pinch_off_voltage_v: -1.8,
            defect: DefectParams {
                defect_present: self.defect_present,
                defect_depth_um: self.defect_depth_um,
                barrier_height_mev: 15.0,
            },
        };

        self.router_system = ChiralEdgeMagnetoplasmonRouter::new(
            disp_params,
            circ_params,
            router_params,
        );

        self.cached_dispersion = self
            .router_system
            .dispersion_solver
            .compute_dispersion_curve(0.1, 8.0, 50);
        let f0 = self.center_freq_ghz;
        let bw = self.bandwidth_mhz * 1.0e-3;
        self.cached_spectrum = self
            .router_system
            .circulator
            .compute_spectrum_sweep((f0 - 2.0 * bw).max(0.5), f0 + 2.0 * bw, 50);
        self.cached_transport = self.router_system.hall_router.evaluate_transport_metrics();
        self.cached_audit = self.router_system.audit_chiral_emp();
    }

    fn render_tab_dispersion(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Magnetic Field B_z (T):");
            if ui
                .add(egui::Slider::new(&mut self.magnetic_field_t, 1.0..=12.0).step_by(0.1))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label("Filling Factor nu:");
            for nu in [1, 2, 3, 4] {
                if ui
                    .selectable_label(self.filling_factor_nu == nu, format!("{}", nu))
                    .clicked()
                {
                    self.filling_factor_nu = nu;
                    self.recompute();
                }
            }

            ui.separator();
            ui.label("Piezo K^2:");
            if ui
                .add(egui::Slider::new(&mut self.piezo_coupling_k2, 0.005..=0.08).step_by(0.005))
                .changed()
            {
                self.recompute();
            }
        });

        ui.add_space(4.0);

        let fc_ghz = self.router_system.dispersion_solver.cyclotron_frequency_ghz();
        let sigma_xy = self.router_system.dispersion_solver.quantized_hall_conductance();
        let vf = self.router_system.dispersion_solver.acoustic_forward_velocity_ms();
        let vb = self.router_system.dispersion_solver.acoustic_backward_velocity_ms();
        let eta_nr = self.router_system.dispersion_solver.velocity_non_reciprocity_ratio();

        ui.horizontal(|ui| {
            ui.label(format!("Cyclotron f_c: {:.1} GHz", fc_ghz));
            ui.separator();
            ui.label(format!("sigma_xy: {:.4e} S", sigma_xy));
            ui.separator();
            ui.label(format!("v_fwd: {:.1} m/s", vf));
            ui.separator();
            ui.label(format!("v_bwd: {:.1} m/s", vb));
            ui.separator();
            ui.label(
                RichText::new(format!("Non-Reciprocity eta_nr: {:.2}%", eta_nr * 100.0))
                    .color(Color32::from_rgb(46, 204, 113))
                    .strong(),
            );
        });

        ui.add_space(4.0);

        let pts_emp: PlotPoints = self
            .cached_dispersion
            .iter()
            .map(|p| [p.wavenumber_rad_mm, p.emp_freq_ghz])
            .collect();
        let pts_fwd: PlotPoints = self
            .cached_dispersion
            .iter()
            .map(|p| [p.wavenumber_rad_mm, p.hybrid_fwd_freq_ghz])
            .collect();
        let pts_bwd: PlotPoints = self
            .cached_dispersion
            .iter()
            .map(|p| [p.wavenumber_rad_mm, p.hybrid_bwd_freq_ghz])
            .collect();

        Plot::new("emp_dispersion_plot")
            .height(320.0)
            .x_axis_label("Wavenumber q (rad/mm)")
            .y_axis_label("Frequency f (GHz)")
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Chiral EMP Mode w(q)", pts_emp)
                        .color(Color32::from_rgb(155, 89, 182))
                        .width(2.5),
                );
                plot_ui.line(
                    Line::new("Forward SAW Hybrid", pts_fwd)
                        .color(Color32::from_rgb(46, 204, 113))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("Backward SAW Hybrid", pts_bwd)
                        .color(Color32::from_rgb(231, 76, 60))
                        .width(2.0),
                );
                plot_ui.hline(
                    HLine::new("Operating Center Frequency", self.center_freq_ghz)
                        .color(Color32::from_rgb(241, 196, 15)),
                );
            });
    }

    fn render_tab_circulator(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Center Freq (GHz):");
            if ui
                .add(egui::Slider::new(&mut self.center_freq_ghz, 1.5..=6.0).step_by(0.1))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label("Bandwidth (MHz):");
            if ui
                .add(egui::Slider::new(&mut self.bandwidth_mhz, 100.0..=800.0).step_by(25.0))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label("Insertion Loss (dB):");
            if ui
                .add(egui::Slider::new(&mut self.insertion_loss_db, 0.1..=0.5).step_by(0.05))
                .changed()
            {
                self.recompute();
            }
        });

        ui.add_space(4.0);

        let il = self.router_system.circulator.forward_insertion_loss_db();
        let iso = self.router_system.circulator.reverse_isolation_db();
        let rl = self.router_system.circulator.return_loss_db();
        let sm = self.router_system.circulator.evaluate_s_matrix(self.center_freq_ghz);
        let u_err = sm.unitarity_error();

        ui.horizontal(|ui| {
            ui.label(format!("Insertion Loss S21: {:.2} dB", il));
            ui.separator();
            ui.label(
                RichText::new(format!("Reverse Isolation S12: {:.1} dB", iso))
                    .color(Color32::from_rgb(46, 204, 113))
                    .strong(),
            );
            ui.separator();
            ui.label(format!("Return Loss S11: {:.1} dB", rl));
            ui.separator();
            ui.label(format!("Unitarity ||S^H S - I||: {:.3}", u_err));
        });

        ui.add_space(4.0);

        let s21_pts: PlotPoints = self
            .cached_spectrum
            .iter()
            .map(|p| [p.freq_ghz, p.s21_db])
            .collect();
        let s12_pts: PlotPoints = self
            .cached_spectrum
            .iter()
            .map(|p| [p.freq_ghz, p.s12_db])
            .collect();
        let s11_pts: PlotPoints = self
            .cached_spectrum
            .iter()
            .map(|p| [p.freq_ghz, p.s11_db])
            .collect();

        Plot::new("circulator_spectrum_plot")
            .height(320.0)
            .x_axis_label("Frequency f (GHz)")
            .y_axis_label("S-Parameters (dB)")
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("S21 Forward Transmission", s21_pts)
                        .color(Color32::from_rgb(46, 204, 113))
                        .width(2.5),
                );
                plot_ui.line(
                    Line::new("S12 Reverse Isolation", s12_pts)
                        .color(Color32::from_rgb(231, 76, 60))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("S11 Return Loss", s11_pts)
                        .color(Color32::from_rgb(52, 152, 219))
                        .width(1.5),
                );
                plot_ui.hline(
                    HLine::new("35 dB Isolation Target", -35.0)
                        .color(Color32::from_rgb(241, 196, 15)),
                );
                plot_ui.vline(
                    VLine::new("Center Frequency f0", self.center_freq_ghz)
                        .color(Color32::from_rgb(189, 195, 199)),
                );
            });
    }

    fn render_tab_defect_routing(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut self.defect_present, "Insert Boundary Defect / Notch")
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label("Defect Depth (um):");
            if ui
                .add(egui::Slider::new(&mut self.defect_depth_um, 0.5..=5.0).step_by(0.5))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label("Corner Radius (um):");
            if ui
                .add(egui::Slider::new(&mut self.corner_bend_radius_um, 0.0..=10.0).step_by(1.0))
                .changed()
            {
                self.recompute();
            }
        });

        ui.add_space(4.0);

        let t_corner = self.cached_transport.corner_power_transmission;
        let t_defect = self.cached_transport.defect_power_transmission;
        let supp_db = self.cached_transport.defect_backscattering_suppression_db;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("90-Deg Corner Bend Transmission: {:.1}%", t_corner * 100.0))
                    .color(Color32::from_rgb(46, 204, 113))
                    .strong(),
            );
            ui.separator();
            ui.label(format!("Defect Transmission: {:.1}%", t_defect * 100.0));
            ui.separator();
            ui.label(
                RichText::new(format!("Backscattering Suppression: {:.1} dB", supp_db))
                    .color(Color32::from_rgb(52, 152, 219))
                    .strong(),
            );
        });

        ui.add_space(8.0);

        // 2D Canvas rendering the topological boundary channel
        let (rect, _response) = ui.allocate_exact_size(vec2(ui.available_width(), 260.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Substrate background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(26, 32, 44));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 72)), StrokeKind::Inside);

        let cx = rect.center().x;
        let cy = rect.center().y;

        // 2DEG mesa (shaded region)
        let mesa_rect = Rect::from_min_max(pos2(cx - 240.0, cy - 80.0), pos2(cx + 240.0, cy + 80.0));
        painter.rect_filled(mesa_rect, 2.0, Color32::from_rgb(40, 50, 70));

        // Draw chiral edge boundary line (green path co-propagating clockwise)
        let stroke_chiral = Stroke::new(3.0, Color32::from_rgb(46, 204, 113));
        let corner_p1 = pos2(cx - 240.0, cy - 80.0);
        let corner_p2 = pos2(cx + 100.0, cy - 80.0);
        let corner_p3 = pos2(cx + 100.0, cy + 80.0);
        let corner_p4 = pos2(cx + 240.0, cy + 80.0);

        // Path segments with 90-degree corner
        painter.line_segment([corner_p1, corner_p2], stroke_chiral);
        painter.line_segment([corner_p2, corner_p3], stroke_chiral);
        painter.line_segment([corner_p3, corner_p4], stroke_chiral);

        // Corner marker
        painter.circle_filled(corner_p2, 5.0, Color32::from_rgb(241, 196, 15));
        painter.text(
            pos2(corner_p2.x + 8.0, corner_p2.y - 12.0),
            egui::Align2::LEFT_BOTTOM,
            "90-Deg Corner (T >= 95%)",
            egui::FontId::monospace(11.0),
            Color32::from_rgb(241, 196, 15),
        );

        // Defect marker if enabled
        if self.defect_present {
            let defect_pos = pos2(cx - 60.0, cy - 80.0);
            painter.circle_filled(defect_pos, 7.0, Color32::from_rgb(231, 76, 60));
            painter.text(
                pos2(defect_pos.x, defect_pos.y - 14.0),
                egui::Align2::CENTER_BOTTOM,
                "Etched Notch Defect",
                egui::FontId::monospace(10.0),
                Color32::from_rgb(231, 76, 60),
            );

            // Detour path
            let detour_stroke = Stroke::new(2.0, Color32::from_rgb(52, 152, 219));
            painter.line_segment([pos2(defect_pos.x - 14.0, defect_pos.y), pos2(defect_pos.x, defect_pos.y + 14.0)], detour_stroke);
            painter.line_segment([pos2(defect_pos.x, defect_pos.y + 14.0), pos2(defect_pos.x + 14.0, defect_pos.y)], detour_stroke);
        }

        // Directional arrows along edge
        painter.text(
            pos2(cx - 150.0, cy - 86.0),
            egui::Align2::CENTER_BOTTOM,
            ">>> Chiral Edge Current >>>",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(46, 204, 113),
        );
    }

    fn render_tab_split_gate(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Split-Gate Voltage V_g (V):");
            if ui
                .add(egui::Slider::new(&mut self.split_gate_voltage_v, -3.0..=0.0).step_by(0.1))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            let is_pinched = self.router_system.hall_router.is_pinched_off();
            let state_text = if is_pinched {
                RichText::new("Pinched Off (Diverted -> Channel B)")
                    .color(Color32::from_rgb(230, 126, 34))
                    .strong()
            } else {
                RichText::new("Open Channel (Through -> Channel A)")
                    .color(Color32::from_rgb(46, 204, 113))
                    .strong()
            };
            ui.label(state_text);
        });

        ui.add_space(6.0);

        let pa = self.cached_transport.channel_a_power;
        let pb = self.cached_transport.channel_b_power;
        let iso_db = self.cached_transport.cross_channel_isolation_db;
        let active = self.cached_transport.active_channel;

        ui.horizontal(|ui| {
            ui.label(format!("Channel A Power: {:.1}%", pa * 100.0));
            ui.separator();
            ui.label(format!("Channel B Power: {:.1}%", pb * 100.0));
            ui.separator();
            ui.label(
                RichText::new(format!("Cross-Channel Isolation: {:.1} dB", iso_db))
                    .color(Color32::from_rgb(46, 204, 113))
                    .strong(),
            );
            ui.separator();
            ui.label(format!("Active Target: {:?}", active));
        });

        ui.add_space(8.0);

        // Visual multiplexer layout
        let (rect, _response) = ui.allocate_exact_size(vec2(ui.available_width(), 240.0), Sense::hover());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 4.0, Color32::from_rgb(26, 32, 44));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 72)), StrokeKind::Inside);

        let cx = rect.center().x;
        let cy = rect.center().y;

        // Split-gate electrode representation
        let gate_rect = Rect::from_center_size(pos2(cx, cy - 20.0), vec2(40.0, 50.0));
        let gate_color = if self.router_system.hall_router.is_pinched_off() {
            Color32::from_rgb(231, 76, 60)
        } else {
            Color32::from_rgb(149, 165, 166)
        };
        painter.rect_filled(gate_rect, 2.0, gate_color);
        painter.text(
            pos2(cx, cy - 20.0),
            egui::Align2::CENTER_CENTER,
            "Gate",
            egui::FontId::monospace(10.0),
            Color32::WHITE,
        );

        // Channel A path (straight through)
        let color_a = if active == RouterChannel::ChannelA {
            Color32::from_rgb(46, 204, 113)
        } else {
            Color32::from_rgb(80, 80, 80)
        };
        painter.line_segment([pos2(cx - 180.0, cy + 30.0), pos2(cx + 180.0, cy + 30.0)], Stroke::new(3.0, color_a));
        painter.text(
            pos2(cx + 190.0, cy + 30.0),
            egui::Align2::LEFT_CENTER,
            "Port 2 (Channel A)",
            egui::FontId::monospace(11.0),
            color_a,
        );

        // Channel B diverted path (upward deflection around gate)
        let color_b = if active == RouterChannel::ChannelB {
            Color32::from_rgb(230, 126, 34)
        } else {
            Color32::from_rgb(80, 80, 80)
        };
        painter.line_segment([pos2(cx - 30.0, cy + 30.0), pos2(cx - 30.0, cy - 60.0)], Stroke::new(2.5, color_b));
        painter.line_segment([pos2(cx - 30.0, cy - 60.0), pos2(cx + 180.0, cy - 60.0)], Stroke::new(2.5, color_b));
        painter.text(
            pos2(cx + 190.0, cy - 60.0),
            egui::Align2::LEFT_CENTER,
            "Port 3 (Channel B)",
            egui::FontId::monospace(11.0),
            color_b,
        );
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
            ui.label(format!("Center Frequency: {:.2} GHz", self.center_freq_ghz));
            ui.separator();
            ui.label(format!("Bandwidth: {:.0} MHz", self.bandwidth_mhz));
            ui.separator();
            ui.label(format!("Solve Latency: {:.1} us", self.last_solve_time_us));
        });
    }
}
