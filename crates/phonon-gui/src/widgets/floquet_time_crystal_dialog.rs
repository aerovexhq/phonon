#![deny(unsafe_code)]

//! Interactive Floquet-Bloch Quantum Acoustic Discrete Time Crystal (DTC) Visualizer
//! for Phonon CAD Studio.
//!
//! Provides:
//! - Stroboscopic Time Dynamics Plot: egui_plot rendering M_z(t) over 60 Floquet cycles,
//!   showing rigid 2T period-doubling oscillation and discrete time-translation symmetry breaking.
//! - Subharmonic Fourier Power Spectrum: egui_plot rendering S(omega) vs omega/Omega in [0.0, 1.0],
//!   highlighting the sharp delta-like peak at the subharmonic ratio 0.5.
//! - Perturbation Rigidity Plateau Curve: egui_plot rendering subharmonic intensity vs pulse error
//!   epsilon in [-0.2, 0.2], showing the broad flat DTC stability plateau.
//! - 2D Lattice Polarization Canvas: interactive diagram of the 1D chain of N acoustic sites displaying
//!   cycle-stepped polarization synchronously reversing every driving period.
//! - Controls: Chain length slider, drive period T, pulse error epsilon slider, disorder W slider,
//!   coupling J slider, initial state selector.
//! - Presets: "Stable Discrete Time Crystal (epsilon=0.05)", "Zero Perturbation DTC (epsilon=0.0)",
//!   "Thermal Ergodic Regime (W=0, no MBL)", "Trivial Paramagnet (J=0)".
//! - Action buttons: "Run Simulation", "Sweep Rigidity Plateau", "Cycle Stroboscopic Step".
//! - Telemetry Footer: Time-Crystalline Phase Status, Subharmonic Peak Ratio (%), Quasi-Energy Pi-Gap (rad),
//!   Rigidity Plateau Width, Edwards-Anderson Order q_EA, Floquet Period T (us).

use std::f64::consts::PI;
use egui::{
    vec2, Align2, Color32, FontId, Pos2, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, Points, VLine};
use phonon_solver::floquet_time_crystal::{
    EdwardsAndersonOrder, FloquetState, FloquetStateKind, FloquetTimeCrystalParams,
    FloquetUnitaryOperator, RigidityPhaseDiagram, StroboscopicTrajectory,
    SubharmonicSpectralAnalysis,
};

/// Theme colors for Floquet Time Crystal Visualizer.
const COLOR_TOPO_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const COLOR_TOPO_EMERALD: Color32 = Color32::from_rgb(52, 211, 153);
const COLOR_TRIVIAL_AMBER: Color32 = Color32::from_rgb(251, 191, 36);
const COLOR_SUBHARMONIC_GOLD: Color32 = Color32::from_rgb(250, 204, 21);
const COLOR_SPIN_UP_ROSE: Color32 = Color32::from_rgb(244, 63, 94);
const COLOR_SPIN_DOWN_BLUE: Color32 = Color32::from_rgb(59, 130, 246);
const COLOR_GRID_LINE: Color32 = Color32::from_rgb(51, 65, 85);
const COLOR_TEXT_DIM: Color32 = Color32::from_rgb(148, 163, 184);

/// Active plot tab for visualizer layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloquetPlotTab {
    #[default]
    StroboscopicDynamics,
    FourierSpectrum,
    RigidityPlateau,
    AllPlotsCombined,
}

impl FloquetPlotTab {
    pub fn label(&self) -> &'static str {
        match self {
            FloquetPlotTab::StroboscopicDynamics => "2T Dynamics M_z(t)",
            FloquetPlotTab::FourierSpectrum => "Subharmonic S(omega)",
            FloquetPlotTab::RigidityPlateau => "Rigidity Plateau",
            FloquetPlotTab::AllPlotsCombined => "Combined View",
        }
    }
}

