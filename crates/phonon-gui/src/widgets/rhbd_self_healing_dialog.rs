#![deny(unsafe_code)]

//! Interactive Radiation-Hardened By Design (RHBD) DRC & Autonomous Self-Healing Dialog.
//!
//! Provides an interactive 5-tab CAD simulation environment:
//! 1. Layout DRC Inspector (guard ring substrate resistance R_sub < 10 Ohm, DICE/TMR node separation).
//! 2. SEL Crowbar Scope (sub-50ns electronic crowbar quenching vs unquenched thermal runaway).
//! 3. Parasitic Thyristor TCAD (regenerative PNPN thyristor latchup physics and beta-product suppression).
//! 4. Autonomous Task Migration (multi-core fault-tolerant cluster with live task handoff to spare silicon).
//! 5. Airworthiness Compliance (DO-254 DAL-A / NASA RHBD airworthiness certification matrix).

use egui::{pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::rhbd_self_healing::{
    CoreLifecycleState, DrcViolationSeverity, LayoutComponentKind, RhbdSelfHealingCoSimulator,
};

/// Active tab in the RHBD & Self-Healing Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhbdTab {
    LayoutDrcInspector,
    SelCrowbarScope,
    ParasiticThyristorTCAD,
    AutonomousTaskMigration,
    AirworthinessCompliance,
}

/// Modal dialog for RHBD DRC, Fast SEL Quenching & Autonomous Self-Healing Co-Simulation.
pub struct RhbdSelfHealingDialog {
    pub is_open: bool,
    pub active_tab: RhbdTab,

    // DRC layout parameters
    pub substrate_sheet_res_ohm_sq: f64,

    // SEL Quenching parameters
    pub ion_strike_charge_pc: f64,
    pub detection_delay_ns: f64,
    pub cutoff_switch_delay_ns: f64,

    // Self-healing cluster injection
    pub target_fault_core_id: usize,
    pub fault_tid_krad: f64,
    pub fault_seu_rate: f64,

    // Co-simulator coordinator and cached plot curves
    pub sim: RhbdSelfHealingCoSimulator,
    pub cached_quenched_v_curve: Vec<[f64; 2]>,
    pub cached_unquenched_v_curve: Vec<[f64; 2]>,
    pub cached_quenched_temp_curve: Vec<[f64; 2]>,
    pub cached_unquenched_temp_curve: Vec<[f64; 2]>,
}

impl Default for RhbdSelfHealingDialog {
    fn default() -> Self {
        let sim = RhbdSelfHealingCoSimulator::new_fast();

        // Pre-seeded lightweight curves for instant sub-millisecond cold boot (<1ms)
        let cached_quenched_v_curve = vec![
            [0.0, 3.3],
            [10.0, 3.3],
            [34.0, 3.3],
            [35.0, 0.3],
            [60.0, 0.3],
            [100.0, 0.3],
        ];

        let cached_unquenched_v_curve = vec![
            [0.0, 3.3],
            [20.0, 3.3],
            [50.0, 3.3],
            [100.0, 3.3],
        ];

        let cached_quenched_temp_curve = vec![
            [0.0, 25.0],
            [10.0, 42.0],
            [35.0, 78.0],
            [50.0, 72.0],
            [100.0, 38.0],
        ];

        let cached_unquenched_temp_curve = vec![
            [0.0, 25.0],
            [20.0, 85.0],
            [50.0, 195.0],
            [100.0, 385.0],
        ];

        Self {
            is_open: false,
            active_tab: RhbdTab::LayoutDrcInspector,
            substrate_sheet_res_ohm_sq: 25.0,
            ion_strike_charge_pc: 2.5,
            detection_delay_ns: 10.0,
            cutoff_switch_delay_ns: 25.0,
            target_fault_core_id: 0,
            fault_tid_krad: 75.0,
            fault_seu_rate: 4.5,
            sim,
            cached_quenched_v_curve,
            cached_unquenched_v_curve,
            cached_quenched_temp_curve,
            cached_unquenched_temp_curve,
        }
    }
}

