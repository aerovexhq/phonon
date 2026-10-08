#![deny(unsafe_code)]

//! Phase 444: Topological Acoustic Boundary-Mode Valley-Hall Quantum Router & Entanglement Concentrator Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring topological acoustic valley-Hall phononic lattices,
//! dynamic piezoelectric valley pseudospin routing across multiple channels,
//! non-linear boundary entanglement concentration, and cryogenic dispersive cavity readout.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::valley_quantum_router::{
    BellDensityMatrix, CavityReadoutSpectrumPoint, DistillationYieldPoint,
    EntanglementConcentratorMetrics, EntanglementConcentratorParams,
    EntanglementConcentratorSolver, RouterChannelTarget, RouterDynamicTracePoint,
    RouterSpectralPoint, TopologicalValleyQuantumProcessor, ValleyHallLatticeMetrics,
    ValleyHallLatticeParams, ValleyHallLatticeSolver, ValleyQuantumRouterMetrics,
    ValleyQuantumRouterParams, ValleyQuantumRouterSolver, ValleyRouterAuditReport,
    ValleyRouterDispersionPoint, ValleySpatialFieldPoint,
};

/// 5 Categorized navigation tabs for the valley quantum router dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValleyRouterTab {
    ValleyHallLatticeDispersion,
    ValleyPolarizationRouter,
    EntanglementConcentration,
    CryogenicDispersiveReadout,
    AuditTelemetry,
}

impl ValleyRouterTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ValleyHallLatticeDispersion => "Valley-Hall Lattice",
            Self::ValleyPolarizationRouter => "Quantum Valley Router",
            Self::EntanglementConcentration => "Entanglement Concentrator",
            Self::CryogenicDispersiveReadout => "Dispersive Cavity Readout",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Topological Acoustic Valley-Hall Quantum Router (Phase 444).
#[derive(Debug, Clone)]
pub struct ValleyQuantumRouterDialog {
    pub is_open: bool,
    pub active_tab: ValleyRouterTab,

    // Lattice parameters
    pub lattice_constant_um: f64,
    pub radius_a_um: f64,
    pub radius_b_um: f64,
    pub corner_bend_angle_deg: f64,
    pub has_boundary_defect: bool,

    // Router parameters
    pub target_channel: RouterChannelTarget,
    pub gate_voltage_v: f64,
    pub gate_rise_time_ns: f64,
    pub routing_junction_length_um: f64,

    // Concentrator parameters
    pub initial_state_alpha: f64,
    pub pump_power_mw: f64,
    pub interaction_length_um: f64,
    pub dispersive_shift_chi_mhz: f64,
    pub cavity_linewidth_kappa_mhz: f64,

    // Cached models and telemetry
    pub cached_lattice_metrics: ValleyHallLatticeMetrics,
    pub cached_dispersion: Vec<ValleyRouterDispersionPoint>,
    pub cached_spatial_profile: Vec<ValleySpatialFieldPoint>,

    pub cached_router_metrics: ValleyQuantumRouterMetrics,
    pub cached_spectral_response: Vec<RouterSpectralPoint>,
    pub cached_dynamic_trace: Vec<RouterDynamicTracePoint>,

    pub cached_concentrator_metrics: EntanglementConcentratorMetrics,
    pub cached_density_matrix: BellDensityMatrix,
    pub cached_readout_spectrum: Vec<CavityReadoutSpectrumPoint>,
    pub cached_distillation_yield: Vec<DistillationYieldPoint>,

