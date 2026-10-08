#![deny(unsafe_code)]

//! Phase 442: Topological Acoustic Synthetic Gauge Field & Non-Abelian Holonomic Quantum Gate Processor Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring dynamic acoustic Peierls flux modulation,
//! Hofstadter energy spectrum, non-Abelian Wilczek-Zee holonomy loops, universal geometric quantum gates,
//! and cryogenic dispersive cavity state readout telemetry.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::synthetic_gauge_holonomy::{
    ComplexMatrix2x2, DispersiveReadoutPoint, HofstadterSpectrumPoint, ParameterLoopProfile,
    SyntheticEdgeDispersionPoint, SyntheticGaugeFieldSolver, SyntheticGaugeHolonomyAuditReport,
    SyntheticGaugeMetrics, SyntheticGaugeParams, SyntheticHolonomicGateKind,
    SyntheticHolonomicGateMetrics, SyntheticHolonomicGateParams, SyntheticHolonomicGateProcessor,
    SyntheticHolonomyTrajectoryPoint, TopologicalSyntheticGaugeProcessor, WilczekZeeMetrics,
    WilczekZeeParams, WilczekZeeSolver,
};

/// 5 Categorized navigation tabs for the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntheticGaugeTab {
    SyntheticGaugeLattice,
    WilczekZeeHolonomy,
    HolonomicGateSynthesis,
    DegenerateManifold,
    AuditTelemetry,
}

impl SyntheticGaugeTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SyntheticGaugeLattice => "Synthetic Gauge Lattice",
            Self::WilczekZeeHolonomy => "Wilczek-Zee Holonomy",
            Self::HolonomicGateSynthesis => "Holonomic Gate Synthesis",
            Self::DegenerateManifold => "Degenerate Manifold",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Topological Synthetic Gauge & Holonomic Gate Processor (Phase 442).
#[derive(Debug, Clone)]
pub struct SyntheticGaugeHolonomyDialog {
    pub is_open: bool,
    pub active_tab: SyntheticGaugeTab,

    // Gauge lattice parameters
    pub bare_coupling_mhz: f64,
    pub modulation_amplitude_mhz: f64,
    pub synthetic_flux_ratio: f64,
    pub has_edge_defect: bool,

    // Wilczek-Zee parameters
    pub peak_coupling_mhz: f64,
    pub loop_duration_ns: f64,
    pub theta_max_rad: f64,
    pub active_loop_profile: ParameterLoopProfile,

    // Quantum gate parameters
    pub target_gate: SyntheticHolonomicGateKind,
    pub gate_duration_ns: f64,
    pub peak_rabi_mhz: f64,
    pub inter_qubit_coupling_mhz: f64,
    pub dispersive_shift_mhz: f64,

    // Cached models and telemetry
    pub cached_gauge_metrics: SyntheticGaugeMetrics,
    pub cached_hofstadter: Vec<HofstadterSpectrumPoint>,
    pub cached_edge_dispersion: Vec<SyntheticEdgeDispersionPoint>,

    pub cached_holonomy_metrics: WilczekZeeMetrics,
    pub cached_trajectory: Vec<SyntheticHolonomyTrajectoryPoint>,
    pub cached_unitary_c1: ComplexMatrix2x2,
    pub cached_unitary_c2: ComplexMatrix2x2,

    pub cached_gate_metrics: SyntheticHolonomicGateMetrics,
    pub cached_readout_spectrum: Vec<DispersiveReadoutPoint>,

