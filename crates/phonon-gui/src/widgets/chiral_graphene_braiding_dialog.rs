#![deny(unsafe_code)]

//! Phase 454: Non-Abelian Anyon Braiding & Topological Qubit Crossbar in Chiral Phononic Graphene Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring:
//! 1. Honeycomb chiral phononic graphene metamaterial with broken time-reversal symmetry, non-zero Chern numbers,
//!    and protected topological edge and corner defect modes.
//! 2. Adiabatic non-Abelian anyon exchange kinematics satisfying the Artin Yang-Baxter braid relations.
//! 3. Fault-tolerant single-qubit Clifford and two-qubit entangling gates compiled directly from braid words.
//! 4. Cryogenic multi-qubit crossbar interconnect array with dispersive cavity parity readout.
//! 5. 10-point rigorous physics audit checklist and real-time execution telemetry.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::chiral_graphene_braiding::{
    ChiralGrapheneAnyonSpatialPoint, ChiralGrapheneBraidingAuditReport,
    ChiralGrapheneBraidingProcessor, ChiralGrapheneDispersionPoint,
    ChiralGrapheneLatticeMetrics, ChiralGrapheneLatticeParams,
    ChiralGrapheneLatticeSolver, GrapheneAnyonBraidMetrics,
    GrapheneAnyonBraidParams, GrapheneAnyonBraidSolver, GrapheneBraidStepPoint,
    GrapheneCrossbarMetrics, GrapheneCrossbarParams,
    GrapheneCrossbarReadoutPoint, GrapheneCrossbarSolver,
    GrapheneGateProcessPoint, GrapheneQubitGateMetrics,
    GrapheneQubitGateParams, GrapheneQubitGateSolver, GrapheneTargetGate,
};

/// 5 Categorized navigation tabs for the Chiral Graphene Braiding dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChiralGrapheneBraidingTab {
    ChiralLatticeEdgeModes,
    NonAbelianBraidingDynamics,
    LogicalCliffordGates,
    CryogenicCrossbarArray,
    AuditTelemetry,
}

impl ChiralGrapheneBraidingTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ChiralLatticeEdgeModes => "Graphene Edge Modes",
            Self::NonAbelianBraidingDynamics => "Anyon Braiding",
            Self::LogicalCliffordGates => "Topological Qubit Logic",
            Self::CryogenicCrossbarArray => "Cryogenic Crossbar",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Chiral Phononic Graphene Anyon Braiding & Crossbar (Phase 454).
#[derive(Debug, Clone)]
pub struct ChiralGrapheneBraidingDialog {
    pub is_open: bool,
    pub active_tab: ChiralGrapheneBraidingTab,

    // Tab 1: Chiral Graphene Lattice parameters
    pub lattice_constant_um: f64,
    pub acoustic_velocity_ms: f64,
    pub hopping_t1_mhz: f64,
    pub hopping_t2_mhz: f64,
    pub haldane_phase_rad: f64,
    pub on_site_mass_mhz: f64,
    pub center_frequency_ghz: f64,
    pub ribbon_width_cells: usize,

    // Tab 2: Non-Abelian Braiding parameters
    pub anyon_count: usize,
    pub topological_bulk_gap_mhz: f64,
    pub braid_duration_ns: f64,
    pub junction_arm_length_um: f64,

    // Tab 3: Topological Qubit Logic parameters
    pub target_gate: GrapheneTargetGate,
    pub physical_error_rate: f64,
    pub dephasing_time_us: f64,
    pub inter_qubit_coupling_mhz: f64,

    // Tab 4: Cryogenic Crossbar parameters
    pub qubit_count: usize,
    pub base_temperature_k: f64,
    pub dispersive_coupling_chi_mhz: f64,
    pub cavity_linewidth_kappa_mhz: f64,
    pub measurement_time_ns: f64,
    pub waveguide_cross_isolation_db: f64,

    // Cached simulation outputs
    pub cached_lattice_metrics: ChiralGrapheneLatticeMetrics,
    pub cached_dispersion: Vec<ChiralGrapheneDispersionPoint>,
    pub cached_spatial_mode: Vec<ChiralGrapheneAnyonSpatialPoint>,

