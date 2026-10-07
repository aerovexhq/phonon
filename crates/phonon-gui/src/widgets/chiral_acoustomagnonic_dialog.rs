#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 423: Phonon Studio Quantum Metamaterial
//! Chiral Acoustomagnonic Isolator & Cryogenic Microwave Qubit Circulator.
//!
//! Visualizes coupled magnon-phonon polariton dispersion, synthetic gauge fields,
//! non-reciprocal SAW transmission on YIG/LiNbO3 heterostructures, and sub-kelvin
//! 3-port circulators for transmon qubit thermal back-action protection.

use egui::{pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::chiral_acoustomagnonic_isolator::{
    AcoustomagnonicAuditReport, AcoustomagnonicDispersionPoint, AcoustomagnonicParams,
    ChiralAcoustomagnonicProcessor, CryogenicCirculatorMetrics, CryogenicCirculatorParams,
    QubitCirculatorSMatrix, SawFrequencyResponsePoint, SawIsolatorMetrics,
};

/// Active tab in the Chiral Acoustomagnonic Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcoustomagnonicTab {
    AcoustomagnonicDispersion,
    NonReciprocalSawIsolator,
    CryogenicQubitCirculator,
    RealSpaceHeterostructure,
    AuditTelemetry,
}

impl AcoustomagnonicTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AcoustomagnonicDispersion => "Acoustomagnonic Dispersion",
            Self::NonReciprocalSawIsolator => "Non-Reciprocal SAW Isolator",
            Self::CryogenicQubitCirculator => "Cryogenic Qubit Circulator",
            Self::RealSpaceHeterostructure => "Heterostructure Canvas",
            Self::AuditTelemetry => "Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 423.
pub struct ChiralAcoustomagnonicDialog {
    pub is_open: bool,
    pub active_tab: AcoustomagnonicTab,

    // Dispersion Parameters
    pub center_freq_ghz: f64,
    pub saw_velocity_ms: f64,
    pub saturation_magnetization_g: f64,
    pub bias_field_oe: f64,
    pub coupling_strength_mhz: f64,
    pub synthetic_gauge_phase_rad: f64,

    // Isolator Parameters
    pub waveguide_length_mm: f64,

    // Cryogenic Circulator Parameters
    pub operating_temp_k: f64,
    pub dispersive_shift_mhz: f64,
    pub cavity_linewidth_mhz: f64,
    pub integration_time_ns: f64,

    // Solver & Cached Results
    pub processor: ChiralAcoustomagnonicProcessor,
    pub cached_dispersion: Vec<AcoustomagnonicDispersionPoint>,
    pub cached_saw_metrics: SawIsolatorMetrics,
    pub cached_spectrum: Vec<SawFrequencyResponsePoint>,
    pub cached_s_matrix: QubitCirculatorSMatrix,
    pub cached_circ_metrics: CryogenicCirculatorMetrics,
    pub cached_audit: AcoustomagnonicAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ChiralAcoustomagnonicDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ChiralAcoustomagnonicDialog {
    /// Fast cold-boot constructor with pre-computed baseline data (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let processor = ChiralAcoustomagnonicProcessor::default();
        let cached_dispersion = processor.dispersion_solver.compute_dispersion_curve(40);
        let cached_saw_metrics = processor.saw_isolator.compute_metrics();
        let cached_spectrum = processor.saw_isolator.compute_spectrum(51, 100.0);
        let cached_s_matrix = processor.qubit_circulator.evaluate_s_matrix();
        let cached_circ_metrics = processor.qubit_circulator.compute_metrics();
        let cached_audit = processor.audit_acoustomagnonic_processor();

        Self {
            is_open: false,
            active_tab: AcoustomagnonicTab::AcoustomagnonicDispersion,
            center_freq_ghz: processor.dispersion_solver.params.center_freq_ghz,
            saw_velocity_ms: processor.dispersion_solver.params.saw_velocity_ms,
            saturation_magnetization_g: processor
                .dispersion_solver
                .params
                .saturation_magnetization_g,
            bias_field_oe: processor.dispersion_solver.params.bias_field_oe,
            coupling_strength_mhz: processor.dispersion_solver.params.coupling_strength_mhz,
            synthetic_gauge_phase_rad: processor
                .dispersion_solver
                .params
                .synthetic_gauge_phase_rad,
            waveguide_length_mm: processor.saw_isolator.waveguide_length_mm,
            operating_temp_k: processor.qubit_circulator.params.operating_temp_k,
            dispersive_shift_mhz: processor.qubit_circulator.params.dispersive_shift_mhz,
            cavity_linewidth_mhz: processor.qubit_circulator.params.cavity_linewidth_mhz,
            integration_time_ns: processor.qubit_circulator.params.integration_time_ns,
            processor,
            cached_dispersion,
            cached_saw_metrics,
            cached_spectrum,
            cached_s_matrix,
            cached_circ_metrics,
            cached_audit,
            last_solve_time_us: 120.0,
        }
    }

