#![deny(unsafe_code)]

//! Phase 455: Floquet Chiral Magnon-Phonon Polariton Router & Dissipative Quantum Memory Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring:
//! 1. Floquet chiral magnon-phonon polariton dispersion under rotating microwave drive,
//!    breaking time-reversal symmetry with large hybridization gap (>= 35.0 MHz) and
//!    wavenumber non-reciprocity (>= 0.15 um^-1).
//! 2. 4-terminal non-reciprocal cyclic acoustic circulator crossbar (1 -> 2 -> 3 -> 4 -> 1)
//!    with low insertion loss (<= 0.40 dB) and high cross-isolation (>= 40.0 dB).
//! 3. Dynamic synthetic gauge field non-Abelian Majorana zero mode braiding in parameter
//!    space with Artin relation error <= 1.0e-5 and gate fidelity >= 99.9%.
//! 4. Dissipative cryogenic topological quantum memory at 20 mK with engineered reservoir
//!    evacuation, achieving retention lifetime tau_ret >= 60.0 us and low thermal occupancy.
//! 5. 10-point rigorous physics audit checklist and real-time execution telemetry.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::floquet_magnon_memory::{
    DissipativeMemoryMetrics, DissipativeMemoryParams, DissipativeMemorySolver,
    DissipativeRetentionCurvePoint, FloquetMagnonMemoryAuditReport,
    FloquetMagnonMemoryProcessor, FloquetPolaritonDispersionParams,
    FloquetPolaritonDispersionPoint, FloquetPolaritonDispersionSolver,
    FloquetPolaritonMetrics, FourTerminalCirculatorMetrics,
    FourTerminalCirculatorParams, FourTerminalCirculatorSolver,
    FourTerminalSMatrixPoint, SyntheticGaugeBraidMetrics,
    SyntheticGaugeBraidParams, SyntheticGaugeBraidSolver,
    SyntheticGaugeTrajectoryPoint,
};

/// 5 Categorized navigation tabs for the Floquet Magnon Memory dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloquetMagnonMemoryTab {
    PolaritonDispersion,
    FourTerminalCirculator,
    SyntheticGaugeBraiding,
    DissipativeMemory,
    AuditTelemetry,
}

impl FloquetMagnonMemoryTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PolaritonDispersion => "Polariton Dispersion",
            Self::FourTerminalCirculator => "4-Terminal Circulator",
            Self::SyntheticGaugeBraiding => "Synthetic Braiding",
            Self::DissipativeMemory => "Dissipative Memory",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Floquet Magnon-Phonon Polariton Router & Quantum Memory (Phase 455).
#[derive(Debug, Clone)]
pub struct FloquetMagnonMemoryDialog {
    pub is_open: bool,
    pub active_tab: FloquetMagnonMemoryTab,

    // Tab 1: Floquet Dispersion parameters
    pub bare_phonon_freq_ghz: f64,
    pub bare_magnon_freq_ghz: f64,
    pub magnetoelastic_coupling_mhz: f64,
    pub floquet_drive_freq_ghz: f64,
    pub floquet_drive_amplitude_oe: f64,

    // Tab 2: 4-Terminal Circulator parameters
    pub center_frequency_ghz: f64,
    pub circulation_bandwidth_mhz: f64,
    pub junction_radius_um: f64,
    pub corner_obstacle_present: bool,

    // Tab 3: Synthetic Gauge Braiding parameters
    pub mode_count: usize,
    pub modulation_frequency_mhz: f64,
    pub synthetic_flux_bias_rad: f64,
    pub braid_cycle_period_ns: f64,
    pub phase_noise_variance_rad: f64,

    // Tab 4: Dissipative Memory parameters
    pub base_temperature_k: f64,
    pub engineered_dissipation_rate_mhz: f64,
    pub poison_evacuation_factor: f64,
    pub dispersive_readout_chi_mhz: f64,

    // Cached simulation outputs
    pub cached_dispersion_metrics: FloquetPolaritonMetrics,
    pub cached_dispersion: Vec<FloquetPolaritonDispersionPoint>,

    pub cached_circulator_metrics: FourTerminalCirculatorMetrics,
    pub cached_s_params: Vec<FourTerminalSMatrixPoint>,

