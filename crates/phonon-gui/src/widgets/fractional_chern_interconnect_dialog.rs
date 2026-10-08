#![deny(unsafe_code)]

//! Phase 440: Quantum Metamaterial Fractional Chern Insulator & Non-Abelian Parafermion Braiding Interconnect Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring fractional Chern numbers C = 1/3,
//! localized Z_3 parafermion zero modes, non-Abelian exchange braiding, universal qudit gate
//! synthesis, and cryogenic dispersive parity readout.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::fractional_chern_interconnect::{
    BraidTrajectoryPoint, BraidingInterconnectMetrics, BraidingInterconnectParams,
    BraidingInterconnectSolver, DomainWallWavefunctionPoint, EdgeDispersionPoint,
    FractionalChernInterconnectAuditReport, FractionalChernInterconnectProcessor,
    FractionalChernLatticeParams, FractionalChernLatticeSolver, FractionalChernMetrics,
    LatticeSpatialPoint, ParafermionDomainWallMetrics, ParafermionDomainWallParams,
    ParafermionDomainWallSolver, ParafermionZeroMode, QuditGateKind, ReadoutSpectrumPoint,
};

/// 5 Categorized navigation tabs for the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractionalChernTab {
    FractionalChernLattice,
    ParafermionDomainWalls,
    NonAbelianBraiding,
    QuditGateSynthesis,
    AuditTelemetry,
}

impl FractionalChernTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FractionalChernLattice => "Fractional Chern Lattice",
            Self::ParafermionDomainWalls => "Parafermion Domain Walls",
            Self::NonAbelianBraiding => "Non-Abelian Braiding",
            Self::QuditGateSynthesis => "Qudit Gate Synthesis",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Fractional Chern & Parafermion Braiding Interconnect (Phase 440).
#[derive(Debug, Clone)]
pub struct FractionalChernInterconnectDialog {
    pub is_open: bool,
    pub active_tab: FractionalChernTab,

    // Lattice parameters
    pub synthetic_flux_rad: f64,
    pub intracell_hopping_mhz: f64,
    pub intercell_hopping_mhz: f64,
    pub has_edge_obstacle: bool,

    // Domain wall parameters
    pub qudit_dimension_zn: usize,
    pub coupling_ferro_mhz: f64,
    pub pairing_gap_mhz: f64,
    pub decay_length_xi_um: f64,

    // Braiding parameters
    pub braid_duration_ns: f64,
    pub target_gate: QuditGateKind,
    pub dispersive_shift_mhz: f64,

    // Cached telemetry and visual models
    pub cached_lattice_metrics: FractionalChernMetrics,
    pub cached_edge_dispersion: Vec<EdgeDispersionPoint>,
    pub cached_lattice_field: Vec<LatticeSpatialPoint>,

    pub cached_domain_wall_metrics: ParafermionDomainWallMetrics,
    pub cached_zero_modes: Vec<ParafermionZeroMode>,
    pub cached_wavefunctions: Vec<DomainWallWavefunctionPoint>,

    pub cached_braiding_metrics: BraidingInterconnectMetrics,
    pub cached_braid_trajectories: Vec<BraidTrajectoryPoint>,
    pub cached_readout_spectrum: Vec<ReadoutSpectrumPoint>,

