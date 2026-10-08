#![deny(unsafe_code)]

//! Phase 445: Quantum Metamaterial Non-Abelian Fractional Parafermion Surface-Code Lattice & Anyonic Braid Repeater Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring fractional parafermion zero-mode braiding,
//! Z_p qudit surface-code error correction, anyonic braid repeaters, and non-Clifford
//! magic-state distillation factories with cryogenic dispersive readout.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::fractional_parafermion_surface::{
    DistillationRoundPoint, FractionalParafermionAuditReport,
    FractionalParafermionBraidTrajectoryPoint, FractionalParafermionLatticeMetrics,
    FractionalParafermionLatticeParams, FractionalParafermionLatticeSolver,
    FractionalParafermionOrder, FractionalParafermionProcessor,
    FractionalParafermionWavepacketPoint, ParafermionStabilizerKind,
    QuditCavitySpectrumPoint, QuditDistillationMetrics, QuditDistillationParams,
    QuditDistillationSolver, SurfaceCodeRepeaterMetrics, SurfaceCodeRepeaterParams,
    SurfaceCodeRepeaterSolver, SurfaceLatticeNode, ThresholdScalingPoint,
};

/// 5 Categorized navigation tabs for the fractional parafermion dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParafermionDialogTab {
    ParafermionBraidingLattice,
    SurfaceCodeSyndromes,
    AnyonicBraidRepeater,
    QuditDistillationFactory,
    AuditTelemetry,
}

impl ParafermionDialogTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ParafermionBraidingLattice => "Parafermion Braiding Lattice",
            Self::SurfaceCodeSyndromes => "Surface Code Syndromes",
            Self::AnyonicBraidRepeater => "Anyonic Braid Repeater",
            Self::QuditDistillationFactory => "Qudit Distillation Factory",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Fractional Parafermion Surface-Code Lattice & Repeater (Phase 445).
#[derive(Debug, Clone)]
pub struct FractionalParafermionDialog {
    pub is_open: bool,
    pub active_tab: ParafermionDialogTab,

    // Lattice parameters
    pub order: FractionalParafermionOrder,
    pub superconducting_gap_mhz: f64,
    pub fqh_bulk_gap_mhz: f64,
    pub junction_width_nm: f64,
    pub braid_duration_ns: f64,

    // Repeater parameters
    pub code_distance: usize,
    pub physical_error_rate: f64,
    pub link_distance_um: f64,
    pub repeater_node_count: usize,
    pub waveguide_loss_db_mm: f64,

    // Distillation parameters
    pub raw_magic_error_rate: f64,
    pub dispersive_shift_chi_mhz: f64,
    pub cavity_linewidth_kappa_mhz: f64,
    pub readout_integration_time_ns: f64,

    // Cached telemetry and datasets
    pub cached_lattice_metrics: FractionalParafermionLatticeMetrics,
    pub cached_wavepackets: Vec<FractionalParafermionWavepacketPoint>,
    pub cached_braid_trajectory: Vec<FractionalParafermionBraidTrajectoryPoint>,

    pub cached_repeater_metrics: SurfaceCodeRepeaterMetrics,
    pub cached_lattice_nodes: Vec<SurfaceLatticeNode>,
    pub cached_scaling_curve: Vec<ThresholdScalingPoint>,

    pub cached_distillation_metrics: QuditDistillationMetrics,
    pub cached_distillation_rounds: Vec<DistillationRoundPoint>,
    pub cached_cavity_spectrum: Vec<QuditCavitySpectrumPoint>,

