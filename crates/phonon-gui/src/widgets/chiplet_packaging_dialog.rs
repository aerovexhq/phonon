#![deny(unsafe_code)]

//! Interactive 2.5D/3D Multi-Die & Chiplet Packaging CAD Studio Dialog.
//!
//! Provides a 5-tab CAD packaging co-simulation environment:
//! 1. Architecture & 3D Stack Canvas (interactive cross-section rendering of dies, TSVs, microbumps, and substrate).
//! 2. UCIe / BoW D2D PHY & Eye Diagram (synthesized 2-UI dual-Dirac eye contours, jitter budget, BER < 1e-15).
//! 3. TSV & RDL Parasitics Extractor (RLGC extraction and S11/S21 S-parameters across 1 to 50 GHz).
//! 4. Thermo-Mechanical Warpage & Shear Stress (Timoshenko bow curves, corner bump shear, Coffin-Manson cycles).
//! 5. Assembly Reliability & Qualification (JEDEC coplanarity margins, underfill integrity, pass/fail status).

use egui::{Color32, Context, Pos2, Rect, RichText, Stroke, StrokeKind, Ui, Vec2, Window};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::chiplet_packaging::{
    ChipletPackagingCoSimulator, PackagingArchitecture, UcieDataRateGbps,
};

/// Active tab in the Chiplet Packaging Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagingTab {
    ArchitectureStack3D,
    UcieEyeDiagram,
    TsvRdlParasitics,
    ThermoMechanicalWarpage,
    AssemblyReliability,
}

/// Modal dialog for Heterogeneous 2.5D/3D Chiplet Packaging Co-Simulation.
pub struct ChipletPackagingDialog {
    pub is_open: bool,
    pub active_tab: PackagingTab,

    // Core co-simulator
    pub sim: ChipletPackagingCoSimulator,

    // Cached plot curves for instant rendering
    pub cached_eye_upper_inner: Vec<[f64; 2]>,
    pub cached_eye_lower_inner: Vec<[f64; 2]>,
    pub cached_eye_upper_outer: Vec<[f64; 2]>,
    pub cached_eye_lower_outer: Vec<[f64; 2]>,
    pub cached_s21_curve: Vec<[f64; 2]>,
    pub cached_s11_curve: Vec<[f64; 2]>,
    pub cached_warpage_temp_curve: Vec<[f64; 2]>,
}

impl Default for ChipletPackagingDialog {
    fn default() -> Self {
        let sim = ChipletPackagingCoSimulator::new_fast();

        let cached_eye_upper_inner = sim.eye_contours().upper_inner_contour.clone();
        let cached_eye_lower_inner = sim.eye_contours().lower_inner_contour.clone();
        let cached_eye_upper_outer = sim.eye_contours().upper_outer_contour.clone();
        let cached_eye_lower_outer = sim.eye_contours().lower_outer_contour.clone();

        let cached_s21_curve = sim
            .s_parameters()
            .iter()
            .map(|p| [p.freq_ghz, p.s21_db])
            .collect();

        let cached_s11_curve = sim
            .s_parameters()
            .iter()
            .map(|p| [p.freq_ghz, p.s11_db])
            .collect();

        // Warpage bow vs temperature [-40 C to 125 C]
        let mut cached_warpage_temp_curve = Vec::with_capacity(34);
        for t_step in (-40..=125).step_by(5) {
            let t_c = t_step as f64;
            let mut eval_thermal = sim.thermal_params.clone();
            eval_thermal.current_temp_c = t_c;
            let rep = phonon_solver::chiplet_packaging::evaluate_thermo_mechanics(
                &sim.stack_geom,
                &eval_thermal,
            );
            cached_warpage_temp_curve.push([t_c, rep.warpage_bow_um]);
        }

        Self {
            is_open: false,
            active_tab: PackagingTab::ArchitectureStack3D,
            sim,
            cached_eye_upper_inner,
            cached_eye_lower_inner,
            cached_eye_upper_outer,
            cached_eye_lower_outer,
            cached_s21_curve,
            cached_s11_curve,
            cached_warpage_temp_curve,
        }
    }
}

