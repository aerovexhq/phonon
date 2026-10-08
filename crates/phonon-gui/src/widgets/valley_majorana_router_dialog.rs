#![deny(unsafe_code)]

//! Phase 453: Quantum Metamaterial Valley-Locked Majorana Zero Mode Acoustic Interconnect & Chiral Majorana Transmon Router Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring:
//! 1. 2D honeycomb acoustic metamaterial with broken spatial inversion symmetry, valley topological bandgap (Delta >= 18.0 MHz),
//!    and valley-Hall topological edge states with Delta C_V = 2 and sharp bend backscattering immunity (>= 94.0%).
//! 2. Coherent electromechanical coupling between localized Majorana zero modes and a superconducting microwave transmon
//!    qubit (g >= 25.0 MHz, C >= 150.0, chi_MZM >= 3.5 MHz), with vacuum Rabi splitting and parity-dependent dispersive readout.
//! 3. 4-port chiral acoustic routing crossbar with low insertion loss (IL <= 0.45 dB), high backward isolation (ISO >= 35.0 dB),
//!    tunable splitting ratio via acoustic phase delay, and wavepacket transfer fidelity >= 99.0%.
//! 4. Fault-tolerant cryogenic quantum interconnect bus operating at 20 mK with ultra-low thermal noise (n_th <= 0.05 quanta),
//!    long dephasing time T2* >= 45.0 us, and suppressed quasiparticle poisoning (Gamma_qp <= 25.0 Hz).
//! 5. 10-point rigorous physics audit checklist and real-time execution telemetry.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::valley_majorana_router::{
    BeamSplitterSMatrixPoint, ChiralBeamSplitterMetrics, ChiralBeamSplitterParams,
    ChiralBeamSplitterSolver, FaultTolerantInterconnectSolver, InterconnectBusMetrics,
    InterconnectBusParams, InterconnectThermalPoint, MajoranaTransmonCouplingSolver,
    MajoranaTransmonMetrics, MajoranaTransmonParams, TransmonRabiSpectrumPoint,
    ValleyMajoranaAuditReport, ValleyMajoranaDispersionPoint, ValleyMajoranaEdgeSpatialPoint,
    ValleyMajoranaLatticeMetrics, ValleyMajoranaLatticeParams, ValleyMajoranaLatticeSolver,
    ValleyMajoranaRouterProcessor,
};

/// 5 Categorized navigation tabs for the Valley-Locked Majorana Router dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValleyMajoranaRouterTab {
    ValleyLatticeEdgeModes,
    MajoranaTransmonCoupling,
    ChiralBeamSplitter,
    FaultTolerantInterconnect,
    AuditTelemetry,
}

impl ValleyMajoranaRouterTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ValleyLatticeEdgeModes => "Valley Edge Modes",
            Self::MajoranaTransmonCoupling => "Majorana-Transmon Qubit",
            Self::ChiralBeamSplitter => "Chiral Beam Splitter",
            Self::FaultTolerantInterconnect => "Cryogenic Interconnect",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Quantum Metamaterial Valley-Locked Majorana Router (Phase 453).
#[derive(Debug, Clone)]
pub struct ValleyMajoranaRouterDialog {
    pub is_open: bool,
    pub active_tab: ValleyMajoranaRouterTab,

    // Tab 1: Valley Lattice parameters
    pub lattice_constant_um: f64,
    pub acoustic_velocity_ms: f64,
    pub radius_sublattice_a_um: f64,
    pub radius_sublattice_b_um: f64,
    pub center_frequency_ghz: f64,
    pub ribbon_width_cells: usize,
    pub bend_angle_deg: f64,

    // Tab 2: Majorana-Transmon Coupling parameters
    pub josephson_energy_ej_ghz: f64,
    pub charging_energy_ec_mhz: f64,
    pub coupling_rate_g_mhz: f64,
    pub detuning_delta_mhz: f64,
    pub acoustic_linewidth_kappa_mhz: f64,
    pub transmon_linewidth_gamma_khz: f64,