    pub cached_audit: ValleyRouterAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ValleyQuantumRouterDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ValleyQuantumRouterDialog {
    /// Instantaneous cold-boot constructor (< 2ms) with pre-seeded baseline state.
    pub fn new_fast() -> Self {
        let lat_params = ValleyHallLatticeParams::default();
        let rtr_params = ValleyQuantumRouterParams::default();
        let conc_params = EntanglementConcentratorParams::default();

        let l_solver = ValleyHallLatticeSolver::new(lat_params.clone());
        let r_solver = ValleyQuantumRouterSolver::new(rtr_params.clone());
        let c_solver = EntanglementConcentratorSolver::new(conc_params.clone());

        let lm = l_solver.evaluate_metrics();
        let disp = l_solver.compute_dispersion(24);
        let spat = l_solver.compute_spatial_profile(24);

        let rm = r_solver.evaluate_metrics();
        let spec = r_solver.compute_spectral_response(24);
        let dyn_tr = r_solver.compute_dynamic_trace(24);

        let cm = c_solver.evaluate_metrics();
        let rho = c_solver.compute_density_matrix();
        let r_spec = c_solver.compute_readout_spectrum(24);
        let dist_y = c_solver.compute_distillation_yield(24);

        let processor = TopologicalValleyQuantumProcessor {
            lattice_solver: l_solver,
            router_solver: r_solver,
            concentrator_solver: c_solver,
        };
        let audit = processor.audit_system();

        Self {
            is_open: false,
            active_tab: ValleyRouterTab::ValleyHallLatticeDispersion,

            lattice_constant_um: lat_params.lattice_constant_um,
            radius_a_um: lat_params.radius_a_um,
            radius_b_um: lat_params.radius_b_um,
            corner_bend_angle_deg: lat_params.corner_bend_angle_deg,
            has_boundary_defect: lat_params.has_boundary_defect,

            target_channel: rtr_params.target_channel,
            gate_voltage_v: rtr_params.gate_voltage_v,
            gate_rise_time_ns: rtr_params.gate_rise_time_ns,
            routing_junction_length_um: rtr_params.routing_junction_length_um,

            initial_state_alpha: conc_params.initial_state_alpha,
            pump_power_mw: conc_params.pump_power_mw,
            interaction_length_um: conc_params.interaction_length_um,
            dispersive_shift_chi_mhz: conc_params.dispersive_shift_chi_mhz,
            cavity_linewidth_kappa_mhz: conc_params.cavity_linewidth_kappa_mhz,

            cached_lattice_metrics: lm,
            cached_dispersion: disp,
            cached_spatial_profile: spat,

            cached_router_metrics: rm,
            cached_spectral_response: spec,
            cached_dynamic_trace: dyn_tr,

            cached_concentrator_metrics: cm,
            cached_density_matrix: rho,
            cached_readout_spectrum: r_spec,
            cached_distillation_yield: dist_y,

            cached_audit: audit,
            last_solve_time_us: 380.0,
        }
    }

    /// Recomputes physics solvers upon parameter adjustment.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let lat_params = ValleyHallLatticeParams {
            lattice_constant_um: self.lattice_constant_um,
            radius_a_um: self.radius_a_um,
            radius_b_um: self.radius_b_um,
            corner_bend_angle_deg: self.corner_bend_angle_deg,
            has_boundary_defect: self.has_boundary_defect,
            ..Default::default()
        };

        let rtr_params = ValleyQuantumRouterParams {
            target_channel: self.target_channel,
            gate_voltage_v: self.gate_voltage_v,
            gate_rise_time_ns: self.gate_rise_time_ns,
            routing_junction_length_um: self.routing_junction_length_um,
            ..Default::default()
        };

        let conc_params = EntanglementConcentratorParams {
            initial_state_alpha: self.initial_state_alpha,
            pump_power_mw: self.pump_power_mw,
            interaction_length_um: self.interaction_length_um,
            dispersive_shift_chi_mhz: self.dispersive_shift_chi_mhz,
            cavity_linewidth_kappa_mhz: self.cavity_linewidth_kappa_mhz,
            ..Default::default()
        };

        let l_solver = ValleyHallLatticeSolver::new(lat_params);
        let r_solver = ValleyQuantumRouterSolver::new(rtr_params);
        let c_solver = EntanglementConcentratorSolver::new(conc_params);

        self.cached_lattice_metrics = l_solver.evaluate_metrics();
        self.cached_dispersion = l_solver.compute_dispersion(28);
        self.cached_spatial_profile = l_solver.compute_spatial_profile(28);

        self.cached_router_metrics = r_solver.evaluate_metrics();
        self.cached_spectral_response = r_solver.compute_spectral_response(28);
        self.cached_dynamic_trace = r_solver.compute_dynamic_trace(28);

        self.cached_concentrator_metrics = c_solver.evaluate_metrics();
        self.cached_density_matrix = c_solver.compute_density_matrix();
        self.cached_readout_spectrum = c_solver.compute_readout_spectrum(28);
        self.cached_distillation_yield = c_solver.compute_distillation_yield(28);

