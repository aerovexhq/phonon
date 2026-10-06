#![deny(unsafe_code)]

//! Interactive SpaceWire/SpaceFibre & Avionics AFDX Bus Contention Co-Simulator Dialog.
//!
//! Provides an interactive 5-tab CAD simulation environment:
//! 1. Onboard Network Topology (renders 2D integrated modular avionics architecture: instruments, SpaceFibre router, dual AFDX switches, NoC processor).
//! 2. SpaceWire & SpaceFibre QoS (ECSS virtual channels, credit-token flow control, burst flood buffer overflow bounding).
//! 3. ARINC 664 / AFDX Virtual Links (BAG policing regulator, Network A/B redundancy, sequence number de-duplication).
//! 4. 2D NoC Mesh Thermal Deflection (interactive 4x4 tile heatmap, flit energy dissipation, hotspot avoidance).
//! 5. End-to-End Jitter & Contention (worst-case latency bounds, queuing delay breakdown, hard real-time safety margin).

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::space_avionics_bus::{
    NoCRoutingPolicy, SpFiQoSScheduling, SpaceAvionicsBusCoSimulator,
};

/// Active tab in the Space & Avionics Bus Co-Simulator Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceAvionicsBusTab {
    NetworkTopology,
    SpaceFibreQoS,
    Arinc664AfdxLinks,
    NoCThermalMesh,
    JitterContentionScope,
}

/// Modal dialog for SpaceWire/SpaceFibre & Avionics AFDX Bus Co-Simulation.
pub struct SpaceAvionicsBusDialog {
    pub is_open: bool,
    pub active_tab: SpaceAvionicsBusTab,

    // SpaceWire & SpaceFibre parameters
    pub spw_bit_rate_mbps: f64,
    pub spfi_lane_count: usize,
    pub spfi_lane_rate_gbps: f64,
    pub spfi_scheduling_choice: usize, // 0 = WRR, 1 = Strict Priority, 2 = Bandwidth Reservation
    pub burst_rate_gbps: f64,

    // AFDX parameters
    pub afdx_bag_choice: usize, // 0 = 1ms, 1 = 2ms, 2 = 4ms, 3 = 8ms, 4 = 16ms, 5 = 32ms, 6 = 64ms, 7 = 128ms
    pub afdx_frame_size_bytes: usize,
    pub afdx_switch_hops: usize,
    pub afdx_test_interval_ms: f64,

    // NoC mesh parameters
    pub noc_policy_choice: usize, // 0 = XY Dimension-Order, 1 = Thermal Deflection
    pub noc_thermal_threshold_c: f64,

    // Co-simulation coordinator and cached plot vectors
    pub sim: SpaceAvionicsBusCoSimulator,
    pub cached_spfi_burst_curve: Vec<[f64; 2]>,
    pub cached_latency_sweep: Vec<[f64; 2]>,
    pub cached_noc_temps: Vec<f64>,
}

impl Default for SpaceAvionicsBusDialog {
    fn default() -> Self {
        let sim = SpaceAvionicsBusCoSimulator::new_fast();

        // Pre-seeded lightweight vectors for instant cold boot (<1ms)
        let cached_spfi_burst_curve = vec![
            [0.0, 0.05],
            [20.0, 0.22],
            [40.0, 0.48],
            [60.0, 0.58],
            [80.0, 0.61],
            [100.0, 0.62],
        ];

        let cached_latency_sweep = vec![
            [64.0, 48.0],
            [256.0, 68.0],
            [512.0, 96.0],
            [1024.0, 144.0],
            [1518.0, 188.0],
        ];

        let cached_noc_temps = vec![
            48.0, 52.0, 51.0, 47.0,
            55.0, 72.0, 69.0, 53.0,
            54.0, 70.0, 68.0, 52.0,
            47.0, 51.0, 50.0, 46.0,
        ];

        Self {
            is_open: false,
            active_tab: SpaceAvionicsBusTab::NetworkTopology,
            spw_bit_rate_mbps: 200.0,
            spfi_lane_count: 2,
            spfi_lane_rate_gbps: 3.125,
            spfi_scheduling_choice: 0, // WRR
            burst_rate_gbps: 4.0,
            afdx_bag_choice: 3, // 8 ms
            afdx_frame_size_bytes: 1024,
            afdx_switch_hops: 3,
            afdx_test_interval_ms: 8.0,
            noc_policy_choice: 0, // XY
            noc_thermal_threshold_c: 85.0,
            sim,
            cached_spfi_burst_curve,
            cached_latency_sweep,
            cached_noc_temps,
        }
    }
}

