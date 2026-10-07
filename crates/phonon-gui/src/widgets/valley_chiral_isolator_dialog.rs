#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 417: Phonon Studio Topological Acoustic
//! Valley-Hall Chiral Edge Filter & Non-Reciprocal Microwave-Phonon Isolator.
//!
//! Visualizes Valley-Hall phononic crystal waveguides with zero backscattering around
//! sharp corners, synthetic gauge field spatio-temporal non-reciprocal isolation,
//! and cryogenic microwave-to-phonon quantum transduction in pure safe Rust.

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints, VLine};
use phonon_solver::valley_chiral_isolator::{
    ChiralIsolatorParams, ChiralSParameterPoint, TransducerParams, TransducerResponsePoint,
    ValleyChiralAuditReport, ValleyChiralIsolator, ValleyChiralParams, ValleyEdgeMode,
    ValleyEdgeParams,
};

/// Active tab within the Valley-Hall & Chiral Isolator Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValleyChiralTab {
    ValleyHallWaveguide,
    ChiralIsolator,
    MicrowaveTransducer,
    RealSpaceMetamaterial,
    AuditTelemetry,
}

impl ValleyChiralTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ValleyHallWaveguide => "1. Valley-Hall Edge Waveguide",
            Self::ChiralIsolator => "2. Non-Reciprocal Chiral Isolator",
            Self::MicrowaveTransducer => "3. Microwave-Phonon Transducer",
            Self::RealSpaceMetamaterial => "4. Real-Space Metamaterial",
            Self::AuditTelemetry => "5. Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 417.
pub struct ValleyChiralIsolatorDialog {
    pub is_open: bool,
    pub active_tab: ValleyChiralTab,

    // Valley-Hall Parameters
    pub lattice_constant_mm: f64,
    pub bare_frequency_ghz: f64,
    pub inversion_asymmetry_delta: f64,
    pub corner_angle_deg: f64,

    // Chiral Isolator Parameters
    pub modulation_freq_mhz: f64,
    pub modulation_depth: f64,
    pub synthetic_gauge_flux_rad: f64,
    pub waveguide_length_mm: f64,

    // Microwave Transducer Parameters
    pub piezo_coupling_keff2: f64,
    pub electrode_pairs: usize,
    pub aperture_um: f64,
    pub temperature_k: f64,

    // Real-space display toggle
    pub obstacle_defect_enabled: bool,

    // Solvers & Cached Results
    pub isolator_system: ValleyChiralIsolator,
    pub cached_dispersion: Vec<ValleyEdgeMode>,
    pub cached_spectrum: Vec<ChiralSParameterPoint>,
    pub cached_transducer_response: Vec<TransducerResponsePoint>,
    pub cached_realspace_field: Vec<Vec<f64>>,
    pub cached_audit: ValleyChiralAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ValleyChiralIsolatorDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ValleyChiralIsolatorDialog {
    /// Fast cold-boot constructor with pre-seeded baseline data (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let params = ValleyChiralParams::default();
        let system = ValleyChiralIsolator::new(params.clone());

        // Pre-seeded lightweight baseline points
        let cached_dispersion = system.valley_hall.compute_edge_dispersion(25);
        let cached_spectrum = system.isolator.compute_spectrum(25, 80.0);
        let cached_transducer_response = system.transducer.compute_spectrum(25, 80.0);
        let cached_realspace_field = system.valley_hall.compute_realspace_intensity_field(24, 16);
        let cached_audit = system.audit_valley_chiral_isolator();

        Self {
            is_open: false,
            active_tab: ValleyChiralTab::ValleyHallWaveguide,

            lattice_constant_mm: params.valley_hall.lattice_constant_mm,
            bare_frequency_ghz: params.valley_hall.bare_frequency_ghz,
            inversion_asymmetry_delta: params.valley_hall.inversion_asymmetry_delta,
            corner_angle_deg: params.valley_hall.corner_angle_deg,

            modulation_freq_mhz: params.isolator.modulation_freq_mhz,
            modulation_depth: params.isolator.modulation_depth,
            synthetic_gauge_flux_rad: params.isolator.synthetic_gauge_flux_rad,
            waveguide_length_mm: params.isolator.waveguide_length_mm,

            piezo_coupling_keff2: params.transducer.piezo_coupling_keff2,
            electrode_pairs: params.transducer.electrode_pairs,
            aperture_um: params.transducer.aperture_um,
            temperature_k: params.transducer.temperature_k,

            obstacle_defect_enabled: false,

            isolator_system: system,
            cached_dispersion,
            cached_spectrum,
            cached_transducer_response,
            cached_realspace_field,
            cached_audit,
            last_solve_time_us: 120.0,
        }
    }

