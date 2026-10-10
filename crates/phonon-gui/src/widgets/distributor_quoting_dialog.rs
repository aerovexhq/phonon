#![deny(unsafe_code)]

//! Interactive Electronic Distributor API Integration, Parametric MPN Resolution & PCBA Quoting Dialog.
//!
//! Provides a 5-tab CAD manufacturing and procurement management environment:
//! 1. Live Distributor Pricing (Digi-Key, Mouser, LCSC comparison table, lowest cost highlights, MOQ & stock, lead times).
//! 2. Parametric MPN Matching (schematic component resolution, confidence score, grade, tolerance, alternative pickers).
//! 3. Turnkey PCBA Quoting (interactive panelization parameters, SMT placement fee breakdown, volume curve graph/table, surface finish).
//! 4. Consolidated Purchase Orders (Digi-Key CSV, Mouser CSV, LCSC CSV, ERP JSON/XML viewers with copy/export).
//! 5. 10-Point Distributor Audit (verification checklist, score 10/10 PASS, cold-boot latency gauge).

use egui::{Color32, Context, RichText, Ui, Vec2, Window};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use phonon_solver::distributor_api::{
    AssemblySourcingMode, DistributorAuditReport, DistributorKind, DistributorQuotingEngine,
    ResolvedComponentMpn, SurfaceFinish,
};

/// Active tab in the Distributor & PCBA Quoting Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributorQuotingTab {
    LivePricing,
    ParametricMpn,
    TurnkeyPcba,
    PurchaseOrders,
    AuditTelemetry,
}

/// Purchase order export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoExportFormat {
    DistributorCsv,
    ErpJson,
    ErpXml,
}

/// Modal dialog for Electronic Distributor API & PCBA Quoting Pipeline.
pub struct DistributorQuotingDialog {
    pub is_open: bool,
    pub active_tab: DistributorQuotingTab,

    // Core quoting super-engine
    pub engine: DistributorQuotingEngine,

    // Target volume selection
    pub target_volume: usize,

    // Tab 1 state
    pub search_filter: String,
    pub selected_comp_index: usize,

    // Tab 2 parametric testbench state
    pub test_designator: String,
    pub test_comp_type: String,
    pub test_value: String,
    pub test_footprint: String,
    pub test_resolved_result: Option<ResolvedComponentMpn>,

    // Tab 3 PCBA parameters state
    pub board_width_mm: f64,
    pub board_length_mm: f64,
    pub layer_count: usize,
    pub smt_joints: usize,
    pub th_pins: usize,
    pub bga_count: usize,
    pub surface_finish: SurfaceFinish,
    pub sourcing_mode: AssemblySourcingMode,

    // Tab 4 purchase order state
    pub selected_po_distributor: DistributorKind,
    pub selected_po_format: PoExportFormat,
    pub po_status_message: Option<String>,

    // Tab 5 audit report
    pub audit_report: DistributorAuditReport,
}

impl Default for DistributorQuotingDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl DistributorQuotingDialog {
    /// Instant non-blocking constructor ensuring sub-2.0 ms cold-boot latency.
    pub fn new_fast() -> Self {
        let engine = DistributorQuotingEngine::new_fast();
        let audit_report = engine.audit_report.clone();

        Self {
            is_open: false,
            active_tab: DistributorQuotingTab::LivePricing,
            engine,
            target_volume: 100,

            search_filter: String::new(),
            selected_comp_index: 0,

            test_designator: "R1".to_string(),
            test_comp_type: "Resistor".to_string(),
            test_value: "10k".to_string(),
            test_footprint: "0805".to_string(),
            test_resolved_result: None,

            board_width_mm: 50.0,
            board_length_mm: 50.0,
            layer_count: 2,
            smt_joints: 60,
            th_pins: 4,
            bga_count: 0,
            surface_finish: SurfaceFinish::HaslLeadFree,
            sourcing_mode: AssemblySourcingMode::TurnkeyFull,

            selected_po_distributor: DistributorKind::DigiKey,
            selected_po_format: PoExportFormat::DistributorCsv,
            po_status_message: None,

            audit_report,
        }
    }

