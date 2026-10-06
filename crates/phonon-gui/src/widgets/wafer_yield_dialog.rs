#![deny(unsafe_code)]

//! Interactive Wafer-Scale Yield, Spatially Correlated Process Variations & Harvesting Economics CAD Dialog.
//!
//! Provides a 5-tab CAD manufacturing and wafer yield optimization environment:
//! 1. 300mm Wafer Map & Spatial Variations (die placement, boundary containment, V_th / L_g fields & SKU coloring).
//! 2. Defect Yield Models (Poisson vs Murphy vs Seeds vs Negative Binomial Stapper curves).
//! 3. Radial Defect Density Profile (CMP / edge exclusion D_0(r) quadratic/quartic rise).
//! 4. Multi-Core Chiplet Harvesting (16-Core Flagship, 12-Core Harvested, 8-Core Salvage, Scrap).
//! 5. Wafer Economics & Gross Margins (wafer fab cost, test/packaging, gross revenue, uplift & margin %).

use egui::{Color32, Context, Pos2, Rect, RichText, Stroke, StrokeKind, Ui, Vec2, Window};
use egui_plot::{Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::wafer_yield::{
    calculate_analytical_gross_dpw, HarvestSkuTier, WaferYieldCoSimulator,
    WaferYieldTelemetryReport, YieldCurvePoint,
};

/// Active tab in the Wafer Yield CAD Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaferYieldTab {
    WaferMap,
    DefectYieldModels,
    RadialDefectProfile,
    ChipletHarvesting,
    WaferEconomics,
}

/// Color representation mode for individual dies on the 300mm wafer map canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaferMapColorMode {
    HarvestSku,
    ThresholdVoltage,
    GateLength,
}

/// Modal dialog for Wafer-Scale Yield, Process Variations & Harvesting Co-Simulation.
pub struct WaferYieldDialog {
    pub is_open: bool,
    pub active_tab: WaferYieldTab,
    pub color_mode: WaferMapColorMode,

    // Core co-simulator
    pub sim: WaferYieldCoSimulator,

    // Cached telemetry report for instant zero-latency UI rendering
    pub cached_report: WaferYieldTelemetryReport,
    pub cached_curves: Vec<YieldCurvePoint>,
}

impl Default for WaferYieldDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl WaferYieldDialog {
    /// Instant non-blocking constructor ensuring sub-microsecond cold boot latency.
    pub fn new_fast() -> Self {
        let sim = WaferYieldCoSimulator::new_fast();

        // Baseline pre-seeded yield curves (15 points across 0.1 to 3.5 cm^2)
        let mut cached_curves = Vec::with_capacity(15);
        for i in 0..15 {
            let a = 0.1 + i as f64 * (3.4 / 14.0);
            let d0 = 0.12;
            let a_crit = a * 0.75;
            let y_pois = (-d0 * a_crit).exp();
            let y_murp = if d0 * a_crit < 1e-4 {
                1.0 - d0 * a_crit
            } else {
                let term = (1.0 - (-d0 * a_crit).exp()) / (d0 * a_crit);
                term * term
            };
            let y_seeds = 1.0 / (1.0 + d0 * a_crit);
            let y_neg_bin = (1.0 + d0 * a_crit / 2.0).powf(-2.0);
            cached_curves.push(YieldCurvePoint {
                die_area_cm2: a,
                poisson_yield: y_pois,
                murphy_yield: y_murp,
                seeds_yield: y_seeds,
                neg_bin_yield: y_neg_bin,
            });
        }

        let cached_report = WaferYieldTelemetryReport {
            gross_dpw: 486,
            analytical_gross_dpw: 494,
            total_good_dies: 381,
            functional_yield_pct: 78.4,
            unharvested_yield_pct: 36.2,
            yield_poisson_pct: 32.5,
            yield_murphy_pct: 42.1,
            yield_seeds_pct: 44.8,
            yield_neg_bin_pct: 48.6,
            gross_revenue_usd: 246500.0,
            unharvested_revenue_usd: 149600.0,
            harvesting_revenue_uplift_pct: 64.8,
            total_mfg_cost_usd: 36814.0,
            gross_profit_usd: 209686.0,
            gross_margin_pct: 85.1,
            cost_per_good_die_usd: 96.6,
            count_tier1_flagship: 176,
            count_tier2_harvested: 132,
            count_tier3_salvage: 73,
            count_scrap: 105,
            vth_mean_v: 0.328,
            vth_std_dev_mv: 18.4,
            lg_mean_nm: 15.92,
            lg_std_dev_nm: 0.52,
        };

        Self {
            is_open: false,
            active_tab: WaferYieldTab::WaferMap,
            color_mode: WaferMapColorMode::HarvestSku,
            sim,
            cached_report,
            cached_curves,
        }
    }

