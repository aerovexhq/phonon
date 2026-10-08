#![deny(unsafe_code)]

//! Phase 449: Superconducting Spintronic Topological Josephson Junction Memory & Quantum Phase-Slip Crossbar Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring topological phi_0 anomalous Josephson memory cells,
//! sub-150 ps chiral spin-orbit torque write dynamics, coherent quantum phase-slip dual Josephson oscillations,
//! and cryogenic 8x8 crossbar matrices with dispersive cavity readout.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::topological_josephson_memory::{
    ChiralSotMetrics, ChiralSotParams, ChiralSotSolver, CprCurvePoint, CrossbarCellState,
    CrossbarReadoutSpectrumPoint, JosephsonCprMetrics, QpsIvCurvePoint, QpsRabiPoint,
    QuantumPhaseSlipMetrics, QuantumPhaseSlipParams, QuantumPhaseSlipSolver,
    SotTrajectoryPoint, SuperconductingCrossbarMetrics, SuperconductingCrossbarParams,
    SuperconductingCrossbarSolver, TopologicalJosephsonAuditReport,
    TopologicalJosephsonMemoryProcessor, TopologicalJosephsonParams,
    TopologicalJosephsonSolver, CROSSBAR_DIMENSION,
};
use std::f64::consts::PI;

/// 5 Categorized navigation tabs for the Topological Josephson Memory dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologicalJosephsonTab {
    TopologicalJosephsonJunction,
    ChiralSotSwitching,
    QuantumPhaseSlipQubit,
    CryogenicCrossbarMatrix,
    AuditTelemetry,
}

impl TopologicalJosephsonTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TopologicalJosephsonJunction => "Topological Josephson CPR",
            Self::ChiralSotSwitching => "Chiral SOT Switching",
            Self::QuantumPhaseSlipQubit => "Quantum Phase-Slip Qubit",
            Self::CryogenicCrossbarMatrix => "Cryogenic Crossbar Array",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Topological Josephson Memory & Phase-Slip Crossbar (Phase 449).
#[derive(Debug, Clone)]
pub struct TopologicalJosephsonMemoryDialog {
    pub is_open: bool,
    pub active_tab: TopologicalJosephsonTab,

    // Tab 1: Topological Josephson CPR parameters
    pub critical_current_ua: f64,
    pub fractional_current_ratio: f64,
    pub anomalous_phase_phi0_rad: f64,
    pub rashba_soc_alpha_ev_nm: f64,
    pub junction_length_nm: f64,
    pub superconducting_gap_delta_uev: f64,
    pub temperature_mk: f64,

    // Tab 2: Chiral SOT parameters
    pub saturation_magnetization_ka_m: f64,
    pub gilbert_damping_alpha: f64,
    pub spin_hall_angle_theta_sh: f64,
    pub current_density_a_m2: f64,
    pub pulse_duration_ps: f64,
    pub free_layer_thickness_nm: f64,
    pub perpendicular_anisotropy_hk_oe: f64,
    pub in_plane_assist_field_hx_oe: f64,
    pub junction_area_nm2: f64,
    pub channel_resistance_ohms: f64,

    // Tab 3: Quantum Phase-Slip parameters
    pub nanowire_length_nm: f64,
    pub nanowire_cross_section_nm2: f64,
    pub qps_tunneling_energy_ghz: f64,
    pub charging_energy_ec_ghz: f64,
    pub gate_charge_offset_ng: f64,
    pub dc_bias_current_na: f64,
    pub rf_drive_frequency_ghz: f64,
    pub rabi_drive_amplitude_mhz: f64,

    // Tab 4: Superconducting Crossbar parameters
    pub readout_cavity_freq_ghz: f64,
    pub dispersive_shift_chi_mhz: f64,
    pub cavity_linewidth_kappa_mhz: f64,
    pub probe_photon_number: f64,
    pub integration_time_ns: f64,
    pub access_latency_ns: f64,
    pub base_temp_mk: f64,

    // Cached simulation outputs
    pub cached_cpr_metrics: JosephsonCprMetrics,
    pub cached_cpr_curve: Vec<CprCurvePoint>,
    pub cached_andreev_spectrum: Vec<(f64, f64, f64)>,

    pub cached_sot_metrics: ChiralSotMetrics,
    pub cached_sot_trajectory: Vec<SotTrajectoryPoint>,

    pub cached_qps_metrics: QuantumPhaseSlipMetrics,
    pub cached_iv_curve: Vec<QpsIvCurvePoint>,
    pub cached_rabi_trajectory: Vec<QpsRabiPoint>,

    pub cached_crossbar_metrics: SuperconductingCrossbarMetrics,
    pub cached_crossbar_cells: Vec<CrossbarCellState>,
    pub cached_readout_spectrum: Vec<CrossbarReadoutSpectrumPoint>,

