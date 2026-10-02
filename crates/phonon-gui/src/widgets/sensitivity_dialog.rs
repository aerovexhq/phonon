#![deny(unsafe_code)]

//! Component Sensitivity Visualizer, Adjoint Sensitivity Ranking, and Worst-Case Corner dialog for Phonon Visual Studio.

use egui::{Color32, RichText, ScrollArea, Ui, Vec2};
use phonon_core::{CircuitGraph, ComponentRecord};
use phonon_solver::mna::non_linear_solver::ModelContext;
use phonon_solver::sensitivity::{
    AdjointSensitivityEngine, CircuitParameter, CornerType, ObjectiveKind,
    SensitivityResult, WorstCaseOptimizer, WorstCaseSummary,
};
use phonon_solver::transient::TransientOptions;

/// Interactive Transient Sensitivity Analysis and Worst-Case Optimization modal dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct SensitivityDialog {
    pub is_open: bool,
    pub selected_objective_idx: usize,
    pub target_node: usize,
    pub target_voltage: f64,
    pub delay_threshold: f64,
    pub parameters: Vec<CircuitParameter>,
    pub sensitivity_results: Vec<SensitivityResult>,
    pub worst_case_summary: Option<WorstCaseSummary>,
    pub status_msg: String,
    pub show_badges_on_canvas: bool,
    pub run_requested: bool,
}

impl Default for SensitivityDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_objective_idx: 0,
            target_node: 1,
            target_voltage: 2.5,
            delay_threshold: 2.5,
            parameters: Vec::new(),
            sensitivity_results: Vec::new(),
            worst_case_summary: None,
            status_msg: "Ready. Select performance objective functional and run sensitivity analysis.".to_string(),
            show_badges_on_canvas: true,
            run_requested: false,
        }
    }
}

impl SensitivityDialog {
    /// Constructs a new SensitivityDialog in closed state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the currently configured objective kind.
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

    /// Returns the visual badge highlight color based on normalized sensitivity magnitude:
    /// Crimson for >20%, Amber for 5-20%, Cyan for <5%.
    pub fn badge_color(norm_sensitivity: f64) -> Color32 {
        let abs_pct = (norm_sensitivity * 100.0).abs();
        if abs_pct > 20.0 {
            Color32::from_rgb(220, 50, 50) // Crimson
        } else if abs_pct >= 5.0 {
            Color32::from_rgb(230, 160, 20) // Amber
        } else {
            Color32::from_rgb(50, 190, 220) // Cyan
        }
    }

    /// Looks up component normalized sensitivity and associated badge color.
    pub fn get_badge_impact(&self, component_name: &str) -> Option<(f64, Color32)> {
        for res in &self.sensitivity_results {
            if res.param_id == component_name {
                return Some((
                    res.normalized_sensitivity,
                    Self::badge_color(res.normalized_sensitivity),
                ));
            }
        }
        None
    }

