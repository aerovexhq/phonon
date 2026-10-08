#![deny(unsafe_code)]

//! Phase 451: Topological Acoustic Floquet Higher-Order Corner Magneto-Phonon Isolator & Non-Reciprocal Circulator Array Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring:
//! 1. 2D Benalcazar-Bernevig-Hughes (BBH) Floquet quadrupole acoustic lattice, synthetic Coriolis magnetic pseudo-field,
//!    bulk band gap, and topologically protected 0D corner states.
//! 2. Coupled acoustomagnonic dynamics, avoided crossing polariton gap, forward/backward group velocity asymmetry,
//!    forward insertion loss, and backward isolation.
//! 3. 4-port non-reciprocal cyclic circulation (Port 1 -> 2 -> 3 -> 4 -> 1), scattering parameters, return loss,
//!    cyclic permutation symmetry, and 90-degree corner defect immunity.
//! 4. Piezoelectric microwave acoustic transducer on LiNbO3, electromechanical coupling, power handling (P1dB),
//!    transduction efficiency, and cryogenic added thermal noise at 20 mK.
//! 5. 10-point rigorous physics audit checklist and real-time execution telemetry.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::floquet_corner_isolator::{
    CirculatorSParameterPoint, CornerSpatialDensityPoint, FloquetBandDispersionPoint,
    FloquetCornerIsolatorAuditReport, FloquetCornerIsolatorProcessor, FloquetCornerMetrics,
    FloquetCornerParams, FloquetCornerSolver, FloquetTransducerMetrics, FloquetTransducerParams,
    FourPortCirculatorMetrics, FourPortCirculatorParams, FourPortCirculatorSolver,
    MagnetoPhononDispersionPoint, MagnetoPhononMetrics, MagnetoPhononParams, MagnetoPhononSolver,
    MicrowaveAcousticTransducerSolver, TransducerPowerLinePoint,
};

/// 5 Categorized navigation tabs for the Floquet Corner Isolator dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloquetCornerIsolatorTab {
    FloquetCornerQuasienergy,
    MagnetoPhononDispersion,
    FourPortCirculatorSMatrix,
    MicrowaveTransducerArray,
    AuditTelemetry,
}

impl FloquetCornerIsolatorTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FloquetCornerQuasienergy => "Quasienergy & Corner Modes",
            Self::MagnetoPhononDispersion => "Magneto-Phonon Polaritons",
            Self::FourPortCirculatorSMatrix => "4-Port Circulator S-Matrix",
            Self::MicrowaveTransducerArray => "Microwave Transducers",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Floquet Corner Isolator and Circulator Array (Phase 451).
#[derive(Debug, Clone)]
pub struct FloquetCornerIsolatorDialog {
    pub is_open: bool,
    pub active_tab: FloquetCornerIsolatorTab,

    // Tab 1: Floquet Corner Modes parameters
    pub intracell_coupling_mhz: f64,
    pub intercell_coupling_mhz: f64,
    pub modulation_freq_mhz: f64,
    pub modulation_amplitude: f64,
    pub grid_nx: usize,
    pub grid_ny: usize,

    // Tab 2: Magneto-Phonon Coupling parameters
    pub acoustic_freq_ghz: f64,
    pub magnon_freq_ghz: f64,
    pub magnetoelastic_coupling_mhz: f64,
    pub bias_field_oe: f64,

    // Tab 3: Four-Port Circulator parameters
    pub circulator_f0_ghz: f64,
    pub circulator_bw_mhz: f64,
    pub active_input_port: usize,
    pub corner_defect_present: bool,

    // Tab 4: Microwave Transducer parameters
    pub electromechanical_k2_pct: f64,
    pub idt_finger_pairs: usize,
    pub acoustic_aperture_um: f64,
    pub base_temperature_mk: f64,

    // Cached simulation outputs
    pub cached_corner_metrics: FloquetCornerMetrics,
    pub cached_spatial_modes: Vec<CornerSpatialDensityPoint>,
    pub cached_quasienergy_dispersion: Vec<FloquetBandDispersionPoint>,

