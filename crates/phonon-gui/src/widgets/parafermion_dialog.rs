#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 422: Phonon Studio Quantum Acoustic
//! Non-Abelian Parafermion Braiding & Fractional Chern Number Interconnect.
//!
//! Visualizes fractional Chern metamaterial lattices, Z_3 / Z_4 parafermionic zero modes,
//! non-Abelian Artin braiding trajectories, universal non-Clifford quantum qudit gate
//! synthesis, and cryogenic dispersive cavity readout of fractional topological charges.

use egui::{pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::parafermion_braiding::{
    FractionalChernNumber, FractionalReadoutParams, FractionalSpectrumPoint,
    ParafermionAuditReport, ParafermionBraidGate, ParafermionBraidingParams, ParafermionGateKind,
    ParafermionLatticeParams, ParafermionMode, ParafermionOrder, ParafermionParams,
    ParafermionProcessor,
};

/// Active tab within the Parafermion Braiding Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParafermionTab {
    DomainWallLattice,
    NonAbelianBraiding,
    UniversalGateSynthesis,
    FractionalChargeReadout,
    AuditTelemetry,
}

impl ParafermionTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::DomainWallLattice => "Domain Wall Lattice",
            Self::NonAbelianBraiding => "Non-Abelian Braiding",
            Self::UniversalGateSynthesis => "Universal Gate Synthesis",
            Self::FractionalChargeReadout => "Fractional Charge Readout",
            Self::AuditTelemetry => "Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 422.
pub struct ParafermionDialog {
    pub is_open: bool,
    pub active_tab: ParafermionTab,

    // Lattice Parameters
    pub selected_order: ParafermionOrder,
    pub selected_chern: FractionalChernNumber,
    pub topological_gap_mhz: f64,
    pub domain_coupling_mhz: f64,
    pub parafermion_count: usize,

    // Braiding Parameters
    pub step_duration_ns: f64,
    pub dephasing_time_us: f64,
    pub selected_gate: ParafermionGateKind,
    pub braid_step_progress: f64,

    // Readout Parameters
    pub cavity_frequency_ghz: f64,
    pub dispersive_shift_mhz: f64,
    pub cavity_linewidth_mhz: f64,
    pub integration_time_ns: f64,

    // Solver & Cached Results
    pub processor: ParafermionProcessor,
    pub cached_modes: Vec<ParafermionMode>,
    pub cached_compiled_gate: ParafermionBraidGate,
    pub cached_spectrum: Vec<FractionalSpectrumPoint>,
    pub cached_realspace: Vec<Vec<f64>>,
    pub cached_audit: ParafermionAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ParafermionDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ParafermionDialog {
    /// Fast cold-boot constructor with pre-seeded baseline data (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let params = ParafermionParams::default();
        let processor = ParafermionProcessor::new(params.clone());

        let cached_modes = processor.lattice.solve_parafermion_modes();
        let cached_compiled_gate = processor.braiding.compile_gate(ParafermionGateKind::NonCliffordT);
        let cached_spectrum = processor.readout.generate_transmission_spectrum(41);
        let cached_realspace = processor.lattice.generate_realspace_intensity();
        let cached_audit = processor.audit_parafermion_processor();

        Self {
            is_open: false,
            active_tab: ParafermionTab::DomainWallLattice,
            selected_order: params.lattice.order,
            selected_chern: params.lattice.chern_number,
            topological_gap_mhz: params.lattice.topological_gap_mhz,
            domain_coupling_mhz: params.lattice.domain_coupling_mhz,
            parafermion_count: params.lattice.parafermion_count,
            step_duration_ns: params.braiding.step_duration_ns,
            dephasing_time_us: params.braiding.dephasing_time_us,
            selected_gate: ParafermionGateKind::NonCliffordT,
            braid_step_progress: 0.0,
            cavity_frequency_ghz: params.readout.cavity_frequency_ghz,
            dispersive_shift_mhz: params.readout.dispersive_shift_mhz,
            cavity_linewidth_mhz: params.readout.cavity_linewidth_mhz,
            integration_time_ns: params.readout.integration_time_ns,
            processor,
            cached_modes,
            cached_compiled_gate,
            cached_spectrum,
            cached_realspace,
            cached_audit,
            last_solve_time_us: 110.0,
        }
    }