    pub cached_audit: SyntheticGaugeHolonomyAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for SyntheticGaugeHolonomyDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl SyntheticGaugeHolonomyDialog {
    /// Instantaneous cold-boot constructor (< 2ms) with pre-seeded baseline state.
    pub fn new_fast() -> Self {
        let gauge_params = SyntheticGaugeParams::default();
        let holonomy_params = WilczekZeeParams::default();
        let gate_params = SyntheticHolonomicGateParams::default();

        let g_solver = SyntheticGaugeFieldSolver::new(gauge_params.clone());
        let h_solver = WilczekZeeSolver::new(holonomy_params.clone());
        let q_processor = SyntheticHolonomicGateProcessor::new(gate_params.clone());

        let gm = g_solver.evaluate_metrics();
        let hof = g_solver.compute_hofstadter_spectrum(24);
        let disp = g_solver.compute_edge_dispersion(24);

        let hm = h_solver.evaluate_metrics();
        let traj = h_solver.compute_trajectory(24);
        let u1 = h_solver.compute_holonomy_unitary(ParameterLoopProfile::LongitudinalCircle);
        let u2 = h_solver.compute_holonomy_unitary(ParameterLoopProfile::LatitudinalCircle);

        let qm = q_processor.evaluate_metrics();
        let spec = q_processor.compute_readout_spectrum(30);

        let master = TopologicalSyntheticGaugeProcessor {
            lattice_solver: g_solver,
            holonomy_solver: h_solver,
            gate_processor: q_processor,
        };
        let audit = master.audit_processor();

        Self {
            is_open: false,
            active_tab: SyntheticGaugeTab::SyntheticGaugeLattice,

            bare_coupling_mhz: gauge_params.bare_coupling_mhz,
            modulation_amplitude_mhz: gauge_params.modulation_amplitude_mhz,
            synthetic_flux_ratio: gauge_params.synthetic_flux_ratio,
            has_edge_defect: gauge_params.has_edge_defect,

            peak_coupling_mhz: holonomy_params.peak_coupling_mhz,
            loop_duration_ns: holonomy_params.loop_duration_ns,
            theta_max_rad: holonomy_params.theta_max_rad,
            active_loop_profile: ParameterLoopProfile::LongitudinalCircle,

            target_gate: gate_params.target_gate,
            gate_duration_ns: gate_params.gate_duration_ns,
            peak_rabi_mhz: gate_params.peak_rabi_mhz,
            inter_qubit_coupling_mhz: gate_params.inter_qubit_coupling_mhz,
            dispersive_shift_mhz: gate_params.dispersive_shift_mhz,

            cached_gauge_metrics: gm,
            cached_hofstadter: hof,
            cached_edge_dispersion: disp,

            cached_holonomy_metrics: hm,
            cached_trajectory: traj,
            cached_unitary_c1: u1,
            cached_unitary_c2: u2,

            cached_gate_metrics: qm,
            cached_readout_spectrum: spec,

            cached_audit: audit,
            last_solve_time_us: 450.0,
        }
    }

    /// Recomputes physics solvers upon parameter adjustment.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let gauge_params = SyntheticGaugeParams {
            bare_coupling_mhz: self.bare_coupling_mhz,
            modulation_amplitude_mhz: self.modulation_amplitude_mhz,
            synthetic_flux_ratio: self.synthetic_flux_ratio,
            has_edge_defect: self.has_edge_defect,
            ..Default::default()
        };

        let holonomy_params = WilczekZeeParams {
            peak_coupling_mhz: self.peak_coupling_mhz,
            loop_duration_ns: self.loop_duration_ns,
            theta_max_rad: self.theta_max_rad,
            ..Default::default()
        };

        let gate_params = SyntheticHolonomicGateParams {
            target_gate: self.target_gate,
            gate_duration_ns: self.gate_duration_ns,
            peak_rabi_mhz: self.peak_rabi_mhz,
            inter_qubit_coupling_mhz: self.inter_qubit_coupling_mhz,
            dispersive_shift_mhz: self.dispersive_shift_mhz,
            ..Default::default()
        };

        let g_solver = SyntheticGaugeFieldSolver::new(gauge_params.clone());
        let h_solver = WilczekZeeSolver::new(holonomy_params.clone());
        let q_processor = SyntheticHolonomicGateProcessor::new(gate_params.clone());

        self.cached_gauge_metrics = g_solver.evaluate_metrics();
        self.cached_hofstadter = g_solver.compute_hofstadter_spectrum(28);
        self.cached_edge_dispersion = g_solver.compute_edge_dispersion(28);