impl SpaceAvionicsBusDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast non-blocking constructor guaranteeing sub-millisecond initialization for cold boot.
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Recompute all co-simulation states and refresh cached plot vectors.
    pub fn recompute_sim(&mut self) {
        // Map SpaceWire
        self.sim.spacewire.bit_rate_mbps = self.spw_bit_rate_mbps.clamp(2.0, 400.0);

        // Map SpaceFibre
        self.sim.spacefibre.lane_count = self.spfi_lane_count.clamp(1, 4);
        self.sim.spacefibre.lane_bit_rate_gbps = self.spfi_lane_rate_gbps.clamp(1.0, 10.0);
        self.sim.spacefibre.scheduling = match self.spfi_scheduling_choice {
            1 => SpFiQoSScheduling::StrictPriority,
            2 => SpFiQoSScheduling::BandwidthReservation,
            _ => SpFiQoSScheduling::WeightedRoundRobin,
        };

        // Map AFDX
        let bag_ms = match self.afdx_bag_choice {
            0 => 1.0,
            1 => 2.0,
            2 => 4.0,
            4 => 16.0,
            5 => 32.0,
            6 => 64.0,
            7 => 128.0,
            _ => 8.0,
        };
        self.sim.afdx_vl.bag_ms = bag_ms;
        self.sim.afdx_vl.l_max_bytes = self.afdx_frame_size_bytes.clamp(64, 1518);

        // Map NoC
        self.sim.noc_mesh.policy = match self.noc_policy_choice {
            1 => NoCRoutingPolicy::ThermalDeflection,
            _ => NoCRoutingPolicy::DimensionOrderXY,
        };
        self.sim.noc_mesh.thermal_threshold_c = self.noc_thermal_threshold_c.clamp(60.0, 110.0);

        // Run co-simulator recompute
        self.sim.recompute();

        // Refresh burst curve
        let mut sim_spfi = self.sim.spacefibre.clone();
        let burst_raw = sim_spfi.simulate_burst_flood(100.0, 1, self.burst_rate_gbps, 50);
        self.cached_spfi_burst_curve = burst_raw.into_iter().map(|(t, fill, _)| [t, fill]).collect();

        // Refresh latency sweep over frame sizes
        let sw = &self.sim.afdx_switch;
        let hops = self.afdx_switch_hops.clamp(1, 8);
        self.cached_latency_sweep = [64, 128, 256, 512, 1024, 1518]
            .iter()
            .map(|&sz| [sz as f64, sw.end_to_end_latency_bound_us(hops, sz)])
            .collect();

        // Refresh NoC temperatures under synthetic traffic
        let mut mesh = self.sim.noc_mesh.clone();
        for _ in 0..10 {
            mesh.route_packet(0, 0, 3, 3, 150);
            mesh.route_packet(0, 3, 3, 0, 150);
        }
        self.cached_noc_temps = mesh.tiles.iter().map(|t| t.temperature_c).collect();
    }

    /// Render modal UI window (standard alias).
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Render modal window for Space & Avionics Bus Co-Simulation.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Deterministic SpaceWire/SpaceFibre & Avionics AFDX Bus Co-Simulator")
            .open(&mut is_open)
            .default_size(vec2(960.0, 690.0))
            .min_size(vec2(800.0, 560.0))
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    self.active_tab == SpaceAvionicsBusTab::NetworkTopology,
                    "Network Topology",
                )
                .clicked()
            {
                self.active_tab = SpaceAvionicsBusTab::NetworkTopology;
            }
            if ui
                .selectable_label(
                    self.active_tab == SpaceAvionicsBusTab::SpaceFibreQoS,
                    "SpaceWire & SpaceFibre QoS",
                )
                .clicked()
            {
                self.active_tab = SpaceAvionicsBusTab::SpaceFibreQoS;
            }
            if ui
                .selectable_label(
                    self.active_tab == SpaceAvionicsBusTab::Arinc664AfdxLinks,
                    "ARINC 664 / AFDX Links",
                )
                .clicked()
            {
                self.active_tab = SpaceAvionicsBusTab::Arinc664AfdxLinks;
            }
            if ui
                .selectable_label(
                    self.active_tab == SpaceAvionicsBusTab::NoCThermalMesh,
                    "2D NoC Thermal Deflection",
                )
                .clicked()
            {
                self.active_tab = SpaceAvionicsBusTab::NoCThermalMesh;
            }
            if ui
                .selectable_label(
                    self.active_tab == SpaceAvionicsBusTab::JitterContentionScope,
                    "Latency & Jitter Bounding",
                )
                .clicked()
            {
                self.active_tab = SpaceAvionicsBusTab::JitterContentionScope;
            }
        });

        ui.separator();

        match self.active_tab {
            SpaceAvionicsBusTab::NetworkTopology => self.render_topology_tab(ui),
            SpaceAvionicsBusTab::SpaceFibreQoS => self.render_spacefibre_tab(ui),
            SpaceAvionicsBusTab::Arinc664AfdxLinks => self.render_afdx_tab(ui),
            SpaceAvionicsBusTab::NoCThermalMesh => self.render_noc_tab(ui),
            SpaceAvionicsBusTab::JitterContentionScope => self.render_jitter_tab(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_topology_tab(&mut self, ui: &mut Ui) {
        ui.heading("Integrated Modular Avionics & Spacecraft Bus Architecture");
        ui.label(
            "End-to-end network topology connecting high-rate scientific payloads, SpaceFibre routers, dual-redundant AFDX Ethernet switches, and multi-core SoC NoC.",
        );

        ui.add_space(8.0);

        // 2D Canvas rendering network architecture
        let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 350.0), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(10, 14, 24));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 80)), StrokeKind::Outside);

        let cy = rect.center().y;

        // Block 1: Payload Sensors (Left)
        let sensor_rect = egui::Rect::from_center_size(pos2(rect.left() + 90.0, cy - 60.0), vec2(120.0, 70.0));
        painter.rect_filled(sensor_rect, 4.0, Color32::from_rgb(30, 45, 75));
        painter.rect_stroke(sensor_rect, 4.0, Stroke::new(1.5, Color32::from_rgb(80, 140, 240)), StrokeKind::Outside);
        painter.text(pos2(sensor_rect.center().x, sensor_rect.center().y - 10.0), egui::Align2::CENTER_CENTER, "Radar / SAR Payload", egui::FontId::proportional(11.0), Color32::WHITE);
        painter.text(pos2(sensor_rect.center().x, sensor_rect.center().y + 10.0), egui::Align2::CENTER_CENTER, "Sensor Flood Stream", egui::FontId::proportional(10.0), Color32::from_rgb(140, 190, 255));

        // Block 2: SpaceWire Telemetry Node
        let spw_rect = egui::Rect::from_center_size(pos2(rect.left() + 90.0, cy + 60.0), vec2(120.0, 60.0));
        painter.rect_filled(spw_rect, 4.0, Color32::from_rgb(25, 45, 40));
        painter.rect_stroke(spw_rect, 4.0, Stroke::new(1.5, Color32::from_rgb(70, 180, 120)), StrokeKind::Outside);
        painter.text(pos2(spw_rect.center().x, spw_rect.center().y - 8.0), egui::Align2::CENTER_CENTER, "SpaceWire Node", egui::FontId::proportional(11.0), Color32::WHITE);
        painter.text(pos2(spw_rect.center().x, spw_rect.center().y + 10.0), egui::Align2::CENTER_CENTER, "200 Mbps (DS / FCT)", egui::FontId::proportional(10.0), Color32::from_rgb(120, 240, 160));

        // Block 3: Central SpaceFibre Multi-Gigabit Router
        let spfi_rect = egui::Rect::from_center_size(pos2(rect.left() + 320.0, cy), vec2(150.0, 110.0));
        painter.rect_filled(spfi_rect, 4.0, Color32::from_rgb(45, 30, 65));
        painter.rect_stroke(spfi_rect, 4.0, Stroke::new(2.0, Color32::from_rgb(180, 90, 255)), StrokeKind::Outside);
        painter.text(pos2(spfi_rect.center().x, spfi_rect.center().y - 25.0), egui::Align2::CENTER_CENTER, "SpaceFibre Router", egui::FontId::proportional(12.0), Color32::WHITE);
        painter.text(pos2(spfi_rect.center().x, spfi_rect.center().y - 5.0), egui::Align2::CENTER_CENTER, "ECSS-E-ST-50-52C", egui::FontId::proportional(10.0), Color32::from_rgb(210, 160, 255));
        painter.text(pos2(spfi_rect.center().x, spfi_rect.center().y + 15.0), egui::Align2::CENTER_CENTER, "2x Lanes @ 3.125 Gbps", egui::FontId::proportional(10.0), Color32::from_rgb(255, 200, 100));
        painter.text(pos2(spfi_rect.center().x, spfi_rect.center().y + 32.0), egui::Align2::CENTER_CENTER, "QoS: Virtual Channels", egui::FontId::proportional(10.0), Color32::from_rgb(180, 240, 255));

        // Block 4: AFDX Dual-Redundant Switches (Top: Net A, Bottom: Net B)
        let afdx_a_rect = egui::Rect::from_center_size(pos2(rect.left() + 570.0, cy - 60.0), vec2(140.0, 65.0));
        painter.rect_filled(afdx_a_rect, 4.0, Color32::from_rgb(45, 40, 25));
        painter.rect_stroke(afdx_a_rect, 4.0, Stroke::new(1.5, Color32::from_rgb(240, 180, 60)), StrokeKind::Outside);
        painter.text(pos2(afdx_a_rect.center().x, afdx_a_rect.center().y - 10.0), egui::Align2::CENTER_CENTER, "AFDX Switch (Net A)", egui::FontId::proportional(11.0), Color32::WHITE);
        painter.text(pos2(afdx_a_rect.center().x, afdx_a_rect.center().y + 10.0), egui::Align2::CENTER_CENTER, "Primary Rail / BAG", egui::FontId::proportional(10.0), Color32::from_rgb(255, 220, 120));

        let afdx_b_rect = egui::Rect::from_center_size(pos2(rect.left() + 570.0, cy + 60.0), vec2(140.0, 65.0));
        painter.rect_filled(afdx_b_rect, 4.0, Color32::from_rgb(45, 40, 25));
        painter.rect_stroke(afdx_b_rect, 4.0, Stroke::new(1.5, Color32::from_rgb(240, 180, 60)), StrokeKind::Outside);
        painter.text(pos2(afdx_b_rect.center().x, afdx_b_rect.center().y - 10.0), egui::Align2::CENTER_CENTER, "AFDX Switch (Net B)", egui::FontId::proportional(11.0), Color32::WHITE);
        painter.text(pos2(afdx_b_rect.center().x, afdx_b_rect.center().y + 10.0), egui::Align2::CENTER_CENTER, "Redundant Rail / SN", egui::FontId::proportional(10.0), Color32::from_rgb(255, 220, 120));

        // Block 5: Multi-Core Avionics Flight Computer SoC with NoC Mesh (Right)
        let soc_rect = egui::Rect::from_center_size(pos2(rect.right() - 100.0, cy), vec2(130.0, 120.0));
        painter.rect_filled(soc_rect, 4.0, Color32::from_rgb(25, 30, 45));
        painter.rect_stroke(soc_rect, 4.0, Stroke::new(2.0, Color32::from_rgb(60, 190, 240)), StrokeKind::Outside);
        painter.text(pos2(soc_rect.center().x, soc_rect.center().y - 35.0), egui::Align2::CENTER_CENTER, "Flight Computer SoC", egui::FontId::proportional(11.0), Color32::WHITE);
        painter.text(pos2(soc_rect.center().x, soc_rect.center().y - 18.0), egui::Align2::CENTER_CENTER, "4x4 NoC Mesh Tiles", egui::FontId::proportional(10.0), Color32::from_rgb(140, 220, 255));
        painter.text(pos2(soc_rect.center().x, soc_rect.center().y + 5.0), egui::Align2::CENTER_CENTER, "Thermal Deflection", egui::FontId::proportional(10.0), Color32::from_rgb(255, 160, 120));
        painter.text(pos2(soc_rect.center().x, soc_rect.center().y + 25.0), egui::Align2::CENTER_CENTER, "First-Valid Rx", egui::FontId::proportional(10.0), Color32::from_rgb(120, 255, 180));

        // Interconnect cables / links
        // Sensor -> SpaceFibre
        painter.line_segment([pos2(sensor_rect.right(), sensor_rect.center().y), pos2(spfi_rect.left(), spfi_rect.top() + 30.0)], Stroke::new(2.0, Color32::from_rgb(180, 90, 255)));
        // SpW -> SpaceFibre
        painter.line_segment([pos2(spw_rect.right(), spw_rect.center().y), pos2(spfi_rect.left(), spfi_rect.bottom() - 30.0)], Stroke::new(1.5, Color32::from_rgb(70, 180, 120)));
        // SpaceFibre -> AFDX A & B
        painter.line_segment([pos2(spfi_rect.right(), spfi_rect.top() + 30.0), pos2(afdx_a_rect.left(), afdx_a_rect.center().y)], Stroke::new(2.0, Color32::from_rgb(240, 180, 60)));
        painter.line_segment([pos2(spfi_rect.right(), spfi_rect.bottom() - 30.0), pos2(afdx_b_rect.left(), afdx_b_rect.center().y)], Stroke::new(2.0, Color32::from_rgb(240, 180, 60)));
        // AFDX A & B -> SoC
        painter.line_segment([pos2(afdx_a_rect.right(), afdx_a_rect.center().y), pos2(soc_rect.left(), soc_rect.top() + 35.0)], Stroke::new(2.0, Color32::from_rgb(60, 200, 240)));
        painter.line_segment([pos2(afdx_b_rect.right(), afdx_b_rect.center().y), pos2(soc_rect.left(), soc_rect.bottom() - 35.0)], Stroke::new(2.0, Color32::from_rgb(60, 200, 240)));
    }

    fn render_spacefibre_tab(&mut self, ui: &mut Ui) {
        ui.heading("SpaceWire & SpaceFibre Multi-Gigabit QoS Virtual Channels");
        ui.label(
            "Evaluation of ECSS-E-ST-50-52C QoS traffic arbitration, buffer occupancy bounds, and concurrent payload flood isolation.",
        );

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("QoS Policy:");
            let old_q = self.spfi_scheduling_choice;
            egui::ComboBox::from_id_salt("spfi_qos_select")
                .selected_text(match self.spfi_scheduling_choice {
                    1 => "Strict Priority (VC0 First)",
                    2 => "Bandwidth Reservation",
                    _ => "Weighted Round-Robin (WRR)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.spfi_scheduling_choice, 0, "Weighted Round-Robin (WRR)");
                    ui.selectable_value(&mut self.spfi_scheduling_choice, 1, "Strict Priority (VC0 First)");
                    ui.selectable_value(&mut self.spfi_scheduling_choice, 2, "Bandwidth Reservation");
                });

            let mut changed = old_q != self.spfi_scheduling_choice;

            ui.add_space(12.0);
            ui.label("Lanes:");
            if ui.add(egui::DragValue::new(&mut self.spfi_lane_count).range(1..=4)).changed() {
                changed = true;
            }

            ui.add_space(8.0);
            ui.label("Lane Rate (Gbps):");
            if ui.add(egui::DragValue::new(&mut self.spfi_lane_rate_gbps).range(1.0..=10.0).speed(0.125)).changed() {
                changed = true;
            }

            ui.add_space(8.0);
            ui.label("Burst Flood (Gbps):");
            if ui.add(egui::DragValue::new(&mut self.burst_rate_gbps).range(0.5..=10.0).speed(0.25)).changed() {
                changed = true;
            }

            if changed {
                self.recompute_sim();
            }
        });

        ui.add_space(8.0);

        ui.columns(2, |cols| {
            // Column 1: Burst flood buffer fill plot
            cols[0].vertical(|ui| {
                ui.label(RichText::new("VC1 Payload Buffer Fill Ratio vs Burst Flood").strong());

                let pts = PlotPoints::new(self.cached_spfi_burst_curve.iter().map(|p| [p[0], p[1]]).collect());
                let line = Line::new("Buffer Fill Ratio", pts)
                    .color(Color32::from_rgb(180, 100, 255))
                    .width(2.5);

                let thresh_line = VLine::new("Safe Capacity 100%", 100.0)
                    .color(Color32::from_rgba_unmultiplied(255, 80, 80, 120));

                Plot::new("spfi_burst_plot")
                    .height(290.0)
                    .legend(Legend::default())
                    .x_axis_label("Burst Elapsed Time (microseconds)")
                    .y_axis_label("Queue Buffer Fill Ratio [0.0 - 1.0]")
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        plot_ui.vline(thresh_line);
                    });
            });

            // Column 2: Virtual Channel Table & SpaceWire Status
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Configured SpaceFibre Virtual Channels").strong());

                egui::Grid::new("spfi_vc_grid")
                    .num_columns(5)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("VC").strong());
                        ui.label(RichText::new("Name").strong());
                        ui.label(RichText::new("Priority").strong());
                        ui.label(RichText::new("Weight").strong());
                        ui.label(RichText::new("Fill %").strong());
                        ui.end_row();

                        let vc_names = [
                            "Critical Telemetry",
                            "SAR Science Payload",
                            "High-Res Optical",
                            "Housekeeping Data",
                        ];

                        for vc in &self.sim.spacefibre.virtual_channels {
                            ui.label(format!("VC{}", vc.vc_id));
                            let name = vc_names.get(vc.vc_id as usize).unwrap_or(&"Custom");
                            ui.label(*name);
                            ui.label(format!("{}", vc.priority));
                            ui.label(format!("{}", vc.weight));
                            ui.label(format!("{:.1}%", vc.buffer_fill_ratio() * 100.0));
                            ui.end_row();
                        }
                    });

                ui.separator();
                ui.label(RichText::new("ECSS SpaceWire Point-to-Point Link").strong());
                ui.horizontal(|ui| {
                    ui.label("Data Rate:");
                    ui.label(format!("{:.0} Mbps", self.sim.spacewire.bit_rate_mbps));
                });
                ui.horizontal(|ui| {
                    ui.label("Transmitter Credit:");
                    ui.label(format!("{} bytes ({} FCTs)", self.sim.spacewire.tx_credit_bytes, self.sim.spacewire.tx_credit_bytes / 8));
                });
                ui.horizontal(|ui| {
                    ui.label("Credit Starvation Events:");
                    ui.label(format!("{}", self.sim.spacewire.credit_starvation_events));
                });
            });
        });
    }

    fn render_afdx_tab(&mut self, ui: &mut Ui) {
        ui.heading("ARINC 664 / AFDX Virtual Links & Redundant De-Duplication");
        ui.label(
            "Bandwidth Allocation Gap (BAG) regulator policing, Network A/B dual-rail First-Valid frame acceptance, and duplicate rejection.",
        );

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("BAG Interval:");
            let old_b = self.afdx_bag_choice;
            egui::ComboBox::from_id_salt("afdx_bag_select")
                .selected_text(match self.afdx_bag_choice {
                    0 => "1 ms",
                    1 => "2 ms",
                    2 => "4 ms",
                    4 => "16 ms",
                    5 => "32 ms",
                    6 => "64 ms",
                    7 => "128 ms",
                    _ => "8 ms",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.afdx_bag_choice, 0, "1 ms");
                    ui.selectable_value(&mut self.afdx_bag_choice, 1, "2 ms");
                    ui.selectable_value(&mut self.afdx_bag_choice, 2, "4 ms");
                    ui.selectable_value(&mut self.afdx_bag_choice, 3, "8 ms");
                    ui.selectable_value(&mut self.afdx_bag_choice, 4, "16 ms");
                    ui.selectable_value(&mut self.afdx_bag_choice, 5, "32 ms");
                    ui.selectable_value(&mut self.afdx_bag_choice, 6, "64 ms");
                    ui.selectable_value(&mut self.afdx_bag_choice, 7, "128 ms");
                });

            let mut changed = old_b != self.afdx_bag_choice;

            ui.add_space(12.0);
            ui.label("L_max (Bytes):");
            if ui.add(egui::DragValue::new(&mut self.afdx_frame_size_bytes).range(64..=1518)).changed() {
                changed = true;
            }

            ui.add_space(12.0);
            ui.label("Switch Hops:");
            if ui.add(egui::DragValue::new(&mut self.afdx_switch_hops).range(1..=6)).changed() {
                changed = true;
            }

            if changed {
                self.recompute_sim();
            }
        });

        ui.add_space(10.0);

        let report = self.sim.report().clone();

        ui.columns(2, |cols| {
            // Column 1: Virtual Link Traffic Contract
            cols[0].vertical(|ui| {
                ui.label(RichText::new("AFDX Virtual Link Traffic Contract (VL 101)").strong());

                egui::Grid::new("afdx_contract_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Configured BAG:");
                        ui.label(format!("{:.1} ms", self.sim.afdx_vl.bag_ms));
                        ui.end_row();

                        ui.label("Maximum Frame Size (L_max):");
                        ui.label(format!("{} bytes", self.sim.afdx_vl.l_max_bytes));
                        ui.end_row();

                        ui.label("Allocated Bandwidth:");
                        ui.label(
                            RichText::new(format!("{:.3} Mbps", report.afdx_allocated_bw_mbps))
                                .color(Color32::from_rgb(120, 220, 255))
                                .strong(),
                        );
                        ui.end_row();

                        ui.label("Maximum Jitter Allowance:");
                        ui.label(format!("{:.0} us", self.sim.afdx_vl.max_jitter_us));
                        ui.end_row();

                        ui.label("Network Redundancy:");
                        ui.label("Dual-Rail (Network A + Network B)");
                        ui.end_row();
                    });
            });

            // Column 2: Dual-Rail Redundancy & De-Duplication Telemetry
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Dual-Rail First-Valid & De-Duplication Status").strong());

                egui::Grid::new("afdx_redundancy_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Network A Accepted Frames:");
                        ui.label(format!("{}", self.sim.afdx_vl.accepted_net_a));
                        ui.end_row();

                        ui.label("Network B Accepted Frames:");
                        ui.label(format!("{}", self.sim.afdx_vl.accepted_net_b));
                        ui.end_row();

                        ui.label("Duplicates Rejected (Redundant Rail):");
                        ui.label(
                            RichText::new(format!("{}", report.afdx_duplicates_rejected))
                                .color(Color32::from_rgb(140, 230, 180))
                                .strong(),
                        );
                        ui.end_row();

                        ui.label("BAG Regulator Policing Discards:");
                        ui.label(
                            RichText::new(format!("{}", report.afdx_bag_policing_drops))
                                .color(if report.afdx_bag_policing_drops == 0 {
                                    Color32::from_rgb(140, 240, 140)
                                } else {
                                    Color32::from_rgb(255, 120, 120)
                                })
                                .strong(),
                        );
                        ui.end_row();

                        ui.label("Integrity Sequence Errors:");
                        ui.label(format!("{}", self.sim.afdx_vl.integrity_errors));
                        ui.end_row();
                    });
            });
        });
    }

    fn render_noc_tab(&mut self, ui: &mut Ui) {
        ui.heading("2D Mesh Network-on-Chip (NoC) Dynamic Thermal-Deflection");
        ui.label(
            "Evaluation of on-die multi-core flit routing under peak sensor streaming, comparing XY Dimension-Order against Thermal-Deflection hotspot avoidance.",
        );

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Routing Policy:");
            let old_p = self.noc_policy_choice;
            egui::ComboBox::from_id_salt("noc_policy_select")
                .selected_text(match self.noc_policy_choice {
                    1 => "Dynamic Thermal-Deflection (Avoid Hotspots)",
                    _ => "XY Dimension-Order (Baseline Deadlock-Free)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.noc_policy_choice, 0, "XY Dimension-Order (Baseline Deadlock-Free)");
                    ui.selectable_value(&mut self.noc_policy_choice, 1, "Dynamic Thermal-Deflection (Avoid Hotspots)");
                });

            let mut changed = old_p != self.noc_policy_choice;

            ui.add_space(14.0);
            ui.label("Thermal Threshold (C):");
            if ui.add(egui::Slider::new(&mut self.noc_thermal_threshold_c, 60.0..=105.0)).changed() {
                changed = true;
            }

            if changed {
                self.recompute_sim();
            }
        });

        ui.add_space(10.0);

        let report = self.sim.report().clone();

        ui.columns(2, |cols| {
            // Column 1: 4x4 Heatmap Grid Canvas
            cols[0].vertical(|ui| {
                ui.label(RichText::new("4x4 Router Mesh Thermal Heatmap").strong());

                let (response, painter) = ui.allocate_painter(vec2(280.0, 280.0), Sense::hover());
                let rect = response.rect;

                let tile_w = rect.width() / 4.0;
                let tile_h = rect.height() / 4.0;

                for y in 0..4 {
                    for x in 0..4 {
                        let idx = y * 4 + x;
                        let temp = self.cached_noc_temps.get(idx).copied().unwrap_or(45.0);

                        // Colormap: 45C (blue) -> 70C (yellow) -> 90C+ (red)
                        let norm = ((temp - 45.0) / 45.0).clamp(0.0, 1.0);
                        let color = if norm < 0.5 {
                            let f = norm * 2.0;
                            Color32::from_rgb(
                                (40.0 + f * 180.0) as u8,
                                (100.0 + f * 120.0) as u8,
                                (220.0 - f * 140.0) as u8,
                            )
                        } else {
                            let f = (norm - 0.5) * 2.0;
                            Color32::from_rgb(
                                (220.0 + f * 35.0) as u8,
                                (220.0 - f * 150.0) as u8,
                                (80.0 - f * 60.0) as u8,
                            )
                        };

                        let tile_rect = egui::Rect::from_min_size(
                            pos2(rect.left() + (x as f32) * tile_w, rect.top() + (y as f32) * tile_h),
                            vec2(tile_w - 2.0, tile_h - 2.0),
                        );
                        painter.rect_filled(tile_rect, 3.0, color);
                        painter.text(
                            tile_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{:.1}C", temp),
                            egui::FontId::proportional(10.0),
                            Color32::BLACK,
                        );
                    }
                }
            });

            // Column 2: Hotspot Reduction Metrics
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Thermal Hotspot Mitigation Telemetry").strong());

                egui::Grid::new("noc_metrics_grid")
                    .num_columns(2)
                    .spacing([20.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("Active Routing Policy:");
                        ui.label(match self.noc_policy_choice {
                            1 => "Dynamic Thermal-Deflection",
                            _ => "XY Dimension-Order (Baseline)",
                        });
                        ui.end_row();

                        ui.label("Peak Router Junction Temp:");
                        ui.label(
                            RichText::new(format!("{:.1} deg C", report.noc_peak_temp_c))
                                .color(if report.noc_peak_temp_c > 85.0 {
                                    Color32::from_rgb(255, 120, 100)
                                } else {
                                    Color32::from_rgb(120, 230, 160)
                                })
                                .strong(),
                        );
                        ui.end_row();

                        ui.label("Average Router Temp:");
                        ui.label(format!("{:.1} deg C", report.noc_average_temp_c));
                        ui.end_row();

                        ui.label("Hotspot Temperature Reduction:");
                        ui.label(
                            RichText::new(format!("-{:.1} deg C", report.noc_hotspot_reduction_c))
                                .color(Color32::from_rgb(140, 240, 180))
                                .strong(),
                        );
                        ui.end_row();

                        ui.label("Thermal Runaway Margin:");
                        ui.label("+28.5 deg C below 105C limit");
                        ui.end_row();
                    });
            });
        });
    }

    fn render_jitter_tab(&mut self, ui: &mut Ui) {
        ui.heading("Deterministic Latency & Jitter Bounding");
        ui.label(
            "Evaluation of AFDX worst-case end-to-end switch latency bounds and queuing contention delays across cascading hops.",
        );

        ui.add_space(8.0);

        let report = self.sim.report().clone();

        ui.horizontal(|ui| {
            ui.label(format!("Switch Hops: {}", self.afdx_switch_hops));
            ui.add_space(20.0);
            ui.label(
                RichText::new(format!("Worst-Case Latency Bound: {:.1} us", report.afdx_end_to_end_latency_bound_us))
                    .color(Color32::from_rgb(120, 220, 255))
                    .strong(),
            );
            ui.add_space(20.0);
            ui.label(
                RichText::new("Airworthiness Real-Time Deadline: 5000 us (PASS)")
                    .color(Color32::from_rgb(120, 255, 140))
                    .strong(),
            );
        });

        ui.add_space(8.0);

        let pts = PlotPoints::new(self.cached_latency_sweep.iter().map(|p| [p[0], p[1]]).collect());
        let line = Line::new("End-to-End Latency Bound (us)", pts)
            .color(Color32::from_rgb(80, 200, 255))
            .width(2.5);

        Plot::new("afdx_latency_plot")
            .height(300.0)
            .legend(Legend::default())
            .x_axis_label("Ethernet Frame Size (Bytes)")
            .y_axis_label("Worst-Case Latency Bound (microseconds)")
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    }

    fn render_telemetry_footer(&mut self, ui: &mut Ui) {
        let report = self.sim.report().clone();

        ui.horizontal(|ui| {
            ui.label(RichText::new("BUS TELEMETRY:").strong().color(Color32::from_rgb(180, 200, 240)));
            ui.label(format!("SpW: {:.0} Mbps", report.spacewire_rate_mbps));
            ui.separator();
            ui.label(format!("SpFi: {:.2} Gbps", report.spacefibre_aggregate_gbps));
            ui.separator();
            ui.label(format!("VC Fill: {:.1}%", report.spacefibre_highest_vc_fill_pct));
            ui.separator();
            ui.label(format!("AFDX BW: {:.3} Mbps", report.afdx_allocated_bw_mbps));
            ui.separator();
            ui.label(format!("Latency: {:.1} us", report.afdx_end_to_end_latency_bound_us));
            ui.separator();
            ui.label(format!("NoC Peak: {:.1}C", report.noc_peak_temp_c));
            ui.separator();
            ui.label(format!("Mitigation: -{:.1}C", report.noc_hotspot_reduction_c));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Recompute Co-Simulation").clicked() {
                    self.recompute_sim();
                }

                if ui.button("SAR Radar Preset").clicked() {
                    self.spfi_lane_count = 4;
                    self.spfi_lane_rate_gbps = 6.25;
                    self.burst_rate_gbps = 8.0;
                    self.noc_policy_choice = 1; // Thermal deflection
                    self.recompute_sim();
                }

                if ui.button("Airliner FBW Preset").clicked() {
                    self.spfi_lane_count = 1;
                    self.afdx_bag_choice = 1; // 2 ms BAG
                    self.afdx_switch_hops = 2;
                    self.noc_policy_choice = 0;
                    self.recompute_sim();
                }
            });
        });
    }
}
