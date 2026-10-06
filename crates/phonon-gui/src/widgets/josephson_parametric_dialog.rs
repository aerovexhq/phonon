#![deny(unsafe_code)]

//! Interactive 5-Tab Superconducting Josephson Parametric Acoustic Waveguide Amplification
//! & Quantum Squeezed Vacuum Engine Studio Dialog for Phonon CAD.
//!
//! Provides:
//! - Tab 1: SQUID Loop & Flux Tuning: 2D curve of resonant frequency f_0(Phi) vs magnetic flux Phi / Phi_0,
//!   SQUID loop schematic diagram with Josephson junction markers, bias line, and interactive flux bias slider.
//! - Tab 2: Parametric Gain & Bandwidth: egui_plot rendering gain curve G(f) in dB showing > 20 dB gain peak,
//!   3-dB bandwidth markers, and pump power ratio slider.
//! - Tab 3: Squeezed Vacuum & Wigner Map: 2D interactive phase-space canvas rendering Wigner function W(X, P)
//!   with Turbo/Magma elliptical intensity contours, polar quadrature variance scan Delta_X_theta^2 with dashed
//!   vacuum SQL reference line, and squeezing depth gauge (>= 6 dB).
//! - Tab 4: CV Entanglement & Cluster State: Dual-mode phase space representation, EPR nullifier bar chart
//!   (< 1.0 inseparability threshold), dilution refrigerator temperature slider (10 mK to 1.0 K), and added noise quanta meter.
//! - Tab 5: Physics Audit & Telemetry: 10-point physics audit checklist with 10/10 PASS score, instantaneous
//!   cold boot (< 2ms latency), and interactive parameter controls.

use std::f64::consts::PI;
use egui::{
    pos2, vec2, Align2, Color32, FontId, ProgressBar, Rect, RichText, Sense, Stroke, StrokeKind,
    Ui,
};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};

use phonon_solver::josephson_parametric_amplifier::{
    JpaAuditReport, JpaWaveguideParams, JosephsonInductanceModel,
    JosephsonParametricProcessor, WignerQuasiProbability, VACUUM_SQL_VARIANCE,
};

/// Studio theme color palette.
const COLOR_ACTIVE_GREEN: Color32 = Color32::from_rgb(52, 211, 153); // Emerald
const COLOR_CYAN_ACCENT: Color32 = Color32::from_rgb(56, 189, 248);  // Cyan
const COLOR_WARN_ROSE: Color32 = Color32::from_rgb(244, 63, 94);     // Rose
const COLOR_AMBER: Color32 = Color32::from_rgb(250, 204, 21);        // Amber
const COLOR_PURPLE: Color32 = Color32::from_rgb(192, 132, 252);      // Purple
const COLOR_CANVAS_BG: Color32 = Color32::from_rgb(15, 23, 42);      // Slate 900
const COLOR_CANVAS_RIM: Color32 = Color32::from_rgb(51, 65, 85);     // Slate 700
const COLOR_TEXT_DIM: Color32 = Color32::from_rgb(148, 163, 184);    // Slate 400

/// Colormap modes for 2D Wigner quasi-probability rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WignerColormap {
    #[default]
    Turbo,
    Magma,
    CoolWarm,
}

impl WignerColormap {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Turbo => "Turbo Colormap",
            Self::Magma => "Magma Colormap",
            Self::CoolWarm => "Cool-Warm Colormap",
        }
    }
}

/// Evaluates normalized scalar value [0.0, 1.0] to Color32.
pub fn map_wigner_color(t_norm: f32, map: WignerColormap) -> Color32 {
    let t = t_norm.clamp(0.0, 1.0);
    match map {
        WignerColormap::Turbo => {
            let r = (34.61 + t * (1172.0 + t * (1072.0 - t * (1434.0 + t * (2313.0 - t * 1038.0)))))
                .clamp(0.0, 255.0) as u8;
            let g = (23.31 + t * (557.3 + t * (1074.0 - t * (2457.0 - t * (1228.0 - t * 545.0)))))
                .clamp(0.0, 255.0) as u8;
            let b = (27.2 + t * (3211.0 - t * (15327.0 - t * (27814.0 - t * (22569.0 - t * 6832.0)))))
                .clamp(0.0, 255.0) as u8;
            Color32::from_rgb(r, g, b)
        }
        WignerColormap::Magma => {
            let r = (255.0 * (1.1 * t).clamp(0.0, 1.0)) as u8;
            let g = (255.0 * (t.powf(1.8) * 0.9).clamp(0.0, 1.0)) as u8;
            let b = (255.0 * (0.3 + 0.7 * (1.0 - t).powi(2)).clamp(0.0, 1.0)) as u8;
            Color32::from_rgb(r, g, b)
        }
        WignerColormap::CoolWarm => {
            let r = (50.0 + 205.0 * t) as u8;
            let g = (60.0 + 120.0 * (1.0 - (2.0 * t - 1.0).abs())) as u8;
            let b = (220.0 - 180.0 * t) as u8;
            Color32::from_rgb(r, g, b)
        }
    }
}

