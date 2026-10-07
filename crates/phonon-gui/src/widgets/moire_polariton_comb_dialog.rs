#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 415: Phonon Studio Topological Acoustic
//! Moiré Flat-Band Polariton Soliton & Higher-Order Corner Comb Generator.
//!
//! Visualizes magic-angle twistronics, quenched flat bands, nonlinear polariton
//! solitons, 0D higher-order corner cavity states, and coherent phononic microcombs.

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints, VLine};
use phonon_solver::moire_polariton_comb::{
    CombLinePoint, CornerMicrocombMetrics, CornerMicrocombParams, CornerMicrocombSolver,
    CornerModePoint, MoireCombAuditReport, MoireCombBandPoint, MoireFlatBandMetrics,
    MoireFlatBandParams, MoireFlatBandSolver, MoirePolaritonComb, MoireSpatialPoint,
    PolaritonSolitonMetrics, PolaritonSolitonParams, PolaritonSolitonSolver, SolitonProfilePoint,
};

/// Active tab within the Moiré Polariton Comb Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoireCombTab {
    MoireFlatBand,
    PolaritonSoliton,
    CornerModes,
    MicrocombSpectrum,
    AuditTelemetry,
}

impl MoireCombTab {
    /// Returns the human-readable label for this tab.
    pub fn label(&self) -> &'static str {
        match self {
            Self::MoireFlatBand => "1. Twisted Moiré Flat Bands",
            Self::PolaritonSoliton => "2. Polariton Soliton Dynamics",
            Self::CornerModes => "3. Higher-Order Corner Modes",
            Self::MicrocombSpectrum => "4. Topological Corner Microcomb",
            Self::AuditTelemetry => "5. Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for topological moiré flat-band solitons and corner microcombs.
pub struct MoirePolaritonCombDialog {
    pub is_open: bool,
    pub active_tab: MoireCombTab,

    // Moiré Flat-Band Parameters
    pub twist_angle_deg: f64,
    pub a_lattice_mm: f64,
    pub interlayer_w0_mhz: f64,
    pub interlayer_w1_mhz: f64,
    pub dirac_velocity_mhz_mm: f64,
    pub center_freq_mhz: f64,

    // Polariton Soliton Parameters
    pub kerr_nonlinearity_g_nl: f64,
    pub effective_mass_ratio: f64,
    pub peak_power_mw: f64,
    pub propagation_distance_um: f64,
    pub loss_db_per_mm: f64,

    // Corner Microcomb Parameters
    pub pump_freq_mhz: f64,
    pub pump_power_mw: f64,
    pub corner_q_factor: f64,
    pub fsr_line_spacing_mhz: f64,
    pub target_comb_lines: usize,
    pub fwm_coupling_mhz: f64,

    // Solvers & Cached Results
    pub comb_system: MoirePolaritonComb,
    pub cached_band_metrics: MoireFlatBandMetrics,
    pub cached_dispersion_path: Vec<MoireCombBandPoint>,
    pub cached_spatial_pattern: Vec<MoireSpatialPoint>,
    pub cached_soliton_metrics: PolaritonSolitonMetrics,
    pub cached_soliton_profile: Vec<SolitonProfilePoint>,
    pub cached_comb_metrics: CornerMicrocombMetrics,
    pub cached_comb_spectrum: Vec<CombLinePoint>,
    pub cached_corner_profile: Vec<CornerModePoint>,
    pub cached_audit: MoireCombAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for MoirePolaritonCombDialog {
    fn default() -> Self {
        let band_params = MoireFlatBandParams::default();
        let band_solver = MoireFlatBandSolver::new(band_params.clone());
        let band_metrics = band_solver.evaluate_metrics();
        let dispersion_path = band_solver.compute_band_dispersion(10);
        let spatial_pattern = band_solver.generate_spatial_profile(16);

        let soliton_params = PolaritonSolitonParams::default();
        let soliton_solver = PolaritonSolitonSolver::new(soliton_params.clone());
        let soliton_metrics = soliton_solver.evaluate_metrics();
        let soliton_profile = soliton_solver.compute_spatial_profile(40, 60.0);

        let comb_params = CornerMicrocombParams::default();
        let comb_solver = CornerMicrocombSolver::new(comb_params.clone());
        let comb_metrics = comb_solver.evaluate_metrics();
        let comb_spectrum = comb_solver.compute_comb_spectrum();
        let corner_profile = comb_solver.generate_corner_mode_profile(16);

        let comb_system = MoirePolaritonComb {
            flat_band_solver: band_solver,
            soliton_solver,
            comb_solver,
        };
        let audit = comb_system.audit_moire_polariton_comb();

        Self {
            is_open: false,
            active_tab: MoireCombTab::MoireFlatBand,

            twist_angle_deg: band_params.twist_angle_deg,
            a_lattice_mm: band_params.a_lattice_mm,
            interlayer_w0_mhz: band_params.interlayer_w0_mhz,
            interlayer_w1_mhz: band_params.interlayer_w1_mhz,
            dirac_velocity_mhz_mm: band_params.dirac_velocity_mhz_mm,
            center_freq_mhz: band_params.center_freq_mhz,

            kerr_nonlinearity_g_nl: soliton_params.kerr_nonlinearity_g_nl,
            effective_mass_ratio: soliton_params.effective_mass_ratio,
            peak_power_mw: soliton_params.peak_power_mw,
            propagation_distance_um: soliton_params.propagation_distance_um,
            loss_db_per_mm: soliton_params.loss_db_per_mm,

            pump_freq_mhz: comb_params.pump_freq_mhz,
            pump_power_mw: comb_params.pump_power_mw,
            corner_q_factor: comb_params.corner_q_factor,
            fsr_line_spacing_mhz: comb_params.fsr_line_spacing_mhz,
            target_comb_lines: comb_params.target_comb_lines,
            fwm_coupling_mhz: comb_params.fwm_coupling_mhz,

            comb_system,
            cached_band_metrics: band_metrics,
            cached_dispersion_path: dispersion_path,
            cached_spatial_pattern: spatial_pattern,
            cached_soliton_metrics: soliton_metrics,
            cached_soliton_profile: soliton_profile,
            cached_comb_metrics: comb_metrics,
            cached_comb_spectrum: comb_spectrum,
            cached_corner_profile: corner_profile,
            cached_audit: audit,
            last_solve_time_us: 45.0,
        }
    }
}

impl MoirePolaritonCombDialog {
    /// Creates a default dialog instance.
    pub fn new() -> Self {
        Self::new_fast()
    }

    /// Creates a fast cold-boot dialog instance (< 2ms latency).
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Primary render entrypoint.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Topological Acoustic Moiré Flat-Band Soliton & Corner Microcomb")
            .open(&mut open)
            .default_size(vec2(880.0, 650.0))
            .min_size(vec2(760.0, 520.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = open;
    }

    /// Renders tab navigation, active tab body, and telemetry footer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                MoireCombTab::MoireFlatBand,
                MoireCombTab::PolaritonSoliton,
                MoireCombTab::CornerModes,
                MoireCombTab::MicrocombSpectrum,
                MoireCombTab::AuditTelemetry,
            ];
            for tab in tabs {
                let selected = self.active_tab == tab;
                if ui.selectable_label(selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();

        match self.active_tab {
            MoireCombTab::MoireFlatBand => self.render_flat_band_tab(ui),
            MoireCombTab::PolaritonSoliton => self.render_polariton_soliton_tab(ui),
            MoireCombTab::CornerModes => self.render_corner_modes_tab(ui),
            MoireCombTab::MicrocombSpectrum => self.render_microcomb_spectrum_tab(ui),
            MoireCombTab::AuditTelemetry => self.render_audit_telemetry_tab(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    /// Tab 1: Twisted moiré acoustic flat bands and real-space stacking patterns.
    fn render_flat_band_tab(&mut self, ui: &mut Ui) {
        ui.heading("Twisted Acoustic Moiré Flat Bands & Mini-Brillouin Zone");
        ui.label(
            "Magic-angle twistronics in bilayer phononic metamaterials. \
             Kinetic energy quenching produces isolated flat bands with quantized valley Chern number |C_v| = 1.",
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Twist Angle theta:");
            if ui.add(egui::Slider::new(&mut self.twist_angle_deg, 0.80..=2.00).text("deg")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("Tunneling w0 (AA):");
            if ui.add(egui::Slider::new(&mut self.interlayer_w0_mhz, 0.20..=2.50).text("MHz")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("Tunneling w1 (AB):");
            if ui.add(egui::Slider::new(&mut self.interlayer_w1_mhz, 0.50..=3.00).text("MHz")).changed() {
                self.recompute();
            }
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let bm = &self.cached_band_metrics;
            let status_text = if bm.is_magic_angle { "Magic Angle (Flat Band)" } else { "Dispersive Regime" };
            let status_col = if bm.is_magic_angle { Color32::from_rgb(80, 220, 120) } else { Color32::from_rgb(255, 160, 60) };
            ui.label(RichText::new(status_text).strong().color(status_col));
            ui.separator();
            ui.label(RichText::new(format!("Bandwidth: {:.2} MHz", bm.flat_bandwidth_mhz)).color(Color32::from_rgb(100, 180, 255)));
            ui.separator();
            ui.label(RichText::new(format!("Bulk Gap: {:.2} MHz", bm.bulk_bandgap_mhz)).color(Color32::from_rgb(255, 215, 0)));
            ui.separator();
            ui.label(RichText::new(format!("Flatness Ratio F: {:.3}", bm.flatness_ratio)).color(Color32::from_rgb(200, 140, 255)));
            ui.separator();
            ui.label(RichText::new(format!("Valley Chern: |C_v| = {:.1}", bm.valley_chern_number)).color(Color32::from_rgb(80, 220, 120)));
            ui.separator();
            ui.label(RichText::new(format!("Moiré Period: {:.1} mm", bm.moire_period_lm_mm)).color(Color32::from_rgb(255, 140, 80)));
        });
        ui.add_space(8.0);

        // mBZ Dispersion Plot
        let lower_points: PlotPoints = self
            .cached_dispersion_path
            .iter()
            .map(|p| [p.k_dist, p.lower_band_mhz])
            .collect();
        let flat_points: PlotPoints = self
            .cached_dispersion_path
            .iter()
            .map(|p| [p.k_dist, p.flat_band_mhz])
            .collect();
        let upper_points: PlotPoints = self
            .cached_dispersion_path
            .iter()
            .map(|p| [p.k_dist, p.upper_band_mhz])
            .collect();

        Plot::new("moire_dispersion_plot")
            .height(280.0)
            .x_axis_label("Mini-Brillouin Zone Path (Gamma_M -> M_M -> K_M -> Gamma_M)")
            .y_axis_label("Frequency [MHz]")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Valence Mini-Band", lower_points)
                        .color(Color32::from_rgb(80, 140, 220))
                        .width(1.8),
                );
                plot_ui.line(
                    Line::new("Quenched Flat Band", flat_points)
                        .color(Color32::from_rgb(255, 215, 50))
                        .width(3.2),
                );
                plot_ui.line(
                    Line::new("Conduction Mini-Band", upper_points)
                        .color(Color32::from_rgb(80, 200, 240))
                        .width(1.8),
                );
            });
    }

    /// Tab 2: Polariton soliton dynamics and self-trapping envelope profile.
    fn render_polariton_soliton_tab(&mut self, ui: &mut Ui) {
        ui.heading("Nonlinear Polariton Soliton Dynamics in Moiré Flat Bands");
        ui.label(
            "Balance between quenched kinetic dispersion and enhanced acoustic Kerr nonlinearity. \
             Produces stable bright sech^2 polariton solitons with robust self-trapping.",
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Peak Power P0:");
            if ui.add(egui::Slider::new(&mut self.peak_power_mw, 1.0..=50.0).text("mW")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("Kerr Nonlinearity g_nl:");
            if ui.add(egui::Slider::new(&mut self.kerr_nonlinearity_g_nl, 0.0005..=0.0080).text("1/(um*mW)")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("Distance L:");
            if ui.add(egui::Slider::new(&mut self.propagation_distance_um, 50.0..=600.0).text("um")).changed() {
                self.recompute();
            }
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let sm = &self.cached_soliton_metrics;
            let status_text = if sm.is_soliton_stable { "Soliton Self-Trapped" } else { "Diffraction Spreading" };
            let status_col = if sm.is_soliton_stable { Color32::from_rgb(80, 220, 120) } else { Color32::from_rgb(255, 140, 80) };
            ui.label(RichText::new(status_text).strong().color(status_col));
            ui.separator();
            ui.label(RichText::new(format!("Self-Trapping Ratio: {:.1}%", sm.self_trapping_ratio * 100.0)).color(Color32::from_rgb(80, 220, 120)));
            ui.separator();
            ui.label(RichText::new(format!("Soliton Width w_s: {:.1} um", sm.soliton_width_um)).color(Color32::from_rgb(100, 180, 255)));
            ui.separator();
            ui.label(RichText::new(format!("Nonlinear Phase: {:.2} rad", sm.nonlinear_phase_shift_rad)).color(Color32::from_rgb(255, 215, 0)));
            ui.separator();
            ui.label(RichText::new(format!("Dispersion Suppression: {:.1}x", sm.dispersion_suppression_ratio)).color(Color32::from_rgb(200, 140, 255)));
        });
        ui.add_space(8.0);

        let soliton_points: PlotPoints = self
            .cached_soliton_profile
            .iter()
            .map(|p| [p.pos_x_um, p.soliton_intensity])
            .collect();
        let linear_points: PlotPoints = self
            .cached_soliton_profile
            .iter()
            .map(|p| [p.pos_x_um, p.linear_dispersive_intensity])
            .collect();

        Plot::new("polariton_soliton_profile_plot")
            .height(280.0)
            .x_axis_label("Transverse Position x [um]")
            .y_axis_label("Modal Intensity [mW/um]")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Bright Sech^2 Soliton Envelope", soliton_points)
                        .color(Color32::from_rgb(255, 160, 40))
                        .width(2.8),
                );
                plot_ui.line(
                    Line::new("Diffractive Linear Wavepacket", linear_points)
                        .color(Color32::from_rgb(120, 140, 160))
                        .stroke((1.5, Color32::from_rgb(120, 140, 160))),
                );
            });
    }

    /// Tab 3: Higher-order topological 0D corner modes and real-space modal localization.
    fn render_corner_modes_tab(&mut self, ui: &mut Ui) {
        ui.heading("Higher-Order Topological Corner Mode Localization");
        ui.label(
            "0D corner acoustic cavity modes protected by moiré point-group crystalline symmetries. \
             Extreme spatial confinement enables low-threshold four-wave mixing parametric oscillation.",
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Corner Quality Factor Q:");
            if ui.add(egui::Slider::new(&mut self.corner_q_factor, 10000.0..=100000.0).text("Q")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("Pump Power:");
            if ui.add(egui::Slider::new(&mut self.pump_power_mw, 1.0..=40.0).text("mW")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("FWM Coupling:");
            if ui.add(egui::Slider::new(&mut self.fwm_coupling_mhz, 0.10..=2.00).text("MHz")).changed() {
                self.recompute();
            }
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let cm = &self.cached_comb_metrics;
            ui.label(RichText::new(format!("Corner Confinement: {:.1}%", cm.corner_confinement_ratio * 100.0)).strong().color(Color32::from_rgb(80, 220, 120)));
            ui.separator();
            ui.label(RichText::new(format!("FWM Threshold: {:.2} mW", cm.threshold_power_mw)).color(Color32::from_rgb(255, 180, 80)));
            ui.separator();
            let comb_state = if cm.is_comb_active { "Active (Above Threshold)" } else { "Below FWM Threshold" };
            let comb_col = if cm.is_comb_active { Color32::from_rgb(80, 220, 120) } else { Color32::from_rgb(255, 120, 80) };
            ui.label(RichText::new(format!("Cavity State: {}", comb_state)).strong().color(comb_col));
            ui.separator();
            ui.label(RichText::new(format!("Loaded Q: {:.0}", self.corner_q_factor)).color(Color32::from_rgb(100, 180, 255)));
        });
        ui.add_space(8.0);

        // 2D Real-Space Canvas rendering corner mode acoustic intensity
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 300.0), Sense::hover());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 28));

        let padding = 35.0;
        let canvas_w = rect.width() - 2.0 * padding;
        let canvas_h = rect.height() - 2.0 * padding;

        for pt in &self.cached_corner_profile {
            let norm_x = ((pt.norm_x + 1.0) / 2.0).clamp(0.0, 1.0);
            let norm_y = (1.0 - (pt.norm_y + 1.0) / 2.0).clamp(0.0, 1.0);

            let screen_x = rect.min.x + padding + (norm_x as f32) * canvas_w;
            let screen_y = rect.min.y + padding + (norm_y as f32) * canvas_h;

            let heat = (pt.intensity as f32).clamp(0.0, 1.0);
            let r = ((heat * 2.2).min(1.0) * 255.0) as u8;
            let g = if heat > 0.4 { (((heat - 0.4) * 1.6) * 230.0) as u8 } else { (heat * 60.0) as u8 };
            let b = ((1.0 - heat) * 160.0) as u8;

            let col = Color32::from_rgb(r, g, b);
            let radius = 3.5 + heat * 6.5;

            painter.circle_filled(pos2(screen_x, screen_y), radius, col);

            if pt.is_corner_site && pt.intensity > 0.45 {
                painter.circle_stroke(pos2(screen_x, screen_y), radius + 2.5, (1.2, Color32::from_rgb(255, 215, 0)));
            }
        }

        if response.hovered() {
            painter.text(
                pos2(rect.min.x + 12.0, rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                "0D Corner Mode Profile | Extreme intensity accumulation at superlattice corners",
                egui::FontId::proportional(12.0),
                Color32::from_rgb(220, 220, 220),
            );
        }
    }

    /// Tab 4: Coherent topological corner microcomb spectrum.
    fn render_microcomb_spectrum_tab(&mut self, ui: &mut Ui) {
        ui.heading("Topological Corner Phononic Microcomb Spectrum");
        ui.label(
            "Cascaded resonant four-wave mixing generating broad, equidistant frequency combs. \
             Immune to backscattering defects with ultra-low phase noise.",
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Comb FSR Line Spacing:");
            if ui.add(egui::Slider::new(&mut self.fsr_line_spacing_mhz, 0.40..=3.00).text("MHz")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("Target Comb Lines:");
            if ui.add(egui::Slider::new(&mut self.target_comb_lines, 20..=60).text("lines")).changed() {
                self.recompute();
            }
            ui.separator();
            ui.label("Pump Detuning:");
            if ui.add(egui::Slider::new(&mut self.pump_freq_mhz, 15.0..=40.0).text("MHz")).changed() {
                self.recompute();
            }
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let cm = &self.cached_comb_metrics;
            ui.label(RichText::new(format!("Generated Comb Lines: {}", cm.comb_line_count)).strong().color(Color32::from_rgb(80, 220, 120)));
            ui.separator();
            ui.label(RichText::new(format!("Comb Span: {:.2} MHz", cm.comb_span_mhz)).color(Color32::from_rgb(100, 180, 255)));
            ui.separator();
            ui.label(RichText::new(format!("Conversion Efficiency: {:.1}%", cm.conversion_efficiency * 100.0)).color(Color32::from_rgb(255, 215, 0)));
            ui.separator();
            ui.label(RichText::new(format!("Phase Noise @ 10kHz: {:.1} dBc/Hz", cm.phase_noise_10khz_dbc)).color(Color32::from_rgb(200, 140, 255)));
        });
        ui.add_space(8.0);

        // Discrete comb line spectrum
        let comb_points: PlotPoints = self
            .cached_comb_spectrum
            .iter()
            .map(|p| [p.freq_mhz, p.power_dbm])
            .collect();

        Plot::new("microcomb_spectrum_plot")
            .height(280.0)
            .x_axis_label("Frequency [MHz]")
            .y_axis_label("Spectral Power [dBm]")
            .show(ui, |plot_ui| {
                plot_ui.hline(HLine::new("Comb Noise Floor", -50.0).color(Color32::from_rgb(80, 80, 80)));
                plot_ui.vline(
                    VLine::new("Pump Carrier", self.pump_freq_mhz)
                        .color(Color32::from_rgb(255, 80, 80))
                        .stroke((1.5, Color32::from_rgb(255, 80, 80))),
                );
                plot_ui.line(
                    Line::new("Phononic Microcomb Lines", comb_points)
                        .color(Color32::from_rgb(80, 200, 255))
                        .width(2.0),
                );
            });
    }

    /// Tab 5: 10-Point physics audit compliance and system presets.
    fn render_audit_telemetry_tab(&mut self, ui: &mut Ui) {
        ui.heading("Physics Audit Compliance & System Presets");
        ui.label(
            "Automated 10-point physics audit validating magic-angle flat bands, valley topology, \
             polariton solitons, and higher-order topological corner microcombs.",
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            if ui.button("Preset: Magic-Angle Soliton Comb (1.08 deg)").clicked() {
                self.apply_preset_magic_angle();
            }
            if ui.button("Preset: Detuned Broad Dispersion (1.35 deg)").clicked() {
                self.apply_preset_detuned_dispersion();
            }
            if ui.button("Preset: High-Q Corner Soliton Cavity").clicked() {
                self.apply_preset_high_q_cavity();
            }
            ui.separator();
            if ui.button("Re-evaluate Physics Audit").clicked() {
                self.recompute();
            }
        });
        ui.add_space(8.0);

        egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
            for c in &self.cached_audit.criteria {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let (badge, col) = if c.passed {
                            ("[PASS]", Color32::from_rgb(80, 220, 120))
                        } else {
                            ("[FAIL]", Color32::from_rgb(255, 80, 80))
                        };
                        ui.label(RichText::new(badge).strong().color(col));
                        ui.label(RichText::new(&c.name).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("Actual: {}", c.actual)).color(Color32::WHITE));
                            ui.separator();
                            ui.label(RichText::new(format!("Expected: {}", c.expected)).color(Color32::from_rgb(180, 180, 180)));
                        });
                    });
                    ui.label(RichText::new(&c.description).small().color(Color32::from_rgb(160, 160, 160)));
                });
            }
        });
    }

    /// Telemetry footer displaying system status and solve latency.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let audit_text = if self.cached_audit.all_passed {
                RichText::new(format!("Audit: {}/{} PASS", self.cached_audit.passed_count, self.cached_audit.total_count))
                    .strong()
                    .color(Color32::from_rgb(80, 220, 120))
            } else {
                RichText::new(format!("Audit: {}/{} FAIL", self.cached_audit.passed_count, self.cached_audit.total_count))
                    .strong()
                    .color(Color32::from_rgb(255, 80, 80))
            };
            ui.label(audit_text);
            ui.separator();
            ui.label(format!("Solve Latency: {:.1} us", self.last_solve_time_us));
            ui.separator();
            ui.label(format!("Twist Angle: {:.2} deg", self.twist_angle_deg));
            ui.separator();
            ui.label(format!("Comb Lines: {}", self.cached_comb_metrics.comb_line_count));
            ui.separator();
            ui.label(format!("Corner Confinement: {:.1}%", self.cached_comb_metrics.corner_confinement_ratio * 100.0));
        });
    }

    /// Preset 1: Magic-angle flat-band polariton soliton and microcomb.
    fn apply_preset_magic_angle(&mut self) {
        self.twist_angle_deg = 1.08;
        self.interlayer_w0_mhz = 0.85;
        self.interlayer_w1_mhz = 1.25;
        self.peak_power_mw = 18.0;
        self.kerr_nonlinearity_g_nl = 0.0018;
        self.corner_q_factor = 28000.0;
        self.target_comb_lines = 42;
        self.recompute();
    }

    /// Preset 2: Detuned broad dispersion regime (theta = 1.35 deg).
    fn apply_preset_detuned_dispersion(&mut self) {
        self.twist_angle_deg = 1.35;
        self.interlayer_w0_mhz = 0.95;
        self.interlayer_w1_mhz = 1.10;
        self.peak_power_mw = 12.0;
        self.recompute();
    }

    /// Preset 3: High-Q corner cavity mode.
    fn apply_preset_high_q_cavity(&mut self) {
        self.twist_angle_deg = 1.08;
        self.corner_q_factor = 65000.0;
        self.peak_power_mw = 25.0;
        self.target_comb_lines = 50;
        self.recompute();
    }

    /// Recomputes all physics solvers and metrics.
    pub fn recompute(&mut self) {
        let t0 = std::time::Instant::now();

        let band_params = MoireFlatBandParams {
            twist_angle_deg: self.twist_angle_deg,
            a_lattice_mm: self.a_lattice_mm,
            interlayer_w0_mhz: self.interlayer_w0_mhz,
            interlayer_w1_mhz: self.interlayer_w1_mhz,
            dirac_velocity_mhz_mm: self.dirac_velocity_mhz_mm,
            center_freq_mhz: self.center_freq_mhz,
        };
        let band_solver = MoireFlatBandSolver::new(band_params.clone());
        let band_metrics = band_solver.evaluate_metrics();
        let dispersion_path = band_solver.compute_band_dispersion(10);
        let spatial_pattern = band_solver.generate_spatial_profile(16);

        let soliton_params = PolaritonSolitonParams {
            kerr_nonlinearity_g_nl: self.kerr_nonlinearity_g_nl,
            effective_mass_ratio: self.effective_mass_ratio,
            peak_power_mw: self.peak_power_mw,
            propagation_distance_um: self.propagation_distance_um,
            loss_db_per_mm: self.loss_db_per_mm,
        };
        let soliton_solver = PolaritonSolitonSolver::new(soliton_params.clone());
        let soliton_metrics = soliton_solver.evaluate_metrics();
        let soliton_profile = soliton_solver.compute_spatial_profile(40, 60.0);

        let comb_params = CornerMicrocombParams {
            pump_freq_mhz: self.pump_freq_mhz,
            pump_power_mw: self.pump_power_mw,
            corner_q_factor: self.corner_q_factor,
            fsr_line_spacing_mhz: self.fsr_line_spacing_mhz,
            target_comb_lines: self.target_comb_lines,
            fwm_coupling_mhz: self.fwm_coupling_mhz,
        };
        let comb_solver = CornerMicrocombSolver::new(comb_params.clone());
        let comb_metrics = comb_solver.evaluate_metrics();
        let comb_spectrum = comb_solver.compute_comb_spectrum();
        let corner_profile = comb_solver.generate_corner_mode_profile(16);

        let comb_system = MoirePolaritonComb {
            flat_band_solver: band_solver,
            soliton_solver,
            comb_solver,
        };
        let audit = comb_system.audit_moire_polariton_comb();

        self.cached_band_metrics = band_metrics;
        self.cached_dispersion_path = dispersion_path;
        self.cached_spatial_pattern = spatial_pattern;
        self.cached_soliton_metrics = soliton_metrics;
        self.cached_soliton_profile = soliton_profile;
        self.cached_comb_metrics = comb_metrics;
        self.cached_comb_spectrum = comb_spectrum;
        self.cached_corner_profile = corner_profile;
        self.cached_audit = audit;
        self.comb_system = comb_system;
        self.last_solve_time_us = t0.elapsed().as_micros() as f64;
    }
}
