#![deny(unsafe_code)]

//! Interactive Production Economics & Hierarchical Bill of Materials (BOM) Dialog.
//!
//! Provides an interactive 5-tab CAD production budgeting and pricing environment:
//! 1. Hierarchical BOM Tree & Split View (subcircuit decomposition with bottom-up rollup).
//! 2. Central Cost Registry Manager (single-source-of-truth pricing propagation).
//! 3. Volume Breakpoint Scaling Curves (economies of scale from N = 1 to 100,000).
//! 4. Manufacturing & Assembly Cost Model (PCB fab, SMT placement, testing, and MSRP margins).
//! 5. Production Export & Quotation (real-time CSV and Markdown production report export).

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use phonon_solver::production_economics::{
    Currency, ProductionEconomicsCoSimulator, VolumeBreakpoint,
};

/// Active tab in the Production Economics Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EconomicsTab {
    HierarchicalBomTree,
    CentralCostRegistry,
    VolumeScalingCurves,
    ManufacturingCostModel,
    ProductionExport,
}

/// Modal dialog for Production Economics & Hierarchical BOM Costing.
pub struct ProductionEconomicsDialog {
    pub is_open: bool,
    pub active_tab: EconomicsTab,

    // Selected currency for display and quotation export
    pub selected_currency: Currency,

    // Flag requesting BOM extraction from active schematic canvas
    pub request_sync_canvas: bool,

    // Filter and search buffers
    pub registry_search_query: String,
    pub new_part_key_buf: String,
    pub new_part_cost_buf: String,

    // Selected export volume target
    pub export_volume_choice: usize, // 0 = 10, 1 = 100, 2 = 1k, 3 = 10k, 4 = 100k

    // Co-simulator coordinator and cached plot curves
    pub sim: ProductionEconomicsCoSimulator,
    pub cached_unit_cost_curve: Vec<[f64; 2]>,
    pub cached_margin_curve: Vec<[f64; 2]>,
    pub cached_breakpoints: Vec<VolumeBreakpoint>,

    // Clipboard notification feedback
    pub feedback_message: Option<(String, f64)>,
}

impl Default for ProductionEconomicsDialog {
    fn default() -> Self {
        let sim = ProductionEconomicsCoSimulator::new_fast();
        let breakpoints = sim
            .volume_model
            .calculate_breakpoints(sim.report().prototype_bom_unit_cost);

        // Pre-seeded lightweight curves for instant cold boot (<1ms)
        let cached_unit_cost_curve = vec![
            [0.0, 172.5],
            [1.0, 23.4],
            [2.0, 6.8],
            [3.0, 3.4],
            [4.0, 2.58],
            [5.0, 2.15],
        ];

        let cached_margin_curve = vec![
            [0.0, -100.0],
            [1.0, 53.2],
            [2.0, 86.4],
            [3.0, 93.2],
            [4.0, 94.8],
            [5.0, 95.7],
        ];

        Self {
            is_open: false,
            active_tab: EconomicsTab::HierarchicalBomTree,
            selected_currency: Currency::USD,
            request_sync_canvas: false,
            registry_search_query: String::new(),
            new_part_key_buf: String::new(),
            new_part_cost_buf: String::new(),
            export_volume_choice: 3, // 10k
            sim,
            cached_unit_cost_curve,
            cached_margin_curve,
            cached_breakpoints: breakpoints,
            feedback_message: None,
        }
    }
}

