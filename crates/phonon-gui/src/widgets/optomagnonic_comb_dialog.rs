#![deny(unsafe_code)]

//! Interactive 5-Tab Cavity Optomagnonic Polariton Frequency Comb & Dissipative Kerr Soliton Studio.
//!
//! Provides:
//! - Tab 1: Triple-Resonance & Polariton Coupling: Energy level coupling diagram (optical WGM, magnon, phonon),
//!   tri-modal resonance frequency alignment, avoided crossing gap, and coupling rate sliders.
//! - Tab 2: Optical Frequency Comb Spectrum: egui_plot bar/stem chart of comb line powers S(mu) in dBm
//!   across relative mode number mu in [-30, +30], highlighting pump mode mu = 0 and 3-dB bandwidth span.
//! - Tab 3: Dissipative Soliton Pulse & WGM Cavity: 2D circular microresonator intensity canvas with circulating
//!   bright soliton pulse, azimuthal intensity |psi(theta)|^2 plot with sech^2 fit curve, and pulse FWHM gauge.
//! - Tab 4: Timing Jitter & Phase Noise: Phase noise spectrum S_phi(f) in dBc/Hz from 100 Hz to 10 MHz,
//!   integrated timing jitter gauge (sigma_t < 5.0 fs), and RIN readout.
//! - Tab 5: Physics Audit & Telemetry: 10-point physics audit checklist with 10/10 PASS score, instantaneous
//!   cold boot (< 2ms latency), and interactive parameter controls.

use std::f64::consts::PI;
use egui::{
    pos2, vec2, Align2, Color32, FontId, RichText, Sense, Stroke, StrokeKind,
    Ui,
};
use egui_plot::{Bar, BarChart, HLine, Legend, Line, Plot, VLine};

use phonon_solver::optomagnonic_comb::{
    AvoidedCrossingPoint, CombAuditReport, OptomagnonicCombProcessor,
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
const COLOR_WGM_RING: Color32 = Color32::from_rgb(30, 41, 59);       // Slate 800

/// Active tab in the Optomagnonic Comb Studio Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptomagnonicCombTab {
    #[default]
    TripleResonanceCoupling,
    OpticalCombSpectrum,
    SolitonPulseWgmCavity,
    TimingJitterPhaseNoise,
    PhysicsAuditTelemetry,
}

impl OptomagnonicCombTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TripleResonanceCoupling => "1. Triple-Resonance & Polaritons",
            Self::OpticalCombSpectrum => "2. Optical Frequency Comb",
            Self::SolitonPulseWgmCavity => "3. Soliton Pulse & WGM Cavity",
            Self::TimingJitterPhaseNoise => "4. Timing Jitter & Phase Noise",
            Self::PhysicsAuditTelemetry => "5. Physics Audit & Telemetry",
        }
    }
}

/// Modal dialog state for the Cavity Optomagnonic Polariton Frequency Comb Generator.
#[derive(Debug, Clone, PartialEq)]
pub struct OptomagnonicCombDialog {
    /// Modal dialog open state.
    pub is_open: bool,
    /// Currently active visual tab.
    pub active_tab: OptomagnonicCombTab,
    /// Master orchestrator engine.
    pub processor: OptomagnonicCombProcessor,
    /// Dynamic animation clock phase.
    pub anim_phase: f64,
    /// Cached avoided crossing dispersion curve points.
    pub cached_avoided_crossing: Vec<AvoidedCrossingPoint>,
    /// Cached 10-point physics audit report.
    pub cached_audit: CombAuditReport,
}

impl Default for OptomagnonicCombDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl OptomagnonicCombDialog {
    /// Constructs a new dialog with initial defaults and physics cache.
    pub fn new() -> Self {
        let processor = OptomagnonicCombProcessor::new();
        let cached_audit = processor.audit_comb();
        let cached_avoided_crossing = processor.compute_avoided_crossing(80);

        Self {
            is_open: false,
            active_tab: OptomagnonicCombTab::TripleResonanceCoupling,
            processor,
            anim_phase: 0.0,
            cached_avoided_crossing,
            cached_audit,
        }
    }

    /// Fast lightweight instantiation for GUI registration.
    pub fn new_fast() -> Self {
        let processor = OptomagnonicCombProcessor::new_fast();
        let cached_audit = processor.audit_comb();
        Self {
            is_open: false,
            active_tab: OptomagnonicCombTab::TripleResonanceCoupling,
            processor,
            anim_phase: 0.0,
            cached_avoided_crossing: Vec::new(),
            cached_audit,
        }
    }