/// Modal dialog for the Phonon Studio Floquet Acoustic Time Crystal Simulator.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetTimeCrystalDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Controls
    /// Number of acoustic chain sites N (4 to 12).
    pub chain_length: usize,
    /// Drive period T in microseconds (0.1 to 10.0 us).
    pub drive_period_us: f64,
    /// Pulse perturbation error epsilon in [-0.25, 0.25].
    pub pulse_error_epsilon: f64,
    /// Ising coupling ratio J / J_default.
    pub coupling_ratio: f64,
    /// MBL longitudinal disorder ratio W / J.
    pub disorder_ratio: f64,
    /// Acoustic dissipation rate gamma in 1/ms.
    pub damping_gamma_per_ms: f64,
    /// Initial quantum spin configuration.
    pub initial_state_kind: FloquetStateKind,

    // Active visualizer state
    /// Active plot tab view.
    pub active_plot_tab: FloquetPlotTab,
    /// Selected stroboscopic step for lattice canvas display.
    pub stroboscopic_step: usize,

    // Simulation Engine & Cached Results
    pub params: FloquetTimeCrystalParams,
    pub trajectory: StroboscopicTrajectory,
    pub spectral_analysis: SubharmonicSpectralAnalysis,
    pub phase_diagram: RigidityPhaseDiagram,
    pub edwards_anderson: EdwardsAndersonOrder,
    pub pi_pairing_gap_rad: f64,

    // Cached Plot Curves
    pub dynamics_points: Vec<[f64; 2]>,
    pub edwards_anderson_points: Vec<[f64; 2]>,
    pub fourier_points: Vec<[f64; 2]>,
    pub plateau_points: Vec<[f64; 2]>,

    pub status_msg: String,
}

impl Default for FloquetTimeCrystalDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl FloquetTimeCrystalDialog {
    /// Creates a new FloquetTimeCrystalDialog initialized to the Stable DTC preset.
    pub fn new() -> Self {
        let params = FloquetTimeCrystalParams::preset_stable_dtc();
        let timestamps: Vec<f64> = (0..60).map(|i| (i as f64) * params.drive_period_t).collect();
        let average_magnetization: Vec<f64> = (0..60).map(|i| if i % 2 == 0 { 0.95 } else { -0.95 }).collect();
        let site_polarizations: Vec<Vec<f64>> = (0..60)
            .map(|i| vec![if i % 2 == 0 { 0.95 } else { -0.95 }; params.chain_length])
            .collect();
        let trajectory = StroboscopicTrajectory {
            timestamps,
            average_magnetization,
            site_polarizations,
            cycle_count: 60,
            drive_period_t: params.drive_period_t,
        };
        let spectral_analysis = SubharmonicSpectralAnalysis::from_trajectory(&trajectory);
        let phase_diagram = RigidityPhaseDiagram {
            epsilons: vec![-0.20, -0.10, 0.00, 0.10, 0.20],
            subharmonic_intensities: vec![0.72, 0.88, 0.96, 0.88, 0.72],
            plateau_min_epsilon: -0.20,
            plateau_max_epsilon: 0.20,
            plateau_width: 0.40,
            is_dtc_phase: true,
        };
        let edwards_anderson = EdwardsAndersonOrder::compute(&trajectory);
        let pi_pairing_gap_rad = std::f64::consts::PI;

        let mut dialog = Self {
            is_open: false,
            chain_length: params.chain_length,
            drive_period_us: params.drive_period_t * 1.0e6,
            pulse_error_epsilon: params.pulse_error_epsilon,
            coupling_ratio: 1.0,
            disorder_ratio: params.disorder_field_w / params.coupling_j.max(1e-12),
            damping_gamma_per_ms: params.damping_gamma * 1.0e-3,
            initial_state_kind: FloquetStateKind::AllUp,
            active_plot_tab: FloquetPlotTab::StroboscopicDynamics,
            stroboscopic_step: 0,
            params,
            trajectory,
            spectral_analysis,
            phase_diagram,
            edwards_anderson,
            pi_pairing_gap_rad,
            dynamics_points: Vec::new(),
            edwards_anderson_points: Vec::new(),
            fourier_points: Vec::new(),
            plateau_points: Vec::new(),
            status_msg: String::from("Topological Discrete Time Crystal Engine Ready"),
        };

        dialog.update_plot_curves();
        dialog
    }