impl ProductionEconomicsDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast non-blocking constructor guaranteeing sub-millisecond initialization for cold boot.
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Synchronizes the Bill of Materials directly from the active schematic canvas.
    pub fn sync_from_canvas(
        &mut self,
        canvas: &crate::schematic::SchematicCanvas,
        subcircuit_defs: &std::collections::HashMap<String, crate::schematic::subcircuit::SubcircuitDefinition>,
    ) {
        self.request_sync_canvas = false;
        let bom = crate::schematic::bom::generate_bom_from_canvas(canvas, subcircuit_defs);
        if !bom.items.is_empty() {
            self.sim.bom = bom;
            self.recompute_sim();
        }
    }

    /// Recomputes all economics models, volume breakpoints, and cached curves.
    pub fn recompute_sim(&mut self) {
        let rep = self.sim.recompute();
        let proto_bom = rep.prototype_bom_unit_cost;
        let breakpoints = self.sim.volume_model.calculate_breakpoints(proto_bom);

        // Build log10-spaced points for egui_plot (log10(1)=0, log10(10)=1, log10(100)=2, etc.)
        self.cached_unit_cost_curve = breakpoints
            .iter()
            .enumerate()
            .map(|(idx, b)| [idx as f64, b.total_unit_cost])
            .collect();

        self.cached_margin_curve = breakpoints
            .iter()
            .enumerate()
            .map(|(idx, b)| [idx as f64, b.gross_margin_pct])
            .collect();

        self.cached_breakpoints = breakpoints;
    }

    /// Render modal UI window (standard alias).
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the complete dialog window using egui 0.37 window borrow pattern.
    pub fn show(&mut self, ctx: &Context) {
        let mut is_open = self.is_open;
        Window::new("Production Economics & Hierarchical BOM Cost Estimator")
            .open(&mut is_open)
            .default_width(980.0)
            .default_height(700.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders dialog content and tabs.
    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    self.active_tab == EconomicsTab::HierarchicalBomTree,
                    "Hierarchical BOM Tree & Split View",
                )
                .clicked()
            {
                self.active_tab = EconomicsTab::HierarchicalBomTree;
            }
            if ui
                .selectable_label(
                    self.active_tab == EconomicsTab::CentralCostRegistry,
                    "Central Cost Registry Manager",
                )
                .clicked()
            {
                self.active_tab = EconomicsTab::CentralCostRegistry;
            }
            if ui
                .selectable_label(
                    self.active_tab == EconomicsTab::VolumeScalingCurves,
                    "Volume Breakpoint Scaling Curves",
                )
                .clicked()
            {
                self.active_tab = EconomicsTab::VolumeScalingCurves;
            }
            if ui
                .selectable_label(
                    self.active_tab == EconomicsTab::ManufacturingCostModel,
                    "Manufacturing & Assembly Model",
                )
                .clicked()
            {
                self.active_tab = EconomicsTab::ManufacturingCostModel;
            }
            if ui
                .selectable_label(
                    self.active_tab == EconomicsTab::ProductionExport,
                    "Production Export & Quotation",
                )
                .clicked()
            {
                self.active_tab = EconomicsTab::ProductionExport;
            }
        });

        ui.separator();

        match self.active_tab {
            EconomicsTab::HierarchicalBomTree => self.render_bom_tree(ui),
            EconomicsTab::CentralCostRegistry => self.render_cost_registry(ui),
            EconomicsTab::VolumeScalingCurves => self.render_volume_curves(ui),
            EconomicsTab::ManufacturingCostModel => self.render_manufacturing_model(ui),
            EconomicsTab::ProductionExport => self.render_production_export(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_bom_tree(&mut self, ui: &mut Ui) {
        ui.heading("Hierarchical Bill of Materials (BOM) & Interactive Split View");
        ui.label("Subcircuits can be maintained as estimated lump-sum costs or split into internal child components with automatic bottom-up rollup.");

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("Sync from Active Schematic").clicked() {
                self.request_sync_canvas = true;
            }
            if ui.button("Split All Subcircuits").clicked() {
                self.sim.bom.split_all();
                self.recompute_sim();
            }
            if ui.button("Collapse All to Lump-Sum").clicked() {
                self.sim.bom.unsplit_all();
                self.recompute_sim();
            }
            if ui.button("Reset Sample Avionics BOM").clicked() {
                self.sim.bom = phonon_solver::production_economics::HierarchicalBom::new_sample_avionics_power_supply();
                self.recompute_sim();
            }
            ui.separator();
            ui.label("Currency:");
            ui.selectable_value(&mut self.selected_currency, Currency::USD, "USD ($)");
            ui.selectable_value(&mut self.selected_currency, Currency::EUR, "EUR");
            ui.selectable_value(&mut self.selected_currency, Currency::GBP, "GBP");
            ui.selectable_value(&mut self.selected_currency, Currency::JPY, "JPY");
        });

        ui.add_space(8.0);

        let mut changed_price: Option<(String, f64)> = None;
        let mut toggle_split_id: Option<String> = None;

        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            egui::Grid::new("hierarchical_bom_grid")
                .striped(true)
                .num_columns(7)
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Designators").strong());
                    ui.label(RichText::new("Part Key").strong());
                    ui.label(RichText::new("Description").strong());
                    ui.label(RichText::new("Qty").strong());
                    ui.label(RichText::new(format!("Unit Price ({})", self.selected_currency.code())).strong());
                    ui.label(RichText::new(format!("Extended ({})", self.selected_currency.code())).strong());
                    ui.label(RichText::new("Hierarchical Split").strong());
                    ui.end_row();

                    for item in &self.sim.bom.items {
                        let eff_unit_usd = item.effective_unit_cost(&self.sim.registry);
                        let ext_usd = item.extended_cost(&self.sim.registry);

                        let eff_unit_loc = self.selected_currency.convert_from_usd(eff_unit_usd);
                        let ext_loc = self.selected_currency.convert_from_usd(ext_usd);

                        // Designator
                        if item.is_subcircuit {
                            ui.label(RichText::new(format!("[SUB] {}", item.designators)).color(Color32::from_rgb(180, 110, 255)).strong());
                        } else {
                            ui.label(&item.designators);
                        }

                        // Part key
                        ui.label(RichText::new(&item.part_key).monospace());

                        // Description
                        ui.label(&item.description);

                        // Quantity
                        ui.label(format!("{}", item.quantity));

                        // Unit price editor
                        ui.horizontal(|ui| {
                            ui.label(self.selected_currency.symbol());
                            let mut val = eff_unit_loc;
                            if item.is_subcircuit && item.is_split {
                                ui.label(RichText::new(format!("{:.4} (Sum)", val)).color(Color32::from_rgb(80, 220, 120)));
                            } else {
                                if ui.add(egui::DragValue::new(&mut val).speed(0.005).range(0.0001..=100000.0)).changed() {
                                    let usd_cost = val / self.selected_currency.rate_from_usd();
                                    changed_price = Some((item.part_key.clone(), usd_cost));
                                }
                            }
                        });

                        // Extended price
                        ui.label(format!("{}{:.4}", self.selected_currency.symbol(), ext_loc));

                        // Split button for subcircuits
                        if item.is_subcircuit {
                            let btn_label = if item.is_split { "Collapse" } else { "Split" };
                            if ui.button(btn_label).clicked() {
                                toggle_split_id = Some(item.id.clone());
                            }
                        } else {
                            ui.label("-");
                        }

                        ui.end_row();

                        // Render indented children if split
                        if item.is_subcircuit && item.is_split {
                            for child in &item.children {
                                let c_unit_usd = child.effective_unit_cost(&self.sim.registry);
                                let c_ext_usd = child.extended_cost(&self.sim.registry);

                                let c_unit_loc = self.selected_currency.convert_from_usd(c_unit_usd);
                                let c_ext_loc = self.selected_currency.convert_from_usd(c_ext_usd);

                                ui.horizontal(|ui| {
                                    ui.label("  ");
                                    ui.label(RichText::new(format!("-> {}", child.designators)).color(Color32::from_rgb(120, 200, 255)));
                                });

                                ui.label(RichText::new(&child.part_key).monospace());
                                ui.label(format!("(Child) {}", child.description));
                                ui.label(format!("{}", child.quantity));

                                ui.horizontal(|ui| {
                                    ui.label(self.selected_currency.symbol());
                                    let mut c_val = c_unit_loc;
                                    if ui.add(egui::DragValue::new(&mut c_val).speed(0.005).range(0.0001..=100000.0)).changed() {
                                        let usd_cost = c_val / self.selected_currency.rate_from_usd();
                                        changed_price = Some((child.part_key.clone(), usd_cost));
                                    }
                                });

                                ui.label(format!("{}{:.4}", self.selected_currency.symbol(), c_ext_loc));
                                ui.label(RichText::new("Child Element").weak());
                                ui.end_row();
                            }
                        }
                    }
                });
        });

        // Apply any split toggles
        if let Some(id) = toggle_split_id {
            if let Some(item) = self.sim.bom.find_item_mut(&id) {
                item.toggle_split();
                self.recompute_sim();
            }
        }

        // Apply synchronous price changes
        if let Some((key, new_cost)) = changed_price {
            self.sim.registry.set_cost(&key, new_cost);
            self.recompute_sim();
        }
    }

    fn render_cost_registry(&mut self, ui: &mut Ui) {
        ui.heading("Central Single-Source-of-Truth Component Cost Registry");
        ui.label("Editing a component price here instantly updates all matching instances across schematics and nested subcircuits.");

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Search Catalog:");
            ui.text_edit_singleline(&mut self.registry_search_query);

            if ui.button("Reset to Standard Catalog Presets").clicked() {
                self.sim.registry = phonon_solver::production_economics::CentralCostRegistry::new_with_standard_defaults();
                self.recompute_sim();
            }
        });

        ui.add_space(6.0);
        ui.label(RichText::new("Add / Override Custom Part Number:").strong());
        ui.horizontal(|ui| {
            ui.label("Part Key:");
            ui.text_edit_singleline(&mut self.new_part_key_buf);
            ui.label("Cost ($):");
            ui.text_edit_singleline(&mut self.new_part_cost_buf);

            if ui.button("Register Price").clicked() {
                if !self.new_part_key_buf.trim().is_empty() {
                    let cost = self.new_part_cost_buf.trim().parse::<f64>().unwrap_or(0.05);
                    self.sim.registry.set_cost(&self.new_part_key_buf, cost);
                    self.new_part_key_buf.clear();
                    self.new_part_cost_buf.clear();
                    self.recompute_sim();
                }
            }
        });

        ui.add_space(8.0);
        let sorted_keys = self.sim.registry.sorted_keys();
        let query = self.registry_search_query.to_lowercase();

        let mut changed_entry: Option<(String, f64)> = None;

        egui::ScrollArea::vertical().max_height(380.0).show(ui, |ui| {
            egui::Grid::new("cost_registry_grid")
                .striped(true)
                .num_columns(5)
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Part Key").strong());
                    ui.label(RichText::new("Unit Price ($)").strong());
                    ui.label(RichText::new("Supplier Part #").strong());
                    ui.label(RichText::new("Manufacturer").strong());
                    ui.label(RichText::new("Description").strong());
                    ui.end_row();

                    for key in sorted_keys {
                        if !query.is_empty() && !key.to_lowercase().contains(&query) {
                            continue;
                        }

                        if let Some(entry) = self.sim.registry.get_entry(&key) {
                            ui.label(RichText::new(&key).monospace().strong());

                            ui.horizontal(|ui| {
                                ui.label("$");
                                let mut cost = entry.unit_cost;
                                if ui.add(egui::DragValue::new(&mut cost).speed(0.005).range(0.0001..=1000.0)).changed() {
                                    changed_entry = Some((key.clone(), cost));
                                }
                            });

                            ui.label(&entry.supplier_pn);
                            ui.label(&entry.manufacturer);
                            ui.label(&entry.description);
                            ui.end_row();
                        }
                    }
                });
        });

        if let Some((k, new_cost)) = changed_entry {
            self.sim.registry.set_cost(&k, new_cost);
            self.recompute_sim();
        }
    }

    fn render_volume_curves(&mut self, ui: &mut Ui) {
        ui.heading("Production Volume Breakpoints & Economies of Scale");
        ui.label("Automated projection of batch manufacturing cost reductions (distributor reels, PCB panelization, SMT setup).");

        ui.add_space(8.0);

        // Volume Table
        egui::Grid::new("volume_breakpoints_grid")
            .striped(true)
            .num_columns(9)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Quantity").strong());
                ui.label(RichText::new("Tier Name").strong());
                ui.label(RichText::new("BOM ($)").strong());
                ui.label(RichText::new("PCB Fab ($)").strong());
                ui.label(RichText::new("SMT Svc ($)").strong());
                ui.label(RichText::new("QA ($)").strong());
                ui.label(RichText::new("Unit COGS ($)").strong());
                ui.label(RichText::new("Batch Total ($)").strong());
                ui.label(RichText::new("Gross Margin %").strong());
                ui.end_row();

                for bp in &self.cached_breakpoints {
                    ui.label(format!("{}", bp.quantity));
                    ui.label(&bp.tier_name);
                    ui.label(format!("${:.3}", bp.component_bom_unit_cost));
                    ui.label(format!("${:.3}", bp.pcb_fab_unit_cost));
                    ui.label(format!("${:.3}", bp.smt_assembly_unit_cost));
                    ui.label(format!("${:.3}", bp.testing_qa_unit_cost));
                    ui.label(RichText::new(format!("${:.3}", bp.total_unit_cost)).strong());
                    ui.label(format!("${:.2}", bp.total_batch_cost));

                    let margin_color = if bp.gross_margin_pct >= 50.0 {
                        Color32::from_rgb(80, 220, 120)
                    } else if bp.gross_margin_pct > 0.0 {
                        Color32::from_rgb(240, 160, 40)
                    } else {
                        Color32::from_rgb(240, 60, 60)
                    };

                    ui.colored_label(margin_color, format!("{:.1}%", bp.gross_margin_pct));
                    ui.end_row();
                }
            });

        ui.add_space(10.0);

        // Plot Unit Cost & Gross Margin curves
        Plot::new("volume_scaling_plot")
            .height(260.0)
            .legend(Legend::default())
            .x_axis_label("Volume Tier (0: 1-off, 1: 10, 2: 100, 3: 1k, 4: 10k, 5: 100k)")
            .y_axis_label("Unit Cost ($) / Margin (%)")
            .show(ui, |plot_ui| {
                let cost_pts = PlotPoints::new(self.cached_unit_cost_curve.clone());
                plot_ui.line(
                    Line::new("Unit COGS ($/unit)", cost_pts)
                        .color(Color32::from_rgb(255, 120, 80))
                        .width(2.5),
                );

                let margin_pts = PlotPoints::new(self.cached_margin_curve.clone());
                plot_ui.line(
                    Line::new("Gross Margin (%)", margin_pts)
                        .color(Color32::from_rgb(80, 220, 140))
                        .width(2.0),
                );
            });
    }

    fn render_manufacturing_model(&mut self, ui: &mut Ui) {
        ui.heading("Manufacturing & Assembly Cost Calibration");
        ui.label("Configure tooling fees, board dimensions, SMT joint placement rates, and retail pricing.");

        ui.add_space(10.0);
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label(RichText::new("Bare PCB Fabrication Parameters").strong());

                ui.horizontal(|ui| {
                    ui.label("PCB Tooling Setup Fee ($):");
                    changed |= ui.add(egui::DragValue::new(&mut self.sim.volume_model.pcb_setup_fee).range(0.0..=500.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Board Area (cm^2):");
                    changed |= ui.add(egui::DragValue::new(&mut self.sim.volume_model.board_area_cm2).range(1.0..=500.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Layer Count:");
                    egui::ComboBox::from_id_salt("pcb_layer_combo")
                        .selected_text(format!("{} Layers", self.sim.volume_model.layer_count))
                        .show_ui(ui, |ui| {
                            changed |= ui.selectable_value(&mut self.sim.volume_model.layer_count, 2, "2 Layers").changed();
                            changed |= ui.selectable_value(&mut self.sim.volume_model.layer_count, 4, "4 Layers").changed();
                            changed |= ui.selectable_value(&mut self.sim.volume_model.layer_count, 6, "6 Layers").changed();
                        });
                });
            });

            ui.group(|ui| {
                ui.label(RichText::new("SMT Assembly & Testing Parameters").strong());

                ui.horizontal(|ui| {
                    ui.label("SMT Stencil Setup Fee ($):");
                    changed |= ui.add(egui::DragValue::new(&mut self.sim.volume_model.smt_setup_fee).range(0.0..=1000.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("SMT Solder Joints Count:");
                    changed |= ui.add(egui::DragValue::new(&mut self.sim.volume_model.solder_joints_count).range(5..=5000)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Cost per Placement Joint ($):");
                    changed |= ui.add(egui::DragValue::new(&mut self.sim.volume_model.placement_cost_per_joint).speed(0.001).range(0.001..=0.05)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("QA & Firmware Flash Cost ($):");
                    changed |= ui.add(egui::DragValue::new(&mut self.sim.volume_model.testing_cost_per_unit).speed(0.05).range(0.0..=20.0)).changed();
                });
            });
        });

        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label(RichText::new("Commercial Retail Target (MSRP)").strong());
            ui.horizontal(|ui| {
                ui.label("Target Selling Price (MSRP $):");
                changed |= ui.add(egui::DragValue::new(&mut self.sim.volume_model.target_msrp).range(1.0..=10000.0)).changed();
            });

            let be = self.sim.volume_model.breakeven_volume_for_margin(
                self.sim.report().prototype_bom_unit_cost,
                50.0,
            );
            if let Some(qty) = be {
                ui.colored_label(
                    Color32::from_rgb(80, 220, 120),
                    format!("Commercial Breakeven (>= 50% Gross Margin): Achieved at N >= {} units", qty),
                );
            } else {
                ui.colored_label(
                    Color32::from_rgb(240, 80, 80),
                    "Target 50% Gross Margin not achievable at current MSRP. Consider increasing MSRP or reducing component costs.",
                );
            }
        });

        if changed {
            self.recompute_sim();
        }
    }

    fn render_production_export(&mut self, ui: &mut Ui) {
        ui.heading("Production Bill of Materials Export & Quotation");
        ui.label("Generate professional production quotations and engineering purchasing tables.");

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Target Batch Run Volume:");
            let volumes = [10, 100, 1_000, 10_000, 100_000];
            egui::ComboBox::from_id_salt("export_vol_combo")
                .selected_text(format!("N = {} units", volumes[self.export_volume_choice]))
                .show_ui(ui, |ui| {
                    for (idx, &vol) in volumes.iter().enumerate() {
                        ui.selectable_value(&mut self.export_volume_choice, idx, format!("N = {} units", vol));
                    }
                });
        });

        let target_vol = [10, 100, 1_000, 10_000, 100_000][self.export_volume_choice];

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("Copy CSV to Clipboard").clicked() {
                let csv = self.sim.bom.export_csv_with_currency(&self.sim.registry, target_vol, self.selected_currency);
                ui.ctx().copy_text(csv);
                self.feedback_message = Some((format!("Copied {} CSV Bill of Materials to clipboard", self.selected_currency.code()), 3.0));
            }

            if ui.button("Copy JSON to Clipboard").clicked() {
                let json = self.sim.bom.export_json(&self.sim.registry, target_vol, self.selected_currency);
                ui.ctx().copy_text(json);
                self.feedback_message = Some((format!("Copied {} JSON Bill of Materials to clipboard", self.selected_currency.code()), 3.0));
            }

            if ui.button("Copy Markdown Quotation Table").clicked() {
                let md = self.sim.bom.export_markdown_table_with_currency(&self.sim.registry, target_vol, self.selected_currency);
                ui.ctx().copy_text(md);
                self.feedback_message = Some((format!("Copied {} Markdown quotation to clipboard", self.selected_currency.code()), 3.0));
            }

            if let Some((ref msg, _)) = self.feedback_message {
                ui.colored_label(Color32::from_rgb(80, 220, 120), msg);
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new(format!("Formatted {} Bill of Materials Preview (CSV):", self.selected_currency.code())).strong());

        let csv_preview = self.sim.bom.export_csv_with_currency(&self.sim.registry, target_vol, self.selected_currency);
        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            ui.text_edit_multiline(&mut csv_preview.as_str());
        });
    }

    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let rep = self.sim.report();
        let sym = self.selected_currency.symbol();
        let p_bom = self.selected_currency.convert_from_usd(rep.prototype_bom_unit_cost);
        let m_bom = self.selected_currency.convert_from_usd(rep.mass_prod_bom_unit_cost_10k);
        let m_cogs = self.selected_currency.convert_from_usd(rep.total_cogs_unit_cost_10k);

        ui.horizontal(|ui| {
            ui.label(format!("Proto BOM: {}{:.3}", sym, p_bom));
            ui.separator();
            ui.label(format!("10k BOM: {}{:.3}", sym, m_bom));
            ui.separator();
            ui.label(format!("10k COGS: {}{:.3}", sym, m_cogs));
            ui.separator();
            ui.label(format!("10k Margin: {:.1}%", rep.gross_margin_pct_10k));
            ui.separator();
            ui.label(format!("Parts: {}", rep.total_component_count));
            ui.separator();
            ui.label(format!("Unique: {}", rep.unique_line_items_count));
            ui.separator();
            ui.label(format!("Subcircuits: {} ({} split)", rep.subcircuit_packages_count, rep.subcircuit_split_count));
            ui.separator();
            ui.colored_label(
                Color32::from_rgb(80, 220, 120),
                format!("Breakeven: N >= {}", rep.breakeven_volume_units),
            );
        });
    }
}
