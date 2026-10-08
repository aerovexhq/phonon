#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 439:
//! Phonon Studio Topological Acoustic Second-Order Boundary-Mode Soliton Logic Gate & Majority Voter.
//!
//! Visualizes 1D non-linear envelope boundary solitons, non-linear collisional phase shifts,
//! all-acoustic 3-input majority voter logic gates, higher-order topological lattice modes,
//! and the 10-point physics audit checklist.

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::topological_soliton_logic::{
    BoundarySolitonMetrics, BoundarySolitonParams, CollisionMetrics, CollisionParams,
    CollisionTrajectoryPoint, GateWaveformPoint, MajorityVoterMetrics, MajorityVoterParams,
    SolitonGateMode, SolitonSpatialPoint, TopologicalSolitonAuditReport,
    TopologicalSolitonProcessor, TruthTableEntry,
};

/// Active tab in the Topological Soliton Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologicalSolitonTab {
    BoundarySoliton,
    CollisionalDynamics,
    MajorityVoter,
    TopologicalLattice,
    AuditTelemetry,
}

impl TopologicalSolitonTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BoundarySoliton => "Boundary Soliton Dynamics",
            Self::CollisionalDynamics => "Collisional Phase Shift",
            Self::MajorityVoter => "Majority Voter Logic",
            Self::TopologicalLattice => "HOTI Lattice & Bulk Gap",
            Self::AuditTelemetry => "Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 439.
pub struct TopologicalSolitonDialog {
    pub is_open: bool,
    pub active_tab: TopologicalSolitonTab,

    // Boundary Soliton Parameters
    pub intra_cell_coupling_mhz: f64,
    pub inter_cell_coupling_mhz: f64,
    pub dispersion_beta2: f64,
    pub kerr_gamma: f64,
    pub soliton_amplitude_eta: f64,
    pub soliton_velocity_um_ns: f64,
    pub sim_time_ns: f64,

    // Collision Parameters
    pub collision_eta1: f64,
    pub collision_eta2: f64,
    pub collision_v1: f64,
    pub collision_v2: f64,

    // Gate Parameters
    pub gate_mode: SolitonGateMode,
    pub input_a: bool,
    pub input_b: bool,
    pub input_c: bool,
    pub pulse_power_mw: f64,

    // Cached Solver State
    pub processor: TopologicalSolitonProcessor,
    pub cached_boundary_metrics: BoundarySolitonMetrics,
    pub cached_boundary_profile: Vec<SolitonSpatialPoint>,
    pub cached_collision_metrics: CollisionMetrics,
    pub cached_trajectory: Vec<CollisionTrajectoryPoint>,
    pub cached_gate_metrics: MajorityVoterMetrics,
    pub cached_truth_table: Vec<TruthTableEntry>,
    pub cached_gate_waveform: Vec<GateWaveformPoint>,
    pub cached_audit: TopologicalSolitonAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for TopologicalSolitonDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl TopologicalSolitonDialog {
    /// Fast cold-boot constructor with pre-computed baseline data (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let boundary_params = BoundarySolitonParams::default();
        let collision_params = CollisionParams::default();
        let gate_params = MajorityVoterParams::default();

        let processor = TopologicalSolitonProcessor::new(
            boundary_params.clone(),
            collision_params.clone(),
            gate_params.clone(),
        );

        let (cached_boundary_metrics, cached_boundary_profile) =
            processor.boundary_solver.solve_soliton(0.0);
        let (cached_collision_metrics, cached_trajectory) =
            processor.collision_solver.solve_collision();
        let (cached_gate_metrics, cached_truth_table, cached_gate_waveform) =
            processor.gate_solver.solve_gate();
        let cached_audit = processor.audit_system();

        Self {
            is_open: false,
            active_tab: TopologicalSolitonTab::BoundarySoliton,

            intra_cell_coupling_mhz: boundary_params.intra_cell_coupling_mhz,
            inter_cell_coupling_mhz: boundary_params.inter_cell_coupling_mhz,
            dispersion_beta2: boundary_params.dispersion_beta2,
            kerr_gamma: boundary_params.kerr_gamma,
            soliton_amplitude_eta: boundary_params.soliton_amplitude_eta,
            soliton_velocity_um_ns: boundary_params.soliton_velocity_um_ns,
            sim_time_ns: 0.0,

            collision_eta1: collision_params.amplitude_eta1,
            collision_eta2: collision_params.amplitude_eta2,
            collision_v1: collision_params.velocity1_um_ns,
            collision_v2: collision_params.velocity2_um_ns,

            gate_mode: gate_params.mode,
            input_a: gate_params.input_a,
            input_b: gate_params.input_b,
            input_c: gate_params.input_c,
            pulse_power_mw: gate_params.pulse_power_mw,

            processor,
            cached_boundary_metrics,
            cached_boundary_profile,
            cached_collision_metrics,
            cached_trajectory,
            cached_gate_metrics,
            cached_truth_table,
            cached_gate_waveform,
            cached_audit,
            last_solve_time_us: 145.0,
        }
    }