        let processor = TopologicalValleyQuantumProcessor {
            lattice_solver: l_solver,
            router_solver: r_solver,
            concentrator_solver: c_solver,
        };
        self.cached_audit = processor.audit_system();
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
        Window::new("Topological Acoustic Valley-Hall Quantum Router & Entanglement Concentrator")
            .open(&mut is_open)
            .default_size([960.0, 680.0])
            .min_width(780.0)
            .min_height(540.0)
            .show(ctx, |ui| {
                self.render_dialog_contents(ui);
            });
        self.is_open = is_open;
    }

    /// Public contents renderer for window and headless integration testing.
    pub fn render_dialog_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                ValleyRouterTab::ValleyHallLatticeDispersion,
                ValleyRouterTab::ValleyPolarizationRouter,
                ValleyRouterTab::EntanglementConcentration,
                ValleyRouterTab::CryogenicDispersiveReadout,
                ValleyRouterTab::AuditTelemetry,
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
            ValleyRouterTab::ValleyHallLatticeDispersion => self.render_lattice_tab(ui),
            ValleyRouterTab::ValleyPolarizationRouter => self.render_router_tab(ui),
            ValleyRouterTab::EntanglementConcentration => self.render_concentrator_tab(ui),
            ValleyRouterTab::CryogenicDispersiveReadout => self.render_readout_tab(ui),
            ValleyRouterTab::AuditTelemetry => self.render_audit_tab(ui),
        }
    }

    fn render_lattice_tab(&mut self, ui: &mut Ui) {
        ui.heading("Topological Valley-Hall Phononic Lattice & Boundary Dispersion");
        ui.label(
            "Broken spatial inversion symmetry (r_A != r_B) creates opposite valley Berry curvature \
             at K and K' points, producing gapless valley-locked boundary states immune to inter-valley scattering.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Sublattice Radius A (um):");
            if ui.add(egui::Slider::new(&mut self.radius_a_um, 6.0..=12.0).step_by(0.1)).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Sublattice Radius B (um):");
            if ui.add(egui::Slider::new(&mut self.radius_b_um, 3.0..=8.0).step_by(0.1)).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Corner Bend:");
            egui::ComboBox::from_id_salt("valley_bend_combo")
                .selected_text(format!("{:.0} deg", self.corner_bend_angle_deg))
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.corner_bend_angle_deg, 0.0, "Straight (0 deg)").clicked() {
                        changed = true;
                    }
                    if ui.selectable_value(&mut self.corner_bend_angle_deg, 60.0, "Z-Bend (60 deg)").clicked() {
                        changed = true;
                    }
                    if ui.selectable_value(&mut self.corner_bend_angle_deg, 120.0, "Omega-Bend (120 deg)").clicked() {
                        changed = true;
                    }
                });
            ui.separator();
            if ui.checkbox(&mut self.has_boundary_defect, "Boundary Vacancy").changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Plots: Boundary Dispersion
        let pts_upper: PlotPoints = self
            .cached_dispersion
            .iter()
            .map(|p| [p.k_parallel_normalized, p.bulk_upper_mhz])
            .collect();
        let pts_lower: PlotPoints = self
            .cached_dispersion
            .iter()
            .map(|p| [p.k_parallel_normalized, p.bulk_lower_mhz])
            .collect();
        let pts_edge: PlotPoints = self
            .cached_dispersion
            .iter()
            .map(|p| [p.k_parallel_normalized, p.valley_edge_freq_mhz])
            .collect();

        let line_upper = Line::new("Bulk Upper Band", pts_upper)
            .color(Color32::from_rgb(148, 163, 184))
            .width(1.5);
        let line_lower = Line::new("Bulk Lower Band", pts_lower)
            .color(Color32::from_rgb(148, 163, 184))
            .width(1.5);
        let line_edge = Line::new("Valley Boundary State", pts_edge)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.5);

        Plot::new("valley_dispersion_plot")
            .height(260.0)
            .x_axis_label("Normalized Wavevector k_parallel * a / 2pi")
            .y_axis_label("Frequency (MHz)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_upper);
                plot_ui.line(line_lower);
                plot_ui.line(line_edge);
            });

        ui.horizontal(|ui| {
            ui.label(format!("Valley Bulk Gap: {:.2} MHz", self.cached_lattice_metrics.valley_bulk_gap_mhz));
            ui.separator();
            ui.label(format!("|C_V| Difference: {:.1}", self.cached_lattice_metrics.valley_chern_difference));
            ui.separator();
            ui.label(format!("Group Velocity: {:.0} m/s", self.cached_lattice_metrics.edge_group_velocity_ms));
            ui.separator();
            ui.label(format!("Corner Transmission: {:.1}%", self.cached_lattice_metrics.corner_transmission_ratio * 100.0));
            ui.separator();
            ui.label(format!("Defect Immunity: {:.1}%", self.cached_lattice_metrics.defect_immunity_ratio * 100.0));
        });
    }

    fn render_router_tab(&mut self, ui: &mut Ui) {
        ui.heading("Dynamic Quantum Valley-Polarization Router");
        ui.label(
            "Routes quantum acoustic wavepackets into target waveguide channels via sub-nanosecond \
             electro-acoustic gate voltage pulses without mechanical moving parts.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Target Channel:");
            egui::ComboBox::from_id_salt("router_channel_combo")
                .selected_text(self.target_channel.label())
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.target_channel, RouterChannelTarget::Port1ForwardK, "Port 1 (Forward K)").clicked() {
                        changed = true;
                    }
                    if ui.selectable_value(&mut self.target_channel, RouterChannelTarget::Port2Deflected60Kp, "Port 2 (60-deg K')").clicked() {
                        changed = true;
                    }
                    if ui.selectable_value(&mut self.target_channel, RouterChannelTarget::Port3Deflected120Kp, "Port 3 (120-deg K')").clicked() {
                        changed = true;
                    }
                });
            ui.separator();
            ui.label("Gate Rise Time (ns):");
            if ui.add(egui::Slider::new(&mut self.gate_rise_time_ns, 0.5..=3.0).step_by(0.1)).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Junction Length (um):");
            if ui.add(egui::Slider::new(&mut self.routing_junction_length_um, 30.0..=120.0).step_by(5.0)).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Plots: S-parameter Spectral Response
        let pts_target: PlotPoints = self
            .cached_spectral_response
            .iter()
            .map(|p| [p.frequency_mhz, p.target_transmission_db])
            .collect();
        let pts_crosstalk: PlotPoints = self
            .cached_spectral_response
            .iter()
            .map(|p| [p.frequency_mhz, p.crosstalk_transmission_db])
            .collect();

        let line_target = Line::new("Target Transmission (dB)", pts_target)
            .color(Color32::from_rgb(34, 197, 94))
            .width(2.5);
        let line_crosstalk = Line::new("Crosstalk Leakage (dB)", pts_crosstalk)
            .color(Color32::from_rgb(239, 68, 68))
            .width(1.8);

        Plot::new("router_spectral_plot")
            .height(260.0)
            .x_axis_label("Acoustic Frequency (MHz)")
            .y_axis_label("S-Parameters (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_target);
                plot_ui.line(line_crosstalk);
                plot_ui.hline(HLine::new("Target Isolation (-35 dB)", -35.0).color(Color32::from_rgb(251, 146, 60)));
            });

        ui.horizontal(|ui| {
            ui.label(format!("Target Transmission: {:.1}%", self.cached_router_metrics.target_transmission_percent));
            ui.separator();
            ui.label(format!("Insertion Loss: {:.2} dB", self.cached_router_metrics.insertion_loss_db));
            ui.separator();
            ui.label(format!("Crosstalk Isolation: {:.1} dB", self.cached_router_metrics.crosstalk_isolation_db));
            ui.separator();
            ui.label(format!("Switching Latency: {:.2} ns", self.cached_router_metrics.switching_latency_ns));
            ui.separator();
            ui.label(format!("Routing Bandwidth: {:.2} MHz", self.cached_router_metrics.routing_bandwidth_mhz));
        });
    }

    fn render_concentrator_tab(&mut self, ui: &mut Ui) {
        ui.heading("Non-Linear Boundary Entanglement Concentration");
        ui.label(
            "Distills pairs of non-maximally entangled input phonon states into high-fidelity Bell states \
             (|Psi+>) via boundary four-wave mixing and parity coincidence verification.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Input Amplitude alpha:");
            if ui.add(egui::Slider::new(&mut self.initial_state_alpha, 0.50..=0.98).step_by(0.01)).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Pump Power (mW):");
            if ui.add(egui::Slider::new(&mut self.pump_power_mw, 5.0..=40.0).step_by(1.0)).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Interaction Length (um):");
            if ui.add(egui::Slider::new(&mut self.interaction_length_um, 50.0..=300.0).step_by(10.0)).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Plot: Distillation Yield vs Interaction Distance
        let pts_conc: PlotPoints = self
            .cached_distillation_yield
            .iter()
            .map(|p| [p.distance_um, p.concurrence])
            .collect();
        let pts_fid: PlotPoints = self
            .cached_distillation_yield
            .iter()
            .map(|p| [p.distance_um, p.fidelity])
            .collect();

        let line_conc = Line::new("Distilled Concurrence C(z)", pts_conc)
            .color(Color32::from_rgb(168, 85, 247))
            .width(2.5);
        let line_fid = Line::new("Bell State Fidelity F(z)", pts_fid)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.0);

        Plot::new("entanglement_yield_plot")
            .height(260.0)
            .x_axis_label("Boundary Interaction Distance z (um)")
            .y_axis_label("Quantum Concurrence / Fidelity")
            .show(ui, |plot_ui| {
                plot_ui.line(line_conc);
                plot_ui.line(line_fid);
                plot_ui.hline(HLine::new("Target Concurrence (0.96)", 0.96).color(Color32::from_rgb(34, 197, 94)));
            });

        ui.horizontal(|ui| {
            ui.label(format!("Initial Concurrence C_0: {:.3}", self.cached_concentrator_metrics.initial_concurrence));
            ui.separator();
            ui.label(format!("Distilled Concurrence C: {:.3}", self.cached_concentrator_metrics.concentrated_concurrence));
            ui.separator();
            ui.label(format!("Bell State Fidelity: {:.4}", self.cached_concentrator_metrics.bell_state_fidelity));
            ui.separator();
            ui.label(format!("Success Probability: {:.1}%", self.cached_concentrator_metrics.success_probability_percent));
        });
    }

    fn render_readout_tab(&mut self, ui: &mut Ui) {
        ui.heading("Cryogenic Dispersive Microwave Cavity Readout");
        ui.label(
            "QND parity readout of concentrated Bell states via dispersive frequency shifts in a superconducting \
             microwave coplanar cavity coupled to acoustic valley waveguides at 20 mK.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Dispersive Shift chi (MHz):");
            if ui.add(egui::Slider::new(&mut self.dispersive_shift_chi_mhz, 2.0..=6.0).step_by(0.1)).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Cavity Linewidth kappa (MHz):");
            if ui.add(egui::Slider::new(&mut self.cavity_linewidth_kappa_mhz, 0.2..=1.0).step_by(0.05)).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Plot: Cavity Transmission Spectrum
        let pts_01: PlotPoints = self
            .cached_readout_spectrum
            .iter()
            .map(|p| [p.detuning_mhz, p.transmission_state01_db])
            .collect();
        let pts_10: PlotPoints = self
            .cached_readout_spectrum
            .iter()
            .map(|p| [p.detuning_mhz, p.transmission_state10_db])
            .collect();
        let pts_bell: PlotPoints = self
            .cached_readout_spectrum
            .iter()
            .map(|p| [p.detuning_mhz, p.transmission_bell_db])
            .collect();

        let line_01 = Line::new("|01> Transmission Peak", pts_01)
            .color(Color32::from_rgb(56, 189, 248))
            .width(1.8);
        let line_10 = Line::new("|10> Transmission Peak", pts_10)
            .color(Color32::from_rgb(251, 146, 60))
            .width(1.8);
        let line_bell = Line::new("Bell State (|01> + |10>)", pts_bell)
            .color(Color32::from_rgb(34, 197, 94))
            .width(2.5);

        Plot::new("dispersive_readout_plot")
            .height(260.0)
            .x_axis_label("Cavity Detuning (MHz)")
            .y_axis_label("Transmission Power |S_21|^2 (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_01);
                plot_ui.line(line_10);
                plot_ui.line(line_bell);
            });

        ui.horizontal(|ui| {
            ui.label(format!("Readout SNR: {:.1} dB", self.cached_concentrator_metrics.dispersive_readout_snr_db));
            ui.separator();
            ui.label(format!("QND Preservation Fidelity: {:.4}", self.cached_concentrator_metrics.qnd_fidelity));
            ui.separator();
            ui.label("Operating Temperature: 20 mK");
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.heading("Physics Audit Checklist & Telemetry (10-Point Audit)");
        ui.label("Automated physics verification for Phase 444.");
        ui.add_space(8.0);

        let report = self.cached_audit.clone();

        let items = [
            ("1. Valley Bulk Gap (Delta_V >= 3.0 MHz)", report.valley_bulk_gap_pass),
            ("2. Quantized Valley Chern Index (|C_V| = 1.0)", report.valley_chern_difference_pass),
            ("3. Valley Edge Group Velocity (v_g >= 1200 m/s)", report.edge_group_velocity_pass),
            ("4. Sharp Corner Bend Transmission (T_corner >= 95.0%)", report.corner_transmission_pass),
            ("5. Multi-Channel Crosstalk Isolation (>= 35.0 dB)", report.crosstalk_isolation_pass),
            ("6. High Target Routing Transmission (>= 90.0%)", report.target_transmission_pass),
            ("7. Sub-3ns Channel Switching Latency (tau <= 3.0 ns)", report.switching_latency_pass),
            ("8. Entanglement Concentration Concurrence (C >= 0.96)", report.concentrated_concurrence_pass),
            ("9. Distilled Bell State Fidelity (F >= 0.995)", report.bell_fidelity_pass),
            ("10. Dispersive Cavity Readout SNR (SNR >= 18.0 dB)", report.dispersive_readout_snr_pass),
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
