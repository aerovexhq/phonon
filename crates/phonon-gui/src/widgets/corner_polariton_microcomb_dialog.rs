#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 409: Topological Corner-Polariton Micro-Comb
//! Soliton & Dissipative Kerr Acoustic Frequency Synthesizer.

use egui::{
    pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Window,
};
use egui_plot::{HLine, Line, Plot, PlotPoints, VLine};
use phonon_solver::corner_polariton_microcomb::{
    AllanDeviationPoint, CornerModeProfile,
    CornerPolaritonMicrocombSynthesizer, CornerPolaritonParams, MicrocombAuditReport,
    PhaseNoisePoint, SolitonCombPoint, SolitonDynamicsParams,
    SolitonTemporalPoint, SynthesizerParams,
};
use crate::time_util::Instant;

/// Active tab in the Corner-Polariton Micro-Comb Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerPolaritonMicrocombTab {
    CornerPolaritonCavity,
    DissipativeSolitonComb,
    KerrBistabilityMI,
    TimingSynthesizerF2F,
    AuditTelemetry,
}

/// CAD modal dialog for topological corner-polariton frequency synthesizers.
pub struct CornerPolaritonMicrocombDialog {
    pub is_open: bool,
    pub active_tab: CornerPolaritonMicrocombTab,

    // Cavity parameters
    pub lattice_size: usize,
    pub intracell_gamma_mhz: f64,
    pub intercell_lambda_mhz: f64,
    pub cavity_resonance_ghz: f64,
    pub intrinsic_loss_khz: f64,
    pub corner_mode_volume_um3: f64,
    pub gvd_beta2_ps2_mm: f64,

    // Soliton parameters
    pub pump_power_mw: f64,
    pub pump_detuning_mhz: f64,
    pub repetition_rate_mhz: f64,
    pub total_loss_rate_mhz: f64,
    pub external_coupling_ratio: f64,
    pub non_linear_gain_gamma: f64,

    // Synthesizer parameters
    pub f_ceo_target_mhz: f64,
    pub pll_bandwidth_khz: f64,