    /// Re-evaluates solvers with current parameter values.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let boundary_params = BoundarySolitonParams {
            intra_cell_coupling_mhz: self.intra_cell_coupling_mhz,
            inter_cell_coupling_mhz: self.inter_cell_coupling_mhz,
            dispersion_beta2: self.dispersion_beta2,
            kerr_gamma: self.kerr_gamma,
            soliton_amplitude_eta: self.soliton_amplitude_eta,
            soliton_velocity_um_ns: self.soliton_velocity_um_ns,
            ..Default::default()
        };

        let collision_params = CollisionParams {
            amplitude_eta1: self.collision_eta1,
            amplitude_eta2: self.collision_eta2,
            velocity1_um_ns: self.collision_v1,
            velocity2_um_ns: self.collision_v2,
            ..Default::default()
        };

        let gate_params = MajorityVoterParams {
            mode: self.gate_mode,
            input_a: self.input_a,
            input_b: self.input_b,
            input_c: self.input_c,
            pulse_power_mw: self.pulse_power_mw,
            ..Default::default()
        };

        self.processor = TopologicalSolitonProcessor::new(
            boundary_params,
            collision_params,
            gate_params,
        );

        let (bm, bp) = self.processor.boundary_solver.solve_soliton(self.sim_time_ns);
        let (cm, ct) = self.processor.collision_solver.solve_collision();
        let (gm, gt, gw) = self.processor.gate_solver.solve_gate();
        let audit = self.processor.audit_system();

        self.cached_boundary_metrics = bm;
        self.cached_boundary_profile = bp;
        self.cached_collision_metrics = cm;
        self.cached_trajectory = ct;
        self.cached_gate_metrics = gm;
        self.cached_truth_table = gt;
        self.cached_gate_waveform = gw;
        self.cached_audit = audit;

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Alias for showing the dialog in egui context.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the modal window within the egui application context.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Topological Acoustic Boundary Soliton Logic Gate & Majority Voter (Phase 439)")
            .open(&mut is_open)
            .default_width(960.0)
            .default_height(660.0)
            .show(ctx, |ui| {
                self.render_dialog_contents(ui);
            });

        self.is_open = is_open;
    }

    /// Renders the contents of the modal dialog into the given Ui.
    pub fn render_dialog_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for tab in [
                TopologicalSolitonTab::BoundarySoliton,
                TopologicalSolitonTab::CollisionalDynamics,
                TopologicalSolitonTab::MajorityVoter,
                TopologicalSolitonTab::TopologicalLattice,
                TopologicalSolitonTab::AuditTelemetry,
            ] {
                let selected = self.active_tab == tab;
                if ui.selectable_label(selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();

        match self.active_tab {
            TopologicalSolitonTab::BoundarySoliton => self.render_boundary_soliton_tab(ui),
            TopologicalSolitonTab::CollisionalDynamics => self.render_collisional_tab(ui),
            TopologicalSolitonTab::MajorityVoter => self.render_majority_voter_tab(ui),
            TopologicalSolitonTab::TopologicalLattice => self.render_lattice_tab(ui),
            TopologicalSolitonTab::AuditTelemetry => self.render_audit_tab(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_boundary_soliton_tab(&mut self, ui: &mut Ui) {
        ui.heading("1D Topological Boundary Envelope Soliton Dynamics");
        ui.label(
            "Visualizes bright envelope solitons localized along the 1D perimeter of a 2D higher-order \
            topological acoustic insulator (HOTI) with bulk quadrupole moment q_xy = 0.5.",
        );
        ui.add_space(8.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Time (ns):");
            changed |= ui.add(egui::Slider::new(&mut self.sim_time_ns, 0.0..=25.0).text("ns")).changed();
            ui.label("Amplitude (eta):");
            changed |= ui.add(egui::Slider::new(&mut self.soliton_amplitude_eta, 0.5..=2.5)).changed();
            ui.label("Velocity (um/ns):");
            changed |= ui.add(egui::Slider::new(&mut self.soliton_velocity_um_ns, 1.0..=6.0)).changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Plot spatial intensity |psi(x)|^2
        let points: PlotPoints = self
            .cached_boundary_profile
            .iter()
            .map(|p| [p.x_um, p.intensity])
            .collect();
        let line = Line::new("Soliton Intensity |psi(x)|^2", points)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.5);

        Plot::new("boundary_soliton_plot")
            .height(280.0)
            .x_axis_label("Waveguide Position x (um)")
            .y_axis_label("Power / Intensity (W)")
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });

        ui.horizontal(|ui| {
            ui.label(format!("Peak Power: {:.2} W", self.cached_boundary_metrics.peak_power_w));
            ui.separator();
            ui.label(format!("FWHM Width: {:.2} um", self.cached_boundary_metrics.fwhm_width_um));
            ui.separator();
            ui.label(format!("Edge Localization: {:.1}%", self.cached_boundary_metrics.edge_localization_ratio * 100.0));
            ui.separator();
            ui.label(format!("Dispersion Error: {:.3}", self.cached_boundary_metrics.dispersion_balance_error));
        });
    }

    fn render_collisional_tab(&mut self, ui: &mut Ui) {
        ui.heading("Non-Linear Soliton-Soliton Collisional Phase Shift");
        ui.label(
            "Models elastic collisions between two topological boundary solitons, verifying shape \
            conservation fidelity F_shape >= 0.950 and phase shift Delta theta.",
        );
        ui.add_space(8.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("v1 (um/ns):");
            changed |= ui.add(egui::Slider::new(&mut self.collision_v1, 0.5..=5.0)).changed();
            ui.label("v2 (um/ns):");
            changed |= ui.add(egui::Slider::new(&mut self.collision_v2, -5.0..=-0.5)).changed();
            ui.label("eta1:");
            changed |= ui.add(egui::Slider::new(&mut self.collision_eta1, 0.5..=2.0)).changed();
            ui.label("eta2:");
            changed |= ui.add(egui::Slider::new(&mut self.collision_eta2, 0.5..=2.0)).changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Plot trajectories x1(t) and x2(t)
        let pts1: PlotPoints = self.cached_trajectory.iter().map(|p| [p.time_ns, p.x1_pos_um]).collect();
        let pts2: PlotPoints = self.cached_trajectory.iter().map(|p| [p.time_ns, p.x2_pos_um]).collect();
        let line1 = Line::new("Soliton 1 Path (um)", pts1).color(Color32::from_rgb(74, 222, 128)).width(2.0);
        let line2 = Line::new("Soliton 2 Path (um)", pts2).color(Color32::from_rgb(248, 113, 113)).width(2.0);

        Plot::new("collision_traj_plot")
            .height(280.0)
            .x_axis_label("Time t (ns)")
            .y_axis_label("Position x (um)")
            .show(ui, |plot_ui| {
                plot_ui.line(line1);
                plot_ui.line(line2);
            });

        ui.horizontal(|ui| {
            ui.label(format!("Phase Shift: {:.3} rad ({:.1} deg)", self.cached_collision_metrics.phase_shift_rad, self.cached_collision_metrics.phase_shift_rad.to_degrees()));
            ui.separator();
            ui.label(format!("Residual: {:.4} rad", self.cached_collision_metrics.phase_shift_residual_rad));
            ui.separator();
            ui.label(format!("Shape Fidelity: {:.2}%", self.cached_collision_metrics.shape_conservation_fidelity * 100.0));
            ui.separator();
            ui.label(format!("Spatial Shift: {:.2} um", self.cached_collision_metrics.spatial_shift_um));
        });
    }

    fn render_majority_voter_tab(&mut self, ui: &mut Ui) {
        ui.heading("All-Acoustic 3-Input Majority Voter Logic Gate");
        ui.label(
            "Synthesizes Y = Maj(A, B, C) = (A and B) or (B and C) or (A and C) through constructive \
            and destructive wave interference at a topological multi-arm junction.",
        );
        ui.add_space(8.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Gate Mode:");
            for mode in [
                SolitonGateMode::Majority,
                SolitonGateMode::AndGate,
                SolitonGateMode::OrGate,
                SolitonGateMode::NandGate,
                SolitonGateMode::NorGate,
            ] {
                if ui.selectable_label(self.gate_mode == mode, mode.display_name()).clicked() {
                    self.gate_mode = mode;
                    changed = true;
                }
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Logic Inputs:");
            changed |= ui.checkbox(&mut self.input_a, "Input A").changed();
            changed |= ui.checkbox(&mut self.input_b, "Input B").changed();
            changed |= ui.checkbox(&mut self.input_c, "Input C").changed();
            ui.separator();
            ui.label("Pulse Power (mW):");
            changed |= ui.add(egui::Slider::new(&mut self.pulse_power_mw, 1.0..=10.0)).changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Output Status Banner
        ui.horizontal(|ui| {
            let out_color = if self.cached_gate_metrics.logic_output {
                Color32::from_rgb(74, 222, 128)
            } else {
                Color32::from_rgb(248, 113, 113)
            };
            ui.label(RichText::new(format!("EVALUATED OUTPUT: {}", if self.cached_gate_metrics.logic_output { "HIGH (1)" } else { "LOW (0)" })).strong().size(15.0).color(out_color));
            ui.separator();
            ui.label(format!("Output Power: {:.2} mW", self.cached_gate_metrics.output_power_mw));
            ui.separator();
            ui.label(format!("Contrast: {:.1} dB", self.cached_gate_metrics.contrast_ratio_db));
            ui.separator();
            ui.label(format!("Latency: {:.1} ns", self.cached_gate_metrics.propagation_delay_ns));
            ui.separator();
            ui.label(format!("Energy: {:.2} fJ", self.cached_gate_metrics.switching_energy_fj));
        });

        ui.add_space(8.0);

        // Output Pulse Waveform Plot
        let pts_out: PlotPoints = self.cached_gate_waveform.iter().map(|p| [p.time_ns, p.output_soliton_power_mw]).collect();
        let line_out = Line::new("Output Power Waveform (mW)", pts_out).color(Color32::from_rgb(168, 85, 247)).width(2.5);

        Plot::new("gate_wave_plot")
            .height(200.0)
            .x_axis_label("Time t (ns)")
            .y_axis_label("Pulse Power (mW)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_out);
                plot_ui.hline(HLine::new("Logic High Threshold", 2.0).color(Color32::from_rgb(100, 116, 139)));
            });

        // 8-State Truth Table Grid
        ui.collapsing("Full 8-State Truth Table Verification", |ui| {
            egui::Grid::new("truth_table_grid").striped(true).show(ui, |ui| {
                ui.strong("A");
                ui.strong("B");
                ui.strong("C");
                ui.strong("Expected");
                ui.strong("Evaluated");
                ui.strong("Power (mW)");
                ui.strong("Status");
                ui.end_row();

                for e in &self.cached_truth_table {
                    ui.label(if e.a { "1" } else { "0" });
                    ui.label(if e.b { "1" } else { "0" });
                    ui.label(if e.c { "1" } else { "0" });
                    ui.label(if e.expected_out { "1" } else { "0" });
                    ui.label(if e.evaluated_out { "1" } else { "0" });
                    ui.label(format!("{:.2}", e.output_power_mw));
                    if e.state_pass {
                        ui.label(RichText::new("PASS").color(Color32::from_rgb(74, 222, 128)));
                    } else {
                        ui.label(RichText::new("FAIL").color(Color32::from_rgb(248, 113, 113)));
                    }
                    ui.end_row();
                }
            });
        });
    }

    fn render_lattice_tab(&mut self, ui: &mut Ui) {
        ui.heading("Higher-Order Topological Insulator (HOTI) Lattice & Bulk Gap");
        ui.label(
            "Visualizes the 2D Benalcazar-Bernevig-Hughes (BBH) quadrupole acoustic lattice with \
            quantized quadrupole moment q_xy = 0.5 protecting gapless 1D boundary modes.",
        );
        ui.add_space(8.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Intracell gamma (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.intra_cell_coupling_mhz, 0.5..=10.0)).changed();
            ui.label("Intercell lambda (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.inter_cell_coupling_mhz, 0.5..=15.0)).changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(12.0);

        let q_xy = self.cached_boundary_metrics.bulk_quadrupole_moment;
        let is_topo = (q_xy - 0.5).abs() < 0.01;
        let phase_text = if is_topo {
            "Topological Higher-Order Phase (q_xy = 0.5)"
        } else {
            "Trivial Phase (q_xy = 0.0)"
        };
        let phase_color = if is_topo {
            Color32::from_rgb(74, 222, 128)
        } else {
            Color32::from_rgb(248, 113, 113)
        };

        ui.label(RichText::new(format!("LATTICE STATE: {}", phase_text)).strong().size(15.0).color(phase_color));
        ui.label(format!("Bulk Bandgap Delta_bulk: {:.2} MHz", self.cached_boundary_metrics.bulk_bandgap_mhz));
        ui.label(format!("Boundary Confinement: {:.1}%", self.cached_boundary_metrics.edge_localization_ratio * 100.0));

        ui.add_space(16.0);

        // 2D diagram of lattice unit cell
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width().min(400.0), 220.0), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), StrokeKind::Outside);

        // Draw 4 corners representing 4 sites of unit cell with pi-flux
        let c = rect.center();
        let r = 50.0;
        let p1 = pos2(c.x - r, c.y - r);
        let p2 = pos2(c.x + r, c.y - r);
        let p3 = pos2(c.x + r, c.y + r);
        let p4 = pos2(c.x - r, c.y + r);

        // Bonds
        let bond_stroke = Stroke::new(2.5, Color32::from_rgb(56, 189, 248));
        let neg_bond_stroke = Stroke::new(2.5, Color32::from_rgb(244, 63, 94)); // pi flux negative bond
        painter.line_segment([p1, p2], bond_stroke);
        painter.line_segment([p2, p3], bond_stroke);
        painter.line_segment([p3, p4], bond_stroke);
        painter.line_segment([p4, p1], neg_bond_stroke); // negative hopping

        // Site nodes
        for (p, label) in [(p1, "1"), (p2, "2"), (p3, "3"), (p4, "4")] {
            painter.circle_filled(p, 10.0, Color32::from_rgb(30, 58, 138));
            painter.circle_stroke(p, 10.0, Stroke::new(1.5, Color32::from_rgb(96, 165, 250)));
            painter.text(p, egui::Align2::CENTER_CENTER, label, egui::FontId::monospace(11.0), Color32::WHITE);
        }
        painter.text(c, egui::Align2::CENTER_CENTER, "pi-Flux", egui::FontId::proportional(13.0), Color32::from_rgb(226, 232, 240));
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.heading("10-Point Physics Audit & Telemetry");
        ui.label("Automated physics verification suite enforcing Phase 439 criteria:");
        ui.add_space(8.0);

        let report = &self.cached_audit;
        let pass_color = Color32::from_rgb(74, 222, 128);
        let fail_color = Color32::from_rgb(248, 113, 113);

        let items = [
            ("1. Bulk Quadrupole Quantization (q_xy = 0.5 +/- 0.01)", report.bulk_quadrupole_quantization_pass),
            ("2. 1D Boundary Soliton Localization (E_edge >= 85.0%)", report.edge_localization_pass),
            ("3. Non-Linear Dispersion Balance (|beta2*eta^2 - gamma*P| <= 0.05)", report.dispersion_balance_pass),
            ("4. Soliton Collision Shape Conservation (F_shape >= 0.950)", report.collision_shape_fidelity_pass),
            ("5. Collisional Phase Shift Predictability (Residual <= 0.08 rad)", report.phase_shift_predictability_pass),
            ("6. 3-Input Truth Table Completeness (8/8 States Verified)", report.truth_table_completeness_pass),
            ("7. All-Acoustic Logic Contrast Ratio (C_logic >= 20.0 dB)", report.logic_contrast_pass),
            ("8. Reconfigurable Gate Modes (AND, OR, Majority operational)", report.reconfigurable_modes_pass),
            ("9. Switching Energy Consumption (E_switch <= 2.0 fJ)", report.switching_energy_pass),
            ("10. Sub-30ns Propagation Latency (tau_prop <= 30.0 ns)", report.propagation_latency_pass),
        ];

        for (desc, pass) in items {
            ui.horizontal(|ui| {
                let badge = if pass { "PASS" } else { "FAIL" };
                let color = if pass { pass_color } else { fail_color };
                ui.label(RichText::new(format!("[{}]", badge)).strong().color(color));
                ui.label(desc);
            });
        }

        ui.add_space(12.0);
        let summary_text = format!("AUDIT SCORE: {} / 10 Criteria Passed", report.total_score);
        let summary_color = if report.all_passed { pass_color } else { fail_color };
        ui.label(RichText::new(summary_text).strong().size(16.0).color(summary_color));
    }

    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Solve Latency: {:.1} us", self.last_solve_time_us)).color(Color32::from_rgb(148, 163, 184)));
            ui.separator();
            ui.label(format!("q_xy: {:.2}", self.cached_boundary_metrics.bulk_quadrupole_moment));
            ui.separator();
            ui.label(format!("Bulk Gap: {:.1} MHz", self.cached_boundary_metrics.bulk_bandgap_mhz));
            ui.separator();
            ui.label(format!("Shape Fidelity: {:.1}%", self.cached_collision_metrics.shape_conservation_fidelity * 100.0));
            ui.separator();
            ui.label(format!("Logic Contrast: {:.1} dB", self.cached_gate_metrics.contrast_ratio_db));
            ui.separator();
            ui.label(format!("Energy: {:.2} fJ", self.cached_gate_metrics.switching_energy_fj));
        });
    }
}