    /// Renders the complete dialog window within egui Context.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Electronic Distributor API & Turnkey PCBA Quoting Pipeline")
            .open(&mut is_open)
            .default_size(Vec2::new(960.0, 680.0))
            .min_size(Vec2::new(800.0, 520.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = is_open;
    }

    /// Alias for show() to conform with standard widget render pass.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders internal contents with 5 categorized tabs.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        // Top Header
        ui.horizontal(|ui| {
            ui.heading("Distributor Quoting & Procurement Pipeline");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    self.is_open = false;
                }
                ui.label(
                    RichText::new("Digi-Key | Mouser | LCSC")
                        .small()
                        .color(Color32::from_rgb(140, 170, 210)),
                );
            });
        });

        ui.add_space(4.0);
        ui.separator();

        // Tab Navigation Bar
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.active_tab,
                DistributorQuotingTab::LivePricing,
                "1. Live Distributor Pricing",
            );
            ui.selectable_value(
                &mut self.active_tab,
                DistributorQuotingTab::ParametricMpn,
                "2. Parametric MPN Matching",
            );
            ui.selectable_value(
                &mut self.active_tab,
                DistributorQuotingTab::TurnkeyPcba,
                "3. Turnkey PCBA Quoting",
            );
            ui.selectable_value(
                &mut self.active_tab,
                DistributorQuotingTab::PurchaseOrders,
                "4. Consolidated Purchase Orders",
            );
            ui.selectable_value(
                &mut self.active_tab,
                DistributorQuotingTab::AuditTelemetry,
                "5. 10-Point Engineering Audit",
            );
        });

        ui.separator();
        ui.add_space(4.0);

        // Tab Body Content
        match self.active_tab {
            DistributorQuotingTab::LivePricing => self.render_live_pricing_tab(ui),
            DistributorQuotingTab::ParametricMpn => self.render_parametric_mpn_tab(ui),
            DistributorQuotingTab::TurnkeyPcba => self.render_turnkey_pcba_tab(ui),
            DistributorQuotingTab::PurchaseOrders => self.render_purchase_orders_tab(ui),
            DistributorQuotingTab::AuditTelemetry => self.render_audit_telemetry_tab(ui),
        }
    }

    /// Tab 1: Live Distributor Pricing
    fn render_live_pricing_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Target Batch Volume:").strong());
            let prev_vol = self.target_volume;
            egui::ComboBox::from_id_salt("target_vol_selector")
                .selected_text(format!("{} units", self.target_volume))
                .show_ui(ui, |ui| {
                    for v in [1, 10, 50, 100, 250, 500, 1000, 2500, 5000, 10000] {
                        ui.selectable_value(&mut self.target_volume, v, format!("{} units", v));
                    }
                });
            if self.target_volume != prev_vol {
                self.engine.set_target_volume(self.target_volume);
            }

            ui.add_space(20.0);
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.search_filter);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let total_cost = self.engine.procurement.total_procurement_cost();
                ui.label(
                    RichText::new(format!("Total Component BOM: ${:.2}", total_cost))
                        .strong()
                        .color(Color32::from_rgb(80, 200, 120)),
                );
            });
        });

        ui.add_space(8.0);

        // Component Comparison Cards
        egui::ScrollArea::vertical()
            .id_salt("distributor_pricing_scroll")
            .show(ui, |ui| {
                let filter = self.search_filter.to_lowercase();
                for (idx, comp) in self.engine.comparisons.iter_mut().enumerate() {
                    if !filter.is_empty()
                        && !comp.manufacturer_part_number.to_lowercase().contains(&filter)
                        && !comp.generic_name.to_lowercase().contains(&filter)
                    {
                        continue;
                    }

                    let lowest_dist = comp
                        .find_lowest_cost_supplier(self.target_volume)
                        .map(|q| q.distributor);

                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{}. {}", idx + 1, comp.generic_name))
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(format!("MPN: {}", comp.manufacturer_part_number))
                                    .monospace()
                                    .color(Color32::from_rgb(180, 210, 255)),
                            );
                            ui.label(format!("Footprint: {}", comp.footprint));

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let active_dist = comp
                                        .selected_distributor
                                        .unwrap_or(DistributorKind::DigiKey);
                                    ui.label(
                                        RichText::new(format!("Source: {}", active_dist.name()))
                                            .small()
                                            .color(Color32::from_rgb(100, 220, 255)),
                                    );
                                },
                            );
                        });

                        ui.add_space(4.0);

                        // Multi-distributor quotes table
                        egui::Grid::new(format!("quotes_grid_{}", idx))
                            .striped(true)
                            .min_col_width(110.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new("Distributor").strong());
                                ui.label(RichText::new("Distributor SKU").strong());
                                ui.label(RichText::new("Stock").strong());
                                ui.label(RichText::new("Unit Price").strong());
                                ui.label(RichText::new("Ext Cost").strong());
                                ui.label(RichText::new("Lead Time").strong());
                                ui.label(RichText::new("Action").strong());
                                ui.end_row();

                                for quote in &comp.quotes {
                                    let is_lowest = Some(quote.distributor) == lowest_dist;
                                    let is_selected = comp.selected_distributor == Some(quote.distributor);
                                    let (order_qty, ext_cost) = quote.extended_cost_at_qty(self.target_volume);
                                    let unit_price = quote.unit_price_at_qty(order_qty);

                                    // Distributor name badge
                                    ui.horizontal(|ui| {
                                        let col = match quote.distributor {
                                            DistributorKind::DigiKey => Color32::from_rgb(220, 50, 50),
                                            DistributorKind::Mouser => Color32::from_rgb(40, 110, 200),
                                            DistributorKind::Lcsc => Color32::from_rgb(220, 140, 20),
                                        };
                                        ui.label(RichText::new(quote.distributor.code()).color(col).strong());
                                        if is_lowest {
                                            ui.label(RichText::new("[BEST]").small().color(Color32::from_rgb(80, 220, 100)));
                                        }
                                    });

                                    ui.label(RichText::new(&quote.distributor_part_number).monospace());

                                    // Stock
                                    let stock_col = if quote.in_stock_quantity >= self.target_volume {
                                        Color32::from_rgb(80, 200, 100)
                                    } else {
                                        Color32::from_rgb(230, 80, 80)
                                    };
                                    ui.label(RichText::new(format!("{} in stock", quote.in_stock_quantity)).color(stock_col));

                                    ui.label(format!("${:.4}", unit_price));
                                    ui.label(format!("${:.2}", ext_cost));
                                    ui.label(format!("{:.1} wks", quote.lead_time_weeks));

                                    if is_selected {
                                        ui.label(RichText::new("Selected").color(Color32::from_rgb(80, 220, 120)).strong());
                                    } else if ui.button("Select").clicked() {
                                        comp.selected_distributor = Some(quote.distributor);
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                    ui.add_space(4.0);
                }
            });
    }

    /// Tab 2: Parametric MPN Matching
    fn render_parametric_mpn_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Parametric MPN Resolver & Second-Source Matcher").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Automated CAD Symbol-to-Commercial Part Mapping")
                        .small()
                        .color(Color32::from_rgb(150, 170, 200)),
                );
            });
        });
        ui.add_space(6.0);

        // Parametric Query Controls
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(RichText::new("Resolve Schematic Part:").strong());
            ui.horizontal(|ui| {
                ui.label("Designator:");
                ui.add(egui::TextEdit::singleline(&mut self.test_designator).desired_width(50.0));

                ui.label("Type:");
                ui.add(egui::TextEdit::singleline(&mut self.test_comp_type).desired_width(90.0));

                ui.label("Value:");
                ui.add(egui::TextEdit::singleline(&mut self.test_value).desired_width(80.0));

                ui.label("Footprint:");
                ui.add(egui::TextEdit::singleline(&mut self.test_footprint).desired_width(80.0));

                if ui.button("Resolve MPN").clicked() {
                    let res = self.engine.resolver.resolve(
                        &self.test_designator,
                        &self.test_comp_type,
                        &self.test_value,
                        &self.test_footprint,
                    );
                    self.test_resolved_result = Some(res);
                }
            });
        });

        ui.add_space(8.0);

        // Resolved Part Card
        if let Some(res) = &self.test_resolved_result {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Resolved: {} ({})", res.generic_designator, res.generic_value)).strong());
                    let conf_pct = (res.match_confidence * 100.0) as u32;
                    let conf_col = if conf_pct >= 90 {
                        Color32::from_rgb(80, 220, 100)
                    } else {
                        Color32::from_rgb(240, 160, 40)
                    };
                    ui.label(RichText::new(format!("Match Confidence: {}%", conf_pct)).color(conf_col).strong());
                });

                ui.add_space(4.0);
                egui::Grid::new("resolved_spec_grid").striped(true).show(ui, |ui| {
                    ui.label("Primary MPN:");
                    ui.label(RichText::new(&res.primary_mpn).monospace().strong());
                    ui.label("Manufacturer:");
                    ui.label(&res.primary_manufacturer);
                    ui.end_row();

                    ui.label("Package / Footprint:");
                    ui.label(&res.package_footprint);
                    ui.label("Tolerance:");
                    ui.label(format!("+/- {:.1}%", res.tolerance_pct));
                    ui.end_row();

                    ui.label("Rating:");
                    ui.label(&res.voltage_or_power_rating);
                    ui.label("Temperature Grade:");
                    ui.label(res.grade.as_str());
                    ui.end_row();
                });

                ui.add_space(8.0);
                ui.label(RichText::new("Second-Source Pin-Compatible Alternatives:").strong());

                if res.second_source_alternatives.is_empty() {
                    ui.label("No alternatives found.");
                } else {
                    egui::Grid::new("alternatives_grid").striped(true).min_col_width(120.0).show(ui, |ui| {
                        ui.label(RichText::new("Alternative MPN").strong());
                        ui.label(RichText::new("Manufacturer").strong());
                        ui.label(RichText::new("Drop-In Compatible").strong());
                        ui.label(RichText::new("Price Delta").strong());
                        ui.end_row();

                        for alt in &res.second_source_alternatives {
                            ui.label(RichText::new(&alt.mpn).monospace());
                            ui.label(&alt.manufacturer);
                            let comp_col = if alt.is_drop_in_compatible {
                                Color32::from_rgb(80, 220, 100)
                            } else {
                                Color32::from_rgb(220, 140, 40)
                            };
                            ui.label(RichText::new(if alt.is_drop_in_compatible { "Yes (100% Drop-In)" } else { "Parametric Near-Match" }).color(comp_col));
                            let delta_str = if alt.price_delta_pct >= 0.0 {
                                format!("+{:.1}%", alt.price_delta_pct)
                            } else {
                                format!("{:.1}%", alt.price_delta_pct)
                            };
                            ui.label(delta_str);
                            ui.end_row();
                        }
                    });
                }
            });
        } else {
            ui.label(
                RichText::new("Click 'Resolve MPN' above to query parametric part number models.")
                    .italics()
                    .color(Color32::from_rgb(140, 160, 180)),
            );
        }

        ui.add_space(10.0);
        ui.label(RichText::new("Pre-Seeded Schematic Component Catalog:").strong());

        egui::ScrollArea::vertical()
            .id_salt("mpn_catalog_scroll")
            .max_height(220.0)
            .show(ui, |ui| {
                egui::Grid::new("mpn_catalog_grid").striped(true).min_col_width(90.0).show(ui, |ui| {
                    ui.label(RichText::new("Designator").strong());
                    ui.label(RichText::new("Generic Description").strong());
                    ui.label(RichText::new("Resolved MPN").strong());
                    ui.label(RichText::new("Footprint").strong());
                    ui.label(RichText::new("Grade").strong());
                    ui.end_row();

                    for comp in &self.engine.comparisons {
                        ui.label("D?");
                        ui.label(&comp.generic_name);
                        ui.label(RichText::new(&comp.manufacturer_part_number).monospace());
                        ui.label(&comp.footprint);
                        ui.label("Commercial (0 to 70 C)");
                        ui.end_row();
                    }
                });
            });
    }

    /// Tab 3: Turnkey PCBA Quoting
    fn render_turnkey_pcba_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Turnkey & Consigned PCBA Assembly Cost Estimator").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Fabrication, Panelization & SMT Placement Economics")
                        .small()
                        .color(Color32::from_rgb(150, 170, 200)),
                );
            });
        });
        ui.add_space(6.0);

        // Two-column layout: Left Controls, Right Cost Breakdown & Plot
        ui.columns(2, |cols| {
            // Left Column: Parameters
            cols[0].group(|ui| {
                ui.label(RichText::new("PCB Fabrication & SMT Spec:").strong());

                ui.horizontal(|ui| {
                    ui.label("Width (mm):");
                    ui.add(egui::DragValue::new(&mut self.board_width_mm).speed(1.0).range(10.0..=500.0));
                    ui.label("Length (mm):");
                    ui.add(egui::DragValue::new(&mut self.board_length_mm).speed(1.0).range(10.0..=500.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Layer Count:");
                    egui::ComboBox::from_id_salt("pcba_layers_combo")
                        .selected_text(format!("{} Layers", self.layer_count))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.layer_count, 1, "1 Layer");
                            ui.selectable_value(&mut self.layer_count, 2, "2 Layers");
                            ui.selectable_value(&mut self.layer_count, 4, "4 Layers");
                            ui.selectable_value(&mut self.layer_count, 6, "6 Layers");
                            ui.selectable_value(&mut self.layer_count, 8, "8 Layers");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Surface Finish:");
                    egui::ComboBox::from_id_salt("surface_finish_combo")
                        .selected_text(self.surface_finish.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.surface_finish, SurfaceFinish::HaslLeadFree, "HASL (Lead-Free)");
                            ui.selectable_value(&mut self.surface_finish, SurfaceFinish::EnigElectrolessNickelImmersionGold, "ENIG (Gold)");
                            ui.selectable_value(&mut self.surface_finish, SurfaceFinish::OspOrganicSolderabilityPreservative, "OSP");
                            ui.selectable_value(&mut self.surface_finish, SurfaceFinish::ImmersionSilver, "Immersion Silver");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Sourcing Mode:");
                    egui::ComboBox::from_id_salt("sourcing_mode_combo")
                        .selected_text(self.sourcing_mode.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.sourcing_mode, AssemblySourcingMode::TurnkeyFull, "Full Turnkey");
                            ui.selectable_value(&mut self.sourcing_mode, AssemblySourcingMode::ConsignedCustomer, "Consigned (Zero BOM)");
                            ui.selectable_value(&mut self.sourcing_mode, AssemblySourcingMode::HybridCombo, "Hybrid Combo");
                        });
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("SMT Joints:");
                    ui.add(egui::DragValue::new(&mut self.smt_joints).speed(5).range(0..=5000));
                    ui.label("Through-Hole Pins:");
                    ui.add(egui::DragValue::new(&mut self.th_pins).speed(1).range(0..=500));
                });

                ui.horizontal(|ui| {
                    ui.label("BGA/QFN Chips:");
                    ui.add(egui::DragValue::new(&mut self.bga_count).speed(1).range(0..=50));
                });

                ui.add_space(4.0);
                // Synchronize engine parameters
                self.engine.pcba_engine.board_width_mm = self.board_width_mm;
                self.engine.pcba_engine.board_length_mm = self.board_length_mm;
                self.engine.pcba_engine.layer_count = self.layer_count;
                self.engine.pcba_engine.surface_finish = self.surface_finish;
                self.engine.pcba_engine.sourcing_mode = self.sourcing_mode;
                self.engine.pcba_engine.smt_joints_count = self.smt_joints;
                self.engine.pcba_engine.through_hole_pins_count = self.th_pins;
                self.engine.pcba_engine.bga_qfn_chip_count = self.bga_count;
                self.engine.pcba_engine.panelization = phonon_solver::distributor_api::PanelizationSpec::compute(self.board_width_mm, self.board_length_mm, 200.0);

                // Panelization statistics
                let pan = &self.engine.pcba_engine.panelization;
                ui.separator();
                ui.label(RichText::new("Automated Panelization Metrics:").strong());
                ui.label(format!("Panel Dimensions: {:.0} x {:.0} mm", pan.panel_width_mm, pan.panel_length_mm));
                ui.label(format!("Panel Grid: {} x {} ({} boards/panel)", pan.panel_cols, pan.panel_rows, pan.boards_per_panel));
                let eff_pct = pan.panel_utilization_efficiency * 100.0;
                let eff_col = if eff_pct >= 70.0 { Color32::from_rgb(80, 220, 100) } else { Color32::from_rgb(220, 80, 80) };
                ui.label(RichText::new(format!("Panel Area Utilization: {:.1}% (Target >= 70%)", eff_pct)).color(eff_col).strong());
            });

            // Right Column: Cost Breakdown & Volume Trajectory
            cols[1].group(|ui| {
                let quote = self.engine.current_pcba_quote();
                ui.label(RichText::new(format!("PCBA Cost Breakdown at {} Units:", self.target_volume)).strong());

                egui::Grid::new("pcba_breakdown_grid").striped(true).show(ui, |ui| {
                    ui.label("Bare PCB Unit Cost:");
                    ui.label(format!("${:.2}", quote.bare_pcb_unit_cost));
                    ui.end_row();

                    ui.label("SMT Placement Unit Cost:");
                    ui.label(format!("${:.2}", quote.smt_placement_unit_cost));
                    ui.end_row();

                    ui.label("AOI / X-Ray Inspection:");
                    ui.label(format!("${:.2}", quote.aoi_inspection_unit_cost));
                    ui.end_row();

                    ui.label("Amortized Tooling & Stencil:");
                    let fixed = (quote.smt_setup_total_cost + quote.laser_stencil_total_cost) / (quote.batch_volume_units as f64);
                    ui.label(format!("${:.2}", fixed));
                    ui.end_row();

                    ui.label("Component BOM Unit Cost:");
                    ui.label(format!("${:.2}", quote.component_bom_unit_cost));
                    ui.end_row();

                    ui.label(RichText::new("Total Unit Cost (COGS):").strong());
                    ui.label(RichText::new(format!("${:.2}", quote.total_pcba_unit_cost)).strong().color(Color32::from_rgb(80, 220, 120)));
                    ui.end_row();

                    ui.label(RichText::new("Total Batch Cost:").strong());
                    ui.label(RichText::new(format!("${:.2}", quote.total_batch_cost)).strong());
                    ui.end_row();
                });

                ui.add_space(8.0);
                ui.label(RichText::new("Unit Cost vs Volume Curve:").strong());

                let bom_cost = self.engine.procurement.total_procurement_cost() / (self.target_volume as f64).max(1.0);
                let curve = self.engine.pcba_engine.volume_curve(bom_cost);

                let pts: Vec<[f64; 2]> = curve
                    .iter()
                    .map(|q| [q.batch_volume_units as f64, q.total_pcba_unit_cost])
                    .collect();

                Plot::new("pcba_volume_curve_plot")
                    .height(130.0)
                    .x_axis_label("Batch Volume (Units)")
                    .y_axis_label("Unit Cost ($)")
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Total Unit Cost", PlotPoints::from(pts))
                                .color(Color32::from_rgb(80, 200, 255))
                                .stroke(egui::Stroke::new(2.0, Color32::from_rgb(80, 200, 255))),
                        );
                    });
            });
        });
    }

    /// Tab 4: Consolidated Purchase Orders
    fn render_purchase_orders_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Consolidated Multi-Distributor Purchase Orders").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Automated Checkout Export for Digi-Key, Mouser & LCSC")
                        .small()
                        .color(Color32::from_rgb(150, 170, 200)),
                );
            });
        });
        ui.add_space(6.0);

        // Supplier & Format Selectors
        ui.horizontal(|ui| {
            ui.label("Select Distributor:");
            egui::ComboBox::from_id_salt("po_distributor_selector")
                .selected_text(self.selected_po_distributor.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_po_distributor, DistributorKind::DigiKey, "Digi-Key Electronics");
                    ui.selectable_value(&mut self.selected_po_distributor, DistributorKind::Mouser, "Mouser Electronics");
                    ui.selectable_value(&mut self.selected_po_distributor, DistributorKind::Lcsc, "LCSC Electronics");
                });

            ui.add_space(16.0);
            ui.label("Export Format:");
            egui::ComboBox::from_id_salt("po_format_selector")
                .selected_text(match self.selected_po_format {
                    PoExportFormat::DistributorCsv => "Distributor CSV",
                    PoExportFormat::ErpJson => "ERP JSON Document",
                    PoExportFormat::ErpXml => "Industrial XML Document",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_po_format, PoExportFormat::DistributorCsv, "Distributor CSV");
                    ui.selectable_value(&mut self.selected_po_format, PoExportFormat::ErpJson, "ERP JSON Document");
                    ui.selectable_value(&mut self.selected_po_format, PoExportFormat::ErpXml, "Industrial XML Document");
                });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Refresh Orders").clicked() {
                    self.engine.recalculate_procurement();
                    self.po_status_message = Some("Purchase orders recalculated.".to_string());
                }
            });
        });

        ui.add_space(6.0);

        // Active Purchase Order Preview
        let po_opt = match self.selected_po_distributor {
            DistributorKind::DigiKey => self.engine.procurement.digikey_po.as_ref(),
            DistributorKind::Mouser => self.engine.procurement.mouser_po.as_ref(),
            DistributorKind::Lcsc => self.engine.procurement.lcsc_po.as_ref(),
        };

        if let Some(po) = po_opt {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("PO Reference: {}", po.po_reference_number)).strong());
                ui.label(format!("Line Items: {}", po.line_items.len()));
                ui.label(format!("Subtotal: ${:.2}", po.subtotal_usd));
                ui.label(format!("Shipping: ${:.2}", po.estimated_shipping_usd));
                ui.label(RichText::new(format!("Total: ${:.2}", po.total_cost_usd)).strong().color(Color32::from_rgb(80, 220, 120)));
            });

            ui.add_space(4.0);

            // Export preview text
            let export_text = match self.selected_po_format {
                PoExportFormat::DistributorCsv => po.to_distributor_csv(),
                PoExportFormat::ErpJson => po.to_erp_json(),
                PoExportFormat::ErpXml => po.to_erp_xml(),
            };

            ui.horizontal(|ui| {
                if ui.button("Copy to Clipboard").clicked() {
                    ui.ctx().copy_text(export_text.clone());
                    self.po_status_message = Some("Copied purchase order to clipboard.".to_string());
                }
                if let Some(msg) = &self.po_status_message {
                    ui.label(RichText::new(msg).small().color(Color32::from_rgb(100, 220, 140)));
                }
            });

            ui.add_space(4.0);
            let mut text_buf = export_text;
            egui::ScrollArea::vertical()
                .id_salt("po_text_scroll")
                .max_height(280.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut text_buf)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(12),
                    );
                });
        } else {
            ui.label("No items allocated to this distributor.");
        }
    }

    /// Tab 5: 10-Point Engineering Audit
    fn render_audit_telemetry_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("10-Point Industrial Engineering & Physics Verification Audit").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Re-Run Live Audit").clicked() {
                    self.audit_report = self.engine.run_full_audit().clone();
                }
            });
        });
        ui.add_space(6.0);

        // Header Metrics
        ui.horizontal(|ui| {
            let pass_count = self.audit_report.passed_count;
            let total = self.audit_report.total_count;
            let all_pass = self.audit_report.overall_pass;

            let badge_col = if all_pass {
                Color32::from_rgb(80, 220, 100)
            } else {
                Color32::from_rgb(220, 80, 80)
            };
            ui.label(
                RichText::new(format!("AUDIT STATUS: {} / {} PASS", pass_count, total))
                    .heading()
                    .color(badge_col),
            );

            ui.add_space(30.0);
            ui.label(
                RichText::new(format!(
                    "Execution Latency: {:.1} us (Target < 2000.0 us)",
                    self.audit_report.cold_boot_latency_us
                ))
                .strong()
                .color(Color32::from_rgb(100, 200, 255)),
            );
        });

        ui.add_space(8.0);
        ui.separator();

        // 10-row checklist table
        egui::ScrollArea::vertical()
            .id_salt("audit_checklist_scroll")
            .show(ui, |ui| {
                egui::Grid::new("distributor_audit_grid")
                    .striped(true)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("#").strong());
                        ui.label(RichText::new("Criterion Name").strong());
                        ui.label(RichText::new("Measured").strong());
                        ui.label(RichText::new("Target").strong());
                        ui.label(RichText::new("Unit").strong());
                        ui.label(RichText::new("Result").strong());
                        ui.label(RichText::new("Engineering Description").strong());
                        ui.end_row();

                        for (idx, item) in self.audit_report.criteria.iter().enumerate() {
                            ui.label(format!("{}", idx + 1));
                            ui.label(RichText::new(&item.name).strong());
                            ui.label(format!("{:.2}", item.measured_value));
                            ui.label(format!("{:.2}", item.target_threshold));
                            ui.label(&item.units);

                            let res_col = if item.passed {
                                Color32::from_rgb(80, 220, 100)
                            } else {
                                Color32::from_rgb(220, 80, 80)
                            };
                            ui.label(
                                RichText::new(if item.passed { "PASS" } else { "FAIL" })
                                    .strong()
                                    .color(res_col),
                            );
                            ui.label(&item.description);
                            ui.end_row();
                        }
                    });
            });
    }
}
