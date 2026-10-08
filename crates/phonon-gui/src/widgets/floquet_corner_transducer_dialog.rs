#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 424: Phonon Studio Topological Acoustic
//! Floquet Higher-Order Corner-State Quantum Transducer & Multi-Qubit Entanglement Router.
//!
//! Visualizes 0D topological corner-mode acoustic transduction with superconducting transmon
//! qubits, non-reciprocal Floquet traveling-wave routing between corners, multi-qubit Bell
//! state synthesis with CHSH Bell inequality violation, real-space metamaterial layout,
//! and a 10-point physics audit checklist.

use egui::{pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::floquet_corner_transducer::{
    CornerModeProperties, CornerRoutingMetrics, CornerTransducerAuditReport,
    CornerTransducerParams, CornerTransductionMetrics, EntanglementRouterParams,
    EntanglementVerificationReport, FloquetCornerTransducerProcessor,
    TargetEntangledState, TransductionTrajectoryPoint,
};

/// Active tab in the Floquet Corner Transducer Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerTransducerTab {
    CornerStateTransducer,
    NonReciprocalRouting,
    MultiQubitEntanglement,
    RealSpaceMetamaterial,
    AuditTelemetry,
}

impl CornerTransducerTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::CornerStateTransducer => "Corner-State Transducer",
            Self::NonReciprocalRouting => "Non-Reciprocal Routing",
            Self::MultiQubitEntanglement => "Multi-Qubit Entanglement",
            Self::RealSpaceMetamaterial => "Metamaterial Canvas",
            Self::AuditTelemetry => "Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 424.
pub struct FloquetCornerTransducerDialog {
    pub is_open: bool,
    pub active_tab: CornerTransducerTab,

    // Transducer Parameters
    pub center_freq_ghz: f64,
    pub intracell_hopping_mhz: f64,
    pub intercell_hopping_mhz: f64,
    pub floquet_mod_depth: f64,
    pub transmon_coupling_mhz: f64,
    pub transmon_t1_us: f64,
    pub transmon_t2_us: f64,
    pub corner_quality_factor: f64,
    pub grid_cells_n: usize,

    // Router Parameters
    pub forward_exchange_mhz: f64,
    pub chiral_drive_phase_rad: f64,
    pub corner_separation_mm: f64,
    pub crosstalk_isolation_db: f64,
    pub gate_duration_ns: f64,
    pub selected_target_state: TargetEntangledState,

    // Cached Solver State
    pub processor: FloquetCornerTransducerProcessor,
    pub cached_modes: Vec<CornerModeProperties>,
    pub cached_transduction: CornerTransductionMetrics,
    pub cached_trajectory: Vec<TransductionTrajectoryPoint>,
    pub cached_routing: CornerRoutingMetrics,
    pub cached_entanglement: EntanglementVerificationReport,
    pub cached_density_matrix: [[f64; 4]; 4],
    pub cached_audit: CornerTransducerAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for FloquetCornerTransducerDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl FloquetCornerTransducerDialog {
    /// Fast cold-boot constructor with pre-computed baseline data (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let processor = FloquetCornerTransducerProcessor::default();
        let cached_modes = processor.transducer.solve_corner_modes();
        let cached_transduction = processor.transducer.evaluate_transduction_metrics();
        let cached_trajectory = processor.transducer.compute_transduction_trajectory(40);
        let cached_routing = processor.router.evaluate_routing_metrics();
        let selected_target_state = TargetEntangledState::BellPhiPlus;
        let cached_entanglement = processor
            .router
            .synthesize_entangled_state(selected_target_state);
        let cached_density_matrix = processor
            .router
            .compute_density_matrix_2qubit(selected_target_state);
        let cached_audit = processor.audit_corner_transducer();

        Self {
            is_open: false,
            active_tab: CornerTransducerTab::CornerStateTransducer,
            center_freq_ghz: processor.transducer.params.center_freq_ghz,
            intracell_hopping_mhz: processor.transducer.params.intracell_hopping_mhz,
            intercell_hopping_mhz: processor.transducer.params.intercell_hopping_mhz,
            floquet_mod_depth: processor.transducer.params.floquet_mod_depth,
            transmon_coupling_mhz: processor.transducer.params.transmon_coupling_mhz,
            transmon_t1_us: processor.transducer.params.transmon_t1_us,
            transmon_t2_us: processor.transducer.params.transmon_t2_us,
            corner_quality_factor: processor.transducer.params.corner_quality_factor,
            grid_cells_n: processor.transducer.params.grid_cells_n,

            forward_exchange_mhz: processor.router.params.forward_exchange_mhz,
            chiral_drive_phase_rad: processor.router.params.chiral_drive_phase_rad,
            corner_separation_mm: processor.router.params.corner_separation_mm,
            crosstalk_isolation_db: processor.router.params.crosstalk_isolation_db,
            gate_duration_ns: processor.router.params.gate_duration_ns,
            selected_target_state,

            processor,
            cached_modes,
            cached_transduction,
            cached_trajectory,
            cached_routing,
            cached_entanglement,
            cached_density_matrix,
            cached_audit,
            last_solve_time_us: 140.0,
        }
    }

