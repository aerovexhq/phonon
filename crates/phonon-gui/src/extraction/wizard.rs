#![deny(unsafe_code)]

//! Visual Studio SPICE model parameter extraction wizard dialog and interactive fitting interface.

use crate::schematic::components::SchematicComponent;
use egui::{Color32, RichText, ScrollArea, Ui, Vec2};
use phonon_models::extraction::{
    generate_bjt_model_deck, generate_bsim4_model_deck, generate_ekv_model_deck,
    BjtBounds, BjtFittingResult, BjtOptimizer, Bsim4Bounds, EkvBounds, EkvFittingResult,
    EkvOptimizer, FittingResult, GaOptimizer, MeasuredCurve,
};

/// Compact semiconductor model type for parameter estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceModelKind {
    Bsim4,
    Ekv,
    GummelPoonBjt,
}

impl Default for DeviceModelKind {
    fn default() -> Self {
        Self::Bsim4
    }
}

/// Interactive SPICE model parameter extraction wizard dialog for Phonon Visual Studio.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractionWizardDialog {
    pub is_open: bool,
    pub model_kind: DeviceModelKind,
    pub current_curve: Option<MeasuredCurve>,
    pub fitting_result: Option<FittingResult>,
    pub ekv_result: Option<EkvFittingResult>,
    pub bjt_result: Option<BjtFittingResult>,
    pub generated_deck: Option<String>,
    pub model_name: String,
    pub is_nmos: bool,
    pub is_npn: bool,
    pub raw_csv_input: String,
    pub status_msg: String,
    pub max_generations: usize,
    pub population_size: usize,
    pub enable_lm_polishing: bool,
    pub bsim4_bounds: Bsim4Bounds,
    pub ekv_bounds: EkvBounds,
    pub bjt_bounds: BjtBounds,
}

impl Default for ExtractionWizardDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            model_kind: DeviceModelKind::Bsim4,
            current_curve: None,
            fitting_result: None,
            ekv_result: None,
            bjt_result: None,
            generated_deck: None,
            model_name: "NMOS_BSIM4".to_string(),
            is_nmos: true,
            is_npn: true,
            raw_csv_input: String::new(),
            status_msg: "Ready. Select a model, load curve data, and run optimization.".to_string(),
            max_generations: 25,
            population_size: 48,
            enable_lm_polishing: true,
            bsim4_bounds: Bsim4Bounds::default(),
            ekv_bounds: EkvBounds::default(),
            bjt_bounds: BjtBounds::default(),
        }
    }
}

