#![deny(unsafe_code)]

//! Phase 448: Moiré Exciton-Polariton Valley Hall Chiral Lasing Metasurface & Opto-Acoustic Synthesizer Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring transition metal dichalcogenide moiré superlattice
//! exciton-polaritons, topological valley Hall edge waveguides with backscattering immunity around sharp bends,
//! non-reciprocal chiral polariton lasing and BEC condensation, and microwave-to-optical opto-acoustic frequency synthesis.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::moire_polariton_laser::{
    ChiralLasingMetrics, ChiralLasingParams, ChiralLasingSolver, CircularPolarization,
    FrequencyCombLine, InputOutputCurvePoint, MoireDispersionPoint, MoireEdgeDispersionPoint,
    MoireLatticeMetrics, MoirePolaritonAuditReport, MoirePolaritonLaserProcessor,
    MoirePolaritonLatticeSolver, MoirePolaritonParams, OptoAcousticMetrics,
    OptoAcousticPhaseNoisePoint, OptoAcousticSynthesizerParams, OptoAcousticSynthesizerSolver,
    TemporalCoherencePoint, ValleyHallEdgeMetrics, ValleyHallEdgeParams, ValleyHallEdgeSolver,
    WaveguideTransmissionPoint,
};

/// 5 Categorized navigation tabs for the Moiré Polariton dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoirePolaritonDialogTab {
    MoirePolaritonBand,
    ValleyHallEdgeWaveguide,
    ChiralPolaritonLaser,
    OptoAcousticSynthesizer,
    AuditTelemetry,
}

impl MoirePolaritonDialogTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::MoirePolaritonBand => "Moiré Polariton Band",
            Self::ValleyHallEdgeWaveguide => "Valley Hall Edge Waveguide",
            Self::ChiralPolaritonLaser => "Chiral Polariton Laser",
            Self::OptoAcousticSynthesizer => "Opto-Acoustic Synthesizer",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Moiré Exciton-Polariton Chiral Lasing Metasurface & Frequency Synthesizer (Phase 448).
#[derive(Debug, Clone)]
pub struct MoirePolaritonDialog {
    pub is_open: bool,
    pub active_tab: MoirePolaritonDialogTab,

    // Tab 1: Moiré Polariton Lattice parameters
    pub twist_angle_deg: f64,
    pub rabi_splitting_mev: f64,
    pub cavity_detuning_mev: f64,
    pub moire_potential_mev: f64,
    pub valley_zeeman_mev: f64,

    // Tab 2: Valley Hall Edge parameters
    pub boundary_length_um: f64,
    pub bulk_valley_gap_mev: f64,
    pub edge_velocity_ms: f64,
    pub intervalley_disorder: f64,
    pub bend_angle_deg: f64,
    pub has_sharp_obstacle: bool,
    pub pump_polarization: CircularPolarization,

    // Tab 3: Chiral Lasing parameters
    pub pump_power_uw_um2: f64,
    pub threshold_power_uw_um2: f64,
    pub polariton_lifetime_ps: f64,
    pub polariton_interaction_uev_um2: f64,
    pub pump_chirality_factor: f64,

    // Tab 4: Opto-Acoustic Synthesizer parameters
    pub carrier_frequency_thz: f64,
    pub microwave_acoustic_freq_ghz: f64,
    pub acoustic_drive_power_mw: f64,
    pub optoacoustic_coupling_mhz: f64,
    pub optical_cavity_linewidth_ghz: f64,

    // Cached simulation outputs
    pub cached_lattice_metrics: MoireLatticeMetrics,
    pub cached_dispersion: Vec<MoireDispersionPoint>,
    pub cached_berry_profile: Vec<(f64, f64, f64)>,

    pub cached_edge_metrics: ValleyHallEdgeMetrics,
    pub cached_edge_dispersion: Vec<MoireEdgeDispersionPoint>,
    pub cached_waveguide_spectrum: Vec<WaveguideTransmissionPoint>,
    pub cached_spatial_profiles: Vec<(f64, f64, f64)>,

    pub cached_lasing_metrics: ChiralLasingMetrics,
    pub cached_io_curve: Vec<InputOutputCurvePoint>,
    pub cached_coherence_curve: Vec<TemporalCoherencePoint>,
    pub cached_angular_profile: Vec<(f64, f64)>,

    pub cached_synth_metrics: OptoAcousticMetrics,
    pub cached_frequency_comb: Vec<FrequencyCombLine>,
    pub cached_phase_noise: Vec<OptoAcousticPhaseNoisePoint>,
    pub cached_efficiency_curve: Vec<(f64, f64)>,