    /// Recomputes the unitary drive operator, stroboscopic trajectory, and spectral order.
    pub fn recompute(&mut self) {
        let drive_period_t = self.drive_period_us * 1.0e-6;
        let t2 = drive_period_t * 0.5;
        let j_base = PI / (2.0 * t2);
        let coupling_j = j_base * self.coupling_ratio;
        let delta_j = 0.2 * coupling_j;
        let disorder_w = coupling_j * self.disorder_ratio;
        let damping_gamma = self.damping_gamma_per_ms * 1.0e3;

        self.params = FloquetTimeCrystalParams::new(
            self.chain_length,
            drive_period_t,
            0.5,
            self.pulse_error_epsilon,
            coupling_j,
            delta_j,
            disorder_w,
            damping_gamma,
        );

        let op = FloquetUnitaryOperator::new(&self.params);
        let state0 = FloquetState::from_kind(self.initial_state_kind, self.chain_length, 42);
        self.trajectory = op.evolve(&state0, 60);
        self.spectral_analysis = SubharmonicSpectralAnalysis::from_trajectory(&self.trajectory);
        self.edwards_anderson = EdwardsAndersonOrder::compute(&self.trajectory);
        self.pi_pairing_gap_rad = op.compute_pi_quasienergy_pairing_gap();

        if self.stroboscopic_step >= self.trajectory.cycle_count {
            self.stroboscopic_step = 0;
        }

        self.update_plot_curves();

        if self.coupling_ratio < 0.05 {
            self.status_msg = "Trivial Paramagnet: Interactions absent, drifting frequency".to_string();
        } else if self.disorder_ratio < 0.05 {
            self.status_msg = "Thermal Ergodic regime: Fast decoherence without MBL disorder".to_string();
        } else if self.spectral_analysis.is_rigidly_locked {
            self.status_msg = format!(
                "Topological DTC phase: Subharmonic locked at 0.5 * Omega (Peak: {:.1}%)",
                self.spectral_analysis.subharmonic_fraction * 100.0
            );
        } else {
            self.status_msg = "Subharmonic Cross-Over: Partial discrete time-translation breaking".to_string();
        }
    }

    /// Sweeps the perturbation parameter epsilon to recalculate the rigidity phase diagram.
    pub fn sweep_rigidity_plateau(&mut self) {
        self.phase_diagram = RigidityPhaseDiagram::compute(&self.params, 31, 40);
        self.update_plateau_curve();
        self.status_msg = format!(
            "Rigidity scan complete: Plateau width Delta eps = {:.3} (DTC: {})",
            self.phase_diagram.plateau_width,
            if self.phase_diagram.is_dtc_phase { "Robust" } else { "Narrow" }
        );
    }

    /// Advances the stroboscopic cycle step by one.
    pub fn cycle_stroboscopic_step(&mut self) {
        if self.trajectory.cycle_count > 0 {
            self.stroboscopic_step = (self.stroboscopic_step + 1) % self.trajectory.cycle_count;
        }
    }

