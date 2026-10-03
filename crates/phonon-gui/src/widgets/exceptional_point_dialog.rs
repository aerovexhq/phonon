#![deny(unsafe_code)]

//! Interactive Exceptional Point Sensor & Parity-Time (PT) Symmetric Circuit Visualizer.
//!
//! Provides Riemann surface complex eigenvalue branch tracking, PT-symmetric coupled
//! RLC transient circuit waveform display, sub-threshold fractional sensitivity gauge,
//! and real-time non-Hermitian telemetry for Phonon Visual Studio.

use egui::{vec2, Color32, ProgressBar, RichText, Ui};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use phonon_solver::ep_sensor::{
    Complex, EpOrder, NonHermitianHamiltonian, PtCircuitParams, PtCircuitState, PtPhase,
};

/// Modal dialog for interactive Non-Hermitian EP sensor and PT circuit analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionalPointDialog {
    /// Modal dialog visibility toggle.
    pub is_open: bool,

    // Controls
    /// Exceptional point degeneracy order (EP2, EP3, EP4).
    pub ep_order: EpOrder,
    /// Applied perturbation magnitude.
    pub perturbation_epsilon: f64,
    /// Applied perturbation phase angle in degrees.
    pub perturbation_phase_deg: f64,
    /// Balanced gain/loss rate gamma in MHz.
    pub gain_loss_gamma_mhz: f64,
    /// Inter-resonator coupling rate kappa in MHz.
    pub coupling_kappa_mhz: f64,
    /// Bare resonance frequency omega_0 in MHz.
    pub bare_frequency_w0_mhz: f64,

    // Circuit transient simulation controls
    /// Simulation duration in nanoseconds.
    pub sim_time_ns: f64,
    /// Number of integration time steps.
    pub num_sim_steps: usize,
    /// Initial voltage across gain tank v_1(0).
    pub initial_v1: f64,
    /// Initial voltage across loss tank v_2(0).
    pub initial_v2: f64,

    // Cached simulation results
    /// Transient circuit waveforms.
    pub circuit_state: Option<PtCircuitState>,
    /// Complex eigenvalue loop points in the (Re(lambda), Im(lambda)) plane.
    pub riemann_loop_points: Vec<[f64; 2]>,
    /// Bifurcation real branch 1 points [epsilon, Re(lambda_1)].
    pub bifurcation_branch_1: Vec<[f64; 2]>,
    /// Bifurcation real branch 2 points [epsilon, Re(lambda_2)].
    pub bifurcation_branch_2: Vec<[f64; 2]>,
    /// Fractional sensitivity curve for EP sensor [log10(eps), log10(Delta lambda)].
    pub sensitivity_curve_ep: Vec<[f64; 2]>,
    /// Linear sensitivity baseline for Hermitian sensor [log10(eps), log10(Delta lambda)].
    pub sensitivity_curve_hermitian: Vec<[f64; 2]>,

    // Telemetry
    /// Current operational PT symmetry phase.
    pub pt_phase: PtPhase,
    /// Petermann excess noise factor K.
    pub petermann_factor: f64,
    /// Sensitivity enhancement factor over Hermitian linear baseline.
    pub sensitivity_enhancement: f64,
    /// Mode splitting Delta omega in MHz.
    pub eigenvalue_splitting_mhz: f64,
    /// Status message displayed in the dialog footer.
    pub status_msg: String,
}

impl Default for ExceptionalPointDialog {
    fn default() -> Self {
        let mut dialog = Self {
            is_open: false,
            ep_order: EpOrder::EP2,
            perturbation_epsilon: 0.005,
            perturbation_phase_deg: 0.0,
            gain_loss_gamma_mhz: 5.0,
            coupling_kappa_mhz: 5.0,
            bare_frequency_w0_mhz: 100.0,

            sim_time_ns: 200.0,
            num_sim_steps: 1000,
            initial_v1: 1.0,
            initial_v2: 0.0,

            circuit_state: None,
            riemann_loop_points: Vec::new(),
            bifurcation_branch_1: Vec::new(),
            bifurcation_branch_2: Vec::new(),
            sensitivity_curve_ep: Vec::new(),
            sensitivity_curve_hermitian: Vec::new(),

            pt_phase: PtPhase::ExceptionalPoint,
            petermann_factor: 100.0,
            sensitivity_enhancement: 141.4,
            eigenvalue_splitting_mhz: 0.707,
            status_msg: "Exceptional Point simulator initialized.".to_string(),
        };
        dialog.run_simulation();
        dialog
    }
}