    pub cached_braid_metrics: SyntheticGaugeBraidMetrics,
    pub cached_trajectories: Vec<SyntheticGaugeTrajectoryPoint>,

    pub cached_memory_metrics: DissipativeMemoryMetrics,
    pub cached_retention_curve: Vec<DissipativeRetentionCurvePoint>,

    pub cached_audit: FloquetMagnonMemoryAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for FloquetMagnonMemoryDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl FloquetMagnonMemoryDialog {
    /// Constructs a fast cold-boot instance under 2.0 ms by seeding pre-computed baseline state.
    pub fn new_fast() -> Self {
        let disp_params = FloquetPolaritonDispersionParams::default();
        let circ_params = FourTerminalCirculatorParams::default();
        let braid_params = SyntheticGaugeBraidParams::default();
        let mem_params = DissipativeMemoryParams::default();

        let disp_solver = FloquetPolaritonDispersionSolver::new(disp_params.clone());
        let circ_solver = FourTerminalCirculatorSolver::new(circ_params.clone());
        let braid_solver = SyntheticGaugeBraidSolver::new(braid_params.clone());
        let mem_solver = DissipativeMemorySolver::new(mem_params.clone());

        let processor = FloquetMagnonMemoryProcessor::new(
            disp_params.clone(),
            circ_params.clone(),
            braid_params.clone(),
            mem_params.clone(),
        );

        let cached_dispersion_metrics = disp_solver.evaluate_metrics();
        let cached_dispersion = disp_solver.compute_dispersion(60);

        let cached_circulator_metrics = circ_solver.evaluate_metrics();
        let cached_s_params = circ_solver.compute_s_parameters(60);

        let cached_braid_metrics = braid_solver.evaluate_metrics();
        let cached_trajectories = braid_solver.compute_synthetic_trajectory(60);

        let cached_memory_metrics = mem_solver.evaluate_metrics();
        let cached_retention_curve = mem_solver.compute_retention_curve(40);

        let cached_audit = processor.evaluate_audit();

        Self {
            is_open: false,
            active_tab: FloquetMagnonMemoryTab::PolaritonDispersion,

            bare_phonon_freq_ghz: disp_params.bare_phonon_freq_ghz,
            bare_magnon_freq_ghz: disp_params.bare_magnon_freq_ghz,
            magnetoelastic_coupling_mhz: disp_params.magnetoelastic_coupling_mhz,
            floquet_drive_freq_ghz: disp_params.floquet_drive_freq_ghz,
            floquet_drive_amplitude_oe: disp_params.floquet_drive_amplitude_oe,

            center_frequency_ghz: circ_params.center_frequency_ghz,
            circulation_bandwidth_mhz: circ_params.circulation_bandwidth_mhz,
            junction_radius_um: circ_params.junction_radius_um,
            corner_obstacle_present: circ_params.corner_obstacle_present,

            mode_count: braid_params.mode_count,
            modulation_frequency_mhz: braid_params.modulation_frequency_mhz,
            synthetic_flux_bias_rad: braid_params.synthetic_flux_bias_rad,
            braid_cycle_period_ns: braid_params.braid_cycle_period_ns,
            phase_noise_variance_rad: braid_params.phase_noise_variance_rad,

            base_temperature_k: mem_params.base_temperature_k,
            engineered_dissipation_rate_mhz: mem_params.engineered_dissipation_rate_mhz,
            poison_evacuation_factor: mem_params.poison_evacuation_factor,
            dispersive_readout_chi_mhz: mem_params.dispersive_readout_chi_mhz,

            cached_dispersion_metrics,
            cached_dispersion,
            cached_circulator_metrics,
            cached_s_params,
            cached_braid_metrics,
            cached_trajectories,
            cached_memory_metrics,
            cached_retention_curve,
            cached_audit,
            last_solve_time_us: 138.0,
        }
    }

    /// Recomputes all active solvers and refreshes cached telemetry.
    pub fn recompute_all(&mut self) {
        let t_start = std::time::Instant::now();

        let disp_params = FloquetPolaritonDispersionParams {
            bare_phonon_freq_ghz: self.bare_phonon_freq_ghz,
            bare_magnon_freq_ghz: self.bare_magnon_freq_ghz,
            magnetoelastic_coupling_mhz: self.magnetoelastic_coupling_mhz,
            floquet_drive_freq_ghz: self.floquet_drive_freq_ghz,
            floquet_drive_amplitude_oe: self.floquet_drive_amplitude_oe,
            gilbert_damping: 1.2e-4,
            acoustic_loss_factor: 6.0e-5,
            acoustic_velocity_ms: 3600.0,
        };

        let circ_params = FourTerminalCirculatorParams {
            center_frequency_ghz: self.center_frequency_ghz,
            circulation_bandwidth_mhz: self.circulation_bandwidth_mhz,
            junction_radius_um: self.junction_radius_um,
            acoustic_impedance_ohms: 50.0,
            corner_obstacle_present: self.corner_obstacle_present,
        };

        let braid_params = SyntheticGaugeBraidParams {
            mode_count: self.mode_count,
            modulation_frequency_mhz: self.modulation_frequency_mhz,
            synthetic_flux_bias_rad: self.synthetic_flux_bias_rad,
            braid_cycle_period_ns: self.braid_cycle_period_ns,
            phase_noise_variance_rad: self.phase_noise_variance_rad,
        };

        let mem_params = DissipativeMemoryParams {
            base_temperature_k: self.base_temperature_k,
            center_frequency_ghz: self.center_frequency_ghz,
            engineered_dissipation_rate_mhz: self.engineered_dissipation_rate_mhz,
            poison_evacuation_factor: self.poison_evacuation_factor,
            dispersive_readout_chi_mhz: self.dispersive_readout_chi_mhz,
        };

        let disp_solver = FloquetPolaritonDispersionSolver::new(disp_params.clone());
        let circ_solver = FourTerminalCirculatorSolver::new(circ_params.clone());
        let braid_solver = SyntheticGaugeBraidSolver::new(braid_params.clone());
        let mem_solver = DissipativeMemorySolver::new(mem_params.clone());

        let processor = FloquetMagnonMemoryProcessor::new(
            disp_params,
            circ_params,
            braid_params,
            mem_params,
        );

        self.cached_dispersion_metrics = disp_solver.evaluate_metrics();
        self.cached_dispersion = disp_solver.compute_dispersion(80);

        self.cached_circulator_metrics = circ_solver.evaluate_metrics();
        self.cached_s_params = circ_solver.compute_s_parameters(80);

        self.cached_braid_metrics = braid_solver.evaluate_metrics();
        self.cached_trajectories = braid_solver.compute_synthetic_trajectory(80);

        self.cached_memory_metrics = mem_solver.evaluate_metrics();
        self.cached_retention_curve = mem_solver.compute_retention_curve(50);

        self.cached_audit = processor.evaluate_audit();
        self.last_solve_time_us = t_start.elapsed().as_micros() as f64;
    }

    /// Convenience alias for `recompute_all`.
    pub fn recompute(&mut self) {
        self.recompute_all();
    }

    /// Renders the modal dialog window into the active egui Context.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Floquet Chiral Polariton Router & Quantum Memory (Phase 455)")
            .open(&mut open)
            .default_size([980.0, 720.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = open;
    }

    /// Compatibility wrapper for CAD studio modal manager.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Direct render method for embedding or headless testing.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        self.render_content(ui);
    }

    /// Primary content renderer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Floquet Chiral Magnon-Phonon Router & Dissipative Quantum Memory")
                    .color(Color32::from_rgb(100, 215, 245))
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (passed, total) = self.cached_audit.score();
                let badge_color = if passed == total {
                    Color32::from_rgb(60, 220, 110)
                } else {
                    Color32::from_rgb(240, 90, 80)
                };
                ui.label(
                    RichText::new(format!("Audit: {}/{} PASS", passed, total))
                        .color(badge_color)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("Latency: {:.1} us", self.last_solve_time_us))
                        .color(Color32::GRAY),
                );
            });
        });

        ui.separator();

        // Tab selection bar
        ui.horizontal(|ui| {
            let tabs = [
                FloquetMagnonMemoryTab::PolaritonDispersion,
                FloquetMagnonMemoryTab::FourTerminalCirculator,
                FloquetMagnonMemoryTab::SyntheticGaugeBraiding,
                FloquetMagnonMemoryTab::DissipativeMemory,
                FloquetMagnonMemoryTab::AuditTelemetry,
            ];

            for tab in tabs {
                let text = if self.active_tab == tab {
                    RichText::new(tab.label())
                        .color(Color32::from_rgb(90, 220, 255))
                        .strong()
                } else {
                    RichText::new(tab.label()).color(Color32::LIGHT_GRAY)
                };

                if ui.selectable_label(self.active_tab == tab, text).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            FloquetMagnonMemoryTab::PolaritonDispersion => {
                self.render_tab_dispersion(ui);
            }
            FloquetMagnonMemoryTab::FourTerminalCirculator => {
                self.render_tab_circulator(ui);
            }
            FloquetMagnonMemoryTab::SyntheticGaugeBraiding => {
                self.render_tab_braiding(ui);
            }
            FloquetMagnonMemoryTab::DissipativeMemory => {
                self.render_tab_memory(ui);
            }
            FloquetMagnonMemoryTab::AuditTelemetry => {
                self.render_tab_audit(ui);
            }
        }
    }

    fn render_tab_dispersion(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Floquet Chiral Acoustomagnonic Polariton Dispersion")
                    .strong()
                    .color(Color32::from_rgb(180, 220, 255)),
            );
            if ui.button("Recompute Dispersion").clicked() {
                self.recompute();
            }
        });

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(280.0);
                ui.label(RichText::new("Floquet Drive Parameters").strong());

                ui.add(
                    egui::Slider::new(&mut self.magnetoelastic_coupling_mhz, 20.0..=80.0)
                        .text("Coupling g_me (MHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.floquet_drive_amplitude_oe, 5.0..=40.0)
                        .text("Drive h_0 (Oe)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.floquet_drive_freq_ghz, 0.20..=1.50)
                        .text("Drive Omega (GHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.bare_phonon_freq_ghz, 3.0..=7.0)
                        .text("Acoustic f_0 (GHz)"),
                );

                ui.separator();
                ui.label(RichText::new("Non-Reciprocity Telemetry").strong());
                let m = &self.cached_dispersion_metrics;
                ui.label(format!("Polariton Gap: {:.2} MHz", m.polariton_hybridization_gap_mhz));
                ui.label(format!("Delta k: {:.3} um^-1", m.wavenumber_non_reciprocity_um_inv));
                ui.label(format!("Forward v_g: {:.0} m/s", m.forward_group_velocity_ms));
                ui.label(format!("Backward v_g: {:.0} m/s", m.backward_group_velocity_ms));
                ui.label(format!("Group Asymm: {:.1}%", m.group_velocity_asymmetry_ratio * 100.0));
                ui.label(format!("Insertion Loss: {:.2} dB", m.forward_insertion_loss_db));
                ui.label(format!("Backward Iso: {:.1} dB", m.backward_isolation_db));
            });

            ui.vertical(|ui| {
                let upper_pts: PlotPoints = self
                    .cached_dispersion
                    .iter()
                    .map(|p| [p.wavenumber_um_inv, p.upper_polariton_ghz])
                    .collect();
                let lower_pts: PlotPoints = self
                    .cached_dispersion
                    .iter()
                    .map(|p| [p.wavenumber_um_inv, p.lower_polariton_ghz])
                    .collect();
                let bwd_pts: PlotPoints = self
                    .cached_dispersion
                    .iter()
                    .map(|p| [p.wavenumber_um_inv, p.backward_branch_ghz])
                    .collect();

                Plot::new("floquet_polariton_dispersion_plot")
                    .height(380.0)
                    .x_axis_label("Wavenumber k (um^-1)")
                    .y_axis_label("Quasi-Energy (GHz)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Forward Upper Branch", upper_pts)
                                .color(Color32::from_rgb(80, 210, 240))
                                .width(2.5),
                        );
                        plot_ui.line(
                            Line::new("Forward Lower Branch", lower_pts)
                                .color(Color32::from_rgb(60, 220, 140))
                                .width(2.5),
                        );
                        plot_ui.line(
                            Line::new("Backward Branch (Time-Reversed)", bwd_pts)
                                .color(Color32::from_rgb(250, 100, 110))
                                .width(2.0),
                        );
                    });
            });
        });
    }

    fn render_tab_circulator(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("4-Terminal Cyclic Acoustic Circulator Crossbar")
                    .strong()
                    .color(Color32::from_rgb(180, 220, 255)),
            );
            if ui.button("Recompute Circulator").clicked() {
                self.recompute();
            }
        });

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(280.0);
                ui.label(RichText::new("Circulator Geometry").strong());

                ui.add(
                    egui::Slider::new(&mut self.circulation_bandwidth_mhz, 80.0..=220.0)
                        .text("3-dB BW (MHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.junction_radius_um, 60.0..=240.0)
                        .text("Radius (um)"),
                );
                ui.checkbox(&mut self.corner_obstacle_present, "Corner Defect Obstacle");

                ui.separator();
                ui.label(RichText::new("Circulator Performance").strong());
                let c = &self.cached_circulator_metrics;
                ui.label(format!("Insertion Loss: {:.2} dB", c.forward_insertion_loss_db));
                ui.label(format!("Cross Isolation: {:.1} dB", c.cross_terminal_isolation_db));
                ui.label(format!("Backward Isolation: {:.1} dB", c.backward_isolation_db));
                ui.label(format!("Return Loss: {:.1} dB", c.port_return_loss_db));
                ui.label(format!("3-dB Bandwidth: {:.1} MHz", c.circulation_bandwidth_3db_mhz));
                ui.label(format!("Corner Transmission: {:.1}%", c.corner_defect_transmission_pct));
                ui.label(format!("Cyclic Symmetry Dev: {:.3} dB", c.cyclic_symmetry_deviation_db));
            });

            ui.vertical(|ui| {
                let s21_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|p| [p.frequency_ghz, p.s21_forward_db])
                    .collect();
                let s31_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|p| [p.frequency_ghz, p.s31_cross_db])
                    .collect();
                let s41_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|p| [p.frequency_ghz, p.s41_backward_db])
                    .collect();
                let s11_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|p| [p.frequency_ghz, p.s11_return_loss_db])
                    .collect();

                Plot::new("four_terminal_s_matrix_plot")
                    .height(380.0)
                    .x_axis_label("Frequency (GHz)")
                    .y_axis_label("Scattering Parameter (dB)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Forward Transmission S21", s21_pts)
                                .color(Color32::from_rgb(60, 220, 110))
                                .width(2.5),
                        );
                        plot_ui.line(
                            Line::new("Cross Isolation S31", s31_pts)
                                .color(Color32::from_rgb(240, 160, 60))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("Backward Isolation S41", s41_pts)
                                .color(Color32::from_rgb(240, 80, 80))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("Return Loss S11", s11_pts)
                                .color(Color32::from_rgb(100, 180, 255))
                                .width(2.0),
                        );
                        plot_ui.hline(
                            HLine::new("Isolation Spec (-36 dB)", -36.0)
                                .color(Color32::DARK_GRAY),
                        );
                    });
            });
        });
    }

    fn render_tab_braiding(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Dynamic Synthetic Gauge Field Non-Abelian Braiding")
                    .strong()
                    .color(Color32::from_rgb(180, 220, 255)),
            );
            if ui.button("Recompute Braiding").clicked() {
                self.recompute();
            }
        });

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(280.0);
                ui.label(RichText::new("Gauge Modulation Parameters").strong());

                ui.add(
                    egui::Slider::new(&mut self.modulation_frequency_mhz, 5.0..=30.0)
                        .text("Mod Freq (MHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.braid_cycle_period_ns, 50.0..=250.0)
                        .text("Cycle Period (ns)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.phase_noise_variance_rad, 0.0005..=0.01)
                        .text("Phase Noise (rad)"),
                );

                ui.separator();
                ui.label(RichText::new("Synthetic Braiding Telemetry").strong());
                let b = &self.cached_braid_metrics;
                ui.label(format!("Artin Braid Error: {:.2e}", b.artin_relation_error));
                ui.label(format!("Far Commut Error: {:.2e}", b.far_commutation_error));
                ui.label(format!("Braiding Fidelity: {:.3}%", b.dynamic_braiding_fidelity_pct));
                ui.label(format!("Berry Phase: {:.3} rad", b.geometric_berry_phase_rad));
                ui.label(format!("Diabatic Leakage: {:.2e}", b.diabatic_leakage_prob));
            });

            ui.vertical(|ui| {
                let l1_pts: PlotPoints = self
                    .cached_trajectories
                    .iter()
                    .map(|p| [p.normalized_phase, p.lambda_1_mhz])
                    .collect();
                let l2_pts: PlotPoints = self
                    .cached_trajectories
                    .iter()
                    .map(|p| [p.normalized_phase, p.lambda_2_mhz])
                    .collect();
                let l3_pts: PlotPoints = self
                    .cached_trajectories
                    .iter()
                    .map(|p| [p.normalized_phase, p.lambda_3_mhz])
                    .collect();
                let gap_pts: PlotPoints = self
                    .cached_trajectories
                    .iter()
                    .map(|p| [p.normalized_phase, p.protection_gap_mhz])
                    .collect();

                Plot::new("synthetic_gauge_couplings_plot")
                    .height(380.0)
                    .x_axis_label("Normalized Modulation Phase theta / (2*pi)")
                    .y_axis_label("Coupling Strength lambda_j(t) (MHz)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("lambda_1(t)", l1_pts)
                                .color(Color32::from_rgb(100, 200, 255))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("lambda_2(t)", l2_pts)
                                .color(Color32::from_rgb(255, 140, 60))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("lambda_3(t)", l3_pts)
                                .color(Color32::from_rgb(220, 80, 240))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("Topological Protection Gap", gap_pts)
                                .color(Color32::from_rgb(60, 220, 110))
                                .width(2.5),
                        );
                    });
            });
        });
    }

    fn render_tab_memory(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Dissipative Cryogenic Non-Abelian Quantum Memory")
                    .strong()
                    .color(Color32::from_rgb(180, 220, 255)),
            );
            if ui.button("Recompute Memory").clicked() {
                self.recompute();
            }
        });

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(280.0);
                ui.label(RichText::new("Engineered Dissipation").strong());

                ui.add(
                    egui::Slider::new(&mut self.engineered_dissipation_rate_mhz, 1.0..=8.0)
                        .text("Bath Rate (MHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.base_temperature_k, 0.010..=0.080)
                        .text("Base Temp (K)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.dispersive_readout_chi_mhz, 2.0..=6.0)
                        .text("Readout chi (MHz)"),
                );

                ui.separator();
                ui.label(RichText::new("Memory Performance").strong());
                let m = &self.cached_memory_metrics;
                ui.label(format!("Retention Time: {:.1} us", m.memory_retention_time_us));
                ui.label(format!("Thermal Occupancy: {:.2e}", m.cryogenic_thermal_occupancy));
                ui.label(format!("Parity Readout: {:.1}%", m.parity_readout_contrast_pct));
                ui.label(format!("Dissipative Boost: {:.1}x", m.dissipative_enhancement_factor));
                ui.label(format!("Evacuation Rate: {:.1} MHz", m.quasiparticle_evacuation_rate_mhz));
                ui.label(format!("Fidelity at 100 us: {:.1}%", m.fidelity_at_100us_pct));
            });

            ui.vertical(|ui| {
                let diss_pts: PlotPoints = self
                    .cached_retention_curve
                    .iter()
                    .map(|p| [p.storage_time_us, p.retention_with_dissipation * 100.0])
                    .collect();
                let un_pts: PlotPoints = self
                    .cached_retention_curve
                    .iter()
                    .map(|p| [p.storage_time_us, p.retention_unassisted * 100.0])
                    .collect();

                Plot::new("dissipative_memory_retention_plot")
                    .height(380.0)
                    .x_axis_label("Storage Retention Delay (us)")
                    .y_axis_label("State Retention Probability (%)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("With Engineered Dissipation", diss_pts)
                                .color(Color32::from_rgb(60, 220, 110))
                                .width(2.5),
                        );
                        plot_ui.line(
                            Line::new("Unassisted Passive Decay", un_pts)
                                .color(Color32::from_rgb(240, 90, 80))
                                .width(2.0),
                        );
                        plot_ui.hline(
                            HLine::new("1/e Threshold", 36.79)
                                .color(Color32::DARK_GRAY),
                        );
                    });
            });
        });
    }

    fn render_tab_audit(&mut self, ui: &mut Ui) {
        let (passed, total) = self.cached_audit.score();

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("10-Point Rigorous Physics Invariant Audit")
                    .strong()
                    .color(Color32::from_rgb(180, 220, 255)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Run Full Re-Audit").clicked() {
                    self.recompute();
                }
            });
        });

        ui.add_space(8.0);

        let audit_items = [
            (
                "1. Floquet Polariton Hybridization Gap",
                format!("{:.2} MHz >= 35.0 MHz", self.cached_dispersion_metrics.polariton_hybridization_gap_mhz),
                self.cached_audit.polariton_hybridization_gap,
            ),
            (
                "2. Wavenumber Non-Reciprocity Delta_k",
                format!("{:.3} um^-1 >= 0.15 um^-1", self.cached_dispersion_metrics.wavenumber_non_reciprocity_um_inv),
                self.cached_audit.wavenumber_non_reciprocity,
            ),
            (
                "3. Forward Polariton Insertion Loss",
                format!("{:.2} dB <= 0.40 dB", self.cached_dispersion_metrics.forward_insertion_loss_db),
                self.cached_audit.forward_insertion_loss,
            ),
            (
                "4. Backward Non-Reciprocal Isolation",
                format!("{:.1} dB >= 36.0 dB", self.cached_dispersion_metrics.backward_isolation_db),
                self.cached_audit.backward_isolation,
            ),
            (
                "5. 4-Terminal Cross-Terminal Isolation",
                format!("{:.1} dB >= 40.0 dB", self.cached_circulator_metrics.cross_terminal_isolation_db),
                self.cached_audit.circulator_cross_isolation,
            ),
            (
                "6. Circulation 3-dB Bandwidth",
                format!("{:.1} MHz >= 120.0 MHz", self.cached_circulator_metrics.circulation_bandwidth_3db_mhz),
                self.cached_audit.circulator_bandwidth,
            ),
            (
                "7. Synthetic Gauge Artin Braid Error",
                format!("{:.2e} <= 1.0e-5 error", self.cached_braid_metrics.artin_relation_error),
                self.cached_audit.synthetic_gauge_artin_error,
            ),
            (
                "8. Dynamic Braiding Process Fidelity",
                format!("{:.3}% >= 99.9%", self.cached_braid_metrics.dynamic_braiding_fidelity_pct),
                self.cached_audit.synthetic_braiding_fidelity,
            ),
            (
                "9. Engineered Dissipative Retention Time",
                format!("{:.1} us >= 60.0 us", self.cached_memory_metrics.memory_retention_time_us),
                self.cached_audit.memory_retention_time,
            ),
            (
                "10. Cryogenic Thermal Noise Occupancy",
                format!("{:.2e} <= 0.05 quanta at 20 mK", self.cached_memory_metrics.cryogenic_thermal_occupancy),
                self.cached_audit.cryogenic_thermal_noise,
            ),
        ];

        egui::Grid::new("floquet_magnon_audit_grid")
            .striped(true)
            .min_col_width(260.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Audit Criterion").strong());
                ui.label(RichText::new("Evaluated Measurement").strong());
                ui.label(RichText::new("Status").strong());
                ui.end_row();

                for (name, val, pass) in audit_items {
                    ui.label(name);
                    ui.label(val);
                    if pass {
                        ui.label(RichText::new("PASS").color(Color32::from_rgb(60, 220, 110)).strong());
                    } else {
                        ui.label(RichText::new("FAIL").color(Color32::from_rgb(240, 80, 80)).strong());
                    }
                    ui.end_row();
                }
            });

        ui.add_space(12.0);
        ui.label(
            RichText::new(format!("Overall Physics Audit Score: {}/{} PASS", passed, total))
                .strong()
                .color(if passed == total {
                    Color32::from_rgb(80, 230, 130)
                } else {
                    Color32::from_rgb(240, 90, 80)
                }),
        );
    }
}