    /// Recomputes all physics solvers and refreshes caches.
    pub fn recompute(&mut self) {
        let start = std::time::Instant::now();

        let params = ParafermionParams {
            lattice: ParafermionLatticeParams {
                order: self.selected_order,
                chern_number: self.selected_chern,
                topological_gap_mhz: self.topological_gap_mhz,
                domain_coupling_mhz: self.domain_coupling_mhz,
                parafermion_count: self.parafermion_count,
                ..Default::default()
            },
            braiding: ParafermionBraidingParams {
                order: self.selected_order,
                step_duration_ns: self.step_duration_ns,
                protection_gap_mhz: self.topological_gap_mhz,
                dephasing_time_us: self.dephasing_time_us,
            },
            readout: FractionalReadoutParams {
                order: self.selected_order,
                cavity_frequency_ghz: self.cavity_frequency_ghz,
                dispersive_shift_mhz: self.dispersive_shift_mhz,
                cavity_linewidth_mhz: self.cavity_linewidth_mhz,
                integration_time_ns: self.integration_time_ns,
                ..Default::default()
            },
        };

        self.processor = ParafermionProcessor::new(params);
        self.cached_modes = self.processor.lattice.solve_parafermion_modes();
        self.cached_compiled_gate = self.processor.braiding.compile_gate(self.selected_gate);
        self.cached_spectrum = self.processor.readout.generate_transmission_spectrum(41);
        self.cached_realspace = self.processor.lattice.generate_realspace_intensity();
        self.cached_audit = self.processor.audit_parafermion_processor();

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Primary display method following CAD dialog conventions.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Quantum Parafermion Braiding & Fractional Interconnect")
            .open(&mut is_open)
            .default_size(vec2(960.0, 680.0))
            .min_size(vec2(820.0, 560.0))
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = is_open;
    }

    /// Alias for show, following CAD dialog conventions.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Public render method for direct UI integration and headless testing.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        // Navigation Bar
        ui.horizontal(|ui| {
            for tab in &[
                ParafermionTab::DomainWallLattice,
                ParafermionTab::NonAbelianBraiding,
                ParafermionTab::UniversalGateSynthesis,
                ParafermionTab::FractionalChargeReadout,
                ParafermionTab::AuditTelemetry,
            ] {
                let is_selected = self.active_tab == *tab;
                let text = if is_selected {
                    RichText::new(tab.label()).color(Color32::from_rgb(96, 165, 250)).strong()
                } else {
                    RichText::new(tab.label())
                };
                if ui.selectable_label(is_selected, text).clicked() {
                    self.active_tab = *tab;
                }
            }
        });
        ui.separator();

        // Preset Toolbar
        ui.horizontal(|ui| {
            ui.label(RichText::new("Presets:").color(Color32::from_rgb(148, 163, 184)));
            if ui.button("Z_3 Fibonacci Parafermion (C_f = 1/3)").clicked() {
                self.selected_order = ParafermionOrder::Z3;
                self.selected_chern = FractionalChernNumber::OneThird;
                self.topological_gap_mhz = 3.5;
                self.selected_gate = ParafermionGateKind::NonCliffordT;
                self.recompute();
            }
            if ui.button("Z_4 Topological Qudit Processor (C_f = 1/2)").clicked() {
                self.selected_order = ParafermionOrder::Z4;
                self.selected_chern = FractionalChernNumber::OneHalf;
                self.topological_gap_mhz = 4.0;
                self.selected_gate = ParafermionGateKind::Hadamard;
                self.recompute();
            }
            if ui.button("Universal Non-Clifford T_3 Gate").clicked() {
                self.selected_gate = ParafermionGateKind::NonCliffordT;
                self.step_duration_ns = 200.0;
                self.recompute();
            }
            if ui.button("Two-Qudit Controlled-SUM Entangler").clicked() {
                self.selected_gate = ParafermionGateKind::ControlledSum;
                self.recompute();
            }
        });
        ui.separator();

