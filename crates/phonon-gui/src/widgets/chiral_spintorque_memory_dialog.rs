#![deny(unsafe_code)]

//! Phase 443: Chiral Phonon-Magnon Spin-Torque Acoustic Memory & Cryogenic Superconducting Spintronic Crossbar Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring chiral acoustic spin-transfer torque switching,
//! sub-45nm Floquet polariton writing heads, dense non-volatile spintronic crossbar arrays,
//! and cryogenic superconducting SQUID inductive readout.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::chiral_spintorque_memory::{
    AcousticSpinTorqueMetrics, AcousticSpinTorqueParams, AcousticSpinTorqueSolver,
    ChiralSpinTorqueAuditReport, ChiralSpinTorqueMemoryProcessor, MagnetizationTrajectoryPoint,
    MemoryCellState, PolaritonIsolationPoint, PolaritonWritingHeadMetrics,
    PolaritonWritingHeadParams, PolaritonWritingHeadSolver, SpatialStrainProfilePoint,
    SpintronicCrossbarMetrics, SpintronicCrossbarParams, SpintronicCrossbarSolver,
    SquidReadoutTracePoint,
};

/// 5 Categorized navigation tabs for the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryDialogTab {
    AcousticSpinTorqueDynamics,
    PolaritonWritingHead,
    SpintronicCrossbarGrid,
    CryogenicReadout,
    AuditTelemetry,
}

impl MemoryDialogTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AcousticSpinTorqueDynamics => "Spin-Torque Dynamics",
            Self::PolaritonWritingHead => "Polariton Writing Head",
            Self::SpintronicCrossbarGrid => "Spintronic Crossbar",
            Self::CryogenicReadout => "Cryogenic SQUID Readout",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Chiral Acoustic Spin-Torque Memory & Crossbar (Phase 443).
#[derive(Debug, Clone)]
pub struct ChiralSpinTorqueMemoryDialog {
    pub is_open: bool,
    pub active_tab: MemoryDialogTab,

    // Torque parameters
    pub acoustic_strain_amplitude: f64,
    pub phonon_chirality_sign: f64,
    pub operating_temp_k: f64,
    pub gilbert_damping_alpha: f64,

    // Writing head parameters
    pub polariton_coupling_mhz: f64,
    pub write_frequency_ghz: f64,
    pub spot_size_nm: f64,
    pub write_power_mw: f64,

    // Crossbar parameters
    pub selected_row: usize,
    pub selected_col: usize,
    pub parallel_resistance_kohm: f64,
    pub tmr_ratio_percent: f64,
    pub read_duration_ns: f64,

    // Cached models and telemetry
    pub cached_torque_metrics: AcousticSpinTorqueMetrics,
    pub cached_trajectory: Vec<MagnetizationTrajectoryPoint>,

    pub cached_head_metrics: PolaritonWritingHeadMetrics,
    pub cached_strain_profile: Vec<SpatialStrainProfilePoint>,
    pub cached_isolation_spectrum: Vec<PolaritonIsolationPoint>,

    pub cached_crossbar_metrics: SpintronicCrossbarMetrics,
    pub cached_crossbar_cells: Vec<MemoryCellState>,
    pub cached_readout_trace: Vec<SquidReadoutTracePoint>,