    // Solver and cached telemetry
    pub synthesizer: CornerPolaritonMicrocombSynthesizer,
    pub cached_modes: Vec<CornerModeProfile>,
    pub cached_temporal: Vec<SolitonTemporalPoint>,
    pub cached_spectrum: Vec<SolitonCombPoint>,
    pub cached_bistability: Vec<(f64, f64, bool)>,
    pub cached_phase_noise: Vec<PhaseNoisePoint>,
    pub cached_allan_deviation: Vec<AllanDeviationPoint>,
    pub cached_audit_report: MicrocombAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for CornerPolaritonMicrocombDialog {
    fn default() -> Self {
        let c_params = CornerPolaritonParams::default();
        let s_params = SolitonDynamicsParams::default();
        let synth_params = SynthesizerParams::default();

        let synthesizer = CornerPolaritonMicrocombSynthesizer::new(
            c_params.clone(),
            s_params.clone(),
            synth_params.clone(),
        );

        let cached_modes = synthesizer.cavity.solve_corner_modes();
        let cached_temporal = synthesizer.soliton.compute_temporal_profile(64);
        let cached_spectrum = synthesizer.soliton.compute_comb_spectrum();
        let cached_bistability = synthesizer.soliton.compute_bistability_curve(40);
        let cached_phase_noise = synthesizer.synthesizer.compute_phase_noise_profile(40);
        let cached_allan_deviation = synthesizer.synthesizer.compute_allan_deviation(25);
        let cached_audit_report = synthesizer.audit_microcomb();

        Self {
            is_open: false,
            active_tab: CornerPolaritonMicrocombTab::CornerPolaritonCavity,

            lattice_size: c_params.lattice_size,
            intracell_gamma_mhz: c_params.intracell_coupling_gamma_mhz,
            intercell_lambda_mhz: c_params.intercell_coupling_lambda_mhz,
            cavity_resonance_ghz: c_params.cavity_resonance_ghz,
            intrinsic_loss_khz: c_params.intrinsic_loss_khz,
            corner_mode_volume_um3: c_params.corner_mode_volume_um3,
            gvd_beta2_ps2_mm: c_params.gvd_beta2_ps2_mm,

            pump_power_mw: s_params.pump_power_mw,
            pump_detuning_mhz: s_params.pump_detuning_mhz,
            repetition_rate_mhz: s_params.repetition_rate_mhz,
            total_loss_rate_mhz: s_params.total_loss_rate_mhz,
            external_coupling_ratio: s_params.external_coupling_ratio,
            non_linear_gain_gamma: s_params.non_linear_gain_gamma,

            f_ceo_target_mhz: synth_params.f_ceo_target_mhz,
            pll_bandwidth_khz: synth_params.pll_bandwidth_khz,

            synthesizer,
            cached_modes,
            cached_temporal,
            cached_spectrum,
            cached_bistability,
            cached_phase_noise,
            cached_allan_deviation,
            cached_audit_report,
            last_solve_time_us: 120.0,
        }
    }
}

impl CornerPolaritonMicrocombDialog {
    /// Creates a fast cold-boot dialog instance with pre-seeded baseline telemetry (< 2ms).
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Primary render entrypoint.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Recomputes all multi-physics models and updates cached state.
    pub fn recompute(&mut self) {
        let start = Instant::now();

        let c_params = CornerPolaritonParams {
            lattice_size: self.lattice_size,
            intracell_coupling_gamma_mhz: self.intracell_gamma_mhz,
            intercell_coupling_lambda_mhz: self.intercell_lambda_mhz,
            cavity_resonance_ghz: self.cavity_resonance_ghz,
            intrinsic_loss_khz: self.intrinsic_loss_khz,
            corner_mode_volume_um3: self.corner_mode_volume_um3,
            kerr_index_n2_m2_w: 2.5e-17,
            gvd_beta2_ps2_mm: self.gvd_beta2_ps2_mm,
        };

        let s_params = SolitonDynamicsParams {
            pump_power_mw: self.pump_power_mw,
            pump_detuning_mhz: self.pump_detuning_mhz,
            repetition_rate_mhz: self.repetition_rate_mhz,
            total_loss_rate_mhz: self.total_loss_rate_mhz,
            external_coupling_ratio: self.external_coupling_ratio,
            non_linear_gain_gamma: self.non_linear_gain_gamma,
            comb_lines_count: 128,
        };

        let synth_params = SynthesizerParams {
            carrier_frequency_ghz: self.cavity_resonance_ghz,
            f_ceo_target_mhz: self.f_ceo_target_mhz,
            pll_bandwidth_khz: self.pll_bandwidth_khz,
            cavity_q_factor: (self.cavity_resonance_ghz * 1e6) / (2.0 * self.intrinsic_loss_khz.max(0.1)),
            photodiode_responsivity_a_w: 0.85,
            shot_noise_floor_dbc_hz: -165.0,
        };

        self.synthesizer = CornerPolaritonMicrocombSynthesizer::new(c_params, s_params, synth_params);
        self.cached_modes = self.synthesizer.cavity.solve_corner_modes();
        self.cached_temporal = self.synthesizer.soliton.compute_temporal_profile(64);
        self.cached_spectrum = self.synthesizer.soliton.compute_comb_spectrum();
        self.cached_bistability = self.synthesizer.soliton.compute_bistability_curve(40);
        self.cached_phase_noise = self.synthesizer.synthesizer.compute_phase_noise_profile(40);
        self.cached_allan_deviation = self.synthesizer.synthesizer.compute_allan_deviation(25);
        self.cached_audit_report = self.synthesizer.audit_microcomb();

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Renders the modal window.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Topological Corner-Polariton Micro-Comb Synthesizer")
            .open(&mut is_open)
            .default_width(920.0)
            .default_height(650.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.active_tab,
                CornerPolaritonMicrocombTab::CornerPolaritonCavity,
                "Corner-Polariton Cavity",
            );
            ui.selectable_value(
                &mut self.active_tab,
                CornerPolaritonMicrocombTab::DissipativeSolitonComb,
                "Dissipative Soliton Comb",
            );
            ui.selectable_value(
                &mut self.active_tab,
                CornerPolaritonMicrocombTab::KerrBistabilityMI,
                "Kerr Bistability & MI",
            );
            ui.selectable_value(
                &mut self.active_tab,
                CornerPolaritonMicrocombTab::TimingSynthesizerF2F,
                "Timing Synthesizer & f-2f",
            );
            ui.selectable_value(
                &mut self.active_tab,
                CornerPolaritonMicrocombTab::AuditTelemetry,
                "Audit & Telemetry",
            );
        });

        ui.separator();