impl ExtractionWizardDialog {
    /// Constructs a new extraction wizard dialog in closed state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Selects the active semiconductor device model kind.
    pub fn set_model_kind(&mut self, kind: DeviceModelKind) {
        if self.model_kind != kind {
            self.model_kind = kind;
            match kind {
                DeviceModelKind::Bsim4 => {
                    self.model_name = if self.is_nmos {
                        "NMOS_BSIM4".to_string()
                    } else {
                        "PMOS_BSIM4".to_string()
                    };
                }
                DeviceModelKind::Ekv => {
                    self.model_name = if self.is_nmos {
                        "NMOS_EKV".to_string()
                    } else {
                        "PMOS_EKV".to_string()
                    };
                }
                DeviceModelKind::GummelPoonBjt => {
                    self.model_name = if self.is_npn {
                        "NPN_BJT".to_string()
                    } else {
                        "PNP_BJT".to_string()
                    };
                }
            }
            self.current_curve = None;
            self.fitting_result = None;
            self.ekv_result = None;
            self.bjt_result = None;
            self.generated_deck = None;
            self.status_msg = format!("Selected model: {:?}", kind);
        }
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

    /// Loads a sample EKV MOSFET transfer curve.
    pub fn load_sample_ekv_transfer(&mut self) {
        let curve = MeasuredCurve::synthetic_ekv_transfer_curve(
            1.2,
            (0.0, 1.5, 50),
            0.50,
            1.5e-4,
            0.50,
            0.05,
        );
        self.status_msg = format!(
            "Loaded sample EKV transfer curve ({} points, Vds = 1.20 V)",
            curve.points.len()
        );
        self.current_curve = Some(curve);
        self.ekv_result = None;
        self.generated_deck = None;
    }

    /// Loads a sample EKV MOSFET output curve.
    pub fn load_sample_ekv_output(&mut self) {
        let curve = MeasuredCurve::synthetic_ekv_output_curve(
            1.0,
            (0.0, 1.8, 50),
            0.50,
            1.5e-4,
            0.50,
            0.05,
        );
        self.status_msg = format!(
            "Loaded sample EKV output curve ({} points, Vgs = 1.00 V)",
            curve.points.len()
        );
        self.current_curve = Some(curve);
        self.ekv_result = None;
        self.generated_deck = None;
    }

    /// Loads a sample Gummel-Poon BJT forward active curve.
    pub fn load_sample_bjt_forward_active(&mut self) {
        let curve = MeasuredCurve::synthetic_bjt_forward_active_curve(
            2.0,
            (0.40, 0.78, 50),
            1.0e-15,
            100.0,
            100.0,
        );
        self.status_msg = format!(
            "Loaded sample BJT forward active curve ({} points, Vce = 2.00 V)",
            curve.points.len()
        );
        self.current_curve = Some(curve);
        self.bjt_result = None;
        self.generated_deck = None;
    }

    /// Loads a sample Gummel-Poon BJT Gummel curve.
    pub fn load_sample_bjt_gummel(&mut self) {
        let curve = MeasuredCurve::synthetic_bjt_gummel_curve(
            2.0,
            (0.35, 0.80, 50),
            1.0e-15,
            100.0,
            100.0,
        );
        self.status_msg = format!(
            "Loaded sample BJT Gummel curve ({} points, Vce = 2.00 V)",
            curve.points.len()
        );
        self.current_curve = Some(curve);
        self.bjt_result = None;
        self.generated_deck = None;
    }

    /// Loads a sample Gummel-Poon BJT output curve.
    pub fn load_sample_bjt_output(&mut self) {
        let curve = MeasuredCurve::synthetic_bjt_output_curve(
            0.70,
            (0.0, 5.0, 50),
            1.0e-15,
            100.0,
            100.0,
        );
        self.status_msg = format!(
            "Loaded sample BJT output curve ({} points, Vbe = 0.70 V)",
            curve.points.len()
        );
        self.current_curve = Some(curve);
        self.bjt_result = None;
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
                self.ekv_result = None;
                self.bjt_result = None;
                self.generated_deck = None;
            }
            Err(e) => {
                self.status_msg = format!("Failed to parse CSV: {}", e);
            }
        }
    }

    /// Executes the parameter extraction engine for the selected model kind.
    pub fn run_fitting(&mut self) {
        let Some(ref curve) = self.current_curve else {
            self.status_msg =
                "No measurement curve loaded. Load a sample or import CSV first.".to_string();
            return;
        };

        match self.model_kind {
            DeviceModelKind::Bsim4 => {
                let result = if self.enable_lm_polishing {
                    GaOptimizer::fit_curve_with_bounds(
                        curve,
                        self.max_generations,
                        self.population_size,
                        &self.bsim4_bounds,
                    )
                } else {
                    GaOptimizer::fit_curve_ga_only(
                        curve,
                        self.max_generations,
                        self.population_size,
                    )
                };

                let deck = generate_bsim4_model_deck(
                    &self.model_name,
                    self.is_nmos,
                    &result.params,
                    &result,
                );

                self.status_msg = format!(
                    "BSIM4 Fitted: RMSE = {:.3e} A, R^2 = {:.5} (Converged: {})",
                    result.rmse, result.r_squared, result.converged
                );
                self.fitting_result = Some(result);
                self.generated_deck = Some(deck);
            }
            DeviceModelKind::Ekv => {
                let result = if self.enable_lm_polishing {
                    EkvOptimizer::fit_curve_with_bounds(
                        curve,
                        self.max_generations,
                        self.population_size,
                        &self.ekv_bounds,
                    )
                } else {
                    EkvOptimizer::fit_curve_ga_only(
                        curve,
                        self.max_generations,
                        self.population_size,
                        &self.ekv_bounds,
                    )
                };

                let deck = generate_ekv_model_deck(&self.model_name, self.is_nmos, &result.params);

                self.status_msg = format!(
                    "EKV Fitted: RMSE = {:.3e} A, R^2 = {:.5} (Converged: {})",
                    result.rmse, result.r_squared, result.converged
                );
                self.ekv_result = Some(result);
                self.generated_deck = Some(deck);
            }
            DeviceModelKind::GummelPoonBjt => {
                let result = if self.enable_lm_polishing {
                    BjtOptimizer::fit_curve_with_bounds(
                        curve,
                        self.max_generations,
                        self.population_size,
                        &self.bjt_bounds,
                    )
                } else {
                    BjtOptimizer::fit_curve_ga_only(
                        curve,
                        self.max_generations,
                        self.population_size,
                        &self.bjt_bounds,
                    )
                };

                let deck = generate_bjt_model_deck(&self.model_name, self.is_npn, &result.params);

                self.status_msg = format!(
                    "BJT Fitted: RMSE = {:.3e} A, R^2 = {:.5} (Converged: {})",
                    result.rmse, result.r_squared, result.converged
                );
                self.bjt_result = Some(result);
                self.generated_deck = Some(deck);
            }
        }
    }

    /// Performs a Levenberg-Marquardt local refinement step on currently fitted parameters.
    pub fn refine_with_lm(&mut self) {
        let Some(ref curve) = self.current_curve else {
            self.status_msg = "Cannot refine: No curve loaded.".to_string();
            return;
        };

        match self.model_kind {
            DeviceModelKind::Bsim4 => {
                let initial = self
                    .fitting_result
                    .as_ref()
                    .map(|r| r.params)
                    .unwrap_or_default();
                let refined = GaOptimizer::polish_parameters(&initial, curve);
                let deck = generate_bsim4_model_deck(
                    &self.model_name,
                    self.is_nmos,
                    &refined.params,
                    &refined,
                );
                self.status_msg = format!(
                    "BSIM4 LM Polished: RMSE = {:.3e} A, R^2 = {:.5}",
                    refined.rmse, refined.r_squared
                );
                self.fitting_result = Some(refined);
                self.generated_deck = Some(deck);
            }
            DeviceModelKind::Ekv => {
                let initial = self
                    .ekv_result
                    .as_ref()
                    .map(|r| r.params)
                    .unwrap_or_default();
                let refined = EkvOptimizer::polish_parameters(&initial, curve);
                let deck = generate_ekv_model_deck(&self.model_name, self.is_nmos, &refined.params);
                self.status_msg = format!(
                    "EKV LM Polished: RMSE = {:.3e} A, R^2 = {:.5}",
                    refined.rmse, refined.r_squared
                );
                self.ekv_result = Some(refined);
                self.generated_deck = Some(deck);
            }
            DeviceModelKind::GummelPoonBjt => {
                let initial = self
                    .bjt_result
                    .as_ref()
                    .map(|r| r.params)
                    .unwrap_or_default();
                let refined = BjtOptimizer::polish_parameters(&initial, curve);
                let deck = generate_bjt_model_deck(&self.model_name, self.is_npn, &refined.params);
                self.status_msg = format!(
                    "BJT LM Polished: RMSE = {:.3e} A, R^2 = {:.5}",
                    refined.rmse, refined.r_squared
                );
                self.bjt_result = Some(refined);
                self.generated_deck = Some(deck);
            }
        }
    }

    /// Updates a selected schematic component's properties and value with the fitted SPICE model.
    pub fn apply_to_component(&mut self, comp: &mut SchematicComponent) {
        comp.model_name = Some(self.model_name.clone());
        comp.value_str = self.model_name.clone();

        if let Some(ref deck) = self.generated_deck {
            update_or_insert_property(&mut comp.properties, "MODEL_DECK", deck);
        }

        match self.model_kind {
            DeviceModelKind::Bsim4 => {
                if let Some(ref res) = self.fitting_result {
                    update_or_insert_property(&mut comp.properties, "VTH0", &format!("{:.6}", res.params.vth0));
                    update_or_insert_property(&mut comp.properties, "U0", &format!("{:.6}", res.params.u0));
                    update_or_insert_property(&mut comp.properties, "VSAT", &format!("{:.1}", res.params.vsat));
                    update_or_insert_property(&mut comp.properties, "DVT0", &format!("{:.6}", res.params.dvt0));
                    update_or_insert_property(&mut comp.properties, "ETA0", &format!("{:.6}", res.params.eta0));
                    update_or_insert_property(&mut comp.properties, "RDSW", &format!("{:.2}", res.params.rdsw));
                    update_or_insert_property(&mut comp.properties, "RMSE", &format!("{:.6e}", res.rmse));
                    update_or_insert_property(&mut comp.properties, "R2", &format!("{:.6}", res.r_squared));
                }
            }
            DeviceModelKind::Ekv => {
                if let Some(ref res) = self.ekv_result {
                    update_or_insert_property(&mut comp.properties, "VTO", &format!("{:.6}", res.params.vto));
                    update_or_insert_property(&mut comp.properties, "KP", &format!("{:.6e}", res.params.kp));
                    update_or_insert_property(&mut comp.properties, "GAMMA", &format!("{:.6}", res.params.gamma));
                    update_or_insert_property(&mut comp.properties, "THETA", &format!("{:.6}", res.params.theta));
                    update_or_insert_property(&mut comp.properties, "RMSE", &format!("{:.6e}", res.rmse));
                    update_or_insert_property(&mut comp.properties, "R2", &format!("{:.6}", res.r_squared));
                }
            }
            DeviceModelKind::GummelPoonBjt => {
                if let Some(ref res) = self.bjt_result {
                    update_or_insert_property(&mut comp.properties, "IS", &format!("{:.6e}", res.params.is));
                    update_or_insert_property(&mut comp.properties, "BF", &format!("{:.4}", res.params.bf));
                    update_or_insert_property(&mut comp.properties, "VAF", &format!("{:.4}", res.params.vaf));
                    update_or_insert_property(&mut comp.properties, "RMSE", &format!("{:.6e}", res.rmse));
                    update_or_insert_property(&mut comp.properties, "R2", &format!("{:.6}", res.r_squared));
                }
            }
        }

        self.status_msg = format!(
            "Applied model '{}' to schematic component '{}'",
            self.model_name, comp.name
        );
    }

    /// Renders the extraction wizard modal dialog window within the egui context.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("SPICE Model Parameter Extraction Wizard")
            .open(&mut is_open)
            .default_size(Vec2::new(760.0, 620.0))
            .min_width(620.0)
            .min_height(440.0)
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

        // Model Architecture Selector
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Model Architecture:").strong());
                if ui
                    .selectable_label(self.model_kind == DeviceModelKind::Bsim4, "BSIM4 (MOSFET)")
                    .clicked()
                {
                    self.set_model_kind(DeviceModelKind::Bsim4);
                }
                if ui
                    .selectable_label(self.model_kind == DeviceModelKind::Ekv, "EKV (MOSFET)")
                    .clicked()
                {
                    self.set_model_kind(DeviceModelKind::Ekv);
                }
                if ui
                    .selectable_label(
                        self.model_kind == DeviceModelKind::GummelPoonBjt,
                        "Gummel-Poon (BJT)",
                    )
                    .clicked()
                {
                    self.set_model_kind(DeviceModelKind::GummelPoonBjt);
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Model Name:");
                ui.text_edit_singleline(&mut self.model_name);

                ui.separator();
                match self.model_kind {
                    DeviceModelKind::Bsim4 | DeviceModelKind::Ekv => {
                        ui.label("Polarity:");
                        if ui.selectable_label(self.is_nmos, "NMOS").clicked() {
                            self.is_nmos = true;
                        }
                        if ui.selectable_label(!self.is_nmos, "PMOS").clicked() {
                            self.is_nmos = false;
                        }
                    }
                    DeviceModelKind::GummelPoonBjt => {
                        ui.label("Polarity:");
                        if ui.selectable_label(self.is_npn, "NPN").clicked() {
                            self.is_npn = true;
                        }
                        if ui.selectable_label(!self.is_npn, "PNP").clicked() {
                            self.is_npn = false;
                        }
                    }
                }

                ui.separator();
                ui.label("GA Gens:");
                ui.add(egui::DragValue::new(&mut self.max_generations).range(5..=100));

                ui.label("Pop Size:");
                ui.add(egui::DragValue::new(&mut self.population_size).range(12..=128));

                ui.separator();
                ui.checkbox(&mut self.enable_lm_polishing, "LM Refinement");
            });
        });

        ui.add_space(4.0);

        // Parameter Bounds Adjustments collapsible section
        ui.collapsing("Parameter Bounds Adjustments", |ui| {
            match self.model_kind {
                DeviceModelKind::Bsim4 => {
                    egui::Grid::new("bsim4_bounds_grid").num_columns(4).spacing([12.0, 4.0]).show(ui, |ui| {
                        ui.label("Vth0 Min/Max:");
                        ui.add(egui::DragValue::new(&mut self.bsim4_bounds.vth0.0).speed(0.01).range(0.0..=1.5));
                        ui.add(egui::DragValue::new(&mut self.bsim4_bounds.vth0.1).speed(0.01).range(0.0..=2.0));
                        ui.end_row();

                        ui.label("U0 Min/Max:");
                        ui.add(egui::DragValue::new(&mut self.bsim4_bounds.u0.0).speed(0.005).range(0.001..=0.3));
                        ui.add(egui::DragValue::new(&mut self.bsim4_bounds.u0.1).speed(0.005).range(0.001..=0.5));
                        ui.end_row();

                        ui.label("Vsat Min/Max:");
                        ui.add(egui::DragValue::new(&mut self.bsim4_bounds.vsat.0).speed(1e3).range(1e4..=5e5));
                        ui.add(egui::DragValue::new(&mut self.bsim4_bounds.vsat.1).speed(1e3).range(1e4..=5e5));
                        ui.end_row();
                    });
                }
                DeviceModelKind::Ekv => {
                    egui::Grid::new("ekv_bounds_grid").num_columns(4).spacing([12.0, 4.0]).show(ui, |ui| {
                        ui.label("Vto Min/Max (V):");
                        ui.add(egui::DragValue::new(&mut self.ekv_bounds.vto.0).speed(0.01).range(0.0..=2.0));
                        ui.add(egui::DragValue::new(&mut self.ekv_bounds.vto.1).speed(0.01).range(0.0..=3.0));
                        ui.end_row();

                        ui.label("KP Min/Max (A/V^2):");
                        ui.add(egui::DragValue::new(&mut self.ekv_bounds.kp.0).speed(1e-6).range(1e-7..=1e-2));
                        ui.add(egui::DragValue::new(&mut self.ekv_bounds.kp.1).speed(1e-6).range(1e-7..=1e-2));
                        ui.end_row();

                        ui.label("Gamma Min/Max (V^0.5):");
                        ui.add(egui::DragValue::new(&mut self.ekv_bounds.gamma.0).speed(0.02).range(0.0..=3.0));
                        ui.add(egui::DragValue::new(&mut self.ekv_bounds.gamma.1).speed(0.02).range(0.0..=5.0));
                        ui.end_row();

                        ui.label("Theta Min/Max (1/V):");
                        ui.add(egui::DragValue::new(&mut self.ekv_bounds.theta.0).speed(0.01).range(0.0..=1.0));
                        ui.add(egui::DragValue::new(&mut self.ekv_bounds.theta.1).speed(0.01).range(0.0..=2.0));
                        ui.end_row();
                    });
                }
                DeviceModelKind::GummelPoonBjt => {
                    egui::Grid::new("bjt_bounds_grid").num_columns(4).spacing([12.0, 4.0]).show(ui, |ui| {
                        ui.label("IS Min/Max (A):");
                        ui.add(egui::DragValue::new(&mut self.bjt_bounds.is_range.0).speed(1e-19).range(1e-21..=1e-9));
                        ui.add(egui::DragValue::new(&mut self.bjt_bounds.is_range.1).speed(1e-19).range(1e-21..=1e-9));
                        ui.end_row();

                        ui.label("BF Min/Max:");
                        ui.add(egui::DragValue::new(&mut self.bjt_bounds.bf_range.0).speed(5.0).range(1.0..=1000.0));
                        ui.add(egui::DragValue::new(&mut self.bjt_bounds.bf_range.1).speed(5.0).range(1.0..=2000.0));
                        ui.end_row();

                        ui.label("VAF Min/Max (V):");
                        ui.add(egui::DragValue::new(&mut self.bjt_bounds.vaf_range.0).speed(5.0).range(1.0..=1000.0));
                        ui.add(egui::DragValue::new(&mut self.bjt_bounds.vaf_range.1).speed(5.0).range(1.0..=2000.0));
                        ui.end_row();
                    });
                }
            }
        });

        ui.add_space(4.0);

        // Data Ingestion Section
        ui.horizontal(|ui| {
            match self.model_kind {
                DeviceModelKind::Bsim4 => {
                    if ui.button("Load Transfer Curve").clicked() {
                        self.load_sample_nmos();
                    }
                    if ui.button("Load Output Curve").clicked() {
                        self.load_sample_output();
                    }
                    if ui.button("Load 4.2K Cryo Curve").clicked() {
                        self.load_sample_cryo();
                    }
                }
                DeviceModelKind::Ekv => {
                    if ui.button("Load EKV Transfer").clicked() {
                        self.load_sample_ekv_transfer();
                    }
                    if ui.button("Load EKV Output").clicked() {
                        self.load_sample_ekv_output();
                    }
                }
                DeviceModelKind::GummelPoonBjt => {
                    if ui.button("Load Forward Active").clicked() {
                        self.load_sample_bjt_forward_active();
                    }
                    if ui.button("Load Gummel Plot").clicked() {
                        self.load_sample_bjt_gummel();
                    }
                    if ui.button("Load BJT Output").clicked() {
                        self.load_sample_bjt_output();
                    }
                }
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

            if ui
                .add_enabled(
                    has_curve,
                    egui::Button::new(RichText::new("LM Local Refinement").strong()),
                )
                .clicked()
            {
                self.refine_with_lm();
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
                    if self.model_kind != DeviceModelKind::GummelPoonBjt {
                        ui.separator();
                        ui.label(format!("W: {:.2} um", curve.channel_width_m * 1e6));
                        ui.separator();
                        ui.label(format!("L: {:.0} nm", curve.channel_length_m * 1e9));
                    }
                });
            });
        }

        ui.add_space(4.0);

        // Results and Generated Deck View
        ScrollArea::vertical().show(ui, |ui| {
            match self.model_kind {
                DeviceModelKind::Bsim4 => {
                    if let Some(ref result) = self.fitting_result {
                        ui.group(|ui| {
                            ui.label(
                                RichText::new("Extracted BSIM4 Compact Parameters")
                                    .strong()
                                    .color(Color32::from_rgb(120, 220, 120)),
                            );

                            egui::Grid::new("bsim4_results_grid")
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
                }
                DeviceModelKind::Ekv => {
                    if let Some(ref result) = self.ekv_result {
                        ui.group(|ui| {
                            ui.label(
                                RichText::new("Extracted EKV Compact Parameters")
                                    .strong()
                                    .color(Color32::from_rgb(120, 220, 120)),
                            );

                            egui::Grid::new("ekv_results_grid")
                                .num_columns(4)
                                .spacing([20.0, 4.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label("VTO (Threshold):");
                                    ui.label(format!("{:.4} V", result.params.vto));
                                    ui.label("KP (Transconductance):");
                                    ui.label(format!("{:.5e} A/V^2", result.params.kp));
                                    ui.end_row();

                                    ui.label("GAMMA (Body Effect):");
                                    ui.label(format!("{:.4} V^0.5", result.params.gamma));
                                    ui.label("THETA (Mobility Degradation):");
                                    ui.label(format!("{:.4} 1/V", result.params.theta));
                                    ui.end_row();

                                    ui.label("RMSE Residual:");
                                    ui.label(format!("{:.4e} A", result.rmse));
                                    ui.label("R^2 Determination:");
                                    let r2_color = if result.r_squared >= 0.98 {
                                        Color32::from_rgb(100, 240, 100)
                                    } else {
                                        Color32::from_rgb(240, 180, 80)
                                    };
                                    ui.label(RichText::new(format!("{:.6}", result.r_squared)).color(r2_color));
                                    ui.end_row();
                                });
                        });
                        ui.add_space(4.0);
                    }
                }
                DeviceModelKind::GummelPoonBjt => {
                    if let Some(ref result) = self.bjt_result {
                        ui.group(|ui| {
                            ui.label(
                                RichText::new("Extracted Gummel-Poon BJT Parameters")
                                    .strong()
                                    .color(Color32::from_rgb(120, 220, 120)),
                            );

                            egui::Grid::new("bjt_results_grid")
                                .num_columns(4)
                                .spacing([20.0, 4.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label("IS (Saturation Current):");
                                    ui.label(format!("{:.4e} A", result.params.is));
                                    ui.label("BF (Forward Beta):");
                                    ui.label(format!("{:.2}", result.params.bf));
                                    ui.end_row();

                                    ui.label("VAF (Early Voltage):");
                                    ui.label(format!("{:.2} V", result.params.vaf));
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
                }
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
                ui.label("Paste CSV data (columns: Vds, Vgs, Vbs, Ids or headered):");
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

fn update_or_insert_property(props: &mut Vec<(String, String)>, key: &str, val: &str) {
    if let Some(existing) = props.iter_mut().find(|(k, _)| k == key) {
        existing.1 = val.to_string();
    } else {
        props.push((key.to_string(), val.to_string()));
    }
}