    pub cached_audit: FractionalParafermionAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for FractionalParafermionDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl FractionalParafermionDialog {
    /// Instantaneous cold-boot constructor (< 2ms) with pre-seeded baseline state.
    pub fn new_fast() -> Self {
        let lat_params = FractionalParafermionLatticeParams::default();
        let rep_params = SurfaceCodeRepeaterParams::default();
        let dist_params = QuditDistillationParams::default();

        let l_solver = FractionalParafermionLatticeSolver::new(lat_params.clone());
        let r_solver = SurfaceCodeRepeaterSolver::new(rep_params.clone());
        let d_solver = QuditDistillationSolver::new(dist_params.clone());

        let cached_lattice_metrics = l_solver.evaluate_metrics();
        let cached_wavepackets = l_solver.compute_wavepackets(40);
        let cached_braid_trajectory = l_solver.compute_braid_trajectory(40);

        let cached_repeater_metrics = r_solver.evaluate_metrics();
        let cached_lattice_nodes = r_solver.generate_lattice_nodes();
        let cached_scaling_curve = r_solver.compute_threshold_scaling(24);

        let cached_distillation_metrics = d_solver.evaluate_metrics();
        let cached_distillation_rounds = d_solver.compute_distillation_rounds();
        let cached_cavity_spectrum = d_solver.compute_cavity_spectrum(32);

        let processor = FractionalParafermionProcessor::new(
            lat_params.clone(),
            rep_params.clone(),
            dist_params.clone(),
        );
        let cached_audit = processor.audit_system();

        Self {
            is_open: false,
            active_tab: ParafermionDialogTab::ParafermionBraidingLattice,

            order: lat_params.order,
            superconducting_gap_mhz: lat_params.superconducting_gap_mhz,
            fqh_bulk_gap_mhz: lat_params.fqh_bulk_gap_mhz,
            junction_width_nm: lat_params.junction_width_nm,
            braid_duration_ns: lat_params.braid_duration_ns,

            code_distance: rep_params.code_distance,
            physical_error_rate: rep_params.physical_error_rate,
            link_distance_um: rep_params.link_distance_um,
            repeater_node_count: rep_params.repeater_node_count,
            waveguide_loss_db_mm: rep_params.waveguide_loss_db_mm,

            raw_magic_error_rate: dist_params.raw_magic_error_rate,
            dispersive_shift_chi_mhz: dist_params.dispersive_shift_chi_mhz,
            cavity_linewidth_kappa_mhz: dist_params.cavity_linewidth_kappa_mhz,
            readout_integration_time_ns: dist_params.readout_integration_time_ns,

            cached_lattice_metrics,
            cached_wavepackets,
            cached_braid_trajectory,

            cached_repeater_metrics,
            cached_lattice_nodes,
            cached_scaling_curve,

            cached_distillation_metrics,
            cached_distillation_rounds,
            cached_cavity_spectrum,

            cached_audit,
            last_solve_time_us: 125.0,
        }
    }

    /// Recomputes all physics solvers with current parameter settings.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let lat_params = FractionalParafermionLatticeParams {
            order: self.order,
            superconducting_gap_mhz: self.superconducting_gap_mhz,
            fqh_bulk_gap_mhz: self.fqh_bulk_gap_mhz,
            junction_width_nm: self.junction_width_nm,
            mode_count: 6,
            braid_duration_ns: self.braid_duration_ns,
            operating_temp_k: 0.015,
        };

        let rep_params = SurfaceCodeRepeaterParams {
            code_distance: self.code_distance,
            physical_error_rate: self.physical_error_rate,
            link_distance_um: self.link_distance_um,
            repeater_node_count: self.repeater_node_count,
            waveguide_loss_db_mm: self.waveguide_loss_db_mm,
            statistical_order: self.order.p(),
        };

        let dist_params = QuditDistillationParams {
            raw_magic_error_rate: self.raw_magic_error_rate,
            distillation_rounds: 2,
            dispersive_shift_chi_mhz: self.dispersive_shift_chi_mhz,
            cavity_linewidth_kappa_mhz: self.cavity_linewidth_kappa_mhz,
            readout_integration_time_ns: self.readout_integration_time_ns,
            probe_photon_number: 9.0,
            operating_temp_k: 0.015,
        };

