#![deny(unsafe_code)]

//! Phase 441: Topological Non-Hermitian Floquet Acoustic Chiral Lasing Metasurface & Vortex Waveguide Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring Floquet chiral gain amplification,
//! orbital angular momentum (OAM) vortex beam emission, non-linear mode competition rate equations,
//! far-field donut radiation patterns, and 10-point quantum acoustics audit telemetry.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::chiral_lasing_metasurface::{
    AcousticVortexMetrics, AcousticVortexParams, AcousticVortexWaveguideSolver,
    ChiralLasingMetasurfaceAuditReport, ChiralLasingMetasurfaceProcessor, ComplexQuasiEnergyPoint,
    FarFieldRadiationPoint, FloquetChiralLatticeMetrics, FloquetChiralLatticeParams,
    FloquetChiralLatticeSolver, LasingSpectrumPoint, LasingTransientPoint, MetasurfaceSpatialNode,
    ModeCompetitionMetrics, ModeCompetitionParams, ModeCompetitionRateSolver, VortexRadialPoint,
};

/// 5 Categorized navigation tabs for the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChiralLasingTab {
    FloquetChiralGain,
    OamVortexWaveguide,
    ModeCompetition,
    ChiralRadiationPattern,
    AuditTelemetry,
}

impl ChiralLasingTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FloquetChiralGain => "Floquet Chiral Gain",
            Self::OamVortexWaveguide => "OAM Vortex Waveguide",
            Self::ModeCompetition => "Mode Competition",
            Self::ChiralRadiationPattern => "Chiral Radiation Pattern",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Topological Chiral Lasing Metasurface (Phase 441).
#[derive(Debug, Clone)]
pub struct ChiralLasingMetasurfaceDialog {
    pub is_open: bool,
    pub active_tab: ChiralLasingTab,

    // Lattice & gain parameters
    pub floquet_modulation_mhz: f64,
    pub floquet_drive_freq_ghz: f64,
    pub gain_rate_gamma_mhz: f64,
    pub loss_rate_gamma_mhz: f64,

    // Vortex parameters
    pub topological_charge_ell: i32,
    pub beam_waist_um: f64,

    // Rate equation parameters
    pub pump_current_ma: f64,
    pub differential_gain: f64,

    // Cached telemetry and visual models
    pub cached_lattice_metrics: FloquetChiralLatticeMetrics,
    pub cached_quasi_energy: Vec<ComplexQuasiEnergyPoint>,
    pub cached_metasurface_nodes: Vec<MetasurfaceSpatialNode>,

    pub cached_vortex_metrics: AcousticVortexMetrics,
    pub cached_radial_profile: Vec<VortexRadialPoint>,
    pub cached_far_field_pattern: Vec<FarFieldRadiationPoint>,

    pub cached_rate_metrics: ModeCompetitionMetrics,
    pub cached_transient: Vec<LasingTransientPoint>,
    pub cached_spectrum: Vec<LasingSpectrumPoint>,