        match self.active_tab {
            ParafermionTab::DomainWallLattice => self.render_lattice_tab(ui),
            ParafermionTab::NonAbelianBraiding => self.render_braiding_tab(ui),
            ParafermionTab::UniversalGateSynthesis => self.render_gate_synthesis_tab(ui),
            ParafermionTab::FractionalChargeReadout => self.render_readout_tab(ui),
            ParafermionTab::AuditTelemetry => self.render_audit_tab(ui),
        }
    }

    fn render_lattice_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Controls column
            ui.vertical(|ui| {
                ui.set_width(280.0);
                ui.heading(RichText::new("Fractional Chern Metamaterial").color(Color32::from_rgb(226, 232, 240)));
                ui.label(RichText::new("2D topological acoustic lattice hosting Z_3 / Z_4 parafermion zero modes at domain walls.").color(Color32::from_rgb(148, 163, 184)));

                ui.add_space(8.0);
                ui.label(RichText::new("Parafermion Symmetry Order:").color(Color32::from_rgb(148, 163, 184)));
                let mut order_changed = false;
                if ui.selectable_value(&mut self.selected_order, ParafermionOrder::Z3, "Z_3 (Qutrits, Non-Clifford)").clicked() {
                    order_changed = true;
                }
                if ui.selectable_value(&mut self.selected_order, ParafermionOrder::Z4, "Z_4 (Four-State Qudits)").clicked() {
                    order_changed = true;
                }

                ui.add_space(8.0);
                let mut slider_changed = false;
                slider_changed |= ui.add(egui::Slider::new(&mut self.topological_gap_mhz, 1.0..=8.0).text("Topological Gap (MHz)")).changed();
                slider_changed |= ui.add(egui::Slider::new(&mut self.domain_coupling_mhz, 0.5..=3.0).text("Domain Coupling (MHz)")).changed();

                if order_changed || slider_changed {
                    self.recompute();
                }

                ui.add_space(12.0);
                ui.separator();
                let gap = self.processor.lattice.protection_gap_mhz();
                let xi = self.processor.lattice.localization_length_mm();
                ui.label(RichText::new(format!("Protection Gap Delta: {:.2} MHz", gap)).color(Color32::from_rgb(147, 197, 253)));
                ui.label(RichText::new(format!("Localization Length xi: {:.2} mm", xi)).color(Color32::from_rgb(52, 211, 153)));
                ui.label(RichText::new(format!("Fractional Chern: {}", self.selected_chern.label())).color(Color32::from_rgb(251, 191, 36)));
            });

            ui.separator();

            // Right column: Real-space lattice canvas
            ui.vertical(|ui| {
                ui.heading(RichText::new("Real-Space Domain Wall Zero Modes").color(Color32::from_rgb(226, 232, 240)));
                ui.label(RichText::new("Glowing localized energy peaks representing localized parafermion wave packets.").color(Color32::from_rgb(148, 163, 184)));

                let (response, painter) = ui.allocate_painter(vec2(340.0, 340.0), Sense::hover());
                let rect = response.rect;

                painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));
                painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), StrokeKind::Outside);

                let n = self.cached_realspace.len();
                if n > 0 {
                    let cell_w = rect.width() / n as f32;
                    let cell_h = rect.height() / n as f32;

                    for y in 0..n {
                        for x in 0..n {
                            let intensity = self.cached_realspace[y][x];
                            let c = (intensity * 255.0).clamp(0.0, 255.0) as u8;
                            let color = Color32::from_rgb(c, (c as f32 * 0.5) as u8, 255 - c);
                            let c_rect = Rect::from_min_size(
                                pos2(rect.min.x + x as f32 * cell_w, rect.min.y + y as f32 * cell_h),
                                vec2(cell_w, cell_h),
                            );
                            painter.rect_filled(c_rect, 0.0, color);
                        }
                    }
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    for mode in &self.cached_modes {
                        ui.label(RichText::new(format!(
                            "alpha_{}: q={:.2} ({:.1}%)",
                            mode.mode_id, mode.fractional_charge, mode.confinement_ratio * 100.0
                        )).color(Color32::from_rgb(96, 165, 250)));
                    }
                });
            });
        });
    }

    fn render_braiding_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("Non-Abelian Parafermion Braiding Dynamics").color(Color32::from_rgb(226, 232, 240)));
            ui.label(RichText::new("Topological exchange operations tau_jk in acoustic domain wall junctions.").color(Color32::from_rgb(148, 163, 184)));

            ui.add_space(8.0);
            ui.add(egui::Slider::new(&mut self.braid_step_progress, 0.0..=1.0).text("Braid Trajectory Stepper"));

            ui.add_space(12.0);
            let (artin_pass, diff) = self.processor.braiding.verify_artin_braid_relation();
            let artin_badge = if artin_pass { "[VERIFIED]" } else { "[VIOLATED]" };
            let artin_color = if artin_pass { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(239, 68, 68) };

            ui.horizontal(|ui| {
                ui.label(RichText::new("Artin Relation: tau_1 tau_2 tau_1 == tau_2 tau_1 tau_2:").color(Color32::from_rgb(226, 232, 240)));
                ui.label(RichText::new(artin_badge).color(artin_color).strong());
                ui.label(RichText::new(format!("(Residual norm: {:.2e})", diff)).color(Color32::from_rgb(148, 163, 184)));
            });

            ui.add_space(12.0);
            ui.label(RichText::new("Spacetime World-Line Braiding Trajectory:").color(Color32::from_rgb(226, 232, 240)));

            let (response, painter) = ui.allocate_painter(vec2(520.0, 180.0), Sense::hover());
            let rect = response.rect;
            painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));

            // Draw world-lines of 3 braided parafermions
            let colors = [
                Color32::from_rgb(96, 165, 250),
                Color32::from_rgb(52, 211, 153),
                Color32::from_rgb(251, 191, 36),
            ];

            let n_pts = 60;
            for i in 0..3 {
                let mut prev_pt = pos2(rect.min.x + 20.0, rect.min.y + 40.0 + i as f32 * 50.0);
                for p in 1..=n_pts {
                    let progress = p as f32 / n_pts as f32;
                    let x = rect.min.x + 20.0 + progress * (rect.width() - 40.0);

                    // Weave trajectory
                    let shift = if progress > 0.2 && progress < 0.8 {
                        let local_t = (progress - 0.2) / 0.6;
                        (local_t * std::f32::consts::PI * 2.0).sin() * 25.0
                    } else {
                        0.0
                    };

                    let sign = if i == 0 { 1.0 } else if i == 1 { -1.0 } else { 0.5 };
                    let y = rect.min.y + 40.0 + i as f32 * 50.0 + shift * sign;
                    let pt = pos2(x, y);

                    painter.line_segment([prev_pt, pt], Stroke::new(2.5, colors[i]));
                    prev_pt = pt;
                }
            }
        });
    }

    fn render_gate_synthesis_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Left column: Target gate selection
            ui.vertical(|ui| {
                ui.set_width(280.0);
                ui.heading(RichText::new("Target Gate Selector").color(Color32::from_rgb(226, 232, 240)));
                ui.label(RichText::new("Compiles target qudit gate into braid word sequences.").color(Color32::from_rgb(148, 163, 184)));

                ui.add_space(8.0);
                let mut changed = false;
                for gate in &[
                    ParafermionGateKind::Hadamard,
                    ParafermionGateKind::PhaseS,
                    ParafermionGateKind::NonCliffordT,
                    ParafermionGateKind::PauliX,
                    ParafermionGateKind::PauliZ,
                    ParafermionGateKind::ControlledSum,
                ] {
                    if ui.selectable_value(&mut self.selected_gate, *gate, gate.label()).clicked() {
                        changed = true;
                    }
                }

                if changed {
                    self.recompute();
                }

                ui.add_space(12.0);
                ui.separator();
                let gate = &self.cached_compiled_gate;
                ui.label(RichText::new(format!("Process Fidelity: {:.4}", gate.process_fidelity)).color(Color32::from_rgb(52, 211, 153)).strong());
                ui.label(RichText::new(format!("Diabatic Leakage: {:.2e}", gate.diabatic_leakage_error)).color(Color32::from_rgb(147, 197, 253)));
                ui.label(RichText::new(format!("Total Duration: {:.1} ns", gate.total_duration_ns)).color(Color32::from_rgb(251, 191, 36)));
                ui.label(RichText::new(format!("Braid Steps: {}", gate.braid_steps_count)).color(Color32::from_rgb(168, 85, 247)));
            });

            ui.separator();

            // Right column: Braid word and Unitary preview
            ui.vertical(|ui| {
                ui.heading(RichText::new("Compiled Braid Sequence").color(Color32::from_rgb(226, 232, 240)));
                ui.label(RichText::new(&self.cached_compiled_gate.braid_word).color(Color32::from_rgb(96, 165, 250)).monospace().strong());

                ui.add_space(12.0);
                ui.heading(RichText::new("Qudit Unitary Matrix Preview").color(Color32::from_rgb(226, 232, 240)));

                let u = self.processor.braiding.generate_unitary_preview(self.selected_gate);
                let dim = u.len();

                egui::Grid::new("unitary_matrix_grid").striped(true).spacing([12.0, 8.0]).show(ui, |ui| {
                    for row in 0..dim {
                        for col in 0..dim {
                            let (re, im) = u[row][col];
                            let text = format!("{:.2} + {:.2}i", re, im);
                            ui.label(RichText::new(text).color(Color32::from_rgb(203, 213, 225)).monospace());
                        }
                        ui.end_row();
                    }
                });
            });
        });
    }

    fn render_readout_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("Dispersive Fractional Charge Readout").color(Color32::from_rgb(226, 232, 240)));
            ui.label(RichText::new("Cryogenic microwave cavity transmission resolving the 3-fold fractional charge spectrum.").color(Color32::from_rgb(148, 163, 184)));

            ui.add_space(8.0);
            let pts0: PlotPoints = self.cached_spectrum.iter().map(|p| [p.detuning_mhz, p.s21_state0_db]).collect();
            let pts1: PlotPoints = self.cached_spectrum.iter().map(|p| [p.detuning_mhz, p.s21_state1_db]).collect();
            let pts2: PlotPoints = self.cached_spectrum.iter().map(|p| [p.detuning_mhz, p.s21_state2_db]).collect();

            let line0 = Line::new("State |0> (q = 0)", pts0).color(Color32::from_rgb(96, 165, 250)).width(2.0);
            let line1 = Line::new("State |1> (q = +1/3)", pts1).color(Color32::from_rgb(52, 211, 153)).width(2.0);
            let line2 = Line::new("State |2> (q = +2/3)", pts2).color(Color32::from_rgb(239, 68, 68)).width(2.0);

            Plot::new("fractional_readout_plot")
                .height(260.0)
                .x_axis_label("Detuning (MHz)")
                .y_axis_label("Transmission |S21|^2 (dB)")
                .show(ui, |plot_ui| {
                    plot_ui.line(line0);
                    plot_ui.line(line1);
                    plot_ui.line(line2);
                    plot_ui.hline(HLine::new("-3 dB Threshold", -3.0).color(Color32::from_rgb(100, 116, 139)));
                });

            ui.add_space(12.0);
            let snr = self.processor.readout.calculate_snr_db();
            let fid = self.processor.readout.calculate_readout_fidelity();

            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Dispersive SNR: {:.1} dB", snr)).color(Color32::from_rgb(52, 211, 153)).strong());
                ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));
                ui.label(RichText::new(format!("Readout Fidelity: {:.4}", fid)).color(Color32::from_rgb(147, 197, 253)).strong());
                ui.label(RichText::new("|").color(Color32::from_rgb(71, 85, 105)));
                ui.label(RichText::new(format!("Cavity Linewidth kappa: {:.2} MHz", self.cavity_linewidth_mhz)).color(Color32::from_rgb(251, 191, 36)));
            });
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("Physics Audit & Engineering Checklist").color(Color32::from_rgb(226, 232, 240)));
            ui.label(RichText::new("Automated verification of topological, braiding, and quantum qudit metrics.").color(Color32::from_rgb(148, 163, 184)));

            ui.add_space(8.0);
            let audit = &self.cached_audit;
            let score_text = format!("Physics Verification Score: {} / 10 PASS", audit.total_pass_score);
            let score_color = if audit.all_passed {
                Color32::from_rgb(52, 211, 153)
            } else {
                Color32::from_rgb(239, 68, 68)
            };
            ui.label(RichText::new(score_text).color(score_color).strong());

            ui.add_space(8.0);
            let items = [
                ("1. Fractional Protection Gap (Delta >= 2.5 MHz)", audit.fractional_gap_pass),
                ("2. Parafermion Confinement Ratio (>= 85%)", audit.parafermion_confinement_pass),
                ("3. Z_m Commutation Algebra Unitarity", audit.commutation_algebra_pass),
                ("4. Artin Non-Abelian Braid Relations", audit.artin_braid_relation_pass),
                ("5. Universal Non-Clifford T_3 Fidelity (>= 0.999)", audit.non_clifford_t_fidelity_pass),
                ("6. Generalized Hadamard H_3 Fidelity (>= 0.999)", audit.hadamard_h3_fidelity_pass),
                ("7. Diabatic Transition Leakage (< 1e-4)", audit.diabatic_leakage_suppression_pass),
                ("8. Dispersive Fractional Splitting (chi >= 3.0 MHz)", audit.dispersive_fractional_split_pass),
                ("9. Dispersive Readout SNR (>= 18.0 dB)", audit.fractional_readout_snr_pass),
                ("10. Single-Shot Readout Fidelity (>= 0.998)", audit.single_shot_fidelity_pass),
            ];

            for (desc, pass) in items {
                let (badge, color) = if pass {
                    ("[PASS]", Color32::from_rgb(52, 211, 153))
                } else {
                    ("[FAIL]", Color32::from_rgb(239, 68, 68))
                };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(badge).color(color).strong());
                    ui.label(RichText::new(desc).color(Color32::from_rgb(226, 232, 240)));
                });
            }

            ui.add_space(12.0);
            ui.label(RichText::new(format!("Last Solver Latency: {:.1} us", self.last_solve_time_us)).color(Color32::from_rgb(148, 163, 184)));
        });
    }
}