impl ExceptionalPointDialog {
    /// Creates a new dialog instance with defaults and pre-computed simulation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Recomputes non-Hermitian eigensystem, Riemann surface, PT circuit, and sensitivity curves.
    pub fn run_simulation(&mut self) {
        let eps = Complex::from_polar(
            self.perturbation_epsilon.max(1e-12),
            self.perturbation_phase_deg.to_radians(),
        );

        let hamiltonian = NonHermitianHamiltonian::new_with_coupling(
            self.ep_order,
            Complex::from_real(self.bare_frequency_w0_mhz),
            self.coupling_kappa_mhz,
        );

        // 1. Solve EP metrics
        self.eigenvalue_splitting_mhz = hamiltonian.eigenvalue_splitting(eps);
        self.petermann_factor = hamiltonian.petermann_factor(eps);
        self.sensitivity_enhancement = hamiltonian.sensitivity_enhancement(eps);

        // 2. Compute Riemann surface trajectory loop in (Re(lambda), Im(lambda)) plane
        let traj = hamiltonian.riemann_surface_trajectory(
            self.perturbation_epsilon.max(1e-4),
            40,
            self.ep_order.order(),
        );
        let mut loop_pts = Vec::with_capacity(traj.len() * self.ep_order.order());
        for pt in traj {
            for ev in pt.eigenvalues {
                loop_pts.push([ev.re, ev.im]);
            }
        }
        self.riemann_loop_points = loop_pts;

        // 3. Compute bifurcation cusp curves: Re(lambda) vs real epsilon in [-0.05, 0.05]
        let mut b1 = Vec::with_capacity(81);
        let mut b2 = Vec::with_capacity(81);
        for step in 0..=80 {
            let eps_val = -0.05 + 0.10 * (step as f64) / 80.0;
            let evs = hamiltonian.solve_eigenvalues(Complex::from_real(eps_val));
            if evs.len() >= 2 {
                b1.push([eps_val, evs[0].re]);
                b2.push([eps_val, evs[1].re]);
            }
        }
        self.bifurcation_branch_1 = b1;
        self.bifurcation_branch_2 = b2;

        // 4. Compute sensitivity enhancement log-log curves over epsilon in [1e-6, 1e-1]
        let mut ep_curve = Vec::with_capacity(51);
        let mut herm_curve = Vec::with_capacity(51);
        for step in 0..=50 {
            let log_eps = -6.0 + 5.0 * (step as f64) / 50.0;
            let eps_mag = 10.0f64.powf(log_eps);
            let ep_split = hamiltonian.eigenvalue_splitting(Complex::from_real(eps_mag));
            ep_curve.push([log_eps, ep_split.max(1e-15).log10()]);
            herm_curve.push([log_eps, eps_mag.log10()]);
        }
        self.sensitivity_curve_ep = ep_curve;
        self.sensitivity_curve_hermitian = herm_curve;

        // 5. Transient coupled RLC circuit simulation
        let rad_factor = 2.0 * std::f64::consts::PI * 1e6;
        let w0_rad = self.bare_frequency_w0_mhz * rad_factor;
        let gamma_rad = self.gain_loss_gamma_mhz * rad_factor;
        let kappa_rad = self.coupling_kappa_mhz * rad_factor;

        let circuit = PtCircuitParams::with_rates(w0_rad, gamma_rad, kappa_rad);
        self.pt_phase = circuit.pt_phase();

        let state = circuit.simulate_transient(
            self.initial_v1,
            self.initial_v2,
            self.sim_time_ns * 1e-9,
            self.num_sim_steps,
        );
        self.circuit_state = Some(state);

        self.status_msg = format!(
            "Converged: Order={}, Splitting={:.3} MHz, Enhancement={:.1}x, PT Phase={}",
            self.ep_order,
            self.eigenvalue_splitting_mhz,
            self.sensitivity_enhancement,
            self.pt_phase.label()
        );
    }