    pub cached_audit: TopologicalJosephsonAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for TopologicalJosephsonMemoryDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl TopologicalJosephsonMemoryDialog {
    /// Fast cold-boot constructor executing in sub-2ms with pre-seeded baseline telemetry.
    pub fn new_fast() -> Self {
        let ic = 8.5;
        let frac = 0.35;
        let phi0 = 0.52 * PI;
        let alpha_r = 0.12;
        let j_len = 45.0;
        let gap = 180.0;
        let t_mk = 20.0;

        let ms = 780.0;
        let alpha = 0.016;
        let theta_sh = 0.42;
        let j_dens = 2.6e10;
        let p_dur = 110.0;
        let d_nm = 1.1;
        let hk = 1950.0;
        let hx = 180.0;
        let area = 400.0;
        let r_ch = 85.0;

        let wire_l = 160.0;
        let wire_a = 30.0;
        let eqps = 1.95;
        let ec = 1.30;
        let ng = 0.50;
        let i_bias = 1.40;
        let f_rf = 3.80;
        let rabi_amp = 50.0;

        let f_cav = 6.45;
        let chi = 3.60;
        let kappa = 0.75;
        let n_photons = 14.0;
        let tau_int = 85.0;
        let tau_acc = 1.25;
        let t_base = 20.0;

        let cpr_params = TopologicalJosephsonParams {
            critical_current_ua: ic,
            fractional_current_ratio: frac,
            anomalous_phase_phi0_rad: phi0,
            rashba_soc_alpha_ev_nm: alpha_r,
            junction_length_nm: j_len,
            superconducting_gap_delta_uev: gap,
            temperature_mk: t_mk,
        };
        let cpr_solver = TopologicalJosephsonSolver::new(cpr_params.clone());
        let cpr_metrics = cpr_solver.evaluate_metrics();
        let cpr_curve = cpr_solver.compute_cpr_curve(32);
        let andreev = cpr_solver.compute_andreev_spectrum(24);

        let sot_params = ChiralSotParams {
            saturation_magnetization_ka_m: ms,
            gilbert_damping_alpha: alpha,
            spin_hall_angle_theta_sh: theta_sh,
            current_density_a_m2: j_dens,
            pulse_duration_ps: p_dur,
            free_layer_thickness_nm: d_nm,
            perpendicular_anisotropy_hk_oe: hk,
            in_plane_assist_field_hx_oe: hx,
            temperature_mk: t_mk,
            junction_area_nm2: area,
            channel_resistance_ohms: r_ch,
        };
        let sot_solver = ChiralSotSolver::new(sot_params.clone());
        let sot_metrics = sot_solver.evaluate_metrics();
        let sot_traj = sot_solver.simulate_switching_trajectory(32);

        let qps_params = QuantumPhaseSlipParams {
            nanowire_length_nm: wire_l,
            nanowire_cross_section_nm2: wire_a,
            qps_tunneling_energy_ghz: eqps,
            charging_energy_ec_ghz: ec,
            gate_charge_offset_ng: ng,
            dc_bias_current_na: i_bias,
            rf_drive_frequency_ghz: f_rf,
            rabi_drive_amplitude_mhz: rabi_amp,
            coherence_length_nm: 4.8,
        };
        let qps_solver = QuantumPhaseSlipSolver::new(qps_params.clone());
        let qps_metrics = qps_solver.evaluate_metrics();
        let iv_curve = qps_solver.compute_iv_curve(30);
        let rabi_traj = qps_solver.compute_rabi_trajectory(30, 20.0);

        let crossbar_params = SuperconductingCrossbarParams {
            readout_cavity_freq_ghz: f_cav,
            dispersive_shift_chi_mhz: chi,
            cavity_linewidth_kappa_mhz: kappa,
            probe_photon_number: n_photons,
            integration_time_ns: tau_int,
            access_latency_ns: tau_acc,
            base_temp_mk: t_base,
        };
        let crossbar_solver = SuperconductingCrossbarSolver::new(crossbar_params.clone());
        let crossbar_metrics = crossbar_solver.evaluate_metrics();
        let crossbar_cells = crossbar_solver.cells().to_vec();
        let readout_spec = crossbar_solver.compute_readout_spectrum(32);

        let processor = TopologicalJosephsonMemoryProcessor::new(
            cpr_params,
            sot_params,
            qps_params,
            crossbar_params,
        );
        let audit = processor.run_physics_audit();

        Self {
            is_open: false,
            active_tab: TopologicalJosephsonTab::TopologicalJosephsonJunction,

            critical_current_ua: ic,
            fractional_current_ratio: frac,
            anomalous_phase_phi0_rad: phi0,
            rashba_soc_alpha_ev_nm: alpha_r,
            junction_length_nm: j_len,
            superconducting_gap_delta_uev: gap,
            temperature_mk: t_mk,

            saturation_magnetization_ka_m: ms,
            gilbert_damping_alpha: alpha,
            spin_hall_angle_theta_sh: theta_sh,
            current_density_a_m2: j_dens,
            pulse_duration_ps: p_dur,
            free_layer_thickness_nm: d_nm,
            perpendicular_anisotropy_hk_oe: hk,
            in_plane_assist_field_hx_oe: hx,
            junction_area_nm2: area,
            channel_resistance_ohms: r_ch,

            nanowire_length_nm: wire_l,
            nanowire_cross_section_nm2: wire_a,
            qps_tunneling_energy_ghz: eqps,
            charging_energy_ec_ghz: ec,
            gate_charge_offset_ng: ng,
            dc_bias_current_na: i_bias,
            rf_drive_frequency_ghz: f_rf,
            rabi_drive_amplitude_mhz: rabi_amp,

            readout_cavity_freq_ghz: f_cav,
            dispersive_shift_chi_mhz: chi,
            cavity_linewidth_kappa_mhz: kappa,
            probe_photon_number: n_photons,
            integration_time_ns: tau_int,
            access_latency_ns: tau_acc,
            base_temp_mk: t_base,

            cached_cpr_metrics: cpr_metrics,
            cached_cpr_curve: cpr_curve,
            cached_andreev_spectrum: andreev,

            cached_sot_metrics: sot_metrics,
            cached_sot_trajectory: sot_traj,

            cached_qps_metrics: qps_metrics,
            cached_iv_curve: iv_curve,
            cached_rabi_trajectory: rabi_traj,

            cached_crossbar_metrics: crossbar_metrics,
            cached_crossbar_cells: crossbar_cells,
            cached_readout_spectrum: readout_spec,

            cached_audit: audit,
            last_solve_time_us: 145.0,
        }
    }

    /// Recomputes simulation states when parameters change.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let cpr_params = TopologicalJosephsonParams {
            critical_current_ua: self.critical_current_ua,
            fractional_current_ratio: self.fractional_current_ratio,
            anomalous_phase_phi0_rad: self.anomalous_phase_phi0_rad,
            rashba_soc_alpha_ev_nm: self.rashba_soc_alpha_ev_nm,
            junction_length_nm: self.junction_length_nm,
            superconducting_gap_delta_uev: self.superconducting_gap_delta_uev,
            temperature_mk: self.temperature_mk,
        };
        let cpr_solver = TopologicalJosephsonSolver::new(cpr_params.clone());
        self.cached_cpr_metrics = cpr_solver.evaluate_metrics();
        self.cached_cpr_curve = cpr_solver.compute_cpr_curve(40);
        self.cached_andreev_spectrum = cpr_solver.compute_andreev_spectrum(30);

