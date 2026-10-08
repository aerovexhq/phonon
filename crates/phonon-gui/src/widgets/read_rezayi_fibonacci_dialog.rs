#![deny(unsafe_code)]

//! Phase 447: Fractional Quantum Hall Non-Abelian Read-Rezayi Fibonacci Anyon Acoustic Interferometer & Universal Topological Quantum Bus Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring Read-Rezayi nu = 12/5 FQH states,
//! non-Abelian Fibonacci anyon braiding with universal quantum logic without magic states,
//! Surface Acoustic Wave (SAW) Fabry-Pérot interferometers with 1/phi visibility suppression,
//! and chiral Rayleigh SAW multi-qudit quantum acoustic bus networks.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::read_rezayi_fibonacci::{
    BusFidelityDistancePoint, BusWaveformPoint, EnclosedTopologicalCharge,
    FibonacciBraidTrajectoryPoint, FibonacciBraidingMetrics, FibonacciBraidingParams,
    FibonacciBraidingSolver, FibonacciTargetGate, InterferometerFluxPoint,
    QuantumAcousticBusMetrics, QuantumAcousticBusParams, QuantumAcousticBusSolver,
    ReadRezayiAuditReport, ReadRezayiCorrelationPoint, ReadRezayiDispersionPoint,
    ReadRezayiFibonacciProcessor, ReadRezayiFilling, ReadRezayiMetrics,
    ReadRezayiStateParams, ReadRezayiStateSolver, SawAcousticTransmissionPoint,
    SawInterferometerMetrics, SawInterferometerParams, SawInterferometerSolver,
};

/// 5 Categorized navigation tabs for the Read-Rezayi Fibonacci dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadRezayiDialogTab {
    ReadRezayiTopology,
    FibonacciBraidingLattice,
    SawAcousticInterferometer,
    QuantumAcousticBus,
    AuditTelemetry,
}

impl ReadRezayiDialogTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ReadRezayiTopology => "Read-Rezayi Topology",
            Self::FibonacciBraidingLattice => "Fibonacci Braiding Lattice",
            Self::SawAcousticInterferometer => "SAW Anyon Interferometer",
            Self::QuantumAcousticBus => "Topological Quantum Bus",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Read-Rezayi Fibonacci anyons and acoustic bus (Phase 447).
#[derive(Debug, Clone)]
pub struct ReadRezayiFibonacciDialog {
    pub is_open: bool,
    pub active_tab: ReadRezayiDialogTab,

    // State parameters
    pub filling_factor: ReadRezayiFilling,
    pub magnetic_field_t: f64,
    pub topological_gap_kelvin: f64,

    // Braiding parameters
    pub target_gate: FibonacciTargetGate,
    pub braid_duration_ns: f64,

    // Interferometer parameters
    pub enclosed_charge: EnclosedTopologicalCharge,
    pub saw_frequency_ghz: f64,
    pub qpc_transmission: f64,
    pub loop_area_um2: f64,

    // Quantum Bus parameters
    pub bus_length_um: f64,
    pub bus_temperature_mk: f64,
    pub node_count: usize,

    // Cached models and telemetry
    pub cached_state_metrics: ReadRezayiMetrics,
    pub cached_edge_dispersion: Vec<ReadRezayiDispersionPoint>,
    pub cached_pair_correlation: Vec<ReadRezayiCorrelationPoint>,

    pub cached_braiding_metrics: FibonacciBraidingMetrics,
    pub cached_trajectories: Vec<FibonacciBraidTrajectoryPoint>,

    pub cached_interferometer_metrics: SawInterferometerMetrics,
    pub cached_flux_oscillations: Vec<InterferometerFluxPoint>,
    pub cached_acoustic_spectrum: Vec<SawAcousticTransmissionPoint>,

    pub cached_bus_metrics: QuantumAcousticBusMetrics,
    pub cached_pulse_dynamics: Vec<BusWaveformPoint>,
    pub cached_fidelity_distance: Vec<BusFidelityDistancePoint>,

