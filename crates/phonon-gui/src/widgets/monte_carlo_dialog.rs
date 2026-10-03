#![deny(unsafe_code)]

//! Interactive Monte Carlo Yield & Histogram Inspector dialog for Phonon Visual Studio.
//!
//! Provides distributed parameter space exploration, Latin Hypercube Sampling (LHS),
//! statistical capability analysis (Cp/Cpk), process corner inspection, and distribution histograms.

use egui::{Color32, RichText, ScrollArea, Stroke, Ui, Vec2};
use egui_plot::{Bar, BarChart, Plot, VLine};
use phonon_core::{CircuitGraph, ComponentRecord};
use phonon_solver::mna::non_linear_solver::ModelContext;
use phonon_solver::monte_carlo::{
    HarvestReport, LhsParameter, MonteCarloHarvester, ParameterDistribution, SamplingMode,
    YieldSpecification,
};
use phonon_solver::sensitivity::ObjectiveKind;
use phonon_solver::transient::TransientOptions;

/// Selectable parameter entry in the dialog UI.
#[derive(Debug, Clone, PartialEq)]
pub struct MonteCarloParamEntry {
    /// Flag indicating whether this parameter is included in the sweep.
    pub is_enabled: bool,
    /// Parameter identifier and baseline properties.
    pub param: LhsParameter,
    /// Selected distribution type index (0: Uniform, 1: Gaussian, 2: LogNormal).
    pub dist_type: usize,
}

impl MonteCarloParamEntry {
    /// Returns an updated `LhsParameter` reflecting current distribution choice and tolerance.
    pub fn to_lhs_parameter(&self) -> LhsParameter {
        let name = &self.param.name;
        let nominal = self.param.nominal;
        let tol = self.param.tolerance_pct;
        let dist = match self.dist_type {
            0 => {
                let delta = (nominal * tol / 100.0).abs();
                ParameterDistribution::Uniform(nominal - delta, nominal + delta)
            }
            1 => {
                let sigma = (nominal * tol / 100.0).abs() / 3.0;
                ParameterDistribution::Gaussian(nominal, sigma)
            }
            2 => {
                let mu = if nominal > 0.0 { nominal.ln() } else { 0.0 };
                let sigma = (tol / 100.0) / 3.0;
                ParameterDistribution::LogNormal(mu, sigma)
            }
            _ => self.param.distribution,
        };

        LhsParameter::new(name.clone(), nominal, dist, tol)
            .with_clamping(self.param.min_clamp.unwrap_or(0.0), self.param.max_clamp.unwrap_or(f64::INFINITY))
    }
}

/// Interactive modal dialog for Monte Carlo and Latin Hypercube yield harvesting.
#[derive(Debug, Clone, PartialEq)]
pub struct MonteCarloYieldDialog {
    /// Visibility flag of the dialog window.
    pub is_open: bool,
    /// Configured sample count (e.g. 100, 1,000, 10,000).
    pub sample_count: usize,
    /// Selected sampling mode (Latin Hypercube vs standard Monte Carlo).
    pub sampling_mode: SamplingMode,
    /// Candidate component parameters extracted from the schematic.
    pub parameters: Vec<MonteCarloParamEntry>,
    /// Selected performance objective metric index.
    pub selected_objective_idx: usize,
    /// Target circuit node index for terminal voltage or overshoot.
    pub target_node: usize,
    /// Target voltage for integral squared error tracking.
    pub target_voltage: f64,
    /// Voltage threshold for propagation delay computation.
    pub delay_threshold: f64,
    /// Performance metric identifier name.
    pub metric_name: String,
    /// Flag enabling the lower specification limit (LSL).
    pub enable_lsl: bool,
    /// Numerical value of the LSL.
    pub lsl_value: f64,
    /// Flag enabling the upper specification limit (USL).
    pub enable_usl: bool,
    /// Numerical value of the USL.
    pub usl_value: f64,
    /// Harvested sweep report containing statistics and histograms.
    pub report: Option<HarvestReport>,
    /// Current execution status message.
    pub status_msg: String,
    /// Flag signaling to the parent application that a sweep should run.
    pub run_requested: bool,
    /// Execution time of the last sweep in milliseconds.
    pub last_elapsed_ms: f64,
}