    /// Recomputes all simulation models when parameters are modified.
    pub fn recompute(&mut self) {
        let t_start = std::time::Instant::now();

        let params = ValleyChiralParams {
            valley_hall: ValleyEdgeParams {
                lattice_constant_mm: self.lattice_constant_mm,
                bare_frequency_ghz: self.bare_frequency_ghz,
                inversion_asymmetry_delta: self.inversion_asymmetry_delta,
                corner_angle_deg: self.corner_angle_deg,
                ..Default::default()
            },
            isolator: ChiralIsolatorParams {
                center_freq_ghz: self.bare_frequency_ghz,
                modulation_freq_mhz: self.modulation_freq_mhz,
                modulation_depth: self.modulation_depth,
                synthetic_gauge_flux_rad: self.synthetic_gauge_flux_rad,
                waveguide_length_mm: self.waveguide_length_mm,
                ..Default::default()
            },
            transducer: TransducerParams {
                rf_freq_ghz: self.bare_frequency_ghz,
                piezo_coupling_keff2: self.piezo_coupling_keff2,
                electrode_pairs: self.electrode_pairs,
                aperture_um: self.aperture_um,
                temperature_k: self.temperature_k,
                ..Default::default()
            },
        };

        self.isolator_system.update_params(params);
        self.cached_dispersion = self.isolator_system.valley_hall.compute_edge_dispersion(31);
        self.cached_spectrum = self.isolator_system.isolator.compute_spectrum(41, 100.0);
        self.cached_transducer_response = self.isolator_system.transducer.compute_spectrum(41, 100.0);
        self.cached_realspace_field = self
            .isolator_system
            .valley_hall
            .compute_realspace_intensity_field(28, 18);
        self.cached_audit = self.isolator_system.audit_valley_chiral_isolator();

        self.last_solve_time_us = t_start.elapsed().as_micros() as f64;
    }

    /// Compatibility helper calling `show(ctx)`.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the modal dialog in the egui context.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new(RichText::new("Topological Acoustic Valley-Hall Chiral Edge Filter & Microwave Isolator").strong())
            .open(&mut is_open)
            .default_size(vec2(860.0, 600.0))
            .min_size(vec2(720.0, 500.0))
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the dialog contents and tab views.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        // Tab Bar
        ui.horizontal(|ui| {
            let tabs = [
                ValleyChiralTab::ValleyHallWaveguide,
                ValleyChiralTab::ChiralIsolator,
                ValleyChiralTab::MicrowaveTransducer,
                ValleyChiralTab::RealSpaceMetamaterial,
                ValleyChiralTab::AuditTelemetry,
            ];
            for t in tabs {
                let is_selected = self.active_tab == t;
                if ui.selectable_label(is_selected, t.label()).clicked() {
                    self.active_tab = t;
                }
            }
        });
        ui.separator();

        match self.active_tab {
            ValleyChiralTab::ValleyHallWaveguide => self.render_tab_valley_hall(ui),
            ValleyChiralTab::ChiralIsolator => self.render_tab_chiral_isolator(ui),
            ValleyChiralTab::MicrowaveTransducer => self.render_tab_transducer(ui),
            ValleyChiralTab::RealSpaceMetamaterial => self.render_tab_realspace(ui),
            ValleyChiralTab::AuditTelemetry => self.render_tab_audit(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_tab_valley_hall(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Topological Valley-Hall Phononic Crystal & Domain Wall Edge States").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Apply Valley Preset").clicked() {
                    self.inversion_asymmetry_delta = 0.15;
                    self.corner_angle_deg = 60.0;
                    self.recompute();
                }
            });
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.label("Asymmetry Delta (GHz):");
            changed |= ui.add(egui::Slider::new(&mut self.inversion_asymmetry_delta, 0.05..=0.35).step_by(0.01)).changed();

            ui.label("Corner Angle (deg):");
            changed |= ui.add(egui::Slider::new(&mut self.corner_angle_deg, 30.0..=150.0).step_by(5.0)).changed();