        let l_solver = FractionalParafermionLatticeSolver::new(lat_params.clone());
        let r_solver = SurfaceCodeRepeaterSolver::new(rep_params.clone());
        let d_solver = QuditDistillationSolver::new(dist_params.clone());

        self.cached_lattice_metrics = l_solver.evaluate_metrics();
        self.cached_wavepackets = l_solver.compute_wavepackets(40);
        self.cached_braid_trajectory = l_solver.compute_braid_trajectory(40);

        self.cached_repeater_metrics = r_solver.evaluate_metrics();
        self.cached_lattice_nodes = r_solver.generate_lattice_nodes();
        self.cached_scaling_curve = r_solver.compute_threshold_scaling(24);

        self.cached_distillation_metrics = d_solver.evaluate_metrics();
        self.cached_distillation_rounds = d_solver.compute_distillation_rounds();
        self.cached_cavity_spectrum = d_solver.compute_cavity_spectrum(32);

        let processor = FractionalParafermionProcessor::new(lat_params, rep_params, dist_params);
        self.cached_audit = processor.audit_system();

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Primary UI render loop for egui modal window.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Quantum Metamaterial Fractional Parafermion Surface Code (Phase 445)")
            .open(&mut open)
            .default_width(840.0)
            .default_height(580.0)
            .show(ctx, |ui| {
                self.render_dialog_contents(ui);
            });
        self.is_open = open;
    }

    /// Renders the internal tabs and active panel.
    pub fn render_dialog_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for tab in [
                ParafermionDialogTab::ParafermionBraidingLattice,
                ParafermionDialogTab::SurfaceCodeSyndromes,
                ParafermionDialogTab::AnyonicBraidRepeater,
                ParafermionDialogTab::QuditDistillationFactory,
                ParafermionDialogTab::AuditTelemetry,
            ] {
                let is_selected = self.active_tab == tab;
                if ui.selectable_label(is_selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            ParafermionDialogTab::ParafermionBraidingLattice => self.render_braiding_lattice_tab(ui),
            ParafermionDialogTab::SurfaceCodeSyndromes => self.render_surface_code_tab(ui),
            ParafermionDialogTab::AnyonicBraidRepeater => self.render_braid_repeater_tab(ui),
            ParafermionDialogTab::QuditDistillationFactory => self.render_distillation_tab(ui),
            ParafermionDialogTab::AuditTelemetry => self.render_audit_tab(ui),
        }
    }

    fn render_braiding_lattice_tab(&mut self, ui: &mut Ui) {
        ui.heading("Fractional Parafermion Zero-Mode Lattice & Non-Abelian Braiding");
        ui.label(
            "Z_3 and Z_4 parafermion zero modes localized at domain walls between fractional quantum Hall regions.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Parafermion Order:");
            let is_z3 = self.order == FractionalParafermionOrder::Z3Clock;
            if ui.selectable_label(is_z3, "Z3 Clock (e/3)").clicked() {
                self.order = FractionalParafermionOrder::Z3Clock;
                changed = true;
            }
            let is_z4 = self.order == FractionalParafermionOrder::Z4Clock;
            if ui.selectable_label(is_z4, "Z4 Clock (e/4)").clicked() {
                self.order = FractionalParafermionOrder::Z4Clock;
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Pairing Gap Delta:");
            if ui.add(egui::Slider::new(&mut self.superconducting_gap_mhz, 2.0..=12.0).suffix(" MHz")).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Braid Duration:");
            if ui.add(egui::Slider::new(&mut self.braid_duration_ns, 40.0..=250.0).suffix(" ns")).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Plots: Wavepacket Localization and Braiding Trajectory
        let wp_pts1: PlotPoints = self
            .cached_wavepackets
            .iter()
            .map(|p| [p.x_um, p.mode1_density])
            .collect();
        let wp_pts2: PlotPoints = self
            .cached_wavepackets
            .iter()
            .map(|p| [p.x_um, p.mode2_density])
            .collect();

        let line_wp1 = Line::new("Mode Gamma_1", wp_pts1)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.0);
        let line_wp2 = Line::new("Mode Gamma_2", wp_pts2)
            .color(Color32::from_rgb(234, 88, 12))
            .width(2.0);

        Plot::new("wavepacket_density_plot")
            .height(180.0)
            .x_axis_label("Waveguide Position (um)")
            .y_axis_label("Normalized Density |Psi|^2")
            .show(ui, |plot_ui| {
                plot_ui.line(line_wp1);
                plot_ui.line(line_wp2);
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(format!("Topological Gap: {:.2} MHz", self.cached_lattice_metrics.topological_gap_mhz));
            ui.separator();
            ui.label(format!("Braid Gate Fidelity: {:.5}", self.cached_lattice_metrics.braid_fidelity));
            ui.separator();
            ui.label(format!("Diabatic Leakage Error: {:.2e}", self.cached_lattice_metrics.diabatic_leakage_error));
            ui.separator();
            ui.label(format!("Poisoning Lifetime: {:.1} us", self.cached_lattice_metrics.poisoning_lifetime_us));
        });
    }

    fn render_surface_code_tab(&mut self, ui: &mut Ui) {
        ui.heading("Z_p Qudit Surface-Code Stabilizer Syndromes");
        ui.label("2D planar patch encoding topological qudits with star and plaquette stabilizers.");
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Code Distance d:");
            let is_d3 = self.code_distance == 3;
            if ui.selectable_label(is_d3, "d = 3").clicked() {
                self.code_distance = 3;
                changed = true;
            }
            let is_d5 = self.code_distance == 5;
            if ui.selectable_label(is_d5, "d = 5").clicked() {
                self.code_distance = 5;
                changed = true;
            }

            ui.separator();
            ui.label("Physical Error Rate:");
            if ui.add(egui::Slider::new(&mut self.physical_error_rate, 0.001..=0.035).logarithmic(true)).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Grid Display of 5x5 nodes
        ui.label("2D Stabilizer Patch Grid (5x5):");
        egui::Grid::new("surface_code_grid")
            .spacing([6.0, 6.0])
            .show(ui, |ui| {
                for r in 0..5 {
                    for c in 0..5 {
                        let node = self.cached_lattice_nodes.iter().find(|n| n.row == r && n.col == c);
                        let (text, color) = match node {
                            Some(n) => {
                                if n.is_data_qudit {
                                    ("D", Color32::from_rgb(147, 197, 253))
                                } else {
                                    match n.stabilizer_kind {
                                        Some(ParafermionStabilizerKind::StarX) => {
                                            if n.syndrome_defect > 0 {
                                                ("S*", Color32::from_rgb(239, 68, 68))
                                            } else {
                                                ("S", Color32::from_rgb(134, 239, 172))
                                            }
                                        }
                                        Some(ParafermionStabilizerKind::PlaquetteZ) => {
                                            if n.syndrome_defect > 0 {
                                                ("P*", Color32::from_rgb(249, 115, 22))
                                            } else {
                                                ("P", Color32::from_rgb(253, 224, 71))
                                            }
                                        }
                                        None => (".", Color32::GRAY),
                                    }
                                }
                            }
                            None => (".", Color32::GRAY),
                        };

                        ui.label(RichText::new(format!("[{text:>2}]")).color(color).monospace());
                    }
                    ui.end_row();
                }
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(format!("Logical Error Rate P_L: {:.2e}", self.cached_repeater_metrics.logical_error_rate));
            ui.separator();
            ui.label(format!("Threshold Error P_th: {:.1}%", self.cached_repeater_metrics.threshold_error_percent));
            ui.separator();
            ui.label(format!("Suppression Factor: {:.1}x", self.cached_repeater_metrics.error_suppression_factor));
        });
    }

    fn render_braid_repeater_tab(&mut self, ui: &mut Ui) {
        ui.heading("Anyonic Braid Repeater & Threshold Scaling Interconnect");
        ui.label("Long-distance acoustic entanglement distribution with anyonic state purification.");
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Link Distance:");
            if ui.add(egui::Slider::new(&mut self.link_distance_um, 20.0..=500.0).suffix(" um")).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Repeater Nodes:");
            if ui.add(egui::Slider::new(&mut self.repeater_node_count, 1..=16)).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Threshold scaling curves
        let pts_d3: PlotPoints = self
            .cached_scaling_curve
            .iter()
            .map(|p| [p.physical_error * 100.0, p.logical_error_d3])
            .collect();
        let pts_d5: PlotPoints = self
            .cached_scaling_curve
            .iter()
            .map(|p| [p.physical_error * 100.0, p.logical_error_d5])
            .collect();
        let pts_raw: PlotPoints = self
            .cached_scaling_curve
            .iter()
            .map(|p| [p.physical_error * 100.0, p.unencoded_error])
            .collect();

        let line_d3 = Line::new("Distance d=3", pts_d3)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.0);
        let line_d5 = Line::new("Distance d=5", pts_d5)
            .color(Color32::from_rgb(168, 85, 247))
            .width(2.0);
        let line_raw = Line::new("Unencoded Physical Error", pts_raw)
            .color(Color32::from_rgb(239, 68, 68))
            .width(1.5);

        Plot::new("threshold_scaling_plot")
            .height(200.0)
            .x_axis_label("Physical Error Rate P_phys (%)")
            .y_axis_label("Logical Error Rate P_L")
            .show(ui, |plot_ui| {
                plot_ui.line(line_d3);
                plot_ui.line(line_d5);
                plot_ui.line(line_raw);
                plot_ui.hline(HLine::new("Threshold Limit", 1.0e-4).color(Color32::GRAY));
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(format!("Repeater Fidelity: {:.4}", self.cached_repeater_metrics.repeater_fidelity));
            ui.separator();
            ui.label(format!("Entanglement Distribution: {:.1} kHz", self.cached_repeater_metrics.distribution_rate_khz));
            ui.separator();
            ui.label(format!("Total Nodes: {}", self.repeater_node_count));
        });
    }

    fn render_distillation_tab(&mut self, ui: &mut Ui) {
        ui.heading("Non-Clifford Magic State Distillation Factory & Dispersive Cavity Readout");
        ui.label("Purification of non-Clifford magic states and multi-level qudit cavity spectroscopy.");
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Raw Magic Error Rate:");
            if ui.add(egui::Slider::new(&mut self.raw_magic_error_rate, 0.01..=0.10)).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Dispersive Shift:");
            if ui.add(egui::Slider::new(&mut self.dispersive_shift_chi_mhz, 1.0..=10.0).suffix(" MHz")).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Distillation table
        ui.label("Distillation Protocol Progress (Iterative Rounds):");
        egui::Grid::new("distillation_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.label(RichText::new("Round").strong());
                ui.label(RichText::new("Magic State Fidelity").strong());
                ui.label(RichText::new("Magic Error Rate").strong());
                ui.label(RichText::new("Cumulative Yield").strong());
                ui.end_row();

                for r in &self.cached_distillation_rounds {
                    ui.label(format!("{}", r.round_index));
                    ui.label(format!("{:.5}", r.magic_state_fidelity));
                    ui.label(format!("{:.2e}", r.magic_error_rate));
                    ui.label(format!("{:.1}%", r.cumulative_yield_percent));
                    ui.end_row();
                }
            });

        ui.add_space(8.0);

        // Cavity Spectrum Plot
        let pts_0: PlotPoints = self
            .cached_cavity_spectrum
            .iter()
            .map(|p| [p.detuning_mhz, p.transmission_state0_db])
            .collect();
        let pts_1: PlotPoints = self
            .cached_cavity_spectrum
            .iter()
            .map(|p| [p.detuning_mhz, p.transmission_state1_db])
            .collect();
        let pts_2: PlotPoints = self
            .cached_cavity_spectrum
            .iter()
            .map(|p| [p.detuning_mhz, p.transmission_state2_db])
            .collect();

        let l0 = Line::new("|0> Ground Peak", pts_0).color(Color32::from_rgb(56, 189, 248));
        let l1 = Line::new("|1> Excited Peak", pts_1).color(Color32::from_rgb(34, 197, 94));
        let l2 = Line::new("|2> Magic State Peak", pts_2).color(Color32::from_rgb(234, 88, 12));

        Plot::new("qudit_cavity_plot")
            .height(180.0)
            .x_axis_label("Detuning (MHz)")
            .y_axis_label("Cavity S21 (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(l0);
                plot_ui.line(l1);
                plot_ui.line(l2);
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(format!("Distilled Fidelity: {:.5}", self.cached_distillation_metrics.distilled_magic_fidelity));
            ui.separator();
            ui.label(format!("Acceptance Rate: {:.1}%", self.cached_distillation_metrics.acceptance_probability_percent));
            ui.separator();
            ui.label(format!("Readout SNR: {:.1} dB", self.cached_distillation_metrics.dispersive_readout_snr_db));
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.heading("Physics Audit Checklist & Telemetry (10-Point Audit)");
        ui.label("Automated physics verification for Phase 445.");
        ui.add_space(8.0);

        let report = self.cached_audit.clone();

        let items = [
            ("1. Topological Parafermion Gap (Delta_para >= 1.8 MHz)", report.topological_gap_pass),
            ("2. Non-Abelian Braid Fidelity (F >= 0.999)", report.braid_fidelity_pass),
            ("3. Diabatic Leakage Error Rate (P_leak < 1e-4)", report.diabatic_leakage_pass),
            ("4. Quasi-Particle Poisoning Lifetime (tau >= 80.0 us)", report.poisoning_lifetime_pass),
            ("5. Surface-Code Error Threshold (P_th >= 1.5%)", report.threshold_error_pass),
            ("6. Suppressed Logical Error Rate (P_L < 1e-4)", report.logical_error_rate_pass),
            ("7. Anyonic Braid Repeater Fidelity (F_rep >= 0.992)", report.repeater_fidelity_pass),
            ("8. Magic State Distillation Fidelity (F_magic >= 0.999)", report.magic_distillation_fidelity_pass),
            ("9. Distillation Acceptance Probability (P_acc >= 18.0%)", report.acceptance_probability_pass),
            ("10. Dispersive Readout SNR (SNR >= 18.5 dB)", report.dispersive_readout_snr_pass),
        ];

        for (label, pass) in items {
            ui.horizontal(|ui| {
                let (badge, color) = if pass {
                    ("[PASS]", Color32::from_rgb(34, 197, 94))
                } else {
                    ("[FAIL]", Color32::from_rgb(239, 68, 68))
                };
                ui.label(RichText::new(badge).color(color).monospace());
                ui.label(label);
            });
        }

        ui.add_space(10.0);
        let mut recompute_needed = false;
        ui.horizontal(|ui| {
            let total_str = format!("Total Audit Score: {}/10", report.total_score);
            let score_color = if report.all_passed {
                Color32::from_rgb(34, 197, 94)
            } else {
                Color32::from_rgb(239, 68, 68)
            };
            ui.label(RichText::new(total_str).color(score_color).strong().size(15.0));

            ui.separator();
            ui.label(format!("Last Recompute Latency: {:.1} us", self.last_solve_time_us));

            ui.separator();
            if ui.button("Recompute Physics Solvers").clicked() {
                recompute_needed = true;
            }
        });

        if recompute_needed {
            self.recompute();
        }
    }
}
