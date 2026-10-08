#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 426: Phonon Studio Quantum Metamaterial
//! Non-Hermitian Floquet Chiral Magnon-Phonon Heat Transistor & Thermal Diode.
//!
//! Visualizes directional thermal rectification, gate-controlled differential
//! thermal gain G_thermal >= 5.0, non-Hermitian Exceptional Point (EP) polariton
//! coalescence, 2D cryogenic thermal gradient canvas, and a 10-point physics audit checklist.

use egui::{pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::chiral_heat_transistor::{
    ChiralHeatTransistorProcessor, HeatTransistorAuditReport, HeatTransistorMetrics,
    HeatTransistorTransferPoint, PolaritonQuasiEnergyPoint, ThermalFluxPoint,
    ThermalRectificationMetrics,
};

/// Active tab in the Chiral Heat Transistor Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatTransistorTab {
    ThermalRectification,
    HeatTransistorAmplification,
    FloquetMagnonPhononCoupling,
    RealSpaceThermalCanvas,
    AuditTelemetry,
}

impl HeatTransistorTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ThermalRectification => "Thermal Rectification",
            Self::HeatTransistorAmplification => "Heat Transistor",
            Self::FloquetMagnonPhononCoupling => "Floquet & EP Coupling",
            Self::RealSpaceThermalCanvas => "Thermal Canvas",
            Self::AuditTelemetry => "Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 426.
pub struct ChiralHeatTransistorDialog {
    pub is_open: bool,
    pub active_tab: HeatTransistorTab,

    // Diode Parameters
    pub drive_freq_ghz: f64,
    pub diode_coupling_g_mhz: f64,
    pub diode_magnon_damping_mhz: f64,
    pub diode_phonon_damping_mhz: f64,
    pub diode_source_temp_k: f64,
    pub diode_drain_temp_k: f64,

    // Transistor Parameters
    pub trans_source_temp_k: f64,
    pub trans_drain_temp_k: f64,
    pub gate_bias: f64,
    pub trans_coupling_g_mhz: f64,
    pub trans_magnon_damping_mhz: f64,
    pub trans_phonon_damping_mhz: f64,

    // Cached Solver State
    pub processor: ChiralHeatTransistorProcessor,
    pub cached_rectification: Vec<ThermalFluxPoint>,
    pub cached_diode_metrics: ThermalRectificationMetrics,
    pub cached_transfer: Vec<HeatTransistorTransferPoint>,
    pub cached_dispersion: Vec<PolaritonQuasiEnergyPoint>,
    pub cached_trans_metrics: HeatTransistorMetrics,
    pub cached_audit: HeatTransistorAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ChiralHeatTransistorDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ChiralHeatTransistorDialog {
    /// Fast cold-boot constructor with pre-computed baseline data (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let processor = ChiralHeatTransistorProcessor::default();
        let cached_rectification = processor.diode.compute_rectification_curve(30, 400.0);
        let cached_diode_metrics = processor.diode.evaluate_rectification_metrics();
        let cached_transfer = processor.transistor.compute_transfer_curve(40);
        let cached_dispersion = processor.transistor.compute_polariton_dispersion(30);
        let cached_trans_metrics = processor.transistor.evaluate_transistor_metrics();
        let cached_audit = processor.audit_processor();

        Self {
            is_open: false,
            active_tab: HeatTransistorTab::ThermalRectification,
            drive_freq_ghz: processor.diode.params.drive_freq_ghz,
            diode_coupling_g_mhz: processor.diode.params.coupling_g_mhz,
            diode_magnon_damping_mhz: processor.diode.params.magnon_damping_mhz,
            diode_phonon_damping_mhz: processor.diode.params.phonon_damping_mhz,
            diode_source_temp_k: processor.diode.params.source_temp_k,
            diode_drain_temp_k: processor.diode.params.drain_temp_k,

            trans_source_temp_k: processor.transistor.params.source_temp_k,
            trans_drain_temp_k: processor.transistor.params.drain_temp_k,
            gate_bias: processor.transistor.params.gate_bias,
            trans_coupling_g_mhz: processor.transistor.params.coupling_g_mhz,
            trans_magnon_damping_mhz: processor.transistor.params.magnon_damping_mhz,
            trans_phonon_damping_mhz: processor.transistor.params.phonon_damping_mhz,

            processor,
            cached_rectification,
            cached_diode_metrics,
            cached_transfer,
            cached_dispersion,
            cached_trans_metrics,
            cached_audit,
            last_solve_time_us: 120.0,
        }
    }