    pub cached_audit: MoirePolaritonAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for MoirePolaritonDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl MoirePolaritonDialog {
    /// Fast cold-boot constructor executing in sub-2ms with pre-seeded baseline telemetry.
    pub fn new_fast() -> Self {
        let twist = 1.8;
        let rabi = 24.0;
        let detuning = 0.0;
        let v_m = 35.0;
        let zeeman = 1.2;

        let len_um = 15.0;
        let gap = 16.0;
        let v_edge = 1.8e5;
        let disorder = 0.025;
        let bend = 60.0;
        let obst = false;
        let pol = CircularPolarization::SigmaPlus;

        let pump = 4.5;
        let p_th = 2.2;
        let tau = 22.0;
        let g_pol = 12.0;
        let chi = 0.95;

        let carr = 400.0;
        let f_ac = 3.6;
        let p_mw = 2.5;
        let g_om = 18.0;
        let kappa = 12.0;

        let lattice_params = MoirePolaritonParams {
            twist_angle_deg: twist,
            rabi_splitting_mev: rabi,
            cavity_detuning_mev: detuning,
            moire_potential_mev: v_m,
            monolayer_lattice_const_nm: 0.328,
            lattice_mismatch: 0.0015,
            exciton_mass_m0: 0.70,
            cavity_photon_energy_ev: 1.65,
            valley_zeeman_splitting_mev: zeeman,
        };
        let lattice_solver = MoirePolaritonLatticeSolver::new(lattice_params.clone());
        let lattice_metrics = lattice_solver.evaluate_metrics();
        let dispersion = lattice_solver.compute_dispersion(32);
        let berry = lattice_solver.compute_berry_curvature_profile(32);

        let edge_params = ValleyHallEdgeParams {
            domain_boundary_length_um: len_um,
            bulk_valley_gap_mev: gap,
            edge_mode_velocity_ms: v_edge,
            intervalley_disorder_strength: disorder,
            bend_angle_deg: bend,
            has_sharp_obstacle: obst,
            pump_polarization: pol,
        };
        let edge_solver = ValleyHallEdgeSolver::new(edge_params.clone());
        let edge_metrics = edge_solver.evaluate_metrics();
        let edge_disp = edge_solver.compute_edge_dispersion(32);
        let spectrum = edge_solver.compute_transmission_spectrum(32);
        let spatial = edge_solver.compute_spatial_profiles(32);

        let lasing_params = ChiralLasingParams {
            pump_power_uw_um2: pump,
            threshold_power_uw_um2: p_th,
            polariton_lifetime_ps: tau,
            reservoir_decay_rate_ghz: 1.2,
            polariton_interaction_uev_um2: g_pol,
            spontaneous_coupling_factor_beta: 0.08,
            pump_chirality_factor: chi,
        };
        let lasing_solver = ChiralLasingSolver::new(lasing_params.clone());
        let lasing_metrics = lasing_solver.evaluate_metrics();
        let io_curve = lasing_solver.compute_input_output_curve(32);
        let coherence = lasing_solver.compute_temporal_coherence(32);
        let angular = lasing_solver.compute_chiral_angular_profile(32);

        let synth_params = OptoAcousticSynthesizerParams {
            carrier_frequency_thz: carr,
            microwave_acoustic_freq_ghz: f_ac,
            acoustic_drive_power_mw: p_mw,
            optoacoustic_coupling_mhz: g_om,
            optical_cavity_linewidth_ghz: kappa,
            acoustic_damping_mhz: 0.80,
            piezo_transduction_efficiency: 0.88,
        };
        let synth_solver = OptoAcousticSynthesizerSolver::new(synth_params.clone());
        let synth_metrics = synth_solver.evaluate_metrics();
        let comb = synth_solver.compute_frequency_comb_spectrum();
        let phase_noise = synth_solver.compute_phase_noise_spectrum(32);
        let eff_curve = synth_solver.compute_efficiency_vs_power(32);

        let processor = MoirePolaritonLaserProcessor::new(
            lattice_params,
            edge_params,
            lasing_params,
            synth_params,
        );
        let audit = processor.audit_processor();

        Self {
            is_open: false,
            active_tab: MoirePolaritonDialogTab::MoirePolaritonBand,
            twist_angle_deg: twist,
            rabi_splitting_mev: rabi,
            cavity_detuning_mev: detuning,
            moire_potential_mev: v_m,
            valley_zeeman_mev: zeeman,

            boundary_length_um: len_um,
            bulk_valley_gap_mev: gap,
            edge_velocity_ms: v_edge,
            intervalley_disorder: disorder,
            bend_angle_deg: bend,
            has_sharp_obstacle: obst,
            pump_polarization: pol,

            pump_power_uw_um2: pump,
            threshold_power_uw_um2: p_th,
            polariton_lifetime_ps: tau,
            polariton_interaction_uev_um2: g_pol,
            pump_chirality_factor: chi,

            carrier_frequency_thz: carr,
            microwave_acoustic_freq_ghz: f_ac,
            acoustic_drive_power_mw: p_mw,
            optoacoustic_coupling_mhz: g_om,
            optical_cavity_linewidth_ghz: kappa,

            cached_lattice_metrics: lattice_metrics,
            cached_dispersion: dispersion,
            cached_berry_profile: berry,

            cached_edge_metrics: edge_metrics,
            cached_edge_dispersion: edge_disp,
            cached_waveguide_spectrum: spectrum,
            cached_spatial_profiles: spatial,

            cached_lasing_metrics: lasing_metrics,
            cached_io_curve: io_curve,
            cached_coherence_curve: coherence,
            cached_angular_profile: angular,

            cached_synth_metrics: synth_metrics,
            cached_frequency_comb: comb,
            cached_phase_noise: phase_noise,
            cached_efficiency_curve: eff_curve,

            cached_audit: audit,
            last_solve_time_us: 180.0,
        }
    }

    /// Recomputes all simulation states when parameters change.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let lattice_params = MoirePolaritonParams {
            twist_angle_deg: self.twist_angle_deg,
            rabi_splitting_mev: self.rabi_splitting_mev,
            cavity_detuning_mev: self.cavity_detuning_mev,
            moire_potential_mev: self.moire_potential_mev,
            monolayer_lattice_const_nm: 0.328,
            lattice_mismatch: 0.0015,
            exciton_mass_m0: 0.70,
            cavity_photon_energy_ev: 1.65,
            valley_zeeman_splitting_mev: self.valley_zeeman_mev,
        };
        let lattice_solver = MoirePolaritonLatticeSolver::new(lattice_params.clone());
        self.cached_lattice_metrics = lattice_solver.evaluate_metrics();
        self.cached_dispersion = lattice_solver.compute_dispersion(32);
        self.cached_berry_profile = lattice_solver.compute_berry_curvature_profile(32);