            if changed {
                self.recompute();
            }
        });

        ui.add_space(8.0);

        // Dispersion & Polarization Plots
        ui.columns(2, |cols| {
            cols[0].label(RichText::new("1D Valley-Hall Edge Dispersion f(k)").strong());
            let line_pts: PlotPoints = self
                .cached_dispersion
                .iter()
                .map(|m| [m.wavenumber_k, m.frequency_ghz])
                .collect();
            let edge_line = Line::new("Edge State f(k)", line_pts)
                .color(Color32::from_rgb(0, 220, 255))
                .width(2.5);

            let f0 = self.bare_frequency_ghz;
            let gap = self.isolator_system.valley_hall.bulk_bandgap_ghz();
            let top_gap = HLine::new("Gap Upper", f0 + gap * 0.5).color(Color32::from_rgb(180, 80, 80));
            let bot_gap = HLine::new("Gap Lower", f0 - gap * 0.5).color(Color32::from_rgb(180, 80, 80));

            Plot::new("plot_valley_dispersion")
                .height(240.0)
                .show(&mut cols[0], |plot_ui| {
                    plot_ui.line(edge_line);
                    plot_ui.hline(top_gap);
                    plot_ui.hline(bot_gap);
                });

            cols[1].label(RichText::new("Valley Polarization P_v(k)").strong());
            let pol_pts: PlotPoints = self
                .cached_dispersion
                .iter()
                .map(|m| [m.wavenumber_k, m.valley_polarization])
                .collect();
            let pol_line = Line::new("Valley Polarization", pol_pts)
                .color(Color32::from_rgb(255, 170, 0))
                .width(2.0);

            Plot::new("plot_valley_pol")
                .height(240.0)
                .show(&mut cols[1], |plot_ui| {
                    plot_ui.line(pol_line);
                    plot_ui.hline(HLine::new("Zero H", 0.0).color(Color32::GRAY));
                    plot_ui.vline(VLine::new("Zero V", 0.0).color(Color32::GRAY));
                });
        });
    }

    fn render_tab_chiral_isolator(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Non-Reciprocal Spatio-Temporal Synthetic Gauge Field Isolator").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Apply 30 dB Isolation Preset").clicked() {
                    self.modulation_depth = 0.28;
                    self.modulation_freq_mhz = 40.0;
                    self.recompute();
                }
            });
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.label("Modulation Depth:");
            changed |= ui.add(egui::Slider::new(&mut self.modulation_depth, 0.10..=0.50).step_by(0.02)).changed();

            ui.label("Pump Freq (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.modulation_freq_mhz, 10.0..=80.0).step_by(2.0)).changed();

            if changed {
                self.recompute();
            }
        });

        ui.add_space(8.0);

        // Transmission Plot: S21 (forward) vs S12 (reverse)
        let s21_pts: PlotPoints = self.cached_spectrum.iter().map(|p| [p.freq_ghz, p.s21_fwd_db]).collect();
        let s12_pts: PlotPoints = self.cached_spectrum.iter().map(|p| [p.freq_ghz, p.s12_rev_db]).collect();

        let s21_line = Line::new("Forward S21 (dB)", s21_pts)
            .color(Color32::from_rgb(0, 255, 120))
            .width(2.5);
        let s12_line = Line::new("Reverse S12 (dB)", s12_pts)
            .color(Color32::from_rgb(255, 60, 60))
            .width(2.5);

        Plot::new("plot_chiral_spectrum")
            .height(260.0)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(s21_line);
                plot_ui.line(s12_line);
                plot_ui.hline(HLine::new("Target Isolation", -30.0).color(Color32::from_rgb(200, 100, 100)));
            });
    }

    fn render_tab_transducer(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Cryogenic Microwave-to-Phonon Quantum Piezoelectric Transducer").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Apply 20 mK Dilution Preset").clicked() {
                    self.temperature_k = 0.015;
                    self.piezo_coupling_keff2 = 0.08;
                    self.recompute();
                }
            });
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.label("Coupling k_eff^2:");
            changed |= ui.add(egui::Slider::new(&mut self.piezo_coupling_keff2, 0.02..=0.15).step_by(0.01)).changed();

            ui.label("Finger Pairs N_p:");
            changed |= ui.add(egui::Slider::new(&mut self.electrode_pairs, 10..=60)).changed();

            ui.label("Temp (K):");
            changed |= ui.add(egui::Slider::new(&mut self.temperature_k, 0.010..=0.100).step_by(0.005)).changed();

            if changed {
                self.recompute();
            }
        });

        ui.add_space(8.0);

        let eta_pts: PlotPoints = self
            .cached_transducer_response
            .iter()
            .map(|p| [p.freq_ghz, p.conversion_efficiency * 100.0])
            .collect();
        let eta_line = Line::new("Conversion Efficiency eta(f) %", eta_pts)
            .color(Color32::from_rgb(0, 200, 255))
            .width(2.5);

        Plot::new("plot_transducer_eta")
            .height(260.0)
            .show(ui, |plot_ui| {
                plot_ui.line(eta_line);
                plot_ui.hline(HLine::new("40% Threshold", 40.0).color(Color32::from_rgb(255, 200, 0)));
            });
    }

    fn render_tab_realspace(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("2D Real-Space Acoustic Metamaterial & Sharp Corner Waveguide").strong());
            ui.checkbox(&mut self.obstacle_defect_enabled, "Simulate Defect Obstacle");
        });

        ui.add_space(8.0);

        // Heatmap rendering
        let (rect, _response) = ui.allocate_exact_size(vec2(ui.available_width(), 260.0), Sense::hover());
        let painter = ui.painter_at(rect);

        let ny = self.cached_realspace_field.len();
        let nx = if ny > 0 { self.cached_realspace_field[0].len() } else { 0 };

        if nx > 0 && ny > 0 {
            let dx = rect.width() / (nx as f32);
            let dy = rect.height() / (ny as f32);

            for y in 0..ny {
                for x in 0..nx {
                    let mut intensity = self.cached_realspace_field[y][x];
                    if self.obstacle_defect_enabled && x == nx / 2 && y == ny / 2 {
                        intensity = 0.05; // Obstacle insertion
                    }

                    let r = (255.0 * intensity.powf(0.6)) as u8;
                    let g = (200.0 * intensity.powf(1.2)) as u8;
                    let b = (255.0 * (1.0 - intensity).max(0.1)) as u8;

                    let c_rect = egui::Rect::from_min_size(
                        pos2(rect.min.x + (x as f32) * dx, rect.min.y + (y as f32) * dy),
                        vec2(dx + 0.5, dy + 0.5),
                    );
                    painter.rect_filled(c_rect, 1.0, Color32::from_rgb(r, g, b));
                }
            }
        }
    }

    fn render_tab_audit(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("10-Point Physics & RF Engineering Readiness Audit").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Re-Run Audit").clicked() {
                    self.recompute();
                }
            });
        });

        ui.add_space(8.0);

        let r = &self.cached_audit;
        ui.label(RichText::new(&r.summary_message).color(if r.all_passed {
            Color32::from_rgb(0, 255, 120)
        } else {
            Color32::from_rgb(255, 100, 100)
        }));

        ui.add_space(8.0);

        let checks = [
            ("1. Inversion-Symmetry Breaking Bulk Gap Opening (>= 0.20 GHz)", r.valley_bandgap_opened),
            ("2. Quantized Valley Chern Number Difference (|Delta C_v| = 1.0)", r.valley_chern_difference_quantized),
            ("3. Gapless Valley-Hall Edge State in Bulk Gap (Confinement >= 85%)", r.topological_edge_state_verified),
            ("4. Zero Backscattering Around Sharp Corners (Transmission >= 95%)", r.sharp_corner_zero_backscattering),
            ("5. Return Loss Suppression at Sharp Bends (S11 <= -25.0 dB)", r.return_loss_suppressed),
            ("6. Non-Reciprocal Low Forward Transmission Loss (S21 >= -0.80 dB)", r.nonreciprocal_fwd_transmission),
            ("7. Deep Reverse Acoustic Isolation (S12 <= -30.0 dB)", r.reverse_acoustic_isolation),
            ("8. Peak Non-Reciprocal Isolation Contrast (S21 - S12 >= 28.0 dB)", r.isolation_contrast_verified),
            ("9. Microwave-to-Phonon Quantum Conversion Efficiency (eta >= 40%)", r.microwave_phonon_efficiency_verified),
            ("10. Cryogenic Added Noise Approaching Quantum Limit (n_add <= 0.55)", r.cryogenic_quantum_noise_limit),
        ];

        for (desc, pass) in checks {
            ui.horizontal(|ui| {
                let badge = if pass { "[PASS]" } else { "[FAIL]" };
                let color = if pass { Color32::from_rgb(0, 230, 100) } else { Color32::from_rgb(255, 60, 60) };
                ui.label(RichText::new(badge).color(color).strong());
                ui.label(desc);
            });
        }
    }

    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let (t_corner, il, rl) = self.isolator_system.valley_hall.evaluate_corner_transmission();
        let s21 = self.isolator_system.isolator.evaluate_s21_fwd_db(self.bare_frequency_ghz);
        let s12 = self.isolator_system.isolator.evaluate_s12_rev_db(self.bare_frequency_ghz);
        let eta = self.isolator_system.transducer.peak_efficiency();
        let n_add = self.isolator_system.transducer.added_noise_quanta();

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("Corner T: {:.1}% (IL: {:.2} dB, RL: {:.1} dB)", t_corner * 100.0, il, rl)).monospace());
            ui.separator();
            ui.label(RichText::new(format!("S21: {:.2} dB | S12: {:.1} dB | Contrast: {:.1} dB", s21, s12, s21 - s12)).monospace());
            ui.separator();
            ui.label(RichText::new(format!("Transduction: {:.1}% | Noise: {:.2} quanta", eta * 100.0, n_add)).monospace());
            ui.separator();
            ui.label(RichText::new(format!("Solve Latency: {:.1} us", self.last_solve_time_us)).monospace());
        });
    }
}