    /// Updates cached plot points.
    fn update_plot_curves(&mut self) {
        // Dynamics M_z(t) vs cycle
        self.dynamics_points.clear();
        for (n, &m) in self.trajectory.average_magnetization.iter().enumerate() {
            self.dynamics_points.push([n as f64, m]);
        }

        // Edwards-Anderson q_EA vs cycle
        self.edwards_anderson_points.clear();
        for (n, &q) in self.edwards_anderson.correlations.iter().enumerate() {
            self.edwards_anderson_points.push([n as f64, q]);
        }

        // Fourier Spectrum S(omega) vs omega / Omega
        self.fourier_points.clear();
        for (idx, &nu) in self.spectral_analysis.frequencies.iter().enumerate() {
            let power = self.spectral_analysis.power_spectrum[idx];
            self.fourier_points.push([nu, power]);
        }

        self.update_plateau_curve();
    }

    /// Updates cached plateau curve points.
    fn update_plateau_curve(&mut self) {
        self.plateau_points.clear();
        for (idx, &eps) in self.phase_diagram.epsilons.iter().enumerate() {
            let intensity = self.phase_diagram.subharmonic_intensities[idx];
            self.plateau_points.push([eps, intensity]);
        }
    }

    /// Presets
    pub fn set_preset_stable_dtc(&mut self) {
        self.pulse_error_epsilon = 0.05;
        self.coupling_ratio = 1.0;
        self.disorder_ratio = 2.0;
        self.recompute();
    }

    pub fn set_preset_zero_perturbation(&mut self) {
        self.pulse_error_epsilon = 0.0;
        self.coupling_ratio = 1.0;
        self.disorder_ratio = 2.0;
        self.recompute();
    }

    pub fn set_preset_thermal_ergodic(&mut self) {
        self.pulse_error_epsilon = 0.05;
        self.coupling_ratio = 1.0;
        self.disorder_ratio = 0.0;
        self.recompute();
    }

    pub fn set_preset_trivial_paramagnet(&mut self) {
        self.pulse_error_epsilon = 0.05;
        self.coupling_ratio = 0.0;
        self.disorder_ratio = 0.0;
        self.recompute();
    }

