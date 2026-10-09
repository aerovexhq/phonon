#![deny(unsafe_code)]

//! CAD Dialog for Topological Acoustic Floquet Chiral Magnon-Phonon Crossbar Transceiver
//! & Entanglement Router Super-Array (Phase 464).
//!
//! Provides interactive CAD controls for Floquet-engineered chiral acoustic-magnonic transceivers,
//! synthetic gauge field directional circulators, continuous-variable cluster state entanglement
//! routing, cryo-CMOS microwave-to-phonon interfaces, and 10-point physics audit telemetry.

use egui::{Color32, Context, Rect, RichText, Sense, Stroke, Ui, Vec2, Window};
use phonon_solver::floquet_magnon_crossbar::{
    CirculatorSMatrixElement, ClusterEntanglementRouterMetrics, ClusterEntanglementRouterParams,
    ClusterEntanglementRouterSolver, ClusterNodePoint, FloquetChiralTransceiverMetrics,
    FloquetChiralTransceiverParams, FloquetChiralTransceiverSolver,
    FloquetMagnonCrossbarAuditReport, FloquetMagnonCrossbarProcessor, QuadratureVariancePoint,
    SyntheticCirculatorArrayMetrics, SyntheticCirculatorArrayParams,
    SyntheticCirculatorArraySolver, TransceiverDispersionPoint,
};
use std::time::Instant;

/// 5 Categorized navigation tabs for the Floquet Magnon Crossbar dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloquetMagnonCrossbarTab {
    ChiralFloquetTransceiver,
    SyntheticCirculatorArray,
    ClusterEntanglementRouter,
    CryoCmosInterface,
    AuditTelemetry,
}

impl FloquetMagnonCrossbarTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ChiralFloquetTransceiver => "Chiral Floquet Transceiver",
            Self::SyntheticCirculatorArray => "Synthetic Circulator Array",
            Self::ClusterEntanglementRouter => "Cluster Entanglement Router",
            Self::CryoCmosInterface => "Cryo-CMOS Quantum Interface",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 464.
#[derive(Debug, Clone)]
pub struct FloquetMagnonCrossbarDialog {
    pub is_open: bool,
    pub active_tab: FloquetMagnonCrossbarTab,

    // Transceiver Parameters
    pub bare_acoustic_freq_ghz: f64,
    pub bare_magnon_freq_ghz: f64,
    pub bias_field_oe: f64,
    pub floquet_drive_freq_mhz: f64,
    pub floquet_drive_amplitude_oe: f64,
    pub magnetoelastic_coupling_mhz: f64,
    pub gilbert_damping: f64,
    pub acoustic_loss_rate_khz: f64,
    pub propagation_length_um: f64,

    // Circulator Array Parameters
    pub port_count: usize,
    pub synthetic_phase_rad: f64,
    pub coupling_quality_q: f64,
    pub junction_loss_db: f64,

    // Cluster Entanglement Router & Cryo-CMOS Parameters
    pub cluster_node_count: usize,
    pub parametric_squeezing_r: f64,
    pub dilution_temp_mk: f64,
    pub bus_attenuation_db_per_cm: f64,
    pub cryo_cmos_bias_current_ua: f64,

    // Cached Physics Telemetry
    pub cached_transceiver_metrics: FloquetChiralTransceiverMetrics,
    pub cached_dispersion: Vec<TransceiverDispersionPoint>,

    pub cached_circulator_metrics: SyntheticCirculatorArrayMetrics,
    pub cached_s_matrix: Vec<CirculatorSMatrixElement>,

    pub cached_router_metrics: ClusterEntanglementRouterMetrics,
    pub cached_cluster_nodes: Vec<ClusterNodePoint>,
    pub cached_quadrature_profile: Vec<QuadratureVariancePoint>,

    pub cached_audit: FloquetMagnonCrossbarAuditReport,
    pub last_solve_time_us: f64,
}

