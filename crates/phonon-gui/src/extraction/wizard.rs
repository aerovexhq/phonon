#![deny(unsafe_code)]

//! Visual Studio SPICE model parameter extraction wizard dialog and interactive fitting interface.

use egui::{Color32, RichText, ScrollArea, Ui, Vec2};
use phonon_models::extraction::{
    generate_bsim4_model_deck, FittingResult, GaOptimizer, MeasuredCurve,
};

/// Interactive SPICE model parameter extraction wizard dialog for Phonon Visual Studio.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractionWizardDialog {
    pub is_open: bool,
    pub current_curve: Option<MeasuredCurve>,
    pub fitting_result: Option<FittingResult>,
    pub generated_deck: Option<String>,
    pub model_name: String,
    pub is_nmos: bool,
    pub raw_csv_input: String,
    pub status_msg: String,
    pub max_generations: usize,
    pub population_size: usize,
}

impl Default for ExtractionWizardDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            current_curve: None,
            fitting_result: None,
            generated_deck: None,
            model_name: "NMOS_BSIM4".to_string(),
            is_nmos: true,
            raw_csv_input: String::new(),
            status_msg: "Ready. Load a sample curve or paste CSV data.".to_string(),
            max_generations: 25,
            population_size: 48,
        }
    }
}

impl ExtractionWizardDialog {
    /// Constructs a new extraction wizard dialog in closed state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a standard 50-point synthetic NMOS transfer curve ($V_{ds} = 1.2\text{ V}$).
    pub fn load_sample_nmos(&mut self) {
        let curve =
            MeasuredCurve::synthetic_nmos_transfer_curve(1.2, (0.0, 1.5, 50), 0.45, 0.05);
        self.status_msg = format!(
            "Loaded sample NMOS transfer curve ({} points, Vds = 1.20 V)",
            curve.points.len()
        );
        self.current_curve = Some(curve);
        self.fitting_result = None;
        self.generated_deck = None;
    }

    /// Loads a standard 50-point synthetic NMOS output curve ($V_{gs} = 1.0\text{ V}$).
    pub fn load_sample_output(&mut self) {
        let curve =
            MeasuredCurve::synthetic_nmos_output_curve(1.0, (0.0, 1.8, 50), 0.45, 0.05);
        self.status_msg = format!(
            "Loaded sample NMOS output curve ({} points, Vgs = 1.00 V)",
            curve.points.len()
        );
        self.current_curve = Some(curve);
        self.fitting_result = None;
        self.generated_deck = None;
    }

    /// Loads a sample cryogenic 4.2 K transfer curve.
    pub fn load_sample_cryo(&mut self) {
        let mut curve =
            MeasuredCurve::synthetic_nmos_transfer_curve(1.0, (0.0, 1.5, 50), 0.55, 0.08);
        curve.temperature_k = 4.2;
        curve.name = "Cryo_CMOS_4.2K_Transfer".to_string();
        self.status_msg = format!(
            "Loaded cryogenic 4.2 K NMOS transfer curve ({} points)",
            curve.points.len()
        );
        self.current_curve = Some(curve);
        self.fitting_result = None;
        self.generated_deck = None;
    }

    /// Parses raw CSV text input and populates the active measurement curve.
    pub fn parse_csv_input(&mut self) {
        if self.raw_csv_input.trim().is_empty() {
            self.status_msg = "CSV input is empty. Paste data before importing.".to_string();
            return;
        }

        match MeasuredCurve::parse_csv(&self.raw_csv_input) {
            Ok(curve) => {
                self.status_msg = format!(
                    "Successfully imported {} measurement points from CSV.",
                    curve.points.len()
                );
                self.current_curve = Some(curve);
                self.fitting_result = None;
                self.generated_deck = None;
            }
            Err(e) => {
                self.status_msg = format!("Failed to parse CSV: {}", e);
            }
        }
    }

    /// Executes the multi-island genetic algorithm parameter extraction engine.
    pub fn run_fitting(&mut self) {
        if let Some(ref curve) = self.current_curve {
            let result =
                GaOptimizer::fit_curve(curve, self.max_generations, self.population_size);
            let deck = generate_bsim4_model_deck(
                &self.model_name,
                self.is_nmos,
                &result.params,
                &result,
            );

            self.status_msg = format!(
                "Fitted: RMSE = {:.3e} A, R^2 = {:.5} (Converged: {})",
                result.rmse, result.r_squared, result.converged
            );
            self.fitting_result = Some(result);
            self.generated_deck = Some(deck);
        } else {
            self.status_msg =
                "No measurement curve loaded. Load a sample or import CSV first.".to_string();
        }
    }