    /// Renders the modal window UI.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Floquet Time Crystal Studio - Topological Acoustic Simulator")
            .open(&mut is_open)
            .default_width(940.0)
            .default_height(680.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders inner window content.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut dirty = false;

        // Top Toolbar: Presets and Colormaps
        ui.horizontal(|ui| {
            ui.label(RichText::new("Presets:").strong().color(COLOR_TOPO_CYAN));

            if ui.button("Stable DTC (eps=0.05)").clicked() {
                self.set_preset_stable_dtc();
            }
            if ui.button("Zero Perturbation DTC (eps=0.0)").clicked() {
                self.set_preset_zero_perturbation();
            }
            if ui.button("Thermal Ergodic (W=0)").clicked() {
                self.set_preset_thermal_ergodic();
            }
            if ui.button("Trivial Paramagnet (J=0)").clicked() {
                self.set_preset_trivial_paramagnet();
            }

            ui.separator();

            ui.label(RichText::new("State:").color(COLOR_TEXT_DIM));
            egui::ComboBox::from_id_salt("ftc_state_kind")
                .selected_text(match self.initial_state_kind {
                    FloquetStateKind::AllUp => "All Up |00...0>",
                    FloquetStateKind::NeelState => "Neel |0101...>",
                    FloquetStateKind::Random => "Random Product",
                })
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.initial_state_kind, FloquetStateKind::AllUp, "All Up |00...0>").clicked() {
                        dirty = true;
                    }
                    if ui.selectable_value(&mut self.initial_state_kind, FloquetStateKind::NeelState, "Neel |0101...>").clicked() {
                        dirty = true;
                    }
                    if ui.selectable_value(&mut self.initial_state_kind, FloquetStateKind::Random, "Random Product").clicked() {
                        dirty = true;
                    }
                });
        });

        ui.add_space(4.0);

        // Parameter Sliders Bar
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Slider::new(&mut self.chain_length, 4..=12)
                        .text("Sites N")
                        .step_by(1.0),
                )
                .changed()
            {
                dirty = true;
            }

            if ui
                .add(
                    egui::Slider::new(&mut self.drive_period_us, 0.2..=5.0)
                        .text("Period T (us)")
                        .step_by(0.1),
                )
                .changed()
            {
                dirty = true;
            }

            if ui
                .add(
                    egui::Slider::new(&mut self.pulse_error_epsilon, -0.25..=0.25)
                        .text("Error eps")
                        .step_by(0.01),
                )
                .changed()
            {
                dirty = true;
            }

            if ui
                .add(
                    egui::Slider::new(&mut self.coupling_ratio, 0.0..=2.5)
                        .text("J / J0")
                        .step_by(0.1),
                )
                .changed()
            {
                dirty = true;
            }

            if ui
                .add(
                    egui::Slider::new(&mut self.disorder_ratio, 0.0..=5.0)
                        .text("W / J")
                        .step_by(0.1),
                )
                .changed()
            {
                dirty = true;
            }
        });

        ui.add_space(4.0);

        // Action Buttons Row
        ui.horizontal(|ui| {
            if ui.button(RichText::new("Run Simulation").strong().color(COLOR_TOPO_EMERALD)).clicked() {
                self.recompute();
            }

            if ui.button(RichText::new("Sweep Rigidity Plateau").color(COLOR_SUBHARMONIC_GOLD)).clicked() {
                self.sweep_rigidity_plateau();
            }

            if ui.button(RichText::new("Cycle Stroboscopic Step").color(COLOR_TOPO_CYAN)).clicked() {
                self.cycle_stroboscopic_step();
            }

            ui.separator();

            ui.label(RichText::new("Step n:").color(COLOR_TEXT_DIM));
            let max_cycle = self.trajectory.cycle_count.saturating_sub(1);
            ui.add(egui::Slider::new(&mut self.stroboscopic_step, 0..=max_cycle).text("/ 60"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(&self.status_msg).size(11.0).color(COLOR_TEXT_DIM));
            });
        });

        if dirty {
            self.recompute();
        }

        ui.separator();

        // 2D Lattice Polarization Canvas (Interactive 1D chain with cycle-stepped polarizations)
        self.render_lattice_canvas(ui);

        ui.separator();

        // Plot Tabs Bar
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.active_plot_tab,
                FloquetPlotTab::StroboscopicDynamics,
                FloquetPlotTab::StroboscopicDynamics.label(),
            );
            ui.selectable_value(
                &mut self.active_plot_tab,
                FloquetPlotTab::FourierSpectrum,
                FloquetPlotTab::FourierSpectrum.label(),
            );
            ui.selectable_value(
                &mut self.active_plot_tab,
                FloquetPlotTab::RigidityPlateau,
                FloquetPlotTab::RigidityPlateau.label(),
            );
            ui.selectable_value(
                &mut self.active_plot_tab,
                FloquetPlotTab::AllPlotsCombined,
                FloquetPlotTab::AllPlotsCombined.label(),
            );
        });

        ui.add_space(4.0);

        // Plot Area
        match self.active_plot_tab {
            FloquetPlotTab::StroboscopicDynamics => self.render_dynamics_plot(ui, 240.0),
            FloquetPlotTab::FourierSpectrum => self.render_fourier_plot(ui, 240.0),
            FloquetPlotTab::RigidityPlateau => self.render_plateau_plot(ui, 240.0),
            FloquetPlotTab::AllPlotsCombined => {
                ui.columns(3, |columns| {
                    columns[0].vertical(|ui| {
                        ui.label(RichText::new("Dynamics M_z(t)").strong().size(11.0));
                        self.render_dynamics_plot(ui, 210.0);
                    });
                    columns[1].vertical(|ui| {
                        ui.label(RichText::new("Fourier Spectrum S(omega)").strong().size(11.0));
                        self.render_fourier_plot(ui, 210.0);
                    });
                    columns[2].vertical(|ui| {
                        ui.label(RichText::new("Rigidity Plateau S_sub(eps)").strong().size(11.0));
                        self.render_plateau_plot(ui, 210.0);
                    });
                });
            }
        }

        ui.separator();

        // Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders the 2D Real-Space Acoustic Lattice Polarization Canvas.
    fn render_lattice_canvas(&self, ui: &mut Ui) {
        let canvas_size = vec2(ui.available_width().max(320.0), 140.0);
        let (response, painter) = ui.allocate_painter(canvas_size, Sense::hover());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, COLOR_GRID_LINE), StrokeKind::Inside);

        let n = self.chain_length.max(2);
        let margin_x = 45.0;
        let spacing_x = (rect.width() - 2.0 * margin_x) / ((n - 1) as f32);
        let center_y = rect.center().y + 5.0;

        let step = self.stroboscopic_step.min(self.trajectory.site_polarizations.len().saturating_sub(1));
        let polarizations: &[f64] = if !self.trajectory.site_polarizations.is_empty() {
            &self.trajectory.site_polarizations[step]
        } else {
            &[]
        };

        // Title and Cycle Indicator
        let title_text = format!(
            "1D Acoustic Spin Chain Polarization [Stroboscopic Cycle n = {} / {} | t = {:.2} us]",
            step,
            self.trajectory.cycle_count.saturating_sub(1),
            (step as f64) * self.params.drive_period_t * 1.0e6
        );
        painter.text(
            Pos2::new(rect.min.x + 12.0, rect.min.y + 12.0),
            Align2::LEFT_TOP,
            title_text,
            FontId::proportional(12.0),
            COLOR_TOPO_CYAN,
        );

        // Draw Ising Coupling Bonds between sites
        for i in 0..(n - 1) {
            let x1 = rect.min.x + margin_x + (i as f32) * spacing_x;
            let x2 = rect.min.x + margin_x + ((i + 1) as f32) * spacing_x;
            let p1 = Pos2::new(x1, center_y);
            let p2 = Pos2::new(x2, center_y);

            painter.line_segment([p1, p2], Stroke::new(2.5, Color32::from_rgb(71, 85, 105)));

            // Bond label
            let bond_mid = Pos2::new((x1 + x2) * 0.5, center_y - 12.0);
            painter.text(
                bond_mid,
                Align2::CENTER_CENTER,
                "J",
                FontId::proportional(9.0),
                COLOR_TEXT_DIM,
            );
        }

        // Draw Acoustic Resonator Sites
        let node_radius = 16.0;
        for i in 0..n {
            let x = rect.min.x + margin_x + (i as f32) * spacing_x;
            let pos = Pos2::new(x, center_y);

            let m_val = if i < polarizations.len() { polarizations[i] } else { 0.0 };
            let is_up = m_val >= 0.0;

            // Interpolate color between Rose (spin up) and Blue (spin down)
            let color = if is_up {
                let factor = (m_val.abs() as f32).clamp(0.2, 1.0);
                Color32::from_rgb(
                    (244.0 * factor) as u8,
                    (63.0 * factor) as u8,
                    (94.0 * factor) as u8,
                )
            } else {
                let factor = (m_val.abs() as f32).clamp(0.2, 1.0);
                Color32::from_rgb(
                    (59.0 * factor) as u8,
                    (130.0 * factor) as u8,
                    (246.0 * factor) as u8,
                )
            };

            // Node circle
            painter.circle_filled(pos, node_radius, color);
            painter.circle_stroke(
                pos,
                node_radius,
                Stroke::new(1.5, if is_up { Color32::WHITE } else { COLOR_TOPO_CYAN }),
            );

            // Polarization arrow indicator
            let arrow_len = 8.0;
            if is_up {
                painter.line_segment(
                    [Pos2::new(x, center_y + arrow_len), Pos2::new(x, center_y - arrow_len)],
                    Stroke::new(2.0, Color32::WHITE),
                );
                // Arrow head up
                painter.line_segment(
                    [Pos2::new(x - 3.5, center_y - arrow_len + 4.0), Pos2::new(x, center_y - arrow_len)],
                    Stroke::new(2.0, Color32::WHITE),
                );
                painter.line_segment(
                    [Pos2::new(x + 3.5, center_y - arrow_len + 4.0), Pos2::new(x, center_y - arrow_len)],
                    Stroke::new(2.0, Color32::WHITE),
                );
            } else {
                painter.line_segment(
                    [Pos2::new(x, center_y - arrow_len), Pos2::new(x, center_y + arrow_len)],
                    Stroke::new(2.0, Color32::WHITE),
                );
                // Arrow head down
                painter.line_segment(
                    [Pos2::new(x - 3.5, center_y + arrow_len - 4.0), Pos2::new(x, center_y + arrow_len)],
                    Stroke::new(2.0, Color32::WHITE),
                );
                painter.line_segment(
                    [Pos2::new(x + 3.5, center_y + arrow_len - 4.0), Pos2::new(x, center_y + arrow_len)],
                    Stroke::new(2.0, Color32::WHITE),
                );
            }

            // Site index label above
            painter.text(
                Pos2::new(x, center_y - node_radius - 8.0),
                Align2::CENTER_BOTTOM,
                format!("Site {i}"),
                FontId::proportional(10.0),
                COLOR_TOPO_CYAN,
            );

            // Numerical polarization value below
            painter.text(
                Pos2::new(x, center_y + node_radius + 6.0),
                Align2::CENTER_TOP,
                format!("{m_val:+.2}"),
                FontId::proportional(10.0),
                if is_up { COLOR_SPIN_UP_ROSE } else { COLOR_SPIN_DOWN_BLUE },
            );
        }
    }

    /// Renders the Stroboscopic Dynamics egui_plot.
    fn render_dynamics_plot(&self, ui: &mut Ui, height: f32) {
        let plot = Plot::new("ftc_dynamics_plot")
            .height(height)
            .x_axis_label("Driving Cycle n")
            .y_axis_label("Magnetization M_z(n T)")
            .legend(Legend::default())
            .include_y(-1.05)
            .include_y(1.05);

        let dyn_line = Line::new("M_z(n T)", PlotPoints::new(self.dynamics_points.clone()))
            .color(COLOR_TOPO_CYAN)
            .width(1.8);

        let dyn_points = Points::new("M_z Samples", PlotPoints::new(self.dynamics_points.clone()))
            .color(COLOR_SUBHARMONIC_GOLD)
            .radius(3.0);

        let ea_line = Line::new("q_EA(n T)", PlotPoints::new(self.edwards_anderson_points.clone()))
            .color(COLOR_TOPO_EMERALD)
            .width(1.4);

        plot.show(ui, |plot_ui| {
            plot_ui.hline(HLine::new("Zero", 0.0).color(COLOR_GRID_LINE));
            plot_ui.line(dyn_line);
            plot_ui.points(dyn_points);
            plot_ui.line(ea_line);
        });
    }

    /// Renders the Subharmonic Fourier Power Spectrum egui_plot.
    fn render_fourier_plot(&self, ui: &mut Ui, height: f32) {
        let plot = Plot::new("ftc_fourier_plot")
            .height(height)
            .x_axis_label("Normalized Frequency omega / Omega")
            .y_axis_label("Power Spectrum S(omega)")
            .legend(Legend::default())
            .include_x(0.0)
            .include_x(1.0);

        let fourier_line = Line::new("S(omega)", PlotPoints::new(self.fourier_points.clone()))
            .color(COLOR_SUBHARMONIC_GOLD)
            .width(2.0);

        let fourier_points = Points::new("DFT Bins", PlotPoints::new(self.fourier_points.clone()))
            .color(COLOR_TOPO_CYAN)
            .radius(2.5);

        plot.show(ui, |plot_ui| {
            plot_ui.vline(
                VLine::new("Subharmonic omega = 0.5 Omega", 0.5)
                    .color(COLOR_SPIN_UP_ROSE)
                    .stroke(Stroke::new(1.8, COLOR_SPIN_UP_ROSE)),
            );
            plot_ui.line(fourier_line);
            plot_ui.points(fourier_points);
        });
    }

    /// Renders the Rigidity Plateau egui_plot.
    fn render_plateau_plot(&self, ui: &mut Ui, height: f32) {
        let plot = Plot::new("ftc_plateau_plot")
            .height(height)
            .x_axis_label("Pulse Error epsilon")
            .y_axis_label("Subharmonic Fraction S_sub")
            .legend(Legend::default())
            .include_y(0.0)
            .include_y(1.05)
            .include_x(-0.22)
            .include_x(0.22);

        let plateau_line = Line::new("S_sub(eps)", PlotPoints::new(self.plateau_points.clone()))
            .color(COLOR_TOPO_EMERALD)
            .width(2.2);

        let plateau_points = Points::new("Samples", PlotPoints::new(self.plateau_points.clone()))
            .color(COLOR_SUBHARMONIC_GOLD)
            .radius(3.0);

        plot.show(ui, |plot_ui| {
            plot_ui.hline(
                HLine::new("DTC Threshold", 0.60)
                    .color(COLOR_TRIVIAL_AMBER)
                    .stroke(Stroke::new(1.2, COLOR_TRIVIAL_AMBER)),
            );
            plot_ui.vline(VLine::new("Zero Error", 0.0).color(COLOR_GRID_LINE));
            plot_ui.line(plateau_line);
            plot_ui.points(plateau_points);
        });
    }

    /// Renders the Telemetry Footer with quantitative indicators.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let (phase_label, phase_color) = if self.coupling_ratio < 0.05 {
            ("Trivial Paramagnet (Unlocked)", COLOR_TRIVIAL_AMBER)
        } else if self.disorder_ratio < 0.05 {
            ("Thermal Ergodic Regime (Fast Decoherence)", COLOR_TRIVIAL_AMBER)
        } else if self.spectral_analysis.is_rigidly_locked && self.phase_diagram.is_dtc_phase {
            ("Topological Discrete Time Crystal (MBL Protected)", COLOR_TOPO_EMERALD)
        } else {
            ("Subharmonic Cross-Over", COLOR_TOPO_CYAN)
        };

        let subharmonic_pct = self.spectral_analysis.subharmonic_fraction * 100.0;
        let pairing_gap = self.pi_pairing_gap_rad;
        let plateau_width = self.phase_diagram.plateau_width;
        let ea_order = self.edwards_anderson.asymptotic_order;
        let t_us = self.params.drive_period_t * 1.0e6;
        let omega_mhz = self.params.floquet_frequency_mhz();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Phase:").strong().size(11.0));
            ui.label(RichText::new(phase_label).strong().color(phase_color).size(11.0));

            ui.separator();

            ui.label(RichText::new(format!("S(Omega/2): {subharmonic_pct:.1}%")).color(COLOR_SUBHARMONIC_GOLD).size(11.0));

            ui.separator();

            ui.label(RichText::new(format!("Pi-Gap: {pairing_gap:.4} rad")).color(COLOR_TOPO_CYAN).size(11.0));

            ui.separator();

            ui.label(RichText::new(format!("Plateau Width: Delta eps = {plateau_width:.3}")).color(COLOR_TOPO_EMERALD).size(11.0));

            ui.separator();

            ui.label(RichText::new(format!("q_EA: {ea_order:.3}")).color(Color32::WHITE).size(11.0));

            ui.separator();

            ui.label(RichText::new(format!("Period: {t_us:.2} us ({omega_mhz:.2} MHz)")).color(COLOR_TEXT_DIM).size(11.0));
        });
    }
}