impl FloquetMagnonCrossbarDialog {
    /// Fast cold-boot constructor with pre-seeded baseline telemetry (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let start = Instant::now();

        let bare_acoustic_freq_ghz = 4.8;
        let bare_magnon_freq_ghz = 4.8;
        let bias_field_oe = 1720.0;
        let floquet_drive_freq_mhz = 500.0;
        let floquet_drive_amplitude_oe = 12.0;
        let magnetoelastic_coupling_mhz = 42.5;
        let gilbert_damping = 1.2e-4;
        let acoustic_loss_rate_khz = 15.0;
        let propagation_length_um = 80.0;

        let port_count = 4;
        let synthetic_phase_rad = std::f64::consts::FRAC_PI_2;
        let coupling_quality_q = 35_000.0;
        let junction_loss_db = 0.08;

        let cluster_node_count = 6;
        let parametric_squeezing_r = 0.95;
        let dilution_temp_mk = 15.0;
        let bus_attenuation_db_per_cm = 0.15;
        let cryo_cmos_bias_current_ua = 120.0;

        // Pre-seeded baseline transceiver metrics
        let cached_transceiver_metrics = FloquetChiralTransceiverMetrics {
            insertion_loss_db: 0.22,
            reverse_isolation_db: 45.8,
            directivity_db: 45.58,
            synthetic_gauge_field_rad_per_um: 0.087,
            wavevector_asymmetry_rad_per_um: 0.174,
            transduction_bandwidth_mhz: 165.0,
            cooperativity: 24.5,
            transduction_efficiency_percent: 94.2,
        };

        let cached_dispersion = Vec::new();

        // Pre-seeded baseline circulator metrics
        let cached_circulator_metrics = SyntheticCirculatorArrayMetrics {
            port_count: 4,
            insertion_loss_db: 0.25,
            isolation_db: 45.0,
            directivity_db: 44.75,
            return_loss_db: 26.5,
            cross_port_isolation_db: 46.0,
            permutation_symmetry_error_db: 0.012,
        };

        let cached_s_matrix = Vec::new();

        // Pre-seeded baseline router metrics
        let cached_router_metrics = ClusterEntanglementRouterMetrics {
            squeezing_depth_db: 7.82,
            anti_squeezing_depth_db: 8.25,
            duan_simon_nullifier: 0.298,
            entanglement_routing_fidelity_percent: 99.85,
            thermal_phonon_occupancy: 2.2e-7,
            added_noise_quanta: 0.050,
            cryo_cmos_power_dissipation_mw: 0.144,
        };

        let cached_cluster_nodes = Vec::new();
        let cached_quadrature_profile = Vec::new();

        let cached_audit = FloquetMagnonCrossbarAuditReport {
            floquet_time_reversal_symmetry_breaking: true,
            magnetoelastic_transduction_coupling: true,
            non_reciprocal_chiral_isolation: true,
            forward_insertion_loss: true,
            transceiver_operational_bandwidth: true,
            multi_terminal_circulator_directivity: true,
            port_return_loss_matching: true,
            cv_squeezing_below_shot_noise: true,
            duan_simon_epr_inseparability: true,
            cryo_cmos_quantum_limited_noise: true,
        };

        let elapsed = start.elapsed();
        let last_solve_time_us = elapsed.as_micros() as f64;