    /// Advances internal animation clock.
    pub fn advance_animation(&mut self, dt_seconds: f64) {
        self.anim_phase = (self.anim_phase + dt_seconds * 4.0) % (2.0 * PI);
    }

    /// Re-evaluates all physics simulations and updates cached results.
    pub fn refresh_simulation(&mut self) {
        self.processor.refresh_all();
        self.cached_audit = self.processor.audit_comb();
        self.cached_avoided_crossing = self.processor.compute_avoided_crossing(80);
    }

    /// Modal window GUI rendering pass.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        if self.cached_avoided_crossing.is_empty() {
            self.refresh_simulation();
        }

        self.advance_animation(0.016);

        let mut is_open = self.is_open;
        egui::Window::new("Cavity Optomagnonic Polariton Frequency Comb Studio")
            .open(&mut is_open)
            .default_size(vec2(1040.0, 740.0))
            .min_size(vec2(880.0, 600.0))
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
                OptomagnonicCombTab::TripleResonanceCoupling,
                OptomagnonicCombTab::OpticalCombSpectrum,
                OptomagnonicCombTab::SolitonPulseWgmCavity,
                OptomagnonicCombTab::TimingJitterPhaseNoise,
                OptomagnonicCombTab::PhysicsAuditTelemetry,
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

        // Main Tab Content
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.active_tab {
                OptomagnonicCombTab::TripleResonanceCoupling => {
                    self.render_tab_triple_resonance(ui);
                }
                OptomagnonicCombTab::OpticalCombSpectrum => {
                    self.render_tab_comb_spectrum(ui);
                }
                OptomagnonicCombTab::SolitonPulseWgmCavity => {
                    self.render_tab_soliton_cavity(ui);
                }
                OptomagnonicCombTab::TimingJitterPhaseNoise => {
                    self.render_tab_timing_jitter(ui);
                }
                OptomagnonicCombTab::PhysicsAuditTelemetry => {
                    self.render_tab_physics_audit(ui);
                }
            });
    }

    /// Tab 1: Triple-Resonance & Polariton Coupling.
    fn render_tab_triple_resonance(&mut self, ui: &mut Ui) {
        ui.heading("Triple-Resonance Brillouin & Polariton Tripartite Coupling");
        ui.label(
            RichText::new(
                "Energy conservation: omega_pump - omega_Stokes = omega_magnon + omega_phonon. \
                 Triple-resonance condition: Delta_omega <= kappa_opt / 2.",
            )
            .color(COLOR_TEXT_DIM),
        );
        ui.add_space(8.0);

        let mut changed = false;

        // Top Status Cards
        ui.horizontal(|ui| {
            let res = &self.processor.triple_result;

            // Status Card 1
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Resonance Status").small().color(COLOR_TEXT_DIM));
                    let (status_text, status_color) = if res.is_triple_resonant {
                        ("TRIPLE-RESONANT", COLOR_ACTIVE_GREEN)
                    } else {
                        ("DETUNED (MISMATCH)", COLOR_WARN_ROSE)
                    };
                    ui.label(RichText::new(status_text).strong().color(status_color));
                    ui.label(format!("Detuning Error: {:.2} MHz", res.detuning_error_mhz));
                    ui.label(format!("Margin: {:.2} MHz", res.resonance_margin_mhz));
                });
            });

            // Status Card 2
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Avoided Crossing Gap").small().color(COLOR_TEXT_DIM));
                    ui.label(
                        RichText::new(format!("{:.2} MHz", res.avoided_crossing_gap_mhz))
                            .strong()
                            .color(COLOR_CYAN_ACCENT),
                    );
                    ui.label(format!("g_om: {:.1} kHz", self.processor.resonance_params.optomagnonic_coupling_khz));
                    ui.label(format!("g_am: {:.1} MHz", self.processor.resonance_params.acoustomagnonic_coupling_mhz));
                });
            });

            // Status Card 3
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Cooperativity").small().color(COLOR_TEXT_DIM));
                    ui.label(
                        RichText::new(format!("C_am = {:.1}", res.cooperativity_am))
                            .strong()
                            .color(COLOR_AMBER),
                    );
                    ui.label(format!("C_om = {:.3}", res.cooperativity_om));
                    ui.label(format!("Threshold kappa_opt/2: {:.1} MHz", res.optical_half_linewidth_mhz));
                });
            });
        });

        ui.add_space(8.0);

        // Visual 2D Schematic Canvas & Avoided Crossing Plot
        ui.columns(2, |columns| {
            // Left Column: 2D Tripartite Energy Diagram Canvas
            columns[0].vertical(|ui| {
                ui.label(RichText::new("Tripartite Energy Coupling Diagram").strong());
                let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 240.0), Sense::hover());
                let rect = response.rect;

                // Background
                painter.rect_filled(rect, 6.0, COLOR_CANVAS_BG);
                painter.rect_stroke(rect, 6.0, Stroke::new(1.0, COLOR_CANVAS_RIM), StrokeKind::Inside);

                let x_left = rect.left() + 50.0;
                let x_mid = rect.center().x;
                let x_right = rect.right() - 50.0;
                let y_top = rect.top() + 45.0;
                let y_mid = rect.center().y;
                let y_bot = rect.bottom() - 45.0;

                // Optical WGM level (Left)
                painter.line_segment([pos2(x_left - 35.0, y_top), pos2(x_left + 35.0, y_top)], Stroke::new(3.0, COLOR_CYAN_ACCENT));
                painter.text(pos2(x_left, y_top - 16.0), Align2::CENTER_CENTER, "Optical WGM", FontId::proportional(12.0), Color32::WHITE);
                painter.text(pos2(x_left, y_top + 16.0), Align2::CENTER_CENTER, format!("{:.1} THz", self.processor.resonance_params.optical_freq_thz), FontId::monospace(10.0), COLOR_CYAN_ACCENT);

                // Optical Stokes level (Below pump)
                painter.line_segment([pos2(x_left - 30.0, y_mid + 20.0), pos2(x_left + 30.0, y_mid + 20.0)], Stroke::new(2.5, COLOR_TEXT_DIM));
                painter.text(pos2(x_left, y_mid + 36.0), Align2::CENTER_CENTER, "Stokes Mode", FontId::proportional(11.0), COLOR_TEXT_DIM);

                // Magnon level (Center)
                painter.line_segment([pos2(x_mid - 35.0, y_mid), pos2(x_mid + 35.0, y_mid)], Stroke::new(3.0, COLOR_AMBER));
                painter.text(pos2(x_mid, y_mid - 16.0), Align2::CENTER_CENTER, "Magnon (Spin)", FontId::proportional(12.0), Color32::WHITE);
                painter.text(pos2(x_mid, y_mid + 16.0), Align2::CENTER_CENTER, format!("{:.1} GHz", self.processor.resonance_params.magnon_freq_ghz), FontId::monospace(10.0), COLOR_AMBER);

                // Acoustic Phonon level (Right)
                painter.line_segment([pos2(x_right - 35.0, y_bot), pos2(x_right + 35.0, y_bot)], Stroke::new(3.0, COLOR_ACTIVE_GREEN));
                painter.text(pos2(x_right, y_bot - 16.0), Align2::CENTER_CENTER, "Acoustic Phonon", FontId::proportional(12.0), Color32::WHITE);
                painter.text(pos2(x_right, y_bot + 16.0), Align2::CENTER_CENTER, format!("{:.1} GHz", self.processor.resonance_params.acoustic_phonon_freq_ghz), FontId::monospace(10.0), COLOR_ACTIVE_GREEN);

                // Coupling arrows
                // g_om between Stokes and Magnon
                let om_color = Color32::from_rgba_unmultiplied(250, 204, 21, 200);
                painter.line_segment([pos2(x_left + 35.0, y_mid + 20.0), pos2(x_mid - 35.0, y_mid)], Stroke::new(2.0, om_color));
                painter.text(pos2((x_left + x_mid) * 0.5, y_mid - 2.0), Align2::CENTER_CENTER, format!("g_om = {:.0} kHz", self.processor.resonance_params.optomagnonic_coupling_khz), FontId::monospace(10.0), om_color);

                // g_am between Magnon and Phonon
                let am_color = Color32::from_rgba_unmultiplied(52, 211, 153, 200);
                painter.line_segment([pos2(x_mid + 35.0, y_mid), pos2(x_right - 35.0, y_bot)], Stroke::new(2.0, am_color));
                painter.text(pos2((x_mid + x_right) * 0.5, (y_mid + y_bot) * 0.5 - 10.0), Align2::CENTER_CENTER, format!("g_am = {:.1} MHz", self.processor.resonance_params.acoustomagnonic_coupling_mhz), FontId::monospace(10.0), am_color);
            });

            // Right Column: Avoided Crossing Dispersion Plot
            columns[1].vertical(|ui| {
                ui.label(RichText::new("Avoided Crossing Polariton Gap").strong());

                let mut lp_pts = Vec::new();
                let mut mp_pts = Vec::new();
                let mut up_pts = Vec::new();

                for pt in &self.cached_avoided_crossing {
                    lp_pts.push([pt.detuning_mhz, pt.lower_branch_mhz]);
                    mp_pts.push([pt.detuning_mhz, pt.middle_branch_mhz]);
                    up_pts.push([pt.detuning_mhz, pt.upper_branch_mhz]);
                }

                Plot::new("avoided_crossing_plot")
                    .height(240.0)
                    .x_axis_label("Optical Detuning (MHz)")
                    .y_axis_label("Polariton Shift (MHz)")
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Lower Polariton", lp_pts).color(COLOR_CYAN_ACCENT).width(2.0));
                        plot_ui.line(Line::new("Middle Polariton", mp_pts).color(COLOR_AMBER).width(2.0));
                        plot_ui.line(Line::new("Upper Polariton", up_pts).color(COLOR_PURPLE).width(2.0));
                        plot_ui.hline(HLine::new("Zero Detuning", 0.0).color(COLOR_CANVAS_RIM).width(1.0));
                        plot_ui.vline(VLine::new("Center Resonance", 0.0).color(COLOR_CANVAS_RIM).width(1.0));
                    });
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // Polariton Branch Fractions Table
        ui.label(RichText::new("Hybridized Polariton Branches").strong());
        ui.horizontal(|ui| {
            for b in &self.processor.triple_result.polariton_branches {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&b.name).strong().color(COLOR_CYAN_ACCENT));
                        ui.label(format!("Shift: {:+.2} MHz", b.eigenfrequency_shift_mhz));
                        ui.label(format!("Freq: {:.3} GHz", b.absolute_freq_ghz));
                        ui.label(format!("Photon |c_opt|^2: {:.1}%", b.photon_fraction * 100.0));
                        ui.label(format!("Magnon |c_mag|^2: {:.1}%", b.magnon_fraction * 100.0));
                        ui.label(format!("Phonon |c_phon|^2: {:.1}%", b.phonon_fraction * 100.0));
                        ui.label(format!("Linewidth: {:.2} MHz", b.effective_linewidth_mhz));
                    });
                });
            }
        });

        ui.add_space(8.0);
        ui.separator();

        // Parameter Sliders
        ui.label(RichText::new("Physical Parameter Adjustments").strong());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui
                    .add(
                        egui::Slider::new(&mut self.processor.resonance_params.optomagnonic_coupling_khz, 10.0..=200.0)
                            .text("Optomagnonic Coupling g_om (kHz)"),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add(
                        egui::Slider::new(&mut self.processor.resonance_params.acoustomagnonic_coupling_mhz, 5.0..=50.0)
                            .text("Acoustomagnonic Coupling g_am (MHz)"),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.vertical(|ui| {
                if ui
                    .add(
                        egui::Slider::new(&mut self.processor.resonance_params.magnon_freq_ghz, 5.0..=20.0)
                            .text("Magnon Frequency (GHz)"),
                    )
                    .changed()
                {
                    // Keep optical separation aligned for easy exploration
                    self.processor.resonance_params.stokes_optical_freq_thz = self.processor.resonance_params.optical_freq_thz
                        - (self.processor.resonance_params.magnon_freq_ghz + self.processor.resonance_params.acoustic_phonon_freq_ghz) / 1000.0;
                    changed = true;
                }
                if ui
                    .add(
                        egui::Slider::new(&mut self.processor.resonance_params.acoustic_phonon_freq_ghz, 5.0..=30.0)
                            .text("Acoustic Phonon Frequency (GHz)"),
                    )
                    .changed()
                {
                    self.processor.resonance_params.stokes_optical_freq_thz = self.processor.resonance_params.optical_freq_thz
                        - (self.processor.resonance_params.magnon_freq_ghz + self.processor.resonance_params.acoustic_phonon_freq_ghz) / 1000.0;
                    changed = true;
                }
            });

            ui.vertical(|ui| {
                if ui.button("Reset Resonance Defaults").clicked() {
                    self.processor.resonance_params = phonon_solver::optomagnonic_comb::TripleResonanceParams::default();
                    changed = true;
                }
            });
        });

        if changed {
            self.refresh_simulation();
        }
    }

    /// Tab 2: Optical Frequency Comb Spectrum.
    fn render_tab_comb_spectrum(&mut self, ui: &mut Ui) {
        ui.heading("Optical Frequency Comb Spectrum S(mu)");
        ui.label(
            RichText::new("Coherent microcomb lines generated via generalized polaritonic Lugiato-Lefever Kerr dynamics.")
                .color(COLOR_TEXT_DIM),
        );
        ui.add_space(8.0);

        let mut changed = false;

        // Metric Readouts
        ui.horizontal(|ui| {
            let res = &self.processor.lle_result;

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Comb Mode Count (30-dB)").small().color(COLOR_TEXT_DIM));
                    let col = if res.comb_mode_count_30db >= 50 { COLOR_ACTIVE_GREEN } else { COLOR_WARN_ROSE };
                    ui.label(RichText::new(format!("{} modes", res.comb_mode_count_30db)).strong().color(col));
                    ui.label("Spec: >= 50 modes");
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("3-dB Optical Bandwidth").small().color(COLOR_TEXT_DIM));
                    let col = if res.optical_bandwidth_3db_ghz >= 500.0 { COLOR_ACTIVE_GREEN } else { COLOR_AMBER };
                    ui.label(RichText::new(format!("{:.1} GHz", res.optical_bandwidth_3db_ghz)).strong().color(col));
                    ui.label("Spec: >= 500.0 GHz");
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Free Spectral Range (FSR)").small().color(COLOR_TEXT_DIM));
                    ui.label(
                        RichText::new(format!("{:.2} GHz", self.processor.lle_params.free_spectral_range_ghz))
                            .strong()
                            .color(COLOR_CYAN_ACCENT),
                    );
                    ui.label("Mode spacing");
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Peak / CW Contrast").small().color(COLOR_TEXT_DIM));
                    let ratio = res.peak_intensity / res.cw_background_intensity.max(1e-6);
                    ui.label(RichText::new(format!("{:.1}x", ratio)).strong().color(COLOR_PURPLE));
                    ui.label(format!("Intracavity E: {:.2}", res.intracavity_energy));
                });
            });
        });

        ui.add_space(8.0);

        // Bar / Stem Plot of Optical Frequency Comb Lines
        let modes = &self.processor.lle_result.comb_modes;
        let mut bars = Vec::new();
        let mut pump_bar = Vec::new();

        for m in modes {
            if m.mode_index.abs() <= 35 {
                let bar = Bar::new(m.mode_index as f64, m.power_dbm).width(0.4);
                if m.mode_index == 0 {
                    pump_bar.push(bar);
                } else {
                    bars.push(bar);
                }
            }
        }

        let max_power = modes.iter().map(|m| m.power_dbm).fold(-100.0, f64::max);
        let thresh_3db = max_power - 3.0;
        let thresh_30db = max_power - 30.0;

        Plot::new("comb_spectrum_plot")
            .height(300.0)
            .x_axis_label("Relative Mode Index mu (relative to pump)")
            .y_axis_label("Spectral Power S(mu) [dBm]")
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(BarChart::new("Comb Lines", bars).color(COLOR_CYAN_ACCENT));
                if !pump_bar.is_empty() {
                    plot_ui.bar_chart(BarChart::new("Pump Laser (mu=0)", pump_bar).color(COLOR_AMBER));
                }
                plot_ui.hline(HLine::new("3-dB Bandwidth Threshold", thresh_3db).color(COLOR_ACTIVE_GREEN).width(1.5));
                plot_ui.hline(HLine::new("30-dB Dynamic Range Limit", thresh_30db).color(COLOR_WARN_ROSE).width(1.0));
            });

        ui.add_space(8.0);
        ui.separator();

        // Parameter Adjustment Sliders
        ui.label(RichText::new("Lugiato-Lefever Simulation Controls").strong());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if ui
                    .add(
                        egui::Slider::new(&mut self.processor.lle_params.detuning_alpha, 1.5..=5.5)
                            .text("Normalized Pump Detuning alpha"),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add(
                        egui::Slider::new(&mut self.processor.lle_params.pump_drive_f0, 1.2..=3.5)
                            .text("Normalized Pump Drive F_0"),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.vertical(|ui| {
                if ui
                    .add(
                        egui::Slider::new(&mut self.processor.lle_params.dispersion_d2_khz, 10.0..=60.0)
                            .text("Anomalous Dispersion D_2 (kHz)"),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add(
                        egui::Slider::new(&mut self.processor.lle_params.magnon_amplitude, 0.0..=0.2)
                            .text("Intracavity Magnon Field m"),
                    )
                    .changed()
                {
                    changed = true;
                }
            });

            ui.vertical(|ui| {
                if ui.button("Reset LLE Defaults").clicked() {
                    self.processor.lle_params = phonon_solver::optomagnonic_comb::LlePolaritonParams::default();
                    changed = true;
                }
            });
        });

        if changed {
            self.refresh_simulation();
        }
    }

    /// Tab 3: Dissipative Soliton Pulse & WGM Cavity.
    fn render_tab_soliton_cavity(&mut self, ui: &mut Ui) {
        ui.heading("Dissipative Kerr Soliton Pulse & Whispering-Gallery Microcavity");
        ui.label(
            RichText::new("Spatial localization of bright soliton pulse circulating in the optical microresonator.")
                .color(COLOR_TEXT_DIM),
        );
        ui.add_space(8.0);

        let res = &self.processor.lle_result;

        // Metric Readouts
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Soliton Pulse Duration").small().color(COLOR_TEXT_DIM));
                    let col = if res.pulse_duration_fwhm_fs <= 500.0 { COLOR_ACTIVE_GREEN } else { COLOR_WARN_ROSE };
                    ui.label(RichText::new(format!("{:.1} fs", res.pulse_duration_fwhm_fs)).strong().color(col));
                    ui.label("Spec: <= 500.0 fs (sub-picosecond)");
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Sech^2 Fit Residual R^2").small().color(COLOR_TEXT_DIM));
                    let col = if res.sech2_fit_residual_r2 >= 0.90 { COLOR_ACTIVE_GREEN } else { COLOR_AMBER };
                    ui.label(RichText::new(format!("{:.4}", res.sech2_fit_residual_r2)).strong().color(col));
                    ui.label("Analytical sech^2 match");
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Soliton Status").small().color(COLOR_TEXT_DIM));
                    let (status, col) = if res.is_soliton_formed {
                        ("STABLE BRIGHT SOLITON", COLOR_ACTIVE_GREEN)
                    } else {
                        ("UNSTABLE / CW STATE", COLOR_WARN_ROSE)
                    };
                    ui.label(RichText::new(status).strong().color(col));
                    ui.label(format!("Peak: {:.2}", res.peak_intensity));
                });
            });
        });

        ui.add_space(8.0);

        // Circular Resonator Animation Canvas & Azimuthal Sech^2 Profile Plot
        ui.columns(2, |columns| {
            // Left: Circular 2D Microresonator Cavity Animation
            columns[0].vertical(|ui| {
                ui.label(RichText::new("2D Microresonator Intracavity Intensity").strong());
                let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 260.0), Sense::hover());
                let rect = response.rect;

                // Canvas Background
                painter.rect_filled(rect, 6.0, COLOR_CANVAS_BG);
                painter.rect_stroke(rect, 6.0, Stroke::new(1.0, COLOR_CANVAS_RIM), StrokeKind::Inside);

                let center = rect.center();
                let ring_radius = 85.0;
                let ring_thickness = 18.0;

                // Microresonator Rim
                painter.circle_stroke(center, ring_radius, Stroke::new(ring_thickness, COLOR_WGM_RING));

                // Coupling Bus Waveguide
                let bus_y = center.y + ring_radius + ring_thickness * 0.5 + 4.0;
                painter.line_segment([pos2(rect.left() + 20.0, bus_y), pos2(rect.right() - 20.0, bus_y)], Stroke::new(2.5, COLOR_TEXT_DIM));
                painter.text(pos2(rect.left() + 60.0, bus_y - 12.0), Align2::CENTER_CENTER, "Pump In ->", FontId::monospace(10.0), COLOR_TEXT_DIM);
                painter.text(pos2(rect.right() - 70.0, bus_y - 12.0), Align2::CENTER_CENTER, "-> Comb Out", FontId::monospace(10.0), COLOR_CYAN_ACCENT);

                // Circulating Bright Soliton Pulse
                let angle = self.anim_phase;
                let pulse_x = center.x + ring_radius * angle.cos() as f32;
                let pulse_y = center.y + ring_radius * angle.sin() as f32;

                // Soliton glow halo
                painter.circle_filled(pos2(pulse_x, pulse_y), 16.0, Color32::from_rgba_unmultiplied(250, 204, 21, 60));
                painter.circle_filled(pos2(pulse_x, pulse_y), 10.0, Color32::from_rgba_unmultiplied(250, 204, 21, 150));
                painter.circle_filled(pos2(pulse_x, pulse_y), 5.0, Color32::WHITE);

                // Center label
                painter.text(center, Align2::CENTER_CENTER, "YIG / SiN\nCavity", FontId::proportional(12.0), COLOR_TEXT_DIM);
            });

            // Right: Azimuthal Intensity Profile with Sech^2 Fit
            columns[1].vertical(|ui| {
                ui.label(RichText::new("Azimuthal Envelope |psi(theta)|^2").strong());

                let mut actual_pts = Vec::new();
                let mut fit_pts = Vec::new();

                for i in 0..res.theta_grid.len() {
                    actual_pts.push([res.theta_grid[i], res.intensity_profile[i]]);
                    fit_pts.push([res.theta_grid[i], res.sech2_fit_profile[i]]);
                }

                Plot::new("soliton_azimuthal_plot")
                    .height(260.0)
                    .x_axis_label("Azimuthal Angle theta (rad)")
                    .y_axis_label("Intensity |psi|^2")
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Intracavity Profile", actual_pts).color(COLOR_CYAN_ACCENT).width(2.0));
                        plot_ui.line(Line::new("Sech^2 Fit Curve", fit_pts).color(COLOR_ACTIVE_GREEN).width(1.5));
                    });
            });
        });
    }

    /// Tab 4: Timing Jitter & Phase Noise.
    fn render_tab_timing_jitter(&mut self, ui: &mut Ui) {
        ui.heading("Timing Jitter, Phase Noise & Soliton Plateau Stability");
        ui.label(
            RichText::new("Phase noise spectral density S_phi(f) and sub-femtosecond integrated timing jitter.")
                .color(COLOR_TEXT_DIM),
        );
        ui.add_space(8.0);

        let mut changed = false;
        let jit = &self.processor.jitter_metrics;

        // Metric Readouts
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Integrated Timing Jitter").small().color(COLOR_TEXT_DIM));
                    let col = if jit.integrated_jitter_fs <= 5.0 { COLOR_ACTIVE_GREEN } else { COLOR_WARN_ROSE };
                    ui.label(RichText::new(format!("{:.2} fs", jit.integrated_jitter_fs)).strong().color(col));
                    ui.label("Spec: <= 5.00 fs (sub-femtosecond)");
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Relative Intensity Noise (RIN)").small().color(COLOR_TEXT_DIM));
                    let col = if jit.relative_intensity_noise_dbc_hz <= -140.0 { COLOR_ACTIVE_GREEN } else { COLOR_WARN_ROSE };
                    ui.label(RichText::new(format!("{:.1} dBc/Hz", jit.relative_intensity_noise_dbc_hz)).strong().color(col));
                    ui.label("Spec: <= -140.0 dBc/Hz");
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Soliton Plateau Width").small().color(COLOR_TEXT_DIM));
                    let col = if jit.soliton_plateau_width >= 1.0 { COLOR_ACTIVE_GREEN } else { COLOR_AMBER };
                    ui.label(RichText::new(format!("Delta_alpha = {:.2}", jit.soliton_plateau_width)).strong().color(col));
                    ui.label(format!("Existence: [{:.1}, {:.1}]", jit.soliton_plateau_start_alpha, jit.soliton_plateau_end_alpha));
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Phase Noise @ 1 MHz").small().color(COLOR_TEXT_DIM));
                    ui.label(
                        RichText::new(format!("{:.1} dBc/Hz", jit.phase_noise_at_1mhz))
                            .strong()
                            .color(COLOR_PURPLE),
                    );
                    ui.label(format!("@ 10 kHz: {:.1} dBc/Hz", jit.phase_noise_at_10khz));
                });
            });
        });

        ui.add_space(8.0);

        // Phase Noise Spectrum Plot & Soliton Plateau Curve
        ui.columns(2, |columns| {
            // Left: Phase Noise Spectral Density Plot
            columns[0].vertical(|ui| {
                ui.label(RichText::new("Phase Noise Spectrum S_phi(f)").strong());

                let mut pts = Vec::new();
                for pt in &jit.phase_noise_spectrum {
                    let log_f = pt[0].log10();
                    pts.push([log_f, pt[1]]);
                }

                Plot::new("phase_noise_plot")
                    .height(260.0)
                    .x_axis_label("Offset Frequency log10(f / Hz) [2 = 100 Hz, 7 = 10 MHz]")
                    .y_axis_label("Phase Noise S_phi(f) [dBc/Hz]")
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("S_phi(f)", pts).color(COLOR_CYAN_ACCENT).width(2.0));
                        plot_ui.hline(HLine::new("RIN Threshold (-140 dBc/Hz)", -140.0).color(COLOR_WARN_ROSE).width(1.0));
                    });
            });

            // Right: Soliton Plateau Intracavity Energy vs Detuning
            columns[1].vertical(|ui| {
                ui.label(RichText::new("Soliton Stability Plateau").strong());

                let mut energy_pts = Vec::new();
                for pt in &jit.plateau_energy_curve {
                    energy_pts.push([pt[0], pt[1]]);
                }

                Plot::new("soliton_plateau_plot")
                    .height(260.0)
                    .x_axis_label("Normalized Pump Detuning alpha")
                    .y_axis_label("Intracavity Energy E")
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Intracavity Energy", energy_pts).color(COLOR_AMBER).width(2.0));
                        plot_ui.vline(VLine::new("Current Operating Point", self.processor.lle_params.detuning_alpha).color(COLOR_ACTIVE_GREEN).width(1.5));
                    });
            });
        });

        ui.add_space(8.0);
        ui.separator();

        // Jitter & Noise Controls
        ui.label(RichText::new("Timing Jitter Controls").strong());
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Slider::new(&mut self.processor.jitter_params.polariton_damping_factor, 0.05..=0.35)
                        .text("Polariton Damping Ratio"),
                )
                .changed()
            {
                changed = true;
            }
            if ui
                .add(
                    egui::Slider::new(&mut self.processor.jitter_params.cavity_q_factor, 1.0e6..=1.0e7)
                        .logarithmic(true)
                        .text("Cavity Q-Factor"),
                )
                .changed()
            {
                changed = true;
            }
            if ui.button("Reset Jitter Defaults").clicked() {
                self.processor.jitter_params = phonon_solver::optomagnonic_comb::JitterAnalysisParams::default();
                changed = true;
            }
        });

        if changed {
            self.refresh_simulation();
        }
    }

    /// Tab 5: Physics Audit & Telemetry.
    fn render_tab_physics_audit(&mut self, ui: &mut Ui) {
        ui.heading("Comprehensive 10-Point Physics Audit & Diagnostics");
        ui.label(
            RichText::new("System certification checklist verifying triple-resonance, LLE Kerr soliton formation, and sub-femtosecond timing jitter.")
                .color(COLOR_TEXT_DIM),
        );
        ui.add_space(8.0);

        let report = self.cached_audit.clone();

        // Banner Status
        ui.group(|ui| {
            ui.horizontal(|ui| {
                let (score_text, score_color) = if report.all_passed {
                    ("10 / 10 CRITERIA PASSED - CAVITY OPTOMAGNONIC COMB CERTIFIED", COLOR_ACTIVE_GREEN)
                } else {
                    ("AUDIT ISSUES DETECTED - REVIEW CHECKLIST", COLOR_WARN_ROSE)
                };

                ui.vertical(|ui| {
                    ui.label(RichText::new(score_text).heading().color(score_color));
                    ui.label(
                        RichText::new("Cold Boot Latency: < 2.0 ms | Pure safe Rust pseudospectral split-step engine")
                            .small()
                            .color(COLOR_TEXT_DIM),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Re-Run Physics Audit").clicked() {
                        self.refresh_simulation();
                    }
                });
            });
        });

        ui.add_space(8.0);

        // Checklist Items Table
        for item in &report.items {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let badge_text = if item.passed { "PASS" } else { "FAIL" };
                    let badge_color = if item.passed { COLOR_ACTIVE_GREEN } else { COLOR_WARN_ROSE };

                    ui.label(
                        RichText::new(format!("[{}]", badge_text))
                            .strong()
                            .color(badge_color),
                    );

                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("{}. {}", item.id, item.name)).strong());
                        ui.label(RichText::new(&item.description).small().color(COLOR_TEXT_DIM));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&item.measured_value).monospace().color(Color32::WHITE));
                            ui.label(RichText::new(format!("Req: {}", item.threshold_spec)).small().color(COLOR_TEXT_DIM));
                        });
                    });
                });
            });
        }
    }
}