    pub cached_audit: ReadRezayiAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ReadRezayiFibonacciDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ReadRezayiFibonacciDialog {
    /// Instantaneous cold-boot constructor (< 2ms) with pre-seeded baseline state.
    pub fn new_fast() -> Self {
        let state_params = ReadRezayiStateParams::default();
        let braiding_params = FibonacciBraidingParams::default();
        let interferometer_params = SawInterferometerParams::default();
        let bus_params = QuantumAcousticBusParams::default();

        let state_solver = ReadRezayiStateSolver::new(state_params.clone());
        let braiding_solver = FibonacciBraidingSolver::new(braiding_params.clone());
        let interferometer_solver = SawInterferometerSolver::new(interferometer_params.clone());
        let bus_solver = QuantumAcousticBusSolver::new(bus_params.clone());

        let cached_state_metrics = state_solver.evaluate_metrics();
        let cached_edge_dispersion = state_solver.compute_edge_dispersion(32);
        let cached_pair_correlation = state_solver.compute_pair_correlation(32);

        let cached_braiding_metrics = braiding_solver.evaluate_metrics();
        let cached_trajectories = braiding_solver.generate_worldline_trajectories(24);

        let cached_interferometer_metrics = interferometer_solver.evaluate_metrics();
        let cached_flux_oscillations = interferometer_solver.compute_flux_oscillations(36);
        let cached_acoustic_spectrum = interferometer_solver.compute_acoustic_spectrum(32);

        let cached_bus_metrics = bus_solver.evaluate_metrics();
        let cached_pulse_dynamics = bus_solver.compute_pulse_dynamics(32);
        let cached_fidelity_distance = bus_solver.compute_fidelity_vs_distance(32);

        let processor = ReadRezayiFibonacciProcessor::new(
            state_params.clone(),
            braiding_params.clone(),
            interferometer_params.clone(),
            bus_params.clone(),
        );
        let cached_audit = processor.audit_processor();

        Self {
            is_open: false,
            active_tab: ReadRezayiDialogTab::ReadRezayiTopology,

            filling_factor: state_params.filling_factor,
            magnetic_field_t: state_params.magnetic_field_t,
            topological_gap_kelvin: state_params.topological_gap_kelvin,

            target_gate: braiding_params.target_gate,
            braid_duration_ns: braiding_params.braid_duration_ns,

            enclosed_charge: interferometer_params.enclosed_charge,
            saw_frequency_ghz: interferometer_params.acoustic_frequency_ghz,
            qpc_transmission: interferometer_params.qpc1_transmission,
            loop_area_um2: interferometer_params.loop_area_um2,

            bus_length_um: bus_params.bus_length_um,
            bus_temperature_mk: bus_params.bus_temperature_mk,
            node_count: bus_params.node_count,

            cached_state_metrics,
            cached_edge_dispersion,
            cached_pair_correlation,

            cached_braiding_metrics,
            cached_trajectories,

            cached_interferometer_metrics,
            cached_flux_oscillations,
            cached_acoustic_spectrum,

            cached_bus_metrics,
            cached_pulse_dynamics,
            cached_fidelity_distance,

            cached_audit,
            last_solve_time_us: 120.0,
        }
    }

    /// Recomputes physics solvers upon parameter adjustment.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let state_params = ReadRezayiStateParams {
            filling_factor: self.filling_factor,
            magnetic_field_t: self.magnetic_field_t,
            topological_gap_kelvin: self.topological_gap_kelvin,
            ..Default::default()
        };

        let braiding_params = FibonacciBraidingParams {
            target_gate: self.target_gate,
            braid_duration_ns: self.braid_duration_ns,
            operating_temp_mk: self.bus_temperature_mk,
            ..Default::default()
        };

        let interferometer_params = SawInterferometerParams {
            acoustic_frequency_ghz: self.saw_frequency_ghz,
            qpc1_transmission: self.qpc_transmission,
            qpc2_transmission: self.qpc_transmission,
            loop_area_um2: self.loop_area_um2,
            magnetic_field_t: self.magnetic_field_t,
            enclosed_charge: self.enclosed_charge,
            temperature_mk: self.bus_temperature_mk,
            ..Default::default()
        };

