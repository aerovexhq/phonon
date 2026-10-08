#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 421: Phonon Studio Topological Acoustic
//! Floquet Higher-Order Corner-State Laser & Non-Hermitian Vortex Amplifier.
//!
//! Visualizes higher-order topological corner-state lasing with high SMSR (>= 35 dB),
//! non-reciprocal spatio-temporal Floquet vortex beam amplification (gain >= 22 dB),
//! 2D Laguerre-Gaussian orbital angular momentum (OAM) modal distributions, and
//! real-time physics audit telemetry.

use egui::{pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::floquet_corner_laser::{
    CornerLaserParams, CornerLasingMode, FloquetCornerLaserParams, FloquetCornerLaserProcessor,
    FloquetLaserAuditReport, LaserSpectralPoint, VortexAmplifierParams, VortexAmplifierPoint,
    VortexOamCharge,
};

/// Active tab within the Floquet Corner Laser & Vortex Amplifier Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloquetCornerLaserTab {
    CornerModeLasing,
    FloquetModulation,
    VortexAmplifier,
    RealSpaceMetamaterial,
    AuditTelemetry,
}

impl FloquetCornerLaserTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::CornerModeLasing => "Corner-State Lasing",
            Self::FloquetModulation => "Floquet Gain Modulation",
            Self::VortexAmplifier => "Non-Reciprocal Vortex Amplifier",
            Self::RealSpaceMetamaterial => "Real-Space Metamaterial",
            Self::AuditTelemetry => "Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 421.
pub struct FloquetCornerLaserDialog {
    pub is_open: bool,
    pub active_tab: FloquetCornerLaserTab,

    // Laser Parameters
    pub intracell_gamma_mhz: f64,
    pub intercell_lambda_mhz: f64,
    pub bare_frequency_ghz: f64,
    pub corner_gain_mhz: f64,
    pub bulk_loss_mhz: f64,
    pub pump_power_mw: f64,
    pub quality_factor: f64,

    // Vortex Amplifier Parameters
    pub modulation_freq_mhz: f64,
    pub dynamic_modulation_amplitude_mhz: f64,
    pub interaction_length_mm: f64,
    pub selected_charge: VortexOamCharge,

    // Solver & Cached Results
    pub processor: FloquetCornerLaserProcessor,
    pub cached_corner_modes: Vec<CornerLasingMode>,
    pub cached_emission_spectrum: Vec<LaserSpectralPoint>,
    pub cached_gain_spectrum: Vec<VortexAmplifierPoint>,
    pub cached_realspace_intensity: Vec<Vec<f64>>,
    pub cached_vortex_intensity: Vec<Vec<f64>>,
    pub cached_vortex_phase: Vec<Vec<f64>>,
    pub cached_audit: FloquetLaserAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for FloquetCornerLaserDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl FloquetCornerLaserDialog {
    /// Fast cold-boot constructor with pre-seeded baseline data (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let params = FloquetCornerLaserParams::default();
        let processor = FloquetCornerLaserProcessor::new(params.clone());

        let cached_corner_modes = processor.laser.solve_corner_modes();
        let cached_emission_spectrum = processor.laser.generate_emission_spectrum(51);
        let cached_gain_spectrum = processor.amplifier.generate_gain_spectrum(51);
        let cached_realspace_intensity = processor.laser.generate_realspace_intensity();
        let (cached_vortex_intensity, cached_vortex_phase) =
            processor.amplifier.generate_spatial_vortex_slice(24);
        let cached_audit = processor.audit_laser_amplifier();

        Self {
            is_open: false,
            active_tab: FloquetCornerLaserTab::CornerModeLasing,
            intracell_gamma_mhz: params.laser.intracell_gamma_mhz,
            intercell_lambda_mhz: params.laser.intercell_lambda_mhz,
            bare_frequency_ghz: params.laser.bare_frequency_ghz,
            corner_gain_mhz: params.laser.corner_gain_mhz,
            bulk_loss_mhz: params.laser.bulk_loss_mhz,
            pump_power_mw: params.laser.pump_power_mw,
            quality_factor: params.laser.quality_factor,
            modulation_freq_mhz: params.amplifier.modulation_freq_mhz,
            dynamic_modulation_amplitude_mhz: params.amplifier.dynamic_modulation_amplitude_mhz,
            interaction_length_mm: params.amplifier.interaction_length_mm,
            selected_charge: params.amplifier.target_charge,
            processor,
            cached_corner_modes,
            cached_emission_spectrum,
            cached_gain_spectrum,
            cached_realspace_intensity,
            cached_vortex_intensity,
            cached_vortex_phase,
            cached_audit,
            last_solve_time_us: 120.0,
        }
    }