        let edge_params = ValleyHallEdgeParams {
            domain_boundary_length_um: self.boundary_length_um,
            bulk_valley_gap_mev: self.bulk_valley_gap_mev,
            edge_mode_velocity_ms: self.edge_velocity_ms,
            intervalley_disorder_strength: self.intervalley_disorder,
            bend_angle_deg: self.bend_angle_deg,
            has_sharp_obstacle: self.has_sharp_obstacle,
            pump_polarization: self.pump_polarization,
        };
        let edge_solver = ValleyHallEdgeSolver::new(edge_params.clone());
        self.cached_edge_metrics = edge_solver.evaluate_metrics();
        self.cached_edge_dispersion = edge_solver.compute_edge_dispersion(32);
        self.cached_waveguide_spectrum = edge_solver.compute_transmission_spectrum(32);
        self.cached_spatial_profiles = edge_solver.compute_spatial_profiles(32);

        let lasing_params = ChiralLasingParams {
            pump_power_uw_um2: self.pump_power_uw_um2,
            threshold_power_uw_um2: self.threshold_power_uw_um2,
            polariton_lifetime_ps: self.polariton_lifetime_ps,
            reservoir_decay_rate_ghz: 1.2,
            polariton_interaction_uev_um2: self.polariton_interaction_uev_um2,
            spontaneous_coupling_factor_beta: 0.08,
            pump_chirality_factor: self.pump_chirality_factor,
        };
        let lasing_solver = ChiralLasingSolver::new(lasing_params.clone());
        self.cached_lasing_metrics = lasing_solver.evaluate_metrics();
        self.cached_io_curve = lasing_solver.compute_input_output_curve(32);
        self.cached_coherence_curve = lasing_solver.compute_temporal_coherence(32);
        self.cached_angular_profile = lasing_solver.compute_chiral_angular_profile(32);