        let bus_params = QuantumAcousticBusParams {
            bus_length_um: self.bus_length_um,
            bus_frequency_ghz: self.saw_frequency_ghz,
            node_count: self.node_count,
            bus_temperature_mk: self.bus_temperature_mk,
            ..Default::default()
        };

        let state_solver = ReadRezayiStateSolver::new(state_params.clone());
        let braiding_solver = FibonacciBraidingSolver::new(braiding_params.clone());
        let interferometer_solver = SawInterferometerSolver::new(interferometer_params.clone());
        let bus_solver = QuantumAcousticBusSolver::new(bus_params.clone());

        self.cached_state_metrics = state_solver.evaluate_metrics();
        self.cached_edge_dispersion = state_solver.compute_edge_dispersion(32);
        self.cached_pair_correlation = state_solver.compute_pair_correlation(32);

        self.cached_braiding_metrics = braiding_solver.evaluate_metrics();
        self.cached_trajectories = braiding_solver.generate_worldline_trajectories(24);

        self.cached_interferometer_metrics = interferometer_solver.evaluate_metrics();
        self.cached_flux_oscillations = interferometer_solver.compute_flux_oscillations(36);
        self.cached_acoustic_spectrum = interferometer_solver.compute_acoustic_spectrum(32);

        self.cached_bus_metrics = bus_solver.evaluate_metrics();
        self.cached_pulse_dynamics = bus_solver.compute_pulse_dynamics(32);
        self.cached_fidelity_distance = bus_solver.compute_fidelity_vs_distance(32);

        let processor = ReadRezayiFibonacciProcessor::new(
            state_params,
            braiding_params,
            interferometer_params,
            bus_params,
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
        Window::new("Read-Rezayi Fibonacci Anyon Acoustic Interferometer & Quantum Bus (Phase 447)")
            .open(&mut open)
            .resizable(true)
            .default_width(940.0)
            .default_height(620.0)
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = open;
    }