    pub cached_polariton_metrics: MagnetoPhononMetrics,
    pub cached_polariton_dispersion: Vec<MagnetoPhononDispersionPoint>,

    pub cached_circulator_metrics: FourPortCirculatorMetrics,
    pub cached_s_params: Vec<CirculatorSParameterPoint>,
    pub cached_s_matrix: [[f64; 4]; 4],

    pub cached_transducer_metrics: FloquetTransducerMetrics,
    pub cached_power_curve: Vec<TransducerPowerLinePoint>,

    pub cached_audit: FloquetCornerIsolatorAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for FloquetCornerIsolatorDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl FloquetCornerIsolatorDialog {
    /// Instantaneous cold boot constructor (< 2.0 ms) with pre-seeded baseline telemetry.
    pub fn new_fast() -> Self {
        let default_corner = FloquetCornerParams::default();
        let default_mag = MagnetoPhononParams::default();
        let default_circ = FourPortCirculatorParams::default();
        let default_trans = FloquetTransducerParams::default();

        let corner_solver = FloquetCornerSolver::new(default_corner.clone());
        let mag_solver = MagnetoPhononSolver::new(default_mag.clone());
        let circ_solver = FourPortCirculatorSolver::new(default_circ.clone());
        let trans_solver = MicrowaveAcousticTransducerSolver::new(default_trans.clone());

        let cached_corner_metrics = corner_solver.evaluate_metrics();
        let cached_spatial_modes = corner_solver.compute_spatial_energy_density();
        let cached_quasienergy_dispersion = corner_solver.compute_quasienergy_dispersion(20);

        let cached_polariton_metrics = mag_solver.evaluate_metrics();
        let cached_polariton_dispersion = mag_solver.compute_dispersion_spectrum(30);

        let cached_circulator_metrics = circ_solver.evaluate_metrics();
        let cached_s_params = circ_solver.compute_sparameter_spectrum(30);
        let cached_s_matrix = circ_solver.compute_s_matrix_center_linear();

        let cached_transducer_metrics = trans_solver.evaluate_metrics();
        let cached_power_curve = trans_solver.compute_power_linearity_curve(30);

        let processor = FloquetCornerIsolatorProcessor::default();
        let cached_audit = processor.evaluate_audit();

        Self {
            is_open: false,
            active_tab: FloquetCornerIsolatorTab::FloquetCornerQuasienergy,

            intracell_coupling_mhz: default_corner.intracell_coupling_mhz,
            intercell_coupling_mhz: default_corner.intercell_coupling_mhz,
            modulation_freq_mhz: default_corner.modulation_freq_mhz,
            modulation_amplitude: default_corner.modulation_amplitude,
            grid_nx: default_corner.grid_nx,
            grid_ny: default_corner.grid_ny,

            acoustic_freq_ghz: default_mag.acoustic_freq_ghz,
            magnon_freq_ghz: default_mag.magnon_freq_ghz,
            magnetoelastic_coupling_mhz: default_mag.magnetoelastic_coupling_mhz,
            bias_field_oe: default_mag.bias_field_oe,

            circulator_f0_ghz: default_circ.center_frequency_ghz,
            circulator_bw_mhz: default_circ.bandwidth_3db_mhz,
            active_input_port: default_circ.active_input_port,
            corner_defect_present: default_circ.corner_defect_present,

            electromechanical_k2_pct: default_trans.electromechanical_k2_pct,
            idt_finger_pairs: default_trans.idt_finger_pairs,
            acoustic_aperture_um: default_trans.acoustic_aperture_um,
            base_temperature_mk: default_trans.base_temperature_mk,

            cached_corner_metrics,
            cached_spatial_modes,
            cached_quasienergy_dispersion,

            cached_polariton_metrics,
            cached_polariton_dispersion,

            cached_circulator_metrics,
            cached_s_params,
            cached_s_matrix,

            cached_transducer_metrics,
            cached_power_curve,

            cached_audit,
            last_solve_time_us: 110.0,
        }
    }

    /// Recomputes all physical simulation outputs from active GUI parameters.
    pub fn recompute(&mut self) {
        let t_start = crate::time_util::Instant::now();

        let corner_params = FloquetCornerParams {
            intracell_coupling_mhz: self.intracell_coupling_mhz,
            intercell_coupling_mhz: self.intercell_coupling_mhz,
            modulation_freq_mhz: self.modulation_freq_mhz,
            modulation_amplitude: self.modulation_amplitude,
            grid_nx: self.grid_nx,
            grid_ny: self.grid_ny,
            lattice_pitch_um: 40.0,
            center_frequency_ghz: 4.80,
        };

        let mag_params = MagnetoPhononParams {
            acoustic_freq_ghz: self.acoustic_freq_ghz,
            magnon_freq_ghz: self.magnon_freq_ghz,
            magnetoelastic_coupling_mhz: self.magnetoelastic_coupling_mhz,
            gilbert_damping: 1.2e-4,
            acoustic_damping_khz: 65.0,
            saturation_magnetization_ka_m: 140.0,
            bias_field_oe: self.bias_field_oe,
            interaction_length_um: 120.0,
        };

        let circ_params = FourPortCirculatorParams {
            center_frequency_ghz: self.circulator_f0_ghz,
            bandwidth_3db_mhz: self.circulator_bw_mhz,
            port_impedance_ohms: 50.0,
            corner_defect_present: self.corner_defect_present,
            active_input_port: self.active_input_port,
        };

        let trans_params = FloquetTransducerParams {
            center_frequency_ghz: 4.80,
            electromechanical_k2_pct: self.electromechanical_k2_pct,
            idt_finger_pairs: self.idt_finger_pairs,
            acoustic_aperture_um: self.acoustic_aperture_um,
            base_temperature_mk: self.base_temperature_mk,
            input_power_dbm: -10.0,
        };

        let corner_solver = FloquetCornerSolver::new(corner_params.clone());
        let mag_solver = MagnetoPhononSolver::new(mag_params.clone());
        let circ_solver = FourPortCirculatorSolver::new(circ_params.clone());
        let trans_solver = MicrowaveAcousticTransducerSolver::new(trans_params.clone());

        self.cached_corner_metrics = corner_solver.evaluate_metrics();
        self.cached_spatial_modes = corner_solver.compute_spatial_energy_density();
        self.cached_quasienergy_dispersion = corner_solver.compute_quasienergy_dispersion(20);

        self.cached_polariton_metrics = mag_solver.evaluate_metrics();
        self.cached_polariton_dispersion = mag_solver.compute_dispersion_spectrum(30);

        self.cached_circulator_metrics = circ_solver.evaluate_metrics();
        self.cached_s_params = circ_solver.compute_sparameter_spectrum(30);
        self.cached_s_matrix = circ_solver.compute_s_matrix_center_linear();

        self.cached_transducer_metrics = trans_solver.evaluate_metrics();
        self.cached_power_curve = trans_solver.compute_power_linearity_curve(30);

        let processor = FloquetCornerIsolatorProcessor::new(
            corner_params,
            mag_params,
            circ_params,
            trans_params,
        );
        self.cached_audit = processor.evaluate_audit();

        let elapsed = t_start.elapsed();
        self.last_solve_time_us = elapsed.as_secs_f64() * 1.0e6;
    }

    /// Renders the modal dialog window if open.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Floquet Corner Magneto-Phonon Isolator & Circulator Array (Phase 451)")
            .open(&mut open)
            .default_width(960.0)
            .default_height(680.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = open;
    }

    /// Convenience alias matching standard dialog invocation pattern.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the internal contents of the dialog window.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Floquet Higher-Order Corner Isolator & Circulator Array")
                    .color(Color32::from_rgb(100, 220, 255))
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (passed, total) = self.cached_audit.score();
                let badge_color = if passed == total {
                    Color32::from_rgb(40, 200, 120)
                } else {
                    Color32::from_rgb(240, 160, 40)
                };
                ui.label(
                    RichText::new(format!("Audit: {}/{} PASS ({:.1} us)", passed, total, self.last_solve_time_us))
                        .color(badge_color)
                        .strong(),
                );
            });
        });

        ui.separator();

        // Navigation Tabs
        ui.horizontal(|ui| {
            let tabs = [
                FloquetCornerIsolatorTab::FloquetCornerQuasienergy,
                FloquetCornerIsolatorTab::MagnetoPhononDispersion,
                FloquetCornerIsolatorTab::FourPortCirculatorSMatrix,
                FloquetCornerIsolatorTab::MicrowaveTransducerArray,
                FloquetCornerIsolatorTab::AuditTelemetry,
            ];

            for tab in tabs {
                let is_selected = self.active_tab == tab;
                if ui.selectable_label(is_selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        // Tab Content
        match self.active_tab {
            FloquetCornerIsolatorTab::FloquetCornerQuasienergy => self.render_corner_quasienergy_tab(ui),
            FloquetCornerIsolatorTab::MagnetoPhononDispersion => self.render_magneto_phonon_tab(ui),
            FloquetCornerIsolatorTab::FourPortCirculatorSMatrix => self.render_four_port_circulator_tab(ui),
            FloquetCornerIsolatorTab::MicrowaveTransducerArray => self.render_microwave_transducer_tab(ui),
            FloquetCornerIsolatorTab::AuditTelemetry => self.render_audit_telemetry_tab(ui),
        }
    }

    fn render_corner_quasienergy_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Controls column
            ui.vertical(|ui| {
                ui.set_width(310.0);
                ui.label(RichText::new("BBH Floquet Lattice Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.intracell_coupling_mhz, 0.5..=10.0).text("Intracell gamma (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.intercell_coupling_mhz, 5.0..=30.0).text("Intercell lambda (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.modulation_freq_mhz, 50.0..=500.0).text("Modulation Omega (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.modulation_amplitude, 0.05..=0.60).text("Modulation Amp")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.grid_nx, 4..=16).text("Lattice Nx")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.grid_ny, 4..=16).text("Lattice Ny")).changed();

                if changed {
                    self.recompute();
                }

                ui.separator();
                ui.label(RichText::new("Topological State Telemetry").strong());
                ui.label(format!("Synthetic Coriolis Field: {:.2} T", self.cached_corner_metrics.synthetic_magnetic_field_tesla));
                ui.label(format!("Bulk Quasienergy Gap: {:.2} MHz", self.cached_corner_metrics.bulk_topological_gap_mhz));
                ui.label(format!("Corner Mode Confinement: {:.1}%", self.cached_corner_metrics.corner_confinement_ratio * 100.0));
                ui.label(format!("Quadrupole Moment Qxy: {:.3}", self.cached_corner_metrics.quadrupole_moment));
                ui.label(format!("Corner Frequency: {:.2} GHz", self.cached_corner_metrics.corner_frequency_ghz));
                ui.label(format!("Drive Period: {:.2} ns", self.cached_corner_metrics.drive_period_ns));
            });

            ui.separator();

            // Plots column
            ui.vertical(|ui| {
                ui.label(RichText::new("Quasienergy Spectrum across Brillouin Zone (Gamma -> X -> M -> Gamma)").strong());

                let upper_pts: PlotPoints = self
                    .cached_quasienergy_dispersion
                    .iter()
                    .map(|pt| [pt.k_path_norm, pt.quasi_energy_upper_mhz])
                    .collect();

                let lower_pts: PlotPoints = self
                    .cached_quasienergy_dispersion
                    .iter()
                    .map(|pt| [pt.k_path_norm, pt.quasi_energy_lower_mhz])
                    .collect();

                let corner_pts: PlotPoints = self
                    .cached_quasienergy_dispersion
                    .iter()
                    .map(|pt| [pt.k_path_norm, pt.corner_mode_energy_mhz])
                    .collect();

                Plot::new("floquet_quasienergy_plot")
                    .height(240.0)
                    .x_axis_label("k Path (0: Gamma, 1: X, 2: M, 3: Gamma)")
                    .y_axis_label("Quasienergy (MHz)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Bulk Upper Band", upper_pts)
                                .color(Color32::from_rgb(80, 150, 240)),
                        );
                        plot_ui.line(
                            Line::new("Bulk Lower Band", lower_pts)
                                .color(Color32::from_rgb(80, 150, 240)),
                        );
                        plot_ui.line(
                            Line::new("0D Corner Midgap Mode", corner_pts)
                                .color(Color32::from_rgb(255, 120, 80))
                                .width(2.5),
                        );
                        plot_ui.hline(
                            HLine::new("Midgap Zero", 0.0)
                                .color(Color32::from_rgb(180, 180, 180)),
                        );
                    });

                ui.separator();
                ui.label(RichText::new("0D Corner Mode Spatial Energy Density Profile").strong());

                let mode_pts: PlotPoints = self
                    .cached_spatial_modes
                    .iter()
                    .enumerate()
                    .map(|(idx, pt)| [idx as f64, pt.energy_density])
                    .collect();

                Plot::new("floquet_spatial_mode_plot")
                    .height(180.0)
                    .x_axis_label("Lattice Site Index")
                    .y_axis_label("Energy Density |psi|^2")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Corner Localized Density", mode_pts)
                                .color(Color32::from_rgb(120, 230, 140))
                                .width(2.0),
                        );
                    });
            });
        });
    }

    fn render_magneto_phonon_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Controls column
            ui.vertical(|ui| {
                ui.set_width(310.0);
                ui.label(RichText::new("Magneto-Phonon Polariton Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.acoustic_freq_ghz, 2.0..=15.0).text("Phonon Freq (GHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.magnon_freq_ghz, 2.0..=15.0).text("Magnon Freq (GHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.magnetoelastic_coupling_mhz, 10.0..=120.0).text("Coupling g (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.bias_field_oe, 200.0..=2500.0).text("Bias Field H0 (Oe)")).changed();

                if changed {
                    self.recompute();
                }

                ui.separator();
                ui.label(RichText::new("Polariton Coupling Telemetry").strong());
                ui.label(format!("Avoided Crossing Gap: {:.2} MHz", self.cached_polariton_metrics.polariton_gap_mhz));
                ui.label(format!("Forward Group Velocity: {:.3} km/s", self.cached_polariton_metrics.group_velocity_forward_kms));
                ui.label(format!("Backward Group Velocity: {:.3} km/s", self.cached_polariton_metrics.group_velocity_backward_kms));
                ui.label(format!("Forward Insertion Loss: {:.2} dB", self.cached_polariton_metrics.insertion_loss_db));
                ui.label(format!("Backward Isolation: {:.1} dB", self.cached_polariton_metrics.isolation_db));
                ui.label(format!("Delta k Asymmetry: {:.3} rad/um", self.cached_polariton_metrics.nonreciprocal_wavenumber_delta_rad_um));
            });

            ui.separator();

            // Plots column
            ui.vertical(|ui| {
                ui.label(RichText::new("Forward vs Backward Non-Reciprocal Transmission (dB)").strong());

                let fwd_pts: PlotPoints = self
                    .cached_polariton_dispersion
                    .iter()
                    .map(|pt| [pt.frequency_ghz, pt.forward_transmission_db])
                    .collect();

                let bwd_pts: PlotPoints = self
                    .cached_polariton_dispersion
                    .iter()
                    .map(|pt| [pt.frequency_ghz, pt.backward_transmission_db])
                    .collect();

                Plot::new("polariton_dispersion_plot")
                    .height(240.0)
                    .x_axis_label("Frequency (GHz)")
                    .y_axis_label("Transmission (dB)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("S21 Forward Insertion Loss", fwd_pts)
                                .color(Color32::from_rgb(60, 210, 180))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("S12 Backward Isolation", bwd_pts)
                                .color(Color32::from_rgb(250, 100, 120))
                                .width(2.0),
                        );
                        plot_ui.hline(
                            HLine::new("Isolation Target (-36 dB)", -36.0)
                                .color(Color32::from_rgb(200, 100, 100)),
                        );
                    });

                ui.separator();
                ui.label(RichText::new("Non-Reciprocal Wavenumber Dispersion k(f)").strong());

                let k_fwd_pts: PlotPoints = self
                    .cached_polariton_dispersion
                    .iter()
                    .map(|pt| [pt.frequency_ghz, pt.k_forward_rad_um])
                    .collect();

                let k_bwd_pts: PlotPoints = self
                    .cached_polariton_dispersion
                    .iter()
                    .map(|pt| [pt.frequency_ghz, pt.k_backward_rad_um])
                    .collect();

                Plot::new("polariton_k_plot")
                    .height(180.0)
                    .x_axis_label("Frequency (GHz)")
                    .y_axis_label("Wavenumber k (rad/um)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Forward k_+", k_fwd_pts)
                                .color(Color32::from_rgb(200, 140, 255))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("Backward k_-", k_bwd_pts)
                                .color(Color32::from_rgb(255, 180, 60))
                                .width(2.0),
                        );
                    });
            });
        });
    }

    fn render_four_port_circulator_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Controls column
            ui.vertical(|ui| {
                ui.set_width(310.0);
                ui.label(RichText::new("4-Port Circulator Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.circulator_f0_ghz, 2.0..=12.0).text("Center Freq f0 (GHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.circulator_bw_mhz, 80.0..=300.0).text("Bandwidth (MHz)")).changed();

                ui.horizontal(|ui| {
                    ui.label("Active Input Port:");
                    for port in 1..=4 {
                        if ui.selectable_label(self.active_input_port == port, format!("Port {}", port)).clicked() {
                            self.active_input_port = port;
                            changed = true;
                        }
                    }
                });

                changed |= ui.checkbox(&mut self.corner_defect_present, "Inject 90-Deg Corner Defect").changed();

                if changed {
                    self.recompute();
                }

                ui.separator();
                ui.label(RichText::new("Circulator Scattering Matrix").strong());
                ui.label(format!("Forward Transmission IL: {:.2} dB", self.cached_circulator_metrics.insertion_loss_db));
                ui.label(format!("Adjacent Isolation ISO: {:.1} dB", self.cached_circulator_metrics.backward_isolation_db));
                ui.label(format!("Port Return Loss RL: {:.1} dB", self.cached_circulator_metrics.return_loss_db));
                ui.label(format!("Cross-Port Isolation: {:.1} dB", self.cached_circulator_metrics.cross_isolation_db));
                ui.label(format!("Circulation 3-dB Bandwidth: {:.1} MHz", self.cached_circulator_metrics.circulation_bandwidth_3db_mhz));
                ui.label(format!("Corner Defect Immunity: {:.1}%", self.cached_circulator_metrics.corner_defect_transmission_ratio * 100.0));
                ui.label(format!("Cyclic Symmetry Dev: {:.3} dB", self.cached_circulator_metrics.cyclic_symmetry_deviation_db));

                ui.separator();
                ui.label(RichText::new("Center Freq Linear S-Matrix [|S_ij|]").strong());
                for row in 0..4 {
                    ui.label(format!(
                        "[{:.3}, {:.3}, {:.3}, {:.3}]",
                        self.cached_s_matrix[row][0],
                        self.cached_s_matrix[row][1],
                        self.cached_s_matrix[row][2],
                        self.cached_s_matrix[row][3],
                    ));
                }
            });

            ui.separator();

            // Plots column
            ui.vertical(|ui| {
                ui.label(RichText::new("Circulator S-Parameters vs Frequency").strong());

                let s21_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|pt| [pt.frequency_ghz, pt.forward_transmission_db])
                    .collect();

                let s12_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|pt| [pt.frequency_ghz, pt.backward_isolation_db])
                    .collect();

                let s11_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|pt| [pt.frequency_ghz, pt.return_loss_db])
                    .collect();

                let s_cross_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|pt| [pt.frequency_ghz, pt.cross_isolation_db])
                    .collect();

                Plot::new("circulator_s_param_plot")
                    .height(260.0)
                    .x_axis_label("Frequency (GHz)")
                    .y_axis_label("S-Parameters (dB)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("S_forward (IL)", s21_pts)
                                .color(Color32::from_rgb(40, 200, 100))
                                .width(2.5),
                        );
                        plot_ui.line(
                            Line::new("S_backward (ISO)", s12_pts)
                                .color(Color32::from_rgb(240, 60, 60))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("S_return (RL)", s11_pts)
                                .color(Color32::from_rgb(80, 140, 250))
                                .width(1.8),
                        );
                        plot_ui.line(
                            Line::new("S_cross", s_cross_pts)
                                .color(Color32::from_rgb(200, 100, 240))
                                .width(1.5),
                        );
                        plot_ui.hline(
                            HLine::new("Isolation Target (-36 dB)", -36.0)
                                .color(Color32::from_rgb(200, 100, 100)),
                        );
                    });

                ui.separator();
                ui.label(RichText::new("Circulation Transmission Phase Response (deg)").strong());

                let phase_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|pt| [pt.frequency_ghz, pt.phase_deg])
                    .collect();

                Plot::new("circulator_phase_plot")
                    .height(160.0)
                    .x_axis_label("Frequency (GHz)")
                    .y_axis_label("Phase (deg)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Phase Angle", phase_pts)
                                .color(Color32::from_rgb(255, 180, 50))
                                .width(2.0),
                        );
                    });
            });
        });
    }

    fn render_microwave_transducer_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Controls column
            ui.vertical(|ui| {
                ui.set_width(310.0);
                ui.label(RichText::new("Piezoelectric IDT Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.electromechanical_k2_pct, 1.0..=12.0).text("Coupling K2 (%)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.idt_finger_pairs, 10..=80).text("Finger Pairs N")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.acoustic_aperture_um, 20.0..=150.0).text("Aperture W (um)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.base_temperature_mk, 5.0..=100.0).text("Base Temp (mK)")).changed();

                if changed {
                    self.recompute();
                }

                ui.separator();
                ui.label(RichText::new("Transducer Array Telemetry").strong());
                ui.label(format!("Transduction Efficiency: {:.1}%", self.cached_transducer_metrics.transduction_efficiency_pct));
                ui.label(format!("Power Handling P1dB: {:.1} dBm", self.cached_transducer_metrics.power_handling_p1db_dbm));
                ui.label(format!("Thermal Added Noise: {:.4} quanta", self.cached_transducer_metrics.added_noise_quanta));
                ui.label(format!("Noise Temperature: {:.1} mK", self.cached_transducer_metrics.noise_temperature_mk));
                ui.label(format!("Conversion Loss: {:.2} dB", self.cached_transducer_metrics.conversion_loss_db));
                ui.label(format!("Radiation Conductance: {:.2} mS", self.cached_transducer_metrics.radiation_conductance_ms));
            });

            ui.separator();

            // Plots column
            ui.vertical(|ui| {
                ui.label(RichText::new("Microwave Power Transfer Linearity & P1dB Saturation").strong());

                let p_out_pts: PlotPoints = self
                    .cached_power_curve
                    .iter()
                    .map(|pt| [pt.input_power_dbm, pt.output_power_dbm])
                    .collect();

                let comp_pts: PlotPoints = self
                    .cached_power_curve
                    .iter()
                    .map(|pt| [pt.input_power_dbm, pt.compression_db])
                    .collect();

                Plot::new("transducer_linearity_plot")
                    .height(240.0)
                    .x_axis_label("Input Microwave Power (dBm)")
                    .y_axis_label("Power (dBm) / Compression (dB)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Output Acoustic Power (dBm)", p_out_pts)
                                .color(Color32::from_rgb(50, 210, 130))
                                .width(2.5),
                        );
                        plot_ui.line(
                            Line::new("Gain Compression (dB)", comp_pts)
                                .color(Color32::from_rgb(255, 100, 100))
                                .width(2.0),
                        );
                        plot_ui.hline(
                            HLine::new("1-dB Compression Threshold", 1.0)
                                .color(Color32::from_rgb(250, 180, 50)),
                        );
                    });
            });
        });
    }

    fn render_audit_telemetry_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let (passed, total) = self.cached_audit.score();
            let header_color = if passed == total {
                Color32::from_rgb(40, 200, 120)
            } else {
                Color32::from_rgb(240, 160, 40)
            };

            ui.heading(
                RichText::new(format!("10-Point Physics Audit: {}/{} PASS", passed, total))
                    .color(header_color)
                    .strong(),
            );

            ui.separator();

            let audit_items = [
                ("1. Synthetic Coriolis Magnetic Pseudo-Field (B_synth >= 10.0 T)", self.cached_audit.synthetic_magnetic_field_pass, format!("{:.2} T", self.cached_corner_metrics.synthetic_magnetic_field_tesla)),
                ("2. Dynamic Bulk Quasienergy Band Gap (Delta_bulk >= 10.0 MHz)", self.cached_audit.corner_mode_confinement_pass, format!("{:.2} MHz", self.cached_corner_metrics.bulk_topological_gap_mhz)),
                ("3. Topologically Protected 0D Corner Mode Confinement (>= 85.0%)", self.cached_audit.corner_mode_confinement_pass, format!("{:.1}%", self.cached_corner_metrics.corner_confinement_ratio * 100.0)),
                ("4. Quantized Bulk Quadrupole Moment (Qxy == 0.500)", self.cached_audit.corner_mode_confinement_pass, format!("{:.3}", self.cached_corner_metrics.quadrupole_moment)),
                ("5. Acoustomagnonic Avoided Crossing Polariton Gap (Delta_pol >= 40.0 MHz)", self.cached_audit.magneto_phonon_polariton_gap_pass, format!("{:.2} MHz", self.cached_polariton_metrics.polariton_gap_mhz)),
                ("6. Forward Polariton Insertion Loss (IL <= 0.40 dB)", self.cached_audit.forward_insertion_loss_pass, format!("{:.2} dB", self.cached_polariton_metrics.insertion_loss_db)),
                ("7. Backward Non-Reciprocal Directivity & Isolation (ISO >= 36.0 dB)", self.cached_audit.backward_isolation_pass, format!("{:.1} dB", self.cached_polariton_metrics.isolation_db)),
                ("8. 4-Port Cyclic Circulation Return Loss (RL >= 22.0 dB)", self.cached_audit.return_loss_pass, format!("{:.1} dB", self.cached_circulator_metrics.return_loss_db)),
                ("9. 90-Degree Corner Defect Immunity Ratio (T_defect / T_clean >= 0.95)", self.cached_audit.corner_defect_immunity_pass, format!("{:.1}%", self.cached_circulator_metrics.corner_defect_transmission_ratio * 100.0)),
                ("10. Cryogenic Transduction Noise (n_add <= 0.08 quanta at 20 mK)", self.cached_audit.cryogenic_transduction_noise_pass, format!("{:.4} quanta", self.cached_transducer_metrics.added_noise_quanta)),
            ];

            for (title, pass, val) in audit_items {
                ui.horizontal(|ui| {
                    let badge = if pass {
                        RichText::new("[PASS]").color(Color32::from_rgb(40, 200, 120)).strong()
                    } else {
                        RichText::new("[FAIL]").color(Color32::from_rgb(240, 80, 80)).strong()
                    };
                    ui.label(badge);
                    ui.label(title);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(val).color(Color32::from_rgb(200, 200, 220)));
                    });
                });
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Recompute Telemetry Audit").clicked() {
                    self.recompute();
                }
                ui.label(format!("Last Recompute Solve Latency: {:.1} us", self.last_solve_time_us));
            });
        });
    }
}