        let synth_params = OptoAcousticSynthesizerParams {
            carrier_frequency_thz: self.carrier_frequency_thz,
            microwave_acoustic_freq_ghz: self.microwave_acoustic_freq_ghz,
            acoustic_drive_power_mw: self.acoustic_drive_power_mw,
            optoacoustic_coupling_mhz: self.optoacoustic_coupling_mhz,
            optical_cavity_linewidth_ghz: self.optical_cavity_linewidth_ghz,
            acoustic_damping_mhz: 0.80,
            piezo_transduction_efficiency: 0.88,
        };
        let synth_solver = OptoAcousticSynthesizerSolver::new(synth_params.clone());
        self.cached_synth_metrics = synth_solver.evaluate_metrics();
        self.cached_frequency_comb = synth_solver.compute_frequency_comb_spectrum();
        self.cached_phase_noise = synth_solver.compute_phase_noise_spectrum(32);
        self.cached_efficiency_curve = synth_solver.compute_efficiency_vs_power(32);

        let processor = MoirePolaritonLaserProcessor::new(
            lattice_params,
            edge_params,
            lasing_params,
            synth_params,
        );
        self.cached_audit = processor.audit_processor();

        self.last_solve_time_us = start.elapsed().as_secs_f64() * 1.0e6;
    }

    /// Renders modal window in the GUI.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Moiré Exciton-Polariton Valley Hall Chiral Lasing Metasurface & Opto-Acoustic Synthesizer (Phase 448)")
            .open(&mut open)
            .resizable(true)
            .default_width(960.0)
            .default_height(640.0)
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = open;
    }

    /// Primary render loop for dialog content.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                MoirePolaritonDialogTab::MoirePolaritonBand,
                MoirePolaritonDialogTab::ValleyHallEdgeWaveguide,
                MoirePolaritonDialogTab::ChiralPolaritonLaser,
                MoirePolaritonDialogTab::OptoAcousticSynthesizer,
                MoirePolaritonDialogTab::AuditTelemetry,
            ];

            for &tab in &tabs {
                let is_selected = self.active_tab == tab;
                if ui.selectable_label(is_selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            MoirePolaritonDialogTab::MoirePolaritonBand => self.render_tab_band(ui),
            MoirePolaritonDialogTab::ValleyHallEdgeWaveguide => self.render_tab_waveguide(ui),
            MoirePolaritonDialogTab::ChiralPolaritonLaser => self.render_tab_laser(ui),
            MoirePolaritonDialogTab::OptoAcousticSynthesizer => self.render_tab_synthesizer(ui),
            MoirePolaritonDialogTab::AuditTelemetry => self.render_tab_audit(ui),
        }
    }

    fn render_tab_band(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Moiré Polariton Band Structure");
                ui.add_space(4.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Twist Angle [deg]:");
                    changed |= ui.add(egui::Slider::new(&mut self.twist_angle_deg, 1.0..=4.0).step_by(0.1)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Rabi Splitting [meV]:");
                    changed |= ui.add(egui::Slider::new(&mut self.rabi_splitting_mev, 15.0..=40.0).step_by(1.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Cavity Detuning [meV]:");
                    changed |= ui.add(egui::Slider::new(&mut self.cavity_detuning_mev, -15.0..=15.0).step_by(0.5)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Moiré Potential [meV]:");
                    changed |= ui.add(egui::Slider::new(&mut self.moire_potential_mev, 15.0..=60.0).step_by(1.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Valley Zeeman [meV]:");
                    changed |= ui.add(egui::Slider::new(&mut self.valley_zeeman_mev, 0.0..=5.0).step_by(0.2)).changed();
                });

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Lattice Invariants & Metrics").strong());
                    ui.label(format!("Moiré Superlattice Period a_M: {:.2} nm", self.cached_lattice_metrics.moire_period_nm));
                    ui.label(format!("Effective Polariton Mass Ratio: {:.3e} m_0", self.cached_lattice_metrics.effective_polariton_mass_ratio));
                    ui.label(format!("Lower Polariton Energy: {:.4} eV", self.cached_lattice_metrics.lower_polariton_energy_ev));
                    ui.label(format!("Hopfield Exciton Fraction |X|^2: {:.2}%", self.cached_lattice_metrics.hopfield_exciton_fraction * 100.0));
                    ui.label(format!("Hopfield Photon Fraction |C|^2: {:.2}%", self.cached_lattice_metrics.hopfield_photon_fraction * 100.0));
                    ui.label(format!("Peak Valley Berry Curvature: {:.1} nm^2", self.cached_lattice_metrics.valley_berry_curvature_nm2));
                    ui.label(format!("Valley Chern Number C_v: +{}", self.cached_lattice_metrics.valley_chern_number));
                    ui.label(format!("Bulk Valley Polariton Gap: {:.2} meV", self.cached_lattice_metrics.bulk_valley_polariton_gap_mev));
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Moiré Exciton-Polariton Dispersion E(k)").strong());
                let pts_lp: PlotPoints = self.cached_dispersion.iter().map(|p| [p.k_normalized, p.lower_polariton_mev]).collect();
                let pts_up: PlotPoints = self.cached_dispersion.iter().map(|p| [p.k_normalized, p.upper_polariton_mev]).collect();
                let pts_photon: PlotPoints = self.cached_dispersion.iter().map(|p| [p.k_normalized, p.bare_photon_mev]).collect();

                Plot::new("polariton_disp_plot")
                    .height(240.0)
                    .x_axis_label("Normalized Momentum k / k_BZ")
                    .y_axis_label("Energy [meV]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Lower Polariton", pts_lp).color(Color32::from_rgb(100, 200, 255)));
                        plot_ui.line(Line::new("Upper Polariton", pts_up).color(Color32::from_rgb(255, 120, 120)));
                        plot_ui.line(Line::new("Bare Cavity Photon", pts_photon).color(Color32::from_rgb(140, 140, 140)));
                    });

                ui.add_space(4.0);
                ui.label(RichText::new("Valley Berry Curvature Distribution Omega(k)").strong());
                let pts_bk: PlotPoints = self.cached_berry_profile.iter().map(|&(k, om_k, _)| [k, om_k]).collect();
                let pts_bkp: PlotPoints = self.cached_berry_profile.iter().map(|&(k, _, om_kp)| [k, om_kp]).collect();

                Plot::new("berry_curvature_plot")
                    .height(180.0)
                    .x_axis_label("Momentum k / k_BZ")
                    .y_axis_label("Berry Curvature [nm^2]")
                    .show(ui, |plot_ui| {
                        plot_ui.hline(HLine::new("Zero Axis", 0.0).color(Color32::from_rgb(100, 100, 100)));
                        plot_ui.line(Line::new("Valley K (+)", pts_bk).color(Color32::from_rgb(100, 255, 120)));
                        plot_ui.line(Line::new("Valley K' (-)", pts_bkp).color(Color32::from_rgb(255, 100, 100)));
                    });
            });
        });
    }

    fn render_tab_waveguide(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Valley Hall Topological Edge Waveguide");
                ui.add_space(4.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Boundary Length [um]:");
                    changed |= ui.add(egui::Slider::new(&mut self.boundary_length_um, 5.0..=40.0).step_by(1.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Bulk Valley Gap [meV]:");
                    changed |= ui.add(egui::Slider::new(&mut self.bulk_valley_gap_mev, 8.0..=30.0).step_by(1.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Edge Mode Velocity [km/s]:");
                    let mut vel_kms = self.edge_velocity_ms / 1.0e3;
                    if ui.add(egui::Slider::new(&mut vel_kms, 80.0..=300.0).step_by(5.0)).changed() {
                        self.edge_velocity_ms = vel_kms * 1.0e3;
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Intervalley Disorder:");
                    changed |= ui.add(egui::Slider::new(&mut self.intervalley_disorder, 0.005..=0.15).step_by(0.005)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Corner Bend Angle [deg]:");
                    changed |= ui.add(egui::Slider::new(&mut self.bend_angle_deg, 30.0..=120.0).step_by(15.0)).changed();
                });

                ui.horizontal(|ui| {
                    changed |= ui.checkbox(&mut self.has_sharp_obstacle, "Sharp Vacancy Obstacle").changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Pump Polarization:");
                    let old_pol = self.pump_polarization;
                    egui::ComboBox::from_id_salt("pump_pol_combo")
                        .selected_text(self.pump_polarization.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.pump_polarization, CircularPolarization::SigmaPlus, CircularPolarization::SigmaPlus.name());
                            ui.selectable_value(&mut self.pump_polarization, CircularPolarization::SigmaMinus, CircularPolarization::SigmaMinus.name());
                            ui.selectable_value(&mut self.pump_polarization, CircularPolarization::Linear, CircularPolarization::Linear.name());
                        });
                    if old_pol != self.pump_polarization {
                        changed = true;
                    }
                });

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Edge Waveguide Telemetry").strong());
                    ui.label(format!("Chiral Group Velocity: {:.1} km/s", self.cached_edge_metrics.chiral_group_velocity_ms / 1.0e3));
                    ui.label(format!("Propagation Loss: {:.2} dB/um", self.cached_edge_metrics.propagation_loss_db_per_um));
                    ui.label(format!("Sharp-Bend Transmission: {:.2}%", self.cached_edge_metrics.sharp_bend_transmission_ratio * 100.0));
                    ui.label(format!("Chiral Valley Isolation: {:.1} dB", self.cached_edge_metrics.chiral_valley_isolation_db));
                    ui.label(format!("Intervalley Backscattering: {:.3e}", self.cached_edge_metrics.intervalley_backscattering_probability));
                    ui.label(format!("Topological Figure of Merit: {:.3}", self.cached_edge_metrics.topological_figure_of_merit));
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Chiral Edge Mode Dispersion Across Bulk Gap").strong());
                let pts_k: PlotPoints = self.cached_edge_dispersion.iter().map(|p| [p.k_edge_um_inv, p.valley_k_branch_mev]).collect();
                let pts_kp: PlotPoints = self.cached_edge_dispersion.iter().map(|p| [p.k_edge_um_inv, p.valley_kp_branch_mev]).collect();
                let pts_c: PlotPoints = self.cached_edge_dispersion.iter().map(|p| [p.k_edge_um_inv, p.bulk_conduction_edge_mev]).collect();
                let pts_v: PlotPoints = self.cached_edge_dispersion.iter().map(|p| [p.k_edge_um_inv, p.bulk_valence_edge_mev]).collect();

                Plot::new("edge_disp_plot")
                    .height(240.0)
                    .x_axis_label("In-Plane Wavevector k [1/um]")
                    .y_axis_label("Energy [meV]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Bulk Conduction Band", pts_c).color(Color32::from_rgb(120, 120, 140)));
                        plot_ui.line(Line::new("Bulk Valence Band", pts_v).color(Color32::from_rgb(120, 120, 140)));
                        plot_ui.line(Line::new("Valley K Chiral Edge (+)", pts_k).color(Color32::from_rgb(100, 255, 120)));
                        plot_ui.line(Line::new("Valley K' Chiral Edge (-)", pts_kp).color(Color32::from_rgb(255, 100, 100)));
                    });

                ui.add_space(4.0);
                ui.label(RichText::new("Waveguide Transmission Spectrum S_21 [dB]").strong());
                let pts_pristine: PlotPoints = self.cached_waveguide_spectrum.iter().map(|p| [p.energy_mev, p.pristine_s21_db]).collect();
                let pts_bent: PlotPoints = self.cached_waveguide_spectrum.iter().map(|p| [p.energy_mev, p.bent_s21_db]).collect();
                let pts_refl: PlotPoints = self.cached_waveguide_spectrum.iter().map(|p| [p.energy_mev, p.reflection_s11_db]).collect();

                Plot::new("waveguide_trans_plot")
                    .height(180.0)
                    .x_axis_label("Energy Relative to Midgap [meV]")
                    .y_axis_label("Parameter [dB]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Pristine S_21", pts_pristine).color(Color32::from_rgb(100, 200, 255)));
                        plot_ui.line(Line::new("Corner Bend S_21", pts_bent).color(Color32::from_rgb(255, 200, 80)));
                        plot_ui.line(Line::new("Reflection S_11", pts_refl).color(Color32::from_rgb(220, 100, 100)));
                    });
            });
        });
    }

    fn render_tab_laser(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Non-Reciprocal Chiral Polariton Laser");
                ui.add_space(4.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Pump Power [uW/um^2]:");
                    changed |= ui.add(egui::Slider::new(&mut self.pump_power_uw_um2, 0.5..=15.0).step_by(0.5)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Threshold P_th [uW/um^2]:");
                    changed |= ui.add(egui::Slider::new(&mut self.threshold_power_uw_um2, 1.0..=5.0).step_by(0.2)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Polariton Lifetime [ps]:");
                    changed |= ui.add(egui::Slider::new(&mut self.polariton_lifetime_ps, 10.0..=50.0).step_by(2.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Interaction g_pol [ueV*um^2]:");
                    changed |= ui.add(egui::Slider::new(&mut self.polariton_interaction_uev_um2, 4.0..=30.0).step_by(2.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Chirality Bias chi [-1..+1]:");
                    changed |= ui.add(egui::Slider::new(&mut self.pump_chirality_factor, -1.0..=1.0).step_by(0.05)).changed();
                });

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Chiral Laser & BEC Condensation Telemetry").strong());
                    let status_text = if self.cached_lasing_metrics.is_above_threshold {
                        "ABOVE THRESHOLD (Coherent BEC Condensate)"
                    } else {
                        "BELOW THRESHOLD (Spontaneous Polariton Gas)"
                    };
                    let status_color = if self.cached_lasing_metrics.is_above_threshold {
                        Color32::from_rgb(80, 220, 120)
                    } else {
                        Color32::from_rgb(255, 180, 80)
                    };
                    ui.label(RichText::new(status_text).strong().color(status_color));
                    ui.label(format!("Condensate Density n_pol: {:.1} um^-2", self.cached_lasing_metrics.polariton_density_um2));
                    ui.label(format!("Laser Linewidth: {:.2} kHz", self.cached_lasing_metrics.emission_linewidth_khz));
                    ui.label(format!("Temporal Coherence Time tau_c: {:.1} ps", self.cached_lasing_metrics.temporal_coherence_time_ps));
                    ui.label(format!("Chiral Front-to-Back Ratio: {:.1} dB", self.cached_lasing_metrics.chiral_front_to_back_ratio_db));
                    ui.label(format!("Non-Linear Blueshift Delta E: {:.3} meV", self.cached_lasing_metrics.polariton_blueshift_mev));
                    ui.label(format!("Second-Order Coherence g^(2)(0): {:.3}", self.cached_lasing_metrics.second_order_coherence_g2_zero));
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Input-Output S-Curve & Linewidth Narrowing").strong());
                let pts_intensity: PlotPoints = self.cached_io_curve.iter().map(|p| [p.pump_power_uw_um2, p.emission_intensity_arb]).collect();
                let pts_linewidth: PlotPoints = self.cached_io_curve.iter().map(|p| [p.pump_power_uw_um2, p.linewidth_khz / 100.0]).collect();

                Plot::new("io_curve_plot")
                    .height(240.0)
                    .x_axis_label("Excitation Pump Power [uW/um^2]")
                    .y_axis_label("Arbitrary Scale")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Emission Intensity", pts_intensity).color(Color32::from_rgb(120, 255, 140)));
                        plot_ui.line(Line::new("Linewidth [x100 kHz]", pts_linewidth).color(Color32::from_rgb(255, 100, 120)));
                    });

                ui.add_space(4.0);
                ui.label(RichText::new("First-Order Temporal Coherence |g^(1)(tau)|").strong());
                let pts_g1: PlotPoints = self.cached_coherence_curve.iter().map(|p| [p.delay_time_ps, p.first_order_coherence_g1]).collect();

                Plot::new("coherence_plot")
                    .height(180.0)
                    .x_axis_label("Time Delay tau [ps]")
                    .y_axis_label("|g^(1)(tau)| [0.0 - 1.0]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("|g^(1)(tau)|", pts_g1).color(Color32::from_rgb(180, 120, 255)));
                    });
            });
        });
    }

    fn render_tab_synthesizer(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Opto-Acoustic Frequency Synthesizer");
                ui.add_space(4.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Carrier Frequency [THz]:");
                    changed |= ui.add(egui::Slider::new(&mut self.carrier_frequency_thz, 350.0..=480.0).step_by(10.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Microwave Acoustic Freq [GHz]:");
                    changed |= ui.add(egui::Slider::new(&mut self.microwave_acoustic_freq_ghz, 2.0..=10.0).step_by(0.2)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Acoustic Drive Power [mW]:");
                    changed |= ui.add(egui::Slider::new(&mut self.acoustic_drive_power_mw, 0.5..=10.0).step_by(0.5)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Coupling Rate g_om [MHz]:");
                    changed |= ui.add(egui::Slider::new(&mut self.optoacoustic_coupling_mhz, 5.0..=35.0).step_by(1.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Cavity Linewidth kappa [GHz]:");
                    changed |= ui.add(egui::Slider::new(&mut self.optical_cavity_linewidth_ghz, 5.0..=25.0).step_by(1.0)).changed();
                });

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Quantum Transduction & Comb Telemetry").strong());
                    ui.label(format!("Modulation Index beta: {:.2} rad", self.cached_synth_metrics.modulation_index_beta));
                    ui.label(format!("Carrier Suppression: {:.1} dB", self.cached_synth_metrics.carrier_suppression_ratio_db));
                    ui.label(format!("First Sideband Fraction P(+/-1): {:.2}%", self.cached_synth_metrics.first_sideband_power_fraction * 100.0));
                    ui.label(format!("Second Sideband Fraction P(+/-2): {:.2}%", self.cached_synth_metrics.second_sideband_power_fraction * 100.0));
                    ui.label(format!("Microwave-to-Optical Efficiency: {:.2}%", self.cached_synth_metrics.microwave_to_optical_efficiency * 100.0));
                    ui.label(format!("Phase Noise at 10 kHz Offset: {:.1} dBc/Hz", self.cached_synth_metrics.phase_noise_at_10khz_dbc_hz));
                    ui.label(format!("Quantum Added Noise: {:.2} quanta", self.cached_synth_metrics.quantum_added_noise_quanta));
                    ui.label(format!("Optomechanical Cooperativity C_om: {:.2}", self.cached_synth_metrics.optomechanical_cooperativity));
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Synthesized Optical Frequency Comb Spectrum").strong());
                let pts_comb: PlotPoints = self.cached_frequency_comb.iter().map(|p| [p.offset_ghz, p.power_db]).collect();

                Plot::new("comb_spectrum_plot")
                    .height(240.0)
                    .x_axis_label("Frequency Offset from Carrier [GHz]")
                    .y_axis_label("Spectral Power [dB]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Optical Comb Lines", pts_comb).color(Color32::from_rgb(100, 220, 255)));
                    });

                ui.add_space(4.0);
                ui.label(RichText::new("Single-Sideband Phase Noise Spectrum L(Delta f)").strong());
                let pts_pn: PlotPoints = self.cached_phase_noise.iter().map(|p| [p.offset_freq_khz, p.phase_noise_dbc_hz]).collect();

                Plot::new("phase_noise_plot")
                    .height(180.0)
                    .x_axis_label("Offset Frequency Delta f [kHz]")
                    .y_axis_label("Phase Noise [dBc/Hz]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Phase Noise", pts_pn).color(Color32::from_rgb(255, 180, 80)));
                    });
            });
        });
    }

    fn render_tab_audit(&mut self, ui: &mut Ui) {
        ui.heading("Rigorous 10-Point Physics Audit Checklist");
        ui.add_space(6.0);

        let pass_color = Color32::from_rgb(80, 220, 120);
        let fail_color = Color32::from_rgb(255, 80, 80);

        let checks = [
            ("1. Moiré Exciton-Polariton Rabi Splitting (hbar * Omega_R >= 15.0 meV)", self.cached_audit.moire_rabi_splitting_pass, "Strong coupling verified in TMD moiré microcavity"),
            ("2. Valley Hall Berry Curvature Quantization (|C_v| = 1)", self.cached_audit.valley_berry_quantization_pass, "Quantized valley Chern number establishes topological phase"),
            ("3. Bulk Topological Valley Gap (Delta_gap >= 8.0 meV)", self.cached_audit.bulk_valley_gap_pass, "Robust bulk bandgap protects helical valley edge states"),
            ("4. Valley-Locked Edge Dispersion Linearity (v_g >= 1.0e5 m/s)", self.cached_audit.valley_edge_velocity_pass, "High group velocity ensures rapid coherent polariton transport"),
            ("5. Sharp-Bend Defect Immunity (T_bend >= 94.0% around 60/120 deg)", self.cached_audit.sharp_bend_immunity_pass, "Intervalley scattering suppressed around sharp corners"),
            ("6. Chiral Valley Isolation (>= 25.0 dB)", self.cached_audit.chiral_valley_isolation_pass, "High polarization selectivity between K and K' channels"),
            ("7. Polariton Lasing Threshold (P_th <= 5.0 uW/um^2)", self.cached_audit.polariton_lasing_threshold_pass, "Low-threshold Bose-Einstein condensation verified"),
            ("8. Unidirectional Lasing Directionality (>= 25.0 dB)", self.cached_audit.chiral_lasing_directionality_pass, "Gain competition locks unidirectional chiral emission"),
            ("9. Opto-Acoustic Modulation Depth (beta >= 0.80 rad)", self.cached_audit.opto_acoustic_modulation_pass, "Strong acoustic deformation potential generates optical sidebands"),
            ("10. Microwave-to-Optical Transduction Efficiency (eta_trans >= 15.0%)", self.cached_audit.microwave_to_optical_transduction_pass, "High optomechanical cooperativity enables coherent quantum conversion"),
        ];

        for (title, passed, desc) in checks {
            ui.horizontal(|ui| {
                let badge = if passed { "[PASS]" } else { "[FAIL]" };
                let color = if passed { pass_color } else { fail_color };
                ui.label(RichText::new(badge).strong().color(color));
                ui.label(RichText::new(title).strong());
                ui.label(RichText::new(format!("- {}", desc)).color(Color32::from_rgb(160, 160, 160)));
            });
            ui.add_space(2.0);
        }

        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            let score_text = format!("Total Audit Score: {}/10 Passed", self.cached_audit.total_score);
            let score_color = if self.cached_audit.all_passed { pass_color } else { fail_color };
            ui.label(RichText::new(score_text).heading().color(score_color));
            if self.cached_audit.all_passed {
                ui.label(RichText::new("- All physics invariants fully verified!").color(pass_color));
            }
        });
    }
}