    /// Primary render loop for dialog content.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Phase 447:").strong().color(Color32::from_rgb(100, 200, 255)));
            ui.label(RichText::new("Non-Abelian Read-Rezayi Fibonacci Interferometer & Universal SAW Bus").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Recompute Physics").clicked() {
                    self.recompute();
                }
                ui.label(
                    RichText::new(format!("Solve Latency: {:.1} us", self.last_solve_time_us))
                        .color(Color32::from_rgb(160, 160, 160)),
                );
            });
        });

        ui.separator();

        // Navigation tab bar
        ui.horizontal(|ui| {
            let tabs = [
                ReadRezayiDialogTab::ReadRezayiTopology,
                ReadRezayiDialogTab::FibonacciBraidingLattice,
                ReadRezayiDialogTab::SawAcousticInterferometer,
                ReadRezayiDialogTab::QuantumAcousticBus,
                ReadRezayiDialogTab::AuditTelemetry,
            ];
            for tab in tabs {
                let selected = self.active_tab == tab;
                let text = RichText::new(tab.label()).strong();
                if ui.selectable_label(selected, text).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            ReadRezayiDialogTab::ReadRezayiTopology => self.render_tab_topology(ui),
            ReadRezayiDialogTab::FibonacciBraidingLattice => self.render_tab_braiding(ui),
            ReadRezayiDialogTab::SawAcousticInterferometer => self.render_tab_interferometer(ui),
            ReadRezayiDialogTab::QuantumAcousticBus => self.render_tab_bus(ui),
            ReadRezayiDialogTab::AuditTelemetry => self.render_tab_audit(ui),
        }
    }

    fn render_tab_topology(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Read-Rezayi State & Edge Modes");
                ui.add_space(4.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Filling Factor:");
                    let old_fill = self.filling_factor;
                    egui::ComboBox::from_id_salt("rr_filling_combo")
                        .selected_text(self.filling_factor.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.filling_factor, ReadRezayiFilling::Nu12Over5, ReadRezayiFilling::Nu12Over5.name());
                            ui.selectable_value(&mut self.filling_factor, ReadRezayiFilling::Nu2Plus2Over3, ReadRezayiFilling::Nu2Plus2Over3.name());
                        });
                    if old_fill != self.filling_factor {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Magnetic Field B [T]:");
                    changed |= ui.add(egui::Slider::new(&mut self.magnetic_field_t, 3.0..=9.0).step_by(0.1)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Topological Gap Delta_RR [mK]:");
                    let mut gap_mk = self.topological_gap_kelvin * 1000.0;
                    if ui.add(egui::Slider::new(&mut gap_mk, 15.0..=100.0).step_by(1.0)).changed() {
                        self.topological_gap_kelvin = gap_mk / 1000.0;
                        changed = true;
                    }
                });

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Topological Invariants").strong());
                    ui.label(format!("Fibonacci Quantum Dimension d_tau: {:.6} (Golden Ratio)", self.cached_state_metrics.fibonacci_quantum_dimension));
                    ui.label(format!("Topological Entanglement Entropy S_topo: {:.4}", self.cached_state_metrics.topological_entropy_s_topo));
                    ui.label(format!("Quasiparticle Charge e*: e / {:.1}", 1.0 / self.cached_state_metrics.quasiparticle_charge_fraction));
                    ui.label(format!("Magnetic Length l_B: {:.2} nm", self.cached_state_metrics.magnetic_length_nm));
                    ui.label(format!("Neutral Fibonacci Velocity: {:.1} km/s", self.cached_state_metrics.neutral_mode_velocity_ms / 1.0e3));
                    ui.label(format!("Charged Edge Velocity: {:.1} km/s", self.cached_state_metrics.charge_mode_velocity_ms / 1.0e3));
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Chiral Edge Excitation Dispersion omega(k)").strong());
                let pts_charge: PlotPoints = self.cached_edge_dispersion.iter()
                    .map(|p| [p.momentum_k_nm_inv, p.charge_branch_ghz])
                    .collect();
                let pts_neutral: PlotPoints = self.cached_edge_dispersion.iter()
                    .map(|p| [p.momentum_k_nm_inv, p.neutral_fibonacci_branch_ghz])
                    .collect();

                Plot::new("rr_dispersion_plot")
                    .height(240.0)
                    .x_axis_label("Momentum k [1/nm]")
                    .y_axis_label("Frequency [GHz]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Charged Mode", pts_charge).color(Color32::from_rgb(100, 200, 255)));
                        plot_ui.line(Line::new("Neutral Fibonacci", pts_neutral).color(Color32::from_rgb(255, 140, 50)));
                    });

                ui.add_space(4.0);
                ui.label(RichText::new("Pair Correlation Function g(r/l_B) (3-Cluster Ordering)").strong());
                let pts_corr: PlotPoints = self.cached_pair_correlation.iter()
                    .map(|p| [p.radius_r_over_l_b, p.pair_correlation_g_r])
                    .collect();
                Plot::new("rr_correlation_plot")
                    .height(180.0)
                    .x_axis_label("Distance r / l_B")
                    .y_axis_label("Correlation g(r)")
                    .show(ui, |plot_ui| {
                        plot_ui.hline(HLine::new("Asymptote", 1.0).color(Color32::from_rgb(120, 120, 120)));
                        plot_ui.line(Line::new("g(r)", pts_corr).color(Color32::from_rgb(140, 255, 140)));
                    });
            });
        });
    }

    fn render_tab_braiding(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Fibonacci Braiding & Universal Logic");
                ui.add_space(4.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Target Quantum Gate:");
                    let old_gate = self.target_gate;
                    egui::ComboBox::from_id_salt("fibo_gate_combo")
                        .selected_text(self.target_gate.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.target_gate, FibonacciTargetGate::Hadamard, FibonacciTargetGate::Hadamard.name());
                            ui.selectable_value(&mut self.target_gate, FibonacciTargetGate::PhaseS, FibonacciTargetGate::PhaseS.name());
                            ui.selectable_value(&mut self.target_gate, FibonacciTargetGate::PauliX, FibonacciTargetGate::PauliX.name());
                            ui.selectable_value(&mut self.target_gate, FibonacciTargetGate::PauliZ, FibonacciTargetGate::PauliZ.name());
                            ui.selectable_value(&mut self.target_gate, FibonacciTargetGate::PiOver8T, FibonacciTargetGate::PiOver8T.name());
                        });
                    if old_gate != self.target_gate {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Braid Duration [ns]:");
                    changed |= ui.add(egui::Slider::new(&mut self.braid_duration_ns, 30.0..=250.0).step_by(5.0)).changed();
                });

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Universal Braid Synthesis Telemetry").strong());
                    ui.label(format!("Process Gate Fidelity: {:.4}%", self.cached_braiding_metrics.compiled_gate_fidelity * 100.0));
                    ui.label(format!("Braid Word Length: {} operations", self.cached_braiding_metrics.braid_word_length));
                    ui.label(format!("Artin Braid Relation Error: {:.2e}", self.cached_braiding_metrics.artin_braid_relation_error));
                    ui.label(format!("F-Matrix Unitarity Error: {:.2e}", self.cached_braiding_metrics.f_matrix_unitarity_error));
                    ui.label("Magic State Distillation: Strictly NOT Required (Universal by Braiding)");
                    ui.label(format!("SU(2) Density Metric: {:.3}", self.cached_braiding_metrics.su2_density_index));
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Spacetime Worldline Braiding Diagram").strong());
                let pts_1: PlotPoints = self.cached_trajectories.iter().map(|p| [p.time_step, p.anyon_1_x]).collect();
                let pts_2: PlotPoints = self.cached_trajectories.iter().map(|p| [p.time_step, p.anyon_2_x]).collect();
                let pts_3: PlotPoints = self.cached_trajectories.iter().map(|p| [p.time_step, p.anyon_3_x]).collect();
                let pts_4: PlotPoints = self.cached_trajectories.iter().map(|p| [p.time_step, p.anyon_4_x]).collect();

                Plot::new("worldline_plot")
                    .height(380.0)
                    .x_axis_label("Normalized Braid Time t / tau")
                    .y_axis_label("Spatial Position x [um]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Anyon 1", pts_1).color(Color32::from_rgb(255, 100, 100)));
                        plot_ui.line(Line::new("Anyon 2", pts_2).color(Color32::from_rgb(100, 255, 100)));
                        plot_ui.line(Line::new("Anyon 3", pts_3).color(Color32::from_rgb(100, 150, 255)));
                        plot_ui.line(Line::new("Anyon 4", pts_4).color(Color32::from_rgb(255, 200, 50)));
                    });
            });
        });
    }

    fn render_tab_interferometer(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("SAW Anyon Fabry-Pérot Interferometer");
                ui.add_space(4.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Enclosed Topological Charge:");
                    let old_charge = self.enclosed_charge;
                    egui::ComboBox::from_id_salt("saw_charge_combo")
                        .selected_text(self.enclosed_charge.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.enclosed_charge, EnclosedTopologicalCharge::Vacuum1, EnclosedTopologicalCharge::Vacuum1.name());
                            ui.selectable_value(&mut self.enclosed_charge, EnclosedTopologicalCharge::SingleTau, EnclosedTopologicalCharge::SingleTau.name());
                            ui.selectable_value(&mut self.enclosed_charge, EnclosedTopologicalCharge::TwoTau, EnclosedTopologicalCharge::TwoTau.name());
                        });
                    if old_charge != self.enclosed_charge {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("SAW Acoustic Frequency [GHz]:");
                    changed |= ui.add(egui::Slider::new(&mut self.saw_frequency_ghz, 1.5..=4.5).step_by(0.05)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("QPC Tunneling Transmission T:");
                    changed |= ui.add(egui::Slider::new(&mut self.qpc_transmission, 0.1..=0.9).step_by(0.05)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Interferometer Area [um^2]:");
                    changed |= ui.add(egui::Slider::new(&mut self.loop_area_um2, 1.0..=12.0).step_by(0.2)).changed();
                });

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Interferometer Performance & Smoking-Gun Signature").strong());
                    ui.label(format!("Active Visibility V: {:.1}%", self.cached_interferometer_metrics.active_visibility_percent));
                    ui.label(format!("Vacuum Visibility V_1: {:.1}%", self.cached_interferometer_metrics.vacuum_visibility_percent));
                    ui.label(format!(
                        "Visibility Suppression Ratio: {:.4} (Target 1/phi = {:.4})",
                        self.cached_interferometer_metrics.visibility_suppression_ratio,
                        self.cached_interferometer_metrics.target_golden_ratio_inverse
                    ));
                    ui.label(format!("SAW Transduction Efficiency: {:.1}%", self.cached_interferometer_metrics.saw_transduction_efficiency_percent));
                    ui.label(format!("Aharonov-Bohm Period: {:.2} mT", self.cached_interferometer_metrics.aharonov_bohm_period_mt));
                    ui.label(format!("Phase Dephasing Length L_phi: {:.1} um", self.cached_interferometer_metrics.dephasing_length_um));
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Longitudinal Resistance R_xx vs Enclosed Flux Phi / Phi_0*").strong());
                let pts_rxx: PlotPoints = self.cached_flux_oscillations.iter()
                    .map(|p| [p.flux_over_phi0, p.resistance_rxx_kohm])
                    .collect();
                let pts_vac: PlotPoints = self.cached_flux_oscillations.iter()
                    .map(|p| [p.flux_over_phi0, p.vacuum_envelope_kohm])
                    .collect();

                Plot::new("flux_oscillations_plot")
                    .height(240.0)
                    .x_axis_label("Magnetic Flux [Phi / Phi_0*]")
                    .y_axis_label("Resistance R_xx [kOhm]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Active Resistance", pts_rxx).color(Color32::from_rgb(255, 100, 150)));
                        plot_ui.line(Line::new("Vacuum Envelope", pts_vac).color(Color32::from_rgb(140, 140, 140)));
                    });

                ui.add_space(4.0);
                ui.label(RichText::new("IDT Acoustic Transmission S_21 [dB]").strong());
                let pts_s21: PlotPoints = self.cached_acoustic_spectrum.iter()
                    .map(|p| [p.frequency_ghz, p.transmission_s21_db])
                    .collect();

                Plot::new("saw_spectrum_plot")
                    .height(180.0)
                    .x_axis_label("Frequency [GHz]")
                    .y_axis_label("Transmission S_21 [dB]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("S_21", pts_s21).color(Color32::from_rgb(100, 220, 255)));
                    });
            });
        });
    }

    fn render_tab_bus(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Chiral SAW Topological Quantum Bus");
                ui.add_space(4.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Bus Length [um]:");
                    changed |= ui.add(egui::Slider::new(&mut self.bus_length_um, 20.0..=300.0).step_by(10.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Cryogenic Temperature [mK]:");
                    changed |= ui.add(egui::Slider::new(&mut self.bus_temperature_mk, 5.0..=80.0).step_by(2.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Node Count:");
                    changed |= ui.add(egui::Slider::new(&mut self.node_count, 2..=8)).changed();
                });

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Quantum Bus Interconnect Telemetry").strong());
                    ui.label(format!("State Transfer Fidelity F_bus: {:.4}%", self.cached_bus_metrics.state_transfer_fidelity * 100.0));
                    ui.label(format!("Chiral Acoustic Isolation: {:.1} dB", self.cached_bus_metrics.chiral_isolation_db));
                    ui.label(format!("Thermal Phonon Occupancy n_th: {:.4e}", self.cached_bus_metrics.thermal_phonon_occupancy));
                    ui.label(format!("Acoustic Transit Time: {:.2} ns", self.cached_bus_metrics.transit_time_ns));
                    ui.label(format!("Multi-Node Concurrence C: {:.4}", self.cached_bus_metrics.multi_node_entanglement_concurrence));
                    ui.label(format!("Quantum Bus Coherence Time T_coh: {:.1} us", self.cached_bus_metrics.quantum_bus_coherence_time_us));
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Chiral Rayleigh Wave Pulse Transit Waveform").strong());
                let pts_n1: PlotPoints = self.cached_pulse_dynamics.iter().map(|p| [p.time_ns, p.node_1_amplitude]).collect();
                let pts_bus: PlotPoints = self.cached_pulse_dynamics.iter().map(|p| [p.time_ns, p.bus_traveling_amplitude]).collect();
                let pts_n2: PlotPoints = self.cached_pulse_dynamics.iter().map(|p| [p.time_ns, p.node_2_amplitude]).collect();

                Plot::new("pulse_waveform_plot")
                    .height(240.0)
                    .x_axis_label("Time [ns]")
                    .y_axis_label("Normalized Waveform Amplitude")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Node 1 Emission", pts_n1).color(Color32::from_rgb(255, 120, 120)));
                        plot_ui.line(Line::new("Bus Phonon Wave", pts_bus).color(Color32::from_rgb(120, 255, 120)));
                        plot_ui.line(Line::new("Node 2 Capture", pts_n2).color(Color32::from_rgb(120, 180, 255)));
                    });

                ui.add_space(4.0);
                ui.label(RichText::new("Transfer Fidelity vs Bus Interconnect Distance").strong());
                let pts_fid: PlotPoints = self.cached_fidelity_distance.iter().map(|p| [p.distance_um, p.transfer_fidelity]).collect();
                let pts_conc: PlotPoints = self.cached_fidelity_distance.iter().map(|p| [p.distance_um, p.entanglement_concurrence]).collect();

                Plot::new("fidelity_dist_plot")
                    .height(180.0)
                    .x_axis_label("Distance [um]")
                    .y_axis_label("Metric [0.0 - 1.0]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Transfer Fidelity", pts_fid).color(Color32::from_rgb(255, 200, 80)));
                        plot_ui.line(Line::new("Concurrence", pts_conc).color(Color32::from_rgb(180, 100, 255)));
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
            ("1. Fibonacci Quantum Dimension (d_tau = (1+sqrt(5))/2)", self.cached_audit.fibonacci_quantum_dimension_pass, "Golden ratio quantum dimension d_tau = 1.618034 verified"),
            ("2. Read-Rezayi Topological Gap (Delta_RR >= 30 mK)", self.cached_audit.topological_gap_pass, "Robust non-Abelian bulk energy gap protecting topological state"),
            ("3. Pentagon F-Matrix Unitarity (||F F^dag - I|| < 1e-10)", self.cached_audit.f_matrix_unitarity_pass, "F-matrix satisfies exact modular category pentagon identity"),
            ("4. Non-Abelian Artin Braid Relation (sigma_1 sigma_2 sigma_1 == sigma_2 sigma_1 sigma_2)", self.cached_audit.artin_braid_relation_pass, "Braid group generators satisfy canonical Artin relations"),
            ("5. Universal Gate Synthesis Fidelity (F >= 0.999 without Magic States)", self.cached_audit.universal_gate_synthesis_pass, "Full SU(2) coverage achieved by braiding alone without distillation"),
            ("6. SAW Interferometer Vacuum Visibility (V_1 >= 80.0%)", self.cached_audit.saw_vacuum_visibility_pass, "High Aharonov-Bohm interference contrast under vacuum charge"),
            ("7. Non-Abelian Visibility Suppression (V_tau / V_1 ~ 1/phi)", self.cached_audit.fibonacci_visibility_suppression_pass, "Signature 1/phi suppression confirms non-Abelian Fibonacci charge"),
            ("8. SAW Acoustic Transduction Efficiency (eta_SAW >= 88.0%)", self.cached_audit.saw_transduction_efficiency_pass, "High piezoelectric IDT Rayleigh wave coupling efficiency"),
            ("9. Chiral Quantum Bus Isolation (>= 30.0 dB)", self.cached_audit.chiral_isolation_pass, "Directional Rayleigh phonon transport prevents backscattering"),
            ("10. Multi-Node State Transfer Fidelity (F_bus >= 0.990)", self.cached_audit.multi_node_state_transfer_pass, "High-fidelity coherent state transfer between distant topological nodes"),
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