impl Default for MonteCarloYieldDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            sample_count: 1000,
            sampling_mode: SamplingMode::LatinHypercube,
            parameters: Vec::new(),
            selected_objective_idx: 0,
            target_node: 1,
            target_voltage: 2.5,
            delay_threshold: 2.5,
            metric_name: "Terminal Voltage V(OUT)".to_string(),
            enable_lsl: true,
            lsl_value: 2.0,
            enable_usl: true,
            usl_value: 3.0,
            report: None,
            status_msg: "Ready. Configure sampling parameters and click Run Monte Carlo Sweep.".to_string(),
            run_requested: false,
            last_elapsed_ms: 0.0,
        }
    }
}

impl MonteCarloYieldDialog {
    /// Constructs a new Monte Carlo dialog in closed state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the currently active objective kind.
    pub fn current_objective(&self) -> ObjectiveKind {
        match self.selected_objective_idx {
            0 => ObjectiveKind::TerminalVoltage {
                node: self.target_node,
            },
            1 => ObjectiveKind::IntegralSquaredError {
                node: self.target_node,
                target_v: self.target_voltage,
            },
            2 => ObjectiveKind::EnergyDissipated,
            3 => ObjectiveKind::PeakOvershoot {
                node: self.target_node,
            },
            4 => ObjectiveKind::DelayToThreshold {
                node: self.target_node,
                threshold: self.delay_threshold,
            },
            _ => ObjectiveKind::TerminalVoltage {
                node: self.target_node,
            },
        }
    }

    /// Constructs the current yield specification from dialog controls.
    pub fn current_spec(&self) -> YieldSpecification {
        YieldSpecification::new(
            &self.metric_name,
            if self.enable_lsl {
                Some(self.lsl_value)
            } else {
                None
            },
            if self.enable_usl {
                Some(self.usl_value)
            } else {
                None
            },
        )
    }

    /// Returns the list of enabled parameters configured for sampling.
    pub fn active_parameters(&self) -> Vec<LhsParameter> {
        self.parameters
            .iter()
            .filter(|e| e.is_enabled)
            .map(|e| e.to_lhs_parameter())
            .collect()
    }

    /// Extracts tunable passive and active component parameters from a circuit graph.
    pub fn extract_parameters_from_graph(&mut self, graph: &CircuitGraph) {
        self.parameters.clear();
        for comp in graph.components() {
            match comp {
                ComponentRecord::Resistor {
                    name, resistance, ..
                } => {
                    self.parameters.push(MonteCarloParamEntry {
                        is_enabled: true,
                        param: LhsParameter::uniform(name.clone(), *resistance, 5.0),
                        dist_type: 0,
                    });
                }
                ComponentRecord::Capacitor {
                    name, capacitance, ..
                } => {
                    self.parameters.push(MonteCarloParamEntry {
                        is_enabled: true,
                        param: LhsParameter::gaussian(name.clone(), *capacitance, 10.0),
                        dist_type: 1,
                    });
                }
                ComponentRecord::Inductor {
                    name, inductance, ..
                } => {
                    self.parameters.push(MonteCarloParamEntry {
                        is_enabled: true,
                        param: LhsParameter::uniform(name.clone(), *inductance, 5.0),
                        dist_type: 0,
                    });
                }
                ComponentRecord::Mosfet { name, .. } => {
                    self.parameters.push(MonteCarloParamEntry {
                        is_enabled: true,
                        param: LhsParameter::gaussian(format!("{}_W", name), 1.0e-6, 10.0),
                        dist_type: 1,
                    });
                }
                _ => {}
            }
        }
    }