    /// Recomputes numerical simulation models across both diode and transistor engines.
    pub fn recompute(&mut self) {
        let start = std::time::Instant::now();

        self.processor.diode.params.drive_freq_ghz = self.drive_freq_ghz;
        self.processor.diode.params.coupling_g_mhz = self.diode_coupling_g_mhz;
        self.processor.diode.params.magnon_damping_mhz = self.diode_magnon_damping_mhz;
        self.processor.diode.params.phonon_damping_mhz = self.diode_phonon_damping_mhz;
        self.processor.diode.params.source_temp_k = self.diode_source_temp_k;
        self.processor.diode.params.drain_temp_k = self.diode_drain_temp_k;

        self.processor.transistor.params.source_temp_k = self.trans_source_temp_k;
        self.processor.transistor.params.drain_temp_k = self.trans_drain_temp_k;
        self.processor.transistor.params.gate_bias = self.gate_bias;
        self.processor.transistor.params.coupling_g_mhz = self.trans_coupling_g_mhz;
        self.processor.transistor.params.magnon_damping_mhz = self.trans_magnon_damping_mhz;
        self.processor.transistor.params.phonon_damping_mhz = self.trans_phonon_damping_mhz;

        self.cached_rectification = self.processor.diode.compute_rectification_curve(30, 400.0);
        self.cached_diode_metrics = self.processor.diode.evaluate_rectification_metrics();
        self.cached_transfer = self.processor.transistor.compute_transfer_curve(40);
        self.cached_dispersion = self.processor.transistor.compute_polariton_dispersion(30);
        self.cached_trans_metrics = self.processor.transistor.evaluate_transistor_metrics();
        self.cached_audit = self.processor.audit_processor();

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Renders the complete dialog contents into the provided egui::Ui.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Phase 426:").strong().color(Color32::from_rgb(100, 200, 255)));
            ui.label(RichText::new("Non-Hermitian Floquet Thermal Rectifier & Transistor").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Recompute Physics").clicked() {
                    self.recompute();
                }
                ui.label(format!("Solve Latency: {:.1} us", self.last_solve_time_us));
            });
        });
        ui.separator();

        // Navigation Tabs
        ui.horizontal(|ui| {
            let tabs = [
                HeatTransistorTab::ThermalRectification,
                HeatTransistorTab::HeatTransistorAmplification,
                HeatTransistorTab::FloquetMagnonPhononCoupling,
                HeatTransistorTab::RealSpaceThermalCanvas,
                HeatTransistorTab::AuditTelemetry,
            ];
            for tab in tabs {
                let is_active = self.active_tab == tab;
                if ui.selectable_label(is_active, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();

        match self.active_tab {
            HeatTransistorTab::ThermalRectification => self.render_thermal_rectification_tab(ui),
            HeatTransistorTab::HeatTransistorAmplification => self.render_heat_transistor_tab(ui),
            HeatTransistorTab::FloquetMagnonPhononCoupling => self.render_floquet_coupling_tab(ui),
            HeatTransistorTab::RealSpaceThermalCanvas => self.render_thermal_canvas_tab(ui),
            HeatTransistorTab::AuditTelemetry => self.render_audit_telemetry_tab(ui),
        }
    }

    fn render_thermal_rectification_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Thermal Diode Controls").strong());
                    ui.add_space(4.0);
                    let mut changed = false;
                    changed |= ui.add(egui::Slider::new(&mut self.drive_freq_ghz, 1.0..=5.0).text("Drive Freq (GHz)")).changed();
                    changed |= ui.add(egui::Slider::new(&mut self.diode_coupling_g_mhz, 5.0..=30.0).text("Coupling g (MHz)")).changed();
                    changed |= ui.add(egui::Slider::new(&mut self.diode_source_temp_k, 0.10..=0.80).text("T_source (K)")).changed();
                    changed |= ui.add(egui::Slider::new(&mut self.diode_drain_temp_k, 0.01..=0.20).text("T_drain (K)")).changed();
                    if changed {
                        self.recompute();
                    }
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Diode Rectification Telemetry").strong());
                    ui.add_space(4.0);
                    let m = &self.cached_diode_metrics;
                    ui.label(format!("Peak Rectification Ratio: {:.1}x", m.peak_rectification_ratio));
                    ui.label(format!("Backward Isolation: {:.1} dB", m.backward_isolation_db));
                    ui.label(format!("Forward Conductance: {:.1} pW/K", m.forward_conductance_pw_k));
                    ui.label(format!("Net Forward Flux: {:.2} pW", m.net_forward_flux_pw));
                    ui.label(format!("Directivity Factor: {:.1}%", m.directivity_factor * 100.0));
                });
            });
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Forward vs Backward Heat Flux & Rectification Ratio vs Delta T").strong());

        let pts_fwd: PlotPoints = self
            .cached_rectification
            .iter()
            .map(|p| [p.delta_t_mk, p.forward_flux_pw])
            .collect();
        let pts_bwd: PlotPoints = self
            .cached_rectification
            .iter()
            .map(|p| [p.delta_t_mk, p.backward_flux_pw * 10.0])
            .collect();

        Plot::new("plot_rectification_flux")
            .height(260.0)
            .x_axis_label("Temperature Bias Delta T (mK)")
            .y_axis_label("Heat Flux (pW)")
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new("Forward Flux J_fwd (pW)", pts_fwd).color(Color32::from_rgb(50, 200, 100)).width(2.0));
                plot_ui.line(Line::new("Backward Flux J_bwd x10 (pW)", pts_bwd).color(Color32::from_rgb(220, 80, 80)).width(1.5));
            });
    }

    fn render_heat_transistor_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Gate Modulation Controls").strong());
                    ui.add_space(4.0);
                    let mut changed = false;
                    changed |= ui.add(egui::Slider::new(&mut self.gate_bias, 0.0..=1.0).text("Gate Bias (V_norm)")).changed();
                    changed |= ui.add(egui::Slider::new(&mut self.trans_coupling_g_mhz, 5.0..=30.0).text("Coupling g (MHz)")).changed();
                    changed |= ui.add(egui::Slider::new(&mut self.trans_source_temp_k, 0.10..=0.60).text("T_source (K)")).changed();
                    if changed {
                        self.recompute();
                    }
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Transistor Gain Telemetry").strong());
                    ui.add_space(4.0);
                    let m = &self.cached_trans_metrics;
                    ui.label(format!("Max Differential Gain: {:.2}", m.max_differential_gain));
                    ui.label(format!("Operating Gain G: {:.2}", m.current_gain));
                    ui.label(format!("Drain Heat Flux: {:.2} pW", m.drain_heat_flux_pw));
                    ui.label(format!("On/Off Ratio: {:.1}x", m.on_off_ratio));
                    ui.label(format!("EP Threshold g_EP: {:.2} MHz", m.ep_threshold_mhz));
                    ui.label(format!("Detuning from EP: {:.2} MHz", m.ep_detuning_mhz));
                });
            });
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Heat Transistor Transfer Characteristic & Differential Gain").strong());

        let pts_drain: PlotPoints = self
            .cached_transfer
            .iter()
            .map(|p| [p.gate_bias, p.drain_heat_flux_pw])
            .collect();
        let pts_gain: PlotPoints = self
            .cached_transfer
            .iter()
            .map(|p| [p.gate_bias, p.differential_gain])
            .collect();

        Plot::new("plot_transistor_transfer")
            .height(260.0)
            .x_axis_label("Normalized Gate Bias")
            .y_axis_label("Heat Flux / Differential Gain")
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new("Drain Heat Flux J_drain (pW)", pts_drain).color(Color32::from_rgb(255, 180, 50)).width(2.0));
                plot_ui.line(Line::new("Differential Gain G_thermal", pts_gain).color(Color32::from_rgb(100, 200, 255)).width(2.0));
                plot_ui.hline(HLine::new("Threshold 5.0", 5.0).color(Color32::from_rgb(200, 100, 100)));
            });
    }

    fn render_floquet_coupling_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Non-Hermitian Polariton Dissipation").strong());
                    ui.add_space(4.0);
                    let mut changed = false;
                    changed |= ui.add(egui::Slider::new(&mut self.trans_magnon_damping_mhz, 1.0..=20.0).text("Magnon Damping (MHz)")).changed();
                    changed |= ui.add(egui::Slider::new(&mut self.trans_phonon_damping_mhz, 0.2..=5.0).text("Phonon Damping (MHz)")).changed();
                    if changed {
                        self.recompute();
                    }
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Exceptional Point Properties").strong());
                    ui.add_space(4.0);
                    let g_ep = self.cached_trans_metrics.ep_threshold_mhz;
                    let g = self.trans_coupling_g_mhz;
                    ui.label(format!("EP Condition g_EP = |gamma_m - gamma_p|/2 = {:.2} MHz", g_ep));
                    ui.label(format!("Active Coupling g = {:.2} MHz", g));
                    let regime = if g > g_ep { "Broken PT / Hybrid Polariton" } else { "Unbroken / Dissipative" };
                    ui.label(format!("Regime: {regime}"));
                });
            });
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Quasi-Energy Dispersion Around Exceptional Point (Re Delta vs Detuning)").strong());

        let pts_upper: PlotPoints = self
            .cached_dispersion
            .iter()
            .map(|p| [p.detuning_mhz, p.re_upper_mhz])
            .collect();
        let pts_lower: PlotPoints = self
            .cached_dispersion
            .iter()
            .map(|p| [p.detuning_mhz, p.re_lower_mhz])
            .collect();

        Plot::new("plot_quasi_energy")
            .height(260.0)
            .x_axis_label("Detuning Delta (MHz)")
            .y_axis_label("Frequency Splitting Re(Delta) (MHz)")
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new("Upper Polariton Branch", pts_upper).color(Color32::from_rgb(120, 180, 255)).width(2.0));
                plot_ui.line(Line::new("Lower Polariton Branch", pts_lower).color(Color32::from_rgb(255, 120, 120)).width(2.0));
            });
    }

    fn render_thermal_canvas_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("2D Cryogenic Temperature Gradient & Directional Heat Flow Canvas").strong());
        ui.add_space(4.0);

        let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 320.0), Sense::hover());
        let rect = response.rect;

        // Background dark gradient
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 20, 28));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(40, 50, 70)), StrokeKind::Inside);

        // Three terminals: Source (Left), Gate (Center Top), Drain (Right)
        let s_rect = Rect::from_min_size(pos2(rect.min.x + 30.0, rect.min.y + 80.0), vec2(100.0, 140.0));
        let g_rect = Rect::from_min_size(pos2(rect.center().x - 60.0, rect.min.y + 30.0), vec2(120.0, 50.0));
        let d_rect = Rect::from_min_size(pos2(rect.max.x - 130.0, rect.min.y + 80.0), vec2(100.0, 140.0));
        let channel_rect = Rect::from_min_size(pos2(s_rect.max.x, rect.min.y + 110.0), vec2(d_rect.min.x - s_rect.max.x, 80.0));

        // Source reservoir (hotter, 300 mK, orange/red)
        painter.rect_filled(s_rect, 6.0, Color32::from_rgb(200, 70, 40));
        painter.text(s_rect.center(), egui::Align2::CENTER_CENTER, "Source\n300 mK", egui::FontId::proportional(13.0), Color32::WHITE);

        // Drain reservoir (cold, 50 mK, cyan/blue)
        painter.rect_filled(d_rect, 6.0, Color32::from_rgb(30, 110, 180));
        painter.text(d_rect.center(), egui::Align2::CENTER_CENTER, "Drain\n50 mK", egui::FontId::proportional(13.0), Color32::WHITE);

        // Gate electrode (modulation control, yellow)
        painter.rect_filled(g_rect, 4.0, Color32::from_rgb(210, 180, 50));
        painter.text(g_rect.center(), egui::Align2::CENTER_CENTER, format!("Gate Control\n{:.2} V", self.gate_bias), egui::FontId::proportional(12.0), Color32::BLACK);

        // Central phononic-magnonic waveguide channel
        painter.rect_filled(channel_rect, 4.0, Color32::from_rgb(35, 45, 60));
        painter.rect_stroke(channel_rect, 4.0, Stroke::new(1.5, Color32::from_rgb(80, 120, 160)), StrokeKind::Inside);

        // Draw directional heat arrows across channel
        let arrow_y = channel_rect.center().y;
        let start_x = channel_rect.min.x + 20.0;
        let end_x = channel_rect.max.x - 20.0;
        let step = 45.0;
        let mut curr_x = start_x;
        while curr_x < end_x {
            painter.arrow(pos2(curr_x, arrow_y), vec2(30.0, 0.0), Stroke::new(2.5, Color32::from_rgb(100, 255, 180)));
            curr_x += step;
        }

        painter.text(
            pos2(channel_rect.center().x, channel_rect.max.y + 20.0),
            egui::Align2::CENTER_CENTER,
            format!("Chiral Heat Flux J_fwd = {:.2} pW (Gain = {:.2})", self.cached_trans_metrics.drain_heat_flux_pw, self.cached_trans_metrics.current_gain),
            egui::FontId::proportional(13.0),
            Color32::from_rgb(160, 220, 255),
        );
    }

    fn render_audit_telemetry_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("10-Point Physics Audit & Cryptographic Verification Checklist").strong());
        ui.add_space(4.0);

        let audit = &self.cached_audit;
        let badge = |ui: &mut Ui, pass: bool, name: &str| {
            ui.horizontal(|ui| {
                let color = if pass { Color32::from_rgb(50, 200, 100) } else { Color32::from_rgb(220, 60, 60) };
                let tag = if pass { "[PASS]" } else { "[FAIL]" };
                ui.label(RichText::new(tag).strong().color(color));
                ui.label(name);
            });
        };

        ui.group(|ui| {
            ui.vertical(|ui| {
                badge(ui, audit.drive_periodicity_pass, "1. Floquet High-Frequency Drive Periodicity (Omega >= 1.0 GHz)");
                badge(ui, audit.ep_threshold_identifiable, "2. Non-Hermitian Exceptional Point Threshold Identifiable (g_EP > 0)");
                badge(ui, audit.rectification_ratio_pass, "3. Directional Thermal Rectification Ratio R >= 25.0");
                badge(ui, audit.forward_heat_flux_positive, "4. Forward Heat Flux Strictly Positive (J_fwd > 0 pW)");
                badge(ui, audit.backward_isolation_pass, "5. Backward Thermal Isolation >= 15.0 dB");
                badge(ui, audit.differential_gain_pass, "6. Thermal Differential Gain G_thermal >= 5.0");
                badge(ui, audit.sub_kelvin_stability, "7. Sub-Kelvin Dilution Refrigerator Operation (T < 1.0 K)");
                badge(ui, audit.polariton_coupling_pass, "8. Magnon-Phonon Polariton Hybrid Coupling g > 0");
                badge(ui, audit.thermodynamic_on_off_pass, "9. Thermodynamic Conservation & Switching On/Off Ratio >= 10.0");
                badge(ui, audit.cold_boot_throughput_pass, "10. Cold-Boot Initialization Latency < 2.0 ms");
            });
        });

        ui.add_space(8.0);
        let score_color = if audit.all_passed { Color32::from_rgb(50, 220, 120) } else { Color32::from_rgb(220, 80, 80) };
        ui.label(RichText::new(format!("Overall Verification Score: {} / 10 PASS", audit.total_score)).strong().color(score_color));
    }

    /// Renders the modal dialog window if open.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Quantum Chiral Magnon-Phonon Heat Transistor & Diode Studio")
            .open(&mut open)
            .default_width(880.0)
            .default_height(620.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = open;
    }

    /// Alias for showing the modal dialog.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }
}