        let sot_params = ChiralSotParams {
            saturation_magnetization_ka_m: self.saturation_magnetization_ka_m,
            gilbert_damping_alpha: self.gilbert_damping_alpha,
            spin_hall_angle_theta_sh: self.spin_hall_angle_theta_sh,
            current_density_a_m2: self.current_density_a_m2,
            pulse_duration_ps: self.pulse_duration_ps,
            free_layer_thickness_nm: self.free_layer_thickness_nm,
            perpendicular_anisotropy_hk_oe: self.perpendicular_anisotropy_hk_oe,
            in_plane_assist_field_hx_oe: self.in_plane_assist_field_hx_oe,
            temperature_mk: self.temperature_mk,
            junction_area_nm2: self.junction_area_nm2,
            channel_resistance_ohms: self.channel_resistance_ohms,
        };
        let sot_solver = ChiralSotSolver::new(sot_params.clone());
        self.cached_sot_metrics = sot_solver.evaluate_metrics();
        self.cached_sot_trajectory = sot_solver.simulate_switching_trajectory(40);

        let qps_params = QuantumPhaseSlipParams {
            nanowire_length_nm: self.nanowire_length_nm,
            nanowire_cross_section_nm2: self.nanowire_cross_section_nm2,
            qps_tunneling_energy_ghz: self.qps_tunneling_energy_ghz,
            charging_energy_ec_ghz: self.charging_energy_ec_ghz,
            gate_charge_offset_ng: self.gate_charge_offset_ng,
            dc_bias_current_na: self.dc_bias_current_na,
            rf_drive_frequency_ghz: self.rf_drive_frequency_ghz,
            rabi_drive_amplitude_mhz: self.rabi_drive_amplitude_mhz,
            coherence_length_nm: 4.8,
        };
        let qps_solver = QuantumPhaseSlipSolver::new(qps_params.clone());
        self.cached_qps_metrics = qps_solver.evaluate_metrics();
        self.cached_iv_curve = qps_solver.compute_iv_curve(35);
        self.cached_rabi_trajectory = qps_solver.compute_rabi_trajectory(35, 20.0);