    pub cached_audit: ChiralLasingMetasurfaceAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ChiralLasingMetasurfaceDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ChiralLasingMetasurfaceDialog {
    /// Instantaneous cold boot constructor (< 2ms) with pre-seeded baseline state.
    pub fn new_fast() -> Self {
        let lattice_params = FloquetChiralLatticeParams::default();
        let vortex_params = AcousticVortexParams::default();
        let rate_params = ModeCompetitionParams::default();

        let l_solver = FloquetChiralLatticeSolver::new(lattice_params.clone());
        let v_solver = AcousticVortexWaveguideSolver::new(vortex_params.clone());
        let r_solver = ModeCompetitionRateSolver::new(rate_params.clone());

        let lm = l_solver.evaluate_metrics();
        let qe = l_solver.compute_quasi_energy_spectrum(32);
        let nodes = l_solver.generate_metasurface_field();

        let vm = v_solver.evaluate_metrics();
        let radial = v_solver.compute_radial_profile(40);
        let far_field = v_solver.compute_far_field_pattern(50);

        let rm = r_solver.evaluate_metrics();
        let trans = r_solver.compute_transient_dynamics(40);
        let spec = r_solver.compute_lasing_spectrum(7);

        let processor = ChiralLasingMetasurfaceProcessor::new(
            lattice_params.clone(),
            vortex_params.clone(),
            rate_params.clone(),
        );
        let audit = processor.audit_lasing_metasurface();

        Self {
            is_open: false,
            active_tab: ChiralLasingTab::FloquetChiralGain,

            floquet_modulation_mhz: lattice_params.floquet_modulation_mhz,
            floquet_drive_freq_ghz: lattice_params.floquet_drive_freq_ghz,
            gain_rate_gamma_mhz: lattice_params.gain_rate_gamma_mhz,
            loss_rate_gamma_mhz: lattice_params.loss_rate_gamma_mhz,

            topological_charge_ell: vortex_params.topological_charge_ell,
            beam_waist_um: vortex_params.beam_waist_um,

            pump_current_ma: rate_params.pump_current_ma,
            differential_gain: rate_params.differential_gain,

            cached_lattice_metrics: lm,
            cached_quasi_energy: qe,
            cached_metasurface_nodes: nodes,

            cached_vortex_metrics: vm,
            cached_radial_profile: radial,
            cached_far_field_pattern: far_field,

            cached_rate_metrics: rm,
            cached_transient: trans,
            cached_spectrum: spec,

            cached_audit: audit,
            last_solve_time_us: 190.0,
        }
    }

    /// Recomputes all physical metrics, waveforms, and audit telemetry.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let mut lp = FloquetChiralLatticeParams::default();
        lp.floquet_modulation_mhz = self.floquet_modulation_mhz;
        lp.floquet_drive_freq_ghz = self.floquet_drive_freq_ghz;
        lp.gain_rate_gamma_mhz = self.gain_rate_gamma_mhz;
        lp.loss_rate_gamma_mhz = self.loss_rate_gamma_mhz;

        let mut vp = AcousticVortexParams::default();
        vp.topological_charge_ell = self.topological_charge_ell;
        vp.beam_waist_um = self.beam_waist_um;

        let mut rp = ModeCompetitionParams::default();
        rp.pump_current_ma = self.pump_current_ma;
        rp.differential_gain = self.differential_gain;

        let l_solver = FloquetChiralLatticeSolver::new(lp.clone());
        let v_solver = AcousticVortexWaveguideSolver::new(vp.clone());
        let r_solver = ModeCompetitionRateSolver::new(rp.clone());

        let lm = l_solver.evaluate_metrics();
        let qe = l_solver.compute_quasi_energy_spectrum(32);
        let nodes = l_solver.generate_metasurface_field();

        let vm = v_solver.evaluate_metrics();
        let radial = v_solver.compute_radial_profile(40);
        let far_field = v_solver.compute_far_field_pattern(50);

        let rm = r_solver.evaluate_metrics();
        let trans = r_solver.compute_transient_dynamics(40);
        let spec = r_solver.compute_lasing_spectrum(7);

        let processor = ChiralLasingMetasurfaceProcessor::new(lp, vp, rp);
        let audit = processor.audit_lasing_metasurface();

        self.cached_lattice_metrics = lm;
        self.cached_quasi_energy = qe;
        self.cached_metasurface_nodes = nodes;

        self.cached_vortex_metrics = vm;
        self.cached_radial_profile = radial;
        self.cached_far_field_pattern = far_field;

        self.cached_rate_metrics = rm;
        self.cached_transient = trans;
        self.cached_spectrum = spec;

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
        Window::new("Topological Non-Hermitian Floquet Acoustic Chiral Laser & Vortex Waveguide (Phase 441)")
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
                ChiralLasingTab::FloquetChiralGain,
                ChiralLasingTab::OamVortexWaveguide,
                ChiralLasingTab::ModeCompetition,
                ChiralLasingTab::ChiralRadiationPattern,
                ChiralLasingTab::AuditTelemetry,
            ] {
                let selected = self.active_tab == tab;
                if ui.selectable_label(selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();

        match self.active_tab {
            ChiralLasingTab::FloquetChiralGain => self.render_chiral_gain_tab(ui),
            ChiralLasingTab::OamVortexWaveguide => self.render_vortex_tab(ui),
            ChiralLasingTab::ModeCompetition => self.render_mode_competition_tab(ui),
            ChiralLasingTab::ChiralRadiationPattern => self.render_radiation_pattern_tab(ui),
            ChiralLasingTab::AuditTelemetry => self.render_audit_tab(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_chiral_gain_tab(&mut self, ui: &mut Ui) {
        ui.heading("Floquet Non-Hermitian Chiral Gain & Topological Bandgap");
        ui.label(
            "Simulates dynamic Floquet modulation breaking time-reversal symmetry combined with distributed \
             non-Hermitian gain and loss, delivering net positive amplification to forward chiral edge states.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Floquet Mod (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.floquet_modulation_mhz, 10.0..=35.0)).changed();
            ui.label("Drive Freq (GHz):");
            changed |= ui.add(egui::Slider::new(&mut self.floquet_drive_freq_ghz, 0.2..=0.6)).changed();
            ui.label("Gain gamma (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.gain_rate_gamma_mhz, 5.0..=25.0)).changed();
            ui.label("Loss gamma (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.loss_rate_gamma_mhz, 10.0..=30.0)).changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Complex Quasi-Energy Spectrum Plot
        let pts_gain: PlotPoints = self
            .cached_quasi_energy
            .iter()
            .filter(|p| p.is_chiral_edge)
            .map(|p| [p.real_mhz, p.imag_gain_mhz])
            .collect();
        let pts_loss: PlotPoints = self
            .cached_quasi_energy
            .iter()
            .filter(|p| !p.is_chiral_edge)
            .map(|p| [p.real_mhz, p.imag_gain_mhz])
            .collect();

        let line_gain = Line::new("Chiral Amplified Edge Mode", pts_gain)
            .color(Color32::from_rgb(74, 222, 128))
            .width(3.0);
        let line_loss = Line::new("Damped Bulk & Counter Modes", pts_loss)
            .color(Color32::from_rgb(248, 113, 113))
            .width(2.0);

        Plot::new("quasi_energy_spectrum_plot")
            .height(260.0)
            .x_axis_label("Re(epsilon) Quasi-Energy (MHz)")
            .y_axis_label("Im(epsilon) Modal Gain / Loss (MHz)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_gain);
                plot_ui.line(line_loss);
                plot_ui.hline(HLine::new("Lasing Threshold (Im = 0)", 0.0).color(Color32::from_rgb(148, 163, 184)));
            });

        ui.horizontal(|ui| {
            ui.label(format!("Chiral Gain: +{:.2} MHz", self.cached_lattice_metrics.chiral_mode_gain_mhz));
            ui.separator();
            ui.label(format!("Counter Loss: {:.2} MHz", self.cached_lattice_metrics.counter_mode_loss_mhz));
            ui.separator();
            ui.label(format!("Chiral Isolation: {:.1} dB", self.cached_lattice_metrics.chiral_isolation_db));
            ui.separator();
            ui.label(format!("Threshold Power: {:.2} mW", self.cached_lattice_metrics.threshold_power_mw));
            ui.separator();
            ui.label(format!("Floquet Gap: {:.1} MHz", self.cached_lattice_metrics.floquet_bandgap_mhz));
        });
    }

    fn render_vortex_tab(&mut self, ui: &mut Ui) {
        ui.heading("Structured Orbital Angular Momentum (OAM) Vortex Waveguide");
        ui.label(
            "Models radiated acoustic vortex fields carrying quantized topological phase winding ell \
             with phase singularity dislocations and donut-shaped acoustic intensity profiles.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("OAM Charge ell:");
            for c in [1, -1, 2, -2] {
                if ui.selectable_label(self.topological_charge_ell == c, format!("{:+}", c)).clicked() {
                    self.topological_charge_ell = c;
                    changed = true;
                }
            }

            ui.separator();
            ui.label("Beam Waist w_0 (um):");
            changed |= ui.add(egui::Slider::new(&mut self.beam_waist_um, 20.0..=60.0)).changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Donut radial profile plot
        let pts_radial: PlotPoints = self
            .cached_radial_profile
            .iter()
            .map(|p| [p.radius_um, p.normalized_intensity])
            .collect();
        let line_radial = Line::new("Donut Beam Intensity I(r)", pts_radial)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.5);

        Plot::new("vortex_radial_profile_plot")
            .height(260.0)
            .x_axis_label("Radial Coordinate r (um)")
            .y_axis_label("Normalized Intensity I(r) / I_max")
            .show(ui, |plot_ui| {
                plot_ui.line(line_radial);
            });

        ui.horizontal(|ui| {
            ui.label(format!("OAM Charge ell: {:+}", self.cached_vortex_metrics.measured_topological_charge));
            ui.separator();
            ui.label(format!("Modal Purity: {:.2}%", self.cached_vortex_metrics.oam_modal_purity * 100.0));
            ui.separator();
            ui.label(format!("Beam Divergence: {:.2} deg", self.cached_vortex_metrics.beam_divergence_deg));
            ui.separator();
            ui.label(format!("Peak Ring Radius: {:.1} um", self.cached_vortex_metrics.peak_ring_radius_um));
            ui.separator();
            ui.label(format!("Core Extinction: {:.1} dB", self.cached_vortex_metrics.core_extinction_db));
        });
    }

    fn render_mode_competition_tab(&mut self, ui: &mut Ui) {
        ui.heading("Chiral Edge State Mode Competition & Dynamic Rate Equations");
        ui.label(
            "Solves coupled non-linear carrier-phonon rate equations with spatial hole burning, verifying \
             single-mode lasing stability, side-mode suppression ratio SMSR >= 30 dB, and sub-12ns turn-on transients.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Pump Current (mA):");
            changed |= ui.add(egui::Slider::new(&mut self.pump_current_ma, 10.0..=40.0)).changed();
            ui.label("Diff Gain:");
            changed |= ui.add(egui::Slider::new(&mut self.differential_gain, 1.5..=4.5)).changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Transient Turn-On Dynamics Plot
        let pts_main: PlotPoints = self
            .cached_transient
            .iter()
            .map(|p| [p.time_ns, p.dominant_mode_photons])
            .collect();
        let pts_side: PlotPoints = self
            .cached_transient
            .iter()
            .map(|p| [p.time_ns, p.side_mode_photons])
            .collect();

        let line_main = Line::new("Dominant Lasing Mode Power (mW)", pts_main)
            .color(Color32::from_rgb(74, 222, 128))
            .width(2.5);
        let line_side = Line::new("Suppressed Side Mode Power (mW)", pts_side)
            .color(Color32::from_rgb(248, 113, 113))
            .width(1.8);

        Plot::new("laser_transient_turn_on_plot")
            .height(260.0)
            .x_axis_label("Time t (ns)")
            .y_axis_label("Modal Acoustic Power (mW)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_main);
                plot_ui.line(line_side);
            });

        ui.horizontal(|ui| {
            ui.label(format!("SMSR: {:.1} dB", self.cached_rate_metrics.smsr_db));
            ui.separator();
            ui.label(format!("Linewidth: {:.2} kHz", self.cached_rate_metrics.emission_linewidth_khz));
            ui.separator();
            ui.label(format!("Turn-On Delay: {:.2} ns", self.cached_rate_metrics.turn_on_delay_ns));
            ui.separator();
            ui.label(format!("Steady State: {:.2} mW", self.cached_rate_metrics.steady_state_power_mw));
        });
    }

    fn render_radiation_pattern_tab(&mut self, ui: &mut Ui) {
        ui.heading("Far-Field Donut Acoustic Radiation Pattern");
        ui.label(
            "Evaluates out-of-plane radiated acoustic vortex emission profiles, donut intensity divergence, \
             and energy coupling efficiency into free acoustics.",
        );
        ui.add_space(8.0);

        // Far-Field Angular Pattern Plot
        let pts_ff: PlotPoints = self
            .cached_far_field_pattern
            .iter()
            .map(|p| [p.angle_deg, p.radiation_db])
            .collect();
        let line_ff = Line::new("Far-Field Donut Radiation (dB)", pts_ff)
            .color(Color32::from_rgb(168, 85, 247))
            .width(2.5);

        Plot::new("far_field_radiation_plot")
            .height(260.0)
            .x_axis_label("Emission Angle theta (deg)")
            .y_axis_label("Normalized Intensity (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_ff);
                plot_ui.hline(HLine::new("3 dB Half-Power", -3.0).color(Color32::from_rgb(148, 163, 184)));
            });

        ui.horizontal(|ui| {
            ui.label(format!("Beam Divergence: {:.2} deg", self.cached_vortex_metrics.beam_divergence_deg));
            ui.separator();
            ui.label(format!("Radiation Efficiency: {:.1}%", self.cached_vortex_metrics.radiation_efficiency * 100.0));
            ui.separator();
            ui.label(format!("Modal Purity: {:.2}%", self.cached_vortex_metrics.oam_modal_purity * 100.0));
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.heading("10-Point Physics & Quantum Acoustics Audit Verification");
        ui.label(
            "Automated audit suite validating Floquet time-reversal breaking, non-Hermitian modal gain, \
             chiral isolation, OAM phase singularity, lasing threshold, and SMSR stability.",
        );
        ui.add_space(8.0);

        let report = &self.cached_audit;
        let pass_color = Color32::from_rgb(74, 222, 128);
        let fail_color = Color32::from_rgb(248, 113, 113);

        let items = [
            ("1. Floquet Time-Reversal Symmetry Breaking (Delta_F >= 5.0 MHz)", report.trs_breaking_pass),
            ("2. Non-Hermitian Chiral Edge Modal Gain (Im(epsilon_chiral) > 0)", report.chiral_modal_gain_pass),
            ("3. Bulk & Counter-Propagating Mode Suppression (Im(epsilon) < 0)", report.bulk_counter_suppression_pass),
            ("4. Non-Reciprocal Chiral Lasing Isolation (I_chiral >= 25.0 dB)", report.chiral_isolation_pass),
            ("5. Quantized OAM Vortex Phase Charge (|ell| >= 1)", report.quantized_oam_charge_pass),
            ("6. OAM Vortex Beam Modal Purity (M_purity >= 92.0%)", report.oam_modal_purity_pass),
            ("7. Lasing Threshold Pump Power (P_th <= 15.0 mW)", report.threshold_power_pass),
            ("8. Single-Mode Stability & SMSR (SMSR >= 30.0 dB)", report.smsr_stability_pass),
            ("9. Schawlow-Townes Narrowed Emission Linewidth (Delta f <= 5.0 kHz)", report.linewidth_narrowing_pass),
            ("10. Sub-12ns Turn-On Transient Latency (tau_turn_on <= 12.0 ns)", report.turn_on_latency_pass),
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
            ui.label(format!("Chiral Gain: +{:.2} MHz", self.cached_lattice_metrics.chiral_mode_gain_mhz));
            ui.separator();
            ui.label(format!("Isolation: {:.1} dB", self.cached_lattice_metrics.chiral_isolation_db));
            ui.separator();
            ui.label(format!("OAM Charge: {:+}", self.cached_vortex_metrics.measured_topological_charge));
            ui.separator();
            ui.label(format!("SMSR: {:.1} dB", self.cached_rate_metrics.smsr_db));
            ui.separator();
            ui.label(format!("Turn-On: {:.2} ns", self.cached_rate_metrics.turn_on_delay_ns));
        });
    }
}