    pub cached_audit: FractionalChernInterconnectAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for FractionalChernInterconnectDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl FractionalChernInterconnectDialog {
    /// Instantaneous cold boot constructor (< 2ms) with pre-seeded baseline state.
    pub fn new_fast() -> Self {
        let lattice_params = FractionalChernLatticeParams::default();
        let domain_wall_params = ParafermionDomainWallParams::default();
        let braiding_params = BraidingInterconnectParams::default();

        let l_solver = FractionalChernLatticeSolver::new(lattice_params.clone());
        let dw_solver = ParafermionDomainWallSolver::new(domain_wall_params.clone());
        let b_solver = BraidingInterconnectSolver::new(braiding_params.clone());

        let lm = l_solver.evaluate_metrics();
        let disp = l_solver.compute_edge_dispersion(32);
        let field = l_solver.generate_lattice_field();

        let dm = dw_solver.evaluate_metrics();
        let z_modes = dw_solver.get_zero_modes();
        let wave = dw_solver.compute_wavefunction_profiles(30);

        let bm = b_solver.evaluate_metrics();
        let trajs = b_solver.compute_braid_trajectories(25);
        let spec = b_solver.compute_readout_spectrum(60);

        let processor = FractionalChernInterconnectProcessor::new(
            lattice_params.clone(),
            domain_wall_params.clone(),
            braiding_params.clone(),
        );
        let audit = processor.audit_interconnect();

        Self {
            is_open: false,
            active_tab: FractionalChernTab::FractionalChernLattice,

            synthetic_flux_rad: lattice_params.synthetic_flux_rad,
            intracell_hopping_mhz: lattice_params.intracell_hopping_mhz,
            intercell_hopping_mhz: lattice_params.intercell_hopping_mhz,
            has_edge_obstacle: lattice_params.has_edge_obstacle,

            qudit_dimension_zn: domain_wall_params.qudit_dimension_zn,
            coupling_ferro_mhz: domain_wall_params.coupling_ferro_mhz,
            pairing_gap_mhz: domain_wall_params.pairing_gap_mhz,
            decay_length_xi_um: domain_wall_params.decay_length_xi_um,

            braid_duration_ns: braiding_params.braid_duration_ns,
            target_gate: braiding_params.target_gate,
            dispersive_shift_mhz: braiding_params.dispersive_shift_mhz,

            cached_lattice_metrics: lm,
            cached_edge_dispersion: disp,
            cached_lattice_field: field,

            cached_domain_wall_metrics: dm,
            cached_zero_modes: z_modes,
            cached_wavefunctions: wave,

            cached_braiding_metrics: bm,
            cached_braid_trajectories: trajs,
            cached_readout_spectrum: spec,

            cached_audit: audit,
            last_solve_time_us: 180.0,
        }
    }

    /// Recomputes all physical metrics, waveforms, and audit telemetry.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let mut lp = FractionalChernLatticeParams::default();
        lp.synthetic_flux_rad = self.synthetic_flux_rad;
        lp.intracell_hopping_mhz = self.intracell_hopping_mhz;
        lp.intercell_hopping_mhz = self.intercell_hopping_mhz;
        lp.has_edge_obstacle = self.has_edge_obstacle;

        let mut dwp = ParafermionDomainWallParams::default();
        dwp.qudit_dimension_zn = self.qudit_dimension_zn;
        dwp.coupling_ferro_mhz = self.coupling_ferro_mhz;
        dwp.pairing_gap_mhz = self.pairing_gap_mhz;
        dwp.decay_length_xi_um = self.decay_length_xi_um;

        let mut bp = BraidingInterconnectParams::default();
        bp.braid_duration_ns = self.braid_duration_ns;
        bp.target_gate = self.target_gate;
        bp.qudit_dimension_zn = self.qudit_dimension_zn;
        bp.dispersive_shift_mhz = self.dispersive_shift_mhz;

        let l_solver = FractionalChernLatticeSolver::new(lp.clone());
        let dw_solver = ParafermionDomainWallSolver::new(dwp.clone());
        let b_solver = BraidingInterconnectSolver::new(bp.clone());

        let lm = l_solver.evaluate_metrics();
        let disp = l_solver.compute_edge_dispersion(32);
        let field = l_solver.generate_lattice_field();

        let dm = dw_solver.evaluate_metrics();
        let z_modes = dw_solver.get_zero_modes();
        let wave = dw_solver.compute_wavefunction_profiles(30);

        let bm = b_solver.evaluate_metrics();
        let trajs = b_solver.compute_braid_trajectories(25);
        let spec = b_solver.compute_readout_spectrum(60);

        let processor = FractionalChernInterconnectProcessor::new(lp, dwp, bp);
        let audit = processor.audit_interconnect();

        self.cached_lattice_metrics = lm;
        self.cached_edge_dispersion = disp;
        self.cached_lattice_field = field;

        self.cached_domain_wall_metrics = dm;
        self.cached_zero_modes = z_modes;
        self.cached_wavefunctions = wave;

        self.cached_braiding_metrics = bm;
        self.cached_braid_trajectories = trajs;
        self.cached_readout_spectrum = spec;

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
        Window::new("Quantum Metamaterial Fractional Chern & Parafermion Interconnect (Phase 440)")
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
                FractionalChernTab::FractionalChernLattice,
                FractionalChernTab::ParafermionDomainWalls,
                FractionalChernTab::NonAbelianBraiding,
                FractionalChernTab::QuditGateSynthesis,
                FractionalChernTab::AuditTelemetry,
            ] {
                let selected = self.active_tab == tab;
                if ui.selectable_label(selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();

        match self.active_tab {
            FractionalChernTab::FractionalChernLattice => self.render_lattice_tab(ui),
            FractionalChernTab::ParafermionDomainWalls => self.render_domain_wall_tab(ui),
            FractionalChernTab::NonAbelianBraiding => self.render_braiding_tab(ui),
            FractionalChernTab::QuditGateSynthesis => self.render_gate_synthesis_tab(ui),
            FractionalChernTab::AuditTelemetry => self.render_audit_tab(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_lattice_tab(&mut self, ui: &mut Ui) {
        ui.heading("Fractional Chern Acoustic Metamaterial & Chiral Edge Waveguide");
        ui.label(
            "Simulates 2D acoustic resonator arrays subject to synthetic gauge flux breaking time-reversal \
             symmetry, producing fractional Chern number C = 1/3, bulk gaps, and chiral edge modes with defect immunity.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Intra Hop (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.intracell_hopping_mhz, 1.0..=5.0)).changed();
            ui.label("Inter Hop (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.intercell_hopping_mhz, 5.0..=12.0)).changed();
            changed |= ui.checkbox(&mut self.has_edge_obstacle, "Insert Edge Obstacle").changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Edge Dispersion Plot
        let points: PlotPoints = self
            .cached_edge_dispersion
            .iter()
            .map(|p| [p.kx_norm, p.frequency_mhz])
            .collect();
        let line = Line::new("Chiral Edge Dispersion (MHz)", points)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.5);

        Plot::new("fractional_edge_dispersion_plot")
            .height(260.0)
            .x_axis_label("Normalized Momentum k_x * a (rad)")
            .y_axis_label("Frequency (MHz)")
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });

        ui.horizontal(|ui| {
            ui.label(format!("Fractional Chern C: {:.3}", self.cached_lattice_metrics.fractional_chern_number));
            ui.separator();
            ui.label(format!("Bulk Bandgap: {:.2} MHz", self.cached_lattice_metrics.bulk_bandgap_mhz));
            ui.separator();
            ui.label(format!("Quasiparticle Charge e*: {:.2}", self.cached_lattice_metrics.quasiparticle_charge_e_star));
            ui.separator();
            ui.label(format!("Edge Velocity: {:.0} m/s", self.cached_lattice_metrics.chiral_edge_velocity_m_s));
            ui.separator();
            ui.label(format!("Defect Transmission: {:.1}%", self.cached_lattice_metrics.defect_transmission_ratio * 100.0));
        });
    }

    fn render_domain_wall_tab(&mut self, ui: &mut Ui) {
        ui.heading("Non-Abelian Parafermion Zero Modes along Domain Walls");
        ui.label(
            "Models localized Z_3 parafermion zero modes formed along 1D domain walls of fractional \
             Chern acoustic metamaterials with proximity exchange bias and acoustic pairing.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Pairing Gap (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.pairing_gap_mhz, 2.0..=10.0)).changed();
            ui.label("Decay Length (um):");
            changed |= ui.add(egui::Slider::new(&mut self.decay_length_xi_um, 1.0..=5.0)).changed();
            ui.label("Qudit Dim Z_N:");
            ui.selectable_value(&mut self.qudit_dimension_zn, 3, "Z_3 (Qutrit)");
            ui.selectable_value(&mut self.qudit_dimension_zn, 4, "Z_4 (Ququart)");
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Wavefunction spatial profiles
        let pts_mode1: PlotPoints = self.cached_wavefunctions.iter().map(|p| [p.x_um, p.mode1_amplitude]).collect();
        let pts_mode2: PlotPoints = self.cached_wavefunctions.iter().map(|p| [p.x_um, p.mode2_amplitude]).collect();
        let pts_energy: PlotPoints = self.cached_wavefunctions.iter().map(|p| [p.x_um, p.total_energy_density]).collect();

        let line_m1 = Line::new("Mode alpha_1", pts_mode1).color(Color32::from_rgb(74, 222, 128)).width(2.0);
        let line_m2 = Line::new("Mode alpha_2", pts_mode2).color(Color32::from_rgb(248, 113, 113)).width(2.0);
        let line_e = Line::new("Total Energy Density", pts_energy).color(Color32::from_rgb(168, 85, 247)).width(2.5);

        Plot::new("domain_wall_wavefunction_plot")
            .height(260.0)
            .x_axis_label("Domain Wall Coordinate x (um)")
            .y_axis_label("Wavefunction Amplitude / Density")
            .show(ui, |plot_ui| {
                plot_ui.line(line_m1);
                plot_ui.line(line_m2);
                plot_ui.line(line_e);
            });

        ui.horizontal(|ui| {
            ui.label(format!("Algebra Residual: {:.2e}", self.cached_domain_wall_metrics.algebra_commutation_residual));
            ui.separator();
            ui.label(format!("Confinement: {:.1}%", self.cached_domain_wall_metrics.spatial_confinement_ratio * 100.0));
            ui.separator();
            ui.label(format!("Degeneracy: {}", self.cached_domain_wall_metrics.ground_state_degeneracy));
            ui.separator();
            ui.label(format!("Splitting: {:.2e} MHz", self.cached_domain_wall_metrics.zero_mode_splitting_mhz));
        });
    }

    fn render_braiding_tab(&mut self, ui: &mut Ui) {
        ui.heading("Non-Abelian Adiabatic Braiding Trajectories & Artin Relations");
        ui.label(
            "Simulates the adiabatic exchange braiding of localized parafermion zero modes along multi-terminal \
             acoustic interconnect buses, strictly satisfying Artin relations tau_1 tau_2 tau_1 = tau_2 tau_1 tau_2.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Braid Duration (ns):");
            changed |= ui.add(egui::Slider::new(&mut self.braid_duration_ns, 20.0..=50.0)).changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Braid Trajectory Spatial Orbit Plot
        let pts_orbit1: PlotPoints = self.cached_braid_trajectories.iter().map(|p| [p.x1_um, p.y1_um]).collect();
        let pts_orbit2: PlotPoints = self.cached_braid_trajectories.iter().map(|p| [p.x2_um, p.y2_um]).collect();

        let line_orb1 = Line::new("Mode 1 Arc", pts_orbit1).color(Color32::from_rgb(56, 189, 248)).width(2.5);
        let line_orb2 = Line::new("Mode 2 Arc", pts_orbit2).color(Color32::from_rgb(251, 146, 60)).width(2.5);

        Plot::new("braid_orbit_plot")
            .height(260.0)
            .x_axis_label("Bus Coordinate x (um)")
            .y_axis_label("Transverse Orbit y (um)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_orb1);
                plot_ui.line(line_orb2);
            });

        ui.horizontal(|ui| {
            ui.label(format!("Artin Braid Residual: {:.2e}", self.cached_braiding_metrics.artin_braid_residual));
            ui.separator();
            ui.label(format!("Braiding Latency: {:.1} ns", self.cached_braiding_metrics.braiding_latency_ns));
            ui.separator();
            ui.label(format!("Diabatic Leakage: {:.2e}", self.cached_braiding_metrics.diabatic_leakage_error));
        });
    }

    fn render_gate_synthesis_tab(&mut self, ui: &mut Ui) {
        ui.heading("Universal Qudit Logic Gate Synthesis & Cryogenic Readout");
        ui.label(
            "Compiles universal single-qutrit gates (F_3, S_3, X_3, Z_3) and two-qutrit CSUM entanglers \
             from braid words, and measures fractional parity states via cryogenic dispersive cavity shifts.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Target Gate:");
            for (g, label) in [
                (QuditGateKind::GeneralizedHadamard, "F_3"),
                (QuditGateKind::PhaseS3, "S_3"),
                (QuditGateKind::ShiftX3, "X_3"),
                (QuditGateKind::ClockZ3, "Z_3"),
                (QuditGateKind::CSumTwoQutrit, "CSUM"),
            ] {
                if ui.selectable_label(self.target_gate == g, label).clicked() {
                    self.target_gate = g;
                    changed = true;
                }
            }

            ui.separator();
            ui.label("Dispersive Shift (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.dispersive_shift_mhz, 2.0..=8.0)).changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.strong("Selected Gate: ");
            ui.label(self.target_gate.name());
            ui.separator();
            ui.strong("Braid Word: ");
            ui.label(RichText::new(self.target_gate.braid_word()).monospace().color(Color32::from_rgb(250, 204, 21)));
            ui.separator();
            ui.strong("Process Fidelity: ");
            ui.label(RichText::new(format!("{:.2}%", self.cached_braiding_metrics.compiled_gate_fidelity * 100.0)).strong().color(Color32::from_rgb(74, 222, 128)));
        });

        ui.add_space(6.0);

        // Cryogenic Readout Spectrum Plot
        let pts_spec: PlotPoints = self.cached_readout_spectrum.iter().map(|p| [p.frequency_mhz, p.transmission_db]).collect();
        let line_spec = Line::new("Cavity Transmission |S_21| (dB)", pts_spec).color(Color32::from_rgb(236, 72, 153)).width(2.2);

        Plot::new("cryogenic_readout_spectrum_plot")
            .height(220.0)
            .x_axis_label("Cavity Frequency (MHz)")
            .y_axis_label("Transmission S_21 (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_spec);
                plot_ui.hline(HLine::new("3 dB Cutoff", -3.0).color(Color32::from_rgb(148, 163, 184)));
            });

        ui.horizontal(|ui| {
            ui.label(format!("Readout SNR: {:.1} dB", self.cached_braiding_metrics.readout_snr_db));
            ui.separator();
            ui.label(format!("QND Readout Fidelity: {:.3}%", self.cached_braiding_metrics.qnd_readout_fidelity * 100.0));
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.heading("10-Point Physics & Quantum Acoustics Audit Verification");
        ui.label(
            "Automated audit suite validating fractional Chern invariants, zero mode localization, \
             braid unitarity, universal gate compilation, and cryogenic telemetry.",
        );
        ui.add_space(8.0);

        let report = &self.cached_audit;
        let pass_color = Color32::from_rgb(74, 222, 128);
        let fail_color = Color32::from_rgb(248, 113, 113);

        let items = [
            ("1. Fractional Chern Invariant Quantization (|C - 1/3| <= 0.01)", report.chern_quantization_pass),
            ("2. Bulk Topological Bandgap (Delta_bulk >= 3.0 MHz)", report.bulk_bandgap_pass),
            ("3. Fractional Quasiparticle Excitation Charge (|e* - 1/3| <= 0.01)", report.quasiparticle_charge_pass),
            ("4. Chiral Edge Backscattering Immunity (T_defect / T_clean >= 0.95)", report.backscattering_immunity_pass),
            ("5. Parafermion Commutation Algebra (|alpha_j * alpha_k - omega * alpha_k * alpha_j| <= 1e-8)", report.commutation_algebra_pass),
            ("6. Domain Wall Zero-Mode Energy Confinement (>= 85.0%)", report.domain_wall_confinement_pass),
            ("7. Non-Abelian Artin Braid Relation (||tau_1 * tau_2 * tau_1 - tau_2 * tau_1 * tau_2|| <= 1e-8)", report.artin_braid_relation_pass),
            ("8. Universal Qudit Gate Process Fidelity (F_gate >= 0.999)", report.gate_fidelity_pass),
            ("9. Cryogenic Dispersive Readout SNR (>= 18.0 dB)", report.cryogenic_readout_snr_pass),
            ("10. Sub-40ns Adiabatic Braiding Latency (tau_braid <= 40.0 ns)", report.braiding_latency_pass),
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
            ui.label(format!("Chern C: {:.3}", self.cached_lattice_metrics.fractional_chern_number));
            ui.separator();
            ui.label(format!("Bulk Gap: {:.1} MHz", self.cached_lattice_metrics.bulk_bandgap_mhz));
            ui.separator();
            ui.label(format!("Gate Fidelity: {:.2}%", self.cached_braiding_metrics.compiled_gate_fidelity * 100.0));
            ui.separator();
            ui.label(format!("Readout SNR: {:.1} dB", self.cached_braiding_metrics.readout_snr_db));
            ui.separator();
            ui.label(format!("Braid Latency: {:.1} ns", self.cached_braiding_metrics.braiding_latency_ns));
        });
    }
}