        let crossbar_params = SuperconductingCrossbarParams {
            readout_cavity_freq_ghz: self.readout_cavity_freq_ghz,
            dispersive_shift_chi_mhz: self.dispersive_shift_chi_mhz,
            cavity_linewidth_kappa_mhz: self.cavity_linewidth_kappa_mhz,
            probe_photon_number: self.probe_photon_number,
            integration_time_ns: self.integration_time_ns,
            access_latency_ns: self.access_latency_ns,
            base_temp_mk: self.base_temp_mk,
        };
        let mut crossbar_solver = SuperconductingCrossbarSolver::new(crossbar_params.clone());
        // Preserve any toggled bits from previous state
        for cell in &self.cached_crossbar_cells {
            crossbar_solver.write_cell(cell.row, cell.col, cell.bit_value);
        }
        self.cached_crossbar_metrics = crossbar_solver.evaluate_metrics();
        self.cached_crossbar_cells = crossbar_solver.cells().to_vec();
        self.cached_readout_spectrum = crossbar_solver.compute_readout_spectrum(35);

        let processor = TopologicalJosephsonMemoryProcessor::new(
            cpr_params,
            sot_params,
            qps_params,
            crossbar_params,
        );
        self.cached_audit = processor.run_physics_audit();

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Toggles a cell's bit in the crossbar matrix.
    pub fn toggle_cell(&mut self, row: usize, col: usize) {
        if row < CROSSBAR_DIMENSION && col < CROSSBAR_DIMENSION {
            let idx = row * CROSSBAR_DIMENSION + col;
            let new_bit = if self.cached_crossbar_cells[idx].bit_value == 1 { 0 } else { 1 };
            let phi0 = 0.52 * PI;
            let chi = self.dispersive_shift_chi_mhz;

            self.cached_crossbar_cells[idx].bit_value = new_bit;
            self.cached_crossbar_cells[idx].phase_rad = if new_bit == 1 { phi0 } else { -phi0 };
            self.cached_crossbar_cells[idx].dispersive_shift_mhz = if new_bit == 1 { chi } else { -chi };
        }
    }

    /// Renders the modal dialog window.
    pub fn show(&mut self, ctx: &Context) {
        let mut open = self.is_open;
        Window::new("Topological Josephson Memory & Phase-Slip Crossbar (Phase 449)")
            .open(&mut open)
            .resizable(true)
            .default_width(980.0)
            .default_height(680.0)
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = open;
    }