    /// Renders modal window.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Phonon Studio Exceptional Point & PT Circuit Simulator")
            .open(&mut is_open)
            .default_size([980.0, 680.0])
            .min_size([750.0, 520.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders dialog controls, telemetry panels, and visualizer plots.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut recompute = false;

        // 1. Top Controls Bar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 4.0);

            // EP Order Selector
            ui.label(RichText::new("EP Order:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
            let order_str = match self.ep_order {
                EpOrder::EP2 => "EP2 (Order 2)",
                EpOrder::EP3 => "EP3 (Order 3)",
                EpOrder::EP4 => "EP4 (Order 4)",
            };
            egui::ComboBox::from_id_salt("ep_order_selector_combo")
                .selected_text(order_str)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.ep_order, EpOrder::EP2, "EP2 (Order 2)").clicked() {
                        recompute = true;
                    }
                    if ui.selectable_value(&mut self.ep_order, EpOrder::EP3, "EP3 (Order 3)").clicked() {
                        recompute = true;
                    }
                    if ui.selectable_value(&mut self.ep_order, EpOrder::EP4, "EP4 (Order 4)").clicked() {
                        recompute = true;
                    }
                });

            ui.separator();

            // Perturbation epsilon
            ui.label(RichText::new("Perturbation epsilon:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
            if ui
                .add(
                    egui::DragValue::new(&mut self.perturbation_epsilon)
                        .range(1e-5..=0.20)
                        .speed(0.0005)
                        .max_decimals(5),
                )
                .changed()
            {
                recompute = true;
            }

            // Gain/loss rate gamma
            ui.label(RichText::new("Gain/Loss gamma:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
            if ui
                .add(
                    egui::DragValue::new(&mut self.gain_loss_gamma_mhz)
                        .range(0.0..=20.0)
                        .speed(0.1)
                        .suffix(" MHz"),
                )
                .changed()
            {
                recompute = true;
            }

            // Coupling rate kappa
            ui.label(RichText::new("Coupling kappa:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
            if ui
                .add(
                    egui::DragValue::new(&mut self.coupling_kappa_mhz)
                        .range(0.1..=20.0)
                        .speed(0.1)
                        .suffix(" MHz"),
                )
                .changed()
            {
                recompute = true;
            }

            // Simulate Circuit Button
            if ui.button(RichText::new("Simulate Circuit").strong()).clicked() {
                recompute = true;
            }
        });

        ui.separator();

        // 2. Real-Time Telemetry Status Strip
        self.render_telemetry_strip(ui);

        ui.separator();

        // 3. Main Visualizer Grids: Riemann Surface, Circuit Waveforms, and Sensitivity Gauge
        ui.columns(2, |columns| {
            // Left Column: Riemann Surface & Eigenvalue Sheets
            columns[0].vertical(|ui| {
                ui.heading(RichText::new("Riemann Surface Eigenvalue Sheets").size(13.0));
                self.render_riemann_surface_plot(ui);

                ui.add_space(8.0);
                ui.heading(RichText::new("Sub-Threshold Sensitivity Enhancement").size(13.0));
                self.render_sensitivity_plot(ui);
            });

            // Right Column: PT Circuit Waveforms & Sensitivity Gauge
            columns[1].vertical(|ui| {
                ui.heading(RichText::new("PT-Symmetric Coupled RLC Waveforms").size(13.0));
                self.render_circuit_waveforms_plot(ui);

                ui.add_space(8.0);
                ui.heading(RichText::new("Sensitivity Metric Comparison").size(13.0));
                self.render_sensitivity_gauge(ui);
            });
        });

        ui.separator();

        // Footer status line
        ui.label(
            RichText::new(&self.status_msg)
                .size(11.0)
                .color(Color32::from_rgb(140, 170, 200)),
        );

        if recompute {
            self.run_simulation();
        }
    }

    /// Renders telemetry metric badges.
    fn render_telemetry_strip(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(16.0, 0.0);

            // PT Phase Badge
            let (badge_text, badge_color) = match self.pt_phase {
                PtPhase::Exact => ("EXACT PHASE", Color32::from_rgb(34, 197, 94)),
                PtPhase::ExceptionalPoint => ("EXCEPTIONAL POINT", Color32::from_rgb(250, 204, 21)),
                PtPhase::Broken => ("BROKEN PHASE", Color32::from_rgb(239, 68, 68)),
            };
            ui.label(RichText::new("PT Status:").size(11.0).color(Color32::from_rgb(150, 160, 175)));
            ui.label(
                RichText::new(badge_text)
                    .size(11.0)
                    .strong()
                    .color(badge_color),
            );

            ui.separator();

            // Petermann Factor
            ui.label(RichText::new("Petermann Factor K:").size(11.0).color(Color32::from_rgb(150, 160, 175)));
            let petermann_str = if self.petermann_factor.is_infinite() || self.petermann_factor > 1e9 {
                "Divergent (Inf)".to_string()
            } else {
                format!("{:.1}", self.petermann_factor)
            };
            ui.label(
                RichText::new(petermann_str)
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(125, 211, 252)),
            );

            ui.separator();

            // Sensitivity Enhancement
            ui.label(RichText::new("Sensitivity Ratio:").size(11.0).color(Color32::from_rgb(150, 160, 175)));
            ui.label(
                RichText::new(format!("{:.1}x", self.sensitivity_enhancement))
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(74, 222, 128)),
            );

            ui.separator();

            // Splitting Delta omega
            ui.label(RichText::new("Mode Splitting:").size(11.0).color(Color32::from_rgb(150, 160, 175)));
            ui.label(
                RichText::new(format!("{:.3} MHz", self.eigenvalue_splitting_mhz))
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(244, 114, 182)),
            );
        });
    }

    /// Renders Riemann surface complex eigenvalue loop trajectory.
    fn render_riemann_surface_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("riemann_surface_complex_plot")
            .legend(Legend::default())
            .x_axis_label("Re(lambda) (MHz)")
            .y_axis_label("Im(lambda) (MHz)")
            .height(180.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            let pts: PlotPoints = PlotPoints::from(self.riemann_loop_points.clone());
            let line = Line::new("Eigenvalue Winding Trajectory", pts)
                .color(Color32::from_rgb(168, 85, 247))
                .width(2.0);
            plot_ui.line(line);

            let b1_pts: PlotPoints = PlotPoints::from(self.bifurcation_branch_1.clone());
            let b1_line = Line::new("Branch 1", b1_pts)
                .color(Color32::from_rgb(56, 189, 248))
                .width(1.5);
            plot_ui.line(b1_line);

            let b2_pts: PlotPoints = PlotPoints::from(self.bifurcation_branch_2.clone());
            let b2_line = Line::new("Branch 2", b2_pts)
                .color(Color32::from_rgb(249, 115, 22))
                .width(1.5);
            plot_ui.line(b2_line);
        });
    }

    /// Renders sensitivity enhancement comparison plot on log-log axes.
    fn render_sensitivity_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("subthreshold_sensitivity_plot")
            .legend(Legend::default())
            .x_axis_label("log10(Perturbation epsilon)")
            .y_axis_label("log10(Splitting Delta lambda)")
            .height(180.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            let ep_pts: PlotPoints = PlotPoints::from(self.sensitivity_curve_ep.clone());
            let ep_line = Line::new("EP Fractional Sensor", ep_pts)
                .color(Color32::from_rgb(34, 197, 94))
                .width(2.2);
            plot_ui.line(ep_line);

            let herm_pts: PlotPoints = PlotPoints::from(self.sensitivity_curve_hermitian.clone());
            let herm_line = Line::new("Hermitian Linear Baseline", herm_pts)
                .color(Color32::from_rgb(156, 163, 175))
                .width(1.5);
            plot_ui.line(herm_line);
        });
    }

    /// Renders coupled PT circuit transient waveforms v_1(t) and v_2(t).
    fn render_circuit_waveforms_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("pt_circuit_waveforms_plot")
            .legend(Legend::default())
            .x_axis_label("Time (ns)")
            .y_axis_label("Voltage (V)")
            .height(180.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            if let Some(state) = &self.circuit_state {
                let v1_pts: PlotPoints = state
                    .time
                    .iter()
                    .zip(state.v1.iter())
                    .map(|(&t, &v)| [t * 1e9, v])
                    .collect();
                let v1_line = Line::new("v1(t) [Gain Tank]", v1_pts)
                    .color(Color32::from_rgb(56, 189, 248))
                    .width(1.8);
                plot_ui.line(v1_line);

                let v2_pts: PlotPoints = state
                    .time
                    .iter()
                    .zip(state.v2.iter())
                    .map(|(&t, &v)| [t * 1e9, v])
                    .collect();
                let v2_line = Line::new("v2(t) [Loss Tank]", v2_pts)
                    .color(Color32::from_rgb(251, 146, 60))
                    .width(1.8);
                plot_ui.line(v2_line);

                let env_pts: PlotPoints = state
                    .time
                    .iter()
                    .zip(state.envelope.iter())
                    .map(|(&t, &env)| [t * 1e9, env])
                    .collect();
                let env_line = Line::new("Envelope", env_pts)
                    .color(Color32::from_rgb(250, 204, 21))
                    .width(1.2);
                plot_ui.line(env_line);
            }
        });
    }

    /// Renders sensitivity gauge comparing EP response against linear baseline.
    fn render_sensitivity_gauge(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Sub-Threshold Sensitivity Enhancement Gauge").size(11.0).color(Color32::from_rgb(180, 200, 220)));

            let ratio = self.sensitivity_enhancement;
            let norm_ratio = (ratio / 200.0).clamp(0.0, 1.0) as f32;

            ui.add(
                ProgressBar::new(norm_ratio)
                    .text(format!("{:.1}x Sensitivity Boost", ratio))
                    .animate(false),
            );

            ui.add_space(6.0);
            ui.label(
                RichText::new(format!(
                    "For perturbation epsilon = {:.5}, EP{} exhibits a {:.1}x sensitivity advantage over any linear Hermitian sensor.",
                    self.perturbation_epsilon,
                    self.ep_order.order(),
                    ratio
                ))
                .size(11.0)
                .color(Color32::from_rgb(160, 180, 200)),
            );
        });
    }
}