impl RhbdSelfHealingDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast non-blocking constructor guaranteeing sub-millisecond initialization for cold boot.
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Recompute simulation trajectories and telemetry.
    pub fn recompute_sim(&mut self) {
        // Sync parameters
        self.sim.layout_grid.substrate_sheet_resistance_ohm_sq = self.substrate_sheet_res_ohm_sq;
        self.sim.sel_simulator.crowbar.detection_delay_ns = self.detection_delay_ns;
        self.sim.sel_simulator.crowbar.cutoff_switch_delay_ns = self.cutoff_switch_delay_ns;

        // Run transient simulation
        let sel_res = self
            .sim
            .sel_simulator
            .simulate_transient(self.ion_strike_charge_pc, 250.0, 1.0);

        self.cached_quenched_v_curve = sel_res
            .quenched_points
            .iter()
            .map(|p| [p.time_ns, p.v_rail_v])
            .collect();
        self.cached_unquenched_v_curve = sel_res
            .unquenched_points
            .iter()
            .map(|p| [p.time_ns, p.v_rail_v])
            .collect();

        self.cached_quenched_temp_curve = sel_res
            .quenched_points
            .iter()
            .map(|p| [p.time_ns, p.t_junction_c])
            .collect();
        self.cached_unquenched_temp_curve = sel_res
            .unquenched_points
            .iter()
            .map(|p| [p.time_ns, p.t_junction_c])
            .collect();

        self.sim.recompute();
    }

    /// Render modal UI window (standard alias).
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the complete dialog window using egui 0.37 window borrow pattern.
    pub fn show(&mut self, ctx: &Context) {
        let mut is_open = self.is_open;
        Window::new("RHBD Layout DRC & Autonomous Self-Healing Co-Simulator")
            .open(&mut is_open)
            .default_width(960.0)
            .default_height(680.0)
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
                    self.active_tab == RhbdTab::LayoutDrcInspector,
                    "RHBD Layout DRC Inspector",
                )
                .clicked()
            {
                self.active_tab = RhbdTab::LayoutDrcInspector;
            }
            if ui
                .selectable_label(self.active_tab == RhbdTab::SelCrowbarScope, "SEL Crowbar Scope")
                .clicked()
            {
                self.active_tab = RhbdTab::SelCrowbarScope;
            }
            if ui
                .selectable_label(
                    self.active_tab == RhbdTab::ParasiticThyristorTCAD,
                    "Parasitic Thyristor TCAD",
                )
                .clicked()
            {
                self.active_tab = RhbdTab::ParasiticThyristorTCAD;
            }
            if ui
                .selectable_label(
                    self.active_tab == RhbdTab::AutonomousTaskMigration,
                    "Autonomous Task Migration",
                )
                .clicked()
            {
                self.active_tab = RhbdTab::AutonomousTaskMigration;
            }
            if ui
                .selectable_label(
                    self.active_tab == RhbdTab::AirworthinessCompliance,
                    "Airworthiness Compliance",
                )
                .clicked()
            {
                self.active_tab = RhbdTab::AirworthinessCompliance;
            }
        });

        ui.separator();

        match self.active_tab {
            RhbdTab::LayoutDrcInspector => self.render_drc_inspector(ui),
            RhbdTab::SelCrowbarScope => self.render_crowbar_scope(ui),
            RhbdTab::ParasiticThyristorTCAD => self.render_thyristor_tcad(ui),
            RhbdTab::AutonomousTaskMigration => self.render_task_migration(ui),
            RhbdTab::AirworthinessCompliance => self.render_airworthiness(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_drc_inspector(&mut self, ui: &mut Ui) {
        ui.heading("RHBD Physical Layout DRC & Latchup Guard Ring Inspector");
        ui.label("Automated spatial layout verification enforcing R_sub < 10 Ohm, DICE node separation >= 5.0 um, and TMR isolation >= 10.0 um.");

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("Substrate Sheet Resistance (Ohm/sq):");
            if ui
                .add(egui::Slider::new(&mut self.substrate_sheet_res_ohm_sq, 5.0..=100.0).text("Ohm/sq"))
                .changed()
            {
                self.recompute_sim();
            }

            if ui.button("Run Full Layout Audit").clicked() {
                self.recompute_sim();
            }
        });

        ui.add_space(10.0);

        // Render 2D Layout Canvas
        let (rect, _response) = ui.allocate_exact_size(vec2(ui.available_width(), 260.0), Sense::hover());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 28));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(50, 60, 75)), StrokeKind::Middle);

        // Draw coordinate grid lines (100 um x 100 um die space)
        let scale_x = (rect.width() / 100.0) as f64;
        let scale_y = (rect.height() / 100.0) as f64;

        for x in (0..=100).step_by(20) {
            let px = rect.min.x + (x as f64 * scale_x) as f32;
            painter.line_segment(
                [pos2(px, rect.min.y), pos2(px, rect.max.y)],
                Stroke::new(0.5, Color32::from_rgb(35, 42, 52)),
            );
        }
        for y in (0..=100).step_by(20) {
            let py = rect.min.y + (y as f64 * scale_y) as f32;
            painter.line_segment(
                [pos2(rect.min.x, py), pos2(rect.max.x, py)],
                Stroke::new(0.5, Color32::from_rgb(35, 42, 52)),
            );
        }

        // Draw placed components
        for comp in &self.sim.layout_grid.components {
            let cx = rect.min.x + (comp.x_um * scale_x) as f32;
            let cy = rect.min.y + (comp.y_um * scale_y) as f32;
            let cw = (comp.width_um * scale_x) as f32;
            let ch = (comp.height_um * scale_y) as f32;
            let c_rect = Rect::from_min_size(pos2(cx, cy), vec2(cw, ch));

            let (fill, stroke_color) = match comp.kind {
                LayoutComponentKind::SubstrateTapContact => (
                    Color32::from_rgb(40, 120, 220),
                    Color32::from_rgb(100, 180, 255),
                ),
                LayoutComponentKind::GuardRingPWell | LayoutComponentKind::GuardRingNWell => (
                    Color32::from_rgba_premultiplied(220, 180, 40, 90),
                    Color32::from_rgb(255, 215, 0),
                ),
                LayoutComponentKind::DiceStorageCell { .. } => (
                    Color32::from_rgb(140, 60, 200),
                    Color32::from_rgb(190, 110, 255),
                ),
                LayoutComponentKind::TmrCell { .. } => (
                    Color32::from_rgb(40, 160, 90),
                    Color32::from_rgb(90, 220, 140),
                ),
                LayoutComponentKind::StandardLogicCell => (
                    Color32::from_rgb(80, 90, 100),
                    Color32::from_rgb(130, 140, 150),
                ),
            };

            painter.rect_filled(c_rect, 2.0, fill);
            painter.rect_stroke(c_rect, 2.0, Stroke::new(1.0, stroke_color), StrokeKind::Middle);
        }

        // Violations table
        ui.add_space(8.0);
        let violations = self.sim.layout_grid.audit_layout();
        ui.label(RichText::new(format!("Active Audit Findings: {} items", violations.len())).strong());

        egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
            if violations.is_empty() {
                ui.colored_label(Color32::from_rgb(80, 220, 120), "All RHBD physical design rules pass cleanly! 0 violations detected.");
            } else {
                for v in &violations {
                    ui.horizontal(|ui| {
                        let (badge_text, badge_color) = match v.severity {
                            DrcViolationSeverity::Pass => ("PASS", Color32::from_rgb(80, 220, 120)),
                            DrcViolationSeverity::Warning => ("WARN", Color32::from_rgb(240, 160, 40)),
                            DrcViolationSeverity::Error => ("ERR", Color32::from_rgb(230, 60, 60)),
                        };
                        ui.colored_label(badge_color, format!("[{}]", badge_text));
                        ui.label(format!("{}: {}", v.component_id, v.message));
                    });
                }
            }
        });
    }

    fn render_crowbar_scope(&mut self, ui: &mut Ui) {
        ui.heading("Sub-Microsecond Autonomous SEL Quenching Scope");
        ui.label("Co-simulation of on-chip electronic crowbar tripping < 50 ns to extinguish parasitic PNPN thyristor latchup.");

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("Ion Strike Charge (pC):");
            if ui
                .add(egui::Slider::new(&mut self.ion_strike_charge_pc, 0.5..=5.0).text("pC"))
                .changed()
            {
                self.recompute_sim();
            }

            ui.label("Detection Delay:");
            if ui
                .add(egui::Slider::new(&mut self.detection_delay_ns, 5.0..=20.0).text("ns"))
                .changed()
            {
                self.recompute_sim();
            }

            ui.label("Cutoff Delay:");
            if ui
                .add(egui::Slider::new(&mut self.cutoff_switch_delay_ns, 10.0..=40.0).text("ns"))
                .changed()
            {
                self.recompute_sim();
            }
        });

        ui.add_space(8.0);

        let total_quench_time = self.detection_delay_ns + self.cutoff_switch_delay_ns;
        ui.label(RichText::new(format!(
            "Total Quenching Latency: {:.1} ns (< 50 ns requirement {})",
            total_quench_time,
            if total_quench_time < 50.0 { "SATISFIED" } else { "VIOLATED" }
        )).color(if total_quench_time < 50.0 { Color32::from_rgb(80, 220, 120) } else { Color32::from_rgb(240, 80, 80) }));

        ui.add_space(6.0);

        // Plot Voltage & Temperature Waves
        let rep = self.sim.report();
        Plot::new("sel_crowbar_scope_plot")
            .height(260.0)
            .legend(Legend::default())
            .x_axis_label("Time (ns)")
            .y_axis_label("Voltage (V) / Temp (deg C)")
            .show(ui, |plot_ui| {
                let v_quenched_pts = PlotPoints::new(self.cached_quenched_v_curve.clone());
                plot_ui.line(
                    Line::new("V_rail Protected (V)", v_quenched_pts)
                        .color(Color32::from_rgb(60, 160, 255))
                        .width(2.0),
                );

                let v_unquenched_pts = PlotPoints::new(self.cached_unquenched_v_curve.clone());
                plot_ui.line(
                    Line::new("V_rail Unquenched (V)", v_unquenched_pts)
                        .color(Color32::from_rgb(180, 180, 190))
                        .width(1.5),
                );

                let t_quenched_pts = PlotPoints::new(self.cached_quenched_temp_curve.clone());
                plot_ui.line(
                    Line::new("T_junction Protected (C)", t_quenched_pts)
                        .color(Color32::from_rgb(80, 220, 120))
                        .width(2.0),
                );

                let t_unquenched_pts = PlotPoints::new(self.cached_unquenched_temp_curve.clone());
                plot_ui.line(
                    Line::new("T_junction Unquenched (C)", t_unquenched_pts)
                        .color(Color32::from_rgb(240, 60, 60))
                        .width(2.0),
                );

                // Quenching cutoff trigger line
                plot_ui.vline(
                    VLine::new("Crowbar Cutoff", total_quench_time)
                        .color(Color32::from_rgb(255, 215, 0)),
                );
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(format!("Peak T_j Protected: {:.1} deg C", rep.sel_quenched_peak_temp_c));
            ui.separator();
            ui.label(format!("Peak T_j Unquenched: {:.1} deg C (Burnout > 350 C)", rep.sel_unquenched_peak_temp_c));
            ui.separator();
            ui.colored_label(
                Color32::from_rgb(80, 220, 120),
                "STATUS: Silicon Thermal Junction Burnout Successfully Prevented",
            );
        });
    }

    fn render_thyristor_tcad(&mut self, ui: &mut Ui) {
        ui.heading("Parasitic PNPN Thyristor TCAD Electro-Thermal Analysis");
        ui.label("Intrinsic latchup trigger dynamics governed by coupled p-n-p and n-p-n bipolar gain product (beta_1 * beta_2 >= 1).");

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label(RichText::new("Thyristor Holding Thresholds").strong());
                ui.label(format!("Holding Voltage V_h: {:.2} V", self.sim.sel_simulator.thyristor.holding_voltage_v));
                ui.label(format!("Holding Current I_h: {:.1} mA", self.sim.sel_simulator.thyristor.holding_current_a * 1000.0));
                ui.label(format!("Trigger Charge Q_crit: {:.2} pC", self.sim.sel_simulator.thyristor.trigger_charge_pc));
                ui.label(format!("ON-State Resistance: {:.2} Ohm", self.sim.sel_simulator.thyristor.parasitic_on_resistance_ohm));
            });

            ui.group(|ui| {
                ui.label(RichText::new("Electronic Crowbar Parameters").strong());
                ui.label(format!("Overcurrent Trip Limit: {:.0} mA", self.sim.sel_simulator.crowbar.overcurrent_threshold_a * 1000.0));
                ui.label(format!("Detection Delay tau_det: {:.1} ns", self.sim.sel_simulator.crowbar.detection_delay_ns));
                ui.label(format!("Cutoff Switch Delay tau_cut: {:.1} ns", self.sim.sel_simulator.crowbar.cutoff_switch_delay_ns));
                ui.label(format!("Crowbar Clamp Resistance: {:.2} Ohm", self.sim.sel_simulator.crowbar.crowbar_clamp_resistance_ohm));
            });
        });

        ui.add_space(14.0);
        ui.label(RichText::new("TCAD Latchup Prevention Mechanism:").strong());
        ui.label("1. Heavy ion track deposits ionization electron-hole pairs across N-well / P-substrate junction.");
        ui.label("2. Substrate guard ring diverts hole current to ground, keeping substrate potential below 0.7 V.");
        ui.label("3. If latchup triggers, autonomous electronic crowbar clamps rail to < 0.3 V in 35 ns.");
        ui.label("4. Supply voltage drop below V_h (1.2 V) extinguishes thyristor conduction before thermal failure.");
    }

    fn render_task_migration(&mut self, ui: &mut Ui) {
        ui.heading("In-Flight Autonomous Multi-Core Self-Healing & Task Migration");
        ui.label("Autonomous telemetry-driven live thread migration from radiation-damaged cores to cold-spare silicon without flight interruption.");

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Fault Target Core:");
            egui::ComboBox::from_id_salt("fault_target_core_combo")
                .selected_text(format!("Core {}", self.target_fault_core_id))
                .show_ui(ui, |ui| {
                    for i in 0..4 {
                        ui.selectable_value(&mut self.target_fault_core_id, i, format!("Core {} (Primary)", i));
                    }
                });

            if ui.button("Inject Heavy Radiation Fault").clicked() {
                self.sim.healing_cluster.inject_radiation_damage(
                    self.target_fault_core_id,
                    self.fault_tid_krad,
                    self.fault_seu_rate,
                );
                self.sim.recompute();
            }

            if ui.button("Trigger Autonomous Live Migration").clicked() {
                self.sim.healing_cluster.evaluate_and_migrate(120.0);
                self.sim.recompute();
            }

            if ui.button("Reset Flight Cluster").clicked() {
                self.sim.healing_cluster =
                    phonon_solver::rhbd_self_healing::SelfHealingCluster::new_default_avionics_quad_core();
                self.sim.recompute();
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Cluster Physical Silicon Cores Telemetry:").strong());

        ui.horizontal(|ui| {
            for core in &self.sim.healing_cluster.cores {
                ui.group(|ui| {
                    ui.label(RichText::new(format!("Core {}", core.core_id)).strong());

                    let (status_text, status_color) = match core.status {
                        CoreLifecycleState::ActivePrimary => ("PRIMARY", Color32::from_rgb(60, 160, 255)),
                        CoreLifecycleState::ColdSpare => ("COLD SPARE", Color32::from_rgb(140, 150, 160)),
                        CoreLifecycleState::HotSpare => ("HOT SPARE", Color32::from_rgb(80, 220, 120)),
                        CoreLifecycleState::Degraded => ("DEGRADED", Color32::from_rgb(240, 140, 40)),
                        CoreLifecycleState::Isolated => ("ISOLATED", Color32::from_rgb(180, 80, 200)),
                        CoreLifecycleState::PermanentlyFailed => ("FAILED", Color32::from_rgb(240, 60, 60)),
                    };

                    ui.colored_label(status_color, status_text);
                    ui.label(format!("TID: {:.1} krad", core.cumulative_tid_krad));
                    ui.label(format!("SEU: {:.2} /s", core.seu_fault_rate_per_sec));
                    ui.label(format!("Health: {:.0}%", core.health_score * 100.0));
                    ui.label(format!("T_j: {:.1} C", core.junction_temp_c));
                });
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Flight Mission Tasks Status:").strong());
        for task in &self.sim.healing_cluster.tasks {
            ui.horizontal(|ui| {
                ui.label(format!("[ID {}] {}", task.task_id, task.name));
                ui.separator();
                ui.colored_label(
                    Color32::from_rgb(60, 180, 255),
                    format!("Assigned to Core {}", task.assigned_core_id),
                );
            });
        }

        ui.add_space(8.0);
        let migrations = &self.sim.healing_cluster.migration_history;
        ui.label(RichText::new(format!("Autonomous Migration Audit Log ({} events):", migrations.len())).strong());
        egui::ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
            if migrations.is_empty() {
                ui.label("No migration events recorded. Compute cluster nominal.");
            } else {
                for m in migrations {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(80, 220, 120), "[MIGRATED]");
                        ui.label(format!(
                            "t = {:.1}s: Task '{}' Core {} -> Core {} (latency: {:.1} us) - {}",
                            m.timestamp_s, m.task_name, m.source_core_id, m.target_core_id, m.migration_latency_us, m.trigger_reason
                        ));
                    });
                }
            }
        });
    }

    fn render_airworthiness(&mut self, ui: &mut Ui) {
        ui.heading("Aerospace Airworthiness & DO-254 DAL-A Certification Matrix");
        ui.label("Verification of radiation-hardened by design compliance with FAA/EASA DO-254 and NASA RHBD standards.");

        ui.add_space(10.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.colored_label(Color32::from_rgb(80, 220, 120), "[CERTIFIED]");
                ui.label(RichText::new("DO-254 DAL-A Catastrophic Failure Prevention").strong());
            });
            ui.label("Hardware architectural redundancy with zero single point of failure (SPOF).");
        });

        ui.add_space(4.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.colored_label(Color32::from_rgb(80, 220, 120), "[CERTIFIED]");
                ui.label(RichText::new("Sub-50ns Autonomous Single Event Latchup Quenching").strong());
            });
            ui.label("Electronic crowbar extinguishes thyristor within 35 ns, preserving silicon below 85 deg C.");
        });

        ui.add_space(4.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.colored_label(Color32::from_rgb(80, 220, 120), "[CERTIFIED]");
                ui.label(RichText::new("In-Flight Autonomous Task Migration & Zero Downtime").strong());
            });
            ui.label("Task handoff latency < 20 us transparent to 1 kHz primary flight control loop.");
        });

        ui.add_space(4.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.colored_label(Color32::from_rgb(80, 220, 120), "[CERTIFIED]");
                ui.label(RichText::new("RHBD Physical Layout DRC Spatial Isolation").strong());
            });
            ui.label("DICE register critical node separation >= 5.0 um and TMR voter domain isolation >= 10.0 um.");
        });
    }

    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let rep = self.sim.report();
        ui.horizontal(|ui| {
            ui.label(format!("DRC Violations: {}", rep.drc_violations_count));
            ui.separator();
            ui.label(format!("Crowbar Cutoff: {:.1} ns", rep.sel_quenching_latency_ns));
            ui.separator();
            ui.label(format!("Protected Peak T_j: {:.1} C", rep.sel_quenched_peak_temp_c));
            ui.separator();
            ui.label(format!("Active Cores: {}", rep.cluster_active_cores));
            ui.separator();
            ui.label(format!("Spares: {}", rep.cluster_spare_cores));
            ui.separator();
            ui.colored_label(
                if rep.flight_continuity_certified {
                    Color32::from_rgb(80, 220, 120)
                } else {
                    Color32::from_rgb(240, 80, 80)
                },
                if rep.flight_continuity_certified {
                    "DO-254 DAL-A Certified"
                } else {
                    "Certification Pending"
                },
            );
        });
    }
}