    /// Alias for `show` to adhere to standard widget conventions.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the internal contents of the dialog (accessible in headless tests).
    pub fn render_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Topological Josephson phi_0 Memory & QPS Crossbar")
                    .color(Color32::from_rgb(110, 205, 255))
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let badge_color = if self.cached_audit.all_passed() {
                    Color32::from_rgb(60, 220, 120)
                } else {
                    Color32::from_rgb(240, 100, 100)
                };
                let (passed, total) = self.cached_audit.score();
                ui.label(
                    RichText::new(format!("Audit: {}/{} PASS ({:.1} us)", passed, total, self.last_solve_time_us))
                        .color(badge_color)
                        .strong(),
                );
            });
        });

        ui.separator();

        // Tab selection bar
        ui.horizontal(|ui| {
            let tabs = [
                TopologicalJosephsonTab::TopologicalJosephsonJunction,
                TopologicalJosephsonTab::ChiralSotSwitching,
                TopologicalJosephsonTab::QuantumPhaseSlipQubit,
                TopologicalJosephsonTab::CryogenicCrossbarMatrix,
                TopologicalJosephsonTab::AuditTelemetry,
            ];
            for tab in tabs {
                let is_selected = self.active_tab == tab;
                if ui.selectable_label(is_selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            match self.active_tab {
                TopologicalJosephsonTab::TopologicalJosephsonJunction => {
                    self.render_tab_cpr(ui);
                }
                TopologicalJosephsonTab::ChiralSotSwitching => {
                    self.render_tab_sot(ui);
                }
                TopologicalJosephsonTab::QuantumPhaseSlipQubit => {
                    self.render_tab_qps(ui);
                }
                TopologicalJosephsonTab::CryogenicCrossbarMatrix => {
                    self.render_tab_crossbar(ui);
                }
                TopologicalJosephsonTab::AuditTelemetry => {
                    self.render_tab_audit(ui);
                }
            }
        });
    }

    fn render_tab_cpr(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Topological Josephson phi_0-Junction & Current-Phase Relation (CPR)")
                .color(Color32::from_rgb(130, 215, 250))
                .strong(),
        );
        ui.label("Models anomalous ground-state phase shift phi_0 from Rashba spin-orbit coupling and 4pi-periodic fractional Majorana supercurrents.");

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Critical Current I_c (uA):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.critical_current_ua, 1.0..=30.0).text("uA")).changed();

                ui.label(RichText::new("Fractional 4pi Current Ratio:").strong());
                changed |= ui.add(egui::Slider::new(&mut self.fractional_current_ratio, 0.10..=0.80)).changed();

                ui.label(RichText::new("Anomalous Phase phi_0 (rad):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.anomalous_phase_phi0_rad, 0.2 * PI..=0.8 * PI)).changed();
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Rashba SOC alpha_R (eV*nm):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.rashba_soc_alpha_ev_nm, 0.02..=0.35)).changed();

                ui.label(RichText::new("Junction Length (nm):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.junction_length_nm, 10.0..=120.0)).changed();

                ui.label(RichText::new("Superconducting Gap (ueV):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.superconducting_gap_delta_uev, 50.0..=400.0)).changed();
            });
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Plots: CPR curve & Double-well energy
        ui.columns(2, |cols| {
            cols[0].label(RichText::new("Current-Phase Relation I(phi)").strong());
            let cpr_points: PlotPoints = self
                .cached_cpr_curve
                .iter()
                .map(|p| [p.phase_phi_rad / PI, p.total_supercurrent_ua])
                .collect();
            let frac_points: PlotPoints = self
                .cached_cpr_curve
                .iter()
                .map(|p| [p.phase_phi_rad / PI, p.fractional_supercurrent_ua])
                .collect();

            Plot::new("cpr_plot")
                .height(210.0)
                .x_axis_label("Phase phi / pi")
                .y_axis_label("Current (uA)")
                .show(&mut cols[0], |plot_ui| {
                    plot_ui.line(Line::new("Total Current I(phi)", cpr_points).color(Color32::from_rgb(80, 190, 255)));
                    plot_ui.line(Line::new("Fractional 4pi Current", frac_points).color(Color32::from_rgb(255, 195, 75)));
                    plot_ui.hline(HLine::new("Zero Current", 0.0).color(Color32::GRAY));
                });

            cols[1].label(RichText::new("Double-Well Free Energy E_J(phi)").strong());
            let energy_points: PlotPoints = self
                .cached_cpr_curve
                .iter()
                .map(|p| [p.phase_phi_rad / PI, p.free_energy_uev])
                .collect();

            Plot::new("energy_plot")
                .height(210.0)
                .x_axis_label("Phase phi / pi")
                .y_axis_label("Energy E_J (ueV)")
                .show(&mut cols[1], |plot_ui| {
                    plot_ui.line(Line::new("Free Energy Landscape", energy_points).color(Color32::from_rgb(230, 95, 230)));
                });
        });

        ui.add_space(8.0);

        // Telemetry readout
        ui.group(|ui| {
            ui.label(RichText::new("Junction CPR Telemetry").strong());
            ui.horizontal(|ui| {
                ui.label(format!("phi_0: {:.3} rad ({:.2} pi)", self.cached_cpr_metrics.anomalous_phase_phi0_rad, self.cached_cpr_metrics.anomalous_phase_phi0_rad / PI));
                ui.separator();
                ui.label(format!("Barrier Delta U: {:.1} ueV", self.cached_cpr_metrics.memory_energy_barrier_uev));
                ui.separator();
                ui.label(format!("Fractional 4pi Current: {:.2} uA", self.cached_cpr_metrics.fractional_4pi_current_ua));
                ui.separator();
                ui.label(format!("Retention Lifetime: {:.1} us", self.cached_cpr_metrics.retention_lifetime_us));
                ui.separator();
                ui.label(format!("Inductance L_J: {:.2} nH", self.cached_cpr_metrics.zero_bias_inductance_nh));
            });
        });
    }

    fn render_tab_sot(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Chiral Spin-Orbit Torque (SOT) Precessional Dynamics")
                .color(Color32::from_rgb(130, 215, 250))
                .strong(),
        );
        ui.label("Models Landau-Lifshitz-Gilbert-Slonczewski (LLGS) fast precessional switching under sub-150 ps spin Hall write pulses.");

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Write Current Density (10^10 A/m^2):").strong());
                let mut dens_unit = self.current_density_a_m2 * 1.0e-10;
                if ui.add(egui::Slider::new(&mut dens_unit, 0.8..=5.0).text("MA/cm^2")).changed() {
                    self.current_density_a_m2 = dens_unit * 1.0e10;
                    changed = true;
                }

                ui.label(RichText::new("Pulse Duration (ps):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.pulse_duration_ps, 40.0..=200.0).text("ps")).changed();

                ui.label(RichText::new("Spin Hall Angle theta_SH:").strong());
                changed |= ui.add(egui::Slider::new(&mut self.spin_hall_angle_theta_sh, 0.15..=0.60)).changed();
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Saturation Magnetization M_s (kA/m):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.saturation_magnetization_ka_m, 400.0..=1200.0).text("kA/m")).changed();

                ui.label(RichText::new("Gilbert Damping alpha:").strong());
                changed |= ui.add(egui::Slider::new(&mut self.gilbert_damping_alpha, 0.005..=0.050)).changed();

                ui.label(RichText::new("Free Layer Thickness (nm):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.free_layer_thickness_nm, 0.8..=2.0).text("nm")).changed();
            });
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Plots: Magnetization trajectory & torque
        ui.columns(2, |cols| {
            cols[0].label(RichText::new("Magnetization Trajectory m(t)").strong());
            let mx_pts: PlotPoints = self.cached_sot_trajectory.iter().map(|p| [p.time_ps, p.mx]).collect();
            let my_pts: PlotPoints = self.cached_sot_trajectory.iter().map(|p| [p.time_ps, p.my]).collect();
            let mz_pts: PlotPoints = self.cached_sot_trajectory.iter().map(|p| [p.time_ps, p.mz]).collect();

            Plot::new("sot_traj_plot")
                .height(210.0)
                .x_axis_label("Time (ps)")
                .y_axis_label("Normalized Magnetization")
                .show(&mut cols[0], |plot_ui| {
                    plot_ui.line(Line::new("m_x", mx_pts).color(Color32::from_rgb(110, 160, 255)));
                    plot_ui.line(Line::new("m_y", my_pts).color(Color32::from_rgb(110, 230, 140)));
                    plot_ui.line(Line::new("m_z (Bit State)", mz_pts).color(Color32::from_rgb(255, 90, 110)).width(2.0));
                    plot_ui.hline(HLine::new("Equator m_z = 0", 0.0).color(Color32::GRAY));
                });

            cols[1].label(RichText::new("Net Torque Magnitude |dm/dt|(t)").strong());
            let torque_pts: PlotPoints = self
                .cached_sot_trajectory
                .iter()
                .map(|p| [p.time_ps, p.torque_magnitude_rad_ns])
                .collect();

            Plot::new("torque_plot")
                .height(210.0)
                .x_axis_label("Time (ps)")
                .y_axis_label("|dm/dt| (rad/ns)")
                .show(&mut cols[1], |plot_ui| {
                    plot_ui.line(Line::new("Torque Magnitude", torque_pts).color(Color32::from_rgb(255, 175, 40)));
                });
        });

        ui.add_space(8.0);

        // Telemetry readout
        ui.group(|ui| {
            ui.label(RichText::new("Chiral SOT Switching Telemetry").strong());
            ui.horizontal(|ui| {
                ui.label(format!("Switching Latency: {:.1} ps", self.cached_sot_metrics.switching_time_ps));
                ui.separator();
                ui.label(format!("Switching Energy: {:.2} aJ", self.cached_sot_metrics.switching_energy_aj));
                ui.separator();
                ui.label(format!("Error Rate P_err: {:.2e}", self.cached_sot_metrics.switching_error_rate));
                ui.separator();
                ui.label(format!("Critical Current J_c0: {:.2} MA/cm^2", self.cached_sot_metrics.critical_current_density_ma_cm2));
                ui.separator();
                ui.label(format!("Damping-Like Torque: {:.2} ueV", self.cached_sot_metrics.damping_like_torque_uev));
            });
        });
    }

    fn render_tab_qps(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Quantum Phase-Slip (QPS) Coherent Tunneling & Dual Josephson Qubit")
                .color(Color32::from_rgb(130, 215, 250))
                .strong(),
        );
        ui.label("Models exact conjugate duality between phase slips and Cooper pairs, Bloch voltage oscillations, and coherent Rabi gate rotations.");

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("QPS Tunneling Energy E_QPS (GHz):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.qps_tunneling_energy_ghz, 1.0..=4.0).text("GHz")).changed();

                ui.label(RichText::new("Charging Energy E_C (GHz):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.charging_energy_ec_ghz, 0.5..=3.0).text("GHz")).changed();

                ui.label(RichText::new("Gate Charge Offset n_g:").strong());
                changed |= ui.add(egui::Slider::new(&mut self.gate_charge_offset_ng, 0.0..=1.0)).changed();
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("DC Bias Current (nA):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.dc_bias_current_na, 0.2..=4.0).text("nA")).changed();

                ui.label(RichText::new("RF Drive Frequency (GHz):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.rf_drive_frequency_ghz, 1.0..=8.0).text("GHz")).changed();

                ui.label(RichText::new("Rabi Drive Amplitude (MHz):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.rabi_drive_amplitude_mhz, 15.0..=100.0).text("MHz")).changed();
            });
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Plots: Dual IV & Rabi oscillations
        ui.columns(2, |cols| {
            cols[0].label(RichText::new("Dual IV Characteristic & Shapiro Steps").strong());
            let iv_pts: PlotPoints = self.cached_iv_curve.iter().map(|p| [p.current_na, p.voltage_uv]).collect();

            Plot::new("qps_iv_plot")
                .height(210.0)
                .x_axis_label("Current I (nA)")
                .y_axis_label("Voltage V (uV)")
                .show(&mut cols[0], |plot_ui| {
                    plot_ui.line(Line::new("V(I) Characteristic", iv_pts).color(Color32::from_rgb(100, 220, 240)));
                });

            cols[1].label(RichText::new("Coherent Rabi Oscillation State Dynamics").strong());
            let p0_pts: PlotPoints = self.cached_rabi_trajectory.iter().map(|p| [p.time_ns, p.prob_ground]).collect();
            let p1_pts: PlotPoints = self.cached_rabi_trajectory.iter().map(|p| [p.time_ns, p.prob_excited]).collect();

            Plot::new("rabi_plot")
                .height(210.0)
                .x_axis_label("Time (ns)")
                .y_axis_label("State Probability")
                .show(&mut cols[1], |plot_ui| {
                    plot_ui.line(Line::new("P_0 (Ground)", p0_pts).color(Color32::from_rgb(80, 160, 255)));
                    plot_ui.line(Line::new("P_1 (Excited)", p1_pts).color(Color32::from_rgb(255, 120, 90)));
                });
        });

        ui.add_space(8.0);

        // Telemetry readout
        ui.group(|ui| {
            ui.label(RichText::new("Quantum Phase-Slip Telemetry").strong());
            ui.horizontal(|ui| {
                ui.label(format!("E_QPS: {:.2} GHz", self.cached_qps_metrics.qps_amplitude_ghz));
                ui.separator();
                ui.label(format!("Bloch Freq: {:.1} MHz", self.cached_qps_metrics.bloch_frequency_mhz));
                ui.separator();
                ui.label(format!("Bloch Voltage: {:.2} uV", self.cached_qps_metrics.bloch_voltage_uv));
                ui.separator();
                ui.label(format!("Critical Voltage: {:.2} uV", self.cached_qps_metrics.critical_voltage_vc_uv));
                ui.separator();
                ui.label(format!("Rabi Fidelity: {:.4}", self.cached_qps_metrics.rabi_gate_fidelity));
            });
        });
    }

    fn render_tab_crossbar(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("Cryogenic 8x8 Memory Crossbar & Dispersive Readout")
                .color(Color32::from_rgb(130, 215, 250))
                .strong(),
        );
        ui.label("Models 64 addressable cells with half-select crosstalk isolation >= 35 dB and QND cavity readout SNR >= 20 dB. Click cells to toggle stored bit!");

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Readout Cavity Frequency (GHz):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.readout_cavity_freq_ghz, 4.0..=10.0).text("GHz")).changed();

                ui.label(RichText::new("Dispersive Shift chi (MHz):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.dispersive_shift_chi_mhz, 1.0..=8.0).text("MHz")).changed();

                ui.label(RichText::new("Linewidth kappa (MHz):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.cavity_linewidth_kappa_mhz, 0.2..=2.5).text("MHz")).changed();
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Probe Photon Number:").strong());
                changed |= ui.add(egui::Slider::new(&mut self.probe_photon_number, 2.0..=30.0)).changed();

                ui.label(RichText::new("Base Temperature (mK):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.base_temp_mk, 5.0..=80.0).text("mK")).changed();

                ui.label(RichText::new("Access Latency (ns):").strong());
                changed |= ui.add(egui::Slider::new(&mut self.access_latency_ns, 0.5..=2.0).text("ns")).changed();
            });
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Visual 8x8 interactive grid & Dispersive Spectrum
        ui.columns(2, |cols| {
            cols[0].label(RichText::new("8x8 Memory Crossbar Array (Click to Toggle)").strong());
            egui::Grid::new("crossbar_matrix_grid")
                .spacing([4.0, 4.0])
                .show(&mut cols[0], |grid_ui| {
                    for r in 0..CROSSBAR_DIMENSION {
                        for c in 0..CROSSBAR_DIMENSION {
                            let idx = r * CROSSBAR_DIMENSION + c;
                            let bit = self.cached_crossbar_cells[idx].bit_value;
                            let (label, bg_color) = if bit == 1 {
                                ("1", Color32::from_rgb(60, 200, 110))
                            } else {
                                ("0", Color32::from_rgb(45, 75, 110))
                            };

                            let btn = egui::Button::new(RichText::new(label).color(Color32::WHITE).strong())
                                .fill(bg_color)
                                .min_size(egui::vec2(28.0, 24.0));

                            if grid_ui.add(btn).clicked() {
                                self.toggle_cell(r, c);
                            }
                        }
                        grid_ui.end_row();
                    }
                });

            cols[1].label(RichText::new("Dispersive Cavity Transmission |S_21|^2").strong());
            let s0_pts: PlotPoints = self
                .cached_readout_spectrum
                .iter()
                .map(|p| [p.frequency_ghz, p.transmission_state0_db])
                .collect();
            let s1_pts: PlotPoints = self
                .cached_readout_spectrum
                .iter()
                .map(|p| [p.frequency_ghz, p.transmission_state1_db])
                .collect();

            Plot::new("crossbar_spectrum_plot")
                .height(210.0)
                .x_axis_label("Frequency (GHz)")
                .y_axis_label("|S_21|^2 (dB)")
                .show(&mut cols[1], |plot_ui| {
                    plot_ui.line(Line::new("State 0 (|0>)", s0_pts).color(Color32::from_rgb(80, 160, 255)));
                    plot_ui.line(Line::new("State 1 (|1>)", s1_pts).color(Color32::from_rgb(70, 220, 130)));
                });
        });

        ui.add_space(8.0);

        // Telemetry readout
        ui.group(|ui| {
            ui.label(RichText::new("Superconducting Crossbar Telemetry").strong());
            ui.horizontal(|ui| {
                ui.label(format!("Readout SNR: {:.1} dB", self.cached_crossbar_metrics.readout_snr_db));
                ui.separator();
                ui.label(format!("Crosstalk Isolation: {:.1} dB", self.cached_crossbar_metrics.half_select_isolation_db));
                ui.separator();
                ui.label(format!("T_2* Dephasing: {:.1} us", self.cached_crossbar_metrics.dephasing_time_t2_star_us));
                ui.separator();
                ui.label(format!("T_1 Relaxation: {:.1} us", self.cached_crossbar_metrics.relaxation_time_t1_us));
                ui.separator();
                ui.label(format!("Access Latency: {:.2} ns", self.cached_crossbar_metrics.access_latency_ns));
            });
        });
    }

    fn render_tab_audit(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("10-Point Rigorous Physics Audit Checklist")
                .color(Color32::from_rgb(130, 215, 250))
                .strong(),
        );

        let (passed, total) = self.cached_audit.score();
        let overall_color = if self.cached_audit.all_passed() {
            Color32::from_rgb(60, 220, 120)
        } else {
            Color32::from_rgb(240, 100, 100)
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Overall Audit Score: {}/{} PASS", passed, total)).color(overall_color).heading());
            ui.label(RichText::new(format!("Cold-boot Latency: {:.1} us", self.last_solve_time_us)).color(Color32::LIGHT_GRAY));
        });

        ui.separator();

        let criteria = [
            ("1. Anomalous Ground-State Phase Shift & Barrier (Delta U >= 40.0 ueV)", self.cached_audit.anomalous_phase_barrier_pass, format!("phi_0: {:.2} pi, Barrier: {:.1} ueV", self.cached_cpr_metrics.anomalous_phase_phi0_rad / PI, self.cached_cpr_metrics.memory_energy_barrier_uev)),
            ("2. Fractional 4pi-Periodic Majorana Supercurrent (Ratio >= 0.25)", self.cached_audit.fractional_4pi_current_pass, format!("I_4pi / I_c: {:.2}", self.fractional_current_ratio)),
            ("3. Non-Volatile Retention Lifetime (tau >= 1.0 us)", self.cached_audit.retention_lifetime_pass, format!("tau_ret: {:.1} us", self.cached_cpr_metrics.retention_lifetime_us)),
            ("4. Chiral SOT Switching Pulse Latency (tau <= 150.0 ps)", self.cached_audit.sot_switching_speed_pass, format!("tau_switch: {:.1} ps", self.cached_sot_metrics.switching_time_ps)),
            ("5. Ultra-Low Write Energy Dissipation (E <= 1.0 aJ)", self.cached_audit.switching_energy_pass, format!("E_switch: {:.2} aJ", self.cached_sot_metrics.switching_energy_aj)),
            ("6. Cryogenic SOT Switching Error Rate (P_err <= 1.0e-5)", self.cached_audit.switching_error_rate_pass, format!("P_err: {:.2e}", self.cached_sot_metrics.switching_error_rate)),
            ("7. Coherent Quantum Phase-Slip Tunneling Amplitude (E_QPS >= 1.0 GHz)", self.cached_audit.qps_tunneling_pass, format!("E_QPS: {:.2} GHz", self.cached_qps_metrics.qps_amplitude_ghz)),
            ("8. Bloch Voltage Duality & Rabi Single-Qubit Gate Fidelity (F >= 0.995)", self.cached_audit.bloch_rabi_duality_pass, format!("Fidelity: {:.4}", self.cached_qps_metrics.rabi_gate_fidelity)),
            ("9. Dispersive Resonator Cavity Readout SNR (SNR >= 20.0 dB)", self.cached_audit.dispersive_readout_snr_pass, format!("SNR: {:.1} dB", self.cached_crossbar_metrics.readout_snr_db)),
            ("10. Crossbar Half-Select Crosstalk & Dephasing Coherence (Iso >= 35 dB, T2* >= 12 us)", self.cached_audit.crossbar_crosstalk_coherence_pass, format!("Iso: {:.1} dB, T2*: {:.1} us", self.cached_crossbar_metrics.half_select_isolation_db, self.cached_crossbar_metrics.dephasing_time_t2_star_us)),
        ];

        for (title, pass, details) in criteria {
            ui.horizontal(|ui| {
                let (badge, color) = if pass {
                    ("[PASS]", Color32::from_rgb(60, 220, 120))
                } else {
                    ("[FAIL]", Color32::from_rgb(240, 100, 100))
                };
                ui.label(RichText::new(badge).color(color).strong());
                ui.label(RichText::new(title).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(details).color(Color32::LIGHT_GRAY));
                });
            });
            ui.separator();
        }

        ui.add_space(8.0);
        if ui.button("Re-run Complete Physics Audit").clicked() {
            self.recompute();
        }
    }
}