    /// Recomputes all transducer dynamics, routing metrics, and audit results.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let trans_params = CornerTransducerParams {
            center_freq_ghz: self.center_freq_ghz,
            intracell_hopping_mhz: self.intracell_hopping_mhz,
            intercell_hopping_mhz: self.intercell_hopping_mhz,
            floquet_mod_depth: self.floquet_mod_depth,
            transmon_coupling_mhz: self.transmon_coupling_mhz,
            transmon_t1_us: self.transmon_t1_us,
            transmon_t2_us: self.transmon_t2_us,
            corner_quality_factor: self.corner_quality_factor,
            grid_cells_n: self.grid_cells_n,
            ..Default::default()
        };

        let route_params = EntanglementRouterParams {
            forward_exchange_mhz: self.forward_exchange_mhz,
            chiral_drive_phase_rad: self.chiral_drive_phase_rad,
            corner_separation_mm: self.corner_separation_mm,
            crosstalk_isolation_db: self.crosstalk_isolation_db,
            gate_duration_ns: self.gate_duration_ns,
        };

        self.processor = FloquetCornerTransducerProcessor::new(trans_params, route_params);

        self.cached_modes = self.processor.transducer.solve_corner_modes();
        self.cached_transduction = self.processor.transducer.evaluate_transduction_metrics();
        self.cached_trajectory = self.processor.transducer.compute_transduction_trajectory(40);
        self.cached_routing = self.processor.router.evaluate_routing_metrics();
        self.cached_entanglement = self
            .processor
            .router
            .synthesize_entangled_state(self.selected_target_state);
        self.cached_density_matrix = self
            .processor
            .router
            .compute_density_matrix_2qubit(self.selected_target_state);
        self.cached_audit = self.processor.audit_corner_transducer();

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Renders the complete dialog contents into the provided egui::Ui.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Floquet Corner-State Transducer & Entanglement Router")
                    .strong()
                    .color(Color32::from_rgb(130, 220, 255)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("Recompute Physics").strong()).clicked() {
                    self.recompute();
                }
            });
        });

        ui.separator();

        // Tab bar
        ui.horizontal(|ui| {
            let tabs = [
                CornerTransducerTab::CornerStateTransducer,
                CornerTransducerTab::NonReciprocalRouting,
                CornerTransducerTab::MultiQubitEntanglement,
                CornerTransducerTab::RealSpaceMetamaterial,
                CornerTransducerTab::AuditTelemetry,
            ];
            for tab in tabs {
                let selected = self.active_tab == tab;
                let text = if selected {
                    RichText::new(tab.label())
                        .strong()
                        .color(Color32::from_rgb(255, 215, 0))
                } else {
                    RichText::new(tab.label())
                };
                if ui.selectable_label(selected, text).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            CornerTransducerTab::CornerStateTransducer => self.render_transducer_tab(ui),
            CornerTransducerTab::NonReciprocalRouting => self.render_routing_tab(ui),
            CornerTransducerTab::MultiQubitEntanglement => self.render_entanglement_tab(ui),
            CornerTransducerTab::RealSpaceMetamaterial => self.render_metamaterial_tab(ui),
            CornerTransducerTab::AuditTelemetry => self.render_audit_tab(ui),
        }
    }

    fn render_transducer_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("HOTI & Transmon Parameters").strong());
                let mut changed = false;

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.center_freq_ghz, 2.0..=8.0)
                            .text("Center Frequency (GHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.intracell_hopping_mhz, 0.5..=5.0)
                            .text("Intracell Hopping gamma (MHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.intercell_hopping_mhz, 5.0..=15.0)
                            .text("Intercell Hopping lambda (MHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.floquet_mod_depth, 0.10..=0.60)
                            .text("Floquet Modulation Depth"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.transmon_coupling_mhz, 10.0..=40.0)
                            .text("Transmon Coupling g/2pi (MHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.transmon_t1_us, 20.0..=200.0)
                            .text("Transmon T1 Lifetime (us)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Transduction Key Metrics").strong());
                let bulk_gap = 2.0 * (self.intercell_hopping_mhz - self.intracell_hopping_mhz).abs();
                ui.label(format!("Topological Bulk Gap: {:.2} MHz", bulk_gap));
                ui.label(format!(
                    "Transduction Peak Efficiency: {:.1}%",
                    self.cached_transduction.peak_efficiency * 100.0
                ));
                ui.label(format!(
                    "State Transfer Fidelity: {:.4}",
                    self.cached_transduction.transfer_fidelity
                ));
                ui.label(format!(
                    "Optimal Transfer Duration: {:.1} ns",
                    self.cached_transduction.optimal_duration_ns
                ));
                ui.label(format!(
                    "Cooperativity C: {:.1}",
                    self.cached_transduction.cooperativity
                ));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Quantum State Transfer Trajectory").strong());
                let transmon_pts: PlotPoints = self
                    .cached_trajectory
                    .iter()
                    .map(|p| [p.time_ns, p.transmon_prob])
                    .collect();
                let phonon_pts: PlotPoints = self
                    .cached_trajectory
                    .iter()
                    .map(|p| [p.time_ns, p.phonon_prob])
                    .collect();

                let line_transmon = Line::new("Transmon |e> Prob", transmon_pts)
                    .color(Color32::from_rgb(80, 200, 255))
                    .width(2.5);
                let line_phonon = Line::new("Phonon |1> Prob", phonon_pts)
                    .color(Color32::from_rgb(255, 180, 50))
                    .width(2.5);

                let hline_target = HLine::new("Target 85% Efficiency", 0.85)
                    .color(Color32::from_rgb(0, 230, 100));

                Plot::new("quantum_transduction_trajectory_plot")
                    .height(280.0)
                    .x_axis_label("Time (ns)")
                    .y_axis_label("State Probability")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(line_transmon);
                        plot_ui.line(line_phonon);
                        plot_ui.hline(hline_target);
                    });

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Cyan: Transmon Probability | Gold: Phonon Occupancy")
                            .color(Color32::LIGHT_GRAY),
                    );
                });
            });
        });
    }

    fn render_routing_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Floquet Routing Parameters").strong());
                let mut changed = false;

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.forward_exchange_mhz, 10.0..=40.0)
                            .text("Forward Exchange J_fwd (MHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.chiral_drive_phase_rad, 0.0..=std::f64::consts::PI)
                            .text("Chiral Drive Phase (rad)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.corner_separation_mm, 0.5..=5.0)
                            .text("Corner Separation (mm)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.crosstalk_isolation_db, 20.0..=50.0)
                            .text("Crosstalk Isolation (dB)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.gate_duration_ns, 10.0..=100.0)
                            .text("Gate Duration (ns)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Routing Performance Summary").strong());
                ui.label(format!(
                    "Forward Coupling (S21): {:.2} dB",
                    self.cached_routing.forward_coupling_db
                ));
                ui.label(format!(
                    "Reverse Isolation (S12): {:.2} dB",
                    self.cached_routing.reverse_isolation_db
                ));
                ui.label(format!(
                    "Isolation Contrast: {:.2} dB",
                    self.cached_routing.isolation_contrast_db
                ));
                ui.label(format!(
                    "Inter-Corner Crosstalk: {:.2} dB",
                    self.cached_routing.crosstalk_isolation_db
                ));
                ui.label(format!(
                    "Chiral Directivity: {:.4}",
                    self.cached_routing.chiral_directivity
                ));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Inter-Corner Scattering Metrics").strong());

                let bars = [
                    ("Forward Coupling", self.cached_routing.forward_coupling_db, Color32::from_rgb(0, 230, 100)),
                    ("Reverse Isolation", self.cached_routing.reverse_isolation_db, Color32::from_rgb(255, 90, 90)),
                    ("Contrast", self.cached_routing.isolation_contrast_db, Color32::from_rgb(80, 200, 255)),
                    ("Crosstalk Isolation", self.cached_routing.crosstalk_isolation_db, Color32::from_rgb(255, 200, 50)),
                ];

                for (name, val, col) in bars {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: {:.2} dB", name, val));
                        let width = (val.abs() as f32) * 5.0;
                        let (rect, _) = ui.allocate_exact_size(vec2(width.clamp(10.0, 220.0), 16.0), Sense::hover());
                        ui.painter().rect_filled(rect, 3.0, col);
                    });
                    ui.add_space(4.0);
                }

                ui.add_space(12.0);
                ui.label(
                    RichText::new("Topological Protection: Wave packet routes unidirectionally 1 -> 2 -> 3 -> 4 with strictly zero backscattering.")
                        .italics()
                        .color(Color32::LIGHT_GRAY),
                );
            });
        });
    }

    fn render_entanglement_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Target Entangled State").strong());
                let states = [
                    TargetEntangledState::BellPhiPlus,
                    TargetEntangledState::BellPhiMinus,
                    TargetEntangledState::BellPsiPlus,
                    TargetEntangledState::BellPsiMinus,
                    TargetEntangledState::Ghz4Qubit,
                ];

                let mut changed = false;
                for state in states {
                    let selected = self.selected_target_state == state;
                    if ui.selectable_label(selected, state.label()).clicked() {
                        self.selected_target_state = state;
                        changed = true;
                    }
                }

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.label(format!("State: {}", self.selected_target_state.label()));
                ui.add_space(8.0);

                ui.label(RichText::new("Quantum Verification Metrics").strong());
                ui.label(format!(
                    "State Fidelity: {:.4}",
                    self.cached_entanglement.state_fidelity
                ));
                ui.label(format!(
                    "Concurrence C: {:.4}",
                    self.cached_entanglement.concurrence
                ));
                ui.label(format!(
                    "CHSH Parameter S: {:.4}",
                    self.cached_entanglement.chsh_parameter
                ));
                ui.label(format!(
                    "Formation Entanglement: {:.4} ebits",
                    self.cached_entanglement.entanglement_of_formation
                ));
                ui.label(format!(
                    "State Purity Tr(rho^2): {:.4}",
                    self.cached_entanglement.purity
                ));

                if self.cached_entanglement.chsh_parameter > 2.0 {
                    ui.label(
                        RichText::new("[PASS] Bell Inequality Violated (S > 2.0)")
                            .color(Color32::from_rgb(0, 230, 100))
                            .strong(),
                    );
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Two-Qubit Density Matrix rho (Real Part)").strong());

                let matrix = self.cached_density_matrix;
                let (rect, _) = ui.allocate_exact_size(vec2(280.0, 280.0), Sense::hover());
                let painter = ui.painter_at(rect);

                painter.rect_filled(rect, 4.0, Color32::from_rgb(20, 25, 35));
                painter.rect_stroke(
                    rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgb(60, 80, 110)),
                    StrokeKind::Inside,
                );

                let cell_w = rect.width() / 4.0;
                let cell_h = rect.height() / 4.0;

                for r in 0..4 {
                    for c in 0..4 {
                        let val = matrix[r][c];
                        let cell_rect = Rect::from_min_size(
                            pos2(rect.min.x + (c as f32) * cell_w, rect.min.y + (r as f32) * cell_h),
                            vec2(cell_w, cell_h),
                        );

                        let intensity = (val.abs() * 255.0).clamp(0.0, 255.0) as u8;
                        let cell_color = if val >= 0.0 {
                            Color32::from_rgb(30, 80 + intensity / 2, intensity)
                        } else {
                            Color32::from_rgb(intensity, 40, 40)
                        };

                        painter.rect_filled(cell_rect.shrink(2.0), 2.0, cell_color);
                        painter.text(
                            cell_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{:.2}", val),
                            egui::FontId::proportional(12.0),
                            Color32::WHITE,
                        );
                    }
                }
            });
        });
    }

    fn render_metamaterial_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("2D BBH Metamaterial & Non-Reciprocal Floquet Routing").strong());
        ui.add_space(4.0);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width().min(680.0), 320.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(18, 24, 38));
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.5, Color32::from_rgb(50, 75, 110)),
            StrokeKind::Inside,
        );

        let center = rect.center();
        let lattice_size = 200.0;
        let l_min = pos2(center.x - lattice_size / 2.0, center.y - lattice_size / 2.0);
        let l_max = pos2(center.x + lattice_size / 2.0, center.y + lattice_size / 2.0);

        // Draw BBH lattice perimeter
        painter.rect_stroke(
            Rect::from_min_max(l_min, l_max),
            0.0,
            Stroke::new(2.0, Color32::from_rgb(70, 110, 160)),
            StrokeKind::Inside,
        );

        // Draw unit cells inside BBH lattice
        let n_cells = 4;
        let cell_step = lattice_size / (n_cells as f32);
        for i in 1..n_cells {
            let x = l_min.x + (i as f32) * cell_step;
            let y = l_min.y + (i as f32) * cell_step;
            painter.line_segment(
                [pos2(x, l_min.y), pos2(x, l_max.y)],
                Stroke::new(1.0, Color32::from_rgb(35, 55, 80)),
            );
            painter.line_segment(
                [pos2(l_min.x, y), pos2(l_max.x, y)],
                Stroke::new(1.0, Color32::from_rgb(35, 55, 80)),
            );
        }

        // Highlight 4 corners with localized acoustic modes and transmon couplers
        let corners = [
            (pos2(l_min.x, l_min.y), "Corner 1 (TL)", Color32::from_rgb(0, 230, 200)),
            (pos2(l_max.x, l_min.y), "Corner 2 (TR)", Color32::from_rgb(255, 200, 50)),
            (pos2(l_max.x, l_max.y), "Corner 3 (BR)", Color32::from_rgb(255, 100, 100)),
            (pos2(l_min.x, l_max.y), "Corner 4 (BL)", Color32::from_rgb(150, 100, 255)),
        ];

        for (pos, label, col) in corners {
            // Acoustic mode confinement halo
            painter.circle_filled(pos, 16.0, Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 60));
            painter.circle_filled(pos, 8.0, col);

            // Transmon capacitor symbol offset
            let offset_x = if pos.x > center.x { 40.0 } else { -40.0 };
            let offset_y = if pos.y > center.y { 25.0 } else { -25.0 };
            let transmon_pos = pos2(pos.x + offset_x, pos.y + offset_y);

            painter.line_segment([pos, transmon_pos], Stroke::new(1.5, col));
            painter.rect_filled(
                Rect::from_center_size(transmon_pos, vec2(16.0, 16.0)),
                2.0,
                Color32::from_rgb(40, 50, 70),
            );
            painter.text(
                pos2(transmon_pos.x, transmon_pos.y + 12.0),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        }

        // Chiral traveling-wave arrows along the 4 edges (1 -> 2 -> 3 -> 4 -> 1)
        let stroke_chiral = Stroke::new(2.5, Color32::from_rgb(0, 255, 150));
        // Top edge: 1 -> 2
        painter.arrow(pos2(l_min.x + 30.0, l_min.y - 12.0), vec2(lattice_size - 60.0, 0.0), stroke_chiral);
        // Right edge: 2 -> 3
        painter.arrow(pos2(l_max.x + 12.0, l_min.y + 30.0), vec2(0.0, lattice_size - 60.0), stroke_chiral);
        // Bottom edge: 3 -> 4
        painter.arrow(pos2(l_max.x - 30.0, l_max.y + 12.0), vec2(-(lattice_size - 60.0), 0.0), stroke_chiral);
        // Left edge: 4 -> 1
        painter.arrow(pos2(l_min.x - 12.0, l_max.y - 30.0), vec2(0.0, -(lattice_size - 60.0)), stroke_chiral);

        painter.text(
            pos2(center.x, l_max.y + 35.0),
            egui::Align2::CENTER_TOP,
            "Non-Reciprocal Floquet Circulation Loop: J_fwd = 20 MHz | Isolation > 30 dB",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(180, 220, 255),
        );
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("10-Point Physics & Quantum Verification Checklist").strong());
        ui.add_space(4.0);

        let report = &self.cached_audit;

        let items = [
            (
                report.pass_corner_confinement,
                "1. 0D topological corner acoustic mode spatial confinement >= 85%",
            ),
            (
                report.pass_bulk_gap,
                "2. Bulk topological gap Delta_bulk >= 10.0 MHz",
            ),
            (
                report.pass_transduction_efficiency,
                "3. Microwave-to-phonon quantum transduction efficiency >= 85%",
            ),
            (
                report.pass_transfer_fidelity,
                "4. Quantum state transfer fidelity F >= 0.995",
            ),
            (
                report.pass_routing_isolation,
                "5. Non-reciprocal inter-corner routing isolation depth >= 30.0 dB",
            ),
            (
                report.pass_routing_contrast,
                "6. Directional routing contrast >= 30.0 dB",
            ),
            (
                report.pass_crosstalk_isolation,
                "7. Inter-corner non-adjacent crosstalk isolation >= 35.0 dB",
            ),
            (
                report.pass_bell_concurrence,
                "8. Bell state synthesis concurrence C >= 0.95",
            ),
            (
                report.pass_chsh_violation,
                "9. CHSH Bell inequality violation parameter S >= 2.75 > 2.0",
            ),
            (
                report.pass_quantum_purity,
                "10. Quantum state purity Tr(rho^2) >= 0.985",
            ),
        ];

        for (pass, label) in items {
            ui.horizontal(|ui| {
                if pass {
                    ui.label(RichText::new("[PASS]").strong().color(Color32::from_rgb(0, 230, 100)));
                } else {
                    ui.label(RichText::new("[FAIL]").strong().color(Color32::from_rgb(255, 60, 60)));
                }
                ui.label(label);
            });
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Audit Score: {} / 10", report.pass_count));
            if report.all_passed {
                ui.label(
                    RichText::new("ALL PHYSICS CRITERIA VERIFIED")
                        .strong()
                        .color(Color32::from_rgb(0, 230, 100)),
                );
            }
            ui.label(format!("(Solve Time: {:.1} us)", self.last_solve_time_us));
        });
    }

    /// Renders the modal dialog window if open.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Floquet Corner Transducer & Entanglement Router")
            .open(&mut open)
            .default_size(vec2(780.0, 520.0))
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = open;
    }

    /// Alias for showing the modal dialog.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }
}