    pub cached_audit: ChiralSpinTorqueAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ChiralSpinTorqueMemoryDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ChiralSpinTorqueMemoryDialog {
    /// Instantaneous cold-boot constructor (< 2ms) with pre-seeded baseline state.
    pub fn new_fast() -> Self {
        let torque_params = AcousticSpinTorqueParams::default();
        let head_params = PolaritonWritingHeadParams::default();
        let crossbar_params = SpintronicCrossbarParams::default();

        let t_solver = AcousticSpinTorqueSolver::new(torque_params.clone());
        let h_solver = PolaritonWritingHeadSolver::new(head_params.clone());
        let c_solver = SpintronicCrossbarSolver::new(crossbar_params.clone());

        let tm = t_solver.evaluate_metrics();
        let traj = t_solver.compute_trajectory(24);

        let hm = h_solver.evaluate_metrics();
        let strain = h_solver.compute_spatial_strain_profile(24);
        let iso = h_solver.compute_isolation_spectrum(24);

        let cm = c_solver.evaluate_metrics();
        let cells = c_solver.cells.clone();
        let trace = c_solver.compute_readout_trace(24);

        let processor = ChiralSpinTorqueMemoryProcessor {
            torque_solver: t_solver,
            head_solver: h_solver,
            crossbar_solver: c_solver,
        };
        let audit = processor.audit_memory_system();

        Self {
            is_open: false,
            active_tab: MemoryDialogTab::AcousticSpinTorqueDynamics,

            acoustic_strain_amplitude: torque_params.acoustic_strain_amplitude,
            phonon_chirality_sign: torque_params.phonon_chirality_sign,
            operating_temp_k: torque_params.operating_temp_k,
            gilbert_damping_alpha: torque_params.gilbert_damping_alpha,

            polariton_coupling_mhz: head_params.polariton_coupling_mhz,
            write_frequency_ghz: head_params.write_frequency_ghz,
            spot_size_nm: head_params.spot_size_nm,
            write_power_mw: head_params.write_power_mw,

            selected_row: crossbar_params.selected_row,
            selected_col: crossbar_params.selected_col,
            parallel_resistance_kohm: crossbar_params.parallel_resistance_kohm,
            tmr_ratio_percent: crossbar_params.tmr_ratio_percent,
            read_duration_ns: crossbar_params.read_duration_ns,

            cached_torque_metrics: tm,
            cached_trajectory: traj,

            cached_head_metrics: hm,
            cached_strain_profile: strain,
            cached_isolation_spectrum: iso,

            cached_crossbar_metrics: cm,
            cached_crossbar_cells: cells,
            cached_readout_trace: trace,

            cached_audit: audit,
            last_solve_time_us: 420.0,
        }
    }

    /// Recomputes physics solvers upon parameter adjustment.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let torque_params = AcousticSpinTorqueParams {
            acoustic_strain_amplitude: self.acoustic_strain_amplitude,
            phonon_chirality_sign: self.phonon_chirality_sign,
            operating_temp_k: self.operating_temp_k,
            gilbert_damping_alpha: self.gilbert_damping_alpha,
            ..Default::default()
        };

        let head_params = PolaritonWritingHeadParams {
            polariton_coupling_mhz: self.polariton_coupling_mhz,
            write_frequency_ghz: self.write_frequency_ghz,
            spot_size_nm: self.spot_size_nm,
            write_power_mw: self.write_power_mw,
            ..Default::default()
        };

        let crossbar_params = SpintronicCrossbarParams {
            selected_row: self.selected_row,
            selected_col: self.selected_col,
            parallel_resistance_kohm: self.parallel_resistance_kohm,
            tmr_ratio_percent: self.tmr_ratio_percent,
            read_duration_ns: self.read_duration_ns,
            ..Default::default()
        };

        let t_solver = AcousticSpinTorqueSolver::new(torque_params.clone());
        let h_solver = PolaritonWritingHeadSolver::new(head_params.clone());
        let mut c_solver = SpintronicCrossbarSolver::new(crossbar_params.clone());
        c_solver.cells = self.cached_crossbar_cells.clone();

        self.cached_torque_metrics = t_solver.evaluate_metrics();
        self.cached_trajectory = t_solver.compute_trajectory(28);

        self.cached_head_metrics = h_solver.evaluate_metrics();
        self.cached_strain_profile = h_solver.compute_spatial_strain_profile(28);
        self.cached_isolation_spectrum = h_solver.compute_isolation_spectrum(28);

        self.cached_crossbar_metrics = c_solver.evaluate_metrics();
        self.cached_readout_trace = c_solver.compute_readout_trace(28);