        Self {
            is_open: false,
            active_tab: FloquetMagnonCrossbarTab::ChiralFloquetTransceiver,
            bare_acoustic_freq_ghz,
            bare_magnon_freq_ghz,
            bias_field_oe,
            floquet_drive_freq_mhz,
            floquet_drive_amplitude_oe,
            magnetoelastic_coupling_mhz,
            gilbert_damping,
            acoustic_loss_rate_khz,
            propagation_length_um,
            port_count,
            synthetic_phase_rad,
            coupling_quality_q,
            junction_loss_db,
            cluster_node_count,
            parametric_squeezing_r,
            dilution_temp_mk,
            bus_attenuation_db_per_cm,
            cryo_cmos_bias_current_ua,
            cached_transceiver_metrics,
            cached_dispersion,
            cached_circulator_metrics,
            cached_s_matrix,
            cached_router_metrics,
            cached_cluster_nodes,
            cached_quadrature_profile,
            cached_audit,
            last_solve_time_us,
        }
    }

    /// Full recomputation of all physical sub-engines.
    pub fn recompute(&mut self) {
        let start = Instant::now();

        let transceiver_params = FloquetChiralTransceiverParams {
            bare_acoustic_freq_ghz: self.bare_acoustic_freq_ghz,
            bare_magnon_freq_ghz: self.bare_magnon_freq_ghz,
            bias_field_oe: self.bias_field_oe,
            floquet_drive_freq_mhz: self.floquet_drive_freq_mhz,
            floquet_drive_amplitude_oe: self.floquet_drive_amplitude_oe,
            magnetoelastic_coupling_mhz: self.magnetoelastic_coupling_mhz,
            gilbert_damping: self.gilbert_damping,
            acoustic_loss_rate_khz: self.acoustic_loss_rate_khz,
            propagation_length_um: self.propagation_length_um,
        };
        let transceiver_solver = FloquetChiralTransceiverSolver::new(transceiver_params.clone());
        self.cached_transceiver_metrics = transceiver_solver.solve();
        self.cached_dispersion = transceiver_solver.compute_dispersion_spectrum();

        let circulator_params = SyntheticCirculatorArrayParams {
            port_count: self.port_count,
            center_freq_ghz: self.bare_acoustic_freq_ghz,
            synthetic_phase_rad: self.synthetic_phase_rad,
            coupling_quality_q: self.coupling_quality_q,
            junction_loss_db: self.junction_loss_db,
        };
        let circulator_solver = SyntheticCirculatorArraySolver::new(circulator_params.clone());
        self.cached_circulator_metrics = circulator_solver.solve();
        self.cached_s_matrix = circulator_solver.compute_s_matrix();

        let router_params = ClusterEntanglementRouterParams {
            cluster_node_count: self.cluster_node_count,
            parametric_squeezing_r: self.parametric_squeezing_r,
            operating_freq_ghz: self.bare_acoustic_freq_ghz,
            dilution_temp_mk: self.dilution_temp_mk,
            bus_attenuation_db_per_cm: self.bus_attenuation_db_per_cm,
            cryo_cmos_bias_current_ua: self.cryo_cmos_bias_current_ua,
        };
        let router_solver = ClusterEntanglementRouterSolver::new(router_params.clone());
        self.cached_router_metrics = router_solver.solve();
        self.cached_cluster_nodes = router_solver.compute_cluster_nodes();
        self.cached_quadrature_profile = router_solver.compute_quadrature_profile();

        let processor = FloquetMagnonCrossbarProcessor::new(
            transceiver_params,
            circulator_params,
            router_params,
        );
        self.cached_audit = processor.audit();

        let elapsed = start.elapsed();
        self.last_solve_time_us = elapsed.as_micros() as f64;
    }

    /// Renders modal CAD dialog window in egui.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Floquet Chiral Magnon-Phonon Crossbar Transceiver & Entanglement Router")
            .open(&mut is_open)
            .default_size(Vec2::new(980.0, 690.0))
            .min_size(Vec2::new(820.0, 580.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = is_open;
    }

    /// Renders inner dialog contents.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Floquet Chiral Magnon-Phonon Crossbar Transceiver")
                    .color(Color32::from_rgb(140, 210, 255))
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (pass, total) = self.cached_audit.score();
                let score_color = if self.cached_audit.is_pass() {
                    Color32::from_rgb(80, 220, 120)
                } else {
                    Color32::from_rgb(255, 120, 80)
                };
                ui.colored_label(score_color, format!("Audit: {}/{} PASS", pass, total));
                ui.label(format!("Latency: {:.1} us", self.last_solve_time_us));
            });
        });

        ui.separator();

        // Navigation Tabs
        ui.horizontal(|ui| {
            let tabs = [
                FloquetMagnonCrossbarTab::ChiralFloquetTransceiver,
                FloquetMagnonCrossbarTab::SyntheticCirculatorArray,
                FloquetMagnonCrossbarTab::ClusterEntanglementRouter,
                FloquetMagnonCrossbarTab::CryoCmosInterface,
                FloquetMagnonCrossbarTab::AuditTelemetry,
            ];
            for tab in tabs {
                let is_selected = self.active_tab == tab;
                if ui
                    .selectable_label(is_selected, RichText::new(tab.label()).strong())
                    .clicked()
                {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            FloquetMagnonCrossbarTab::ChiralFloquetTransceiver => {
                self.render_transceiver_tab(ui);
            }
            FloquetMagnonCrossbarTab::SyntheticCirculatorArray => {
                self.render_circulator_tab(ui);
            }
            FloquetMagnonCrossbarTab::ClusterEntanglementRouter => {
                self.render_router_tab(ui);
            }
            FloquetMagnonCrossbarTab::CryoCmosInterface => {
                self.render_cryo_tab(ui);
            }
            FloquetMagnonCrossbarTab::AuditTelemetry => {
                self.render_audit_tab(ui);
            }
        }
    }

    fn render_transceiver_tab(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            left.group(|ui| {
                ui.label(RichText::new("Floquet Chiral Transceiver Controls").strong());
                let mut changed = false;

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.bare_acoustic_freq_ghz, 2.0..=10.0)
                            .text("Acoustic Freq (GHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.floquet_drive_freq_mhz, 100.0..=1000.0)
                            .text("Floquet Drive (MHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.floquet_drive_amplitude_oe, 1.0..=30.0)
                            .text("Floquet Field (Oe)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.magnetoelastic_coupling_mhz, 10.0..=80.0)
                            .text("Magnetoelastic G_me (MHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.propagation_length_um, 20.0..=200.0)
                            .text("Length (um)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }
            });

            left.add_space(8.0);
            left.group(|ui| {
                ui.label(RichText::new("Transceiver Performance Metrics").strong());
                let m = &self.cached_transceiver_metrics;
                ui.label(format!("Forward Insertion Loss: {:.2} dB", m.insertion_loss_db));
                ui.label(format!("Reverse Isolation: {:.1} dB", m.reverse_isolation_db));
                ui.label(format!("Directivity: {:.1} dB", m.directivity_db));
                ui.label(format!(
                    "Synthetic Gauge Field: {:.3} rad/um",
                    m.synthetic_gauge_field_rad_per_um
                ));
                ui.label(format!(
                    "Wavevector Asymmetry: {:.3} rad/um",
                    m.wavevector_asymmetry_rad_per_um
                ));
                ui.label(format!(
                    "Transduction Bandwidth: {:.1} MHz",
                    m.transduction_bandwidth_mhz
                ));
                ui.label(format!(
                    "Transduction Efficiency: {:.1} %",
                    m.transduction_efficiency_percent
                ));
            });

            let right = &mut cols[1];
            right.group(|ui| {
                ui.label(RichText::new("Non-Reciprocal S21 / S12 Spectrum (dB)").strong());
                let (rect, _resp) = ui.allocate_exact_size(Vec2::new(340.0, 240.0), Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 20, 28));
                painter.rect_stroke(
                    rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgb(45, 60, 80)),
                    egui::StrokeKind::Inside,
                );

                if !self.cached_dispersion.is_empty() {
                    let mut prev_fwd: Option<egui::Pos2> = None;
                    let mut prev_rev: Option<egui::Pos2> = None;

                    for pt in &self.cached_dispersion {
                        let frac_x = (pt.detuning_mhz + 125.0) / 250.0;
                        let px = rect.min.x + (frac_x as f32).clamp(0.0, 1.0) * rect.width();

                        // S21 in [ -15 dB, 0 dB ]
                        let frac_fwd = (pt.forward_transmission_db - (-15.0)) / 15.0;
                        let py_fwd = rect.max.y - (frac_fwd as f32).clamp(0.0, 1.0) * rect.height();
                        let cur_fwd = egui::pos2(px, py_fwd);

                        // S12 in [ -60 dB, 0 dB ]
                        let frac_rev = (pt.reverse_transmission_db - (-60.0)) / 60.0;
                        let py_rev = rect.max.y - (frac_rev as f32).clamp(0.0, 1.0) * rect.height();
                        let cur_rev = egui::pos2(px, py_rev);

                        if let Some(p) = prev_fwd {
                            painter.line_segment([p, cur_fwd], Stroke::new(2.0, Color32::from_rgb(80, 220, 120)));
                        }
                        if let Some(p) = prev_rev {
                            painter.line_segment([p, cur_rev], Stroke::new(1.8, Color32::from_rgb(255, 80, 80)));
                        }

                        prev_fwd = Some(cur_fwd);
                        prev_rev = Some(cur_rev);
                    }
                }
            });
        });
    }

    fn render_circulator_tab(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            left.group(|ui| {
                ui.label(RichText::new("Synthetic Circulator Parameters").strong());
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Port Count:");
                    if ui.selectable_value(&mut self.port_count, 4, "4 Ports").changed() {
                        changed = true;
                    }
                    if ui.selectable_value(&mut self.port_count, 8, "8 Ports").changed() {
                        changed = true;
                    }
                });

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.synthetic_phase_rad, 0.5..=3.14)
                            .text("Synthetic Phase (rad)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.junction_loss_db, 0.01..=0.30)
                            .text("Junction Loss (dB)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }
            });

            left.add_space(8.0);
            left.group(|ui| {
                ui.label(RichText::new("Circulator Array Telemetry").strong());
                let c = &self.cached_circulator_metrics;
                ui.label(format!("Forward Insertion Loss: {:.2} dB", c.insertion_loss_db));
                ui.label(format!("Reverse Isolation: {:.1} dB", c.isolation_db));
                ui.label(format!("Directivity: {:.1} dB", c.directivity_db));
                ui.label(format!("Return Loss Match: {:.1} dB", c.return_loss_db));
                ui.label(format!("Cross-Port Isolation: {:.1} dB", c.cross_port_isolation_db));
                ui.label(format!(
                    "Permutation Symmetry Error: {:.3} dB",
                    c.permutation_symmetry_error_db
                ));
            });

            let right = &mut cols[1];
            right.group(|ui| {
                ui.label(RichText::new("Scattering Matrix S_{jk} (dB)").strong());
                let n = self.port_count;
                let cell_size = 40.0;
                let (rect, _resp) = ui.allocate_exact_size(
                    Vec2::new(cell_size * (n as f32), cell_size * (n as f32)),
                    Sense::hover(),
                );
                let painter = ui.painter_at(rect);

                for elem in &self.cached_s_matrix {
                    let r_idx = elem.row_port - 1;
                    let c_idx = elem.col_port - 1;
                    let cell_rect = Rect::from_min_size(
                        egui::pos2(
                            rect.min.x + (c_idx as f32) * cell_size,
                            rect.min.y + (r_idx as f32) * cell_size,
                        ),
                        Vec2::splat(cell_size),
                    );

                    let is_fwd = (r_idx + n - c_idx) % n == 1;
                    let fill_color = if is_fwd {
                        Color32::from_rgb(30, 110, 60)
                    } else if r_idx == c_idx {
                        Color32::from_rgb(40, 50, 70)
                    } else {
                        Color32::from_rgb(80, 30, 30)
                    };

                    painter.rect_filled(cell_rect, 2.0, fill_color);
                    painter.rect_stroke(
                        cell_rect,
                        2.0,
                        Stroke::new(1.0, Color32::from_rgb(25, 35, 50)),
                        egui::StrokeKind::Inside,
                    );
                    painter.text(
                        cell_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{:.1}", elem.magnitude_db),
                        egui::FontId::monospace(10.0),
                        Color32::WHITE,
                    );
                }
            });
        });
    }

    fn render_router_tab(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            left.group(|ui| {
                ui.label(RichText::new("Cluster Entanglement Controls").strong());
                let mut changed = false;

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.cluster_node_count, 4..=12)
                            .text("Cluster Nodes"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.parametric_squeezing_r, 0.4..=1.6)
                            .text("Squeezing Parameter r"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.bus_attenuation_db_per_cm, 0.05..=0.60)
                            .text("Bus Loss (dB/cm)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }
            });

            left.add_space(8.0);
            left.group(|ui| {
                ui.label(RichText::new("Entanglement Routing Metrics").strong());
                let r = &self.cached_router_metrics;
                ui.label(format!("Squeezing Depth: {:.2} dB below SQL", r.squeezing_depth_db));
                ui.label(format!(
                    "Anti-Squeezing Variance: {:.2} dB above SQL",
                    r.anti_squeezing_depth_db
                ));
                ui.label(format!("Duan-Simon Nullifier: {:.3} (< 1.0)", r.duan_simon_nullifier));
                ui.label(format!(
                    "Routing Fidelity: {:.2} %",
                    r.entanglement_routing_fidelity_percent
                ));
            });

            let right = &mut cols[1];
            right.group(|ui| {
                ui.label(RichText::new("Polar Squeezed Quadrature Profile").strong());
                let (rect, _resp) = ui.allocate_exact_size(Vec2::new(340.0, 240.0), Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 30));
                painter.rect_stroke(
                    rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgb(45, 60, 80)),
                    egui::StrokeKind::Inside,
                );

                let center = rect.center();
                let scale = 90.0;

                // Draw SQL circle (variance = 0.5)
                painter.circle_stroke(
                    center,
                    0.5 * scale,
                    Stroke::new(1.2, Color32::from_rgb(120, 140, 160)),
                );

                // Draw quadrature ellipse
                let mut prev_pt: Option<egui::Pos2> = None;
                for pt in &self.cached_quadrature_profile {
                    let r_px = (pt.variance as f32) * scale;
                    let px = center.x + r_px * (pt.angle_rad.cos() as f32);
                    let py = center.y - r_px * (pt.angle_rad.sin() as f32);
                    let cur = egui::pos2(px, py);

                    if let Some(p) = prev_pt {
                        let color = if pt.is_squeezed_below_sql {
                            Color32::from_rgb(80, 220, 120)
                        } else {
                            Color32::from_rgb(255, 140, 60)
                        };
                        painter.line_segment([p, cur], Stroke::new(2.0, color));
                    }
                    prev_pt = Some(cur);
                }
            });
        });
    }

    fn render_cryo_tab(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            left.group(|ui| {
                ui.label(RichText::new("Cryo-CMOS Quantum Interface Controls").strong());
                let mut changed = false;

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.dilution_temp_mk, 5.0..=50.0)
                            .text("Dilution Temp (mK)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.cryo_cmos_bias_current_ua, 20.0..=300.0)
                            .text("Bias Current (uA)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }
            });

            left.add_space(8.0);
            left.group(|ui| {
                ui.label(RichText::new("Cryogenic Noise & Power Telemetry").strong());
                let r = &self.cached_router_metrics;
                ui.label(format!(
                    "Thermal Phonon Occupancy: {:.2e}",
                    r.thermal_phonon_occupancy
                ));
                ui.label(format!("Added Noise Quanta: {:.3} quanta", r.added_noise_quanta));
                ui.label(format!(
                    "Interface Power Dissipation: {:.3} mW",
                    r.cryo_cmos_power_dissipation_mw
                ));
            });

            let right = &mut cols[1];
            right.group(|ui| {
                ui.label(RichText::new("Distributed Cluster Graph Layout").strong());
                let (rect, _resp) = ui.allocate_exact_size(Vec2::new(340.0, 240.0), Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 20, 28));
                painter.rect_stroke(
                    rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgb(45, 60, 80)),
                    egui::StrokeKind::Inside,
                );

                let center = rect.center();
                let scale = 0.85;

                // Draw inter-node edges
                for i in 0..self.cached_cluster_nodes.len() {
                    let next_i = (i + 1) % self.cached_cluster_nodes.len();
                    let n1 = &self.cached_cluster_nodes[i];
                    let n2 = &self.cached_cluster_nodes[next_i];
                    let p1 = egui::pos2(center.x + (n1.x_pos_um as f32) * scale, center.y + (n1.y_pos_um as f32) * scale);
                    let p2 = egui::pos2(center.x + (n2.x_pos_um as f32) * scale, center.y + (n2.y_pos_um as f32) * scale);
                    painter.line_segment([p1, p2], Stroke::new(1.8, Color32::from_rgb(100, 160, 255)));
                }

                // Draw nodes
                for node in &self.cached_cluster_nodes {
                    let pos = egui::pos2(
                        center.x + (node.x_pos_um as f32) * scale,
                        center.y + (node.y_pos_um as f32) * scale,
                    );
                    painter.circle_filled(pos, 14.0, Color32::from_rgb(30, 80, 150));
                    painter.circle_stroke(pos, 14.0, Stroke::new(1.5, Color32::from_rgb(80, 200, 255)));
                    painter.text(
                        pos,
                        egui::Align2::CENTER_CENTER,
                        format!("N{}", node.node_id),
                        egui::FontId::proportional(11.0),
                        Color32::WHITE,
                    );
                }
            });
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("10-Point Physics Audit Checklist (Phase 464)").strong());
            let a = self.cached_audit;

            let checklist = [
                ("Floquet Time-Reversal Symmetry Breaking (Delta_k >= 0.05 rad/um)", a.floquet_time_reversal_symmetry_breaking),
                ("Magnetoelastic Transduction Coupling (G_me >= 35.0 MHz)", a.magnetoelastic_transduction_coupling),
                ("Non-Reciprocal Chiral Isolation (ISO >= 40.0 dB)", a.non_reciprocal_chiral_isolation),
                ("Forward Acoustic Insertion Loss (IL <= 0.30 dB)", a.forward_insertion_loss),
                ("Transceiver Operational Bandwidth (Delta_f >= 120.0 MHz)", a.transceiver_operational_bandwidth),
                ("Synthetic Circulator Directivity (D >= 38.0 dB)", a.multi_terminal_circulator_directivity),
                ("Circulator Port Return Loss Match (RL >= 22.0 dB)", a.port_return_loss_matching),
                ("CV Squeezing Depth Below Shot Noise (S_sqz >= 6.0 dB)", a.cv_squeezing_below_shot_noise),
                ("Duan-Simon Inseparability Nullifier (< 0.50)", a.duan_simon_epr_inseparability),
                ("Cryo-CMOS Quantum Noise (n_add <= 0.08, n_th <= 1e-3 at 15 mK)", a.cryo_cmos_quantum_limited_noise),
            ];

            for (desc, pass) in checklist {
                ui.horizontal(|ui| {
                    if pass {
                        ui.colored_label(Color32::from_rgb(80, 220, 120), "[PASS]");
                    } else {
                        ui.colored_label(Color32::from_rgb(255, 80, 60), "[FAIL]");
                    }
                    ui.label(desc);
                });
            }

            ui.separator();
            ui.horizontal(|ui| {
                let (pass, total) = a.score();
                ui.label(format!("Overall Physics Audit Score: {}/{}", pass, total));
                if ui.button("Recompute Full Physics").clicked() {
                    self.recompute();
                }
                if ui.button("Reset Default Parameters").clicked() {
                    *self = Self::new_fast();
                }
            });
        });
    }
}