    /// Renders the extraction wizard modal dialog window within the egui context.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("SPICE Model Parameter Extraction Wizard")
            .open(&mut is_open)
            .default_size(Vec2::new(720.0, 560.0))
            .min_width(580.0)
            .min_height(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders inner dialog contents.
    fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Compact SPICE Model Parameter Extraction");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    self.is_open = false;
                }
            });
        });

        ui.separator();

        // Status banner
        ui.horizontal(|ui| {
            ui.label(RichText::new("Status:").strong());
            ui.label(RichText::new(&self.status_msg).color(Color32::from_rgb(180, 220, 255)));
        });

        ui.add_space(4.0);

        // Model Configuration Controls
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Model Name:");
                ui.text_edit_singleline(&mut self.model_name);

                ui.separator();
                ui.label("Polarity:");
                if ui.selectable_label(self.is_nmos, "NMOS").clicked() {
                    self.is_nmos = true;
                }
                if ui.selectable_label(!self.is_nmos, "PMOS").clicked() {
                    self.is_nmos = false;
                }

                ui.separator();
                ui.label("GA Generations:");
                ui.add(egui::DragValue::new(&mut self.max_generations).range(5..=100));

                ui.label("Pop Size:");
                ui.add(egui::DragValue::new(&mut self.population_size).range(12..=128));
            });
        });

        ui.add_space(4.0);

        // Data Ingestion Section
        ui.horizontal(|ui| {
            if ui.button("Load NMOS Transfer Curve").clicked() {
                self.load_sample_nmos();
            }
            if ui.button("Load NMOS Output Curve").clicked() {
                self.load_sample_output();
            }
            if ui.button("Load 4.2K Cryo Curve").clicked() {
                self.load_sample_cryo();
            }

            ui.separator();

            let has_curve = self.current_curve.is_some();
            if ui
                .add_enabled(
                    has_curve,
                    egui::Button::new(
                        RichText::new("Run Genetic Fitting")
                            .color(Color32::from_rgb(255, 255, 255))
                            .strong(),
                    )
                    .fill(Color32::from_rgb(40, 110, 180)),
                )
                .clicked()
            {
                self.run_fitting();
            }
        });

        ui.add_space(4.0);

        // Curve Telemetry Inspection
        if let Some(ref curve) = self.current_curve {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Active Curve: {}", curve.name)).strong());
                    ui.separator();
                    ui.label(format!("Points: {}", curve.points.len()));
                    ui.separator();
                    ui.label(format!("Temp: {:.1} K", curve.temperature_k));
                    ui.separator();
                    ui.label(format!("W: {:.2} um", curve.channel_width_m * 1e6));
                    ui.separator();
                    ui.label(format!("L: {:.0} nm", curve.channel_length_m * 1e9));
                });
            });
        }

        ui.add_space(4.0);

        // Results and Generated Deck View
        ScrollArea::vertical().show(ui, |ui| {
            if let Some(ref result) = self.fitting_result {
                ui.group(|ui| {
                    ui.label(
                        RichText::new("Extracted BSIM4 Compact Parameters")
                            .strong()
                            .color(Color32::from_rgb(120, 220, 120)),
                    );

                    egui::Grid::new("extraction_results_grid")
                        .num_columns(4)
                        .spacing([20.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Vth0 (Threshold):");
                            ui.label(format!("{:.4} V", result.params.vth0));
                            ui.label("U0 (Low-Field Mobility):");
                            ui.label(format!("{:.5} m^2/V-s", result.params.u0));
                            ui.end_row();

                            ui.label("Vsat (Saturation Velocity):");
                            ui.label(format!("{:.1} m/s", result.params.vsat));
                            ui.label("DVT0 (Short-Channel):");
                            ui.label(format!("{:.4}", result.params.dvt0));
                            ui.end_row();

                            ui.label("ETA0 (DIBL Coefficient):");
                            ui.label(format!("{:.4}", result.params.eta0));
                            ui.label("RDSW (Series Resistance):");
                            ui.label(format!("{:.2} Ohm-um", result.params.rdsw));
                            ui.end_row();

                            ui.label("Subthreshold Swing:");
                            ui.label(format!("{:.2} mV/dec", result.params.subthreshold_swing_mv_dec));
                            ui.label("RMSE Residual:");
                            ui.label(format!("{:.4e} A", result.rmse));
                            ui.end_row();

                            ui.label("R^2 Determination:");
                            let r2_color = if result.r_squared >= 0.98 {
                                Color32::from_rgb(100, 240, 100)
                            } else {
                                Color32::from_rgb(240, 180, 80)
                            };
                            ui.label(RichText::new(format!("{:.6}", result.r_squared)).color(r2_color));
                            ui.label("Convergence:");
                            ui.label(if result.converged { "CONVERGED" } else { "ITERATING" });
                            ui.end_row();
                        });
                });

                ui.add_space(4.0);
            }

            if let Some(ref deck) = self.generated_deck {
                ui.group(|ui| {
                    ui.label(RichText::new("Generated SPICE .MODEL Deck").strong());
                    ui.add(
                        egui::TextEdit::multiline(&mut deck.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_rows(6)
                            .desired_width(f32::INFINITY),
                    );
                });
            }

            // CSV Manual Ingestion Collapsible Area
            ui.collapsing("Import Custom Measurement CSV Data", |ui| {
                ui.label("Paste CSV data (columns: Vds, Vgs, Ids or headered):");
                ui.add(
                    egui::TextEdit::multiline(&mut self.raw_csv_input)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(4)
                        .desired_width(f32::INFINITY),
                );
                if ui.button("Parse & Ingest CSV").clicked() {
                    self.parse_csv_input();
                }
            });
        });
    }
}