        match self.active_tab {
            CornerPolaritonMicrocombTab::CornerPolaritonCavity => self.render_cavity_tab(ui),
            CornerPolaritonMicrocombTab::DissipativeSolitonComb => self.render_soliton_tab(ui),
            CornerPolaritonMicrocombTab::KerrBistabilityMI => self.render_bistability_tab(ui),
            CornerPolaritonMicrocombTab::TimingSynthesizerF2F => self.render_synthesizer_tab(ui),
            CornerPolaritonMicrocombTab::AuditTelemetry => self.render_audit_tab(ui),
        }
    }

    fn render_cavity_tab(&mut self, ui: &mut Ui) {
        let mut recompute_requested = false;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("SOTI Quadrupole Polariton Cavity");
                ui.label("Sub-diffraction 0D acoustic polaritons localized in a 2D quadrupole lattice.");

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.strong("Lattice & Topological Couplings");
                    if ui.add(egui::Slider::new(&mut self.intracell_gamma_mhz, 0.5..=8.0).text("Intracell gamma (MHz)")).changed() {
                        recompute_requested = true;
                    }
                    if ui.add(egui::Slider::new(&mut self.intercell_lambda_mhz, 4.0..=20.0).text("Intercell lambda (MHz)")).changed() {
                        recompute_requested = true;
                    }
                    if ui.add(egui::Slider::new(&mut self.cavity_resonance_ghz, 1.0..=6.0).text("Resonance f0 (GHz)")).changed() {
                        recompute_requested = true;
                    }
                    if ui.add(egui::Slider::new(&mut self.intrinsic_loss_khz, 5.0..=100.0).text("Linewidth (kHz)")).changed() {
                        recompute_requested = true;
                    }
                    if ui.add(egui::Slider::new(&mut self.corner_mode_volume_um3, 0.01..=0.20).text("V_mode (um^3)")).changed() {
                        recompute_requested = true;
                    }
                    if ui.add(egui::Slider::new(&mut self.gvd_beta2_ps2_mm, -1.0..=-0.05).text("GVD beta2 (ps^2/mm)")).changed() {
                        recompute_requested = true;
                    }
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("2D Real-Space Energy Density Canvas");
                let (rect, _response) = ui.allocate_exact_size(vec2(280.0, 240.0), Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 18, 28));
                painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(50, 60, 90)), StrokeKind::Inside);

                if let Some(mode) = self.cached_modes.first() {
                    let n = mode.spatial_amplitudes.len();
                    if n > 0 {
                        let cell_w = (rect.width() - 20.0) / (n as f32);
                        let cell_h = (rect.height() - 20.0) / (n as f32);

                        for r in 0..n {
                            for c in 0..n {
                                let val = mode.spatial_amplitudes[r][c];
                                let norm = (val * (n as f64) * 2.0).clamp(0.0, 1.0);
                                let red = (norm * 255.0) as u8;
                                let blue = ((1.0 - norm) * 200.0) as u8;
                                let green = (norm * 140.0) as u8;

                                let cell_rect = Rect::from_min_size(
                                    pos2(rect.min.x + 10.0 + (c as f32) * cell_w, rect.min.y + 10.0 + (r as f32) * cell_h),
                                    vec2(cell_w - 2.0, cell_h - 2.0),
                                );
                                painter.rect_filled(cell_rect, 2.0, Color32::from_rgb(red, green, blue));
                            }
                        }
                    }
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            let gap = self.synthesizer.cavity.bulk_bandgap_mhz();
            let is_topo = self.synthesizer.cavity.is_topological();
            let q = self.cached_modes.first().map(|m| m.quality_factor).unwrap_or(1.0e5);
            let conf = self.cached_modes.first().map(|m| m.corner_confinement_ratio * 100.0).unwrap_or(90.0);

            ui.metric("Topological Phase", if is_topo { "SOTI Topological".to_string() } else { "Trivial".to_string() });
            ui.metric("Bulk Bandgap", format!("{:.2} MHz", gap));
            ui.metric("Corner Confinement", format!("{:.1}%", conf));
            ui.metric("Loaded Q Factor", format!("{:.2e}", q));
            ui.metric("Dispersive D2", format!("{:.1} kHz", self.synthesizer.cavity.dispersion_parameter_d2_khz()));
        });

        if recompute_requested {
            self.recompute();
        }
    }

    fn render_soliton_tab(&mut self, ui: &mut Ui) {
        let mut recompute_requested = false;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Kerr Soliton Drive & Multi-Comb Parameters");
                ui.group(|ui| {
                    if ui.add(egui::Slider::new(&mut self.pump_power_mw, 5.0..=60.0).text("Pump Power (mW)")).changed() {
                        recompute_requested = true;
                    }
                    if ui.add(egui::Slider::new(&mut self.pump_detuning_mhz, 2.0..=30.0).text("Pump Detuning (MHz)")).changed() {
                        recompute_requested = true;
                    }
                    if ui.add(egui::Slider::new(&mut self.repetition_rate_mhz, 100.0..=1000.0).text("Repetition Rate (MHz)")).changed() {
                        recompute_requested = true;
                    }
                    if ui.add(egui::Slider::new(&mut self.total_loss_rate_mhz, 1.0..=10.0).text("Total Loss Rate (MHz)")).changed() {
                        recompute_requested = true;
                    }
                    if ui.add(egui::Slider::new(&mut self.external_coupling_ratio, 0.2..=0.95).text("Coupling Ratio")).changed() {
                        recompute_requested = true;
                    }
                });

                let is_soliton = self.synthesizer.soliton.is_soliton_state();
                let p_th = self.synthesizer.soliton.compute_mi_threshold_power_mw();
                let tau_ps = self.synthesizer.soliton.soliton_pulse_duration_ps();
                let cov = self.synthesizer.soliton.octave_coverage_ratio();

                ui.add_space(8.0);
                ui.metric("Soliton State", if is_soliton { "LOCKED (Dissipative Soliton)".to_string() } else { "UNLOCKED (Chaotic / MI)".to_string() });
                ui.metric("MI Threshold P_th", format!("{:.2} mW", p_th));
                ui.metric("Soliton FWHM Pulse", format!("{:.2} ps", tau_ps));
                ui.metric("Bandwidth Coverage", format!("{:.2}x", cov));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Frequency Comb Spectrum (dBm)");
                let spectrum_points: Vec<[f64; 2]> = self.cached_spectrum.iter().map(|p| [p.frequency_ghz, p.power_dbm]).collect();
                Plot::new("comb_spectrum_plot")
                    .height(180.0)
                    .width(440.0)
                    .x_axis_label("Frequency (GHz)")
                    .y_axis_label("Power (dBm)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Comb Teeth", PlotPoints::new(spectrum_points)).color(Color32::from_rgb(0, 200, 255)));
                        plot_ui.hline(HLine::new("Shot Noise Floor", -120.0).color(Color32::GRAY));
                    });

                ui.heading("Temporal Soliton Profile (mW)");
                let temporal_points: Vec<[f64; 2]> = self.cached_temporal.iter().map(|p| [p.time_ps, p.intensity_mw]).collect();
                Plot::new("temporal_profile_plot")
                    .height(160.0)
                    .width(440.0)
                    .x_axis_label("Time (ps)")
                    .y_axis_label("Intensity (mW)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Temporal Sech^2 Soliton", PlotPoints::new(temporal_points)).color(Color32::from_rgb(255, 120, 50)));
                    });
            });
        });

        if recompute_requested {
            self.recompute();
        }
    }

    fn render_bistability_tab(&mut self, ui: &mut Ui) {
        ui.heading("Kerr Non-Linear Bistability & Modulational Instability");
        ui.label("Intracavity intensity vs detuning displaying the S-curve bifurcation and soliton existence tongue.");

        ui.add_space(8.0);
        let bistability_pts: Vec<[f64; 2]> = self.cached_bistability.iter().map(|(d, i, _)| [*d, *i]).collect();
        Plot::new("bistability_curve_plot")
            .height(300.0)
            .width(ui.available_width() - 20.0)
            .x_axis_label("Detuning (MHz)")
            .y_axis_label("Intracavity Intensity (a.u.)")
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new("Bistability Curve", PlotPoints::new(bistability_pts)).color(Color32::from_rgb(180, 100, 255)));
                plot_ui.vline(VLine::new("Current Detuning", self.pump_detuning_mhz).color(Color32::YELLOW));
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let p_th = self.synthesizer.soliton.compute_mi_threshold_power_mw();
            ui.metric("Pump Power", format!("{:.2} mW", self.pump_power_mw));
            ui.metric("Threshold P_th", format!("{:.2} mW", p_th));
            ui.metric("Parametric Gain Gamma", format!("{:.2e} (W*m)^-1", self.non_linear_gain_gamma));
            ui.metric("Detuning Delta", format!("{:.2} MHz", self.pump_detuning_mhz));
        });
    }

    fn render_synthesizer_tab(&mut self, ui: &mut Ui) {
        let mut recompute_requested = false;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Timing Synthesis & Locking");
                ui.group(|ui| {
                    if ui.add(egui::Slider::new(&mut self.f_ceo_target_mhz, 20.0..=150.0).text("f_ceo Target (MHz)")).changed() {
                        recompute_requested = true;
                    }
                    if ui.add(egui::Slider::new(&mut self.pll_bandwidth_khz, 50.0..=500.0).text("PLL Bandwidth (kHz)")).changed() {
                        recompute_requested = true;
                    }
                });

                let snr = self.synthesizer.synthesizer.f2f_beatnote_snr_db();
                let jitter = self.synthesizer.synthesizer.compute_integrated_timing_jitter_fs();
                let adev_1s = self.synthesizer.synthesizer.allan_deviation_at_1s();
                let l_10k = self.synthesizer.synthesizer.phase_noise_at_offset(10.0e3);

                ui.add_space(8.0);
                ui.metric("f-2f Beat-Note SNR", format!("{:.1} dB", snr));
                ui.metric("Phase Noise @ 10 kHz", format!("{:.1} dBc/Hz", l_10k));
                ui.metric("Integrated Jitter", format!("{:.2} fs", jitter));
                ui.metric("Allan Deviation (1s)", format!("{:.2e}", adev_1s));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("Single-Sideband Phase Noise L(f)");
                let pnoise_pts: Vec<[f64; 2]> = self.cached_phase_noise.iter().map(|p| [p.offset_frequency_hz.log10(), p.phase_noise_dbc_hz]).collect();
                Plot::new("phase_noise_plot")
                    .height(180.0)
                    .width(440.0)
                    .x_axis_label("Log10 Offset Frequency (Hz)")
                    .y_axis_label("L(f) (dBc/Hz)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("SSB Phase Noise", PlotPoints::new(pnoise_pts)).color(Color32::from_rgb(50, 220, 150)));
                        plot_ui.hline(HLine::new("Target 10kHz Limit", -120.0).color(Color32::from_rgb(255, 80, 80)));
                    });

                ui.heading("Two-Sample Allan Deviation sigma_y(tau)");
                let adev_pts: Vec<[f64; 2]> = self.cached_allan_deviation.iter().map(|p| [p.tau_seconds.log10(), p.adev.log10()]).collect();
                Plot::new("allan_deviation_plot")
                    .height(160.0)
                    .width(440.0)
                    .x_axis_label("Log10 Averaging Time tau (s)")
                    .y_axis_label("Log10 sigma_y(tau)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(Line::new("Allan Deviation", PlotPoints::new(adev_pts)).color(Color32::from_rgb(255, 200, 50)));
                    });
            });
        });

        if recompute_requested {
            self.recompute();
        }
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        let mut recompute_requested = false;

        let passed_count = self.cached_audit_report.passed_count;
        let total_count = self.cached_audit_report.total_count;
        let is_fully_compliant = self.cached_audit_report.is_fully_compliant;

        let pass_color = Color32::from_rgb(80, 220, 120);
        let fail_color = Color32::from_rgb(255, 80, 80);

        ui.horizontal(|ui| {
            let score_text = format!("Score: {}/{} PASS", passed_count, total_count);
            let score_color = if is_fully_compliant { pass_color } else { fail_color };
            ui.label(RichText::new(score_text).color(score_color).heading());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Re-run Physics Audit").clicked() {
                    recompute_requested = true;
                }
            });
        });

        ui.add_space(6.0);

        egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
            for (idx, item) in self.cached_audit_report.items.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let status_badge = if item.passed {
                            RichText::new("[PASS]").color(pass_color).strong()
                        } else {
                            RichText::new("[FAIL]").color(fail_color).strong()
                        };
                        ui.label(status_badge);
                        ui.strong(format!("{}. {}", idx + 1, item.name));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("Measured: {}", item.measured)).color(Color32::from_rgb(220, 220, 255)));
                        ui.separator();
                        ui.label(RichText::new(format!("Threshold: {}", item.threshold)).color(Color32::GRAY));
                    });
                    ui.label(RichText::new(&item.details).italics().color(Color32::from_rgb(180, 180, 190)));
                });
                ui.add_space(2.0);
            }
        });

        if recompute_requested {
            self.recompute();
        }
    }
}

trait MetricUiExt {
    fn metric(&mut self, label: &str, value: String);
}

impl MetricUiExt for Ui {
    fn metric(&mut self, label: &str, value: String) {
        self.group(|ui| {
            ui.label(RichText::new(label).small().color(Color32::GRAY));
            ui.label(RichText::new(value).strong().color(Color32::WHITE));
        });
    }
}