/// Active visual tab in the Josephson Parametric Amplifier Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JosephsonParametricTab {
    #[default]
    SquidFluxTuning,
    ParametricGainBandwidth,
    SqueezedVacuumWigner,
    CvEntanglementClusterState,
    PhysicsAuditTelemetry,
}

impl JosephsonParametricTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SquidFluxTuning => "1. SQUID Loop & Flux Tuning",
            Self::ParametricGainBandwidth => "2. Parametric Gain & Bandwidth",
            Self::SqueezedVacuumWigner => "3. Squeezed Vacuum & Wigner Map",
            Self::CvEntanglementClusterState => "4. CV Entanglement & Cluster State",
            Self::PhysicsAuditTelemetry => "5. Physics Audit & Telemetry",
        }
    }
}

/// Modal dialog state for the Superconducting Josephson Parametric Waveguide.
#[derive(Debug, Clone, PartialEq)]
pub struct JosephsonParametricDialog {
    /// Modal dialog open status.
    pub is_open: bool,
    /// Currently active visual tab.
    pub active_tab: JosephsonParametricTab,
    /// Master orchestrator engine.
    pub processor: JosephsonParametricProcessor,
    /// Active colormap for phase space rendering.
    pub colormap: WignerColormap,
    /// Wigner grid dimension (e.g. 35 x 35).
    pub wigner_grid_size: usize,
    /// Dynamic animation phase.
    pub anim_phase: f64,
    /// Cached frequency tuning curve points [Phi/Phi_0, f_0(GHz)].
    pub cached_tuning_curve: Vec<[f64; 2]>,
    /// Cached inductance curve points [Phi/Phi_0, L_eff(pH)].
    pub cached_inductance_curve: Vec<[f64; 2]>,
    /// Cached parametric gain spectrum points [f(GHz), Gain(dB)].
    pub cached_gain_spectrum: Vec<[f64; 2]>,
    /// Cached 2D Wigner quasi-probability distribution.
    pub cached_wigner: WignerQuasiProbability,
    /// Cached polar quadrature variance scan points [theta, var].
    pub cached_quadrature_scan: Vec<[f64; 2]>,
    /// Cached 10-point physics audit report.
    pub cached_audit: JpaAuditReport,
}

impl Default for JosephsonParametricDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl JosephsonParametricDialog {
    /// Constructs a new dialog with initial defaults and synthesized physics cache.
    pub fn new() -> Self {
        let processor = JosephsonParametricProcessor::default();
        let audit = processor.audit_jpa();
        let wigner = processor.compute_wigner(35, 3.0);

        let mut dialog = Self {
            is_open: false,
            active_tab: JosephsonParametricTab::SquidFluxTuning,
            processor,
            colormap: WignerColormap::Turbo,
            wigner_grid_size: 35,
            anim_phase: 0.0,
            cached_tuning_curve: Vec::new(),
            cached_inductance_curve: Vec::new(),
            cached_gain_spectrum: Vec::new(),
            cached_wigner: wigner,
            cached_quadrature_scan: Vec::new(),
            cached_audit: audit,
        };
        dialog.refresh_simulation();
        dialog
    }

    /// Fast lightweight instantiation for GUI registration.
    pub fn new_fast() -> Self {
        Self::new()
    }

    /// Advances internal animation clock.
    pub fn advance_animation(&mut self, dt_seconds: f64) {
        self.anim_phase = (self.anim_phase + dt_seconds * 3.0) % (2.0 * PI);
    }

    /// Re-evaluates tuning curves, gain spectra, Wigner maps, and audits.
    pub fn refresh_simulation(&mut self) {
        self.processor.update_params();

        // 1. SQUID flux tuning and inductance curves across Phi in [-0.45, 0.45]
        self.cached_tuning_curve = self.processor.compute_tuning_curve(100);
        self.cached_inductance_curve =
            JosephsonInductanceModel::compute_inductance_curve(&self.processor.waveguide_params, 100);

        // 2. Gain spectrum across +/- 200 MHz of center resonance
        let f_0 = self.processor.waveguide_params.resonant_frequency_ghz();
        let span_ghz = 0.25;
        self.cached_gain_spectrum = self.processor.compute_gain_spectrum(
            f_0 - span_ghz,
            f_0 + span_ghz,
            120,
        );

        // 3. Wigner quasi-probability distribution
        self.cached_wigner = self.processor.compute_wigner(self.wigner_grid_size, 3.0);

        // 4. Polar quadrature variance scan
        self.cached_quadrature_scan = self.processor.compute_quadrature_scan(100);

        // 5. 10-point physics audit
        self.cached_audit = self.processor.audit_jpa();
    }

    /// Modal window GUI rendering pass.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        self.advance_animation(0.016);