    /// Pre-seeds simulator with an instant full run for active studio view.
    pub fn new_with_baseline() -> Self {
        let mut dlg = Self::new_fast();
        dlg.run_solve();
        dlg
    }

    /// Executes full co-simulation solve and updates cached telemetry.
    pub fn run_solve(&mut self) {
        let report = self.sim.run_simulation();
        self.cached_report = report;
        self.cached_curves = self.sim.yield_curves.clone();
    }

    /// Alias for show(ctx) to maintain consistency with other dialog widgets.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the complete 5-tab Wafer Yield & DFM CAD window.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new(RichText::new("Wafer-Scale Yield, DFM & Harvesting Economics").strong())
            .open(&mut is_open)
            .default_size(Vec2::new(960.0, 680.0))
            .min_size(Vec2::new(760.0, 520.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = is_open;
    }

    fn render_contents(&mut self, ui: &mut Ui) {
        // Top action bar
        ui.horizontal(|ui| {
            if ui.button(RichText::new("Run Wafer Yield Simulation").strong()).clicked() {
                self.run_solve();
            }

            ui.separator();

            // Tab selectors
            ui.selectable_value(&mut self.active_tab, WaferYieldTab::WaferMap, "Wafer Map");
            ui.selectable_value(&mut self.active_tab, WaferYieldTab::DefectYieldModels, "Yield Models");
            ui.selectable_value(&mut self.active_tab, WaferYieldTab::RadialDefectProfile, "Radial Defect");
            ui.selectable_value(&mut self.active_tab, WaferYieldTab::ChipletHarvesting, "Chiplet Harvesting");
            ui.selectable_value(&mut self.active_tab, WaferYieldTab::WaferEconomics, "Wafer Economics");
        });

        ui.separator();

        // Telemetry header ribbon
        ui.horizontal(|ui| {
            let r = &self.cached_report;
            ui.label(RichText::new(format!("Gross DPW: {}", r.gross_dpw)).color(Color32::from_rgb(100, 200, 255)));
            ui.label("|");
            ui.label(RichText::new(format!("Functional Yield: {:.1}%", r.functional_yield_pct)).color(Color32::from_rgb(46, 204, 113)));
            ui.label("|");
            ui.label(RichText::new(format!("Unharvested Yield: {:.1}%", r.unharvested_yield_pct)).color(Color32::from_rgb(230, 126, 34)));
            ui.label("|");
            ui.label(RichText::new(format!("Revenue: ${:.0}k", r.gross_revenue_usd / 1000.0)).color(Color32::from_rgb(255, 215, 0)));
            ui.label("|");
            ui.label(RichText::new(format!("Gross Margin: {:.1}%", r.gross_margin_pct)).color(Color32::from_rgb(155, 89, 182)));
            ui.label("|");
            ui.label(RichText::new(format!("Uplift: +{:.1}%", r.harvesting_revenue_uplift_pct)).color(Color32::from_rgb(52, 152, 219)));
        });

        ui.separator();

        // Tab body
        match self.active_tab {
            WaferYieldTab::WaferMap => self.render_wafer_map_tab(ui),
            WaferYieldTab::DefectYieldModels => self.render_defect_yield_models_tab(ui),
            WaferYieldTab::RadialDefectProfile => self.render_radial_defect_profile_tab(ui),
            WaferYieldTab::ChipletHarvesting => self.render_chiplet_harvesting_tab(ui),
            WaferYieldTab::WaferEconomics => self.render_wafer_economics_tab(ui),
        }
    }

    fn render_wafer_map_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(260.0);
                ui.heading("Wafer & Die Geometry");

                let geom = &mut self.sim.geometry;
                ui.add(egui::Slider::new(&mut geom.diameter_mm, 150.0..=450.0).text("Wafer Diameter (mm)"));
                ui.add(egui::Slider::new(&mut geom.edge_exclusion_mm, 1.0..=10.0).text("Edge Exclusion (mm)"));
                ui.add(egui::Slider::new(&mut geom.die_width_mm, 4.0..=35.0).text("Die Width (mm)"));
                ui.add(egui::Slider::new(&mut geom.die_height_mm, 4.0..=35.0).text("Die Height (mm)"));
                ui.add(egui::Slider::new(&mut geom.scribe_street_um, 20.0..=200.0).text("Scribe Street (um)"));

                ui.separator();
                ui.heading("Map Display Mode");
                ui.radio_value(&mut self.color_mode, WaferMapColorMode::HarvestSku, "Harvest SKU Tier");
                ui.radio_value(&mut self.color_mode, WaferMapColorMode::ThresholdVoltage, "Threshold Voltage (V_th)");
                ui.radio_value(&mut self.color_mode, WaferMapColorMode::GateLength, "Gate Length (L_g)");

                ui.separator();
                ui.heading("Geometry Telemetry");
                let a_cm2 = self.sim.geometry.die_area_cm2();
                let r_active = self.sim.geometry.active_radius_mm();
                ui.label(format!("Die Area: {:.2} mm^2 ({:.3} cm^2)", self.sim.geometry.die_area_mm2(), a_cm2));
                ui.label(format!("Active Radius: {:.1} mm", r_active));
                ui.label(format!("Analytical Gross DPW: {}", calculate_analytical_gross_dpw(&self.sim.geometry)));

                ui.separator();
                ui.heading("Legend");
                match self.color_mode {
                    WaferMapColorMode::HarvestSku => {
                        ui.colored_label(Color32::from_rgb(46, 204, 113), "[■] Tier 1: 16-Core Flagship ($850)");
                        ui.colored_label(Color32::from_rgb(52, 152, 219), "[■] Tier 2: 12-Core Harvested ($550)");
                        ui.colored_label(Color32::from_rgb(243, 156, 18), "[■] Tier 3: 8-Core Salvage ($320)");
                        ui.colored_label(Color32::from_rgb(231, 76, 60), "[■] Scrap / Fatal Defect ($0)");
                    }
                    WaferMapColorMode::ThresholdVoltage => {
                        ui.label("V_th variation: Blue (Low) -> Green (Nominal) -> Red (High Edge)");
                    }
                    WaferMapColorMode::GateLength => {
                        ui.label("L_g variation: Blue (Short channel) -> Green -> Red (Long)");
                    }
                }
            });

            ui.separator();

            // 2D Wafer Map Canvas
            ui.vertical(|ui| {
                ui.heading("300mm Silicon Wafer Layout & Placed Die Grid");
                let (response, painter) = ui.allocate_painter(Vec2::new(480.0, 480.0), egui::Sense::hover());
                let rect = response.rect;
                let center = rect.center();
                let wafer_radius_pixels = 220.0_f32;
                let scale = wafer_radius_pixels / self.sim.geometry.wafer_radius_mm() as f32;

                // Draw background substrate circle
                painter.circle_filled(center, wafer_radius_pixels, Color32::from_rgb(30, 34, 42));
                painter.circle_stroke(center, wafer_radius_pixels, Stroke::new(2.0, Color32::from_rgb(90, 100, 115)));

                // Draw notch / flat indicator at bottom
                let notch_pos = Pos2::new(center.x, center.y + wafer_radius_pixels);
                painter.circle_filled(notch_pos, 4.0, Color32::from_rgb(180, 190, 205));

                // Draw edge exclusion ring
                let exclusion_radius_pixels = self.sim.geometry.active_radius_mm() as f32 * scale;
                painter.circle_stroke(center, exclusion_radius_pixels, Stroke::new(1.0, Color32::from_rgb(160, 140, 50)));

                // Draw individual placed dies
                let half_w_px = (self.sim.geometry.die_width_mm as f32 * 0.5) * scale;
                let half_h_px = (self.sim.geometry.die_height_mm as f32 * 0.5) * scale;

                let hovered_pos = response.hover_pos();
                let mut hovered_die_info = None;

                for die in &self.sim.simulated_dies {
                    let cx = center.x + die.pos.center_x_mm as f32 * scale;
                    let cy = center.y - die.pos.center_y_mm as f32 * scale;

                    let die_rect = Rect::from_min_max(
                        Pos2::new(cx - half_w_px, cy - half_h_px),
                        Pos2::new(cx + half_w_px, cy + half_h_px),
                    );

                    let fill_color = match self.color_mode {
                        WaferMapColorMode::HarvestSku => match die.harvest.sku_tier {
                            HarvestSkuTier::Flagship16Core => Color32::from_rgb(46, 204, 113),
                            HarvestSkuTier::Harvested12Core => Color32::from_rgb(52, 152, 219),
                            HarvestSkuTier::Salvage8Core => Color32::from_rgb(243, 156, 18),
                            HarvestSkuTier::Scrap => Color32::from_rgb(231, 76, 60),
                        },
                        WaferMapColorMode::ThresholdVoltage => {
                            let delta = die.process.vth_delta_pct.clamp(-10.0, 10.0);
                            if delta < 0.0 {
                                let t = (-delta / 10.0) as f32;
                                Color32::from_rgb(
                                    (46.0 * (1.0 - t) + 41.0 * t) as u8,
                                    (204.0 * (1.0 - t) + 128.0 * t) as u8,
                                    (113.0 * (1.0 - t) + 185.0 * t) as u8,
                                )
                            } else {
                                let t = (delta / 10.0) as f32;
                                Color32::from_rgb(
                                    (46.0 * (1.0 - t) + 231.0 * t) as u8,
                                    (204.0 * (1.0 - t) + 76.0 * t) as u8,
                                    (113.0 * (1.0 - t) + 60.0 * t) as u8,
                                )
                            }
                        }
                        WaferMapColorMode::GateLength => {
                            let delta = die.process.lg_delta_pct.clamp(-6.0, 6.0);
                            let t = ((delta + 6.0) / 12.0) as f32;
                            Color32::from_rgb(
                                (50.0 + 180.0 * t) as u8,
                                (180.0 - 60.0 * t) as u8,
                                (220.0 - 150.0 * t) as u8,
                            )
                        }
                    };

                    painter.rect_filled(die_rect, 1.0, fill_color);
                    painter.rect_stroke(die_rect, 1.0, Stroke::new(0.5, Color32::from_rgb(20, 24, 30)), StrokeKind::Inside);

                    // Check hover
                    if let Some(hpos) = hovered_pos {
                        if die_rect.contains(hpos) {
                            hovered_die_info = Some(die.clone());
                            // Draw highlight border
                            painter.rect_stroke(die_rect, 1.0, Stroke::new(1.5, Color32::WHITE), StrokeKind::Inside);
                        }
                    }
                }

                // Hover tooltip display
                if let Some(hdie) = hovered_die_info {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("Die [{}, {}]:", hdie.pos.col, hdie.pos.row)).strong());
                            ui.label(format!("r = {:.1} mm", hdie.pos.distance_from_center_mm));
                            ui.label(format!("SKU: {}", hdie.harvest.sku_tier.display_name()));
                            ui.label(format!("Defects: (Core: {}, Uncore: {})", hdie.harvest.core_defects, hdie.harvest.uncore_defects));
                            ui.label(format!("V_th: {:.3} V ({:+.1}%)", hdie.process.vth_v, hdie.process.vth_delta_pct));
                            ui.label(format!("L_g: {:.2} nm", hdie.process.lg_nm));
                        });
                    });
                } else {
                    ui.label("Hover cursor over any placed die to inspect local parameters and defect count.");
                }
            });
        });
    }

    fn render_defect_yield_models_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(260.0);
                ui.heading("Yield Model Parameters");

                let def = &mut self.sim.defect;
                ui.add(egui::Slider::new(&mut def.d0_center_cm2, 0.02..=0.60).text("D_0 Center (def/cm^2)"));
                ui.add(egui::Slider::new(&mut def.cluster_alpha, 0.2..=5.0).text("Cluster Alpha (Negative Binomial)"));
                ui.add(egui::Slider::new(&mut def.critical_area_factor, 0.4..=1.0).text("Critical Area Factor"));

                ui.separator();
                ui.heading("Yield Formulations");
                ui.label(RichText::new("Poisson Model:").strong());
                ui.label("Y = exp(-D_0 * A)");
                ui.label("Assumes pure random defects without clustering.");

                ui.add_space(4.0);
                ui.label(RichText::new("Murphy Model:").strong());
                ui.label("Y = ((1 - exp(-D_0 * A)) / (D_0 * A))^2");
                ui.label("Triangular defect density distribution.");

                ui.add_space(4.0);
                ui.label(RichText::new("Seeds Model:").strong());
                ui.label("Y = 1 / (1 + D_0 * A)");
                ui.label("Empirical hyperbolic yield curve.");

                ui.add_space(4.0);
                ui.label(RichText::new("Negative Binomial (Stapper):").strong());
                ui.label("Y = (1 + D_0 * A / alpha)^(-alpha)");
                ui.label("Gamma-distributed clustering. Industry standard for modern multi-layer chips.");
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Yield vs Die Area (cm^2) Comparison");

                let die_area = self.sim.geometry.die_area_cm2();
                let pts_poisson: Vec<[f64; 2]> = self.cached_curves.iter().map(|p| [p.die_area_cm2, p.poisson_yield * 100.0]).collect();
                let pts_murphy: Vec<[f64; 2]> = self.cached_curves.iter().map(|p| [p.die_area_cm2, p.murphy_yield * 100.0]).collect();
                let pts_seeds: Vec<[f64; 2]> = self.cached_curves.iter().map(|p| [p.die_area_cm2, p.seeds_yield * 100.0]).collect();
                let pts_neg_bin: Vec<[f64; 2]> = self.cached_curves.iter().map(|p| [p.die_area_cm2, p.neg_bin_yield * 100.0]).collect();

                let line_poisson = Line::new("Poisson", PlotPoints::from(pts_poisson))
                    .color(Color32::from_rgb(231, 76, 60))
                    .stroke(Stroke::new(2.0, Color32::from_rgb(231, 76, 60)));
                let line_murphy = Line::new("Murphy", PlotPoints::from(pts_murphy))
                    .color(Color32::from_rgb(243, 156, 18))
                    .stroke(Stroke::new(2.0, Color32::from_rgb(243, 156, 18)));
                let line_seeds = Line::new("Seeds", PlotPoints::from(pts_seeds))
                    .color(Color32::from_rgb(52, 152, 219))
                    .stroke(Stroke::new(2.0, Color32::from_rgb(52, 152, 219)));
                let line_neg_bin = Line::new("Negative Binomial (Stapper)", PlotPoints::from(pts_neg_bin))
                    .color(Color32::from_rgb(46, 204, 113))
                    .stroke(Stroke::new(2.5, Color32::from_rgb(46, 204, 113)));

                Plot::new("defect_yield_plot")
                    .height(380.0)
                    .x_axis_label("Die Area (cm^2)")
                    .y_axis_label("Functional Yield (%)")
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(line_poisson);
                        plot_ui.line(line_murphy);
                        plot_ui.line(line_seeds);
                        plot_ui.line(line_neg_bin);

                        // Vertical marker at current die area
                        plot_ui.vline(VLine::new("Active Die Area", die_area).stroke(Stroke::new(1.5, Color32::from_rgb(255, 255, 100))));
                    });

                // Yield at current die area summary
                let r = &self.cached_report;
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("At Area = {:.2} cm^2:", die_area)).strong());
                    ui.label(format!("Poisson: {:.1}%", r.yield_poisson_pct));
                    ui.label(format!("Murphy: {:.1}%", r.yield_murphy_pct));
                    ui.label(format!("Seeds: {:.1}%", r.yield_seeds_pct));
                    ui.label(RichText::new(format!("Negative Binomial: {:.1}%", r.yield_neg_bin_pct)).strong().color(Color32::from_rgb(46, 204, 113)));
                });
            });
        });
    }

    fn render_radial_defect_profile_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(260.0);
                ui.heading("Radial Defect Parameters");

                let def = &mut self.sim.defect;
                ui.add(egui::Slider::new(&mut def.d0_center_cm2, 0.02..=0.60).text("Center D_0 (def/cm^2)"));
                ui.add(egui::Slider::new(&mut def.kappa_edge, 0.5..=15.0).text("Edge Escalation Multiplier"));
                ui.add(egui::Slider::new(&mut def.edge_exponent, 1.0..=8.0).text("Polynomial Exponent"));

                ui.separator();
                ui.heading("Physical Mechanisms");
                ui.label("Modern 300mm wafer processes exhibit sharp defect escalation near wafer bevel:");
                ui.label("- Resist edge-bead removal (EBR) solvent splash");
                ui.label("- CMP slurry particle accumulation & uneven polish pad pressure");
                ui.label("- Plasma sheath distortion & chuck clamp electrostatic fields");
                ui.label("- Gas flow eddy currents in deposition chambers");
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Defect Density Profile D_0(r) across Wafer Radius");

                let r_active = self.sim.geometry.active_radius_mm();
                let r_wafer = self.sim.geometry.wafer_radius_mm();
                let steps = 40;
                let mut pts_profile = Vec::with_capacity(steps);

                for i in 0..=steps {
                    let r = (i as f64 / steps as f64) * r_wafer;
                    let d = self.sim.defect.defect_density_at_radius(r, r_active);
                    pts_profile.push([r, d]);
                }

                let line_d0 = Line::new("D_0(r)", PlotPoints::from(pts_profile))
                    .color(Color32::from_rgb(231, 76, 60))
                    .stroke(Stroke::new(2.5, Color32::from_rgb(231, 76, 60)));

                Plot::new("radial_defect_plot")
                    .height(380.0)
                    .x_axis_label("Radial Position r (mm)")
                    .y_axis_label("Defect Density D_0 (defects/cm^2)")
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(line_d0);
                        plot_ui.vline(VLine::new("Active Wafer Boundary", r_active).stroke(Stroke::new(1.5, Color32::from_rgb(243, 156, 18))));
                        plot_ui.vline(VLine::new("Wafer Perimeter (300mm)", r_wafer).stroke(Stroke::new(1.0, Color32::from_rgb(150, 150, 150))));
                    });

                let d_center = self.sim.defect.defect_density_at_radius(0.0, r_active);
                let d_edge = self.sim.defect.defect_density_at_radius(r_active, r_active);
                ui.horizontal(|ui| {
                    ui.label(format!("Center D_0: {:.3} def/cm^2", d_center));
                    ui.label("|");
                    ui.label(format!("Edge D_0 (r = {:.1} mm): {:.3} def/cm^2", r_active, d_edge));
                    ui.label("|");
                    ui.label(format!("Ratio: {:.1}x", d_edge / d_center.max(1e-6)));
                });
            });
        });
    }

    fn render_chiplet_harvesting_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(280.0);
                ui.heading("Architecture & Harvesting Rules");

                let arch = &mut self.sim.arch;
                ui.add(egui::Slider::new(&mut arch.uncore_area_fraction, 0.10..=0.45).text("Uncore Area Fraction"));
                ui.add(egui::Slider::new(&mut arch.price_tier1_flagship_usd, 300.0..=2500.0).text("Flagship Price ($)"));
                ui.add(egui::Slider::new(&mut arch.price_tier2_harvested_usd, 200.0..=1800.0).text("Harvested Price ($)"));
                ui.add(egui::Slider::new(&mut arch.price_tier3_salvage_usd, 100.0..=1200.0).text("Salvage Price ($)"));

                ui.separator();
                ui.heading("Harvesting Logic");
                ui.label(RichText::new("Tier 1: 16-Core Flagship").color(Color32::from_rgb(46, 204, 113)));
                ui.label("0 fatal defects anywhere on die. 100% active cores.");

                ui.add_space(4.0);
                ui.label(RichText::new("Tier 2: 12-Core Harvested").color(Color32::from_rgb(52, 152, 219)));
                ui.label("1-2 core defects. 4 cores disabled via on-chip laser fuses.");

                ui.add_space(4.0);
                ui.label(RichText::new("Tier 3: 8-Core Salvage").color(Color32::from_rgb(243, 156, 18)));
                ui.label("3-4 core defects. 8 cores disabled. Mid-range SKU.");

                ui.add_space(4.0);
                ui.label(RichText::new("Scrap / Rejection").color(Color32::from_rgb(231, 76, 60)));
                ui.label("> 4 core defects OR fatal defect in uncore root.");
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Harvested SKU Distribution Breakdown");
                let r = &self.cached_report;

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(46, 204, 113), RichText::new("Tier 1: 16-Core Flagship").strong());
                        ui.label(format!("Count: {} dies ({:.1}%)", r.count_tier1_flagship, (r.count_tier1_flagship as f64 / r.gross_dpw.max(1) as f64) * 100.0));
                        ui.label(format!("Revenue: ${:.0}k", (r.count_tier1_flagship as f64 * self.sim.arch.price_tier1_flagship_usd) / 1000.0));
                    });
                });

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(52, 152, 219), RichText::new("Tier 2: 12-Core Harvested").strong());
                        ui.label(format!("Count: {} dies ({:.1}%)", r.count_tier2_harvested, (r.count_tier2_harvested as f64 / r.gross_dpw.max(1) as f64) * 100.0));
                        ui.label(format!("Revenue: ${:.0}k", (r.count_tier2_harvested as f64 * self.sim.arch.price_tier2_harvested_usd) / 1000.0));
                    });
                });

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(243, 156, 18), RichText::new("Tier 3: 8-Core Salvage").strong());
                        ui.label(format!("Count: {} dies ({:.1}%)", r.count_tier3_salvage, (r.count_tier3_salvage as f64 / r.gross_dpw.max(1) as f64) * 100.0));
                        ui.label(format!("Revenue: ${:.0}k", (r.count_tier3_salvage as f64 * self.sim.arch.price_tier3_salvage_usd) / 1000.0));
                    });
                });

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(231, 76, 60), RichText::new("Scrap / Fatal Defects").strong());
                        ui.label(format!("Count: {} dies ({:.1}%)", r.count_scrap, (r.count_scrap as f64 / r.gross_dpw.max(1) as f64) * 100.0));
                        ui.label("Revenue: $0");
                    });
                });

                ui.separator();
                ui.heading("Yield Comparison: Single-Die vs Multi-Tier Harvesting");
                ui.horizontal(|ui| {
                    ui.label(format!("Standard (Unharvested) Yield: {:.1}%", r.unharvested_yield_pct));
                    ui.label("->");
                    ui.label(RichText::new(format!("Functional Harvested Yield: {:.1}%", r.functional_yield_pct)).strong().color(Color32::from_rgb(46, 204, 113)));
                    ui.label(format!("(+{:.1}% Yield Recovery)", r.functional_yield_pct - r.unharvested_yield_pct));
                });
            });
        });
    }

    fn render_wafer_economics_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(280.0);
                ui.heading("Wafer & Packaging Costs");

                let econ = &mut self.sim.economics;
                ui.add(egui::Slider::new(&mut econ.wafer_fab_cost_usd, 5000.0..=30000.0).text("Wafer Fab Cost ($)"));
                ui.add(egui::Slider::new(&mut econ.probe_test_cost_per_die_usd, 2.0..=50.0).text("Probe Test / Die ($)"));
                ui.add(egui::Slider::new(&mut econ.packaging_cost_per_good_die_usd, 10.0..=150.0).text("Pkg & Assembly / Die ($)"));

                ui.separator();
                ui.heading("Financial Formulas");
                ui.label("Total Cost = C_wafer + (DPW * C_test) + (Good * C_pkg)");
                ui.label("Gross Revenue = N_1 * P_1 + N_2 * P_2 + N_3 * P_3");
                ui.label("Gross Profit = Revenue - Total Cost");
                ui.label("Gross Margin = (Profit / Revenue) * 100%");
                ui.label("Uplift = (Rev_harvest - Rev_flagship) / Rev_flagship");
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Wafer Fabrication & Harvesting P&L Statement");
                let r = &self.cached_report;

                egui::Grid::new("economics_grid")
                    .striped(true)
                    .min_col_width(180.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Line Item").strong());
                        ui.label(RichText::new("Amount (USD)").strong());
                        ui.end_row();

                        ui.label("Wafer Fabrication Cost (EUV 300mm):");
                        ui.label(format!("${:.2}", self.sim.economics.wafer_fab_cost_usd));
                        ui.end_row();

                        ui.label(format!("Wafer Sort / Probe Test ({} dies):", r.gross_dpw));
                        ui.label(format!("${:.2}", r.gross_dpw as f64 * self.sim.economics.probe_test_cost_per_die_usd));
                        ui.end_row();

                        let good_dies = r.count_tier1_flagship + r.count_tier2_harvested + r.count_tier3_salvage;
                        ui.label(format!("Packaging & Final Test ({} marketable dies):", good_dies));
                        ui.label(format!("${:.2}", good_dies as f64 * self.sim.economics.packaging_cost_per_good_die_usd));
                        ui.end_row();

                        ui.label(RichText::new("Total Manufacturing Cost:").strong());
                        ui.label(RichText::new(format!("${:.2}", r.total_mfg_cost_usd)).strong().color(Color32::from_rgb(231, 76, 60)));
                        ui.end_row();

                        ui.label("Unharvested Revenue (Flagship only):");
                        ui.label(format!("${:.2}", r.unharvested_revenue_usd));
                        ui.end_row();

                        ui.label(RichText::new("Total Harvested Gross Revenue:").strong());
                        ui.label(RichText::new(format!("${:.2}", r.gross_revenue_usd)).strong().color(Color32::from_rgb(46, 204, 113)));
                        ui.end_row();

                        ui.label(RichText::new("Harvesting Revenue Uplift:").strong());
                        ui.label(RichText::new(format!("+${:.2} (+{:.1}%)", r.gross_revenue_usd - r.unharvested_revenue_usd, r.harvesting_revenue_uplift_pct)).strong().color(Color32::from_rgb(52, 152, 219)));
                        ui.end_row();

                        ui.label(RichText::new("Gross Profit per Wafer:").strong());
                        ui.label(RichText::new(format!("${:.2}", r.gross_profit_usd)).strong().color(Color32::from_rgb(255, 215, 0)));
                        ui.end_row();

                        ui.label(RichText::new("Wafer Gross Margin:").strong());
                        ui.label(RichText::new(format!("{:.1}%", r.gross_margin_pct)).strong().color(Color32::from_rgb(155, 89, 182)));
                        ui.end_row();

                        ui.label(RichText::new("Effective Cost per Good Die:").strong());
                        ui.label(RichText::new(format!("${:.2} / die", r.cost_per_good_die_usd)).strong());
                        ui.end_row();
                    });
            });
        });
    }
}