    /// Executes the multi-threaded Monte Carlo / LHS sweep across circuit simulation models.
    pub fn run_sweep(
        &mut self,
        graph: &CircuitGraph,
        context: &ModelContext,
        options: &TransientOptions,
    ) {
        if self.parameters.is_empty() {
            self.extract_parameters_from_graph(graph);
        }
        let active_params = self.active_parameters();
        if active_params.is_empty() {
            self.status_msg = "No enabled parameters selected for Monte Carlo sweep.".to_string();
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        let start = std::time::Instant::now();

        let harvester = MonteCarloHarvester::new(active_params)
            .with_sampling_mode(self.sampling_mode)
            .with_bins(25);

        let spec = self.current_spec();
        let objective = self.current_objective();

        match harvester.harvest_circuit(
            graph,
            context,
            options,
            &objective,
            &spec,
            self.sample_count,
        ) {
            Ok(rep) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.last_elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                }
                #[cfg(target_arch = "wasm32")]
                {
                    self.last_elapsed_ms = 0.0;
                }
                self.status_msg = format!(
                    "Monte Carlo sweep completed: {} samples in {:.1} ms. Yield: {:.2}% ({}/{} passed)",
                    rep.metrics.sample_count,
                    self.last_elapsed_ms,
                    rep.metrics.yield_percentage,
                    rep.passed_count,
                    rep.metrics.sample_count,
                );
                self.report = Some(rep);
            }
            Err(e) => {
                self.status_msg = format!("Monte Carlo sweep failed: {}", e);
            }
        }
    }

    /// Executes sweep using a custom evaluation closure.
    pub fn run_sweep_custom<F>(&mut self, eval_fn: F)
    where
        F: Fn(&[f64]) -> f64 + Sync + Send,
    {
        let active_params = self.active_parameters();
        if active_params.is_empty() {
            self.status_msg = "No enabled parameters selected for Monte Carlo sweep.".to_string();
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        let start = std::time::Instant::now();

        let harvester = MonteCarloHarvester::new(active_params)
            .with_sampling_mode(self.sampling_mode)
            .with_bins(25);

        let spec = self.current_spec();
        let rep = harvester.harvest_custom(&spec, self.sample_count, eval_fn);

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.last_elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.last_elapsed_ms = 0.0;
        }
        self.status_msg = format!(
            "Monte Carlo sweep completed: {} samples in {:.1} ms. Yield: {:.2}% ({}/{} passed)",
            rep.metrics.sample_count,
            self.last_elapsed_ms,
            rep.metrics.yield_percentage,
            rep.passed_count,
            rep.metrics.sample_count,
        );
        self.report = Some(rep);
    }

    /// Renders the Monte Carlo dialog window and interactive charts.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Monte Carlo Yield Harvester & Latin Hypercube Inspector")
            .open(&mut is_open)
            .resizable(true)
            .default_size(Vec2::new(760.0, 620.0))
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the inner content of the dialog.
    pub fn render_content(&mut self, ui: &mut Ui) {
        // Status header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&self.status_msg)
                    .color(Color32::from_rgb(180, 210, 240))
                    .size(12.0),
            );
        });
        ui.separator();

        ScrollArea::vertical().show(ui, |ui| {
            // 1. Sweep Configuration Bar
            ui.heading("Sweep Engine Configuration");
            ui.horizontal(|ui| {
                ui.label("Sample Count:");
                egui::ComboBox::from_id_salt("mc_sample_count_combo")
                    .selected_text(format!("{}", self.sample_count))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.sample_count, 100, "100 samples");
                        ui.selectable_value(&mut self.sample_count, 500, "500 samples");
                        ui.selectable_value(&mut self.sample_count, 1000, "1,000 samples");
                        ui.selectable_value(&mut self.sample_count, 5000, "5,000 samples");
                        ui.selectable_value(&mut self.sample_count, 10000, "10,000 samples");
                    });

                ui.separator();
                ui.label("Sampling Mode:");
                ui.radio_value(
                    &mut self.sampling_mode,
                    SamplingMode::LatinHypercube,
                    "Latin Hypercube (LHS)",
                );
                ui.radio_value(
                    &mut self.sampling_mode,
                    SamplingMode::MonteCarlo,
                    "Monte Carlo (MC)",
                );

                ui.separator();
                if ui
                    .button(
                        RichText::new("Run Monte Carlo Sweep")
                            .color(Color32::from_rgb(80, 220, 140))
                            .strong(),
                    )
                    .clicked()
                {
                    self.run_requested = true;
                }
            });
            ui.separator();

            // 2. Specification Limits (LSL / USL) and Objective
            ui.heading("Performance Objective & Specification Limits");
            ui.horizontal(|ui| {
                ui.label("Metric Name:");
                ui.text_edit_singleline(&mut self.metric_name);

                ui.separator();
                ui.label("Objective Functional:");
                egui::ComboBox::from_id_salt("mc_objective_selector")
                    .selected_text(match self.selected_objective_idx {
                        0 => "Terminal Voltage V(T)",
                        1 => "Integral Squared Error (ISE)",
                        2 => "Total Energy Dissipated",
                        3 => "Peak Overshoot Max V",
                        4 => "Delay to Threshold",
                        _ => "Terminal Voltage V(T)",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_objective_idx, 0, "Terminal Voltage V(T)");
                        ui.selectable_value(&mut self.selected_objective_idx, 1, "Integral Squared Error (ISE)");
                        ui.selectable_value(&mut self.selected_objective_idx, 2, "Total Energy Dissipated");
                        ui.selectable_value(&mut self.selected_objective_idx, 3, "Peak Overshoot Max V");
                        ui.selectable_value(&mut self.selected_objective_idx, 4, "Delay to Threshold");
                    });
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.enable_lsl, "Lower Spec Limit (LSL):");
                ui.add_enabled(
                    self.enable_lsl,
                    egui::DragValue::new(&mut self.lsl_value).speed(0.01),
                );

                ui.separator();
                ui.checkbox(&mut self.enable_usl, "Upper Spec Limit (USL):");
                ui.add_enabled(
                    self.enable_usl,
                    egui::DragValue::new(&mut self.usl_value).speed(0.01),
                );

                if self.selected_objective_idx == 0 || self.selected_objective_idx == 3 {
                    ui.separator();
                    ui.label("Target Node:");
                    ui.add(egui::DragValue::new(&mut self.target_node).speed(1));
                }
            });
            ui.separator();

            // 3. Component Parameter Selection Table
            ui.heading(format!(
                "Tunable Component Parameters ({} Active)",
                self.active_parameters().len()
            ));
            if self.parameters.is_empty() {
                ui.label(
                    RichText::new("No component parameters loaded. Click Run Sweep to auto-extract from schematic.")
                        .color(Color32::from_rgb(180, 180, 180))
                        .italics(),
                );
            } else {
                ScrollArea::vertical()
                    .max_height(140.0)
                    .show(ui, |ui| {
                        egui::Grid::new("mc_param_table")
                            .striped(true)
                            .min_col_width(90.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new("Include").strong());
                                ui.label(RichText::new("Component").strong());
                                ui.label(RichText::new("Nominal").strong());
                                ui.label(RichText::new("Tolerance %").strong());
                                ui.label(RichText::new("Distribution").strong());
                                ui.end_row();

                                for entry in &mut self.parameters {
                                    ui.checkbox(&mut entry.is_enabled, "");
                                    ui.label(&entry.param.name);
                                    ui.label(format!("{:.4e}", entry.param.nominal));
                                    ui.add(
                                        egui::DragValue::new(&mut entry.param.tolerance_pct)
                                            .speed(0.1)
                                            .range(0.1..=100.0),
                                    );

                                    egui::ComboBox::from_id_salt(format!("dist_{}", entry.param.name))
                                        .selected_text(match entry.dist_type {
                                            0 => "Uniform",
                                            1 => "Gaussian",
                                            2 => "LogNormal",
                                            _ => "Uniform",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut entry.dist_type, 0, "Uniform");
                                            ui.selectable_value(&mut entry.dist_type, 1, "Gaussian");
                                            ui.selectable_value(&mut entry.dist_type, 2, "LogNormal");
                                        });

                                    ui.end_row();
                                }
                            });
                    });
            }
            ui.separator();

            // 4. Statistical Summary View & Capability Metrics
            if let Some(rep) = &self.report {
                let m = &rep.metrics;
                ui.heading("Statistical Moments & Process Capability Summary");

                let yield_color = if m.yield_percentage >= 95.0 {
                    Color32::from_rgb(60, 220, 100) // High Yield Green
                } else if m.yield_percentage >= 80.0 {
                    Color32::from_rgb(240, 180, 40) // Warning Amber
                } else {
                    Color32::from_rgb(240, 60, 60) // Critical Red
                };

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("Yield: {:.2}%", m.yield_percentage))
                            .color(yield_color)
                            .size(18.0)
                            .strong(),
                    );
                    ui.separator();
                    ui.label(format!("Samples: {}", m.sample_count));
                    ui.label(format!("Passed: {}", rep.passed_count));
                    ui.label(format!("Failed: {}", rep.failed_count));
                });

                egui::Grid::new("mc_stats_summary_grid")
                    .striped(true)
                    .min_col_width(120.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Mean (mu):").strong());
                        ui.label(format!("{:.4e}", m.mean));
                        ui.label(RichText::new("Std Dev (sigma):").strong());
                        ui.label(format!("{:.4e}", m.std_dev));
                        ui.label(RichText::new("Variance (sigma^2):").strong());
                        ui.label(format!("{:.4e}", m.variance));
                        ui.end_row();

                        ui.label(RichText::new("Min:").strong());
                        ui.label(format!("{:.4e}", m.min));
                        ui.label(RichText::new("Median:").strong());
                        ui.label(format!("{:.4e}", m.median));
                        ui.label(RichText::new("Max:").strong());
                        ui.label(format!("{:.4e}", m.max));
                        ui.end_row();

                        ui.label(RichText::new("Skewness:").strong());
                        ui.label(format!("{:.4}", m.skewness));
                        ui.label(RichText::new("Kurtosis:").strong());
                        ui.label(format!("{:.4}", m.kurtosis));
                        ui.label(RichText::new("Capability Cp:").strong());
                        ui.label(match m.cp {
                            Some(val) => format!("{:.3}", val),
                            None => "N/A".to_string(),
                        });
                        ui.end_row();

                        ui.label(RichText::new("Capability Cpk:").strong());
                        ui.label(match m.cpk {
                            Some(val) => format!("{:.3}", val),
                            None => "N/A".to_string(),
                        });
                        ui.label(RichText::new("-3sigma Bound:").strong());
                        ui.label(format!("{:.4e}", rep.corners.minus_3sigma));
                        ui.label(RichText::new("+3sigma Bound:").strong());
                        ui.label(format!("{:.4e}", rep.corners.plus_3sigma));
                        ui.end_row();
                    });

                ui.separator();

                // 5. Visual Histogram Plot / Distribution Bars with Threshold Markers
                ui.heading("Yield Histogram & Specification Limit Markers");
                let spec = &rep.spec;

                let bars: Vec<Bar> = rep
                    .histogram
                    .iter()
                    .map(|b| {
                        let center = 0.5 * (b.bin_min + b.bin_max);
                        let width = (b.bin_max - b.bin_min).abs();
                        let in_spec = spec.meets_spec(center);
                        let color = if in_spec {
                            Color32::from_rgb(60, 160, 240) // Cyan
                        } else {
                            Color32::from_rgb(230, 60, 60) // Red
                        };
                        Bar::new(center, b.count as f64)
                            .width(width * 0.95)
                            .fill(color)
                    })
                    .collect();

                let bar_chart = BarChart::new("Yield Distribution", bars)
                    .color(Color32::from_rgb(60, 160, 240));

                Plot::new("mc_yield_histogram_plot")
                    .height(200.0)
                    .allow_zoom(true)
                    .allow_drag(true)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(bar_chart);

                        // LSL Threshold Marker
                        if let Some(lsl) = spec.lsl {
                            plot_ui.vline(
                                VLine::new("LSL", lsl)
                                    .color(Color32::from_rgb(240, 80, 80))
                                    .stroke(Stroke::new(2.0, Color32::from_rgb(240, 80, 80))),
                            );
                        }

                        // USL Threshold Marker
                        if let Some(usl) = spec.usl {
                            plot_ui.vline(
                                VLine::new("USL", usl)
                                    .color(Color32::from_rgb(240, 80, 80))
                                    .stroke(Stroke::new(2.0, Color32::from_rgb(240, 80, 80))),
                            );
                        }

                        // Mean Marker
                        plot_ui.vline(
                            VLine::new("Mean", m.mean)
                                .color(Color32::from_rgb(80, 220, 120))
                                .stroke(Stroke::new(1.5, Color32::from_rgb(80, 220, 120))),
                        );
                    });
            }
        });
    }
}