    /// Recomputes dispersion curves, S-parameter spectrum, and audit metrics.
    pub fn recompute(&mut self) {
        let start = std::time::Instant::now();

        let ac_params = AcoustomagnonicParams {
            center_freq_ghz: self.center_freq_ghz,
            saw_velocity_ms: self.saw_velocity_ms,
            saturation_magnetization_g: self.saturation_magnetization_g,
            bias_field_oe: self.bias_field_oe,
            coupling_strength_mhz: self.coupling_strength_mhz,
            synthetic_gauge_phase_rad: self.synthetic_gauge_phase_rad,
            ..Default::default()
        };

        let circ_params = CryogenicCirculatorParams {
            operating_temp_k: self.operating_temp_k,
            center_freq_ghz: self.center_freq_ghz + 1.0,
            dispersive_shift_mhz: self.dispersive_shift_mhz,
            cavity_linewidth_mhz: self.cavity_linewidth_mhz,
            integration_time_ns: self.integration_time_ns,
            ..Default::default()
        };

        self.processor = ChiralAcoustomagnonicProcessor::new(
            ac_params,
            self.waveguide_length_mm,
            circ_params,
        );

        self.cached_dispersion = self
            .processor
            .dispersion_solver
            .compute_dispersion_curve(40);
        self.cached_saw_metrics = self.processor.saw_isolator.compute_metrics();
        self.cached_spectrum = self.processor.saw_isolator.compute_spectrum(51, 100.0);
        self.cached_s_matrix = self.processor.qubit_circulator.evaluate_s_matrix();
        self.cached_circ_metrics = self.processor.qubit_circulator.compute_metrics();
        self.cached_audit = self.processor.audit_acoustomagnonic_processor();

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Renders the complete dialog contents into the provided egui::Ui.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Chiral Acoustomagnonic Isolator & Cryogenic Circulator")
                    .strong()
                    .color(Color32::from_rgb(130, 220, 255)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("Recompute Physics").strong()).clicked() {
                    self.recompute();
                }
            });
        });

        ui.separator();

        // Tab bar
        ui.horizontal(|ui| {
            let tabs = [
                AcoustomagnonicTab::AcoustomagnonicDispersion,
                AcoustomagnonicTab::NonReciprocalSawIsolator,
                AcoustomagnonicTab::CryogenicQubitCirculator,
                AcoustomagnonicTab::RealSpaceHeterostructure,
                AcoustomagnonicTab::AuditTelemetry,
            ];
            for tab in tabs {
                let selected = self.active_tab == tab;
                let text = if selected {
                    RichText::new(tab.label())
                        .strong()
                        .color(Color32::from_rgb(255, 215, 0))
                } else {
                    RichText::new(tab.label())
                };
                if ui.selectable_label(selected, text).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            AcoustomagnonicTab::AcoustomagnonicDispersion => self.render_dispersion_tab(ui),
            AcoustomagnonicTab::NonReciprocalSawIsolator => self.render_isolator_tab(ui),
            AcoustomagnonicTab::CryogenicQubitCirculator => self.render_circulator_tab(ui),
            AcoustomagnonicTab::RealSpaceHeterostructure => self.render_heterostructure_tab(ui),
            AcoustomagnonicTab::AuditTelemetry => self.render_audit_tab(ui),
        }
    }

    fn render_dispersion_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Heterostructure Parameters").strong());
                let mut changed = false;

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.center_freq_ghz, 2.0..=8.0)
                            .text("Center Frequency (GHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.bias_field_oe, 300.0..=1500.0)
                            .text("Bias Field H0 (Oe)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.coupling_strength_mhz, 10.0..=80.0)
                            .text("Coupling g_me (MHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.synthetic_gauge_phase_rad, 0.0..=1.2)
                            .text("Synthetic Gauge Phase (rad)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }

                ui.separator();
                let gap_mhz = self.processor.dispersion_solver.polariton_gap_mhz();
                let delta_k = self
                    .processor
                    .dispersion_solver
                    .nonreciprocal_wavevector_splitting();

                ui.label(format!("Polariton Avoided Crossing Gap: {:.2} MHz", gap_mhz));
                ui.label(format!("Wavevector Splitting Delta_k: {:.4} rad/um", delta_k));
                ui.label(format!(
                    "Effective SAW Velocity: {:.1} m/s",
                    self.saw_velocity_ms
                ));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Coupled Magnon-Phonon Polariton Dispersion").strong());

                let mut pts_fwd_l = Vec::new();
                let mut pts_fwd_u = Vec::new();
                let mut pts_bwd_l = Vec::new();
                let mut pts_saw = Vec::new();
                let mut pts_mag = Vec::new();

                for p in &self.cached_dispersion {
                    pts_fwd_l.push([p.k_um, p.forward_lower_ghz]);
                    pts_fwd_u.push([p.k_um, p.forward_upper_ghz]);
                    pts_bwd_l.push([p.k_um, p.backward_lower_ghz]);
                    pts_saw.push([p.k_um, p.bare_saw_ghz]);
                    pts_mag.push([p.k_um, p.bare_magnon_ghz]);
                }

                Plot::new("polariton_dispersion_plot")
                    .height(280.0)
                    .x_axis_label("Wavenumber k (rad/um)")
                    .y_axis_label("Frequency (GHz)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(
                            Line::new("Forward Lower Polariton", PlotPoints::new(pts_fwd_l))
                                .color(Color32::from_rgb(0, 200, 255))
                                .width(2.5),
                        );
                        plot_ui.line(
                            Line::new("Forward Upper Polariton", PlotPoints::new(pts_fwd_u))
                                .color(Color32::from_rgb(100, 240, 160))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("Backward Lower Polariton (-k)", PlotPoints::new(pts_bwd_l))
                                .color(Color32::from_rgb(255, 120, 80))
                                .width(1.8),
                        );
                        plot_ui.line(
                            Line::new("Bare Acoustic SAW", PlotPoints::new(pts_saw))
                                .color(Color32::from_rgb(160, 160, 160))
                                .style(egui_plot::LineStyle::Dashed { length: 4.0 }),
                        );
                        plot_ui.line(
                            Line::new("Bare Damon-Eshbach Spin Wave", PlotPoints::new(pts_mag))
                                .color(Color32::from_rgb(220, 140, 255))
                                .style(egui_plot::LineStyle::Dashed { length: 4.0 }),
                        );
                    });
            });
        });
    }

    fn render_isolator_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Waveguide Geometry").strong());
                let mut changed = false;

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.waveguide_length_mm, 1.0..=6.0)
                            .text("Waveguide Length (mm)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }

                ui.separator();
                let m = &self.cached_saw_metrics;
                ui.label(format!("Forward Insertion Loss: {:.2} dB", m.insertion_loss_db));
                ui.label(format!("Reverse Isolation Depth: {:.1} dB", m.isolation_depth_db));
                ui.label(format!("Isolation Contrast: {:.1} dB", m.isolation_contrast_db));
                ui.label(format!("3-dB Bandwidth: {:.1} MHz", m.bandwidth_3db_mhz));
                ui.label(format!("Return Loss S11: {:.1} dB", m.return_loss_s11_db));
                ui.label(format!(
                    "Forward Transmission: {:.1}%",
                    m.forward_transmission * 100.0
                ));
                ui.label(format!(
                    "Reverse Leakage: {:.4}%",
                    m.backward_transmission * 100.0
                ));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Non-Reciprocal S-Parameter Transmission Spectrum").strong());

                let mut pts_s21 = Vec::new();
                let mut pts_s12 = Vec::new();
                let mut pts_s11 = Vec::new();

                for p in &self.cached_spectrum {
                    pts_s21.push([p.freq_ghz, p.s21_db]);
                    pts_s12.push([p.freq_ghz, p.s12_db]);
                    pts_s11.push([p.freq_ghz, p.s11_db]);
                }

                Plot::new("saw_s_parameter_plot")
                    .height(280.0)
                    .x_axis_label("Frequency (GHz)")
                    .y_axis_label("Transmission (dB)")
                    .show(ui, |plot_ui| {
                        plot_ui.hline(
                            HLine::new("Isolation Threshold (-35 dB)", -35.0)
                                .color(Color32::from_rgb(255, 90, 90)),
                        );
                        plot_ui.line(
                            Line::new("Forward Transmission S21", PlotPoints::new(pts_s21))
                                .color(Color32::from_rgb(50, 220, 100))
                                .width(2.5),
                        );
                        plot_ui.line(
                            Line::new("Reverse Isolation S12", PlotPoints::new(pts_s12))
                                .color(Color32::from_rgb(255, 80, 80))
                                .width(2.0),
                        );
                        plot_ui.line(
                            Line::new("Return Loss S11", PlotPoints::new(pts_s11))
                                .color(Color32::from_rgb(180, 180, 180))
                                .style(egui_plot::LineStyle::Dashed { length: 3.0 }),
                        );
                    });
            });
        });
    }

    fn render_circulator_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Dilution Refrigerator Operating Parameters").strong());
                let mut changed = false;

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.operating_temp_k, 0.010..=0.100)
                            .text("Temperature T (K)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.dispersive_shift_mhz, 1.0..=8.0)
                            .text("Dispersive Shift chi (MHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.cavity_linewidth_mhz, 0.5..=3.0)
                            .text("Cavity Linewidth kappa (MHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.integration_time_ns, 50.0..=500.0)
                            .text("Integration Time (ns)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }

                ui.separator();
                let m = &self.cached_circ_metrics;
                ui.label(format!("Added Noise Quanta: {:.4} quanta", m.added_noise_quanta));
                ui.label(format!(
                    "Thermal Leakage from 4K: {:.2e} photons",
                    m.thermal_leakage_photons
                ));
                ui.label(format!("Thermal Isolation: {:.1} dB", m.thermal_isolation_db));
                ui.label(format!("Qubit Readout SNR: {:.2} dB", m.readout_snr_db));
                ui.label(format!(
                    "QND Readout Fidelity: {:.4}",
                    m.qnd_readout_fidelity
                ));
                ui.label(format!("Measurement Dephasing: {:.2} MHz", m.dephasing_rate_mhz));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("3-Port Cyclic Circulator Scattering Matrix").strong());

                let sm = &self.cached_s_matrix;
                ui.monospace(format!(
                    "[ |S11|={:.4}   |S12|={:.4}   |S13|={:.4} ]",
                    sm.s_reflection_mag, sm.s_reverse_mag, sm.s_forward_mag
                ));
                ui.monospace(format!(
                    "[ |S21|={:.4}   |S22|={:.4}   |S23|={:.4} ]",
                    sm.s_forward_mag, sm.s_reflection_mag, sm.s_reverse_mag
                ));
                ui.monospace(format!(
                    "[ |S31|={:.4}   |S32|={:.4}   |S33|={:.4} ]",
                    sm.s_reverse_mag, sm.s_forward_mag, sm.s_reflection_mag
                ));

                ui.add_space(8.0);
                ui.label(format!("Forward Transmission: {:.2} dB", sm.s_forward_db));
                ui.label(format!("Reverse Isolation: {:.1} dB", sm.s_reverse_db));
                ui.label(format!("Port Return Loss: {:.1} dB", sm.return_loss_db));
                ui.label(format!(
                    "Scattering Matrix Unitarity Deficit: {:.4e}",
                    sm.unitarity_deficit
                ));

                ui.add_space(10.0);
                ui.label(RichText::new("Readout Routing Diagram:").strong());
                ui.label("Port 1 (Qubit Drive Input) -> Port 2 (Transmon Cavity Interface)");
                ui.label("Port 2 (Qubit Reflected Signal) -> Port 3 (Cryogenic HEMT Amplifier Chain)");
                ui.label("Port 3 (HEMT Thermal Noise) -> Isolated from Port 2 by > 35 dB");
            });
        });
    }

    fn render_heterostructure_tab(&mut self, ui: &mut Ui) {
        ui.label(
            RichText::new("YIG / LiNbO3 Chiral Heterostructure Real-Space Canvas")
                .strong()
                .color(Color32::from_rgb(255, 215, 0)),
        );

        let (rect, _response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 240.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Background substrate (LiNbO3 piezoelectric)
        painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 32));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(60, 80, 110)),
            StrokeKind::Inside,
        );

        // YIG Magnetic Garnet Strip in the center
        let yig_rect = Rect::from_min_size(
            pos2(rect.min.x + 80.0, rect.min.y + 70.0),
            vec2(rect.width() - 160.0, 100.0),
        );
        painter.rect_filled(yig_rect, 2.0, Color32::from_rgb(35, 55, 85));
        painter.rect_stroke(
            yig_rect,
            2.0,
            Stroke::new(1.5, Color32::from_rgb(0, 180, 255)),
            StrokeKind::Inside,
        );

        // IDT Finger Transducers on the Left and Right
        let idt_left = Rect::from_min_size(
            pos2(rect.min.x + 20.0, rect.min.y + 60.0),
            vec2(45.0, 120.0),
        );
        let idt_right = Rect::from_min_size(
            pos2(rect.max.x - 65.0, rect.min.y + 60.0),
            vec2(45.0, 120.0),
        );

        painter.rect_filled(idt_left, 2.0, Color32::from_rgb(220, 180, 40));
        painter.rect_filled(idt_right, 2.0, Color32::from_rgb(220, 180, 40));

        // Draw interdigital comb teeth
        for i in 0..6 {
            let y = idt_left.min.y + 10.0 + (i as f32) * 18.0;
            painter.line_segment(
                [pos2(idt_left.min.x + 4.0, y), pos2(idt_left.max.x - 4.0, y)],
                Stroke::new(2.0, Color32::from_rgb(40, 40, 40)),
            );
            painter.line_segment(
                [pos2(idt_right.min.x + 4.0, y), pos2(idt_right.max.x - 4.0, y)],
                Stroke::new(2.0, Color32::from_rgb(40, 40, 40)),
            );
        }

        // Draw forward propagating SAW wave packets (+x arrow)
        let wave_y = yig_rect.center().y;
        for i in 0..8 {
            let x = yig_rect.min.x + 20.0 + (i as f32) * 35.0;
            painter.circle_filled(pos2(x, wave_y), 6.0, Color32::from_rgb(0, 220, 120));
        }

        // Arrow indicating forward propagation
        painter.arrow(
            pos2(yig_rect.min.x + 40.0, wave_y - 25.0),
            vec2(120.0, 0.0),
            Stroke::new(3.0, Color32::from_rgb(50, 255, 120)),
        );

        // Text annotations
        painter.text(
            pos2(rect.min.x + 22.0, rect.min.y + 40.0),
            egui::Align2::LEFT_TOP,
            "Input Port 1 (IDT)",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(240, 240, 240),
        );
        painter.text(
            pos2(rect.max.x - 70.0, rect.min.y + 40.0),
            egui::Align2::RIGHT_TOP,
            "Output Port 2 (IDT)",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(240, 240, 240),
        );
        painter.text(
            pos2(yig_rect.center().x, yig_rect.max.y + 10.0),
            egui::Align2::CENTER_TOP,
            "YIG Magnetic Thin Film on LiNbO3 Piezoelectric Substrate",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(180, 220, 255),
        );
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("10-Point Physics & Quantum Verification Checklist").strong());
        ui.add_space(4.0);

        let report = &self.cached_audit;

        let items = [
            (
                report.pass_polariton_gap,
                "1. Avoided crossing polariton gap Delta_omega >= 40.0 MHz",
            ),
            (
                report.pass_wavevector_splitting,
                "2. Non-reciprocal wavevector splitting Delta_k >= 0.035 rad/um",
            ),
            (
                report.pass_insertion_loss,
                "3. Forward acoustic SAW insertion loss IL <= 0.60 dB",
            ),
            (
                report.pass_isolation_depth,
                "4. Reverse acoustic SAW isolation depth >= 35.0 dB",
            ),
            (
                report.pass_isolation_contrast,
                "5. Directional isolation contrast (ISO - IL) >= 34.0 dB",
            ),
            (
                report.pass_bandwidth,
                "6. 3-dB operational circulation bandwidth >= 30.0 MHz",
            ),
            (
                report.pass_circulator_unitarity,
                "7. 3-port cyclic circulator scattering matrix unitarity error < 0.05",
            ),
            (
                report.pass_quantum_added_noise,
                "8. Cryogenic added noise quanta n_add <= 0.55 at 20 mK",
            ),
            (
                report.pass_thermal_leakage_suppression,
                "9. Thermal photon back-action leakage from 4K stage < 1e-3 photons",
            ),
            (
                report.pass_readout_performance,
                "10. Qubit dispersive readout SNR >= 18.5 dB & fidelity >= 0.998",
            ),
        ];

        for (pass, label) in items {
            ui.horizontal(|ui| {
                if pass {
                    ui.label(RichText::new("[PASS]").strong().color(Color32::from_rgb(0, 230, 100)));
                } else {
                    ui.label(RichText::new("[FAIL]").strong().color(Color32::from_rgb(255, 60, 60)));
                }
                ui.label(label);
            });
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Audit Score: {} / 10", report.pass_count));
            if report.all_passed {
                ui.label(
                    RichText::new("ALL PHYSICS CRITERIA VERIFIED")
                        .strong()
                        .color(Color32::from_rgb(0, 230, 100)),
                );
            }
            ui.label(format!("(Solve Time: {:.1} us)", self.last_solve_time_us));
        });
    }

    /// Renders the modal dialog window if open.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Chiral Acoustomagnonic Isolator & Cryogenic Circulator")
            .open(&mut open)
            .default_size(vec2(780.0, 520.0))
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