        let mut is_open = self.is_open;
        egui::Window::new("Superconducting Josephson Parametric Acoustic Waveguide Studio")
            .open(&mut is_open)
            .default_size(vec2(1000.0, 720.0))
            .min_size(vec2(860.0, 600.0))
            .show(ctx, |ui| {
                self.render_dialog_contents(ui);
            });
        self.is_open = is_open;
    }

    /// Rendering of dialog tab bar, contents, and telemetry footer.
    pub fn render_dialog_contents(&mut self, ui: &mut Ui) {
        // Tab Bar
        ui.horizontal(|ui| {
            let tabs = [
                JosephsonParametricTab::SquidFluxTuning,
                JosephsonParametricTab::ParametricGainBandwidth,
                JosephsonParametricTab::SqueezedVacuumWigner,
                JosephsonParametricTab::CvEntanglementClusterState,
                JosephsonParametricTab::PhysicsAuditTelemetry,
            ];
            for tab in tabs {
                let is_selected = self.active_tab == tab;
                if ui
                    .selectable_label(is_selected, RichText::new(tab.label()).strong())
                    .clicked()
                {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        // Tab Content
        match self.active_tab {
            JosephsonParametricTab::SquidFluxTuning => self.render_tab_squid_flux_tuning(ui),
            JosephsonParametricTab::ParametricGainBandwidth => self.render_tab_parametric_gain(ui),
            JosephsonParametricTab::SqueezedVacuumWigner => self.render_tab_squeezed_vacuum(ui),
            JosephsonParametricTab::CvEntanglementClusterState => self.render_tab_cv_entanglement(ui),
            JosephsonParametricTab::PhysicsAuditTelemetry => self.render_tab_audit_telemetry(ui),
        }

        ui.separator();

        // Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Tab 1: SQUID Loop & Flux Tuning.
    fn render_tab_squid_flux_tuning(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("SQUID Resonant Frequency Tuning Curve & Loop Geometry").heading());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Reset Defaults").clicked() {
                    self.processor.waveguide_params = JpaWaveguideParams::default();
                    self.refresh_simulation();
                }
            });
        });

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Flux Bias Phi / Phi_0:");
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.processor.waveguide_params.flux_bias_ratio,
                        -0.45..=0.45,
                    )
                    .step_by(0.01),
                )
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Critical Current I_c:");
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.processor.waveguide_params.critical_current_ua,
                        1.0..=5.0,
                    )
                    .suffix(" uA"),
                )
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Shunt C:");
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.processor.waveguide_params.shunt_capacitance_pf,
                        0.2..=2.0,
                    )
                    .suffix(" pF"),
                )
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Geometric L:");
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.processor.waveguide_params.geometric_inductance_ph,
                        10.0..=100.0,
                    )
                    .suffix(" pH"),
                )
                .changed()
            {
                changed = true;
            }
        });

        if changed {
            self.refresh_simulation();
        }

        ui.add_space(4.0);

        let avail_w = ui.available_width().max(400.0);
        let col_w = (avail_w - 16.0) * 0.5;
        let sub_h = 300.0;

        ui.horizontal(|ui| {
            // Left Panel: SQUID Loop Schematic Canvas
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("SQUID Schematic & Josephson Junctions").strong().color(COLOR_CYAN_ACCENT));
                self.render_squid_loop_diagram(ui, col_w, sub_h);
            });

            // Right Panel: 2D Frequency Tuning Curve Plot
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("Tunable Plasma Frequency f_0(Phi)").strong().color(COLOR_ACTIVE_GREEN));

                let pts: PlotPoints = self
                    .cached_tuning_curve
                    .iter()
                    .map(|p| [p[0], p[1]])
                    .collect();

                let line = Line::new("f_0(Phi)", pts)
                    .color(COLOR_ACTIVE_GREEN)
                    .width(2.5);

                let current_flux = self.processor.waveguide_params.flux_bias_ratio;
                let current_f = self.processor.waveguide_params.resonant_frequency_ghz();

                let plot = Plot::new("squid_tuning_plot")
                    .height(sub_h)
                    .x_axis_label("Magnetic Flux Bias Phi / Phi_0")
                    .y_axis_label("Resonant Frequency f_0 (GHz)")
                    .legend(Legend::default())
                    .show_axes([true, true]);

                plot.show(ui, |plot_ui| {
                    plot_ui.line(line);
                    plot_ui.vline(VLine::new("Bias Point", current_flux).color(COLOR_AMBER));
                    plot_ui.hline(HLine::new("Tuned f_0", current_f).color(COLOR_CYAN_ACCENT));
                });
            });
        });
    }

    /// Renders SQUID loop schematic diagram with Josephson junctions, bias coil, and flux markers.
    fn render_squid_loop_diagram(&self, ui: &mut Ui, w: f32, h: f32) {
        let (response, painter) = ui.allocate_painter(vec2(w, h), Sense::hover());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 6.0, COLOR_CANVAS_BG);
        painter.rect_stroke(rect, 6.0, Stroke::new(1.0, COLOR_CANVAS_RIM), StrokeKind::Inside);

        let center = rect.center();
        let loop_rx = 70.0;
        let loop_ry = 90.0;

        // Draw Superconducting SQUID Ring Loop
        let ring_rect = Rect::from_center_size(center, vec2(loop_rx * 2.0, loop_ry * 2.0));
        painter.rect_stroke(
            ring_rect,
            12.0,
            Stroke::new(4.0, Color32::from_rgb(56, 189, 248)),
            StrokeKind::Middle,
        );

        // Top and Bottom Superconducting Lead Traces
        let top_lead_tip = center - vec2(0.0, loop_ry + 35.0);
        let bot_lead_tip = center + vec2(0.0, loop_ry + 35.0);
        painter.line_segment([center - vec2(0.0, loop_ry), top_lead_tip], Stroke::new(4.0, COLOR_CYAN_ACCENT));
        painter.line_segment([center + vec2(0.0, loop_ry), bot_lead_tip], Stroke::new(4.0, COLOR_CYAN_ACCENT));

        // Josephson Junctions on Left and Right Arms (X markers)
        let jj_size = 10.0;
        let left_jj = center - vec2(loop_rx, 0.0);
        let right_jj = center + vec2(loop_rx, 0.0);

        for jj in [left_jj, right_jj] {
            // Background box clearing the wire
            painter.rect_filled(
                Rect::from_center_size(jj, vec2(jj_size * 2.2, jj_size * 2.2)),
                2.0,
                COLOR_CANVAS_BG,
            );
            // 'X' cross symbol
            painter.line_segment(
                [jj - vec2(jj_size, jj_size), jj + vec2(jj_size, jj_size)],
                Stroke::new(2.5, COLOR_AMBER),
            );
            painter.line_segment(
                [jj - vec2(jj_size, -jj_size), jj + vec2(jj_size, -jj_size)],
                Stroke::new(2.5, COLOR_AMBER),
            );
            painter.circle_filled(jj, 3.0, Color32::WHITE);
        }

        // Labels for Junctions
        painter.text(left_jj - vec2(16.0, 0.0), Align2::RIGHT_CENTER, "JJ_1 (I_c)", FontId::proportional(11.0), COLOR_AMBER);
        painter.text(right_jj + vec2(16.0, 0.0), Align2::LEFT_CENTER, "JJ_2 (I_c)", FontId::proportional(11.0), COLOR_AMBER);

        // Central Magnetic Flux Vector Phi_ext
        painter.circle_filled(center, 18.0, Color32::from_rgba_premultiplied(192, 132, 252, 60));
        painter.circle_stroke(center, 18.0, Stroke::new(1.5, COLOR_PURPLE));
        let flux_ratio = self.processor.waveguide_params.flux_bias_ratio;
        painter.text(
            center,
            Align2::CENTER_CENTER,
            format!("{:.2} Phi_0", flux_ratio),
            FontId::proportional(11.0),
            COLOR_PURPLE,
        );

        // Acoustic Waveguide Coupling Indicator on Right Side
        let wg_x = right_jj.x + 35.0;
        let idt_rect = Rect::from_min_max(pos2(wg_x, center.y - 45.0), pos2(wg_x + 18.0, center.y + 45.0));
        painter.rect_filled(idt_rect, 2.0, Color32::from_rgb(30, 41, 59));
        painter.rect_stroke(idt_rect, 2.0, Stroke::new(1.0, COLOR_ACTIVE_GREEN), StrokeKind::Inside);
        painter.text(
            idt_rect.center(),
            Align2::CENTER_CENTER,
            "SAW",
            FontId::proportional(10.0),
            COLOR_ACTIVE_GREEN,
        );

        // Coupling text
        let g_mhz = self.processor.waveguide_params.piezo_coupling_mhz;
        painter.text(
            pos2(wg_x + 9.0, idt_rect.max.y + 12.0),
            Align2::CENTER_CENTER,
            format!("g={:.1}MHz", g_mhz),
            FontId::proportional(10.0),
            COLOR_ACTIVE_GREEN,
        );

        // Legend at bottom left
        let f_0 = self.processor.waveguide_params.resonant_frequency_ghz();
        let l_eff = self.processor.waveguide_params.effective_inductance_ph();
        painter.text(
            rect.min + vec2(10.0, 10.0),
            Align2::LEFT_TOP,
            format!("f_0 = {:.3} GHz | L_eff = {:.1} pH", f_0, l_eff),
            FontId::proportional(11.0),
            Color32::WHITE,
        );
    }

    /// Tab 2: Parametric Gain & Bandwidth.
    fn render_tab_parametric_gain(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Parametric Amplification Response & Instantaneous Bandwidth").heading());
        });

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Pump Power Ratio:");
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.processor.waveguide_params.pump_power_ratio,
                        0.0..=0.95,
                    )
                    .step_by(0.01),
                )
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Pump Freq (GHz):");
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.processor.waveguide_params.pump_frequency_ghz,
                        4.0..=24.0,
                    )
                    .step_by(0.1),
                )
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("3-dB BW (MHz):");
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.processor.waveguide_params.bandwidth_3db_mhz,
                        20.0..=200.0,
                    )
                    .suffix(" MHz"),
                )
                .changed()
            {
                changed = true;
            }
        });

        if changed {
            self.refresh_simulation();
        }

        ui.add_space(4.0);

        // Metrics Banner
        let resp = &self.processor.amp_response;
        ui.horizontal(|ui| {
            ui.label(RichText::new("Signal Gain G_max:").strong());
            ui.label(
                RichText::new(format!("{:.2} dB", resp.gain_max_db))
                    .color(COLOR_ACTIVE_GREEN)
                    .strong(),
            );
            ui.separator();
            ui.label(RichText::new("De-amplified G_min:").strong());
            ui.label(
                RichText::new(format!("{:.2} dB", resp.gain_min_db))
                    .color(COLOR_CYAN_ACCENT)
                    .strong(),
            );
            ui.separator();
            ui.label(RichText::new("3-dB Bandwidth:").strong());
            ui.label(format!("{:.1} MHz", resp.bandwidth_3db_mhz));
            ui.separator();
            ui.label(RichText::new("Squeezing Parameter r:").strong());
            ui.label(format!("{:.3}", resp.squeezing_parameter_r));
        });

        ui.add_space(4.0);

        // Gain Curve egui_plot
        let pts: PlotPoints = self
            .cached_gain_spectrum
            .iter()
            .map(|p| [p[0], p[1]])
            .collect();

        let line = Line::new("G(f) Gain (dB)", pts)
            .color(COLOR_ACTIVE_GREEN)
            .width(2.5);

        let f_0 = resp.center_freq_ghz;
        let g_max = resp.gain_max_db;
        let g_3db = g_max - 3.0103;
        let bw_ghz = resp.bandwidth_3db_mhz * 1e-3;

        let plot = Plot::new("jpa_gain_plot")
            .height(340.0)
            .x_axis_label("Frequency f (GHz)")
            .y_axis_label("Parametric Signal Gain (dB)")
            .legend(Legend::default())
            .show_axes([true, true]);

        plot.show(ui, |plot_ui| {
            plot_ui.line(line);
            plot_ui.hline(HLine::new("G_max Peak", g_max).color(COLOR_AMBER));
            plot_ui.hline(HLine::new("3-dB Cutoff Level", g_3db).color(COLOR_CYAN_ACCENT));
            plot_ui.vline(VLine::new("f_low 3-dB", f_0 - 0.5 * bw_ghz).color(COLOR_WARN_ROSE));
            plot_ui.vline(VLine::new("f_high 3-dB", f_0 + 0.5 * bw_ghz).color(COLOR_WARN_ROSE));
        });
    }

    /// Tab 3: Squeezed Vacuum & Wigner Map.
    fn render_tab_squeezed_vacuum(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Quantum Squeezed Vacuum & 2D Wigner Phase Space").heading());
        });

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Squeezing Angle (rad):");
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.processor.squeezing_params.squeezing_angle_rad,
                        0.0..=PI,
                    )
                    .step_by(0.05),
                )
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Colormap:");
            egui::ComboBox::from_id_salt("wigner_cmap_combo")
                .selected_text(self.colormap.label())
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.colormap, WignerColormap::Turbo, WignerColormap::Turbo.label()).clicked() {
                        changed = true;
                    }
                    if ui.selectable_value(&mut self.colormap, WignerColormap::Magma, WignerColormap::Magma.label()).clicked() {
                        changed = true;
                    }
                    if ui.selectable_value(&mut self.colormap, WignerColormap::CoolWarm, WignerColormap::CoolWarm.label()).clicked() {
                        changed = true;
                    }
                });

            ui.separator();

            ui.label("Grid Resolution:");
            egui::ComboBox::from_id_salt("wigner_res_combo")
                .selected_text(format!("{} x {}", self.wigner_grid_size, self.wigner_grid_size))
                .show_ui(ui, |ui| {
                    for res in [25, 35, 45] {
                        if ui.selectable_value(&mut self.wigner_grid_size, res, format!("{} x {}", res, res)).clicked() {
                            changed = true;
                        }
                    }
                });
        });

        if changed {
            self.refresh_simulation();
        }

        ui.add_space(4.0);

        // Metrics Banner
        let resp = &self.processor.amp_response;
        ui.horizontal(|ui| {
            ui.label(RichText::new("Sub-SQL Squeezing:").strong());
            ui.label(
                RichText::new(format!("{:.2} dB", resp.squeezing_depth_db))
                    .color(COLOR_ACTIVE_GREEN)
                    .strong(),
            );
            ui.separator();
            ui.label(RichText::new("Delta X_min^2:").strong());
            ui.label(format!("{:.4}", resp.delta_x_min_sq));
            ui.separator();
            ui.label(RichText::new("Delta X_max^2:").strong());
            ui.label(format!("{:.4}", resp.delta_x_max_sq));
            ui.separator();
            ui.label(RichText::new("Heisenberg Product:").strong());
            let color = if resp.heisenberg_preserved { COLOR_ACTIVE_GREEN } else { COLOR_WARN_ROSE };
            ui.label(RichText::new(format!("{:.4}", resp.heisenberg_product)).color(color).strong());
            ui.label("(SQL = 0.0625)");
        });

        ui.add_space(4.0);

        let avail_w = ui.available_width().max(400.0);
        let col_w = (avail_w - 16.0) * 0.5;
        let sub_h = 300.0;

        ui.horizontal(|ui| {
            // Left Panel: 2D Wigner Function Heatmap Canvas
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("2D Wigner Quasi-Probability W(X, P)").strong().color(COLOR_CYAN_ACCENT));
                self.render_wigner_canvas(ui, col_w, sub_h);
            });

            // Right Panel: Polar Quadrature Variance Scan
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("Quadrature Variance vs Angle Delta X_theta^2").strong().color(COLOR_ACTIVE_GREEN));

                let pts: PlotPoints = self
                    .cached_quadrature_scan
                    .iter()
                    .map(|p| [p[0], p[1]])
                    .collect();

                let line = Line::new("Delta X_theta^2", pts)
                    .color(COLOR_ACTIVE_GREEN)
                    .width(2.5);

                let plot = Plot::new("quadrature_scan_plot")
                    .height(sub_h)
                    .x_axis_label("Quadrature Angle theta (rad)")
                    .y_axis_label("Variance (quanta)")
                    .legend(Legend::default())
                    .show_axes([true, true]);

                plot.show(ui, |plot_ui| {
                    plot_ui.line(line);
                    plot_ui.hline(HLine::new("Vacuum SQL (0.50)", VACUUM_SQL_VARIANCE).color(COLOR_AMBER));
                });
            });
        });
    }

    /// Renders 2D Wigner quasi-probability density heatmap onto an egui painter canvas.
    fn render_wigner_canvas(&self, ui: &mut Ui, w: f32, h: f32) {
        let (response, painter) = ui.allocate_painter(vec2(w, h), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 6.0, COLOR_CANVAS_BG);
        painter.rect_stroke(rect, 6.0, Stroke::new(1.0, COLOR_CANVAS_RIM), StrokeKind::Inside);

        let wigner = &self.cached_wigner;
        let n = wigner.grid_size;
        if n < 2 {
            return;
        }

        let margin = 20.0;
        let plot_rect = Rect::from_min_max(
            rect.min + vec2(margin, margin),
            rect.max - vec2(margin, margin),
        );

        let cell_w = plot_rect.width() / n as f32;
        let cell_h = plot_rect.height() / n as f32;

        let w_max = wigner.w_max.max(1e-9);
        let w_min = wigner.w_min;
        let span = (w_max - w_min).max(1e-9);

        // Render discrete phase space cells
        for ix in 0..n {
            let px = plot_rect.min.x + ix as f32 * cell_w;
            for ip in 0..n {
                let py = plot_rect.max.y - (ip + 1) as f32 * cell_h;
                let val = wigner.at(ix, ip);
                let t_norm = ((val - w_min) / span) as f32;
                let color = map_wigner_color(t_norm, self.colormap);

                painter.rect_filled(
                    Rect::from_min_size(pos2(px, py), vec2(cell_w + 0.5, cell_h + 0.5)),
                    0.0,
                    color,
                );
            }
        }

        // Draw Phase Space Axes (X and P) through the center
        let center = plot_rect.center();
        painter.line_segment(
            [pos2(plot_rect.min.x, center.y), pos2(plot_rect.max.x, center.y)],
            Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 120)),
        );
        painter.line_segment(
            [pos2(center.x, plot_rect.min.y), pos2(center.x, plot_rect.max.y)],
            Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 120)),
        );

        // Axis labels
        painter.text(pos2(plot_rect.max.x - 4.0, center.y + 12.0), Align2::RIGHT_CENTER, "X (quadrature)", FontId::proportional(10.0), Color32::WHITE);
        painter.text(pos2(center.x + 8.0, plot_rect.min.y + 8.0), Align2::LEFT_TOP, "P (momentum)", FontId::proportional(10.0), Color32::WHITE);

        // Aspect ratio label
        painter.text(
            rect.min + vec2(8.0, 8.0),
            Align2::LEFT_TOP,
            format!("Aspect: {:.2} | Integral: {:.3}", wigner.aspect_ratio, wigner.total_integral),
            FontId::proportional(11.0),
            COLOR_AMBER,
        );
    }

    /// Tab 4: CV Entanglement & Cluster State.
    fn render_tab_cv_entanglement(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Continuous-Variable EPR Entanglement & Cluster States").heading());
        });

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Dilution Temperature:");
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.processor.squeezing_params.operating_temp_k,
                        0.010..=1.000,
                    )
                    .step_by(0.01)
                    .suffix(" K"),
                )
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Beam Splitter R:");
            if ui
                .add(
                    egui::Slider::new(
                        &mut self.processor.cluster_params.beam_splitter_reflectivity,
                        0.10..=0.90,
                    )
                    .step_by(0.02),
                )
                .changed()
            {
                changed = true;
            }

            ui.separator();

            ui.label("Mode Count:");
            egui::ComboBox::from_id_salt("mode_count_combo")
                .selected_text(format!("{}-Mode", self.processor.cluster_params.mode_count))
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.processor.cluster_params.mode_count, 2, "2-Mode EPR Pair").clicked() {
                        changed = true;
                    }
                    if ui.selectable_value(&mut self.processor.cluster_params.mode_count, 4, "4-Mode Cluster State").clicked() {
                        changed = true;
                    }
                });
        });

        if changed {
            self.refresh_simulation();
        }

        ui.add_space(4.0);

        let m = &self.processor.entanglement_metrics;

        // Entanglement Status Cards
        ui.horizontal(|ui| {
            let status_color = if m.inseparability_certified {
                COLOR_ACTIVE_GREEN
            } else {
                COLOR_WARN_ROSE
            };
            let status_text = if m.inseparability_certified {
                "GENUINE CV ENTANGLED"
            } else {
                "SEPARABLE / CLASSICAL"
            };

            ui.label(RichText::new(format!("[{}]", status_text)).color(status_color).strong());
            ui.separator();
            ui.label(RichText::new("EPR Nullifier:").strong());
            ui.label(RichText::new(format!("{:.4}", m.epr_nullifier_variance)).color(status_color).strong());
            ui.label("(< 1.0 threshold)");
            ui.separator();
            ui.label(RichText::new("Added Noise:").strong());
            ui.label(format!("{:.4} quanta", m.added_noise_quanta));
            ui.separator();
            ui.label(RichText::new("Thermal n_th:").strong());
            ui.label(format!("{:.2e}", m.thermal_occupancy_n_th));
        });

        ui.add_space(6.0);

        let avail_w = ui.available_width().max(400.0);
        let col_w = (avail_w - 16.0) * 0.5;
        let sub_h = 280.0;

        ui.horizontal(|ui| {
            // Left Panel: Dual-Mode Phase Space Representation
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("Dual-Mode Acoustic Phase Space").strong().color(COLOR_CYAN_ACCENT));
                self.render_dual_mode_canvas(ui, col_w, sub_h);
            });

            // Right Panel: Duan-Simon Inseparability Meter & Telemetry
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.label(RichText::new("Duan-Simon Inseparability Nullifier Meter").strong().color(COLOR_ACTIVE_GREEN));

                ui.add_space(10.0);
                ui.label(RichText::new("EPR Inseparability Criterion:").strong());
                ui.label("Delta(X_1 - X_2)^2 + Delta(P_1 + P_2)^2 < 1.0");

                ui.add_space(12.0);
                let nullifier_ratio = (m.epr_nullifier_variance / 1.0).clamp(0.0, 1.0) as f32;
                ui.label(format!("Normalized Variance: {:.1}% of Classical Boundary", nullifier_ratio * 100.0));
                ui.add(ProgressBar::new(nullifier_ratio).animate(true));

                ui.add_space(14.0);
                ui.label(RichText::new("Cluster State Fidelity:").strong());
                ui.add(ProgressBar::new(m.cluster_fidelity as f32).text(format!("{:.1}%", m.cluster_fidelity * 100.0)));

                ui.add_space(14.0);
                ui.label(RichText::new("Cryogenic Added Noise Meter:").strong());
                let noise_ratio = (m.added_noise_quanta / 0.55).clamp(0.0, 1.0) as f32;
                ui.add(ProgressBar::new(noise_ratio).text(format!("{:.3} / 0.550 quanta", m.added_noise_quanta)));

                ui.add_space(14.0);
                ui.label(RichText::new("1-dB Compression Dynamic Range:").strong());
                ui.label(format!("{:.1} dBm (P_1dB >= -110.0 dBm)", m.power_1db_compression_dbm));
            });
        });
    }

    /// Renders dual-mode phase space representation canvas.
    fn render_dual_mode_canvas(&self, ui: &mut Ui, w: f32, h: f32) {
        let (response, painter) = ui.allocate_painter(vec2(w, h), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 6.0, COLOR_CANVAS_BG);
        painter.rect_stroke(rect, 6.0, Stroke::new(1.0, COLOR_CANVAS_RIM), StrokeKind::Inside);

        let center = rect.center();
        let mode1_center = center - vec2(70.0, 0.0);
        let mode2_center = center + vec2(70.0, 0.0);

        // Draw Vacuum SQL circles (dashed reference)
        let r_sql = 40.0;
        painter.circle_stroke(mode1_center, r_sql, Stroke::new(1.2, Color32::from_rgba_premultiplied(250, 204, 21, 100)));
        painter.circle_stroke(mode2_center, r_sql, Stroke::new(1.2, Color32::from_rgba_premultiplied(250, 204, 21, 100)));

        // Squeezed ellipses for Mode 1 and Mode 2
        let resp = &self.processor.amp_response;
        let rx = (resp.delta_x_min_sq.sqrt() / 0.7071) as f32 * r_sql;
        let ry = (resp.delta_x_max_sq.sqrt() / 0.7071) as f32 * r_sql;

        // Mode 1: squeezed along X
        painter.rect_filled(Rect::from_center_size(mode1_center, vec2(rx * 2.0, ry * 2.0)), 12.0, Color32::from_rgba_premultiplied(52, 211, 153, 90));
        painter.rect_stroke(Rect::from_center_size(mode1_center, vec2(rx * 2.0, ry * 2.0)), 12.0, Stroke::new(2.0, COLOR_ACTIVE_GREEN), StrokeKind::Middle);

        // Mode 2: squeezed along P
        painter.rect_filled(Rect::from_center_size(mode2_center, vec2(ry * 2.0, rx * 2.0)), 12.0, Color32::from_rgba_premultiplied(56, 189, 248, 90));
        painter.rect_stroke(Rect::from_center_size(mode2_center, vec2(ry * 2.0, rx * 2.0)), 12.0, Stroke::new(2.0, COLOR_CYAN_ACCENT), StrokeKind::Middle);

        // Connecting entanglement beam splitter wave
        painter.line_segment([mode1_center + vec2(rx, 0.0), mode2_center - vec2(ry, 0.0)], Stroke::new(2.0, COLOR_PURPLE));
        painter.circle_filled(center, 5.0, COLOR_PURPLE);

        // Labels
        painter.text(mode1_center - vec2(0.0, r_sql + 14.0), Align2::CENTER_CENTER, "Mode 1 (SAW A)", FontId::proportional(11.0), COLOR_ACTIVE_GREEN);
        painter.text(mode2_center - vec2(0.0, r_sql + 14.0), Align2::CENTER_CENTER, "Mode 2 (SAW B)", FontId::proportional(11.0), COLOR_CYAN_ACCENT);
        painter.text(center + vec2(0.0, 20.0), Align2::CENTER_CENTER, "50:50 BS", FontId::proportional(10.0), COLOR_PURPLE);
    }

    /// Tab 5: Physics Audit & Telemetry.
    fn render_tab_audit_telemetry(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("10-Point Josephson Parametric Amplifier Physics Audit").heading());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Re-Run Verification Audit").clicked() {
                    self.refresh_simulation();
                }
            });
        });

        let audit = &self.cached_audit;

        ui.add_space(4.0);

        // Top Status Cards
        ui.horizontal(|ui| {
            let pass_color = if audit.overall_pass {
                COLOR_ACTIVE_GREEN
            } else {
                COLOR_WARN_ROSE
            };
            ui.label(
                RichText::new(format!("AUDIT STATUS: {} / {} PASS", audit.passed_count, audit.total_count))
                    .color(pass_color)
                    .strong()
                    .size(14.0),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("Cold Boot Latency: {:.1} us (< 2000 us)", audit.cold_boot_latency_us))
                    .color(COLOR_CYAN_ACCENT)
                    .strong(),
            );
            ui.separator();
            ui.label(RichText::new("Dilution Temp: 10 mK").color(COLOR_AMBER));
        });

        ui.add_space(6.0);

        // Audit Table
        egui::ScrollArea::vertical()
            .max_height(340.0)
            .show(ui, |ui| {
                egui::Grid::new("jpa_audit_grid")
                    .striped(true)
                    .min_col_width(80.0)
                    .spacing(vec2(12.0, 8.0))
                    .show(ui, |ui| {
                        // Header
                        ui.label(RichText::new("Status").strong());
                        ui.label(RichText::new("Audit Criterion").strong());
                        ui.label(RichText::new("Measured").strong());
                        ui.label(RichText::new("Target").strong());
                        ui.label(RichText::new("Units").strong());
                        ui.label(RichText::new("Physical Specification Description").strong());
                        ui.end_row();

                        // Criteria Rows
                        for c in &audit.criteria {
                            let status_color = if c.passed {
                                COLOR_ACTIVE_GREEN
                            } else {
                                COLOR_WARN_ROSE
                            };
                            let status_str = if c.passed { "[PASS]" } else { "[FAIL]" };

                            ui.label(RichText::new(status_str).color(status_color).strong());
                            ui.label(RichText::new(c.name).strong());
                            ui.label(format!("{:.4e}", c.measured_value));
                            ui.label(format!("{:.4e}", c.target_threshold));
                            ui.label(c.units);
                            ui.label(RichText::new(c.description).color(COLOR_TEXT_DIM));
                            ui.end_row();
                        }
                    });
            });
    }

    /// Telemetry Status Footer.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let p = &self.processor.waveguide_params;
        let r = &self.processor.amp_response;
        let m = &self.processor.entanglement_metrics;

        ui.horizontal(|ui| {
            ui.label(RichText::new("JPA Telemetry:").strong());
            ui.label(format!("f0: {:.2} GHz", p.resonant_frequency_ghz()));
            ui.label(format!("L_eff: {:.1} pH", p.effective_inductance_ph()));
            ui.separator();
            ui.label(
                RichText::new(format!("G_max: {:.2} dB", r.gain_max_db))
                    .color(COLOR_ACTIVE_GREEN),
            );
            ui.label(
                RichText::new(format!("Squeezing: {:.2} dB", r.squeezing_depth_db))
                    .color(COLOR_CYAN_ACCENT),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("EPR Nullifier: {:.3}", m.epr_nullifier_variance))
                    .color(if m.inseparability_certified { COLOR_ACTIVE_GREEN } else { COLOR_WARN_ROSE }),
            );
            ui.label(format!("n_add: {:.3} quanta", m.added_noise_quanta));
            ui.separator();
            ui.label(format!("T_bath: {:.0} mK", m.operating_temp_k * 1000.0));
        });
    }
}