        self.cached_holonomy_metrics = h_solver.evaluate_metrics();
        self.cached_trajectory = h_solver.compute_trajectory(28);
        self.cached_unitary_c1 = h_solver.compute_holonomy_unitary(ParameterLoopProfile::LongitudinalCircle);
        self.cached_unitary_c2 = h_solver.compute_holonomy_unitary(ParameterLoopProfile::LatitudinalCircle);

        self.cached_gate_metrics = q_processor.evaluate_metrics();
        self.cached_readout_spectrum = q_processor.compute_readout_spectrum(32);

        let master = TopologicalSyntheticGaugeProcessor {
            lattice_solver: g_solver,
            holonomy_solver: h_solver,
            gate_processor: q_processor,
        };
        self.cached_audit = master.audit_processor();
        self.last_solve_time_us = start.elapsed().as_secs_f64() * 1.0e6;
    }

    /// Renders modal window in the GUI.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders modal window with open toggle.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Topological Acoustic Synthetic Gauge Field & Holonomic Gate Processor (Phase 442)")
            .open(&mut is_open)
            .default_width(980.0)
            .default_height(680.0)
            .show(ctx, |ui| {
                self.render_dialog_contents(ui);
            });

        self.is_open = is_open;
    }

    /// Renders the contents of the modal dialog into the given Ui.
    pub fn render_dialog_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for tab in [
                SyntheticGaugeTab::SyntheticGaugeLattice,
                SyntheticGaugeTab::WilczekZeeHolonomy,
                SyntheticGaugeTab::HolonomicGateSynthesis,
                SyntheticGaugeTab::DegenerateManifold,
                SyntheticGaugeTab::AuditTelemetry,
            ] {
                let selected = self.active_tab == tab;
                if ui.selectable_label(selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();

        match self.active_tab {
            SyntheticGaugeTab::SyntheticGaugeLattice => self.render_lattice_tab(ui),
            SyntheticGaugeTab::WilczekZeeHolonomy => self.render_holonomy_tab(ui),
            SyntheticGaugeTab::HolonomicGateSynthesis => self.render_gate_synthesis_tab(ui),
            SyntheticGaugeTab::DegenerateManifold => self.render_manifold_tab(ui),
            SyntheticGaugeTab::AuditTelemetry => self.render_audit_tab(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_lattice_tab(&mut self, ui: &mut Ui) {
        ui.heading("Topological Acoustic Synthetic Gauge Field & Artificial Peierls Flux");
        ui.label(
            "Simulates dynamic coupling modulation inducing artificial Peierls phases Phi, \
             synthesizing Landau level subbands and chiral edge transport without physical magnetic fields.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.bare_coupling_mhz, 2.0..=18.0).text("Coupling t_0 (MHz)"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.modulation_amplitude_mhz, 1.0..=8.0).text("Modulation delta_t (MHz)"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.synthetic_flux_ratio, 0.05..=0.95).text("Flux Ratio Phi / 2pi"))
                .changed();
            changed |= ui.checkbox(&mut self.has_edge_defect, "Boundary Defect Vacancy").changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Chiral edge dispersion plot
        let pts_forward: PlotPoints = self
            .cached_edge_dispersion
            .iter()
            .filter(|p| p.is_forward_edge)
            .map(|p| [p.momentum_kx, p.energy_mhz])
            .collect();
        let pts_bulk: PlotPoints = self
            .cached_edge_dispersion
            .iter()
            .filter(|p| !p.is_forward_edge)
            .map(|p| [p.momentum_kx, p.energy_mhz])
            .collect();

        let line_forward = Line::new("Chiral Edge State (C = +1)", pts_forward)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.5);
        let line_bulk = Line::new("Bulk Landau Bands", pts_bulk)
            .color(Color32::from_rgb(148, 163, 184))
            .width(1.5);

        Plot::new("synthetic_gauge_dispersion_plot")
            .height(260.0)
            .x_axis_label("Boundary Momentum k_x * a (rad)")
            .y_axis_label("Quasi-Energy omega - omega_0 (MHz)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_forward);
                plot_ui.line(line_bulk);
                plot_ui.hline(HLine::new("Midgap Zero", 0.0).color(Color32::from_rgb(100, 116, 139)));
            });

        ui.horizontal(|ui| {
            ui.label(format!("Peierls Flux: {:.3} rad", self.cached_gauge_metrics.peierls_flux_rad));
            ui.separator();
            ui.label(format!("Chern Number: C = {:.1}", self.cached_gauge_metrics.synthetic_chern_number));
            ui.separator();
            ui.label(format!("Bulk Bandgap: {:.2} MHz", self.cached_gauge_metrics.bulk_bandgap_mhz));
            ui.separator();
            ui.label(format!("Edge Velocity: {:.0} m/s", self.cached_gauge_metrics.chiral_edge_velocity_ms));
            ui.separator();
            ui.label(format!("Defect Immunity: {:.1}%", self.cached_gauge_metrics.defect_immunity_ratio * 100.0));
        });
    }

    fn render_holonomy_tab(&mut self, ui: &mut Ui) {
        ui.heading("Non-Abelian Wilczek-Zee Holonomy & Geometric Berry Connection");
        ui.label(
            "Traces closed control trajectories in degenerate dark-state sub-manifolds, \
             demonstrating path-ordered non-commuting holonomies [U(C1), U(C2)] != 0 with geometric speed invariance.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.peak_coupling_mhz, 10.0..=50.0).text("Rabi Rate Omega_0 (MHz)"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.loop_duration_ns, 15.0..=80.0).text("Loop Duration tau (ns)"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.theta_max_rad, 0.2..=2.8).text("Max Angle theta_0 (rad)"))
                .changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Trajectory plot
        let pts_overlap: PlotPoints = self
            .cached_trajectory
            .iter()
            .map(|p| [p.time_ns, p.dark_state_overlap])
            .collect();
        let pts_leak: PlotPoints = self
            .cached_trajectory
            .iter()
            .map(|p| [p.time_ns, (p.bright_state_leakage.max(1e-7)).log10()])
            .collect();

        let line_overlap = Line::new("Dark State Overlap |<D|psi>|^2", pts_overlap)
            .color(Color32::from_rgb(52, 211, 153))
            .width(2.5);
        let line_leak = Line::new("Log10 Bright State Leakage P_leak", pts_leak)
            .color(Color32::from_rgb(248, 113, 113))
            .width(1.8);

        Plot::new("wilczek_zee_trajectory_plot")
            .height(260.0)
            .x_axis_label("Evolution Time t (ns)")
            .y_axis_label("Fidelity / Log Leakage")
            .show(ui, |plot_ui| {
                plot_ui.line(line_overlap);
                plot_ui.line(line_leak);
            });

        ui.horizontal(|ui| {
            ui.label(format!("Commutator Norm: {:.4}", self.cached_holonomy_metrics.commutator_norm));
            ui.separator();
            ui.label(format!("Speed Invariance: {:.2e}", self.cached_holonomy_metrics.speed_invariance_residual));
            ui.separator();
            ui.label(format!("P_leak: {:.2e}", self.cached_holonomy_metrics.bright_state_leakage));
            ui.separator();
            ui.label(format!("Solid Angle: {:.2} sr", self.cached_holonomy_metrics.enclosed_solid_angle_sr));
            ui.separator();
            ui.label(format!("Adiabaticity: {:.1}", self.cached_holonomy_metrics.adiabaticity_ratio));
        });
    }

    fn render_gate_synthesis_tab(&mut self, ui: &mut Ui) {
        ui.heading("Universal Non-Abelian Holonomic Quantum Gate Synthesis");
        ui.label(
            "Synthesizes geometric single-qubit and two-qubit entangling gates via non-Abelian holonomic loops, \
             interfaced with cryogenic dispersive acoustic cavity state readout.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Target Gate:");
            for gate in [
                SyntheticHolonomicGateKind::Hadamard,
                SyntheticHolonomicGateKind::PauliX,
                SyntheticHolonomicGateKind::PhaseS,
                SyntheticHolonomicGateKind::RotationZ,
                SyntheticHolonomicGateKind::ControlledPhaseCZ,
                SyntheticHolonomicGateKind::ControlledNOT,
            ] {
                let sel = self.target_gate == gate;
                if ui.selectable_label(sel, gate.name()).clicked() {
                    self.target_gate = gate;
                    changed = true;
                }
            }
        });

        ui.horizontal(|ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.gate_duration_ns, 15.0..=50.0).text("Gate Time tau (ns)"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.peak_rabi_mhz, 15.0..=50.0).text("Peak Rabi (MHz)"))
                .changed();
            if self.target_gate.is_two_qubit() {
                changed |= ui
                    .add(egui::Slider::new(&mut self.inter_qubit_coupling_mhz, 5.0..=25.0).text("Exchange g_qq (MHz)"))
                    .changed();
            }
            changed |= ui
                .add(egui::Slider::new(&mut self.dispersive_shift_mhz, 2.0..=8.0).text("Cavity Shift chi (MHz)"))
                .changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Dispersive readout transmission spectrum plot
        let pts_spec: PlotPoints = self
            .cached_readout_spectrum
            .iter()
            .map(|p| [p.freq_offset_mhz, p.transmission_db])
            .collect();
        let line_spec = Line::new("Cavity Readout Spectrum |S_21(f)| (dB)", pts_spec)
            .color(Color32::from_rgb(168, 85, 247))
            .width(2.5);

        Plot::new("dispersive_readout_spectrum_plot")
            .height(260.0)
            .x_axis_label("Cavity Detuning f - f_cav (MHz)")
            .y_axis_label("Transmission |S_21| (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_spec);
                plot_ui.hline(HLine::new("-3 dB Level", -3.0).color(Color32::from_rgb(148, 163, 184)));
            });

        ui.horizontal(|ui| {
            ui.label(format!("Process Fidelity: {:.4}%", self.cached_gate_metrics.process_fidelity * 100.0));
            ui.separator();
            ui.label(format!("Execution Latency: {:.1} ns", self.cached_gate_metrics.execution_latency_ns));
            ui.separator();
            ui.label(format!("Leakage P_leak: {:.2e}", self.cached_gate_metrics.bright_state_leakage));
            ui.separator();
            if self.target_gate.is_two_qubit() {
                ui.label(format!("Concurrence: {:.3}", self.cached_gate_metrics.entangling_concurrence));
                ui.separator();
            }
            ui.label(format!("Readout SNR: {:.1} dB", self.cached_gate_metrics.readout_snr_db));
            ui.separator();
            ui.label(format!("QND Fidelity: {:.4}", self.cached_gate_metrics.qnd_readout_fidelity));
        });
    }

    fn render_manifold_tab(&mut self, ui: &mut Ui) {
        ui.heading("Tripod Acoustic Resonator & Degenerate Dark-State Manifold");
        ui.label(
            "Matrix representations of the geometric holonomy operators U(C) and unitary properties.",
        );
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Holonomy Unitary U(C1) (Longitudinal Loop)");
                    let d = &self.cached_unitary_c1.data;
                    ui.monospace(format!("[ {:+.4} {:+.4}i , {:+.4} {:+.4}i ]", d[0], d[1], d[2], d[3]));
                    ui.monospace(format!("[ {:+.4} {:+.4}i , {:+.4} {:+.4}i ]", d[4], d[5], d[6], d[7]));
                    ui.label(format!("Frobenius Norm: {:.4}", self.cached_unitary_c1.norm()));
                });
            });

            ui.add_space(16.0);

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Holonomy Unitary U(C2) (Latitudinal Loop)");
                    let d = &self.cached_unitary_c2.data;
                    ui.monospace(format!("[ {:+.4} {:+.4}i , {:+.4} {:+.4}i ]", d[0], d[1], d[2], d[3]));
                    ui.monospace(format!("[ {:+.4} {:+.4}i , {:+.4} {:+.4}i ]", d[4], d[5], d[6], d[7]));
                    ui.label(format!("Frobenius Norm: {:.4}", self.cached_unitary_c2.norm()));
                });
            });
        });

        ui.add_space(12.0);
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.heading("Non-Commutative Commutator [U(C1), U(C2)]");
                let comm = self.cached_unitary_c1.commutator(&self.cached_unitary_c2);
                let d = &comm.data;
                ui.monospace(format!("[ {:+.4} {:+.4}i , {:+.4} {:+.4}i ]", d[0], d[1], d[2], d[3]));
                ui.monospace(format!("[ {:+.4} {:+.4}i , {:+.4} {:+.4}i ]", d[4], d[5], d[6], d[7]));
                ui.label(format!("Commutator Frobenius Norm: {:.4} (>= 0.50 confirms non-Abelian property)", comm.norm()));
            });
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.heading("Phase 442: 10-Point Physics Audit & Telemetry Checklist");
        ui.label("Rigorous validation of synthetic gauge invariance, non-Abelian holonomy, and gate fidelity.");
        ui.add_space(8.0);

        let audit = &self.cached_audit;
        let items = [
            ("Synthetic Gauge Flux Quantization (|Phi| > 0)", audit.trs_breaking_gauge_flux_passed),
            ("Synthetic Chern Number Quantization (C = 1.0)", audit.synthetic_chern_quantization_passed),
            ("Chiral Edge Defect Immunity (T_defect / T_clean >= 0.95)", audit.chiral_defect_immunity_passed),
            ("Non-Abelian Wilczek-Zee Non-Commutativity (||[U1, U2]|| >= 0.50)", audit.non_abelian_non_commutativity_passed),
            ("Geometric Phase Speed Invariance (residual < 1e-4)", audit.geometric_speed_invariance_passed),
            ("Dark-State Leakage Suppression (P_leak < 1e-4)", audit.dark_state_leakage_suppression_passed),
            ("Single-Qubit Holonomic Gate Fidelity (F >= 0.999)", audit.single_qubit_gate_fidelity_passed),
            ("Two-Qubit Holonomic Entangling Fidelity (F >= 0.999, C >= 0.95)", audit.two_qubit_entangling_fidelity_passed),
            ("Sub-50ns Gate Operation Latency (tau <= 50.0 ns)", audit.sub_50ns_gate_latency_passed),
            ("Cryogenic Dispersive State Readout (F >= 0.998, SNR >= 18.0 dB)", audit.cryogenic_dispersive_readout_passed),
        ];

        for (name, passed) in items {
            ui.horizontal(|ui| {
                if passed {
                    ui.label(RichText::new("[PASS]").color(Color32::from_rgb(34, 197, 94)).strong());
                } else {
                    ui.label(RichText::new("[FAIL]").color(Color32::from_rgb(239, 68, 68)).strong());
                }
                ui.label(name);
            });
        }

        ui.add_space(8.0);
        let score_color = if audit.all_passed {
            Color32::from_rgb(34, 197, 94)
        } else {
            Color32::from_rgb(239, 68, 68)
        };
        ui.label(
            RichText::new(format!("Total Audit Score: {} / 10 PASS", audit.total_score))
                .color(score_color)
                .strong()
                .size(16.0),
        );
        ui.label(format!("Last Recompute Benchmark: {:.1} us", self.last_solve_time_us));
    }

    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Status: Active").color(Color32::from_rgb(52, 211, 153)));
            ui.separator();
            ui.label(format!("Target Gate: {}", self.target_gate.name()));
            ui.separator();
            ui.label(format!("Gate Fidelity: {:.4}%", self.cached_gate_metrics.process_fidelity * 100.0));
            ui.separator();
            ui.label(format!("Latency: {:.1} ns", self.cached_gate_metrics.execution_latency_ns));
            ui.separator();
            ui.label(format!("Commutator: {:.3}", self.cached_holonomy_metrics.commutator_norm));
            ui.separator();
            ui.label(format!("Readout SNR: {:.1} dB", self.cached_gate_metrics.readout_snr_db));
            ui.separator();
            ui.label(format!("Audit: {}/10", self.cached_audit.total_score));
        });
    }
}