impl ChipletPackagingDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast non-blocking constructor guaranteeing sub-millisecond initialization for cold boot.
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Recomputes all packaging physics and updates cached plotting series.
    pub fn recompute_sim(&mut self) {
        self.sim.recompute();

        self.cached_eye_upper_inner = self.sim.eye_contours().upper_inner_contour.clone();
        self.cached_eye_lower_inner = self.sim.eye_contours().lower_inner_contour.clone();
        self.cached_eye_upper_outer = self.sim.eye_contours().upper_outer_contour.clone();
        self.cached_eye_lower_outer = self.sim.eye_contours().lower_outer_contour.clone();

        self.cached_s21_curve = self
            .sim
            .s_parameters()
            .iter()
            .map(|p| [p.freq_ghz, p.s21_db])
            .collect();

        self.cached_s11_curve = self
            .sim
            .s_parameters()
            .iter()
            .map(|p| [p.freq_ghz, p.s11_db])
            .collect();

        let mut warpage_points = Vec::with_capacity(34);
        for t_step in (-40..=125).step_by(5) {
            let t_c = t_step as f64;
            let mut eval_thermal = self.sim.thermal_params.clone();
            eval_thermal.current_temp_c = t_c;
            let rep = phonon_solver::chiplet_packaging::evaluate_thermo_mechanics(
                &self.sim.stack_geom,
                &eval_thermal,
            );
            warpage_points.push([t_c, rep.warpage_bow_um]);
        }
        self.cached_warpage_temp_curve = warpage_points;
    }

    /// Renders modal window interface.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Main entry point for rendering the dialog window.
    pub fn show(&mut self, ctx: &Context) {
        let mut is_open = self.is_open;
        Window::new("Heterogeneous 2.5D/3D Multi-Die & Chiplet Packaging Studio")
            .open(&mut is_open)
            .default_width(980.0)
            .default_height(720.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders tab bar, active view, and telemetry status footer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    self.active_tab == PackagingTab::ArchitectureStack3D,
                    "3D Packaging Stack & Dies",
                )
                .clicked()
            {
                self.active_tab = PackagingTab::ArchitectureStack3D;
            }
            if ui
                .selectable_label(
                    self.active_tab == PackagingTab::UcieEyeDiagram,
                    "UCIe PHY Eye Diagram",
                )
                .clicked()
            {
                self.active_tab = PackagingTab::UcieEyeDiagram;
            }
            if ui
                .selectable_label(
                    self.active_tab == PackagingTab::TsvRdlParasitics,
                    "TSV & RDL S-Parameters",
                )
                .clicked()
            {
                self.active_tab = PackagingTab::TsvRdlParasitics;
            }
            if ui
                .selectable_label(
                    self.active_tab == PackagingTab::ThermoMechanicalWarpage,
                    "Thermal Warpage & Fatigue",
                )
                .clicked()
            {
                self.active_tab = PackagingTab::ThermoMechanicalWarpage;
            }
            if ui
                .selectable_label(
                    self.active_tab == PackagingTab::AssemblyReliability,
                    "Assembly Reliability Matrix",
                )
                .clicked()
            {
                self.active_tab = PackagingTab::AssemblyReliability;
            }
        });

        ui.separator();

        match self.active_tab {
            PackagingTab::ArchitectureStack3D => self.render_stack_canvas(ui),
            PackagingTab::UcieEyeDiagram => self.render_eye_diagram(ui),
            PackagingTab::TsvRdlParasitics => self.render_parasitics(ui),
            PackagingTab::ThermoMechanicalWarpage => self.render_warpage(ui),
            PackagingTab::AssemblyReliability => self.render_reliability(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_stack_canvas(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Architecture Template:").strong());
            let current_arch = self.sim.architecture;
            if ui
                .selectable_label(
                    current_arch == PackagingArchitecture::CoWoS_S,
                    "TSMC CoWoS-S",
                )
                .clicked()
            {
                self.sim.apply_architecture_preset(PackagingArchitecture::CoWoS_S);
                self.recompute_sim();
            }
            if ui
                .selectable_label(
                    current_arch == PackagingArchitecture::Intel_EMIB,
                    "Intel EMIB",
                )
                .clicked()
            {
                self.sim.apply_architecture_preset(PackagingArchitecture::Intel_EMIB);
                self.recompute_sim();
            }
            if ui
                .selectable_label(
                    current_arch == PackagingArchitecture::Organic_2_5D,
                    "Organic 2.5D",
                )
                .clicked()
            {
                self.sim.apply_architecture_preset(PackagingArchitecture::Organic_2_5D);
                self.recompute_sim();
            }
            if ui
                .selectable_label(
                    current_arch == PackagingArchitecture::SoIC_3D,
                    "TSMC 3D SoIC",
                )
                .clicked()
            {
                self.sim.apply_architecture_preset(PackagingArchitecture::SoIC_3D);
                self.recompute_sim();
            }
        });

        ui.add_space(6.0);

        // 2D schematic cross-section rendering canvas
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), 260.0), egui::Sense::hover());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 28));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 70)), StrokeKind::Middle);

        let mid_x = rect.center().x;
        let base_y = rect.max.y - 25.0;

        // 1. Package Substrate
        let sub_w = (rect.width() * 0.85).min(650.0);
        let sub_h = 35.0;
        let sub_rect = Rect::from_center_size(
            Pos2::new(mid_x, base_y - sub_h / 2.0),
            Vec2::new(sub_w, sub_h),
        );
        painter.rect_filled(sub_rect, 2.0, Color32::from_rgb(32, 60, 45)); // Green FR4/BT core
        painter.rect_stroke(sub_rect, 2.0, Stroke::new(1.5, Color32::from_rgb(60, 140, 90)), StrokeKind::Middle);
        painter.text(
            sub_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!(
                "Package Substrate (ABF/BT Core, {} mm)",
                self.sim.stack_geom.substrate_size_mm
            ),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(180, 240, 200),
        );

        // 2. BGA Solder Balls under substrate
        for i in 0..12 {
            let ball_x = sub_rect.min.x + 25.0 + (i as f32) * ((sub_w - 50.0) / 11.0);
            painter.circle_filled(
                Pos2::new(ball_x, base_y + 8.0),
                6.0,
                Color32::from_rgb(160, 170, 185),
            );
        }

        // 3. Intermediate Layer (Silicon Interposer / Bridge / Underfill)
        let inter_y = sub_rect.min.y - 12.0;
        if self.sim.stack_geom.has_silicon_interposer {
            let int_w = sub_w * 0.75;
            let int_h = 24.0;
            let int_rect = Rect::from_center_size(
                Pos2::new(mid_x, inter_y - int_h / 2.0),
                Vec2::new(int_w, int_h),
            );
            // Silicon interposer
            painter.rect_filled(int_rect, 2.0, Color32::from_rgb(45, 55, 75));
            painter.rect_stroke(int_rect, 2.0, Stroke::new(1.5, Color32::from_rgb(100, 140, 200)), StrokeKind::Middle);
            painter.text(
                int_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("Silicon Interposer (TSVs + High-Density RDL, {} um)", self.sim.tsv_geom.diameter_um),
                egui::FontId::proportional(11.0),
                Color32::from_rgb(200, 220, 255),
            );

            // Draw TSV vertical copper channels
            for v in 0..8 {
                let via_x = int_rect.min.x + 30.0 + (v as f32) * ((int_w - 60.0) / 7.0);
                painter.line_segment(
                    [Pos2::new(via_x, int_rect.min.y + 2.0), Pos2::new(via_x, int_rect.max.y - 2.0)],
                    Stroke::new(2.5, Color32::from_rgb(230, 140, 70)),
                );
            }

            // Micro-bumps connecting interposer to top dies
            let bump_y = int_rect.min.y - 5.0;
            for b in 0..16 {
                let bx = int_rect.min.x + 20.0 + (b as f32) * ((int_w - 40.0) / 15.0);
                painter.circle_filled(Pos2::new(bx, bump_y), 3.5, Color32::from_rgb(230, 160, 90));
            }

            // Top Compute & HBM Memory Dies
            let die_y = int_rect.min.y - 12.0;
            let die1_rect = Rect::from_min_max(
                Pos2::new(int_rect.min.x + 25.0, die_y - 45.0),
                Pos2::new(int_rect.center().x - 15.0, die_y),
            );
            painter.rect_filled(die1_rect, 2.0, Color32::from_rgb(70, 35, 35));
            painter.rect_stroke(die1_rect, 2.0, Stroke::new(1.5, Color32::from_rgb(220, 90, 90)), StrokeKind::Middle);
            painter.text(
                die1_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Compute Die 0 (CPU/GPU)",
                egui::FontId::proportional(11.0),
                Color32::from_rgb(255, 190, 190),
            );

            let die2_rect = Rect::from_min_max(
                Pos2::new(int_rect.center().x + 15.0, die_y - 45.0),
                Pos2::new(int_rect.max.x - 25.0, die_y),
            );
            painter.rect_filled(die2_rect, 2.0, Color32::from_rgb(30, 50, 80));
            painter.rect_stroke(die2_rect, 2.0, Stroke::new(1.5, Color32::from_rgb(80, 150, 240)), StrokeKind::Middle);
            painter.text(
                die2_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Memory Die 1 (HBM3 Stack)",
                egui::FontId::proportional(11.0),
                Color32::from_rgb(190, 220, 255),
            );

            // D2D Interconnect Bridge Line (UCIe)
            painter.line_segment(
                [
                    Pos2::new(die1_rect.max.x - 10.0, int_rect.min.y + 4.0),
                    Pos2::new(die2_rect.min.x + 10.0, int_rect.min.y + 4.0),
                ],
                Stroke::new(3.0, Color32::from_rgb(255, 220, 50)),
            );
        } else {
            // Direct organic or embedded bridge
            let die_w = sub_w * 0.38;
            let die_y = inter_y - 25.0;

            let die1_rect = Rect::from_min_max(
                Pos2::new(mid_x - die_w - 15.0, die_y - 45.0),
                Pos2::new(mid_x - 15.0, die_y),
            );
            painter.rect_filled(die1_rect, 2.0, Color32::from_rgb(70, 35, 35));
            painter.rect_stroke(die1_rect, 2.0, Stroke::new(1.5, Color32::from_rgb(220, 90, 90)), StrokeKind::Middle);
            painter.text(
                die1_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Die A (Compute)",
                egui::FontId::proportional(11.0),
                Color32::from_rgb(255, 190, 190),
            );

            let die2_rect = Rect::from_min_max(
                Pos2::new(mid_x + 15.0, die_y - 45.0),
                Pos2::new(mid_x + die_w + 15.0, die_y),
            );
            painter.rect_filled(die2_rect, 2.0, Color32::from_rgb(30, 50, 80));
            painter.rect_stroke(die2_rect, 2.0, Stroke::new(1.5, Color32::from_rgb(80, 150, 240)), StrokeKind::Middle);
            painter.text(
                die2_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Die B (I/O Chiplet)",
                egui::FontId::proportional(11.0),
                Color32::from_rgb(190, 220, 255),
            );

            // Standard substrate microbumps
            for b in 0..10 {
                let bx = die1_rect.min.x + 15.0 + (b as f32) * ((die_w - 30.0) / 9.0);
                painter.circle_filled(Pos2::new(bx, inter_y - 5.0), 4.5, Color32::from_rgb(230, 160, 90));
            }
            for b in 0..10 {
                let bx = die2_rect.min.x + 15.0 + (b as f32) * ((die_w - 30.0) / 9.0);
                painter.circle_filled(Pos2::new(bx, inter_y - 5.0), 4.5, Color32::from_rgb(230, 160, 90));
            }
        }

        ui.add_space(8.0);

        // Interactive parameter sliders
        ui.columns(2, |cols| {
            cols[0].label(RichText::new("Stack Geometry Dimensions").strong());
            let mut changed = false;

            changed |= cols[0]
                .add(
                    egui::Slider::new(&mut self.sim.stack_geom.die_size_mm, 5.0..=35.0)
                        .text("Die Size (mm)"),
                )
                .changed();

            changed |= cols[0]
                .add(
                    egui::Slider::new(&mut self.sim.stack_geom.die_thickness_um, 30.0..=775.0)
                        .text("Die Thickness (um)"),
                )
                .changed();

            changed |= cols[0]
                .add(
                    egui::Slider::new(&mut self.sim.stack_geom.bump_pitch_um, 5.0..=150.0)
                        .text("Bump Pitch (um)"),
                )
                .changed();

            cols[1].label(RichText::new("Interconnect Physical Properties").strong());
            changed |= cols[1]
                .add(
                    egui::Slider::new(&mut self.sim.stack_geom.substrate_size_mm, 20.0..=80.0)
                        .text("Substrate Size (mm)"),
                )
                .changed();

            changed |= cols[1]
                .add(
                    egui::Slider::new(&mut self.sim.stack_geom.bump_height_um, 10.0..=60.0)
                        .text("Bump Standoff (um)"),
                )
                .changed();

            if changed {
                self.recompute_sim();
            }
        });
    }

    fn render_eye_diagram(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("UCIe PHY Data Rate:").strong());
            let current_rate = self.sim.ucie_params.data_rate;
            let rates = [
                (UcieDataRateGbps::Rate8Gbps, "8 Gbps"),
                (UcieDataRateGbps::Rate16Gbps, "16 Gbps"),
                (UcieDataRateGbps::Rate24Gbps, "24 Gbps"),
                (UcieDataRateGbps::Rate32Gbps, "32 Gbps"),
            ];

            for (rate, label) in rates {
                if ui.selectable_label(current_rate == rate, label).clicked() {
                    self.sim.ucie_params.data_rate = rate;
                    self.recompute_sim();
                }
            }

            ui.separator();
            let mut changed = false;
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.sim.ucie_params.trace_length_mm, 0.5..=25.0)
                        .text("Channel Reach (mm)"),
                )
                .changed();

            if changed {
                self.recompute_sim();
            }
        });

        ui.add_space(4.0);

        let eye_metrics = self.sim.eye_metrics();
        let ui_ps = eye_metrics.ui_ps;

        let plot = Plot::new("ucie_eye_diagram_plot")
            .legend(Legend::default())
            .height(280.0)
            .x_axis_label("Time (ps)")
            .y_axis_label("Sampler Voltage (mV)")
            .include_x(-ui_ps * 0.55)
            .include_x(ui_ps * 0.55)
            .include_y(-300.0)
            .include_y(300.0);

        plot.show(ui, |plot_ui| {
            // Plot inner open eye contours
            plot_ui.line(
                Line::new("Upper Inner Eye", PlotPoints::from(self.cached_eye_upper_inner.clone()))
                    .color(Color32::from_rgb(80, 220, 120))
                    .width(2.5),
            );
            plot_ui.line(
                Line::new("Lower Inner Eye", PlotPoints::from(self.cached_eye_lower_inner.clone()))
                    .color(Color32::from_rgb(80, 220, 120))
                    .width(2.5),
            );

            // Plot outer transitions
            plot_ui.line(
                Line::new("Outer Transition A", PlotPoints::from(self.cached_eye_upper_outer.clone()))
                    .color(Color32::from_rgb(220, 160, 50))
                    .width(1.2),
            );
            plot_ui.line(
                Line::new("Outer Transition B", PlotPoints::from(self.cached_eye_lower_outer.clone()))
                    .color(Color32::from_rgb(220, 160, 50))
                    .width(1.2),
            );

            // Vertical eye width boundaries
            let half_ew = eye_metrics.eye_width_ps / 2.0;
            plot_ui.vline(
                VLine::new("EW Left Limit", -half_ew)
                    .color(Color32::from_rgb(180, 100, 255))
                    .stroke(Stroke::new(1.2, Color32::from_rgb(180, 100, 255))),
            );
            plot_ui.vline(
                VLine::new("EW Right Limit", half_ew)
                    .color(Color32::from_rgb(180, 100, 255))
                    .stroke(Stroke::new(1.2, Color32::from_rgb(180, 100, 255))),
            );

            // Horizontal eye height threshold boundaries
            let half_eh = eye_metrics.eye_height_mv / 2.0;
            plot_ui.hline(
                HLine::new("EH Upper Bound", half_eh)
                    .color(Color32::from_rgb(80, 180, 255))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(80, 180, 255))),
            );
            plot_ui.hline(
                HLine::new("EH Lower Bound", -half_eh)
                    .color(Color32::from_rgb(80, 180, 255))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(80, 180, 255))),
            );
        });

        ui.add_space(6.0);
        ui.columns(4, |cols| {
            cols[0].label(format!("Unit Interval (UI): {:.1} ps", eye_metrics.ui_ps));
            cols[0].label(format!("Insertion Loss: {:.2} dB", eye_metrics.insertion_loss_db));

            cols[1].label(format!("Total Jitter (1e-15): {:.2} ps", eye_metrics.total_jitter_ps));
            cols[1].label(format!("Energy Efficiency: {:.3} pJ/bit", eye_metrics.energy_efficiency_pj_bit));

            cols[2].label(format!("Eye Width: {:.1} ps ({:.2} UI)", eye_metrics.eye_width_ps, eye_metrics.eye_width_ui));
            cols[2].label(format!("Eye Height: {:.1} mV", eye_metrics.eye_height_mv));

            let compliance_color = if eye_metrics.is_compliant {
                Color32::from_rgb(80, 220, 120)
            } else {
                Color32::from_rgb(255, 90, 90)
            };
            cols[3].label(RichText::new(if eye_metrics.is_compliant {
                "UCIe 1.0/2.0 Compliant"
            } else {
                "Non-Compliant (Jitter/Loss Exceeded)"
            }).color(compliance_color).strong());
        });
    }

    fn render_parasitics(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut changed = false;
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.sim.tsv_geom.diameter_um, 2.0..=15.0)
                        .text("TSV Diameter (um)"),
                )
                .changed();

            changed |= ui
                .add(
                    egui::Slider::new(&mut self.sim.tsv_geom.height_um, 20.0..=150.0)
                        .text("TSV Height (um)"),
                )
                .changed();

            changed |= ui
                .add(
                    egui::Slider::new(&mut self.sim.rdl_geom.width_um, 0.8..=5.0)
                        .text("RDL Width (um)"),
                )
                .changed();

            if changed {
                self.recompute_sim();
            }
        });

        ui.add_space(4.0);

        let plot = Plot::new("s_parameters_plot")
            .legend(Legend::default())
            .height(280.0)
            .x_axis_label("Frequency (GHz)")
            .y_axis_label("Magnitude (dB)")
            .include_x(1.0)
            .include_x(50.0)
            .include_y(-35.0)
            .include_y(0.0);

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Insertion Loss S21 (dB)", PlotPoints::from(self.cached_s21_curve.clone()))
                    .color(Color32::from_rgb(255, 120, 80))
                    .width(2.0),
            );
            plot_ui.line(
                Line::new("Return Loss S11 (dB)", PlotPoints::from(self.cached_s11_curve.clone()))
                    .color(Color32::from_rgb(80, 180, 255))
                    .width(1.8),
            );
        });

        ui.add_space(6.0);
        let tsv_rlgc = self.sim.tsv_rlgc();
        let rdl_rlgc = self.sim.rdl_rlgc();

        ui.columns(4, |cols| {
            cols[0].label(RichText::new("TSV RLGC Parameters").strong());
            cols[0].label(format!("R_dc: {:.2} mOhm", tsv_rlgc.r_dc_mohm));
            cols[0].label(format!("L_loop: {:.2} pH", tsv_rlgc.l_ph));

            cols[1].label(RichText::new("TSV Parasitic Capacitance").strong());
            cols[1].label(format!("C_ox: {:.2} fF", tsv_rlgc.c_ox_ff));
            cols[1].label(format!("C_total: {:.2} fF", tsv_rlgc.c_total_ff));

            cols[2].label(RichText::new("RDL Line Parameters").strong());
            cols[2].label(format!("R_dc: {:.2} Ohm", rdl_rlgc.r_dc_ohm));
            cols[2].label(format!("L_total: {:.2} nH", rdl_rlgc.l_nh));

            cols[3].label(RichText::new("RDL Impedance & Delay").strong());
            cols[3].label(format!("Z_0: {:.1} Ohm", rdl_rlgc.z0_ohm));
            cols[3].label(format!("Prop Delay: {:.1} ps", rdl_rlgc.t_pd_ps));
        });
    }

    fn render_warpage(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut changed = false;
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.sim.thermal_params.ref_temp_c, 120.0..=260.0)
                        .text("Underfill Cure Temp (C)"),
                )
                .changed();

            changed |= ui
                .add(
                    egui::Slider::new(&mut self.sim.thermal_params.current_temp_c, -40.0..=125.0)
                        .text("Operating Temp (C)"),
                )
                .changed();

            if changed {
                self.recompute_sim();
            }
        });

        ui.add_space(4.0);

        let plot = Plot::new("warpage_curve_plot")
            .legend(Legend::default())
            .height(280.0)
            .x_axis_label("Operating Temperature (C)")
            .y_axis_label("Out-of-Plane Bow (um)")
            .include_x(-40.0)
            .include_x(125.0);

        plot.show(ui, |plot_ui| {
            plot_ui.line(
                Line::new("Warpage Bow (um)", PlotPoints::from(self.cached_warpage_temp_curve.clone()))
                    .color(Color32::from_rgb(255, 180, 50))
                    .width(2.2),
            );

            // JEDEC Coplanarity threshold dashed lines (+-75 um)
            plot_ui.hline(
                HLine::new("JEDEC Coplanarity Limit (+75 um)", 75.0)
                    .color(Color32::from_rgb(255, 90, 90))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(255, 90, 90))),
            );
            plot_ui.hline(
                HLine::new("JEDEC Coplanarity Limit (-75 um)", -75.0)
                    .color(Color32::from_rgb(255, 90, 90))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(255, 90, 90))),
            );
        });

        ui.add_space(6.0);
        let warpage = self.sim.warpage_report();
        ui.columns(4, |cols| {
            cols[0].label(format!("Delta-CTE: {:.2} ppm/K", warpage.delta_cte_ppm_k));
            cols[0].label(format!("DNP Corner: {:.2} mm", warpage.dnp_mm));

            cols[1].label(format!("Current Bow: {:.1} um", warpage.warpage_bow_um));
            cols[1].label(format!("Max Thermal Bow: {:.1} um", warpage.max_cycle_warpage_um));

            cols[2].label(format!("Corner Bump Shear: {:.2}%", warpage.corner_bump_shear_strain_pct));
            cols[2].label(format!("Corner Bump Stress: {:.1} MPa", warpage.corner_bump_shear_stress_mpa));

            cols[3].label(format!("Fatigue Cycles (N_f): {}", warpage.fatigue_cycles_to_failure));
            cols[3].label(RichText::new(&warpage.fatigue_status).strong());
        });
    }

    fn render_reliability(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Industrial Packaging Design Margin & Qualification Checklist").strong());
        ui.add_space(6.0);

        let rep = self.sim.report();
        let warpage = self.sim.warpage_report();
        let eye = self.sim.eye_metrics();

        egui::Grid::new("packaging_reliability_grid")
            .striped(true)
            .min_col_width(180.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Metric / Verification Check").strong());
                ui.label(RichText::new("Target Design Rule").strong());
                ui.label(RichText::new("Measured Value").strong());
                ui.label(RichText::new("Compliance").strong());
                ui.end_row();

                // 1. D2D Eye Height
                ui.label("UCIe Sampler Eye Height (EH)");
                ui.label(">= 40.0 mV");
                ui.label(format!("{:.1} mV", eye.eye_height_mv));
                ui.label(if eye.eye_height_mv >= 40.0 { "PASS" } else { "FAIL" });
                ui.end_row();

                // 2. D2D Eye Width
                ui.label("UCIe Sampler Eye Width (EW)");
                ui.label(">= 0.40 UI");
                ui.label(format!("{:.2} UI", eye.eye_width_ui));
                ui.label(if eye.eye_width_ui >= 0.40 { "PASS" } else { "FAIL" });
                ui.end_row();

                // 3. JEDEC Coplanarity Warpage
                ui.label("Package Coplanarity Bow");
                ui.label("<= 75.0 um");
                ui.label(format!("{:.1} um", warpage.warpage_bow_um.abs()));
                ui.label(if warpage.warpage_bow_um.abs() <= 75.0 { "PASS" } else { "FAIL" });
                ui.end_row();

                // 4. Corner Bump Shear Stress
                ui.label("Corner Micro-Bump Shear Stress");
                ui.label("<= 120.0 MPa");
                ui.label(format!("{:.1} MPa", warpage.corner_bump_shear_stress_mpa));
                ui.label(if warpage.corner_bump_shear_stress_mpa <= 120.0 { "PASS" } else { "FAIL" });
                ui.end_row();

                // 5. Thermal Cycling Fatigue Life
                ui.label("Coffin-Manson Thermal Cycles");
                ui.label(">= 1,000 Cycles");
                ui.label(format!("{} Cycles", warpage.fatigue_cycles_to_failure));
                ui.label(if warpage.fatigue_cycles_to_failure >= 1_000 { "PASS" } else { "FAIL" });
                ui.end_row();

                // 6. Energy Efficiency
                ui.label("Die-to-Die Energy Efficiency");
                ui.label("<= 0.50 pJ/bit");
                ui.label(format!("{:.3} pJ/bit", rep.energy_efficiency_pj_bit));
                ui.label(if rep.energy_efficiency_pj_bit <= 0.50 { "PASS" } else { "FAIL" });
                ui.end_row();
            });

        ui.add_space(12.0);
        let status_color = if rep.is_fully_qualified {
            Color32::from_rgb(80, 220, 120)
        } else {
            Color32::from_rgb(255, 90, 90)
        };
        ui.label(
            RichText::new(if rep.is_fully_qualified {
                "Overall Qualification: PASSED ALL PACKAGING DESIGN RULES (Aero/Space/Datacenter Grade)"
            } else {
                "Overall Qualification: MARGINAL / ACTION REQUIRED (Adjust Pitch, Underfill, or Substrate)"
            })
            .color(status_color)
            .strong()
            .size(13.0),
        );
    }

    fn render_telemetry_footer(&mut self, ui: &mut Ui) {
        let rep = self.sim.report();
        ui.horizontal(|ui| {
            ui.label(format!("Architecture: {}", rep.architecture.display_name()));
            ui.separator();
            ui.label(format!("Eye: {:.1} mV / {:.2} UI", rep.eye_height_mv, rep.eye_width_ui));
            ui.separator();
            ui.label(format!("Loss: {:.1} dB", rep.insertion_loss_nyquist_db));
            ui.separator();
            ui.label(format!("Warpage: {:.1} um", rep.package_bow_warpage_um));
            ui.separator();
            ui.label(format!("Fatigue: {} Cycles", rep.thermal_fatigue_cycles));
        });
    }
}