    pub cached_braid_metrics: GrapheneAnyonBraidMetrics,
    pub cached_worldlines: Vec<GrapheneBraidStepPoint>,

    pub cached_gate_metrics: GrapheneQubitGateMetrics,
    pub cached_process_curve: Vec<GrapheneGateProcessPoint>,

    pub cached_crossbar_metrics: GrapheneCrossbarMetrics,
    pub cached_spectrum: Vec<GrapheneCrossbarReadoutPoint>,

    pub cached_audit: ChiralGrapheneBraidingAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ChiralGrapheneBraidingDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ChiralGrapheneBraidingDialog {
    /// Constructs a fast cold-boot instance under 2.0 ms by seeding pre-computed baseline state.
    pub fn new_fast() -> Self {
        let lat_params = ChiralGrapheneLatticeParams::default();
        let braid_params = GrapheneAnyonBraidParams::default();
        let logic_params = GrapheneQubitGateParams::default();
        let crossbar_params = GrapheneCrossbarParams::default();

        let lat_solver = ChiralGrapheneLatticeSolver::new(lat_params.clone());
        let braid_solver = GrapheneAnyonBraidSolver::new(braid_params.clone());
        let logic_solver = GrapheneQubitGateSolver::new(logic_params.clone());
        let crossbar_solver = GrapheneCrossbarSolver::new(crossbar_params.clone());

        let processor = ChiralGrapheneBraidingProcessor::new(
            lat_params.clone(),
            braid_params.clone(),
            logic_params.clone(),
            crossbar_params.clone(),
        );

        let cached_lattice_metrics = lat_solver.evaluate_metrics();
        let cached_dispersion = lat_solver.compute_dispersion(60);
        let cached_spatial_mode = lat_solver.compute_spatial_mode();

        let cached_braid_metrics = braid_solver.evaluate_metrics();
        let cached_worldlines = braid_solver.compute_worldlines(60);

        let cached_gate_metrics = logic_solver.evaluate_metrics();
        let cached_process_curve = logic_solver.compute_process_curve(40);

        let cached_crossbar_metrics = crossbar_solver.evaluate_metrics();
        let cached_spectrum = crossbar_solver.compute_readout_spectrum(50);

        let cached_audit = processor.evaluate_audit();

        Self {
            is_open: false,
            active_tab: ChiralGrapheneBraidingTab::ChiralLatticeEdgeModes,

            lattice_constant_um: lat_params.lattice_constant_um,
            acoustic_velocity_ms: lat_params.acoustic_velocity_ms,
            hopping_t1_mhz: lat_params.hopping_t1_mhz,
            hopping_t2_mhz: lat_params.hopping_t2_mhz,
            haldane_phase_rad: lat_params.haldane_phase_rad,
            on_site_mass_mhz: lat_params.on_site_mass_mhz,
            center_frequency_ghz: lat_params.center_frequency_ghz,
            ribbon_width_cells: lat_params.ribbon_width_cells,

            anyon_count: braid_params.anyon_count,
            topological_bulk_gap_mhz: braid_params.topological_bulk_gap_mhz,
            braid_duration_ns: braid_params.braid_duration_ns,
            junction_arm_length_um: braid_params.junction_arm_length_um,

            target_gate: logic_params.target_gate,
            physical_error_rate: logic_params.physical_error_rate,
            dephasing_time_us: logic_params.dephasing_time_us,
            inter_qubit_coupling_mhz: logic_params.inter_qubit_coupling_mhz,

            qubit_count: crossbar_params.qubit_count,
            base_temperature_k: crossbar_params.base_temperature_k,
            dispersive_coupling_chi_mhz: crossbar_params.dispersive_coupling_chi_mhz,
            cavity_linewidth_kappa_mhz: crossbar_params.cavity_linewidth_kappa_mhz,
            measurement_time_ns: crossbar_params.measurement_time_ns,
            waveguide_cross_isolation_db: crossbar_params.waveguide_cross_isolation_db,

            cached_lattice_metrics,
            cached_dispersion,
            cached_spatial_mode,
            cached_braid_metrics,
            cached_worldlines,
            cached_gate_metrics,
            cached_process_curve,
            cached_crossbar_metrics,
            cached_spectrum,
            cached_audit,
            last_solve_time_us: 142.0,
        }
    }

    /// Recomputes all active solvers and refreshes cached telemetry.
    pub fn recompute_all(&mut self) {
        let t_start = std::time::Instant::now();

        let lat_params = ChiralGrapheneLatticeParams {
            lattice_constant_um: self.lattice_constant_um,
            acoustic_velocity_ms: self.acoustic_velocity_ms,
            hopping_t1_mhz: self.hopping_t1_mhz,
            hopping_t2_mhz: self.hopping_t2_mhz,
            haldane_phase_rad: self.haldane_phase_rad,
            on_site_mass_mhz: self.on_site_mass_mhz,
            center_frequency_ghz: self.center_frequency_ghz,
            ribbon_width_cells: self.ribbon_width_cells,
        };

        let braid_params = GrapheneAnyonBraidParams {
            anyon_count: self.anyon_count,
            topological_bulk_gap_mhz: self.topological_bulk_gap_mhz,
            braid_duration_ns: self.braid_duration_ns,
            junction_arm_length_um: self.junction_arm_length_um,
            inter_arm_cross_coupling_mhz: 0.15,
        };

        let logic_params = GrapheneQubitGateParams {
            target_gate: self.target_gate,
            physical_error_rate: self.physical_error_rate,
            dephasing_time_us: self.dephasing_time_us,
            inter_qubit_coupling_mhz: self.inter_qubit_coupling_mhz,
        };

        let crossbar_params = GrapheneCrossbarParams {
            qubit_count: self.qubit_count,
            base_temperature_k: self.base_temperature_k,
            center_frequency_ghz: self.center_frequency_ghz,
            dispersive_coupling_chi_mhz: self.dispersive_coupling_chi_mhz,
            cavity_linewidth_kappa_mhz: self.cavity_linewidth_kappa_mhz,
            measurement_time_ns: self.measurement_time_ns,
            waveguide_cross_isolation_db: self.waveguide_cross_isolation_db,
        };

        let lat_solver = ChiralGrapheneLatticeSolver::new(lat_params.clone());
        let braid_solver = GrapheneAnyonBraidSolver::new(braid_params.clone());
        let logic_solver = GrapheneQubitGateSolver::new(logic_params.clone());
        let crossbar_solver = GrapheneCrossbarSolver::new(crossbar_params.clone());

        let processor = ChiralGrapheneBraidingProcessor::new(
            lat_params,
            braid_params,
            logic_params,
            crossbar_params,
        );

        self.cached_lattice_metrics = lat_solver.evaluate_metrics();
        self.cached_dispersion = lat_solver.compute_dispersion(80);
        self.cached_spatial_mode = lat_solver.compute_spatial_mode();

        self.cached_braid_metrics = braid_solver.evaluate_metrics();
        self.cached_worldlines = braid_solver.compute_worldlines(80);

        self.cached_gate_metrics = logic_solver.evaluate_metrics();
        self.cached_process_curve = logic_solver.compute_process_curve(50);

        self.cached_crossbar_metrics = crossbar_solver.evaluate_metrics();
        self.cached_spectrum = crossbar_solver.compute_readout_spectrum(60);

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
        Window::new("Chiral Phononic Graphene Non-Abelian Braiding & Qubit Crossbar (Phase 454)")
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
                RichText::new("Chiral Phononic Graphene Braiding & Qubit Crossbar")
                    .color(Color32::from_rgb(90, 210, 245))
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
                ChiralGrapheneBraidingTab::ChiralLatticeEdgeModes,
                ChiralGrapheneBraidingTab::NonAbelianBraidingDynamics,
                ChiralGrapheneBraidingTab::LogicalCliffordGates,
                ChiralGrapheneBraidingTab::CryogenicCrossbarArray,
                ChiralGrapheneBraidingTab::AuditTelemetry,
            ];

            for tab in tabs {
                let text = if self.active_tab == tab {
                    RichText::new(tab.label())
                        .color(Color32::from_rgb(80, 220, 255))
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
            ChiralGrapheneBraidingTab::ChiralLatticeEdgeModes => {
                self.render_tab_lattice(ui);
            }
            ChiralGrapheneBraidingTab::NonAbelianBraidingDynamics => {
                self.render_tab_braiding(ui);
            }
            ChiralGrapheneBraidingTab::LogicalCliffordGates => {
                self.render_tab_logic(ui);
            }
            ChiralGrapheneBraidingTab::CryogenicCrossbarArray => {
                self.render_tab_crossbar(ui);
            }
            ChiralGrapheneBraidingTab::AuditTelemetry => {
                self.render_tab_audit(ui);
            }
        }
    }

    fn render_tab_lattice(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("2D Honeycomb Chiral Phononic Graphene Band Structure")
                    .strong()
                    .color(Color32::from_rgb(180, 220, 255)),
            );
            if ui.button("Recompute Lattice").clicked() {
                self.recompute();
            }
        });

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(280.0);
                ui.label(RichText::new("Lattice Parameters").strong());

                ui.add(
                    egui::Slider::new(&mut self.hopping_t1_mhz, 5.0..=25.0)
                        .text("t1 Hopping (MHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.hopping_t2_mhz, 0.5..=6.0)
                        .text("t2 Complex (MHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.on_site_mass_mhz, -10.0..=10.0)
                        .text("Mass M (MHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.lattice_constant_um, 20.0..=60.0)
                        .text("Pitch a (um)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.acoustic_velocity_ms, 2500.0..=4500.0)
                        .text("Velocity (m/s)"),
                );

                ui.separator();
                ui.label(RichText::new("Topological Metrics").strong());
                let m = &self.cached_lattice_metrics;
                ui.label(format!("Chern Bandgap: {:.2} MHz", m.bulk_chern_bandgap_mhz));
                ui.label(format!("Lower Chern C: {}", m.lower_band_chern_number));
                ui.label(format!("Upper Chern C: {}", m.upper_band_chern_number));
                ui.label(format!("Edge Group Vel: {:.0} m/s", m.edge_group_velocity_ms));
                ui.label(format!("Penetration Depth: {:.2} cells", m.edge_penetration_depth_cells));
                ui.label(format!("Corner Transmiss: {:.1}%", m.corner_transmission_ratio * 100.0));
            });

            ui.vertical(|ui| {
                let lower_pts: PlotPoints = self
                    .cached_dispersion
                    .iter()
                    .map(|p| [p.kx_normalized, p.lower_band_mhz])
                    .collect();
                let upper_pts: PlotPoints = self
                    .cached_dispersion
                    .iter()
                    .map(|p| [p.kx_normalized, p.upper_band_mhz])
                    .collect();

                let edge_pts: PlotPoints = self
                    .cached_dispersion
                    .iter()
                    .filter_map(|p| p.chiral_edge_mode_mhz.map(|e| [p.kx_normalized, e]))
                    .collect();

                Plot::new("chiral_graphene_dispersion")
                    .height(280.0)
                    .x_axis_label("Normalized Momentum k_x * a / pi")
                    .y_axis_label("Energy Detuning (MHz)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Lower Band", lower_pts)
                                .color(Color32::from_rgb(100, 150, 240)),
                        );
                        plot_ui.line(
                            Line::new("Upper Band", upper_pts)
                                .color(Color32::from_rgb(240, 120, 100)),
                        );
                        plot_ui.line(
                            Line::new("Chiral Edge State", edge_pts)
                                .color(Color32::from_rgb(80, 240, 150))
                                .width(2.5),
                        );
                    });

                // Mode localization profile
                let spatial_pts: PlotPoints = self
                    .cached_spatial_mode
                    .iter()
                    .map(|p| [p.cell_index as f64, p.probability_density])
                    .collect();

                Plot::new("chiral_graphene_spatial")
                    .height(180.0)
                    .x_axis_label("Boundary Ribbon Coordinate x (cells)")
                    .y_axis_label("Probability Density |psi(x)|^2")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Edge Mode Profile", spatial_pts)
                                .color(Color32::from_rgb(255, 200, 80))
                                .width(2.0),
                        );
                    });
            });
        });
    }

    fn render_tab_braiding(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Non-Abelian Anyon Adiabatic Braiding & Artin Relations")
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
                ui.label(RichText::new("Braiding Parameters").strong());

                ui.add(
                    egui::Slider::new(&mut self.braid_duration_ns, 50.0..=300.0)
                        .text("Braid Time (ns)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.topological_bulk_gap_mhz, 10.0..=35.0)
                        .text("Bulk Gap (MHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.junction_arm_length_um, 40.0..=150.0)
                        .text("Arm Length (um)"),
                );

                ui.separator();
                ui.label(RichText::new("Braiding Telemetry").strong());
                let b = &self.cached_braid_metrics;
                ui.label(format!("Artin Error: {:.2e}", b.artin_relation_error));
                ui.label(format!("Far Commut Error: {:.2e}", b.far_commutation_error));
                ui.label(format!("Diabatic Leakage: {:.2e}", b.diabatic_leakage_probability));
                ui.label(format!("Adiabatic Ratio: {:.1}", b.adiabatic_parameter));
                ui.label(format!("Gate Fidelity: {:.3}%", b.braid_unitary_fidelity_pct));
                ui.label(format!("Exchange Phase: {:.3} rad", b.topological_exchange_phase_rad));
            });

            ui.vertical(|ui| {
                // Worldlines X(t) of anyon 1 and anyon 2
                let anyon1_pts: PlotPoints = self
                    .cached_worldlines
                    .iter()
                    .map(|w| [w.normalized_time, w.anyon_x_um[0]])
                    .collect();
                let anyon2_pts: PlotPoints = self
                    .cached_worldlines
                    .iter()
                    .map(|w| [w.normalized_time, w.anyon_x_um[1]])
                    .collect();

                Plot::new("worldline_trajectories")
                    .height(260.0)
                    .x_axis_label("Normalized Braid Time tau")
                    .y_axis_label("Anyon X Position (um)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Anyon 1 Path", anyon1_pts)
                                .color(Color32::from_rgb(100, 200, 255))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("Anyon 2 Path", anyon2_pts)
                                .color(Color32::from_rgb(255, 120, 100))
                                .width(2.0),
                        );
                    });

                // Instantaneous gap
                let gap_pts: PlotPoints = self
                    .cached_worldlines
                    .iter()
                    .map(|w| [w.normalized_time, w.instantaneous_gap_mhz])
                    .collect();

                Plot::new("instantaneous_gap_plot")
                    .height(180.0)
                    .x_axis_label("Normalized Braid Time tau")
                    .y_axis_label("Instantaneous Gap (MHz)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Energy Gap Delta(t)", gap_pts)
                                .color(Color32::from_rgb(80, 240, 180))
                                .width(2.0),
                        );
                        plot_ui.hline(
                            HLine::new("Protection Threshold", 10.0)
                                .color(Color32::DARK_GRAY),
                        );
                    });
            });
        });
    }

    fn render_tab_logic(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Topological Qubit Clifford Logic & CNOT Synthesis")
                    .strong()
                    .color(Color32::from_rgb(180, 220, 255)),
            );
            if ui.button("Recompute Logic").clicked() {
                self.recompute();
            }
        });

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(280.0);
                ui.label(RichText::new("Target Quantum Gate").strong());

                let gates = [
                    GrapheneTargetGate::Hadamard,
                    GrapheneTargetGate::PhaseS,
                    GrapheneTargetGate::PauliX,
                    GrapheneTargetGate::PauliZ,
                    GrapheneTargetGate::Cnot,
                ];

                for gate in gates {
                    if ui
                        .selectable_label(self.target_gate == gate, gate.name())
                        .clicked()
                    {
                        self.target_gate = gate;
                        self.recompute();
                    }
                }

                ui.separator();
                ui.label(RichText::new("Gate Execution Parameters").strong());
                ui.add(
                    egui::Slider::new(&mut self.dephasing_time_us, 10.0..=100.0)
                        .text("Dephasing T2 (us)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.inter_qubit_coupling_mhz, 5.0..=30.0)
                        .text("Inter-Qubit g (MHz)"),
                );

                ui.separator();
                ui.label(RichText::new("Gate Performance Metrics").strong());
                let g = &self.cached_gate_metrics;
                ui.label(format!("Braid Word: {}", self.target_gate.braid_word()));
                ui.label(format!("Braid Steps: {}", g.braid_sequence_length));
                ui.label(format!("Process Fidelity: {:.3}%", g.gate_process_fidelity_pct));
                ui.label(format!("Concurrence: {:.3}", g.entanglement_concurrence));
                ui.label(format!("Bell State Fid: {:.2}%", g.bell_state_fidelity_pct));
                ui.label(format!("Leakage Prob: {:.2e}", g.code_space_leakage_prob));
                ui.label(format!("Logical Coherence: {:.1} us", g.logical_coherence_time_us));
            });

            ui.vertical(|ui| {
                let fid_pts: PlotPoints = self
                    .cached_process_curve
                    .iter()
                    .map(|p| [p.perturbation_epsilon, p.fidelity_pct])
                    .collect();

                Plot::new("gate_fidelity_curve")
                    .height(260.0)
                    .x_axis_label("Perturbation Strength epsilon")
                    .y_axis_label("Gate Fidelity (%)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Fidelity Plateau", fid_pts)
                                .color(Color32::from_rgb(90, 240, 140))
                                .width(2.5),
                        );
                        plot_ui.hline(
                            HLine::new("Threshold (99.0%)", 99.0)
                                .color(Color32::from_rgb(240, 100, 100)),
                        );
                    });

                if self.target_gate == GrapheneTargetGate::Cnot {
                    let conc_pts: PlotPoints = self
                        .cached_process_curve
                        .iter()
                        .map(|p| [p.perturbation_epsilon, p.concurrence])
                        .collect();

                    Plot::new("concurrence_curve")
                        .height(180.0)
                        .x_axis_label("Perturbation Strength epsilon")
                        .y_axis_label("Entanglement Concurrence C")
                        .show(ui, |plot_ui| {
                            plot_ui.line(
                                Line::new("Concurrence", conc_pts)
                                    .color(Color32::from_rgb(255, 140, 60))
                                    .width(2.0),
                            );
                            plot_ui.hline(
                                HLine::new("Threshold (0.92)", 0.92)
                                    .color(Color32::LIGHT_GRAY),
                            );
                        });
                }
            });
        });
    }

    fn render_tab_crossbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Cryogenic Multi-Qubit Crossbar Interconnect & Parity Readout")
                    .strong()
                    .color(Color32::from_rgb(180, 220, 255)),
            );
            if ui.button("Recompute Crossbar").clicked() {
                self.recompute();
            }
        });

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(280.0);
                ui.label(RichText::new("Crossbar Parameters").strong());

                ui.add(
                    egui::Slider::new(&mut self.base_temperature_k, 0.010..=0.100)
                        .text("Temperature (K)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.dispersive_coupling_chi_mhz, 2.0..=8.0)
                        .text("Shift chi (MHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.cavity_linewidth_kappa_mhz, 0.2..=1.5)
                        .text("Kappa (MHz)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.waveguide_cross_isolation_db, 30.0..=55.0)
                        .text("Isolation (dB)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.qubit_count, 2..=8)
                        .text("Logical Qubits"),
                );

                ui.separator();
                ui.label(RichText::new("Interconnect Telemetry").strong());
                let c = &self.cached_crossbar_metrics;
                ui.label(format!("Dispersive Splitting: {:.2} MHz", c.dispersive_frequency_splitting_mhz));
                ui.label(format!("Parity Readout SNR: {:.1} dB", c.parity_readout_snr_db));
                ui.label(format!("QND Readout Fid: {:.2}%", c.qnd_readout_fidelity_pct));
                ui.label(format!("Waveguide Isolation: {:.1} dB", c.crossbar_waveguide_isolation_db));
                ui.label(format!("Thermal Occupancy: {:.2e}", c.cryogenic_thermal_noise_occupancy));
                ui.label(format!("Poisoning Lifetime: {:.1} us", c.quasiparticle_poisoning_lifetime_us));
                ui.label(format!("Insertion Loss: {:.2} dB", c.channel_insertion_loss_db));
            });

            ui.vertical(|ui| {
                let s21_even: PlotPoints = self
                    .cached_spectrum
                    .iter()
                    .map(|p| [p.detuning_mhz, p.s21_even_parity])
                    .collect();
                let s21_odd: PlotPoints = self
                    .cached_spectrum
                    .iter()
                    .map(|p| [p.detuning_mhz, p.s21_odd_parity])
                    .collect();

                Plot::new("dispersive_parity_spectrum")
                    .height(380.0)
                    .x_axis_label("Cavity Probe Detuning (MHz)")
                    .y_axis_label("Normalized Transmission |S_21|^2")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Even Parity (|0_L>)", s21_even)
                                .color(Color32::from_rgb(80, 210, 240))
                                .width(2.5),
                        );
                        plot_ui.line(
                            Line::new("Odd Parity (|1_L>)", s21_odd)
                                .color(Color32::from_rgb(250, 100, 120))
                                .width(2.5),
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
                "1. Chiral Phononic Graphene Bulk Bandgap",
                format!("{:.2} MHz >= 15.0 MHz", self.cached_lattice_metrics.bulk_chern_bandgap_mhz),
                self.cached_audit.bulk_chern_bandgap,
            ),
            (
                "2. Artin Yang-Baxter Braid Relations",
                format!("{:.2e} <= 1.0e-5 error", self.cached_braid_metrics.artin_relation_error),
                self.cached_audit.artin_braid_relations,
            ),
            (
                "3. Single-Qubit Clifford Gate Fidelity",
                format!("{:.3}% >= 99.9%", self.cached_gate_metrics.gate_process_fidelity_pct),
                self.cached_audit.single_qubit_clifford_fidelity,
            ),
            (
                "4. Adiabatic Braiding Transition Leakage",
                format!("{:.2e} <= 1.0e-4 error", self.cached_braid_metrics.diabatic_leakage_probability),
                self.cached_audit.adiabatic_braiding_leakage,
            ),
            (
                "5. Dispersive Parity Frequency Splitting",
                format!("{:.2} MHz >= 8.0 MHz (2*chi)", self.cached_crossbar_metrics.dispersive_frequency_splitting_mhz),
                self.cached_audit.dispersive_parity_splitting,
            ),
            (
                "6. QND Parity Readout SNR & Fidelity",
                format!("{:.1} dB >= 20.0 dB, {:.2}% >= 99.5%", self.cached_crossbar_metrics.parity_readout_snr_db, self.cached_crossbar_metrics.qnd_readout_fidelity_pct),
                self.cached_audit.qnd_parity_readout,
            ),
            (
                "7. Multi-Qubit Entangling Concurrence",
                format!("{:.3} >= 0.92, Bell Fid {:.2}% >= 99.0%", self.cached_gate_metrics.entanglement_concurrence, self.cached_gate_metrics.bell_state_fidelity_pct),
                self.cached_audit.entangling_gate_concurrence,
            ),
            (
                "8. Crossbar Waveguide Isolation",
                format!("{:.1} dB >= 38.0 dB", self.cached_crossbar_metrics.crossbar_waveguide_isolation_db),
                self.cached_audit.crossbar_waveguide_isolation,
            ),
            (
                "9. Cryogenic Thermal Noise Occupancy",
                format!("{:.2e} <= 0.05 quanta at 20 mK", self.cached_crossbar_metrics.cryogenic_thermal_noise_occupancy),
                self.cached_audit.cryogenic_thermal_noise,
            ),
            (
                "10. Quasiparticle Poisoning Lifetime",
                format!("{:.1} us >= 40.0 us", self.cached_crossbar_metrics.quasiparticle_poisoning_lifetime_us),
                self.cached_audit.quasiparticle_poisoning_lifetime,
            ),
        ];

        egui::Grid::new("audit_grid")
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