    /// Recomputes all physics solvers and refreshes caches.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let params = FloquetCornerLaserParams {
            laser: CornerLaserParams {
                intracell_gamma_mhz: self.intracell_gamma_mhz,
                intercell_lambda_mhz: self.intercell_lambda_mhz,
                bare_frequency_ghz: self.bare_frequency_ghz,
                corner_gain_mhz: self.corner_gain_mhz,
                bulk_loss_mhz: self.bulk_loss_mhz,
                pump_power_mw: self.pump_power_mw,
                quality_factor: self.quality_factor,
                ..Default::default()
            },
            amplifier: VortexAmplifierParams {
                modulation_freq_mhz: self.modulation_freq_mhz,
                dynamic_modulation_amplitude_mhz: self.dynamic_modulation_amplitude_mhz,
                interaction_length_mm: self.interaction_length_mm,
                target_charge: self.selected_charge,
                ..Default::default()
            },
        };

        self.processor = FloquetCornerLaserProcessor::new(params);
        self.cached_corner_modes = self.processor.laser.solve_corner_modes();
        self.cached_emission_spectrum = self.processor.laser.generate_emission_spectrum(51);
        self.cached_gain_spectrum = self.processor.amplifier.generate_gain_spectrum(51);
        self.cached_realspace_intensity = self.processor.laser.generate_realspace_intensity();
        let (v_int, v_phase) = self.processor.amplifier.generate_spatial_vortex_slice(24);
        self.cached_vortex_intensity = v_int;
        self.cached_vortex_phase = v_phase;
        self.cached_audit = self.processor.audit_laser_amplifier();

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Primary display method following CAD dialog conventions.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Topological Floquet Corner Laser & Vortex Amplifier")
            .open(&mut is_open)
            .default_size(vec2(940.0, 680.0))
            .min_size(vec2(800.0, 560.0))
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = is_open;
    }

    /// Alias for show, following CAD dialog conventions.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Public render method for direct UI integration and headless testing.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        // Navigation Bar
        ui.horizontal(|ui| {
            for tab in &[
                FloquetCornerLaserTab::CornerModeLasing,
                FloquetCornerLaserTab::FloquetModulation,
                FloquetCornerLaserTab::VortexAmplifier,
                FloquetCornerLaserTab::RealSpaceMetamaterial,
                FloquetCornerLaserTab::AuditTelemetry,
            ] {
                let is_selected = self.active_tab == *tab;
                let text = if is_selected {
                    RichText::new(tab.label()).color(Color32::from_rgb(96, 165, 250)).strong()
                } else {
                    RichText::new(tab.label())
                };
                if ui.selectable_label(is_selected, text).clicked() {
                    self.active_tab = *tab;
                }
            }
        });
        ui.separator();

        // Preset Toolbar
        ui.horizontal(|ui| {
            ui.label(RichText::new("Presets:").color(Color32::from_rgb(148, 163, 184)));
            if ui.button("Topological Corner Laser (SMSR 42 dB)").clicked() {
                self.intracell_gamma_mhz = 2.0;
                self.intercell_lambda_mhz = 8.0;
                self.pump_power_mw = 25.0;
                self.corner_gain_mhz = 6.0;
                self.bulk_loss_mhz = 3.5;
                self.recompute();
            }
            if ui.button("Sub-Threshold Linear Regime").clicked() {
                self.pump_power_mw = 3.0;
                self.recompute();
            }
            if ui.button("l = +1 Chiral Vortex Amplifier (24 dB Gain)").clicked() {
                self.selected_charge = VortexOamCharge::PlusOne;
                self.dynamic_modulation_amplitude_mhz = 8.5;
                self.interaction_length_mm = 24.0;
                self.recompute();
            }
            if ui.button("l = +2 Double Vortex Amplifier").clicked() {
                self.selected_charge = VortexOamCharge::PlusTwo;
                self.dynamic_modulation_amplitude_mhz = 10.0;
                self.interaction_length_mm = 30.0;
                self.recompute();
            }
        });
        ui.separator();

        match self.active_tab {
            FloquetCornerLaserTab::CornerModeLasing => self.render_corner_lasing_tab(ui),
            FloquetCornerLaserTab::FloquetModulation => self.render_floquet_modulation_tab(ui),
            FloquetCornerLaserTab::VortexAmplifier => self.render_vortex_amplifier_tab(ui),
            FloquetCornerLaserTab::RealSpaceMetamaterial => self.render_realspace_tab(ui),
            FloquetCornerLaserTab::AuditTelemetry => self.render_audit_tab(ui),
        }
    }

    fn render_corner_lasing_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Controls column
            ui.vertical(|ui| {
                ui.set_width(280.0);
                ui.heading(RichText::new("Acoustic Lasing Controls").color(Color32::from_rgb(226, 232, 240)));
                ui.label(RichText::new("Second-order topological corner-mode lasing with bulk loss suppression.").color(Color32::from_rgb(148, 163, 184)));

                ui.add_space(8.0);
                let mut changed = false;
                changed |= ui.add(egui::Slider::new(&mut self.pump_power_mw, 0.5..=60.0).text("Pump Power (mW)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.intracell_gamma_mhz, 0.5..=10.0).text("Intracell Gamma (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.intercell_lambda_mhz, 1.0..=15.0).text("Intercell Lambda (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.corner_gain_mhz, 1.0..=12.0).text("Corner Gain (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.bulk_loss_mhz, 0.5..=8.0).text("Bulk Loss (MHz)")).changed();

                if changed {
                    self.recompute();
                }

                ui.add_space(12.0);
                ui.separator();
                let p_th = self.processor.laser.threshold_pump_power_mw();
                let p_out = self.processor.laser.calculate_output_power_mw(self.pump_power_mw);
                let smsr = self.processor.laser.calculate_smsr_db();
                let lw = self.processor.laser.calculate_laser_linewidth_khz();

                ui.label(RichText::new(format!("Lasing Threshold P_th: {:.2} mW", p_th)).color(Color32::from_rgb(147, 197, 253)));
                ui.label(RichText::new(format!("Coherent Power P_out: {:.2} mW", p_out)).color(Color32::from_rgb(52, 211, 153)));
                ui.label(RichText::new(format!("Side-Mode SMSR: {:.1} dB", smsr)).color(Color32::from_rgb(251, 191, 36)));
                ui.label(RichText::new(format!("Laser Linewidth: {:.1} kHz", lw)).color(Color32::from_rgb(168, 85, 247)));
            });

            ui.separator();

            // Right column: Emission spectrum and L-I curves
            ui.vertical(|ui| {
                ui.heading(RichText::new("Coherent Emission Spectrum").color(Color32::from_rgb(226, 232, 240)));
                let points: PlotPoints = self
                    .cached_emission_spectrum
                    .iter()
                    .map(|pt| [pt.detuning_mhz, pt.intensity_db])
                    .collect();

                let line = Line::new("Acoustic Emission Intensity (dB)", points)
                    .color(Color32::from_rgb(52, 211, 153))
                    .width(2.0);

                Plot::new("emission_spectrum_plot")
                    .height(260.0)
                    .x_axis_label("Detuning (MHz)")
                    .y_axis_label("Spectral Power (dB)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        plot_ui.hline(HLine::new("0 dB Threshold", 0.0).color(Color32::from_rgb(100, 116, 139)));
                    });

                ui.add_space(8.0);
                ui.label(RichText::new("0D Corner Mode Status:").color(Color32::from_rgb(148, 163, 184)));
                ui.horizontal(|ui| {
                    for mode in &self.cached_corner_modes {
                        ui.label(RichText::new(format!(
                            "C{}: {:.4} GHz ({:.1}% Confined)",
                            mode.corner_id, mode.frequency_ghz, mode.confinement_ratio * 100.0
                        )).color(Color32::from_rgb(96, 165, 250)));
                    }
                });
            });
        });
    }

    fn render_floquet_modulation_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("Spatio-Temporal Floquet Gain-Loss Modulation").color(Color32::from_rgb(226, 232, 240)));
            ui.label(RichText::new("Dynamic Floquet modulation gamma(t, phi) breaks time-reversal symmetry, enabling non-reciprocal vortex routing.").color(Color32::from_rgb(148, 163, 184)));

            ui.add_space(8.0);
            let mut changed = false;
            ui.horizontal(|ui| {
                changed |= ui.add(egui::Slider::new(&mut self.modulation_freq_mhz, 20.0..=150.0).text("Modulation Freq (MHz)")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.dynamic_modulation_amplitude_mhz, 1.0..=15.0).text("Dynamic Amplitude (MHz)")).changed();
            });

            if changed {
                self.recompute();
            }

            ui.add_space(12.0);
            ui.label(RichText::new("Floquet Gain Distribution in Time-Azimuth Phase Space:").color(Color32::from_rgb(226, 232, 240)));

            let (response, painter) = ui.allocate_painter(vec2(520.0, 220.0), Sense::hover());
            let rect = response.rect;

            painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), StrokeKind::Outside);

            let n_t = 30;
            let n_phi = 30;
            let cell_w = rect.width() / n_t as f32;
            let cell_h = rect.height() / n_phi as f32;

            for it in 0..n_t {
                for ip in 0..n_phi {
                    let t = (it as f64 / n_t as f64) * 2.0 * std::f64::consts::PI;
                    let phi = (ip as f64 / n_phi as f64) * 2.0 * std::f64::consts::PI;
                    let mod_val = self.processor.params.amplifier.static_gain_mhz
                        + self.dynamic_modulation_amplitude_mhz * (t - phi).cos();

                    let norm = (mod_val / 20.0).clamp(0.0, 1.0);
                    let color = Color32::from_rgb(
                        (norm * 255.0) as u8,
                        (50.0 + norm * 150.0) as u8,
                        (200.0 * (1.0 - norm)) as u8,
                    );

                    let c_rect = Rect::from_min_size(
                        pos2(rect.min.x + it as f32 * cell_w as f32, rect.min.y + ip as f32 * cell_h as f32),
                        vec2(cell_w as f32, cell_h as f32),
                    );
                    painter.rect_filled(c_rect, 0.0, color);
                }
            }
        });
    }

    fn render_vortex_amplifier_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Controls column
            ui.vertical(|ui| {
                ui.set_width(280.0);
                ui.heading(RichText::new("Vortex Amplifier Controls").color(Color32::from_rgb(226, 232, 240)));
                ui.label(RichText::new("Non-reciprocal chiral amplification of orbital angular momentum (OAM) acoustic beams.").color(Color32::from_rgb(148, 163, 184)));

                ui.add_space(8.0);
                ui.label(RichText::new("Target OAM Charge:").color(Color32::from_rgb(148, 163, 184)));
                let mut charge_changed = false;
                for charge in &[
                    VortexOamCharge::MinusTwo,
                    VortexOamCharge::MinusOne,
                    VortexOamCharge::Zero,
                    VortexOamCharge::PlusOne,
                    VortexOamCharge::PlusTwo,
                ] {
                    if ui.selectable_value(&mut self.selected_charge, *charge, charge.label()).clicked() {
                        charge_changed = true;
                    }
                }

                ui.add_space(8.0);
                let mut slider_changed = false;
                slider_changed |= ui.add(egui::Slider::new(&mut self.interaction_length_mm, 5.0..=50.0).text("Interaction Length (mm)")).changed();
                slider_changed |= ui.add(egui::Slider::new(&mut self.dynamic_modulation_amplitude_mhz, 2.0..=15.0).text("Pump Amp (MHz)")).changed();

                if charge_changed || slider_changed {
                    self.recompute();
                }

                ui.add_space(12.0);
                ui.separator();
                let f_gain = self.processor.amplifier.forward_power_gain_db();
                let r_gain = self.processor.amplifier.reverse_power_gain_db();
                let isolation = self.processor.amplifier.isolation_contrast_db();
                let purity = self.processor.amplifier.calculate_oam_purity_percent();

                ui.label(RichText::new(format!("Forward Vortex Gain: {:.1} dB", f_gain)).color(Color32::from_rgb(52, 211, 153)));
                ui.label(RichText::new(format!("Reverse Transmission: {:.1} dB", r_gain)).color(Color32::from_rgb(239, 68, 68)));
                ui.label(RichText::new(format!("Directional Isolation: {:.1} dB", isolation)).color(Color32::from_rgb(147, 197, 253)));
                ui.label(RichText::new(format!("OAM Modal Purity: {:.1} %", purity)).color(Color32::from_rgb(251, 191, 36)));
            });

            ui.separator();

            // Right column: Gain spectrum & Donut Beam Canvas
            ui.vertical(|ui| {
                ui.heading(RichText::new("Directional Vortex Gain Spectrum").color(Color32::from_rgb(226, 232, 240)));
                let f_points: PlotPoints = self
                    .cached_gain_spectrum
                    .iter()
                    .map(|pt| [pt.detuning_mhz, pt.forward_gain_db])
                    .collect();
                let r_points: PlotPoints = self
                    .cached_gain_spectrum
                    .iter()
                    .map(|pt| [pt.detuning_mhz, pt.reverse_gain_db])
                    .collect();

                let forward_line = Line::new("Forward Gain G_fwd (dB)", f_points)
                    .color(Color32::from_rgb(52, 211, 153))
                    .width(2.0);
                let reverse_line = Line::new("Reverse Gain G_rev (dB)", r_points)
                    .color(Color32::from_rgb(239, 68, 68))
                    .width(2.0);

                Plot::new("vortex_gain_plot")
                    .height(200.0)
                    .x_axis_label("Detuning (MHz)")
                    .y_axis_label("Power Gain (dB)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(forward_line);
                        plot_ui.line(reverse_line);
                        plot_ui.hline(HLine::new("0 dB Level", 0.0).color(Color32::from_rgb(100, 116, 139)));
                    });

                ui.add_space(8.0);
                ui.label(RichText::new("2D Laguerre-Gaussian Vortex Beam Profile (Donut Intensity):").color(Color32::from_rgb(226, 232, 240)));

                let (response, painter) = ui.allocate_painter(vec2(180.0, 180.0), Sense::hover());
                let rect = response.rect;
                painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));

                let grid_size = self.cached_vortex_intensity.len();
                if grid_size > 0 {
                    let cell_w = rect.width() / grid_size as f32;
                    let cell_h = rect.height() / grid_size as f32;

                    for y in 0..grid_size {
                        for x in 0..grid_size {
                            let intensity = self.cached_vortex_intensity[y][x];
                            let c = (intensity * 255.0).clamp(0.0, 255.0) as u8;
                            let color = Color32::from_rgb(c, (c as f32 * 0.7) as u8, 255 - c);
                            let c_rect = Rect::from_min_size(
                                pos2(rect.min.x + x as f32 * cell_w, rect.min.y + y as f32 * cell_h),
                                vec2(cell_w, cell_h),
                            );
                            painter.rect_filled(c_rect, 0.0, color);
                        }
                    }
                }
            });
        });
    }

    fn render_realspace_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("Real-Space Higher-Order Metamaterial Lattice").color(Color32::from_rgb(226, 232, 240)));
            ui.label(RichText::new("2D acoustic resonator array displaying selective 0D topological corner mode energy concentration.").color(Color32::from_rgb(148, 163, 184)));

            ui.add_space(8.0);
            let (response, painter) = ui.allocate_painter(vec2(360.0, 360.0), Sense::hover());
            let rect = response.rect;

            painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), StrokeKind::Outside);

            let n = self.cached_realspace_intensity.len();
            if n > 0 {
                let cell_w = rect.width() / n as f32;
                let cell_h = rect.height() / n as f32;

                for y in 0..n {
                    for x in 0..n {
                        let intensity = self.cached_realspace_intensity[y][x];
                        let c = (intensity * 255.0).clamp(0.0, 255.0) as u8;
                        let color = Color32::from_rgb(c, (c as f32 * 0.4) as u8, (c as f32 * 0.9) as u8);
                        let c_rect = Rect::from_min_size(
                            pos2(rect.min.x + x as f32 * cell_w, rect.min.y + y as f32 * cell_h),
                            vec2(cell_w, cell_h),
                        );
                        painter.rect_filled(c_rect, 0.0, color);
                    }
                }
            }
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("Physics Audit & Engineering Checklist").color(Color32::from_rgb(226, 232, 240)));
            ui.label(RichText::new("Automated verification of topological, lasing, and non-reciprocal acoustic metrics.").color(Color32::from_rgb(148, 163, 184)));

            ui.add_space(8.0);
            let audit = &self.cached_audit;
            let score_text = format!("Physics Verification Score: {} / 10 PASS", audit.total_pass_score);
            let score_color = if audit.all_passed {
                Color32::from_rgb(52, 211, 153)
            } else {
                Color32::from_rgb(239, 68, 68)
            };
            ui.label(RichText::new(score_text).color(score_color).strong());

            ui.add_space(8.0);
            let items = [
                ("1. Higher-Order Bulk Bandgap (Delta >= 8.0 MHz)", audit.higher_order_topology_pass),
                ("2. 0D Corner Mode Spatial Confinement (>= 85%)", audit.corner_confinement_pass),
                ("3. Lasing Threshold Pump Power (P_th <= 20.0 mW)", audit.lasing_threshold_pass),
                ("4. Coherent Output Acoustic Power (P_out >= 2.0 mW)", audit.coherent_output_power_pass),
                ("5. Side-Mode Suppression Ratio (SMSR >= 35.0 dB)", audit.side_mode_suppression_pass),
                ("6. Schawlow-Townes Linewidth Narrowing (<= 50 kHz)", audit.linewidth_narrowing_pass),
                ("7. Forward Vortex Power Gain (G_fwd >= 22.0 dB)", audit.vortex_forward_gain_pass),
                ("8. Non-Reciprocal Isolation Contrast (>= 25.0 dB)", audit.non_reciprocal_isolation_pass),
                ("9. Vortex OAM Modal Purity (>= 90.0%)", audit.oam_modal_purity_pass),
                ("10. Spatio-Temporal Floquet Dynamical Stability", audit.floquet_stability_pass),
            ];

            for (desc, pass) in items {
                let (badge, color) = if pass {
                    ("[PASS]", Color32::from_rgb(52, 211, 153))
                } else {
                    ("[FAIL]", Color32::from_rgb(239, 68, 68))
                };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(badge).color(color).strong());
                    ui.label(RichText::new(desc).color(Color32::from_rgb(226, 232, 240)));
                });
            }

            ui.add_space(12.0);
            ui.label(RichText::new(format!("Last Solver Latency: {:.1} us", self.last_solve_time_us)).color(Color32::from_rgb(148, 163, 184)));
        });
    }
}