    // Tab 3: Chiral Beam Splitter parameters
    pub routing_bandwidth_mhz: f64,
    pub phase_bias_rad: f64,
    pub junction_loss_db_mm: f64,
    pub junction_length_um: f64,

    // Tab 4: Fault-Tolerant Interconnect parameters
    pub bus_length_mm: f64,
    pub base_temperature_k: f64,
    pub waveguide_loss_db_mm: f64,
    pub superconducting_gap_uev: f64,

    // Cached simulation outputs
    pub cached_lattice_metrics: ValleyMajoranaLatticeMetrics,
    pub cached_dispersion: Vec<ValleyMajoranaDispersionPoint>,
    pub cached_spatial_mode: Vec<ValleyMajoranaEdgeSpatialPoint>,

    pub cached_transmon_metrics: MajoranaTransmonMetrics,
    pub cached_rabi_spectrum: Vec<TransmonRabiSpectrumPoint>,

    pub cached_splitter_metrics: ChiralBeamSplitterMetrics,
    pub cached_s_params: Vec<BeamSplitterSMatrixPoint>,

    pub cached_interconnect_metrics: InterconnectBusMetrics,
    pub cached_thermal_curve: Vec<InterconnectThermalPoint>,

    pub cached_audit: ValleyMajoranaAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ValleyMajoranaRouterDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ValleyMajoranaRouterDialog {
    /// Constructs a fast cold-boot instance under 2.0 ms by seeding pre-computed baseline state.
    pub fn new_fast() -> Self {
        let lat_params = ValleyMajoranaLatticeParams::default();
        let tr_params = MajoranaTransmonParams::default();
        let sp_params = ChiralBeamSplitterParams::default();
        let bus_params = InterconnectBusParams::default();

        let lat_solver = ValleyMajoranaLatticeSolver::new(lat_params.clone());
        let tr_solver = MajoranaTransmonCouplingSolver::new(tr_params.clone());
        let sp_solver = ChiralBeamSplitterSolver::new(sp_params.clone());
        let bus_solver = FaultTolerantInterconnectSolver::new(bus_params.clone());

        let processor = ValleyMajoranaRouterProcessor::new(
            lat_params.clone(),
            tr_params.clone(),
            sp_params.clone(),
            bus_params.clone(),
        );

        let cached_lattice_metrics = lat_solver.evaluate_metrics();
        let cached_dispersion = lat_solver.compute_dispersion(60);
        let cached_spatial_mode = lat_solver.compute_spatial_mode();

        let cached_transmon_metrics = tr_solver.evaluate_metrics();
        let cached_rabi_spectrum = tr_solver.compute_spectrum(60);

        let cached_splitter_metrics = sp_solver.evaluate_metrics();
        let cached_s_params = sp_solver.compute_s_parameters(60);

        let cached_interconnect_metrics = bus_solver.evaluate_metrics();
        let cached_thermal_curve = bus_solver.compute_thermal_curve(40);

        let cached_audit = processor.evaluate_audit();

        Self {
            is_open: false,
            active_tab: ValleyMajoranaRouterTab::ValleyLatticeEdgeModes,

            lattice_constant_um: lat_params.lattice_constant_um,
            acoustic_velocity_ms: lat_params.acoustic_velocity_ms,
            radius_sublattice_a_um: lat_params.radius_sublattice_a_um,
            radius_sublattice_b_um: lat_params.radius_sublattice_b_um,
            center_frequency_ghz: lat_params.center_frequency_ghz,
            ribbon_width_cells: lat_params.ribbon_width_cells,
            bend_angle_deg: lat_params.bend_angle_deg,

            josephson_energy_ej_ghz: tr_params.josephson_energy_ej_ghz,
            charging_energy_ec_mhz: tr_params.charging_energy_ec_mhz,
            coupling_rate_g_mhz: tr_params.coupling_rate_g_mhz,
            detuning_delta_mhz: tr_params.detuning_delta_mhz,
            acoustic_linewidth_kappa_mhz: tr_params.acoustic_linewidth_kappa_mhz,
            transmon_linewidth_gamma_khz: tr_params.transmon_linewidth_gamma_khz,

            routing_bandwidth_mhz: sp_params.routing_bandwidth_mhz,
            phase_bias_rad: sp_params.phase_bias_rad,
            junction_loss_db_mm: sp_params.junction_loss_db_mm,
            junction_length_um: sp_params.junction_length_um,

            bus_length_mm: bus_params.bus_length_mm,
            base_temperature_k: bus_params.base_temperature_k,
            waveguide_loss_db_mm: bus_params.waveguide_loss_db_mm,
            superconducting_gap_uev: bus_params.superconducting_gap_uev,

            cached_lattice_metrics,
            cached_dispersion,
            cached_spatial_mode,
            cached_transmon_metrics,
            cached_rabi_spectrum,
            cached_splitter_metrics,
            cached_s_params,
            cached_interconnect_metrics,
            cached_thermal_curve,
            cached_audit,
            last_solve_time_us: 135.0,
        }
    }

    /// Recomputes all active solvers and refreshes cached telemetry.
    pub fn recompute_all(&mut self) {
        let t_start = std::time::Instant::now();

        let lat_params = ValleyMajoranaLatticeParams {
            lattice_constant_um: self.lattice_constant_um,
            acoustic_velocity_ms: self.acoustic_velocity_ms,
            radius_sublattice_a_um: self.radius_sublattice_a_um,
            radius_sublattice_b_um: self.radius_sublattice_b_um,
            center_frequency_ghz: self.center_frequency_ghz,
            ribbon_width_cells: self.ribbon_width_cells,
            bend_angle_deg: self.bend_angle_deg,
        };

        let tr_params = MajoranaTransmonParams {
            josephson_energy_ej_ghz: self.josephson_energy_ej_ghz,
            charging_energy_ec_mhz: self.charging_energy_ec_mhz,
            acoustic_freq_ghz: self.center_frequency_ghz,
            coupling_rate_g_mhz: self.coupling_rate_g_mhz,
            acoustic_linewidth_kappa_mhz: self.acoustic_linewidth_kappa_mhz,
            transmon_linewidth_gamma_khz: self.transmon_linewidth_gamma_khz,
            detuning_delta_mhz: self.detuning_delta_mhz,
        };

        let sp_params = ChiralBeamSplitterParams {
            center_frequency_ghz: self.center_frequency_ghz,
            routing_bandwidth_mhz: self.routing_bandwidth_mhz,
            phase_bias_rad: self.phase_bias_rad,
            junction_loss_db_mm: self.junction_loss_db_mm,
            junction_length_um: self.junction_length_um,
            acoustic_impedance_ohms: 50.0,
        };

        let bus_params = InterconnectBusParams {
            bus_length_mm: self.bus_length_mm,
            center_frequency_ghz: self.center_frequency_ghz,
            base_temperature_k: self.base_temperature_k,
            waveguide_loss_db_mm: self.waveguide_loss_db_mm,
            superconducting_gap_uev: self.superconducting_gap_uev,
            electromechanical_coupling_k2: 0.055,
        };

        let lat_solver = ValleyMajoranaLatticeSolver::new(lat_params.clone());
        let tr_solver = MajoranaTransmonCouplingSolver::new(tr_params.clone());
        let sp_solver = ChiralBeamSplitterSolver::new(sp_params.clone());
        let bus_solver = FaultTolerantInterconnectSolver::new(bus_params.clone());

        let processor = ValleyMajoranaRouterProcessor::new(
            lat_params,
            tr_params,
            sp_params,
            bus_params,
        );

        self.cached_lattice_metrics = lat_solver.evaluate_metrics();
        self.cached_dispersion = lat_solver.compute_dispersion(80);
        self.cached_spatial_mode = lat_solver.compute_spatial_mode();

        self.cached_transmon_metrics = tr_solver.evaluate_metrics();
        self.cached_rabi_spectrum = tr_solver.compute_spectrum(80);

        self.cached_splitter_metrics = sp_solver.evaluate_metrics();
        self.cached_s_params = sp_solver.compute_s_parameters(80);

        self.cached_interconnect_metrics = bus_solver.evaluate_metrics();
        self.cached_thermal_curve = bus_solver.compute_thermal_curve(50);

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
        Window::new("Quantum Metamaterial Valley-Locked Majorana Router (Phase 453)")
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
                RichText::new("Valley-Locked Majorana Acoustic Interconnect & Transmon Router")
                    .color(Color32::from_rgb(80, 210, 240))
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
                    RichText::new(format!("Audit: {}/{} PASS ({:.1} us)", passed, total, self.last_solve_time_us))
                        .color(badge_color)
                        .strong(),
                );
            });
        });

        ui.add_space(4.0);

        // Tab bar
        ui.horizontal(|ui| {
            let tabs = [
                ValleyMajoranaRouterTab::ValleyLatticeEdgeModes,
                ValleyMajoranaRouterTab::MajoranaTransmonCoupling,
                ValleyMajoranaRouterTab::ChiralBeamSplitter,
                ValleyMajoranaRouterTab::FaultTolerantInterconnect,
                ValleyMajoranaRouterTab::AuditTelemetry,
            ];
            for tab in tabs {
                let is_selected = self.active_tab == tab;
                if ui.selectable_label(is_selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            ValleyMajoranaRouterTab::ValleyLatticeEdgeModes => self.render_valley_lattice_tab(ui),
            ValleyMajoranaRouterTab::MajoranaTransmonCoupling => self.render_transmon_coupling_tab(ui),
            ValleyMajoranaRouterTab::ChiralBeamSplitter => self.render_chiral_splitter_tab(ui),
            ValleyMajoranaRouterTab::FaultTolerantInterconnect => self.render_interconnect_tab(ui),
            ValleyMajoranaRouterTab::AuditTelemetry => self.render_audit_telemetry_tab(ui),
        }
    }

    fn render_valley_lattice_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                ui.label(RichText::new("Valley Lattice Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.radius_sublattice_a_um, 5.0..=15.0).text("r_A (um)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.radius_sublattice_b_um, 3.0..=12.0).text("r_B (um)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.lattice_constant_um, 20.0..=60.0).text("a (um)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.acoustic_velocity_ms, 2000.0..=6000.0).text("v_a (m/s)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.center_frequency_ghz, 1.0..=10.0).text("f0 (GHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.ribbon_width_cells, 12..=48).text("Width Cells")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.bend_angle_deg, 30.0..=150.0).text("Bend Angle (deg)")).changed();

                if changed {
                    self.recompute_all();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Topological Valley Telemetry").strong());
                let m = &self.cached_lattice_metrics;
                ui.label(format!("Valley Bandgap: {:.2} MHz (Target >= 18.0)", m.valley_bandgap_mhz));
                ui.label(format!("Valley Chern Index |Delta C_V|: {} (Target == 2)", m.delta_valley_chern_number));
                ui.label(format!("C_K = {}, C_K' = {}", m.valley_k_chern_number, m.valley_kprime_chern_number));
                ui.label(format!("Edge Confinement Depth: {:.2} cells (Target <= 2.0)", m.edge_mode_decay_depth_cells));
                ui.label(format!("Bend Transmission: {:.1}% (Target >= 94.0%)", m.bend_transmission_ratio * 100.0));
                ui.label(format!("Dirac Cone Velocity: {:.1} m/s", m.dirac_velocity_ms));
                ui.label(format!("Edge Group Velocity: {:.1} m/s", m.edge_group_velocity_ms));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("1D Projected Valley-Hall Edge State Band Structure").strong());
                let edge_pts: PlotPoints = self
                    .cached_dispersion
                    .iter()
                    .map(|p| [p.k_parallel_norm, p.edge_mode_ghz])
                    .collect();

                let lower_pts: PlotPoints = self
                    .cached_dispersion
                    .iter()
                    .map(|p| [p.k_parallel_norm, p.lower_bulk_ghz])
                    .collect();

                let upper_pts: PlotPoints = self
                    .cached_dispersion
                    .iter()
                    .map(|p| [p.k_parallel_norm, p.upper_bulk_ghz])
                    .collect();

                let edge_line = Line::new("Topological Edge Mode", edge_pts)
                    .color(Color32::from_rgb(80, 220, 140));
                let lower_line = Line::new("Lower Bulk Band", lower_pts)
                    .color(Color32::from_rgb(140, 160, 190));
                let upper_line = Line::new("Upper Bulk Band", upper_pts)
                    .color(Color32::from_rgb(140, 160, 190));

                Plot::new("valley_dispersion_plot")
                    .height(380.0)
                    .x_axis_label("Normalized Wavevector k_parallel / (pi / a)")
                    .y_axis_label("Frequency (GHz)")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(lower_line);
                        plot_ui.line(upper_line);
                        plot_ui.line(edge_line);
                    });
            });
        });
    }

    fn render_transmon_coupling_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                ui.label(RichText::new("Transmon & Coupling Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.josephson_energy_ej_ghz, 10.0..=35.0).text("E_J (GHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.charging_energy_ec_mhz, 150.0..=450.0).text("E_C (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.coupling_rate_g_mhz, 10.0..=60.0).text("g (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.detuning_delta_mhz, 50.0..=400.0).text("Delta (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.acoustic_linewidth_kappa_mhz, 0.05..=1.5).text("kappa (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.transmon_linewidth_gamma_khz, 5.0..=50.0).text("gamma (kHz)")).changed();

                if changed {
                    self.recompute_all();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Coherent Transduction Metrics").strong());
                let m = &self.cached_transmon_metrics;
                ui.label(format!("Transmon Frequency omega_q: {:.3} GHz", m.transmon_frequency_ghz));
                ui.label(format!("Anharmonicity alpha: {:.1} MHz", m.transmon_anharmonicity_mhz));
                ui.label(format!("Coupling g / (2*pi): {:.2} MHz (Target >= 25.0)", m.coupling_rate_g_mhz));
                ui.label(format!("Dispersive Parity Shift: {:.2} MHz (Target >= 3.5)", m.dispersive_shift_chi_mhz));
                ui.label(format!("Cooperativity C: {:.1} (Target >= 150.0)", m.cooperativity));
                ui.label(format!("Vacuum Rabi Period: {:.2} ns", m.vacuum_rabi_period_ns));
                ui.label(format!("Parity Readout Contrast: {:.1}%", m.parity_readout_contrast_pct));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Parity-Resolved Dispersive Cavity Transmission Spectrum S_21(delta)").strong());
                let even_pts: PlotPoints = self
                    .cached_rabi_spectrum
                    .iter()
                    .map(|p| [p.probe_detuning_mhz, p.transmission_even_parity])
                    .collect();

                let odd_pts: PlotPoints = self
                    .cached_rabi_spectrum
                    .iter()
                    .map(|p| [p.probe_detuning_mhz, p.transmission_odd_parity])
                    .collect();

                let even_line = Line::new("Even Parity (+chi)", even_pts)
                    .color(Color32::from_rgb(60, 200, 240));
                let odd_line = Line::new("Odd Parity (-chi)", odd_pts)
                    .color(Color32::from_rgb(240, 120, 50));

                Plot::new("transmon_spectrum_plot")
                    .height(380.0)
                    .x_axis_label("Probe Detuning from Cavity Resonance (MHz)")
                    .y_axis_label("Normalized Transmission S_21^2")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(even_line);
                        plot_ui.line(odd_line);
                    });
            });
        });
    }

    fn render_chiral_splitter_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                ui.label(RichText::new("Chiral Router Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.routing_bandwidth_mhz, 40.0..=250.0).text("Bandwidth (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.phase_bias_rad, 0.0..=std::f64::consts::PI).text("Phase Bias (rad)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.junction_loss_db_mm, 0.02..=0.35).text("Loss (dB/mm)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.junction_length_um, 20.0..=200.0).text("Length (um)")).changed();

                if changed {
                    self.recompute_all();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("4-Port Routing Performance").strong());
                let m = &self.cached_splitter_metrics;
                ui.label(format!("Forward Insertion Loss: {:.2} dB (Target <= 0.45)", m.insertion_loss_db));
                ui.label(format!("Backward Isolation: {:.1} dB (Target >= 35.0)", m.backward_isolation_db));
                ui.label(format!("Return Loss: {:.1} dB (Target >= 22.0)", m.return_loss_db));
                ui.label(format!("Port 2 Split Fraction: {:.1}%", m.port2_split_fraction * 100.0));
                ui.label(format!("Port 3 Split Fraction: {:.1}%", m.port3_split_fraction * 100.0));
                ui.label(format!("Flying Wavepacket Fidelity: {:.2}% (Target >= 99.0%)", m.transfer_fidelity_pct));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("4-Port Chiral Routing S-Parameters Across Operating Band").strong());
                let s21_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|p| [p.frequency_ghz, p.s21_db])
                    .collect();

                let s31_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|p| [p.frequency_ghz, p.s31_db])
                    .collect();

                let s41_pts: PlotPoints = self
                    .cached_s_params
                    .iter()
                    .map(|p| [p.frequency_ghz, p.s41_db])
                    .collect();

                let s21_line = Line::new("S21 Forward Port 2 (dB)", s21_pts)
                    .color(Color32::from_rgb(60, 220, 120));
                let s31_line = Line::new("S31 Deflected Port 3 (dB)", s31_pts)
                    .color(Color32::from_rgb(80, 160, 240));
                let s41_line = Line::new("S41 Isolated Port 4 (dB)", s41_pts)
                    .color(Color32::from_rgb(240, 80, 70));

                let thresh_iso = HLine::new("Isolation Target (-35 dB)", -35.0)
                    .color(Color32::from_rgb(220, 160, 40));

                Plot::new("chiral_s_params_plot")
                    .height(380.0)
                    .x_axis_label("Frequency (GHz)")
                    .y_axis_label("Scattering Parameter (dB)")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(s21_line);
                        plot_ui.line(s31_line);
                        plot_ui.line(s41_line);
                        plot_ui.hline(thresh_iso);
                    });
            });
        });
    }

    fn render_interconnect_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                ui.label(RichText::new("Interconnect Parameters").strong());

                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.bus_length_mm, 0.5..=10.0).text("Bus Length (mm)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.base_temperature_k, 0.005..=0.150).text("Temperature (K)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.waveguide_loss_db_mm, 0.05..=0.50).text("Loss (dB/mm)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.superconducting_gap_uev, 100.0..=500.0).text("Delta_sc (ueV)")).changed();

                if changed {
                    self.recompute_all();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Cryogenic Coherence Telemetry").strong());
                let m = &self.cached_interconnect_metrics;
                ui.label(format!("Thermal Occupancy n_th: {:.4e} quanta (Target <= 0.05)", m.thermal_noise_occupancy));
                ui.label(format!("Quasiparticle Rate: {:.1} Hz (Target <= 25.0)", m.quasiparticle_poisoning_rate_hz));
                ui.label(format!("Dephasing Time T2*: {:.1} us (Target >= 45.0)", m.dephasing_time_t2_us));
                ui.label(format!("Relaxation Time T1: {:.1} us", m.relaxation_time_t1_us));
                ui.label(format!("Total Bus Loss: {:.2} dB", m.total_bus_loss_db));
                ui.label(format!("End-to-End Link Fidelity: {:.1}%", m.end_to_end_fidelity_pct));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Dephasing Coherence Time T2* vs Cryogenic Stage Temperature").strong());
                let t2_pts: PlotPoints = self
                    .cached_thermal_curve
                    .iter()
                    .map(|p| [p.temperature_k * 1000.0, p.dephasing_time_us])
                    .collect();

                let t2_line = Line::new("T2* Coherence Time (us)", t2_pts)
                    .color(Color32::from_rgb(180, 100, 240));

                let thresh_t2 = HLine::new("45 us Target Line", 45.0)
                    .color(Color32::from_rgb(60, 220, 120));

                Plot::new("thermal_coherence_plot")
                    .height(380.0)
                    .x_axis_label("Stage Temperature (mK)")
                    .y_axis_label("Dephasing Time T2* (us)")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(t2_line);
                        plot_ui.hline(thresh_t2);
                    });
            });
        });
    }

    fn render_audit_telemetry_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("10-Point Rigorous Physics Audit Checklist").heading().strong());
            ui.add_space(4.0);

            let audit = &self.cached_audit;
            let (passed, total) = audit.score();

            ui.label(
                RichText::new(format!("Overall Score: {}/{} PASS", passed, total))
                    .color(if passed == total { Color32::from_rgb(60, 220, 120) } else { Color32::from_rgb(240, 90, 70) })
                    .heading()
                    .strong(),
            );

            ui.add_space(8.0);

            let items = [
                ("1. Valley Topological Bandgap (Delta >= 18.0 MHz)", audit.valley_bandgap_pass, "Inversion-broken honeycomb valley bandgap Delta >= 18.0 MHz"),
                ("2. Valley Chern Index Difference (|Delta C_V| == 2)", audit.valley_chern_pass, "Topological valley-Hall domain wall Chern number jump |Delta C_V| == 2"),
                ("3. Edge Modal Localization Depth (xi <= 2.0 cells)", audit.edge_decay_depth_pass, "Transverse spatial confinement decay depth xi <= 2.0 unit cells"),
                ("4. Sharp Bend Transmission (T_bend >= 94.0%)", audit.bend_transmission_pass, "Backscattering immunity around sharp 60-deg / 120-deg bends >= 94.0%"),
                ("5. Transmon-Majorana Coupling (g >= 25.0 MHz)", audit.transmon_coupling_pass, "Coherent electromechanical coupling rate g / (2*pi) >= 25.0 MHz"),
                ("6. Strong Coupling Cooperativity (C >= 150.0)", audit.cooperativity_pass, "Acoustic-transmon cooperativity C = g^2 / (kappa * gamma) >= 150.0"),
                ("7. Dispersive Parity Shift (chi_MZM >= 3.5 MHz)", audit.dispersive_shift_pass, "Parity-dependent dispersive frequency shift chi_MZM >= 3.5 MHz"),
                ("8. 4-Port Chiral Backward Isolation (ISO >= 35.0 dB)", audit.chiral_isolation_pass, "Cross-port backward routing isolation ISO >= 35.0 dB"),
                ("9. Flying Wavepacket Transfer Fidelity (F >= 99.0%)", audit.transfer_fidelity_pass, "Single-phonon Majorana flying wavepacket routing fidelity >= 99.0%"),
                ("10. Cryogenic Added Noise at 20 mK (n_th <= 0.05 quanta)", audit.thermal_noise_pass, "Bose-Einstein thermal noise occupancy at 20 mK <= 0.05 quanta"),
            ];

            egui::Grid::new("valley_majorana_audit_grid")
                .striped(true)
                .min_col_width(280.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Criterion").strong());
                    ui.label(RichText::new("Status").strong());
                    ui.label(RichText::new("Physical Specification").strong());
                    ui.end_row();

                    for (name, pass, spec) in items {
                        ui.label(name);
                        if pass {
                            ui.label(RichText::new("PASS").color(Color32::from_rgb(60, 220, 120)).strong());
                        } else {
                            ui.label(RichText::new("FAIL").color(Color32::from_rgb(240, 80, 70)).strong());
                        }
                        ui.label(spec);
                        ui.end_row();
                    }
                });

            ui.add_space(12.0);
            if ui.button("Re-evaluate Physics Audit").clicked() {
                self.recompute_all();
            }
        });
    }
}