    /// Returns sensitivity results sorted in descending order of absolute normalized sensitivity.
    pub fn sorted_results(&self) -> Vec<&SensitivityResult> {
        let mut sorted: Vec<&SensitivityResult> = self.sensitivity_results.iter().collect();
        sorted.sort_by(|a, b| {
            b.normalized_sensitivity
                .abs()
                .partial_cmp(&a.normalized_sensitivity.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// Extracts tunable passive and active component parameters from a circuit graph.
    pub fn extract_parameters_from_graph(&mut self, graph: &CircuitGraph) {
        self.parameters.clear();
        for comp in graph.components() {
            match comp {
                ComponentRecord::Resistor {
                    name, resistance, ..
                } => {
                    self.parameters.push(CircuitParameter::Resistance {
                        id: name.clone(),
                        nominal: *resistance,
                        tolerance: 0.05,
                    });
                }
                ComponentRecord::Capacitor {
                    name, capacitance, ..
                } => {
                    self.parameters.push(CircuitParameter::Capacitance {
                        id: name.clone(),
                        nominal: *capacitance,
                        tolerance: 0.10,
                    });
                }
                ComponentRecord::Inductor {
                    name, inductance, ..
                } => {
                    self.parameters.push(CircuitParameter::Inductance {
                        id: name.clone(),
                        nominal: *inductance,
                        tolerance: 0.05,
                    });
                }
                ComponentRecord::Mosfet { name, .. } => {
                    self.parameters.push(CircuitParameter::MosfetWidth {
                        id: name.clone(),
                        nominal: 1.0e-6,
                        tolerance: 0.10,
                    });
                }
                _ => {}
            }
        }
    }

    /// Executes the backward adjoint sensitivity analysis and worst-case corner search.
    pub fn run_analysis(
        &mut self,
        graph: &CircuitGraph,
        context: &ModelContext,
        options: &TransientOptions,
    ) {
        if self.parameters.is_empty() {
            self.extract_parameters_from_graph(graph);
        }
        if self.parameters.is_empty() {
            self.status_msg = "No tunable parameters found in schematic circuit.".to_string();
            return;
        }

        let engine = AdjointSensitivityEngine::new(
            graph.clone(),
            context.clone(),
            options.clone(),
            self.current_objective(),
            self.parameters.clone(),
        );

        match engine.solve() {
            Ok(results) => {
                let optimizer = WorstCaseOptimizer::new(engine);
                self.worst_case_summary = optimizer.find_worst_case_corners().ok();
                self.sensitivity_results = results;
                self.status_msg = format!(
                    "Adjoint analysis completed: {} component parameters evaluated and ranked.",
                    self.sensitivity_results.len()
                );
            }
            Err(e) => {
                self.status_msg = format!("Adjoint sensitivity analysis failed: {}", e);
            }
        }
    }

    /// Renders the sensitivity dialog window and interactive charts.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Non-Linear Transient Sensitivity Analysis & Worst-Case Optimization")
            .open(&mut is_open)
            .resizable(true)
            .default_size(Vec2::new(720.0, 560.0))
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
                    .color(Color32::from_rgb(180, 200, 230))
                    .size(12.0),
            );
        });
        ui.separator();

        // 1. Objective Functional Selector
        ui.heading("Performance Objective Functional J(x, p)");
        ui.horizontal(|ui| {
            ui.label("Objective Metric:");
            egui::ComboBox::from_id_salt("objective_kind_selector")
                .selected_text(match self.selected_objective_idx {
                    0 => "Terminal Voltage V(T)",
                    1 => "Integral Squared Error (ISE)",
                    2 => "Total Energy Dissipated",
                    3 => "Peak Overshoot Max V",
                    4 => "Delay to Threshold",
                    _ => "Unknown",
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
            if self.selected_objective_idx != 2 {
                ui.label("Target Node ID:");
                ui.add(egui::DragValue::new(&mut self.target_node).range(1..=32));
            }
            if self.selected_objective_idx == 1 {
                ui.label("Target V:");
                ui.add(egui::DragValue::new(&mut self.target_voltage).speed(0.1));
            }
            if self.selected_objective_idx == 4 {
                ui.label("Threshold V:");
                ui.add(egui::DragValue::new(&mut self.delay_threshold).speed(0.1));
            }
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_badges_on_canvas, "Display Impact Badges on Canvas Pins");
            if ui.button("Run Sensitivity Analysis").clicked() {
                self.run_requested = true;
            }
        });
        ui.separator();

        // 2. Sensitivity Ranking Bar Chart
        ui.heading("Parametric Sensitivity Ranking (Normalized % Sensitivity: (p0/J0) * dJ/dp)");

        let sorted = self.sorted_results();
        if sorted.is_empty() {
            ui.label("No sensitivity data. Run analysis to rank components.");
        } else {
            ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                for res in sorted {
                    let norm_pct = res.normalized_sensitivity * 100.0;
                    let color = Self::badge_color(res.normalized_sensitivity);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&res.param_id)
                                .strong()
                                .color(Color32::from_rgb(220, 230, 245))
                                .size(12.0),
                        );
                        ui.label(format!("Nominal: {:.4e}", res.nominal_value));
                        ui.label(format!("Grad: {:.4e}", res.gradient));
                        ui.label(
                            RichText::new(format!("{:+.2}%", norm_pct))
                                .color(color)
                                .strong(),
                        );

                        // Visual progress bar representing sensitivity impact
                        let bar_width = (norm_pct.abs() * 3.0).clamp(6.0, 180.0) as f32;
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, 12.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, color);
                    });
                }
            });
        }
        ui.separator();

        // 3. Worst-Case Optimization Summary Table
        ui.heading("Worst-Case Parameter Corners & PVT Bounds");
        if let Some(summary) = &self.worst_case_summary {
            ui.horizontal(|ui| {
                ui.label(format!("Nominal Objective J0: {:.6e}", summary.nominal.objective_value));
                ui.separator();
                ui.label(
                    RichText::new(format!("Max Degradation: {:.2}%", summary.max_degradation_percent))
                        .color(if summary.max_degradation_percent > 20.0 {
                            Color32::from_rgb(220, 50, 50)
                        } else {
                            Color32::from_rgb(230, 160, 20)
                        })
                        .strong(),
                );
            });

            ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                egui::Grid::new("worst_case_grid").striped(true).show(ui, |ui| {
                    ui.label(RichText::new("Corner").strong());
                    ui.label(RichText::new("Objective J").strong());
                    ui.label(RichText::new("Deviation %").strong());
                    ui.end_row();

                    // Nominal
                    ui.label("Nominal");
                    ui.label(format!("{:.6e}", summary.nominal.objective_value));
                    ui.label("0.00%");
                    ui.end_row();

                    // Worst-Case Max
                    ui.label("Worst-Case Max");
                    ui.label(format!("{:.6e}", summary.worst_max.objective_value));
                    ui.label(format!("{:+.2}%", summary.worst_max.deviation_percent));
                    ui.end_row();

                    // Worst-Case Min
                    ui.label("Worst-Case Min");
                    ui.label(format!("{:.6e}", summary.worst_min.objective_value));
                    ui.label(format!("{:+.2}%", summary.worst_min.deviation_percent));
                    ui.end_row();

                    // PVT Corners
                    for c in &summary.corners {
                        let name = match &c.corner {
                            CornerType::PvtExtreme { corner_name } => corner_name.as_str(),
                            _ => "Corner",
                        };
                        ui.label(name);
                        ui.label(format!("{:.6e}", c.objective_value));
                        ui.label(format!("{:+.2}%", c.deviation_percent));
                        ui.end_row();
                    }
                });
            });
        } else {
            ui.label("Run sensitivity analysis to compute worst-case parameter corners.");
        }
    }
}