        let processor = ChiralSpinTorqueMemoryProcessor {
            torque_solver: t_solver,
            head_solver: h_solver,
            crossbar_solver: c_solver,
        };
        self.cached_audit = processor.audit_memory_system();
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
        Window::new("Chiral Phonon-Magnon Spin-Torque Memory & Superconducting Crossbar (Phase 443)")
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
                MemoryDialogTab::AcousticSpinTorqueDynamics,
                MemoryDialogTab::PolaritonWritingHead,
                MemoryDialogTab::SpintronicCrossbarGrid,
                MemoryDialogTab::CryogenicReadout,
                MemoryDialogTab::AuditTelemetry,
            ] {
                let selected = self.active_tab == tab;
                if ui.selectable_label(selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();

        match self.active_tab {
            MemoryDialogTab::AcousticSpinTorqueDynamics => self.render_torque_tab(ui),
            MemoryDialogTab::PolaritonWritingHead => self.render_head_tab(ui),
            MemoryDialogTab::SpintronicCrossbarGrid => self.render_crossbar_tab(ui),
            MemoryDialogTab::CryogenicReadout => self.render_readout_tab(ui),
            MemoryDialogTab::AuditTelemetry => self.render_audit_tab(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_torque_tab(&mut self, ui: &mut Ui) {
        ui.heading("Chiral Acoustic Spin-Torque Dynamics & LLGS Magnetization Switching");
        ui.label(
            "Simulates sub-nanosecond non-volatile magnetization reversal driven by circularly polarized \
             acoustic surface phonons transferring angular momentum directly to ferromagnet cells.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.acoustic_strain_amplitude, 1.0e-4..=3.0e-4).text("Strain epsilon_ac"))
                .changed();
            ui.label("Phonon Chirality:");
            if ui.selectable_label(self.phonon_chirality_sign > 0.0, "Right (+1)").clicked() {
                self.phonon_chirality_sign = 1.0;
                changed = true;
            }
            if ui.selectable_label(self.phonon_chirality_sign < 0.0, "Left (-1)").clicked() {
                self.phonon_chirality_sign = -1.0;
                changed = true;
            }
            changed |= ui
                .add(egui::Slider::new(&mut self.operating_temp_k, 0.1..=300.0).text("Temp (K)"))
                .changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Magnetization vector trajectory plot
        let pts_mz: PlotPoints = self
            .cached_trajectory
            .iter()
            .map(|p| [p.time_ns, p.m_z])
            .collect();
        let pts_mx: PlotPoints = self
            .cached_trajectory
            .iter()
            .map(|p| [p.time_ns, p.m_x])
            .collect();
        let pts_my: PlotPoints = self
            .cached_trajectory
            .iter()
            .map(|p| [p.time_ns, p.m_y])
            .collect();

        let line_mz = Line::new("Perpendicular m_z (Bit State)", pts_mz)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.5);
        let line_mx = Line::new("In-Plane m_x", pts_mx)
            .color(Color32::from_rgb(244, 114, 182))
            .width(1.5);
        let line_my = Line::new("In-Plane m_y", pts_my)
            .color(Color32::from_rgb(52, 211, 153))
            .width(1.5);

        Plot::new("spin_torque_trajectory_plot")
            .height(260.0)
            .x_axis_label("Time t (ns)")
            .y_axis_label("Normalized Magnetization m_i")
            .show(ui, |plot_ui| {
                plot_ui.line(line_mz);
                plot_ui.line(line_mx);
                plot_ui.line(line_my);
                plot_ui.hline(HLine::new("Zero Threshold", 0.0).color(Color32::from_rgb(100, 116, 139)));
            });

        ui.horizontal(|ui| {
            ui.label(format!("Switch Latency: {:.2} ns", self.cached_torque_metrics.switching_latency_ns));
            ui.separator();
            ui.label(format!("Critical Strain: {:.2e}", self.cached_torque_metrics.critical_strain_amplitude));
            ui.separator();
            ui.label(format!("Thermal Stability Delta: {:.1}", self.cached_torque_metrics.thermal_stability_factor));
            ui.separator();
            ui.label(format!("Write Energy: {:.2} fJ", self.cached_torque_metrics.write_energy_fj));
            ui.separator();
            ui.label(format!("Final State m_z: {:+.1}", self.cached_torque_metrics.final_magnetization_mz));
        });
    }

    fn render_head_tab(&mut self, ui: &mut Ui) {
        ui.heading("Floquet Magnon-Phonon Polariton Writing Head");
        ui.label(
            "Models time-reversal broken Floquet polariton acoustic transducers delivering \
             sub-45nm spatial strain focusing and unidirectional write torque emission.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.spot_size_nm, 25.0..=50.0).text("Beam Waist w_0 (nm)"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.write_power_mw, 1.0..=10.0).text("Write Power P_in (mW)"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.polariton_coupling_mhz, 20.0..=80.0).text("Coupling g_mp (MHz)"))
                .changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Spatial strain profile plot
        let pts_strain: PlotPoints = self
            .cached_strain_profile
            .iter()
            .map(|p| [p.position_nm, p.normalized_strain])
            .collect();
        let pts_torque: PlotPoints = self
            .cached_strain_profile
            .iter()
            .map(|p| [p.position_nm, p.torque_density_arb])
            .collect();

        let line_strain = Line::new("Focused Acoustic Strain Profile", pts_strain)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.5);
        let line_torque = Line::new("Torque Density Profile", pts_torque)
            .color(Color32::from_rgb(251, 146, 60))
            .width(1.8);

        Plot::new("strain_focusing_profile_plot")
            .height(260.0)
            .x_axis_label("Spatial Position x (nm)")
            .y_axis_label("Normalized Strain / Torque")
            .show(ui, |plot_ui| {
                plot_ui.line(line_strain);
                plot_ui.line(line_torque);
                plot_ui.hline(HLine::new("FWHM Level (0.5)", 0.5).color(Color32::from_rgb(148, 163, 184)));
            });

        ui.horizontal(|ui| {
            ui.label(format!("Focal Spot FWHM: {:.1} nm", self.cached_head_metrics.focal_spot_fwhm_nm));
            ui.separator();
            ui.label(format!("Directional Isolation: {:.1} dB", self.cached_head_metrics.directional_isolation_db));
            ui.separator();
            ui.label(format!("Forward Efficiency: {:.1}%", self.cached_head_metrics.forward_efficiency_percent));
            ui.separator();
            ui.label(format!("Dissipation / Bit: {:.2} fJ", self.cached_head_metrics.dissipation_per_bit_fj));
        });
    }

    fn render_crossbar_tab(&mut self, ui: &mut Ui) {
        ui.heading("Cryogenic Spintronic Crossbar Grid (8x8 Array)");
        ui.label(
            "Matrix addressing of non-volatile magnetic memory cells. Click a cell to toggle its binary state (0 / 1).",
        );
        ui.add_space(6.0);

        let mut cell_toggled = false;
        let num_rows = 8;
        let num_cols = 8;

        ui.horizontal(|ui| {
            ui.label(format!("Selected Cell: Row {}, Col {}", self.selected_row, self.selected_col));
            if ui.button("Write Logic '0'").clicked() {
                let idx = self.selected_row * num_cols + self.selected_col;
                if idx < self.cached_crossbar_cells.len() {
                    self.cached_crossbar_cells[idx] = MemoryCellState::ParallelState0;
                    cell_toggled = true;
                }
            }
            if ui.button("Write Logic '1'").clicked() {
                let idx = self.selected_row * num_cols + self.selected_col;
                if idx < self.cached_crossbar_cells.len() {
                    self.cached_crossbar_cells[idx] = MemoryCellState::AntiParallelState1;
                    cell_toggled = true;
                }
            }
        });

        ui.add_space(8.0);

        // 8x8 Grid table
        for r in 0..num_rows {
            ui.horizontal(|ui| {
                ui.label(format!("WL {}:", r));
                for c in 0..num_cols {
                    let idx = r * num_cols + c;
                    let is_one = self.cached_crossbar_cells.get(idx).map(|s| s.is_logic_one()).unwrap_or(false);
                    let is_selected = self.selected_row == r && self.selected_col == c;

                    let (color, text) = if is_one {
                        (Color32::from_rgb(239, 68, 68), "1 (AP)")
                    } else {
                        (Color32::from_rgb(34, 197, 94), "0 (P)")
                    };

                    let btn_text = if is_selected {
                        RichText::new(format!("[{}]", text)).color(color).strong()
                    } else {
                        RichText::new(text).color(color)
                    };

                    if ui.button(btn_text).clicked() {
                        self.selected_row = r;
                        self.selected_col = c;
                        // Toggle state on click
                        if idx < self.cached_crossbar_cells.len() {
                            let new_state = if is_one {
                                MemoryCellState::ParallelState0
                            } else {
                                MemoryCellState::AntiParallelState1
                            };
                            self.cached_crossbar_cells[idx] = new_state;
                            cell_toggled = true;
                        }
                    }
                }
            });
        }

        if cell_toggled {
            self.recompute();
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(format!("Capacity: {} bits", self.cached_crossbar_metrics.total_capacity_bits));
            ui.separator();
            ui.label(format!("TMR Ratio: {:.1}%", self.cached_crossbar_metrics.measured_tmr_percent));
            ui.separator();
            ui.label(format!("R_P: {:.2} kOhm", self.parallel_resistance_kohm));
            ui.separator();
            ui.label(format!("R_AP: {:.2} kOhm", self.cached_crossbar_metrics.antiparallel_resistance_kohm));
            ui.separator();
            ui.label(format!("Crosstalk ISO: {:.1} dB", self.cached_crossbar_metrics.crosstalk_isolation_db));
        });
    }

    fn render_readout_tab(&mut self, ui: &mut Ui) {
        ui.heading("Cryogenic Superconducting SQUID Inductive Readout");
        ui.label(
            "Time-resolved voltage pulse measurement using high-sensitivity SQUID inductive sensing.",
        );
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.read_duration_ns, 0.5..=2.0).text("Read Duration tau (ns)"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut self.tmr_ratio_percent, 180.0..=300.0).text("TMR Ratio (%)"))
                .changed();
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // SQUID readout trace plot
        let pts_trace: PlotPoints = self
            .cached_readout_trace
            .iter()
            .map(|p| [p.time_ns, p.signal_voltage_uv])
            .collect();
        let pts_noise: PlotPoints = self
            .cached_readout_trace
            .iter()
            .map(|p| [p.time_ns, p.noise_floor_uv])
            .collect();

        let line_trace = Line::new("SQUID Sense Voltage (uV)", pts_trace)
            .color(Color32::from_rgb(168, 85, 247))
            .width(2.5);
        let line_noise = Line::new("Thermal Noise Floor (uV)", pts_noise)
            .color(Color32::from_rgb(148, 163, 184))
            .width(1.5);

        Plot::new("squid_readout_trace_plot")
            .height(260.0)
            .x_axis_label("Read Time t (ns)")
            .y_axis_label("Sense Voltage (uV)")
            .show(ui, |plot_ui| {
                plot_ui.line(line_trace);
                plot_ui.line(line_noise);
            });

        ui.horizontal(|ui| {
            ui.label(format!("Readout SNR: {:.1} dB", self.cached_crossbar_metrics.readout_snr_db));
            ui.separator();
            ui.label(format!("Read Latency: {:.2} ns", self.cached_crossbar_metrics.readout_latency_ns));
            ui.separator();
            ui.label(format!("TMR: {:.1}%", self.cached_crossbar_metrics.measured_tmr_percent));
            ui.separator();
            let cur_state = self
                .cached_crossbar_cells
                .get(self.selected_row * 8 + self.selected_col)
                .map(|s| s.bit_char())
                .unwrap_or('0');
            ui.label(format!("Addressed Cell State: Logic '{}'", cur_state));
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.heading("Phase 443: 10-Point Physics Audit & Telemetry Checklist");
        ui.label("Rigorous validation of spin-torque switching, focal spot, TMR ratio, and SQUID readout.");
        ui.add_space(8.0);

        let audit = &self.cached_audit;
        let items = [
            ("Sub-Nanosecond Switching Latency (tau_switch <= 1.0 ns)", audit.sub_ns_switching_latency_passed),
            ("Critical Strain Amplitude (epsilon_crit <= 2.5e-4)", audit.threshold_critical_strain_passed),
            ("Non-Volatile Thermal Stability (Delta >= 60.0)", audit.non_volatile_thermal_stability_passed),
            ("Floquet Polariton Directional Isolation (ISO >= 30.0 dB)", audit.polariton_directional_isolation_passed),
            ("Low Write Dissipation per Bit (E_bit <= 15.0 fJ)", audit.low_write_dissipation_passed),
            ("Sub-45nm Spatial Strain Focal Spot (FWHM <= 45.0 nm)", audit.sub_45nm_focal_spot_passed),
            ("High Tunneling Magnetoresistance (TMR >= 180.0%)", audit.high_tmr_ratio_passed),
            ("Crossbar Line Crosstalk Isolation (ISO >= 35.0 dB)", audit.crossbar_crosstalk_isolation_passed),
            ("Fast Cryogenic Readout Latency (tau_read <= 2.0 ns)", audit.fast_readout_latency_passed),
            ("Superconducting SQUID Readout SNR (SNR >= 22.0 dB)", audit.squid_readout_snr_passed),
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
            ui.label(format!("Switching: {:.2} ns", self.cached_torque_metrics.switching_latency_ns));
            ui.separator();
            ui.label(format!("Spot: {:.1} nm", self.cached_head_metrics.focal_spot_fwhm_nm));
            ui.separator();
            ui.label(format!("TMR: {:.1}%", self.cached_crossbar_metrics.measured_tmr_percent));
            ui.separator();
            ui.label(format!("Energy: {:.1} fJ", self.cached_head_metrics.dissipation_per_bit_fj));
            ui.separator();
            ui.label(format!("Readout SNR: {:.1} dB", self.cached_crossbar_metrics.readout_snr_db));
            ui.separator();
            ui.label(format!("Audit: {}/10", self.cached_audit.total_score));
        });
    }
}
